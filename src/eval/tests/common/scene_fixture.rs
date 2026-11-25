//! 场景成绩单端到端测试共用的夹具：自造合成片段（wav + clips.jsonl）与场景任务清单。
//! 被 `scorecard_e2e.rs` 与 `scorecard_early_e2e.rs` 用 `#[path]` 引入。

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vdn_eval::runner::{Baseline, Opts};
use vdn_eval::suite::{LEVELS, parse_levels};
use vdn_eval::synth::{NoiseKind, SceneSpec, TalkerSpec, render};
use vdn_eval::wav;

pub const SR: u32 = vdn_types::MODEL_RATE;

pub fn tracks(v: impl Fn(&str) -> String, stems: &[&str]) -> Value {
    let mut st = serde_json::Map::new();
    for s in stems {
        st.insert(s.to_string(), json!(v(&format!("stem-{s}"))));
    }
    json!({"mix": v("mix"), "owner": v("owner"), "owner_dry": v("owner"), "floor": v("noise"),
           "noise": v("noise"), "interf": v("interf"), "stems": st})
}

/// 写一段：返回 clips.jsonl 的一行。`with_machine` 的段多一个「咖啡机」声音标注。
pub fn write_clip(root: &Path, i: usize, with_machine: bool) -> Value {
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
    for (k, x) in [
        ("mix", &c.mix),
        ("owner", &c.clean),
        ("noise", &c.noise),
        ("interf", &c.interf),
    ] {
        wav::write(&root.join(rel(k)), x, SR).unwrap();
    }
    for s in &stems {
        wav::write(&root.join(rel(&format!("stem-{s}"))), &c.noise, SR).unwrap();
    }
    let sha = |k: &str| wav::sha256(&root.join(rel(k))).unwrap();
    let n = c.mix.len() as u64;
    let owner_iv: Vec<[u64; 2]> = c
        .truth
        .owner
        .iter()
        .map(|&(a, b)| [a as u64, b as u64])
        .collect();
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
        "pair": format!("{id}-x"), "lang": "en", "polarity": "suppress", "dur_samples": n, "sr": SR,
        "audio": tracks(|k| rel(k), &stems), "has_clean_ref": true, "additive": true,
        "targets": [{"spk": "s1", "dist_m": 0.5, "azimuth_deg": 0.0, "level_dbfs": -26.0, "iv": owner_iv, "src": {"kind": "synth", "spk": "s1"}}],
        "sounds": sounds,
        "nominal": {"snr_db": 10.0, "sir_db": null, "t60_s": 0.3, "owner_dist_m": 0.5, "owner_level_dbfs": -26.0},
        "measured": {"snr_db": 10.0, "snr_all_db": 9.0, "owner_level_dbfs": -26.0, "sir_db": null, "t60_s": 0.3, "drr_db": 5.0, "overlap_ratio": 0.0},
        "owner_iv": owner_iv, "interf_iv": [], "chain": {"flat": true}, "seed": 100 + i,
        "hash": {"samples": tracks(|_| "0".repeat(64), &stems), "wav_sha256": tracks(sha, &stems)},
    })
}

/// 夹具根目录：每次调用独立（进程号 + 进程内序号），同时跑的测试进程或线程互不删对方的夹具
/// （见 `setup_dirs_are_isolated`）。顺手删掉一小时前留下的旧夹具，免得越积越多。
pub fn fixture_root(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("scorecard-fx");
    let pid = std::process::id();
    if let Ok(rd) = std::fs::read_dir(&base) {
        for e in rd.flatten() {
            let old = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|d| d.as_secs() > 3600);
            let mine = e
                .file_name()
                .to_string_lossy()
                .starts_with(&format!("{pid}-"));
            if old && !mine {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    }
    base.join(format!("{pid}-{}-{tag}", N.fetch_add(1, Ordering::SeqCst)))
}

pub fn setup(tag: &str) -> (PathBuf, String) {
    let root = fixture_root(tag);
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

pub fn opts() -> Opts {
    Opts {
        ci: true,
        audio_dir: None,
        baseline: Baseline::new(),
        levels: parse_levels(LEVELS).unwrap(),
    }
}
