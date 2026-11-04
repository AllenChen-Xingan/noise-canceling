//! `bench-split`：把一次判断里的声纹计算拆开计时（exp015）。
//!
//! 对一段 WAV 按判断回路的窗长 / 步长（正本 `JudgeParams`，可用 `--win` `--judge-hop` 覆盖）切窗，每个窗依次计时：
//! - `fbank`：算 fbank + 窗内减均值（与 `Embedder::embed` 里同一组调用）
//! - `embed`：`Embedder::embed` 整个调用（= fbank + 减均值 + 建张量 + 模型推理 + 归一化）
//! - `infer_est`：同一个窗的 `embed − fbank`，即「推理及其余」
//! - `enroll`：喂一次注册器 + 与上一个窗的余弦（判断回路里 fbank 与模型之外的那一截）
//!
//! 只计时，不改认人行为；判断回路里「有声 < 30% 不算声纹」的跳过逻辑不在这里，送进来的窗全算。

use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};
use vdn_speaker::embed::{Embedder, cosine};
use vdn_speaker::enroll::{EnrollParams, Enroller};
use vdn_speaker::fbank::{Fbank, FbankConfig, cmn};

use crate::params::{arg, judge_params, speaker_path};
use crate::wav::read_wav;
use crate::{R, SR, ort_env};

/// 窗起点：从 0 开始每 `hop` 一个，窗必须整个落在信号里。
pub(crate) fn window_starts(n: usize, win: usize, hop: usize) -> Vec<usize> {
    if win == 0 || hop == 0 || n < win {
        return Vec::new();
    }
    (0..=(n - win) / hop).map(|k| k * hop).collect()
}

/// 最近秩分位数（`q` ∈ [0, 1]）；空输入返回 `None`。
pub(crate) fn percentile(xs: &[u64], q: f64) -> Option<u64> {
    if xs.is_empty() {
        return None;
    }
    let mut v = xs.to_vec();
    v.sort_unstable();
    let idx = ((q.clamp(0.0, 1.0) * v.len() as f64).ceil() as usize).clamp(1, v.len()) - 1;
    v.get(idx).copied()
}

/// 一组纳秒样本 → `{n, p50_us, p90_us, max_us, mean_us}`。
pub(crate) fn summary(xs: &[u64]) -> Value {
    let us = |v: Option<u64>| v.map(|x| x as f64 / 1e3);
    let mean = if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<u64>() as f64 / xs.len() as f64 / 1e3)
    };
    json!({
        "n": xs.len(),
        "p50_us": us(percentile(xs, 0.5)),
        "p90_us": us(percentile(xs, 0.9)),
        "max_us": us(xs.iter().copied().max()),
        "mean_us": mean,
    })
}

fn ns(t0: Instant) -> u64 {
    t0.elapsed().as_nanos() as u64
}

