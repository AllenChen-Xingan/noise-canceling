//! 交叉校验（CLAUDE.md 第五节第 5 条）：Rust 类型 ↔ 仓里的 schema 文件 ↔ 仓里的数据文件。
//!
//! 每份 schema 都有正例（仓里的文件一致）和负例（改了 Rust 不重新导出 → 检查报错）。
//! 文件过 schema 用 `boon`（另一家实现）校验，不用导出 schema 的同一个库。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vdn_control::Config;
use vdn_schema::{DIR, check, eval, export, root, shm, write};

fn committed(file: &str) -> Value {
    let p = root().join(DIR).join(file);
    let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}：{e}", p.display()));
    serde_json::from_str(&t).unwrap()
}

/// 用仓里的 schema 校验一份 JSON；返回 Err(原因)。
fn validate(schema: &Value, inst: &Value) -> Result<(), String> {
    let mut schemas = boon::Schemas::new();
    let mut c = boon::Compiler::new();
    c.add_resource("file:///schema.json", schema.clone())
        .map_err(|e| e.to_string())?;
    let idx = c
        .compile("file:///schema.json", &mut schemas)
        .map_err(|e| e.to_string())?;
    schemas.validate(inst, idx).map_err(|e| e.to_string())
}

/// 临时仓库根：只放 `specs/schema/`，由当前 Rust 正本导出。
fn temp_root(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("schema-{tag}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    write(&d).unwrap();
    assert!(check(&d).is_empty(), "{:#?}", check(&d));
    d
}

fn put(root: &Path, file: &str, v: &Value) {
    std::fs::write(root.join(DIR).join(file), vdn_schema::pretty(v)).unwrap();
}

// ───────────── 机制 ─────────────

/// 正例：仓里的 schema 文件与 Rust 正本逐字一致。红了就跑 `cargo run -p vdn-schema` 并提交。
#[test]
fn committed_schemas_match_rust() {
    let p = check(&root());
    assert!(p.is_empty(), "{p:#?}");
}

/// 负例：任何一份导出物与仓里不同（= 改了 Rust 没重新导出），检查都报出这份文件。
#[test]
fn any_stale_doc_is_reported() {
    let d = temp_root("stale");
    for doc in export() {
        let path = d.join(DIR).join(doc.file);
        std::fs::write(&path, doc.text.replace("\"", "\"x")).unwrap();
        let p = check(&d);
        assert!(
            p.iter()
                .any(|m| m.contains(doc.file) && m.contains("不一致")),
            "{}：{p:#?}",
            doc.file
        );
        std::fs::write(&path, &doc.text).unwrap();
    }
    assert!(check(&d).is_empty());
}

/// 负例：目录里多一份没人导出的 json（类型删了 / 改名留下的旧文件）也报。
#[test]
fn orphan_file_is_reported() {
    let d = temp_root("orphan");
    put(&d, "old.schema.json", &json!({}));
    assert!(check(&d).iter().any(|m| m.contains("old.schema.json")));
}

// ───────────── 共享内存头 ─────────────

/// 逐项比对：仓里布局文件的每个结构、每个字段（名 · 偏移 · 大小）都等于 `offset_of!` / `size_of` 算出的值。
#[test]
fn shm_layout_fields_match_offset_of() {
    let doc = committed(shm::LAYOUT_FILE);
    assert_eq!(
        doc["version"],
        json!(vdn_shm::layout::VERSION),
        "版本号与 layout::VERSION 不同"
    );
    assert_eq!(doc["header_bytes"], json!(vdn_shm::layout::HEADER_BYTES));
    assert_eq!(doc["slot_bytes"], json!(vdn_shm::layout::SLOT_BYTES));
    let structs = doc["structs"].as_array().unwrap();
    assert_eq!(structs.len(), vdn_shm::desc::STRUCTS.len());
    for (s, j) in vdn_shm::desc::STRUCTS.iter().zip(structs) {
        assert_eq!(j["name"], json!(s.name));
        assert_eq!(j["size"], json!(s.size), "{} 大小", s.name);
        assert_eq!(j["align"], json!(s.align), "{} 对齐", s.name);
        let fs = j["fields"].as_array().unwrap();
        assert_eq!(fs.len(), s.fields.len(), "{} 字段数", s.name);
        for (f, jf) in s.fields.iter().zip(fs) {
            assert_eq!(jf["name"], json!(f.name), "{} 字段顺序", s.name);
            assert_eq!(jf["offset"], json!(f.offset), "{}.{} 偏移", s.name, f.name);
            assert_eq!(jf["size"], json!(f.size), "{}.{} 大小", s.name, f.name);
        }
    }
    // 展开表的几个锚点直接用 offset_of! 再算一遍（不经 desc）
    use core::mem::{offset_of, size_of};
    use vdn_shm::layout::{ConfigLine, ConsLine, Header, ProdLine, RingHdr};
    let flat = doc["flat"].as_array().unwrap();
    let at = |path: &str| {
        flat.iter()
            .find(|f| f["path"] == path)
            .unwrap_or_else(|| panic!("flat 里没有 {path}"))["offset"]
            .as_u64()
            .unwrap() as usize
    };
    assert_eq!(
        at("config.session"),
        offset_of!(Header, config) + offset_of!(ConfigLine, session)
    );
    assert_eq!(
        at("rings[2].cons.gaps"),
        offset_of!(Header, rings)
            + 2 * size_of::<RingHdr>()
            + offset_of!(RingHdr, cons)
            + offset_of!(ConsLine, gaps)
    );
    assert_eq!(
        at("rings[1].prod.wake"),
        offset_of!(Header, rings)
            + size_of::<RingHdr>()
            + offset_of!(RingHdr, prod)
            + offset_of!(ProdLine, wake)
    );
}

/// 负例：布局变了（这里把一个偏移挪 8 字节模拟）而版本号没升 → 守卫报错，`write` 一个文件都不写。
#[test]
fn shm_layout_change_without_version_bump_is_red() {
    let body = shm::layout_body();
    let fp = shm::fingerprint(&body);
    let mut moved = body.clone();
    moved["flat"][1]["offset"] = json!(16);
    let fp2 = shm::fingerprint(&moved);
    assert_ne!(fp, fp2);
    let v = vdn_shm::layout::VERSION;
    let hist = [(v, fp.as_str())];
    // 布局变了、版本没升
    let e = shm::guard(&hist, v, &fp2).unwrap_err();
    assert!(e.contains("必须升版本"), "{e}");
    // 升了版本、但没在 HISTORY 登记：也红（导出工具不会替你登记）
    let e = shm::guard(&hist, v + 1, &fp2).unwrap_err();
    assert!(e.contains("还没登记"), "{e}");
    // 升了版本且登记了：放行
    assert!(shm::guard(&[(v, fp.as_str()), (v + 1, fp2.as_str())], v + 1, &fp2).is_ok());
    // 版本往回降、历史乱序、空历史：红
    assert!(shm::guard(&hist, v - 1, &fp).is_err());
    assert!(shm::guard(&[(v + 1, "a"), (v, fp.as_str())], v, &fp).is_err());
    assert!(shm::guard(&[], v, &fp).is_err());

    // 端到端：历史与当前布局不符 → write 一个文件都不写、check 报错
    let d = temp_root("bump");
    let bogus = [(v, "fnv1a64:0000000000000000")];
    std::fs::remove_file(d.join(DIR).join(shm::LAYOUT_FILE)).unwrap();
    let e = vdn_schema::write_with(&d, &bogus).unwrap_err();
    assert!(e.contains("必须升版本"), "{e}");
    assert!(
        !d.join(DIR).join(shm::LAYOUT_FILE).exists(),
        "守卫没过还写了文件"
    );
    write(&d).unwrap();
    assert!(
        vdn_schema::check_with(&d, &bogus)
            .iter()
            .any(|m| m.contains("必须升版本"))
    );
}

/// 当前版本正是 HISTORY 的最后一行、指纹相同。
#[test]
fn shm_version_registered() {
    assert_eq!(
        shm::HISTORY.last().copied(),
        Some((
            vdn_shm::layout::VERSION,
            shm::fingerprint(&shm::layout_body()).as_str()
        ))
    );
}

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
/// 口径：schema 的 maxLength 数字符、Rust 数字节；只用 ASCII 名字时两者相同（非 ASCII 时 schema 更宽，Rust 兜底）。
#[test]
fn bus_name_pattern_agrees_with_validate() {
    let schema = committed("config.schema.json");
    let long_ok = format!("/vdn-{}", "a".repeat(Config::BUS_NAME_MAX - 5));
    let long_bad = format!("/vdn-{}", "a".repeat(Config::BUS_NAME_MAX - 4));
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

/// 模拟「Rust 加了一个字段、没重新导出」：把当前导出删掉这一个字段当作仓里的旧文件，
/// 其余（标题、注释、别的字段）都不变，所以报红只能来自这一个字段（第 1 轮 codex #4）。
fn assert_missing_field_is_red(tag: &str, file: &str, pointer: &str, field: &str) {
    let d = temp_root(tag);
    let fresh: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join(DIR).join(file)).unwrap()).unwrap();
    let mut old = fresh.clone();
    let props = old
        .pointer_mut(pointer)
        .and_then(Value::as_object_mut)
        .unwrap_or_else(|| panic!("{file} 里没有 {pointer}"));
    assert!(
        props.remove(field).is_some(),
        "{file}{pointer} 里没有 {field}"
    );
    if let Some(Value::Array(req)) = old
        .pointer_mut(pointer.trim_end_matches("/properties"))
        .and_then(|o| o.get_mut("required"))
    {
        req.retain(|r| r != field);
    }
    put(&d, file, &old);
    let p = check(&d);
    assert_eq!(p.len(), 1, "只该报这一份：{p:#?}");
    assert!(p[0].contains(file) && p[0].contains("不一致"), "{p:#?}");
    // 放回去就绿
    put(&d, file, &fresh);
    assert!(check(&d).is_empty());
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

/// 往返的比较口径：null 与缺省等价、空数组与缺省等价（serde 写出 `None` 为 null、空 Vec 为 []），
/// 数字一律按 f64 比（文件里写 3，写出为 3.0）。
fn norm(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(_, x)| !x.is_null() && x.as_array().is_none_or(|a| !a.is_empty()))
                .map(|(k, x)| (k.clone(), norm(x)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(norm).collect()),
        Value::Number(n) => json!(n.as_f64()),
        x => x.clone(),
    }
}

fn roundtrip<T: serde::de::DeserializeOwned + serde::Serialize>(path: &str, v: &Value) {
    let t: T = serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("{path}：{e}"));
    let out = serde_json::to_value(&t).unwrap();
    assert_eq!(norm(v), norm(&out), "{path}：往返后语义不同");
    // 写出的结果还能读回，再写一次不变
    let t2: T = serde_json::from_value(out.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&t2).unwrap(),
        out,
        "{path}：第二次往返变了"
    );
}

/// 4 份清单文件：反序列化成格式类型再序列化，语义等价（口径见 `norm`）。
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
