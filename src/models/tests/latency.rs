//! 延迟账（D32 ≤ 45 ms · 规约 R13 · 本批硬约束 1）。不需要模型文件，CI 上也跑。逐项硬加，不认「重叠」推断。
//!
//! - 路由池里每个模型：攒 hop + 算法延迟（代码声明）+ 平台规划预算 ≤ 45 ms——**在提议的 IO 块（96 帧）下**
//! - 路由池里的模型同延迟（引擎按池内最大延迟补齐，放进一个慢的会拖慢所有场景）
//! - 按 009 原目标（IO 256 帧）是 47 ms、超 2 ms：钉成已知缺口，P3 实测（E-lat）裁决

use vdn_models::ModelKind;
use vdn_types::latency::planned_platform;
use vdn_types::{IO_BLOCK_009, IO_BLOCK_PROPOSED, LATENCY_BUDGET_MS, LatencyLedger};

fn full(kind: ModelKind, io_block: u32) -> LatencyLedger {
    let mut l = kind.latency_ledger();
    l.extend(planned_platform(io_block));
    l
}

#[test]
fn every_routed_model_fits_with_proposed_io_block() {
    for k in ModelKind::ALL {
        let l = full(k, IO_BLOCK_PROPOSED);
        if let Err(t) = l.check(LATENCY_BUDGET_MS) {
            panic!(
                "{k:?} 延迟账 {t:.2} ms 超 {LATENCY_BUDGET_MS} ms：\n{}",
                l.render()
            );
        }
    }
}

#[test]
fn model_part_is_32_ms_for_both() {
    // GTCRN 256+256、FE-M 160+352 样本 @16 kHz，都是 32 ms（= STFT 窗长 512）
    for k in ModelKind::ALL {
        assert!((k.latency_ledger().total_ms() - 32.0).abs() < 1e-9, "{k:?}");
    }
}

#[test]
fn route_pool_has_equal_latency() {
    let totals: Vec<f64> = ModelKind::ALL
        .iter()
        .map(|k| full(*k, IO_BLOCK_PROPOSED).total_ms())
        .collect();
    let (lo, hi) = totals
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), t| (a.min(*t), b.max(*t)));
    assert!(hi - lo < 1e-9, "池内延迟不一致：{totals:?}");
}

#[test]
fn ledger_numbers_are_pinned() {
    // 任何一项变了这条会红——改数字时同步改 specs/rust/architecture.md §3.7.6
    for k in ModelKind::ALL {
        let proposed = full(k, IO_BLOCK_PROPOSED);
        assert!(
            (proposed.total_ms() - 131.0 / 3.0).abs() < 1e-9,
            "{}",
            proposed.render()
        );
        assert!((proposed.budget_ms() - 35.0 / 3.0).abs() < 1e-9);
        // 已知缺口：009 原目标 IO 256 帧 → 47 ms，超预算
        let orig = full(k, IO_BLOCK_009);
        assert!((orig.total_ms() - 47.0).abs() < 1e-9, "{}", orig.render());
        assert!(orig.check(LATENCY_BUDGET_MS).is_err());
    }
}
