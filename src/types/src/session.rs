//! 一场会话的记录：头（当时用的是哪份表、哪份配置、哪张处理图、哪个模型）+ 决策事件（[`crate::record`]）。
//! **不含音频**（D19）、只在本机（D39）。
//!
//! 文本形态：
//! ```text
//! vdn-session = 2
//! rate = 48000
//! table = <版本> <内容 ID>
//! profile = <版本> <内容 ID>
//! graph = <处理图内容 ID>      ← 格式 2 起；可缺省
//! model = <模型 ID>             ← 格式 2 起；可缺省
//! [events]
//! e = …
//! ```
//!
//! 头部键的正本是 [`HEADER_KEYS`]（穷举解构 `SessionHeader`：加字段不加键，编译失败）；`to_text` / `from_text`
//! 用到的键与它逐个比对（测试），schema 导出（`specs/schema/session-header.fields.json`）也读它。
//!
//! 旧格式（graph 线定 2026-11-02）：**格式 1 照常能读**，读进来 `graph` / `model` 是 `None`（当时没记）；
//! 格式 1 的文件里出现这两个键算坏文件（那时还没有这两个键）。写出一律用最新格式。
//! 格式 2 里两个键仍可缺省：没接处理图的调用方（离线工具）不必编一个假 ID。
//! `rate` · `table` · `profile` 两个格式里都必填：缺了、或任一头部键写两次，算坏文件（codex 第 1 轮 #2）。

use core::fmt::Write as _;

use crate::graph::CheckedGraph;
use crate::params::ModelId;
use crate::record::DecisionEvent;
use crate::text::{self, Line, TextError};
use crate::version::{ContentId, ProfileVersion, TableVersion};

/// 会话头。不记设备名、不记时间以外的任何身份信息。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SessionHeader {
    /// IO 采样率（事件时刻的单位）
    pub rate: u32,
    pub table: (TableVersion, ContentId),
    pub profile: (ProfileVersion, ContentId),
    /// 处理图的内容 ID（`ProcessingGraph::content_id`）；`None` = 没记（格式 1 或调用方没接处理图）
    pub graph: Option<ContentId>,
    /// 模型 ID（与 `graph` 里的模型节点同一个；用 [`SessionHeader::set_graph`] 一起填）
    pub model: Option<ModelId>,
}

impl SessionHeader {
    /// 从核对过的处理图填 `graph` 与 `model`：两者同出一处，不会对不上；没核对的图进不来（#68）。构造期用（会分配）。
    pub fn set_graph(&mut self, g: &CheckedGraph) {
        self.graph = Some(g.content_id());
        self.model = Some(g.graph().model);
    }
}

/// 会话头的一个键（文本格式的一行）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderKey {
    /// `SessionHeader` 的字段名
    pub field: &'static str,
    /// 文本里的键
    pub key: &'static str,
    /// 值的写法
    pub value: &'static str,
    /// 从哪个格式号起有
    pub since: u32,
    /// 可缺省
    pub optional: bool,
    pub doc: &'static str,
}

/// `header_keys!(field => key, since, optional, "值写法", "说明"; …)`：穷举解构 `SessionHeader`（漏字段编译失败）。
macro_rules! header_keys {
    ($($f:ident => $key:literal, $since:literal, $opt:literal, $val:literal, $doc:literal);* $(;)?) => {{
        #[allow(dead_code)]
        fn exhaustive(h: &SessionHeader) {
            let SessionHeader { $($f: _),* } = h;
        }
        [$(HeaderKey { field: stringify!($f), key: $key, value: $val, since: $since, optional: $opt, doc: $doc }),*]
    }};
}

/// 头部键的正本，按文本里的顺序。
pub const HEADER_KEYS: [HeaderKey; 5] = header_keys!(
    rate => "rate", 1, false, "<u32>", "IO 采样率（Hz），事件时刻的单位";
    table => "table", 1, false, "<u32 版本> <16 位十六进制内容 ID>", "场景参数表";
    profile => "profile", 1, false, "<u32 版本> <16 位十六进制内容 ID>", "我的配置";
    graph => "graph", 2, true, "<16 位十六进制内容 ID>", "处理图（ProcessingGraph::content_id）";
    model => "model", 2, true, "<模型 ID>", "模型（ModelId::as_str）";
);

