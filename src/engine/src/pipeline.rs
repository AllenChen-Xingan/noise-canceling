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
//! `Pipeline::new` / `with_block` 不带级（`S = ()`），`Pipeline::with_stages` 带级。
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

use core::fmt;

use vdn_dsp::resample::{self, Decimator3, FACTOR, Interpolator3};
use vdn_dsp::{DelayLine, FtzGuard};
use vdn_types::{
    Basis, BypassReason, FrameCtx, LatencyItem, LatencyLedger, Model, ParamSet, SampleClock, StageChain, Status,
};

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

/// 构造失败。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineError {
    /// IO 采样率必须正好是模型采样率的 3 倍（本批只有固定 3 倍重采样）
    UnsupportedRate { io_rate: u32, model_rate: u32 },
    /// `Pipeline::with_block` 的块长必须 > 0 且整除 48 kHz hop
    BlockNotAligned { block: usize, hop_io: usize },
    /// 后处理级的延迟必须为 0（规约 §3.3：「每级 dry 对齐」做出来之前）
    StageLatency { stage: &'static str, samples: usize },
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::UnsupportedRate {
                io_rate,
                model_rate,
            } => write!(
                f,
                "IO 采样率 {io_rate} Hz 不是模型采样率 {model_rate} Hz 的 3 倍（本批只支持固定 3 倍重采样）"
            ),
            PipelineError::BlockNotAligned { block, hop_io } => write!(
                f,
                "块长 {block} 不整除 IO 采样率下的 hop {hop_io}（定块长省延迟要求块长 > 0 且整除 hop）"
            ),
            PipelineError::StageLatency { stage, samples } => write!(
                f,
                "后处理级 {stage} 声明延迟 {samples} 个样本；本批只许 0 延迟的级（规约 §3.3）"
            ),
        }
    }
}

impl std::error::Error for PipelineError {}

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
        Self::with_stages(engine, io_rate, None, ())
    }

    /// 定块长构造（省延迟）：调用方保证每次 `process` 的块长都是 `block`。`block` 必须 > 0 且整除
    /// `hop_io`（= 模型 hop × 3），否则返回 `BlockNotAligned`。输出预填 `hop_io - block`，比 `new` 少 `block - 1` 个样本。
    /// 块长不对时不 panic，见 `process`。
    pub fn with_block(
        engine: Engine<M>,
        io_rate: u32,
        block: usize,
    ) -> Result<Self, PipelineError> {
        Self::with_stages(engine, io_rate, Some(block), ())
    }
}

