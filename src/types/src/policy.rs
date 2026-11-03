//! 策略：场景 → 参数组（D40「场景路由」）。跑在判断回路（约 0.5 s，C 档），只出参数、不碰声音、不加延迟。
//!
//! 数据流：感知给 [`SceneLabel`] → [`HysteresisGate`] 决定「换不换场景」→ [`resolve_params`] 叠出目标参数
//! （表 → 我的配置 → 当前旋钮）→ [`Decision`] 交给 engine，engine 按 [`Transition`] 平滑过去。
//!
//! 「策略可单独替换」（005 第〇节）：[`Policy`] 是 trait；[`TablePolicy`] 是查表 + 滞回的参考实现，
//! 以后的老虎机 / 学到的策略换掉它即可。用户优先：手动选场景、锁定、旋钮都盖过自动（005「用户优先」）。

use crate::frame::SampleClock;
use crate::memory::{Overrides, UserProfile};
use crate::params::ParamSet;
use crate::record::DecisionEvent;
use crate::scene::{FineSceneId, SceneLabel, UserScene};
use crate::table::SceneTable;

/// 用户层的场景选择（D41：可手动切）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UserSceneChoice {
    /// 自动判断
    #[default]
    Auto,
    /// 用户手动选了一个基本场景：细场景只在它下面挑
    Manual(UserScene),
}

/// 人此刻的控制面板状态。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UserControl {
    pub scene: UserSceneChoice,
    /// 锁定：参数冻结在锁定那一刻，自动不再改（D41：可锁定）
    pub lock: bool,
    /// 当前旋钮（还没记进「我的配置」的临时值）
    pub knobs: Overrides,
}

/// 目标参数从哪来（进决策日志）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamSource {
    /// 场景参数表
    Table,
    /// 表 + 我的配置
    Profile,
    /// 表 + 我的配置 + 当前旋钮
    UserKnob,
    /// 锁定中
    Locked,
}

impl ParamSource {
    pub fn as_str(self) -> &'static str {
        match self {
            ParamSource::Table => "table",
            ParamSource::Profile => "profile",
            ParamSource::UserKnob => "knob",
            ParamSource::Locked => "locked",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            ParamSource::Table,
            ParamSource::Profile,
            ParamSource::UserKnob,
            ParamSource::Locked,
        ]
        .into_iter()
        .find(|x| x.as_str() == s)
    }
}

/// 滞回：边界上不来回跳（005：「连续确认数秒才换场景」）。默认值由 scene 线在模拟数据上标定（exp006 held-out：
/// 已生效场景准确率用户层 93%、换场景后生效延迟中位 3.7 s / p90 6.4 s、错切 0.88 次/分钟；
/// `docs/experiments/exp006-scene-sim/builds/20261030214530-44df/{sweep,eval_final}.json`）。感知侧另有 0.5 s 后验 EMA（在 scene 内）。
/// 设备位（蓝牙 / 削波）不进滞回，另用施密特触发，生效后再拼进细场景。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hysteresis {
    /// 新场景要连续被判中这么久才换（秒）
    pub confirm_s: f32,
    /// 低于这个置信度的判断不计入确认
    pub min_confidence: f32,
    /// 换过一次后至少停留这么久（秒）
    pub min_dwell_s: f32,
}

impl Default for Hysteresis {
    fn default() -> Self {
        Self {
            confirm_s: crate::desc::HYST_CONFIRM_S.default,
            min_confidence: crate::desc::HYST_MIN_CONFIDENCE.default,
            min_dwell_s: crate::desc::HYST_MIN_DWELL_S.default,
        }
    }
}

impl Hysteresis {
    /// 每个字段都在描述表的范围内（加载配置、标定扫参时调；越界拒收）。
    pub fn validate(&self) -> Result<(), crate::desc::RangeError> {
        use crate::desc::*;
        // 穷举解构：加了字段而没登记描述表，编译失败
        let Hysteresis {
            confirm_s,
            min_confidence,
            min_dwell_s,
        } = *self;
        HYST_CONFIRM_S.check(confirm_s)?;
        HYST_MIN_CONFIDENCE.check(min_confidence)?;
        HYST_MIN_DWELL_S.check(min_dwell_s)?;
        Ok(())
    }

