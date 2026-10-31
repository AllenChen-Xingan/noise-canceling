//! 记忆：「我的配置」（005 第三节 B「我的声音配置」、D40「场景内自适应」）。
//!
//! 用户在某个场景拧过的旋钮记在这里，下次进同样场景自动套用。按场景存，**只存参数，不存音频**（D39：
//! 不上传任何东西；D19：会话记录不含音频）——类型里就没有放音频的位置。
//!
//! 叠加顺序（[`UserProfile::apply`]）：表内参数 → 用户层场景的覆盖 → 细场景的覆盖。
//! 学习部件（[`Learner`]）只在空闲时产出**候选**新配置，过门禁才换（D38），这里只定接口、不实现。

use core::fmt::Write as _;

use crate::params::{ModelId, ParamField, ParamSet, ToneId};
use crate::record::SessionRecord;
use crate::scene::{FineSceneId, UserScene};
use crate::text::{self, Line, TextError};
use crate::version::{ConfigBinding, ContentId, ProfileVersion};

/// 覆盖：`None` = 不改表里的值。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Overrides {
    pub model: Option<ModelId>,
    pub tone: Option<ToneId>,
    /// 下标 = `ParamField::ALL` 里的位置
    values: [Option<f32>; ParamField::ALL.len()],
}

fn field_index(f: ParamField) -> usize {
    ParamField::ALL.iter().position(|x| *x == f).unwrap_or(0)
}

impl Overrides {
    pub fn get(&self, f: ParamField) -> Option<f32> {
        self.values.get(field_index(f)).copied().flatten()
    }

    pub fn set(&mut self, f: ParamField, v: Option<f32>) {
        if let Some(slot) = self.values.get_mut(field_index(f)) {
            *slot = v;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.model.is_none() && self.tone.is_none() && self.values.iter().all(Option::is_none)
    }

    /// 盖到 `p` 上。越界的值夹进范围（旋钮 UI 本该限住，这里再兜一层）。
    pub fn apply_to(&self, p: &mut ParamSet) {
        if let Some(m) = self.model {
            p.model = m;
        }
        if let Some(t) = self.tone {
            p.tone = t;
        }
        for f in ParamField::ALL {
            if let Some(v) = self.get(f).filter(|v| v.is_finite()) {
                let (lo, hi) = f.range();
                p.set(f, v.clamp(lo, hi));
            }
        }
    }
}

/// 覆盖挂在哪一层。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SceneKey {
    User(UserScene),
    Fine(FineSceneId),
}

/// 「我的配置」。只在本机，可一键清空，可按版本回滚（D38）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserProfile {
    pub version: ProfileVersion,
    /// 绑定（模型 · 设备 · 系统大版本）；加载时与当前环境不符就回出厂（`None` = 未绑定，按出厂处理由调用方定）
    pub binding: Option<ConfigBinding>,
    entries: Vec<(SceneKey, Overrides)>,
}

impl UserProfile {
    pub const FORMAT: u32 = 1;
    const HEADER: &'static str = "vdn-profile";

    pub fn get(&self, k: SceneKey) -> Option<&Overrides> {
        self.entries
            .iter()
            .find(|(key, _)| *key == k)
            .map(|(_, o)| o)
    }

    /// 改一个场景的覆盖；空覆盖 = 删掉这一条。不自动加版本号（由调用方决定什么时候成一版）。
    pub fn put(&mut self, k: SceneKey, o: Overrides) {
        self.entries.retain(|(key, _)| *key != k);
        if !o.is_empty() {
            self.entries.push((k, o));
            self.entries.sort_by_key(|(key, _)| *key);
        }
    }

    /// 用户拧了一个旋钮：记进该场景的覆盖。
    pub fn remember_knob(&mut self, k: SceneKey, f: ParamField, v: f32) {
        let mut o = self.get(k).copied().unwrap_or_default();
        o.set(f, Some(v));
        self.put(k, o);
    }

    pub fn entries(&self) -> &[(SceneKey, Overrides)] {
        &self.entries
    }

