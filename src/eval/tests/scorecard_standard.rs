//! 成绩单标准：人读的 `specs/rubric/scorecard.md` 与机读正本（`scorecard::standard` · `thresholds` 表）逐行比对。
//! - 第一节「场景键 → 场景类」表 = `standard::group_of`（每个开会场景一行，不多不少）
//! - 第三节「每项与过关线」表：项的集合 = 否决项 + 共同项 + 各类加看项；「用在」列与代码一致；
//!   「过关线」列的第一个数 = 代码实际用的过关线（成绩单标准给的参数，否则 thresholds 表的默认值）

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use vdn_eval::scene::MeetingPlace;
use vdn_eval::scorecard::standard::{COMMON, Group, VETO, group_of};

fn doc() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs/rubric/scorecard.md");
    std::fs::read_to_string(p).unwrap()
}

fn section<'a>(t: &'a str, from: &str, to: &str) -> &'a str {
    let a = t.find(from).unwrap_or_else(|| panic!("找不到 {from}"));
    let r = &t[a..];
    &r[..r.find(to).unwrap_or(r.len())]
}

/// 表格行的各格（去掉首尾竖线、修剪空白）。
fn cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

/// 文本里第一个数（带负号）。
fn first_number(s: &str) -> Option<f64> {
    let s = s.replace('−', "-");
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let neg = cs[i] == '-' && cs.get(i + 1).is_some_and(char::is_ascii_digit);
        let st = if neg { i + 1 } else { i };
        if cs.get(st).is_some_and(char::is_ascii_digit) {
            let mut j = st;
            while j < cs.len() && (cs[j].is_ascii_digit() || cs[j] == '.') {
                j += 1;
            }
            let v: f64 = cs[st..j].iter().collect::<String>().parse().ok()?;
            return Some(if neg { -v } else { v });
        }
        i += 1;
    }
    None
}

/// 文本里全部的数（带负号），按出现顺序。
fn all_numbers(s: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let mut rest = s.replace('−', "-");
    while let Some(x) = first_number(&rest) {
        out.push(x);
        // 跳过这个数本身
        let t = format!("{}", x.abs());
        let at = rest.find(&t).map_or(rest.len(), |i| i + t.len());
        rest = rest[at..].to_string();
    }
    out
}

/// 任务必须给的参数（成绩单标准里写的），按顺序；没有就是空。
fn explicit_params(v: &str) -> Vec<f64> {
    VETO.iter()
        .chain(&COMMON)
        .chain(Group::ALL.iter().flat_map(|g| g.extra()))
        .find(|i| i.v == v && !i.params.is_empty())
        .map(|i| i.params.iter().map(|(_, x)| *x).collect())
        .unwrap_or_default()
}

#[test]
fn scene_group_table_matches_code() {
    let t = doc();
    let sec = section(&t, "## 一、", "## 二、");
    let mut doc_map = BTreeMap::new();
    for l in sec.lines().filter(|l| l.starts_with("| `")) {
        let c = cells(l);
        doc_map.insert(
            c[0].trim_matches('`').to_string(),
            c[2].trim_matches('`').to_string(),
        );
    }
    let code: BTreeMap<String, String> = MeetingPlace::ALL
        .iter()
        .filter_map(|s| group_of(*s).map(|g| (s.as_str(), g.as_str().to_string())))
        .collect();
    assert_eq!(
        doc_map, code,
        "scorecard.md 第一节与 standard::group_of 不一致"
    );
}

/// 代码实际用的过关线。
fn effective(v: &str) -> Option<f64> {
    if v == "V-LAT" {
        return Some(vdn_eval::system::BUDGET_MS);
    }
    if v == "V-NAN" {
        return Some(0.0);
    }
    let all = VETO
        .iter()
        .chain(&COMMON)
        .chain(Group::ALL.iter().flat_map(|g| g.extra()));
    for i in all.filter(|i| i.v == v) {
        if let Some((_, x)) = i.params.iter().find(|(k, _)| *k == "max" || *k == "min") {
            return Some(*x);
        }
    }
    let l = vdn_eval::thresholds::ALL.iter().find(|l| l.id == v)?;
    ["max", "min"]
        .iter()
        .map(|k| l.def(k))
        .find(|x| x.is_finite())
}

