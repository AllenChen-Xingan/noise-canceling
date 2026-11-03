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

/// 降噪总线头的配置：总线头里的 L_total / lead / 降噪块长**只从这里写**。
/// 正本：管线延迟与块长 = `Worker`（来自 `Pipeline`），lead 与容量 = 调用方的 `vdn_control::Config`。
pub fn denoise_bus_config<P: Processor>(w: &Worker<P>, lead: usize, capacity: usize) -> Result<BusConfig, String> {
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
            }
        }
        w.mean_ns = total / blocks.max(1) as u64;
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
        let t0 = std::time::Instant::now();
        self.proc.process(&self.inbuf, &mut self.outbuf);
        let dt = u64::try_from(t0.elapsed().as_nanos()).unwrap_or(u64::MAX);
        self.stats.max_process_ns = self.stats.max_process_ns.max(dt);
        self.stats.total_process_ns = self.stats.total_process_ns.saturating_add(dt);
        if dt > self.block_ns() {
            self.stats.over_budget += 1;
        }
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
