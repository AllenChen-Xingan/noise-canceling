//! ESTOI（扩展短时客观可懂度，Jensen & Taal 2016），口径照 pystoi 0.4.1（读懂后用 Rust 重写，不照抄）：
//! 重采样到 10 kHz（Octave 兼容的 Kaiser 窗多相重采样）→ 按干净轨去掉比最响帧低 40 dB 的帧（256 点 Hann、跳 128，重叠相加拼回）
//! → 512 点 STFT → 15 个 1/3 倍频程带（最低中心 150 Hz）→ 每 30 帧一段，先按带、再按帧去均值归一，求相关的均值。
//!
//! 和上游的差别：pystoi 归一前加 `EPS·randn` 防除零（量级 1e-16，对结果无影响），这里不加随机数、改为分母加 EPS，结果确定。
//! 对拍：`tests/fixtures/estoi-pystoi.json`（上游 Python 算的参考值），差 ≤ 0.01。

use realfft::RealFftPlanner;

const FS: usize = 10_000;
const N_FRAME: usize = 256;
const HOP: usize = N_FRAME / 2;
const NFFT: usize = 512;
const NUMBAND: usize = 15;
const MINFREQ: f64 = 150.0;
const N_SEG: usize = 30;
const DYN_RANGE: f64 = 40.0;
const EPS: f64 = f64::EPSILON;

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// 第一类零阶修正贝塞尔函数（级数，Kaiser 窗用）。
fn bessel_i0(x: f64) -> f64 {
    let (mut sum, mut term) = (1.0, 1.0);
    let q = x * x / 4.0;
    for k in 1..200 {
        term *= q / (k as f64 * k as f64);
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        let px = std::f64::consts::PI * x;
        px.sin() / px
    }
}

/// pystoi `_resample_window_oct`：p、q 已约分。返回归一化（和为 1）后的窗。
fn oct_window(p: usize, q: usize) -> Vec<f64> {
    let stop = 1.0 / (2.0 * p.max(q) as f64);
    let roll = stop / 10.0;
    let rej_db = 60.0; // −20 · log10(1e−3)
    let l = ((rej_db - 8.0) / (28.714 * roll)).ceil() as i64;
    let beta = 0.1102 * (rej_db - 8.7);
    let m = (2 * l + 1) as f64;
    let mut h: Vec<f64> = (-l..=l)
        .enumerate()
        .map(|(n, t)| {
            let r = 2.0 * n as f64 / (m - 1.0) - 1.0;
            let kaiser = bessel_i0(beta * (1.0 - r * r).max(0.0).sqrt()) / bessel_i0(beta);
            kaiser * 2.0 * p as f64 * stop * sinc(2.0 * stop * t as f64)
        })
        .collect();
    let s: f64 = h.iter().sum();
    for v in &mut h {
        *v /= s;
    }
    h
}

/// scipy `resample_poly(x, up, down, window=h)` 的直接形式（零填充边界）。
pub(crate) fn resample_oct(x: &[f64], fs_out: usize, fs_in: usize) -> Vec<f64> {
    let g = gcd(fs_out, fs_in);
    let (up, down) = (fs_out / g, fs_in / g);
    if up == down {
        return x.to_vec();
    }
    let h: Vec<f64> = oct_window(up, down).iter().map(|v| v * up as f64).collect();
    let half = (h.len() - 1) / 2;
    let pre_pad = down - half % down;
    let pre_remove = (half + pre_pad) / down;
    let n_in = x.len();
    let n_out = (n_in * up).div_ceil(down);
    (0..n_out)
        .map(|i| {
            let t = ((i + pre_remove) * down) as i64 - pre_pad as i64;
            // u[idx] 只在 idx ≡ 0 (mod up) 处非零：j = t − idx
            let mut acc = 0.0;
            let mut idx = t - t.rem_euclid(up as i64);
            while idx >= 0 {
                let j = t - idx;
                if j as usize >= h.len() {
                    break;
                }
                let k = (idx as usize) / up;
                if k < n_in {
                    acc += h[j as usize] * x[k];
                }
                idx -= up as i64;
            }
            acc
        })
        .collect()
}

