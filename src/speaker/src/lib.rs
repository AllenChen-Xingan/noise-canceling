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
//! vdn-speaker：认出你的声音（P2 离线，`docs/plan/003_tech-design.md` 第二节）。
//!
//! 两条回路：
//! - **实时回路**（每 hop，16 kHz）：[`envelope::VadEnvelopeStage`]（VAD 驱动的快开慢关 + 衰减下限）→
//!   [`gate::AsymGate`]（不对称门控）。零前瞻、零算法延迟，`process` 不分配
//! - **判断回路**（约 0.5 s 一次，非实时线程）：[`judge::Judge`] = fbank → 声纹（[`embed::Embedder`]，ONNX）→
//!   [`enroll::Enroller`]（零设置注册）→ `vdn_types::SpeakerVerdict`。只改参数、不碰声音
//!
//! 实时级按 B 档写（不分配、不加锁、不 panic）；判断回路是 C 档。
//! 与 engine 的接口（`Stage` · `FrameCtx` · `SpeakerVerdict`）直接用 `vdn-types`（正本），本 crate 不再留副本。

pub mod enroll;
pub mod envelope;
pub mod fbank;
pub mod gate;
pub mod judge;

pub mod vad;

#[cfg(feature = "onnx")]
pub mod embed;
pub mod vecmath;


