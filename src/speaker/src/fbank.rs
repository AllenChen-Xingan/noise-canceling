//! Kaldi 兼容的 log-mel fbank（自写；对拍基准 = `kaldi-native-fbank`，与 torchaudio `compliance.kaldi.fbank` 同算法）。
//!
//! 只实现声纹前端需要的那一组选项：`snip_edges = true`、去直流、预加重 0.97、`use_energy = false`、
//! `use_power = true`、`dither = 0`、帧长补到 2 的幂做 FFT、mel 用 Kaldi 的 `1127·ln(1 + f/700)`。
//! 窗型和输入缩放可配：WeSpeaker = hamming + 输入 ×32768；3D-Speaker CAM++ = povey + 输入保持 [-1, 1]（009 §5）。
//!
//! 一个 [`Fbank`] 自带分帧 FFT 和工作区，`compute_into` 不分配（声纹在非实时线程，但每 0.5 s 一次，没必要每次分配）。
//!
//! 频谱那一段（分帧 → 加窗 → 补零 → FFT → 功率）不在这里写：调统一模块 `vdn_dsp::FrameFft` 与 `vdn_dsp::power_into`
//!（D82「同一处代码，各自切法」）。这里只留 Kaldi 特有的每帧预处理（去直流 · 预加重 · 输入缩放）和 mel 滤波器组。
//! 窗种类用正本 `vdn_types::Window`（`Kaldi*` 三种）。

#![allow(clippy::indexing_slicing)] // NRT: 判断回路（C 档线程，约 0.5 s 一次），不在降噪线程上跑

use vdn_dsp::{FrameFft, power_into};
pub use vdn_types::Window;
use vdn_types::{StftSpec, Window as W};

#[derive(Clone, Copy, Debug)]
pub struct FbankConfig {
    pub sample_rate: f32,
    pub frame_length: usize,
    pub frame_shift: usize,
    pub num_mel_bins: usize,
    pub low_freq: f32,
    /// ≤ 0 时表示「奈奎斯特 + high_freq」（Kaldi 约定）
    pub high_freq: f32,
    pub preemph: f32,
    /// Kaldi 窗：`KaldiHamming` / `KaldiPovey` / `KaldiHanning`（对称，分母 n−1）
    pub window: Window,
    /// 计算前把输入乘上它（WeSpeaker 训练时用 16 bit 整数幅度，即 32768）
    pub input_scale: f32,
}

impl FbankConfig {
    /// 频谱口径：窗长 `frame_length`、补零到 2 的幂、帧移 `frame_shift`。
    pub fn spec(&self) -> StftSpec {
        StftSpec {
            n_fft: self.frame_length.next_power_of_two(),
            win_len: self.frame_length,
            hop: self.frame_shift,
            sample_rate: self.sample_rate as u32,
            window: self.window,
        }
    }

    /// WeSpeaker（ECAPA / ResNet / CAM++ 的 wespeaker 版）前端：`resources/wespeaker/wespeaker/bin/infer_onnx.py:31-50`。
    pub fn wespeaker() -> Self {
        Self {
            sample_rate: vdn_types::MODEL_RATE as f32,
            frame_length: 400,
            frame_shift: 160,
            num_mel_bins: 80,
            low_freq: 20.0,
            high_freq: 0.0,
            preemph: 0.97,
            window: W::KaldiHamming,
            input_scale: 32768.0,
        }
    }
}

pub struct Fbank {
    cfg: FbankConfig,
    /// 每个 mel 带：(起始 FFT bin, 权重)
    banks: Vec<(usize, Vec<f32>)>,
    fft: FrameFft<f32>,
    /// 本帧功率谱（bins 个）
    power: Vec<f32>,
}

fn mel(f: f64) -> f64 {
    1127.0 * (1.0 + f / 700.0).ln()
}

impl Fbank {
    pub fn new(cfg: FbankConfig) -> Self {
        let fft = FrameFft::<f32>::from_spec(&cfg.spec());
        let n_fft = fft.n_fft();
        // mel 滤波器组：Kaldi `MelBanks`（只用前 n_fft/2 个 bin，奈奎斯特 bin 权重为 0）
        let nyq = 0.5 * cfg.sample_rate as f64;
        let high = if cfg.high_freq <= 0.0 {
            nyq + cfg.high_freq as f64
        } else {
            cfg.high_freq as f64
        };
        let (mlo, mhi) = (mel(cfg.low_freq as f64), mel(high));
        let delta = (mhi - mlo) / (cfg.num_mel_bins as f64 + 1.0);
        let bin_w = cfg.sample_rate as f64 / n_fft as f64;
        let banks = (0..cfg.num_mel_bins)
            .map(|m| {
                let l = mlo + m as f64 * delta;
                let c = l + delta;
                let r = c + delta;
                let mut start = usize::MAX;
                let mut w = Vec::new();
                for k in 0..n_fft / 2 {
                    let me = mel(bin_w * k as f64);
                    if me > l && me < r {
                        let v = if me <= c {
                            (me - l) / (c - l)
                        } else {
                            (r - me) / (r - c)
                        };
                        if start == usize::MAX {
                            start = k;
                        }
                        // 中间不会断开（三角形连续），按序推入
                        w.resize(k - start, 0.0);
                        w.push(v as f32);
                    }
                }
                (start.min(n_fft / 2), w)
            })
            .collect();
        Self {
            cfg,
            banks,
            power: vec![0.0; fft.bins()],
            fft,
        }
    }

