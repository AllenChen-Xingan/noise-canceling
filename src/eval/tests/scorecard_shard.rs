//! 分片跑成绩单（大总管 2026-11-25 定 `--shard i/n`）：分 n 片各跑一部分再合并，结果必须和单进程逐字节相同。
//! 比的是：逐段分数 `clips.jsonl` 全文、成绩单 JSON（meta 用同一份）全文、成绩单 Markdown 全文。

use serde_json::{Value, json};
use vdn_eval::scorecard::card::{Card, Rules};
use vdn_eval::scorecard::load::{parse_jsonl, to_jsonl};
use vdn_eval::scorecard::run_shard;
use vdn_eval::scorecard::shard::{Part, Shard, asr_workers, merge, merge_dirs, suite_fingerprint};
use vdn_eval::suite::{Suite, parse_suite};
use vdn_eval::system::SystemKind;
use vdn_eval::wav;

#[path = "common/scene_fixture.rs"]
mod fx;
use fx::{opts, setup};

const SYS: [SystemKind; 1] = [SystemKind::Ref];

fn systems() -> Vec<String> {
    SYS.iter().map(|s| s.as_str().to_string()).collect()
}

fn part(suite: &Suite, root: &std::path::Path, i: usize, n: usize) -> Part {
    let r = run_shard(
        suite,
        &SYS,
        &opts(),
        None,
        Some(root),
        Rules::V2,
        Some(Shard { i, n }),
    );
    Part {
        shard: Shard { i, n },
        jsonl: to_jsonl(&r.rows).unwrap(),
        missing: r.missing,
        row_units: r.row_units,
        missing_units: r.missing_units,
    }
}

/// 单进程一次、分 n 片再合并一次，三样产物逐字节对照。
fn assert_same_as_single(suite: &Suite, root: &std::path::Path, n: usize) {
    let single = run_shard(suite, &SYS, &opts(), None, Some(root), Rules::V2, None);
    assert!(!single.rows.is_empty());
    let mut parts: Vec<Part> = (0..n).map(|i| part(suite, root, i, n)).collect();
    assert!(
        n == 1 || parts.iter().all(|p| p.row_units.len() < single.rows.len()),
        "每片只跑一部分"
    );
    parts.reverse(); // 合并时片的顺序不该有影响
    let (jsonl, missing) = merge(suite, &systems(), &parts).unwrap();
    assert_eq!(
        jsonl,
        to_jsonl(&single.rows).unwrap(),
        "clips.jsonl 逐字节相同"
    );
    let meta = json!({"t": 1});
    let a = Card::build_with(&single.rows, single.missing.clone(), Rules::V2);
    let b = Card::build_with(&parse_jsonl(&jsonl).unwrap(), missing, Rules::V2);
    let ja = serde_json::to_string_pretty(&a.to_json(meta.clone())).unwrap();
    let jb = serde_json::to_string_pretty(&b.to_json(meta)).unwrap();
    assert_eq!(ja, jb, "成绩单 JSON 逐字节相同");
    assert_eq!(
        a.to_markdown("t"),
        b.to_markdown("t"),
        "成绩单 Markdown 逐字节相同"
    );
}

fn fixture(tag: &str) -> (std::path::PathBuf, Suite) {
    let (root, text) = setup(tag);
    (root, parse_suite(&text).unwrap())
}

#[test]
fn three_shards_merge_byte_identical() {
    let (root, suite) = fixture("shard3");
    assert_same_as_single(&suite, &root, 3);
}

#[test]
fn one_shard_is_the_whole_run() {
    let (root, suite) = fixture("shard1");
    assert_same_as_single(&suite, &root, 1);
}

#[test]
fn shards_with_a_missing_clip_merge_byte_identical() {
    let (root, suite) = fixture("shard-miss");
    // 一段混合轨被改：这一段进「没跑成的」，看缺段记录合并后也一样
    let p = root.join("clips/cafe-c03-mix.wav");
    let (mut x, sr) = wav::read(&p).unwrap();
    x[100] += 0.1;
    wav::write(&p, &x, sr).unwrap();
    assert_same_as_single(&suite, &root, 4);
}

/// codex 本分支第 1 轮 P1 #3：同一个片段 id 在清单里出现两次（两条任务指向同一批片段，类比两个 build 的同名段），
/// 单进程会出两份；合并按单位位置排，不能当成重复拒掉，结果也要逐字节相同。
#[test]
fn same_clip_id_in_two_tasks_merges_byte_identical() {
    let (root, text) = setup("shard-dup-id");
    let mut v: Value = serde_json::from_str(&text).unwrap();
    let mut t2 = v["tasks"][0].clone();
    t2["id"] = json!(format!("{}-again", t2["id"].as_str().unwrap()));
    v["tasks"].as_array_mut().unwrap().push(t2);
    let suite = parse_suite(&v.to_string()).unwrap();
    assert_same_as_single(&suite, &root, 3);
}

