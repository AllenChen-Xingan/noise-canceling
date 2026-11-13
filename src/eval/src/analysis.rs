//! 分析积木：P.56 主动电平、分帧能量、真值掩码、频谱（用 `vdn-dsp` 的 STFT，和实时同一份代码）、统计量。

use vdn_dsp::StreamingStft;

use crate::db10;
use crate::synth::Truth;

/// 掩码的帧长：10 ms @16 kHz。
pub const MASK_FRAME: usize = 160;

/// P.56 方法 B 的主动语音电平（自写，按标准描述：T = 0.03 s 的两级包络、H = 0.2 s 拖尾、M = 15.9 dB 余量、
/// 2 倍步进的门限阶梯，两级之间线性插值）。返回 (电平 dBov, 活动比例)；全零或太短时返回 `None`。
/// 0 dBov = 满幅（幅度 1.0）的均方。
pub fn active_level(x: &[f32], sr: u32) -> Option<(f64, f64)> {
    const NJ: usize = 16;
    const M: f64 = 15.9;
    if x.is_empty() {
        return None;
    }
    let sr = sr as f64;
    let g = (-1.0 / (sr * 0.03)).exp();
    let hang = (0.2 * sr).round() as usize;
    let c: Vec<f64> = (0..NJ).map(|j| 2f64.powi(j as i32 - 15)).collect();
    let mut a = [0usize; NJ];
    let mut h = [hang; NJ];
    let (mut p, mut q, mut sq) = (0.0f64, 0.0f64, 0.0f64);
    for &v in x {
        let v = v as f64;
        sq += v * v;
        p = g * p + (1.0 - g) * v.abs();
        q = g * q + (1.0 - g) * p;
        for j in 0..NJ {
            if q >= c[j] {
                a[j] += 1;
                h[j] = 0;
            } else if h[j] < hang {
                a[j] += 1;
                h[j] += 1;
            }
        }
    }
    if sq <= 0.0 {
        return None;
    }
    let n = x.len() as f64;
    let mut prev: Option<(f64, f64, f64)> = None; // (A_j, δ_j, a_j)
    for j in 0..NJ {
        if a[j] == 0 {
            break;
        }
        let aj = db10(sq / a[j] as f64);
        let cj = 20.0 * c[j].log10();
        let d = aj - cj;
        if d <= M {
            return Some(match prev {
                None => (aj, a[j] as f64 / n),
                Some((ap, dp, actp)) => {
                    // δ 随 j 单调下降；在 δ = M 处线性插值
                    let t = if (dp - d).abs() < 1e-12 {
                        0.0
                    } else {
                        (dp - M) / (dp - d)
                    };
                    (ap + t * (aj - ap), (actp + t * (a[j] as f64 - actp)) / n)
                }
            });
        }
        prev = Some((aj, d, a[j] as f64));
    }
    prev.map(|(ap, _, act)| (ap, act / n))
}

/// 只要电平（找不到时为 −∞）。
pub fn active_level_db(x: &[f32], sr: u32) -> f64 {
    active_level(x, sr).map_or(f64::NEG_INFINITY, |v| v.0)
}

/// 分帧均方（不重叠）。最后不满一帧的丢掉。
pub fn frame_power(x: &[f32], flen: usize) -> Vec<f64> {
    x.chunks_exact(flen)
        .map(|c| c.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / flen as f64)
        .collect()
}

/// 帧级掩码（10 ms）。`guard` 帧数向两边扩，避开分析窗的边缘效应。
#[derive(Clone, Debug)]
pub struct Masks {
    pub owner: Vec<bool>,
    pub interf: Vec<bool>,
    /// 主讲人与干扰者都不在（扩 guard 后），且混合信号不是数字静音
    pub noise_only: Vec<bool>,
    /// 只有干扰者（主讲人扩 guard 后也不在）
    pub interf_only: Vec<bool>,
}

fn paint(ivs: &[(usize, usize)], nf: usize, guard: usize) -> Vec<bool> {
    let mut m = vec![false; nf];
    for &(a, b) in ivs {
        let fa = (a / MASK_FRAME).saturating_sub(guard);
        let fb = (b.div_ceil(MASK_FRAME) + guard).min(nf);
        for v in m.iter_mut().take(fb).skip(fa) {
            *v = true;
        }
    }
    m
}

