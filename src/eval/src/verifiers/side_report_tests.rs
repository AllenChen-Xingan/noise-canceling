//! 调研 025 要求的旁报（A2 · A4 · B1 · B2 · B3 · C1）：只加数、不改判定。每条一个已知输入 → 已知读数。
use super::*;
use crate::analysis::masks;
use crate::synth::{InterfMode, InterfSpec, NoiseKind, SceneSpec, TalkerSpec, render};
use crate::system::{SystemKind, SystemSpec, run};

fn clip(noise_db: f64, interf: bool) -> Clip {
    render(&SceneSpec {
        dur_s: 10.0,
        seed: 3,
        owner: Some(TalkerSpec {
            level_dbov: -26.0,
            f0_lo: 100.0,
            f0_hi: 140.0,
            tonal: false,
        }),
        owner_start_s: 0.5,
        pause_s: (0.6, 1.2),
        interferers: if interf {
            vec![InterfSpec {
                mode: InterfMode::Alternate,
                sir_db: 0.0,
                f0_lo: 180.0,
                f0_hi: 240.0,
            }]
        } else {
            Vec::new()
        },
        noise: NoiseKind::Pink,
        snr_db: noise_db,
        tail_silence_s: 0.0,
    })
}

fn score(id: &str, c: &Clip, out: &[f32], noise_out: Option<&[f32]>, r: Option<&RunOut>) -> Score {
    let m = masks(&c.truth, &c.mix, 0.03);
    let ctx = Ctx {
        clip: c,
        out,
        run: r,
        masks: &m,
        noise_out,
        ci: false,
    };
    eval(id, &ctx, &Params::default())
}

fn extra(s: &Score, k: &str) -> Option<f64> {
    s.extra.iter().find(|(x, _)| *x == k).map(|(_, v)| *v)
}

#[test]
fn nan_counts_samples_over_full_scale() {
    let c = clip(10.0, false);
    let mut out = c.mix.clone();
    out[10] = 1.5;
    out[20] = -2.0;
    let s = score("V-NAN", &c, &out, None, None);
    assert_eq!(extra(&s, "over_1"), Some(2.0));
    assert!(s.verdict.is_pass(), "幅度 1–4 只旁报，不进判定");
}

#[test]
fn frame_reports_over_one_period_and_first_frame() {
    let c = clip(10.0, false);
    let spec = SystemSpec {
        kind: SystemKind::Ref,
        safety: true,
        faults: Vec::new(),
    };
    let r = run(&spec, &c.mix).unwrap();
    let s = score("V-FRAME", &c, &r.out, None, Some(&r));
    assert!(extra(&s, "first").is_some_and(f64::is_finite), "{s}");
    assert!(extra(&s, "over_1").is_some(), "{s}");
}

#[test]
fn nr_reports_low_level_pause_frames() {
    // 底噪很低（信噪比 60 dB）：停顿帧几乎全低于 −48 dBov
    let c = clip(60.0, false);
    let s = score("V-NR", &c, &c.mix, None, None);
    assert!(extra(&s, "low_frac").unwrap() > 0.9, "{s}");
    // 正常底噪（10 dB）：低电平帧很少，排除后的 TNLR 与主值相同（原声直通都是 0）
    let c = clip(10.0, false);
    let s = score("V-NR", &c, &c.mix, None, None);
    assert!(extra(&s, "low_frac").unwrap() < 0.1, "{s}");
    assert!(extra(&s, "tnlr_excl_low_db").unwrap().abs() < 1e-9);
}

#[test]
fn intf_minus_floor_isolates_interferer() {
    // 只压旁人 20 dB、底噪原样：扣底噪后的读数应比主值更接近 −20
    let c = clip(10.0, true);
    let out: Vec<f32> = c
        .clean
        .iter()
        .zip(&c.noise)
        .zip(&c.interf)
        .map(|((s, n), i)| s + n + 0.1 * i)
        .collect();
    let s = score("V-INTF", &c, &out, None, None);
    let (main, mf) = (s.value.unwrap(), extra(&s, "minus_floor_db").unwrap());
    assert!(
        mf < main && (mf + 20.0).abs() < 3.0,
        "主值 {main} · 扣底噪 {mf}"
    );
}

#[test]
fn bgstab_bias_corrected_passthrough_is_zero() {
    // 原声直通：输出 = 混合，「只有噪音」那次的输出 = 噪音 + 干扰。修正后必须 ≈ 0 dB（025 B3 验收）
    let c = clip(10.0, true);
    let bg: Vec<f32> = c.noise.iter().zip(&c.interf).map(|(a, b)| a + b).collect();
    let s = score("V-BGSTAB", &c, &c.mix, Some(&bg), None);
    assert!(extra(&s, "bias_corr_db").unwrap() < 1e-9, "{s}");
    assert!(extra(&s, "bias_corr_p95_db").unwrap() < 1e-9);
}

#[test]
fn music_reports_residual_below_speech() {
    let c = clip(10.0, false);
    let out_hi: Vec<f32> = c
        .clean
        .iter()
        .zip(&c.noise)
        .map(|(s, n)| s + 0.3 * n)
        .collect();
    let out_lo: Vec<f32> = c
        .clean
        .iter()
        .zip(&c.noise)
        .map(|(s, n)| s + 0.03 * n)
        .collect();
    let a = extra(
        &score("V-MUSIC", &c, &out_hi, None, None),
        "resid_below_speech_db",
    )
    .unwrap();
    let b = extra(
        &score("V-MUSIC", &c, &out_lo, None, None),
        "resid_below_speech_db",
    )
    .unwrap();
    // 残留压低 20 dB，「比人声低多少」应多约 20 dB
    assert!((b - a - 20.0).abs() < 2.0, "{a} → {b}");
}
