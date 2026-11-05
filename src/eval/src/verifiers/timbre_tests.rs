//! V-TIMBRE（调研 026）：量法的判据测试 J1 / J2 / J4 / J5，停顿外扩与 runner 一致，
//! 以及标准 N4 与代码正本 [`timbre`] 逐项比对（故意改错要红）。

use super::*;
use crate::analysis::{Masks, masks};
use crate::synth::{NoiseKind, SceneSpec, TalkerSpec, render};

fn clip() -> Clip {
    render(&SceneSpec {
        dur_s: 12.0,
        seed: 26,
        owner: Some(TalkerSpec {
            level_dbov: -26.0,
            f0_lo: 100.0,
            f0_hi: 160.0,
            tonal: false,
        }),
        owner_start_s: 0.5,
        pause_s: (0.4, 1.2),
        interferers: Vec::new(),
        noise: NoiseKind::Pink,
        snr_db: 20.0,
        tail_silence_s: 0.0,
    })
}

/// 干净人声那一路的评分上下文（同 `runner::Prepared::clean_view`：旁人与「只有噪音」掩码全空）。
fn clean_pass(c: &Clip, out: &[f32]) -> Score {
    let m = masks(&c.truth, &c.mix, 0.03);
    let n = m.owner.len();
    let cv = Masks {
        owner: m.owner.clone(),
        interf: vec![false; n],
        noise_only: vec![false; n],
        interf_only: vec![false; n],
    };
    let ctx = Ctx {
        clip: c,
        out,
        run: None,
        masks: &cv,
        noise_out: None,
        ci: true,
    };
    eval("V-TIMBRE", &ctx, &Params::default())
}

fn val(c: &Clip, out: &[f32]) -> f64 {
    let s = clean_pass(c, out);
    s.value.unwrap_or_else(|| panic!("应有读数：{s}"))
}

/// RBJ 双二阶滤波（`kind`：0 低搁架 · 1 高搁架 · 2 峰值，S = 1；峰值的 `q` 为品质因数）。
fn biquad(x: &[f32], sr: f64, kind: u8, fc: f64, gain_db: f64, q: f64) -> Vec<f32> {
    let a = 10f64.powf(gain_db / 40.0);
    let w = 2.0 * std::f64::consts::PI * fc / sr;
    let (cw, sw) = (w.cos(), w.sin());
    let (b, d) = match kind {
        2 => {
            let al = sw / (2.0 * q);
            (
                [1.0 + al * a, -2.0 * cw, 1.0 - al * a],
                [1.0 + al / a, -2.0 * cw, 1.0 - al / a],
            )
        }
        _ => {
            let sa = 2.0 * a.sqrt() * sw / 2.0 * 2f64.sqrt();
            let s = if kind == 1 { 1.0 } else { -1.0 };
            (
                [
                    a * ((a + 1.0) + s * (a - 1.0) * cw + sa),
                    -2.0 * s * a * ((a - 1.0) + s * (a + 1.0) * cw),
                    a * ((a + 1.0) + s * (a - 1.0) * cw - sa),
                ],
                [
                    (a + 1.0) - s * (a - 1.0) * cw + sa,
                    2.0 * s * ((a - 1.0) - s * (a + 1.0) * cw),
                    (a + 1.0) - s * (a - 1.0) * cw - sa,
                ],
            )
        }
    };
    let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
    x.iter()
        .map(|&v| {
            let v = v as f64;
            let y = (b[0] * v + b[1] * x1 + b[2] * x2 - d[1] * y1 - d[2] * y2) / d[0];
            (x2, x1, y2, y1) = (x1, v, y1, y);
            y as f32
        })
        .collect()
}

/// RBJ 2 阶低通（Q = 0.707）串 4 节。
fn lowpass(x: &[f32], sr: f64, fc: f64) -> Vec<f32> {
    let w = 2.0 * std::f64::consts::PI * fc / sr;
    let (cw, al) = (w.cos(), w.sin() / (2.0 * std::f64::consts::FRAC_1_SQRT_2));
    let b = [(1.0 - cw) / 2.0, 1.0 - cw, (1.0 - cw) / 2.0];
    let d = [1.0 + al, -2.0 * cw, 1.0 - al];
    let mut y: Vec<f32> = x.to_vec();
    for _ in 0..4 {
        let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
        y = y
            .iter()
            .map(|&v| {
                let v = v as f64;
                let o = (b[0] * v + b[1] * x1 + b[2] * x2 - d[1] * y1 - d[2] * y2) / d[0];
                (x2, x1, y2, y1) = (x1, v, y1, o);
                o as f32
            })
            .collect();
    }
    y
}

#[test]
fn unchanged_and_volume_only_read_zero() {
    let c = clip();
    assert!(val(&c, &c.clean) < 1e-9);
    let g: Vec<f32> = c.clean.iter().map(|v| v * 0.3).collect();
    assert!(val(&c, &g) < 1e-6, "只改音量应读 0");
}

/// J1：250 Hz 低搁架 ±6 dB 要看得见。旧量法在真人声上读 0.56 / 0.74（exp017），在这里的合成人声上读得到，
/// 所以这条在旧代码上不红，只防退步；旧量法的这个盲区由 exp017 的真人声实测与对拍（`tests/timbre_parity.rs`）证明。
#[test]
fn low_shelf_is_seen() {
    let c = clip();
    for g in [-6.0, 6.0] {
        let y = biquad(&c.clean, c.sr as f64, 0, 250.0, g, 0.0);
        let v = val(&c, &y);
        assert!(v > 1.0, "低搁架 {g} dB 只读 {v}");
    }
}

