//! vdn-denoised 的循环体（`Worker`），lib 化是为了让 `vdn chain-test` 在进程内复用同一份代码。
//!
//! - 档：B（软实时）。`Worker::step` 不分配、不加锁、不 panic；推理库内部的分配按 §4 B 档登记
//! - 对 [`Processor`] 泛型：真链路用 `vdn_engine::Pipeline`，单测用假处理器
//! - 直接用 `vdn_shm::Bus`（单测用进程内的 `Bus::heap`）；正式依赖里**不含 `vdn-capture`**（它带设备库，本 crate 禁 cpal），只在 dev 依赖里用它对拍
//!
//! 对齐约定：建总线后先往 Processed 推 `lead` 个零，再逐块推处理结果；总线头写 `L_total = 管线延迟 + lead`。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]

use std::path::Path;

use vdn_engine::{Engine, Model, Pipeline, SafetyConfig};
use vdn_models::{AnyModel, ModelKind};
use vdn_shm::{Bus, BusConfig, RingId};

/// IO 采样率（正本在 vdn-control）
pub use vdn_control::IO_RATE;

/// 一块进、等长一块出，延迟固定（48 kHz 样本）。
pub trait Processor {
    fn process(&mut self, input: &[f32], output: &mut [f32]);
    fn latency_samples(&self) -> usize;
    /// 运行中切直通 / 降噪（实时可调；两模式延迟相同）。默认不支持（什么都不做）
    fn set_mode(&mut self, _mode: vdn_control::Mode) {}
    /// 一个「工作周期」有多少样本：推理每这么多样本做一次（Pipeline = 模型 hop × 3，GTCRN 768 = 16 ms）。
    /// 调度参数要盖住一个周期里的总工作量（1 块推理 + 其余轻处理），不是单块耗时。默认 = 0（不知道，按单块算）
    fn period_samples(&self) -> usize {
        0
    }
}

impl<M: Model> Processor for Pipeline<M> {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        Pipeline::process(self, input, output);
    }
    fn latency_samples(&self) -> usize {
        Pipeline::latency_samples(self)
    }
    fn set_mode(&mut self, mode: vdn_control::Mode) {
        Pipeline::set_mode(self, engine_mode(mode));
    }
    fn period_samples(&self) -> usize {
        Pipeline::hop_io(self)
    }
}

/// `vdn-control` 的模式 → `vdn-engine` 的模式（两边一一对应；正本是 vdn-engine，这里只翻译）
pub fn engine_mode(m: vdn_control::Mode) -> vdn_engine::Mode {
    match m {
        vdn_control::Mode::Denoise => vdn_engine::Mode::Denoise,
        vdn_control::Mode::Passthrough => vdn_engine::Mode::Passthrough,
    }
}

/// 控制线程 → 降噪线程的命令队列（规约 §3.4：rtrb，小 `Copy` 命令；降噪线程每块之间取）
pub fn command_channel(
    capacity: usize,
) -> (
    rtrb::Producer<vdn_control::Cmd>,
    rtrb::Consumer<vdn_control::Cmd>,
) {
    rtrb::RingBuffer::new(capacity.max(1))
}

/// 建管线：`block = Some(b)` 用定块长（`with_block`，每次 `process` 恰好 b 个样本，省延迟）；`None` 任意块长。
fn build<M: Model>(engine: Engine<M>, block: Option<usize>) -> Result<Pipeline<M>, String> {
    match block {
        Some(b) => Pipeline::with_block(engine, IO_RATE, b),
        None => Pipeline::new(engine, IO_RATE),
    }
    .map_err(|e| e.to_string())
}

/// 直通管线（`testing::Identity`，不需要模型权重）。
pub fn identity_pipeline(
    block: Option<usize>,
) -> Result<Pipeline<vdn_engine::testing::Identity>, String> {
    build(
        Engine::new(vdn_engine::testing::Identity, SafetyConfig::off()),
        block,
    )
}

/// 真模型管线（三件保命按默认开）。`path` 不给就按 vdn-models 的默认路径查找。
pub fn model_pipeline(
    kind: ModelKind,
    path: Option<&Path>,
    block: Option<usize>,
) -> Result<Pipeline<AnyModel>, String> {
    let p = match path {
        Some(p) => p.to_path_buf(),
        None => kind
            .default_path()
            .ok_or("找不到模型文件：用 --model-path 指定，或设环境变量 VDN_GTCRN_ONNX / VDN_FASTENHANCER_ONNX")?,
    };
    let model = kind.open(&p).map_err(|e| format!("{}：{e}", p.display()))?;
    build(Engine::new(model, SafetyConfig::default()), block)
}

