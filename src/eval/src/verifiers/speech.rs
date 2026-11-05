//! 人声细节 verifier（012 §3.1 · §3.2，定标前一律 L0）：轻声字被吞 V-WEAK · 人声忽大忽小 V-PUMPV ·
//! 爆音 V-CLICK · 可懂度 V-ESTOI。都要干净参考（V-CLICK 要带噪输入），真实录音没有参考时判「不知道」。

use vdn_dsp::FrameFft;
use vdn_types::{StftSpec, Window};

use super::estoi::estoi;
use super::perceptual::proj_energy;
use super::*;

/// V-WEAK 的 1/3 倍频程带中心（500 Hz–4 kHz，IEC 61260 标称值）。
const WEAK_CF: [f64; 10] = [
    500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0,
];

/// 有声帧：帧中心落在主讲人掩码里。
fn owner_at(c: &Ctx, center: usize) -> bool {
    c.masks
        .owner
        .get(center / MASK_FRAME)
        .copied()
        .unwrap_or(false)
}

/// 每帧每带的电平（dB），125 ms 周期 Hann 窗（补零到 2 的幂）、50% 重叠，f64；只留有声帧。返回 [带][帧]。
/// 频谱由统一模块 `vdn_dsp::FrameFft` 算（D82）。
fn band_levels(c: &Ctx, x: &[f32]) -> Vec<Vec<f64>> {
    let sr = c.clip.sr as f64;
    let win = (0.125 * sr) as usize;
    let nfft = win.next_power_of_two();
    let mut fft = FrameFft::<f64>::from_spec(&StftSpec {
        n_fft: nfft,
        win_len: win,
        hop: win / 2,
        sample_rate: c.clip.sr,
        window: Window::HannPeriodic,
    });
    // 上沿越过奈奎斯特频率的带不计（#110：低采样率下切片越界）；切片再截到频谱长度兜底
    let bins = nfft / 2 + 1;
    let edges: Vec<(usize, usize)> = WEAK_CF
        .iter()
        .filter(|f| *f * 2f64.powf(1.0 / 6.0) <= sr / 2.0)
        .map(|f| {
            let lo = (f * 2f64.powf(-1.0 / 6.0) * nfft as f64 / sr).round() as usize;
            let hi = (f * 2f64.powf(1.0 / 6.0) * nfft as f64 / sr).round() as usize;
            (lo.min(bins - 1), hi.max(lo + 1).min(bins))
        })
        .collect();
    let mut out = vec![Vec::new(); edges.len()];
    let n = x.len().min(c.clip.clean.len());
    let x64: Vec<f64> = x[..n].iter().map(|&v| v as f64).collect();
    for k in 0..fft.num_frames(n) {
        if !owner_at(c, k * fft.hop() + win / 2) {
            continue;
        }
        let Some(spec) = fft.frame(&x64, k) else {
            break;
        };
        for (b, &(lo, hi)) in edges.iter().enumerate() {
            let p: f64 = spec[lo..hi].iter().map(|z| z.norm_sqr()).sum();
            out[b].push(db10(p));
        }
    }
    out
}

pub(super) fn v_weak(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-WEAK";
    if c.clip.clean.iter().all(|v| *v == 0.0) {
        return Score::unknown(ID, "dB", Better::Lower, "没有干净参考");
    }
    let lc = band_levels(c, &c.clip.clean);
    let lo = band_levels(c, c.out);
    if lc.len() * 2 < WEAK_CF.len() {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            format!(
                "采样率 {} Hz：奈奎斯特以下只剩 {}/{} 个频带，少于一半",
                c.clip.sr,
                lc.len(),
                WEAK_CF.len()
            ),
        );
    }
    let frames = lc.first().map_or(0, Vec::len);
    if frames < 20 {
        return Score::unknown(ID, "dB", Better::Lower, format!("有声帧只有 {frames} 个"));
    }
    let (mut l30, mut dr) = (Vec::new(), Vec::new());
    for (bc, bo) in lc.iter().zip(&lo) {
        let (c30, c99) = (quantile(bc, 0.30), quantile(bc, 0.99));
        let (o30, o99) = (quantile(bo, 0.30), quantile(bo, 0.99));
        l30.push(c30 - o30);
        dr.push((o99 - o30) - (c99 - c30));
    }
    let loss = mean(&l30);
    let ddr = mean(&dr);
    let mut s = Score::judged(
        ID,
        loss,
        "dB",
        Better::Lower,
        Some(p.f_or("max", th::WEAK.def("max"))),
        th::WEAK.edges,
    )
    .with_extra("delta_range_db", ddr)
    .with_extra("frames", frames as f64);
    // 只在有频带被奈奎斯特截掉时报带数，16 kHz 输出与改前逐位相同（spectrum_golden）
    if lc.len() < WEAK_CF.len() {
        s = s.with_extra("bands", lc.len() as f64);
    }
    if ddr < p.f_or("compress_min", th::WEAK.def("compress_min")) {
        s.note = format!("动态范围变化 {ddr:.1} dB：被压缩（012 V-WEAK）");
    }
    s
}

