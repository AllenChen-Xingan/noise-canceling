//! 成绩单端到端：自造 6 段合成片段（写 wav + clips.jsonl，带声音标注）→ 场景任务清单 → `scorecard::run`（参考降噪器）
//! → 汇总 → Markdown / JSON。不需要模型：QA 层的吃字与听感判「不知道」，所以场景结论只能是「待测」或「不过」。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vdn_eval::runner::{Baseline, Opts};
use vdn_eval::scene::SceneClip;
use vdn_eval::scorecard::card::Card;
use vdn_eval::scorecard::rules::Status;
use vdn_eval::suite::{parse_levels, parse_suite, LEVELS};
use vdn_eval::synth::{NoiseKind, SceneSpec, TalkerSpec, render};
use vdn_eval::system::SystemKind;
use vdn_eval::wav;

fn tracks(v: impl Fn(&str) -> String, stems: &[&str]) -> Value {
    let mut st = serde_json::Map::new();
    for s in stems {
        st.insert(s.to_string(), json!(v(&format!("stem-{s}"))));
    }
    json!({"mix": v("mix"), "owner": v("owner"), "owner_dry": v("owner"), "floor": v("noise"),
           "noise": v("noise"), "interf": v("interf"), "stems": st})
}

/// 写一段：返回 clips.jsonl 的一行。`with_machine` 的段多一个「咖啡机」声音标注。
fn write_clip(root: &Path, i: usize, with_machine: bool) -> Value {
    let c = render(&SceneSpec {
        dur_s: 6.0,
        seed: 100 + i as u64,
        owner: Some(TalkerSpec {
            level_dbov: -26.0,
            f0_lo: 100.0,
            f0_hi: 140.0,
            tonal: false,
        }),
        owner_start_s: 0.5,
        pause_s: (0.4, 1.2),
        interferers: Vec::new(),
        noise: NoiseKind::Cafe,
        snr_db: if with_machine { 0.0 } else { 15.0 },
        tail_silence_s: 0.0,
    });
    let id = format!("cafe-c{i:02}");
    let rel = |k: &str| format!("clips/{id}-{k}.wav");
    let mut stems = vec!["bed0"];
    if with_machine {
        stems.push("cm0");
    }
    for (k, x) in [("mix", &c.mix), ("owner", &c.clean), ("noise", &c.noise), ("interf", &c.interf)] {
        wav::write(&root.join(rel(k)), x, 16_000).unwrap();
    }
    for s in &stems {
        wav::write(&root.join(rel(&format!("stem-{s}"))), &c.noise, 16_000).unwrap();
    }
    let sha = |k: &str| wav::sha256(&root.join(rel(k))).unwrap();
    let n = c.mix.len() as u64;
    let owner_iv: Vec<[u64; 2]> = c.truth.owner.iter().map(|&(a, b)| [a as u64, b as u64]).collect();
    let snd = |inst: &str, cls: &str, group: &str| {
        json!({"inst": inst, "cls": cls, "group": group, "is_speech": false, "iv": [[0, n]],
               "level_dbfs": -40.0, "rel_db": -10.0, "dist_m": null, "azimuth_deg": null,
               "src": {"kind": "synth", "fn": cls}, "license": "own"})
    };
    let mut sounds = vec![snd("bed0", "bed-cafe", "bed")];
    if with_machine {
        sounds.push(snd("cm0", "coffee-machine", "pattern"));
    }
    json!({
        "id": id, "scene": "cafe", "tier": "common", "device": "mba-builtin", "mic_mode": "standard",
        "pair": format!("{id}-x"), "lang": "en", "polarity": "suppress", "dur_samples": n, "sr": 16000,
        "audio": tracks(|k| rel(k), &stems), "has_clean_ref": true, "additive": true,
        "targets": [{"spk": "s1", "dist_m": 0.5, "azimuth_deg": 0.0, "level_dbfs": -26.0, "iv": owner_iv}],
        "sounds": sounds,
        "nominal": {"snr_db": 10.0, "sir_db": null, "t60_s": 0.3, "owner_dist_m": 0.5, "owner_level_dbfs": -26.0},
        "measured": {"snr_db": 10.0, "sir_db": null, "t60_s": 0.3, "drr_db": 5.0, "overlap_ratio": 0.0},
        "owner_iv": owner_iv, "interf_iv": [], "chain": {"flat": true}, "seed": 100 + i,
        "hash": {"samples": tracks(|_| "0".repeat(64), &stems), "wav_sha256": tracks(sha, &stems)},
    })
}

fn setup(tag: &str) -> (PathBuf, String) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("scorecard-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("clips")).unwrap();
    // 3 段有咖啡机（0 dB，更吵）、3 段没有（15 dB）
    let lines: Vec<String> = (0..6)
        .map(|i| write_clip(&root, i, i < 3).to_string())
        .collect();
    let jl = root.join("clips.jsonl");
    std::fs::write(&jl, lines.join("\n") + "\n").unwrap();
    let suite = json!({"name": "scenes-e2e", "tasks": [{
        "id": "S-cafe-common-mba", "set": "capability", "polarity": "suppress",
        "origin": "scorecard 端到端测试", "desc": "咖啡馆 · 常见档 · MacBook Air",
        "real": {"kind": "scene", "clips": jl.to_string_lossy(), "scene": "cafe", "tier": "common", "device": "mba-builtin"},
        "graders": []}]});
    (root, suite.to_string())
}

fn opts() -> Opts {
    Opts {
        ci: true,
        audio_dir: None,
        baseline: Baseline::new(),
        levels: parse_levels(LEVELS).unwrap(),
    }
}

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
    let (rows, skipped) = vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, Some(&root));
    assert!(skipped.is_empty(), "{skipped:?}");
    assert_eq!(rows.len(), 6);
    let card = Card::build(&rows, skipped);
    assert_eq!(card.cards.len(), 1, "一个 场景 × 档 × 设备 × 系统");
    let c = &card.cards[0];
    assert_eq!(c.clips, 6);
    // 否决项排在最前；吃字没接 ASR → 待测；听感没接模型 → 待测
    assert!(c.items[0].veto && c.items[3].v == "V-EAT-ASR");
    assert_eq!(c.items[3].status, Status::Unknown);
    assert_eq!(c.listen.status, Some(Status::Unknown));
    assert_ne!(c.status, Status::Pass, "缺 QA 项时不许判「过」");
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
    let (rows, skipped) = vdn_eval::scorecard::run(&suite, &[SystemKind::Ref], &opts(), None, Some(&root));
    assert_eq!(rows.len(), 5);
    assert!(skipped.iter().any(|(k, w)| k == "cafe-c00" && w.contains("sha256")));
}
