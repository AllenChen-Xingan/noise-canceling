//! 早期回声参考（D99 第 2 条）的端到端测试：成绩单旁列、生成器真实写出的行、夹具目录隔离。
//! 夹具在 `common/scene_fixture.rs`。

use std::path::Path;

use serde_json::{Value, json};
use vdn_eval::scene::SceneClip;
use vdn_eval::scorecard::card::Card;
use vdn_eval::suite::parse_suite;
use vdn_eval::system::SystemKind;
use vdn_eval::wav;

#[path = "common/scene_fixture.rs"]
mod fx;
use fx::{SR, opts, setup};

/// exp013 生成器真实写出的一行（带 `owner_early`，冒烟 build 取的）：Rust 读得了，三处（路径 · 两种哈希）都在。
#[test]
fn generated_early_line_parses() {
    let p =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/scene-clip-early-example.jsonl");
    let v = SceneClip::parse_jsonl(&std::fs::read_to_string(p).unwrap()).unwrap();
    assert_eq!(v.len(), 1);
    let c = &v[0];
    assert!(
        c.audio
            .owner_early
            .as_deref()
            .unwrap()
            .ends_with("owner_early.wav")
    );
    assert_eq!(c.hash.wav_sha256.owner_early.as_ref().unwrap().len(), 64);
    assert_eq!(c.hash.samples.owner_early.as_ref().unwrap().len(), 64);
}

/// 给 setup 出的每段加一条早期回声参考 `owner_early.wav`（`f` 把干净参考变成早期参考），改写 clips.jsonl。
fn add_early(root: &Path, f: impl Fn(&[f32]) -> Vec<f32>) {
    let jl = root.join("clips.jsonl");
    let text = std::fs::read_to_string(&jl).unwrap();
    let mut out = String::new();
    for l in text.lines() {
        let mut v: Value = serde_json::from_str(l).unwrap();
        let owner = v["audio"]["owner"].as_str().unwrap().to_string();
        let (x, _) = wav::read(&root.join(&owner)).unwrap();
        let rel = owner.replace("-owner.wav", "-owner_early.wav");
        wav::write(&root.join(&rel), &f(&x), SR).unwrap();
        v["audio"]["owner_early"] = json!(rel);
        v["hash"]["wav_sha256"]["owner_early"] = json!(wav::sha256(&root.join(&rel)).unwrap());
        v["hash"]["samples"]["owner_early"] = json!("0".repeat(64));
        out += &(v.to_string() + "\n");
    }
    std::fs::write(&jl, out).unwrap();
}

