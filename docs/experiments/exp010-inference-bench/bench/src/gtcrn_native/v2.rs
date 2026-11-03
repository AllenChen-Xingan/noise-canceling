//! 研究 020：GTCRN 算子的 NEON 重写（后端名 `native-v2` / `native-v2-fast`）。
//!
//! 只换「每个算子怎么算」，模型结构、权重、BN 折叠、状态都沿用 `GtcrnNative`（正本在上层文件）。
//! 与原写法的数学关系（每条都是同一个式子，只改求和顺序或用融合乘加；误差靠 BE-PARITY 与单元测试判）：
//! - 逐点卷积：每次算 2 个输出通道 × 33 个频点（8 个向量 + 1 个标量尾），输入行读一次用两次
//! - 频率卷积（k=5、步长 2、补 2）：输入先拆成偶 / 奇两相并补零，y[j] = Σ_t w_t·xpad[2j+t]
//!   变成对两相的连续读取（t 偶取偶相 [j + t/2]，t 奇取奇相 [j + (t−1)/2]），无边界判断、可整向量化
//! - 转置卷积（k=5、步长 2、补 2）：按输出奇偶拆开，偶输出 y[2m] = w0·x[m+1] + w2·x[m] + w4·x[m−1]，
//!   奇输出 y[2m+1] = w1·x[m+1] + w3·x[m]，各自连续累加后交错写回（原写法是步长 2 的散射写）
//! - GRU：Wi、Wh 预先转置成 [输入][3H]，按输入逐个广播做向量乘加（无横向求和）；
//!   新状态 h' = n + z·(h − n)（与 (1−z)·n + z·h 相等，少一次乘）
//! - 全连接：同 GRU，权重转置后按输入广播
//! - 非线性：`FAST = false` 用 libm（逐道），`FAST = true` 用向量化的 Cephes 式 exp（与标量 exp_fast 同系数）
//! 每帧零分配：缓冲都在构造时分配好（`ConvF::ph`、`DeconvF::xp`），其余是栈上定长数组。

use super::*;
use core::arch::aarch64::*;

/// 一组 NEON 算子写法；`FAST` 选非线性的算法
pub struct V2<const FAST: bool>;

/// 频率卷积每相的长度：输出按 4 补齐，再加抽头偏移（最多 +2）与余量
pub fn conv_phase_len(fout: usize) -> usize {
    fout.div_ceil(4) * 4 + 4
}
/// 转置卷积补零后的输入长度：输入按 4 补齐，再加两端各 1 个 0 与余量
pub fn deconv_pad_len(fin: usize) -> usize {
    fin.div_ceil(4) * 4 + 4
}

/// 单行最多几个向量（频点 ≤ 68 → 17 个）
const MAXV: usize = 17;

// ───────────── 非线性 ─────────────

#[inline(always)]
unsafe fn lanes(x: float32x4_t, f: impl Fn(f32) -> f32) -> float32x4_t {
    let mut a = [0.0f32; 4];
    unsafe {
        vst1q_f32(a.as_mut_ptr(), x);
        for v in &mut a {
            *v = f(*v);
        }
        vld1q_f32(a.as_ptr())
    }
}

