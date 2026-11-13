use serde_json::{Value, json};

use super::*;

fn repo_json(rel: &str) -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    serde_json::from_str(&std::fs::read_to_string(&p).expect("读仓内数据")).expect("JSON")
}

fn mk(tp: usize, fn_: usize, tn: usize, fp: usize) -> Vec<(V, V)> {
    let mut v = Vec::new();
    v.extend(std::iter::repeat_n((V::Fail, V::Fail), tp));
    v.extend(std::iter::repeat_n((V::Fail, V::Pass), fn_));
    v.extend(std::iter::repeat_n((V::Pass, V::Pass), tn));
    v.extend(std::iter::repeat_n((V::Pass, V::Fail), fp));
    v
}

#[test]
fn rates_from_known_matrix() {
    // evals FAQ 的例子：人判 10 个坏的抓到 8 个；100 个好的放过 95 个
    let c = Confusion::of(&mk(8, 2, 95, 5));
    assert!((c.tpr() - 0.8).abs() < 1e-12);
    assert!((c.tnr() - 0.95).abs() < 1e-12);
}

#[test]
fn always_pass_judge_has_zero_tpr() {
    // 5% 坏的、一律判过：总一致率 95%，真阳性率 0
    let c = Confusion::of(&mk(0, 5, 95, 0));
    assert_eq!(c.tpr(), 0.0);
    assert_eq!(c.tnr(), 1.0);
}

#[test]
fn wilson_known_value() {
    let (lo, hi) = wilson(8, 10);
    assert!((lo - 0.4902).abs() < 1e-3 && (hi - 0.9433).abs() < 1e-3);
    assert_eq!(wilson(0, 5).0, 0.0);
    assert!(wilson(0, 0).0.is_nan());
}

fn item<'a>(usable: Option<&'a str>, eaten: Option<bool>, s: &'a Value, n: &'a Value) -> Item<'a> {
    Item {
        clip: "c",
        model: "m",
        usable,
        eaten,
        scores: s,
        noisy: n,
        pred: &Value::Null,
        pred_noisy: &Value::Null,
    }
}

#[test]
fn human_cuts() {
    let z = json!({});
    let t = |u: &str, h| item(Some(u), Some(false), &z, &z).truth(h);
    assert_eq!(t("no", Human::UsableLoose), Some(V::Fail));
    assert_eq!(t("marginal", Human::UsableLoose), Some(V::Pass));
    assert_eq!(t("marginal", Human::UsableStrict), Some(V::Fail));
    assert_eq!(t("ok", Human::UsableStrict), Some(V::Pass));
    assert_eq!(t("bogus", Human::UsableStrict), None);
    assert_eq!(
        item(None, Some(true), &z, &z).truth(Human::Eaten),
        Some(V::Fail)
    );
    assert_eq!(item(None, None, &z, &z).truth(Human::Eaten), None);
}

/// 吃字线从正本 `V-EAT-ASR.fail_chars` 取：多错正好等于线 → 不过，少一个 → 过。
#[test]
fn eat_line_from_table() {
    let lim = th::EAT_ASR.def("fail_chars");
    let j = judges().into_iter().find(|j| j.id == "eat_count").unwrap();
    let n = json!({"del": 0.0, "sub": 0.0, "ref_len": 1000.0});
    let at = json!({"del": lim, "sub": 0.0, "ref_len": 1000.0});
    let below = json!({"del": lim - 1.0, "sub": 0.0, "ref_len": 1000.0});
    assert_eq!((j.judge)(&item(None, None, &at, &n)), Some(V::Fail));
    assert_eq!((j.judge)(&item(None, None, &below, &n)), Some(V::Pass));
}

/// 没给评分模型表：听感几行条数为 0，全数记「判不了」，不报错。
#[test]
fn missing_predictors_skip_listening() {
    let a = repo_json(
        "docs/experiments/exp003-offline-ab/results/20261030-human-build-20261030110755-f8c3.json",
    );
    let s = repo_json("docs/experiments/exp003-offline-ab/scores.json");
    let rows = table(&items(&a, &s, &Value::Null), 1, 50);
    let sq = rows.iter().find(|r| r["judge"] == "scoreq_v1").unwrap();
    assert_eq!(sq["n"], 0);
    assert_eq!(sq["skipped"], 36);
    let eat = rows.iter().find(|r| r["judge"] == "eat_pp").unwrap();
    assert_eq!(eat["n"], 36);
}

/// 交叉校验：与 exp022 的 Python 独立实现逐格比对混淆矩阵（两边都从 `thresholds/table.rs` 取线）。
#[test]
fn matches_exp022_python() {
    let a = repo_json(
        "docs/experiments/exp003-offline-ab/results/20261030-human-build-20261030110755-f8c3.json",
    );
    let s = repo_json("docs/experiments/exp003-offline-ab/scores.json");
    let p = repo_json("docs/experiments/exp009-more-ears/results/exp003_predictors.json");
    let py = repo_json("docs/experiments/exp022-judge-tpr-tnr/results/tpr_tnr.json");
    let rows = table(&items(&a, &s, &p), 1, 50);
    let human = |h: &str| match h {
        "usable_loose" => "h_loose",
        "usable_strict" => "h_strict",
        _ => "h_eaten",
    };
    let mut checked = 0;
    for r in &rows {
        // 两边命名约定：Python 的 judge_key = "j_" + 本文件的 judge id
        let jk = format!("j_{}", r["judge"].as_str().unwrap());
        let hk = human(r["human"].as_str().unwrap());
        let other = py["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["judge_key"] == jk.as_str() && x["human_key"] == hk)
            .unwrap_or_else(|| panic!("exp022 里没有 {jk} × {hk}"));
        for k in ["tp", "fn", "tn", "fp"] {
            assert_eq!(
                r["confusion"][k], other["confusion"][k],
                "{jk} × {hk} 的 {k}"
            );
        }
        checked += 1;
    }
    assert_eq!(checked, 10);
}

/// codex 第 1 轮 P1 #1：吃字题没答（字段缺、null、空列表）的屏不进吃字的混淆矩阵，记「判不了」；
/// 明确答「都没有」（`["none"]`）照常算人判过。
#[test]
fn unanswered_eaten_is_skipped_not_pass() {
    let mk = |eaten: Value| {
        json!({"trials": [{"kind": "real", "clip": "c1", "slots": {"A": "m"},
            "by_model": {"usable": {"m": "ok"}, "eaten": eaten}}]})
    };
    let s = json!({"per_clip": {"c1": {"noisy": {"del": 0.0, "sub": 0.0, "ref_len": 10.0},
        "m": {"del": 0.0, "sub": 0.0, "ref_len": 10.0}}}});
    let eat_n = |a: &Value| {
        let rows = table(&items(a, &s, &Value::Null), 1, 10);
        let r = rows.iter().find(|r| r["judge"] == "eat_pp").unwrap();
        (r["n"].as_u64().unwrap(), r["skipped"].as_u64().unwrap())
    };
    let missing = json!({"trials": [{"kind": "real", "clip": "c1", "slots": {"A": "m"},
        "by_model": {"usable": {"m": "ok"}}}]});
    assert_eq!(eat_n(&missing), (0, 1));
    assert_eq!(eat_n(&mk(Value::Null)), (0, 1));
    assert_eq!(eat_n(&mk(json!([]))), (0, 1));
    assert_eq!(eat_n(&mk(json!(["none"]))), (1, 0));
}
