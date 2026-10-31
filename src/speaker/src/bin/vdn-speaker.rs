//! vdn-speaker：P2 离线实验命令行（exp005）。
//!
//! ```text
//! vdn-speaker run   <noisy.wav> <out_dir> [--model-path gtcrn.onnx] [--speaker-onnx ecapa.onnx] [--preload owner.json] [--save-owner owner.json]
//! vdn-speaker fbank <in.wav> <out.f32>        # fbank 原始特征（CMN 前），行主序 float32，对拍用
//! vdn-speaker embed <in.wav>                  # 打印整段的声纹向量（JSON），对拍用
//! ```
//!
//! `run` 一次出 5 个版本（GTCRN 降噪只跑一遍，共用）：
//! - `raw`：模型原样输出 · `floor`：只开衰减下限 −12 dB · `energy`：p1 能量版快开慢关
//! - `vad`：VAD 版快开慢关（= 个性化关）· `personal`：VAD 版 + 判断回路 + 不对称门控（= 个性化开）
//!
//! 判断回路的判决按**实测计算时间**延后生效（向上取整到 hop），模拟它在另一个线程上跑。

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;
use vdn_engine::{Engine, SafetyConfig, offline};
use vdn_models::ModelKind;
use vdn_speaker::embed::{Embedder, default_model_path};
use vdn_speaker::envelope::{EnergyEnvelopeStage, EnvelopeParams, FloorStage, VadEnvelopeStage};
use vdn_speaker::enroll::{EnrollEvent, EnrollParams, Enroller};
use vdn_speaker::fbank::FbankConfig;
use vdn_speaker::gate::{AsymGate, GateParams};
use vdn_speaker::judge::{Judge, JudgeParams};
use vdn_speaker::{FrameCtx, SpeakerClass, SpeakerVerdict, Stage};

type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const SR: u32 = 16000;
const HOP: usize = 256;

fn read_wav(p: &Path) -> R<Vec<f32>> {
    let mut r = hound::WavReader::open(p)?;
    let s = r.spec();
    if s.sample_rate != SR || s.channels != 1 {
        return Err(format!("{}: 只收 16 kHz 单声道", p.display()).into());
    }
    Ok(match s.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let k = 1.0 / (1u64 << (s.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|v| v.map(|x| x as f32 * k))
                .collect::<Result<_, _>>()?
        }
    })
}

fn write_wav(p: &Path, x: &[f32]) -> R<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SR,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(p, spec)?;
    for &v in x {
        w.write_sample(v)?;
    }
    w.finalize()?;
    Ok(())
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

fn speaker_path(args: &[String]) -> R<PathBuf> {
    arg(args, "--speaker-onnx")
        .map(PathBuf::from)
        .or_else(default_model_path)
        .ok_or_else(|| "找不到声纹模型（--speaker-onnx 或 VDN_SPEAKER_ONNX）".into())
}

/// 把一个 Stage 套在整段（已对齐的 wet / dry）上，逐 hop 跑；`verdicts[k]` 是第 k 个 hop 看到的判决。
fn run_stage<S: Stage>(st: &mut S, wet: &[f32], dry: &[f32], verdicts: &[SpeakerVerdict]) -> Vec<f32> {
    let mut out = wet.to_vec();
    for (k, (w, d)) in out.chunks_mut(HOP).zip(dry.chunks(HOP)).enumerate() {
        let ctx = FrameCtx {
            speaker: verdicts.get(k).copied().unwrap_or(SpeakerVerdict::UNKNOWN),
            sample_pos: (k * HOP) as u64,
        };
        st.process(w, d, ctx);
    }
    out
}

fn class_str(c: SpeakerClass) -> &'static str {
    match c {
        SpeakerClass::Unknown => "unknown",
        SpeakerClass::NoSpeech => "nospeech",
        SpeakerClass::Target => "owner",
        SpeakerClass::Other => "others",
    }
}

