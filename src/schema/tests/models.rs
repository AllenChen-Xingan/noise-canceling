//! 交叉校验 · 模型接口：`models.interface.json` ↔ `ModelKind` 的声明（CI 里跑）。
//! 「ONNX 文件 ↔ 声明」那一半要仓外权重，在 `src/models/tests/interface.rs`（`#[ignore]`，人工步骤见文件头）。

use serde_json::{Value, json};
use vdn_models::{ModelId, ModelKind};
use vdn_schema::{DIR, check, models};

mod common;
use common::*;

/// 每个模型的名字 · 采样率 · hop · 延迟 · 输入输出与声明相同；名字是正本名，顺序同 `ModelId::ALL`。
#[test]
fn interface_file_matches_declarations() {
    let doc = committed(models::INTERFACE_FILE);
    let ms = doc["models"].as_array().unwrap();
    assert_eq!(ms.len(), ModelKind::ALL.len());
    for ((m, k), id) in ms.iter().zip(ModelKind::ALL).zip(ModelId::ALL) {
        assert_eq!(k.id(), id);
        assert_eq!(m["name"], json!(id.as_str()));
        let d = k.declared();
        assert_eq!(m["sample_rate"], json!(d.sample_rate));
        assert_eq!(m["hop"], json!(d.hop));
        assert_eq!(m["latency_samples"], json!(d.latency_samples));
        let io = k.interface();
        for (side, decl) in [("inputs", io.inputs), ("outputs", io.outputs)] {
            let got = m[side].as_array().unwrap();
            assert_eq!(got.len(), decl.len(), "{k:?} {side}");
            for (g, t) in got.iter().zip(decl) {
                assert_eq!(g["name"], json!(t.name));
                assert_eq!(g["shape"], json!(t.shape));
            }
        }
    }
}

/// 负例：声明改了（模拟 FE-M 的 hop 变了）而文件没重新导出 → 检查报 models.interface.json。
#[test]
fn interface_change_without_export_is_red() {
    let d = temp_root("models");
    let mut v: Value = serde_json::from_str(
        &std::fs::read_to_string(d.join(DIR).join(models::INTERFACE_FILE)).unwrap(),
    )
    .unwrap();
    v["models"][1]["hop"] = json!(128);
    put(&d, models::INTERFACE_FILE, &v);
    let p = check(&d);
    assert!(
        p.len() == 1 && p[0].contains(models::INTERFACE_FILE),
        "{p:#?}"
    );
}
