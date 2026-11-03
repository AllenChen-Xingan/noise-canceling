//! 离线命令 `vdn denoise` · `vdn bench`：读写 16 kHz 单声道 WAV，不碰音频设备。

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::json;
use vdn_engine::{Engine, FaultInjector, FaultKind, Model, SafetyConfig, offline};
use vdn_models::{AnyModel, BoxError, ModelKind};

use crate::GLOBAL;
use crate::args::{BenchArgs, DenoiseArgs, ModelOpts};

pub fn open_model(m: &ModelOpts) -> Result<(AnyModel, PathBuf), BoxError> {
    let kind = ModelKind::from(m.model);
    let path = match &m.model_path {
        Some(p) => p.clone(),
        None => kind
            .default_path()
            .ok_or("找不到模型文件：用 --model-path 指定，或设环境变量 VDN_GTCRN_ONNX / VDN_FASTENHANCER_ONNX")?,
    };
    Ok((kind.open(&path)?, path))
}

pub fn read_wav(path: &Path) -> Result<Vec<f32>, BoxError> {
    let mut r = hound::WavReader::open(path)?;
    let spec = r.spec();
    if spec.channels != 1 || spec.sample_rate != 16000 {
        return Err(format!(
            "{}：只收 16 kHz 单声道（这份是 {} Hz、{} 声道）",
            path.display(),
            spec.sample_rate,
            spec.channels
        )
        .into());
    }
    let x = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (spec.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|s| s.map(|v| v as f32 * scale))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    Ok(x)
}

pub fn write_wav(path: &Path, y: &[f32]) -> Result<(), BoxError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for &v in y {
        w.write_sample(v)?;
    }
    w.finalize()?;
    Ok(())
}

