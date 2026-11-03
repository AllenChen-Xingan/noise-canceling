//! Pipeline 的 48 kHz 送原声（规约 §3.3 规则 2 · R11）：故障区间、无咔哒、后处理级坏输出、NaN 输入（用假模型）。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use vdn_engine::testing::DelayGain;
use vdn_engine::{
    Engine, FaultInjector, FaultKind, FrameCtx, Pipeline, SafetyConfig, Stage, Status, stages,
};
use vdn_types::NodeKind;
use vdn_types::graph::test_kind;

mod common;

use common::{HOP_IO, SR, energy, run, signal};

const BAD: NodeKind = test_kind(0xF104, "bad", 0);

/// 只开「出事送原声」：衰减下限、快开慢关都关，模型输出全零 → 正常时输出 ≈ 0，送原声时 = 48 kHz 原声。
fn fallback_only() -> SafetyConfig {
    SafetyConfig {
        floor: false,
        fast_open: false,
        deadline: Some(Duration::from_millis(20)),
        ..SafetyConfig::default()
    }
}

/// 故障 hop `h`（及其保持期 `hold` 个 hop）在输出里占的区间。
fn bypass_span(p_prefill: usize, h: usize, hold: usize) -> std::ops::Range<usize> {
    let start = h * HOP_IO + p_prefill;
    start..start + (1 + hold) * HOP_IO
}

/// 跑一遍故障计划，核对送原声区间。`block` = 每次喂的块长；`fixed` = 用 `with_block(block)` 构造。
fn check_fault_spans(fixed: bool, block: usize) {
    // hop 20 NaN、45 后端报错、70 幅度异常、95 超时（超时不复位、不保持）
    let plan = vec![
        (20, FaultKind::Nan),
        (45, FaultKind::Error),
        (70, FaultKind::Explode),
        (95, FaultKind::Stall(Duration::from_millis(40))),
    ];
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.0), plan);
    let cfg = fallback_only();
    // 保持期 hop 数，算法同 Engine::new
    let hold = (cfg.recover_hold_s * 16_000.0 / 256.0).ceil() as usize;
    let (mut p, prefill) = if fixed {
        (
            Pipeline::with_block(Engine::new(m, cfg), SR, block).unwrap(),
            HOP_IO - block,
        )
    } else {
        (Pipeline::new(Engine::new(m, cfg), SR).unwrap(), HOP_IO - 1)
    };
    let l = p.latency_samples();
    let x = signal(110 * HOP_IO, 0.1);
    let mut y = vec![0.0f32; x.len()];
    let mut statuses = Vec::new();
    for (xi, yi) in x.chunks(block).zip(y.chunks_mut(block)) {
        p.process(xi, yi);
        statuses.push(p.last_status());
    }
    let tag = format!("fixed={fixed} block={block}");
    assert!(y.iter().all(|v| v.is_finite()), "{tag}");
    // 全程幅度上限：坏值（1e3、NaN）没有从任何地方漏出来
    let peak = |v: &[f32]| v.iter().fold(0.0f32, |a, b| a.max(b.abs()));
    assert!(
        peak(&y) <= peak(&x) * 1.1,
        "{tag}：输出峰值 {} > 原声 {}",
        peak(&y),
        peak(&x)
    );
    for r in [
        vdn_types::BypassReason::NonFinite,
        vdn_types::BypassReason::Backend,
        vdn_types::BypassReason::Abnormal,
        vdn_types::BypassReason::Deadline,
    ] {
        assert!(statuses.contains(&Status::Bypassed(r)), "{tag}：没报 {r:?}");
    }
    assert!(statuses.contains(&Status::Holding), "{tag}");
    assert!(statuses.contains(&Status::Crossfading), "{tag}");

    let spans = [
        bypass_span(prefill, 20, hold),
        bypass_span(prefill, 45, hold),
        bypass_span(prefill, 70, hold),
        bypass_span(prefill, 95, 0),
    ];
    for s in &spans {
        // 接缝偏移 5 ms 衰减到 1%，跳过前 10 ms（480 个样本）后应逐样本等于延迟 L 的 48 kHz 原声；
        // 区间末尾多 96 个样本（切回 hop 开头的升采样拖尾）也标成了送原声
        let inner = s.start + 480..s.end + 96;
        for t in inner.clone() {
            assert!(
                (y[t] - x[t - l]).abs() < 1e-4,
                "{tag} 区间 {s:?} 样本 {t}：{} vs 原声 {}",
                y[t],
                x[t - l]
            );
        }
        // 15 kHz 分量还在：16 kHz 原声没有它，这条证明用的是 48 kHz 原声
        let hf_in: f64 = energy(&x[inner.start - l..inner.end - l]);
        let hf_out: f64 = energy(&y[inner.clone()]);
        assert!(
            (hf_out / hf_in - 1.0).abs() < 1e-3,
            "{tag} 能量比 {}",
            hf_out / hf_in
        );
    }
    // 送原声区间之外（留出淡化）输出接近 0（模型输出全零）
    let quiet = spans[0].end + 2 * HOP_IO..spans[1].start - HOP_IO;
    assert!(
        energy(&y[quiet.clone()]) < 1e-6 * energy(&x[quiet]),
        "{tag}"
    );
}

