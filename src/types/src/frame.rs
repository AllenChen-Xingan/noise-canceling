//! 逐帧传递的小类型（规约 §3.3）。全部 `Copy`，按值传，不分配。

/// 流时间：自开流起的 **IO 采样率** 样本数（规约 §3.3：时间以 Pipeline 的样本计数为准）。
/// 判断回路、决策日志、感知输出都用它打时间戳，回放时与音频逐样本对得上。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleClock(pub u64);

impl SampleClock {
    /// 换算成秒（`rate` = IO 采样率）。
    pub fn seconds(self, rate: u32) -> f64 {
        if rate == 0 {
            0.0
        } else {
            self.0 as f64 / f64::from(rate)
        }
    }

    /// `self - earlier`，不会下溢（时钟回绕或乱序时返回 0）。
    pub fn since(self, earlier: SampleClock) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

/// 置信度，构造时夹到 `0.0..=1.0`，NaN 当 0。
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

/// 声纹判决的类别。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpeakerClass {
    /// 还没注册 / 不确定：门控**放开、不压**
    #[default]
    Unknown,
    /// 窗里人声太少、判不了：门控**保持现状**（停顿里不反复开合）
    NoSpeech,
    /// 是注册的那个人
    Target,
    /// 是别人
    Other,
}

/// 声纹判决（`vdn-speaker` 产出，经 triple_buffer 送进实时线程，规约 §3.4）。
///
/// 带判决所用声纹窗的起止时刻：门控规则「新人声起始之后算出来的判决才作数」——起始前的 `Other`
/// 不能拿来压人，否则邻座之后你开口的第一个字会被吞（p2 提议，2026-10-31）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeakerVerdict {
    pub class: SpeakerClass,
    pub confidence: Confidence,
    /// 判决所用声纹窗的首样本（IO 采样率）
    pub window_start: SampleClock,
    /// 判决所用声纹窗的末样本之后（IO 采样率）
    pub window_end: SampleClock,
    /// 主人说话电平（dBFS）；`None` = 还没注册。门控用它判「近讲起始」，远场更轻的起始不触发放开
    pub owner_level_db: Option<f32>,
}

impl Default for SpeakerVerdict {
    fn default() -> Self {
        Self {
            class: SpeakerClass::Unknown,
            confidence: Confidence::ZERO,
            window_start: SampleClock(0),
            window_end: SampleClock(0),
            owner_level_db: None,
        }
    }
}

impl SpeakerVerdict {
    /// 这个判决是否在 `onset`（一段新人声的起始）之后才算出来、可以拿来动作。
    pub fn valid_after(&self, onset: SampleClock) -> bool {
        self.window_start >= onset
    }
}

/// 每帧上下文。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameCtx {
    /// 本 hop 首样本的流时间（IO 采样率；规约 §3.3「时间以 Pipeline 的样本计数为准」）
    pub now: SampleClock,
    pub speaker: SpeakerVerdict,
    /// 本帧人声概率 0..=1（VAD，如 earshot，每 16 ms）；`None` = 没跑 VAD。门控与修音链 AGC 读它
    pub voice_prob: Option<f32>,
}

/// 实时 → 判断回路的每帧特征包（ondevice 需求 ③）：每 hop 一个，经 rtrb 送出，**满了丢弃并计数**，实时侧绝不等。
/// 噪声谱复用模型的 STFT，不另算。字段是草案，由感知（scene 线）定稿。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameFeatures {
    /// 本 hop 首样本（IO 采样率）
    pub now: SampleClock,
    /// 输入电平（dBFS）
    pub level_db: f32,
    /// 底噪跟踪值（dBFS）
    pub noise_db: f32,
    /// 估计信噪比（dB）
    pub snr_db: f32,
    /// 人声概率（VAD）；`None` = 没跑
    pub voice_prob: Option<f32>,
    /// 噪声谱（按 [`FrameFeatures::BANDS`] 个频带汇总的 dB），取自模型 STFT
    pub bands_db: [f32; FrameFeatures::BANDS],
    /// 本 hop 是否整段送了原声
    pub bypassed: bool,
}

impl FrameFeatures {
    pub const BANDS: usize = 16;
}

/// 为什么整段送原声。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BypassReason {
    Backend,
    NonFinite,
    Abnormal,
    Deadline,
    BadLen,
}

/// 一次 `process` 的结果（规约 §3.3）。
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    /// 本 hop 整段原声
    Bypassed(BypassReason),
    /// 故障后多送原声的保持期
    Holding,
    /// 原声 → 模型的淡化期
    Crossfading,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_clamps_and_kills_nan() {
        assert_eq!(Confidence::new(1.5).get(), 1.0);
        assert_eq!(Confidence::new(-0.1).get(), 0.0);
        assert_eq!(Confidence::new(f32::NAN).get(), 0.0);
    }

    #[test]
    fn verdict_only_counts_after_onset() {
        let v = SpeakerVerdict {
            class: SpeakerClass::Other,
            window_start: SampleClock(1000),
            window_end: SampleClock(2000),
            ..Default::default()
        };
        assert!(v.valid_after(SampleClock(1000)));
        assert!(!v.valid_after(SampleClock(1500)));
    }

    #[test]
    fn clock_never_underflows() {
        assert_eq!(SampleClock(5).since(SampleClock(9)), 0);
        assert_eq!(SampleClock(48_000).seconds(48_000), 1.0);
        assert_eq!(SampleClock(1).seconds(0), 0.0);
    }
}
