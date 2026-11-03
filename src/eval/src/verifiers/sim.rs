//! 模拟真值 verifier（只在模拟片段上 L3）：门控误关 · 误开 · 停顿关门 · 切换延迟。

use super::*;

/// 门控类 verifier 容许的拖尾（主讲人停下后多久内门控开着不算误开）。
pub const GATE_TOLERANCE_S: f64 = 0.2;

pub(super) struct HopTruth {
    owner: Vec<bool>,
    interf: Vec<bool>,
    /// 主讲人 ±GATE_TOLERANCE_S 内有声
    owner_near: Vec<bool>,
    gate: Vec<bool>,
    hop: usize,
}

pub(super) fn hop_truth(c: &Ctx, r: &RunOut) -> HopTruth {
    let hop = r.hop;
    let n = c.clip.len() / hop;
    let cover = |ivs: &[(usize, usize)], pad: usize| -> Vec<bool> {
        let mut acc = vec![0usize; n];
        for &(a, b) in ivs {
            let (a, b) = (a.saturating_sub(pad), (b + pad).min(n * hop));
            for (h, v) in acc.iter_mut().enumerate() {
                let (s, e) = (h * hop, (h + 1) * hop);
                let ov = e.min(b).saturating_sub(s.max(a));
                *v += ov;
            }
        }
        acc.iter().map(|&v| v * 2 >= hop).collect()
    };
    let tol = (GATE_TOLERANCE_S * c.clip.sr as f64) as usize;
    HopTruth {
        owner: cover(&c.clip.truth.owner, 0),
        interf: cover(&c.clip.truth.interf, 0),
        owner_near: cover(&c.clip.truth.owner, tol),
        gate: (0..n).map(|h| r.gate_at(h * hop + hop / 2)).collect(),
        hop,
    }
}

/// 主讲人「开口」：静默 ≥ 300 ms 后的音节起点（样本）。
pub(super) fn owner_onsets(c: &Ctx) -> Vec<usize> {
    let gap = (0.3 * c.clip.sr as f64) as usize;
    let mut out = Vec::new();
    let mut last: Option<usize> = None;
    for &(a, b) in &c.clip.truth.owner {
        if last.is_none_or(|e| a >= e + gap) {
            out.push(a);
        }
        last = Some(b);
    }
    out
}

pub(super) fn need_sim(
    c: &Ctx,
    id: &'static str,
    unit: &'static str,
) -> Result<HopTruth, Box<Score>> {
    if !c.clip.tracks {
        return Err(Box::new(Score::unknown(
            id,
            unit,
            Better::Lower,
            "没有逐轨分存，没有逐帧真值",
        )));
    }
    let Some(r) = c.run else {
        return Err(Box::new(Score::unknown(
            id,
            unit,
            Better::Lower,
            "没有系统运行记录",
        )));
    };
    Ok(hop_truth(c, r))
}

