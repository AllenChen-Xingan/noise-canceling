//! `specs/rubric/scorecard.md` 第十节（配对比较）、第十一节（早期回声参考旁列）的表与代码逐行比对。
//! - 判断表 = `compare::Call::ALL` 的中文名（顺序一致）
//! - 参数表 = `thresholds::AGG` 里这几个参数的默认值
//! - 否决项表 = `standard::VETO` 每项一行，逐段判法 = `compare::veto_rule`
//! - 早期参考表 = `standard::EARLY_REF`（集合相等）

use std::collections::BTreeSet;
use std::path::PathBuf;

use vdn_eval::scorecard::compare::{Call, veto_rule};
use vdn_eval::scorecard::standard::{EARLY_REF, VETO};
use vdn_eval::thresholds as th;

fn doc() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../specs/rubric/scorecard.md");
    std::fs::read_to_string(p).unwrap()
}

fn section<'a>(t: &'a str, from: &str, to: &str) -> &'a str {
    let a = t.find(from).unwrap_or_else(|| panic!("找不到 {from}"));
    let r = &t[a..];
    &r[..r.find(to).unwrap_or(r.len())]
}

/// 表头第一格为 `head` 的那张表的数据行（各格已修剪）。
fn table(sec: &str, head: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut on = false;
    for l in sec.lines() {
        let l = l.trim();
        if !l.starts_with('|') {
            if on {
                break;
            }
            continue;
        }
        let cells: Vec<String> = l
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().trim_matches('`').to_string())
            .collect();
        if !on {
            on = cells[0] == head;
            continue;
        }
        if cells[0].starts_with("---") {
            continue;
        }
        out.push(cells);
    }
    assert!(!out.is_empty(), "找不到表「{head}」");
    out
}

#[test]
fn compare_tables_match_code() {
    let t = doc();
    let sec = section(&t, "## 十、", "## 十一、");
    let calls: Vec<String> = table(sec, "判断").into_iter().map(|r| r[0].clone()).collect();
    let want: Vec<String> = Call::ALL.iter().map(|c| c.zh().to_string()).collect();
    assert_eq!(calls, want);

    for r in table(sec, "参数（`SC-AGG`）".trim_matches('`')) {
        let v: f64 = r[1].parse().unwrap();
        assert_eq!(v, th::AGG.def(&r[0]), "SC-AGG.{}", r[0]);
    }

    let rows = table(sec, "否决项");
    let doc_ids: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
    let code_ids: Vec<&str> = VETO.iter().map(|i| i.v).collect();
    assert_eq!(doc_ids, code_ids);
    for r in &rows {
        assert_eq!(r[1], veto_rule(&r[0]), "{}", r[0]);
    }
}

#[test]
fn early_ref_table_matches_code() {
    let t = doc();
    let sec = section(&t, "## 十一、", "## 十二、");
    let doc_ids: BTreeSet<String> = table(sec, "有早期参考旁列的项")
        .into_iter()
        .map(|r| r[0].clone())
        .collect();
    let code: BTreeSet<String> = EARLY_REF.iter().map(|s| s.to_string()).collect();
    assert_eq!(doc_ids, code);
}
