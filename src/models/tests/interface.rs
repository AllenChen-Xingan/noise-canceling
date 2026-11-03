//! 模型接口三方一致的「文件 ↔ 声明」一半：真 ONNX 文件实读的输入输出（名字 · 顺序 · 形状）必须等于
//! `ModelKind::interface()`，打开后的 hop · 延迟 · 采样率 · 名字必须等于 `ModelKind::declared()` 与正本名。
//! 「声明 ↔ schema」那一半在 `vdn-schema` 的 `tests/models.rs`（CI 里跑）。
//!
//! 人工步骤（权重在仓外，CI 没有）：
//! 1. 权重放到 `~/vdn-data/models/…`（路径约定见 `ModelKind::default_path`），或设 `VDN_GTCRN_ONNX` / `VDN_FASTENHANCER_ONNX`
//! 2. `VDN_REQUIRE_MODELS=1 cargo test -p vdn-models --test interface -- --ignored --nocapture`
//! 3. 全过即三方一致；红了看打印的「声明 / 文件」两列，改 `src/models/src/interface.rs` 后重跑 `cargo run -p vdn-schema`

use vdn_backend_ort as session;
use vdn_engine::{Engine, Pipeline, SafetyConfig};
use vdn_models::interface::{check_session, read_session};
use vdn_models::{ModelKind, artifact_id};
use vdn_types::Model;

fn file_matches_declaration(kind: ModelKind) {
    let Some(path) = kind.default_path() else {
        assert!(
            std::env::var_os("VDN_REQUIRE_MODELS").is_none(),
            "VDN_REQUIRE_MODELS 已设，但找不到 {:?} 的模型文件",
            kind
        );
        eprintln!("跳过：找不到 {kind:?} 的模型文件");
        return;
    };
    let s = session::open(&path).expect("打开会话");
    let (ins, outs) = read_session(&s);
    eprintln!(
        "{kind:?} {}\n  输入 {ins:?}\n  输出 {outs:?}",
        path.display()
    );
    check_session(&kind.interface(), &s).unwrap_or_else(|e| panic!("{kind:?}：{e}"));
    let m = kind.open(&path).expect("打开模型");
    let d = kind.declared();
    assert_eq!(
        (m.sample_rate(), m.hop(), m.latency_samples()),
        (d.sample_rate, d.hop, d.latency_samples)
    );
    assert_eq!(m.name(), kind.id().as_str());
    // 模型节点：真权重算出的制品内容 ID 进图；与正在跑的模型名比对通过（#68）
    let art = artifact_id(&path).expect("读权重");
    assert_eq!(
        art,
        artifact_id(&path).unwrap(),
        "同一文件两次算出的 ID 应相同"
    );
    // IO 采样率 = 模型采样率 × 重采样倍数（不另写数字）
    let io_rate = d.sample_rate * vdn_dsp::resample::FACTOR as u32;
    let p = Pipeline::new(Engine::new(m, SafetyConfig::off()), io_rate).expect("IO 采样率");
    let g = p
        .graph(kind.id(), art)
        .expect("模型节点应与 Model::name() 一致");
    eprintln!("{kind:?} 制品内容 ID {art} · 图 ID {}", g.content_id());
}

#[test]
#[ignore = "要仓外权重，人工跑：见文件头"]
fn gtcrn_file_matches_declaration() {
    file_matches_declaration(ModelKind::Gtcrn);
}

#[test]
#[ignore = "要仓外权重，人工跑：见文件头"]
fn fastenhancer_m_file_matches_declaration() {
    file_matches_declaration(ModelKind::FastEnhancerM);
}

/// 制品内容 ID = 文件字节的哈希：内容相同 → 相同，差一个字节 → 不同；文件不存在 → 报错。
#[test]
fn artifact_id_follows_file_bytes() {
    let d = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("artifact-id");
    std::fs::create_dir_all(&d).unwrap();
    let (a, b, c) = (d.join("a.onnx"), d.join("b.onnx"), d.join("c.onnx"));
    std::fs::write(&a, b"weights-v1").unwrap();
    std::fs::write(&b, b"weights-v1").unwrap();
    std::fs::write(&c, b"weights-v2").unwrap();
    assert_eq!(artifact_id(&a).unwrap(), artifact_id(&b).unwrap());
    assert_ne!(artifact_id(&a).unwrap(), artifact_id(&c).unwrap());
    assert_eq!(
        artifact_id(&a).unwrap(),
        vdn_types::ContentId::of(b"weights-v1")
    );
    assert!(artifact_id(&d.join("missing.onnx")).is_err());
}
