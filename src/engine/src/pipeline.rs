//! `Pipeline`：IO 采样率（48 kHz）、任意块长的处理入口（规约 §3.3）。
//!
//! 一个样本的路径：
//! ```text
//! 48k 输入 ─┬→ 攒满一个 48k hop（= 模型 hop × 3）→ 降采样 → Engine::process → 升采样 → 输出队列 ─┐
//!           └→ 延迟线（L_total）→ 48k 原声 ───────────────────────────────────────────────────────┤
//!                                                         按模式混合（切换时按一个 48k hop 线性淡化）→ 48k 输出
//! ```
//!
//! 延迟（48 kHz 样本，逐项硬加，`latency_samples()`）：
//! - 重分块：输出队列开头预填 `hop_io - 1` 个零。逐样本看，第 t 个输入进来后、取第 t 个输出前，队列里至少剩 1 个样本，
//!   所以任意块长都不会欠载；块内先攒后取与逐样本交替等价，因此**输出与切块方式无关**（规约 G6）
//!   - 定块长（`Pipeline::with_block`，人定 2026-11-02「可以省」）：调用方保证每次块长都是 `block`、且 `block` 整除
//!     `hop_io`，预填降到 `hop_io - block`（块长 96、GTCRN 时省 95 个样本 ≈ 2 ms）。块边界和 hop 边界对齐，攒满 hop 的
//!     那一块里先跑 hop 再取输出，所以同样不欠载。块长不对时照常处理、不 panic，但可能欠载补零（见 `process`）
//! - 两次重采样：`2 × resample::delay_high()`
//! - 模型：`Model::latency_samples() × 3`
//!
//! - 后处理级：本批只许 0
//!
//! 和规约 §3.3 示意的差别：`process` 不收 `FrameCtx`（与 010 第十节一致）；要给后处理级声纹判决等上下文用 `process_ctx`，
//! 其中 `now` 由 Pipeline 按自己的样本计数填（本 hop 首样本）。没有 `max_block`（缓冲只按 hop 定）。
//! 延迟账口径：`latency_ledger()` 里的重采样是代码声明的 2 ms；`vdn_types::latency::planned_platform()` 里还按规划预算记着
//! 重采样 4 ms，两本账不要直接 `extend` 到一起，否则重采样记两次（拼平台项时去掉预算里的重采样项）。
//!
//! 后处理级（规约 §3.3 · D66「处理图」第一步）：模型之后、升采样之前，按顺序跑一串 [`Stage`](vdn_types::Stage)
//! （[`StageChain`]，`stages![a, b]`，静态分发、不分配），跑在模型采样率，`dry` 是 Engine 按模型延迟对齐的 16 kHz 原声。
//! 本批只许 0 延迟的级：构造时有级的 `latency_samples() != 0` 就返回 `PipelineError::StageLatency`（「每级 dry 对齐」做出来前）。
//! `Pipeline::new` / `with_block` 不带级（`S = ()`），`Pipeline::with_stages` 带级，并要一份处理图描述（`vdn_types::graph`）：
//! 构造时把级串 `describe()` 报的种类 · 顺序 · 延迟与描述逐级比对（`vdn_types::match_chain`），不一致返回
//! `PipelineError::Graph`。比对只在构造期做，实时 `process` 不多一步。
//!
//! 送原声（规约 §3.3 规则 2、§3.6、R11）：两种模式延迟相同（原声走同一个 `L_total` 延迟线）。模型级故障由
//! `Engine::process_status` 判定并回报 [`Status`]，**Pipeline 用 48 kHz 原声切换与淡化**（不用 16 kHz 原声，否则缺 8 kHz 以上）：
//! 每个 48k hop 的状态随它的升采样输出一起进输出队列，出队时逐样本混合——切到原声用接缝偏移衰减（坏输出不能拿来淡化），
//! 切回模型用线性交叉淡化，时长都是 `SafetyConfig::crossfade_s`（按 48 kHz 计）。
//! 状态按 48k hop 对齐，升采样 FIR 的延迟与拖尾（各 48 个样本）让切过去的点最多早 96 个样本；切回时把第一个正常 hop 的前
//! `2 × delay_high` 个样本也标成送原声，淡化从纯模型内容开始。后处理级的输出也查（非有限 / 超 `max_abs`），坏了整 hop 送原声、
//! 后处理级复位（R11）。输入里的 NaN / Inf 按 0 处理，原声路与接缝偏移都不会被它锁死。
//!
//! 已知缺口：衰减下限与快开慢关仍在 Engine 里混 16 kHz 原声（正常降噪时原声成分只到 8 kHz）；等衰减下限移成后处理级时一起改，
//! 用 48 kHz 原声会把 8 kHz 以上没降过的噪声一起放回来，是听感取舍（待人定）。
//!
//! FTZ（R9）：降采样、后处理级、升采样、出队混合各自包在 `vdn_dsp::FtzGuard` 里；`Engine::process_status`（含推理调用）在守卫外。
//!
//! 实时路径（`process` / `process_ctx` / `set_mode` / `reset`）不分配、不加锁、不 panic（`tests/pipeline.rs` 证明不分配）。

