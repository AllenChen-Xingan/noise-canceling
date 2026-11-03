//! 研究 020 第二批（D79：「快 / 省电」以能效核、16 ms 节奏为准，按能效核特点写）：后端名 `native-v3`。
//!
//! 在 v2-fast 的基础上，再改三处（都是同一个式子，只改求和顺序 / 用 FMA / 换倒数算法；误差靠单元测试与 BE-PARITY 判）：
//! 1. **非线性的除法换成倒数估计 + 2 次牛顿迭代**（`v2::NL_RCP`）：除法单元不流水、能效核上延迟长；
//!    sigmoid = 1/(1+e^{−x})，tanh = 1 − 2·(1/(e^{2x}+1))
//! 2. **深度卷积 3×3 向量化**：每个输入行先拷进两端补 0 的缓冲（[0, 33 个值, 0…]），
//!    out[j] = Σ_kt (w0·pad[j] + w1·pad[j+1] + w2·pad[j+2])，无边界判断，9 个向量一行；PReLU 在寄存器里做完再写回
//!    （原写法 5.0 µs、离下界 5.1 倍：每个 j 判两次边界，挡住向量化）
//! 3. **LayerNorm 与 ERB 的求和向量化**：均值、方差、ERB 每行点积从「单累加器顺序求和」（浮点不许重排，只能串行）
//!    换成 4 个 NEON 累加器；仿射加回用 FMA（原写法两处 LN 合计 5.2 µs、离下界 10–12 倍）
//! 4. **GRU 显式交错**（能效核乱序窗口小，不能指望硬件自己把独立的链重叠起来）：沿频率 GRU 的 4 条链
//!    （2 组 × 正 / 反向）每步一起算，沿时间 GRU 的 33 个频点 2 个一组一起算；每条链内的式子同 v2，
//!    交错只改「先算谁」，不改任何一个值的运算
//! 其余算子（逐点卷积、频率卷积、转置卷积、全连接）沿用 v2 的写法。每帧零分配：缓冲都是栈上定长数组。

use super::v2;
use super::*;
use core::arch::aarch64::*;

/// `IL`：GRU 是否显式交错（第 4 条改写）；`NL`：非线性算法（v2::NL_*）。
/// 后端：native-v3 = <false, NL_RCP> · native-v3i = <true, NL_RCP> · native-v4 = <false, NL_DIV> · native-v4i = <true, NL_DIV>。
/// 实测（20261102-v3short）：NL_RCP 让两个 GRU 各慢约 1.1 µs（倒数 + 2 次牛顿是 5 条依赖指令，比一条除法还长），所以 v4 改回 NL_DIV
pub struct V3<const IL: bool, const NL: u8>;

/// 深度卷积 + PReLU，结果写 h2（环形缓存的更新在 GtBlock::run 里，两种写法共用）
fn depthwise_v3(b: &mut GtBlock) {
    const PAD: usize = 40; // 1 + 33 + 补齐（最远读到 4·8 + 2 + 3 = 37）
    let f = F2;
    let mut pad = [[0.0f32; PAD]; 3];
    let mut out = [0.0f32; 36];
    assert!(b.dw.len() >= C * 9 && b.db.len() >= C && b.h2.len() >= C * f);
    for c in 0..C {
        for (kt, row) in pad.iter_mut().enumerate() {
            row[1..1 + f].copy_from_slice(b.dw_src(c, kt));
        }
        unsafe {
            let mut acc = [vdupq_n_f32(b.db[c]); 9];
            for (kt, row) in pad.iter().enumerate() {
                let w = b.dw.as_ptr().add(c * 9 + kt * 3);
                let (w0, w1, w2) = (*w, *w.add(1), *w.add(2));
                let p = row.as_ptr();
                for (v, a) in acc.iter_mut().enumerate() {
                    let q = p.add(4 * v);
                    *a = vfmaq_n_f32(vfmaq_n_f32(vfmaq_n_f32(*a, vld1q_f32(q), w0), vld1q_f32(q.add(1)), w1), vld1q_f32(q.add(2)), w2);
                }
            }
            for (v, a) in acc.iter().enumerate() {
                vst1q_f32(out.as_mut_ptr().add(4 * v), v2::prelu4(*a, b.a2));
            }
        }
        b.h2[c * f..(c + 1) * f].copy_from_slice(&out[..f]);
    }
}

