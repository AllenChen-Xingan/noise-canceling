//! 频域后处理级（草案扩展 trait，polish 自有）：模型适配器在「模型输出频谱 → iSTFT」之间调用，
//! 吃同一帧的带噪频谱与降噪后频谱（`specs/rust/architecture.md` §3.1「共用一次 STFT 以后用扩展 trait 补」）。
//!
//! 时域那一半（`Stage`、`FrameCtx`）的正本在 `vdn-types`；这个 trait 在 `vdn-types` 里还没有对应物，
//! 第二阶段「处理图」接 engine 时再定要不要搬进 `vdn-types`（大总管定 2026-11-02，方案 A）。

use vdn_types::FrameCtx;

pub use realfft::num_complex::Complex32;

/// 两份频谱必须来自同一套 STFT（同窗、同 hop、同帧位置）。
pub trait SpectralStage: Send {
    fn name(&self) -> &'static str;
    fn reset(&mut self);
    fn process_spectrum(&mut self, wet: &mut [Complex32], dry: &[Complex32], ctx: FrameCtx);
}