/// 一场会话的记录。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SessionRecord {
    pub header: SessionHeader,
    pub events: Vec<DecisionEvent>,
}

impl SessionRecord {
    /// 2：头部加 `graph` · `model`（2026-11-02）
    pub const FORMAT: u32 = 2;
    pub const HEADER: &'static str = "vdn-session";

    pub fn to_text(&self) -> String {
        let h = &self.header;
        let mut s = String::new();
        let _ = writeln!(s, "{} = {}", Self::HEADER, Self::FORMAT);
        let _ = writeln!(s, "rate = {}", h.rate);
        let _ = writeln!(s, "table = {} {}", h.table.0.0, h.table.1);
        let _ = writeln!(s, "profile = {} {}", h.profile.0.0, h.profile.1);
        if let Some(g) = h.graph {
            let _ = writeln!(s, "graph = {g}");
        }
        if let Some(m) = h.model {
            let _ = writeln!(s, "model = {}", m.as_str());
        }
        s.push_str("[events]\n");
        for e in &self.events {
            let _ = writeln!(s, "e = {}", e.to_line());
        }
        s
    }

    pub fn from_text(src: &str) -> Result<Self, TextError> {
        let mut r = SessionRecord::default();
        let mut format = 0;
        let mut in_events = false;
        // 每个头部键出现过没有（与 HEADER_KEYS 同序）：必填键缺了、同一个键写两次都算坏文件
        let mut seen = [false; HEADER_KEYS.len()];
        text::parse(src, |line, l| {
            match l {
                Line::Kv { key, value } if format == 0 => {
                    let f: u32 = text::num(line, value)?;
                    if key != Self::HEADER || f == 0 || f > Self::FORMAT {
                        return Err(text::err(line, "不是会话记录，或格式号比本程序新"));
                    }
                    format = f;
                }
                Line::Section { kind: "events", .. } if format > 0 => in_events = true,
                Line::Section { .. } => return Err(text::err(line, "未知的小节")),
                Line::Kv { key: "e", value } if in_events => {
                    r.events.push(DecisionEvent::from_line(line, value)?)
                }
                Line::Kv { key, value } if !in_events => {
                    let at = HEADER_KEYS.iter().position(|k| k.key == key);
                    let since = at.and_then(|i| HEADER_KEYS.get(i)).map(|k| k.since);
                    if since.is_none_or(|v| v > format) {
                        return Err(text::err(line, "未知的键（或这个格式号还没有这个键）"));
                    }
                    if let Some(s) = at.and_then(|i| seen.get_mut(i)) {
                        if *s {
                            return Err(text::err(line, "这个键写了两次"));
                        }
                        *s = true;
                    }
                    let mut it = value.split_whitespace();
                    let mut tok = || it.next().ok_or(text::err(line, "值不全"));
                    let id = |s: &str| ContentId::parse(s).ok_or(text::err(line, "内容 ID 不对"));
                    let h = &mut r.header;
                    match key {
                        "rate" => h.rate = text::num(line, tok()?)?,
                        "table" => {
                            h.table = (TableVersion(text::num(line, tok()?)?), id(tok()?)?);
                        }
                        "profile" => {
                            h.profile = (ProfileVersion(text::num(line, tok()?)?), id(tok()?)?);
                        }
                        "graph" => h.graph = Some(id(tok()?)?),
                        "model" => {
                            h.model =
                                Some(ModelId::parse(tok()?).ok_or(text::err(line, "未知的模型"))?)
                        }
                        _ => return Err(text::err(line, "未知的键")),
                    }
                    if it.next().is_some() {
                        return Err(text::err(line, "值后面多了东西"));
                    }
                }
                Line::Kv { .. } => return Err(text::err(line, "未知的键")),
            }
            Ok(())
        })?;
        if format == 0 {
            return Err(text::err(0, "空文件"));
        }
        // 必填键（该格式里有、且不可缺省）一个都不能少
        for (k, s) in HEADER_KEYS.iter().zip(seen) {
            if !s && !k.optional && k.since <= format {
                return Err(text::err(0, "缺少必填的头部键"));
            }
        }
        Ok(r)
    }
}

#[cfg(test)]
mod tests;