/// 4 × 4 路累加求和（长度须为 16 的倍数）
#[inline(always)]
unsafe fn vsum16(p: *const f32, n: usize, f: impl Fn(float32x4_t) -> float32x4_t) -> f32 {
    unsafe {
        let mut acc = [vdupq_n_f32(0.0); 4];
        let mut i = 0;
        while i < n {
            for (k, a) in acc.iter_mut().enumerate() {
                *a = vaddq_f32(*a, f(vld1q_f32(p.add(i + 4 * k))));
            }
            i += 16;
        }
        vaddvq_f32(vaddq_f32(vaddq_f32(acc[0], acc[1]), vaddq_f32(acc[2], acc[3])))
    }
}

/// x += LN(z)：均值、方差（两遍，与原写法同一公式）向量化求和；仿射 (z−μ)·inv·w + b 用 FMA
fn ln_add_v3(x: &mut [f32], z: &[f32], w: &[f32], b: &[f32]) {
    let n = z.len();
    if n % 16 != 0 || x.len() < n || w.len() < n || b.len() < n {
        return layer_norm_add(x, z, w, b);
    }
    unsafe {
        let mean = vsum16(z.as_ptr(), n, |v| v) / n as f32;
        let m = vdupq_n_f32(mean);
        let var = vsum16(z.as_ptr(), n, |v| {
            let d = vsubq_f32(v, m);
            vmulq_f32(d, d)
        }) / n as f32;
        let inv = vdupq_n_f32(1.0 / (var + LN_EPS).sqrt());
        let (px, pz, pw, pb) = (x.as_mut_ptr(), z.as_ptr(), w.as_ptr(), b.as_ptr());
        let mut i = 0;
        while i < n {
            let t = vmulq_f32(vsubq_f32(vld1q_f32(pz.add(i)), m), inv);
            let y = vfmaq_f32(vld1q_f32(pb.add(i)), t, vld1q_f32(pw.add(i)));
            vst1q_f32(px.add(i), vaddq_f32(vld1q_f32(px.add(i)), y));
            i += 4;
        }
    }
}

/// R 条互相独立的 GRU 单步交错推进（H 个隐单元、NG = 3H/4 个门向量、输入长 I）。
/// 与 `v2::gru_step` 是同一个式子；循环把 r 放在最内层，展开后 R 条链的指令交替排列。
/// SAFETY：调用方保证 xs[r] 至少 I 个、hs[r] 指向至少 H 个可写且互不重叠的 f32
#[inline(always)]
unsafe fn gru_multi<const H: usize, const NG: usize, const I: usize, const R: usize, const NL: u8>(
    gs: [&Gru; R],
    xs: [*const f32; R],
    hs: [*mut f32; R],
) {
    const { assert!(NG * 4 == 3 * H) };
    unsafe {
        let mut gi = [[vdupq_n_f32(0.0); NG]; R];
        let mut gh = [[vdupq_n_f32(0.0); NG]; R];
        for r in 0..R {
            for v in 0..NG {
                gi[r][v] = vld1q_f32(gs[r].bi.as_ptr().add(4 * v));
                gh[r][v] = vld1q_f32(gs[r].bh.as_ptr().add(4 * v));
            }
        }
        for k in 0..I {
            for r in 0..R {
                let (xk, row) = (*xs[r].add(k), gs[r].wit.as_ptr().add(k * 3 * H));
                for v in 0..NG {
                    gi[r][v] = vfmaq_n_f32(gi[r][v], vld1q_f32(row.add(4 * v)), xk);
                }
            }
        }
        for k in 0..H {
            for r in 0..R {
                let (hk, row) = (*hs[r].add(k), gs[r].wht.as_ptr().add(k * 3 * H));
                for v in 0..NG {
                    gh[r][v] = vfmaq_n_f32(gh[r][v], vld1q_f32(row.add(4 * v)), hk);
                }
            }
        }
        let q = H / 4;
        for c in 0..q {
            for r in 0..R {
                let rr = v2::sig4::<NL>(vaddq_f32(gi[r][c], gh[r][c]));
                let z = v2::sig4::<NL>(vaddq_f32(gi[r][q + c], gh[r][q + c]));
                let n = v2::tanh4::<NL>(vfmaq_f32(gi[r][2 * q + c], rr, gh[r][2 * q + c]));
                let hp = hs[r].add(4 * c);
                vst1q_f32(hp, vfmaq_f32(n, z, vsubq_f32(vld1q_f32(hp), n)));
            }
        }
    }
}

