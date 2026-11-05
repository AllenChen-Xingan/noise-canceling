//! 量输出的个性化降噪几项（人定 D85 题 4 = A；调研 025 D1 · D2b · D3 · D5，论证见 025 `notes/group-D.md`）。
//! 原门控几项（V-SIM-GATE · V-SIM-FA）量的是引擎内部开关，降为引擎诊断；这里改量**输出声音**。
//!
//! - **V-TSOS 用户被误删**：Eskimez 等 2022 式 (2)(3)。逐帧：Σ_f max(0, |S|^p − |Ŝ|^p) > γ · Σ_f |S|^p 记一帧误删；
//!   S = 干净参考谱，Ŝ = 输出谱，p = 0.3，γ = 0.1。只算用户有声帧。主数 = 误删帧百分比
//! - **V-SWALLOW 开口后被吞多久**：每个开口起点之后第一段连续误删的时长（起点帧没误删记 0），报本段 p90
//! - **V-LEAK 旁人漏过事件率**：只有旁人说话的 1 秒窗（用户一帧不说、旁人说话过半）里，输出比输入压低不到 X dB 的窗占比
//! - **V-PSE-OVL 同时说话只留用户**：用户与旁人真实同时有声的帧，桥接其间 300 ms 内的停顿后，连续 ≥ 0.5 s 的段上 SI-SDRi；旁报失败率（SI-SDRi < 1 dB 的段占比）
//!
//! 四项的过关线都还没定（TSOS 比原声直通多多少算不过、X 取几 dB：待人拍板），成绩单里只报不判。

use super::*;

/// 有声帧的 STFT 设定：与音色项同为 512 / 256（32 ms 窗、16 ms 跳）。
const NFFT: usize = 512;
const HOP: usize = 256;

fn spec_frames(c: &Ctx, x: &[f32]) -> Spectrogram {
    Spectrogram::new(&x[..x.len().min(c.clip.clean.len())], NFFT, HOP)
}

/// 用户有声的谱帧：帧中心落在主讲人掩码里。
fn owner_frame(c: &Ctx, sg: &Spectrogram, k: usize) -> bool {
    sg.mask_frame(k)
        .is_some_and(|m| c.masks.owner.get(m).copied().unwrap_or(false))
}

/// 逐帧误删标记（只给用户有声帧）：(谱帧号, 是否误删)。
pub fn tsos_flags(c: &Ctx, out: &[f32]) -> Vec<(usize, bool)> {
    let pw = th::TSOS.def("p") / 2.0; // 谱是功率（|X|²），|X|^p = (|X|²)^(p/2)
    let gamma = th::TSOS.def("gamma");
    let ss = spec_frames(c, &c.clip.clean);
    let so = spec_frames(c, out);
    (0..ss.frames.len().min(so.frames.len()))
        .filter(|&k| owner_frame(c, &ss, k))
        .map(|k| {
            let (mut los, mut ref_p) = (0.0f64, 0.0f64);
            for (s, o) in ss.frames[k].iter().zip(&so.frames[k]) {
                let sp = (*s as f64).powf(pw);
                let op = (*o as f64).powf(pw);
                los += (sp - op).max(0.0);
                ref_p += sp;
            }
            (k, ref_p > 0.0 && los > gamma * ref_p)
        })
        .collect()
}

/// 连续误删段的长度（帧数），按出现顺序。相邻用户帧之间有空档（不是用户帧）就断开。
fn runs(flags: &[(usize, bool)]) -> Vec<usize> {
    let mut out = Vec::new();
    let (mut n, mut last) = (0usize, None::<usize>);
    for &(k, f) in flags {
        let contiguous = last.is_some_and(|l| l + 1 == k);
        if f && (contiguous || n == 0) {
            n += 1;
        } else if f {
            out.push(n);
            n = 1;
        } else if n > 0 {
            out.push(n);
            n = 0;
        }
        last = Some(k);
    }
    if n > 0 {
        out.push(n);
    }
    out
}

