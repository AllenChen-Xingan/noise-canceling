//! 整段入口 vs 流式入口逐位相同；补零正确；f64 路径；钩子顺序；功率公式。

use super::*;
use crate::StreamingStft;
use realfft::num_complex::Complex32;
use vdn_types::{FRONTEND, StftSpec, Window};

fn signal(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| ((i as f32) * 0.021).sin() * 0.4 + ((i as f32) * 0.37).cos() * 0.1)
        .collect()
}

/// Kaldi 认人的切法：400 / 补零到 512 / 160 / 汉明。
const KALDI: StftSpec = StftSpec {
    n_fft: 512,
    win_len: 400,
    hop: 160,
    window: Window::KaldiHamming,
    ..FRONTEND
};

/// 同一口径（FRONTEND）下：流式喂完第 k+1 个 hop 时的那帧 = 整段第 k−1 帧（n_fft = 2·hop，流式缓冲开头是零）。
#[test]
fn whole_matches_streaming_bitwise() {
    let x = signal(FRONTEND.hop * 12);
    let mut st = StreamingStft::from_spec(&FRONTEND);
    let mut fr = FrameFft::<f32>::from_spec(&FRONTEND);
    let mut checked = 0;
    for (k, h) in x.chunks(FRONTEND.hop).enumerate() {
        let a: Vec<Complex32> = st.analyze(h).to_vec();
        if k >= 1 {
            let b = fr.frame(&x, k - 1).expect("帧在范围内");
            assert_eq!(a, b.to_vec(), "k={k}");
            checked += 1;
        }
    }
    assert_eq!(checked, 11);
    assert_eq!(fr.num_frames(x.len()), 11);
}

/// 补零：只取 win_len 个样本加窗，后面全零，再做 n_fft 点 FFT。参照直接调 realfft 手算。
#[test]
fn zero_padding_is_correct() {
    let x = signal(1600);
    let mut fr = FrameFft::<f32>::from_spec(&KALDI);
    assert_eq!(
        (fr.n_fft(), fr.win_len(), fr.hop(), fr.bins()),
        (512, 400, 160, 257)
    );
    let w: Vec<f32> = crate::window::for_spec(&KALDI);
    let fft = RealFftPlanner::<f32>::new().plan_fft_forward(512);
    for k in [0usize, 3, fr.num_frames(x.len()) - 1] {
        let mut buf = vec![0.0f32; 512];
        for i in 0..400 {
            buf[i] = x[k * 160 + i] * w[i];
        }
        let mut want = fft.make_output_vec();
        fft.process(&mut buf, &mut want).unwrap();
        // 先跑一帧别的，确认尾部的零每次都重新写（不是靠初值）
        fr.frame(&x, 1);
        assert_eq!(fr.frame(&x, k).unwrap().to_vec(), want, "k={k}");
    }
}

/// 钩子在加窗之前、拿到的是原样本拷贝；越界返回 None 且不做 FFT；帧数按 snip_edges。
#[test]
fn hook_order_bounds_and_count() {
    let x = signal(1600);
    let mut a = FrameFft::<f32>::from_spec(&KALDI);
    let mut b = FrameFft::<f32>::from_spec(&KALDI);
    let shifted: Vec<f32> = x.iter().map(|v| v * 2.0 - 0.1).collect();
    let got = a
        .frame_with(&x, 2, |f| {
            for v in f.iter_mut() {
                *v = *v * 2.0 - 0.1;
            }
        })
        .unwrap()
        .to_vec();
    assert_eq!(got, b.frame(&shifted, 2).unwrap().to_vec());
    assert_eq!(a.num_frames(399), 0);
    assert_eq!(a.num_frames(400), 1);
    assert_eq!(a.num_frames(1600), 8);
    let n = a.forward_count();
    assert!(a.frame(&x, 8).is_none());
    assert!(a.frame(&x, usize::MAX).is_none());
    assert_eq!(a.forward_count(), n);
}

/// f64 路径：评测器那种切法（周期 Hann，2000 补零到 2048），与直接调 realfft 的 f64 手算逐位相同，单音落在对的频点。
#[test]
fn f64_path_matches_manual() {
    let spec = StftSpec {
        n_fft: 2048,
        win_len: 2000,
        hop: 1000,
        window: Window::HannPeriodic,
        ..FRONTEND
    };
    let sr = f64::from(FRONTEND.sample_rate);
    let x: Vec<f64> = (0..6000)
        .map(|i| (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr).sin())
        .collect();
    let mut fr = FrameFft::<f64>::from_spec(&spec);
    let w: Vec<f64> = crate::window::for_spec(&spec);
    let fft = RealFftPlanner::<f64>::new().plan_fft_forward(2048);
    let mut buf = vec![0.0f64; 2048];
    for i in 0..2000 {
        buf[i] = w[i] * x[1000 + i];
    }
    let mut want = fft.make_output_vec();
    fft.process(&mut buf, &mut want).unwrap();
    let got = fr.frame(&x, 1).unwrap().to_vec();
    assert_eq!(got, want);
    let mut p = vec![0.0f64; fr.bins()];
    power_into(&got, &mut p);
    let peak = p
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| i)
        .unwrap();
    // 1000 Hz × 2048 / 16000 = 128
    assert_eq!(peak, 128);
}

/// `power_into` 与 `norm_sqr` 逐位相同（场景旧代码、认人 fbank 旧代码用的都是 `norm_sqr`），f32、f64 都是。
#[test]
fn power_matches_norm_sqr() {
    let spec: Vec<Complex32> = (0..257)
        .map(|k| Complex32::new((k as f32 * 0.3).sin() * 1e3, (k as f32 * 0.7).cos() * 1e-3))
        .collect();
    let mut p = vec![0.0; 257];
    power_into(&spec, &mut p);
    for (a, c) in p.iter().zip(&spec) {
        assert_eq!(a.to_bits(), c.norm_sqr().to_bits());
    }
    let s64: Vec<Complex<f64>> = spec
        .iter()
        .map(|c| Complex::new(f64::from(c.re) * 1.1, f64::from(c.im)))
        .collect();
    let mut p64 = vec![0.0f64; 257];
    power_into(&s64, &mut p64);
    for (a, c) in p64.iter().zip(&s64) {
        assert_eq!(a.to_bits(), c.norm_sqr().to_bits());
    }
}

#[test]
#[should_panic(expected = "口径不自洽")]
fn rejects_win_longer_than_fft() {
    let _ = FrameFft::<f32>::from_spec(&StftSpec {
        win_len: 513,
        ..FRONTEND
    });
}

#[test]
#[should_panic(expected = "不支持补零")]
fn streaming_rejects_padding() {
    let _ = StreamingStft::from_spec(&KALDI);
}
