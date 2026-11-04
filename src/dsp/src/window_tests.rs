//! 每种窗与改前各处的原公式逐位相同。下面的「原公式」是从 `d23bfe66` 的源码逐字抄来的参照
//!（测试里的参照，不是第二份实现；出处写在每个闭包上），改 [`coef`] 时它们不许跟着改。

use super::*;

const LENS: [usize; 6] = [2, 3, 256, 400, 512, 2000];

fn bits32(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn bits64(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

#[test]
fn sqrt_hann_matches_original() {
    // 原 src/dsp/src/window.rs::sqrt_hann_periodic
    let orig = |n: usize| -> Vec<f32> {
        (0..n)
            .map(|i| {
                let v = 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos();
                v.sqrt() as f32
            })
            .collect()
    };
    for n in LENS {
        assert_eq!(bits32(&sqrt_hann_periodic(n)), bits32(&orig(n)), "n={n}");
    }
}

#[test]
fn kaldi_windows_match_fbank_original() {
    // 原 src/speaker/src/fbank.rs::Fbank::new 的窗
    let orig = |w: Window, n: usize| -> Vec<f32> {
        (0..n)
            .map(|i| {
                let a = 2.0 * std::f64::consts::PI / (n as f64 - 1.0);
                let x = i as f64;
                (match w {
                    Window::KaldiHamming => 0.54 - 0.46 * (a * x).cos(),
                    Window::KaldiHanning => 0.5 - 0.5 * (a * x).cos(),
                    _ => (0.5 - 0.5 * (a * x).cos()).powf(0.85),
                }) as f32
            })
            .collect()
    };
    for w in [Window::KaldiHamming, Window::KaldiHanning, Window::KaldiPovey] {
        for n in LENS {
            assert_eq!(bits32(&table(w, n)), bits32(&orig(w, n)), "{w:?} n={n}");
        }
    }
}

#[test]
fn hann_periodic_matches_speech_original() {
    // 原 src/eval/src/verifiers/speech.rs::band_levels 的窗（f64）
    let orig = |win: usize| -> Vec<f64> {
        (0..win)
            .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / win as f64).cos())
            .collect()
    };
    for n in LENS {
        assert_eq!(
            bits64(&table(Window::HannPeriodic, n)),
            bits64(&orig(n)),
            "n={n}"
        );
    }
}

#[test]
fn hann_inner_matches_estoi_original() {
    // 原 src/eval/src/verifiers/estoi.rs::hann_inner（f64）
    let orig = |n: usize| -> Vec<f64> {
        let m = (n + 2) as f64;
        (1..=n)
            .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (m - 1.0)).cos())
            .collect()
    };
    for n in LENS {
        assert_eq!(
            bits64(&table(Window::HannNumpyInner, n)),
            bits64(&orig(n)),
            "n={n}"
        );
    }
}

/// f32 表 = f64 表逐个 `as f32`（先 f64 算再转）。
#[test]
fn f32_table_is_f64_table_cast() {
    for w in [
        Window::SqrtHannPeriodic,
        Window::HannPeriodic,
        Window::HannNumpyInner,
        Window::KaldiHamming,
        Window::KaldiHanning,
        Window::KaldiPovey,
    ] {
        let a: Vec<f32> = table(w, 400);
        let b: Vec<f32> = table::<f64>(w, 400).iter().map(|&v| v as f32).collect();
        assert_eq!(bits32(&a), bits32(&b), "{w:?}");
    }
}

/// 造窗长度取 `win_len`，不是 `n_fft`。
#[test]
fn for_spec_uses_win_len() {
    let spec = vdn_types::StftSpec {
        n_fft: 512,
        win_len: 400,
        hop: 160,
        window: Window::KaldiHamming,
        ..vdn_types::FRONTEND
    };
    let w: Vec<f32> = for_spec(&spec);
    assert_eq!(bits32(&w), bits32(&table(Window::KaldiHamming, 400)));
    let f: Vec<f32> = for_spec(&vdn_types::FRONTEND);
    assert_eq!(bits32(&f), bits32(&sqrt_hann_periodic(512)));
}

#[test]
fn squared_window_overlap_adds_to_one_at_half_hop() {
    let n = 512;
    let w = sqrt_hann_periodic(n);
    for i in 0..n / 2 {
        let s = w[i] * w[i] + w[i + n / 2] * w[i + n / 2];
        assert!((s - 1.0).abs() < 1e-6, "i={i} s={s}");
    }
}