/// 拒绝：片不齐、同一单位同一系统重复、少一条分数（截掉一行）、分数和缺段记录同时有、单位不归这一片。
#[test]
fn incomplete_or_inconsistent_parts_are_refused() {
    let (root, suite) = fixture("shard-bad");
    let (p0, p1) = (part(&suite, &root, 0, 2), part(&suite, &root, 1, 2));
    let sys = systems();
    assert!(
        merge(&suite, &sys, &[p0.clone()])
            .unwrap_err()
            .contains("不齐")
    );
    assert!(
        merge(&suite, &sys, &[p0.clone(), p0.clone(), p1.clone()])
            .unwrap_err()
            .contains("不齐")
    );
    // 截掉一行
    let mut cut = p1.clone();
    let lines: Vec<&str> = cut.jsonl.lines().collect();
    cut.jsonl = lines[..lines.len() - 1]
        .iter()
        .map(|l| format!("{l}\n"))
        .collect();
    cut.row_units.pop();
    assert!(
        merge(&suite, &sys, &[p0.clone(), cut])
            .unwrap_err()
            .contains("少了")
    );
    // 同一单位既有分数又有缺段记录
    let mut both = p1.clone();
    let m = parse_jsonl(&both.jsonl).unwrap()[0].clone();
    both.missing.push(vdn_eval::scorecard::Missing {
        scene: m.scene,
        tier: m.tier,
        device: m.device.clone(),
        system: m.system.clone(),
        what: m.clip.clone(),
        why: "x".into(),
    });
    both.missing_units.push(both.row_units[0]);
    assert!(
        merge(&suite, &sys, &[p0.clone(), both])
            .unwrap_err()
            .contains("两条")
    );
    // 单位不归这一片
    let mut wrong = p1.clone();
    wrong.row_units[0] = p0.row_units[0];
    assert!(
        merge(&suite, &sys, &[p0, wrong])
            .unwrap_err()
            .contains("不归")
    );
}

/// 按目录合并：各片 meta 要一致、清单指纹要和合并用的清单相同；计时相加。
#[test]
fn merge_dirs_checks_meta_fingerprint_and_sums_timing() {
    let (root, suite) = fixture("shard-dirs");
    let single = run_shard(&suite, &SYS, &opts(), None, Some(&root), Rules::V2, None);
    let fp = suite_fingerprint(&suite);
    let write = |i: usize, n: usize, commit: &str, fp: &str| {
        let p = part(&suite, &root, i, n);
        let d = root.join(format!("out-{commit}-{fp}-{i}of{n}"));
        std::fs::create_dir_all(&d).unwrap();
        let meta = json!({"commit": commit, "suite": "s.json", "suite_fingerprint": fp, "systems": systems(),
            "qa": false, "safety": "task", "rules": "v2", "external": [], "shard": format!("{i}/{n}"),
            "shard_units": {"rows": p.row_units, "missing": p.missing_units}, "timing": {"system": 1.5}, "asr": null});
        let card = Card::build_with(
            &parse_jsonl(&p.jsonl).unwrap(),
            p.missing.clone(),
            Rules::V2,
        );
        std::fs::write(d.join("scorecard.json"), card.to_json(meta).to_string()).unwrap();
        std::fs::write(d.join("clips.jsonl"), &p.jsonl).unwrap();
        d
    };
    let dirs: Vec<_> = (0..3).map(|i| write(i, 3, "abc", &fp)).collect();
    let (text, missing, meta) = merge_dirs(&suite, &dirs).unwrap();
    assert_eq!(text, to_jsonl(&single.rows).unwrap());
    assert!(missing.is_empty());
    assert_eq!(
        (meta["shards"].as_u64(), meta["timing"]["system"].as_f64()),
        (Some(3), Some(4.5))
    );
    assert!(merge_dirs(&suite, &dirs[..2]).unwrap_err().contains("不齐"));
    let other = write(2, 3, "zzz", &fp);
    assert!(
        merge_dirs(&suite, &[dirs[0].clone(), dirs[1].clone(), other])
            .unwrap_err()
            .contains("commit")
    );
    let stale: Vec<_> = (0..3)
        .map(|i| write(i, 3, "abc", "0000000000000000"))
        .collect();
    assert!(merge_dirs(&suite, &stale).unwrap_err().contains("指纹"));
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