/// `vdn-control` 的模型名 → `vdn-models` 的类型。
pub fn model_kind(c: vdn_control::ModelChoice) -> ModelKind {
    match c {
        vdn_control::ModelChoice::Gtcrn => ModelKind::Gtcrn,
        vdn_control::ModelChoice::FastenhancerM => ModelKind::FastEnhancerM,
    }
}

/// [`model_kind`] 的反方向（命令行 `--model` 覆盖配置时用）。两边都穷举，加模型不改这里编译失败。
pub fn model_choice(k: ModelKind) -> vdn_control::ModelChoice {
    match k {
        ModelKind::Gtcrn => vdn_control::ModelChoice::Gtcrn,
        ModelKind::FastEnhancerM => vdn_control::ModelChoice::FastenhancerM,
    }
}

/// 降噪线程的实时调度参数（D68，规则正本 `vdn_rt_thread::RtParams::from_measured`）：
/// 周期 = 一个工作周期（GTCRN 768 样本 = 16 ms，按推理节奏报、不按 2 ms 唤醒节奏报），
/// computation = 预热量到的每周期总耗时 p99.9 × `MEASURED_MARGIN`（夹在周期内）
pub fn denoise_rt_params<P: Processor>(w: &Worker<P>, wu: &Warmup) -> vdn_rt_thread::RtParams {
    let period_ns = w.block_ns() * wu.period_blocks.max(1) as u64;
    vdn_rt_thread::RtParams::from_measured(
        period_ns,
        wu.p999_period_ns.max(1),
        vdn_rt_thread::MEASURED_MARGIN,
    )
}

/// 降噪总线头的配置：总线头里的 L_total / lead / 降噪块长**只从这里写**。
/// 正本：管线延迟与块长 = `Worker`（来自 `Pipeline`），lead 与容量 = 调用方的 `vdn_control::Config`。
pub fn denoise_bus_config<P: Processor>(
    w: &Worker<P>,
    lead: usize,
    capacity: usize,
) -> Result<BusConfig, String> {
    let u = |v: usize, what: &str| u32::try_from(v).map_err(|_| format!("{what} = {v} 太大"));
    if capacity <= w.l_total(lead) + 2 * w.block() {
        return Err(format!(
            "环容量 {capacity} 太小：至少要 L_total {} + 2 块 = {}",
            w.l_total(lead),
            w.l_total(lead) + 2 * w.block()
        ));
    }
    Ok(BusConfig {
        rate: IO_RATE,
        capacity_frames: u(capacity, "容量")?,
        l_total: u(w.l_total(lead), "L_total")?,
        lead: u(lead, "lead")?,
        denoise_block: u(w.block(), "降噪块长")?,
    })
}

/// 计数器（只增不减）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkerStats {
    /// 处理过的块数（= 心跳次数）
    pub blocks: u64,
    /// Processed 环满，没推进去的样本
    pub proc_overflow: u64,
    /// 单块处理（`Processor::process`）最长耗时（ns）。GTCRN 每 hop 才推理一次，最长的那块就是推理那块
    pub max_process_ns: u64,
    /// 处理耗时总和（ns），除以 `blocks` 得平均
    pub total_process_ns: u64,
    /// 处理耗时超过一块音频时长的块数
    pub over_budget: u64,
    /// 降噪落后时跳过的 Raw 样本数（同时记进 Raw 环的缺口计数）
    pub raw_skipped: u64,
    /// 执行过的控制命令数
    pub commands: u64,
    /// 醒来迟到（直接判据之一，不靠读回调度策略）：开始处理时 Raw 里积了多少样本的最大值。
    /// 准时醒来时 ≈ 一块 IO；多出来的就是迟到的量
    pub max_raw_backlog: u64,
    /// 处理超时（直接判据之二）：最近一个工作周期（`period_blocks` 块）的总处理耗时超过周期时长的次数
    pub over_period: u64,
    /// 最近一个工作周期总处理耗时的最大值（ns）
    pub max_period_ns: u64,
    /// 读回调度策略的次数 / 其中「降级」（读回失败或不是 time-constraint）的次数。
    /// 注意：读回看不到系统内部的降级，所以「醒来迟到」「超周期」才是直接判据
    pub rt_checks: u64,
    pub rt_downgrades: u64,
    /// 插进 Processed 的零占位样本数
    pub placeholders_pushed: u64,
}

