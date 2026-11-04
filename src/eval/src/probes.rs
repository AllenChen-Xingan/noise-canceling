//! 防作弊探针（rubric 第 7 节「rubric 自己的单元测试」· 012 §2.3 第 2 条）：对一份正常输出人为劣化，
//! 对应的 verifier **必须**判它不过、且比原输出差。任何一条不成立 = verifier 坏了，CI 红。
//!
//! 另外两种「刷分」探针（Anthropic 文章「防评分器被钻空子」）：什么都不做（输出 = 输入），全部静音（输出 = 0）。
//! 它们证明单看 V-NR 会被刷（全静音 V-NR 满分），必须有制衡项（V-SPLVL）把它拦下。

use crate::analysis::truth_from_clean;
use crate::synth::Clip;

/// 清单里的名字由 serde 定；`as_str` 与它的一致性由本文件的测试核对。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Probe {
    /// 削每个词首 50 ms（置零）
    #[serde(rename = "onset_cut_50ms")]
    OnsetCut,
    /// 非语音段 −40 dB
    #[serde(rename = "nonspeech_minus40")]
    NonSpeechMinus40,
    /// 1 kHz 低通（四阶：两个二阶巴特沃斯级联）
    #[serde(rename = "lowpass_1k")]
    Lowpass1k,
    /// 什么都不做：输出 = 带噪输入（exp005「所有的都没降噪」）
    Identity,
    /// 全部静音：输出 = 0（刷 V-NR 的极端做法）
    Mute,
    /// 弱音节（有声 125 ms 段里干净轨能量最低的 30%）压 10 dB：V-WEAK 必须抓到
    #[serde(rename = "weak_minus10")]
    WeakMinus10,
    /// 整体乘 2 Hz、±6 dB 的增益抖动：V-PUMPV 必须抓到
    #[serde(rename = "gain_wobble_6db")]
    GainWobble,
    /// 在主讲人有声段里均匀插 5 个单样本脉冲（幅度 0.5）：V-CLICK 必须抓到
    #[serde(rename = "clicks_5")]
    Clicks5,
    /// 加噪到 0 dB 信噪比（相对干净轨，用本段自己的噪声轨）：输出比输入还吵，V-ESTOI 必须抓到
    #[serde(rename = "noise_0db")]
    Noise0db,
}

impl Probe {
    pub const ALL: [Probe; 9] = [
        Probe::OnsetCut,
        Probe::NonSpeechMinus40,
        Probe::Lowpass1k,
        Probe::Identity,
        Probe::Mute,
        Probe::WeakMinus10,
        Probe::GainWobble,
        Probe::Clicks5,
        Probe::Noise0db,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Probe::OnsetCut => "onset_cut_50ms",
            Probe::NonSpeechMinus40 => "nonspeech_minus40",
            Probe::Lowpass1k => "lowpass_1k",
            Probe::Identity => "identity",
            Probe::Mute => "mute",
            Probe::WeakMinus10 => "weak_minus10",
            Probe::GainWobble => "gain_wobble_6db",
            Probe::Clicks5 => "clicks_5",
            Probe::Noise0db => "noise_0db",
        }
    }

    /// 把 `out`（系统的正常输出）劣化。
    pub fn apply(self, clip: &Clip, out: &[f32]) -> Vec<f32> {
        match self {
            Probe::OnsetCut => {
                let mut y = out.to_vec();
                let w = (0.05 * clip.sr as f64) as usize;
                let mut last_end: Option<usize> = None;
                for (a, b) in truth_from_clean(&clip.clean) {
                    if last_end.is_none_or(|e| a >= e + 1600) {
                        for v in y.iter_mut().skip(a).take(w) {
                            *v = 0.0;
                        }
                    }
                    last_end = Some(b);
                }
                y
            }
            Probe::NonSpeechMinus40 => {
                let mut y = out.to_vec();
                // 「非语音段」= 干净参考按 20 ms 帧判无声的部分
                let mut speech = vec![false; y.len()];
                for (a, b) in truth_from_clean(&clip.clean) {
                    for v in speech.iter_mut().take(b.min(y.len())).skip(a) {
                        *v = true;
                    }
                }
                for (v, s) in y.iter_mut().zip(&speech) {
                    if !s {
                        *v *= 0.01;
                    }
                }
                y
            }
            Probe::Lowpass1k => {
                let sr = clip.sr as f64;
                let mut y = out.to_vec();
                for q in [0.5412, 1.3066] {
                    biquad_lowpass(&mut y, 1000.0, q, sr);
                }
                y
            }
            Probe::Identity => clip.mix.clone(),
            Probe::Mute => vec![0.0; out.len()],
            Probe::WeakMinus10 => weak_minus10(clip, out),
            Probe::Noise0db => {
                let rms = |x: &[f32]| {
                    (x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len().max(1) as f64)
                        .sqrt()
                };
                let g = (rms(&clip.clean) / rms(&clip.noise).max(1e-12)) as f32;
                out.iter()
                    .zip(&clip.noise)
                    .map(|(o, n)| o + g * n)
                    .collect()
            }
            Probe::GainWobble => {
                let sr = clip.sr as f64;
                out.iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let g = 6.0 * (2.0 * std::f64::consts::PI * 2.0 * i as f64 / sr).sin();
                        v * 10f64.powf(g / 20.0) as f32
                    })
                    .collect()
            }
            Probe::Clicks5 => {
                let mut y = out.to_vec();
                let ivs = truth_from_clean(&clip.clean);
                let total: usize = ivs.iter().map(|(a, b)| b - a).sum();
                for k in 0..5 {
                    // 第 k 个点落在全部有声样本的 (k + 0.5) / 5 处
                    let mut want = (total as f64 * (k as f64 + 0.5) / 5.0) as usize;
                    for &(a, b) in &ivs {
                        if want < b - a {
                            if let Some(v) = y.get_mut(a + want) {
                                *v += 0.5;
                            }
                            break;
                        }
                        want -= b - a;
                    }
                }
                y
            }
        }
    }
}