/// 早期回声参考旁列（D99 第 2 条）：有 owner_early 的段，EARLY_REF 项多一个 early_ref 旁证，其余项没有；
/// 早期参考 = 主参考时旁证与主值逐位相同；clips.jsonl 写出读回保留；成绩单表格与 JSON 带旁列；判定不受影响。
#[test]
fn early_reference_side_column() {
    use vdn_eval::scorecard::early::{EARLY_MARK, value_of};
    use vdn_eval::scorecard::standard::EARLY_REF;
    let has = |s: &vdn_eval::Score| s.extra.iter().any(|(k, _)| *k == EARLY_MARK);
    let run = |root: &Path, text: &str| {
        let suite = parse_suite(text).unwrap();
        vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, Some(root))
    };
    // 旧 build（没有 owner_early）：没有旁证
    let (root0, t0) = setup("early-none");
    let (base, _) = run(&root0, &t0);
    assert!(base.iter().flat_map(|r| &r.scores).all(|(_, _, s)| !has(s)));
    // 早期参考 = 主参考：旁证与主值逐位相同
    let (root1, t1) = setup("early-same");
    add_early(&root1, <[f32]>::to_vec);
    let (same, miss) = run(&root1, &t1);
    assert!(miss.is_empty(), "{miss:?}");
    for r in &same {
        for (v, _, s) in &r.scores {
            assert_eq!(has(s), EARLY_REF.contains(v), "{} · {v}", r.clip);
            if EARLY_REF.contains(v) {
                assert_eq!(
                    value_of(s),
                    s.value.filter(|x| x.is_finite()),
                    "{} · {v}",
                    r.clip
                );
            }
        }
    }
    // 早期参考 ≠ 主参考（压低 6 dB）：主值、判定与没有旁列时完全相同，旁证不同
    let (root2, t2) = setup("early-diff");
    add_early(&root2, |x| x.iter().map(|v| v * 0.5).collect());
    let (diff, _) = run(&root2, &t2);
    let mut changed = 0;
    for (a, b) in base.iter().zip(&diff) {
        for ((v, _, sa), (_, _, sb)) in a.scores.iter().zip(&b.scores) {
            assert_eq!(
                sa.value.map(f64::to_bits),
                sb.value.map(f64::to_bits),
                "{} · {v}",
                a.clip
            );
            assert_eq!(sa.verdict.as_str(), sb.verdict.as_str());
            if value_of(sb).is_some() && value_of(sb) != sb.value {
                changed += 1;
            }
        }
    }
    assert!(changed > 0, "压低 6 dB 的早期参考应让部分旁列读数变化");
    // clips.jsonl 写出读回保留旁证
    let back = vdn_eval::scorecard::load::parse_jsonl(
        &vdn_eval::scorecard::load::to_jsonl(&diff).unwrap(),
    )
    .unwrap();
    for (a, b) in diff.iter().zip(&back) {
        for ((_, _, sa), (_, _, sb)) in a.scores.iter().zip(&b.scores) {
            // serde_json 读浮点可能差最后一位：按相对容差比（有没有值必须一致）
            match (value_of(sa), value_of(sb)) {
                (Some(x), Some(y)) => assert!(
                    (x - y).abs() <= 1e-12 * x.abs().max(y.abs()).max(1e-300),
                    "{x} vs {y}"
                ),
                (x, y) => assert_eq!(x, y),
            }
        }
    }
    // 成绩单：有旁列的项带 early_mean 与「早期参考」字样；状态与没有旁列时一样
    let card = Card::build(&diff, Vec::new());
    let card0 = Card::build(&base, Vec::new());
    let est = card.cards[0]
        .items
        .iter()
        .find(|i| i.v == "V-ESTOI")
        .unwrap();
    assert!(est.early_mean.is_some());
    assert!(card.to_markdown("t").contains("早期参考"));
    let js = card.to_json(json!({}));
    let items = js["scenes"][0]["items"].as_array().unwrap();
    assert!(items.iter().any(|i| i.get("early_mean").is_some()));
    for (x, y) in card.cards[0].items.iter().zip(&card0.cards[0].items) {
        assert_eq!(x.status, y.status, "{}", x.v);
    }
    // 早期参考哈希对不上：整段记缺，不打分
    let (root3, t3) = setup("early-hash");
    add_early(&root3, <[f32]>::to_vec);
    let p = root3.join("clips/cafe-c00-owner_early.wav");
    wav::write(&p, &[0.1f32; 16], SR).unwrap();
    let (rows3, miss3) = run(&root3, &t3);
    assert_eq!(rows3.len(), 5);
    assert!(
        miss3
            .iter()
            .any(|m| m.what == "cafe-c00" && m.why.contains("sha256")),
        "{miss3:?}"
    );
}

/// 并行 / 并发隔离（2026-11-24 整套重跑时 early_reference_side_column 偶发「没跑成的段」）：
/// 旧 setup 按标签定目录、开头先删同名目录，另一个同时跑的测试进程用同一标签就会把正在用的夹具删掉。
/// 现在每次 setup 都得到独立目录：同一标签连调两次，第一份夹具仍完整可读。
#[test]
fn setup_dirs_are_isolated() {
    let (r1, _) = setup("iso");
    let (r2, _) = setup("iso");
    assert_ne!(r1, r2, "同一标签两次 setup 必须是不同目录");
    let text = std::fs::read_to_string(r1.join("clips.jsonl")).unwrap();
    let clips = SceneClip::parse_jsonl(&text).unwrap();
    for c in &clips {
        assert!(
            r1.join(&c.audio.mix).exists(),
            "第一份夹具被第二次 setup 删了：{}",
            c.audio.mix
        );
    }
}