pub(super) fn v_sim_gate(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-GATE";
    let t = match need_sim(c, ID, "比例") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let act: Vec<usize> = (0..t.owner.len()).filter(|&h| t.owner[h]).collect();
    if act.len() < 20 {
        return Score::unknown(ID, "比例", Better::Lower, "主讲人有声 hop 不足 20 个");
    }
    let miss = act.iter().filter(|&&h| !t.gate[h]).count() as f64 / act.len() as f64;
    // 旁报：主讲人（含拖尾容许）与干扰者都不在的 hop 里门控开着的比例（噪声误开）
    let quiet: Vec<usize> = (0..t.owner.len())
        .filter(|&h| !t.owner_near[h] && !t.interf[h])
        .collect();
    let noise_open = if quiet.is_empty() {
        f64::NAN
    } else {
        quiet.iter().filter(|&&h| t.gate[h]).count() as f64 / quiet.len() as f64
    };
    let w = (0.2 * c.clip.sr as f64) as usize;
    let mut on_total = 0usize;
    let mut on_miss = 0usize;
    for s in owner_onsets(c) {
        for h in s / t.hop..((s + w) / t.hop).min(t.owner.len()) {
            if t.owner[h] {
                on_total += 1;
                on_miss += usize::from(!t.gate[h]);
            }
        }
    }
    let onset_miss = if on_total > 0 {
        on_miss as f64 / on_total as f64
    } else {
        0.0
    };
    let mut s = Score::judged(
        ID,
        miss,
        "比例",
        Better::Lower,
        Some(p.f_or("max", th::SIM_GATE.def("max"))),
        &[],
    )
    .with_extra("onset_miss", onset_miss)
    .with_extra("noise_open", noise_open)
    .with_extra("owner_hops", act.len() as f64);
    let omax = p.f_or("onset_max", th::SIM_GATE.def("onset_max"));
    if s.verdict.is_pass() && onset_miss > omax {
        s.verdict = Verdict::Fail;
        s.note = format!(
            "字头 200 ms 漏开 {:.0}% > {:.0}%",
            onset_miss * 100.0,
            omax * 100.0
        );
    }
    s
}

pub(super) fn v_sim_fa(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-FA";
    let t = match need_sim(c, ID, "比例") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let only: Vec<usize> = (0..t.owner.len())
        .filter(|&h| t.interf[h] && !t.owner_near[h])
        .collect();
    if only.len() < 20 {
        return Score::unknown(ID, "比例", Better::Lower, "只有干扰者说话的 hop 不足 20 个");
    }
    let fa = only.iter().filter(|&&h| t.gate[h]).count() as f64 / only.len() as f64;
    Score::judged(
        ID,
        fa,
        "比例",
        Better::Lower,
        Some(p.f_or("max", th::SIM_FA.def("max"))),
        &[],
    )
    .with_extra("interf_only_hops", only.len() as f64)
}

pub(super) fn v_sim_quiet(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-QUIET";
    let t = match need_sim(c, ID, "比例") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let quiet: Vec<usize> = (0..t.owner.len())
        .filter(|&h| !t.owner_near[h] && !t.interf[h])
        .collect();
    if quiet.len() < 20 {
        return Score::unknown(ID, "比例", Better::Lower, "没人说话的 hop 不足 20 个");
    }
    let open = quiet.iter().filter(|&&h| t.gate[h]).count() as f64 / quiet.len() as f64;
    Score::judged(
        ID,
        open,
        "比例",
        Better::Lower,
        Some(p.f_or("max", th::SIM_QUIET.def("max"))),
        &[],
    )
    .with_extra("quiet_hops", quiet.len() as f64)
}

pub(super) fn v_sim_swlat(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-SWLAT";
    let t = match need_sim(c, ID, "ms") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let ms_per_hop = t.hop as f64 * 1000.0 / c.clip.sr as f64;
    let mut lats = Vec::new();
    let mut already_open = 0usize;
    for s in owner_onsets(c) {
        let h0 = s / t.hop;
        if h0 == 0 || h0 >= t.gate.len() {
            continue;
        }
        if t.gate[h0 - 1] {
            already_open += 1;
            continue;
        }
        let lim = (h0 + 64).min(t.gate.len());
        let first = (h0..lim).find(|&h| t.gate[h]).unwrap_or(lim);
        lats.push((first - h0) as f64 * ms_per_hop);
    }
    if lats.len() < 3 {
        return Score::unknown(
            ID,
            "ms",
            Better::Lower,
            format!("门控关着时开口的起点只有 {} 个", lats.len()),
        );
    }
    Score::judged(
        ID,
        quantile(&lats, 0.9),
        "ms",
        Better::Lower,
        Some(p.f_or("max", th::SIM_SWLAT.def("max"))),
        &[],
    )
    .with_extra("onsets", lats.len() as f64)
    .with_extra("already_open", already_open as f64)
}
