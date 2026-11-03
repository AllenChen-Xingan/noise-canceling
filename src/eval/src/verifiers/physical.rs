//! 物理量 verifier（判事实，012 §2.1 直接 L3）：延迟账 · 最坏一帧 · 数值失效 · 静音 · 旁路切换。

use super::*;

pub(super) fn v_lat(c: &Ctx) -> Score {
    const ID: &str = "V-LAT";
    let Some(r) = c.run else {
        return Score::unknown(ID, "ms", Better::Lower, "没有系统运行记录");
    };
    let sr = c.clip.sr as f64;
    let Some(lag) = xcorr_lag(&c.clip.mix, &r.raw, 2048, 32_000) else {
        return Score::unknown(ID, "ms", Better::Lower, "输入太弱，互相关找不到峰");
    };
    let ok_algo = lag.abs_diff(r.latency) <= 1;
    let mut s = Score::judged(
        ID,
        r.ledger_total_ms,
        "ms",
        Better::Lower,
        Some(BUDGET_MS),
        &[],
    )
    .with_extra("measured_algo_ms", lag as f64 * 1000.0 / sr)
    .with_extra("declared_algo_ms", r.latency as f64 * 1000.0 / sr);
    if !ok_algo {
        s.verdict = Verdict::Fail;
        s.note = format!(
            "实测算法延迟 {lag} 样本 ≠ 声明 {} 样本（延迟账与实际不符）",
            r.latency
        );
    } else {
        s.note =
            "账面 = 引擎延迟账 + 平台预算（IO 块 96 帧）；平台项是预算，P3 实测前只证明账面不超"
                .into();
    }
    s
}

pub(super) fn v_frame(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-FRAME";
    let Some(r) = c.run else {
        return Score::unknown(ID, "×周期", Better::Lower, "没有系统运行记录");
    };
    if r.engine_times.is_empty() {
        return Score::unknown(ID, "×周期", Better::Lower, "没有计时");
    }
    let period = r.hop as f64 / c.clip.sr as f64;
    let t: Vec<f64> = r
        .engine_times
        .iter()
        .map(|d| d.as_secs_f64() / period)
        .collect();
    let max = t.iter().cloned().fold(0.0, f64::max);
    let s = Score::judged(
        ID,
        max,
        "×周期",
        Better::Lower,
        Some(p.f_or("max", th::FRAME.def("max"))),
        &[],
    )
    .with_extra("p999", quantile(&t, 0.999));
    if s.verdict.is_fail() && (c.ci || cfg!(debug_assertions)) {
        let why = if c.ci {
            "CI 虚拟机计时不可信（规约 §7 G10 放 L4）"
        } else {
            "debug 构建计时偏慢，用 --release 重跑"
        };
        return s.demote_unknown(why);
    }
    s
}

pub(super) fn v_nan(c: &Ctx) -> Score {
    let bad = c.out.iter().filter(|v| !v.is_finite()).count();
    let huge = c
        .out
        .iter()
        .filter(|v| v.is_finite() && v.abs() > 4.0)
        .count();
    Score::judged(
        "V-NAN",
        (bad + huge) as f64,
        "个",
        Better::Lower,
        Some(0.0),
        &[],
    )
    .with_extra("nonfinite", bad as f64)
    .with_extra("over_4", huge as f64)
}

pub(super) fn v_mute(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-MUTE";
    let x = &c.clip.mix;
    let mut start = x.len();
    while start > 0 && x[start - 1] == 0.0 {
        start -= 1;
    }
    let settle = (p.f_or("settle_s", th::MUTE.def("settle_s")) * c.clip.sr as f64) as usize;
    let from = start + settle;
    if x.len() < from + c.clip.sr as usize / 4 {
        return Score::unknown(
            ID,
            "dBFS",
            Better::Lower,
            "输入末尾没有 ≥ 0.25 s 的数字静音",
        );
    }
    let seg = &c.out[from..x.len().min(c.out.len())];
    let (mn, mx) = seg
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), &v| {
            (a.min(v), b.max(v))
        });
    let p2p = (mx - mn) as f64;
    let v = if p2p.is_finite() {
        20.0 * p2p.max(1e-12).log10()
    } else {
        f64::INFINITY
    };
    Score::judged(
        ID,
        v,
        "dBFS",
        Better::Lower,
        Some(p.f_or("max_dbfs", th::MUTE.def("max_dbfs"))),
        &[],
    )
    .with_extra("silence_s", (x.len() - start) as f64 / c.clip.sr as f64)
}

pub(super) fn v_switch(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SWITCH";
    let Some(r) = c.run else {
        return Score::unknown(ID, "×", Better::Lower, "没有系统运行记录");
    };
    let mut points = Vec::new();
    for h in 1..r.bypass.len() {
        if r.bypass[h] != r.bypass[h - 1] {
            // 切换发生在原始时间轴 h·hop，对齐后减去 latency
            if let Some(t) = (h * r.hop).checked_sub(r.latency) {
                points.push(t);
            }
        }
    }
    if points.is_empty() {
        return Score::unknown(ID, "×", Better::Lower, "本次没有发生送原声切换");
    }
    let dx: Vec<f64> = c
        .clip
        .mix
        .windows(2)
        .map(|w| (w[1] - w[0]).abs() as f64)
        .collect();
    let refd = quantile(&dx, 0.999).max(1e-6);
    let mut worst = 0.0f64;
    for &t in &points {
        let a = t.saturating_sub(64).max(1);
        let b = (t + 64).min(c.out.len());
        for i in a..b {
            worst = worst.max((c.out[i] - c.out[i - 1]).abs() as f64);
        }
    }
    Score::judged(
        ID,
        worst / refd,
        "×",
        Better::Lower,
        Some(p.f_or("max", th::SWITCH.def("max"))),
        &[],
    )
    .with_extra("switches", points.len() as f64)
}
