//! 成绩单端到端：自造 6 段合成片段（写 wav + clips.jsonl，带声音标注）→ 场景任务清单 → `scorecard::run`（参考降噪器）
//! → 汇总 → Markdown / JSON。不需要模型：QA 层的吃字与听感判「不知道」，所以场景结论只能是「待测」或「不过」。
//! 夹具在 `common/scene_fixture.rs`；早期回声参考的端到端测试在 `scorecard_early_e2e.rs`。

use std::path::Path;

use serde_json::json;
use vdn_eval::scene::SceneClip;
use vdn_eval::scorecard::card::Card;
use vdn_eval::scorecard::rules::CardStatus;
use vdn_eval::suite::parse_suite;
use vdn_eval::system::SystemKind;
use vdn_eval::wav;

#[path = "common/scene_fixture.rs"]
mod fx;
use fx::{SR, opts, setup};

#[test]
fn fixture_lines_parse_as_scene_clips() {
    let (root, _) = setup("parse");
    let t = std::fs::read_to_string(root.join("clips.jsonl")).unwrap();
    let v = SceneClip::parse_jsonl(&t).unwrap();
    assert_eq!(v.len(), 6);
    // 负例：拼错键 · 未知场景键 · 区间越界 · stems 与 sounds 对不上
    let line: Value = serde_json::from_str(t.lines().next().unwrap()).unwrap();
    let bad = |f: &dyn Fn(&mut Value)| {
        let mut x = line.clone();
        f(&mut x);
        SceneClip::parse_jsonl(&x.to_string()).is_err()
    };
    assert!(bad(&|x| x["scnee"] = json!("cafe")));
    assert!(bad(&|x| x["scene"] = json!("beach")));
    assert!(bad(&|x| x["owner_iv"] = json!([[0, 10_000_000]])));
    assert!(bad(&|x| x["sounds"].as_array_mut().unwrap().clear()));
    assert!(bad(&|x| x["device"] = json!("mba")));
}

#[test]
fn scorecard_end_to_end_on_ref_system() {
    let (root, suite_text) = setup("e2e");
    let suite = parse_suite(&suite_text).unwrap();
    let (rows, skipped) =
        vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, Some(&root));
    assert!(skipped.is_empty(), "{skipped:?}");
    assert_eq!(rows.len(), 6);
    let card = Card::build(&rows, skipped);
    assert_eq!(card.cards.len(), 1, "一个 场景 × 档 × 设备 × 系统");
    let c = &card.cards[0];
    assert_eq!(c.clips, 6);
    // 否决项排在最前；吃字没接 ASR → 待测；听感没接模型 → 待测
    assert!(c.items[0].veto && c.items[3].v == "V-EAT-ASR");
    assert_eq!(c.items[3].status, CardStatus::Unknown);
    assert_eq!(c.listen.status, Some(CardStatus::Unknown));
    assert_ne!(c.status, CardStatus::Pass, "缺 QA 项时不许判「过」");
    // 每一项都有数（DSP 项）或明确判不了
    for i in &c.items {
        assert_eq!(i.pass + i.fail + i.unknown, 6, "{}", i.v);
    }
    let md = card.to_markdown("测试");
    assert!(md.contains("cafe · common · mba-builtin · ref"));
    assert!(md.contains("V-ESTOI") && md.contains("听感"));
    let j = card.to_json(json!({"t": 1}));
    assert_eq!(j["scenes"][0]["clips"], 6);
    assert_eq!(j["clips_scored"], 6);
}

#[test]
fn hash_mismatch_is_skipped_not_scored() {
    let (root, suite_text) = setup("hash");
    // 改掉一段的混合轨：哈希对不上 → 这一段不打分，进「没跑的」
    let p = root.join("clips/cafe-c00-mix.wav");
    let (mut x, sr) = wav::read(&p).unwrap();
    x[100] += 0.1;
    wav::write(&p, &x, sr).unwrap();
    let suite = parse_suite(&suite_text).unwrap();
    let (rows, skipped) =
        vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, Some(&root));
    assert_eq!(rows.len(), 5);
    assert!(
        skipped
            .iter()
            .any(|m| m.what == "cafe-c00" && m.why.contains("sha256"))
    );
    // 缺段的场景不能判「过」
    let card = Card::build(&rows, skipped);
    assert_eq!(card.cards[0].missing.len(), 1);
    assert_ne!(card.cards[0].status, CardStatus::Pass);
}

