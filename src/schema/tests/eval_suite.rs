//! 交叉校验 · 评测清单：清单文件过 schema、往返等价、坏任务 schema 与解析器判定一致。

use serde_json::{Value, json};
use vdn_schema::{eval, root};

mod common;
use common::*;

// ───────────── 评测清单 ─────────────

/// 4 份清单文件全部过各自的 schema。
#[test]
fn eval_suite_files_pass_schema() {
    for (path, schema_file) in eval::DATA_FILES {
        let schema = committed(schema_file);
        let t = std::fs::read_to_string(root().join(path)).unwrap();
        let v: Value = serde_json::from_str(&t).unwrap();
        validate(&schema, &v).unwrap_or_else(|e| panic!("{path} 不过 {schema_file}：{e}"));
    }
}

/// `suite/` 下的每份 json 都登记在 DATA_FILES 里（新加清单不会漏掉校验）。
#[test]
fn eval_data_files_registered() {
    let d = root().join("src/eval/suite");
    for e in std::fs::read_dir(&d).unwrap().flatten() {
        let n = format!("src/eval/suite/{}", e.file_name().to_string_lossy());
        if n.ends_with(".json") {
            assert!(
                eval::DATA_FILES.iter().any(|(p, _)| *p == n),
                "{n} 没登记在 vdn_schema::eval::DATA_FILES"
            );
        }
    }
}

/// schema 与解析器对坏文件的判定一致：拼错键名、未知 verifier、未知噪音、多余的 kind 字段。
#[test]
fn eval_schema_and_parser_agree_on_bad_tasks() {
    let schema = committed(eval::SUITE_FILE);
    let task = |extra: Value| {
        let mut t = json!({"id": "R99", "set": "regression", "origin": "测试",
            "scene": {"noise": "pink"}, "graders": [{"v": "V-NAN"}]});
        for (k, v) in extra.as_object().unwrap() {
            t[k] = v.clone();
        }
        json!({"tasks": [t]})
    };
    let cases = [
        (task(json!({})), true),
        (task(json!({"gradres": []})), false),
        (task(json!({"graders": [{"v": "V-NOPE"}]})), false),
        (
            task(json!({"graders": [{"v": "V-NAN", "max": "3"}]})),
            false,
        ),
        (task(json!({"scene": {"noise": "rain"}})), false),
        (task(json!({"scene": {"pause_s": [1]}})), false),
        (task(json!({"set": "smoke"})), false),
        (
            task(json!({"real": {"kind": "exp003", "clips": "a", "clip": "b", "extra": 1}})),
            false,
        ),
        (
            task(json!({"faults": [{"hop": 3, "kind": "stall"}]})),
            false,
        ),
        (
            task(json!({"probe": {"base": "R99", "kind": "mute"}})),
            true,
        ),
    ];
    for (doc, want) in cases {
        let by_schema = validate(&schema, &doc).is_ok();
        let by_rust = vdn_eval::suite::parse_suite(&doc.to_string()).is_ok();
        assert_eq!(by_schema, want, "schema：{doc}");
        assert_eq!(by_rust, want, "解析器：{doc}");
    }
}

/// 整数范围（codex 第 3 轮 #4）：每个整数字段都有上下界；超出 Rust 类型的数 schema 与解析器都拒。
#[test]
fn integer_bounds_agree_with_rust_types() {
    fn walk(path: &str, v: &Value, bad: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                let int_format = m
                    .get("format")
                    .and_then(Value::as_str)
                    .is_some_and(|f| f.starts_with("int") || f.starts_with("uint"));
                if int_format && !(m.contains_key("minimum") && m.contains_key("maximum")) {
                    bad.push(path.to_string());
                }
                for (k, x) in m {
                    walk(&format!("{path}/{k}"), x, bad);
                }
            }
            Value::Array(a) => {
                for (i, x) in a.iter().enumerate() {
                    walk(&format!("{path}/{i}"), x, bad);
                }
            }
            _ => {}
        }
    }
    let mut bad = Vec::new();
    for f in [
        "config.schema.json",
        eval::SUITE_FILE,
        eval::LEVELS_FILE,
        eval::BASELINE_FILE,
    ] {
        walk(f, &committed(f), &mut bad);
    }
    assert!(bad.is_empty(), "整数字段缺上下界：{bad:#?}");

    let schema = committed(eval::SUITE_FILE);
    let task = r#"{"id": "R99", "set": "regression", "origin": "测试",
        "scene": {"noise": "pink"}, "graders": [{"v": "V-NAN"}]}"#;
    let cases = [
        ("18446744073709551615", true),   // u64::MAX
        ("100000000000000000000", false), // 1e20，超出 u64
        ("-1", false),
    ];
    for (seed, want) in cases {
        let text = format!(r#"{{"seed": {seed}, "tasks": [{task}]}}"#);
        let doc: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            validate(&schema, &doc).is_ok(),
            want,
            "schema：seed = {seed}"
        );
        assert_eq!(
            vdn_eval::suite::parse_suite(&text).is_ok(),
            want,
            "解析器：seed = {seed}"
        );
    }
}