pub(super) fn v_pumpv(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-PUMPV";
    let w = (0.1 * c.clip.sr as f64) as usize;
    let n = c.out.len().min(c.clip.clean.len());
    let mut r = Vec::new();
    let mut s = 0;
    while s + w <= n {
        let voiced = (s..s + w)
            .step_by(MASK_FRAME)
            .filter(|&t| owner_at(c, t))
            .count();
        if voiced * MASK_FRAME * 10 >= w * 8 {
            let (po, pc) = proj_energy(&c.out[s..s + w], &c.clip.clean[s..s + w]);
            if pc > 1e-12 {
                r.push(db10(po) - db10(pc));
            }
        }
        s += w;
    }
    if r.len() < 10 {
        return Score::unknown(
            ID,
            "dB²",
            Better::Lower,
            format!("有声窗只有 {} 个", r.len()),
        );
    }
    let m = mean(&r);
    let var = r.iter().map(|v| (v - m).powi(2)).sum::<f64>() / r.len() as f64;
    Score::judged(
        ID,
        var,
        "dB²",
        Better::Lower,
        Some(p.f_or("max", th::PUMPV.def("max"))),
        &[],
    )
    .with_extra("windows", r.len() as f64)
    .with_extra("mean_db", m)
}

/// 一阶差分超过局部（20 ms）中位绝对偏差 k 倍的位置。
fn outliers(x: &[f32], sr: u32, k: f64) -> Vec<bool> {
    let d: Vec<f64> = x.windows(2).map(|w| (w[1] - w[0]).abs() as f64).collect();
    let blk = (0.02 * sr as f64) as usize;
    let mut flag = vec![false; d.len()];
    for (bi, chunk) in d.chunks(blk).enumerate() {
        let med = quantile(chunk, 0.5);
        let mad = quantile(
            &chunk.iter().map(|v| (v - med).abs()).collect::<Vec<_>>(),
            0.5,
        );
        let thr = med + k * mad.max(1e-6);
        for (i, v) in chunk.iter().enumerate() {
            flag[bi * blk + i] = *v > thr;
        }
    }
    flag
}

/// 残差：输出减去「随时间平滑变化的增益 × 干净轨」（干净轨全零时就是输出本身）。
/// 增益逐 5 ms 块最小二乘估，块中心之间线性插值：块要短（降噪器逐帧调增益，20 ms 块在 4 Hz ±30% 的平滑增益上
/// 误报 1746 个/分钟），又不能逐块阶跃（块边界的增益台阶本身会在残差里造出跳变）。
fn residual(x: &[f32], clean: &[f32], sr: u32) -> Vec<f32> {
    let blk = (0.005 * sr as f64) as usize;
    let n = x.len().min(clean.len());
    let nb = n / blk;
    if nb == 0 {
        return x.to_vec();
    }
    let g: Vec<f64> = (0..nb)
        .map(|b| {
            let (mut oc, mut cc) = (0.0f64, 0.0f64);
            for t in b * blk..(b + 1) * blk {
                oc += x[t] as f64 * clean[t] as f64;
                cc += clean[t] as f64 * clean[t] as f64;
            }
            if cc > 0.0 { oc / cc } else { 0.0 }
        })
        .collect();
    let center = |b: usize| (b * blk + blk / 2) as f64;
    let mut r = x.to_vec();
    for (t, v) in r.iter_mut().enumerate().take(n) {
        let pos = (t as f64 - blk as f64 / 2.0) / blk as f64;
        let b0 = pos.floor().clamp(0.0, (nb - 1) as f64) as usize;
        let b1 = (b0 + 1).min(nb - 1);
        let w = if b1 == b0 {
            0.0
        } else {
            ((t as f64 - center(b0)) / blk as f64).clamp(0.0, 1.0)
        };
        let gt = g[b0] * (1.0 - w) + g[b1] * w;
        *v -= (gt * clean[t] as f64) as f32;
    }
    r
}

