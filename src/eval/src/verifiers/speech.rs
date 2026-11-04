//! 人声细节 verifier（012 §3.1 · §3.2，定标前一律 L0）：轻声字被吞 V-WEAK · 人声忽大忽小 V-PUMPV ·
//! 爆音 V-CLICK · 可懂度 V-ESTOI。都要干净参考（V-CLICK 要带噪输入），真实录音没有参考时判「不知道」。

use realfft::RealFftPlanner;

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

/// 每帧每带的电平（dB），125 ms Hann 窗、50% 重叠；只留有声帧。返回 [带][帧]。
fn band_levels(c: &Ctx, x: &[f32]) -> Vec<Vec<f64>> {
    let sr = c.clip.sr as f64;
    let win = (0.125 * sr) as usize;
    let hop = win / 2;
    let nfft = win.next_power_of_two();
    let w: Vec<f64> = (0..win)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / win as f64).cos())
        .collect();
    let mut planner = RealFftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(nfft);
    let mut buf = fft.make_input_vec();
    let mut spec = fft.make_output_vec();
    let edges: Vec<(usize, usize)> = WEAK_CF
        .iter()
        .map(|f| {
            let lo = (f * 2f64.powf(-1.0 / 6.0) * nfft as f64 / sr).round() as usize;
            let hi = (f * 2f64.powf(1.0 / 6.0) * nfft as f64 / sr).round() as usize;
            (lo, hi.max(lo + 1))
        })
        .collect();
    let mut out = vec![Vec::new(); WEAK_CF.len()];
    let n = x.len().min(c.clip.clean.len());
    let mut s = 0;
    while s + win <= n {
        if owner_at(c, s + win / 2) {
            buf.iter_mut().for_each(|v| *v = 0.0);
            for k in 0..win {
                buf[k] = w[k] * x[s + k] as f64;
            }
            let _ = fft.process(&mut buf, &mut spec);
            for (b, &(lo, hi)) in edges.iter().enumerate() {
                let p: f64 = spec[lo..hi].iter().map(|z| z.norm_sqr()).sum();
                out[b].push(db10(p));
            }
        }
        s += hop;
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
        return Score::unknown(ID, "dB²", Better::Lower, format!("有声窗只有 {} 个", r.len()));
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
        let mad = quantile(&chunk.iter().map(|v| (v - med).abs()).collect::<Vec<_>>(), 0.5);
        let thr = med + k * mad.max(1e-6);
        for (i, v) in chunk.iter().enumerate() {
            flag[bi * blk + i] = *v > thr;
        }
    }
    flag
}

pub(super) fn v_click(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-CLICK";
    let k = p.f_or("k", th::CLICK.def("k"));
    let sr = c.clip.sr;
    let fo = outliers(c.out, sr, k);
    let fm = outliers(&c.clip.mix, sr, k);
    // 同一位置 ±2 ms 内输入也有离群点 → 算原声自带的瞬态，不计
    let guard = (0.002 * sr as f64) as usize;
    let mut count = 0usize;
    let mut last: Option<usize> = None;
    for (i, &f) in fo.iter().enumerate() {
        if !f {
            continue;
        }
        let a = i.saturating_sub(guard);
        let b = (i + guard + 1).min(fm.len());
        if fm.get(a..b).is_some_and(|w| w.iter().any(|x| *x)) {
            continue;
        }
        // 相邻 5 ms 内的离群点算同一个咔嗒
        if last.is_none_or(|l| i > l + 5 * guard / 2) {
            count += 1;
        }
        last = Some(i);
    }
    let minutes = c.out.len() as f64 / sr as f64 / 60.0;
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
    let (Some(o), Some(m)) = (
        estoi(&c.clip.clean, c.out, sr),
        estoi(&c.clip.clean, &c.clip.mix, sr),
    ) else {
        return Score::unknown(ID, "Δ", Better::Higher, "去掉静音帧后不足 30 帧（约 0.4 s）");
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
        assert!(b.verdict.is_fail(), "{id} 探针 {} 应不过：{b}", probe.as_str());
        let (va, vb) = (a.value.unwrap(), b.value.unwrap());
        assert!(a.better.worse(vb, va), "{id}：探针 {vb} 不比原输出 {va} 差");
    }

    #[test]
    fn weak_catches_weak_minus10() {
        must_catch("V-WEAK", Probe::WeakMinus10);
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
    fn estoi_catches_lowpass() {
        must_catch("V-ESTOI", Probe::Lowpass1k);
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
        assert!(matches!(
            score("V-WEAK", &c, &out).verdict,
            Verdict::Unknown(_)
        ));
    }
}