/// 负例：清单类型加了字段而 schema 没重新导出 → 检查报出对应文件（三份各一例）。
#[test]
fn eval_type_change_without_export_is_red() {
    assert_missing_field_is_red(
        "eval-suite",
        eval::SUITE_FILE,
        "/$defs/TaskFile/properties",
        "xfail",
    );
    assert_missing_field_is_red("eval-levels", eval::LEVELS_FILE, "/properties", "updated");
    assert_missing_field_is_red(
        "eval-baseline",
        eval::BASELINE_FILE,
        "/properties",
        "run_id",
    );
}

#[test]
fn same_keeps_integers_exact() {
    assert!(!same(
        &json!(9_007_199_254_740_992_u64),
        &json!(9_007_199_254_740_993_u64)
    ));
    assert!(same(&json!(3), &json!(3.0)));
    assert!(!same(&json!(3), &json!(3.5)));
    assert!(same(
        &norm(&json!({"a": null, "b": [], "c": 1})),
        &json!({"c": 1})
    ));
    // 大整数种子往返不丢精度
    let doc = json!({"seed": 9_007_199_254_740_993_u64, "defaults": {"trials": 3}, "tasks": []});
    let t: vdn_eval::files::SuiteFile = serde_json::from_value(doc.clone()).unwrap();
    assert!(same(&norm(&doc), &norm(&serde_json::to_value(&t).unwrap())));
}

/// 4 份清单文件：反序列化成格式类型再序列化，语义等价（口径见 `norm` 与 `same`）。
#[test]
fn eval_files_roundtrip_semantic() {
    use vdn_eval::files::{BaselineFile, LevelsFile, SuiteFile};
    for (path, schema_file) in eval::DATA_FILES {
        let v: Value =
            serde_json::from_str(&std::fs::read_to_string(root().join(path)).unwrap()).unwrap();
        match schema_file {
            eval::SUITE_FILE => roundtrip::<SuiteFile>(path, &v),
            eval::LEVELS_FILE => roundtrip::<LevelsFile>(path, &v),
            eval::BASELINE_FILE => roundtrip::<BaselineFile>(path, &v),
            other => panic!("{path}：没写往返的 schema {other}"),
        }
    }
}

/// 两个写入口（`ci --write-baseline`、`calib --write-levels`）序列化的类型，写出来的东西过 schema、能读回；
/// 基线文件元数据不对时 `BaselineFile::parse` 报错，不当成空基线（第 1 轮 codex #2）。
#[test]
fn eval_writer_outputs_pass_schema() {
    use vdn_eval::files::{BaselineFile, LevelEntry, LevelsFile};
    let b = BaselineFile {
        commit: "abc1234-dirty".into(),
        doc: "d".into(),
        run_id: "r".into(),
        values: [("ref/R01/0/V-MUSIC".to_string(), 1.25)].into(),
    };
    let bv = serde_json::to_value(&b).unwrap();
    validate(&committed(eval::BASELINE_FILE), &bv).unwrap();
    assert_eq!(
        BaselineFile::parse(&bv.to_string()).unwrap().values,
        b.values
    );
    let l = LevelsFile {
        doc: "d".into(),
        updated: "2026-11-02".into(),
        verifiers: [(
            "V-NAN".to_string(),
            LevelEntry {
                basis: "物理量".into(),
                level: vdn_eval::Level::L3,
            },
        )]
        .into(),
    };
    validate(
        &committed(eval::LEVELS_FILE),
        &serde_json::to_value(&l).unwrap(),
    )
    .unwrap();
    // values 合法、元数据坏 → 报错
    for bad in [
        json!({"values": {"a": 1.0}}),
        json!({"commit": "c", "doc": "d", "run_id": "r", "values": {"a": 1.0}, "extra": 1}),
        json!({"commit": "c", "doc": "d", "run_id": "r", "values": {"a": "1"}}),
    ] {
        assert!(BaselineFile::parse(&bad.to_string()).is_err(), "{bad}");
        assert!(
            validate(&committed(eval::BASELINE_FILE), &bad).is_err(),
            "{bad}"
        );
    }
}

