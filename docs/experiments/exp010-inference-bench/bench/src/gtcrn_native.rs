//! GTCRN 流式版手写实现（纯 Rust，无依赖）。
//!
//! 照 `resources/gtcrn/stream/gtcrn_stream.py` 的 `StreamGTCRN.forward` 逐层写，权重来自
//! `tools/dump_gtcrn.py` 导出的 state_dict（DNS3 权重，与 `gtcrn_simple.onnx` 同源）。
//! 构造期分配全部缓冲；`infer` 每帧零分配、无锁、无系统调用。
//! 布局：特征图一律 `[通道][频点]` 行主序；DPGRNN 内部用 `[频点][通道]`。

use std::collections::HashMap;
use std::path::Path;

const BINS: usize = 257;
const ERB_LOW: usize = 65; // erb_subband_1
const ERB_HIGH: usize = 64; // erb_subband_2
const F0: usize = ERB_LOW + ERB_HIGH; // 129
const F1: usize = 65;
const F2: usize = 33;
const BN_EPS: f32 = 1e-5;
const LN_EPS: f32 = 1e-8;

pub type Err = Box<dyn std::error::Error + Send + Sync>;

// ── 可选：矩阵乘交给 Accelerate（cblas_sgemm，Apple Silicon 上走 AMX / SME）──
// 后端名 native-blas。改的是 1×1 卷积、编码器卷积（im2col）、DPGRNN 的全连接；GRU、转置卷积不变。
// 用全局开关而不是逐层传参：实验期少改结构；开关只在构造时设一次，`infer` 里只读（Relaxed 原子读，无锁）。
// 求和顺序与手写循环不同，不再逐位相同，靠对拍判。
pub static BLAS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[inline]
fn blas() -> bool {
    BLAS.load(std::sync::atomic::Ordering::Relaxed)
}
#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn cblas_sgemm(
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
fn sgemm_acc(
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

/// 扁平权重表：`<prefix>.bin` + `<prefix>.txt`。
pub struct Weights {
    data: Vec<f32>,
    index: HashMap<String, (Vec<usize>, usize, usize)>,
}

impl Weights {
    pub fn load(prefix: &Path) -> Result<Self, Err> {
        let bin = std::fs::read(prefix.with_extension("bin"))?;
        let txt = std::fs::read_to_string(prefix.with_extension("txt"))?;
        let data: Vec<f32> = bin
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let mut index = HashMap::new();
        for line in txt.lines() {
            let p: Vec<&str> = line.split(' ').collect();
            if p.len() != 4 {
                return Err(format!("坏行：{line}").into());
            }
            let shape = p[1]
                .split(',')
                .map(str::parse)
                .collect::<Result<Vec<usize>, _>>()?;
            index.insert(p[0].to_string(), (shape, p[2].parse()?, p[3].parse()?));
        }
        if bin.len() % 4 != 0 {
            return Err(format!("权重 bin 长 {} 不是 4 的倍数", bin.len()).into());
        }
        let total: usize = index.values().map(|(_, _, n)| *n).sum();
        if total != data.len() {
            return Err(format!("txt 记的元素总数 {total} ≠ bin 里的 {}", data.len()).into());
        }
        Ok(Self { data, index })
    }

    fn get(&self, name: &str, numel: usize) -> Result<Vec<f32>, Err> {
        let (_, off, n) = self
            .index
            .get(name)
            .ok_or_else(|| format!("缺权重 {name}"))?;
        if *n != numel {
            return Err(format!("{name} 个数 {n} ≠ 期望 {numel}").into());
        }
        // txt 与 bin 对不上时返回错误，不 panic（自审 P2-5）
        self.data
            .get(*off..*off + *n)
            .map(<[f32]>::to_vec)
            .ok_or_else(|| format!("{name} 越界：偏移 {off} + {n} > {}", self.data.len()).into())
    }

    fn scalar(&self, name: &str) -> Result<f32, Err> {
        Ok(self.get(name, 1)?[0])
    }

    /// BN（推理态）折成逐通道 scale / shift。
    fn bn(&self, p: &str, c: usize) -> Result<(Vec<f32>, Vec<f32>), Err> {
        let g = self.get(&format!("{p}.weight"), c)?;
        let b = self.get(&format!("{p}.bias"), c)?;
        let m = self.get(&format!("{p}.running_mean"), c)?;
        let v = self.get(&format!("{p}.running_var"), c)?;
        let scale: Vec<f32> = g
            .iter()
            .zip(&v)
            .map(|(g, v)| g / (v + BN_EPS).sqrt())
            .collect();
        let shift = b
            .iter()
            .zip(&m)
            .zip(&scale)
            .map(|((b, m), s)| b - m * s)
            .collect();
        Ok((scale, shift))
    }
}

// ───────────────────── 计算量与打点（研究 020 下界分析） ─────────────────────
// 每层的计算量由各层自己的尺寸当场算出（正本 = 本文件的结构体字段），层的顺序与名字只在 `infer_probed` 里写一处。
// `infer` = `infer_probed(.., &mut NoProbe)`，空打点编译后消失，输出与原来逐位相同。

/// 一段计算的工作量。`chain_*` 是关键路径（最短依赖链）：归约按树形算（⌈log2 K⌉ + 1 步），
/// 循环网络沿序列串行相乘。下界分析拿它乘本机实测的单步延迟。
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Cost {
    pub macs: u64,      // 乘加次数
    pub nonlin: u64,    // exp 级非线性（sigmoid / tanh）次数
    pub other: u64,     // 其它逐元素运算（PReLU、加法、sqrt、拷贝的元素数）
    pub w_bytes: u64,   // 读权重与偏置的字节数
    pub io_bytes: u64,  // 读入 + 写出特征的字节数
    pub chain_fma: u64, // 关键路径上依赖的乘加 / 加法步数
    pub chain_nl: u64,  // 关键路径上依赖的非线性次数
}

impl std::ops::Add for Cost {
    type Output = Cost;
    fn add(self, o: Cost) -> Cost {
        Cost {
            macs: self.macs + o.macs,
            nonlin: self.nonlin + o.nonlin,
            other: self.other + o.other,
            w_bytes: self.w_bytes + o.w_bytes,
            io_bytes: self.io_bytes + o.io_bytes,
            chain_fma: self.chain_fma + o.chain_fma,
            chain_nl: self.chain_nl + o.chain_nl,
        }
    }
}

/// ⌈log2 n⌉（n ≥ 1）：n 个数树形求和的层数
pub fn clog2(n: usize) -> u64 {
    if n <= 1 {
        0
    } else {
        (usize::BITS - (n - 1).leading_zeros()) as u64
    }
}

const F32B: u64 = 4;

/// 打点：每段计算之后调一次。`cost` 只在需要时才求值（计时打点不调它）。
pub trait Probe {
    #[inline(always)]
    fn mark(&mut self, _blk: &'static str, _op: &'static str, _cost: impl FnOnce() -> Cost) {}
}
/// 不打点（正式推理用）
pub struct NoProbe;
impl Probe for NoProbe {}

// ───────────────────────── 积木 ─────────────────────────

// ── 可选：内联多项式 exp（后端名 native-fast）──
// 系统 libm 的 expf / tanhf 占手写版约 23% 时间（profile_native.txt），且是外部调用、挡住向量化。
// 这里用 2^n · p(r) 的经典做法：n = round(x·log2e)，r = x − n·ln2（ln2 拆两段减小舍入），p 为 6 阶泰勒/极小化多项式，
// 在 |r| ≤ ln2/2 上相对误差约 2e-7（推断，同 Cephes expf 的阶数）。结果不再与 libm 逐位相同，靠对拍判。
pub static FAST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[inline]
fn fast() -> bool {
    FAST.load(std::sync::atomic::Ordering::Relaxed)
}
#[inline(always)]
fn exp_fast(x: f32) -> f32 {
    let x = x.clamp(-87.0, 88.0);
    let n = (x * std::f32::consts::LOG2_E).round();
    let r = x - n * 0.693_359_4 - n * -2.121_944_4e-4; // ln2 = 0.693359375 − 2.12194440e-4
    // Cephes expf 系数
    let p = 1.987_569_1e-4_f32;
    let p = p * r + 1.398_199_9e-3;
    let p = p * r + 8.333_452e-3;
    let p = p * r + 4.166_579_6e-2;
    let p = p * r + 1.666_666_5e-1;
    let p = p * r + 5.000_000_1e-1;
    let y = p * r * r + r + 1.0;
    f32::from_bits(((n as i32 + 127) as u32) << 23) * y
}
#[inline(always)]
fn tanh_fast(x: f32) -> f32 {
    // tanh(x) = 1 − 2 / (e^{2x} + 1)；x 很小时有相消，绝对误差约 1e-7 级（相对 tanh 的取值 ±1 可忽略）
    1.0 - 2.0 / (exp_fast(2.0 * x) + 1.0)
}

#[inline(always)]
fn sig_fast(x: f32) -> f32 {
    1.0 / (1.0 + exp_fast(-x))
}
#[inline]
fn sigmoid(x: f32) -> f32 {
    if fast() {
        1.0 / (1.0 + exp_fast(-x))
    } else {
        1.0 / (1.0 + (-x).exp())
    }
}
#[inline]
fn tanh_(x: f32) -> f32 {
    if fast() { tanh_fast(x) } else { x.tanh() }
}

#[inline]
fn prelu(x: f32, a: f32) -> f32 {
    if x >= 0.0 { x } else { a * x }
}

/// 1×1 卷积（逐点），权重 `[out][in]`，已折 BN（可选）。
struct Point {
    w: Vec<f32>,
    b: Vec<f32>,
    cin: usize,
    cout: usize,
}

impl Point {
    /// `transposed`：ConvTranspose 的权重是 `[in][out]`，转成 `[out][in]`。
    fn load(
        w: &Weights,
        p: &str,
        cin: usize,
        cout: usize,
        transposed: bool,
        bn: Option<&str>,
    ) -> Result<Self, Err> {
        let raw = w.get(&format!("{p}.weight"), cin * cout)?;
        let mut ww = vec![0.0; cin * cout];
        for o in 0..cout {
            for i in 0..cin {
                ww[o * cin + i] = if transposed {
                    raw[i * cout + o]
                } else {
                    raw[o * cin + i]
                };
            }
        }
        let mut b = w.get(&format!("{p}.bias"), cout)?;
        if let Some(bn) = bn {
            let (s, t) = w.bn(bn, cout)?;
            for o in 0..cout {
                for i in 0..cin {
                    ww[o * cin + i] *= s[o];
                }
                b[o] = b[o] * s[o] + t[o];
            }
        }
        Ok(Self {
            w: ww,
            b,
            cin,
            cout,
        })
    }

    fn cost(&self, f: usize) -> Cost {
        let (ci, co, f) = (self.cin as u64, self.cout as u64, f as u64);
        Cost {
            macs: co * ci * f,
            w_bytes: (co * ci + co) * F32B,
            io_bytes: (ci + co) * f * F32B,
            chain_fma: clog2(self.cin) + 1,
            ..Cost::default()
        }
    }

    fn run(&self, x: &[f32], y: &mut [f32], f: usize) {
        if blas() {
            for o in 0..self.cout {
                y[o * f..(o + 1) * f].fill(self.b[o]);
            }
            sgemm_acc(self.cout, f, self.cin, &self.w, self.cin, x, f, false, y, f);
            return;
        }
        for o in 0..self.cout {
            let yo = &mut y[o * f..(o + 1) * f];
            yo.fill(self.b[o]);
            for i in 0..self.cin {
                let wi = self.w[o * self.cin + i];
                for (a, &xv) in yo.iter_mut().zip(&x[i * f..(i + 1) * f]) {
                    *a += wi * xv;
                }
            }
        }
    }
}

/// 沿频率轴的分组一维卷积（编码器 ConvBlock：k=5，stride 2，pad 2）。权重 `[out][in/g][k]`，已折 BN。
struct ConvF {
    w: Vec<f32>,
    b: Vec<f32>,
    cin: usize,
    cout: usize,
    groups: usize,
    fin: usize,
    fout: usize,
    alpha: f32,
    col: Vec<f32>, // im2col 缓冲 [cig*K][fout]（只 native-blas 用）
}

const K: usize = 5;

impl ConvF {
    fn load(
        w: &Weights,
        p: &str,
        cin: usize,
        cout: usize,
        groups: usize,
        fin: usize,
    ) -> Result<Self, Err> {
        let cig = cin / groups;
        let mut ww = w.get(&format!("{p}.conv.weight"), cout * cig * K)?;
        let mut b = w.get(&format!("{p}.conv.bias"), cout)?;
        let (s, t) = w.bn(&format!("{p}.bn"), cout)?;
        for o in 0..cout {
            for v in &mut ww[o * cig * K..(o + 1) * cig * K] {
                *v *= s[o];
            }
            b[o] = b[o] * s[o] + t[o];
        }
        let fout = (fin + 2 * 2 - K) / 2 + 1;
        Ok(Self {
            w: ww,
            b,
            cin,
            cout,
            groups,
            fin,
            fout,
            alpha: w.scalar(&format!("{p}.act.weight"))?,
            col: vec![0.0; cig * K * fout],
        })
    }

    /// 只数有效抽头（越界补 0 的不算），与 `run` 的循环区间相同
    fn cost(&self) -> Cost {
        let cig = self.cin / self.groups;
        let taps: usize = (0..K)
            .map(|t| {
                let jlo = (2usize.saturating_sub(t)).div_ceil(2);
                let jhi = (self.fin + 2 - t).div_ceil(2).min(self.fout);
                jhi.saturating_sub(jlo)
            })
            .sum();
        let (ci, co) = (self.cin as u64, self.cout as u64);
        Cost {
            macs: co * (cig * taps) as u64,
            other: co * self.fout as u64,
            w_bytes: (co * (cig * K) as u64 + co) * F32B,
            io_bytes: (ci * self.fin as u64 + co * self.fout as u64) * F32B,
            chain_fma: clog2(cig * K) + 2, // 归约 + PReLU
            ..Cost::default()
        }
    }

    fn run(&mut self, x: &[f32], y: &mut [f32]) {
        let (cig, cog) = (self.cin / self.groups, self.cout / self.groups);
        if blas() {
            let fo = self.fout;
            for g in 0..self.groups {
                // col[(ci*K + t)][j] = x[g*cig+ci][2j + t − 2]，越界补 0
                for ci in 0..cig {
                    let xi = &x[(g * cig + ci) * self.fin..(g * cig + ci + 1) * self.fin];
                    for t in 0..K {
                        let row = &mut self.col[(ci * K + t) * fo..(ci * K + t + 1) * fo];
                        row.fill(0.0);
                        let jlo = (2usize.saturating_sub(t)).div_ceil(2);
                        let jhi = (self.fin + 2 - t).div_ceil(2).min(fo);
                        for j in jlo..jhi {
                            row[j] = xi[2 * j + t - 2];
                        }
                    }
                }
                for o in g * cog..(g + 1) * cog {
                    y[o * fo..(o + 1) * fo].fill(self.b[o]);
                }
                let wg = &self.w[g * cog * cig * K..];
                sgemm_acc(
                    cog,
                    fo,
                    cig * K,
                    wg,
                    cig * K,
                    &self.col,
                    fo,
                    false,
                    &mut y[g * cog * fo..],
                    fo,
                );
            }
            for v in y[..self.cout * fo].iter_mut() {
                *v = prelu(*v, self.alpha);
            }
            return;
        }
        for o in 0..self.cout {
            let g = o / cog;
            let yo = &mut y[o * self.fout..(o + 1) * self.fout];
            yo.fill(self.b[o]);
            for ci in 0..cig {
                let xi = &x[(g * cig + ci) * self.fin..(g * cig + ci + 1) * self.fin];
                for t in 0..K {
                    let wv = self.w[(o * cig + ci) * K + t];
                    // 有效输出区间：0 ≤ 2j + t − 2 < fin。先算好区间，内层不再逐元素判断（加法顺序不变，结果逐位相同）
                    let jlo = (2usize.saturating_sub(t)).div_ceil(2);
                    let jhi = (self.fin + 2 - t).div_ceil(2).min(self.fout);
                    for j in jlo..jhi {
                        yo[j] += wv * xi[2 * j + t - 2];
                    }
                }
            }
            for v in yo.iter_mut() {
                *v = prelu(*v, self.alpha);
            }
        }
    }
}

/// 沿频率轴的分组一维转置卷积（解码器 ConvBlock：k=5，stride 2，pad 2）。权重 `[in][out/g][k]`，已折 BN。
struct DeconvF {
    w: Vec<f32>,
    b: Vec<f32>,
    cin: usize,
    cout: usize,
    groups: usize,
    fin: usize,
    fout: usize,
    alpha: f32,
    tanh: bool,
}

impl DeconvF {
    fn load(
        w: &Weights,
        p: &str,
        cin: usize,
        cout: usize,
        groups: usize,
        fin: usize,
        tanh: bool,
    ) -> Result<Self, Err> {
        let cog = cout / groups;
        let mut ww = w.get(&format!("{p}.conv.weight"), cin * cog * K)?;
        let mut b = w.get(&format!("{p}.conv.bias"), cout)?;
        let (s, t) = w.bn(&format!("{p}.bn"), cout)?;
        let cig = cin / groups;
        for ic in 0..cin {
            let g = ic / cig;
            for ol in 0..cog {
                for k in 0..K {
                    ww[(ic * cog + ol) * K + k] *= s[g * cog + ol];
                }
            }
        }
        for o in 0..cout {
            b[o] = b[o] * s[o] + t[o];
        }
        let fout = (fin - 1) * 2 - 2 * 2 + K;
        let alpha = if tanh {
            0.0
        } else {
            w.scalar(&format!("{p}.act.weight"))?
        };
        Ok(Self {
            w: ww,
            b,
            cin,
            cout,
            groups,
            fin,
            fout,
            alpha,
            tanh,
        })
    }

    fn run(&self, x: &[f32], y: &mut [f32]) {
        let (cig, cog) = (self.cin / self.groups, self.cout / self.groups);
        for o in 0..self.cout {
            y[o * self.fout..(o + 1) * self.fout].fill(self.b[o]);
        }
        for ic in 0..self.cin {
            let g = ic / cig;
            let xi = &x[ic * self.fin..(ic + 1) * self.fin];
            for ol in 0..cog {
                let o = g * cog + ol;
                let yo = &mut y[o * self.fout..(o + 1) * self.fout];
                for k in 0..K {
                    let wv = self.w[(ic * cog + ol) * K + k];
                    // 有效输入区间：0 ≤ 2i + k − 2 < fout（同上，只去掉分支）
                    let ilo = (2usize.saturating_sub(k)).div_ceil(2);
                    let ihi = (self.fout + 2 - k).div_ceil(2).min(self.fin);
                    for i in ilo..ihi {
                        yo[2 * i + k - 2] += wv * xi[i];
                    }
                }
            }
        }
        for v in y[..self.cout * self.fout].iter_mut() {
            *v = if self.tanh {
                tanh_(*v)
            } else {
                prelu(*v, self.alpha)
            };
        }
    }
}

/// PyTorch GRU 单步（门序 r, z, n）。
struct Gru {
    wi: Vec<f32>, // [3H][I]
    wh: Vec<f32>, // [3H][H]
    bi: Vec<f32>,
    bh: Vec<f32>,
    i: usize,
    h: usize,
}

const MAX_G: usize = 48;

impl Gru {
    fn load(w: &Weights, p: &str, sfx: &str, i: usize, h: usize) -> Result<Self, Err> {
        if 3 * h > MAX_G {
            return Err(format!("{p}: 3H = {} 超过栈缓冲 MAX_G = {MAX_G}", 3 * h).into());
        }
        Ok(Self {
            wi: w.get(&format!("{p}.weight_ih_l0{sfx}"), 3 * h * i)?,
            wh: w.get(&format!("{p}.weight_hh_l0{sfx}"), 3 * h * h)?,
            bi: w.get(&format!("{p}.bias_ih_l0{sfx}"), 3 * h)?,
            bh: w.get(&format!("{p}.bias_hh_l0{sfx}"), 3 * h)?,
            i,
            h,
        })
    }

    /// `x` 长 I，`h` 长 H，原地更新 `h`。
    #[inline]
    fn step(&self, x: &[f32], h: &mut [f32]) {
        let (ni, nh) = (self.i, self.h);
        let mut gi = [0.0f32; MAX_G];
        let mut gh = [0.0f32; MAX_G];
        for g in 0..3 * nh {
            let mut a = self.bi[g];
            for (w, v) in self.wi[g * ni..(g + 1) * ni].iter().zip(x) {
                a += w * v;
            }
            gi[g] = a;
            let mut b = self.bh[g];
            for (w, v) in self.wh[g * nh..(g + 1) * nh].iter().zip(h.iter()) {
                b += w * v;
            }
            gh[g] = b;
        }
        // 开关在循环外读一次，循环体里没有分支，便于向量化
        if fast() {
            for k in 0..nh {
                let r = sig_fast(gi[k] + gh[k]);
                let z = sig_fast(gi[nh + k] + gh[nh + k]);
                let n = tanh_fast(gi[2 * nh + k] + r * gh[2 * nh + k]);
                h[k] = (1.0 - z) * n + z * h[k];
            }
        } else {
            for k in 0..nh {
                let r = sigmoid(gi[k] + gh[k]);
                let z = sigmoid(gi[nh + k] + gh[nh + k]);
                let n = tanh_(gi[2 * nh + k] + r * gh[2 * nh + k]);
                h[k] = (1.0 - z) * n + z * h[k];
            }
        }
    }
}

/// 全连接，权重 `[out][in]`。
struct Linear {
    w: Vec<f32>,
    b: Vec<f32>,
    i: usize,
    o: usize,
}

impl Linear {
    fn load(w: &Weights, p: &str, i: usize, o: usize) -> Result<Self, Err> {
        Ok(Self {
            w: w.get(&format!("{p}.weight"), o * i)?,
            b: w.get(&format!("{p}.bias"), o)?,
            i,
            o,
        })
    }
    #[inline]
    fn run(&self, x: &[f32], y: &mut [f32]) {
        for k in 0..self.o {
            let mut a = self.b[k];
            for (w, v) in self.w[k * self.i..(k + 1) * self.i].iter().zip(x) {
                a += w * v;
            }
            y[k] = a;
        }
    }
}

// ───────────────────── GT 卷积块（编码 / 解码共用） ─────────────────────

const C: usize = 16;
const CH: usize = 8; // C/2

struct GtBlock {
    p1: Point,
    a1: f32,
    dw: Vec<f32>, // [16][3][3]，已折 BN（解码器的权重已由 convert_to_stream 翻转成普通卷积）
    db: Vec<f32>,
    a2: f32,
    p2: Point,
    gru: Gru,
    fc: Linear,
    dil: usize,
    // 缓存：环形，`2*dil` 帧，每帧 [16][33]
    ring: Vec<f32>,
    head: usize,
    hstate: [f32; 16],
    // 临时
    sfe: Vec<f32>, // [24][33]
    h1: Vec<f32>,  // [16][33]
    h2: Vec<f32>,  // [16][33]
    h3: Vec<f32>,  // [8][33]
}

impl GtBlock {
    fn load(w: &Weights, p: &str, dil: usize, deconv: bool) -> Result<Self, Err> {
        let dname = if deconv { "ConvTranspose2d" } else { "Conv2d" };
        let mut dw = w.get(&format!("{p}.depth_conv.{dname}.weight"), C * 9)?;
        let mut db = w.get(&format!("{p}.depth_conv.{dname}.bias"), C)?;
        let (s, t) = w.bn(&format!("{p}.depth_bn"), C)?;
        for c in 0..C {
            for v in &mut dw[c * 9..(c + 1) * 9] {
                *v *= s[c];
            }
            db[c] = db[c] * s[c] + t[c];
        }
        Ok(Self {
            p1: Point::load(
                w,
                &format!("{p}.point_conv1"),
                3 * CH,
                C,
                deconv,
                Some(&format!("{p}.point_bn1")),
            )?,
            a1: w.scalar(&format!("{p}.point_act.weight"))?,
            dw,
            db,
            a2: w.scalar(&format!("{p}.depth_act.weight"))?,
            p2: Point::load(
                w,
                &format!("{p}.point_conv2"),
                C,
                CH,
                deconv,
                Some(&format!("{p}.point_bn2")),
            )?,
            gru: Gru::load(w, &format!("{p}.tra.att_gru"), "", CH, 2 * CH)?,
            fc: Linear::load(w, &format!("{p}.tra.att_fc"), 2 * CH, CH)?,
            dil,
            ring: vec![0.0; 2 * dil * C * F2],
            head: 0,
            hstate: [0.0; 16],
            sfe: vec![0.0; 3 * CH * F2],
            h1: vec![0.0; C * F2],
            h2: vec![0.0; C * F2],
            h3: vec![0.0; CH * F2],
        })
    }

    fn reset(&mut self) {
        self.ring.fill(0.0);
        self.head = 0;
        self.hstate = [0.0; 16];
    }

    /// `x`：[16][33] → `y`：[16][33]
    fn run(&mut self, x: &[f32], y: &mut [f32]) {
        let f = F2;
        // x1 = 前 8 通道 → SFE → 24 通道
        sfe3(&x[..CH * f], CH, f, &mut self.sfe);
        self.p1.run(&self.sfe, &mut self.h1, f);
        for v in self.h1.iter_mut() {
            *v = prelu(*v, self.a1);
        }
        // 流式深度卷积：inp 的时间下标 0..=2d，2d 是当前帧；kt 取 inp[kt*d]
        let len = 2 * self.dil;
        let frame = C * f;
        for c in 0..C {
            let out = &mut self.h2[c * f..(c + 1) * f];
            out.fill(self.db[c]);
            for kt in 0..3 {
                let ti = kt * self.dil;
                let src: &[f32] = if ti == len {
                    &self.h1[c * f..(c + 1) * f]
                } else {
                    let slot = (self.head + ti) % len;
                    &self.ring[slot * frame + c * f..slot * frame + (c + 1) * f]
                };
                let w = &self.dw[c * 9 + kt * 3..c * 9 + kt * 3 + 3];
                // 频率 pad 1
                for j in 0..f {
                    let mut a = w[1] * src[j];
                    if j > 0 {
                        a += w[0] * src[j - 1];
                    }
                    if j + 1 < f {
                        a += w[2] * src[j + 1];
                    }
                    out[j] += a;
                }
            }
        }
        // 当前帧写进最旧的槽位
        let slot = self.head;
        self.ring[slot * frame..(slot + 1) * frame].copy_from_slice(&self.h1);
        self.head = (self.head + 1) % len;
        for v in self.h2.iter_mut() {
            *v = prelu(*v, self.a2);
        }
        self.p2.run(&self.h2, &mut self.h3, f);
        // TRA
        let mut zt = [0.0f32; CH];
        for c in 0..CH {
            zt[c] = self.h3[c * f..(c + 1) * f]
                .iter()
                .map(|v| v * v)
                .sum::<f32>()
                / f as f32;
        }
        self.gru.step(&zt, &mut self.hstate);
        let mut at = [0.0f32; CH];
        self.fc.run(&self.hstate, &mut at);
        // shuffle：y[2c] = h1[c]*at[c]，y[2c+1] = x2[c]
        for c in 0..CH {
            let a = sigmoid(at[c]);
            for j in 0..f {
                y[(2 * c) * f + j] = self.h3[c * f + j] * a;
            }
            y[(2 * c + 1) * f..(2 * c + 2) * f].copy_from_slice(&x[(CH + c) * f..(CH + c + 1) * f]);
        }
    }
}

/// SFE（unfold 1×3，pad 1）：[c][f] → [c*3 + j][f] = x[c][f + j − 1]
fn sfe3(x: &[f32], c: usize, f: usize, y: &mut [f32]) {
    for ci in 0..c {
        let xi = &x[ci * f..(ci + 1) * f];
        for j in 0..3 {
            let yo = &mut y[(ci * 3 + j) * f..(ci * 3 + j + 1) * f];
            for k in 0..f {
                let idx = k as isize + j as isize - 1;
                yo[k] = if idx >= 0 && (idx as usize) < f {
                    xi[idx as usize]
                } else {
                    0.0
                };
            }
        }
    }
}

// ───────────────────────── DPGRNN ─────────────────────────

struct Dp {
    intra: [Gru; 4], // rnn1 fwd, rnn1 bwd, rnn2 fwd, rnn2 bwd（H=4）
    intra_fc: Linear,
    intra_lnw: Vec<f32>,
    intra_lnb: Vec<f32>,
    inter: [Gru; 2], // H=8
    inter_fc: Linear,
    inter_lnw: Vec<f32>,
    inter_lnb: Vec<f32>,
    cache: Vec<f32>, // [33][16]
    xt: Vec<f32>,    // [33][16]
    y: Vec<f32>,
    z: Vec<f32>,
}

impl Dp {
    fn load(w: &Weights, p: &str) -> Result<Self, Err> {
        let g = |n: &str, s: &str, h: usize| Gru::load(w, &format!("{p}.{n}"), s, CH, h);
        Ok(Self {
            intra: [
                g("intra_rnn.rnn1", "", 4)?,
                g("intra_rnn.rnn1", "_reverse", 4)?,
                g("intra_rnn.rnn2", "", 4)?,
                g("intra_rnn.rnn2", "_reverse", 4)?,
            ],
            intra_fc: Linear::load(w, &format!("{p}.intra_fc"), C, C)?,
            intra_lnw: w.get(&format!("{p}.intra_ln.weight"), F2 * C)?,
            intra_lnb: w.get(&format!("{p}.intra_ln.bias"), F2 * C)?,
            inter: [g("inter_rnn.rnn1", "", 8)?, g("inter_rnn.rnn2", "", 8)?],
            inter_fc: Linear::load(w, &format!("{p}.inter_fc"), C, C)?,
            inter_lnw: w.get(&format!("{p}.inter_ln.weight"), F2 * C)?,
            inter_lnb: w.get(&format!("{p}.inter_ln.bias"), F2 * C)?,
            cache: vec![0.0; F2 * C],
            xt: vec![0.0; F2 * C],
            y: vec![0.0; F2 * C],
            z: vec![0.0; F2 * C],
        })
    }

    fn reset(&mut self) {
        self.cache.fill(0.0);
    }

    /// 原地：`feat` [16][33]
    fn run(&mut self, feat: &mut [f32]) {
        // 转成 [F][C]
        for c in 0..C {
            for f in 0..F2 {
                self.xt[f * C + c] = feat[c * F2 + f];
            }
        }
        // intra：沿频率的双向分组 GRU，初值 0
        for grp in 0..2 {
            let (fw, bw) = (&self.intra[2 * grp], &self.intra[2 * grp + 1]);
            let mut h = [0.0f32; 4];
            for f in 0..F2 {
                fw.step(&self.xt[f * C + grp * CH..f * C + grp * CH + CH], &mut h);
                self.y[f * C + grp * CH..f * C + grp * CH + 4].copy_from_slice(&h);
            }
            let mut h = [0.0f32; 4];
            for f in (0..F2).rev() {
                bw.step(&self.xt[f * C + grp * CH..f * C + grp * CH + CH], &mut h);
                self.y[f * C + grp * CH + 4..f * C + grp * CH + 8].copy_from_slice(&h);
            }
        }
        if blas() {
            fc_batch(&self.intra_fc, &self.y, &mut self.z);
        } else {
            for f in 0..F2 {
                self.intra_fc
                    .run(&self.y[f * C..(f + 1) * C], &mut self.z[f * C..(f + 1) * C]);
            }
        }
        layer_norm_add(&mut self.xt, &self.z, &self.intra_lnw, &self.intra_lnb);
        // inter：沿时间（单步）的分组 GRU，状态逐频点缓存
        for f in 0..F2 {
            for grp in 0..2 {
                let r = f * C + grp * CH..f * C + grp * CH + CH;
                self.inter[grp].step(&self.xt[r.clone()], &mut self.cache[r.clone()]);
            }
            if !blas() {
                self.inter_fc.run(
                    &self.cache[f * C..(f + 1) * C],
                    &mut self.z[f * C..(f + 1) * C],
                );
            }
        }
        if blas() {
            // 第 f 行的全连接只依赖 cache[f]，它在上面第 f 步之后就不再变，所以可以挪到循环外一次算完
            fc_batch(&self.inter_fc, &self.cache, &mut self.z);
        }
        layer_norm_add(&mut self.xt, &self.z, &self.inter_lnw, &self.inter_lnb);
        for c in 0..C {
            for f in 0..F2 {
                feat[c * F2 + f] = self.xt[f * C + c];
            }
        }
    }
}

/// Z[33×16] = X[33×16]·Wᵀ + b（逐频点全连接一次做完）
fn fc_batch(l: &Linear, x: &[f32], z: &mut [f32]) {
    for f in 0..F2 {
        z[f * l.o..(f + 1) * l.o].copy_from_slice(&l.b);
    }
    sgemm_acc(F2, l.o, l.i, x, l.i, &l.w, l.i, true, z, l.o);
}

/// x += LN(z)，LN 在整个 (33,16) 上归一化，带逐元素仿射。
fn layer_norm_add(x: &mut [f32], z: &[f32], w: &[f32], b: &[f32]) {
    let n = z.len() as f32;
    let mean = z.iter().sum::<f32>() / n;
    let var = z.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
    let inv = 1.0 / (var + LN_EPS).sqrt();
    for (((xv, zv), wv), bv) in x.iter_mut().zip(z).zip(w).zip(b) {
        *xv += (zv - mean) * inv * wv + bv;
    }
}

// ───────────────────────── ERB ─────────────────────────

/// 稀疏的 ERB 矩阵：每行只存非零区间（跳过的都是精确 0，结果与稠密乘法相同）。
struct Sparse {
    rows: Vec<(usize, Vec<f32>)>, // (起始列, 非零段)
}

impl Sparse {
    fn from_dense(w: &[f32], rows: usize, cols: usize) -> Self {
        let mut out = Vec::with_capacity(rows);
        for r in 0..rows {
            let row = &w[r * cols..(r + 1) * cols];
            let first = row.iter().position(|v| *v != 0.0).unwrap_or(0);
            let last = row.iter().rposition(|v| *v != 0.0).map_or(first, |p| p + 1);
            out.push((first, row[first..last.max(first)].to_vec()));
        }
        Self { rows: out }
    }
    #[inline]
    fn run(&self, x: &[f32], y: &mut [f32]) {
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
    erb: Sparse,  // 64 × 192
    ierb: Sparse, // 192 × 64
    en0: ConvF,
    en1: ConvF,
    en_gt: [GtBlock; 3],
    dp: [Dp; 2],
    de_gt: [GtBlock; 3],
    de3: DeconvF,
    de4: DeconvF,
    feat0: Vec<f32>,  // [3][257]
    feat1: Vec<f32>,  // [3][129]
    feat2: Vec<f32>,  // [9][129]
    e0: Vec<f32>,     // [16][65]
    e: [Vec<f32>; 4], // e[0]=en_outs[1] [16][33]，e[1..4]=en_outs[2..5]
    t0: Vec<f32>,     // [16][33]
    t1: Vec<f32>,     // [16][33]
    d3: Vec<f32>,     // [16][65]
    d4: Vec<f32>,     // [2][129]
    m: Vec<f32>,      // [2][257]
}

impl GtcrnNative {
    pub fn load(prefix: &Path) -> Result<Self, Err> {
        let w = Weights::load(prefix)?;
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
        // 特征：mag, re, im
        for k in 0..BINS {
            self.feat0[k] = (re[k] * re[k] + im[k] * im[k] + 1e-12).sqrt();
            self.feat0[BINS + k] = re[k];
            self.feat0[2 * BINS + k] = im[k];
        }
        // ERB 压缩
        for c in 0..3 {
            let (src, dst) = (
                &self.feat0[c * BINS..(c + 1) * BINS],
                &mut self.feat1[c * F0..(c + 1) * F0],
            );
            dst[..ERB_LOW].copy_from_slice(&src[..ERB_LOW]);
            self.erb.run(&src[ERB_LOW..], &mut dst[ERB_LOW..]);
        }
        sfe3(&self.feat1, 3, F0, &mut self.feat2);
        // 编码
        self.en0.run(&self.feat2, &mut self.e0);
        let [e1, e2, e3, e4] = &mut self.e;
        self.en1.run(&self.e0, e1);
        self.en_gt[0].run(e1, e2);
        self.en_gt[1].run(e2, e3);
        self.en_gt[2].run(e3, e4);
        // DPGRNN ×2
        self.t0.copy_from_slice(e4);
        self.dp[0].run(&mut self.t0);
        self.dp[1].run(&mut self.t0);
        // 解码（跳连相加）
        add_into(&mut self.t0, &self.e[3]);
        self.de_gt[0].run(&self.t0, &mut self.t1);
        add_into(&mut self.t1, &self.e[2]);
        self.de_gt[1].run(&self.t1, &mut self.t0);
        add_into(&mut self.t0, &self.e[1]);
        self.de_gt[2].run(&self.t0, &mut self.t1);
        add_into(&mut self.t1, &self.e[0]);
        self.de3.run(&self.t1, &mut self.d3);
        add_into(&mut self.d3, &self.e0);
        self.de4.run(&self.d3, &mut self.d4);
        // ERB 还原
        for c in 0..2 {
            let (src, dst) = (
                &self.d4[c * F0..(c + 1) * F0],
                &mut self.m[c * BINS..(c + 1) * BINS],
            );
            dst[..ERB_LOW].copy_from_slice(&src[..ERB_LOW]);
            self.ierb.run(&src[ERB_LOW..], &mut dst[ERB_LOW..]);
        }
        // 复数比值掩码
        for k in 0..BINS {
            let (m0, m1) = (self.m[k], self.m[BINS + k]);
            let (r, i) = (re[k], im[k]);
            re[k] = r * m0 - i * m1;
            im[k] = i * m0 + r * m1;
        }
    }
}

fn add_into(x: &mut [f32], y: &[f32]) {
    for (a, b) in x.iter_mut().zip(y) {
        *a += b;
    }
}

#[cfg(test)]
mod tests {
    //! 不需要权重的单元测试。要权重的端到端对拍在 main.rs 的 tests 里（#[ignore]）。
    use super::*;

    /// 确定性伪随机（不引依赖）
    fn rnd(n: usize, seed: u32) -> Vec<f32> {
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

    #[test]
    fn exp_fast_close_to_libm() {
        let mut worst = 0.0f32;
        let mut x = -80.0f32;
        while x <= 80.0 {
            let rel = ((exp_fast(x) - x.exp()) / x.exp()).abs();
            worst = worst.max(rel);
            x += 0.01;
        }
        assert!(worst < 1e-6, "exp_fast 最大相对误差 {worst}");
    }

    #[test]
    fn tanh_sigmoid_fast_close_to_libm() {
        let (mut wt, mut ws) = (0.0f32, 0.0f32);
        let mut x = -20.0f32;
        while x <= 20.0 {
            wt = wt.max((tanh_fast(x) - x.tanh()).abs());
            ws = ws.max((sig_fast(x) - 1.0 / (1.0 + (-x).exp())).abs());
            x += 0.001;
        }
        assert!(
            wt < 1e-6 && ws < 1e-6,
            "tanh 最大绝对误差 {wt}，sigmoid {ws}"
        );
        assert!(
            exp_fast(-1000.0).is_finite() && exp_fast(1000.0).is_finite(),
            "越界输入要被钳住"
        );
    }

    /// 原来逐元素判断边界的写法，作参照
    fn conv_ref(c: &ConvF, x: &[f32], y: &mut [f32]) {
        let (cig, cog) = (c.cin / c.groups, c.cout / c.groups);
        for o in 0..c.cout {
            let g = o / cog;
            let yo = &mut y[o * c.fout..(o + 1) * c.fout];
            yo.fill(c.b[o]);
            for ci in 0..cig {
                let xi = &x[(g * cig + ci) * c.fin..(g * cig + ci + 1) * c.fin];
                for t in 0..K {
                    let wv = c.w[(o * cig + ci) * K + t];
                    for (j, a) in yo.iter_mut().enumerate() {
                        let idx = (j * 2 + t) as isize - 2;
                        if idx >= 0 && (idx as usize) < c.fin {
                            *a += wv * xi[idx as usize];
                        }
                    }
                }
            }
            for v in yo.iter_mut() {
                *v = prelu(*v, c.alpha);
            }
        }
    }

    fn deconv_ref(d: &DeconvF, x: &[f32], y: &mut [f32]) {
        let (cig, cog) = (d.cin / d.groups, d.cout / d.groups);
        for o in 0..d.cout {
            y[o * d.fout..(o + 1) * d.fout].fill(d.b[o]);
        }
        for ic in 0..d.cin {
            let g = ic / cig;
            let xi = &x[ic * d.fin..(ic + 1) * d.fin];
            for ol in 0..cog {
                let o = g * cog + ol;
                let yo = &mut y[o * d.fout..(o + 1) * d.fout];
                for k in 0..K {
                    let wv = d.w[(ic * cog + ol) * K + k];
                    for (i, &xv) in xi.iter().enumerate() {
                        let j = (i * 2 + k) as isize - 2;
                        if j >= 0 && (j as usize) < d.fout {
                            yo[j as usize] += wv * xv;
                        }
                    }
                }
            }
        }
        for v in y[..d.cout * d.fout].iter_mut() {
            *v = if d.tanh { v.tanh() } else { prelu(*v, d.alpha) };
        }
    }

    /// 去分支后的卷积与原写法逐位相同（GTCRN 实际用到的三种尺寸）
    #[test]
    fn conv_branch_free_bitwise_equal() {
        for (cin, cout, groups, fin) in [(9, 16, 1, F0), (16, 16, 2, F1), (4, 4, 2, 7)] {
            let cig = cin / groups;
            let fout = (fin + 2 * 2 - K) / 2 + 1;
            let mut c = ConvF {
                w: rnd(cout * cig * K, 1),
                b: rnd(cout, 2),
                cin,
                cout,
                groups,
                fin,
                fout,
                alpha: 0.25,
                col: vec![0.0; cig * K * fout],
            };
            let x = rnd(cin * fin, 3);
            let (mut y1, mut y2) = (vec![0.0; cout * fout], vec![0.0; cout * fout]);
            conv_ref(&c, &x, &mut y1);
            BLAS.store(false, std::sync::atomic::Ordering::Relaxed);
            c.run(&x, &mut y2);
            assert!(
                y1.iter().zip(&y2).all(|(a, b)| a.to_bits() == b.to_bits()),
                "ConvF {cin}->{cout}/{groups} @{fin}"
            );
        }
    }

    #[test]
    fn deconv_branch_free_bitwise_equal() {
        for (cin, cout, groups, fin, tanh) in [
            (16, 16, 2, F2, false),
            (16, 2, 1, F1, true),
            (4, 4, 2, 5, false),
        ] {
            let cog = cout / groups;
            let fout = (fin - 1) * 2 - 2 * 2 + K;
            let d = DeconvF {
                w: rnd(cin * cog * K, 4),
                b: rnd(cout, 5),
                cin,
                cout,
                groups,
                fin,
                fout,
                alpha: 0.1,
                tanh,
            };
            let x = rnd(cin * fin, 6);
            let (mut y1, mut y2) = (vec![0.0; cout * fout], vec![0.0; cout * fout]);
            deconv_ref(&d, &x, &mut y1);
            d.run(&x, &mut y2);
            assert!(
                y1.iter().zip(&y2).all(|(a, b)| a.to_bits() == b.to_bits()),
                "DeconvF {cin}->{cout}/{groups} @{fin}"
            );
        }
    }

    #[test]
    fn weights_reject_mismatch() {
        let dir = std::env::temp_dir().join(format!("exp010-w-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("w");
        std::fs::write(p.with_extension("bin"), [0u8; 8]).unwrap(); // 2 个 f32
        std::fs::write(p.with_extension("txt"), "a 3 0 3\n").unwrap(); // 声称 3 个
        assert!(Weights::load(&p).is_err(), "总数对不上要报错");
        std::fs::write(p.with_extension("txt"), "a 2 0 2\n").unwrap();
        let w = Weights::load(&p).unwrap();
        assert!(w.get("a", 3).is_err() && w.get("b", 2).is_err() && w.get("a", 2).is_ok());
        std::fs::write(p.with_extension("bin"), [0u8; 7]).unwrap();
        assert!(Weights::load(&p).is_err(), "bin 长度不是 4 的倍数要报错");
        std::fs::remove_dir_all(&dir).ok();
    }
}