/// exp：n = round(x·log2e)，r = x − n·ln2（ln2 拆两段），p(r) 用 Cephes expf 系数，结果 = 2^n · p
#[inline(always)]
unsafe fn vexp(x: float32x4_t) -> float32x4_t {
    unsafe {
        let x = vminq_f32(vmaxq_f32(x, vdupq_n_f32(-87.0)), vdupq_n_f32(88.0));
        let n = vrndnq_f32(vmulq_n_f32(x, std::f32::consts::LOG2_E));
        let r = vfmsq_f32(x, n, vdupq_n_f32(0.693_359_4));
        let r = vfmsq_f32(r, n, vdupq_n_f32(-2.121_944_4e-4));
        let mut p = vdupq_n_f32(1.987_569_1e-4);
        p = vfmaq_f32(vdupq_n_f32(1.398_199_9e-3), p, r);
        p = vfmaq_f32(vdupq_n_f32(8.333_452e-3), p, r);
        p = vfmaq_f32(vdupq_n_f32(4.166_579_6e-2), p, r);
        p = vfmaq_f32(vdupq_n_f32(1.666_666_5e-1), p, r);
        p = vfmaq_f32(vdupq_n_f32(5.000_000_1e-1), p, r);
        let y = vfmaq_f32(vaddq_f32(r, vdupq_n_f32(1.0)), p, vmulq_f32(r, r));
        let e = vreinterpretq_f32_s32(vshlq_n_s32::<23>(vaddq_s32(vcvtq_s32_f32(n), vdupq_n_s32(127))));
        vmulq_f32(e, y)
    }
}

#[inline(always)]
unsafe fn sig4<const FAST: bool>(x: float32x4_t) -> float32x4_t {
    unsafe {
        if FAST {
            // 输入夹到 ±87：x < −87 时 1/(1+e^{−x}) 会落进次正规数（单元测试抓到 6e-39），夹后最小 1.6e-38 仍是正规数；
            // 被夹的部分真值 < 1.7e-38，绝对差 < 2e-38
            let one = vdupq_n_f32(1.0);
            let x = vminq_f32(vmaxq_f32(x, vdupq_n_f32(-87.0)), vdupq_n_f32(87.0));
            vdivq_f32(one, vaddq_f32(one, vexp(vnegq_f32(x))))
        } else {
            lanes(x, |v| 1.0 / (1.0 + (-v).exp()))
        }
    }
}

#[inline(always)]
unsafe fn tanh4<const FAST: bool>(x: float32x4_t) -> float32x4_t {
    unsafe {
        if FAST {
            // tanh(x) = 1 − 2 / (e^{2x} + 1)
            let one = vdupq_n_f32(1.0);
            let e = vexp(vaddq_f32(x, x));
            vsubq_f32(one, vdivq_f32(vdupq_n_f32(2.0), vaddq_f32(e, one)))
        } else {
            lanes(x, f32::tanh)
        }
    }
}

#[inline(always)]
unsafe fn prelu4(x: float32x4_t, a: f32) -> float32x4_t {
    unsafe { vbslq_f32(vcgeq_f32(x, vdupq_n_f32(0.0)), x, vmulq_n_f32(x, a)) }
}

/// 把运行时的块长 1..=9 分派到常量泛型（累加器才能留在寄存器里）
macro_rules! by_n {
    ($n:expr, $f:ident :: <$($g:tt),*> ($($a:expr),*)) => {
        match $n {
            1 => $f::<1, $($g),*>($($a),*),
            2 => $f::<2, $($g),*>($($a),*),
            3 => $f::<3, $($g),*>($($a),*),
            4 => $f::<4, $($g),*>($($a),*),
            5 => $f::<5, $($g),*>($($a),*),
            6 => $f::<6, $($g),*>($($a),*),
            7 => $f::<7, $($g),*>($($a),*),
            8 => $f::<8, $($g),*>($($a),*),
            9 => $f::<9, $($g),*>($($a),*),
            _ => unreachable!(),
        }
    };
}

// ───────────── 频率卷积（编码器） ─────────────

/// 一个输出通道、一段 N 个向量：acc = b + Σ_ci Σ_t w[ci][t] · 相[t%2][v0·4 + 4v + t/2]
#[inline(always)]
unsafe fn conv_chunk<const N: usize, const Z: usize>(
    ph: *const f32,
    pl: usize,
    w: *const f32,
    cig: usize,
    b: f32,
    v0: usize,
    out: *mut f32,
) {
    unsafe {
        let mut acc = [vdupq_n_f32(b); N];
        for ci in 0..cig {
            let e = ph.add(ci * 2 * pl + v0 * 4);
            let o = e.add(pl);
            let wr = w.add(ci * K);
            for t in 0..K {
                let src = if t % 2 == 0 { e } else { o }.add(t / 2);
                let wt = *wr.add(t);
                for (v, a) in acc.iter_mut().enumerate() {
                    *a = vfmaq_n_f32(*a, vld1q_f32(src.add(4 * v)), wt);
                }
            }
        }
        for (v, a) in acc.iter().enumerate() {
            vst1q_f32(out.add(4 * (v0 + v)), *a);
        }
    }
}