pub fn masks(truth: &Truth, mix: &[f32], guard_s: f64) -> Masks {
    let nf = mix.len() / MASK_FRAME;
    let guard = (guard_s * 1000.0 / 10.0).round() as usize;
    let owner = paint(&truth.owner, nf, 0);
    let interf = paint(&truth.interf, nf, 0);
    let og = paint(&truth.owner, nf, guard);
    let ig = paint(&truth.interf, nf, guard);
    let pw = frame_power(mix, MASK_FRAME);
    let noise_only = (0..nf).map(|f| !og[f] && !ig[f] && pw[f] > 1e-14).collect();
    let interf_only = (0..nf).map(|f| interf[f] && !og[f]).collect();
    Masks {
        owner,
        interf,
        noise_only,
        interf_only,
    }
}

/// 没有逐段真值的真实片段：从干净参考按 012 V-ONSET 的口径（20 ms 帧、比最大帧低 35 dB 以内算有声）求区间。
pub fn truth_from_clean(clean: &[f32]) -> Vec<(usize, usize)> {
    const F: usize = 320;
    let p = frame_power(clean, F);
    let max = p.iter().cloned().fold(0.0f64, f64::max);
    if max <= 0.0 {
        return Vec::new();
    }
    let thr = max * 10f64.powf(-35.0 / 10.0);
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, &v) in p.iter().enumerate() {
        match (v > thr, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                out.push((s * F, i * F));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push((s * F, p.len() * F));
    }
    out
}

/// 幅度平方谱，逐帧（帧 k 覆盖样本 `[(k+1)·hop − n_fft, (k+1)·hop)`，中心约在 `(k+1)·hop − n_fft/2`）。
/// 用 `vdn_dsp::StreamingStft`，窗 = `Window::SqrtHannPeriodicSquared`（sqrt-Hann 转 f32 后再平方，约等于周期 Hann）。
pub struct Spectrogram {
    pub n_fft: usize,
    pub hop: usize,
    pub frames: Vec<Vec<f32>>,
}

impl Spectrogram {
    pub fn new(x: &[f32], n_fft: usize, hop: usize) -> Spectrogram {
        let win = vdn_dsp::window::table(vdn_types::Window::SqrtHannPeriodicSquared, n_fft);
        let mut st = StreamingStft::new(n_fft, hop, win);
        let frames = x
            .chunks_exact(hop)
            .map(|c| st.analyze(c).iter().map(|z| z.norm_sqr()).collect())
            .collect();
        Spectrogram { n_fft, hop, frames }
    }

    /// 帧 k 的中心样本。
    pub fn center(&self, k: usize) -> isize {
        ((k + 1) * self.hop) as isize - (self.n_fft / 2) as isize
    }

    /// 帧 k 中心落在哪个 10 ms 掩码帧（落在开头之前时为 `None`）。
    pub fn mask_frame(&self, k: usize) -> Option<usize> {
        let c = self.center(k);
        (c >= 0).then_some(c as usize / MASK_FRAME)
    }

    pub fn bin_hz(&self, b: usize, sr: u32) -> f64 {
        b as f64 * sr as f64 / self.n_fft as f64
    }
}

pub fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// 峰度 E[(x−μ)⁴] / σ⁴（非超额峰度，高斯为 3）。
pub fn kurtosis(v: &[f64]) -> f64 {
    let m = mean(v);
    let (mut m2, mut m4) = (0.0, 0.0);
    for &x in v {
        let d = x - m;
        m2 += d * d;
        m4 += d * d * d * d;
    }
    let n = v.len().max(1) as f64;
    let (m2, m4) = (m2 / n, m4 / n);
    if m2 <= 0.0 { f64::NAN } else { m4 / (m2 * m2) }
}

