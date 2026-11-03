//! 其余四个子命令：denoise · envelope · fbank · embed。

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::json;
use vdn_engine::{Engine, SafetyConfig, offline};
use vdn_models::ModelKind;
use vdn_speaker::embed::Embedder;
use vdn_speaker::envelope::{EnergyEnvelopeStage, EnvelopeParams, VadEnvelopeStage};
use vdn_speaker::fbank::FbankConfig;
use vdn_types::{FrameCtx, SampleClock, Stage};

use crate::params::{arg, speaker_path};
use crate::wav::{read_wav, write_wav};
use crate::{HOP, R, SR, ort_env};

/// 只跑快开慢关两版（能量 / VAD），给 exp004 素材复算：写 `<prefix>.energy.wav`、`<prefix>.vad.wav`、`<prefix>.env.json`。
pub(crate) fn cmd_envelope(args: &[String]) -> R<()> {
    let dry = read_wav(Path::new(args.first().ok_or("缺 <noisy.wav>")?))?;
    let wet = read_wav(Path::new(args.get(1).ok_or("缺 <wet.wav>")?))?;
    let prefix = args.get(2).ok_or("缺 <out_prefix>")?;
    let n_hops = dry.len().min(wet.len()) / HOP;
    let ep = EnvelopeParams::p1_like(SR as f32);
    let mut en = EnergyEnvelopeStage::new(ep, HOP);
    let mut ve = VadEnvelopeStage::new(ep);
    let (mut y_en, mut y_ve) = (wet[..n_hops * HOP].to_vec(), wet[..n_hops * HOP].to_vec());
    let (mut eo, mut vo, mut vp) = (Vec::new(), Vec::new(), Vec::new());
    for k in 0..n_hops {
        let r = k * HOP..(k + 1) * HOP;
        let ctx = FrameCtx {
            now: SampleClock((k * HOP) as u64),
            ..FrameCtx::default()
        };
        en.process(&mut y_en[r.clone()], &dry[r.clone()], ctx);
        ve.process(&mut y_ve[r.clone()], &dry[r], ctx);
        eo.push(en.voiced() as u8);
        vo.push(ve.voiced() as u8);
        vp.push(ve.vad_prob());
    }
    write_wav(Path::new(&format!("{prefix}.energy.wav")), &y_en)?;
    write_wav(Path::new(&format!("{prefix}.vad.wav")), &y_ve)?;
    std::fs::write(
        format!("{prefix}.env.json"),
        serde_json::to_string(&json!({"energy_open": eo, "vad_open": vo, "vad_prob": vp}))?,
    )?;
    Ok(())
}

/// GTCRN 降噪（保命全关 = 模型原样输出，与输入逐样本对齐），写 32-bit float WAV。
pub(crate) fn cmd_denoise(args: &[String]) -> R<()> {
    let input = PathBuf::from(args.first().ok_or("缺 <in.wav>")?);
    let out = PathBuf::from(args.get(1).ok_or("缺 <out.wav>")?);
    let dry = read_wav(&input)?;
    let mpath = arg(args, "--model-path")
        .map(PathBuf::from)
        .or_else(|| ModelKind::Gtcrn.default_path())
        .ok_or("找不到 GTCRN（--model-path）")?;
    let mut engine = Engine::new(ModelKind::Gtcrn.open(&mpath)?, SafetyConfig::off());
    let wet = offline::run(&mut engine, &dry, HOP).output;
    write_wav(&out, &wet)?;
    println!(
        "{}",
        json!({"model": mpath.display().to_string(), "latency_samples": engine.latency_samples()})
    );
    Ok(())
}

pub(crate) fn cmd_fbank(args: &[String]) -> R<()> {
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

pub(crate) fn cmd_embed(args: &[String]) -> R<()> {
    let x = read_wav(Path::new(args.first().ok_or("缺 <in.wav>")?))?;
    let env = ort_env()?;
    let mut e = Embedder::open(
        &env,
        &speaker_path(args)?,
        FbankConfig::wespeaker(),
        x.len(),
    )?;
    let n = 20;
    let mut v = e.embed(&x)?;
    // 计时：同一段重复 n 次
    let t0 = Instant::now();
    for _ in 0..n {
        v = e.embed(&x)?;
    }
    let ms = t0.elapsed().as_secs_f64() * 1e3 / n as f64;
    println!(
        "{}",
        json!({"dim": v.len(), "ms_per_call": ms, "samples": x.len(), "emb": v})
    );
    Ok(())
}
