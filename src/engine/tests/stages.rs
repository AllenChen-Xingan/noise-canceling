//! Pipeline 的后处理级、48 kHz 送原声、FTZ 范围（用假模型，不需要 ONNX Runtime）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

#[cfg(debug_assertions)]
use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_engine::testing::{DelayGain, Gain, Identity};
use vdn_engine::{
    Engine, FaultInjector, FaultKind, FrameCtx, Pipeline, PipelineError, SafetyConfig, Stage,
    StageChain, Status, stages,
};
use vdn_types::{Basis, Confidence, Model, ModelError, SpeakerClass, SpeakerVerdict};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

const SR: u32 = 48_000;
const HOP_IO: usize = 768;

/// 通带内多正弦 + 一个 15 kHz 分量（16 kHz 模型路径过不去，只有 48 kHz 原声里有）。
fn signal(n: usize, hf: f32) -> Vec<f32> {
    let fs = [220.0f32, 523.0, 1187.0, 2213.0, 3301.0, 4519.0, 6007.0];
    (0..n)
        .map(|i| {
            let t = i as f32 / SR as f32;
            let lo: f32 = fs
                .iter()
                .map(|f| 0.05 * (2.0 * std::f32::consts::PI * f * t + f * 0.01).sin())
                .sum();
            lo + hf * (2.0 * std::f32::consts::PI * 15_000.0 * t).sin()
        })
        .collect()
}

fn run<M: Model, S: StageChain>(p: &mut Pipeline<M, S>, x: &[f32], block: usize) -> Vec<f32> {
    let mut y = vec![0.0; x.len()];
    for (xi, yi) in x.chunks(block).zip(y.chunks_mut(block)) {
        p.process(xi, yi);
    }
    y
}

fn energy(v: &[f32]) -> f64 {
    v.iter().map(|s| f64::from(*s).powi(2)).sum()
}

/// 有延迟的级：构造时应被拒绝。
struct Lookahead;
impl Stage for Lookahead {
    fn name(&self) -> &'static str {
        "lookahead"
    }
    fn reset(&mut self) {}
    fn process(&mut self, _wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {}
    fn latency_samples(&self) -> usize {
        16
    }
}

/// wet := dry（证明后处理级拿到的 dry 与 wet 对齐）。
struct UseDry;
impl Stage for UseDry {
    fn name(&self) -> &'static str {
        "use_dry"
    }
    fn reset(&mut self) {}
    fn process(&mut self, wet: &mut [f32], dry: &[f32], _ctx: FrameCtx) {
        wet.copy_from_slice(dry);
    }
}

/// 记下最近一次的 `ctx.now` 与声纹类别。
struct CtxProbe {
    now: Arc<AtomicU64>,
    target: Arc<AtomicBool>,
}
impl Stage for CtxProbe {
    fn name(&self) -> &'static str {
        "ctx_probe"
    }
    fn reset(&mut self) {}
    fn process(&mut self, _wet: &mut [f32], _dry: &[f32], ctx: FrameCtx) {
        self.now.store(ctx.now.0, Ordering::Relaxed);
        self.target
            .store(ctx.speaker.class == SpeakerClass::Target, Ordering::Relaxed);
    }
}

#[test]
fn gain_stages_scale_wet_in_order_and_keep_latency() {
    let x = signal(48_000, 0.0);
    let mut plain = Pipeline::new(Engine::new(Identity, SafetyConfig::off()), SR).unwrap();
    let mut staged = Pipeline::with_stages(
        Engine::new(Identity, SafetyConfig::off()),
        SR,
        None,
        stages![Gain(0.5), Gain(0.5)],
    )
    .unwrap();
    assert_eq!(staged.latency_samples(), plain.latency_samples());
    let a = run(&mut plain, &x, 333);
    let b = run(&mut staged, &x, 333);
    for (i, (u, v)) in a.iter().zip(&b).enumerate() {
        assert!((u * 0.25 - v).abs() < 1e-6, "样本 {i}：{u} × 0.25 ≠ {v}");
    }
}

#[test]
fn empty_stage_chain_is_bit_identical_to_new() {
    let x = signal(30_000, 0.02);
    let mk = || Engine::new(DelayGain::new(256, 256, 0.6), SafetyConfig::default());
    let a = run(&mut Pipeline::new(mk(), SR).unwrap(), &x, 96);
    let b = run(
        &mut Pipeline::with_stages(mk(), SR, None, stages![]).unwrap(),
        &x,
        96,
    );
    assert_eq!(a, b);
}

#[test]
fn stage_with_latency_is_rejected() {
    let r = Pipeline::with_stages(
        Engine::new(Identity, SafetyConfig::off()),
        SR,
        Some(96),
        stages![Gain(1.0), Lookahead],
    );
    assert!(matches!(
        r.err(),
        Some(PipelineError::StageLatency {
            stage: "lookahead",
            samples: 16
        })
    ));
}