/// 咔嗒的位置（样本号，5 ms 内的并成一个）。诊断与 [`v_click`] 共用。
/// 咔嗒 = 输出自身一阶差分的离群点（012 原定义），且同时满足：跳变听得见（不低于人声峰值以下 floor_db）；
/// 这一步跳变里干净人声解释不了的部分占 resid_frac 以上（人声自身的陡变、平滑的增益变化都由干净轨解释）；
/// 带噪输入（有分轨时还有背景轨）±2 ms 内没有离群点（原声自带的瞬态不算）。
pub fn click_positions(c: &Ctx, p: &Params) -> Vec<usize> {
    let k = p.f_or("k", th::CLICK.def("k"));
    let sr = c.clip.sr;
    let frac = p.f_or("resid_frac", th::CLICK.def("resid_frac"));
    let peak = quantile(
        &c.clip
            .clean
            .iter()
            .map(|v| v.abs() as f64)
            .collect::<Vec<_>>(),
        0.999,
    );
    let floor = (peak * 10f64.powf(p.f_or("floor_db", th::CLICK.def("floor_db")) / 20.0)) as f32;
    let r = residual(c.out, &c.clip.clean, sr);
    let fo = outliers(c.out, sr, k);
    // 原声自带的瞬态：带噪输入里的离群点；有分轨时再加上「背景轨（噪音 + 干扰）自身」的离群点——
    // 杯碟、开罐这类突发声在混合里被人声盖住、算不上离群点，降噪后露出来，那是原有的声音穿过来了，不是降噪造的咔嗒
    let mut fm = outliers(&c.clip.mix, sr, k);
    if c.clip.tracks {
        let bg: Vec<f32> = c
            .clip
            .noise
            .iter()
            .zip(&c.clip.interf)
            .map(|(a, b)| a + b)
            .collect();
        for (m, b) in fm.iter_mut().zip(outliers(&bg, sr, k)) {
            *m |= b;
        }
    }
    let guard = (0.002 * sr as f64) as usize;
    let mut pos = Vec::new();
    let mut last: Option<usize> = None;
    for (i, &f) in fo.iter().enumerate() {
        if !f || i + 1 >= r.len() {
            continue;
        }
        let d_out = (c.out[i + 1] - c.out[i]).abs();
        let d_res = (r[i + 1] - r[i]).abs();
        if d_out < floor || d_res < frac as f32 * d_out {
            continue;
        }
        let a = i.saturating_sub(guard);
        let b = (i + guard + 1).min(fm.len());
        if fm.get(a..b).is_some_and(|w| w.iter().any(|x| *x)) {
            continue;
        }
        if last.is_none_or(|l| i > l + 5 * guard / 2) {
            pos.push(i);
        }
        last = Some(i);
    }
    pos
}

pub(super) fn v_click(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-CLICK";
    let count = click_positions(c, p).len();
    let minutes = c.out.len() as f64 / c.clip.sr as f64 / 60.0;
    if minutes <= 0.0 {
        return Score::unknown(ID, "个/分钟", Better::Lower, "输出为空");
    }
    Score::judged(
        ID,
        count as f64 / minutes,
        "个/分钟",
        Better::Lower,
        Some(p.f_or("max", th::CLICK.def("max"))),
        &[],
    )
    .with_extra("count", count as f64)
}