pub(super) fn v_tsos(c: &Ctx) -> Score {
    const ID: &str = "V-TSOS";
    let f = tsos_flags(c, c.out);
    if f.len() < 20 {
        return Score::unknown(
            ID,
            "%",
            Better::Lower,
            format!("用户有声谱帧只有 {} 个", f.len()),
        );
    }
    let hit = f.iter().filter(|(_, x)| *x).count();
    let hop_s = HOP as f64 / c.clip.sr as f64;
    let rs = runs(&f);
    let long = (1.0 / hop_s).ceil() as usize;
    let mut s = Score::judged(
        ID,
        100.0 * hit as f64 / f.len() as f64,
        "%",
        Better::Lower,
        None,
        &[],
    )
    .with_extra("frames", f.len() as f64)
    .with_extra(
        "events_1s",
        rs.iter().filter(|&&n| n >= long).count() as f64,
    )
    .with_extra(
        "longest_s",
        rs.iter().copied().max().unwrap_or(0) as f64 * hop_s,
    );
    // 字头 200 ms 内的误删比例单列（025 group-D 结论；codex 中途 2 第 1 轮 P2 #4）：字头 = 前面停了 ≥ 300 ms 的开口
    let (on_n, on_hit) = onset_window_hits(c, &f);
    s = s.with_extra("onset_frames", on_n as f64).with_extra(
        "onset200_pct",
        if on_n > 0 {
            100.0 * on_hit as f64 / on_n as f64
        } else {
            f64::NAN
        },
    );
    s.verdict = Verdict::Unknown("只报不判：过关线（比原声直通多多少算不过）待人拍板".into());
    s
}

/// 字头窗（开口起 200 ms，开口前须停 ≥ 300 ms）里的用户有声谱帧数与其中误删的帧数。
fn onset_window_hits(c: &Ctx, f: &[(usize, bool)]) -> (usize, usize) {
    let sr = c.clip.sr as f64;
    let (win, quiet) = ((0.2 * sr) as isize, (0.3 * sr) as usize);
    let mut starts = Vec::new();
    let mut prev_end: Option<usize> = None;
    for &(a, e) in &c.clip.truth.owner {
        if prev_end.is_none_or(|p| a >= p + quiet) {
            starts.push(a as isize);
        }
        prev_end = Some(e);
    }
    let center = |k: usize| ((k + 1) * HOP) as isize - (NFFT / 2) as isize;
    let inside: Vec<bool> = f
        .iter()
        .map(|(k, x)| {
            let ck = center(*k);
            starts.iter().any(|&st| ck >= st && ck < st + win) && *x
        })
        .collect();
    let n = f
        .iter()
        .filter(|(k, _)| {
            starts
                .iter()
                .any(|&st| center(*k) >= st && center(*k) < st + win)
        })
        .count();
    (n, inside.iter().filter(|v| **v).count())
}

pub(super) fn v_swallow(c: &Ctx) -> Score {
    const ID: &str = "V-SWALLOW";
    let f = tsos_flags(c, c.out);
    let at: std::collections::BTreeMap<usize, bool> = f.iter().copied().collect();
    let hop = HOP as f64;
    let mut d = Vec::new();
    for &(start, _) in &c.clip.truth.owner {
        // 起点所在谱帧：中心 (k+1)·hop − nfft/2 ≥ start 的第一帧
        let k0 = ((start + NFFT / 2) as f64 / hop).ceil().max(1.0) as usize - 1;
        let mut n = 0usize;
        while at.get(&(k0 + n)).copied().unwrap_or(false) {
            n += 1;
        }
        d.push(n as f64 * hop / c.clip.sr as f64 * 1000.0);
    }
    if d.len() < 3 {
        return Score::unknown(
            ID,
            "ms",
            Better::Lower,
            format!("开口起点只有 {} 个", d.len()),
        );
    }
    let mut s = Score::judged(ID, quantile(&d, 0.9), "ms", Better::Lower, None, &[])
        .with_extra("onsets", d.len() as f64);
    s.verdict = Verdict::Unknown("只报不判：025 D3 先只报；按场景组合并 ≥ 30 个起点后再判".into());
    s
}

pub(super) fn v_leak(c: &Ctx) -> Score {
    const ID: &str = "V-LEAK";
    let x = th::LEAK.def("min_att_db");
    let win = (c.clip.sr as usize / MASK_FRAME).max(1); // 1 s 的掩码帧数
    let (own, itf) = (&c.masks.owner, &c.masks.interf);
    let nf = own.len().min(itf.len());
    let (mut n, mut leak) = (0usize, 0usize);
    // 1 秒窗按 100 ms 步长滑动：旁人独说跨在两个整秒之间也能检出（codex 中途 2 第 1 轮 P2 #3）；窗之间有重叠，读数是窗的占比
    let step = (win / 10).max(1);
    for w in (0..nf.saturating_sub(win) / step + usize::from(nf >= win))
        .map(|i| i * step..i * step + win)
    {
        // 「只有旁人说话的窗」：用户一帧都不在说，旁人说话的帧过半（音节之间的空隙照算在窗里）
        if own[w.clone()].iter().any(|v| *v)
            || itf[w.clone()].iter().filter(|v| **v).count() * 2 < win
        {
            continue;
        }
        let sel: Vec<bool> = (0..nf).map(|i| w.contains(&i)).collect();
        let (Some((po, _)), Some((pi, _))) =
            (power_over(c.out, &sel), power_over(&c.clip.mix, &sel))
        else {
            continue;
        };
        n += 1;
        leak += usize::from(db10(po) - db10(pi) > -x);
    }
    if n == 0 {
        return Score::unknown(ID, "%", Better::Lower, "没有只有旁人说话的整秒窗");
    }
    let mut s = Score::judged(
        ID,
        100.0 * leak as f64 / n as f64,
        "%",
        Better::Lower,
        None,
        &[],
    )
    .with_extra("windows", n as f64);
    s.verdict = Verdict::Unknown(format!("只报不判：压低不到 {x} dB 算漏过，X 待人拍板"));
    s
}

