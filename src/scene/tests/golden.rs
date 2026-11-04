//! 场景特征对拍（013 第 8 节第 2 条）：自算入口 `push_hop` 在固定信号上的全部判断（35 维特征 · 后验 · 细场景）
//! 压成一个哈希，与改代码**之前**生成的基准逐位比。红了 = 场景数值行为变了（红线：不许变），先停下报。
//!
//! 基准在 `tests/fixtures/scene_golden.txt`；只在没改过 `features.rs` 的代码上生成：`VDN_BLESS=1 cargo test -p vdn-scene --test golden`。
//! 基准带平台标签（含系统主版本，如 `aarch64-macos-27`）：ln / exp / powf 末位随 macOS 大版本可能变，
//! 逐位对拍只在生成基准的那个平台上成立，别的平台（如 CI 机）打印原因后跳过。
mod common;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/scene_golden.txt"
);
const FIXTURE_RAGGED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/scene_ragged_golden.txt"
);

#[test]
fn self_compute_matches_golden() {
    let js = common::run(&common::signal(), |sa, h| sa.push_hop(h));
    let got = common::digest(&js);
    let Some((id, n)) = common::golden(FIXTURE, got, js.len()) else {
        return;
    };
    assert_eq!(js.len(), n, "判断次数变了");
    assert_eq!(got, id, "场景判断与改前不同（红线），先停下报");
}

/// 不整齐的输入长度（不足一个 hop、空、超过一个 hop）下，自算入口与改前逐位相同：改前按实际长度移位，不补零。
#[test]
fn ragged_hops_match_golden() {
    let js = common::run_ragged(&common::signal(), |sa, h| sa.push_hop(h));
    let got = common::digest(&js);
    let Some((id, n)) = common::golden(FIXTURE_RAGGED, got, js.len()) else {
        return;
    };
    assert_eq!(js.len(), n, "判断次数变了");
    assert_eq!(got, id, "不整齐输入下场景判断与改前不同（红线），先停下报");
}
