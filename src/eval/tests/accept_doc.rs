//! 计划 014 第三节的验收线 ↔ 正本 `thresholds::ACCEPT` 逐个比对（CLAUDE.md 第五节第 5 条：一个真相源 + 交叉校验）。
//!
//! 规矩：014 第三节里每个验收数后面跟着 `（SC-ACCEPT.<键>）`，数本身必须等于表里的默认值；
//! 表里的每个参数都必须出现在这一节里。改文档、改表，任一边改了另一边没跟 → 红。

use std::path::PathBuf;

use vdn_eval::thresholds::ACCEPT;

/// 014 第三节写了的那几个键。`seeds` 不在这里：种子数来自 D99 第 1 条，014 没写这个数。
const DOC_KEYS: [&str; 5] = [
    "estoi_gain",
    "nrmos_gain",
    "onset_regress",
    "bgstab_regress",
    "timbre_regress",
];

fn repo(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("读 {rel}：{e}"))
}

/// 第三节（`## 三、` 到下一个 `## ` 之间）。
fn section_three(t: &str) -> &str {
    let a = t.find("## 三、").unwrap_or_else(|| panic!("找不到第三节"));
    let r = &t[a..];
    let end = r.find("\n## ").map(|i| i + 1).unwrap_or(r.len());
    &r[..end]
}

/// 抽出 `<数>（SC-ACCEPT.<键>）` 对：键 → 数。
fn pairs(sec: &str) -> Vec<(String, f64)> {
    const MARK: &str = "（SC-ACCEPT.";
    let mut out = Vec::new();
    let mut rest = sec;
    while let Some(i) = rest.find(MARK) {
        let tail = &rest[i + MARK.len()..];
        let key = tail.split('）').next().unwrap_or("").trim().to_string();
        let tok: String = rest[..i]
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '+' | '-'))
            .collect::<Vec<char>>()
            .into_iter()
            .rev()
            .collect();
        out.push((key, tok.parse::<f64>().unwrap_or(f64::NAN)));
        rest = &rest[i + MARK.len()..];
    }
    out
}

/// 比对一处：返回全部问题（空 = 对得上）。
fn check(sec: &str) -> Vec<String> {
    let mut errs = Vec::new();
    let found = pairs(sec);
    if found.is_empty() {
        errs.push("第三节没有任何 `（SC-ACCEPT.<键>）` 标注".to_string());
    }
    for (key, v) in &found {
        let Some(p) = ACCEPT.params.iter().find(|p| p.key == key) else {
            errs.push(format!("表里没有 SC-ACCEPT 参数 {key}"));
            continue;
        };
        let Some(d) = p.default else {
            errs.push(format!("SC-ACCEPT.{key} 没有默认值"));
            continue;
        };
        if !v.is_finite() {
            errs.push(format!(
                "SC-ACCEPT.{key} 前面读不出数（写的不是 +0.01 这种数）"
            ));
            continue;
        }
        if (d - v).abs() > 1e-12 {
            errs.push(format!("SC-ACCEPT.{key}：文档 {v} ↔ 表 {d}"));
        }
    }
    for p in ACCEPT.params.iter().filter(|p| DOC_KEYS.contains(&p.key)) {
        if !found.iter().any(|(k, _)| k == p.key) {
            errs.push(format!("第三节没写 SC-ACCEPT.{}", p.key));
        }
    }
    if !sec.contains("SC-ACCEPT") || !sec.contains("src/eval/src/thresholds/") {
        errs.push("第三节开头没写「数的正本是 SC-ACCEPT + 路径」".to_string());
    }
    errs
}

#[test]
fn section_three_numbers_match_the_table() {
    let doc = repo("docs/plan/014_training-kickoff.md");
    let errs = check(section_three(&doc));
    assert!(errs.is_empty(), "{errs:#?}");
    assert_eq!(ACCEPT.id, "SC-ACCEPT");
    // 五个数都读到，且键与表里的一一对应
    let keys: Vec<String> = pairs(section_three(&doc))
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    assert_eq!(keys.len(), DOC_KEYS.len(), "{keys:?}");
    for k in DOC_KEYS {
        assert!(keys.contains(&k.to_string()), "缺 {k}");
    }
    // 名单里的键必须在正本表里（拼错了这里先红）
    for k in DOC_KEYS {
        assert!(ACCEPT.params.iter().any(|p| p.key == k), "表里没有 {k}");
    }
}

/// 负例：把文档里的一个数改掉 → 红；删掉一个标注 → 红。
#[test]
fn changed_number_or_missing_key_is_red() {
    let doc = repo("docs/plan/014_training-kickoff.md");
    let sec = section_three(&doc).to_string();
    assert!(check(&sec).is_empty(), "{:?}", check(&sec));

    let estoi = format!("{}（SC-ACCEPT.estoi_gain）", ACCEPT.def("estoi_gain"));
    assert!(sec.contains(&estoi), "文档里找不到 {estoi}");
    let bad = sec.replace(&estoi, "+0.02（SC-ACCEPT.estoi_gain）");
    let errs = check(&bad);
    assert!(errs.iter().any(|e| e.contains("estoi_gain")), "{errs:?}");

    let dropped = sec.replace("（SC-ACCEPT.timbre_regress）", "");
    let errs = check(&dropped);
    assert!(
        errs.iter().any(|e| e.contains("timbre_regress")),
        "{errs:?}"
    );

    // 表里有、文档里写错键名
    let renamed = sec.replace("（SC-ACCEPT.onset_regress）", "（SC-ACCEPT.onset_regres）");
    let errs = check(&renamed);
    assert!(errs.iter().any(|e| e.contains("onset_regres")), "{errs:?}");
}
