//! Pipeline 的后处理级（含与处理图描述的比对）、FTZ 范围、实时不分配（用假模型，不需要 ONNX Runtime）。
//! 48 kHz 送原声在 `fallback.rs`。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[cfg(debug_assertions)]
use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_engine::testing::{DelayGain, GAIN, Gain, Identity};
use vdn_engine::{
    Engine, FaultInjector, FaultKind, FrameCtx, Pipeline, PipelineError, SafetyConfig, Stage,
    StageChain, stages,
};
use vdn_types::graph::test_kind;
use vdn_types::{
    Basis, ChainMismatch, Confidence, Model, ModelError, NodeKind, SpeakerClass, SpeakerVerdict,
};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

mod common;

use common::{HOP_IO, SR, energy, run, signal};

const LOOKAHEAD: NodeKind = test_kind(0xF101, "lookahead", 16);
const USE_DRY: NodeKind = test_kind(0xF102, "use_dry", 0);
const CTX_PROBE: NodeKind = test_kind(0xF103, "ctx_probe", 0);
const FTZ_SPY: NodeKind = test_kind(0xF105, "ftz_spy", 0);

/// 有延迟的级：构造时应被拒绝。
struct Lookahead;
impl Stage for Lookahead {
    fn kind(&self) -> &'static NodeKind {
        &LOOKAHEAD
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
    fn kind(&self) -> &'static NodeKind {
        &USE_DRY
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
    fn kind(&self) -> &'static NodeKind {
        &CTX_PROBE
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
    let x = signal(SR as usize, 0.0);
    let mut plain = Pipeline::new(Engine::new(Identity, SafetyConfig::off()), SR).unwrap();
    let mut staged = Pipeline::with_stages(
        Engine::new(Identity, SafetyConfig::off()),
        SR,
        None,
        &[&GAIN, &GAIN],
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
        &mut Pipeline::with_stages(mk(), SR, None, &[], stages![]).unwrap(),
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
        &[&GAIN, &LOOKAHEAD],
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
        &[&GAIN, &USE_DRY],
        stages![Gain(1.0), UseDry],
    )
    .unwrap();
    let l = p.latency_ledger();
    let names: Vec<_> = l.items().iter().map(|i| i.component).collect();
    assert_eq!(
        names,
        ["rebuffer", "resample", "delay_gain", "gain", "use_dry"]
    );
    assert!(l.items().iter().all(|i| i.basis == Basis::Declared));
    let ms = p.latency_samples() as f64 / f64::from(SR) * 1000.0;
    assert!((l.total_ms() - ms).abs() < 1e-9);
}

/// 模型输出全零，后处理级把 wet 换成 Engine 给的 16 kHz 对齐原声：输出应是延迟 `latency_samples()` 的输入（通带内）。
#[test]
fn stage_dry_is_aligned_with_declared_latency() {
    let x = signal(SR as usize, 0.0);
    let mut p = Pipeline::with_stages(
        Engine::new(DelayGain::new(256, 256, 0.0), SafetyConfig::off()),
        SR,
        Some(96),
        &[&USE_DRY],
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
        &[&CTX_PROBE],
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
        vdn_types::MODEL_RATE
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
    fn kind(&self) -> &'static NodeKind {
        &FTZ_SPY
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
    let (m_on, m_calls) = (
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicU64::new(0)),
    );
    let (s_off, s_calls) = (
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicU64::new(0)),
    );
    let model = FtzSpyModel {
        seen_on: m_on.clone(),
        calls: m_calls.clone(),
    };
    let stage = FtzSpyStage {
        seen_off: s_off.clone(),
        calls: s_calls.clone(),
    };
    let mut p = Pipeline::with_stages(
        Engine::new(model, SafetyConfig::default()),
        SR,
        None,
        &[&FTZ_SPY],
        stages![stage],
    )
    .unwrap();
    assert!(!vdn_dsp::ftz::active());
    let x = signal(20 * HOP_IO, 0.0);
    let _ = run(&mut p, &x, 100);
    assert_eq!(m_calls.load(Ordering::Relaxed), 20);
    assert_eq!(s_calls.load(Ordering::Relaxed), 20);
    assert!(
        !m_on.load(Ordering::Relaxed),
        "推理调用时 FTZ 开着（违反 R9）"
    );
    if vdn_dsp::ftz::SUPPORTED {
        assert!(!s_off.load(Ordering::Relaxed), "后处理级没在 FTZ 守卫里");
    }
    assert!(
        !vdn_dsp::ftz::active(),
        "process 返回后应恢复调用方的浮点模式"
    );
}

