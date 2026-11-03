//! `run` 子命令：读降噪输出 → 判断回路（或重放判决）→ 一次出 6 个版本的音频 + 日志。

use std::path::PathBuf;
use std::time::Instant;

use serde_json::json;
use vdn_speaker::embed::Embedder;
use vdn_speaker::enroll::{EnrollOutcome, EnrollParams, Enroller};
use vdn_speaker::envelope::{EnergyEnvelopeStage, EnvelopeParams, FloorStage, VadEnvelopeStage};
use vdn_speaker::fbank::FbankConfig;
use vdn_speaker::gate::{AsymGate, GateParams};
use vdn_speaker::judge::Judge;
use vdn_types::{FrameCtx, SampleClock, SpeakerVerdict, Stage};

use crate::params::{arg, gate_params, judge_params, speaker_path};
use crate::verdict::{class_str, verdict_from_json, verdict_to_json};
use crate::wav::{read_wav, write_wav};
use crate::{HOP, R, SR, ort_env};

/// 把一个 Stage 套在整段（已对齐的 wet / dry）上，逐 hop 跑；`verdicts[k]` 是第 k 个 hop 看到的判决。
fn run_stage<S: Stage>(
    st: &mut S,
    wet: &[f32],
    dry: &[f32],
    verdicts: &[SpeakerVerdict],
) -> Vec<f32> {
    let mut out = wet.to_vec();
    for (k, (w, d)) in out.chunks_mut(HOP).zip(dry.chunks(HOP)).enumerate() {
        let ctx = FrameCtx {
            now: SampleClock((k * HOP) as u64),
            speaker: verdicts
                .get(k)
                .copied()
                .unwrap_or(SpeakerVerdict::default()),
            voice_prob: None,
        };
        st.process(w, d, ctx);
    }
    out
}