    /// 夹进描述表的范围（NaN 当默认值）。
    pub fn clamped(self) -> Self {
        use crate::desc::*;
        Self {
            confirm_s: HYST_CONFIRM_S.clamp(self.confirm_s),
            min_confidence: HYST_MIN_CONFIDENCE.clamp(self.min_confidence),
            min_dwell_s: HYST_MIN_DWELL_S.clamp(self.min_dwell_s),
        }
    }
}

/// 平滑过渡（005：「参数在几百毫秒到数秒内平滑过渡；换模型交叉淡化」）。占位初值，待听测标定。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    /// 连续参数的一阶平滑时间常数（秒），engine 用 `vdn_dsp::one_pole_coef` 换成逐 hop 系数
    pub glide_s: f32,
    /// 换模型的交叉淡化时长（秒）；等增益淡化（不用等功率，相关信号会鼓 +3 dB）
    pub model_crossfade_s: f32,
    /// 连续参数的 dB 类字段每秒最多变多少（限速；ondevice 需求 ②，参照 1 dB/s）
    pub max_db_per_s: f32,
    /// 用户手动改动的最短斜坡（秒，20–50 ms），防拉链声
    pub manual_ramp_s: f32,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            glide_s: crate::desc::TRANS_GLIDE_S.default,
            model_crossfade_s: crate::desc::TRANS_MODEL_CROSSFADE_S.default,
            max_db_per_s: crate::desc::TRANS_MAX_DB_PER_S.default,
            manual_ramp_s: crate::desc::TRANS_MANUAL_RAMP_S.default,
        }
    }
}

impl Transition {
    /// 每个字段都在描述表的范围内（加载配置时调；越界拒收）。
    pub fn validate(&self) -> Result<(), crate::desc::RangeError> {
        use crate::desc::*;
        // 穷举解构：加了字段而没登记描述表，编译失败
        let Transition {
            glide_s,
            model_crossfade_s,
            max_db_per_s,
            manual_ramp_s,
        } = *self;
        TRANS_GLIDE_S.check(glide_s)?;
        TRANS_MODEL_CROSSFADE_S.check(model_crossfade_s)?;
        TRANS_MAX_DB_PER_S.check(max_db_per_s)?;
        TRANS_MANUAL_RAMP_S.check(manual_ramp_s)?;
        Ok(())
    }
}

/// 滞回状态机。纯逻辑、不分配。
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
        Self {
            params,
            ..Default::default()
        }
    }

    pub fn committed(&self) -> (UserScene, FineSceneId) {
        (self.committed_user, self.committed)
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.params);
    }

    /// 喂一个感知结果；换场景时返回 `(旧, 新)`。`rate` = IO 采样率。
    /// 场景按 `(用户层, 细场景)` 这一对比较：分类器只出用户层（`fine` = 0）时也能切换（自审 P0）。
    pub fn step(&mut self, label: &SceneLabel, rate: u32) -> Option<(FineSceneId, FineSceneId)> {
        let secs = |n: u64| n as f64 / f64::from(rate.max(1));
        if label.confidence.get() < self.params.min_confidence {
            // 「连续确认」：一次不确定就打断，候选从头来（宁可晚切，不误切）
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

/// 叠出目标参数：场景参数表 → 我的配置 → 当前旋钮。手动选了用户层场景时，不属于它的细场景退回用户层。
pub fn resolve_params(
    table: &SceneTable,
    profile: &UserProfile,
    control: &UserControl,
    user: UserScene,
    fine: FineSceneId,
) -> (UserScene, FineSceneId, ParamSet, ParamSource) {
    let (user, fine, mut p, src) = resolve_base(table, profile, control, user, fine);
    if !control.knobs.is_empty() {
        control.knobs.apply_to(&mut p);
        return (user, fine, p, ParamSource::UserKnob);
    }
    (user, fine, p, src)
}

/// 表 → 我的配置（不含旋钮）。锁定时冻结的就是这一层。
fn resolve_base(
    table: &SceneTable,
    profile: &UserProfile,
    control: &UserControl,
    user: UserScene,
    fine: FineSceneId,
) -> (UserScene, FineSceneId, ParamSet, ParamSource) {
    // 细场景在表里时，用户层以表为准（标签里的 user 只在细场景未知时用）
    let user = table.lookup(fine).map_or(user, |e| e.user);
    let (user, fine) = match control.scene {
        UserSceneChoice::Auto => (user, fine),
        UserSceneChoice::Manual(u) => match table.lookup(fine) {
            Some(e) if e.user == u => (u, fine),
            _ => (u, FineSceneId::UNKNOWN),
        },
    };
    let mut p = match table.lookup(fine) {
        Some(e) => e.params,
        None => *table.user_params(user),
    };
    let mut src = ParamSource::Table;
    let before = p;
    profile.apply(user, fine, &mut p);
    if p != before {
        src = ParamSource::Profile;
    }
    (user, fine, p, src)
}

/// 策略的输入（借用，判断回路每次调用现拼）。
#[derive(Clone, Copy, Debug)]
pub struct PolicyInput<'a> {
    pub label: SceneLabel,
    /// IO 采样率
    pub rate: u32,
    pub table: &'a SceneTable,
    pub profile: &'a UserProfile,
    pub control: &'a UserControl,
}

/// 策略的输出。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decision {
    pub at: SampleClock,
    pub user: UserScene,
    pub scene: FineSceneId,
    pub target: ParamSet,
    pub source: ParamSource,
    pub transition: Transition,
    /// 这一次换了场景：`(旧, 新)`
    pub switched: Option<(FineSceneId, FineSceneId)>,
    /// 目标参数和上一次不同
    pub changed: bool,
}

