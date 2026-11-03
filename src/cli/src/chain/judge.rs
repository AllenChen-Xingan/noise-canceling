//! `chain-test` 的判据（V1–V4）与结束时打印的几行：只读 [`ChainRun`]，不跑链路。

use super::{ChainRun, LIMIT_150MS, RATE, SILENCE, SKIP};
use crate::verify::{
    best_lag, longest_silence, ms48, nonfinite, onset, pearson_at_lag, rms, test_tone,
};
/// 一条判据的结论。
pub struct ChainVerdict {
    pub id: &'static str,
    pub what: &'static str,
    pub pass: bool,
    pub detail: String,
}

impl std::fmt::Display for ChainVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}：{} · {}",
            self.id,
            self.what,
            if self.pass { "过" } else { "不过" },
            self.detail
        )
    }
}

/// 延迟分项一行。
pub fn latency_line(r: &ChainRun) -> String {
    format!(
        "L_total = {} 样本（{:.2} ms）= Pipeline {}（{:.2} ms，降噪块长 {}）+ 环余量 {}（{:.2} ms）",
        r.l_total,
        ms48(r.l_total),
        r.l_pipe,
        ms48(r.l_pipe),
        r.denoise_block,
        r.lead,
        ms48(r.lead)
    )
}

/// 计数一行。
pub fn counters_line(r: &ChainRun) -> String {
    let f = &r.fwd;
    format!(
        "计数：虚拟麦欠载 {} 块 / {} 样本 · 处理后欠载 {} 样本 · 硬切 {} · Raw 溢出 {} · Out 溢出 {} · 丢弃 {} · 重新对账 {} · 切原声 {} 次 · 降噪太慢 {} 次 · 跳过积压 {} 样本（送原声窗口 {} 次）· 降噪块数 {}",
        r.hal_underrun_blocks,
        r.hal_underrun_samples,
        f.proc_underrun,
        f.cuts,
        f.raw_overflow,
        f.out_overflow,
        f.discarded,
        f.resyncs,
        f.stale_switches,
        f.slow_episodes,
        r.worker.raw_skipped,
        f.gap_holds,
        r.worker.blocks
    )
}

/// 耗时一行（降噪线程）。
pub fn timing_line(r: &ChainRun) -> String {
    let w = &r.worker;
    let block_ms = ms48(r.denoise_block);
    format!(
        "耗时：预热 {} 块（第一块 {:.3} ms，之后最长 {:.3} ms；每周期 {} 块总耗时 p99.9 {:.3} ms）· 服务期单块最长 {:.3} ms、平均 {:.4} ms、超一块时长（{:.3} ms）{} 块 · 每周期总耗时最长 {:.3} ms、超周期时长 {} 次 · 醒来时 Raw 最多积 {} 样本（{:.2} ms）· 调度读回 {} 次、降级 {} 次 · 虚拟麦读相位 {} 样本（{:.2} ms，不在 L_total 里）· 实时提升失败 {} 个线程",
        r.warmup.blocks,
        r.warmup.first_ns as f64 / 1e6,
        r.warmup.max_after_first_ns as f64 / 1e6,
        r.warmup.period_blocks,
        r.warmup.p999_period_ns as f64 / 1e6,
        w.max_process_ns as f64 / 1e6,
        w.total_process_ns as f64 / 1e6 / w.blocks.max(1) as f64,
        block_ms,
        w.over_budget,
        w.max_period_ns as f64 / 1e6,
        w.over_period,
        w.max_raw_backlog,
        ms48(w.max_raw_backlog as usize),
        w.rt_checks,
        w.rt_downgrades,
        r.hal_offset,
        ms48(r.hal_offset),
        r.rt_failures
    )
}

/// V1：输出与「输入延迟 L_total」相关 ≥ 0.999，虚拟麦欠载 0。
pub fn judge_v1(r: &ChainRun) -> ChainVerdict {
    let corr = pearson_at_lag(&r.x, &r.y, r.l_total, SKIP).unwrap_or(0.0);
    let found = onset(&r.y, 0.05).and_then(|on| {
        best_lag(&r.x, &r.y, on.saturating_sub(600), on + 600, SKIP, 9600).map(|(lag, _)| lag)
    });
    // 欠载 0 是 V1 的一部分（010 第十一节）：直通时处理后的 = 延迟原声，欠载用原声补了相关也看不出来，必须单独判
    let pass = corr >= 0.999 && r.hal_underrun_samples == 0 && r.fwd.proc_underrun == 0;
    ChainVerdict {
        id: "V1",
        what: "直通链路逐样本对齐",
        pass,
        detail: format!(
            "相关 {corr:.6}（线 ≥ 0.999）· 实测延迟 {}（应为 {}）· 虚拟麦欠载 {} 样本 · 处理后欠载 {} 样本（两项线都 = 0）",
            found.map_or("未找到".to_owned(), |l| l.to_string()),
            r.l_total,
            r.hal_underrun_samples,
            r.fwd.proc_underrun
        ),
    }
}

