//! #110：按频带算的项在低采样率下不许崩、不许被空频带拖偏（调研 025 ①-1b）。
//! 16 kHz 读数逐位不变由 `speech.rs` / 探针测试守；这里只管 16 kHz 以外。

use super::*;
use crate::analysis::masks;
use crate::synth::{NoiseKind, SceneSpec, TalkerSpec, Truth, render};

fn clip16() -> Clip {
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

/// 线性插值重采样（测试用，混叠无所谓：只要求采样率真的变了）。
fn resample(x: &[f32], from: u32, to: u32) -> Vec<f32> {
    let n = (x.len() as u64 * to as u64 / from as u64) as usize;
    (0..n)
        .map(|i| {
            let t = i as f64 * from as f64 / to as f64;
            let k = t.floor() as usize;
            let f = (t - k as f64) as f32;
            let a = x[k.min(x.len() - 1)];
            let b = x[(k + 1).min(x.len() - 1)];
            a + (b - a) * f
        })
        .collect()
}

fn at_rate(sr: u32) -> Clip {
    let c = clip16();
    let r = |v: &[f32]| resample(v, c.sr, sr);
    let iv = |v: &[(usize, usize)]| -> Vec<(usize, usize)> {
        let s = |i: usize| (i as u64 * sr as u64 / c.sr as u64) as usize;
        v.iter().map(|(a, b)| (s(*a), s(*b))).collect()
    };
    Clip {
        sr,
        clean: r(&c.clean),
        interf: r(&c.interf),
        noise: r(&c.noise),
        mix: r(&c.mix),
        truth: Truth {
            owner: iv(&c.truth.owner),
            interf: iv(&c.truth.interf),
        },
        tracks: c.tracks,
    }
}

/// 输出 = 干净轨 + 低 30 dB 残留噪声（同 `speech.rs` 的「好输出」）。
fn score(id: &str, c: &Clip) -> Score {
    let out: Vec<f32> = c
        .clean
        .iter()
        .zip(&c.noise)
        .map(|(s, n)| s + 0.03 * n)
        .collect();
    let m = masks(&c.truth, &c.mix, 0.03);
    let ctx = Ctx {
        clip: c,
        out: &out,
        run: None,
        masks: &m,
        noise_out: None,
        ci: true,
    };
    eval(id, &ctx, &Params::default())
}

fn extra(s: &Score, k: &str) -> Option<f64> {
    s.extra.iter().find(|(x, _)| *x == k).map(|(_, v)| *v)
}

/// 8 kHz：最高一带（4 kHz，上沿 4.49 kHz）越过奈奎斯特，旧代码切片越界 panic。
/// 修后不计这一带，剩 9/10 带照常打分。
#[test]
fn weak_at_8k_does_not_panic_and_scores() {
    let s = score("V-WEAK", &at_rate(8_000));
    let v = s.value.expect("剩 9 带应照常打分");
    assert!(v.is_finite(), "{s}");
    assert_eq!(extra(&s, "bands"), Some(9.0), "{s}");
}

/// 2 kHz：只剩 500 · 630 · 800 Hz 三带（< 一半）→ 判「不知道」并写明原因。
#[test]
fn weak_with_too_few_bands_is_unknown() {
    let s = score("V-WEAK", &at_rate(2_000));
    assert!(matches!(s.verdict, Verdict::Unknown(_)), "{s}");
}

/// 12 / 8 / 6 kHz：上沿越过奈奎斯特频率的 ERB 带不计，剩下过半照常打分；不许出现 −200 dB 级的空带读数（调研 026 ③）。
#[test]
fn timbre_drops_bands_above_nyquist() {
    for sr in [12_000, 8_000, 6_000] {
        let s = score("V-TIMBRE", &at_rate(sr));
        assert!(s.value.is_some_and(f64::is_finite), "{sr}: {s}");
        let n = extra(&s, "bands").unwrap_or_else(|| panic!("{sr}: 缺 bands"));
        assert!((15.0..30.0).contains(&n), "{sr}: bands = {n}");
        let m = extra(&s, "max_db").unwrap_or_else(|| panic!("{sr}: 缺 max_db"));
        assert!(m < 40.0, "{sr}: max_db = {m}");
    }
}

/// 2 kHz：奈奎斯特 1 kHz 以下只剩 12 个完整 ERB 带，不到一半（15）→ 判「不知道」。
#[test]
fn timbre_with_too_few_bands_is_unknown() {
    let s = score("V-TIMBRE", &at_rate(2_000));
    assert!(matches!(s.verdict, Verdict::Unknown(_)), "{s}");
}
