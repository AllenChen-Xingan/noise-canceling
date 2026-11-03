//! 决策日志：会话记录的最小形态（005 第三节 A「会话记录」；D19 / D39：**不含音频**、只在本机）。
//!
//! 每个决定写一条 [`DecisionEvent`]，事后能回答「当时为什么这么做」；学习部件只读这个。
//! 事件是 `Copy` 小结构：判断回路（C 档）直接 push 进 `Vec`；实时线程的事件（送原声）
//! 以后经 rtrb 送出来再记，实时线程本身不写日志（R3）。
//!
//! 文本形态：头部若干 `键 = 值`，然后 `[events]` 小节，一行一个 `e = <时刻> <种类> <参数…>`（空格分隔、按位置）。

use core::fmt::Write as _;

use crate::frame::{BypassReason, Confidence, SampleClock};
use crate::params::{ModelId, ParamField, ParamSet, ToneId};
use crate::policy::{ParamSource, UserSceneChoice};
use crate::scene::{FineSceneId, UserScene};
use crate::text::{self, Line, TextError};
use crate::version::{ContentId, ProfileVersion, TableVersion};

/// 人的反馈（进化回路的燃料，005 第三节 B 第 1 步）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackKind {
    /// 「这里吃字了」
    WordsEaten,
    /// 「这里有怪声」
    Artifact,
    /// 「噪音还是太大」
    TooNoisy,
    /// 「这样好」
    Good,
    /// AB 对比里选了 A / B
    PreferA,
    PreferB,
}

/// 声纹注册状态变化（`vdn-speaker` 产出）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnrollEvent {
    /// 自动注册完成、锁定
    Locked,
    /// 重新锁定（声纹更新）
    Relocked,
    /// 清空重来
    Reset,
}

/// 人的一次操作。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserAction {
    /// 撤销上一次操作
    Undo,
    DenoiseOn,
    DenoiseOff,
    Knob {
        field: ParamField,
        value: f32,
    },
    ChooseScene(UserSceneChoice),
    Lock,
    Unlock,
    ManualModel(ModelId),
    Feedback(FeedbackKind),
    /// 换了输入设备（不记设备名：可能含个人信息）；`rate` = 新的 IO 采样率，之后的事件时刻按它换算
    DeviceChanged {
        rate: u32,
    },
}

/// 一条决策事件。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecisionEvent {
    /// 感知的原始输出（每次判断一条，未经滞回）：细场景分类的回放与回归靠它（005 第三节 A）
    SceneObserved {
        at: SampleClock,
        user: UserScene,
        fine: FineSceneId,
        confidence: Confidence,
    },
    /// 策略确认换场景（过了滞回）
    SceneCommitted {
        at: SampleClock,
        from: FineSceneId,
        to: FineSceneId,
        user: UserScene,
        confidence: Confidence,
    },
    /// 策略下发了一组新目标参数
    ParamsApplied {
        at: SampleClock,
        scene: FineSceneId,
        source: ParamSource,
        params: ParamSet,
    },
    /// 人的操作 = 反馈事件总线（ondevice #7）：带当时的细场景与配置内容 ID，学习部件据此归因
    UserAction {
        at: SampleClock,
        scene: FineSceneId,
        config: ContentId,
        action: UserAction,
    },
    /// 模型级送原声（engine 计数，按段汇总后记一条）
    Bypass {
        at: SampleClock,
        reason: BypassReason,
        hops: u32,
    },
    Enroll {
        at: SampleClock,
        kind: EnrollEvent,
    },
    TableChanged {
        at: SampleClock,
        version: TableVersion,
        id: ContentId,
    },
    ProfileChanged {
        at: SampleClock,
        version: ProfileVersion,
        id: ContentId,
    },
}

impl DecisionEvent {
    pub fn at(&self) -> SampleClock {
        match *self {
            DecisionEvent::SceneObserved { at, .. }
            | DecisionEvent::SceneCommitted { at, .. }
            | DecisionEvent::ParamsApplied { at, .. }
            | DecisionEvent::UserAction { at, .. }
            | DecisionEvent::Bypass { at, .. }
            | DecisionEvent::Enroll { at, .. }
            | DecisionEvent::TableChanged { at, .. }
            | DecisionEvent::ProfileChanged { at, .. } => at,
        }
    }

