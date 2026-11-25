//! 分片跑成绩单（大总管 2026-11-25 定 `--shard i/n`）：分 n 片各跑一部分再合并，结果必须和单进程逐字节相同。
//! 比的是：逐段分数 `clips.jsonl` 全文、成绩单 JSON（meta 用同一份）全文、成绩单 Markdown 全文。

use serde_json::json;
use vdn_eval::scorecard::card::{Card, Rules};
use vdn_eval::scorecard::load::{parse_jsonl, to_jsonl};
use vdn_eval::scorecard::shard::{Shard, asr_workers, merge};
use vdn_eval::scorecard::{Missing, run_shard};
use vdn_eval::suite::parse_suite;
use vdn_eval::system::SystemKind;
use vdn_eval::wav;

#[path = "common/scene_fixture.rs"]
mod fx;
use fx::{opts, setup};

/// 单进程一次、分 n 片再合并一次，三样产物逐字节对照。
fn assert_same_as_single(tag: &str, n: usize, break_one: bool) {
    let (root, suite_text) = setup(tag);
    if break_one {
        // 一段混合轨被改：这一段进「没跑成的」，看缺段记录合并后也一样
        let p = root.join("clips/cafe-c03-mix.wav");
        let (mut x, sr) = wav::read(&p).unwrap();
        x[100] += 0.1;
        wav::write(&p, &x, sr).unwrap();
    }
    let suite = parse_suite(&suite_text).unwrap();
    let sys = [SystemKind::Ref];
    let single = run_shard(&suite, &sys, &opts(), None, Some(&root), Rules::V2, None);
    assert!(!single.rows.is_empty());
    let mut parts: Vec<(String, Vec<Missing>)> = Vec::new();
    for i in 0..n {
        let r = run_shard(&suite, &sys, &opts(), None, Some(&root), Rules::V2, Some(Shard { i, n }));
        assert!(r.rows.len() < single.rows.len() || n == 1, "每片只跑一部分");
        parts.push((to_jsonl(&r.rows).unwrap(), r.missing));
    }
    // 合并时片的顺序不该有影响
    parts.reverse();
    let (jsonl, missing) = merge(&suite, &parts).unwrap();
    assert_eq!(jsonl, to_jsonl(&single.rows).unwrap(), "clips.jsonl 逐字节相同");
    let meta = json!({"t": 1});
    let a = Card::build_with(&single.rows, single.missing.clone(), Rules::V2);
    let b = Card::build_with(&parse_jsonl(&jsonl).unwrap(), missing, Rules::V2);
    let ja = serde_json::to_string_pretty(&a.to_json(meta.clone())).unwrap();
    let jb = serde_json::to_string_pretty(&b.to_json(meta)).unwrap();
    assert_eq!(ja, jb, "成绩单 JSON 逐字节相同");
    assert_eq!(a.to_markdown("t"), b.to_markdown("t"), "成绩单 Markdown 逐字节相同");
    if break_one {
        assert_eq!(a.cards[0].missing.len(), 1);
    }
}

#[test]
fn three_shards_merge_byte_identical() {
    assert_same_as_single("shard3", 3, false);
}

#[test]
fn shards_with_a_missing_clip_merge_byte_identical() {
    assert_same_as_single("shard-miss", 4, true);
}

#[test]
fn one_shard_is_the_whole_run() {
    assert_same_as_single("shard1", 1, false);
}

/// 同一段出现在两片里、或少了一片，都拒绝合并（不悄悄出一份残缺或重复的成绩单）。
#[test]
fn duplicate_or_missing_part_is_refused() {
    let (root, suite_text) = setup("shard-bad");
    let suite = parse_suite(&suite_text).unwrap();
    let sys = [SystemKind::Ref];
    let part = |i, n| {
        let r = run_shard(&suite, &sys, &opts(), None, Some(&root), Rules::V2, Some(Shard { i, n }));
        (to_jsonl(&r.rows).unwrap(), r.missing)
    };
    let (p0, p1) = (part(0, 2), part(1, 2));
    assert!(merge(&suite, &[p0.clone(), p0.clone(), p1.clone()]).unwrap_err().contains("重复"));
    assert!(merge(&suite, &[p0]).unwrap_err().contains("少"));
}

#[test]
fn shard_parse_and_asr_workers() {
    assert_eq!(Shard::parse("0/3").unwrap(), Shard { i: 0, n: 3 });
    for bad in ["3/3", "1/0", "a/2", "1", "-1/2"] {
        assert!(Shard::parse(bad).is_err(), "{bad}");
    }
    // 每片识别并发：8 ÷ n 向上取整，不低于 1（大总管 2026-11-25）
    assert_eq!([1, 2, 3, 4, 8, 16].map(asr_workers), [8, 4, 3, 2, 1, 1]);
}