/// J2：同样一个倍频程宽的 −6 dB，放在 250 Hz、1 kHz、4 kHz，最大与最小读数之比 < 1.5。
/// 旧量法在这段合成人声上读 2.91 / 3.70 / 1.89（比 1.96），新量法 1.57 / 1.73 / 1.57（比 1.10）。
#[test]
fn same_dip_reads_alike_across_frequency() {
    let c = clip();
    let q = 2f64.sqrt(); // 一个倍频程带宽
    let v: Vec<f64> = [250.0, 1000.0, 4000.0]
        .iter()
        .map(|&f| val(&c, &biquad(&c.clean, c.sr as f64, 2, f, -6.0, q)))
        .collect();
    let (lo, hi) = v
        .iter()
        .fold((f64::MAX, 0.0f64), |(a, b), &x| (a.min(x), b.max(x)));
    assert!(
        hi / lo < 1.5,
        "250 Hz / 1 kHz / 4 kHz：{v:.2?}，比 {:.2}",
        hi / lo
    );
}

/// J4：低通之后再改整体音量，读数不变（截下限在去掉整体增益之后）。
#[test]
fn filter_then_gain_is_invariant() {
    let c = clip();
    let y = lowpass(&c.clean, c.sr as f64, 2000.0);
    let base = val(&c, &y);
    for g in [0.1f32, 10.0] {
        let z: Vec<f32> = y.iter().map(|v| v * g).collect();
        let v = val(&c, &z);
        assert!((v - base).abs() < 1e-6, "×{g}：{v} vs {base}");
    }
    assert!(base > 3.0, "低通 2 kHz 应判不过：{base}");
}

/// J5：干净人声那一路也扣停顿帧里的底噪——输出多出 −20 dB 的白噪声，读数仍在 A 档内。
#[test]
fn clean_pass_hiss_is_subtracted() {
    let c = clip();
    let ps: f64 = c.clean.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / c.clean.len() as f64;
    let mut st = 0x2026u64;
    let noise: Vec<f32> = (0..c.clean.len())
        .map(|_| {
            st = st
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((st >> 40) as f64 / (1u64 << 24) as f64 - 0.5) as f32
        })
        .collect();
    let pn: f64 = noise.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / noise.len() as f64;
    let k = (ps / pn / 100.0).sqrt() as f32;
    let y: Vec<f32> = c.clean.iter().zip(&noise).map(|(s, n)| s + k * n).collect();
    let v = val(&c, &y);
    assert!(v < crate::thresholds::TIMBRE.edges[0], "−20 dB 底噪读 {v}");
    let s = clean_pass(&c, &y);
    let qf = s
        .extra
        .iter()
        .find(|(k, _)| *k == "quiet_frames")
        .map(|(_, v)| *v);
    assert!(qf.is_some_and(|n| n > 10.0), "应自求出停顿帧：{s}");
}

/// 干净人声那一路自求的停顿帧，与 runner 用 30 ms 外扩求出的「只有噪音」掩码一致（无旁人、混合信号处处非零）。
#[test]
fn self_found_pauses_match_runner_guard() {
    let c = clip();
    let m = masks(&c.truth, &c.mix, 0.03);
    let n = m.owner.len();
    let cv = Masks {
        owner: m.owner.clone(),
        interf: vec![false; n],
        noise_only: vec![false; n],
        interf_only: vec![false; n],
    };
    assert_eq!(timbre_quiet_mask(&cv), m.noise_only);
}

/// 标准 N4 与代码正本比对：每个数都要在 N4 那一段里按约定写法出现。返回缺的那几项。
fn n4_mismatches(doc: &str) -> Vec<String> {
    use timbre::*;
    let Some(a) = doc.find("**N4 音色**") else {
        return vec!["找不到 **N4 音色** 一段".into()];
    };
    let para = doc[a..].lines().next().unwrap_or("");
    let e = crate::thresholds::TIMBRE.edges;
    let max = crate::thresholds::TIMBRE.def("max");
    let want = [
        format!("{N_FFT} 点 Hann"),
        format!("跳 {HOP}"),
        format!("{LO_HZ}–{HI_HZ} Hz"),
        format!("{N_BANDS} 带"),
        format!("至少留 {}%", (KEEP_FRAC * 100.0).round()),
        format!("外扩 {} ms", GUARD_FRAMES * 10),
        format!("最低记 −{} dB", -FLOOR_DB),
        format!("D > {max} · C ≤ {} · B ≤ {} · A ≤ {}", e[2], e[1], e[0]),
    ];
    want.into_iter()
        .filter(|w| !para.contains(w.as_str()))
        .collect()
}

#[test]
fn rubric_n4_matches_code() {
    let doc = include_str!("../../../../specs/rubric/denoise.md");
    let miss = n4_mismatches(doc);
    assert!(miss.is_empty(), "N4 与代码正本不一致，缺：{miss:?}");
    // 故意改错要红
    for (from, to) in [
        ("30 带", "31 带"),
        ("B ≤ 2", "B ≤ 2.5"),
        ("跳 512", "跳 256"),
    ] {
        let bad = doc.replacen(from, to, 1);
        assert_ne!(bad, doc, "N4 里找不到「{from}」");
        assert!(
            !n4_mismatches(&bad).is_empty(),
            "把「{from}」改成「{to}」应被查出"
        );
    }
}

#[test]
fn bands_span_100_to_7900_in_30() {
    let b = timbre::bands();
    assert_eq!(b.len(), 30);
    assert_eq!(b[0].0, 100.0);
    assert_eq!(b[29].1, 7900.0);
    assert!(
        b.windows(2)
            .all(|w| (w[0].1 - w[1].0).abs() < 1e-9 && w[0].0 < w[0].1)
    );
}
