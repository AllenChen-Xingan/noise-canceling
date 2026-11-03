//! vdn-types：全 workspace 共用的词汇（`specs/rust/architecture.md` §2.1、§3）。
//!
//! 零依赖，不做 I/O，不认识具体模型、平台、推理库。分三块：
//!
//! | 块 | 模块 | 跑在哪档 |
//! |---|---|---|
//! | 反射回路（10 ms，逐帧） | [`model`] · [`frame`] · [`stage`] | B：实时，不分配 |
//! | 判断回路（约 0.5 s） | [`scene`]（感知输出）· [`params`] · [`table`]（场景参数表）· [`policy`]（策略） | C：不碰声音，只出参数 |
//! | 学习与记录（数天） | [`memory`]（我的配置）· [`record`]（决策事件）· [`session`]（会话记录） | C：空闲时 |
//! | 横切 | [`latency`]（延迟账）· [`version`]（版本号与内容 ID）· [`graph`]（处理图描述）· [`rate`]（采样率正本） | 构造期 |
//!
//! 接口骨架只定类型与纯函数；场景分类、声纹、学习的**实现**不在这里（D35 · D40 · D41，`docs/plan/005_evolution-and-simulation.md`）。
//! 序列化用自写的行文本格式（[`text`]），不引 serde：核心 crate 不依赖 serde（规约 §2.2 规则 1）。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]

pub mod desc;
pub mod frame;
pub mod graph;
pub mod latency;
pub mod memory;
pub mod model;
pub mod params;
pub mod policy;
pub mod rate;
pub mod record;
pub mod scene;
pub mod session;
pub mod stage;
pub mod table;
pub mod text;
pub mod version;

pub use desc::{ParamDesc, RangeError, Unit};
pub use frame::{
    BypassReason, Confidence, FrameCtx, FrameFeatures, SampleClock, SpeakerClass, SpeakerVerdict,
    Status,
};
pub use graph::{ChainMismatch, Hook, NodeKind, ProcessingGraph, match_chain};
pub use latency::{
    Basis, IO_BLOCK_009, IO_BLOCK_PROPOSED, LATENCY_BUDGET_MS, LatencyItem, LatencyLedger,
};
pub use memory::{Learner, Overrides, SceneKey, UserProfile};
pub use model::{Model, ModelError, SpectrumTap};
pub use params::{ModelId, ParamBlock, ParamError, ParamField, ParamSet, ToneId};
pub use policy::{
    Decision, Hysteresis, HysteresisGate, ParamSource, Policy, PolicyInput, TablePolicy,
    Transition, UserControl, UserSceneChoice, resolve_params,
};
pub use rate::{IO_RATE, MODEL_RATE};
pub use record::{DecisionEvent, EnrollEvent, FeedbackKind, UserAction};
pub use scene::{FineSceneId, SceneLabel, SceneName, UserScene};
pub use session::{HeaderKey, SessionHeader, SessionRecord};
pub use stage::{Stage, StageChain};
pub use table::{SceneEntry, SceneTable, TableError};
pub use version::{ConfigBinding, ContentId, ProfileVersion, TableVersion};
