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
use vdn_shm::BusConfig;

mod worker;

pub use worker::{Step, Warmup, Worker, WorkerStats};

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

#[cfg(test)]
mod tests;

#[cfg(all(test, debug_assertions))]
#[global_allocator]
static A: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;
