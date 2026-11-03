//! 原写法的积木：非线性、逐点卷积、频率卷积、转置卷积、GRU、全连接。

use super::*;

// ───────────────────────── 积木 ─────────────────────────

// ── 可选：内联多项式 exp（后端名 native-fast）──
// 系统 libm 的 expf / tanhf 占手写版约 23% 时间（profile_native.txt），且是外部调用、挡住向量化。
// 这里用 2^n · p(r) 的经典做法：n = round(x·log2e)，r = x − n·ln2（ln2 拆两段减小舍入），p 为 6 阶泰勒/极小化多项式，
// 在 |r| ≤ ln2/2 上相对误差约 2e-7（推断，同 Cephes expf 的阶数）。结果不再与 libm 逐位相同，靠对拍判。
pub static FAST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[inline]
pub(crate) fn fast() -> bool {
    FAST.load(std::sync::atomic::Ordering::Relaxed)
}
#[inline(always)]
pub(crate) fn exp_fast(x: f32) -> f32 {
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
pub(crate) fn tanh_fast(x: f32) -> f32 {
    // tanh(x) = 1 − 2 / (e^{2x} + 1)；x 很小时有相消，绝对误差约 1e-7 级（相对 tanh 的取值 ±1 可忽略）
    1.0 - 2.0 / (exp_fast(2.0 * x) + 1.0)
}

#[inline(always)]
pub(crate) fn sig_fast(x: f32) -> f32 {
    1.0 / (1.0 + exp_fast(-x))
}
/// 给 hw-peak 量延迟用（与推理里用的是同一个函数）
pub fn exp_fast_pub(x: f32) -> f32 {
    exp_fast(x)
}
pub fn sig_fast_pub(x: f32) -> f32 {
    sig_fast(x)
}
#[inline]
pub(crate) fn sigmoid(x: f32) -> f32 {
    if fast() {
        1.0 / (1.0 + exp_fast(-x))
    } else {
        1.0 / (1.0 + (-x).exp())
    }
}
#[inline]
pub(crate) fn tanh_(x: f32) -> f32 {
    if fast() { tanh_fast(x) } else { x.tanh() }
}

#[inline]
pub(crate) fn prelu(x: f32, a: f32) -> f32 {
    if x >= 0.0 { x } else { a * x }
}

/// 1×1 卷积（逐点），权重 `[out][in]`，已折 BN（可选）。
pub(crate) struct Point {
    pub(crate) w: Vec<f32>,
    pub(crate) b: Vec<f32>,
    pub(crate) cin: usize,
    pub(crate) cout: usize,
}

impl Point {
    /// `transposed`：ConvTranspose 的权重是 `[in][out]`，转成 `[out][in]`。
    pub(crate) fn load(
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

    pub(crate) fn cost(&self, f: usize) -> Cost {
        let (ci, co, f) = (self.cin as u64, self.cout as u64, f as u64);
        Cost {
            macs: co * ci * f,
            w_bytes: (co * ci + co) * F32B,
            io_bytes: (ci + co) * f * F32B,
            chain_fma: clog2(self.cin) + 1,
            ..Cost::default()
        }
    }

    pub(crate) fn run(&self, x: &[f32], y: &mut [f32], f: usize) {
        if blas() {
            for o in 0..self.cout {
                y[o * f..(o + 1) * f].fill(self.b[o]);
            }
            sgemm_acc(self.cout, f, self.cin, &self.w, self.cin, x, f, false, y, f);
            return;
        }
        self.run_plain(x, y, f);
    }

    /// 手写循环版（不看 BLAS 开关；v2 / v3 的退路用它，codex 020 第 1 轮 #2）
    pub(crate) fn run_plain(&self, x: &[f32], y: &mut [f32], f: usize) {
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
pub(crate) struct ConvF {
    pub(crate) w: Vec<f32>,
    pub(crate) b: Vec<f32>,
    pub(crate) cin: usize,
    pub(crate) cout: usize,
    pub(crate) groups: usize,
    pub(crate) fin: usize,
    pub(crate) fout: usize,
    pub(crate) alpha: f32,
    pub(crate) col: Vec<f32>, // im2col 缓冲 [cig*K][fout]（只 native-blas 用）
    pub(crate) ph: Vec<f32>,  // v2：每个输入通道的偶 / 奇两相（补零），[cin][2][v2::conv_phase_len(fout)]
}

pub(crate) const K: usize = 5;

impl ConvF {
    pub(crate) fn load(
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
            ph: vec![0.0; cin * 2 * v2::conv_phase_len(fout)],
        })
    }

    /// 只数有效抽头（越界补 0 的不算），与 `run` 的循环区间相同
    pub(crate) fn cost(&self) -> Cost {
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

    pub(crate) fn run(&mut self, x: &[f32], y: &mut [f32]) {
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
pub(crate) struct DeconvF {
    pub(crate) w: Vec<f32>,
    pub(crate) b: Vec<f32>,
    pub(crate) cin: usize,
    pub(crate) cout: usize,
    pub(crate) groups: usize,
    pub(crate) fin: usize,
    pub(crate) fout: usize,
    pub(crate) alpha: f32,
    pub(crate) tanh: bool,
    pub(crate) xp: Vec<f32>, // v2：输入两端各补一个 0，[cin][v2::deconv_pad_len(fin)]
}

impl DeconvF {
    pub(crate) fn load(
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
            xp: vec![0.0; cin * v2::deconv_pad_len(fin)],
        })
    }

    pub(crate) fn cost(&self) -> Cost {
        let (cig, cog) = (self.cin / self.groups, self.cout / self.groups);
        let taps: usize = (0..K)
            .map(|k| {
                let ilo = (2usize.saturating_sub(k)).div_ceil(2);
                let ihi = (self.fout + 2 - k).div_ceil(2).min(self.fin);
                ihi.saturating_sub(ilo)
            })
            .sum();
        let (ci, co, fo) = (self.cin as u64, self.cout as u64, self.fout as u64);
        let act = co * fo;
        Cost {
            macs: ci * (cog * taps) as u64,
            nonlin: if self.tanh { act } else { 0 },
            other: if self.tanh { 0 } else { act },
            w_bytes: (ci * (cog * K) as u64 + co) * F32B,
            io_bytes: (ci * self.fin as u64 + act) * F32B,
            // stride 2、k 5：每个输出点最多收 3 个抽头 × 每组输入通道
            chain_fma: clog2(cig * 3) + 1 + u64::from(!self.tanh),
            chain_nl: u64::from(self.tanh),
        }
    }

    pub(crate) fn run(&self, x: &[f32], y: &mut [f32]) {
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
pub(crate) struct Gru {
    pub(crate) wi: Vec<f32>, // [3H][I]
    pub(crate) wh: Vec<f32>, // [3H][H]
    pub(crate) bi: Vec<f32>,
    pub(crate) bh: Vec<f32>,
    pub(crate) i: usize,
    pub(crate) h: usize,
    pub(crate) wit: Vec<f32>, // [I][3H]：Wi 的转置（v2 按道广播乘加用）
    pub(crate) wht: Vec<f32>, // [H][3H]
}

/// [rows][cols] → [cols][rows]
pub(crate) fn transpose(w: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut t = vec![0.0; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            t[c * rows + r] = w[r * cols + c];
        }
    }
    t
}

pub(crate) const MAX_G: usize = 48;

impl Gru {
    pub(crate) fn load(w: &Weights, p: &str, sfx: &str, i: usize, h: usize) -> Result<Self, Err> {
        if 3 * h > MAX_G {
            return Err(format!("{p}: 3H = {} 超过栈缓冲 MAX_G = {MAX_G}", 3 * h).into());
        }
        let wi = w.get(&format!("{p}.weight_ih_l0{sfx}"), 3 * h * i)?;
        let wh = w.get(&format!("{p}.weight_hh_l0{sfx}"), 3 * h * h)?;
        Ok(Self {
            wit: transpose(&wi, 3 * h, i),
            wht: transpose(&wh, 3 * h, h),
            wi,
            wh,
            bi: w.get(&format!("{p}.bias_ih_l0{sfx}"), 3 * h)?,
            bh: w.get(&format!("{p}.bias_hh_l0{sfx}"), 3 * h)?,
            i,
            h,
        })
    }

    /// 单步的工作量。关键路径只算依赖上一步状态 h 的部分：Wh·h（树形）→ 加 gi → sigmoid(r) →
    /// r·gh + gi → tanh → (1−z)·n + z·h（2 步）；z 的 sigmoid 与 r 并行，不在路径上。gi = Wi·x 不依赖 h。
    pub(crate) fn step_cost(&self) -> Cost {
        let (i, h) = (self.i as u64, self.h as u64);
        Cost {
            macs: 3 * h * (i + h),
            nonlin: 3 * h,
            other: 4 * h,
            w_bytes: (3 * h * (i + h) + 6 * h) * F32B,
            io_bytes: (i + 2 * h) * F32B,
            chain_fma: clog2(self.h) + 1 + 1 + 1 + 2,
            chain_nl: 2,
        }
    }

    /// `x` 长 I，`h` 长 H，原地更新 `h`。
    #[inline]
    pub(crate) fn step(&self, x: &[f32], h: &mut [f32]) {
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
pub(crate) struct Linear {
    pub(crate) w: Vec<f32>,
    pub(crate) b: Vec<f32>,
    pub(crate) i: usize,
    pub(crate) o: usize,
    pub(crate) wt: Vec<f32>, // [I][O]：转置（v2 用）
}

impl Linear {
    pub(crate) fn load(w: &Weights, p: &str, i: usize, o: usize) -> Result<Self, Err> {
        let ww = w.get(&format!("{p}.weight"), o * i)?;
        Ok(Self {
            wt: transpose(&ww, o, i),
            w: ww,
            b: w.get(&format!("{p}.bias"), o)?,
            i,
            o,
        })
    }
    pub(crate) fn cost(&self) -> Cost {
        let (i, o) = (self.i as u64, self.o as u64);
        Cost {
            macs: o * i,
            w_bytes: (o * i + o) * F32B,
            io_bytes: (i + o) * F32B,
            chain_fma: clog2(self.i) + 1,
            ..Cost::default()
        }
    }

    #[inline]
    pub(crate) fn run(&self, x: &[f32], y: &mut [f32]) {
        for k in 0..self.o {
            let mut a = self.b[k];
            for (w, v) in self.w[k * self.i..(k + 1) * self.i].iter().zip(x) {
                a += w * v;
            }
            y[k] = a;
        }
    }
}
