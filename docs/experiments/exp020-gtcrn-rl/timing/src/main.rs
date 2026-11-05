//! exp020 计时：一段场景录音上，系统各跑一次（含噪 · 只有噪音 · 只有干净人声）+ 每个 CI 层 verifier 单独计时。
//!
//! 用法：`exp020-timing <片段目录> [系统，默认 gtcrn] [重复次数，默认 5]`
//! 片段目录里要有 mix.wav · owner.wav · interf.wav · noise.wav（exp013 正式 build 的格式）。
//! 输出一行 JSON：每项的中位耗时（ms）与最小 / 最大值。只读输入，不写文件。

use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};
use vdn_eval::analysis::{masks, truth_from_clean};
use vdn_eval::synth::{Clip, SR, Truth};
use vdn_eval::system::{SystemKind, SystemSpec, run};
use vdn_eval::verifiers::{Ctx, Params, REGISTRY, eval};

/// 计时的 verifier：登记表里 CI 层的全部（QA 层要外部模型，另算）。
fn ci_ids() -> Vec<&'static str> {
    REGISTRY
        .iter()
        .filter(|i| i.layer == "ci")
        .map(|i| i.id)
        .collect()
}

fn median(xs: &mut [f64]) -> f64 {
    assert!(!xs.is_empty(), "median of empty");
    xs.sort_by(|a, b| a.total_cmp(b));
    let n = xs.len();
    if n % 2 == 1 {
        xs[n / 2]
    } else {
        0.5 * (xs[n / 2 - 1] + xs[n / 2])
    }
}

/// 跑 `reps` 次，返回 (中位, 最小, 最大) ms 与最后一次的结果。
fn timed<T>(reps: usize, mut f: impl FnMut() -> T) -> (Value, T) {
    let mut ts = Vec::with_capacity(reps);
    let mut last = None;
    for _ in 0..reps {
        let t0 = Instant::now();
        last = Some(f());
        ts.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let lo = ts.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = ts.iter().copied().fold(0.0, f64::max);
    let md = median(&mut ts);
    (
        json!({"median_ms": md, "min_ms": lo, "max_ms": hi, "reps": reps}),
        last.expect("reps >= 1"),
    )
}

fn load(dir: &Path) -> Result<Clip, String> {
    let rd = |k: &str| -> Result<Vec<f32>, String> {
        let (x, sr) = vdn_eval::wav::read(&dir.join(format!("{k}.wav")))?;
        if sr != SR {
            return Err(format!("{k}.wav 采样率 {sr}，要 {SR}"));
        }
        Ok(x)
    };
    let (mix, clean, interf, noise) = (rd("mix")?, rd("owner")?, rd("interf")?, rd("noise")?);
    let n = [mix.len(), clean.len(), interf.len(), noise.len()]
        .into_iter()
        .min()
        .unwrap_or(0);
    let cut = |v: Vec<f32>| v[..n].to_vec();
    let (mix, clean, interf, noise) = (cut(mix), cut(clean), cut(interf), cut(noise));
    Ok(Clip {
        sr: SR,
        truth: Truth {
            owner: truth_from_clean(&clean),
            interf: truth_from_clean(&interf),
        },
        clean,
        interf,
        noise,
        mix,
        tracks: true,
    })
}

/// 对一个已载入的片段做全部计时。
fn measure(clip: &Clip, sys: SystemKind, reps: usize) -> Result<Value, String> {
    let spec = SystemSpec {
        kind: sys,
        safety: true,
        faults: Vec::new(),
    };
    let (t_run, r) = timed(reps, || run(&spec, &clip.mix));
    let r = r?;
    let bg: Vec<f32> = clip.noise.iter().zip(&clip.interf).map(|(a, b)| a + b).collect();
    let (t_noise, nr) = timed(reps, || run(&spec, &bg));
    let noise_out = nr?.out;
    let (t_clean, _) = timed(reps, || run(&spec, &clip.clean));
    let (t_masks, m) = timed(reps, || masks(&clip.truth, &clip.mix, 0.03));
    let ctx = Ctx {
        clip,
        out: &r.out,
        run: Some(&r),
        masks: &m,
        noise_out: Some(&noise_out),
        ci: false,
    };
    let p = Params::default();
    let mut vs = serde_json::Map::new();
    for id in ci_ids() {
        let (t, s) = timed(reps, || eval(id, &ctx, &p));
        let mut t = t;
        t["verdict"] = json!(format!("{:?}", s.verdict));
        vs.insert(id.to_string(), t);
    }
    Ok(json!({
        "system": sys.as_str(),
        "audio_s": clip.len() as f64 / f64::from(SR),
        "run_mix": t_run,
        "run_noise_only": t_noise,
        "run_clean_only": t_clean,
        "masks": t_masks,
        "verifiers": vs,
    }))
}

fn main() -> std::process::ExitCode {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let Some(dir) = a.first() else {
        eprintln!("用法：exp020-timing <片段目录> [系统] [重复次数]");
        return std::process::ExitCode::FAILURE;
    };
    let sys_name = a.get(1).map_or("gtcrn", String::as_str);
    let reps: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(5).max(1);
    let r = SystemKind::parse(sys_name)
        .ok_or_else(|| format!("未知系统 {sys_name}"))
        .and_then(|k| load(Path::new(dir)).and_then(|c| measure(&c, k, reps)));
    match r {
        Ok(v) => {
            println!("{v}");
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("exp020-timing 出错：{e}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_eval::synth::{InterfMode, InterfSpec, NoiseKind, SceneSpec, TalkerSpec, render};

    #[test]
    fn median_odd_even() {
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&mut [4.0, 1.0, 2.0, 3.0]), 2.5);
    }

    /// 计时覆盖登记表里每一个 CI 层 verifier，且每项都真的被调用（结果 id 对得上）。
    #[test]
    fn measure_covers_every_ci_verifier() {
        let clip = render(&SceneSpec {
            dur_s: 6.0,
            seed: 7,
            owner: Some(TalkerSpec {
                level_dbov: -26.0,
                f0_lo: 100.0,
                f0_hi: 140.0,
                tonal: false,
            }),
            owner_start_s: 0.5,
            pause_s: (0.5, 1.0),
            interferers: vec![InterfSpec {
                mode: InterfMode::Overlap,
                sir_db: 0.0,
                f0_lo: 180.0,
                f0_hi: 240.0,
            }],
            noise: NoiseKind::Pink,
            snr_db: 10.0,
            tail_silence_s: 0.0,
        });
        let v = measure(&clip, SystemKind::Passthrough, 1).unwrap();
        let got = v["verifiers"].as_object().unwrap();
        assert_eq!(got.len(), ci_ids().len());
        for id in ci_ids() {
            assert!(got[id]["median_ms"].as_f64().unwrap() >= 0.0, "{id}");
        }
        assert!(!ci_ids().is_empty());
    }
}