    pub fn config(&self) -> &FbankConfig {
        &self.cfg
    }

    /// `snip_edges = true` 时的帧数。
    pub fn num_frames(&self, n_samples: usize) -> usize {
        self.fft.num_frames(n_samples)
    }

    /// 把 `x` 的 fbank 写进 `out`（行主序，`num_frames × num_mel_bins`），返回帧数。`out` 不够长时只写能放下的帧。
    pub fn compute_into(&mut self, x: &[f32], out: &mut [f32]) -> usize {
        let nb = self.cfg.num_mel_bins;
        let nf = self.num_frames(x.len()).min(out.len() / nb);
        let (scale, pe) = (self.cfg.input_scale, self.cfg.preemph);
        for f in 0..nf {
            // Kaldi 每帧预处理（统一模块在加窗之前调它；拿到的是这一帧原样本的拷贝）
            let pre = |fr: &mut [f32]| {
                let len = fr.len();
                // 去直流用 f64 累加（009 §2：安静频带的数值误差）
                let mean = (fr.iter().map(|&v| v as f64).sum::<f64>() / len as f64) as f32 * scale;
                for d in fr.iter_mut() {
                    *d = *d * scale - mean;
                }
                // 预加重：从后往前，第 0 个样本用自己（Kaldi 约定）
                if pe != 0.0 {
                    for j in (1..len).rev() {
                        fr[j] -= pe * fr[j - 1];
                    }
                    fr[0] -= pe * fr[0];
                }
            };
            // 帧号 < num_frames，不会越界
            let Some(spec) = self.fft.frame_with(x, f, pre) else {
                return f;
            };
            power_into(spec, &mut self.power);
            let row = &mut out[f * nb..(f + 1) * nb];
            for (o, (start, w)) in row.iter_mut().zip(&self.banks) {
                let mut e = 0.0f32;
                for (k, &wk) in w.iter().enumerate() {
                    e += self.power[start + k] * wk;
                }
                *o = e.max(f32::EPSILON).ln();
            }
        }
        nf
    }
}

/// 逐列减均值（CMN，不做 CVN）：WeSpeaker 在每个声纹窗内做（`infer_onnx.py:47-48`）。
pub fn cmn(feats: &mut [f32], num_frames: usize, num_bins: usize) {
    if num_frames == 0 {
        return;
    }
    for b in 0..num_bins {
        let mut s = 0.0f64;
        for f in 0..num_frames {
            s += feats[f * num_bins + b] as f64;
        }
        let m = (s / num_frames as f64) as f32;
        for f in 0..num_frames {
            feats[f * num_bins + b] -= m;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_count_snip_edges() {
        let fb = Fbank::new(FbankConfig::wespeaker());
        assert_eq!(fb.num_frames(399), 0);
        assert_eq!(fb.num_frames(400), 1);
        assert_eq!(fb.num_frames(vdn_types::MODEL_RATE as usize), 98);
    }

    #[test]
    fn tone_peaks_in_right_band() {
        let mut fb = Fbank::new(FbankConfig::wespeaker());
        let x: Vec<f32> = (0..1600)
            .map(|i| {
                0.1 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32
                    / vdn_types::MODEL_RATE as f32)
                    .sin()
            })
            .collect();
        let nf = fb.num_frames(x.len());
        let mut out = vec![0.0; nf * 80];
        fb.compute_into(&x, &mut out);
        let row = &out[..80];
        let argmax = row
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap_or(0);
        // 1 kHz 的 mel ≈ 1000；20–8000 Hz 分 81 段，每段约 (2840−31)/81 ≈ 34.7 mel，落在第 ~27 个带附近
        assert!((24..=30).contains(&argmax), "argmax={argmax}");
    }

    #[test]
    fn every_bank_nonempty() {
        let fb = Fbank::new(FbankConfig::wespeaker());
        for (i, (_, w)) in fb.banks.iter().enumerate() {
            assert!(!w.is_empty(), "第 {i} 个带是空的");
        }
    }
}