/// `np.hanning(n + 2)[1:-1]`。
fn hann_inner(n: usize) -> Vec<f64> {
    let m = (n + 2) as f64;
    (1..=n)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (m - 1.0)).cos())
        .collect()
}

fn frame_starts(len: usize) -> impl Iterator<Item = usize> {
    (0..len.saturating_sub(N_FRAME)).step_by(HOP)
}

fn remove_silent(x: &[f64], y: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let w = hann_inner(N_FRAME);
    let starts: Vec<usize> = frame_starts(x.len()).collect();
    let en: Vec<f64> = starts
        .iter()
        .map(|&s| {
            let e: f64 = (0..N_FRAME).map(|k| (w[k] * x[s + k]).powi(2)).sum();
            20.0 * (e.sqrt() + EPS).log10()
        })
        .collect();
    let max = en.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let keep: Vec<usize> = starts
        .iter()
        .zip(&en)
        .filter(|(_, e)| max - DYN_RANGE - **e < 0.0)
        .map(|(s, _)| *s)
        .collect();
    if keep.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let out_len = (keep.len() - 1) * HOP + N_FRAME;
    let (mut xo, mut yo) = (vec![0.0; out_len], vec![0.0; out_len]);
    for (i, &s) in keep.iter().enumerate() {
        for k in 0..N_FRAME {
            xo[i * HOP + k] += w[k] * x[s + k];
            yo[i * HOP + k] += w[k] * y[s + k];
        }
    }
    (xo, yo)
}

/// 1/3 倍频程带：每带 `[lo, hi)` 的 bin 范围（pystoi `thirdoct`）。
fn third_oct() -> Vec<(usize, usize)> {
    let nb = NFFT / 2 + 1;
    let f: Vec<f64> = (0..nb)
        .map(|i| i as f64 * FS as f64 / NFFT as f64)
        .collect();
    let nearest = |hz: f64| -> usize {
        let mut best = 0;
        for (i, v) in f.iter().enumerate() {
            if (v - hz).powi(2) < (f[best] - hz).powi(2) {
                best = i;
            }
        }
        best
    };
    (0..NUMBAND)
        .map(|k| {
            let k = k as f64;
            let lo = MINFREQ * 2f64.powf((2.0 * k - 1.0) / 6.0);
            let hi = MINFREQ * 2f64.powf((2.0 * k + 1.0) / 6.0);
            (nearest(lo), nearest(hi))
        })
        .collect()
}

/// 每帧 15 带的幅度（`sqrt(OBM · |X|²)`），形状 [帧][带]。
fn tob(x: &[f64]) -> Vec<[f64; NUMBAND]> {
    let w = hann_inner(N_FRAME);
    let bands = third_oct();
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(NFFT);
    let mut buf = fft.make_input_vec();
    let mut spec = fft.make_output_vec();
    frame_starts(x.len())
        .map(|s| {
            buf.iter_mut().for_each(|v| *v = 0.0);
            for k in 0..N_FRAME {
                buf[k] = w[k] * x[s + k];
            }
            // 长度固定，不会出错
            let _ = fft.process(&mut buf, &mut spec);
            let mut row = [0.0; NUMBAND];
            for (b, &(lo, hi)) in bands.iter().enumerate() {
                row[b] = spec[lo..hi]
                    .iter()
                    .map(|z| z.norm_sqr())
                    .sum::<f64>()
                    .sqrt();
            }
            row
        })
        .collect()
}

