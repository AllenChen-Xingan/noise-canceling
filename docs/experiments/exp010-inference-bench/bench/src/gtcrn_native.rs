//! GTCRN 流式版手写实现（纯 Rust，无依赖）。
//!
//! 照 `resources/gtcrn/stream/gtcrn_stream.py` 的 `StreamGTCRN.forward` 逐层写，权重来自
//! `tools/dump_gtcrn.py` 导出的 state_dict（DNS3 权重，与 `gtcrn_simple.onnx` 同源）。
//! 构造期分配全部缓冲；`infer` 每帧零分配、无锁、无系统调用。
//! 布局：特征图一律 `[通道][频点]` 行主序；DPGRNN 内部用 `[频点][通道]`。

use std::path::Path;

pub(crate) const BINS: usize = 257;
pub(crate) const ERB_LOW: usize = 65; // erb_subband_1
pub(crate) const ERB_HIGH: usize = 64; // erb_subband_2
pub(crate) const F0: usize = ERB_LOW + ERB_HIGH; // 129
pub(crate) const F1: usize = 65;
pub(crate) const F2: usize = 33;
pub(crate) const BN_EPS: f32 = 1e-5;
pub(crate) const LN_EPS: f32 = 1e-8;

pub type Err = Box<dyn std::error::Error + Send + Sync>;

// ── 可选：矩阵乘交给 Accelerate（cblas_sgemm，Apple Silicon 上走 AMX / SME）──
// 后端名 native-blas。改的是 1×1 卷积、编码器卷积（im2col）、DPGRNN 的全连接；GRU、转置卷积不变。
// 用全局开关而不是逐层传参：实验期少改结构；开关只在构造时设一次，`infer` 里只读（Relaxed 原子读，无锁）。
// 求和顺序与手写循环不同，不再逐位相同，靠对拍判。
pub static BLAS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[inline]
pub(crate) fn blas() -> bool {
    BLAS.load(std::sync::atomic::Ordering::Relaxed)
}
#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn cblas_sgemm(
        order: i32,
        ta: i32,
        tb: i32,
        m: i32,
        n: i32,
        k: i32,
        alpha: f32,
        a: *const f32,
        lda: i32,
        b: *const f32,
        ldb: i32,
        beta: f32,
        c: *mut f32,
        ldc: i32,
    );
}
/// C[m×n] += A[m×k]·op(B)，行主序；`tb` 为真时 B 按 [n×k] 存（用其转置）。C 事先填好偏置。
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) fn sgemm_acc(
    m: usize,
    n: usize,
    k: usize,
    a: &[f32],
    lda: usize,
    b: &[f32],
    ldb: usize,
    tb: bool,
    c: &mut [f32],
    ldc: usize,
) {
    debug_assert!(a.len() >= (m - 1) * lda + k && c.len() >= (m - 1) * ldc + n);
    unsafe {
        cblas_sgemm(
            101,
            111,
            if tb { 112 } else { 111 },
            m as i32,
            n as i32,
            k as i32,
            1.0,
            a.as_ptr(),
            lda as i32,
            b.as_ptr(),
            ldb as i32,
            1.0,
            c.as_mut_ptr(),
            ldc as i32,
        )
    }
}

mod block;
mod cost;
mod dpgrnn;
mod kern;
mod ops;
#[cfg(test)]
mod tests;
pub mod v2;
pub mod v3;
mod weights;

use block::*;
use dpgrnn::*;
use kern::*;
use ops::*;
pub use cost::*;
pub use kern::Kind;
pub use ops::{FAST, exp_fast_pub, sig_fast_pub};
pub use weights::Weights;

// ───────────────────────── ERB ─────────────────────────

/// 稀疏的 ERB 矩阵：每行只存非零区间（跳过的都是精确 0，结果与稠密乘法相同）。
pub(crate) struct Sparse {
    pub(crate) rows: Vec<(usize, Vec<f32>)>, // (起始列, 非零段)
}

impl Sparse {
    pub(crate) fn from_dense(w: &[f32], rows: usize, cols: usize) -> Self {
        let mut out = Vec::with_capacity(rows);
        for r in 0..rows {
            let row = &w[r * cols..(r + 1) * cols];
            let first = row.iter().position(|v| *v != 0.0).unwrap_or(0);
            let last = row.iter().rposition(|v| *v != 0.0).map_or(first, |p| p + 1);
            out.push((first, row[first..last.max(first)].to_vec()));
        }
        Self { rows: out }
    }
    pub(crate) fn cost(&self) -> Cost {
        let nz: usize = self.rows.iter().map(|(_, seg)| seg.len()).sum();
        let longest = self.rows.iter().map(|(_, seg)| seg.len()).max().unwrap_or(1);
        Cost {
            macs: nz as u64,
            w_bytes: nz as u64 * F32B,
            io_bytes: (nz + self.rows.len()) as u64 * F32B, // 输入按非零段读
            chain_fma: clog2(longest) + 1,
            ..Cost::default()
        }
    }

