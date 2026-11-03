//! 模型制品内容 ID（#68）：权重文件全部字节的 FNV-1a（`ContentId::of`，与 `ParamBlock::model_hash` 同一算法）。
//! 进处理图的模型节点（`ProcessingGraph::artifact`），同名模型换了权重它就变。
#![allow(clippy::disallowed_methods)] // NRT: 构造期读整个权重文件算哈希，不在实时路径上

use std::path::Path;

use vdn_backend_ort::BoxError;
use vdn_types::ContentId;

/// 读整个权重文件，算制品内容 ID。构造期用。
pub fn artifact_id(path: &Path) -> Result<ContentId, BoxError> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}：{e}", path.display()))?;
    Ok(ContentId::of(&bytes))
}