/// 预热结果（耗时都按单块 `process` 计，ns）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Warmup {
    pub blocks: usize,
    pub first_ns: u64,
    pub max_ns: u64,
    /// 去掉第一块后的最长耗时（稳态里推理那一块的耗时）
    pub max_after_first_ns: u64,
    pub mean_ns: u64,
    /// 一个工作周期是几块（`period_samples / 块长`，至少 1）
    pub period_blocks: usize,
    /// 每个工作周期的总处理耗时（滑动窗口，去掉第一块）的 p99.9（ns）——给实时调度参数的 computation 用
    /// （口径大总管定：盖住一周期内的总工作量，不是单块推理）
    pub p999_period_ns: u64,
    /// 同上的最大值（ns）
    pub max_period_ns: u64,
}

/// 线性插值的分位数（`q` 取 0–1）；空的返回 0。
fn quantile(sorted: &[u64], q: f64) -> u64 {
    let Some(last) = sorted.len().checked_sub(1) else {
        return 0;
    };
    let pos = q.clamp(0.0, 1.0) * last as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    let (a, b) = (
        sorted.get(lo).copied().unwrap_or(0),
        sorted.get(hi).copied().unwrap_or(0),
    );
    a + ((b.saturating_sub(a)) as f64 * (pos - lo as f64)) as u64
}

/// 一次 `step` 的结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Raw 不够一块，没干活
    Idle,
    /// 处理了一块
    Block,
}

/// 最多同时排队几段「跳过占位」；满了并进最后一段
const MAX_MARKERS: usize = 8;

/// 跳过积压后要插进 Processed 的零占位：在输出流第 `at` 个样本处插 `count` 个。
#[derive(Clone, Copy, Debug, Default)]
struct Marker {
    at: u64,
    count: u64,
}

pub struct Worker<P> {
    proc: P,
    block: usize,
    inbuf: Vec<f32>,
    outbuf: Vec<f32>,
    zero_buf: Vec<f32>,
    /// 已推进 Processed 的样本数（含 prime 的零和跳过占位）：输出流的位置
    pushed: u64,
    /// Raw 积压超过这么多就跳过旧的（`None` = 不跳）
    skip_over: Option<usize>,
    markers: [Marker; MAX_MARKERS],
    n_markers: usize,
    /// 最近一个工作周期里各块的处理耗时（环形，固定大小，不分配）
    recent: [u64; MAX_PERIOD_BLOCKS],
    recent_pos: usize,
    /// 一个工作周期几块（≤ MAX_PERIOD_BLOCKS）
    period_blocks: usize,
    stats: WorkerStats,
}

/// 一个工作周期最多算多少块（GTCRN 768 / 96 = 8；留余量）
const MAX_PERIOD_BLOCKS: usize = 64;

// 构造：非实时
impl<P: Processor> Worker<P> {
    pub fn new(proc: P, block: usize) -> Self {
        let block = block.max(1);
        let proc_period = proc.period_samples();
        Self {
            proc,
            block,
            inbuf: vec![0.0; block],
            outbuf: vec![0.0; block],
            zero_buf: vec![0.0; block],
            pushed: 0,
            skip_over: None,
            markers: [Marker::default(); MAX_MARKERS],
            n_markers: 0,
            recent: [0; MAX_PERIOD_BLOCKS],
            recent_pos: 0,
            period_blocks: (proc_period / block).clamp(1, MAX_PERIOD_BLOCKS),
            stats: WorkerStats::default(),
        }
    }

    /// 降噪落后时跳过 Raw 积压：可读量超过 `lead + 2 × 降噪块` 就丢掉旧的、只留一块。
    /// 丢掉的那段原声，处理后的反正已经迟到（送音端会按账丢）；跳过让降噪进程直接回到「现在」，不白算。
    /// 对齐：管线里还在路上的 L_pipe 个输出属于跳过前的声音，所以在输出流「现在 + L_pipe」处插同样多个零占位，账不错位。
    /// 占位是带外告知的（大总管定，不在声音里做记号）：跳过的长度记进 Raw 环的缺口计数（`note_gap`），
    /// 送音端看到缺口计数涨了，就把接下来一个 L_total 窗口当心跳过期处理（用延迟原声、淡入淡出），零占位不会被播出
    pub fn skip_backlog_over_lead(mut self, lead: usize) -> Self {
        self.skip_over = Some(lead + 2 * self.block);
        self
    }

    /// 链路总延迟 = 管线延迟 + 环余量。
    pub fn l_total(&self, lead: usize) -> usize {
        self.proc.latency_samples() + lead
    }

    pub fn processor(&self) -> &P {
        &self.proc
    }
    pub fn processor_mut(&mut self) -> &mut P {
        &mut self.proc
    }
    pub fn block(&self) -> usize {
        self.block
    }

    /// 一块音频的时长（ns）
    pub fn block_ns(&self) -> u64 {
        self.block as u64 * 1_000_000_000 / u64::from(IO_RATE)
    }