/// 有声段切成 125 ms 一块（与 V-WEAK 的分析窗同长），干净轨能量最低的 30% 块，输出压 10 dB。
fn weak_minus10(clip: &Clip, out: &[f32]) -> Vec<f32> {
    let f = (0.125 * clip.sr as f64) as usize;
    let mut y = out.to_vec();
    let mut blocks: Vec<(usize, f64)> = Vec::new();
    for (a, b) in truth_from_clean(&clip.clean) {
        let mut t = a;
        while t + f <= b.min(y.len()) {
            let e: f64 = clip.clean[t..t + f]
                .iter()
                .map(|v| (*v as f64).powi(2))
                .sum();
            blocks.push((t, e));
            t += f;
        }
    }
    blocks.sort_by(|p, q| p.1.total_cmp(&q.1));
    let n = blocks.len() * 3 / 10;
    for &(t, _) in &blocks[..n] {
        for v in &mut y[t..t + f] {
            *v *= 0.316;
        }
    }
    y
}

/// RBJ cookbook 二阶低通，原地。
fn biquad_lowpass(x: &mut [f32], f: f64, q: f64, sr: f64) {
    let w = 2.0 * std::f64::consts::PI * f / sr;
    let alpha = w.sin() / (2.0 * q);
    let cw = w.cos();
    let a0 = 1.0 + alpha;
    let b0 = (1.0 - cw) / 2.0 / a0;
    let b1 = (1.0 - cw) / a0;
    let b2 = b0;
    let a1 = -2.0 * cw / a0;
    let a2 = (1.0 - alpha) / a0;
    let (mut x1, mut x2, mut y1, mut y2) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for v in x.iter_mut() {
        let x0 = *v as f64;
        let y0 = b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
        x2 = x1;
        x1 = x0;
        y2 = y1;
        y1 = y0;
        *v = y0 as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowpass_kills_4k() {
        let sr = f64::from(vdn_types::MODEL_RATE);
        let mut x: Vec<f32> = (0..vdn_types::MODEL_RATE as usize)
            .map(|i| (2.0 * std::f64::consts::PI * 4000.0 * i as f64 / sr).sin() as f32)
            .collect();
        for q in [0.5412, 1.3066] {
            biquad_lowpass(&mut x, 1000.0, q, sr);
        }
        let p = x[8000..].iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / 8000.0;
        assert!(10.0 * p.log10() < -40.0);
    }
}
