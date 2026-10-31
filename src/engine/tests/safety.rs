//! 三件保命的行为测试 + 不分配证明（用假模型，不需要 ONNX Runtime）。

use std::time::Duration;

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_engine::testing::DelayGain;
use vdn_engine::{Engine, FaultInjector, FaultKind, SafetyConfig, offline};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

const HOP: usize = 256;
const LAT: usize = 256;

fn tone(n: usize) -> Vec<f32> {
    (0..n).map(|i| 0.3 * (i as f32 * 0.05).sin()).collect()
}

#[test]
fn safety_off_is_bit_exact_model_output() {
    let x = tone(HOP * 20);
    let mut e = Engine::new(DelayGain::new(HOP, LAT, 0.5), SafetyConfig::off());
    let y = offline::run(&mut e, &x, 0).output;
    for (a, b) in x.iter().zip(&y) {
        assert_eq!(a * 0.5, *b);
    }
}

#[test]
fn floor_mixes_aligned_dry() {
    // 模型把一切压成 0：输出 = 原声 × 10^(-12/20)，且与原声逐样本对齐
    let x = tone(HOP * 20);
    let cfg = SafetyConfig {
        fast_open: false,
        ..SafetyConfig::default()
    };
    let mut e = Engine::new(DelayGain::new(HOP, LAT, 0.0), cfg);
    let y = offline::run(&mut e, &x, 0).output;
    let g = 10f32.powf(-12.0 / 20.0);
    for (a, b) in x.iter().zip(&y) {
        assert!((a * g - b).abs() < 1e-6);
    }
}

#[test]
fn nan_falls_back_to_aligned_dry_without_click() {
    let x = tone(HOP * 40);
    let plan = vec![(10, FaultKind::Nan)];
    let m = FaultInjector::new(DelayGain::new(HOP, LAT, 1.0), plan);
    let mut e = Engine::new(m, SafetyConfig::off_but_fallback());
    let y = offline::run(&mut e, &x, 0).output;
    assert!(y.iter().all(|v| v.is_finite()));
    // 增益 1 的模型 = 原声，所以送原声前后都应几乎等于输入（无爆音、无错位）
    let max_err = x.iter().zip(&y).map(|(a, b)| (a - b).abs()).fold(0.0, f32::max);
    assert!(max_err < 1e-4, "max_err={max_err}");
    let s = e.stats();
    assert_eq!(s.fault_nonfinite, 1);
    assert_eq!(s.model_resets, 1);
    assert!(s.bypass_hops >= 1);
}

#[test]
fn bypass_seam_is_continuous() {
    // 模型增益 0.2 vs 原声：切换时输出相邻样本差不应大于信号本身的相邻差 + 一点余量
    let x = tone(HOP * 60);
    let plan = vec![(20, FaultKind::Error), (40, FaultKind::Explode)];
    let m = FaultInjector::new(DelayGain::new(HOP, LAT, 0.2), plan);
    let mut e = Engine::new(m, SafetyConfig::off_but_fallback());
    let y = offline::run(&mut e, &x, 0).output;
    let max_step_x = x.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f32::max);
    let max_step_y = y.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0, f32::max);
    assert!(max_step_y < max_step_x * 2.5, "x {max_step_x} y {max_step_y}");
    let s = e.stats();
    assert_eq!((s.fault_backend, s.fault_abnormal), (1, 1));
}

#[test]
fn deadline_miss_uses_dry_but_keeps_state() {
    let x = tone(HOP * 20);
    let plan = vec![(5, FaultKind::Stall(Duration::from_millis(30)))];
    let m = FaultInjector::new(DelayGain::new(HOP, LAT, 1.0), plan);
    let cfg = SafetyConfig {
        deadline: Some(Duration::from_millis(10)),
        ..SafetyConfig::off_but_fallback()
    };
    let mut e = Engine::new(m, cfg);
    let _ = offline::run(&mut e, &x, 0);
    let s = e.stats();
    assert_eq!(s.fault_deadline, 1);
    assert_eq!(s.model_resets, 0);
}

#[test]
fn process_does_not_allocate() {
    let x = tone(HOP);
    let mut y = vec![0.0; HOP];
    let plan = vec![(3, FaultKind::Nan), (9, FaultKind::Error), (15, FaultKind::Explode)];
    let m = FaultInjector::new(DelayGain::new(HOP, LAT, 0.3), plan);
    let mut e = Engine::new(m, SafetyConfig::default());
    assert_no_alloc(|| {
        for _ in 0..40 {
            e.process(&x, &mut y);
        }
    });
    assert_eq!(e.stats().model_resets, 3);
}

trait OffButFallback {
    fn off_but_fallback() -> Self;
}

impl OffButFallback for SafetyConfig {
    fn off_but_fallback() -> Self {
        SafetyConfig {
            fallback: true,
            ..SafetyConfig::off()
        }
    }
}