fn gru_ok(g: &Gru, i: usize, h: usize) -> bool {
    g.i == i && g.h == h && g.wit.len() >= i * 3 * h && g.wht.len() >= h * 3 * h && g.bi.len() >= 3 * h && g.bh.len() >= 3 * h
}

fn gru4_v3<const NL: u8>(gs: [&Gru; 4], xs: [&[f32]; 4], hs: &mut [[f32; 4]; 4]) {
    if !gs.iter().all(|g| gru_ok(g, 8, 4)) || !xs.iter().all(|x| x.len() >= 8) {
        for ((g, x), h) in gs.into_iter().zip(xs).zip(hs.iter_mut()) {
            <v2::V2<NL> as Kern>::gru(g, x, h);
        }
        return;
    }
    let [h0, h1, h2, h3] = hs;
    unsafe {
        gru_multi::<4, 3, 8, 4, NL>(
            gs,
            [xs[0].as_ptr(), xs[1].as_ptr(), xs[2].as_ptr(), xs[3].as_ptr()],
            [h0.as_mut_ptr(), h1.as_mut_ptr(), h2.as_mut_ptr(), h3.as_mut_ptr()],
        )
    }
}

fn gru_rows_v3<const NL: u8>(g: &Gru, xs: &[f32], hs: &mut [f32], stride: usize, n: usize) {
    let need = |len: usize| if n == 0 { 0 } else { (n - 1) * stride + len };
    if !gru_ok(g, 8, 8) || stride < 8 || xs.len() < need(8) || hs.len() < need(8) {
        for r in 0..n {
            <v2::V2<NL> as Kern>::gru(g, &xs[r * stride..r * stride + g.i], &mut hs[r * stride..r * stride + g.h]);
        }
        return;
    }
    let (px, ph) = (xs.as_ptr(), hs.as_mut_ptr());
    let mut r = 0;
    unsafe {
        // stride ≥ 8 = H，相邻两行的状态不重叠
        while r + 2 <= n {
            gru_multi::<8, 6, 8, 2, NL>(
                [g, g],
                [px.add(r * stride), px.add((r + 1) * stride)],
                [ph.add(r * stride), ph.add((r + 1) * stride)],
            );
            r += 2;
        }
        if r < n {
            gru_multi::<8, 6, 8, 1, NL>([g], [px.add(r * stride)], [ph.add(r * stride)]);
        }
    }
}

