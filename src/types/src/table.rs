//! 场景参数表（D40 · D41）：每个用户层场景一组基础参数，每个内部细场景一组参数。
//!
//! - **版本化**：[`TableVersion`] + 规范文本的 [`ContentId`]，决策日志里两个都记
//! - **可热更新、不在实时线程分配**：整张表是定长 `Copy` 结构（容量 [`SceneTable::CAP`]），
//!   在控制线程解析、校验好，按值交给策略；策略只把选中的 [`ParamSet`] 送进实时线程
//! - **能序列化**：[`SceneTable::to_text`] / [`SceneTable::from_text`]（格式见 [`crate::text`]），往返不丢信息
//!
//! 细场景小节先继承所属用户层场景的参数，再写自己改动的键；`to_text` 总是写全，保证内容 ID 稳定。

use core::fmt;
use core::fmt::Write as _;

use crate::params::{ModelId, ParamField, ParamSet, ToneId};
use crate::scene::{FineSceneId, SceneLabel, SceneName, UserScene};
use crate::text::{self, Line, TextError};
use crate::version::{ContentId, TableVersion};

/// 一个细场景。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneEntry {
    pub id: FineSceneId,
    pub name: SceneName,
    /// 归属的用户层场景（用户界面上显示这个）
    pub user: UserScene,
    pub params: ParamSet,
}

/// 表不合法。
#[derive(Clone, Debug, PartialEq)]
pub enum TableError {
    Text(TextError),
    /// 文件头不是 `vdn-scene-table = <格式号>`，或格式号比本程序新
    Format,
    Full,
    DuplicateId(FineSceneId),
    /// 编号 0 保留给「未知」
    ReservedId,
    Param(crate::params::ParamError),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::Text(e) => write!(f, "{e}"),
            TableError::Format => write!(f, "不是场景参数表，或格式号比本程序新"),
            TableError::Full => write!(f, "细场景超过 {} 个", SceneTable::CAP),
            TableError::DuplicateId(id) => write!(f, "细场景编号 {} 重复", id.0),
            TableError::ReservedId => write!(f, "细场景编号 0 保留给「未知」"),
            TableError::Param(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TableError {}

impl From<TextError> for TableError {
    fn from(e: TextError) -> Self {
        TableError::Text(e)
    }
}

const EMPTY: SceneEntry = SceneEntry {
    id: FineSceneId::UNKNOWN,
    name: match SceneName::new_const("-") {
        Some(n) => n,
        None => unreachable_name(),
    },
    user: UserScene::Quiet,
    params: ParamSet::DEFAULT,
};

const fn unreachable_name() -> SceneName {
    // 只在常量求值期走到；"-" 合法，不会走到这里
    SceneName::EMPTY
}

/// 场景参数表。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneTable {
    pub version: TableVersion,
    /// 用户层场景的基础参数，下标 = `UserScene::ALL` 里的位置
    user: [ParamSet; UserScene::ALL.len()],
    entries: [SceneEntry; SceneTable::CAP],
    len: usize,
}

impl Default for SceneTable {
    fn default() -> Self {
        Self {
            version: TableVersion(0),
            user: [ParamSet::DEFAULT; UserScene::ALL.len()],
            entries: [EMPTY; Self::CAP],
            len: 0,
        }
    }
}

fn user_index(u: UserScene) -> usize {
    UserScene::ALL.iter().position(|x| *x == u).unwrap_or(0)
}

impl SceneTable {
    /// 细场景容量。够用就不涨：涨了整表变大，热更新按值拷贝的成本也涨。
    pub const CAP: usize = 64;
    /// 文本格式号。
    pub const FORMAT: u32 = 1;
    const HEADER: &'static str = "vdn-scene-table";

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn entries(&self) -> &[SceneEntry] {
        self.entries.get(..self.len).unwrap_or(&[])
    }

    pub fn user_params(&self, u: UserScene) -> &ParamSet {
        self.user.get(user_index(u)).unwrap_or(&ParamSet::DEFAULT)
    }

    pub fn set_user_params(&mut self, u: UserScene, p: ParamSet) -> Result<(), TableError> {
        p.validate().map_err(TableError::Param)?;
        if let Some(slot) = self.user.get_mut(user_index(u)) {
            *slot = p;
        }
        Ok(())
    }

    pub fn lookup(&self, id: FineSceneId) -> Option<&SceneEntry> {
        self.entries().iter().find(|e| e.id == id)
    }