fn conv_v2(c: &mut ConvF, x: &[f32], y: &mut [f32]) {
    let (cig, cog) = (c.cin / c.groups, c.cout / c.groups);
    let pl = conv_phase_len(c.fout);
    let nv = c.fout.div_ceil(4);
    assert!(nv <= MAXV && x.len() >= c.cin * c.fin && y.len() >= c.cout * c.fout);
    assert!(c.ph.len() >= c.cin * 2 * pl && c.w.len() >= c.cout * cig * K);
    // 两相：偶相 e[m] = xpad[2m]，奇相 o[m] = xpad[2m+1]，xpad[p] = x[p−2]（越界为 0）
    for ci in 0..c.cin {
        let xi = &x[ci * c.fin..(ci + 1) * c.fin];
        let (e, o) = c.ph[ci * 2 * pl..(ci + 1) * 2 * pl].split_at_mut(pl);
        for m in 0..pl {
            let at = |p: usize| if (2..c.fin + 2).contains(&p) { xi[p - 2] } else { 0.0 };
            e[m] = at(2 * m);
            o[m] = at(2 * m + 1);
        }
    }
    let mut tmp = [0.0f32; 4 * MAXV];
    for o in 0..c.cout {
        let g = o / cog;
        let ph = c.ph[g * cig * 2 * pl..].as_ptr();
        let w = c.w[o * cig * K..].as_ptr();
        let mut v0 = 0;
        while v0 < nv {
            let n = (nv - v0).min(9);
            unsafe { by_n!(n, conv_chunk::<0>(ph, pl, w, cig, c.b[o], v0, tmp.as_mut_ptr())) };
            v0 += n;
        }
        let yo = &mut y[o * c.fout..(o + 1) * c.fout];
        unsafe {
            for v in 0..nv {
                let p = tmp.as_mut_ptr().add(4 * v);
                vst1q_f32(p, prelu4(vld1q_f32(p), c.alpha));
            }
        }
        yo.copy_from_slice(&tmp[..c.fout]);
    }
}

// ───────────── 转置卷积（解码器） ─────────────

/// 一个输出通道、一段 N 个向量的偶 / 奇输出
#[inline(always)]
unsafe fn deconv_chunk<const N: usize, const Z: usize>(
    xp: *const f32,
    dl: usize,
    w: *const f32, // 指向 w[(ic0·cog + ol)·K]，相邻输入通道相隔 cog·K
    wstride: usize,
    cig: usize,
    b: f32,
    v0: usize,
    te: *mut f32,
    to: *mut f32,
) {
    unsafe {
        let mut ae = [vdupq_n_f32(b); N];
        let mut ao = [vdupq_n_f32(b); N];
        for ic in 0..cig {
            let src = xp.add(ic * dl + 4 * v0);
            let wr = w.add(ic * wstride);
            let (w0, w1, w2, w3, w4) = (*wr, *wr.add(1), *wr.add(2), *wr.add(3), *wr.add(4));
            for v in 0..N {
                // xp[m] = x[m−1]：偶 y[2m] = w0·xp[m+2] + w2·xp[m+1] + w4·xp[m]；奇 y[2m+1] = w1·xp[m+2] + w3·xp[m+1]
                let x0 = vld1q_f32(src.add(4 * v));
                let x1 = vld1q_f32(src.add(4 * v + 1));
                let x2 = vld1q_f32(src.add(4 * v + 2));
                ae[v] = vfmaq_n_f32(vfmaq_n_f32(vfmaq_n_f32(ae[v], x2, w0), x1, w2), x0, w4);
                ao[v] = vfmaq_n_f32(vfmaq_n_f32(ao[v], x2, w1), x1, w3);
            }
        }
        for v in 0..N {
            vst1q_f32(te.add(4 * (v0 + v)), ae[v]);
            vst1q_f32(to.add(4 * (v0 + v)), ao[v]);
        }
    }
}

