//! exp011 驱动：对缓存的（修音后）GTCRN 输出做「衰减下限 / 快开慢关 + 湿干比」后处理，并用 vdn-eval 的红线评测打分。
//!
//! 从标准输入读 JSONL，每行一个任务，结果按行写到标准输出：
//! - 处理并打分：`{"id", "dry", "wet", "clean", "out", "floor_db", "detect_db", "strength"}`（可选 open_db · release_s · hangover_s）
//! - 只打分：`{"id", "dry", "clean", "score": <已处理好的 wav>}`
//!
//! 包络口径照 `src/engine/src/engine.rs` 的 `OpenCloseParams` 构造（attack 2 ms、abs_min −70 dB、底噪上涨 6 dB/s），
//! hop 256 @ 16 kHz，判据信号 = 修音后的 wet。湿干比放最后一步：out = dry + strength·(gated − dry)（本实验的选择，不是规约）。

use std::io::{BufRead, Write};

use serde_json::{Value, json};
use vdn_dsp::{OpenCloseEnvelope, OpenCloseParams, db_to_lin};
use vdn_eval::analysis::{masks, truth_from_clean};
use vdn_eval::synth::{Clip, Truth};
use vdn_eval::verifiers::{Ctx, Params, eval};

const SR: u32 = 16000;
const HOP: usize = 256;

fn read(path: &str) -> Vec<f32> {
    let mut r = hound::WavReader::open(path).unwrap_or_else(|e| panic!("读不了 {path}：{e}"));
    let s = r.spec();
    assert_eq!((s.sample_rate, s.channels), (SR, 1), "{path} 要 16 kHz 单声道");
    match s.sample_format {
        hound::SampleFormat::Int => {
            let k = (1i64 << (s.bits_per_sample - 1)) as f32;
            r.samples::<i32>().map(|v| v.unwrap() as f32 / k).collect()
        }
        hound::SampleFormat::Float => r.samples::<f32>().map(|v| v.unwrap()).collect(),
    }
}

fn write(path: &str, x: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SR,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap_or_else(|e| panic!("写不了 {path}：{e}"));
    for v in x {
        w.write_sample(*v).unwrap();
    }
    w.finalize().unwrap();
}

fn f(j: &Value, k: &str, d: f32) -> f32 {
    j.get(k).and_then(Value::as_f64).map(|v| v as f32).unwrap_or(d)
}

/// 返回（输出, 每 hop 是否判有声）。
fn post(dry: &[f32], wet: &[f32], j: &Value) -> (Vec<f32>, Vec<bool>) {
    let floor_lin = db_to_lin(f(j, "floor_db", -12.0));
    let mut env = OpenCloseEnvelope::new(OpenCloseParams {
        sr: SR as f32,
        closed: floor_lin,
        open: db_to_lin(f(j, "open_db", -6.0)).max(floor_lin),
        attack_s: 0.002,
        release_s: f(j, "release_s", 0.3),
        hangover_s: f(j, "hangover_s", 0.15),
        detect_db: f(j, "detect_db", 9.0),
        abs_min_db: -70.0,
        noise_rise_db_per_s: 6.0,
    });
    let s = f(j, "strength", 1.0);
    let n = dry.len().min(wet.len());
    let mut out = vec![0.0f32; n];
    let mut voiced = Vec::with_capacity(n / HOP + 1);
    let mut mix = vec![0.0f32; HOP];
    let mut a = 0;
    while a < n {
        let b = (a + HOP).min(n);
        let m = &mut mix[..b - a];
        env.process(&wet[a..b], m);
        voiced.push(env.voiced());
        for i in a..b {
            let gated = wet[i] + m[i - a] * (dry[i] - wet[i]);
            out[i] = dry[i] + s * (gated - dry[i]);
        }
        a = b;
    }
    (out, voiced)
}

/// 红线与旁证：V-NAN（L3）· V-ONSET 吞字头 · V-SPLVL 人声被压低（L0，定标前）· V-TIMBRE · V-MUSIC · V-NR（旁证）。
fn score(dry: &[f32], clean: &[f32], out: &[f32]) -> Value {
    let n = dry.len().min(clean.len()).min(out.len());
    let mix = dry[..n].to_vec();
    let clean = clean[..n].to_vec();
    let noise: Vec<f32> = mix.iter().zip(&clean).map(|(m, c)| m - c).collect();
    let clip = Clip {
        sr: SR,
        truth: Truth {
            owner: truth_from_clean(&clean),
            interf: Vec::new(),
        },
        clean,
        interf: vec![0.0; n],
        noise,
        mix,
        tracks: false,
    };
    let m = masks(&clip.truth, &clip.mix, 0.03);
    let ctx = Ctx {
        clip: &clip,
        out: &out[..n],
        run: None,
        masks: &m,
        noise_out: None,
        ci: false,
    };
    let checks: [(&str, Value); 6] = [
        ("V-NAN", json!({})),
        ("V-ONSET", json!({"max": 3})),
        ("V-SPLVL", json!({"min": -3})),
        ("V-TIMBRE", json!({"max": 6})),
        ("V-MUSIC", json!({"max": 2})),
        ("V-NR", json!({})),
    ];
    let mut r = serde_json::Map::new();
    for (id, p) in checks {
        let p = Params(p.as_object().cloned().unwrap_or_default());
        let s = eval(id, &ctx, &p);
        r.insert(
            id.to_string(),
            json!({"value": s.value, "verdict": s.verdict.as_str(), "threshold": s.threshold}),
        );
    }
    Value::Object(r)
}

fn main() {
    let stdin = std::io::stdin();
    let mut so = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let j: Value = serde_json::from_str(&line).expect("任务行不是 JSON");
        let s = |k: &str| j[k].as_str().unwrap_or_else(|| panic!("缺 {k}")).to_string();
        let dry = read(&s("dry"));
        let clean = read(&s("clean"));
        let (out, voiced_frac) = if let Some(p) = j.get("score").and_then(Value::as_str) {
            (read(p), None)
        } else {
            let wet = read(&s("wet"));
            let (o, v) = post(&dry, &wet, &j);
            write(&s("out"), &o);
            let frac = v.iter().filter(|x| **x).count() as f64 / v.len().max(1) as f64;
            (o, Some(frac))
        };
        let r = json!({"id": j["id"], "voiced_frac": voiced_frac, "scores": score(&dry, &clean, &out)});
        writeln!(so, "{r}").unwrap();
    }
}
