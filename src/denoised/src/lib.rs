//! vdn-denoised 的循环体（`Worker`），lib 化是为了让 `vdn chain-test` 在进程内复用同一份代码。
//!
//! - 档：B（软实时）。`Worker::step` 不分配、不加锁、不 panic；推理库内部的分配按 §4 B 档登记
//! - 对 [`Processor`] 泛型：真链路用 `vdn_engine::Pipeline`，单测用假处理器

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

use vdn_capture::{Beat, Pop, Push};

/// 一块进、等长一块出，延迟固定（48 kHz 样本）。
pub trait Processor {
    fn process(&mut self, input: &[f32], output: &mut [f32]);
    fn latency_samples(&self) -> usize;
}

/// 计数器（只增不减）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorkerStats {
    /// 处理过的块数（= 心跳次数）
    pub blocks: u64,
    /// Processed 环满，没推进去的样本
    pub proc_overflow: u64,
}

/// 一次 `step` 的结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Raw 不够一块，没干活
    Idle,
    /// 处理了一块
    Block,
}

pub struct Worker<P> {
    proc: P,
    block: usize,
    inbuf: Vec<f32>,
    outbuf: Vec<f32>,
    stats: WorkerStats,
}

// 构造：非实时
impl<P: Processor> Worker<P> {
    pub fn new(proc: P, block: usize) -> Self {
        let block = block.max(1);
        Self {
            proc,
            block,
            inbuf: vec![0.0; block],
            outbuf: vec![0.0; block],
            stats: WorkerStats::default(),
        }
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
}

// 实时（B 档）
#[deny(clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros)]
impl<P: Processor> Worker<P> {
    pub fn stats(&self) -> WorkerStats {
        self.stats
    }

    /// 开头往 Processed 推 `lead` 个零（对齐约定见 vdn-capture 文档）。返回实际推进的个数。
    pub fn prime<O: Push>(&mut self, processed: &mut O, lead: usize) -> usize {
        self.outbuf.fill(0.0);
        let mut left = lead;
        while left > 0 {
            let k = left.min(self.outbuf.len());
            let got = processed.push(self.outbuf.get(..k).unwrap_or(&[]));
            left -= got.min(left);
            if got < k {
                break;
            }
        }
        lead - left
    }

    /// Raw 够一块就处理一块、推 Processed、心跳一次。
    pub fn step<R: Pop, O: Push, B: Beat + ?Sized>(&mut self, raw: &mut R, processed: &mut O, beat: &mut B) -> Step {
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
        self.proc.process(&self.inbuf, &mut self.outbuf);
        let pushed = processed.push(&self.outbuf);
        self.stats.proc_overflow += (self.block - pushed.min(self.block)) as u64;
        self.stats.blocks += 1;
        beat.beat();
        Step::Block
    }
}

/// 输出 = 输入、延迟 0：只在单测里用（真链路的直通走 `Pipeline` + `testing::Identity`）。
#[derive(Clone, Copy, Debug, Default)]
pub struct Copy0;

impl Processor for Copy0 {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (o, i) in output.iter_mut().zip(input) {
            *o = *i;
        }
    }
    fn latency_samples(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_capture::testing::{FakeBeat, FakeRing};
    use vdn_capture::{Forwarder, ForwarderConfig, Heartbeat};

    #[test]
    fn prime_then_steps_beat_once_per_block() {
        let mut w = Worker::new(Copy0, 480);
        let (mut raw, mut processed, mut hb) = (FakeRing::new(1 << 14), FakeRing::new(1 << 14), FakeBeat::default());
        assert_eq!(w.prime(&mut processed, 1440), 1440);
        assert_eq!(w.step(&mut raw, &mut processed, &mut hb), Step::Idle);
        raw.push(&[1.0; 1000]);
        assert_eq!(w.step(&mut raw, &mut processed, &mut hb), Step::Block);
        assert_eq!(w.step(&mut raw, &mut processed, &mut hb), Step::Block);
        assert_eq!(w.step(&mut raw, &mut processed, &mut hb), Step::Idle);
        assert_eq!(hb.heartbeat(), 2);
        assert_eq!(processed.len(), 1440 + 960);
        assert_eq!(w.stats().blocks, 2);
    }

    #[test]
    fn prime_stops_when_full() {
        let mut w = Worker::new(Copy0, 480);
        let mut processed = FakeRing::new(1000);
        assert_eq!(w.prime(&mut processed, 1440), 1000);
    }

    #[test]
    fn worker_and_forwarder_align_end_to_end() {
        let block = 480;
        let lead = 1440;
        let mut w = Worker::new(Copy0, block);
        let (mut raw, mut processed, mut out, mut hb) =
            (FakeRing::new(1 << 14), FakeRing::new(1 << 14), FakeRing::new(1 << 20), FakeBeat::default());
        w.prime(&mut processed, lead);
        let mut f = Forwarder::new(ForwarderConfig::new(w.l_total(lead), block), lead);
        let x: Vec<f32> = (0..100 * block).map(|i| (i as f32 * 0.01).sin()).collect();
        for c in x.chunks(block) {
            f.process(c, &mut raw, &mut processed, &mut out, &hb);
            w.step(&mut raw, &mut processed, &mut hb);
        }
        let y = out.drain_all();
        for k in lead..y.len() {
            assert!((y[k] - x[k - lead]).abs() < 1e-6);
        }
        assert_eq!(f.stats().proc_underrun, 0);
        assert!(f.is_fresh());
    }
}