#[test]
fn item_table_matches_code() {
    let t = doc();
    let sec = section(&t, "## 三、", "## 四、");
    let rows: Vec<Vec<String>> = sec
        .lines()
        .filter(|l| l.starts_with("| V-"))
        .map(cells)
        .collect();
    let doc_ids: BTreeSet<&str> = rows.iter().map(|r| r[0].as_str()).collect();
    let mut code_ids: BTreeSet<&str> = VETO.iter().chain(&COMMON).map(|i| i.v).collect();
    for g in Group::ALL {
        code_ids.extend(g.extra().iter().map(|i| i.v));
    }
    assert_eq!(doc_ids, code_ids, "第三节的项与代码不一致");
    for r in &rows {
        let (v, used, line) = (r[0].as_str(), r[2].as_str(), r[3].as_str());
        assert_eq!(
            used.contains("否决"),
            VETO.iter().any(|i| i.v == v),
            "{v} 的「否决」"
        );
        assert_eq!(
            used.contains("共同"),
            COMMON.iter().any(|i| i.v == v),
            "{v} 的「共同」"
        );
        assert_eq!(
            used.contains("只报不判"),
            vdn_eval::scorecard::standard::is_report_only(v),
            "{v} 的「只报不判」"
        );
        for g in Group::ALL {
            let in_code = g.extra().iter().any(|i| i.v == v);
            // 「quiet」等类名按词出现；meeting-room 含连字符，单独按全名找
            let in_doc = used.split(['·', ' ']).any(|w| w.trim() == g.as_str());
            assert_eq!(in_doc, in_code, "{v} 的「用在」与 {} 类不一致", g.as_str());
        }
        // 过关线格里的全部数 = 任务给的全部参数（按顺序）；没给参数的 = 代码实际用的那一个
        let got = all_numbers(line);
        let mut want = explicit_params(v);
        if want.is_empty() {
            match effective(v) {
                Some(x) => want.push(x),
                // 代码里没有过关线的，只能是只报不判、且文档写明「待人拍板」（数可以是括号里的暂取值，不比）
                None => {
                    assert!(
                        vdn_eval::scorecard::standard::is_report_only(v) && line.starts_with("待人拍板"),
                        "{v} 在代码里查不到过关线，文档也没写「待人拍板」：{line:?}"
                    );
                    continue;
                }
            }
        }
        assert_eq!(
            got.len(),
            want.len(),
            "{v}：过关线格 {line:?} 里的数 {got:?} ↔ 代码 {want:?}"
        );
        for (g, w) in got.iter().zip(&want) {
            assert!((g - w).abs() < 1e-9, "{v}：scorecard.md 写 {g}，代码用 {w}");
        }
        // 不等号方向与登记表的好坏方向一致：越小越好写 ≤，越大越好写 ≥
        let better = vdn_eval::verifiers::info(v).unwrap().better;
        if line.contains('≤') {
            assert_eq!(better, vdn_eval::Better::Lower, "{v} 写了 ≤ 但越大越好");
        }
        if line.contains('≥') {
            assert_eq!(better, vdn_eval::Better::Higher, "{v} 写了 ≥ 但越小越好");
        }
    }
}

#[test]
fn split_line_in_doc_matches_code() {
    let t = doc();
    let want = vdn_eval::thresholds::NRMOS_SPLIT.def("max");
    assert!(
        t.contains(&format!("Δ 相差 ≥ {want}")),
        "第四节的分歧线与代码 {want} 不一致"
    );
    assert!(
        t.contains(&format!("分歧线 {want}")),
        "第三节脚注的分歧线与代码 {want} 不一致"
    );
}

/// 第三节「新规则（v2）判法」列 = `standard` 的 v2 分类与换线（`PHYSICAL` · `V2_REPORT_ONLY` · `CLEAN_PASS` · `V2_PARAMS`）。
#[test]
fn v2_column_matches_code() {
    use vdn_eval::scorecard::standard::{CLEAN_PASS, PHYSICAL, V2_PARAMS, V2_REPORT_ONLY};
    let t = doc();
    let sec = section(&t, "## 三、", "## 四、");
    let mut n = 0;
    for r in sec.lines().filter(|l| l.starts_with("| V-")).map(cells) {
        let (v, col) = (r[0].as_str(), r.last().unwrap().as_str());
        n += 1;
        let report = V2_REPORT_ONLY.contains(&v);
        assert_eq!(col.starts_with("只报"), report, "{v}：v2 列 {col:?}");
        assert_eq!(
            col.starts_with("逐段"),
            PHYSICAL.contains(&v) || v == "V-EAT-ASR",
            "{v}：v2 列 {col:?}"
        );
        assert_eq!(
            col.starts_with("区间"),
            !report && !PHYSICAL.contains(&v) && v != "V-EAT-ASR",
            "{v}：v2 列 {col:?}"
        );
        assert_eq!(
            col.contains("干净人声单独过一遍"),
            CLEAN_PASS.contains(&v),
            "{v}：v2 列 {col:?}"
        );
        if let Some((_, _, x)) = V2_PARAMS.iter().find(|(id, _, _)| *id == v) {
            assert_eq!(first_number(col), Some(*x), "{v}：v2 列的线 ↔ 代码 {x}");
        }
        if v == "V-EAT-ASR" {
            let k = vdn_eval::thresholds::EAT_ASR.def("fail_chars");
            assert!(
                col.contains(&format!("≥ {k} 字")),
                "{v}：v2 列 {col:?} ↔ 代码 {k} 字"
            );
        }
        if v.starts_with("V-NRMOS") {
            let d = vdn_eval::thresholds::NRMOS_DELTA.def("max");
            assert!(
                col.contains(&format!("≥ -{d}")),
                "{v}：v2 列 {col:?} ↔ 代码 δ {d}"
            );
        }
    }
    assert!(n > 15);
}

/// 第四节 v2 的数 = `thresholds` 表 `SC-AGG` · `SC-NRMOS-DELTA` · 吃字 `fail_chars` · `standard::PHYSICAL`。
#[test]
fn v2_rules_numbers_match_code() {
    use vdn_eval::thresholds::{AGG, EAT_ASR, NRMOS_DELTA};
    let t = doc();
    let sec = section(&t, "### 新规则（v2", "### 旧规则（v1");
    let want = [
        format!("{}% 区间", AGG.def("level") * 100.0),
        format!("重采样 {} 次", AGG.def("reps")),
        format!("少于 {} 组", AGG.def("min_groups")),
        format!("α = {}", AGG.def("alpha")),
        format!("Δ ≥ -{}", NRMOS_DELTA.def("max")),
        format!("≥ {} 字", EAT_ASR.def("fail_chars")),
        format!("最差的 {} 段", vdn_eval::scorecard::rules_v2::LISTEN_WORST),
        vdn_eval::scorecard::standard::PHYSICAL.join(" · "),
    ];
    for w in want {
        assert!(
            sec.contains(&w),
            "第四节 v2 里找不到「{w}」（与代码不一致）"
        );
    }
}