/// 轨长与清单 dur_samples 不符：报错、不截短（codex 第 1 轮 P1 #2）。
#[test]
fn track_length_mismatch_is_reported_not_truncated() {
    let (root, suite_text) = setup("len");
    let jl = root.join("clips.jsonl");
    let t = std::fs::read_to_string(&jl).unwrap();
    let mut lines: Vec<Value> = t
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let n = lines[0]["dur_samples"].as_u64().unwrap();
    lines[0]["dur_samples"] = json!(n + 160);
    std::fs::write(
        &jl,
        lines
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    let suite = parse_suite(&suite_text).unwrap();
    let (rows, missing) =
        vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, Some(&root));
    assert_eq!(rows.len(), 5);
    assert!(
        missing
            .iter()
            .any(|m| m.what == "cafe-c00" && m.why.contains("dur_samples"))
    );
}

/// 整条任务展开不出片段：成绩单里仍有这一行（待测），不会消失（codex 第 1 轮 P1 #1）。
#[test]
fn task_without_clips_still_appears() {
    let (_root, suite_text) = setup("empty");
    let mut v: Value = serde_json::from_str(&suite_text).unwrap();
    v["tasks"][0]["real"]["tier"] = json!("stress");
    let suite = parse_suite(&v.to_string()).unwrap();
    let (rows, missing) = vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, None);
    assert!(rows.is_empty());
    let card = Card::build(&rows, missing);
    assert_eq!(card.cards.len(), 1);
    assert_eq!(card.cards[0].status, CardStatus::Unknown);
    assert!(card.to_markdown("t").contains("没跑成的"));
}

/// 清单标明没有干净参考：整段不打分、进「没跑成的」（codex 第 3 轮 P1 #1）。
#[test]
fn clip_without_clean_ref_is_not_scored() {
    let (root, suite_text) = setup("noref");
    let jl = root.join("clips.jsonl");
    let t = std::fs::read_to_string(&jl).unwrap();
    let mut lines: Vec<Value> = t
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    lines[0]["has_clean_ref"] = json!(false);
    std::fs::write(
        &jl,
        lines
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    let suite = parse_suite(&suite_text).unwrap();
    let (rows, missing) =
        vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, Some(&root));
    assert_eq!(rows.len(), 5);
    assert!(
        missing
            .iter()
            .any(|m| m.what == "cafe-c00" && m.why.contains("has_clean_ref"))
    );
}

/// 误传 ci 清单、空清单：直接报错，不出空成绩单（codex 第 4 轮 P2 #1）。
#[test]
fn non_scene_or_empty_suite_is_rejected() {
    let ci = parse_suite(vdn_eval::suite::CI_SUITE).unwrap();
    let e = vdn_eval::scorecard::check_suite(&ci).unwrap_err();
    assert!(e.contains("不是场景任务"), "{e}");
    let empty = parse_suite(r#"{"name": "x", "tasks": []}"#).unwrap();
    assert!(vdn_eval::scorecard::check_suite(&empty).is_err());
    let (_, suite_text) = setup("check");
    assert!(vdn_eval::scorecard::check_suite(&parse_suite(&suite_text).unwrap()).is_ok());
    // 场景是 other 的任务：报错（codex 第 5 轮 P2 #2）
    let mut v: Value = serde_json::from_str(&suite_text).unwrap();
    v["tasks"][0]["real"]["scene"] = json!("other");
    let e = vdn_eval::scorecard::check_suite(&parse_suite(&v.to_string()).unwrap()).unwrap_err();
    assert!(e.contains("other"), "{e}");
}

/// 同一输出目录跑第二次：报错，旧成绩单一个字不动（codex 第 5 轮 P1 #1）。
#[test]
fn second_write_to_same_dir_is_refused() {
    use vdn_eval::scorecard::card::{ensure_fresh, write_new};
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("scorecard-nooverwrite");
    let _ = std::fs::remove_dir_all(&d);
    ensure_fresh(&d).unwrap();
    write_new(&d, "{\"a\": 1}\n", "# 第一次\n").unwrap();
    assert!(ensure_fresh(&d).is_err());
    assert!(write_new(&d, "{}\n", "# 第二次\n").is_err());
    assert_eq!(
        std::fs::read_to_string(d.join("scorecard.md")).unwrap(),
        "# 第一次\n"
    );
    assert_eq!(
        std::fs::read_to_string(d.join("scorecard.json")).unwrap(),
        "{\"a\": 1}\n"
    );
}

/// 进仓的样例行（`tests/fixtures/scene-clip-example.jsonl`，vdn-schema 的 schema 测试也用它）能被解析器读。
#[test]
fn committed_example_line_parses() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scene-clip-example.jsonl");
    let v = SceneClip::parse_jsonl(&std::fs::read_to_string(p).unwrap()).unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].sounds.len(), 2);
    assert!(v[0].audio.owner_early.is_none(), "旧格式行没有早期参考");
}