    /// 绑定是否与当前环境一致；不一致（或未绑定）时调用方应回出厂（ondevice #8）。
    pub fn binding_matches(&self, now: &ConfigBinding) -> bool {
        self.binding.as_ref() == Some(now)
    }

    /// 表内参数 → 用户层覆盖 → 细场景覆盖。
    pub fn apply(&self, user: UserScene, fine: FineSceneId, p: &mut ParamSet) {
        if let Some(o) = self.get(SceneKey::User(user)) {
            o.apply_to(p);
        }
        if fine != FineSceneId::UNKNOWN
            && let Some(o) = self.get(SceneKey::Fine(fine))
        {
            o.apply_to(p);
        }
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "{} = {}", Self::HEADER, Self::FORMAT);
        let _ = writeln!(s, "version = {}", self.version.0);
        if let Some(b) = &self.binding {
            let _ = writeln!(s, "binding = {}", b.to_value());
        }
        for (k, o) in &self.entries {
            match k {
                SceneKey::User(u) => {
                    let _ = writeln!(s, "[user {u}]");
                }
                SceneKey::Fine(id) => {
                    let _ = writeln!(s, "[fine {}]", id.0);
                }
            }
            if let Some(m) = o.model {
                let _ = writeln!(s, "model = {}", m.as_str());
            }
            if let Some(t) = o.tone {
                let _ = writeln!(s, "tone = {}", t.0);
            }
            for f in ParamField::ALL {
                if let Some(v) = o.get(f) {
                    let _ = writeln!(s, "{} = {}", f.as_str(), text::f32_str(v));
                }
            }
        }
        s
    }

    pub fn content_id(&self) -> ContentId {
        ContentId::of(self.to_text().as_bytes())
    }

    pub fn from_text(src: &str) -> Result<Self, TextError> {
        let mut prof = UserProfile::default();
        let mut seen_header = false;
        let mut cur: Option<(SceneKey, Overrides)> = None;
        text::parse(src, |line, l| {
            match l {
                Line::Kv { key, value } if !seen_header => {
                    let f: u32 = text::num(line, value)?;
                    if key != Self::HEADER || f == 0 || f > Self::FORMAT {
                        return Err(text::err(line, "不是我的配置文件，或格式号比本程序新"));
                    }
                    seen_header = true;
                }
                Line::Section { .. } if !seen_header => return Err(text::err(line, "缺文件头")),
                Line::Section { kind, name } => {
                    if let Some((k, o)) = cur.take() {
                        prof.put(k, o);
                    }
                    let key = match kind {
                        "user" => SceneKey::User(
                            UserScene::parse(name).ok_or(text::err(line, "未知的用户层场景"))?,
                        ),
                        "fine" => SceneKey::Fine(FineSceneId(text::num(line, name)?)),
                        _ => return Err(text::err(line, "未知的小节")),
                    };
                    if prof.get(key).is_some() {
                        return Err(text::err(line, "同一场景的小节重复"));
                    }
                    cur = Some((key, Overrides::default()));
                }
                Line::Kv { key, value } => match cur.as_mut() {
                    None if key == "version" => {
                        prof.version = ProfileVersion(text::num(line, value)?)
                    }
                    None if key == "binding" => {
                        prof.binding = Some(
                            ConfigBinding::parse(value).ok_or(text::err(line, "绑定格式不对"))?,
                        )
                    }
                    None => return Err(text::err(line, "未知的键")),
                    Some((_, o)) => match key {
                        "model" => {
                            o.model =
                                Some(ModelId::parse(value).ok_or(text::err(line, "未知的模型"))?)
                        }
                        "tone" => o.tone = Some(ToneId(text::num(line, value)?)),
                        _ => {
                            let f = ParamField::parse(key).ok_or(text::err(line, "未知的参数"))?;
                            let v: f32 = text::num(line, value)?;
                            if !v.is_finite() {
                                return Err(text::err(line, "数值必须有限"));
                            }
                            o.set(f, Some(v));
                        }
                    },
                },
            }
            Ok(())
        })?;
        if !seen_header {
            return Err(text::err(0, "空文件"));
        }
        if let Some((k, o)) = cur {
            prof.put(k, o);
        }
        Ok(prof)
    }
}