/// V2：降噪死后 ≤ 150 ms 切原声；全程无 > 150 ms 静音。
pub fn judge_v2(r: &ChainRun) -> ChainVerdict {
    let switch = match (r.killed_at, r.fwd.last_stale_at) {
        (Some(k), Some(s)) if s >= k => Some((s - k) as usize),
        _ => None,
    };
    let (sil_at, sil_len) = longest_silence(&r.y, SILENCE, r.l_total + SKIP);
    let pass = switch.is_some_and(|s| s <= LIMIT_150MS) && sil_len <= LIMIT_150MS;
    ChainVerdict {
        id: "V2",
        what: "降噪进程死掉仍出声",
        pass,
        detail: format!(
            "切原声用时 {}（线 ≤ 150 ms）· 最长静音 {:.1} ms，起于 {:.1} ms（线 ≤ 150 ms）",
            switch.map_or("未切换".to_owned(), |s| format!("{:.1} ms", ms48(s))),
            ms48(sil_len),
            ms48(sil_at)
        ),
    }
}

/// V3：真模型链路出声：非静音、有限值、长度对齐。
pub fn judge_v3(r: &ChainRun) -> ChainVerdict {
    let tail = r.y.get(r.l_total + SKIP..).unwrap_or(&[]);
    let level = rms(tail);
    let bad = nonfinite(&r.y);
    let pass = level > 0.01 && bad == 0 && r.y.len() == r.x.len();
    ChainVerdict {
        id: "V3",
        what: "真模型链路出声",
        pass,
        detail: format!(
            "输出均方根 {level:.4}（线 > 0.01）· 非有限样本 {bad}（线 = 0）· 长度 {} / {}",
            r.y.len(),
            r.x.len()
        ),
    }
}

/// V4：文件里应是测试正弦延迟 `l_total`（同 V1 的相关判据；欠载看不到，由 vdn-capture 的计数报）。
pub fn judge_file(y: &[f32], l_total: usize, from: usize) -> ChainVerdict {
    let x = test_tone(RATE, y.len(), 0);
    let skip = SKIP.max(from);
    let corr = pearson_at_lag(&x, y, l_total, skip).unwrap_or(0.0);
    let found = if from == 0 {
        onset(y, 0.05).and_then(|on| {
            best_lag(&x, y, on.saturating_sub(600), on + 600, SKIP, 9600).map(|(lag, _)| lag)
        })
    } else {
        best_lag(
            &x,
            y,
            l_total.saturating_sub(600),
            l_total + 600,
            skip,
            9600,
        )
        .map(|(lag, _)| lag)
    };
    let (_, sil) = longest_silence(y, SILENCE, l_total + skip);
    let pass = corr >= 0.999;
    ChainVerdict {
        id: "V4",
        what: "两个真进程",
        pass,
        detail: format!(
            "相关 {corr:.6}（线 ≥ 0.999）· 实测延迟 {}（应为 {l_total}）· 最长静音 {:.1} ms",
            found.map_or("未找到".to_owned(), |l| l.to_string()),
            ms48(sil)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_capture::ForwarderStats;

    /// 直通时欠载用延迟原声补上，相关照样 1.0——V1 必须单独判欠载（补测揪出的那类问题）
    #[test]
    fn v1_fails_on_underrun_even_with_perfect_correlation() {
        let l = 2208;
        let x = test_tone(RATE, 48_000, 0);
        let mut y = vec![0.0; l];
        y.extend_from_slice(&x[..48_000 - l]);
        let mut r = ChainRun {
            x,
            y,
            l_total: l,
            l_pipe: 768,
            lead: 1440,
            denoise_block: 96,
            fwd: ForwarderStats::default(),
            hal_underrun_blocks: 0,
            hal_underrun_samples: 0,
            killed_at: None,
            worker: vdn_denoised::WorkerStats::default(),
            warmup: vdn_denoised::Warmup::default(),
            hal_offset: 144,
            rt_failures: 0,
        };
        assert!(judge_v1(&r).pass);
        r.fwd.proc_underrun = 96;
        let v = judge_v1(&r);
        assert!(!v.pass, "处理后欠载 96 还判过：{v}");
        r.fwd.proc_underrun = 0;
        r.hal_underrun_samples = 1;
        assert!(!judge_v1(&r).pass);
    }

    #[test]
    fn judge_file_on_synthetic() {
        let x = test_tone(RATE, 48_000, 0);
        let mut y = vec![0.0; 2303];
        y.extend_from_slice(&x[..48_000 - 2303]);
        assert!(judge_file(&y, 2303, 0).pass);
        assert!(!judge_file(&y, 2303 + 300, 0).pass);
    }
}