/// 分位（线性插值，`q` ∈ [0, 1]）。
pub fn quantile(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    let mut s: Vec<f64> = v.iter().copied().filter(|x| !x.is_nan()).collect();
    if s.is_empty() {
        return f64::NAN;
    }
    s.sort_by(|a, b| a.total_cmp(b));
    let pos = q.clamp(0.0, 1.0) * (s.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
}

/// `guide` 里能量最大的 `win` 样本窗的起点（步长 100 ms）。延迟测量把窗放在人声最多处，
/// 照 TS 26.132 §7.10.4.2「窗心对准句子」（#109：固定在开头时，人声晚出现的段会量错）。
pub fn busiest_start(guide: &[f32], win: usize, step: usize) -> usize {
    if guide.len() <= win {
        return 0;
    }
    let e: Vec<f64> = guide.iter().map(|v| (*v as f64).powi(2)).collect();
    let mut acc: f64 = e[..win].iter().sum();
    let (mut best, mut at) = (acc, 0);
    let mut s = 0;
    while s + step + win <= e.len() {
        acc += e[s + win..s + win + step].iter().sum::<f64>() - e[s..s + step].iter().sum::<f64>();
        s += step;
        if acc > best {
            (best, at) = (acc, s);
        }
    }
    at
}

/// `y` 相对 `x` 的延迟（样本）：在 `0..=max_lag` 里找**归一化**互相关 |r| 的峰（#109）。
/// 窗取 `x[start..start + win]`，每个滞后除以两窗能量的几何平均，所以窗内能量随滞后变大不会抬高次峰。
/// 窗口太靠后、放不下最大滞后时往前挪；整段比 `win + max_lag` 还短时窗缩到 `|y| − max_lag`。
/// 输入窗全零、或所有滞后的输出窗都全零时返回 `None`（不另设相关下限：「量不准改判不知道」要人批准，025 ①-1）。
pub fn xcorr_lag(x: &[f32], y: &[f32], max_lag: usize, start: usize, win: usize) -> Option<usize> {
    // 窗口给滞后搜索留出尾部空间：start + n + max_lag ≤ |y|；放不下时先把窗往前挪，再缩短窗（codex 中途 1 第 2 轮 P1 #1）
    let n = win.min(x.len()).min(y.len().saturating_sub(max_lag));
    let start = start
        .min(x.len() - n)
        .min(y.len().saturating_sub(max_lag + n));
    let xs = &x[start..start + n];
    let ex: f64 = xs.iter().map(|a| (*a as f64).powi(2)).sum();
    if n == 0 || ex <= 0.0 {
        return None;
    }
    let mut best: Option<(usize, f64)> = None;
    for lag in 0..=max_lag {
        let b = start + lag;
        if b + n > y.len() {
            break;
        }
        let ys = &y[b..b + n];
        let ey: f64 = ys.iter().map(|v| (*v as f64).powi(2)).sum();
        if ey <= 0.0 {
            continue;
        }
        let dot: f64 = xs.iter().zip(ys).map(|(a, v)| *a as f64 * *v as f64).sum();
        let r = (dot / (ex * ey).sqrt()).abs();
        if best.is_none_or(|(_, br)| r > br) {
            best = Some((lag, r));
        }
    }
    best.and_then(|(l, r)| (r > 0.0).then_some(l))
}

/// 35 ms 时间常数（P.1120 §11.14.5）的平滑功率，逐 10 ms 掩码帧取帧末值，单位 dB。
pub fn smoothed_level_db(x: &[f32], sr: u32, tau_s: f64) -> Vec<f64> {
    let a = 1.0 - (-1.0 / (tau_s * sr as f64)).exp();
    let mut e = 0.0f64;
    let mut out = Vec::with_capacity(x.len() / MASK_FRAME);
    for c in x.as_chunks::<MASK_FRAME>().0 {
        for &v in c {
            e += a * ((v as f64).powi(2) - e);
        }
        out.push(db10(e));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p56_of_continuous_sine_is_its_rms() {
        // 连续正弦：活动 100%，主动电平 = RMS 电平 = 20log10(A/√2)
        let a = 0.1f32;
        let x: Vec<f32> = (0..32000)
            .map(|i| {
                a * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / vdn_types::MODEL_RATE as f32)
                    .sin()
            })
            .collect();
        let (lvl, act) = active_level(&x, vdn_types::MODEL_RATE).unwrap();
        let want = 20.0 * (a as f64 / 2f64.sqrt()).log10();
        assert!((lvl - want).abs() < 0.3, "{lvl} vs {want}");
        assert!(act > 0.95, "{act}");
    }

    #[test]
    fn p56_ignores_silence_half() {
        // 一半正弦一半静音：主动电平不变（≈ 连续时），活动约一半
        let a = 0.1f32;
        let mut x: Vec<f32> = (0..32000)
            .map(|i| {
                a * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / vdn_types::MODEL_RATE as f32)
                    .sin()
            })
            .collect();
        x.extend(std::iter::repeat_n(0.0, 32000));
        let (lvl, act) = active_level(&x, vdn_types::MODEL_RATE).unwrap();
        let want = 20.0 * (a as f64 / 2f64.sqrt()).log10();
        assert!((lvl - want).abs() < 0.6, "{lvl} vs {want}");
        assert!((0.45..0.62).contains(&act), "{act}");
    }

    #[test]
    fn xcorr_finds_delay() {
        let mut r = crate::rng::Rng::new(3);
        let x: Vec<f32> = (0..8000).map(|_| r.normal() as f32).collect();
        let mut y = vec![0.0f32; 300];
        y.extend_from_slice(&x);
        assert_eq!(xcorr_lag(&x, &y, 600, 0, 4000), Some(300));
    }

    /// #109 缺陷样例：前 1.9 s 只有底噪，之后接一段渐强的浊音（基音周期正好 106 样本）。
    /// 输出 = 输入（原声直通），延迟必须量出 0。旧公式只看开头 2 s、不归一化，会被基音次峰抬到 106。
    fn late_voiced_onset() -> Vec<f32> {
        let sr = vdn_types::MODEL_RATE as usize;
        let mut r = crate::rng::Rng::new(7);
        let n = sr * 6;
        let onset = sr * 19 / 10;
        (0..n)
            .map(|i| {
                let floor = 1e-4 * r.normal() as f32;
                if i < onset {
                    return floor;
                }
                // 渐强 0.3 s 后持平；脉冲串 + 两个谐波，周期 106 样本
                let env = ((i - onset) as f32 / (0.3 * sr as f32)).min(1.0);
                let ph = (i % 106) as f32 / 106.0 * std::f32::consts::TAU;
                floor + 0.2 * env * (ph.sin() + 0.5 * (2.0 * ph).sin() + 0.3 * (3.0 * ph).sin())
            })
            .collect()
    }

    #[test]
    fn passthrough_with_late_voiced_onset_measures_zero() {
        let x = late_voiced_onset();
        let sr = vdn_types::MODEL_RATE as usize;
        let start = busiest_start(&x, 4 * sr, sr / 10);
        assert!(start >= sr * 19 / 10, "窗应落在人声段：{start}");
        assert_eq!(xcorr_lag(&x, &x, 2048, start, 4 * sr), Some(0));
        // 旧窗口（开头 2 s）在归一化后也必须量对：归一化本身就能压住次峰
        assert_eq!(xcorr_lag(&x, &x, 2048, 0, 2 * sr), Some(0));
    }

    /// 同一信号人为延后 300 样本：窗放在人声段，量出 300。
    #[test]
    fn late_voiced_onset_with_known_delay() {
        let x = late_voiced_onset();
        let sr = vdn_types::MODEL_RATE as usize;
        let mut y = vec![0.0f32; 300];
        y.extend_from_slice(&x);
        let start = busiest_start(&x, 4 * sr, sr / 10);
        assert_eq!(xcorr_lag(&x, &y, 2048, start, 4 * sr), Some(300));
    }

    #[test]
    fn xcorr_silent_is_none() {
        let z = vec![0.0f32; 8000];
        assert_eq!(xcorr_lag(&z, &z, 100, 0, 4000), None);
    }

    /// 输出与输入等长（延后 300 样本、尾部截掉）：窗口必须给滞后搜索留出尾部空间（codex 中途 1 第 2 轮 P1 #1）。
    #[test]
    fn equal_length_delayed_output_short_and_loud_tail() {
        let sr = vdn_types::MODEL_RATE as usize;
        let delay = |x: &[f32]| -> Vec<f32> {
            let mut y = vec![0.0f32; 300];
            y.extend_from_slice(&x[..x.len() - 300]);
            y
        };
        // 3 秒：4 秒窗盖住整段
        let mut r = crate::rng::Rng::new(5);
        let x: Vec<f32> = (0..3 * sr).map(|_| r.normal() as f32).collect();
        assert_eq!(xcorr_lag(&x, &delay(&x), 2048, 0, 4 * sr), Some(300));
        // 8 秒、最响的在末尾：选到的窗贴着末尾
        let x: Vec<f32> = (0..8 * sr)
            .map(|i| r.normal() as f32 * if i > 6 * sr { 1.0 } else { 0.05 })
            .collect();
        let start = busiest_start(&x, 4 * sr, sr / 10);
        assert_eq!(start, 4 * sr);
        assert_eq!(xcorr_lag(&x, &delay(&x), 2048, start, 4 * sr), Some(300));
    }

    /// 低相关但非零的输出（与输入几乎无关的噪声）：照旧给出一个滞后，不改判「不知道」——
    /// 那是要人批准的判定改动（025 ①-1），这里只修量法（codex 中途 1 第 1 轮 P1 #3）。
    #[test]
    fn low_correlation_still_measures() {
        let mut r = crate::rng::Rng::new(9);
        let x: Vec<f32> = (0..8000).map(|_| r.normal() as f32).collect();
        let y: Vec<f32> = (0..8000).map(|_| r.normal() as f32).collect();
        assert!(xcorr_lag(&x, &y, 100, 0, 4000).is_some());
    }

    #[test]
    fn kurtosis_of_gaussian_is_three() {
        let mut r = crate::rng::Rng::new(5);
        let v: Vec<f64> = (0..200_000).map(|_| r.normal()).collect();
        assert!((kurtosis(&v) - 3.0).abs() < 0.1);
    }
}