/// 学习部件（005 第三节 B 的「归纳 → 提议」）。**本批不实现**。
///
/// 约定：只在空闲时调（插电 · 未开会 · 不发热 · 省电核低优先级，D38）；输入只有会话记录与当前配置，
/// 没有音频；返回的是**候选**，调用方先过回归门禁 / 影子比对（004 · 007），更好才换，版本号 +1，可回滚。
/// 在线只调参数、不改权重（意向书现有措辞，005 第五节待人定 #2）。
pub trait Learner {
    fn propose(&mut self, history: &[SessionRecord], current: &UserProfile) -> Option<UserProfile>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layering_order_table_then_user_then_fine() {
        let mut prof = UserProfile::default();
        prof.remember_knob(SceneKey::User(UserScene::Cafe), ParamField::FloorDb, -20.0);
        prof.remember_knob(SceneKey::User(UserScene::Cafe), ParamField::Strength, 0.7);
        prof.remember_knob(SceneKey::Fine(FineSceneId(7)), ParamField::FloorDb, -25.0);
        let mut p = ParamSet::default();
        prof.apply(UserScene::Cafe, FineSceneId(7), &mut p);
        assert_eq!(p.floor_db, -25.0);
        assert_eq!(p.strength, 0.7);
        let mut q = ParamSet::default();
        prof.apply(UserScene::Cafe, FineSceneId(8), &mut q);
        assert_eq!(q.floor_db, -20.0);
    }

    #[test]
    fn overrides_clamp_and_ignore_nan() {
        let mut o = Overrides::default();
        o.set(ParamField::FloorDb, Some(10.0));
        o.set(ParamField::Strength, Some(f32::NAN));
        let mut p = ParamSet::default();
        o.apply_to(&mut p);
        assert_eq!(p.floor_db, 0.0);
        assert_eq!(p.strength, 1.0);
    }

    #[test]
    fn text_roundtrip() {
        let mut prof = UserProfile {
            version: ProfileVersion(4),
            binding: Some(ConfigBinding {
                model_hash: ContentId::of(b"m"),
                device: ContentId::of(b"d"),
                os_major: 27,
            }),
            ..Default::default()
        };
        prof.remember_knob(SceneKey::Fine(FineSceneId(7)), ParamField::DetectDb, 10.5);
        let mut o = Overrides {
            model: Some(ModelId::FastEnhancerM),
            ..Default::default()
        };
        o.set(ParamField::FloorDb, Some(-30.0));
        prof.put(SceneKey::User(UserScene::Music), o);
        let back = UserProfile::from_text(&prof.to_text()).unwrap();
        assert_eq!(back, prof);
        assert_eq!(back.content_id(), prof.content_id());
    }

    #[test]
    fn rejects_duplicates_and_non_finite() {
        let dup = "vdn-profile = 1\n[user cafe]\nstrength = 0.5\n[user cafe]\nstrength = 0.6\n";
        assert!(UserProfile::from_text(dup).is_err());
        let nan = "vdn-profile = 1\n[user cafe]\nstrength = NaN\n";
        assert!(UserProfile::from_text(nan).is_err());
    }

    #[test]
    fn empty_override_removes_entry() {
        let mut prof = UserProfile::default();
        prof.remember_knob(SceneKey::User(UserScene::Quiet), ParamField::Strength, 0.5);
        prof.put(SceneKey::User(UserScene::Quiet), Overrides::default());
        assert!(prof.entries().is_empty());
    }

    #[test]
    fn profile_text_has_only_parameter_keys() {
        // D39：只存参数。文本里每个键都必须是已知参数名（没有地方塞音频或别的东西）
        let mut prof = UserProfile::default();
        prof.remember_knob(SceneKey::User(UserScene::Cafe), ParamField::FloorDb, -20.0);
        for l in prof.to_text().lines().filter(|l| !l.starts_with('[')) {
            let k = l.split('=').next().unwrap().trim();
            assert!(
                ["vdn-profile", "version", "binding", "model", "tone"].contains(&k)
                    || ParamField::parse(k).is_some(),
                "{k}"
            );
        }
    }
}