    #[inline]
    pub(crate) fn run(&self, x: &[f32], y: &mut [f32]) {
        for (yv, (s, seg)) in y.iter_mut().zip(&self.rows) {
            let mut a = 0.0;
            for (w, v) in seg.iter().zip(&x[*s..]) {
                a += w * v;
            }
            *yv = a;
        }
    }
}

// ───────────────────────── 整网 ─────────────────────────

pub struct GtcrnNative {
    pub(crate) erb: Sparse,  // 64 × 192
    pub(crate) ierb: Sparse, // 192 × 64
    pub(crate) en0: ConvF,
    pub(crate) en1: ConvF,
    pub(crate) en_gt: [GtBlock; 3],
    pub(crate) dp: [Dp; 2],
    pub(crate) de_gt: [GtBlock; 3],
    pub(crate) de3: DeconvF,
    pub(crate) de4: DeconvF,
    pub(crate) feat0: Vec<f32>,  // [3][257]
    pub(crate) feat1: Vec<f32>,  // [3][129]
    pub(crate) feat2: Vec<f32>,  // [9][129]
    pub(crate) e0: Vec<f32>,     // [16][65]
    pub(crate) e: [Vec<f32>; 4], // e[0]=en_outs[1] [16][33]，e[1..4]=en_outs[2..5]
    pub(crate) t0: Vec<f32>,     // [16][33]
    pub(crate) t1: Vec<f32>,     // [16][33]
    pub(crate) d3: Vec<f32>,     // [16][65]
    pub(crate) d4: Vec<f32>,     // [2][129]
    pub(crate) m: Vec<f32>,      // [2][257]
    pub(crate) kind: Kind,
    pub(crate) prefetch: bool, // 每帧开头把 v2 用到的全部权重预取进 L2（研究 020 / D79）
}

impl GtcrnNative {
    pub fn load(prefix: &Path) -> Result<Self, Err> {
        Self::from_weights(&Weights::load(prefix)?)
    }

    pub(crate) fn from_weights(w: &Weights) -> Result<Self, Err> {
        let ew = w.get("erb.erb_fc.weight", ERB_HIGH * (BINS - ERB_LOW))?;
        let iw = w.get("erb.ierb_fc.weight", (BINS - ERB_LOW) * ERB_HIGH)?;
        let gt = |i: usize, dil: usize, de: bool| {
            let p = if de {
                format!("decoder.de_convs.{i}")
            } else {
                format!("encoder.en_convs.{i}")
            };
            GtBlock::load(&w, &p, dil, de)
        };
        Ok(Self {
            erb: Sparse::from_dense(&ew, ERB_HIGH, BINS - ERB_LOW),
            ierb: Sparse::from_dense(&iw, BINS - ERB_LOW, ERB_HIGH),
            en0: ConvF::load(&w, "encoder.en_convs.0", 9, C, 1, F0)?,
            en1: ConvF::load(&w, "encoder.en_convs.1", C, C, 2, F1)?,
            en_gt: [gt(2, 1, false)?, gt(3, 2, false)?, gt(4, 5, false)?],
            dp: [Dp::load(&w, "dpgrnn1")?, Dp::load(&w, "dpgrnn2")?],
            de_gt: [gt(0, 5, true)?, gt(1, 2, true)?, gt(2, 1, true)?],
            de3: DeconvF::load(&w, "decoder.de_convs.3", C, C, 2, F2, false)?,
            de4: DeconvF::load(&w, "decoder.de_convs.4", C, 2, 1, F1, true)?,
            feat0: vec![0.0; 3 * BINS],
            feat1: vec![0.0; 3 * F0],
            feat2: vec![0.0; 9 * F0],
            e0: vec![0.0; C * F1],
            e: [
                vec![0.0; C * F2],
                vec![0.0; C * F2],
                vec![0.0; C * F2],
                vec![0.0; C * F2],
            ],
            t0: vec![0.0; C * F2],
            t1: vec![0.0; C * F2],
            d3: vec![0.0; C * F1],
            d4: vec![0.0; 2 * F0],
            m: vec![0.0; 2 * BINS],
            kind: Kind::Scalar,
            prefetch: false,
        })
    }

    pub fn reset(&mut self) {
        for b in self.en_gt.iter_mut().chain(self.de_gt.iter_mut()) {
            b.reset();
        }
        for d in &mut self.dp {
            d.reset();
        }
    }

