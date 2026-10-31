//! engine 接口的最小镜像（`specs/rust/architecture.md` §3.3）。
//!
//! 本批 p2 只新建自己的 crate，不依赖 `vdn-types`（align 在建）。这里**逐字段照 align 的 `vdn-types` 草案**
//! （`../noise_canceling-align/src/types/src/frame.rs`、`stage.rs`，2026-10-30 按 p2 的提议加了判决时间戳、
//! NoSpeech、主人电平、`FrameCtx::{now, voice_prob}`），接入时把 `use crate::types::…` 换成 `use vdn_types::…`。
//!
//! `SampleClock` 在 vdn-types 里按 **IO 采样率**计。本 crate 凡拿时长和 `SampleClock` 比，都按参数里的 `clock_rate` 换算
//! （`GateParams::clock_rate`、`JudgeParams::clock_per_sample`）；离线实验里时钟 = 16 kHz 模型样本，接入时设成 IO 采样率。

/// 样本计数（时间戳一律用它）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleClock(pub u64);

impl SampleClock {
    pub fn seconds(self, rate: u32) -> f64 {
        if rate == 0 {
            0.0
        } else {
            self.0 as f64 / f64::from(rate)
        }
    }
    pub fn since(self, earlier: SampleClock) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

/// 0..=1 的置信度；NaN 记为 0。
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Confidence(f32);

impl Confidence {
    pub const ZERO: Confidence = Confidence(0.0);
    pub const ONE: Confidence = Confidence(1.0);
    pub fn new(v: f32) -> Self {
        if v.is_nan() {
            Self(0.0)
        } else {
            Self(v.clamp(0.0, 1.0))
        }
    }
    pub fn get(self) -> f32 {
        self.0
    }
}

/// 判断回路给出的「现在谁在说」。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpeakerClass {
    /// 没注册好、或相似度落在两档阈值之间：放开（不压）
    #[default]
    Unknown,
    /// 窗里人声太少，判不了人：门控保持现状（不因为停顿而放开，否则背景忽大忽小）
    NoSpeech,
    /// 窗里是主人（或主人加别人）
    Target,
    /// 窗里有人声，且确认只有别人在说
    Other,
}

/// 判决结果。由判断回路（非实时线程，约 0.5 s 一次）发布，实时线程每 hop 读最新一份。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpeakerVerdict {
    pub class: SpeakerClass,
    /// 本 crate 填「与主人声纹的余弦相似度」夹到 0..=1；未注册为 0
    pub confidence: Confidence,
    /// 判决所用声纹窗的起点 / 终点（不含）
    pub window_start: SampleClock,
    pub window_end: SampleClock,
    /// 注册器记下的主人说话电平（dB，降噪输出上有声帧的均方）；未注册为 None。门控据此分辨「近讲起始」
    pub owner_level_db: Option<f32>,
}

impl SpeakerVerdict {
    /// 判决窗整个落在 `onset` 之后才作数（起始之前算的判决不能拿来压刚开口的人）。
    pub fn valid_after(&self, onset: SampleClock) -> bool {
        self.window_start >= onset
    }
}

/// 每 hop 传给 Stage 的上下文（按值传）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameCtx {
    /// 本 hop 第一个样本的位置
    pub now: SampleClock,
    pub speaker: SpeakerVerdict,
    /// 本 hop 的人声概率（earshot，降噪输出上），没有 VAD 时为 None
    pub voice_prob: Option<f32>,
}

/// 降噪之后的处理级，跑在模型采样率；`wet` 原地改，`dry` 是与 `wet` 对齐的原声。
/// 规约 §3.3：P2 前所有 Stage 延迟必须为 0。
pub trait Stage: Send {
    fn name(&self) -> &'static str;
    fn reset(&mut self);
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
    /// align 草案另有 `fn set_params(&mut self, _p: &ParamSet) {}`（默认空，判断回路下发参数用）；
    /// `ParamSet` 在 vdn-types 里，本镜像不复制，接入时门控从那里取 floor_db 等
    /// 本级附加延迟（样本）；进延迟账，本 crate 的级全部为 0
    fn latency_samples(&self) -> usize {
        0
    }
}
