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
//! 和规约 §3.3 示意的差别：`process` 不收 `FrameCtx`（与 010 第十节一致，接声纹判决时再加）；没有 `max_block`（缓冲只按 hop 定）。
//! 延迟账口径：`latency_ledger()` 里的重采样是代码声明的 2 ms；`vdn_types::latency::planned_platform()` 里还按规划预算记着
//! 重采样 4 ms，两本账不要直接 `extend` 到一起，否则重采样记两次（拼平台项时去掉预算里的重采样项）。
//!
//! 送原声：两种模式延迟相同（原声走同一个 `L_total` 延迟线）。模型级故障的送原声本批仍在 `Engine` 里按 16 kHz 做
//! （p1 现状）；改成「Engine 只回报 `Status`、Pipeline 用 48 kHz 原声切换」要改 `Engine::process` 的签名，归后续批次。
//!
//! 实时路径（`process` / `set_mode` / `reset`）不分配、不加锁、不 panic（`tests/pipeline.rs` 证明不分配）。

use core::fmt;

use vdn_dsp::DelayLine;
use vdn_dsp::resample::{self, Decimator3, FACTOR, Interpolator3};
use vdn_types::{Basis, LatencyItem, LatencyLedger, Model};

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
}

pub struct Pipeline<M: Model> {
    engine: Engine<M>,
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
    q_read: usize,
    q_len: usize,

    dry_line: DelayLine,
    dry: Vec<f32>,

    mode: Mode,
    /// 淡化进度 0..=hop_io，原声占比 = fade_pos / hop_io（整数计数，淡化正好 hop_io 个样本）
    fade_pos: usize,
    stats: PipelineStats,
}

// 构造：非实时线程（会分配）
impl<M: Model> Pipeline<M> {
    /// `io_rate` 必须是 `engine.sample_rate() × 3`。块长无上限：内部缓冲只按 hop 定，不按块长定。
    pub fn new(engine: Engine<M>, io_rate: u32) -> Result<Self, PipelineError> {
        Self::build(engine, io_rate, None)
    }

    /// 定块长构造（省延迟）：调用方保证每次 `process` 的块长都是 `block`。`block` 必须 > 0 且整除
    /// `hop_io`（= 模型 hop × 3），否则返回 `BlockNotAligned`。输出预填 `hop_io - block`，比 `new` 少 `block - 1` 个样本。
    /// 块长不对时不 panic，见 `process`。
    pub fn with_block(
        engine: Engine<M>,
        io_rate: u32,
        block: usize,
    ) -> Result<Self, PipelineError> {
        Self::build(engine, io_rate, Some(block))
    }

    fn build(engine: Engine<M>, io_rate: u32, block: Option<usize>) -> Result<Self, PipelineError> {
        let model_rate = engine.sample_rate();
        if model_rate == 0 || u64::from(io_rate) != u64::from(model_rate) * FACTOR as u64 {
            return Err(PipelineError::UnsupportedRate {
                io_rate,
                model_rate,
            });
        }
        let hop_model = engine.hop();
        let hop_io = hop_model * FACTOR;
        let prefill = match block {
            None => hop_io - 1,
            Some(b) if b > 0 && hop_io.is_multiple_of(b) => hop_io - b,
            Some(b) => return Err(PipelineError::BlockNotAligned { block: b, hop_io }),
        };
        let latency = prefill + 2 * resample::delay_high() + engine.latency_samples() * FACTOR;
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
            q_read: 0,
            q_len: 0,
            dry_line: DelayLine::new(latency),
            dry: vec![0.0; hop_io],
            mode: Mode::Denoise,
            fade_pos: 0,
            stats: PipelineStats::default(),
            engine,
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
        l
    }
}

// 实时（规约 §5.3、R1–R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<M: Model> Pipeline<M> {
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

    /// 清空所有状态（含 Engine 与模型），回到刚构造时；模式保持，淡化直接落到目标。
    pub fn reset(&mut self) {
        self.engine.reset();
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
        }
        self.acc_len += k;
        if self.acc_len == self.hop_io {
            self.acc_len = 0;
            self.run_hop();
        }
        self.pop(y);

        let target = self.fade_target();
        let full = self.hop_io;
        let dry = self.dry.get_mut(..k).unwrap_or(&mut []);
        self.dry_line.process(x, dry);
        for (o, d) in y.iter_mut().zip(dry.iter()) {
            if self.fade_pos < target {
                self.fade_pos += 1;
            } else if self.fade_pos > target {
                self.fade_pos -= 1;
            }
            // 全原声时直接取原声：逐样本精确，模型输出是 NaN / Inf 也漏不过来（R11）；
            // 淡化中模型输出非有限时同样直接送原声
            if self.fade_pos == 0 {
                continue;
            }
            if self.fade_pos >= full || !o.is_finite() {
                *o = *d;
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

    /// 一个 48k hop：降采样 → Engine → 升采样 → 进输出队列。
    fn run_hop(&mut self) {
        self.stats.hops += 1;
        self.dec.process(&self.acc, &mut self.lo_in);
        self.engine.process(&self.lo_in, &mut self.lo_out);
        self.int.process(&self.lo_out, &mut self.hi_out);
        // 入队（容量 2 × hop_io ≥ prefill + hop_io，不会溢出）
        let cap = self.q.len();
        let mut w = (self.q_read + self.q_len) % cap;
        for &v in &self.hi_out {
            if let Some(s) = self.q.get_mut(w) {
                *s = v;
            }
            w += 1;
            if w == cap {
                w = 0;
            }
        }
        let len = self.q_len + self.hop_io;
        debug_assert!(len <= cap, "输出队列溢出");
        if len > cap {
            self.stats.queue_faults += (len - cap) as u64;
        }
        self.q_len = len.min(cap);
    }

    fn pop(&mut self, y: &mut [f32]) {
        let cap = self.q.len();
        for o in y.iter_mut() {
            // 由预填保证不欠载；万一欠载补零（`new` 构造时不应发生；`with_block` 构造时块长不对会发生，不算 bug）
            debug_assert!(self.q_len > 0 || self.block.is_some(), "输出队列欠载");
            if self.q_len == 0 {
                self.stats.queue_faults += 1;
                *o = 0.0;
                continue;
            }
            *o = self.q.get(self.q_read).copied().unwrap_or(0.0);
            self.q_read += 1;
            if self.q_read == cap {
                self.q_read = 0;
            }
            self.q_len -= 1;
        }
    }
}