    /// 预热：喂 `blocks` 块零、丢掉输出（推理库第一次调用慢，别让它落在服务期）。
    /// 对齐不受影响：之后管线的第 k 个输出仍对应第 k − L_pipe 个真输入，之前的输入就是这些零（和占位样本同理）。
    pub fn warm_up(&mut self, blocks: usize) -> Warmup {
        let mut w = Warmup {
            blocks,
            ..Warmup::default()
        };
        let mut total = 0u64;
        let mut times = Vec::with_capacity(blocks);
        self.inbuf.fill(0.0);
        for k in 0..blocks {
            let t0 = std::time::Instant::now();
            self.proc.process(&self.inbuf, &mut self.outbuf);
            let dt = u64::try_from(t0.elapsed().as_nanos()).unwrap_or(u64::MAX);
            total = total.saturating_add(dt);
            w.max_ns = w.max_ns.max(dt);
            if k == 0 {
                w.first_ns = dt;
            } else {
                w.max_after_first_ns = w.max_after_first_ns.max(dt);
                times.push(dt);
            }
        }
        w.mean_ns = total / blocks.max(1) as u64;
        // 每个工作周期的总耗时：在去掉第一块后的序列上滑动求和（窗口 = 一个周期的块数）
        w.period_blocks = (self.proc.period_samples() / self.block).max(1);
        let mut sums: Vec<u64> = times
            .windows(w.period_blocks)
            .map(|win| win.iter().sum())
            .collect();
        sums.sort_unstable();
        w.p999_period_ns = quantile(&sums, 0.999);
        w.max_period_ns = sums.last().copied().unwrap_or(0);
        w
    }
}

