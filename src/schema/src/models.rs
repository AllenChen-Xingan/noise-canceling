//! 模型接口 → `models.interface.json`（正本：`vdn_models::ModelKind` 的 `declared()` 与 `interface()`）。
//!
//! 三方一致的「声明 ↔ schema」一半在这里（CI 逐字比对）；「ONNX 文件 ↔ 声明」一半在 `vdn-models` 的
//! `tests/interface.rs`（要仓外权重，人工跑）和适配器的 `open`（不一致打不开）。

use serde_json::{Value, json};
use vdn_models::interface::TensorDecl;
use vdn_models::{Declared, ModelKind};

pub const INTERFACE_FILE: &str = "models.interface.json";

fn tensors(ts: &[TensorDecl]) -> Value {
    ts.iter()
        .map(|t| {
            // 穷举解构：TensorDecl 加字段 → 这里编译失败，提醒导出
            let TensorDecl { name, shape } = *t;
            json!({ "name": name, "shape": shape, "dtype": "f32" })
        })
        .collect()
}

fn model(k: ModelKind) -> Value {
    let Declared {
        sample_rate,
        hop,
        latency_samples,
    } = k.declared();
    let io = k.interface();
    json!({
        "name": k.id().as_str(),
        "sample_rate": sample_rate,
        "hop": hop,
        "latency_samples": latency_samples,
        "inputs": tensors(io.inputs),
        "outputs": tensors(io.outputs),
    })
}

pub fn interface_doc() -> Value {
    json!({
        "$comment": "由 `cargo run -p vdn-schema` 从 src/models/src/lib.rs（ModelKind::declared）与 src/models/src/interface.rs 导出，勿手改。shape 里 -1 = 动态维；与 ONNX 文件的比对见 src/models/tests/interface.rs",
        "models": ModelKind::ALL.iter().map(|k| model(*k)).collect::<Vec<_>>(),
    })
}