    /// 一帧：`re`/`im` 各 257 → 原地写成增强后的频谱。
    pub fn infer(&mut self, re: &mut [f32], im: &mut [f32]) {
        self.infer_probed(re, im, &mut NoProbe);
    }

    /// 同 `infer`，每段计算之后打一次点（研究 020 逐层计时与下界分析）。层的顺序与名字的正本在这里。
    pub fn infer_probed<P: Probe>(&mut self, re: &mut [f32], im: &mut [f32], p: &mut P) {
        match self.kind {
            Kind::Scalar => self.infer_impl::<Scalar, P>(re, im, p),
            Kind::V2 => self.infer_impl::<v2::V2<{ v2::NL_LIBM }>, P>(re, im, p),
            Kind::V2Fast => self.infer_impl::<v2::V2<{ v2::NL_DIV }>, P>(re, im, p),
            Kind::V3 => self.infer_impl::<v3::V3<false, { v2::NL_RCP }>, P>(re, im, p),
            Kind::V3i => self.infer_impl::<v3::V3<true, { v2::NL_RCP }>, P>(re, im, p),
            Kind::V4 => self.infer_impl::<v3::V3<false, { v2::NL_DIV }>, P>(re, im, p),
            Kind::V4i => self.infer_impl::<v3::V3<true, { v2::NL_DIV }>, P>(re, im, p),
        }
    }

    pub fn set_kind(&mut self, k: Kind) {
        self.kind = k;
    }

    pub fn set_prefetch(&mut self, on: bool) {
        self.prefetch = on;
    }

    /// v2 每帧会读到的全部参数数组（转置后的 GRU / 全连接权重、卷积权重、偏置、LN 仿射、ERB 非零段）
    pub(crate) fn for_each_v2_weight(&self, mut f: impl FnMut(&[f32])) {
        let gru = |g: &Gru, f: &mut dyn FnMut(&[f32])| {
            f(&g.wit);
            f(&g.wht);
            f(&g.bi);
            f(&g.bh);
        };
        for (_, seg) in self.erb.rows.iter().chain(&self.ierb.rows) {
            f(seg);
        }
        for c in [&self.en0, &self.en1] {
            f(&c.w);
            f(&c.b);
        }
        for b in self.en_gt.iter().chain(&self.de_gt) {
            for v in [&b.p1.w, &b.p1.b, &b.dw, &b.db, &b.p2.w, &b.p2.b, &b.fc.wt, &b.fc.b] {
                f(v);
            }
            gru(&b.gru, &mut f);
        }
        for d in &self.dp {
            for g in d.intra.iter().chain(&d.inter) {
                gru(g, &mut f);
            }
            for v in [
                &d.intra_fc.wt,
                &d.intra_fc.b,
                &d.inter_fc.wt,
                &d.inter_fc.b,
                &d.intra_lnw,
                &d.intra_lnb,
                &d.inter_lnw,
                &d.inter_lnb,
            ] {
                f(v);
            }
        }
        for d in [&self.de3, &self.de4] {
            f(&d.w);
            f(&d.b);
        }
    }

    /// 把权重逐个 64 字节发 L2 预取（不等待；数据到达前后续指令照常执行）
    #[inline(never)]
    pub(crate) fn prefetch_weights(&self) {
        self.for_each_v2_weight(|w| {
            let (p, n) = (w.as_ptr().cast::<u8>(), std::mem::size_of_val(w));
            let mut i = 0;
            while i < n {
                // SAFETY：只发预取提示，地址在切片内，不读写内存
                unsafe {
                    core::arch::asm!("prfm pldl2keep, [{0}]", in(reg) p.add(i), options(nostack, readonly, preserves_flags))
                };
                i += 64;
            }
        });
    }