pub(crate) fn cmd_run(args: &[String]) -> R<()> {
    let input = PathBuf::from(args.first().ok_or("缺 <noisy.wav>")?);
    let out_dir = PathBuf::from(args.get(1).ok_or("缺 <out_dir>")?);
    std::fs::create_dir_all(&out_dir)?;
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("x")
        .trim_end_matches("_noisy")
        .trim_end_matches("_mix")
        .to_string();
    // 重放：读上一次的判决序列，只重跑实时级（调门控参数用，不重算声纹）
    let replay: Option<serde_json::Value> = match arg(args, "--replay") {
        Some(p) => Some(serde_json::from_str(&std::fs::read_to_string(p)?)?),
        None => None,
    };
    let tag = arg(args, "--tag").unwrap_or_default();
    if replay.is_some() && tag.is_empty() {
        return Err("--replay 必须带 --tag（不许覆盖被重放的那份日志与音频）".into());
    }
    let overwrite = args.iter().any(|a| a == "--overwrite");
    let log_path = out_dir.join(format!("{stem}.log{tag}.json"));
    if log_path.exists() && !overwrite {
        return Err(format!(
            "{} 已存在；换 --tag，或明确加 --overwrite",
            log_path.display()
        )
        .into());
    }
    let dry = read_wav(&input)?;

    // 1) 降噪输出：`--wet` 给出（由 `vdn-speaker-lab denoise` 在另一个进程里产出——ort 一个进程只许一个 Environment，
    //    vdn-models 与本 crate 各建一个会冲突，接入时要共用一个，见 README「接入注意」）
    let wet_path = PathBuf::from(
        arg(args, "--wet").ok_or("缺 --wet <denoised.wav>（先跑 vdn-speaker-lab denoise）")?,
    );
    let wet = read_wav(&wet_path)?;
    if wet.len() != dry.len() {
        return Err("--wet 与输入长度不同".into());
    }
    let n_hops = dry.len() / HOP;
    let (dry, wet) = (&dry[..n_hops * HOP], &wet[..n_hops * HOP]);

    // 2) 判断回路：喂降噪输出 + 逐帧 VAD（与 VadEnvelopeStage 同一个 VAD 口径），按实测计算时间延后生效
    // 先校验再建任何部件：声纹模型的窗长与 Judge 用同一个已校验的值（codex 第 2 轮 P2 #1）
    let jp = judge_params(args)?;
    let mut vad = vdn_speaker::vad::Vad::new();
    let mut vad_probs = Vec::with_capacity(n_hops);
    let mut verdicts = vec![SpeakerVerdict::default(); n_hops + 1];
    let mut events: Vec<serde_json::Value> = Vec::new();
    let (judge_logs, clusters, owner_idx, emb_dim);
    if let Some(old) = &replay {
        for k in 0..n_hops {
            vad_probs.push(vad.push(&wet[k * HOP..(k + 1) * HOP]));
        }
        let evs = old["verdict_events"]
            .as_array()
            .ok_or("重放日志里没有 verdict_events")?;
        let mut cur = SpeakerVerdict::default();
        let mut it = evs.iter().peekable();
        for (k, slot) in verdicts.iter_mut().enumerate().take(n_hops) {
            while let Some(e) = it.peek() {
                if e["at_hop"].as_u64().unwrap_or(u64::MAX) as usize <= k {
                    cur = verdict_from_json(e);
                    it.next();
                } else {
                    break;
                }
            }
            *slot = cur;
        }
        events = evs.clone();
        judge_logs = old["judge_logs"].clone();
        clusters = old["clusters"].clone();
        owner_idx = old["owner_cluster"].clone();
        emb_dim = old["emb_dim"].clone();
    } else {
        let env = ort_env()?;
        let emb = Embedder::open(
            &env,
            &speaker_path(args)?,
            FbankConfig::wespeaker(),
            (jp.window_s * SR as f32) as usize,
        )?;
        emb_dim = json!(emb.dim());
        let mut enroller = Enroller::new(EnrollParams::default());
        if let Some(p) = arg(args, "--preload") {
            let v: Vec<f32> = serde_json::from_str(&std::fs::read_to_string(p)?)?;
            enroller.preload(&v);
        }
        let mut judge = Judge::new(jp, emb, enroller);
        let mut cur = SpeakerVerdict::default();
        let mut pending: std::collections::VecDeque<(usize, SpeakerVerdict)> = Default::default();
        // 声纹线程是单线程串行的：上一份还没算完时，下一份要排队（自审 P2-8）
        let mut busy_until = 0.0f64;
        let slowdown: f64 = match arg(args, "--judge-slowdown") {
            Some(v) => v.parse()?,
            None => 1.0,
        };
        for k in 0..n_hops {
            // 本 hop 生效的判决
            while let Some(&(at, v)) = pending.front() {
                if at <= k {
                    cur = v;
                    pending.pop_front();
                } else {
                    break;
                }
            }
            verdicts[k] = cur;
            let w = &wet[k * HOP..(k + 1) * HOP];
            let p = vad.push(w);
            vad_probs.push(p);
            judge.push_audio(
                w,
                &dry[k * HOP..(k + 1) * HOP],
                SampleClock((k * HOP) as u64),
            );
            for _ in 0..vad.take_new_frames() {
                judge.push_vad(p);
            }
            if let Some(v) = judge.tick() {
                let c = judge
                    .logs
                    .last()
                    .map(|l| l.compute)
                    .unwrap_or_default()
                    .as_secs_f64()
                    * slowdown;
                // 到点 = 本 hop 结束；开始算 = max(到点, 上一份算完)；生效 = 算完后的下一个 hop
                let due = ((k + 1) * HOP) as f64 / SR as f64;
                let done = due.max(busy_until) + c;
                busy_until = done;
                let at = (done * SR as f64 / HOP as f64).ceil() as usize;
                pending.push_back((at, v));
                events.push(verdict_to_json(&v, at));
            }
        }
        judge_logs = json!(judge.logs.iter().map(|l| json!({
            "t": l.window_end as f64 / SR as f64,
            "vf": l.voiced_frac,
            "level_db": if l.level_db.is_finite() { json!(l.level_db) } else { json!(null) },
            "sim": if l.owner_sim.is_finite() { json!(l.owner_sim) } else { json!(null) },
            "prev_sim": if l.prev_sim.is_finite() { json!(l.prev_sim) } else { json!(null) },
            "enrolled": l.enrolled,
            "event": match l.event { EnrollOutcome::None => json!(null), EnrollOutcome::Locked{cluster} => json!({"locked": cluster}), EnrollOutcome::Relocked{from,to} => json!({"relocked": [from,to]}) },
            "class": class_str(l.class),
            "compute_ms": l.compute.as_secs_f64() * 1e3,
        })).collect::<Vec<_>>());
        if let (Some(p), Some(o)) = (arg(args, "--save-owner"), judge.enroller.owner()) {
            std::fs::write(p, serde_json::to_string(o)?)?;
        }
        clusters = json!(judge.enroller.clusters().iter().zip(judge.enroller.scores())
            .map(|(c, s)| json!({"dur_s": c.dur_s, "level_db": c.level_db, "cohesion": c.cohesion, "score": s, "centroid": c.centroid}))
            .collect::<Vec<_>>());
        owner_idx = json!(judge.enroller.owner_index());
    }

    // 3) 各版本
    let ep = EnvelopeParams::p1_like(SR as f32);
    let mut versions: Vec<(&str, Vec<f32>)> = vec![("raw", wet.to_vec())];
    versions.push((
        "floor",
        run_stage(&mut FloorStage::new(ep.floor_db), wet, dry, &verdicts),
    ));
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
        let ctx = FrameCtx {
            now: SampleClock((k * HOP) as u64),
            ..FrameCtx::default()
        };
        en.process(&mut y_en[r.clone()], &dry[r.clone()], ctx);
        energy_open.push(en.voiced() as u8);
        let t0 = Instant::now();
        ve.process(&mut y_ve[r.clone()], &dry[r], ctx);
        stage_ns_vad.push(t0.elapsed().as_nanos() as u64);
        vad_open.push(ve.voiced() as u8);
    }
    // 个性化：VAD 版 → 门控
    let gp = gate_params(args)?;
    let mut gate = AsymGate::new(gp);
    let mut y_p = y_ve.clone();
    let mut gains = Vec::with_capacity(n_hops);
    let mut onset_cum = Vec::with_capacity(n_hops);
    let mut stage_ns_gate = Vec::with_capacity(n_hops);
    for k in 0..n_hops {
        let r = k * HOP..(k + 1) * HOP;
        // 门控用与 VAD 包络同一个 VAD 的概率（ctx.voice_prob），接入 engine 后也是这样传
        let ctx = FrameCtx {
            now: SampleClock((k * HOP) as u64),
            speaker: verdicts[k],
            voice_prob: Some(vad_probs[k]),
        };
        let t0 = Instant::now();
        gate.process(&mut y_p[r.clone()], &dry[r], ctx);
        stage_ns_gate.push(t0.elapsed().as_nanos() as u64);
        gains.push(gate.gain());
        onset_cum.push(gate.onsets());
    }
    // 激进版（大总管 2026-10-30 定：给人听「压得干净 vs 不吃字」的取舍）：不混回原声、门控下限 −20 dB、
    // 认人确认 1 次、慢收 0.3 s；其余与保守版相同（同一份判决序列）
    let gp_aggr = GateParams {
        floor_db: -20.0,
        confirm_n: 1,
        release_s: 0.3, // tidy: 预设值，不随默认变：激进版的慢收时长（大总管 2026-10-30 定，见上方注释）；GateParams 对应 gate.release_s（默认 0.5），与 release_s 默认 0.3 只是同名
        ..gp
    };
    let mut gate_a = AsymGate::new(gp_aggr);
    let mut y_a = wet.to_vec();
    let mut gains_aggr = Vec::with_capacity(n_hops);
    for k in 0..n_hops {
        let r = k * HOP..(k + 1) * HOP;
        let ctx = FrameCtx {
            now: SampleClock((k * HOP) as u64),
            speaker: verdicts[k],
            voice_prob: Some(vad_probs[k]),
        };
        gate_a.process(&mut y_a[r.clone()], &dry[r], ctx);
        gains_aggr.push(gate_a.gain());
    }
    versions.push(("energy", y_en));
    versions.push(("vad", y_ve));
    versions.push(("personal", y_p));
    versions.push(("aggressive", y_a));
    for (name, y) in &versions {
        // 重放只写个性化那一版
        if replay.is_none() || *name == "personal" || *name == "aggressive" {
            write_wav(&out_dir.join(format!("{stem}.{name}{tag}.wav")), y)?;
        }
    }

    // 4) 日志
    let log = json!({
        "input": input.display().to_string(),
        "sr": SR, "hop": HOP,
        "wet": wet_path.display().to_string(),
        "emb_dim": emb_dim,
        "judge": {"window_s": jp.window_s, "hop_s": jp.hop_s, "owner_hi": jp.owner_hi, "others_lo": jp.others_lo},
        "gate": format!("{gp:?}"),
        "gate_aggressive": format!("{gp_aggr:?}"),
        "replay_of": arg(args, "--replay"),
        "judge_logs": judge_logs,
        "verdict_events": events,
        "owner_cluster": owner_idx,
        "clusters": clusters,
        "per_hop": {
            "gain": gains,
            "gain_aggressive": gains_aggr,
            "onset_cum": onset_cum,
            "energy_open": energy_open,
            "vad_open": vad_open,
            "vad_prob": vad_probs,
            "verdict": verdicts[..n_hops].iter().map(|v| class_str(v.class)).collect::<Vec<_>>(),
            "stage_ns_vad_env": stage_ns_vad,
            "stage_ns_gate": stage_ns_gate,
        },
        "onsets": gate.onsets(),
    });
    std::fs::write(&log_path, serde_json::to_string(&log)?)?;
    println!("{stem}{tag}: hops={n_hops} onsets={}", gate.onsets());
    Ok(())
}