    /// `<时刻> <种类> <参数…>`
    pub fn to_line(&self) -> String {
        let mut s = format!("{} ", self.at().0);
        let _ = match *self {
            DecisionEvent::SceneObserved {
                user,
                fine,
                confidence,
                ..
            } => write!(
                s,
                "observed {} {} {}",
                user,
                fine.0,
                text::f32_str(confidence.get())
            ),
            DecisionEvent::SceneCommitted {
                from,
                to,
                user,
                confidence,
                ..
            } => write!(
                s,
                "scene {} {} {} {}",
                from.0,
                to.0,
                user,
                text::f32_str(confidence.get())
            ),
            DecisionEvent::ParamsApplied {
                scene,
                source,
                params,
                ..
            } => {
                let _ = write!(
                    s,
                    "params {} {} {} {}",
                    scene.0,
                    source.as_str(),
                    params.model.as_str(),
                    params.tone.0
                );
                for f in ParamField::ALL {
                    let _ = write!(s, " {}", text::f32_str(params.get(f)));
                }
                Ok(())
            }
            DecisionEvent::UserAction {
                scene,
                config,
                action,
                ..
            } => {
                let _ = write!(s, "user {} {} ", scene.0, config);
                match action {
                    UserAction::Undo => write!(s, "undo"),
                    UserAction::DenoiseOn => write!(s, "on"),
                    UserAction::DenoiseOff => write!(s, "off"),
                    UserAction::Knob { field, value } => {
                        write!(s, "knob {} {}", field, text::f32_str(value))
                    }
                    UserAction::ChooseScene(UserSceneChoice::Auto) => write!(s, "scene auto"),
                    UserAction::ChooseScene(UserSceneChoice::Manual(u)) => write!(s, "scene {u}"),
                    UserAction::Lock => write!(s, "lock"),
                    UserAction::Unlock => write!(s, "unlock"),
                    UserAction::ManualModel(m) => write!(s, "model {}", m.as_str()),
                    UserAction::Feedback(k) => write!(s, "feedback {}", feedback_str(k)),
                    UserAction::DeviceChanged { rate } => write!(s, "device {rate}"),
                }
            }
            DecisionEvent::Bypass { reason, hops, .. } => {
                write!(s, "bypass {} {}", bypass_str(reason), hops)
            }
            DecisionEvent::Enroll { kind, .. } => write!(s, "enroll {}", enroll_str(kind)),
            DecisionEvent::TableChanged { version, id, .. } => {
                write!(s, "table {} {}", version.0, id)
            }
            DecisionEvent::ProfileChanged { version, id, .. } => {
                write!(s, "profile {} {}", version.0, id)
            }
        };
        s
    }

    pub fn from_line(line: usize, v: &str) -> Result<Self, TextError> {
        let bad = || text::err(line, "事件格式不对");
        let mut it = v.split_whitespace();
        let mut next = || it.next().ok_or_else(bad);
        let at = SampleClock(text::num(line, next()?)?);
        let ev = match next()? {
            "observed" => DecisionEvent::SceneObserved {
                at,
                user: UserScene::parse(next()?).ok_or_else(bad)?,
                fine: FineSceneId(text::num(line, next()?)?),
                confidence: Confidence::new(text::num(line, next()?)?),
            },
            "scene" => DecisionEvent::SceneCommitted {
                at,
                from: FineSceneId(text::num(line, next()?)?),
                to: FineSceneId(text::num(line, next()?)?),
                user: UserScene::parse(next()?).ok_or_else(bad)?,
                confidence: Confidence::new(text::num(line, next()?)?),
            },
            "params" => {
                let scene = FineSceneId(text::num(line, next()?)?);
                let source = ParamSource::parse(next()?).ok_or_else(bad)?;
                let mut params = ParamSet {
                    model: ModelId::parse(next()?).ok_or_else(bad)?,
                    tone: ToneId(text::num(line, next()?)?),
                    ..ParamSet::DEFAULT
                };
                for f in ParamField::ALL {
                    params.set(f, text::num(line, next()?)?);
                }
                DecisionEvent::ParamsApplied {
                    at,
                    scene,
                    source,
                    params,
                }
            }
            "user" => {
                let scene = FineSceneId(text::num(line, next()?)?);
                let config = ContentId::parse(next()?).ok_or_else(bad)?;
                let action = match next()? {
                    "undo" => UserAction::Undo,
                    "on" => UserAction::DenoiseOn,
                    "off" => UserAction::DenoiseOff,
                    "knob" => UserAction::Knob {
                        field: ParamField::parse(next()?).ok_or_else(bad)?,
                        value: text::num(line, next()?)?,
                    },
                    "scene" => match next()? {
                        "auto" => UserAction::ChooseScene(UserSceneChoice::Auto),
                        u => UserAction::ChooseScene(UserSceneChoice::Manual(
                            UserScene::parse(u).ok_or_else(bad)?,
                        )),
                    },
                    "lock" => UserAction::Lock,
                    "unlock" => UserAction::Unlock,
                    "model" => UserAction::ManualModel(ModelId::parse(next()?).ok_or_else(bad)?),
                    "feedback" => UserAction::Feedback(parse_feedback(next()?).ok_or_else(bad)?),
                    "device" => UserAction::DeviceChanged {
                        rate: text::num(line, next()?)?,
                    },
                    _ => return Err(bad()),
                };
                DecisionEvent::UserAction {
                    at,
                    scene,
                    config,
                    action,
                }
            }
            "bypass" => DecisionEvent::Bypass {
                at,
                reason: parse_bypass(next()?).ok_or_else(bad)?,
                hops: text::num(line, next()?)?,
            },
            "enroll" => DecisionEvent::Enroll {
                at,
                kind: parse_enroll(next()?).ok_or_else(bad)?,
            },
            "table" => DecisionEvent::TableChanged {
                at,
                version: TableVersion(text::num(line, next()?)?),
                id: ContentId::parse(next()?).ok_or_else(bad)?,
            },
            "profile" => DecisionEvent::ProfileChanged {
                at,
                version: ProfileVersion(text::num(line, next()?)?),
                id: ContentId::parse(next()?).ok_or_else(bad)?,
            },
            _ => return Err(bad()),
        };
        if it.next().is_some() {
            return Err(bad());
        }
        Ok(ev)
    }
}

