//! 模型适配器接口。每个模型（GTCRN、FastEnhancer-M、以后的别的）实现它，engine 只认这个 trait。
//!
//! 约定：
//! - 一律「一个 hop 进、一个 hop 出」的波形接口；频域模型自己在适配器里做 STFT/OLA（用 `vdn-dsp`）
//! - 输出比输入晚 `latency_samples()` 个样本（engine 用它对齐原声）
//! - `process` 在实时线程上跑：不分配、不加锁、不做 I/O；出错返回 `Err`，不 panic
//! - `reset` 把内部状态（RNN 缓存、STFT 缓冲）清回初值，可以在实时线程上调
//!
//! 以后要加的东西不改这个 trait（rustarch 初审）：实时回调块长 ≠ hop 时，外面加定长分块层；
//! GTCRN ↔ FE-M 切换由外层同时持有两套「模型 + 分块」、补齐到较大延迟后交叉淡化，不塞进 `Engine`；
//! 声纹门控做成独立的处理级；需要共享频谱时再加扩展 trait。

use std::fmt;

/// 适配器出错的原因。不带 `String`，保证出错路径也不分配。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelError {
    /// 推理后端报错（例如 ONNX Runtime 返回错误）
    Backend,
    /// 输出形状和声明的不一致
    Shape,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::Backend => write!(f, "推理后端报错"),
            ModelError::Shape => write!(f, "输出形状不对"),
        }
    }
}

impl std::error::Error for ModelError {}

/// `Send`：模型要搬到实时线程上跑。
pub trait Model: Send {
    /// 短名，用于日志与结果文件（例如 `"gtcrn"`）
    fn name(&self) -> &'static str;
    /// 采样率（Hz）
    fn sample_rate(&self) -> u32;
    /// 每次 `process` 进出的样本数
    fn hop(&self) -> usize;
    /// 输出比输入晚的样本数
    fn latency_samples(&self) -> usize;
    /// 状态清回初值
    fn reset(&mut self);
    /// 处理一个 hop。`input.len() == output.len() == hop()`。
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError>;
}