// ───────────── 场景片段清单 ─────────────

/// 进仓的样例行过 schema，解析器也读得了（正例）。
#[test]
fn scene_clip_example_passes_schema_and_parser() {
    let schema = committed(eval::SCENE_CLIP_FILE);
    let t = std::fs::read_to_string(root().join(eval::SCENE_CLIP_EXAMPLE)).unwrap();
    for line in t.lines().filter(|l| !l.trim().is_empty()) {
        let v: Value = serde_json::from_str(line).unwrap();
        validate(&schema, &v).unwrap_or_else(|e| panic!("样例行不过 schema：{e}"));
    }
    vdn_eval::scene::SceneClip::parse_jsonl(&t).unwrap();
}

/// 坏行：schema 与解析器判定一致（负例）。只比 schema 表达得了的约束；
/// 「stems 与 sounds 对得上」「区间不越界」这类跨字段约束只在解析器里（`SceneClip::check`），这里不比。
#[test]
fn scene_clip_schema_and_parser_agree_on_bad_lines() {
    let schema = committed(eval::SCENE_CLIP_FILE);
    let t = std::fs::read_to_string(root().join(eval::SCENE_CLIP_EXAMPLE)).unwrap();
    let base: Value = serde_json::from_str(t.lines().next().unwrap()).unwrap();
    let edit = |f: &dyn Fn(&mut Value)| {
        let mut x = base.clone();
        f(&mut x);
        x
    };
    let cases = [
        (edit(&|_| {}), true),
        (edit(&|x| x["scnee"] = json!("cafe")), false),
        (edit(&|x| x["scene"] = json!("beach")), false),
        (edit(&|x| x["device"] = json!("mba")), false),
        (edit(&|x| x["mic_mode"] = json!("isolation")), false),
        (edit(&|x| x["tier"] = json!("hard")), false),
        (edit(&|x| x["sounds"][0]["group"] = json!("noise")), false),
        (edit(&|x| x["owner_iv"] = json!([[1, 2, 3]])), false),
        (
            edit(&|x| x["hash"]["wav_sha256"]["stems"] = json!(7)),
            false,
        ),
        (edit(&|x| x["nominal"]["sir_db"] = json!(null)), true),
        // 方位衰减可选；有就必须三段齐全
        (
            edit(&|x| {
                x["sounds"][1]
                    .as_object_mut()
                    .unwrap()
                    .remove("direction_db")
                    .map_or((), |_| ())
            }),
            true,
        ),
        (
            edit(&|x| x["sounds"][1]["direction_db"] = json!({"low": 0.0, "mid": 0.0})),
            false,
        ),
        (
            edit(&|x| {
                x["targets"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("src")
                    .map_or((), |_| ())
            }),
            false,
        ),
        (
            edit(&|x| {
                x["measured"]
                    .as_object_mut()
                    .unwrap()
                    .remove("snr_all_db")
                    .map_or((), |_| ())
            }),
            false,
        ),
    ];
    for (v, ok) in cases {
        let s_ok = validate(&schema, &v).is_ok();
        let p_ok = serde_json::from_value::<vdn_eval::scene::SceneClip>(v.clone()).is_ok();
        assert_eq!(s_ok, ok, "schema 判错：{v}");
        assert_eq!(p_ok, ok, "解析器判错：{v}");
    }
}
