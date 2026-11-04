//! `bench-split`：把一次判断里的声纹计算拆开计时（exp015）。
//!
//! 对一段 WAV 按判断回路的窗长 / 步长（正本 `JudgeParams`，可用 `--win` `--judge-hop` 覆盖）切窗，每个窗依次计时：
//! - `fbank`：算 fbank + 窗内减均值（与 `Embedder::embed` 里同一组调用）
//! - `embed`：`Embedder::embed` 整个调用（= fbank + 减均值 + 建张量 + 模型推理 + 归一化）
//! - `infer_est`：同一个窗的 `embed − fbank`（有符号，可为负），即「推理及其余」
//! - `synthetic_enroll_and_cos`：**合成**注册负载——每窗都喂注册器（固定电平、整 hop 时长）+ 与上一窗、与主人（锁定后）各一次余弦。
//!   真实判断回路按有声比例和相似度筛选才注册、用真实电平，状态轨迹不同，所以这只是这条合成轨迹的测量，不是真实成本的上限
//!
//! 测量偏差的处理：正式计时前用真实窗预热；fbank 与 embed 谁先跑按「第几遍 + 第几窗」交替，所以同一个窗在相邻两遍里两种顺序都测到，
//! 两组 p50 里音频内容的影响大致抵消（遍数取偶数时完全对半），分开报两种顺序的 p50（顺序敏感性）。
//! fbank 独立计时用的是另一份工作区，和 embed 里那一次不是同一次调用，所以 `infer_est` 是估计。
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
pub(crate) fn percentile(xs: &[i64], q: f64) -> Option<i64> {
    if xs.is_empty() {
        return None;
    }
    let mut v = xs.to_vec();
    v.sort_unstable();
    let idx = ((q.clamp(0.0, 1.0) * v.len() as f64).ceil() as usize).clamp(1, v.len()) - 1;
    v.get(idx).copied()
}

/// 一组纳秒样本 → `{n, p50_us, p90_us, max_us, mean_us}`。
pub(crate) fn summary(xs: &[i64]) -> Value {
    let us = |v: Option<i64>| v.map(|x| x as f64 / 1e3);
    let mean = if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<i64>() as f64 / xs.len() as f64 / 1e3)
    };
    json!({
        "n": xs.len(),
        "p50_us": us(percentile(xs, 0.5)),
        "p90_us": us(percentile(xs, 0.9)),
        "max_us": us(xs.iter().copied().max()),
        "mean_us": mean,
    })
}

fn ns(t0: Instant) -> i64 {
    i64::try_from(t0.elapsed().as_nanos()).unwrap_or(i64::MAX)
}

/// 第 `rep` 遍、第 `k` 个窗是否先跑 fbank。相邻窗交替，同一窗每遍反转，让每个窗两种顺序都测到。
pub(crate) fn fbank_first(rep: usize, k: usize) -> bool {
    (rep + k) % 2 == 0
}

/// 预热轮数：正式计时前，前几个真实窗各跑几遍（`Embedder::open` 只用 1 s 探针预热过一次）。
const WARMUP_WINDOWS: usize = 3;
const WARMUP_PASSES: usize = 3;

/// `vdn-speaker-lab bench-split <in.wav> [--reps N] [--win S] [--judge-hop S] [--speaker-onnx P]`，结果 JSON 打到标准输出。
pub(crate) fn cmd_bench_split(args: &[String]) -> R<()> {
    let x = read_wav(Path::new(args.first().ok_or("缺 <in.wav>")?))?;
    let jp = judge_params(args)?;
    let reps: usize = arg(args, "--reps").map(|v| v.parse()).transpose()?.unwrap_or(6);
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
    for _ in 0..WARMUP_PASSES {
        for &st in starts.iter().take(WARMUP_WINDOWS) {
            let w = x.get(st..st + win).ok_or("窗越界")?;
            let nf = fb.compute_into(w, &mut feats);
            cmn(&mut feats, nf, nb);
            black_box(emb.embed(w)?);
        }
    }
    // 按顺序分组的样本：[fbank 先跑, embed 先跑]
    let (mut fb_by, mut emb_by) = ([Vec::new(), Vec::new()], [Vec::new(), Vec::new()]);
    let mut prev: Option<Vec<f32>> = None;
    for r in 0..reps {
        for (k, &st) in starts.iter().enumerate() {
            let i = r * starts.len() + k;
            let w = x.get(st..st + win).ok_or("窗越界")?;
            let mut time_fbank = || {
                let t0 = Instant::now();
                let nf = fb.compute_into(w, &mut feats);
                cmn(&mut feats, nf, nb);
                black_box(&feats);
                ns(t0)
            };
            let first = fbank_first(r, k);
            let (a, b, v) = if first {
                let a = time_fbank();
                let t0 = Instant::now();
                let v = emb.embed(w)?;
                (a, ns(t0), v)
            } else {
                let t0 = Instant::now();
                let v = emb.embed(w)?;
                let b = ns(t0);
                (time_fbank(), b, v)
            };
            let t0 = Instant::now();
            black_box(prev.as_ref().map(|p| cosine(p, &v)));
            black_box(enroller.owner().map(|o| cosine(o, &v)));
            let t_s = i as f64 * jp.hop_s as f64;
            black_box(enroller.observe(&v, t_s, jp.hop_s, -23.0));
            t_enr.push(ns(t0));
            let g = usize::from(!first);
            if let (Some(fg), Some(eg)) = (fb_by.get_mut(g), emb_by.get_mut(g)) {
                fg.push(a);
                eg.push(b);
            }
            t_fb.push(a);
            t_emb.push(b);
            t_inf.push(b - a);
            prev = Some(v);
        }
    }
    let p50 = |v: &[i64]| percentile(v, 0.5).unwrap_or(0) as f64;
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
            "synthetic_enroll_and_cos": summary(&t_enr),
            "order_sensitivity": {
                "fbank_first": {"fbank": summary(&fb_by[0]), "embed": summary(&emb_by[0])},
                "embed_first": {"fbank": summary(&fb_by[1]), "embed": summary(&emb_by[1])},
            },
            "warmup": {"windows": WARMUP_WINDOWS, "passes": WARMUP_PASSES},
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
        let xs = [5i64, 1, 4, 2, 3];
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
        // 有符号差值：负数照常统计，不截成 0
        assert_eq!(summary(&[-1000, 1000])["mean_us"], 0.0);
        assert_eq!(summary(&[-3000, 1000])["p50_us"], -3.0);
    }

    #[test]
    fn order_alternates_and_flips_per_rep() {
        let firsts: Vec<bool> = (0..4).map(|k| fbank_first(0, k)).collect();
        assert_eq!(firsts, vec![true, false, true, false]);
        // 窗数为偶数（c01 有 158 个窗）时，同一个窗在相邻两遍里顺序相反
        for k in 0..158 {
            assert_ne!(fbank_first(0, k), fbank_first(1, k), "窗 {k}");
        }
        // 偶数遍：每个窗两种顺序各一半
        let n_first = (0..6).filter(|&r| fbank_first(r, 7)).count();
        assert_eq!(n_first, 3);
    }

    /// 计时用的 fbank 调用与 `Embedder::embed` 里的一致：帧数与判断窗对上（1.5 s → 148 帧）。
    #[test]
    fn fbank_frames_match_judge_window() {
        let fb = Fbank::new(FbankConfig::wespeaker());
        assert_eq!(fb.num_frames(24000), 148);
    }
}