pub(super) fn v_estoi(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-ESTOI";
    let sr = c.clip.sr;
    // 没有干净参考（全零）时两次 ESTOI 都是 0、Δ = 0 会「过线」：显式判不了（codex 第 1 轮 P1 #3）
    if c.clip.clean.iter().all(|v| *v == 0.0) {
        return Score::unknown(ID, "Δ", Better::Higher, "没有干净参考");
    }
    let (Some(o), Some(m)) = (
        estoi(&c.clip.clean, c.out, sr),
        estoi(&c.clip.clean, &c.clip.mix, sr),
    ) else {
        return Score::unknown(
            ID,
            "Δ",
            Better::Higher,
            "去掉静音帧后不足 30 帧（约 0.4 s）",
        );
    };
    Score::judged(
        ID,
        o - m,
        "Δ",
        Better::Higher,
        Some(p.f_or("min", th::ESTOI.def("min"))),
        th::ESTOI.edges,
    )
    .with_extra("estoi_out", o)
    .with_extra("estoi_mix", m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::masks;
    use crate::probes::Probe;
    use crate::synth::{NoiseKind, SceneSpec, TalkerSpec, render};

    fn clip() -> Clip {
        render(&SceneSpec {
            dur_s: 10.0,
            seed: 7,
            owner: Some(TalkerSpec {
                level_dbov: -26.0,
                f0_lo: 100.0,
                f0_hi: 140.0,
                tonal: false,
            }),
            owner_start_s: 0.5,
            pause_s: (0.4, 1.2),
            interferers: Vec::new(),
            noise: NoiseKind::Pink,
            snr_db: 10.0,
            tail_silence_s: 0.0,
        })
    }

    /// 「好输出」：干净轨 + 低 30 dB 的残留噪声（像一个合格的降噪器）。
    fn good(c: &Clip) -> Vec<f32> {
        c.clean
            .iter()
            .zip(&c.noise)
            .map(|(s, n)| s + 0.03 * n)
            .collect()
    }

    fn score(id: &str, c: &Clip, out: &[f32]) -> Score {
        let m = masks(&c.truth, &c.mix, 0.03);
        let ctx = Ctx {
            clip: c,
            out,
            run: None,
            masks: &m,
            noise_out: None,
            ci: true,
        };
        eval(id, &ctx, &Params::default())
    }

    /// 好输出过、探针劣化后不过且更差（012 §2.3 第 2 条）。
    fn must_catch(id: &str, probe: Probe) {
        let c = clip();
        let g = good(&c);
        let a = score(id, &c, &g);
        assert!(a.verdict.is_pass(), "{id} 好输出应过：{a} {}", a.note);
        let bad = probe.apply(&c, &g);
        let b = score(id, &c, &bad);
        assert!(
            b.verdict.is_fail(),
            "{id} 探针 {} 应不过：{b}",
            probe.as_str()
        );
        let (va, vb) = (a.value.unwrap(), b.value.unwrap());
        assert!(a.better.worse(vb, va), "{id}：探针 {vb} 不比原输出 {va} 差");
    }

    #[test]
    fn weak_catches_weak_drop() {
        must_catch("V-WEAK", Probe::WeakDrop40);
    }

    #[test]
    fn pumpv_catches_gain_wobble() {
        must_catch("V-PUMPV", Probe::GainWobble);
    }

    #[test]
    fn click_catches_clicks() {
        must_catch("V-CLICK", Probe::Clicks5);
    }

    #[test]
    fn estoi_catches_added_noise() {
        must_catch("V-ESTOI", Probe::Noise0db);
    }

    /// 负例：平滑的增益变化（4 Hz、±30%，像降噪器逐帧调增益）不是咔嗒，必须报 0。
    #[test]
    fn smooth_gain_change_is_not_a_click() {
        let c = clip();
        let sr = c.sr as f64;
        let out: Vec<f32> = good(&c)
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let g = 1.0 + 0.3 * (2.0 * std::f64::consts::PI * 4.0 * i as f64 / sr).sin();
                v * g as f32
            })
            .collect();
        let s = score("V-CLICK", &c, &out);
        assert_eq!(s.value, Some(0.0), "{s}");
    }

    /// 负例：门控式的增益台阶，但用 10 ms 斜坡过渡（平滑），也不是咔嗒。
    #[test]
    fn ramped_gate_steps_are_not_clicks() {
        let c = clip();
        let ramp = (0.01 * c.sr as f64) as usize;
        let period = c.sr as usize / 2;
        let out: Vec<f32> = good(&c)
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let ph = i % period;
                let g = if ph < ramp {
                    0.25 + 0.75 * ph as f32 / ramp as f32
                } else if ph < period / 2 {
                    1.0
                } else if ph < period / 2 + ramp {
                    1.0 - 0.75 * (ph - period / 2) as f32 / ramp as f32
                } else {
                    0.25
                };
                v * g
            })
            .collect();
        let s = score("V-CLICK", &c, &out);
        assert_eq!(s.value, Some(0.0), "{s}");
    }

    #[test]
    fn identity_output_is_not_a_click() {
        // 输出 = 带噪输入：输入自带的陡变不算咔嗒
        let c = clip();
        let s = score("V-CLICK", &c, &c.mix);
        assert_eq!(s.value, Some(0.0), "{s}");
    }

    #[test]
    fn weak_is_gain_sensitive() {
        // 不先扣整体增益：整体压 6 dB，L30 损失应约 6 dB
        let c = clip();
        let out: Vec<f32> = good(&c).iter().map(|v| v * 0.5).collect();
        let s = score("V-WEAK", &c, &out);
        let v = s.value.unwrap();
        assert!((v - 6.0).abs() < 1.0, "{v}");
    }

    #[test]
    fn silent_reference_is_unknown() {
        let mut c = clip();
        c.clean.iter_mut().for_each(|v| *v = 0.0);
        let out = c.mix.clone();
        for id in ["V-WEAK", "V-ESTOI"] {
            assert!(
                matches!(score(id, &c, &out).verdict, Verdict::Unknown(_)),
                "{id}"
            );
        }
    }
}