fn deconv_v2<const FAST: bool>(d: &mut DeconvF, x: &[f32], y: &mut [f32]) {
    let (cig, cog) = (d.cin / d.groups, d.cout / d.groups);
    let dl = deconv_pad_len(d.fin);
    let nv = d.fin.div_ceil(4);
    assert!(nv <= MAXV && d.fout == 2 * d.fin - 1);
    assert!(x.len() >= d.cin * d.fin && y.len() >= d.cout * d.fout && d.xp.len() >= d.cin * dl);
    assert!(d.w.len() >= d.cin * cog * K);
    for ic in 0..d.cin {
        let row = &mut d.xp[ic * dl..(ic + 1) * dl];
        row.fill(0.0);
        row[1..=d.fin].copy_from_slice(&x[ic * d.fin..(ic + 1) * d.fin]);
    }
    let (mut te, mut to) = ([0.0f32; 4 * MAXV], [0.0f32; 4 * MAXV]);
    for o in 0..d.cout {
        let (g, ol) = (o / cog, o % cog);
        let xp = d.xp[g * cig * dl..].as_ptr();
        let w = d.w[(g * cig * cog + ol) * K..].as_ptr();
        let mut v0 = 0;
        while v0 < nv {
            let n = (nv - v0).min(9);
            unsafe {
                by_n!(
                    n,
                    deconv_chunk::<0>(xp, dl, w, cog * K, cig, d.b[o], v0, te.as_mut_ptr(), to.as_mut_ptr())
                )
            };
            v0 += n;
        }
        unsafe {
            for v in 0..nv {
                for t in [te.as_mut_ptr(), to.as_mut_ptr()] {
                    let p = t.add(4 * v);
                    let a = vld1q_f32(p);
                    vst1q_f32(p, if d.tanh { tanh4::<FAST>(a) } else { prelu4(a, d.alpha) });
                }
            }
        }
        let yo = &mut y[o * d.fout..(o + 1) * d.fout];
        for m in 0..d.fin {
            yo[2 * m] = te[m];
            if m + 1 < d.fin {
                yo[2 * m + 1] = to[m];
            }
        }
    }
}

// ───────────── 逐点卷积 ─────────────

/// f = 33 专用：每次 2 个输出通道，8 个向量 + 1 个标量尾
fn point_v2(p: &Point, x: &[f32], y: &mut [f32], f: usize) {
    if f != F2 || p.cout % 2 != 0 {
        return p.run(x, y, f);
    }
    let ci = p.cin;
    assert!(x.len() >= ci * f && y.len() >= p.cout * f && p.w.len() >= p.cout * ci);
    unsafe {
        for o in (0..p.cout).step_by(2) {
            let (b0, b1) = (p.b[o], p.b[o + 1]);
            let mut a0 = [vdupq_n_f32(b0); 8];
            let mut a1 = [vdupq_n_f32(b1); 8];
            let (mut t0, mut t1) = (b0, b1);
            let w0 = p.w.as_ptr().add(o * ci);
            let w1 = w0.add(ci);
            for i in 0..ci {
                let row = x.as_ptr().add(i * f);
                let (s0, s1) = (*w0.add(i), *w1.add(i));
                for v in 0..8 {
                    let xv = vld1q_f32(row.add(4 * v));
                    a0[v] = vfmaq_n_f32(a0[v], xv, s0);
                    a1[v] = vfmaq_n_f32(a1[v], xv, s1);
                }
                let xt = *row.add(32);
                t0 = s0.mul_add(xt, t0);
                t1 = s1.mul_add(xt, t1);
            }
            let y0 = y.as_mut_ptr().add(o * f);
            let y1 = y0.add(f);
            for v in 0..8 {
                vst1q_f32(y0.add(4 * v), a0[v]);
                vst1q_f32(y1.add(4 * v), a1[v]);
            }
            *y0.add(32) = t0;
            *y1.add(32) = t1;
        }
    }
}

