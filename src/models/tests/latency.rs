//! 延迟账（D32 ≤ 45 ms · 规约 R13 · 本批硬约束 1）。不需要模型文件，CI 上也跑。
//!
//! - 路由池里每个模型：攒 hop + 算法延迟 + 平台规划预算（009 目标值）≤ 45 ms
//! - 路由池里的模型同延迟（引擎按池内最大延迟补齐，放进一个慢的会拖慢所有场景）

use vdn_models::ModelKind;
use vdn_types::{LATENCY_BUDGET_MS, latency::planned_platform};

fn full(kind: ModelKind) -> vdn_types::LatencyLedger {
    let mut l = kind.latency_ledger();
    l.extend(planned_platform());
    l
}

#[test]
fn every_routed_model_fits_the_budget() {
    for k in ModelKind::ALL {
        let l = full(k);
        if let Err(t) = l.check(LATENCY_BUDGET_MS) {
            panic!("{k:?} 延迟账 {t:.2} ms 超 {LATENCY_BUDGET_MS} ms：\n{}", l.render());
        }
    }
}

#[test]
fn model_part_is_32_ms_for_both() {
    // 两个模型：GTCRN 256+256、FE-M 160+352 样本 @16 kHz，都是 32 ms
    for k in ModelKind::ALL {
        assert!((k.latency_ledger().total_ms() - 32.0).abs() < 1e-9, "{k:?}");
    }
}

#[test]
fn route_pool_has_equal_latency() {
    let totals: Vec<f64> = ModelKind::ALL.iter().map(|k| full(*k).total_ms()).collect();
    let (lo, hi) = totals.iter().fold((f64::MAX, f64::MIN), |(a, b), t| (a.min(*t), b.max(*t)));
    assert!(hi - lo < 1e-9, "池内延迟不一致：{totals:?}");
}

#[test]
fn ledger_numbers_are_pinned() {
    // 现在的账：按「IO 块整除并对齐模型 hop」口径 41.67 ms；逐项硬加 47.00 ms（超 2 ms，待 P3 脉冲法裁决）。
    // 任何一项变了这条会红——改数字时同步改 specs/rust/architecture.md「延迟账」一节
    let l = full(ModelKind::Gtcrn);
    assert!((l.total_ms() - 125.0 / 3.0).abs() < 1e-9, "{}", l.render());
    assert!((l.naive_total_ms() - 47.0).abs() < 1e-9, "{}", l.render());
}
