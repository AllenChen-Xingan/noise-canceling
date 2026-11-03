//! 降噪循环体 `Worker`：计数器、预热、跳过积压的零占位、逐块处理（实时 B 档）。

use vdn_shm::{Bus, RingId};

use crate::{IO_RATE, Processor};

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
pub(crate) fn quantile(sorted: &[u64], q: f64) -> u64 {
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
