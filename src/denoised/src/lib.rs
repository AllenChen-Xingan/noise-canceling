//! vdn-denoised 的循环体（`Worker`），lib 化是为了让 `vdn chain-test` 在进程内复用同一份代码。
//!
//! - 档：B（软实时）。`Worker::step` 不分配、不加锁、不 panic；推理库内部的分配按 §4 B 档登记
//! - 对 [`Processor`] 泛型：真链路用 `vdn_engine::Pipeline`，单测用假处理器
//! - 直接用 `vdn_shm::Bus`（单测用进程内的 `Bus::heap`）；**不依赖 `vdn-capture`**（它带设备库，本 crate 禁 cpal）
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
use vdn_shm::{Bus, RingId};

/// IO 采样率（docs/plan/010 第十节）
pub const IO_RATE: u32 = 48_000;

/// 一块进、等长一块出，延迟固定（48 kHz 样本）。
pub trait Processor {
    fn process(&mut self, input: &[f32], output: &mut [f32]);
    fn latency_samples(&self) -> usize;
}

impl<M: Model> Processor for Pipeline<M> {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        Pipeline::process(self, input, output);
    }
    fn latency_samples(&self) -> usize {
        Pipeline::latency_samples(self)
    }
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
pub fn identity_pipeline(block: Option<usize>) -> Result<Pipeline<vdn_engine::testing::Identity>, String> {
    build(Engine::new(vdn_engine::testing::Identity, SafetyConfig::off()), block)
}

/// 真模型管线（三件保命按默认开）。`path` 不给就按 vdn-models 的默认路径查找。
pub fn model_pipeline(kind: ModelKind, path: Option<&Path>, block: Option<usize>) -> Result<Pipeline<AnyModel>, String> {
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
        lead - left
    }

    /// Raw 够一块就处理一块、推 Processed、心跳一次。
    pub fn step(&mut self, bus: &Bus) -> Step {
        let (raw, processed) = (bus.ring(RingId::Raw), bus.ring(RingId::Processed));
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
        bus.beat();
        Step::Block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_capture::shm::forward;
    use vdn_capture::{Forwarder, ForwarderConfig};
    use vdn_shm::BusConfig;

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

    #[test]
    fn prime_stops_when_full() {
        let mut w = Worker::new(Copy0, 480);
        let bus = heap(0);
        let cap = bus.capacity();
        assert_eq!(w.prime(&bus, cap + 1000), cap);
    }

    #[test]
    fn identity_pipeline_latency() {
        let p = identity_pipeline().unwrap();
        assert_eq!(Processor::latency_samples(&p), 863);
    }

    /// Worker + Forwarder 经同一条进程内总线：直通管线下输出 = 输入延迟 L_total（重采样有轻微纹波，按相关判）
    #[test]
    fn worker_and_forwarder_align_end_to_end() {
        let (block, lead) = (480, 1440);
        let mut w = Worker::new(identity_pipeline().unwrap(), block);
        let l_total = w.l_total(lead);
        let bus = heap(l_total);
        w.prime(&bus, lead);
        let mut f = Forwarder::new(ForwarderConfig::new(l_total, block));
        let x = vdn_capture::testing::test_tone(IO_RATE, 200 * block, 0);
        let mut y = vec![0.0; x.len()];
        for (c, yc) in x.chunks(block).zip(y.chunks_mut(block)) {
            forward(&mut f, &bus, c);
            w.step(&bus);
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
}
