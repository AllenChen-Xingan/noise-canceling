//! DPGRNN（沿频率双向 GRU + 沿时间 GRU）与 LayerNorm。

use super::*;

// ───────────────────────── DPGRNN ─────────────────────────

pub(crate) struct Dp {
    pub(crate) intra: [Gru; 4], // rnn1 fwd, rnn1 bwd, rnn2 fwd, rnn2 bwd（H=4）
    pub(crate) intra_fc: Linear,
    pub(crate) intra_lnw: Vec<f32>,
    pub(crate) intra_lnb: Vec<f32>,
    pub(crate) inter: [Gru; 2], // H=8
    pub(crate) inter_fc: Linear,
    pub(crate) inter_lnw: Vec<f32>,
    pub(crate) inter_lnb: Vec<f32>,
    pub(crate) cache: Vec<f32>, // [33][16]
    pub(crate) xt: Vec<f32>,    // [33][16]
    pub(crate) y: Vec<f32>,
    pub(crate) z: Vec<f32>,
}

impl Dp {
    pub(crate) fn load(w: &Weights, p: &str) -> Result<Self, Err> {
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

    pub(crate) fn reset(&mut self) {
        self.cache.fill(0.0);
    }

    /// 原地：`feat` [16][33]
    pub(crate) fn run<K: Kern, P: Probe>(&mut self, feat: &mut [f32], p: &mut P, blk: &'static str) {
        // 转成 [F][C]
        for c in 0..C {
            for f in 0..F2 {
                self.xt[f * C + c] = feat[c * F2 + f];
            }
        }
        // intra：沿频率的双向分组 GRU，初值 0。4 条链（2 组 × 正 / 反向）互相独立，
        // 每一步把 4 条一起推进（研究 020：交错让乱序执行同时跑几条依赖链；每条链内的运算与顺序不变）
        let mut hs = [[0.0f32; 4]; 4]; // [grp*2 + dir]
        let gs = [&self.intra[0], &self.intra[1], &self.intra[2], &self.intra[3]];
        for st in 0..F2 {
            let (ff, fb) = (st, F2 - 1 - st);
            let xr = |f: usize, grp: usize| &self.xt[f * C + grp * CH..f * C + grp * CH + CH];
            K::gru4(gs, [xr(ff, 0), xr(fb, 0), xr(ff, 1), xr(fb, 1)], &mut hs);
            for grp in 0..2 {
                self.y[ff * C + grp * CH..ff * C + grp * CH + 4].copy_from_slice(&hs[2 * grp]);
                self.y[fb * C + grp * CH + 4..fb * C + grp * CH + 8].copy_from_slice(&hs[2 * grp + 1]);
            }
        }
        p.mark(blk, "intra_gru", || {
            // 4 个 GRU 各沿频率 33 步；前向、后向、两组互相独立，关键路径 = 33 步串行
            let s = self.intra[0].step_cost();
            scale(s, 4 * F2 as u64, F2 as u64, 4)
                + Cost {
                    other: 2 * (F2 * C) as u64, // 转置进 + 拷贝状态
                    io_bytes: 2 * (F2 * C) as u64 * F32B,
                    ..Cost::default()
                }
        });
        if blas() {
            fc_batch(&self.intra_fc, &self.y, &mut self.z);
        } else {
            for f in 0..F2 {
                K::linear(
                    &self.intra_fc,
                    &self.y[f * C..(f + 1) * C],
                    &mut self.z[f * C..(f + 1) * C],
                );
            }
        }
        p.mark(blk, "intra_fc", || scale(self.intra_fc.cost(), F2 as u64, 1, 1));
        K::ln_add(&mut self.xt, &self.z, &self.intra_lnw, &self.intra_lnb);
        p.mark(blk, "intra_ln", || ln_cost(F2 * C));
        // inter：沿时间（单步）的分组 GRU，状态逐频点缓存；33 个频点 × 2 组互相独立（每个单步的运算与顺序不变）
        for grp in 0..2 {
            K::gru_rows(&self.inter[grp], &self.xt[grp * CH..], &mut self.cache[grp * CH..], C, F2);
        }
        for f in 0..F2 {
            if !blas() {
                K::linear(
                    &self.inter_fc,
                    &self.cache[f * C..(f + 1) * C],
                    &mut self.z[f * C..(f + 1) * C],
                );
            }
        }
        if blas() {
            // 第 f 行的全连接只依赖 cache[f]，它在上面第 f 步之后就不再变，所以可以挪到循环外一次算完
            fc_batch(&self.inter_fc, &self.cache, &mut self.z);
        }
        p.mark(blk, "inter_gru+fc", || {
            // 33 个频点 × 2 组各一步，互相独立：关键路径 = 一步 GRU + 一次全连接
            scale(self.inter[0].step_cost(), 2 * F2 as u64, 1, 2)
                + scale(self.inter_fc.cost(), F2 as u64, 1, 1)
        });
        K::ln_add(&mut self.xt, &self.z, &self.inter_lnw, &self.inter_lnb);
        for c in 0..C {
            for f in 0..F2 {
                feat[c * F2 + f] = self.xt[f * C + c];
            }
        }
        p.mark(blk, "inter_ln+out", || {
            ln_cost(F2 * C)
                + Cost {
                    other: (F2 * C) as u64,
                    io_bytes: 2 * (F2 * C) as u64 * F32B,
                    ..Cost::default()
                }
        });
    }
}

/// Z[33×16] = X[33×16]·Wᵀ + b（逐频点全连接一次做完）
pub(crate) fn fc_batch(l: &Linear, x: &[f32], z: &mut [f32]) {
    for f in 0..F2 {
        z[f * l.o..(f + 1) * l.o].copy_from_slice(&l.b);
    }
    sgemm_acc(F2, l.o, l.i, x, l.i, &l.w, l.i, true, z, l.o);
}

/// x += LN(z)，LN 在整个 (33,16) 上归一化，带逐元素仿射。
pub(crate) fn layer_norm_add(x: &mut [f32], z: &[f32], w: &[f32], b: &[f32]) {
    let n = z.len() as f32;
    let mean = z.iter().sum::<f32>() / n;
    let var = z.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
    let inv = 1.0 / (var + LN_EPS).sqrt();
    for (((xv, zv), wv), bv) in x.iter_mut().zip(z).zip(w).zip(b) {
        *xv += (zv - mean) * inv * wv + bv;
    }
}
