//! 三件保命的行为测试 + 不分配证明（用假模型，不需要 ONNX Runtime）。

use std::time::Duration;

#[cfg(debug_assertions)]
use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_engine::testing::DelayGain;
use vdn_engine::{Engine, FaultInjector, FaultKind, SafetyConfig, offline};

// AllocDisabler 只在 debug_assertions 下存在：release 下只跑行为测试，不分配断言跳过
#[cfg(debug_assertions)]
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
    let max_err = x
        .iter()
        .zip(&y)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f32::max);
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
    let max_step_x = x
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max);
    let max_step_y = y
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max);
    assert!(
        max_step_y < max_step_x * 2.5,
        "x {max_step_x} y {max_step_y}"
    );
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

#[cfg(debug_assertions)]
#[test]
fn process_does_not_allocate() {
    let x = tone(HOP);
    let mut y = vec![0.0; HOP];
    let plan = vec![
        (3, FaultKind::Nan),
        (9, FaultKind::Error),
        (15, FaultKind::Explode),
    ];
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

#[test]
fn latency_ledger_is_hop_plus_model_latency() {
    // GTCRN 形态：hop 256、算法延迟 256 @16 kHz → 32 ms；保命不加延迟
    let e = Engine::new(DelayGain::new(256, 256, 1.0), SafetyConfig::default());
    let l = e.latency_ledger();
    assert_eq!(l.items().len(), 2);
    assert!((l.total_ms() - 32.0).abs() < 1e-9);
    assert!(l.check(vdn_types::LATENCY_BUDGET_MS).is_ok());
}

#[test]
fn apply_params_maps_policy_output() {
    let p = vdn_types::ParamSet {
        floor_db: -20.0,
        detect_db: 12.0,
        ..Default::default()
    };
    let mut c = SafetyConfig::default();
    c.apply_params(&p);
    assert_eq!((c.floor_db, c.detect_db), (-20.0, 12.0));
}

/// 数值默认值的正本是 `vdn_types::desc`：SafetyConfig 的默认与表逐条相同，且与 ParamSet 的默认一致（同一条描述）。
#[test]
fn safety_defaults_follow_desc_table() {
    use vdn_types::desc::*;
    let c = SafetyConfig::default();
    let pairs = [
        (c.max_abs, &ENGINE_MAX_ABS),
        (c.recover_hold_s, &ENGINE_RECOVER_HOLD_S),
        (c.crossfade_s, &ENGINE_CROSSFADE_S),
        (c.floor_db, &FLOOR_DB),
        (c.open_db, &OPEN_DB),
        (c.attack_s, &ATTACK_S),
        (c.release_s, &RELEASE_S),
        (c.hangover_s, &HANGOVER_S),
        (c.detect_db, &DETECT_DB),
    ];
    for (v, d) in pairs {
        assert_eq!(v, d.default, "{}", d.key);
    }
    let mut from_set = SafetyConfig::off();
    from_set.apply_params(&vdn_types::ParamSet::DEFAULT);
    assert_eq!(
        (
            from_set.floor_db,
            from_set.open_db,
            from_set.detect_db,
            from_set.release_s,
            from_set.hangover_s
        ),
        (
            c.floor_db,
            c.open_db,
            c.detect_db,
            c.release_s,
            c.hangover_s
        )
    );
}

/// 描述表允许的边界值都能进 Engine（`Engine::new` 断言 `max_abs > 0`、时长 ≥ 0；codex 第 1 轮 P2 #4）。
#[test]
fn desc_boundary_values_are_accepted_by_engine() {
    use vdn_types::desc::*;
    for (max_abs, hold, xf) in [
        (ENGINE_MAX_ABS.min, ENGINE_RECOVER_HOLD_S.min, ENGINE_CROSSFADE_S.min),
        (ENGINE_MAX_ABS.max, ENGINE_RECOVER_HOLD_S.max, ENGINE_CROSSFADE_S.max),
    ] {
        let cfg = SafetyConfig {
            max_abs,
            recover_hold_s: hold,
            crossfade_s: xf,
            ..SafetyConfig::default()
        };
        let mut e = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg);
        let run = offline::run(&mut e, &vec![0.1f32; HOP * 8], 0);
        assert!(run.output.iter().all(|v| v.is_finite()));
    }
}

/// 越界配置进 Engine 时按描述表夹值：Engine 自己与 Pipeline 读到的 `config()` 是同一个值（codex 第 2 轮 P2 #2）。
#[test]
fn engine_clamps_config_by_desc_table() {
    use vdn_types::desc::*;
    let cfg = SafetyConfig {
        crossfade_s: ENGINE_CROSSFADE_S.max + 1.0,
        max_abs: ENGINE_MAX_ABS.min - 0.5,
        floor_db: FLOOR_DB.min - 10.0,
        release_s: f32::NAN,
        ..SafetyConfig::default()
    };
    let e = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg);
    let c = e.config();
    assert_eq!(c.crossfade_s, ENGINE_CROSSFADE_S.max);
    assert_eq!(c.max_abs, ENGINE_MAX_ABS.min);
    assert_eq!(c.floor_db, FLOOR_DB.min);
    assert_eq!(c.release_s, RELEASE_S.default);
}
