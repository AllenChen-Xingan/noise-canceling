//! vdn-speaker：认出你的声音（P2 离线，`docs/plan/003_tech-design.md` 第二节）。
//!
//! 两条回路：
//! - **实时回路**（每 hop，16 kHz）：[`envelope::VadEnvelopeStage`]（VAD 驱动的快开慢关 + 衰减下限）→
//!   [`gate::AsymGate`]（不对称门控）。零前瞻、零算法延迟，`process` 不分配
//! - **判断回路**（约 0.5 s 一次，非实时线程）：[`judge::Judge`] = fbank → 声纹（[`embed::Embedder`]，ONNX）→
//!   [`enroll::Enroller`]（零设置注册）→ [`types::SpeakerVerdict`]。只改参数、不碰声音
//!
//! 与 engine 的接口按 `specs/rust/architecture.md` §3.3 在 [`types`] 里写了最小镜像，接入由下一批统一做。

pub mod enroll;
pub mod envelope;
pub mod fbank;
pub mod gate;
pub mod judge;
pub mod types;
pub mod vad;

#[cfg(feature = "onnx")]
pub mod embed;
#[cfg(not(feature = "onnx"))]
pub mod embed {
    //! 不带 ONNX 时只留向量小工具。
    pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
    pub fn l2_normalize(v: &mut [f32]) {
        let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if n > 0.0 && n.is_finite() {
            for x in v.iter_mut() {
                *x /= n;
            }
        }
    }
    pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }
}

pub use types::{FrameCtx, SpeakerClass, SpeakerVerdict, Stage};