// ───────────── GRU 与全连接 ─────────────

/// H 个隐单元，NG = 3H/4 个门向量
#[inline(always)]
unsafe fn gru_step<const H: usize, const NG: usize, const FAST: bool>(g: &Gru, x: &[f32], h: &mut [f32]) {
    const { assert!(NG * 4 == 3 * H) };
    assert!(x.len() >= g.i && h.len() >= H && g.wit.len() >= g.i * 3 * H && g.wht.len() >= H * 3 * H);
    unsafe {
        let mut gi = [vdupq_n_f32(0.0); NG];
        let mut gh = [vdupq_n_f32(0.0); NG];
        for v in 0..NG {
            gi[v] = vld1q_f32(g.bi.as_ptr().add(4 * v));
            gh[v] = vld1q_f32(g.bh.as_ptr().add(4 * v));
        }
        for k in 0..g.i {
            let (xk, row) = (x[k], g.wit.as_ptr().add(k * 3 * H));
            for (v, a) in gi.iter_mut().enumerate() {
                *a = vfmaq_n_f32(*a, vld1q_f32(row.add(4 * v)), xk);
            }
        }
        for k in 0..H {
            let (hk, row) = (h[k], g.wht.as_ptr().add(k * 3 * H));
            for (v, a) in gh.iter_mut().enumerate() {
                *a = vfmaq_n_f32(*a, vld1q_f32(row.add(4 * v)), hk);
            }
        }
        let q = H / 4;
        for c in 0..q {
            let r = sig4::<FAST>(vaddq_f32(gi[c], gh[c]));
            let z = sig4::<FAST>(vaddq_f32(gi[q + c], gh[q + c]));
            let n = tanh4::<FAST>(vfmaq_f32(gi[2 * q + c], r, gh[2 * q + c]));
            let hp = h.as_mut_ptr().add(4 * c);
            vst1q_f32(hp, vfmaq_f32(n, z, vsubq_f32(vld1q_f32(hp), n)));
        }
    }
}

#[inline(always)]
unsafe fn linear_v<const OV: usize>(l: &Linear, x: &[f32], y: &mut [f32]) {
    assert!(l.o == 4 * OV && x.len() >= l.i && y.len() >= l.o && l.wt.len() >= l.i * l.o);
    unsafe {
        let mut acc = [vdupq_n_f32(0.0); OV];
        for (v, a) in acc.iter_mut().enumerate() {
            *a = vld1q_f32(l.b.as_ptr().add(4 * v));
        }
        for k in 0..l.i {
            let (xk, row) = (x[k], l.wt.as_ptr().add(k * l.o));
            for (v, a) in acc.iter_mut().enumerate() {
                *a = vfmaq_n_f32(*a, vld1q_f32(row.add(4 * v)), xk);
            }
        }
        for (v, a) in acc.iter().enumerate() {
            vst1q_f32(y.as_mut_ptr().add(4 * v), *a);
        }
    }
}

