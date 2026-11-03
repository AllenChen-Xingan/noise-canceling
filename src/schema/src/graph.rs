//! 处理图的节点种类 → `processing-graph.kinds.json`；会话记录头 → `session-header.fields.json`。
//!
//! 正本都在 `vdn-types`（核心 crate，不引 serde）：节点种类是 `vdn_types::graph::KINDS`，节点字段名是
//! `graph::NODE_FIELDS`（穷举解构）；会话头的键是 `vdn_types::session::HEADER_KEYS`（穷举解构）。这里只转成 JSON。

use serde_json::{Value, json};
use vdn_types::graph::{self, NodeKind, ProcessingGraph};
use vdn_types::session::{HEADER_KEYS, SessionRecord};

pub const KINDS_FILE: &str = "processing-graph.kinds.json";
pub const SESSION_FILE: &str = "session-header.fields.json";

fn kind_json(k: &NodeKind) -> Value {
    json!({
        "id": k.id,
        "name": k.name,
        "version": k.version,
        "hook": k.hook.as_str(),
        "latency_samples": k.latency_samples,
        "params": k.params.iter().map(|p| json!({ "id": p.id, "key": p.key })).collect::<Vec<_>>(),
        "doc": k.doc,
    })
}

/// 节点种类登记表。
pub fn kinds_doc() -> Value {
    json!({
        "$comment": "由 `cargo run -p vdn-schema` 从 src/types/src/graph.rs 的 KINDS 导出，勿手改",
        "format": ProcessingGraph::FORMAT,
        "node_fields": graph::NODE_FIELDS,
        "test_id_min": graph::TEST_ID_MIN,
        "canonical_text": "vdn-graph = <format>\\nmodel = <模型 ID>\\n[spectral]\\nnode = …\\n[stages]\\nnode = …；\
                           node = <id> <name> <version> <hook> <latency_samples> <参数 id,…>；内容 ID = FNV-1a 64",
        "kinds": graph::KINDS.iter().map(|k| kind_json(k)).collect::<Vec<_>>(),
    })
}

/// 会话记录头的键。
pub fn session_doc() -> Value {
    json!({
        "$comment": "由 `cargo run -p vdn-schema` 从 src/types/src/session.rs 的 HEADER_KEYS 导出，勿手改",
        "header": SessionRecord::HEADER,
        "format": SessionRecord::FORMAT,
        "keys": HEADER_KEYS.iter().map(|k| json!({
            "field": k.field,
            "key": k.key,
            "value": k.value,
            "since": k.since,
            "optional": k.optional,
            "doc": k.doc,
        })).collect::<Vec<_>>(),
        "events_section": "events",
    })
}
