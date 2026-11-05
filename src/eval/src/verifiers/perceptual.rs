//! 感知量 verifier（定标前一律 L0）：背景稳定 · 吞字头 · 压噪 · 邻桌 · 人声电平 · 音乐噪声 · 真空感 · 音色。

use super::*;

/// 主讲人与干扰者都不在（扩 `guard_frames` 帧）的帧。
fn quiet_frames(c: &Ctx, guard_frames: usize) -> Vec<bool> {
    let nf = c.masks.noise_only.len();
    let mut q = c.masks.noise_only.clone();
    for f in 0..nf {
        if c.masks.owner[f] || c.masks.interf[f] {
            let a = f.saturating_sub(guard_frames);
            let b = (f + guard_frames + 1).min(nf);
            for v in q.iter_mut().take(b).skip(a) {
                *v = false;
            }
        }
    }
    q
}

pub(super) fn v_bgstab(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-BGSTAB";
    let Some(nout) = c.noise_out else {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            "没有「只有噪音」的对照处理（真实录音的噪声不能逐样本重放）",
        );
    };
    let q = quiet_frames(c, 5);
    let a = frame_power(c.out, MASK_FRAME);
    let b = frame_power(nout, MASK_FRAME);
    let start = (p.f_or("from_s", th::BGSTAB.def("from_s")) * 100.0) as usize;
    let d: Vec<Option<f64>> = (0..q.len().min(a.len()).min(b.len()))
        .map(|f| (q[f] && f >= start && b[f] > 1e-14).then(|| db10(a[f]) - db10(b[f])))
        .collect();
    let (worst, n, _) = worst3(&d);
    // 025 B3：扣掉输入侧的同一差值（混合 − 噪音 + 干扰，主讲人混响尾巴落在停顿帧里），原声直通应 ≈ 0 dB。
    // 主值改不改待大总管定；先作旁报，并报 3 帧均值绝对值的 95 分位
    let bg: Vec<f32> = c.clip.noise.iter().zip(&c.clip.interf).map(|(x, y)| x + y).collect();
    let (mi, bi) = (frame_power(&c.clip.mix, MASK_FRAME), frame_power(&bg, MASK_FRAME));
    let dc: Vec<Option<f64>> = d
        .iter()
        .enumerate()
        .map(|(f, x)| {
            let e = (mi.get(f)? > &1e-14 && bi.get(f)? > &1e-14).then(|| db10(mi[f]) - db10(bi[f]))?;
            x.map(|v| v - e)
        })
        .collect();
    let (worst_c, _, p95_c) = worst3(&dc);
    if n < 30 {
        return Score::unknown(ID, "dB", Better::Lower, format!("可比的停顿帧只有 {n} 个"));
    }
    Score::judged(
        ID,
        worst.abs(),
        "dB",
        Better::Lower,
        Some(p.f_or("max", th::BGSTAB.def("max"))),
        th::BGSTAB.edges,
    )
    .with_extra("signed_db", worst)
    .with_extra("frames", n as f64)
    .with_extra("bias_corr_db", worst_c.abs())
    .with_extra("bias_corr_p95_db", p95_c)
}

/// 3 帧滑动平均里绝对值最大的那个（带符号）、可比的窗数、绝对值的 95 分位。
fn worst3(d: &[Option<f64>]) -> (f64, usize, f64) {
    let mut worst = 0.0f64;
    let mut all = Vec::new();
    for w in d.windows(3) {
        if let [Some(x), Some(y), Some(z)] = w {
            let m = (x + y + z) / 3.0;
            if m.abs() > worst.abs() {
                worst = m;
            }
            all.push(m.abs());
        }
    }
    let p95 = if all.is_empty() { f64::NAN } else { quantile(&all, 0.95) };
    (worst, all.len(), p95)
}

/// 输出投影到干净参考上的能量 ⟨o, c⟩² / ‖c‖²：只留下与参考相关的部分，不受不相关的残留噪声影响
///（怀疑派自审 P1：停顿帧估的残留噪声比说话时小约 6 dB——门控开着时混回的原声更多——扣不干净会把人声损失藏掉）。
pub(super) fn proj_energy(o: &[f32], c: &[f32]) -> (f64, f64) {
    let (mut oc, mut cc) = (0.0f64, 0.0f64);
    for (a, b) in o.iter().zip(c) {
        oc += *a as f64 * *b as f64;
        cc += *b as f64 * *b as f64;
    }
    let p = if cc > 0.0 { oc * oc / cc } else { 0.0 };
    (p, cc)
}