impl<const FAST: bool> Kern for V2<FAST> {
    #[inline(always)]
    fn point(p: &Point, x: &[f32], y: &mut [f32], f: usize) {
        point_v2(p, x, y, f)
    }
    #[inline(always)]
    fn conv(c: &mut ConvF, x: &[f32], y: &mut [f32]) {
        conv_v2(c, x, y)
    }
    #[inline(always)]
    fn deconv(d: &mut DeconvF, x: &[f32], y: &mut [f32]) {
        deconv_v2::<FAST>(d, x, y)
    }
    #[inline(always)]
    fn gru(g: &Gru, x: &[f32], h: &mut [f32]) {
        unsafe {
            match g.h {
                4 => gru_step::<4, 3, FAST>(g, x, h),
                8 => gru_step::<8, 6, FAST>(g, x, h),
                16 => gru_step::<16, 12, FAST>(g, x, h),
                _ => g.step(x, h),
            }
        }
    }
    #[inline(always)]
    fn linear(l: &Linear, x: &[f32], y: &mut [f32]) {
        unsafe {
            match l.o {
                8 => linear_v::<2>(l, x, y),
                16 => linear_v::<4>(l, x, y),
                _ => l.run(x, y),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! 每个算子：v2 对原写法（同一组随机权重与输入）最大绝对差有界；不需要仓外权重。
    use super::super::tests_support::rnd;
    use super::*;

    fn max_diff(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max)
    }

    #[test]
    fn conv_v2_matches_scalar() {
        for (cin, cout, groups, fin) in [(9, 16, 1, F0), (16, 16, 2, F1), (4, 4, 2, 7), (2, 2, 1, 3)] {
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
                col: Vec::new(),
                ph: vec![0.0; cin * 2 * conv_phase_len(fout)],
            };
            let x = rnd(cin * fin, 3);
            let (mut y1, mut y2) = (vec![0.0; cout * fout], vec![0.0; cout * fout]);
            BLAS.store(false, std::sync::atomic::Ordering::Relaxed);
            c.run(&x, &mut y1);
            conv_v2(&mut c, &x, &mut y2);
            let d = max_diff(&y1, &y2);
            assert!(d < 1e-5, "ConvF {cin}->{cout}/{groups} @{fin}: {d}");
        }
    }

    #[test]
    fn deconv_v2_matches_scalar() {
        for (cin, cout, groups, fin, tanh) in
            [(16, 16, 2, F2, false), (16, 2, 1, F1, true), (4, 4, 2, 5, false), (2, 2, 1, 2, true)]
        {
            let cog = cout / groups;
            let fout = (fin - 1) * 2 - 2 * 2 + K;
            let mut d = DeconvF {
                w: rnd(cin * cog * K, 4),
                b: rnd(cout, 5),
                cin,
                cout,
                groups,
                fin,
                fout,
                alpha: 0.1,
                tanh,
                xp: vec![0.0; cin * deconv_pad_len(fin)],
            };
            let x = rnd(cin * fin, 6);
            let (mut y1, mut y2, mut y3) = (vec![0.0; cout * fout], vec![0.0; cout * fout], vec![0.0; cout * fout]);
            FAST.store(false, std::sync::atomic::Ordering::Relaxed);
            d.run(&x, &mut y1);
            deconv_v2::<false>(&mut d, &x, &mut y2);
            deconv_v2::<true>(&mut d, &x, &mut y3);
            let (d2, d3) = (max_diff(&y1, &y2), max_diff(&y1, &y3));
            assert!(d2 < 1e-5 && d3 < 1e-5, "DeconvF {cin}->{cout}/{groups} @{fin}: {d2} {d3}");
        }
    }

    #[test]
    fn point_gru_linear_v2_match_scalar() {
        FAST.store(false, std::sync::atomic::Ordering::Relaxed);
        for (cin, cout) in [(24, 16), (16, 8)] {
            let p = Point { w: rnd(cin * cout, 7), b: rnd(cout, 8), cin, cout };
            let x = rnd(cin * F2, 9);
            let (mut y1, mut y2) = (vec![0.0; cout * F2], vec![0.0; cout * F2]);
            p.run(&x, &mut y1, F2);
            point_v2(&p, &x, &mut y2, F2);
            assert!(max_diff(&y1, &y2) < 1e-5, "Point {cin}->{cout}");
        }
        for (i, h) in [(8, 4), (8, 8), (8, 16)] {
            let (wi, wh) = (rnd(3 * h * i, 10), rnd(3 * h * h, 11));
            let g = Gru {
                wit: transpose(&wi, 3 * h, i),
                wht: transpose(&wh, 3 * h, h),
                wi,
                wh,
                bi: rnd(3 * h, 12),
                bh: rnd(3 * h, 13),
                i,
                h,
            };
            let x = rnd(i, 14);
            let h0 = rnd(h, 15);
            let (mut a, mut b, mut c) = (h0.clone(), h0.clone(), h0.clone());
            // 连跑 33 步（沿频率 GRU 的长度），看误差不累积放大
            for _ in 0..33 {
                g.step(&x, &mut a);
                unsafe {
                    match h {
                        4 => {
                            gru_step::<4, 3, false>(&g, &x, &mut b);
                            gru_step::<4, 3, true>(&g, &x, &mut c)
                        }
                        8 => {
                            gru_step::<8, 6, false>(&g, &x, &mut b);
                            gru_step::<8, 6, true>(&g, &x, &mut c)
                        }
                        _ => {
                            gru_step::<16, 12, false>(&g, &x, &mut b);
                            gru_step::<16, 12, true>(&g, &x, &mut c)
                        }
                    }
                }
            }
            let (db, dc) = (max_diff(&a, &b), max_diff(&a, &c));
            assert!(db < 1e-5 && dc < 1e-5, "GRU I{i} H{h}: libm {db}，fast {dc}");
        }
        for (i, o) in [(16, 16), (16, 8)] {
            let w = rnd(o * i, 16);
            let l = Linear { wt: transpose(&w, o, i), w, b: rnd(o, 17), i, o };
            let x = rnd(i, 18);
            let (mut y1, mut y2) = (vec![0.0; o], vec![0.0; o]);
            l.run(&x, &mut y1);
            unsafe {
                if o == 16 {
                    linear_v::<4>(&l, &x, &mut y2)
                } else {
                    linear_v::<2>(&l, &x, &mut y2)
                }
            }
            assert!(max_diff(&y1, &y2) < 1e-5, "Linear {i}->{o}");
        }
    }

    /// v2-fast 的非线性在极端输入上不出非有限值、不出次正规数（次正规数在 ARM 上不慢，但会让下一层带着它算；
    /// 这条查 20261102-v2 那帧 296 ms 是否与它有关——结论是那帧压在系统睡眠上，这里作防线）
    #[test]
    fn fast_nonlin_no_subnormal_or_nonfinite() {
        let xs = [-1e30f32, -1e4, -100.0, -88.0, -87.0, -20.0, -1e-30, 0.0, 1e-30, 1e-7, 20.0, 87.0, 88.0, 100.0, 1e4, 1e30];
        for &x in &xs {
            let out = unsafe {
                let v = vdupq_n_f32(x);
                let mut a = [0.0f32; 3];
                for (k, r) in [vexp(v), sig4::<true>(v), tanh4::<true>(v)].into_iter().enumerate() {
                    let mut b = [0.0f32; 4];
                    vst1q_f32(b.as_mut_ptr(), r);
                    a[k] = b[0];
                }
                a
            };
            for (k, y) in out.iter().enumerate() {
                assert!(y.is_finite(), "x = {x}，第 {k} 个函数得 {y}");
                assert!(*y == 0.0 || y.is_normal(), "x = {x}，第 {k} 个函数得次正规数 {y:e}");
            }
            assert!((0.0..=1.0).contains(&out[1]) && (-1.0..=1.0).contains(&out[2]), "x = {x}：{out:?}");
        }
    }

    #[test]
    fn vexp_close_to_libm() {
        let mut worst = 0.0f32;
        let mut x = -80.0f32;
        while x <= 80.0 {
            let v = unsafe {
                let mut a = [0.0f32; 4];
                vst1q_f32(a.as_mut_ptr(), vexp(vdupq_n_f32(x)));
                a[0]
            };
            worst = worst.max(((v - x.exp()) / x.exp()).abs());
            x += 0.01;
        }
        assert!(worst < 1e-6, "vexp 最大相对误差 {worst}");
    }
}