/// 频率卷积，两个输出通道一组（第 5 条改写，v4 起）：同一组输入通道的两相读一次、喂两个输出的累加器，
/// 每次加载配两次 FMA（v2 是一次加载一次 FMA，按实测 en0 约 14 次乘加 / ns，推断卡在加载上）。
/// 每个输出的式子与 v2 相同（同样的项、同样的顺序），所以与 v2 逐位相同（单元测试）
#[inline(always)]
unsafe fn conv_chunk2<const N: usize, const Z: usize>(
    ph: *const f32,
    pl: usize,
    w: [*const f32; 2],
    cig: usize,
    b: [f32; 2],
    v0: usize,
    out: [*mut f32; 2],
) {
    unsafe {
        let mut a0 = [vdupq_n_f32(b[0]); N];
        let mut a1 = [vdupq_n_f32(b[1]); N];
        for ci in 0..cig {
            let e = ph.add(ci * 2 * pl + v0 * 4);
            let o = e.add(pl);
            let (w0, w1) = (w[0].add(ci * K), w[1].add(ci * K));
            for t in 0..K {
                let src = if t % 2 == 0 { e } else { o }.add(t / 2);
                let (s0, s1) = (*w0.add(t), *w1.add(t));
                for v in 0..N {
                    let x = vld1q_f32(src.add(4 * v));
                    a0[v] = vfmaq_n_f32(a0[v], x, s0);
                    a1[v] = vfmaq_n_f32(a1[v], x, s1);
                }
            }
        }
        for v in 0..N {
            vst1q_f32(out[0].add(4 * (v0 + v)), a0[v]);
            vst1q_f32(out[1].add(4 * (v0 + v)), a1[v]);
        }
    }
}

macro_rules! by_n2 {
    ($n:expr, $($a:expr),*) => {
        match $n {
            1 => conv_chunk2::<1, 0>($($a),*),
            2 => conv_chunk2::<2, 0>($($a),*),
            3 => conv_chunk2::<3, 0>($($a),*),
            4 => conv_chunk2::<4, 0>($($a),*),
            5 => conv_chunk2::<5, 0>($($a),*),
            6 => conv_chunk2::<6, 0>($($a),*),
            7 => conv_chunk2::<7, 0>($($a),*),
            8 => conv_chunk2::<8, 0>($($a),*),
            9 => conv_chunk2::<9, 0>($($a),*),
            _ => unreachable!(),
        }
    };
}

fn conv_v4(c: &mut ConvF, x: &[f32], y: &mut [f32]) {
    let (cig, cog) = (c.cin / c.groups, c.cout / c.groups);
    let pl = v2::conv_phase_len(c.fout);
    let nv = c.fout.div_ceil(4);
    if cog % 2 != 0 || nv > 17 {
        return v2::conv_v2(c, x, y);
    }
    assert!(x.len() >= c.cin * c.fin && y.len() >= c.cout * c.fout);
    assert!(c.ph.len() >= c.cin * 2 * pl && c.w.len() >= c.cout * cig * K);
    for ci in 0..c.cin {
        let xi = &x[ci * c.fin..(ci + 1) * c.fin];
        let (e, o) = c.ph[ci * 2 * pl..(ci + 1) * 2 * pl].split_at_mut(pl);
        for m in 0..pl {
            let at = |p: usize| if (2..c.fin + 2).contains(&p) { xi[p - 2] } else { 0.0 };
            e[m] = at(2 * m);
            o[m] = at(2 * m + 1);
        }
    }
    let mut t0 = [0.0f32; 68];
    let mut t1 = [0.0f32; 68];
    for o in (0..c.cout).step_by(2) {
        let g = o / cog;
        let ph = c.ph[g * cig * 2 * pl..].as_ptr();
        let w = [c.w[o * cig * K..].as_ptr(), c.w[(o + 1) * cig * K..].as_ptr()];
        let b = [c.b[o], c.b[o + 1]];
        let mut v0 = 0;
        while v0 < nv {
            // 两个输出各 N 个累加器：N ≤ 9 → 最多 18 个，加 1 个加载，放得进 32 个寄存器
            let n = (nv - v0).min(9);
            unsafe { by_n2!(n, ph, pl, w, cig, b, v0, [t0.as_mut_ptr(), t1.as_mut_ptr()]) };
            v0 += n;
        }
        for (k, t) in [&mut t0, &mut t1].into_iter().enumerate() {
            unsafe {
                for v in 0..nv {
                    let p = t.as_mut_ptr().add(4 * v);
                    vst1q_f32(p, v2::prelu4(vld1q_f32(p), c.alpha));
                }
            }
            y[(o + k) * c.fout..(o + k + 1) * c.fout].copy_from_slice(&t[..c.fout]);
        }
    }
}