pub(super) fn v_onset(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-ONSET";
    const F: usize = 320;
    const W: usize = 800; // 50 ms
    let ivs = truth_from_clean(&c.clip.clean);
    let mut onsets = Vec::new();
    let mut last_end: Option<usize> = None;
    for &(a, b) in &ivs {
        if last_end.is_none_or(|e| a >= e + 5 * F) && a + W <= c.out.len() {
            onsets.push(a);
        }
        last_end = Some(b);
    }
    if onsets.len() < 3 {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            format!("字头只有 {} 个", onsets.len()),
        );
    }
    let loss = |eo: f64, ec: f64| -> f64 { (db10(ec) - db10(eo.max(ec * 1e-4))).min(40.0) };
    let per: Vec<f64> = onsets
        .iter()
        .map(|&s| {
            let (po, pc) = proj_energy(&c.out[s..s + W], &c.clip.clean[s..s + W]);
            loss(po, pc)
        })
        .collect();
    // 有声段：逐 20 ms 帧投影后合计（容许慢变的增益）
    let (mut ec, mut eo) = (0.0, 0.0);
    for &(a, b) in &ivs {
        let b = b.min(c.out.len());
        let mut t = a;
        while t + F <= b {
            let (po, pc) = proj_energy(&c.out[t..t + F], &c.clip.clean[t..t + F]);
            eo += po;
            ec += pc;
            t += F;
        }
    }
    let avg = loss(eo, ec);
    let excess = mean(&per) - avg;
    Score::judged(
        ID,
        excess,
        "dB",
        Better::Lower,
        Some(p.f_or("max", th::ONSET.def("max"))),
        th::ONSET.edges,
    )
    .with_extra("onsets", per.len() as f64)
    .with_extra("p90_onset_loss_db", quantile(&per, 0.9))
    .with_extra("voiced_loss_db", avg)
}

fn tnlr(c: &Ctx) -> Option<(f64, usize)> {
    let (po, n) = power_over(c.out, &c.masks.noise_only)?;
    let (pi, _) = power_over(&c.clip.mix, &c.masks.noise_only)?;
    (n >= 50 && pi > 1e-14).then(|| (db10(po) - db10(pi), n))
}

pub(super) fn v_nr(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-NR";
    let Some((t, n)) = tnlr(c) else {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            "纯噪声帧不足 0.5 s，或输入停顿是数字静音",
        );
    };
    let mut s =
        Score::judged(ID, t, "dB", Better::Lower, p.f("max"), &[]).with_extra("frames", n as f64);
    let own: Vec<bool> = (0..c.masks.owner.len())
        .map(|f| c.masks.owner[f] && !c.masks.interf[f])
        .collect();
    if let (Some((so, _)), Some((si, _)), Some((no, _)), Some((ni, _))) = (
        power_over(c.out, &own),
        power_over(&c.clip.mix, &own),
        power_over(c.out, &c.masks.noise_only),
        power_over(&c.clip.mix, &c.masks.noise_only),
    ) {
        let snr = |s: f64, n: f64| db10((s - n).max(s * 1e-4)) - db10(n.max(1e-14));
        // 「简化 SNRI」（025 B1 改名）：说话帧与停顿帧的能量差，不是逐频带的标准 SNRI
        s = s.with_extra("snri_db", snr(so, no) - snr(si, ni));
    }
    // 025 B1 旁报：输入低于 G.160 `cft_lvl`（−48 dBov）的停顿帧比例，及排除它们后的 TNLR；判定不变
    let lim = th::NR.def("low_dbov");
    let fp = frame_power(&c.clip.mix, MASK_FRAME);
    let low: Vec<bool> = (0..c.masks.noise_only.len())
        .map(|f| c.masks.noise_only[f] && fp.get(f).is_some_and(|p| db10(*p) < lim))
        .collect();
    let nl = low.iter().filter(|v| **v).count();
    s = s.with_extra("low_frac", nl as f64 / n.max(1) as f64);
    let keep: Vec<bool> = (0..low.len()).map(|f| c.masks.noise_only[f] && !low[f]).collect();
    if let (Some((po, k)), Some((pi, _))) = (power_over(c.out, &keep), power_over(&c.clip.mix, &keep))
        && k >= 50
        && pi > 1e-14
    {
        s = s.with_extra("tnlr_excl_low_db", db10(po) - db10(pi));
    }
    s
}

