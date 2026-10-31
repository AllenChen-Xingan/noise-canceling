//! 过渡期兼容：`Model` / `ModelError` 已搬进 `vdn-types`（规约 §2.3、§3.1），这里只转出，旧路径 `vdn_engine::model::Model` 仍可用。

pub use vdn_types::{Model, ModelError};
