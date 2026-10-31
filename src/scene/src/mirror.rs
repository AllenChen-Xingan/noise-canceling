//! `vdn-types` 的最小镜像（common.md 硬约束 3：只新建自己的 crate，接入由下一批统一做）。
//!
//! 照 align 的草案写（`../noise_canceling-align/src/types/src/{frame,scene,params,policy}.rs`，2026-10-31）：
//! 只抄本 crate 用得到的字段与 `HysteresisGate` 的判定逻辑（要在模拟数据上标定它的三个数）。
//! **定稿后整个文件删掉，改 `use vdn_types::…`**。`HysteresisGate::step` 与草案逐行一致（标定结果才能直接搬过去）。

/// 流时间（样本数）。镜像里用模型采样率计（离线 16 kHz）；接入后按 vdn-types 改成 IO 采样率。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SampleClock(pub u64);

impl SampleClock {
    pub fn since(self, earlier: SampleClock) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

/// 置信度，夹到 0..=1，NaN 当 0。
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Confidence(f32);

impl Confidence {
    pub fn new(v: f32) -> Self {
        if v.is_nan() { Self(0.0) } else { Self(v.clamp(0.0, 1.0)) }
    }
    pub fn get(self) -> f32 {
        self.0
    }
}

/// 用户层基本场景（D41）。类目最终由人定（见 exp006 README「待人拍板」）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum UserScene {
    #[default]
    Quiet,
    Cafe,
    Commute,
    Music,
    /// 多人会议（占位，align 已加进草案）：保留近处所有人声，只压远处噪音；分类器暂不自动判，只能手动选
    Meeting,
}

impl UserScene {
    pub const ALL: [UserScene; 5] = [UserScene::Quiet, UserScene::Cafe, UserScene::Commute, UserScene::Music, UserScene::Meeting];
    pub fn as_str(self) -> &'static str {
        match self {
            UserScene::Quiet => "quiet",
            UserScene::Cafe => "cafe",
            UserScene::Commute => "commute",
            UserScene::Music => "music",
            UserScene::Meeting => "meeting",
        }
    }
}

/// 内部细场景编号；0 = 未知。含义见 [`crate::route`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FineSceneId(pub u16);

impl FineSceneId {
    pub const UNKNOWN: FineSceneId = FineSceneId(0);
}

/// 感知一次的输出。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLabel {
    pub user: UserScene,
    pub fine: FineSceneId,
    pub confidence: Confidence,
    pub at: SampleClock,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModelId {
    #[default]
    Gtcrn,
    FastEnhancerM,
}

impl ModelId {
    pub fn as_str(self) -> &'static str {
        match self {
            ModelId::Gtcrn => "gtcrn",
            ModelId::FastEnhancerM => "fastenhancer-m",
        }
    }
}

/// 修音链预设编号（0 = 不加工）；含义见 `vdn_polish::PolishParams::preset`。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ToneId(pub u8);

/// 一组参数（镜像 vdn-types `ParamSet`）。门控相关五个数沿用草案默认值，本线只决定 `model` 与 `tone`。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamSet {
    pub model: ModelId,
    pub tone: ToneId,
    pub strength: f32,
    pub floor_db: f32,
    pub open_db: f32,
    pub detect_db: f32,
    pub release_s: f32,
    pub hangover_s: f32,
}

impl ParamSet {
    pub const DEFAULT: ParamSet = ParamSet {
        model: ModelId::Gtcrn,
        tone: ToneId(0),
        strength: 1.0,
        floor_db: -12.0,
        open_db: -6.0,
        detect_db: 9.0,
        release_s: 0.3,
        hangover_s: 0.15,
    };
}

/// 滞回参数（镜像）。默认值改成 exp006 标定的结果（草案占位是 2 s / 0.6 / 5 s）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hysteresis {
    pub confirm_s: f32,
    pub min_confidence: f32,
    pub min_dwell_s: f32,
}

impl Default for Hysteresis {
    fn default() -> Self {
        crate::weights::HYSTERESIS
    }
}

/// 滞回状态机（镜像草案 `policy.rs` 的判定逻辑，行为须一致：低置信不推进也不清零候选；回到当前场景清掉候选）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HysteresisGate {
    pub params: Hysteresis,
    committed: FineSceneId,
    committed_user: UserScene,
    committed_at: Option<SampleClock>,
    candidate: Option<(FineSceneId, UserScene, SampleClock)>,
}

impl HysteresisGate {
    pub fn new(params: Hysteresis) -> Self {
        Self { params, committed: FineSceneId::UNKNOWN, committed_user: UserScene::Quiet, committed_at: None, candidate: None }
    }

    pub fn committed(&self) -> (UserScene, FineSceneId) {
        (self.committed_user, self.committed)
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.params);
    }

    /// 喂一个感知结果；换场景时返回 `(旧, 新)`。`rate` = `SampleClock` 的采样率。
    /// 与 align 草案（2026-10-31 自审后版）逐行一致：按 `(user, fine)` 比较；一次低置信就打断、候选从头算。
    pub fn step(&mut self, label: &SceneLabel, rate: u32) -> Option<(FineSceneId, FineSceneId)> {
        let secs = |n: u64| n as f64 / f64::from(rate.max(1));
        if label.confidence.get() < self.params.min_confidence {
            self.candidate = None;
            return None;
        }
        if (label.user, label.fine) == (self.committed_user, self.committed) {
            self.candidate = None;
            return None;
        }
        let since = match self.candidate {
            Some((id, u, t)) if (id, u) == (label.fine, label.user) => t,
            _ => {
                self.candidate = Some((label.fine, label.user, label.at));
                label.at
            }
        };
        let confirmed = secs(label.at.since(since)) >= f64::from(self.params.confirm_s);
        let dwelt = match self.committed_at {
            None => true,
            Some(t) => secs(label.at.since(t)) >= f64::from(self.params.min_dwell_s),
        };
        if confirmed && dwelt {
            let from = self.committed;
            self.committed = label.fine;
            self.committed_user = label.user;
            self.committed_at = Some(label.at);
            self.candidate = None;
            Some((from, label.fine))
        } else {
            None
        }
    }
}
