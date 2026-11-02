//! vdn-capture：送音进程的核心逻辑（`Forwarder`）+ 链路两端共用的小 trait。
//!
//! 链路（docs/plan/010 第十节）：输入一块原声 → 推 Raw → 从 Processed 取同样多 →
//! 按心跳决定用处理后的声音还是本地延迟 `L_total` 的原声 → 推 Out。
//!
//! - 档：`Forwarder::process*` 跑在 IO 回调里（A 档）：不分配、不加锁、不 panic
//! - 依赖闭包里**不许有** `vdn-engine`、`vdn-models`、推理库（D03 崩溃隔离，§2.2 规则 3）
//! - `Forwarder` 对 [`Push`] / [`Pop`] / [`Heartbeat`] 泛型：真环由 bin 接 `vdn-shm`，单测用 [`testing`] 里的假环
//!
//! 对齐约定（送原声能无缝接上的前提）：降噪进程建总线后，先往 Processed 推
//! `lead = L_total - 管线延迟` 个零，再逐块推处理结果。于是 Processed 的第 k 个样本
//! = 原声第 `k - L_total` 个样本处理后的结果，和本地延迟线输出逐样本同一时刻。

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

pub mod testing;

use vdn_dsp::DelayLine;

/// 单生产者一端：尽量写入，返回实际写入的样本数（环满时少于 `x.len()`）。
pub trait Push {
    fn push(&mut self, x: &[f32]) -> usize;
}

/// 单消费者一端：尽量读出，返回实际读出的样本数（不够时少于 `out.len()`）。
pub trait Pop {
    fn pop(&mut self, out: &mut [f32]) -> usize;
    /// 现在可读的样本数
    fn available(&self) -> usize;
}

/// 读心跳：降噪进程每处理一块 +1。
pub trait Heartbeat {
    fn heartbeat(&self) -> u64;
}

/// 写心跳（降噪进程一侧）。
pub trait Beat {
    fn beat(&mut self);
}

impl<T: Push + ?Sized> Push for &mut T {
    fn push(&mut self, x: &[f32]) -> usize {
        (**self).push(x)
    }
}
impl<T: Pop + ?Sized> Pop for &mut T {
    fn pop(&mut self, out: &mut [f32]) -> usize {
        (**self).pop(out)
    }
    fn available(&self) -> usize {
        (**self).available()
    }
}
impl<T: Heartbeat + ?Sized> Heartbeat for &T {
    fn heartbeat(&self) -> u64 {
        (**self).heartbeat()
    }
}

/// `Forwarder` 的参数（全部按 48 kHz 样本数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForwarderConfig {
    /// 总延迟：本地原声延迟线的长度，等于总线头里的 `l_total`
    pub l_total: usize,
    /// 每次 `process` 最多处理多少样本；更长的输入内部切块
    pub max_block: usize,
    /// 心跳多少样本不动算过期（100 ms = 4800）
    pub timeout: usize,
    /// 处理后 ↔ 原声 的淡化长度（10 ms = 480）
    pub fade: usize,
    /// Processed 里积压超过这么多就丢掉多余的（防延迟越积越大）；0 = 自动取 `l_total + 2 × max_block`
    pub max_backlog: usize,
}

impl ForwarderConfig {
    pub fn new(l_total: usize, max_block: usize) -> Self {
        Self {
            l_total,
            max_block,
            timeout: 4800,
            fade: 480,
            max_backlog: 0,
        }
    }
}

/// 计数器（只增不减；实时线程写，非实时线程读拷贝）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ForwarderStats {
    /// 处理过的样本总数
    pub samples: u64,
    /// Raw 环满，没推进去的原声样本
    pub raw_overflow: u64,
    /// 心跳新鲜时 Processed 不够、用原声补的样本
    pub proc_underrun: u64,
    /// 心跳过期期间输出的样本（整段送原声）
    pub stale_samples: u64,
    /// Out 环满，没推进去的样本
    pub out_overflow: u64,
    /// 心跳 新鲜 → 过期 的次数（进程级送原声触发次数）
    pub stale_switches: u64,
    /// 心跳 过期 → 新鲜 的次数（含启动时第一次）
    pub recoveries: u64,
    /// 为保持对齐或限积压而丢掉的处理后样本
    pub discarded: u64,
    /// 最近一次 新鲜 → 过期 发生在第几个样本（`samples` 计）
    pub last_stale_at: Option<u64>,
}