#[test]
fn faults_switch_to_aligned_48k_dry_with_high_band() {
    check_fault_spans(true, 96);
    check_fault_spans(false, 96);
    check_fault_spans(false, 333);
}

#[test]
fn switching_to_and_from_dry_has_no_click() {
    let plan = vec![(20, FaultKind::Nan), (60, FaultKind::Error)];
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.3), plan);
    let mut p = Pipeline::new(Engine::new(m, fallback_only()), SR).unwrap();
    // 不加高频分量：原声相邻差小，判据才够紧（带 15 kHz 时硬切也可能过）
    let x = signal(80 * HOP_IO, 0.0);
    let y = run(&mut p, &x, 512);
    let step = |v: &[f32]| {
        v.windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0f32, f32::max)
    };
    let (dx, dy) = (step(&x), step(&y));
    // 切换处没有跳变：相邻样本差不超过原声本身的最大差（留 10% 余量）
    assert!(dy <= dx * 1.1, "最大相邻差 {dy} > 原声 {dx} × 1.1");
}

#[test]
fn no_fallback_never_switches() {
    let plan = vec![(10, FaultKind::Explode)];
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.0), plan);
    let mut p = Pipeline::new(Engine::new(m, SafetyConfig::off()), SR).unwrap();
    let x = signal(20 * HOP_IO, 0.0);
    let mut y = vec![0.0f32; x.len()];
    for (xi, yi) in x.chunks(HOP_IO).zip(y.chunks_mut(HOP_IO)) {
        p.process(xi, yi);
        assert_eq!(p.last_status(), Status::Ok);
    }
}

/// 输出 NaN 或巨大值的后处理级（从第 `from` 次调用起）。
struct BadStage {
    calls: usize,
    from: usize,
    value: f32,
    resets: Arc<AtomicU64>,
}
impl Stage for BadStage {
    fn kind(&self) -> &'static NodeKind {
        &BAD
    }
    fn reset(&mut self) {
        self.resets.fetch_add(1, Ordering::Relaxed);
    }
    fn process(&mut self, wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {
        self.calls += 1;
        if self.calls == self.from {
            wet.fill(self.value);
        }
    }
}

#[test]
fn bad_stage_output_switches_to_dry_and_resets_stages() {
    for (value, reason) in [
        (f32::NAN, vdn_types::BypassReason::NonFinite),
        (1e3, vdn_types::BypassReason::Abnormal),
    ] {
        let resets = Arc::new(AtomicU64::new(0));
        let stage = BadStage {
            calls: 0,
            from: 10,
            value,
            resets: resets.clone(),
        };
        let mut p = Pipeline::with_stages(
            Engine::new(DelayGain::new(256, 256, 0.0), fallback_only()),
            SR,
            Some(96),
            &[&BAD],
            stages![stage],
        )
        .unwrap();
        let resets_after_new = resets.load(Ordering::Relaxed);
        let x = signal(20 * HOP_IO, 0.1);
        let mut y = vec![0.0f32; x.len()];
        let mut seen = false;
        for (xi, yi) in x.chunks(96).zip(y.chunks_mut(96)) {
            p.process(xi, yi);
            seen |= p.last_status() == Status::Bypassed(reason);
        }
        assert!(seen, "{value}：没报 {reason:?}");
        assert_eq!(p.stats().stage_faults, 1);
        assert_eq!(
            resets.load(Ordering::Relaxed),
            resets_after_new + 1,
            "后处理级应复位一次"
        );
        let peak = y.iter().fold(0.0f32, |a, b| a.max(b.abs()));
        assert!(
            y.iter().all(|v| v.is_finite()) && peak < 1.0,
            "{value}：坏值漏出"
        );
        // 第 10 个 hop（序号 9）整段 = 48 kHz 原声
        let l = p.latency_samples();
        let s = bypass_span(HOP_IO - 96, 9, 0);
        for t in s.start + 480..s.end {
            assert!((y[t] - x[t - l]).abs() < 1e-4, "{value} 样本 {t}");
        }
    }
}

#[test]
fn nan_input_does_not_lock_output() {
    let plan = vec![(12, FaultKind::Error)];
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.5), plan);
    let mut p = Pipeline::new(Engine::new(m, fallback_only()), SR).unwrap();
    let mut x = signal(40 * HOP_IO, 0.0);
    // 让坏输入经延迟线正好在切到原声附近出现
    let l = p.latency_samples();
    for t in [12 * HOP_IO + HOP_IO - 1 - l, 5 * HOP_IO, 5 * HOP_IO + 1] {
        x[t] = f32::NAN;
    }
    x[7 * HOP_IO] = f32::INFINITY;
    let y = run(&mut p, &x, 96);
    assert!(y.iter().all(|v| v.is_finite()), "输出出现 NaN / Inf");
    assert_eq!(p.stats().bad_input, 4);
    // 最后 5 个 hop 恢复正常：输出 ≈ 0.5 × 延迟原声，不是锁死
    let tail = 35 * HOP_IO..40 * HOP_IO;
    let err: f64 = tail
        .clone()
        .map(|t| f64::from(y[t] - 0.5 * x[t - l]).powi(2))
        .sum();
    assert!(err < 1e-3 * energy(&x[tail]), "尾段没恢复：{err}");
}