/// 把不超过 `gap` 帧的停顿填上（音节之间的空隙不算「不说话」）。
fn bridge(m: &[bool], gap: usize) -> Vec<bool> {
    let mut out = m.to_vec();
    let mut last: Option<usize> = None;
    for (i, &v) in m.iter().enumerate() {
        if v {
            if let Some(l) = last
                && i - l - 1 <= gap
            {
                out[l + 1..i].iter_mut().for_each(|x| *x = true);
            }
            last = Some(i);
        }
    }
    out
}

/// SI-SDR（dB）：est 投影到 reference 上的分量与残差之比。
pub fn si_sdr(est: &[f32], reference: &[f32]) -> Option<f64> {
    let dot: f64 = est
        .iter()
        .zip(reference)
        .map(|(a, b)| *a as f64 * *b as f64)
        .sum();
    let rr: f64 = reference.iter().map(|b| (*b as f64).powi(2)).sum();
    if rr <= 0.0 {
        return None;
    }
    let a = dot / rr;
    let (mut t, mut e) = (0.0f64, 0.0f64);
    for (x, r) in est.iter().zip(reference) {
        let tv = a * *r as f64;
        t += tv * tv;
        e += (*x as f64 - tv).powi(2);
    }
    // 输出全零：投影和残差都是 0，不是「完美重建」，而是用户被整段删掉 → 记成下限（codex 中途 2 第 1 轮 P1 #1）
    if t <= 0.0 {
        return Some(-SI_SDR_CAP);
    }
    Some(
        if e > 0.0 {
            db10(t) - db10(e)
        } else {
            SI_SDR_CAP
        }
        .clamp(-SI_SDR_CAP, SI_SDR_CAP),
    )
}

/// SI-SDR 读数的上下限（dB）：完美重建记 +200，全零输出记 −200（与 `db10` 的 1e−20 下限同量级）。
pub const SI_SDR_CAP: f64 = 200.0;

pub(super) fn v_pse_ovl(c: &Ctx) -> Score {
    const ID: &str = "V-PSE-OVL";
    let min_len = (0.5 * c.clip.sr as f64) as usize;
    // 先取两人真实同时有声的帧，再桥接这些重叠帧之间 300 ms 以内的停顿（各自先桥接会把轮流说话造成「重叠」，
    // codex 中途 2 第 2 轮 P1 #1）
    let gap = (0.3 * c.clip.sr as f64) as usize / MASK_FRAME;
    let raw: Vec<bool> = c
        .masks
        .owner
        .iter()
        .zip(&c.masks.interf)
        .map(|(a, b)| *a && *b)
        .collect();
    let ov = bridge(&raw, gap);
    let n = ov.len();
    let mut segs = Vec::new();
    let mut k = 0;
    while k < n {
        if ov[k] {
            let a = k;
            while k < n && ov[k] {
                k += 1;
            }
            let (s, e) = (
                a * MASK_FRAME,
                (k * MASK_FRAME).min(c.out.len()).min(c.clip.clean.len()),
            );
            if e > s && e - s >= min_len {
                segs.push((s, e));
            }
        } else {
            k += 1;
        }
    }
    let gains: Vec<f64> = segs
        .iter()
        .filter_map(|&(s, e)| {
            let r = &c.clip.clean[s..e];
            Some(si_sdr(&c.out[s..e], r)? - si_sdr(&c.clip.mix[s..e], r)?)
        })
        .collect();
    if gains.is_empty() {
        return Score::unknown(ID, "dB", Better::Higher, "没有 ≥ 0.5 s 的用户与旁人重叠段");
    }
    let fail = gains
        .iter()
        .filter(|g| **g < th::PSE_OVL.def("fail_below"))
        .count();
    let mut s = Score::judged(ID, mean(&gains), "dB", Better::Higher, None, &[])
        .with_extra("segments", gains.len() as f64)
        .with_extra("fail_rate", fail as f64 / gains.len() as f64);
    s.verdict = Verdict::Unknown("只报不判：025 D5 先出基线".into());
    s
}

#[cfg(test)]
#[path = "pse_tests.rs"]
mod tests;
