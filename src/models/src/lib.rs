//! vdn-models：具体模型的适配器（实现 `vdn_types::Model`）。ONNX Runtime 只在这一个 crate 里出现。
//!
//! 实时安全的现状（详见 exp004 README）：适配器自己的代码不分配；但 `ort` 的 `run` 在 Rust 侧仍有少量分配，
//! ORT C++ 内部的分配也绕过 Rust 分配器——这是登记在案的债，推理线程按软实时管。

pub mod fastenhancer;
pub mod gtcrn;
mod session;

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

    /// 打开模型。
    pub fn open(self, path: &Path) -> Result<AnyModel, BoxError> {
        Ok(match self {
            ModelKind::Gtcrn => AnyModel::Gtcrn(Gtcrn::open(path)?),
            ModelKind::FastEnhancerM => AnyModel::FastEnhancer(FastEnhancer::open(path)?),
        })
    }
}

/// 运行期可选的模型（rustarch 初审：手写 enum 静态分发，不用 `Box<dyn>`；两个变体不值得上宏）。
// 只在启动时建一次、不在实时路径上搬动，大小差异无所谓
#[allow(clippy::large_enum_variant)]
pub enum AnyModel {
    Gtcrn(Gtcrn),
    FastEnhancer(FastEnhancer),
}

macro_rules! each {
    ($self:ident, $m:ident => $e:expr) => {
        match $self {
            AnyModel::Gtcrn($m) => $e,
            AnyModel::FastEnhancer($m) => $e,
        }
    };
}

impl vdn_types::Model for AnyModel {
    fn name(&self) -> &'static str {
        each!(self, m => m.name())
    }
    fn sample_rate(&self) -> u32 {
        each!(self, m => m.sample_rate())
    }
    fn hop(&self) -> usize {
        each!(self, m => m.hop())
    }
    fn latency_samples(&self) -> usize {
        each!(self, m => m.latency_samples())
    }
    fn reset(&mut self) {
        each!(self, m => m.reset())
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), vdn_types::ModelError> {
        each!(self, m => m.process(input, output))
    }
}
