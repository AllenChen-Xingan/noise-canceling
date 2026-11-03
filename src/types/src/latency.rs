//! 延迟账（D32：端到端附加延迟 ≤ 45 ms；规约 §5 R13；ADR 005 是延迟账的正本）。
//!
//! 每个部件声明自己占多少（算法延迟 + 缓冲），engine 汇总成 [`LatencyLedger`]，测试断言总数 ≤ [`LATENCY_BUDGET_MS`]。
//! 还没写进代码的部件（重采样、IO、跨进程环、HAL）先按 009 延迟账「目标上限」一列记成 [`Basis::Budget`]，
//! P3 脉冲法实测后改成 [`Basis::Measured`]。**逐项硬加，不认任何「两段等待重叠」的推断**（自审：那条推断对 FE-M 不成立）。
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
    pub note: &'static str,
}

impl LatencyItem {
    pub const fn new(
        component: &'static str,
        samples: u32,
        rate: u32,
        basis: Basis,
        note: &'static str,
    ) -> Self {
        Self {
            component,
            samples,
            rate,
            basis,
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

    /// 合计（逐项硬加）。
    pub fn total_ms(&self) -> f64 {
        self.items.iter().map(LatencyItem::ms).sum()
    }

    /// 合计里有多少来自规划预算（还没实测、还没写成代码）。
    pub fn budget_ms(&self) -> f64 {
        self.items
            .iter()
            .filter(|i| i.basis == Basis::Budget)
            .map(LatencyItem::ms)
            .sum()
    }

    /// 合计 ≤ 预算返回 `Ok(合计)`，否则 `Err(合计)`。
    pub fn check(&self, budget_ms: f64) -> Result<f64, f64> {
        let t = self.total_ms();
        if t <= budget_ms + 1e-9 { Ok(t) } else { Err(t) }
    }

    /// Markdown 表，测试失败时打印、也可贴进文档。
    pub fn render(&self) -> String {
        let mut s =
            String::from("| 部件 | 样本 @ 采样率 | ms | 依据 | 备注 |\n|---|---|---|---|---|\n");
        for i in &self.items {
            let _ = writeln!(
                s,
                "| {} | {} @ {} | {:.2} | {} | {} |",
                i.component,
                i.samples,
                i.rate,
                i.ms(),
                i.basis.as_str(),
                i.note
            );
        }
        let _ = writeln!(
            s,
            "| **合计** | | **{:.2}**（其中规划预算 {:.2}） | | 预算 {LATENCY_BUDGET_MS} |",
            self.total_ms(),
            self.budget_ms()
        );
        s
    }
}

/// 009 延迟账「目标上限」里的麦克风 IO 块：256 帧 @48k（5.33 ms）。按它逐项硬加，合计 47 ms，**超 45 ms 2 ms**。
pub const IO_BLOCK_009: u32 = 256;

/// 提议的 IO 块：96 帧 @48k（2 ms）【推】。96 是两个模型 hop（GTCRN 768、FE-M 480 帧 @48k）的最大公约数，
/// 回调与两种 hop 都对齐；按它逐项硬加 43.67 ms，留 1.33 ms 余量。代价：每秒约 500 次回调（多唤醒、多一点 CPU 与 xrun 风险），
/// 设备 / 蓝牙不一定给这么小的块——P3 实测（E-lat）后定，不行走 `decision.md` ⟨拍板 6⟩。
pub const IO_BLOCK_PROPOSED: u32 = 96;

/// 还没写进代码的平台部分（P3 / P4），按 009 `decision.md` 延迟账「目标上限」一列；`io_block` = 麦克风 IO 块（帧 @48k）。
/// 全部【推】，P3 脉冲法实测后替换。
pub fn planned_platform(io_block: u32) -> [LatencyItem; 5] {
    const IO: u32 = crate::IO_RATE;
    [
        LatencyItem::new(
            "resample-down",
            96,
            IO,
            Basis::Budget,
            "48→16 kHz，目标每跳 ≤ 2 ms（009）",
        ),
        LatencyItem::new(
            "resample-up",
            96,
            IO,
            Basis::Budget,
            "16→48 kHz，目标每跳 ≤ 2 ms（009）",
        ),
        LatencyItem::new("io-input", io_block, IO, Basis::Budget, "麦克风 IO 块"),
        LatencyItem::new(
            "ipc-ring",
            144,
            IO,
            Basis::Budget,
            "跨进程环余量 ≤ 3 ms（009）",
        ),
        LatencyItem::new(
            "hal-output",
            128,
            IO,
            Basis::Budget,
            "HAL 传输余量 128 帧（009）",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_and_totals() {
        let mut l = LatencyLedger::new();
        l.push(LatencyItem::new(
            "hop-buffer",
            256,
            crate::MODEL_RATE,
            Basis::Declared,
            "",
        ))
        .push(LatencyItem::new("model", 256, crate::MODEL_RATE, Basis::Declared, ""));
        assert_eq!(l.total_ms(), 32.0);
        assert_eq!(l.budget_ms(), 0.0);
        assert_eq!(l.check(LATENCY_BUDGET_MS), Ok(32.0));
        assert!(l.check(30.0).is_err());
    }

    #[test]
    fn planned_platform_matches_009_targets() {
        let ms: Vec<f64> = planned_platform(IO_BLOCK_009)
            .iter()
            .map(LatencyItem::ms)
            .collect();
        let want = [2.0, 2.0, 16.0 / 3.0, 3.0, 8.0 / 3.0];
        for (a, b) in ms.iter().zip(want) {
            assert!((a - b).abs() < 1e-9, "{a} vs {b}");
        }
    }

    #[test]
    fn proposed_io_block_divides_both_model_hops() {
        // GTCRN hop 256、FE-M hop 160 @16 kHz → ×3 到 48 kHz
        for hop_io in [256 * 3, 160 * 3] {
            assert_eq!(hop_io % IO_BLOCK_PROPOSED, 0);
        }
    }

    #[test]
    fn render_mentions_every_item() {
        let mut l = LatencyLedger::new();
        l.extend(planned_platform(IO_BLOCK_PROPOSED));
        let r = l.render();
        for i in l.items() {
            assert!(r.contains(i.component));
        }
    }
}