mod error;
mod rt;

use vdn_dsp::DelayLine;
use vdn_dsp::resample::{self, Decimator3, FACTOR, Interpolator3};
use vdn_types::{
    Basis, ChainMismatch, CheckedGraph, ContentId, FrameCtx, LatencyItem, LatencyLedger, Model, ModelId,
    NodeKind, ProcessingGraph, StageChain, Status, match_chain,
};

pub use error::PipelineError;

use crate::engine::Engine;

/// 输出哪一路。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mode {
    /// 模型输出（含 Engine 的三件保命）
    #[default]
    Denoise,
    /// 48 kHz 原声经同一个 `L_total` 延迟（模型照常在跑，切回时不用重新预热）
    Passthrough,
}

/// 计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PipelineStats {
    /// 跑过的 48k hop 数
    pub hops: u64,
    /// 输入输出不等长的调用次数（调用方的 bug）
    pub bad_len: u64,
    /// 输出队列欠载或溢出的样本数。`new` 构造时由预填推导不会发生（非 0 说明有 bug）；`with_block` 构造时，
    /// 块长不对会欠载补零并计在这里。欠载后降噪路比原声路晚，直到 `reset()`
    pub queue_faults: u64,
    /// `with_block` 构造时，块长不等于 `block` 的调用次数（`new` 构造时恒为 0）
    pub misaligned_blocks: u64,
    /// 后处理级输出非有限或幅度超 `SafetyConfig::max_abs`、整 hop 改送原声的次数（`fallback` 开时才查）
    pub stage_faults: u64,
    /// 输入里非有限（NaN / Inf）、按 0 处理的样本数
    pub bad_input: u64,
}

/// `S`：后处理级串，默认 `()`（不带级）。
pub struct Pipeline<M: Model, S: StageChain = ()> {
    engine: Engine<M>,
    stages: S,
    /// 构造时比对过的级串描述（处理图的时域段）
    stage_kinds: &'static [&'static NodeKind],
    io_rate: u32,
    /// 48 kHz hop
    hop_io: usize,
    /// `with_block` 的定块长；`None` = 任意块长
    block: Option<usize>,
    prefill: usize,
    latency: usize,

    dec: Decimator3,
    int: Interpolator3,
    /// 攒 48k 输入
    acc: Vec<f32>,
    acc_len: usize,
    /// 16k 降采样后 / 模型输出
    lo_in: Vec<f32>,
    lo_out: Vec<f32>,
    /// 一个 48k hop 的升采样结果
    hi_out: Vec<f32>,
    /// 输出队列（环形，容量 2 × hop_io）
    q: Vec<f32>,
    /// 与 `q` 逐样本对应：这个样本所在 hop 是否该整段送原声（Engine 回报 Bypassed / Holding）
    q_bypass: Vec<bool>,
    /// 出队时取出的状态（长度 hop_io，`step` 里只用前 k 个）
    out_bypass: Vec<bool>,
    q_read: usize,
    q_len: usize,

    dry_line: DelayLine,
    dry: Vec<f32>,

    /// 上一个入队的 hop 是否送原声（切回时把升采样拖尾也标成送原声）
    prev_bypass: bool,
    /// 模型级送原声（48 kHz）：原声占比 0..=1、每样本淡化步长、接缝偏移及其衰减、上一个样本的混合结果与原声
    fb_ratio: f32,
    fb_step: f32,
    seam: f32,
    seam_decay: f32,
    last_mix: f32,
    last_dry: f32,
    last_status: Status,
    /// 已收到的输入样本数（IO 采样率），给 `FrameCtx::now`
    clock: u64,
    /// 调用方给的上下文（`process_ctx`）；`now` 每 hop 由 Pipeline 填
    ctx: FrameCtx,

    mode: Mode,
    /// 淡化进度 0..=hop_io，原声占比 = fade_pos / hop_io（整数计数，淡化正好 hop_io 个样本）
    fade_pos: usize,
    stats: PipelineStats,
}

