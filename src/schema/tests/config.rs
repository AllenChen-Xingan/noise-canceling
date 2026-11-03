//! 交叉校验 · Config：示例过 schema、往返等价、范围与总线名和 `Config::validate` 逐例一致。

use serde_json::{Value, json};
use vdn_control::Config;
use vdn_schema::{DIR, root};

mod common;
use common::*;

// ───────────── Config ─────────────

fn config_examples() -> Vec<(String, Value)> {
    let d = root().join(DIR).join("examples");
    let mut out: Vec<(String, Value)> = std::fs::read_dir(&d)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("config.") && n.ends_with(".json"))
        .map(|n| {
            let t = std::fs::read_to_string(d.join(&n)).unwrap();
            let v = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{n}：{e}"));
            (n, v)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(
        out.len() >= 2,
        "examples/ 里至少要有 config.default.json 和一份部分配置"
    );
    out
}

/// 仓里所有 Config 示例都过 schema，也都被 `Config::from_json` 接受。
#[test]
fn config_examples_pass_schema_and_parse() {
    let schema = committed("config.schema.json");
    for (n, v) in config_examples() {
        validate(&schema, &v).unwrap_or_else(|e| panic!("{n} 不过 schema：{e}"));
        Config::from_json(&v.to_string()).unwrap_or_else(|e| panic!("{n}：{e}"));
    }
}

/// 往返（口径：语义等价）：读进来再写出去，
/// ① 原文件里写了的每个键，写出后值相同（数字按 JSON 数值比，不比空白和键顺序）；
/// ② 原文件没写的键，写出后等于 `Config::default()` 的值；
/// ③ 写出的文本再读一次，得到同一个 `Config`。
#[test]
fn config_roundtrip_semantic() {
    let def = serde_json::to_value(Config::default()).unwrap();
    for (n, orig) in config_examples() {
        let c = Config::from_json(&orig.to_string()).unwrap();
        let out: Value = serde_json::from_str(&c.to_json()).unwrap();
        let (o, w) = (orig.as_object().unwrap(), out.as_object().unwrap());
        for (k, v) in w {
            let want = o.get(k).unwrap_or(&def[k]);
            assert_eq!(v, want, "{n}：键 {k}");
        }
        assert_eq!(
            o.keys().filter(|k| !w.contains_key(*k)).count(),
            0,
            "{n}：有键写出时丢了"
        );
        assert_eq!(Config::from_json(&c.to_json()).unwrap(), c, "{n}");
    }
}

/// 默认值示例必须与 `Config::default()` 一致（示例是第二处表示）。
#[test]
fn config_default_example_is_default() {
    let v: Value = config_examples()
        .into_iter()
        .find(|(n, _)| n == "config.default.json")
        .unwrap()
        .1;
    assert_eq!(v, serde_json::to_value(Config::default()).unwrap());
}

/// 单字段的范围：schema 的判定与 `Config::validate` 在边界上逐例一致（范围数字只写在 Config 的常量里）。
#[test]
fn config_schema_ranges_agree_with_validate() {
    let schema = committed("config.schema.json");
    let (blo, bhi) = Config::BLOCK_RANGE;
    let (hlo, hhi) = Config::HEARTBEAT_RANGE_MS;
    let lmax = Config::LEAD_MAX;
    // lead 取得够大，避开「lead ≥ IO 块」这条跨字段规则
    let cases = [
        json!({"block": blo - 1, "lead_frames": bhi}),
        json!({"block": blo, "lead_frames": bhi}),
        json!({"block": bhi, "lead_frames": bhi}),
        json!({"block": bhi + 1, "lead_frames": bhi + 1}),
        json!({"heartbeat_timeout_ms": hlo - 1}),
        json!({"heartbeat_timeout_ms": hlo}),
        json!({"heartbeat_timeout_ms": hhi}),
        json!({"heartbeat_timeout_ms": hhi + 1}),
        json!({"denoise_block": 0}),
        json!({"denoise_block": 1}),
        json!({"denoise_block": bhi}),
        json!({"denoise_block": bhi + 1}),
        json!({"lead_frames": lmax}),
        json!({"lead_frames": lmax + 1}),
        json!({"input_device": "  "}),
        json!({"input_device": "x"}),
        json!({"output_device": ""}),
        json!({"mode": "bypass"}),
        json!({"model": "gtcrn"}),
        json!({"blok": 96}),
    ];
    for c in cases {
        let by_schema = validate(&schema, &c).is_ok();
        let by_rust = Config::from_json(&c.to_string()).is_ok();
        assert_eq!(
            by_schema, by_rust,
            "{c}：schema {by_schema} · Rust {by_rust}"
        );
    }
}

/// 总线名：schema 的正则 + 长度 与 `validate_bus_name` 逐例一致。
/// 名字只许可见 ASCII，所以 schema 的 maxLength（数字符）与 Rust 的长度（数字节）是同一个数。
#[test]
fn bus_name_pattern_agrees_with_validate() {
    let schema = committed("config.schema.json");
    let long_ok = format!("/vdn-{}", "a".repeat(Config::BUS_NAME_MAX - 5));
    let long_bad = format!("/vdn-{}", "a".repeat(Config::BUS_NAME_MAX - 4));
    // 14 个字符、32 字节：按字符数会放行，按字节数要拒（codex 第 2 轮 #2）
    let multi = format!("/vdn-{}", "中".repeat(9));
    for name in [
        "/vdn-bus",
        "/vdn-x",
        "/vdn-",
        "/vdn",
        "vdn-bus",
        "/other",
        "/vdn-a/b",
        "/vdn-a\u{0}b",
        "/vdn-a b",
        "/vdn-~!",
        "/vdn-中",
        &multi,
        &long_ok,
        &long_bad,
    ] {
        let by_rust = vdn_control::validate_bus_name(name).is_ok();
        // 另一个总线名取不会撞的合法值，只测这一个字段
        let c = json!({"bus_name": name, "out_bus_name": "/vdn-zz-other"});
        let by_schema = validate(&schema, &c).is_ok();
        assert_eq!(
            by_schema, by_rust,
            "{name:?}：schema {by_schema} · Rust {by_rust}"
        );
    }
}

/// 负例：Config 加了字段而 schema 没重新导出 → 检查报 config.schema.json。
#[test]
fn config_new_field_without_export_is_red() {
    assert_missing_field_is_red(
        "config",
        "config.schema.json",
        "/properties",
        "denoise_block",
    );
}

/// 配置里的模型名（serde 的写法、schema 的枚举、默认值）与正本 `ModelId::NAMES` 一致，顺序也一致。
/// `ModelChoice` 是 vdn-control 里的第二处表示（它不依赖 vdn-types），由这条测试钉住。
#[test]
fn config_model_names_follow_model_id() {
    use vdn_control::ModelChoice;
    use vdn_types::ModelId;
    let s = committed("config.schema.json");
    let en: Vec<&str> = s["$defs"]["ModelChoice"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(en, ModelId::NAMES);
    for id in ModelId::ALL {
        let c: ModelChoice = serde_json::from_value(json!(id.as_str())).unwrap();
        assert_eq!(serde_json::to_value(c).unwrap(), json!(id.as_str()));
    }
    assert_eq!(
        serde_json::to_value(ModelChoice::default()).unwrap(),
        json!(ModelId::default().as_str())
    );
}