#[test]
fn ledger_lists_stages_and_sums_to_latency() {
    let p = Pipeline::with_stages(
        Engine::new(DelayGain::new(256, 256, 1.0), SafetyConfig::default()),
        SR,
        Some(96),
        stages![Gain(1.0), UseDry],
    )
    .unwrap();
    let l = p.latency_ledger();
    let names: Vec<_> = l.items().iter().map(|i| i.component).collect();
    assert_eq!(names, ["rebuffer", "resample", "delay_gain", "gain", "use_dry"]);
    assert!(l.items().iter().all(|i| i.basis == Basis::Declared));
    let ms = p.latency_samples() as f64 / f64::from(SR) * 1000.0;
    assert!((l.total_ms() - ms).abs() < 1e-9);
}

/// 模型输出全零，后处理级把 wet 换成 Engine 给的 16 kHz 对齐原声：输出应是延迟 `latency_samples()` 的输入（通带内）。
#[test]
fn stage_dry_is_aligned_with_declared_latency() {
    let x = signal(48_000, 0.0);
    let mut p = Pipeline::with_stages(
        Engine::new(DelayGain::new(256, 256, 0.0), SafetyConfig::off()),
        SR,
        Some(96),
        stages![UseDry],
    )
    .unwrap();
    let l = p.latency_samples();
    let y = run(&mut p, &x, 96);
    let tail = 4 * l..x.len();
    let err: f64 = tail
        .clone()
        .map(|t| f64::from(y[t] - x[t - l]).powi(2))
        .sum();
    let rel = err / energy(&x[tail]);
    assert!(rel < 1e-3, "相对误差 {rel:e}：后处理级的 dry 没对齐");
}

#[test]
fn stages_get_pipeline_clock_and_caller_ctx() {
    let now = Arc::new(AtomicU64::new(u64::MAX));
    let target = Arc::new(AtomicBool::new(false));
    let probe = CtxProbe {
        now: now.clone(),
        target: target.clone(),
    };
    let mut p = Pipeline::with_stages(
        Engine::new(Identity, SafetyConfig::off()),
        SR,
        None,
        stages![probe],
    )
    .unwrap();
    let ctx = FrameCtx {
        speaker: SpeakerVerdict {
            class: SpeakerClass::Target,
            confidence: Confidence::ONE,
            ..SpeakerVerdict::default()
        },
        ..FrameCtx::default()
    };
    let x = vec![0.0f32; HOP_IO * 5 + 10];
    let mut y = vec![0.0f32; x.len()];
    p.process_ctx(&x, &mut y, ctx);
    // 跑了 5 个 hop，最后一个 hop 首样本 = 4 × 768
    assert_eq!(now.load(Ordering::Relaxed), 4 * HOP_IO as u64);
    assert!(target.load(Ordering::Relaxed));
    p.process(&x[..HOP_IO], &mut y[..HOP_IO]);
    assert_eq!(now.load(Ordering::Relaxed), 5 * HOP_IO as u64);
    assert!(!target.load(Ordering::Relaxed), "process 用默认上下文");
}

// ---------- 送原声：48 kHz（规约 §3.3 规则 2 · R11） ----------

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

#[test]
fn faults_switch_to_aligned_48k_dry_with_high_band() {
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
    let mut p = Pipeline::with_block(Engine::new(m, cfg), SR, 96).unwrap();
    let prefill = HOP_IO - 96;
    let l = p.latency_samples();
    let x = signal(110 * HOP_IO, 0.1);
    let mut y = vec![0.0f32; x.len()];
    let mut statuses = Vec::new();
    for (xi, yi) in x.chunks(96).zip(y.chunks_mut(96)) {
        p.process(xi, yi);
        statuses.push(p.last_status());
    }
    assert!(y.iter().all(|v| v.is_finite()));
    assert!(statuses.contains(&Status::Bypassed(vdn_types::BypassReason::NonFinite)));
    assert!(statuses.contains(&Status::Bypassed(vdn_types::BypassReason::Backend)));
    assert!(statuses.contains(&Status::Bypassed(vdn_types::BypassReason::Abnormal)));
    assert!(statuses.contains(&Status::Bypassed(vdn_types::BypassReason::Deadline)));
    assert!(statuses.contains(&Status::Holding));
    assert!(statuses.contains(&Status::Crossfading));

    let spans = [
        bypass_span(prefill, 20, hold),
        bypass_span(prefill, 45, hold),
        bypass_span(prefill, 70, hold),
        bypass_span(prefill, 95, 0),
    ];
    for s in &spans {
        // 接缝偏移 5 ms 衰减到 1%，跳过前 10 ms（480 个样本）后应逐样本等于延迟 L 的 48 kHz 原声
        let inner = s.start + 480..s.end;
        for t in inner.clone() {
            assert!(
                (y[t] - x[t - l]).abs() < 1e-4,
                "区间 {s:?} 样本 {t}：{} vs 原声 {}",
                y[t],
                x[t - l]
            );
        }
        // 15 kHz 分量还在：16 kHz 原声没有它，这条证明用的是 48 kHz 原声
        let hf_in: f64 = energy(&x[inner.start - l..inner.end - l]);
        let hf_out: f64 = energy(&y[inner.clone()]);
        assert!((hf_out / hf_in - 1.0).abs() < 1e-3, "能量比 {}", hf_out / hf_in);
    }
    // 送原声区间之外（留出淡化）输出接近 0（模型输出全零）
    let quiet = spans[0].end + 2 * HOP_IO..spans[1].start - HOP_IO;
    assert!(energy(&y[quiet.clone()]) < 1e-6 * energy(&x[quiet]));
}

