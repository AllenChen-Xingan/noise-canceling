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

/// 声纹判决（`vdn-speaker` 产出，经 triple_buffer 送进实时线程，规约 §3.4）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SpeakerVerdict {
    /// 还没注册 / 还没判出来：门控不动作
    #[default]
    Unknown,
    /// 是注册的那个人
    Target(Confidence),
    /// 是别人
    Other(Confidence),
}

/// 每帧上下文。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameCtx {
    pub speaker: SpeakerVerdict,
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
    fn clock_never_underflows() {
        assert_eq!(SampleClock(5).since(SampleClock(9)), 0);
        assert_eq!(SampleClock(48_000).seconds(48_000), 1.0);
        assert_eq!(SampleClock(1).seconds(0), 0.0);
    }
}