/// 送音进程的转发器：原声 → Raw；Processed / 延迟原声 → Out。
pub struct Forwarder {
    cfg: ForwarderConfig,
    delay: DelayLine,
    raw_delayed: Vec<f32>,
    proc_buf: Vec<f32>,
    out_buf: Vec<f32>,
    /// 处理后声音的权重（0 = 全原声，1 = 全处理后）
    gain: f32,
    step: f32,
    last_beat: Option<u64>,
    since_beat: usize,
    fresh: bool,
    /// 对齐账：> 0 = Processed 欠我们这么多样本（下次到了先丢掉）；< 0 = Processed 超前（先不取，用原声）
    skip: i64,
    stats: ForwarderStats,
}

// 构造：非实时线程
impl Forwarder {
    pub fn new(cfg: ForwarderConfig) -> Self {
        let max_block = cfg.max_block.max(1);
        let mut cfg = cfg;
        cfg.max_block = max_block;
        if cfg.max_backlog == 0 {
            cfg.max_backlog = cfg.l_total + 2 * max_block;
        }
        Self {
            cfg,
            delay: DelayLine::new(cfg.l_total),
            raw_delayed: vec![0.0; max_block],
            proc_buf: vec![0.0; max_block],
            out_buf: vec![0.0; max_block],
            gain: 0.0,
            step: 1.0 / cfg.fade.max(1) as f32,
            last_beat: None,
            since_beat: 0,
            fresh: false,
            skip: 0,
            stats: ForwarderStats::default(),
        }
    }
}

// 实时（specs/rust/architecture.md §5.3）
#[deny(clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros)]
impl Forwarder {
    pub fn config(&self) -> ForwarderConfig {
        self.cfg
    }
    pub fn stats(&self) -> ForwarderStats {
        self.stats
    }
    /// 心跳是否新鲜（新鲜 = 用处理后的声音）
    pub fn is_fresh(&self) -> bool {
        self.fresh
    }
    /// 当前处理后声音的权重
    pub fn gain(&self) -> f32 {
        self.gain
    }

    /// 一块原声进，一块结果推进 Out。
    pub fn process<R, P, O, H>(&mut self, input: &[f32], raw: &mut R, processed: &mut P, out: &mut O, hb: &H)
    where
        R: Push,
        P: Pop,
        O: Push,
        H: Heartbeat + ?Sized,
    {
        let mb = self.cfg.max_block;
        for chunk in input.chunks(mb) {
            let n = chunk.len();
            let mut buf = core::mem::take(&mut self.out_buf);
            if let Some(y) = buf.get_mut(..n) {
                self.chunk(chunk, y, raw, processed, hb);
                let pushed = out.push(y);
                self.stats.out_overflow += (n - pushed.min(n)) as u64;
            }
            self.out_buf = buf;
        }
    }

    /// 同 [`Self::process`]，但结果写进 `output`（与 `input` 等长）而不推 Out：开发期直接接扬声器 / 文件用。
    pub fn process_into<R, P, H>(&mut self, input: &[f32], output: &mut [f32], raw: &mut R, processed: &mut P, hb: &H)
    where
        R: Push,
        P: Pop,
        H: Heartbeat + ?Sized,
    {
        let n = input.len().min(output.len());
        let mb = self.cfg.max_block;
        let (head, tail) = output.split_at_mut(n);
        tail.fill(0.0);
        let input = input.get(..n).unwrap_or(&[]);
        for (x, y) in input.chunks(mb).zip(head.chunks_mut(mb)) {
            self.chunk(x, y, raw, processed, hb);
        }
    }

