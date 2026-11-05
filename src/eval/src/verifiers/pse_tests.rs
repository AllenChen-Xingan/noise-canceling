//! 量输出的个性化降噪几项的单元测试：已知损伤要量得出，原样输出要量出「没坏」。
use super::*;
use crate::analysis::masks;
use crate::synth::{InterfMode, InterfSpec, NoiseKind, SceneSpec, TalkerSpec, render};

fn clip(mode: Option<InterfMode>) -> Clip {
    render(&SceneSpec {
        dur_s: 12.0,
        seed: 11,
        owner: Some(TalkerSpec {
            level_dbov: -26.0,
            f0_lo: 100.0,
            f0_hi: 140.0,
            tonal: false,
        }),
        owner_start_s: 0.5,
        pause_s: (0.6, 1.4),
        interferers: mode
            .map(|m| {
                vec![InterfSpec {
                    mode: m,
                    sir_db: 0.0,
                    f0_lo: 180.0,
                    f0_hi: 240.0,
                }]
            })
            .unwrap_or_default(),
        noise: NoiseKind::Pink,
        snr_db: 20.0,
        tail_silence_s: 0.0,
    })
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

fn extra(s: &Score, k: &str) -> f64 {
    s.extra
        .iter()
        .find(|(x, _)| *x == k)
        .map(|(_, v)| *v)
        .unwrap()
}

#[test]
fn tsos_zero_for_clean_and_high_for_suppressed_speech() {
    let c = clip(None);
    let s = score("V-TSOS", &c, &c.clean);
    assert_eq!(s.value, Some(0.0), "输出 = 干净参考不该有误删：{s}");
    assert!(matches!(s.verdict, Verdict::Unknown(_)), "只报不判");
    // 整体压 20 dB：|Ŝ|^0.3 = 0.5·|S|^0.3，损失 0.5 > γ = 0.1 → 每帧都误删
    let low: Vec<f32> = c.clean.iter().map(|v| v * 0.1).collect();
    let s = score("V-TSOS", &c, &low);
    assert!(s.value.unwrap() > 95.0, "{s}");
    assert!(extra(&s, "longest_s") > 0.5);
}

#[test]
fn swallow_measures_cut_after_onset() {
    let c = clip(None);
    // 只切每个区间开头的 100 ms（区间不足 150 ms 的不切，免得相邻区间的切口连成一片）
    let cut = (0.1 * c.sr as f64) as usize;
    let mut out = c.clean.clone();
    for &(a, e) in &c.truth.owner {
        if e - a >= cut * 3 / 2 {
            out[a..a + cut].iter_mut().for_each(|v| *v = 0.0);
        }
    }
    let s = score("V-SWALLOW", &c, &out);
    let p90 = s.value.unwrap();
    // 吞 100 ms：按 16 ms 跳、32 ms 窗量，允许 ±3 跳
    assert!((50.0..=150.0).contains(&p90), "{s}");
    assert_eq!(score("V-SWALLOW", &c, &c.clean).value, Some(0.0));
}

#[test]
fn bridge_fills_short_gaps_only() {
    let m = [true, false, false, true, false, false, false, false, true];
    assert_eq!(
        bridge(&m, 2),
        vec![true, true, true, true, false, false, false, false, true]
    );
}

#[test]
fn runs_split_on_gaps_and_non_hits() {
    let f = [
        (0, true),
        (1, true),
        (2, false),
        (3, true),
        (5, true),
        (6, true),
    ];
    assert_eq!(runs(&f), vec![2, 1, 2]);
    assert!(runs(&[(0, false)]).is_empty());
}

#[test]
fn leak_counts_unsuppressed_interferer_windows() {
    let c = clip(Some(InterfMode::OtherFirst));
    let s = score("V-LEAK", &c, &c.mix);
    assert_eq!(
        s.value,
        Some(100.0),
        "原样输出：旁人一点没压，全算漏过：{s}"
    );
    assert!(extra(&s, "windows") >= 1.0);
    // 旁人压 30 dB：都不算漏过
    let out: Vec<f32> = c
        .clean
        .iter()
        .zip(&c.noise)
        .zip(&c.interf)
        .map(|((s, n), i)| s + 0.03 * n + 0.03 * i)
        .collect();
    assert_eq!(score("V-LEAK", &c, &out).value, Some(0.0));
}

#[test]
fn pse_ovl_zero_for_passthrough_positive_for_extraction() {
    let c = clip(Some(InterfMode::Overlap));
    let s = score("V-PSE-OVL", &c, &c.mix);
    assert!(s.value.unwrap().abs() < 1e-9, "原声直通 SI-SDRi = 0：{s}");
    assert_eq!(extra(&s, "fail_rate"), 1.0);
    let s = score("V-PSE-OVL", &c, &c.clean);
    assert!(s.value.unwrap() > 10.0, "{s}");
    assert_eq!(extra(&s, "fail_rate"), 0.0);
}

#[test]
fn si_sdr_basics() {
    let x: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.1).sin()).collect();
    // 完全一样：残差只剩 f32 舍入，读数很高
    assert!(si_sdr(&x, &x).unwrap() > 100.0);
    // 尺度不变：放大 3 倍读数仍很高；加一半独立噪声读数明显下降
    let y: Vec<f32> = x.iter().map(|v| v * 3.0).collect();
    assert!(si_sdr(&y, &x).unwrap() > 100.0);
    let z: Vec<f32> = x
        .iter()
        .enumerate()
        .map(|(i, v)| v + 0.5 * (i as f32 * 1.7).cos())
        .collect();
    assert!(si_sdr(&z, &x).unwrap() < 20.0);
    assert_eq!(si_sdr(&x, &vec![0.0; 1000]), None);
}