fn feedback_str(k: FeedbackKind) -> &'static str {
    match k {
        FeedbackKind::WordsEaten => "words_eaten",
        FeedbackKind::Artifact => "artifact",
        FeedbackKind::TooNoisy => "too_noisy",
        FeedbackKind::Good => "good",
        FeedbackKind::PreferA => "prefer_a",
        FeedbackKind::PreferB => "prefer_b",
    }
}

fn parse_feedback(s: &str) -> Option<FeedbackKind> {
    [
        FeedbackKind::WordsEaten,
        FeedbackKind::Artifact,
        FeedbackKind::TooNoisy,
        FeedbackKind::Good,
        FeedbackKind::PreferA,
        FeedbackKind::PreferB,
    ]
    .into_iter()
    .find(|k| feedback_str(*k) == s)
}

fn enroll_str(k: EnrollEvent) -> &'static str {
    match k {
        EnrollEvent::Locked => "locked",
        EnrollEvent::Relocked => "relocked",
        EnrollEvent::Reset => "reset",
    }
}

fn parse_enroll(s: &str) -> Option<EnrollEvent> {
    [
        EnrollEvent::Locked,
        EnrollEvent::Relocked,
        EnrollEvent::Reset,
    ]
    .into_iter()
    .find(|k| enroll_str(*k) == s)
}

fn bypass_str(r: BypassReason) -> &'static str {
    match r {
        BypassReason::Backend => "backend",
        BypassReason::NonFinite => "nonfinite",
        BypassReason::Abnormal => "abnormal",
        BypassReason::Deadline => "deadline",
        BypassReason::BadLen => "badlen",
    }
}

fn parse_bypass(s: &str) -> Option<BypassReason> {
    [
        BypassReason::Backend,
        BypassReason::NonFinite,
        BypassReason::Abnormal,
        BypassReason::Deadline,
        BypassReason::BadLen,
    ]
    .into_iter()
    .find(|r| bypass_str(*r) == s)
}

/// 会话头：当时用的是哪份表、哪份配置、什么采样率。不记设备名、不记时间以外的任何身份信息。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SessionHeader {
    /// IO 采样率（事件时刻的单位）
    pub rate: u32,
    pub table: (TableVersion, ContentId),
    pub profile: (ProfileVersion, ContentId),
}

/// 一场会话的记录。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SessionRecord {
    pub header: SessionHeader,
    pub events: Vec<DecisionEvent>,
}

impl SessionRecord {
    pub const FORMAT: u32 = 1;
    const HEADER: &'static str = "vdn-session";

    pub fn to_text(&self) -> String {
        let h = &self.header;
        let mut s = String::new();
        let _ = writeln!(s, "{} = {}", Self::HEADER, Self::FORMAT);
        let _ = writeln!(s, "rate = {}", h.rate);
        let _ = writeln!(s, "table = {} {}", h.table.0.0, h.table.1);
        let _ = writeln!(s, "profile = {} {}", h.profile.0.0, h.profile.1);
        s.push_str("[events]\n");
        for e in &self.events {
            let _ = writeln!(s, "e = {}", e.to_line());
        }
        s
    }

