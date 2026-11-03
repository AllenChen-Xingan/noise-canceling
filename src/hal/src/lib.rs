//! vdn-hal：自有虚拟麦克风（Core Audio HAL 插件，D55：用 Rust 写）。
//!
//! 链路位置（`docs/plan/010_rust-skeleton.md` 第十节）：送音进程把最终声音推进共享内存的 `Out` 环，
//! 插件在 coreaudiod 的 IO 线程里调 [`HalSource::render`] 取出来，交给会议软件。
//!
//! 本批只做：
//! - [`HalSource`]：从一个 [`SampleSource`]（将来是 `vdn-shm` 的 `Out` 环消费端）取声音，不够补零并计欠载。实时块零分配
//! - [`factory::VdnHalFactory`]：CFPlugIn 工厂函数，**返回空指针 = 不加载**。真驱动接口（`AudioServerPlugInDriverInterface`）待实现（D05）
//! - `bundle/Info.plist` 模板
//!
//! 不安装、不复制进 `/Library/Audio/Plug-Ins/HAL/`（要人输密码，D05 另约时间）。

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

pub mod factory;
pub mod source;

pub use source::{FnSource, HalCounters, HalSource, RenderStatus, SampleSource};