/// 仓里的场景任务清单（scenesuite 的 `suite/scenes.json`）与它引用的 `clips.jsonl` 都读得了：
/// 每条任务是场景任务、展开出至少 1 段、每段的场景 / 档 / 设备与任务一致，片段 id 全局不重复。
/// 清单在 scenesuite 合并前不在本分支：那时只打印提示（合并后这条测试才真正生效）。
/// 可以用 `VDN_SCENES_SUITE=<scenes.json 绝对路径>` 指向别处的清单（相对路径按清单所在仓库根解析）。
#[test]
fn repo_scene_suite_parses_and_expands() {
    let (path, root) = match std::env::var("VDN_SCENES_SUITE") {
        Ok(p) => {
            let p = PathBuf::from(p);
            let root = p.ancestors().nth(4).unwrap().to_path_buf();
            (p, root)
        }
        Err(_) => {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            (root.join("src/eval/suite/scenes.json"), root)
        }
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("没有 {}（scenesuite 未合并），跳过", path.display());
        return;
    };
    let suite = parse_suite(&text).unwrap();
    assert!(!suite.tasks.is_empty());
    let mut ids = std::collections::BTreeSet::new();
    let mut n = 0;
    for t in &suite.tasks {
        let Some(vdn_eval::suite::RealSource::Scene {
            clips,
            scene,
            tier,
            device,
        }) = &t.real
        else {
            panic!("{} 不是场景任务", t.id);
        };
        let jl =
            std::fs::read_to_string(root.join(clips)).unwrap_or_else(|e| panic!("{clips}：{e}"));
        let all = SceneClip::parse_jsonl(&jl).unwrap();
        let mine: Vec<_> = all
            .iter()
            .filter(|c| c.scene == *scene && c.tier == *tier && c.device == *device)
            .collect();
        assert!(!mine.is_empty(), "{} 展开不出片段", t.id);
        for c in mine {
            assert!(ids.insert(c.id.clone()), "片段 {} 被两条任务展开", c.id);
            n += 1;
        }
    }
    eprintln!("{} 条任务展开 {n} 段", suite.tasks.len());
}

/// 人定 D85 题 2 = B：新规则下人声起伏 · 音色 · 轻声字在「干净人声单独过一遍」的输出上算，分数带标记；
/// 原声直通过一遍干净人声 = 干净人声本身，三项读数都接近 0。旧规则 v1 照旧在带噪输出上算、不带标记。
#[test]
fn clean_pass_items_scored_on_clean_input_under_v2() {
    use vdn_eval::scorecard::card::Rules;
    use vdn_eval::scorecard::standard::{CLEAN_PASS, CLEAN_PASS_MARK};
    let (root, suite_text) = setup("cleanpass");
    let suite = parse_suite(&suite_text).unwrap();
    let mark = |s: &vdn_eval::Score| {
        s.extra
            .iter()
            .any(|(k, v)| *k == CLEAN_PASS_MARK && *v == 1.0)
    };
    let run = |r| {
        vdn_eval::scorecard::run_with(
            &suite,
            &[SystemKind::Passthrough],
            &opts(),
            None,
            Some(&root),
            r,
        )
        .0
    };
    let v2 = run(Rules::V2);
    assert_eq!(v2.len(), 6);
    for r in &v2 {
        let picked: Vec<_> = r
            .scores
            .iter()
            .filter(|(v, _, _)| CLEAN_PASS.contains(v))
            .collect();
        assert_eq!(picked.len(), CLEAN_PASS.len(), "{}：三项都要在", r.clip);
        for (v, _, s) in picked {
            assert!(mark(s), "{v} 应带干净人声标记");
            // 每项都必须出分，不许「全无分也通过」（codex 中途 2 第 2 轮 P2 #2）
            let x = s
                .value
                .unwrap_or_else(|| panic!("{} · {v} 没出分：{}", r.clip, s.verdict.as_str()));
            assert!(
                x.is_finite() && x.abs() < 0.5,
                "{} · {v}：原声直通过干净人声读数应近 0，得 {x}",
                r.clip
            );
        }
        // 其余项不带标记
        assert!(
            r.scores
                .iter()
                .filter(|(v, _, _)| !CLEAN_PASS.contains(v))
                .all(|(_, _, s)| !mark(s))
        );
    }
    let v1 = run(Rules::V1);
    assert!(
        v1.iter().flat_map(|r| &r.scores).all(|(_, _, s)| !mark(s)),
        "v1 不跑干净人声那一路"
    );
}