/// 调用方线程自己开着 FTZ（宿主、外层守卫）时，推理仍在 FTZ 关着时跑，返回后调用方的状态原样恢复。
#[test]
fn inference_runs_without_ftz_even_if_caller_enabled_it() {
    let (m_on, m_calls) = (
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicU64::new(0)),
    );
    let model = FtzSpyModel {
        seen_on: m_on.clone(),
        calls: m_calls.clone(),
    };
    let mut p = Pipeline::new(Engine::new(model, SafetyConfig::default()), SR).unwrap();
    let x = signal(10 * HOP_IO, 0.0);
    let mut y = vec![0.0f32; x.len()];
    {
        let _caller = vdn_dsp::FtzGuard::new();
        p.process(&x, &mut y);
        assert_eq!(
            vdn_dsp::ftz::active(),
            vdn_dsp::ftz::SUPPORTED,
            "调用方的 FTZ 应原样恢复"
        );
    }
    assert_eq!(m_calls.load(Ordering::Relaxed), 10);
    assert!(
        !m_on.load(Ordering::Relaxed),
        "调用方开着 FTZ 时推理跟着开了（违反 R9）"
    );
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
        &[&GAIN, &USE_DRY, &GAIN],
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

// ---------- 级串与处理图描述比对（构造期，D72） ----------

fn build<S: StageChain>(
    kinds: &'static [&'static NodeKind],
    s: S,
) -> Result<Pipeline<Identity, S>, PipelineError> {
    Pipeline::with_stages(
        Engine::new(Identity, SafetyConfig::off()),
        SR,
        None,
        kinds,
        s,
    )
}

fn graph_err<S: StageChain>(kinds: &'static [&'static NodeKind], s: S) -> ChainMismatch {
    match build(kinds, s) {
        Err(PipelineError::Graph(m)) => m,
        Err(e) => panic!("应报描述不一致，实际 {e}"),
        Ok(_) => panic!("描述不一致却构造成功"),
    }
}

/// 同编号、改了名字：有人在实现里手抄了一份种类
const GAIN_RENAMED: NodeKind = NodeKind {
    name: "gain2",
    ..GAIN
};

/// 运行时延迟与描述不同的级：描述说 0，运行时报 `lat`
struct Lagging(usize);
impl Stage for Lagging {
    fn kind(&self) -> &'static NodeKind {
        &USE_DRY
    }
    fn reset(&mut self) {}
    fn process(&mut self, _wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {}
    fn latency_samples(&self) -> usize {
        self.0
    }
}

struct Renamed;
impl Stage for Renamed {
    fn kind(&self) -> &'static NodeKind {
        &GAIN_RENAMED
    }
    fn reset(&mut self) {}
    fn process(&mut self, _wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {}
}

#[test]
fn pipeline_rejects_missing_stage() {
    assert_eq!(
        graph_err(&[&GAIN, &USE_DRY], stages![Gain(1.0)]),
        ChainMismatch::Missing {
            index: 1,
            expected: "use_dry"
        }
    );
    // 反过来：级串多一级
    assert!(matches!(
        graph_err(&[&GAIN], stages![Gain(1.0), UseDry]),
        ChainMismatch::Extra { index: 1, .. }
    ));
}

#[test]
fn pipeline_rejects_reversed_order() {
    assert_eq!(
        graph_err(&[&GAIN, &USE_DRY], stages![UseDry, Gain(1.0)]),
        ChainMismatch::Kind {
            index: 0,
            expected: "gain",
            actual: "use_dry"
        }
    );
}

#[test]
fn pipeline_rejects_wrong_name() {
    assert_eq!(
        graph_err(&[&GAIN], stages![Renamed]),
        ChainMismatch::Kind {
            index: 0,
            expected: "gain",
            actual: "gain2"
        }
    );
}

#[test]
fn pipeline_rejects_latency_different_from_description() {
    assert_eq!(
        graph_err(&[&USE_DRY], stages![Lagging(8)]),
        ChainMismatch::Latency {
            index: 0,
            stage: "use_dry",
            declared: 0,
            actual: 8
        }
    );
    // 描述与运行时都说有延迟：过了比对，再被「本批只许 0 延迟」拦下
    assert!(matches!(
        build(&[&LOOKAHEAD], stages![Lookahead]),
        Err(PipelineError::StageLatency { samples: 16, .. })
    ));
    let e = build(&[&USE_DRY], stages![Lagging(8)]).err().unwrap();
    assert!(std::error::Error::source(&e).is_some());
    assert!(e.to_string().contains("use_dry"), "{e}");
}
