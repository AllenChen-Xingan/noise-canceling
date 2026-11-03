//! `Pipeline` 构造失败的原因（构造期，非实时）。

use core::fmt;

use vdn_types::ChainMismatch;

/// 构造失败。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineError {
    /// IO 采样率必须正好是模型采样率的 3 倍（本批只有固定 3 倍重采样）
    UnsupportedRate { io_rate: u32, model_rate: u32 },
    /// `Pipeline::with_block` 的块长必须 > 0 且整除 48 kHz hop
    BlockNotAligned { block: usize, hop_io: usize },
    /// 后处理级的延迟必须为 0（规约 §3.3：「每级 dry 对齐」做出来之前）
    StageLatency { stage: &'static str, samples: usize },
    /// 实际级串与处理图描述对不上（种类 · 顺序 · 延迟，`vdn_types::match_chain`）
    Graph(ChainMismatch),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineError::UnsupportedRate {
                io_rate,
                model_rate,
            } => write!(
                f,
                "IO 采样率 {io_rate} Hz 不是模型采样率 {model_rate} Hz 的 3 倍（本批只支持固定 3 倍重采样）"
            ),
            PipelineError::BlockNotAligned { block, hop_io } => write!(
                f,
                "块长 {block} 不整除 IO 采样率下的 hop {hop_io}（定块长省延迟要求块长 > 0 且整除 hop）"
            ),
            PipelineError::StageLatency { stage, samples } => write!(
                f,
                "后处理级 {stage} 声明延迟 {samples} 个样本；本批只许 0 延迟的级（规约 §3.3）"
            ),
            PipelineError::Graph(m) => write!(f, "后处理级串与处理图描述不一致：{m}"),
        }
    }
}

impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PipelineError::Graph(m) => Some(m),
            _ => None,
        }
    }
}
