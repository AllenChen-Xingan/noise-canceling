//! engine 接口的最小镜像（`specs/rust/architecture.md` §3.3）。
//!
//! 本批 p2 只新建自己的 crate，不依赖 `vdn-types`（align 在建）；接入 engine 时把这里的类型换成 `vdn-types` 的，
//! 字段与语义按下面写的对齐。比规约多出来的字段（`SpeakerVerdict` 的时间戳与相似度）是给 align 的提议。

/// 判断回路给出的「现在谁在说」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeakerClass {
    /// 没注册好、或相似度落在两档阈值之间：一律当「不压」
    Unknown,
    /// 窗里人声太少，判不了人：门控保持现状（不因为停顿而放开，否则背景忽大忽小）
    NoSpeech,
    /// 窗里是主人（或主人加别人）
    Owner,
    /// 窗里有人声，且确认只有别人在说
    OthersOnly,
}

/// 判决结果。由判断回路（非实时线程，约 0.5 s 一次）发布，实时线程每 hop 读最新一份。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeakerVerdict {
    pub class: SpeakerClass,
    /// 这份判决所用窗口的起点（模型采样率下的样本序号）。门控据此判断「判决是否覆盖了最近一次人声起始」
    pub window_start: u64,
    /// 窗口终点（不含）
    pub window_end: u64,
    /// 与主人声纹的余弦相似度；未注册时为 NaN
    pub owner_sim: f32,
    /// 注册器记下的主人说话电平（dB，降噪输出上有声帧的均方）；未注册时为 NaN。门控用它分辨「近讲起始」
    pub owner_level_db: f32,
}

impl SpeakerVerdict {
    pub const UNKNOWN: Self = Self {
        class: SpeakerClass::Unknown,
        window_start: 0,
        window_end: 0,
        owner_sim: f32::NAN,
        owner_level_db: f32::NAN,
    };
}

/// 每 hop 传给 Stage 的上下文（按值传）。
#[derive(Clone, Copy, Debug)]
pub struct FrameCtx {
    pub speaker: SpeakerVerdict,
    /// 本 hop 第一个样本的序号（规约：时间以 Pipeline 的样本计数为准）
    pub sample_pos: u64,
}

/// 降噪之后的处理级，跑在模型采样率；`wet` 原地改，`dry` 是与 `wet` 对齐的原声。
/// 规约 §3.3：P2 前所有 Stage 延迟必须为 0。
pub trait Stage: Send {
    fn reset(&mut self);
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
}
