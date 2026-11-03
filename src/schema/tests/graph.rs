//! 处理图节点种类与会话记录头：仓里的导出文件 ↔ `vdn-types` 正本 ↔ 实际文本格式，三方比对（D72）。
//! 「导出物与仓里逐字一致」由 `crosscheck.rs` 的通用测试管；这里查文件**内容**和正本的行为对得上。

use serde_json::Value;
use vdn_schema::graph::{EXAMPLE, KINDS_FILE, SESSION_FILE, kinds_doc, session_doc};
use vdn_schema::{DIR, pretty, root};
use vdn_types::graph::{self, KINDS, NODE_FIELDS};
use vdn_types::session::{HEADER_KEYS, SessionHeader, SessionRecord};
use vdn_types::{ContentId, ModelId, ProcessingGraph};

fn committed(file: &str) -> Value {
    let t = std::fs::read_to_string(root().join(DIR).join(file)).unwrap();
    serde_json::from_str(&t).unwrap()
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    v[key].as_array().unwrap()
}

/// 仓里的种类表逐条等于 `KINDS`：编号 · 名字 · 版本 · 挂点 · 延迟 · 参数（编号与键名都要等于 desc 正本）。
#[test]
fn committed_kinds_match_registry() {
    let doc = committed(KINDS_FILE);
    assert_eq!(doc["format"], ProcessingGraph::FORMAT);
    assert_eq!(doc["test_id_min"], graph::TEST_ID_MIN);
    let fields: Vec<&str> = arr(&doc, "node_fields")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(fields, NODE_FIELDS);
    let kinds = arr(&doc, "kinds");
    assert_eq!(kinds.len(), KINDS.len());
    for (v, k) in kinds.iter().zip(KINDS) {
        // 每个节点对象的键 = NODE_FIELDS（穷举解构的那张表）
        let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut want = NODE_FIELDS.to_vec();
        want.sort_unstable();
        assert_eq!(keys, want, "{}", k.name);
        assert_eq!(v["id"], k.id);
        assert_eq!(v["name"], k.name);
        assert_eq!(v["version"], k.version);
        assert_eq!(v["hook"], k.hook.as_str());
        assert_eq!(v["latency_samples"], k.latency_samples);
        let params = arr(v, "params");
        assert_eq!(params.len(), k.params.len(), "{}", k.name);
        for (p, d) in params.iter().zip(k.params) {
            let id = u16::try_from(p["id"].as_u64().unwrap()).unwrap();
            let desc = vdn_types::desc::by_id(id).unwrap();
            assert_eq!(desc.id, d.id);
            assert_eq!(p["key"], desc.key, "{} 的参数 {id}", k.name);
        }
    }
}

/// 样例的内容 ID 只靠文件里的规范文本就能复算（外部实现照这个核对哈希）。
#[test]
fn committed_example_hash_recomputes() {
    let ex = &committed(KINDS_FILE)["example"];
    let text = ex["canonical_text"].as_str().unwrap();
    assert_eq!(text, EXAMPLE.canonical_text());
    let id = ContentId::of(text.as_bytes()).to_string();
    assert_eq!(ex["content_id"], id.as_str());
    assert_eq!(id, EXAMPLE.content_id().to_string());
}

/// 负例：改了种类的版本（或参数）不重新导出 → 导出物与仓里不同（检查会红）。
#[test]
fn kind_change_without_export_is_red() {
    let committed_text = std::fs::read_to_string(root().join(DIR).join(KINDS_FILE)).unwrap();
    assert_eq!(
        pretty(&kinds_doc()),
        committed_text,
        "先跑 `cargo run -p vdn-schema`"
    );
    let mut v = kinds_doc();
    v["kinds"][0]["version"] = Value::from(KINDS[0].version + 1);
    assert_ne!(pretty(&v), committed_text);
}

/// 仓里的会话头键表 = `HEADER_KEYS`，而且与 `to_text` 实际写出的键逐个一致。
#[test]
fn committed_session_keys_match_text_format() {
    let doc = committed(SESSION_FILE);
    assert_eq!(doc["header"], SessionRecord::HEADER);
    assert_eq!(doc["format"], SessionRecord::FORMAT);
    let keys = arr(&doc, "keys");
    assert_eq!(keys.len(), HEADER_KEYS.len());
    for (v, k) in keys.iter().zip(HEADER_KEYS) {
        assert_eq!(v["key"], k.key);
        assert_eq!(v["field"], k.field);
        assert_eq!(v["since"], k.since);
        assert_eq!(v["optional"], k.optional);
    }
    // 全字段的头写出来，键的顺序就是文件里的顺序
    let mut h = SessionHeader {
        rate: 48_000,
        ..SessionHeader::default()
    };
    h.set_graph(&ProcessingGraph::bare(ModelId::Gtcrn));
    let text = SessionRecord {
        header: h,
        events: Vec::new(),
    }
    .to_text();
    let written: Vec<&str> = text
        .lines()
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter_map(|l| l.split(" = ").next())
        .collect();
    let file_keys: Vec<&str> = keys.iter().map(|v| v["key"].as_str().unwrap()).collect();
    assert_eq!(written, file_keys);
}

/// 文件里写的 `since` 与解析器的行为一致：在 `since` 格式里能读，在更早的格式里拒读。
#[test]
fn session_key_since_agrees_with_parser() {
    let doc = committed(SESSION_FILE);
    let full = {
        let mut h = SessionHeader::default();
        h.set_graph(&ProcessingGraph::bare(ModelId::Gtcrn));
        SessionRecord {
            header: h,
            events: Vec::new(),
        }
        .to_text()
    };
    for v in arr(&doc, "keys") {
        let key = v["key"].as_str().unwrap();
        let since = v["since"].as_u64().unwrap();
        let line = full
            .lines()
            .find(|l| l.starts_with(&format!("{key} = ")))
            .unwrap();
        let at = |f: u64| format!("{} = {f}\n{line}\n[events]\n", SessionRecord::HEADER);
        assert!(
            SessionRecord::from_text(&at(since)).is_ok(),
            "{key} 在格式 {since}"
        );
        if since > 1 {
            assert!(
                SessionRecord::from_text(&at(since - 1)).is_err(),
                "{key} 在格式 {}",
                since - 1
            );
        }
    }
}

/// 负例：会话头加了键不重新导出 → 导出物与仓里不同。
#[test]
fn session_change_without_export_is_red() {
    let committed_text = std::fs::read_to_string(root().join(DIR).join(SESSION_FILE)).unwrap();
    assert_eq!(
        pretty(&session_doc()),
        committed_text,
        "先跑 `cargo run -p vdn-schema`"
    );
    let mut v = session_doc();
    v["format"] = Value::from(SessionRecord::FORMAT + 1);
    assert_ne!(pretty(&v), committed_text);
}