pub(super) fn v_intf(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-INTF";
    let (Some((po, n)), Some((pi, _))) = (
        power_over(c.out, &c.masks.interf_only),
        power_over(&c.clip.mix, &c.masks.interf_only),
    ) else {
        return Score::unknown(ID, "dB", Better::Lower, "没有只有干扰者说话的帧");
    };
    if n < 50 {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            format!("只有干扰者的帧只有 {n} 个"),
        );
    }
    let mut s = Score::judged(
        ID,
        db10(po) - db10(pi),
        "dB",
        Better::Lower,
        p.f("max"),
        &[],
    )
    .with_extra("frames", n as f64);
    // 025 B2 旁报：两边都扣掉停顿帧里的残留底噪后再比（只看旁人人声本身被压了多少）；主值仍按混合信号做分母
    if let (Some((no, _)), Some((ni, _))) = (
        power_over(c.out, &c.masks.noise_only),
        power_over(&c.clip.mix, &c.masks.noise_only),
    ) && pi > ni
    {
        s = s.with_extra("minus_floor_db", db10((po - no).max(1e-14)) - db10(pi - ni));
    }
    s
}

pub(super) fn v_splvl(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SPLVL";
    let own: Vec<bool> = (0..c.masks.owner.len())
        .map(|f| c.masks.owner[f] && !c.masks.interf[f])
        .collect();
    let (mut ps, mut pc, mut n) = (0.0, 0.0, 0usize);
    for (f, &m) in own.iter().enumerate() {
        let (a, b) = (f * MASK_FRAME, (f + 1) * MASK_FRAME);
        if m && b <= c.out.len() {
            let (po, cc) = proj_energy(&c.out[a..b], &c.clip.clean[a..b]);
            ps += po;
            pc += cc;
            n += 1;
        }
    }
    if n < 50 || pc <= 1e-14 {
        return Score::unknown(ID, "dB", Better::Higher, "主讲人单独说话的帧不足 0.5 s");
    }
    let ps = ps.max(pc * 1e-4);
    let d = (db10(ps) - db10(pc)).min(0.0);
    Score::judged(
        ID,
        d,
        "dB",
        Better::Higher,
        Some(p.f_or("min", th::SPLVL.def("min"))),
        th::SPLVL.edges,
    )
}

fn noise_spec_frames(c: &Ctx, sg: &Spectrogram) -> Vec<usize> {
    let q = &c.masks.noise_only;
    (0..sg.frames.len())
        .filter(|&k| {
            sg.mask_frame(k)
                .is_some_and(|m| m >= 2 && m + 2 < q.len() && (m - 2..=m + 2).all(|f| q[f]))
        })
        .collect()
}

pub(super) fn v_music(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-MUSIC";
    let so = Spectrogram::new(c.out, 512, 256);
    let si = Spectrogram::new(&c.clip.mix, 512, 256);
    let ks = noise_spec_frames(c, &so);
    if ks.len() < 30 {
        return Score::unknown(
            ID,
            "×",
            Better::Lower,
            format!("纯噪声谱帧只有 {} 个", ks.len()),
        );
    }
    let (b0, b1) = (4usize, 192usize);
    // 每个频点先除以它在停顿帧上的均值再合并：否则合并后的峰度量到的主要是谱倾斜（怀疑派自审 P1：白噪 9、粉噪 112）
    let collect = |sg: &Spectrogram| -> Vec<f64> {
        let mut out = Vec::with_capacity(ks.len() * (b1 - b0 + 1));
        for b in b0..=b1 {
            let m = ks.iter().map(|&k| sg.frames[k][b] as f64).sum::<f64>() / ks.len() as f64;
            if m > 0.0 {
                out.extend(ks.iter().map(|&k| sg.frames[k][b] as f64 / m));
            }
        }
        out
    };
    let ki = kurtosis(&collect(&si));
    let ko = kurtosis(&collect(&so));
    if !ki.is_finite() || !ko.is_finite() || ki <= 0.0 {
        return Score::unknown(
            ID,
            "×",
            Better::Lower,
            "峰度算不出（输入或输出在停顿里全零）",
        );
    }
    // 孤立亮点
    let db = |k: usize, b: usize| 10.0 * (so.frames[k][b] as f64).max(1e-20).log10();
    let mut peaks = 0usize;
    let set: std::collections::HashSet<usize> = ks.iter().copied().collect();
    for &k in &ks {
        if !(set.contains(&(k.wrapping_sub(1))) && set.contains(&(k + 1))) {
            continue;
        }
        for b in b0 + 1..b1 {
            let v = db(k, b);
            let nb = [db(k - 1, b), db(k + 1, b), db(k, b - 1), db(k, b + 1)];
            if nb.iter().any(|&x| x > v - 6.0) {
                continue;
            }
            let mut hood: Vec<f64> = Vec::with_capacity(9);
            for dk in [k - 1, k, k + 1] {
                for dbin in [b - 1, b, b + 1] {
                    hood.push(db(dk, dbin));
                }
            }
            if v >= quantile(&hood, 0.5) + 12.0 {
                peaks += 1;
            }
        }
    }
    let secs = ks.len() as f64 * 256.0 / c.clip.sr as f64;
    Score::judged(
        ID,
        ko / ki,
        "×",
        Better::Lower,
        Some(p.f_or("max", th::MUSIC.def("max"))),
        th::MUSIC.edges,
    )
    .with_extra("kurt_in", ki)
    .with_extra("kurt_out", ko)
    .with_extra("isolated_peaks_per_s", peaks as f64 / secs)
    // 025 C1 旁报：停顿里的残留噪声比输出人声的主动电平低多少 dB（残留很低时音乐噪声未必听得见；免判门限待人耳校准）
    .with_extra(
        "resid_below_speech_db",
        match (
            crate::analysis::active_level(c.out, c.clip.sr),
            power_over(c.out, &c.masks.noise_only),
        ) {
            (Some((lvl, _)), Some((pn, _))) if pn > 0.0 => lvl - db10(pn),
            _ => f64::NAN,
        },
    )
}