fn sparse_v3(s: &Sparse, x: &[f32], y: &mut [f32]) {
    for (yv, (st, seg)) in y.iter_mut().zip(&s.rows) {
        *yv = crate::dsp::dot_neon(seg, &x[*st..*st + seg.len()]);
    }
}

impl<const IL: bool, const NL: u8> Kern for V3<IL, NL> {
    #[inline(always)]
    fn sig1(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }
    #[inline(always)]
    fn use_blas() -> bool {
        false
    }
    #[inline(always)]
    fn point(p: &Point, x: &[f32], y: &mut [f32], f: usize) {
        v2::point_v2(p, x, y, f)
    }
    #[inline(always)]
    fn conv(c: &mut ConvF, x: &[f32], y: &mut [f32]) {
        // v3（NL_RCP）保持实测时的写法；v4 起两个输出通道一组
        if NL == v2::NL_DIV { conv_v4(c, x, y) } else { v2::conv_v2(c, x, y) }
    }
    #[inline(always)]
    fn deconv(d: &mut DeconvF, x: &[f32], y: &mut [f32]) {
        v2::deconv_v2::<NL>(d, x, y)
    }
    #[inline(always)]
    fn gru(g: &Gru, x: &[f32], h: &mut [f32]) {
        <v2::V2<NL> as Kern>::gru(g, x, h)
    }
    #[inline(always)]
    fn linear(l: &Linear, x: &[f32], y: &mut [f32]) {
        <v2::V2<NL> as Kern>::linear(l, x, y)
    }
    #[inline(always)]
    fn depthwise(b: &mut GtBlock) {
        depthwise_v3(b)
    }
    #[inline(always)]
    fn ln_add(x: &mut [f32], z: &[f32], w: &[f32], b: &[f32]) {
        ln_add_v3(x, z, w, b)
    }
    #[inline(always)]
    fn sparse(s: &Sparse, x: &[f32], y: &mut [f32]) {
        sparse_v3(s, x, y)
    }
    #[inline(always)]
    fn gru4(gs: [&Gru; 4], xs: [&[f32]; 4], hs: &mut [[f32; 4]; 4]) {
        if IL {
            gru4_v3::<NL>(gs, xs, hs)
        } else {
            for ((g, x), h) in gs.into_iter().zip(xs).zip(hs.iter_mut()) {
                Self::gru(g, x, h);
            }
        }
    }
    #[inline(always)]
    fn gru_rows(g: &Gru, xs: &[f32], hs: &mut [f32], stride: usize, n: usize) {
        if IL {
            gru_rows_v3::<NL>(g, xs, hs, stride, n)
        } else {
            for r in 0..n {
                Self::gru(g, &xs[r * stride..r * stride + g.i], &mut hs[r * stride..r * stride + g.h]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests_support::rnd;
    use super::super::v2::NL_RCP;
    use super::*;

    fn max_diff(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max)
    }

    fn lane0(v: float32x4_t) -> f32 {
        let mut a = [0.0f32; 4];
        unsafe { vst1q_f32(a.as_mut_ptr(), v) };
        a[0]
    }

    #[test]
    fn recip_nonlin_close_to_libm() {
        let (mut ws, mut wt, mut wr) = (0.0f32, 0.0f32, 0.0f32);
        let mut x = -30.0f32;
        while x <= 30.0 {
            unsafe {
                ws = ws.max((lane0(v2::sig4::<NL_RCP>(vdupq_n_f32(x))) - 1.0 / (1.0 + (-x).exp())).abs());
                wt = wt.max((lane0(v2::tanh4::<NL_RCP>(vdupq_n_f32(x))) - x.tanh()).abs());
                let d = 1.0 + x.abs();
                wr = wr.max(((lane0(v2::vrecip(vdupq_n_f32(d))) - 1.0 / d) * d).abs());
            }
            x += 0.001;
        }
        assert!(ws < 1e-6 && wt < 1e-6 && wr < 1e-6, "sigmoid {ws}，tanh {wt}，倒数相对误差 {wr}");
        for x in [-1e30f32, -100.0, -87.0, 0.0, 87.0, 100.0, 1e30] {
            let (s, t) = unsafe { (lane0(v2::sig4::<NL_RCP>(vdupq_n_f32(x))), lane0(v2::tanh4::<NL_RCP>(vdupq_n_f32(x)))) };
            assert!(s.is_finite() && (s == 0.0 || s.is_normal()) && (0.0..=1.0).contains(&s), "sigmoid({x}) = {s:e}");
            assert!(t.is_finite() && (-1.0..=1.0).contains(&t), "tanh({x}) = {t}");
        }
    }

    #[test]
    fn depthwise_v3_matches_scalar() {
        let w = Weights::fake(0.1);
        for dil in [1, 2, 5] {
            let mut a = GtBlock::load(&w, "b", dil, false).unwrap();
            a.dw = rnd(C * 9, 1);
            a.db = rnd(C, 2);
            a.a2 = 0.2;
            a.h1 = rnd(C * F2, 3);
            a.ring = rnd(2 * dil * C * F2, 4);
            for head in 0..2 * dil {
                a.head = head;
                a.depthwise_scalar();
                let y1 = a.h2.clone();
                depthwise_v3(&mut a);
                let d = max_diff(&y1, &a.h2);
                assert!(d < 1e-5, "dil {dil} head {head}: {d}");
            }
        }
    }

    #[test]
    fn ln_and_sparse_v3_match_scalar() {
        let n = F2 * C;
        let (z, w, b) = (rnd(n, 5), rnd(n, 6), rnd(n, 7));
        let x0 = rnd(n, 8);
        let (mut x1, mut x2) = (x0.clone(), x0.clone());
        layer_norm_add(&mut x1, &z, &w, &b);
        ln_add_v3(&mut x2, &z, &w, &b);
        assert!(max_diff(&x1, &x2) < 1e-5, "LN {}", max_diff(&x1, &x2));
        // 稀疏：每行非零段长度不同（含 1、15、16、17、40）
        let rows = [(0usize, 1usize), (3, 15), (10, 16), (20, 17), (5, 40)];
        let x = rnd(64, 9);
        let s = Sparse {
            rows: rows.iter().enumerate().map(|(i, (st, len))| (*st, rnd(*len, 10 + i as u32))).collect(),
        };
        let (mut y1, mut y2) = (vec![0.0; rows.len()], vec![0.0; rows.len()]);
        s.run(&x, &mut y1);
        sparse_v3(&s, &x, &mut y2);
        assert!(max_diff(&y1, &y2) < 1e-5, "ERB {}", max_diff(&y1, &y2));
    }

    fn rand_gru(i: usize, h: usize, seed: u32) -> Gru {
        let (wi, wh) = (rnd(3 * h * i, seed), rnd(3 * h * h, seed + 1));
        Gru {
            wit: transpose(&wi, 3 * h, i),
            wht: transpose(&wh, 3 * h, h),
            wi,
            wh,
            bi: rnd(3 * h, seed + 2),
            bh: rnd(3 * h, seed + 3),
            i,
            h,
        }
    }

    /// 交错版与逐条调用 v3 的单步结果逐位相同（同一个式子、同样的运算顺序，只是指令交替排列）
    #[test]
    fn gru_interleave_bitwise_equal_to_single_steps() {
        let gs: Vec<Gru> = (0..4).map(|k| rand_gru(8, 4, 100 + 10 * k)).collect();
        let xs: Vec<Vec<f32>> = (0..4).map(|k| rnd(8, 200 + k)).collect();
        let mut a = [[0.1f32, -0.2, 0.3, 0.0]; 4];
        let mut b = a;
        for _ in 0..33 {
            let gr = [&gs[0], &gs[1], &gs[2], &gs[3]];
            let xr = [&xs[0][..], &xs[1][..], &xs[2][..], &xs[3][..]];
            for k in 0..4 {
                <v2::V2<NL_RCP> as Kern>::gru(gr[k], xr[k], &mut a[k]);
            }
            gru4_v3::<NL_RCP>(gr, xr, &mut b);
        }
        assert!(a.iter().flatten().zip(b.iter().flatten()).all(|(x, y)| x.to_bits() == y.to_bits()), "{a:?} vs {b:?}");
        // 行版：33 行（奇数，走尾巴），stride 16
        let g = rand_gru(8, 8, 300);
        let xs = rnd(33 * 16, 301);
        let h0 = rnd(33 * 16, 302);
        let (mut h1, mut h2) = (h0.clone(), h0.clone());
        for r in 0..33 {
            <v2::V2<NL_RCP> as Kern>::gru(&g, &xs[r * 16..r * 16 + 8], &mut h1[r * 16..r * 16 + 8]);
        }
        gru_rows_v3::<NL_RCP>(&g, &xs, &mut h2, 16, 33);
        assert!(h1.iter().zip(&h2).all(|(x, y)| x.to_bits() == y.to_bits()), "gru_rows 与逐行不同");
    }

    /// v4 的频率卷积（两输出一组）与 v2 逐位相同
    #[test]
    fn conv_v4_bitwise_equal_v2() {
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
                col: Vec::new(),
                ph: vec![0.0; cin * 2 * v2::conv_phase_len(fout)],
            };
            let x = rnd(cin * fin, 3);
            let (mut y1, mut y2) = (vec![0.0; cout * fout], vec![0.0; cout * fout]);
            v2::conv_v2(&mut c, &x, &mut y1);
            conv_v4(&mut c, &x, &mut y2);
            assert!(y1.iter().zip(&y2).all(|(a, b)| a.to_bits() == b.to_bits()), "ConvF {cin}->{cout}/{groups} @{fin}");
        }
    }

    /// 整网：v3i（GRU 交错）与 v3 逐位相同（20 帧带状态）
    #[test]
    fn v3i_whole_net_bitwise_equal_v3() {
        let w = Weights::fake(0.03);
        let (mut a, mut b) = (GtcrnNative::from_weights(&w).unwrap(), GtcrnNative::from_weights(&w).unwrap());
        a.set_kind(Kind::V3);
        b.set_kind(Kind::V3i);
        for f in 0..20 {
            let (mut r1, mut i1) = (rnd(BINS, 90 + f), rnd(BINS, 110 + f));
            let (mut r2, mut i2) = (r1.clone(), i1.clone());
            a.infer(&mut r1, &mut i1);
            b.infer(&mut r2, &mut i2);
            assert!(r1.iter().chain(&i1).zip(r2.iter().chain(&i2)).all(|(x, y)| x.to_bits() == y.to_bits()), "第 {f} 帧不同");
        }
    }

    /// 整网：v3 对 v2-fast（同一组假权重、同一串输入，跑 20 帧带状态），差有界
    #[test]
    fn v3_whole_net_close_to_v2_fast() {
        let w = Weights::fake(0.03);
        let (mut a, mut b) = (GtcrnNative::from_weights(&w).unwrap(), GtcrnNative::from_weights(&w).unwrap());
        a.set_kind(Kind::V2Fast);
        b.set_kind(Kind::V3);
        let mut worst = 0.0f32;
        for f in 0..20 {
            let (mut r1, mut i1) = (rnd(BINS, 50 + f), rnd(BINS, 70 + f));
            let (mut r2, mut i2) = (r1.clone(), i1.clone());
            a.infer(&mut r1, &mut i1);
            b.infer(&mut r2, &mut i2);
            worst = worst.max(max_diff(&r1, &r2)).max(max_diff(&i1, &i2));
        }
        assert!(worst < 1e-4, "v3 对 v2-fast 最大差 {worst}");
    }
}
