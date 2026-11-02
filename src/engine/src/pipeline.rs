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
//! - 两次重采样：`2 × resample::delay_high()`
//! - 模型：`Model::latency_samples() × 3`
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
}

pub struct Pipeline<M: Model> {
    engine: Engine<M>,
    io_rate: u32,
    /// 48 kHz hop
    hop_io: usize,
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
    /// 原声占比 0..=1（1 = 全原声）
    dry_gain: f32,
    fade_step: f32,
    stats: PipelineStats,
}

// 构造：非实时线程（会分配）
impl<M: Model> Pipeline<M> {
    /// `io_rate` 必须是 `engine.sample_rate() × 3`。块长无上限：内部缓冲只按 hop 定，不按块长定。
    pub fn new(engine: Engine<M>, io_rate: u32) -> Result<Self, PipelineError> {
        let model_rate = engine.sample_rate();
        if model_rate == 0 || u64::from(io_rate) != u64::from(model_rate) * FACTOR as u64 {
            return Err(PipelineError::UnsupportedRate {
                io_rate,
                model_rate,
            });
        }
        let hop_model = engine.hop();
        let hop_io = hop_model * FACTOR;
        let prefill = hop_io - 1;
        let latency = prefill + 2 * resample::delay_high() + engine.latency_samples() * FACTOR;
        let mut p = Self {
            io_rate,
            hop_io,
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
            dry_gain: 0.0,
            fade_step: 1.0 / hop_io as f32,
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
            "攒满一个 48 kHz hop（模型 hop × 3）；输出队列预填 hop - 1",
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
        self.dry_gain = match self.mode {
            Mode::Denoise => 0.0,
            Mode::Passthrough => 1.0,
        };
    }

    /// 处理任意长度的一块。`output.len()` 应等于 `input.len()`（调用方的 bug：debug 下 panic；
    /// release 下只处理较短的那段，`output` 多出的部分清零并计数，不 panic）。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert_eq!(input.len(), output.len());
        let n = input.len().min(output.len());
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

        let dry = self.dry.get_mut(..k).unwrap_or(&mut []);
        self.dry_line.process(x, dry);
        let target = match self.mode {
            Mode::Denoise => 0.0,
            Mode::Passthrough => 1.0,
        };
        for (o, d) in y.iter_mut().zip(dry.iter()) {
            if self.dry_gain != target {
                self.dry_gain = if self.dry_gain < target {
                    (self.dry_gain + self.fade_step).min(target)
                } else {
                    (self.dry_gain - self.fade_step).max(target)
                };
            }
            *o += self.dry_gain * (d - *o);
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
        self.q_len = (self.q_len + self.hop_io).min(cap);
    }

    fn pop(&mut self, y: &mut [f32]) {
        let cap = self.q.len();
        for o in y.iter_mut() {
            // 由预填保证不欠载；万一欠载（不应发生）补零
            if self.q_len == 0 {
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