fn cmd_run(args: &[String]) -> R<()> {
    let input = PathBuf::from(args.first().ok_or("缺 <noisy.wav>")?);
    let out_dir = PathBuf::from(args.get(1).ok_or("缺 <out_dir>")?);
    std::fs::create_dir_all(&out_dir)?;
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("x").trim_end_matches("_noisy").to_string();
    let dry = read_wav(&input)?;

    // 1) 降噪（GTCRN，保命全关 = 模型原样输出，与输入逐样本对齐）
    let mpath = arg(args, "--model-path")
        .map(PathBuf::from)
        .or_else(|| ModelKind::Gtcrn.default_path())
        .ok_or("找不到 GTCRN（--model-path）")?;
    let model = ModelKind::Gtcrn.open(&mpath)?;
    let mut engine = Engine::new(model, SafetyConfig::off());
    let wet = offline::run(&mut engine, &dry, HOP).output;
    let n_hops = dry.len() / HOP;
    let (dry, wet) = (&dry[..n_hops * HOP], &wet[..n_hops * HOP]);

    // 2) 判断回路：喂降噪输出 + 逐帧 VAD（与 VadEnvelopeStage 同一个 VAD 口径），按实测计算时间延后生效
    let jp = JudgeParams::default();
    let emb = Embedder::open(&speaker_path(args)?, FbankConfig::wespeaker(), (jp.window_s * SR as f32) as usize)?;
    let emb_dim = emb.dim();
    let mut enroller = Enroller::new(EnrollParams::default());
    if let Some(p) = arg(args, "--preload") {
        let v: Vec<f32> = serde_json::from_str(&std::fs::read_to_string(p)?)?;
        enroller.preload(&v);
    }
    let mut judge = Judge::new(jp, emb, enroller);
    let mut vad = vdn_speaker::vad::Vad::new();
    let mut verdicts = vec![SpeakerVerdict::UNKNOWN; n_hops + 1];
    let mut cur = SpeakerVerdict::UNKNOWN;
    let mut pending: Vec<(usize, SpeakerVerdict)> = Vec::new();
    let mut vad_probs = Vec::with_capacity(n_hops);
    for k in 0..n_hops {
        // 本 hop 生效的判决
        while let Some(&(at, v)) = pending.first() {
            if at <= k {
                cur = v;
                pending.remove(0);
            } else {
                break;
            }
        }
        verdicts[k] = cur;
        let w = &wet[k * HOP..(k + 1) * HOP];
        let p = vad.push(w);
        vad_probs.push(p);
        judge.push_audio(w);
        for _ in 0..vad.take_new_frames() {
            judge.push_vad(p);
        }
        if let Some(v) = judge.tick() {
            let c = judge.logs.last().map(|l| l.compute).unwrap_or_default();
            let delay = (c.as_secs_f64() * SR as f64 / HOP as f64).ceil() as usize;
            pending.push((k + 1 + delay, v));
        }
    }

    // 3) 各版本
    let ep = EnvelopeParams::p1_like(SR as f32);
    let mut versions: Vec<(&str, Vec<f32>)> = vec![("raw", wet.to_vec())];
    versions.push(("floor", run_stage(&mut FloorStage::new(ep.floor_db), wet, dry, &verdicts)));
    // 能量版 / VAD 版：顺带记逐 hop「开着」的标记
    let mut en = EnergyEnvelopeStage::new(ep, HOP);
    let mut ve = VadEnvelopeStage::new(ep);
    let mut energy_open = Vec::with_capacity(n_hops);
    let mut vad_open = Vec::with_capacity(n_hops);
    let mut y_en = wet.to_vec();
    let mut y_ve = wet.to_vec();
    let mut stage_ns_vad = Vec::with_capacity(n_hops);
    for k in 0..n_hops {
        let r = k * HOP..(k + 1) * HOP;
        let ctx = FrameCtx { speaker: SpeakerVerdict::UNKNOWN, sample_pos: (k * HOP) as u64 };
        en.process(&mut y_en[r.clone()], &dry[r.clone()], ctx);
        energy_open.push(en.voiced() as u8);
        let t0 = Instant::now();
        ve.process(&mut y_ve[r.clone()], &dry[r], ctx);
        stage_ns_vad.push(t0.elapsed().as_nanos() as u64);
        vad_open.push(ve.voiced() as u8);
    }
    // 个性化：VAD 版 → 门控
    let mut gate = AsymGate::new(GateParams::default());
    let mut y_p = y_ve.clone();
    let mut gains = Vec::with_capacity(n_hops);
    let mut stage_ns_gate = Vec::with_capacity(n_hops);
    for k in 0..n_hops {
        let r = k * HOP..(k + 1) * HOP;
        let ctx = FrameCtx { speaker: verdicts[k], sample_pos: (k * HOP) as u64 };
        let t0 = Instant::now();
        gate.process(&mut y_p[r.clone()], &dry[r], ctx);
        stage_ns_gate.push(t0.elapsed().as_nanos() as u64);
        gains.push(gate.gain());
    }
    versions.push(("energy", y_en));
    versions.push(("vad", y_ve));
    versions.push(("personal", y_p));
    for (name, y) in &versions {
        write_wav(&out_dir.join(format!("{stem}.{name}.wav")), y)?;
    }

    // 4) 日志
    let logs: Vec<_> = judge
        .logs
        .iter()
        .map(|l| {
            json!({
                "t": l.window_end as f64 / SR as f64,
                "vf": l.voiced_frac,
                "level_db": if l.level_db.is_finite() { json!(l.level_db) } else { json!(null) },
                "sim": if l.owner_sim.is_finite() { json!(l.owner_sim) } else { json!(null) },
                "prev_sim": if l.prev_sim.is_finite() { json!(l.prev_sim) } else { json!(null) },
                "enrolled": l.enrolled,
                "event": match l.event { EnrollEvent::None => json!(null), EnrollEvent::Locked{cluster} => json!({"locked": cluster}), EnrollEvent::Relocked{from,to} => json!({"relocked": [from,to]}) },
                "class": class_str(l.class),
                "compute_ms": l.compute.as_secs_f64() * 1e3,
            })
        })
        .collect();
    let owner = judge.enroller.owner().map(|o| o.to_vec());
    if let (Some(p), Some(o)) = (arg(args, "--save-owner"), owner.as_ref()) {
        std::fs::write(p, serde_json::to_string(o)?)?;
    }
    let clusters: Vec<_> = judge
        .enroller
        .clusters()
        .iter()
        .zip(judge.enroller.scores())
        .map(|(c, s)| json!({"dur_s": c.dur_s, "level_db": c.level_db, "cohesion": c.cohesion, "score": s, "centroid": c.centroid}))
        .collect();
    let log = json!({
        "input": input.display().to_string(),
        "sr": SR, "hop": HOP,
        "model": mpath.display().to_string(),
        "model_latency_samples": engine.latency_samples(),
        "emb_dim": emb_dim,
        "judge": {"window_s": jp.window_s, "hop_s": jp.hop_s, "owner_hi": jp.owner_hi, "others_lo": jp.others_lo},
        "judge_logs": logs,
        "owner_cluster": judge.enroller.owner_index(),
        "clusters": clusters,
        "per_hop": {
            "gain": gains,
            "energy_open": energy_open,
            "vad_open": vad_open,
            "vad_prob": vad_probs,
            "verdict": verdicts[..n_hops].iter().map(|v| class_str(v.class)).collect::<Vec<_>>(),
            "stage_ns_vad_env": stage_ns_vad,
            "stage_ns_gate": stage_ns_gate,
        },
        "onsets": gate.onsets(),
    });
    std::fs::write(out_dir.join(format!("{stem}.log.json")), serde_json::to_string(&log)?)?;
    println!("{stem}: hops={n_hops} owner={:?}", judge.enroller.owner_index());
    Ok(())
}

