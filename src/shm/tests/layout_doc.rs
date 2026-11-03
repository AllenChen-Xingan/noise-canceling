//! 唯一真相源 + 交叉校验（人定，CLAUDE.md 第五节第 5 条）：头部布局的正本是 `src/layout.rs` 里的 Rust 结构。
//! 第二处表示有两份，都由本测试和正本比对：
//! 1. `layout.rs` 头注释里的字段清单（第 0 行 · 第 1 行 · 生产方行 · 消费方行）：字段名与顺序必须和结构一致
//! 2. `VERSION` 上方的版本历史：最后一条的编号必须等于 `VERSION`
//!
//! 偏移量与大小由 `src/desc.rs`（`offset_of!` 算）导出到 `specs/schema/shm-header.layout.json`，
//! 由 vdn-schema 的测试逐项比对；改布局不升 `VERSION` 也由那里判红。
#![cfg(not(feature = "loom"))] // loom 的原子量只能在 loom::model 里用；loom 模式只跑 tests/loom.rs

const SRC: &str = include_str!("../src/layout.rs");

/// `pub struct Name { pub a: T, pub b: T }` → [a, b]（按源码顺序）
fn struct_fields(name: &str) -> Vec<String> {
    let head = format!("pub struct {name} {{");
    let start = SRC
        .find(&head)
        .unwrap_or_else(|| panic!("layout.rs 里没有 {name}"))
        + head.len();
    let body = &SRC[start..start + SRC[start..].find('}').unwrap()];
    body.lines()
        .map(str::trim)
        .filter_map(|l| l.strip_prefix("pub "))
        .filter_map(|l| l.split(':').next())
        .map(|s| s.trim().to_string())
        .collect()
}

/// 头注释里 `marker` 之后、到 `end` 之前、用 ` · ` 分隔的字段名
fn doc_fields(marker: &str, end: char) -> Vec<String> {
    let line = SRC
        .lines()
        .take_while(|l| l.starts_with("//!"))
        .find(|l| l.contains(marker))
        .unwrap_or_else(|| panic!("头注释里没有「{marker}」"));
    let after = &line[line.find(marker).unwrap() + marker.len()..];
    after[..after.find(end).unwrap_or(after.len())]
        .split('·')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
fn doc_field_lists_match_structs() {
    assert_eq!(
        doc_fields("第 0 行：", '（'),
        struct_fields("ConfigLine"),
        "配置行"
    );
    assert_eq!(
        doc_fields("第 1 行：", ' '),
        struct_fields("BeatLine"),
        "心跳行"
    );
    assert_eq!(
        doc_fields("生产方一行（", '）'),
        struct_fields("ProdLine"),
        "生产方行"
    );
    assert_eq!(
        doc_fields("消费方一行（", '）'),
        struct_fields("ConsLine"),
        "消费方行"
    );
}

#[test]
fn version_history_ends_at_current_version() {
    let at = SRC.find("pub const VERSION: u32").unwrap();
    let last = SRC[..at]
        .lines()
        .rev()
        .take_while(|l| l.trim_start().starts_with("///"))
        .filter_map(|l| l.trim_start().strip_prefix("/// - "))
        .filter_map(|l| l.split('：').next()?.parse::<u32>().ok())
        .next()
        .expect("VERSION 上方没有版本历史");
    assert_eq!(
        last,
        vdn_shm::layout::VERSION,
        "版本历史最后一条 ≠ VERSION：升版本时两处一起改"
    );
}
