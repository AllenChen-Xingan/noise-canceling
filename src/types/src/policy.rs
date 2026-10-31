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

/// 滞回：边界上不来回跳（005：「连续确认数秒才换场景」）。数值是占位初值，待模拟环境标定（D41）。
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
            confirm_s: 2.0,
            min_confidence: 0.6,
            min_dwell_s: 5.0,
        }
    }
}

/// 平滑过渡（005：「参数在几百毫秒到数秒内平滑过渡；换模型交叉淡化」）。占位初值，待听测标定。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    /// 连续参数的一阶平滑时间常数（秒），engine 用 `vdn_dsp::one_pole_coef` 换成逐 hop 系数
    pub glide_s: f32,
    /// 换模型的交叉淡化时长（秒）
    pub model_crossfade_s: f32,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            glide_s: 1.0,
            model_crossfade_s: 0.05,
        }
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
    pub fn step(&mut self, label: &SceneLabel, rate: u32) -> Option<(FineSceneId, FineSceneId)> {
        let secs = |n: u64| n as f64 / f64::from(rate.max(1));
        if label.confidence.get() < self.params.min_confidence {
            // 不确定的判断不推进、也不清零候选：短暂的低置信不该让确认从头来
            return None;
        }
        if label.fine == self.committed {
            self.candidate = None;
            return None;
        }
        let since = match self.candidate {
            Some((id, _, t)) if id == label.fine => t,
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
    if !control.knobs.is_empty() {
        control.knobs.apply_to(&mut p);
        src = ParamSource::UserKnob;
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
}

/// 参考实现：查表 + 滞回 + 用户优先。不学习、不探索。
#[derive(Clone, Debug, Default)]
pub struct TablePolicy {
    pub gate: HysteresisGate,
    pub transition: Transition,
    last: Option<ParamSet>,
}

impl TablePolicy {
    pub fn new(h: Hysteresis, t: Transition) -> Self {
        Self {
            gate: HysteresisGate::new(h),
            transition: t,
            last: None,
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
    }

    fn decide(&mut self, i: &PolicyInput<'_>) -> Decision {
        let switched = self.gate.step(&i.label, i.rate);
        let (u, f) = self.gate.committed();
        let (user, scene, mut target, mut source) =
            resolve_params(i.table, i.profile, i.control, u, f);
        if i.control.lock
            && let Some(prev) = self.last
        {
            target = prev;
            source = ParamSource::Locked;
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
mod tests {
    use super::*;
    use crate::frame::Confidence;
    use crate::params::{ModelId, ParamField};
    use crate::scene::SceneName;
    use crate::table::SceneEntry;

    const SR: u32 = 48_000;

    fn label(fine: u16, user: UserScene, sec: f64, conf: f32) -> SceneLabel {
        SceneLabel {
            user,
            fine: FineSceneId(fine),
            confidence: Confidence::new(conf),
            at: SampleClock((sec * f64::from(SR)) as u64),
        }
    }

    fn table() -> SceneTable {
        let mut t = SceneTable::default();
        let p = ParamSet {
            floor_db: -20.0,
            model: ModelId::FastEnhancerM,
            ..ParamSet::default()
        };
        t.insert(SceneEntry {
            id: FineSceneId(7),
            name: SceneName::new("cafe+bt").unwrap(),
            user: UserScene::Cafe,
            params: p,
        })
        .unwrap();
        t
    }

    #[test]
    fn needs_continuous_confirmation() {
        let mut g = HysteresisGate::new(Hysteresis::default());
        assert_eq!(g.step(&label(7, UserScene::Cafe, 0.0, 0.9), SR), None);
        assert_eq!(g.step(&label(7, UserScene::Cafe, 1.0, 0.9), SR), None);
        assert_eq!(
            g.step(&label(7, UserScene::Cafe, 2.0, 0.9), SR),
            Some((FineSceneId(0), FineSceneId(7)))
        );
        assert_eq!(g.committed(), (UserScene::Cafe, FineSceneId(7)));
    }

    #[test]
    fn flapping_never_switches() {
        let mut g = HysteresisGate::new(Hysteresis::default());
        for k in 0..40 {
            let fine = if k % 2 == 0 { 7 } else { 8 };
            assert_eq!(
                g.step(&label(fine, UserScene::Cafe, k as f64 * 0.5, 0.9), SR),
                None
            );
        }
    }

    #[test]
    fn low_confidence_does_not_count() {
        let mut g = HysteresisGate::new(Hysteresis::default());
        for k in 0..20 {
            assert_eq!(
                g.step(&label(7, UserScene::Cafe, k as f64 * 0.5, 0.3), SR),
                None
            );
        }
    }

    #[test]
    fn dwell_blocks_quick_second_switch() {
        let mut g = HysteresisGate::new(Hysteresis::default());
        g.step(&label(7, UserScene::Cafe, 0.0, 0.9), SR);
        assert!(g.step(&label(7, UserScene::Cafe, 2.0, 0.9), SR).is_some());
        // 2 s 后又稳定判成 8，确认够了但停留不够 5 s
        assert_eq!(g.step(&label(8, UserScene::Music, 2.5, 0.9), SR), None);
        assert_eq!(g.step(&label(8, UserScene::Music, 4.5, 0.9), SR), None);
        assert!(g.step(&label(8, UserScene::Music, 7.0, 0.9), SR).is_some());
    }

    #[test]
    fn layering_and_user_priority() {
        let t = table();
        let mut prof = UserProfile::default();
        prof.remember_knob(
            crate::memory::SceneKey::Fine(FineSceneId(7)),
            ParamField::Strength,
            0.8,
        );
        let ctl = UserControl::default();
        let (_, _, p, src) = resolve_params(&t, &prof, &ctl, UserScene::Cafe, FineSceneId(7));
        assert_eq!(
            (p.floor_db, p.strength, src),
            (-20.0, 0.8, ParamSource::Profile)
        );

        let mut ctl = UserControl::default();
        ctl.knobs.set(ParamField::FloorDb, Some(-6.0));
        let (_, _, p, src) = resolve_params(&t, &prof, &ctl, UserScene::Cafe, FineSceneId(7));
        assert_eq!((p.floor_db, src), (-6.0, ParamSource::UserKnob));

        // 手动选了「安静」：咖啡馆下的细场景不用，退回安静的基础参数
        let ctl = UserControl {
            scene: UserSceneChoice::Manual(UserScene::Quiet),
            ..Default::default()
        };
        let (u, f, p, _) = resolve_params(&t, &prof, &ctl, UserScene::Cafe, FineSceneId(7));
        assert_eq!(
            (u, f, p.model),
            (UserScene::Quiet, FineSceneId::UNKNOWN, ModelId::Gtcrn)
        );
    }

    #[test]
    fn table_policy_logs_switch_and_respects_lock() {
        let t = table();
        let prof = UserProfile::default();
        let mut pol = TablePolicy::new(Hysteresis::default(), Transition::default());
        let mut log = Vec::new();
        let mut ctl = UserControl::default();
        for k in 0..=4 {
            let l = label(7, UserScene::Cafe, k as f64 * 0.5, 0.9);
            let d = pol.decide(&PolicyInput {
                label: l,
                rate: SR,
                table: &t,
                profile: &prof,
                control: &ctl,
            });
            d.log_into(&l, &mut log);
        }
        // 第一次：默认参数（ParamsApplied）；2 s 时换到 7（SceneCommitted + ParamsApplied）
        assert_eq!(log.len(), 3);
        assert!(matches!(
            log[1],
            DecisionEvent::SceneCommitted {
                to: FineSceneId(7),
                ..
            }
        ));

        ctl.lock = true;
        for k in 5..30 {
            let l = label(1, UserScene::Quiet, k as f64 * 0.5, 0.9);
            let d = pol.decide(&PolicyInput {
                label: l,
                rate: SR,
                table: &t,
                profile: &prof,
                control: &ctl,
            });
            assert_eq!(d.target.model, ModelId::FastEnhancerM);
            assert_eq!(d.source, ParamSource::Locked);
        }
    }
}