impl Decision {
    /// 把值得记的部分写进决策日志（没变就什么也不写）。
    pub fn log_into(&self, label: &SceneLabel, out: &mut Vec<DecisionEvent>) {
        if let Some((from, to)) = self.switched {
            out.push(DecisionEvent::SceneCommitted {
                at: self.at,
                from,
                to,
                user: self.user,
                confidence: label.confidence,
            });
        }
        if self.changed {
            out.push(DecisionEvent::ParamsApplied {
                at: self.at,
                scene: self.scene,
                source: self.source,
                params: self.target,
            });
        }
    }
}

/// 策略接口。
pub trait Policy: Send {
    fn name(&self) -> &'static str;
    fn reset(&mut self);
    fn decide(&mut self, input: &PolicyInput<'_>) -> Decision;
    /// 反馈入口（005 第三节 B 第 3 步「按反馈加权」的老虎机从这里拿奖励）：用户操作、送原声等事件都喂进来。
    /// 查表策略不用，默认什么也不做
    fn observe(&mut self, _event: &DecisionEvent) {}
}

/// 参考实现：查表 + 滞回 + 用户优先。不学习、不探索。
#[derive(Clone, Debug, Default)]
pub struct TablePolicy {
    pub gate: HysteresisGate,
    pub transition: Transition,
    last: Option<ParamSet>,
    /// 锁定那一刻的「表 + 我的配置」
    locked: Option<(UserScene, FineSceneId, ParamSet)>,
}

impl TablePolicy {
    pub fn new(h: Hysteresis, t: Transition) -> Self {
        Self {
            gate: HysteresisGate::new(h),
            transition: t,
            last: None,
            locked: None,
        }
    }
}

impl Policy for TablePolicy {
    fn name(&self) -> &'static str {
        "table"
    }

    fn reset(&mut self) {
        self.gate.reset();
        self.last = None;
        self.locked = None;
    }

    fn decide(&mut self, i: &PolicyInput<'_>) -> Decision {
        let switched = self.gate.step(&i.label, i.rate);
        let (u, f) = self.gate.committed();
        // 锁定冻结「表 + 我的配置」；旋钮照常盖在上面（用户优先，自审 P1）
        let (user, scene, mut target, mut source) = if i.control.lock {
            let (lu, lf, lp) = *self.locked.get_or_insert_with(|| {
                let (u, f, p, _) = resolve_base(i.table, i.profile, i.control, u, f);
                (u, f, p)
            });
            (lu, lf, lp, ParamSource::Locked)
        } else {
            self.locked = None;
            resolve_base(i.table, i.profile, i.control, u, f)
        };
        if !i.control.knobs.is_empty() {
            i.control.knobs.apply_to(&mut target);
            source = ParamSource::UserKnob;
        }
        let changed = self.last != Some(target);
        self.last = Some(target);
        Decision {
            at: i.label.at,
            user,
            scene,
            target,
            source,
            transition: self.transition,
            switched,
            changed,
        }
    }
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
