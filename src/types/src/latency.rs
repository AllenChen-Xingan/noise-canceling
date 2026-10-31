//! 延迟账（D32：端到端附加延迟 ≤ 45 ms；规约 §5 R13；ADR 005 是延迟账的正本）。
//!
//! 每个部件声明自己占多少（算法延迟 + 缓冲），engine 汇总成 [`LatencyLedger`]，测试断言总数 ≤ [`LATENCY_BUDGET_MS`]。
//! 还没写进代码的部件（重采样、IO、跨进程环、HAL）先按 009 延迟账「目标上限」一列记成 [`Basis::Budget`]，
//! P3 脉冲法实测后改成 [`Basis::Measured`]。
//!
//! 判断回路（感知、策略，约 0.5 s）只改参数、不碰声音，**不进延迟账**（本批硬约束 1）。

use core::fmt::Write as _;

/// D32。
pub const LATENCY_BUDGET_MS: f64 = 45.0;

/// 这一项的数字从哪来。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Basis {
    /// 代码里声明的（`Model::latency_samples()`、hop 等），可由测试核对
    Declared,
    /// 规划预算：部件还没写，按 009 `decision.md` 延迟账「目标上限」记【推】
    Budget,
    /// 实测（P3 脉冲法）
    Measured,
}

impl Basis {
    fn as_str(self) -> &'static str {
        match self {
            Basis::Declared => "代码声明",
            Basis::Budget => "规划预算",
            Basis::Measured => "实测",
        }
    }
}

/// 一项延迟。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatencyItem {
    pub component: &'static str,
    pub samples: u32,
    /// `samples` 的采样率
    pub rate: u32,
    pub basis: Basis,
    /// 这段等待与另一项**同时发生**、不另外计时（填被并入的那一项的 `component`）。要写清成立条件
    pub absorbed_by: Option<&'static str>,
    pub note: &'static str,
}

impl LatencyItem {
    pub const fn new(component: &'static str, samples: u32, rate: u32, basis: Basis, note: &'static str) -> Self {
        Self {
            component,
            samples,
            rate,
            basis,
            absorbed_by: None,
            note,
        }
    }

    pub fn ms(&self) -> f64 {
        if self.rate == 0 {
            0.0
        } else {
            f64::from(self.samples) * 1000.0 / f64::from(self.rate)
        }
    }
}

/// 延迟账。构造期用（非实时），可分配。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LatencyLedger {
    items: Vec<LatencyItem>,
}

impl LatencyLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, item: LatencyItem) -> &mut Self {
        self.items.push(item);
        self
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = LatencyItem>) -> &mut Self {
        self.items.extend(items);
        self
    }

    pub fn items(&self) -> &[LatencyItem] {
        &self.items
    }

    /// 合计：并入别项的不重复计。
    pub fn total_ms(&self) -> f64 {
        self.items.iter().filter(|i| i.absorbed_by.is_none()).map(LatencyItem::ms).sum()
    }

    /// 逐项硬加（不认任何「同时发生」）：最保守的上界。
    pub fn naive_total_ms(&self) -> f64 {
        self.items.iter().map(LatencyItem::ms).sum()
    }

    /// 合计 ≤ 预算返回 `Ok(合计)`，否则 `Err(合计)`。并入项指向的部件必须在账上，否则当作硬加。
    pub fn check(&self, budget_ms: f64) -> Result<f64, f64> {
        let dangling: f64 = self
            .items
            .iter()
            .filter(|i| {
                i.absorbed_by
                    .is_some_and(|c| !self.items.iter().any(|j| j.component == c && j.absorbed_by.is_none()))
            })
            .map(LatencyItem::ms)
            .sum();
        let t = self.total_ms() + dangling;
        if t <= budget_ms + 1e-9 { Ok(t) } else { Err(t) }
    }

    /// Markdown 表，测试失败时打印、也可贴进文档。
    pub fn render(&self) -> String {
        let mut s = String::from("| 部件 | 样本 @ 采样率 | ms | 依据 | 备注 |\n|---|---|---|---|---|\n");
        for i in &self.items {
            let ms = match i.absorbed_by {
                Some(c) => format!("（{:.2}，并入 {c}）", i.ms()),
                None => format!("{:.2}", i.ms()),
            };
            let _ = writeln!(
                s,
                "| {} | {} @ {} | {} | {} | {} |",
                i.component,
                i.samples,
                i.rate,
                ms,
                i.basis.as_str(),
                i.note
            );
        }
        let _ = writeln!(
            s,
            "| **合计** | | **{:.2}**（逐项硬加 {:.2}） | | 预算 {LATENCY_BUDGET_MS} |",
            self.total_ms(),
            self.naive_total_ms()
        );
        s
    }
}

