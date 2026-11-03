//! 交叉校验：apple-rs 的 bench JSON 字段 ⟷ rustbe 的 bench（CLAUDE.md 第五节第 5 条：第二处表示必须机器比对）。
//!
//! 唯一真相源 = rustbe 的 `../bench/src/main.rs`（同一个仓库里的相对路径）里 `gtcrn-bench` / `fem-bench` 那条输出。
//! 规则：
//! - rustbe 有、apple-rs 没有 → 失败（缺）
//! - apple-rs 有、rustbe 没有、又不在下面的「苹果侧专有」清单里 → 失败（多）。清单里每个字段都写了为什么只有这边有
//! - 真相源文件不存在或读不了 → **失败**（不许没比对也显示通过）；`EXP010_BENCH_MAIN` 可指到别的路径
//! 长远：两边合成一个共用 crate，就不用再比（已转交大总管）。
use std::collections::BTreeSet;
use std::path::PathBuf;

/// 只有苹果侧有的字段，以及为什么。
const APPLE_ONLY: &[(&str, &str)] = &[
    ("artifact", "被测的模型文件名（mlpackage / mlmodelc）"),
    ("warmup", "预热帧数（rustbe 固定 200，这边可调）"),
    ("thread_policy", "tc / none，区分 --qos rt 与 --thread-policy"),
    ("spin", "帧间忙等对照"),
    ("tc_computation_ms", "time-constraint 参数可调"),
    ("tc_constraint_ms", "time-constraint 参数可调"),
    ("interval_wg_deadline_ms", "自建工作组区间"),
    ("interval_wg_errors", "自建工作组区间"),
    ("compile_ms", "Core ML / BNNS 的编译单列（rustbe 对应的是 ort_env_ms）"),
    ("warmup_ms", "BE-COLD：冷第 0 帧 + 预热"),
    ("first_warm_frame_ms", "BE-COLD：预热后第 1 帧"),
    ("win_start_unix", "耗电只平均计时窗内样本（power_parse.py v2）"),
    ("win_end_unix", "同上"),
    ("wall_s", "同上，实测墙钟"),
    ("parity_frames", "苹果侧在同一次运行里对 ORT 转储对拍（rustbe 有单独的 parity 子命令）"),
    ("parity_max_abs", "同上"),
    ("parity_worst_frame", "同上"),
    ("parity_snr_db", "同上"),
    ("parity_min_frame_snr_db", "同上"),
    ("parity_nonfinite_values", "同上"),
    ("parity_first_bad", "同上"),
    ("extra", "后端自带的附加信息（BNNS 工作区大小等）"),
];

/// 从源码里取 JSON 键：所有形如 `\"key\":` 的片段。
fn keys(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let pat = "\\\"";
    let mut rest = src;
    while let Some(i) = rest.find(pat) {
        let after = &rest[i + pat.len()..];
        if let Some(j) = after.find(pat) {
            let k = &after[..j];
            if after[j + pat.len()..].starts_with(':') && !k.is_empty() && k.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                out.insert(k.to_string());
            }
            rest = &after[j..];
        } else {
            break;
        }
    }
    out
}

/// 取某一条 println 的格式串：从含 `anchor` 的那一行往前到 `println!(`，往后到不以 `\` 续行的那一行。
fn println_block(src: &str, anchor: &str) -> Option<String> {
    let lines: Vec<&str> = src.lines().collect();
    let at = lines.iter().position(|l| l.contains(anchor))?;
    let mut start = at;
    while start > 0 && !lines[start].contains("println!(") && !lines[start].contains("format!(") {
        start -= 1;
    }
    let mut end = at;
    while end + 1 < lines.len() && lines[end].trim_end().ends_with('\\') {
        end += 1;
    }
    Some(lines[start..=end].join("\n"))
}

#[test]
fn bench_fields_match_rustbe() {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let truth = std::env::var("EXP010_BENCH_MAIN").map(PathBuf::from).unwrap_or_else(|_| here.join("../bench/src/main.rs"));
    // 读不到真相源就失败，不许「没比对也显示通过」（codex 第 6 轮 #4；CLAUDE.md 第五节第 5 条：无检查不许合并）
    let rb = std::fs::read_to_string(&truth).unwrap_or_else(|e| {
        panic!("读不到真相源 {}：{e}。还没 merge main？或用 EXP010_BENCH_MAIN 指到 rustbe 的 bench/src/main.rs", truth.display())
    });
    // rustbe：bench 那条输出 + 按节奏时拼进去的 pace_extra
    let mut want = BTreeSet::new();
    for anchor in ["\\\"backend\\\":\\\"{kind}\\\"", "\\\"open_to_warm_ms\\\""] {
        let b = println_block(&rb, anchor).unwrap_or_else(|| panic!("真相源里找不到锚点 {anchor}，rustbe 的输出格式变了，请更新本测试"));
        want.extend(keys(&b));
    }
    let me = std::fs::read_to_string(here.join("src/main.rs")).unwrap();
    let mine = keys(&println_block(&me, "\\\"backend\\\":\\\"{backend}\\\"").expect("apple-rs 的 bench 输出找不到"));
    let allowed: BTreeSet<String> = APPLE_ONLY.iter().map(|(k, _)| k.to_string()).collect();

    let missing: Vec<_> = want.difference(&mine).collect();
    let extra: Vec<_> = mine.difference(&want).filter(|k| !allowed.contains(*k)).collect();
    let stale: Vec<_> = allowed.iter().filter(|k| want.contains(*k) || !mine.contains(*k)).collect();
    assert!(missing.is_empty(), "apple-rs 缺 rustbe 的字段：{missing:?}（真相源 {}）", truth.display());
    assert!(extra.is_empty(), "apple-rs 多出没登记的字段：{extra:?}——要么删，要么写进 APPLE_ONLY 并说明为什么");
    assert!(stale.is_empty(), "APPLE_ONLY 里有过时条目（rustbe 已有，或 apple-rs 已不输出）：{stale:?}");
}

#[test]
fn key_extractor_works() {
    let s = r#"println!("{{\"a_1\":{x},\"b\":\"{y}\"}}");"#;
    let k = keys(s);
    assert_eq!(k.into_iter().collect::<Vec<_>>(), vec!["a_1".to_string(), "b".to_string()]);
}
