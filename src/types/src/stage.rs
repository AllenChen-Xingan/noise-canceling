//! 后处理级（门控、衰减下限、音色链节点），跑在模型采样率（规约 §3.3）。

use crate::frame::FrameCtx;

pub trait Stage: Send {
    /// 短名，进延迟账与决策日志
    fn name(&self) -> &'static str;
    /// 状态清回初值；可在实时线程调
    fn reset(&mut self);
    /// 原地处理一个 hop。`wet` 是模型输出，`dry` 是对齐后的原声。不分配、不 panic
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
    /// 本级引入的延迟（模型采样率样本）。规约 §3.3：在「每级 dry 对齐」做出来之前必须为 0；
    /// 修音链零前瞻，带宽扩展例外（≤ 1 ms，本批硬约束 1）
    fn latency_samples(&self) -> usize {
        0
    }
}