    /// 加一个细场景（构造期，控制线程）。
    pub fn insert(&mut self, e: SceneEntry) -> Result<(), TableError> {
        if e.id == FineSceneId::UNKNOWN {
            return Err(TableError::ReservedId);
        }
        if self.lookup(e.id).is_some() {
            return Err(TableError::DuplicateId(e.id));
        }
        e.params.validate().map_err(TableError::Param)?;
        let slot = self.entries.get_mut(self.len).ok_or(TableError::Full)?;
        *slot = e;
        self.len += 1;
        Ok(())
    }

    /// 某个感知结果该用的表内参数：细场景查得到就用它，否则退回用户层场景的基础参数。
    pub fn params_for(&self, label: &SceneLabel) -> ParamSet {
        match self.lookup(label.fine) {
            Some(e) => e.params,
            None => *self.user_params(label.user),
        }
    }

    /// 规范文本（写全每个字段，细场景按编号升序）。
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "{} = {}", Self::HEADER, Self::FORMAT);
        let _ = writeln!(s, "version = {}", self.version.0);
        for u in UserScene::ALL {
            let _ = writeln!(s, "[user {u}]");
            write_params(&mut s, self.user_params(u));
        }
        let mut sorted: Vec<&SceneEntry> = self.entries().iter().collect();
        sorted.sort_by_key(|e| e.id);
        for e in sorted {
            let _ = writeln!(s, "[fine {}]", e.id.0);
            let _ = writeln!(s, "name = {}", e.name.as_str());
            let _ = writeln!(s, "user = {}", e.user);
            write_params(&mut s, &e.params);
        }
        s
    }

    pub fn content_id(&self) -> ContentId {
        ContentId::of(self.to_text().as_bytes())
    }

    pub fn from_text(src: &str) -> Result<Self, TableError> {
        let mut t = SceneTable::default();
        let mut seen_header = false;
        let mut sec = Section::Top;
        let mut pending: Vec<(usize, &str, &str)> = Vec::new();
        let mut fail: Option<TableError> = None;

        let res = text::parse(src, |line, l| {
            match l {
                Line::Kv { key, value } if !seen_header => {
                    if key != Self::HEADER {
                        return Err(text::err(line, "第一行必须是 vdn-scene-table = 格式号"));
                    }
                    let f: u32 = text::num(line, value)?;
                    if f == 0 || f > Self::FORMAT {
                        return Err(text::err(line, "格式号比本程序新"));
                    }
                    seen_header = true;
                }
                Line::Section { .. } if !seen_header => {
                    return Err(text::err(line, "缺文件头"));
                }
                Line::Section { kind, name } => {
                    if let Err(e) = t.close(&sec, &mut pending) {
                        fail = Some(e);
                        return Err(text::err(line, "上一小节不合法"));
                    }
                    sec = match kind {
                        "user" => Section::User(
                            UserScene::parse(name).ok_or(text::err(line, "未知的用户层场景"))?,
                        ),
                        "fine" => {
                            let id: u16 = text::num(line, name)?;
                            Section::Fine(FineSceneId(id))
                        }
                        _ => return Err(text::err(line, "未知的小节")),
                    };
                }
                Line::Kv { key, value } => match sec {
                    Section::Top => match key {
                        "version" => t.version = TableVersion(text::num(line, value)?),
                        _ => return Err(text::err(line, "未知的键")),
                    },
                    _ => pending.push((line, key, value)),
                },
            }
            Ok(())
        });
        if let Some(e) = fail {
            return Err(e);
        }
        res?;
        if !seen_header {
            return Err(TableError::Format);
        }
        t.close(&sec, &mut pending)?;
        Ok(t)
    }

    fn close(&mut self, sec: &Section, pending: &mut Vec<(usize, &str, &str)>) -> Result<(), TableError> {
        let kvs = core::mem::take(pending);
        match *sec {
            Section::Top => Ok(()),
            Section::User(u) => {
                let mut p = *self.user_params(u);
                for (line, k, v) in kvs {
                    apply_param(&mut p, line, k, v)?;
                }
                self.set_user_params(u, p)
            }
            Section::Fine(id) => {
                let mut name = None;
                let mut user = None;
                for (line, k, v) in &kvs {
                    match *k {
                        "name" => {
                            name = Some(SceneName::new(v).ok_or(text::err(*line, "细场景名不合法"))?)
                        }
                        "user" => {
                            user = Some(UserScene::parse(v).ok_or(text::err(*line, "未知的用户层场景"))?)
                        }
                        _ => {}
                    }
                }
                let user = user.ok_or(TableError::Text(text::err(0, "细场景缺 user")))?;
                let name = name.ok_or(TableError::Text(text::err(0, "细场景缺 name")))?;
                let mut p = *self.user_params(user);
                for (line, k, v) in kvs {
                    if k != "name" && k != "user" {
                        apply_param(&mut p, line, k, v)?;
                    }
                }
                self.insert(SceneEntry {
                    id,
                    name,
                    user,
                    params: p,
                })
            }
        }
    }
}