// 构造：非实时线程（会分配）
impl<M: Model> Pipeline<M> {
    /// `io_rate` 必须是 `engine.sample_rate() × 3`。块长无上限：内部缓冲只按 hop 定，不按块长定。
    pub fn new(engine: Engine<M>, io_rate: u32) -> Result<Self, PipelineError> {
        Self::with_stages(engine, io_rate, None, &[], ())
    }

    /// 定块长构造（省延迟）：调用方保证每次 `process` 的块长都是 `block`。`block` 必须 > 0 且整除
    /// `hop_io`（= 模型 hop × 3），否则返回 `BlockNotAligned`。输出预填 `hop_io - block`，比 `new` 少 `block - 1` 个样本。
    /// 块长不对时不 panic，见 `process`。
    pub fn with_block(
        engine: Engine<M>,
        io_rate: u32,
        block: usize,
    ) -> Result<Self, PipelineError> {
        Self::with_stages(engine, io_rate, Some(block), &[], ())
    }
}

// 构造：非实时线程（会分配）
impl<M: Model, S: StageChain> Pipeline<M, S> {
    /// 带后处理级构造：`block` 为 `None` 同 `new`（任意块长），`Some(b)` 同 `with_block`。
    /// `stages` 按顺序跑在模型之后（`vdn_types::stages![a, b]`）；`stage_kinds` 是它在处理图里的描述（时域段，
    /// 如 `ProcessingGraph::stages`），逐级比对种类 · 顺序 · 延迟，不一致返回 `Graph`；
    /// 任一级 `latency_samples() != 0` 返回 `StageLatency`。
    pub fn with_stages(
        engine: Engine<M>,
        io_rate: u32,
        block: Option<usize>,
        stage_kinds: &'static [&'static NodeKind],
        stages: S,
    ) -> Result<Self, PipelineError> {
        let model_rate = engine.sample_rate();
        if model_rate == 0 || u64::from(io_rate) != u64::from(model_rate) * FACTOR as u64 {
            return Err(PipelineError::UnsupportedRate {
                io_rate,
                model_rate,
            });
        }
        match_chain(stage_kinds, &stages).map_err(PipelineError::Graph)?;
        let mut bad = None;
        stages.describe(&mut |kind, samples| {
            if samples != 0 && bad.is_none() {
                bad = Some(PipelineError::StageLatency {
                    stage: kind.name,
                    samples,
                });
            }
        });
        if let Some(e) = bad {
            return Err(e);
        }
        let hop_model = engine.hop();
        let hop_io = hop_model * FACTOR;
        let prefill = match block {
            None => hop_io - 1,
            Some(b) if b > 0 && hop_io.is_multiple_of(b) => hop_io - b,
            Some(b) => return Err(PipelineError::BlockNotAligned { block: b, hop_io }),
        };
        let latency = prefill
            + 2 * resample::delay_high()
            + (engine.latency_samples() + stages.latency_samples_all()) * FACTOR;
        // 淡化时长限在 [0, 1] 秒：Inf / 极大值会让淡化永不收敛，NaN 落到 1 个样本
        let xf = (vdn_types::desc::ENGINE_CROSSFADE_S.clamp(engine.config().crossfade_s)
            * io_rate as f32)
            .max(1.0);
        let mut p = Self {
            io_rate,
            hop_io,
            block,
            prefill,
            latency,
            dec: Decimator3::new(),
            int: Interpolator3::new(),
            acc: vec![0.0; hop_io],
            acc_len: 0,
            lo_in: vec![0.0; hop_model],
            lo_out: vec![0.0; hop_model],
            hi_out: vec![0.0; hop_io],
            q: vec![0.0; 2 * hop_io],
            q_bypass: vec![false; 2 * hop_io],
            out_bypass: vec![false; hop_io],
            q_read: 0,
            q_len: 0,
            dry_line: DelayLine::new(latency),
            dry: vec![0.0; hop_io],
            prev_bypass: false,
            fb_ratio: 0.0,
            fb_step: 1.0 / xf,
            seam: 0.0,
            // 接缝偏移在 crossfade_s 内衰减到约 1%（同 Engine）
            seam_decay: (0.01f32.ln() / xf).exp(),
            last_mix: 0.0,
            last_dry: 0.0,
            last_status: Status::Ok,
            clock: 0,
            ctx: FrameCtx::default(),
            mode: Mode::Denoise,
            fade_pos: 0,
            stats: PipelineStats::default(),
            engine,
            stages,
            stage_kinds,
        };
        p.reset();
        Ok(p)
    }

    /// 延迟账（48 kHz 口径）：重分块 + 两次重采样 + 模型算法延迟。
    /// Pipeline 的重分块就是 Engine 账上「攒满一个 hop」那段等待，同一段等待只记一次，所以不照搬 Engine 的 hop-buffer 项。
    /// 合计 = `latency_samples()`（单测核对）。构造期调用（会分配）。
    pub fn latency_ledger(&self) -> LatencyLedger {
        let mut l = LatencyLedger::new();
        l.push(LatencyItem::new(
            "rebuffer",
            self.prefill as u32,
            self.io_rate,
            Basis::Declared,
            match self.block {
                None => "攒满一个 48 kHz hop（模型 hop × 3）；输出队列预填 hop - 1",
                Some(_) => "攒满一个 48 kHz hop（模型 hop × 3）；定块长，输出队列预填 hop - 块长",
            },
        ))
        .push(LatencyItem::new(
            "resample",
            (2 * resample::delay_high()) as u32,
            self.io_rate,
            Basis::Declared,
            "48→16 + 16→48，各 (TAPS-1)/2",
        ))
        .push(LatencyItem::new(
            self.engine.model().name(),
            (self.engine.latency_samples() * FACTOR) as u32,
            self.io_rate,
            Basis::Declared,
            "模型算法延迟（Model::latency_samples × 3）",
        ));
        let rate = self.io_rate;
        self.stages.describe(&mut |kind, samples| {
            l.push(LatencyItem::new(
                kind.name,
                (samples * FACTOR) as u32,
                rate,
                Basis::Declared,
                "后处理级（Stage::latency_samples × 3；本批只许 0）",
            ));
        });
        l
    }

    /// 构造时比对过的级串描述（处理图的时域段）。
    pub fn stage_kinds(&self) -> &'static [&'static NodeKind] {
        self.stage_kinds
    }

    /// 这条 Pipeline 对应的处理图。模型节点由调用方给：`model` 与 `artifact`（权重的制品内容 ID，
    /// `vdn_models::artifact_id`）；Pipeline 只认 `Model` trait，所以在这里把 `model` 与正在跑的 `Model::name()`
    /// 比对，不符返回错误（#68）。频域段由模型适配器负责，这里留空。会话记录头用它的内容 ID。构造期用。
    pub fn graph(
        &self,
        model: ModelId,
        artifact: ContentId,
    ) -> Result<CheckedGraph, ChainMismatch> {
        ProcessingGraph {
            stages: self.stage_kinds,
            ..ProcessingGraph::bare(model, artifact)
        }
        .verify(self.engine.model(), &self.stages)
    }
}