// 实时（B 档）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<P: Processor> Worker<P> {
    pub fn stats(&self) -> WorkerStats {
        self.stats
    }

    /// 开头往 Processed 推 `lead` 个零。返回实际推进的个数。
    pub fn prime(&mut self, bus: &Bus, lead: usize) -> usize {
        let ring = bus.ring(RingId::Processed);
        self.outbuf.fill(0.0);
        let mut left = lead;
        while left > 0 {
            let k = left.min(self.outbuf.len());
            let got = ring.push(self.outbuf.get(..k).unwrap_or(&[]));
            left -= got.min(left);
            if got < k {
                break;
            }
        }
        self.pushed += (lead - left) as u64;
        lead - left
    }

    fn add_marker(&mut self, at: u64, count: u64) {
        if let Some(m) = self.markers.get_mut(self.n_markers) {
            *m = Marker { at, count };
            self.n_markers += 1;
        } else if let Some(last) = self.markers.last_mut() {
            last.count += count;
        }
    }

    /// 推 `n` 个零占位（按 zero_buf 分段）
    fn push_placeholders(&mut self, processed: &vdn_shm::Ring<'_>, n: u64) {
        let mut left = n;
        while left > 0 {
            let k = usize::try_from(left)
                .unwrap_or(usize::MAX)
                .min(self.zero_buf.len());
            let got = processed.push(self.zero_buf.get(..k).unwrap_or(&[]));
            self.stats.proc_overflow += (k - got.min(k)) as u64;
            left -= k as u64;
        }
        self.stats.placeholders_pushed += n;
        self.pushed += n;
    }

    /// 推一块处理结果，途中到了占位位置就先插零
    fn push_block(&mut self, processed: &vdn_shm::Ring<'_>) {
        let n = self.block;
        let mut i = 0usize;
        while self.n_markers > 0 {
            let Some(m) = self.markers.first().copied() else {
                break;
            };
            let pos = self.pushed;
            let end_of_block = pos + (n - i) as u64;
            if m.at > end_of_block {
                break;
            }
            // 先推到标记位置
            let k = usize::try_from(m.at.saturating_sub(pos))
                .unwrap_or(0)
                .min(n - i);
            let got = processed.push(self.outbuf.get(i..i + k).unwrap_or(&[]));
            self.stats.proc_overflow += (k - got.min(k)) as u64;
            self.pushed += k as u64;
            i += k;
            self.push_placeholders(processed, m.count);
            if let Some(live) = self.markers.get_mut(..self.n_markers) {
                live.copy_within(1.., 0);
            }
            self.n_markers -= 1;
        }
        let rest = self.outbuf.get(i..).unwrap_or(&[]);
        let got = processed.push(rest);
        self.stats.proc_overflow += (rest.len() - got.min(rest.len())) as u64;
        self.pushed += rest.len() as u64;
    }

    /// 记一次调度策略读回结果（调用方每秒读一次 `vdn_rt_thread::check_current()`，别每块读）
    pub fn note_rt_check(&mut self, time_constraint: bool) {
        self.stats.rt_checks += 1;
        if !time_constraint {
            self.stats.rt_downgrades += 1;
        }
    }

    /// 把队列里的控制命令都执行掉（块与块之间调）。返回 `false` = 收到 Shutdown
    pub fn apply_commands(&mut self, rx: &mut rtrb::Consumer<vdn_control::Cmd>) -> bool {
        let mut keep_going = true;
        while let Ok(cmd) = rx.pop() {
            self.stats.commands += 1;
            match cmd {
                vdn_control::Cmd::SetMode(m) => self.proc.set_mode(m),
                vdn_control::Cmd::Shutdown => keep_going = false,
            }
        }
        keep_going
    }

    /// Raw 够一块就处理一块、推 Processed、心跳一次。
    pub fn step(&mut self, bus: &Bus) -> Step {
        let (raw, processed) = (bus.ring(RingId::Raw), bus.ring(RingId::Processed));
        let avail = raw.available();
        self.stats.max_raw_backlog = self.stats.max_raw_backlog.max(avail as u64);
        if let Some(over) = self.skip_over
            && avail > over
        {
            let d = raw.discard(avail - self.block);
            if d > 0 {
                raw.note_gap(d);
                self.stats.raw_skipped += d as u64;
                let at = self.pushed + self.proc.latency_samples() as u64;
                self.add_marker(at, d as u64);
            }
        }
        if raw.available() < self.block {
            return Step::Idle;
        }
        let got = raw.pop(&mut self.inbuf);
        if got < self.block {
            // 不该发生（单消费者）；不完整的那块补零照常处理，保持流长度
            if let Some(rest) = self.inbuf.get_mut(got..) {
                rest.fill(0.0);
            }
        }
        let t0 = std::time::Instant::now();
        self.proc.process(&self.inbuf, &mut self.outbuf);
        let dt = u64::try_from(t0.elapsed().as_nanos()).unwrap_or(u64::MAX);
        self.stats.max_process_ns = self.stats.max_process_ns.max(dt);
        self.stats.total_process_ns = self.stats.total_process_ns.saturating_add(dt);
        if dt > self.block_ns() {
            self.stats.over_budget += 1;
        }
        // 最近一个工作周期的总耗时（滑动）
        if let Some(slot) = self.recent.get_mut(self.recent_pos) {
            *slot = dt;
        }
        self.recent_pos = (self.recent_pos + 1) % self.period_blocks;
        let window: u64 = self
            .recent
            .get(..self.period_blocks)
            .map_or(0, |w| w.iter().sum());
        self.stats.max_period_ns = self.stats.max_period_ns.max(window);
        if window > self.block_ns() * self.period_blocks as u64 {
            self.stats.over_period += 1;
        }
        self.push_block(&processed);
        self.stats.blocks += 1;
        bus.beat();
        Step::Block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_capture::shm::forward_same_bus;
    use vdn_capture::{Forwarder, ForwarderConfig};

    /// 输出 = 输入、延迟 0
    struct Copy0;
    impl Processor for Copy0 {
        fn process(&mut self, input: &[f32], output: &mut [f32]) {
            output.copy_from_slice(input);
        }
        fn latency_samples(&self) -> usize {
            0
        }
    }

    fn heap(l_total: usize) -> Bus {
        Bus::heap(BusConfig {
            rate: IO_RATE,
            capacity_frames: 1 << 14,
            l_total: l_total as u32,
            lead: 0,
            denoise_block: 96,
        })
        .unwrap()
    }

    #[test]
    fn prime_then_steps_beat_once_per_block() {
        let mut w = Worker::new(Copy0, 480);
        let bus = heap(1440);
        assert_eq!(w.prime(&bus, 1440), 1440);
        assert_eq!(w.step(&bus), Step::Idle);
        bus.ring(RingId::Raw).push(&[1.0; 1000]);
        assert_eq!(w.step(&bus), Step::Block);
        assert_eq!(w.step(&bus), Step::Block);
        assert_eq!(w.step(&bus), Step::Idle);
        assert_eq!(bus.heartbeat(), 2);
        assert_eq!(bus.ring(RingId::Processed).available(), 1440 + 960);
        assert_eq!(w.stats().blocks, 2);
    }

    /// 交叉校验（唯一真相源，CLAUDE.md 第五节第 5 条）：总线名规则的正本在 vdn-shm（`Bus::open` 报 `InvalidName`），
    /// vdn-control 的 `validate_bus_name` 是给配置文件早报错用的第二处表示，两者对每个样例必须一致
    #[test]
    fn bus_name_rule_matches_vdn_shm() {
        let long = format!("/vdn-{}", "a".repeat(30));
        // 14 个字符、32 字节
        let multi = format!("/vdn-{}", "中".repeat(9));
        for name in [
            "/vdn-bus",
            "/vdn-out",
            "/vdn-chaintest-out",
            "/vdn-",
            "/vdn",
            "/vdnx",
            "vdn-bus",
            "/other",
            "/vdn-a/b",
            "/vdn-a\0b",
            &long,
            "/vdn-ok_1.2",
            // 名字只许可见 ASCII（vdn-shm 收窄，codex 第 2 轮 #2）：空格、中文、多字节超 31 字节、再含 / 都拒；可见符号放行
            "/vdn-a b",
            "/vdn-中",
            &multi,
            "/vdn-x/",
            "/vdn-~!",
        ] {
            let control_ok = vdn_control::validate_bus_name(name).is_ok();
            let shm_ok = !matches!(Bus::open(name), Err(vdn_shm::ShmError::InvalidName));
            assert_eq!(
                control_ok, shm_ok,
                "{name:?}：vdn-control {control_ok} · vdn-shm {shm_ok}"
            );
        }
    }

    /// 交叉校验：IO 采样率三处（vdn-control 正本 · 总线头 · 降噪总线配置）一致
    #[test]
    fn io_rate_single_source() {
        let w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
        let cfg = denoise_bus_config(&w, 288, 48_000).unwrap();
        assert_eq!(cfg.rate, vdn_control::IO_RATE);
        assert_eq!(cfg.lead, 288);
        assert_eq!(cfg.denoise_block, 96);
        assert_eq!(cfg.l_total as usize, w.l_total(288));
        let bus = Bus::heap(cfg).unwrap();
        assert_eq!(bus.config(), cfg);
    }

    /// 输出 = 0.5 × 输入延迟 768（和原声可区分，延迟同直通管线）
    struct HalfDelay {
        line: Vec<f32>,
        pos: usize,
    }
    impl Processor for HalfDelay {
        fn process(&mut self, input: &[f32], output: &mut [f32]) {
            for (x, y) in input.iter().zip(output.iter_mut()) {
                *y = 0.5 * std::mem::replace(&mut self.line[self.pos], *x);
                self.pos = (self.pos + 1) % self.line.len();
            }
        }
        fn latency_samples(&self) -> usize {
            self.line.len()
        }
    }

    /// 降噪停 100 块后恢复：跳过 Raw 积压（零占位 + 缺口计数，带外），送音端在那个窗口送原声；
    /// 全程不静音、不跳变，恢复后立刻回到「现在」并逐样本对齐（不追 1 s 的积压）
    #[test]
    fn skip_backlog_keeps_alignment_without_silence() {
        let (block, lead) = (96, 288);
        let mut w = Worker::new(
            HalfDelay {
                line: vec![0.0; 768],
                pos: 0,
            },
            block,
        )
        .skip_backlog_over_lead(lead);
        let l_total = w.l_total(lead);
        let bus = heap(l_total);
        w.prime(&bus, lead);
        let mut f = Forwarder::new(ForwarderConfig::new(l_total, block));
        let n = 1500;
        let x = vdn_capture::testing::test_tone(IO_RATE, n * block, 0);
        let mut y = vec![0.0f32; x.len()];
        for (i, (c, yc)) in x.chunks(block).zip(y.chunks_mut(block)).enumerate() {
            forward_same_bus(&mut f, &bus, c);
            if !(500..600).contains(&i) {
                while w.step(&bus) == Step::Block {}
            }
            assert_eq!(bus.ring(RingId::Out).pop(yc), c.len());
        }
        let s = w.stats();
        assert!(
            s.raw_skipped > 0 && s.placeholders_pushed == s.raw_skipped,
            "{s:?}"
        );
        assert!(bus.ring(RingId::Raw).gaps() > 0);
        let fs = f.stats();
        assert!(fs.gap_holds >= 1, "{fs:?}");
        assert_eq!(fs.nonfinite, 0);
        // 不静音（零占位没被播出）、不跳变
        let quiet = y[l_total..]
            .split(|v| v.abs() >= 1e-3)
            .map(<[f32]>::len)
            .max()
            .unwrap_or(0);
        assert!(quiet < 50, "最长静音 {quiet} 样本");
        let jump = y
            .windows(2)
            .skip(l_total)
            .map(|p| (p[1] - p[0]).abs())
            .fold(0.0, f32::max);
        assert!(jump < 0.13, "最大跳变 {jump}");
        // 停顿结束后不到 0.2 s 就回到处理后的（0.5 倍）并逐样本对齐
        let back = (600 + 100) * block;
        for k in back..y.len() {
            assert!(
                (y[k] - 0.5 * x[k - l_total]).abs() < 1e-4,
                "k={k}: {} vs {}",
                y[k],
                0.5 * x[k - l_total]
            );
        }
        assert!(f.is_fresh());
    }

    /// 运行中切模式：命令经 rtrb 送进降噪线程，块之间执行；Pipeline 按一个 hop 淡化，输出连续
    #[test]
    fn mode_switch_over_rtrb_is_applied_between_blocks() {
        use vdn_control::{Cmd, Mode};
        let mut w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
        let (mut tx, mut rx) = command_channel(8);
        let bus = heap(w.l_total(288));
        w.prime(&bus, 288);
        let x = vdn_capture::testing::test_tone(IO_RATE, 96, 0);
        assert!(tx.push(Cmd::SetMode(Mode::Passthrough)).is_ok());
        assert!(tx.push(Cmd::SetMode(Mode::Denoise)).is_ok());
        assert!(w.apply_commands(&mut rx));
        bus.ring(RingId::Raw).push(&x);
        assert_eq!(w.step(&bus), Step::Block);
        assert!(tx.push(Cmd::Shutdown).is_ok());
        assert!(!w.apply_commands(&mut rx));
        assert_eq!(w.stats().commands, 3);
        assert_eq!(
            engine_mode(Mode::Passthrough),
            vdn_engine::Mode::Passthrough
        );
    }

    /// 预热量的「每周期总耗时」：窗口 = 一个工作周期的块数（定块长 96 的 Identity 管线：hop 768 → 8 块）
    #[test]
    fn warmup_measures_per_period_work() {
        let mut w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
        let wu = w.warm_up(64);
        assert_eq!(wu.period_blocks, 8);
        assert!(wu.max_period_ns >= wu.p999_period_ns);
        assert!(
            wu.max_period_ns >= wu.max_after_first_ns,
            "一个周期的总和不小于其中最长的一块"
        );
        // 没有周期概念的处理器按单块算
        let mut w0 = Worker::new(Copy0, 96);
        assert_eq!(w0.warm_up(16).period_blocks, 1);
        assert_eq!(quantile(&[], 0.999), 0);
        assert_eq!(quantile(&[10, 20], 1.0), 20);
        assert_eq!(quantile(&[10, 20], 0.5), 15);
    }

    /// 直接判据计数：处理慢于周期时长就记「超周期」；Raw 积压越多「醒来迟到」越大
    #[test]
    fn overrun_and_backlog_counters() {
        /// 每块睡 3 ms（> 一块 2 ms），一个周期（2 块）总耗时 6 ms > 4 ms
        struct Slow;
        impl Processor for Slow {
            fn process(&mut self, i: &[f32], o: &mut [f32]) {
                std::thread::sleep(std::time::Duration::from_millis(3));
                o.copy_from_slice(i);
            }
            fn latency_samples(&self) -> usize {
                0
            }
            fn period_samples(&self) -> usize {
                192
            }
        }
        let mut w = Worker::new(Slow, 96);
        let bus = heap(0);
        bus.ring(RingId::Raw).push(&[0.0; 96 * 5]);
        while w.step(&bus) == Step::Block {}
        let s = w.stats();
        assert_eq!(s.blocks, 5);
        assert_eq!(s.max_raw_backlog, 480, "第一次开始处理时积了 5 块");
        assert!(s.over_period >= 4, "{s:?}");
        assert!(s.max_period_ns >= 6_000_000, "{s:?}");
    }

    /// D68 调度参数：周期报一个工作周期（GTCRN 定块 96 的管线 hop 768 = 16 ms，不是 2 ms），
    /// computation = 每周期总耗时 p99.9 × 1.5，夹在周期内
    #[test]
    fn denoise_rt_params_use_period_and_measured_window() {
        let mut w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
        let wu = w.warm_up(64);
        let p = denoise_rt_params(&w, &wu);
        assert_eq!(p.period_ns, 16_000_000);
        assert_eq!(p.constraint_ns, p.period_ns);
        // 规则只在正本 `RtParams::from_measured` 里：这里比对「同样的输入 → 同样的输出」，不抄夹取规则
        assert_eq!(
            p,
            vdn_rt_thread::RtParams::from_measured(
                p.period_ns,
                wu.p999_period_ns.max(1),
                vdn_rt_thread::MEASURED_MARGIN
            )
        );
        // 直通管线实测极小（2026-11-02 `vdn run` code 4 的起因）：给出的参数内核也要接受
        assert!(p.validate().is_ok(), "{p:?} · 预热 {wu:?}");
        assert!(p.computation_ns >= vdn_rt_thread::MIN_COMPUTATION_NS);
    }

    #[test]
    fn rt_check_counts_downgrades() {
        let mut w = Worker::new(Copy0, 96);
        w.note_rt_check(true);
        w.note_rt_check(false);
        w.note_rt_check(true);
        assert_eq!((w.stats().rt_checks, w.stats().rt_downgrades), (3, 1));
    }

    #[test]
    fn prime_stops_when_full() {
        let mut w = Worker::new(Copy0, 480);
        let bus = heap(0);
        let cap = bus.capacity();
        assert_eq!(w.prime(&bus, cap + 1000), cap);
    }

    #[test]
    fn identity_pipeline_latency() {
        let p = identity_pipeline(None).unwrap();
        assert_eq!(Processor::latency_samples(&p), 863);
        let p = identity_pipeline(Some(96)).unwrap();
        assert_eq!(Processor::latency_samples(&p), 768);
        assert!(identity_pipeline(Some(100)).is_err());
    }

    /// Worker + Forwarder 经同一条进程内总线：直通管线下输出 = 输入延迟 L_total（重采样有轻微纹波，按相关判）
    #[test]
    fn worker_and_forwarder_align_end_to_end() {
        // IO 块 480，降噪块 96（定块长管线）
        let (block, lead, dblock) = (480, 1440, 96);
        let mut w = Worker::new(identity_pipeline(Some(dblock)).unwrap(), dblock);
        // 预热不影响对齐（输出丢掉、之前的输入等于零）
        let wu = w.warm_up(16);
        assert_eq!(wu.blocks, 16);
        let l_total = w.l_total(lead);
        let bus = heap(l_total);
        w.prime(&bus, lead);
        let mut f = Forwarder::new(ForwarderConfig::new(l_total, block));
        let x = vdn_capture::testing::test_tone(IO_RATE, 200 * block, 0);
        let mut y = vec![0.0; x.len()];
        for (c, yc) in x.chunks(block).zip(y.chunks_mut(block)) {
            forward_same_bus(&mut f, &bus, c);
            while w.step(&bus) == Step::Block {}
            assert_eq!(bus.ring(RingId::Out).pop(yc), c.len());
        }
        assert_eq!(f.stats().proc_underrun, 0);
        assert!(f.is_fresh());
        // 相关（跳过开头 100 ms）
        let skip = 4800;
        let (a, b) = (&x[skip..x.len() - l_total], &y[skip + l_total..]);
        let dot: f64 = a
            .iter()
            .zip(b)
            .map(|(p, q)| f64::from(*p) * f64::from(*q))
            .sum();
        let na: f64 = a.iter().map(|p| f64::from(*p).powi(2)).sum();
        let nb: f64 = b.iter().map(|q| f64::from(*q).powi(2)).sum();
        let r = dot / (na * nb).sqrt();
        assert!(r > 0.999, "相关 {r}");
    }
    /// 实时路径不分配（R1）：真进程内总线 + 定块长直通管线，走遍 正常 → 降噪停（Raw 溢出、缺口、心跳过期）
    /// → 恢复（迟到丢弃）→ 积压超限重新对账 这几条路径，Forwarder 与 Worker 都在 assert_no_alloc 里跑
    #[test]
    fn realtime_paths_do_not_allocate() {
        let (block, lead, dblock) = (480, 1440, 96);
        let mut w = Worker::new(identity_pipeline(Some(dblock)).unwrap(), dblock);
        let l_total = w.l_total(lead);
        let bus = heap(l_total);
        w.prime(&bus, lead);
        let mut f = Forwarder::new(ForwarderConfig::new(l_total, block));
        let x = vdn_capture::testing::test_tone(IO_RATE, block, 0);
        let mut y = vec![0.0f32; block];
        let junk = vec![0.5f32; 20 * block];
        // 热身
        for _ in 0..10 {
            forward_same_bus(&mut f, &bus, &x);
            while w.step(&bus) == Step::Block {}
            bus.ring(RingId::Out).pop(&mut y);
        }
        assert_no_alloc::assert_no_alloc(|| {
            for i in 0..400 {
                forward_same_bus(&mut f, &bus, &x);
                if !(100..200).contains(&i) {
                    while w.step(&bus) == Step::Block {}
                }
                if i == 300 {
                    bus.ring(RingId::Processed).push(&junk);
                }
                bus.ring(RingId::Out).pop(&mut y);
            }
        });
        let s = f.stats();
        assert!(s.raw_overflow > 0, "{s:?}");
        assert!(s.stale_switches >= 1, "{s:?}");
        assert!(s.discarded > 0, "{s:?}");
        assert!(s.resyncs >= 1, "{s:?}");
        assert!(s.cuts >= 1, "{s:?}");
    }
}

#[cfg(all(test, debug_assertions))]
#[global_allocator]
static A: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;