fn cmd_fbank(args: &[String]) -> R<()> {
    let x = read_wav(Path::new(args.first().ok_or("缺 <in.wav>")?))?;
    let out = PathBuf::from(args.get(1).ok_or("缺 <out.f32>")?);
    let mut fb = vdn_speaker::fbank::Fbank::new(FbankConfig::wespeaker());
    let nf = fb.num_frames(x.len());
    let mut f = vec![0.0f32; nf * 80];
    fb.compute_into(&x, &mut f);
    let bytes: Vec<u8> = f.iter().flat_map(|v| v.to_le_bytes()).collect();
    std::fs::write(&out, bytes)?;
    println!("{{\"frames\": {nf}, \"bins\": 80}}");
    Ok(())
}

fn cmd_embed(args: &[String]) -> R<()> {
    let x = read_wav(Path::new(args.first().ok_or("缺 <in.wav>")?))?;
    let mut e = Embedder::open(&speaker_path(args)?, FbankConfig::wespeaker(), x.len())?;
    let n = 20;
    let mut v = e.embed(&x)?;
    // 计时：同一段重复 n 次
    let t0 = Instant::now();
    for _ in 0..n {
        v = e.embed(&x)?;
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3 / n as f64;
    println!("{}", json!({"dim": v.len(), "ms_per_call": ms, "samples": x.len(), "emb": v}));
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("run") => cmd_run(&args[1..]),
        Some("fbank") => cmd_fbank(&args[1..]),
        Some("embed") => cmd_embed(&args[1..]),
        _ => Err("用法：vdn-speaker run|fbank|embed …（见源码头注释）".into()),
    };
    if let Err(e) = r {
        eprintln!("vdn-speaker: {e}");
        std::process::exit(1);
    }
}
