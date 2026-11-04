//! 场景特征对拍（013 第 8 节第 2 条）：自算入口 `push_hop` 在固定信号上的全部判断（35 维特征 · 后验 · 细场景）
//! 压成一个哈希，与改代码**之前**生成的基准逐位比。红了 = 场景数值行为变了（红线：不许变），先停下报。
//!
//! 基准在 `tests/fixtures/scene_golden.txt`；只在没改过 `features.rs` 的代码上生成：`VDN_BLESS=1 cargo test -p vdn-scene --test golden`。
mod common;

use vdn_types::ContentId;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/scene_golden.txt");

#[test]
fn self_compute_matches_golden() {
    let js = common::run(&common::signal(), |sa, h| sa.push_hop(h));
    let got = common::digest(&js);
    let line = format!("{got} {}", js.len());
    if std::env::var_os("VDN_BLESS").is_some() {
        std::fs::write(FIXTURE, format!("{line}\n")).expect("写基准");
        eprintln!("已写基准：{line}");
        return;
    }
    let want = std::fs::read_to_string(FIXTURE).expect("缺基准文件（见文件头怎么生成）");
    let mut it = want.split_whitespace();
    let id = it.next().and_then(ContentId::parse).expect("基准第 1 列是 16 位十六进制");
    let n: usize = it.next().and_then(|s| s.parse().ok()).expect("基准第 2 列是判断次数");
    assert_eq!(js.len(), n, "判断次数变了");
    assert_eq!(got, id, "场景判断与改前不同（红线），先停下报");
}
