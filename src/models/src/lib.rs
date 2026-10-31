//! vdn-models：具体模型的适配器（实现 `vdn_types::Model`）。ONNX Runtime 只在这一个 crate 里出现。
//!
//! 实时安全的现状（详见 exp004 README）：适配器自己的代码不分配；但 `ort` 的 `run` 在 Rust 侧仍有少量分配，
//! ORT C++ 内部的分配也绕过 Rust 分配器——这是登记在案的债，推理线程按软实时管。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing
    )
)]
pub mod fastenhancer;
pub mod gtcrn;
mod session;

use std::path::{Path, PathBuf};

use vdn_types::{Basis, LatencyItem, LatencyLedger, ModelId};

pub use fastenhancer::FastEnhancer;
pub use gtcrn::Gtcrn;
pub use session::BoxError;

/// 可选的模型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelKind {
    Gtcrn,
    FastEnhancerM,
}

/// 模型声明的流式形态（不打开文件就能知道；打开后由 `tests/smoke.rs` 核对）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declared {
    pub sample_rate: u32,
    pub hop: usize,
    /// `Model::latency_samples()`
    pub latency_samples: usize,
}

impl ModelKind {
    pub const ALL: [ModelKind; 2] = [ModelKind::Gtcrn, ModelKind::FastEnhancerM];

    /// 参数表与决策日志里的模型编号。
    pub fn id(self) -> ModelId {
        match self {
            ModelKind::Gtcrn => ModelId::Gtcrn,
            ModelKind::FastEnhancerM => ModelId::FastEnhancerM,
        }
    }

    pub fn from_id(id: ModelId) -> Self {
        match id {
            ModelId::Gtcrn => ModelKind::Gtcrn,
            ModelId::FastEnhancerM => ModelKind::FastEnhancerM,
        }
    }

    /// 声明的 hop 与延迟（GTCRN：STFT 512/256；FE-M wav2wav：图内 STFT 512、hop 160，exp004 实测对拍）。
    pub const fn declared(self) -> Declared {
        match self {
            ModelKind::Gtcrn => Declared {
                sample_rate: 16_000,
                hop: 256,
                latency_samples: 256,
            },
            ModelKind::FastEnhancerM => Declared {
                sample_rate: 16_000,
                hop: 160,
                latency_samples: 352,
            },
        }
    }

    /// 本模型在延迟账上的两项：攒满一个 hop + 算法延迟（与 `Engine::latency_ledger` 同口径）。
    pub fn latency_ledger(self) -> LatencyLedger {
        let d = self.declared();
        let mut l = LatencyLedger::new();
        l.push(LatencyItem::new(
            "hop-buffer",
            d.hop as u32,
            d.sample_rate,
            Basis::Declared,
            "攒满一个模型 hop 才能处理",
        ))
        .push(LatencyItem::new(
            self.id().as_str(),
            d.latency_samples as u32,
            d.sample_rate,
            Basis::Declared,
            "模型算法延迟",
        ));
        l
    }

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

// 实时（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
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
