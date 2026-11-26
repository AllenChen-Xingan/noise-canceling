//! 造 `vdn-eval accept` 测试用的片段清单与评测清单（`--suite`）。
//! 被 `accept_fixture.rs` 用 `#[path]` 引入（拆开只为让每个文件 ≤ 600 行）。

use std::path::Path;

use serde_json::{Value, json};

use vdn_eval::scene::MeetingPlace;

/// 一条片段清单记录（够 `SceneClip::parse_jsonl` 读得过）。
fn clip_line(id: &str, scene: &str, tier: &str, device: &str) -> Value {
    let h = json!({"mix": "aa", "owner": "bb", "owner_dry": "cc", "stems": {"bed-x": "dd"},
                   "floor": "ee", "noise": "ff", "interf": "gg"});
    let dur = u64::from(vdn_types::MODEL_RATE);
    json!({
        "id": id, "scene": scene, "tier": tier, "device": device, "mic_mode": "unknown",
        "pair": id, "lang": "en", "polarity": "suppress", "dur_samples": dur,
        "sr": vdn_types::MODEL_RATE,
        "audio": {"mix": "m.wav", "owner": "o.wav", "owner_dry": "od.wav", "floor": "f.wav",
                  "noise": "n.wav", "interf": "i.wav", "stems": {"bed-x": "s.wav"}},
        "has_clean_ref": true, "additive": true,
        "targets": [{"spk": "1", "dist_m": 0.5, "azimuth_deg": 0.0, "level_dbfs": -30.0,
                     "iv": [[0, 8000]],
                     "src": {"kind": "librispeech", "spk": "1", "utts": ["u"]}}],
        "sounds": [{"inst": "bed-x", "cls": "bed-other", "group": "bed", "is_speech": false,
                    "iv": [[0, dur]], "level_dbfs": -40.0, "rel_db": -10.0,
                    "dist_m": null, "azimuth_deg": null,
                    "src": {"kind": "synth", "fn": "noise", "at": 0}, "license": "own"}],
        "nominal": {"snr_db": 5.0, "sir_db": null, "t60_s": 0.2, "owner_dist_m": 0.5,
                    "owner_level_dbfs": -30.0},
        "measured": {"snr_db": 5.0, "snr_all_db": 5.0, "owner_level_dbfs": -30.0,
                     "sir_db": null, "t60_s": 0.2, "drr_db": 2.0, "overlap_ratio": 0.0},
        "owner_iv": [[0, 8000]], "interf_iv": [], "chain": {}, "seed": 1,
        "hash": {"samples": h.clone(), "wav_sha256": h}
    })
}

/// 写片段清单 + 评测清单，返回（评测清单路径, 清单指纹）。
/// `scenes` = （场景, 这个场景的全部分段 id）；清单只用来定预期集合，id 顺序无所谓。
pub fn write(root: &Path, scenes: &[(MeetingPlace, Vec<String>)]) -> (String, String) {
    let mut lines = String::new();
    for (scene, ids) in scenes {
        let key = scene.as_str();
        for id in ids {
            lines.push_str(&clip_line(id, &key, "common", "mba-builtin").to_string());
            lines.push('\n');
        }
    }
    let manifest = root.join("clips.jsonl");
    std::fs::write(&manifest, lines).unwrap();
    let mp = manifest.to_string_lossy().to_string();
    let tasks: Vec<Value> = scenes
        .iter()
        .map(|(scene, _)| {
            let key = scene.as_str();
            json!({"id": format!("S-{key}-common"), "set": "capability", "polarity": "suppress",
                   "origin": "t", "desc": "t", "graders": [],
                   "real": {"kind": "scene", "clips": mp, "scene": key, "tier": "common",
                            "device": "mba-builtin"}})
        })
        .collect();
    let text = json!({"name": "accept-test", "seed": 1, "defaults": {"trials": 1},
        "tasks": tasks})
    .to_string();
    let path = root.join("suite.json");
    std::fs::write(&path, &text).unwrap();
    let fp = vdn_eval::scorecard::shard::suite_fingerprint(
        &vdn_eval::suite::parse_suite(&text).unwrap(),
    );
    (path.to_string_lossy().to_string(), fp)
}
