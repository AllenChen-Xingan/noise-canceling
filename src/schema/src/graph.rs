//! 处理图的节点种类 → `processing-graph.kinds.json`；会话记录头 → `session-header.fields.json`。
//!
//! 正本都在 `vdn-types`（核心 crate，不引 serde）：节点种类是 `vdn_types::graph::KINDS`，节点字段名是
//! `graph::NODE_FIELDS`（穷举解构）；会话头的键是 `vdn_types::session::HEADER_KEYS`（穷举解构）。这里只转成 JSON。

use serde_json::{Value, json};
use vdn_types::ModelId;
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

/// 规范文本的样例（由代码生成，不手写格式说明）：外部实现可拿它核对自己的写法与哈希。
pub const EXAMPLE: ProcessingGraph = ProcessingGraph {
    model: ModelId::Gtcrn,
    spectral: &[&graph::POLISH_SPECTRAL],
    stages: &[&graph::ASYM_GATE, &graph::POLISH_AGC],
};

fn example_json() -> Value {
    json!({
        "canonical_text": EXAMPLE.canonical_text(),
        "content_id": EXAMPLE.content_id().to_string(),
        "content_id_algorithm": "FNV-1a 64（vdn_types::ContentId::of），16 位小写十六进制",
    })
}

/// 节点种类登记表。
pub fn kinds_doc() -> Value {
    json!({
        "$comment": "由 `cargo run -p vdn-schema` 从 src/types/src/graph.rs 的 KINDS 导出，勿手改",
        "format": ProcessingGraph::FORMAT,
        "node_fields": graph::NODE_FIELDS,
        "test_id_min": graph::TEST_ID_MIN,
        "retired_ids": graph::RETIRED,
        "example": example_json(),
        "kinds": graph::KINDS.iter().map(|k| kind_json(k)).collect::<Vec<_>>(),
    })
}

/// 对照基线（main 上已提交的种类表）把关「编号永不复用」：
/// - 基线里的每个编号：还在登记表里 → 名字不变、版本不降；版本没升时内容（挂点 · 延迟 · 参数）必须原样
/// - 不在登记表里 → 必须在 `RETIRED`；基线已退役的编号仍须退役
/// - 新种类不许用基线里别的编号用过的名字
pub fn guard_kinds(old: &Value) -> Result<(), String> {
    let now = kinds_doc();
    let field = |k: &Value, f: &str| k.get(f).cloned().unwrap_or(Value::Null);
    let olds = old["kinds"].as_array().ok_or("基线种类表没有 kinds")?;
    for o in olds {
        let id = o["id"].as_u64().ok_or("基线种类缺编号")?;
        let name = field(o, "name");
        let cur = now["kinds"]
            .as_array()
            .and_then(|a| a.iter().find(|k| k["id"].as_u64() == Some(id)));
        let Some(cur) = cur else {
            if !graph::RETIRED.iter().any(|r| u64::from(*r) == id) {
                return Err(format!("种类 {id}（{name}）删了但没登记进 RETIRED"));
            }
            continue;
        };
        if field(cur, "name") != name {
            return Err(format!(
                "种类 {id} 的名字从 {name} 改成了 {}（编号与名字一一对应）",
                cur["name"]
            ));
        }
        let (ov, cv) = (o["version"].as_u64(), cur["version"].as_u64());
        if cv < ov {
            return Err(format!("种类 {id}（{name}）的版本倒退了"));
        }
        if cv == ov {
            for f in ["hook", "latency_samples", "params"] {
                if field(cur, f) != field(o, f) {
                    return Err(format!("种类 {id}（{name}）改了 {f} 却没升版本"));
                }
            }
        }
    }
    for r in old["retired_ids"].as_array().into_iter().flatten() {
        if !graph::RETIRED
            .iter()
            .any(|x| Some(u64::from(*x)) == r.as_u64())
        {
            return Err(format!("退役编号 {r} 不许撤回"));
        }
    }
    for k in graph::KINDS {
        let reused = olds
            .iter()
            .any(|o| o["name"] == k.name && o["id"].as_u64() != Some(u64::from(k.id)));
        if reused {
            return Err(format!("名字 {} 在基线里属于别的编号", k.name));
        }
    }
    Ok(())
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