enum Section {
    Top,
    User(UserScene),
    Fine(FineSceneId),
}

pub(crate) fn write_params(s: &mut String, p: &ParamSet) {
    let _ = writeln!(s, "model = {}", p.model.as_str());
    let _ = writeln!(s, "tone = {}", p.tone.0);
    for f in ParamField::ALL {
        let _ = writeln!(s, "{} = {}", f.as_str(), text::f32_str(p.get(f)));
    }
}

pub(crate) fn apply_param(p: &mut ParamSet, line: usize, k: &str, v: &str) -> Result<(), TextError> {
    match k {
        "model" => p.model = ModelId::parse(v).ok_or(text::err(line, "未知的模型"))?,
        "tone" => p.tone = ToneId(text::num(line, v)?),
        _ => {
            let f = ParamField::parse(k).ok_or(text::err(line, "未知的参数"))?;
            p.set(f, text::num(line, v)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Confidence, SampleClock};

    fn sample() -> SceneTable {
        let mut t = SceneTable {
            version: TableVersion(3),
            ..Default::default()
        };
        let mut cafe = ParamSet::default();
        cafe.floor_db = -18.0;
        t.set_user_params(UserScene::Cafe, cafe).unwrap();
        let mut p = cafe;
        p.model = ModelId::FastEnhancerM;
        p.detect_db = 12.5;
        t.insert(SceneEntry {
            id: FineSceneId(7),
            name: SceneName::new("cafe+neighbor+bt").unwrap(),
            user: UserScene::Cafe,
            params: p,
        })
        .unwrap();
        t
    }

    #[test]
    fn text_roundtrip_is_lossless_and_id_stable() {
        let t = sample();
        let txt = t.to_text();
        let back = SceneTable::from_text(&txt).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.content_id(), t.content_id());
    }

    #[test]
    fn fine_section_inherits_user_params() {
        let src = "vdn-scene-table = 1\nversion = 1\n[user cafe]\nfloor_db = -20\n[fine 3]\nname = c3\nuser = cafe\ndetect_db = 11\n";
        let t = SceneTable::from_text(src).unwrap();
        let e = t.lookup(FineSceneId(3)).unwrap();
        assert_eq!(e.params.floor_db, -20.0);
        assert_eq!(e.params.detect_db, 11.0);
    }

    #[test]
    fn rejects_bad_tables() {
        assert!(SceneTable::from_text("version = 1\n").is_err());
        assert!(SceneTable::from_text("vdn-scene-table = 99\n").is_err());
        let dup = "vdn-scene-table = 1\n[fine 3]\nname = a\nuser = quiet\n[fine 3]\nname = b\nuser = quiet\n";
        assert_eq!(
            SceneTable::from_text(dup).unwrap_err(),
            TableError::DuplicateId(FineSceneId(3))
        );
        let zero = "vdn-scene-table = 1\n[fine 0]\nname = a\nuser = quiet\n";
        assert_eq!(SceneTable::from_text(zero).unwrap_err(), TableError::ReservedId);
        let oob = "vdn-scene-table = 1\n[user quiet]\nfloor_db = 5\n";
        assert!(matches!(SceneTable::from_text(oob), Err(TableError::Param(_))));
    }

    #[test]
    fn unknown_fine_scene_falls_back_to_user_layer() {
        let t = sample();
        let label = SceneLabel {
            user: UserScene::Cafe,
            fine: FineSceneId(99),
            confidence: Confidence::ONE,
            at: SampleClock(0),
        };
        assert_eq!(t.params_for(&label).floor_db, -18.0);
        assert_eq!(t.params_for(&label).model, ModelId::Gtcrn);
    }

    #[test]
    fn capacity_is_enforced() {
        let mut t = SceneTable::default();
        for i in 1..=SceneTable::CAP as u16 {
            t.insert(SceneEntry {
                id: FineSceneId(i),
                name: SceneName::new("x").unwrap(),
                user: UserScene::Quiet,
                params: ParamSet::default(),
            })
            .unwrap();
        }
        let more = SceneEntry {
            id: FineSceneId(1000),
            ..*t.entries().first().unwrap()
        };
        assert_eq!(t.insert(more).unwrap_err(), TableError::Full);
    }

    #[test]
    fn table_is_copy_and_small_enough_to_swap_by_value() {
        fn is_copy<T: Copy>() {}
        is_copy::<SceneTable>();
        // 热更新按值拷贝：守住体积，超了先想清楚
        assert!(core::mem::size_of::<SceneTable>() <= 8 * 1024);
    }
}