pub(super) fn v_vacuum(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-VACUUM";
    let Some((t, _)) = tnlr(c) else {
        return Score::unknown(ID, "dB", Better::Higher, "纯噪声帧不足 0.5 s");
    };
    Score::judged(
        ID,
        t,
        "dB",
        Better::Higher,
        Some(p.f_or("min", th::VACUUM.def("min"))),
        &[],
    )
}

pub(super) fn v_timbre(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-TIMBRE";
    const BANDS: [(f64, f64); 4] = [
        (100.0, 1000.0),
        (1000.0, 2000.0),
        (2000.0, 4000.0),
        (4000.0, 7900.0),
    ];
    let so = Spectrogram::new(c.out, 512, 256);
    let sc = Spectrogram::new(&c.clip.clean, 512, 256);
    let own = |m: usize| m < c.masks.owner.len() && c.masks.owner[m] && !c.masks.interf[m];
    let speech: Vec<usize> = (0..so.frames.len())
        .filter(|&k| so.mask_frame(k).is_some_and(own))
        .collect();
    let quiet = noise_spec_frames(c, &so);
    if speech.len() < 50 {
        return Score::unknown(ID, "dB", Better::Lower, "主讲人有声谱帧不足");
    }
    let sr = c.clip.sr;
    // 上沿越过奈奎斯特频率的带不计（#110：否则空带读成 −200 dB、拖偏相对总电平）
    let keep: Vec<usize> = (0..BANDS.len())
        .filter(|&i| BANDS[i].1 <= sr as f64 / 2.0)
        .collect();
    if keep.len() < 3 {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            format!(
                "采样率 {sr} Hz：奈奎斯特以下只剩 {} 个完整频带，少于 3 个",
                keep.len()
            ),
        );
    }
    let band_pow = |sg: &Spectrogram, ks: &[usize]| -> [f64; 4] {
        let mut out = [0.0; 4];
        if ks.is_empty() {
            return out;
        }
        for &k in ks {
            for (b, v) in sg.frames[k].iter().enumerate() {
                let hz = sg.bin_hz(b, sr);
                for (i, (lo, hi)) in BANDS.iter().enumerate() {
                    if hz >= *lo && hz < *hi {
                        out[i] += *v as f64;
                    }
                }
            }
        }
        out.map(|v| v / ks.len() as f64)
    };
    let pc = band_pow(&sc, &speech);
    let po = band_pow(&so, &speech);
    let pn = band_pow(&so, &quiet);
    let po: Vec<f64> = po
        .iter()
        .zip(&pn)
        .map(|(o, n)| (o - n).max(0.1 * o).max(1e-20))
        .collect();
    let tot_c: f64 = keep.iter().map(|&i| pc[i]).sum();
    let tot_o: f64 = keep.iter().map(|&i| po[i]).sum();
    if tot_c <= 0.0 {
        return Score::unknown(ID, "dB", Better::Lower, "干净参考全零");
    }
    let dev: Vec<(usize, f64)> = keep
        .iter()
        .map(|&i| (i, (db10(po[i]) - db10(tot_o)) - (db10(pc[i]) - db10(tot_c))))
        .collect();
    let worst = dev
        .iter()
        .map(|(_, d)| *d)
        .fold(0.0f64, |a, b| if b.abs() > a.abs() { b } else { a });
    const NAMES: [&str; 4] = ["dev_0_1k", "dev_1_2k", "dev_2_4k", "dev_4_8k"];
    dev.iter().fold(
        Score::judged(
            ID,
            worst.abs(),
            "dB",
            Better::Lower,
            Some(p.f_or("max", th::TIMBRE.def("max"))),
            th::TIMBRE.edges,
        ),
        |s, (i, d)| s.with_extra(NAMES[*i], *d),
    )
}
