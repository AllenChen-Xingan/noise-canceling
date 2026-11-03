//! 频域后处理级（polish 自有的扩展 trait）：模型适配器在「模型输出频谱 → iSTFT」之间调用，
//! 吃同一帧的带噪频谱与降噪后频谱（`specs/rust/architecture.md` §3.1「共用一次 STFT 以后用扩展 trait 补」）。
//!
//! 归属（graph 线定 2026-11-02）：**trait 留在 polish，节点种类进 `vdn-types` 的处理图**。
//! - 图里它挂在模型节点上（`ProcessingGraph::spectral`，挂点 `Hook::Spectral`），种类正本是
//!   `vdn_types::graph::POLISH_SPECTRAL`；实现只返回种类，名字从种类取（与时域 `Stage` 同一规矩）
//! - 不搬进 `vdn-types`：那边零依赖，要自带复数类型，每帧还得与 `realfft` 的 `Complex32` 互转（复制，或
//!   `unsafe` 重解释，而 types 是 `forbid(unsafe_code)`）；现在只有 polish 一个实现、还没有调用方（engine 不做 STFT）。
//!   等模型适配器真正调用频域级时，在适配器构造处照 `match_chain` 比对 `spectral` 段，那时再看要不要搬

use vdn_types::{FrameCtx, NodeKind};

pub use realfft::num_complex::Complex32;

/// 两份频谱必须来自同一套 STFT（同窗、同 hop、同帧位置）。
pub trait SpectralStage: Send {
    /// 本级的节点种类（挂点必须是 `Hook::Spectral`）
    fn kind(&self) -> &'static NodeKind;
    fn reset(&mut self);
    fn process_spectrum(&mut self, wet: &mut [Complex32], dry: &[Complex32], ctx: FrameCtx);
}