/// 还没写进代码的平台部分（P3 / P4），按 009 `decision.md` 延迟账「目标上限」一列。全部【推】，P3 脉冲法实测后替换。
///
/// 「麦克风 IO 缓冲」记为并入 `hop-buffer`：IO 块（256 帧 @48k = 5.33 ms）整除模型 hop（16 ms = 768 帧 @48k）且对齐时，
/// 等 IO 块凑齐与等模型 hop 凑齐结束于同一时刻，不另外计时【推】。设备不按 256 帧给、或块边界与 hop 不对齐时不成立，
/// 要按 [`LatencyLedger::naive_total_ms`] 算——这一条由 P3 实测裁决。
pub fn planned_platform() -> [LatencyItem; 5] {
    const IO: u32 = 48_000;
    [
        LatencyItem::new("resample-down", 96, IO, Basis::Budget, "48→16 kHz，目标每跳 ≤ 2 ms（009）"),
        LatencyItem::new("resample-up", 96, IO, Basis::Budget, "16→48 kHz，目标每跳 ≤ 2 ms（009）"),
        LatencyItem {
            absorbed_by: Some("hop-buffer"),
            ..LatencyItem::new("io-input", 256, IO, Basis::Budget, "256 帧；整除并对齐模型 hop 时与攒 hop 同时发生")
        },
        LatencyItem::new("ipc-ring", 144, IO, Basis::Budget, "跨进程环余量 ≤ 3 ms（009）"),
        LatencyItem::new("hal-output", 128, IO, Basis::Budget, "HAL 传输余量 128 帧（009）"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_and_totals() {
        let mut l = LatencyLedger::new();
        l.push(LatencyItem::new("hop-buffer", 256, 16_000, Basis::Declared, ""))
            .push(LatencyItem::new("model", 256, 16_000, Basis::Declared, ""));
        assert_eq!(l.total_ms(), 32.0);
        assert_eq!(l.check(LATENCY_BUDGET_MS), Ok(32.0));
        assert!(l.check(30.0).is_err());
    }

    #[test]
    fn absorbed_items_count_once_only_if_target_present() {
        let mut l = LatencyLedger::new();
        l.extend(planned_platform());
        // hop-buffer 不在账上：io-input 的「并入」不成立，按硬加
        assert!((l.check(100.0).unwrap() - l.naive_total_ms()).abs() < 1e-9);
        l.push(LatencyItem::new("hop-buffer", 256, 16_000, Basis::Declared, ""));
        assert!(l.check(100.0).unwrap() < l.naive_total_ms());
    }

    #[test]
    fn planned_platform_matches_009_targets() {
        let ms: Vec<f64> = planned_platform().iter().map(LatencyItem::ms).collect();
        let want = [2.0, 2.0, 16.0 / 3.0, 3.0, 8.0 / 3.0];
        for (a, b) in ms.iter().zip(want) {
            assert!((a - b).abs() < 1e-9, "{a} vs {b}");
        }
    }

    #[test]
    fn render_mentions_every_item() {
        let mut l = LatencyLedger::new();
        l.extend(planned_platform());
        let r = l.render();
        for i in l.items() {
            assert!(r.contains(i.component));
        }
    }
}
