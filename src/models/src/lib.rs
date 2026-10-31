//! vdn-models：具体模型的适配器（实现 `vdn_engine::Model`）。ONNX Runtime 只在这一个 crate 里出现。
//!
//! 实时安全的现状（详见 exp004 README）：适配器自己的代码不分配；但 `ort` 的 `run` 在 Rust 侧仍有少量分配，
//! ORT C++ 内部的分配也绕过 Rust 分配器——这是登记在案的债，推理线程按软实时管。

pub mod fastenhancer;
pub mod gtcrn;
pub mod session;

use std::path::{Path, PathBuf};

pub use fastenhancer::FastEnhancer;
pub use gtcrn::Gtcrn;
pub use session::BoxError;

/// 可选的模型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelKind {
    Gtcrn,
    FastEnhancerM,
}

impl ModelKind {
    /// 模型文件的默认查找顺序：环境变量 → `~/vdn-data/models/…` → 当前目录下的 `resources/…`。
    pub fn default_path(self) -> Option<PathBuf> {
        let (env, data_rel, res_rel) = match self {
            ModelKind::Gtcrn => (
                "VDN_GTCRN_ONNX",
                "vdn-data/models/gtcrn/gtcrn_simple.onnx",
                "resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx",
            ),
            ModelKind::FastEnhancerM => (
                "VDN_FASTENHANCER_ONNX",
                "vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx",
                "resources/weights/fastenhancer/fastenhancer_m.onnx",
            ),
        };
        let mut candidates = Vec::new();
        if let Some(p) = std::env::var_os(env) {
            candidates.push(PathBuf::from(p));
        }
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(Path::new(&home).join(data_rel));
        }
        candidates.push(PathBuf::from(res_rel));
        candidates.into_iter().find(|p| p.is_file())
    }

    /// 打开模型，装进 `Box<dyn Model>`。
    pub fn open(self, path: &Path) -> Result<Box<dyn vdn_engine::Model + Send>, BoxError> {
        Ok(match self {
            ModelKind::Gtcrn => Box::new(Gtcrn::open(path)?),
            ModelKind::FastEnhancerM => Box::new(FastEnhancer::open(path)?),
        })
    }
}
