//! 原子类型门面：平时是 `core::sync::atomic`；开 feature `loom` 时换成 loom 的，让 loom 模型检查**同一份**环协议代码。
//! 只换跨线程协议用到的原子量（索引 · 计数 · 唤醒计数字 · 样本槽）；进程内的开关（如 `wake_broken`）不换。

#[cfg(feature = "loom")]
pub use loom::sync::atomic::{AtomicU32, AtomicU64};

#[cfg(not(feature = "loom"))]
pub use core::sync::atomic::{AtomicU32, AtomicU64};
