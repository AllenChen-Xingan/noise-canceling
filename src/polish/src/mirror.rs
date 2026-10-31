//! 对 engine 接口的最小镜像（common.md 硬约束 3：scene / p2 只新建自己的 crate，接入 engine 由下一批统一做）。
//!
//! 照 `specs/rust/architecture.md` §3.3 的 `trait Stage` 写；`SpectralStage` 是 §3.1 说的「共用一次 STFT」扩展 trait 的草案。
//! align 的 `vdn-types` 草案在 `../noise_canceling-align/src/types/src/{frame,stage}.rs`；定稿后这里整个删掉，改用 `vdn_types::{FrameCtx, Stage}`，字段差异以 `vdn-types` 为准。

/// 每帧随路传的上下文（镜像）。vdn-types 草案（align，2026-10-31）只有 `speaker: SpeakerVerdict`；
/// `voice_prob` 是 p2 向 align 提议加的字段（vdn-speaker 的 earshot VAD，每 16 ms 一帧，[0, 1]），没有时填 `None`。
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameCtx {
    pub voice_prob: Option<f32>,
}

/// 时域后处理级（镜像 §3.3）：跑在模型采样率，一个 hop 进出，不加延迟。
pub trait Stage: Send {
    fn name(&self) -> &'static str;
    fn reset(&mut self);
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
    /// 本级引入的延迟（模型采样率样本）；修音链各节点都是 0
    fn latency_samples(&self) -> usize {
        0
    }
}

/// 频域后处理级（草案）：模型适配器在「模型输出频谱 → iSTFT」之间调用，吃同一帧的带噪频谱与降噪后频谱。
/// 两份频谱必须来自同一套 STFT（同窗、同 hop、同帧位置）。
pub trait SpectralStage: Send {
    fn name(&self) -> &'static str;
    fn reset(&mut self);
    fn process_spectrum(&mut self, wet: &mut [Complex32], dry: &[Complex32], ctx: FrameCtx);
}

pub use realfft::num_complex::Complex32;