    pub fn from_text(src: &str) -> Result<Self, TextError> {
        let mut r = SessionRecord::default();
        let mut seen_header = false;
        let mut in_events = false;
        text::parse(src, |line, l| {
            match l {
                Line::Kv { key, value } if !seen_header => {
                    let f: u32 = text::num(line, value)?;
                    if key != Self::HEADER || f == 0 || f > Self::FORMAT {
                        return Err(text::err(line, "不是会话记录，或格式号比本程序新"));
                    }
                    seen_header = true;
                }
                Line::Section { kind: "events", .. } if seen_header => in_events = true,
                Line::Section { .. } => return Err(text::err(line, "未知的小节")),
                Line::Kv { key: "e", value } if in_events => {
                    r.events.push(DecisionEvent::from_line(line, value)?)
                }
                Line::Kv { key, value } if !in_events => {
                    let mut it = value.split_whitespace();
                    let mut tok = || it.next().ok_or(text::err(line, "值不全"));
                    match key {
                        "rate" => r.header.rate = text::num(line, tok()?)?,
                        "table" => {
                            r.header.table = (
                                TableVersion(text::num(line, tok()?)?),
                                ContentId::parse(tok()?).ok_or(text::err(line, "内容 ID 不对"))?,
                            )
                        }
                        "profile" => {
                            r.header.profile = (
                                ProfileVersion(text::num(line, tok()?)?),
                                ContentId::parse(tok()?).ok_or(text::err(line, "内容 ID 不对"))?,
                            )
                        }
                        _ => return Err(text::err(line, "未知的键")),
                    }
                }
                Line::Kv { .. } => return Err(text::err(line, "未知的键")),
            }
            Ok(())
        })?;
        if !seen_header {
            return Err(text::err(0, "空文件"));
        }
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_kind() -> Vec<DecisionEvent> {
        let at = SampleClock(96_000);
        let p = ParamSet {
            floor_db: -18.5,
            model: ModelId::FastEnhancerM,
            ..ParamSet::default()
        };
        let mut v = vec![
            DecisionEvent::SceneObserved {
                at,
                user: UserScene::Cafe,
                fine: FineSceneId(7),
                confidence: Confidence::new(0.5),
            },
            DecisionEvent::SceneCommitted {
                at,
                from: FineSceneId(0),
                to: FineSceneId(7),
                user: UserScene::Cafe,
                confidence: Confidence::new(0.82),
            },
            DecisionEvent::ParamsApplied {
                at,
                scene: FineSceneId(7),
                source: ParamSource::Profile,
                params: p,
            },
            DecisionEvent::Bypass {
                at,
                reason: BypassReason::Deadline,
                hops: 3,
            },
            DecisionEvent::Enroll {
                at,
                kind: EnrollEvent::Relocked,
            },
            DecisionEvent::TableChanged {
                at,
                version: TableVersion(3),
                id: ContentId::of(b"t"),
            },
            DecisionEvent::ProfileChanged {
                at,
                version: ProfileVersion(2),
                id: ContentId::of(b"p"),
            },
        ];
        for action in [
            UserAction::DenoiseOn,
            UserAction::DenoiseOff,
            UserAction::Knob {
                field: ParamField::Strength,
                value: 0.6,
            },
            UserAction::ChooseScene(UserSceneChoice::Auto),
            UserAction::ChooseScene(UserSceneChoice::Manual(UserScene::Music)),
            UserAction::Lock,
            UserAction::Unlock,
            UserAction::ManualModel(ModelId::Gtcrn),
            UserAction::Feedback(FeedbackKind::WordsEaten),
            UserAction::DeviceChanged { rate: 44_100 },
            UserAction::Feedback(FeedbackKind::PreferB),
            UserAction::Undo,
        ] {
            v.push(DecisionEvent::UserAction {
                at,
                scene: FineSceneId(7),
                config: ContentId::of(b"cfg"),
                action,
            });
        }
        v
    }

    #[test]
    fn every_event_kind_roundtrips() {
        for e in every_kind() {
            let line = e.to_line();
            assert_eq!(DecisionEvent::from_line(1, &line).unwrap(), e, "{line}");
        }
    }

    #[test]
    fn session_roundtrips() {
        let r = SessionRecord {
            header: SessionHeader {
                rate: crate::IO_RATE,
                table: (TableVersion(3), ContentId::of(b"t")),
                profile: (ProfileVersion(1), ContentId::of(b"p")),
            },
            events: every_kind(),
        };
        assert_eq!(SessionRecord::from_text(&r.to_text()).unwrap(), r);
    }

    #[test]
    fn rejects_trailing_tokens_and_unknown_kinds() {
        assert!(DecisionEvent::from_line(1, "0 user on extra").is_err());
        assert!(DecisionEvent::from_line(1, "0 audio 1 2 3").is_err());
    }

    #[test]
    fn events_are_small_copy_values() {
        fn is_copy<T: Copy>() {}
        is_copy::<DecisionEvent>();
        assert!(core::mem::size_of::<DecisionEvent>() <= 64);
    }
}
