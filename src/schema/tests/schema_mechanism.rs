//! 交叉校验 · 机制（CLAUDE.md 第五节第 5 条）：仓里的 schema 文件与 Rust 正本逐字一致；旧文件、多余文件都报。
//! 文件过 schema 用 `boon`（另一家实现）校验，不用导出 schema 的同一个库。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vdn_control::Config;
use vdn_schema::{DIR, check, eval, export, root, shm, write};

mod common;
use common::*;

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