// 构造：非实时线程（会分配）
impl<M: Model, S: StageChain> Pipeline<M, S> {
    /// 带后处理级构造：`block` 为 `None` 同 `new`（任意块长），`Some(b)` 同 `with_block`。
    /// `stages` 按顺序跑在模型之后（`vdn_types::stages![a, b]`）；任一级 `latency_samples() != 0` 返回 `StageLatency`。
    pub fn with_stages(
        engine: Engine<M>,
        io_rate: u32,
        block: Option<usize>,
        stages: S,
    ) -> Result<Self, PipelineError> {
        let model_rate = engine.sample_rate();
        if model_rate == 0 || u64::from(io_rate) != u64::from(model_rate) * FACTOR as u64 {
            return Err(PipelineError::UnsupportedRate {
                io_rate,
                model_rate,
            });
        }
        let mut bad = None;
        stages.describe(&mut |stage, samples| {
            if samples != 0 && bad.is_none() {
                bad = Some(PipelineError::StageLatency { stage, samples });
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
        let xf = (engine.config().crossfade_s.clamp(0.0, 1.0) * io_rate as f32).max(1.0);
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
        self.stages.describe(&mut |stage, samples| {
            l.push(LatencyItem::new(
                stage,
                (samples * FACTOR) as u32,
                rate,
                Basis::Declared,
                "后处理级（Stage::latency_samples × 3；本批只许 0）",
            ));
        });
        l
    }
}

// 实时（规约 §5.3、R1–R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<M: Model, S: StageChain> Pipeline<M, S> {
    /// 整条链的延迟（IO 采样率样本）。两种模式相同。
    pub fn latency_samples(&self) -> usize {
        self.latency
    }

    pub fn io_rate(&self) -> u32 {
        self.io_rate
    }

    /// 48 kHz hop（= 模型 hop × 3）。
    pub fn hop_io(&self) -> usize {
        self.hop_io
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// 切换模式：从下一个样本起，按一个 48 kHz hop 的时长线性淡化过去。
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn stats(&self) -> PipelineStats {
        self.stats
    }

    pub fn engine(&self) -> &Engine<M> {
        &self.engine
    }

    pub fn stages(&self) -> &S {
        &self.stages
    }

    /// 把策略参数下发给每个后处理级（实时线程可调，不分配）。
    pub fn set_stage_params(&mut self, p: &ParamSet) {
        self.stages.set_params_all(p);
    }

    /// 最近一个 48k hop 的状态（**输入侧**：刚攒满、跑完推理的那个 hop，比耳朵听到的早约一个 hop；一次调用跑了几个 hop
    /// 时只留最后一个——要数次数看 `engine().stats()` 与 `stats().stage_faults`）：Engine 回报的 `Ok` / `Bypassed` / `Holding`；Engine 已恢复（`Ok`）但
    /// Pipeline 还在把 48 kHz 原声淡回模型输出时报 `Crossfading`。
    pub fn last_status(&self) -> Status {
        match self.last_status {
            Status::Ok | Status::Crossfading if self.fb_ratio > 0.0 => Status::Crossfading,
            Status::Crossfading => Status::Ok,
            s => s,
        }
    }

    /// 清空所有状态（含 Engine 与模型），回到刚构造时；模式保持，淡化直接落到目标。
    pub fn reset(&mut self) {
        self.engine.reset();
        self.stages.reset_all();
        self.q_bypass.fill(false);
        self.fb_ratio = 0.0;
        self.seam = 0.0;
        self.last_mix = 0.0;
        self.last_dry = 0.0;
        self.last_status = Status::Ok;
        self.prev_bypass = false;
        // clock 不归零：流时间单调，reset 前发出的声纹判决的时间戳不会跑到 now 之后
        self.dec.reset();
        self.int.reset();
        self.acc.fill(0.0);
        self.acc_len = 0;
        self.q.fill(0.0);
        self.q_read = 0;
        self.q_len = self.prefill;
        self.dry_line.reset();
        self.fade_pos = self.fade_target();
    }

    /// 处理任意长度的一块。`output.len()` 应等于 `input.len()`（调用方的 bug：debug 下 panic；
    /// release 下只处理较短的那段，`output` 多出的部分清零并计数，不 panic）。
    ///
    /// `with_block` 构造时块长不等于 `block`：不 panic，计入 `stats().misaligned_blocks`，照常处理（宁可抖动也不断声）。
    /// 这时输出队列可能欠载，欠载的样本补零（计入 `queue_faults`），之后降噪路比声明延迟晚最多 `block - 1` 个样本、
    /// 与原声路错位，直到 `reset()`。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.process_ctx(input, output, FrameCtx::default());
    }

    /// 同 `process`，另给后处理级一份上下文（声纹判决、人声概率）。`ctx.now` 不用：Pipeline 每 hop 按自己的样本计数填。
    pub fn process_ctx(&mut self, input: &[f32], output: &mut [f32], ctx: FrameCtx) {
        self.ctx = ctx;
        debug_assert_eq!(input.len(), output.len());
        let n = input.len().min(output.len());
        if self.block.is_some_and(|b| b != n) {
            self.stats.misaligned_blocks += 1;
        }
        let (head, tail) = output.split_at_mut(n);
        if !tail.is_empty() || input.len() != n {
            self.stats.bad_len += 1;
        }
        tail.fill(0.0);
        let (mut src, mut dst) = (input.get(..n).unwrap_or(&[]), head);
        while !src.is_empty() {
            let k = src.len().min(self.hop_io - self.acc_len);
            let (xi, rest_in) = src.split_at(k);
            let (yi, rest_out) = dst.split_at_mut(k);
            self.step(xi, yi);
            src = rest_in;
            dst = rest_out;
        }
    }

    /// `x.len() == y.len() <= hop_io - acc_len`。
    fn step(&mut self, x: &[f32], y: &mut [f32]) {
        let k = x.len();
        if let Some(slot) = self.acc.get_mut(self.acc_len..self.acc_len + k) {
            slot.copy_from_slice(x);
            for v in slot.iter_mut().filter(|v| !v.is_finite()) {
                *v = 0.0;
                self.stats.bad_input += 1;
            }
        }
        self.acc_len += k;
        self.clock += k as u64;
        if self.acc_len == self.hop_io {
            self.acc_len = 0;
            self.run_hop();
        }

        let _ftz = FtzGuard::new();
        self.pop(y);
        let target = self.fade_target();
        let full = self.hop_io;
        let dry = self.dry.get_mut(..k).unwrap_or(&mut []);
        self.dry_line.process(x, dry);
        let flags = self.out_bypass.get(..k).unwrap_or(&[]);
        for ((o, d), &bypass) in y.iter_mut().zip(dry.iter_mut()).zip(flags) {
            // 输入的 NaN / Inf 经延迟线到这里，按 0 处理（已在入口计数）
            if !d.is_finite() {
                *d = 0.0;
            }
            let d = *d;
            // 1. 模型级送原声：按 Engine 回报的状态，用 48 kHz 原声切换（规约 §3.3 规则 2、R11）
            if !o.is_finite() {
                *o = d;
            }
            if bypass {
                if self.fb_ratio < 1.0 {
                    // 切到原声：坏输出不能拿来淡化，用接缝偏移从上一个输出样本平滑过渡
                    self.seam = self.last_mix - self.last_dry;
                    if !self.seam.is_finite() {
                        self.seam = 0.0;
                    }
                    self.fb_ratio = 1.0;
                }
                *o = d + self.seam;
                self.seam *= self.seam_decay;
            } else if self.fb_ratio > 0.0 || self.seam != 0.0 {
                // 切回模型：线性交叉淡化
                *o += self.fb_ratio * (d - *o) + self.seam;
                self.fb_ratio = (self.fb_ratio - self.fb_step).max(0.0);
                self.seam *= self.seam_decay;
                if self.seam.abs() < 1e-9 {
                    self.seam = 0.0;
                }
            }
            self.last_mix = *o;
            self.last_dry = d;

            // 2. 模式（降噪 / 直通）淡化
            if self.fade_pos < target {
                self.fade_pos += 1;
            } else if self.fade_pos > target {
                self.fade_pos -= 1;
            }
            if self.fade_pos == 0 {
                continue;
            }
            if self.fade_pos >= full {
                *o = d;
            } else {
                let g = self.fade_pos as f32 / full as f32;
                *o += g * (d - *o);
            }
        }
    }

    fn fade_target(&self) -> usize {
        match self.mode {
            Mode::Denoise => 0,
            Mode::Passthrough => self.hop_io,
        }
    }

    /// 一个 48k hop：降采样 → Engine（推理）→ 后处理级 → 升采样 → 连同状态进输出队列。
    fn run_hop(&mut self) {
        self.stats.hops += 1;
        {
            let _ftz = FtzGuard::new();
            self.dec.process(&self.acc, &mut self.lo_in);
        }
        // 推理在 FTZ 守卫外（R9：守卫开着等于对 ORT 开 FTZ，DeepFilterNet #713）
        let mut status = self.engine.process_status(&self.lo_in, &mut self.lo_out);
        let mut bypass = matches!(status, Status::Bypassed(_) | Status::Holding);
        let ctx = FrameCtx {
            now: SampleClock(self.clock.saturating_sub(self.hop_io as u64)),
            ..self.ctx
        };
        {
            let _ftz = FtzGuard::new();
            self.stages
                .process_all(&mut self.lo_out, self.engine.aligned_dry(), ctx);
            // 后处理级的输出也按 Engine 的规则查（R11）：坏了整 hop 送原声、复位后处理级；坏值不进升采样器
            let cfg = self.engine.config();
            if cfg.fallback && !bypass {
                let mut peak = 0.0f32;
                let mut finite = true;
                for v in &self.lo_out {
                    finite &= v.is_finite();
                    peak = peak.max(v.abs());
                }
                if !finite || peak > cfg.max_abs {
                    status = Status::Bypassed(if finite {
                        BypassReason::Abnormal
                    } else {
                        BypassReason::NonFinite
                    });
                    bypass = true;
                    self.stats.stage_faults += 1;
                    self.lo_out.copy_from_slice(self.engine.aligned_dry());
                    self.stages.reset_all();
                }
            }
            self.int.process(&self.lo_out, &mut self.hi_out);
        }
        self.last_status = status;
        // 切回的第一个 hop：前 2 × delay_high 个样本里还有上一 hop 的 16 kHz 垫底（升采样延迟与拖尾），也标成送原声
        let tail = if self.prev_bypass && !bypass {
            2 * resample::delay_high()
        } else {
            0
        };
        self.prev_bypass = bypass;
        // 入队（容量 2 × hop_io ≥ prefill + hop_io，不会溢出）
        let cap = self.q.len();
        let mut w = (self.q_read + self.q_len) % cap;
        for (j, &v) in self.hi_out.iter().enumerate() {
            if let Some(s) = self.q.get_mut(w) {
                *s = v;
            }
            if let Some(b) = self.q_bypass.get_mut(w) {
                *b = bypass || j < tail;
            }
            w += 1;
            if w == cap {
                w = 0;
            }
        }
        let len = self.q_len + self.hop_io;
        debug_assert!(len <= cap, "输出队列溢出");
        if len > cap {
            // 到不了（见上）；万一溢出，最旧的样本已被覆盖，读指针跟着前移，保持 q 与 q_bypass 顺序一致
            let over = len - cap;
            self.stats.queue_faults += over as u64;
            self.q_read = (self.q_read + over) % cap;
        }
        self.q_len = len.min(cap);
    }

    /// 出队 `y.len()` 个样本，状态写进 `out_bypass` 的前 `y.len()` 个。
    fn pop(&mut self, y: &mut [f32]) {
        let cap = self.q.len();
        for (o, b) in y.iter_mut().zip(self.out_bypass.iter_mut()) {
            // 由预填保证不欠载；万一欠载补零（`new` 构造时不应发生；`with_block` 构造时块长不对会发生，不算 bug）
            debug_assert!(self.q_len > 0 || self.block.is_some(), "输出队列欠载");
            if self.q_len == 0 {
                self.stats.queue_faults += 1;
                *o = 0.0;
                *b = false;
                continue;
            }
            *o = self.q.get(self.q_read).copied().unwrap_or(0.0);
            *b = self.q_bypass.get(self.q_read).copied().unwrap_or(false);
            self.q_read += 1;
            if self.q_read == cap {
                self.q_read = 0;
            }
            self.q_len -= 1;
        }
    }
}