    pub(crate) fn infer_impl<K: Kern, P: Probe>(&mut self, re: &mut [f32], im: &mut [f32], p: &mut P) {
        let bins = BINS as u64;
        if self.prefetch {
            self.prefetch_weights();
            p.mark("in", "prefetch", Cost::default);
        }
        // 特征：mag, re, im
        for k in 0..BINS {
            self.feat0[k] = (re[k] * re[k] + im[k] * im[k] + 1e-12).sqrt();
            self.feat0[BINS + k] = re[k];
            self.feat0[2 * BINS + k] = im[k];
        }
        p.mark("in", "feat", || Cost {
            macs: 2 * bins,
            other: 3 * bins, // sqrt + 2 次拷贝
            io_bytes: 5 * bins * F32B,
            chain_fma: 2 + 2, // 平方和 + sqrt（按 2 步计）
            ..Cost::default()
        });
        // ERB 压缩
        for c in 0..3 {
            let (src, dst) = (
                &self.feat0[c * BINS..(c + 1) * BINS],
                &mut self.feat1[c * F0..(c + 1) * F0],
            );
            dst[..ERB_LOW].copy_from_slice(&src[..ERB_LOW]);
            K::sparse(&self.erb, &src[ERB_LOW..], &mut dst[ERB_LOW..]);
        }
        sfe3(&self.feat1, 3, F0, &mut self.feat2);
        p.mark("in", "erb+sfe", || {
            scale(self.erb.cost(), 3, 1, 1)
                + Cost {
                    other: (3 * ERB_LOW + 9 * F0) as u64,
                    io_bytes: (3 * ERB_LOW * 2 + 3 * F0 + 9 * F0) as u64 * F32B,
                    ..Cost::default()
                }
        });
        // 编码
        K::conv(&mut self.en0, &self.feat2, &mut self.e0);
        p.mark("en0", "conv", || self.en0.cost());
        let [e1, e2, e3, e4] = &mut self.e;
        K::conv(&mut self.en1, &self.e0, e1);
        p.mark("en1", "conv", || self.en1.cost());
        self.en_gt[0].run::<K, P>(e1, e2, p, "en_gt0");
        self.en_gt[1].run::<K, P>(e2, e3, p, "en_gt1");
        self.en_gt[2].run::<K, P>(e3, e4, p, "en_gt2");
        // DPGRNN ×2
        self.t0.copy_from_slice(e4);
        self.dp[0].run::<K, P>(&mut self.t0, p, "dp0");
        self.dp[1].run::<K, P>(&mut self.t0, p, "dp1");
        // 解码（跳连相加）
        let skip = || Cost {
            other: (C * F2) as u64,
            io_bytes: 3 * (C * F2) as u64 * F32B,
            chain_fma: 1,
            ..Cost::default()
        };
        add_into(&mut self.t0, &self.e[3]);
        p.mark("de_gt0", "skip", skip);
        self.de_gt[0].run::<K, P>(&self.t0, &mut self.t1, p, "de_gt0");
        add_into(&mut self.t1, &self.e[2]);
        p.mark("de_gt1", "skip", skip);
        self.de_gt[1].run::<K, P>(&self.t1, &mut self.t0, p, "de_gt1");
        add_into(&mut self.t0, &self.e[1]);
        p.mark("de_gt2", "skip", skip);
        self.de_gt[2].run::<K, P>(&self.t0, &mut self.t1, p, "de_gt2");
        add_into(&mut self.t1, &self.e[0]);
        K::deconv(&mut self.de3, &self.t1, &mut self.d3);
        p.mark("de3", "skip+deconv", || skip() + self.de3.cost());
        add_into(&mut self.d3, &self.e0);
        K::deconv(&mut self.de4, &self.d3, &mut self.d4);
        p.mark("de4", "skip+deconv", || {
            let n = (C * F1) as u64;
            self.de4.cost()
                + Cost {
                    other: n,
                    io_bytes: 3 * n * F32B,
                    chain_fma: 1,
                    ..Cost::default()
                }
        });
        // ERB 还原
        for c in 0..2 {
            let (src, dst) = (
                &self.d4[c * F0..(c + 1) * F0],
                &mut self.m[c * BINS..(c + 1) * BINS],
            );
            dst[..ERB_LOW].copy_from_slice(&src[..ERB_LOW]);
            K::sparse(&self.ierb, &src[ERB_LOW..], &mut dst[ERB_LOW..]);
        }
        // 复数比值掩码
        for k in 0..BINS {
            let (m0, m1) = (self.m[k], self.m[BINS + k]);
            let (r, i) = (re[k], im[k]);
            re[k] = r * m0 - i * m1;
            im[k] = i * m0 + r * m1;
        }
        p.mark("out", "ierb+mask", || {
            scale(self.ierb.cost(), 2, 1, 1)
                + Cost {
                    macs: 4 * bins,
                    other: (2 * ERB_LOW) as u64,
                    io_bytes: (2 * ERB_LOW * 2 + 6 * BINS) as u64 * F32B,
                    chain_fma: 2,
                    ..Cost::default()
                }
        });
    }
}

pub(crate) fn add_into(x: &mut [f32], y: &[f32]) {
    for (a, b) in x.iter_mut().zip(y) {
        *a += b;
    }
}

#[cfg(test)]
mod tests_support {
    /// 确定性伪随机（不引依赖）
    pub fn rnd(n: usize, seed: u32) -> Vec<f32> {
        let mut s = seed.wrapping_mul(2_654_435_761).max(1);
        (0..n)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 17;
                s ^= s << 5;
                (s as f32 / u32::MAX as f32) * 2.0 - 1.0
            })
            .collect()
    }
}

