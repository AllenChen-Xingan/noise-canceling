//! 模型适配器接口（从 p1 的 `vdn-engine::model` 原样搬来，规约 §3.1）。
//!
//! 约定：
//! - 一律「一个 hop 进、一个 hop 出」的波形接口；频域模型自己在适配器里做 STFT/OLA（用 `vdn-dsp`）
//! - 输出比输入晚 `latency_samples()` 个样本（engine 用它对齐原声、记延迟账）
//! - `process` 在实时线程上跑：不分配、不加锁、不做 I/O；出错返回 `Err`，不 panic
//! - `reset` 把内部状态清回初值，可以在实时线程上调
//! - 构造（打开会话、预分配、预热）在非实时线程
//!
//! 以后要加的东西不改这个 trait：共享频谱用扩展 trait；换模型由外层持有两套并交叉淡化（规约 §3.5）。

use core::fmt;

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
    /// 输出比输入晚的样本数（算法延迟，不含攒满一个 hop 的等待；那一项由 engine 记账）
    fn latency_samples(&self) -> usize;
    /// 状态清回初值
    fn reset(&mut self);
    /// 处理一个 hop。`input.len() == output.len() == hop()`。
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError>;
}