/// `vdn-speaker-lab bench-split <in.wav> [--reps N] [--win S] [--judge-hop S] [--speaker-onnx P]`，结果 JSON 打到标准输出。
pub(crate) fn cmd_bench_split(args: &[String]) -> R<()> {
    let x = read_wav(Path::new(args.first().ok_or("缺 <in.wav>")?))?;
    let jp = judge_params(args)?;
    let reps: usize = arg(args, "--reps").map(|v| v.parse()).transpose()?.unwrap_or(5);
    let win = (jp.window_s * SR as f32) as usize;
    let hop = (jp.hop_s * SR as f32) as usize;
    let starts = window_starts(x.len(), win, hop);
    if starts.is_empty() || reps == 0 {
        return Err("信号比一个判断窗短，或 --reps 为 0".into());
    }
    let cfg = FbankConfig::wespeaker();
    let env = ort_env()?;
    let mut emb = Embedder::open(&env, &speaker_path(args)?, cfg, win)?;
    let mut fb = Fbank::new(cfg);
    let nb = cfg.num_mel_bins;
    let mut feats = vec![0.0f32; fb.num_frames(win) * nb];
    let mut enroller = Enroller::new(EnrollParams::default());
    let total = reps * starts.len();
    let (mut t_fb, mut t_emb, mut t_inf, mut t_enr) = (
        Vec::with_capacity(total),
        Vec::with_capacity(total),
        Vec::with_capacity(total),
        Vec::with_capacity(total),
    );
    let mut prev: Option<Vec<f32>> = None;
    for r in 0..reps {
        for (k, &s) in starts.iter().enumerate() {
            let w = x.get(s..s + win).ok_or("窗越界")?;
            let t0 = Instant::now();
            let nf = fb.compute_into(w, &mut feats);
            cmn(&mut feats, nf, nb);
            black_box(&feats);
            let a = ns(t0);
            let t0 = Instant::now();
            let v = emb.embed(w)?;
            let b = ns(t0);
            let t0 = Instant::now();
            let ps = prev.as_ref().map(|p| cosine(p, &v));
            let t_s = (r * starts.len() + k) as f64 * jp.hop_s as f64;
            black_box(enroller.observe(&v, t_s, jp.hop_s, -23.0));
            black_box(ps);
            t_enr.push(ns(t0));
            t_fb.push(a);
            t_emb.push(b);
            t_inf.push(b.saturating_sub(a));
            prev = Some(v);
        }
    }
    let p50 = |v: &[u64]| percentile(v, 0.5).unwrap_or(0) as f64;
    let share = p50(&t_fb) / p50(&t_emb).max(1.0);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "samples": x.len(),
            "window_s": jp.window_s,
            "hop_s": jp.hop_s,
            "frames_per_window": fb.num_frames(win),
            "windows": starts.len(),
            "reps": reps,
            "emb_dim": emb.dim(),
            "fbank": summary(&t_fb),
            "embed": summary(&t_emb),
            "infer_est": summary(&t_inf),
            "enroll_and_cos": summary(&t_enr),
            "fbank_share_of_embed_p50": share,
        }))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_cover_whole_windows_only() {
        assert_eq!(window_starts(10, 4, 3), vec![0, 3, 6]);
        assert_eq!(window_starts(9, 4, 3), vec![0, 3]);
        assert_eq!(window_starts(3, 4, 3), Vec::<usize>::new());
        assert_eq!(window_starts(10, 0, 3), Vec::<usize>::new());
        assert_eq!(window_starts(10, 4, 0), Vec::<usize>::new());
        // 判断回路默认：1.5 s 窗、0.5 s 步长，80 s 素材 → 158 个窗
        assert_eq!(window_starts(80 * 16000, 24000, 8000).len(), 158);
    }

    #[test]
    fn percentile_nearest_rank() {
        let xs = [5u64, 1, 4, 2, 3];
        assert_eq!(percentile(&xs, 0.5), Some(3));
        assert_eq!(percentile(&xs, 0.0), Some(1));
        assert_eq!(percentile(&xs, 1.0), Some(5));
        assert_eq!(percentile(&xs, 0.9), Some(5));
        assert_eq!(percentile(&[], 0.5), None);
    }

    #[test]
    fn summary_reports_microseconds() {
        let s = summary(&[1000, 2000, 3000]);
        assert_eq!(s["n"], 3);
        assert_eq!(s["p50_us"], 2.0);
        assert_eq!(s["max_us"], 3.0);
        assert_eq!(s["mean_us"], 2.0);
        assert!(summary(&[])["p50_us"].is_null());
    }

    /// 计时用的 fbank 调用与 `Embedder::embed` 里的一致：帧数与判断窗对上（1.5 s → 148 帧）。
    #[test]
    fn fbank_frames_match_judge_window() {
        let fb = Fbank::new(FbankConfig::wespeaker());
        assert_eq!(fb.num_frames(24000), 148);
    }
}