#[test]
fn switching_to_and_from_dry_has_no_click() {
    let plan = vec![(20, FaultKind::Nan), (60, FaultKind::Error)];
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.3), plan);
    let mut p = Pipeline::new(Engine::new(m, fallback_only()), SR).unwrap();
    let x = signal(80 * HOP_IO, 0.05);
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

// ---------- FTZ 只包 DSP、不包推理（R9） ----------

/// 推理调用时记下 FTZ 是否开着。
struct FtzSpyModel {
    seen_on: Arc<AtomicBool>,
    calls: Arc<AtomicU64>,
}
impl Model for FtzSpyModel {
    fn name(&self) -> &'static str {
        "ftz_spy"
    }
    fn sample_rate(&self) -> u32 {
        16_000
    }
    fn hop(&self) -> usize {
        256
    }
    fn latency_samples(&self) -> usize {
        0
    }
    fn reset(&mut self) {}
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        if vdn_dsp::ftz::active() {
            self.seen_on.store(true, Ordering::Relaxed);
        }
        self.calls.fetch_add(1, Ordering::Relaxed);
        output.copy_from_slice(input);
        Ok(())
    }
}

/// 后处理级里记下 FTZ 是否关着。
struct FtzSpyStage {
    seen_off: Arc<AtomicBool>,
    calls: Arc<AtomicU64>,
}
impl Stage for FtzSpyStage {
    fn name(&self) -> &'static str {
        "ftz_spy"
    }
    fn reset(&mut self) {}
    fn process(&mut self, _wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {
        if !vdn_dsp::ftz::active() {
            self.seen_off.store(true, Ordering::Relaxed);
        }
        self.calls.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn ftz_wraps_stages_but_not_inference() {
    let (m_on, m_calls) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicU64::new(0)));
    let (s_off, s_calls) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicU64::new(0)));
    let model = FtzSpyModel {
        seen_on: m_on.clone(),
        calls: m_calls.clone(),
    };
    let stage = FtzSpyStage {
        seen_off: s_off.clone(),
        calls: s_calls.clone(),
    };
    let mut p =
        Pipeline::with_stages(Engine::new(model, SafetyConfig::default()), SR, None, stages![stage])
            .unwrap();
    assert!(!vdn_dsp::ftz::active());
    let x = signal(20 * HOP_IO, 0.0);
    let _ = run(&mut p, &x, 100);
    assert_eq!(m_calls.load(Ordering::Relaxed), 20);
    assert_eq!(s_calls.load(Ordering::Relaxed), 20);
    assert!(!m_on.load(Ordering::Relaxed), "推理调用时 FTZ 开着（违反 R9）");
    if vdn_dsp::ftz::SUPPORTED {
        assert!(!s_off.load(Ordering::Relaxed), "后处理级没在 FTZ 守卫里");
    }
    assert!(!vdn_dsp::ftz::active(), "process 返回后应恢复调用方的浮点模式");
}

#[cfg(debug_assertions)]
#[test]
fn staged_pipeline_with_faults_does_not_allocate() {
    let plan = vec![
        (3, FaultKind::Nan),
        (9, FaultKind::Error),
        (15, FaultKind::Explode),
    ];
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.8), plan);
    let mut p = Pipeline::with_stages(
        Engine::new(m, SafetyConfig::default()),
        SR,
        Some(96),
        stages![Gain(0.9), UseDry, Gain(1.1)],
    )
    .unwrap();
    let x = signal(30 * HOP_IO, 0.05);
    let mut y = vec![0.0f32; x.len()];
    let ctx = FrameCtx::default();
    assert_no_alloc(|| {
        for (xi, yi) in x.chunks(96).zip(y.chunks_mut(96)) {
            p.process_ctx(xi, yi, ctx);
        }
        p.reset();
        let _ = p.last_status();
    });
}
