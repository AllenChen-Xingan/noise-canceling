//! 对拍驱动：读一份清单（JSON），对每条用 vdn-eval 的正本算 V-SPLVL · V-ONSET 与 10 ms 掩码，打印 JSON。
//!
//! 清单：`[{"name", "clean", "out", "mix"?, "owner": [[a,b],…], "interf": [[a,b],…]}]`，
//! 音频是小端 f32 原始样本（`.f32`，16 kHz），避免 WAV 量化影响对拍。
//! 用法：`exp025-parity <清单.json>` → 标准输出一个 JSON 数组。只读调用，不改 `src/eval`。

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};
use vdn_eval::analysis::{masks, truth_from_clean};
use vdn_eval::synth::{Clip, Truth};
use vdn_eval::verifiers::{Ctx, Params, eval};

fn read_f32(p: &Path) -> Vec<f32> {
    let b = std::fs::read(p).unwrap_or_else(|e| panic!("读 {}：{e}", p.display()));
    assert!(b.len() % 4 == 0, "{} 不是 f32 原始样本", p.display());
    b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn ivs(v: Option<&Value>) -> Vec<(usize, usize)> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|p| {
                    let p = p.as_array().expect("区间是 [a, b]");
                    (p[0].as_u64().unwrap() as usize, p[1].as_u64().unwrap() as usize)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn bits(m: &[bool]) -> String {
    m.iter().map(|&b| if b { '1' } else { '0' }).collect()
}

fn score_json(s: &vdn_eval::Score) -> Value {
    let extra: Map<String, Value> = s
        .extra
        .iter()
        .map(|(k, v)| ((*k).to_string(), json!(v)))
        .collect();
    json!({"value": s.value, "verdict": s.verdict.as_str(), "extra": extra})
}

fn main() {
    let arg = std::env::args().nth(1).expect("用法：exp025-parity <清单.json>");
    let base = PathBuf::from(&arg).parent().map(Path::to_path_buf).unwrap_or_default();
    let list: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(&arg).expect("读清单")).expect("清单 JSON");
    let mut out = Vec::new();
    for case in &list {
        let get = |k: &str| case.get(k).and_then(Value::as_str).map(|s| base.join(s));
        let clean = read_f32(&get("clean").expect("缺 clean"));
        let o = read_f32(&get("out").expect("缺 out"));
        let mix = get("mix").map(|p| read_f32(&p)).unwrap_or_else(|| clean.clone());
        let n = clean.len();
        let truth = Truth { owner: ivs(case.get("owner")), interf: ivs(case.get("interf")) };
        let clip = Clip {
            sr: 16_000,
            clean: clean.clone(),
            interf: vec![0.0; n],
            noise: vec![0.0; n],
            mix,
            truth: truth.clone(),
            tracks: true,
        };
        let m = masks(&truth, &clip.mix, 0.03);
        let ctx = Ctx { clip: &clip, out: &o, run: None, masks: &m, noise_out: None, ci: false };
        let p = Params(Map::new());
        let tfc: Vec<[usize; 2]> = truth_from_clean(&clean).into_iter().map(|(a, b)| [a, b]).collect();
        out.push(json!({
            "name": case.get("name"),
            "splvl": score_json(&eval("V-SPLVL", &ctx, &p)),
            "onset": score_json(&eval("V-ONSET", &ctx, &p)),
            "owner_mask": bits(&m.owner),
            "interf_mask": bits(&m.interf),
            "truth_from_clean": tfc,
        }));
    }
    println!("{}", serde_json::to_string(&out).unwrap());
}