pub fn parse_inject(s: &str) -> Result<Vec<(u64, FaultKind)>, BoxError> {
    s.split(',')
        .filter(|t| !t.is_empty())
        .map(|t| {
            let (hop, kind) = t.split_once(':').ok_or("--inject 格式：<hop>:<kind>")?;
            let hop: u64 = hop.trim().parse()?;
            let kind = match kind.trim() {
                "nan" => FaultKind::Nan,
                "error" => FaultKind::Error,
                "explode" => FaultKind::Explode,
                k => match k.strip_prefix("stall=") {
                    Some(ms) => FaultKind::Stall(Duration::from_secs_f32(ms.parse::<f32>()? / 1e3)),
                    None => return Err(format!("未知故障：{k}").into()),
                },
            };
            Ok((hop, kind))
        })
        .collect()
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn percentiles(t: &[Duration]) -> serde_json::Value {
    let mut v: Vec<f64> = t.iter().map(|d| ms(*d)).collect();
    v.sort_by(|a, b| a.total_cmp(b));
    // 线性插值，同 numpy.percentile 默认
    let p = |q: f64| {
        if v.is_empty() {
            return f64::NAN;
        }
        let pos = q / 100.0 * (v.len() - 1) as f64;
        let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
        v[lo] + (v[hi] - v[lo]) * (pos - lo as f64)
    };
    json!({"n": v.len(), "p50": p(50.0), "p99": p(99.0), "p999": p(99.9), "max": v.last().copied().unwrap_or(f64::NAN)})
}

/// 命令行给的参数按描述表（`vdn_types::desc`，范围的正本）检查：越界拒收，不悄悄夹值。
pub fn check_safety_ranges(cfg: &SafetyConfig) -> Result<(), vdn_types::RangeError> {
    use vdn_types::desc::*;
    FLOOR_DB.check(cfg.floor_db)?;
    OPEN_DB.check(cfg.open_db)?;
    HANGOVER_S.check(cfg.hangover_s)?;
    RELEASE_S.check(cfg.release_s)?;
    Ok(())
}

pub fn denoise(a: DenoiseArgs) -> Result<(), BoxError> {
    let x = read_wav(&a.input)?;
    let (model, path) = open_model(&a.m)?;
    let mut cfg = if a.no_safety {
        SafetyConfig::off()
    } else {
        SafetyConfig::default()
    };
    if !a.no_safety {
        cfg.fallback = !a.no_fallback;
        cfg.floor = !a.no_floor;
        cfg.fast_open = !a.no_fast_open && !a.no_floor;
        cfg.floor_db = a.floor_db;
        cfg.open_db = a.open_db;
        cfg.hangover_s = a.hangover_ms / 1e3;
        cfg.release_s = a.release_ms / 1e3;
        cfg.deadline = a.deadline_ms.map(|v| Duration::from_secs_f32(v / 1e3));
        check_safety_ranges(&cfg)?;
        if cfg.open_db < cfg.floor_db {
            return Err("--open-db 必须 ≥ --floor-db".into());
        }
    }
    let plan = match &a.inject {
        Some(s) => parse_inject(s)?,
        None => Vec::new(),
    };
    let model = FaultInjector::new(model, plan);
    let mut engine = Engine::new(model, cfg.clone());
    let run = offline::run(&mut engine, &x, a.lead_in);
    write_wav(&a.output, &run.output)?;
    let s = engine.stats();
    let nonfinite = run.output.iter().filter(|v| !v.is_finite()).count();
    eprintln!(
        "vdn: {} → {}（{}，{} 个 hop，送原声 {} hop，复位 {} 次，有声 {} hop，非有限样本 {}）",
        a.input.display(),
        a.output.display(),
        engine.model().name(),
        s.hops,
        s.bypass_hops,
        s.model_resets,
        s.voiced_hops,
        nonfinite
    );
    if let Some(j) = &a.json {
        let out = json!({
            "input": a.input, "output": a.output, "model": engine.model().name(), "model_path": path,
            "hop": engine.hop(), "latency_samples": engine.latency_samples(), "lead_in": a.lead_in,
            "safety": format!("{cfg:?}"),
            "stats": {
                "hops": s.hops, "fault_backend": s.fault_backend, "fault_nonfinite": s.fault_nonfinite,
                "fault_abnormal": s.fault_abnormal, "fault_deadline": s.fault_deadline,
                "model_resets": s.model_resets, "bypass_hops": s.bypass_hops, "voiced_hops": s.voiced_hops,
                "nonfinite_samples": nonfinite,
            },
            "model_ms": percentiles(&run.model_times),
            "engine_ms": percentiles(&run.engine_times),
        });
        std::fs::write(j, serde_json::to_string_pretty(&out)?)?;
    }
    Ok(())
}

pub fn bench(a: BenchArgs) -> Result<(), BoxError> {
    let x = read_wav(&a.input)?;
    if x.is_empty() {
        return Err("输入为空".into());
    }
    let (model, path) = open_model(&a.m)?;
    let mut raw = Engine::new(model, SafetyConfig::off());
    let hop = raw.hop();
    let need = (a.warmup + a.steps) * hop;
    let xs: Vec<f32> = x.iter().copied().cycle().take(need).collect();
    let mut y = vec![0.0f32; hop];
    let mut t_model = Vec::with_capacity(a.steps);
    let mut t_engine = Vec::with_capacity(a.steps);
    let mut nonfinite = 0usize;
    let (mut alloc_total, mut alloc_max, mut alloc_hops) = (0usize, 0usize, 0usize);
    for (k, xi) in xs.chunks_exact(hop).enumerate() {
        let region = stats_alloc::Region::new(GLOBAL);
        let t0 = std::time::Instant::now();
        raw.process(xi, &mut y);
        let te = t0.elapsed();
        let ch = region.change();
        if k >= a.warmup {
            let n = ch.allocations + ch.reallocations;
            alloc_total += n;
            alloc_max = alloc_max.max(n);
            alloc_hops += usize::from(n > 0);
            t_model.push(raw.last_model_time());
            t_engine.push(te);
            nonfinite += y.iter().filter(|v| !v.is_finite()).count();
        }
    }
    // 同一模型再跑一遍「三件保命全开」，看保命本身加了多少
    let (model2, _) = open_model(&a.m)?;
    let mut safe = Engine::new(model2, SafetyConfig::default());
    let mut t_safe = Vec::with_capacity(a.steps);
    for (k, xi) in xs.chunks_exact(hop).enumerate() {
        let t0 = std::time::Instant::now();
        safe.process(xi, &mut y);
        if k >= a.warmup {
            t_safe.push(t0.elapsed());
        }
    }
    let period_ms = hop as f64 / 16.0;
    let out = json!({
        "model": raw.model().name(), "model_path": path, "input": a.input,
        "hop": hop, "period_ms": period_ms, "warmup": a.warmup, "steps": a.steps,
        "threads": 1, "nonfinite_samples": nonfinite,
        "rust_allocs_per_hop": {
            "mean": alloc_total as f64 / a.steps.max(1) as f64, "max": alloc_max,
            "hops_with_alloc": alloc_hops, "note": "只计 Rust 全局分配器；ORT C++ 内部分配不在内",
        },
        "model_ms": percentiles(&t_model),
        "engine_raw_ms": percentiles(&t_engine),
        "engine_safety_ms": percentiles(&t_safe),
    });
    let text = serde_json::to_string_pretty(&out)?;
    println!("{text}");
    if let Some(j) = &a.json {
        std::fs::write(j, text)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::{Cli, Cmd};
    use clap::Parser;

    /// 命令行默认值与描述表同源；越界按描述表拒收。
    #[test]
    fn safety_args_follow_desc_table() {
        use vdn_types::desc::*;
        let cfg = SafetyConfig::default();
        check_safety_ranges(&cfg).unwrap();
        let cli = Cli::try_parse_from(["vdn", "denoise", "a.wav", "b.wav"]).unwrap();
        let Cmd::Denoise(a) = cli.cmd else {
            panic!("不是 denoise");
        };
        assert_eq!((a.floor_db, a.open_db), (FLOOR_DB.default, OPEN_DB.default));
        assert_eq!(a.hangover_ms / 1e3, HANGOVER_S.default);
        assert_eq!(a.release_ms / 1e3, RELEASE_S.default);
        for (d, bad) in [
            (
                &FLOOR_DB,
                SafetyConfig {
                    floor_db: FLOOR_DB.min - 1.0,
                    ..cfg.clone()
                },
            ),
            (
                &OPEN_DB,
                SafetyConfig {
                    open_db: OPEN_DB.max + 1.0,
                    ..cfg.clone()
                },
            ),
            (
                &HANGOVER_S,
                SafetyConfig {
                    hangover_s: -0.1,
                    ..cfg.clone()
                },
            ),
            (
                &RELEASE_S,
                SafetyConfig {
                    release_s: RELEASE_S.max + 1.0,
                    ..cfg.clone()
                },
            ),
        ] {
            assert_eq!(check_safety_ranges(&bad).unwrap_err().desc.id, d.id);
        }
    }
}