    /// `x.len() == y.len() ≤ max_block`
    fn chunk<R, P, H>(&mut self, x: &[f32], y: &mut [f32], raw: &mut R, processed: &mut P, hb: &H)
    where
        R: Push,
        P: Pop,
        H: Heartbeat + ?Sized,
    {
        let n = x.len();
        // 1. 原声推 Raw；推不进去的样本，降噪进程那边的流就少了这一段 → Processed 超前
        let pushed = raw.push(x).min(n);
        let dropped = n - pushed;
        self.stats.raw_overflow += dropped as u64;
        self.skip -= dropped as i64;

        // 2. 本地延迟原声
        match self.raw_delayed.get_mut(..n) {
            Some(rd) => self.delay.process(x, rd),
            None => return,
        }

        // 3. 心跳
        let beat = hb.heartbeat();
        match self.last_beat {
            Some(b) if b == beat => {
                self.since_beat = self.since_beat.saturating_add(n);
                if self.fresh && self.since_beat > self.cfg.timeout {
                    self.fresh = false;
                    self.stats.stale_switches += 1;
                    self.stats.last_stale_at = Some(self.stats.samples);
                }
            }
            Some(_) => {
                self.last_beat = Some(beat);
                self.since_beat = 0;
                if !self.fresh {
                    self.fresh = true;
                    self.stats.recoveries += 1;
                }
            }
            // 第一次看到：只记基准，等它动了才算新鲜
            None => {
                self.last_beat = Some(beat);
                self.since_beat = 0;
            }
        }

        // 4. 对齐账与积压
        if self.skip > 0 {
            let want = usize::try_from(self.skip).unwrap_or(usize::MAX);
            let d = self.discard(processed, want);
            self.skip -= d as i64;
        }
        let backlog = processed.available();
        if backlog > self.cfg.max_backlog {
            let d = self.discard(processed, backlog - self.cfg.max_backlog);
            // 丢掉的是最旧的；既然限了积压，旧的对齐账作废
            if d > 0 {
                self.skip = 0;
            }
        }
        let hold = if self.skip < 0 {
            let h = usize::try_from(-self.skip).unwrap_or(usize::MAX).min(n);
            self.skip += h as i64;
            h
        } else {
            0
        };

        // 5. 取处理后的（前 hold 个位置不取，用原声）
        let Some(pb) = self.proc_buf.get_mut(hold..n) else {
            return;
        };
        let got = processed.pop(pb).min(n - hold);
        let missing = n - hold - got;
        self.skip += missing as i64;
        if self.fresh {
            self.stats.proc_underrun += (missing + hold) as u64;
        } else {
            self.stats.stale_samples += n as u64;
        }

        // 6. 混合：缺的位置用原声；权重按样本走向目标
        let target = if self.fresh { 1.0 } else { 0.0 };
        let first_proc = hold;
        let last_proc = hold + got;
        let rd = self.raw_delayed.get(..n).unwrap_or(&[]);
        for (i, ((yi, &r), &p)) in y.iter_mut().zip(rd.iter()).zip(self.proc_buf.iter()).enumerate() {
            let p = if (first_proc..last_proc).contains(&i) { p } else { r };
            if self.gain < target {
                self.gain = (self.gain + self.step).min(1.0);
            } else if self.gain > target {
                self.gain = (self.gain - self.step).max(0.0);
            }
            *yi = self.gain * p + (1.0 - self.gain) * r;
        }
        self.stats.samples += n as u64;
    }

    /// 从 Processed 丢掉至多 `want` 个样本，返回丢了多少。
    fn discard<P: Pop>(&mut self, processed: &mut P, want: usize) -> usize {
        let mut left = want;
        while left > 0 {
            let k = left.min(self.proc_buf.len());
            let Some(b) = self.proc_buf.get_mut(..k) else {
                break;
            };
            let got = processed.pop(b);
            left -= got.min(left);
            if got < k {
                break;
            }
        }
        let d = want - left;
        self.stats.discarded += d as u64;
        d
    }
}

#[cfg(test)]
mod tests;