/// 一段（[N_SEG][NUMBAND]）先按带（沿时间）去均值归一，再按帧（沿带）去均值归一。
fn row_col_norm(seg: &[[f64; NUMBAND]]) -> Vec<[f64; NUMBAND]> {
    let mut s = seg.to_vec();
    for b in 0..NUMBAND {
        let m = s.iter().map(|r| r[b]).sum::<f64>() / s.len() as f64;
        s.iter_mut().for_each(|r| r[b] -= m);
        let n = s.iter().map(|r| r[b] * r[b]).sum::<f64>().sqrt() + EPS;
        s.iter_mut().for_each(|r| r[b] /= n);
    }
    for r in &mut s {
        let m = r.iter().sum::<f64>() / NUMBAND as f64;
        r.iter_mut().for_each(|v| *v -= m);
        let n = r.iter().map(|v| v * v).sum::<f64>().sqrt() + EPS;
        r.iter_mut().for_each(|v| *v /= n);
    }
    s
}

/// ESTOI(干净, 被测)。长度不同时取较短的；去静音后不足 30 帧返回 `None`（pystoi 此时返回 1e−5 并告警）。
pub fn estoi(clean: &[f32], test: &[f32], sr: u32) -> Option<f64> {
    let n = clean.len().min(test.len());
    let x: Vec<f64> = clean[..n].iter().map(|&v| v as f64).collect();
    let y: Vec<f64> = test[..n].iter().map(|&v| v as f64).collect();
    let (x, y) = if sr as usize == FS {
        (x, y)
    } else {
        (
            resample_oct(&x, FS, sr as usize),
            resample_oct(&y, FS, sr as usize),
        )
    };
    let (x, y) = remove_silent(&x, &y);
    let (xt, yt) = (tob(&x), tob(&y));
    if xt.len() < N_SEG {
        return None;
    }
    let segs = xt.len() - N_SEG + 1;
    let mut total = 0.0;
    for m in 0..segs {
        let a = row_col_norm(&xt[m..m + N_SEG]);
        let b = row_col_norm(&yt[m..m + N_SEG]);
        let s: f64 = a
            .iter()
            .zip(&b)
            .map(|(ra, rb)| ra.iter().zip(rb).map(|(p, q)| p * q).sum::<f64>())
            .sum();
        total += s / N_SEG as f64;
    }
    Some(total / segs as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone_bursts(sr: u32, secs: f64, seed: u64) -> Vec<f32> {
        let mut r = crate::rng::Rng::new(seed);
        let n = (sr as f64 * secs) as usize;
        (0..n)
            .map(|i| {
                let t = i as f64 / sr as f64;
                let on = ((t * 3.0) as usize).is_multiple_of(2);
                let v = (2.0 * std::f64::consts::PI * 440.0 * t).sin() * 0.3
                    + (2.0 * std::f64::consts::PI * 1300.0 * t).sin() * 0.1
                    + 0.01 * r.normal();
                if on {
                    v as f32
                } else {
                    (0.01 * r.normal()) as f32
                }
            })
            .collect()
    }

    #[test]
    fn identical_is_one() {
        let x = tone_bursts(16_000, 3.0, 1);
        let d = estoi(&x, &x, 16_000).unwrap();
        assert!((d - 1.0).abs() < 1e-6, "{d}");
    }

    #[test]
    fn noise_lowers_score_and_scale_invariant() {
        let x = tone_bursts(16_000, 3.0, 2);
        let mut r = crate::rng::Rng::new(9);
        let y: Vec<f32> = x.iter().map(|v| v + 0.3 * r.normal() as f32).collect();
        let d = estoi(&x, &y, 16_000).unwrap();
        assert!(d < 0.9, "{d}");
        let y2: Vec<f32> = y.iter().map(|v| v * 0.25).collect();
        let d2 = estoi(&x, &y2, 16_000).unwrap();
        assert!((d - d2).abs() < 1e-9);
    }

    #[test]
    fn too_short_is_none() {
        let x = tone_bursts(16_000, 0.2, 3);
        assert!(estoi(&x, &x, 16_000).is_none());
    }

    #[test]
    fn resample_keeps_dc_and_length() {
        let x = vec![1.0f64; 1600];
        let y = resample_oct(&x, 10_000, 16_000);
        assert_eq!(y.len(), 1000);
        // 中段直流增益 1
        assert!((y[500] - 1.0).abs() < 1e-3, "{}", y[500]);
    }
}
