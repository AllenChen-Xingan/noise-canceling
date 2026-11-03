//! 延迟账（D32 ≤ 45 ms · 规约 R13 · 本批硬约束 1）。不需要模型文件，CI 上也跑。逐项硬加，不认「重叠」推断。
//!
//! **正本是 `Pipeline` 代码**（唯一真相源，人定 2026-11-02）：本文件不抄任何延迟数字，一律从
//! `Pipeline::latency_samples()` / `Pipeline::latency_ledger()` 和 `ModelKind::declared()` 取。
//!
//! 一条链 = Pipeline 部分（重分块 + 两次重采样 + 模型 + 后处理级，代码声明）+ 还没写进代码的平台部分。
//! - Pipeline 用假模型 `DelayGain` 搭，hop 与延迟取自 `ModelKind::declared()`（`tests/smoke.rs` 打开真模型核对过声明）
//! - 平台部分取 `vdn_types::latency::planned_platform()`，**去掉其中的重采样项**：Pipeline 账上已按代码记了重采样，
//!   预算里的那两项是写代码前的占位，再加就记两次
//! - 算不进的（写清）：麦克风与输出设备本身的硬件 / 驱动延迟、蓝牙、系统混音器——P3 脉冲法实测（ADR 005）；
//!   跨进程环余量与 HAL 余量仍是预算（`Basis::Budget`），不是实测
//!
//! 断言：提议 IO 块（96 帧，`Pipeline::with_block`）下每个模型 ≤ 45 ms、池内同延迟；009 原目标 IO 块（256 帧，
//! 任意块长 `Pipeline::new`，因为 256 不整除 FE-M 的 480）也算一遍并打印，结果只记录不断言过线。

use vdn_engine::testing::DelayGain;
use vdn_engine::{Engine, Pipeline, SafetyConfig};
use vdn_models::ModelKind;
use vdn_types::latency::planned_platform;
use vdn_types::{
    Basis, IO_BLOCK_009, IO_BLOCK_PROPOSED, LATENCY_BUDGET_MS, LatencyItem, LatencyLedger,
};

const IO_RATE: u32 = 48_000;

/// 按声明搭一条 Pipeline（假模型，不加载权重）。`block` = None 走任意块长。
fn pipeline(kind: ModelKind, block: Option<usize>) -> Pipeline<DelayGain> {
    let d = kind.declared();
    let e = Engine::new(
        DelayGain::new(d.hop, d.latency_samples, 1.0),
        SafetyConfig::default(),
    );
    assert_eq!(
        e.sample_rate(),
        d.sample_rate,
        "DelayGain 的采样率要与声明一致"
    );
    match block {
        Some(b) => Pipeline::with_block(e, IO_RATE, b).unwrap(),
        None => Pipeline::new(e, IO_RATE).unwrap(),
    }
}

/// 平台部分：规划预算去掉重采样（Pipeline 已按代码记）。
fn platform(io_block: u32) -> Vec<LatencyItem> {
    planned_platform(io_block)
        .into_iter()
        .filter(|i| !i.component.starts_with("resample"))
        .collect()
}

/// 整条链的账：Pipeline 账 + 平台部分。
fn full(p: &Pipeline<DelayGain>, io_block: u32) -> LatencyLedger {
    let mut l = p.latency_ledger();
    l.extend(platform(io_block));
    l
}

fn ms(samples: usize) -> f64 {
    samples as f64 * 1000.0 / f64::from(IO_RATE)
}

#[test]
fn pipeline_ledger_equals_latency_samples() {
    for k in ModelKind::ALL {
        for block in [None, Some(IO_BLOCK_PROPOSED as usize)] {
            let p = pipeline(k, block);
            let l = p.latency_ledger();
            assert!(
                (l.total_ms() - ms(p.latency_samples())).abs() < 1e-9,
                "{k:?} {block:?}：账 {} ms ≠ latency_samples {} ms\n{}",
                l.total_ms(),
                ms(p.latency_samples()),
                l.render()
            );
            assert_eq!(l.budget_ms(), 0.0, "Pipeline 部分应全是代码声明");
            // 重采样只记一次：Pipeline 账里有，平台部分里没有
            let resample_items = l
                .items()
                .iter()
                .chain(platform(IO_BLOCK_PROPOSED).iter())
                .filter(|i| i.component.starts_with("resample"))
                .count();
            assert_eq!(resample_items, 1);
        }
    }
}

#[test]
fn every_routed_model_fits_with_proposed_io_block() {
    for k in ModelKind::ALL {
        let p = pipeline(k, Some(IO_BLOCK_PROPOSED as usize));
        let l = full(&p, IO_BLOCK_PROPOSED);
        // 合计 = Pipeline 声明 + 平台预算（逐项硬加）
        let platform_ms: f64 = platform(IO_BLOCK_PROPOSED)
            .iter()
            .map(LatencyItem::ms)
            .sum();
        assert!((l.total_ms() - (ms(p.latency_samples()) + platform_ms)).abs() < 1e-9);
        if let Err(t) = l.check(LATENCY_BUDGET_MS) {
            panic!(
                "{k:?} 延迟账 {t:.2} ms 超 {LATENCY_BUDGET_MS} ms：\n{}",
                l.render()
            );
        }
        println!("{k:?}（IO 块 {IO_BLOCK_PROPOSED}）：\n{}", l.render());
    }
}

#[test]
fn route_pool_has_equal_latency() {
    for block in [None, Some(IO_BLOCK_PROPOSED as usize)] {
        let lat: Vec<usize> = ModelKind::ALL
            .iter()
            .map(|k| pipeline(*k, block).latency_samples())
            .collect();
        assert!(
            lat.windows(2).all(|w| w[0] == w[1]),
            "池内延迟不一致（{block:?}）：{lat:?}"
        );
    }
}

#[test]
fn model_part_matches_declared() {
    // Pipeline 账里的模型项 = 声明的算法延迟 × 3（16 → 48 kHz）
    for k in ModelKind::ALL {
        let d = k.declared();
        let p = pipeline(k, None);
        let l = p.latency_ledger();
        let model = l
            .items()
            .iter()
            .find(|i| i.component == "delay_gain")
            .unwrap();
        assert_eq!(model.basis, Basis::Declared);
        let want = d.latency_samples as f64 * 1000.0 / f64::from(d.sample_rate);
        assert!((model.ms() - want).abs() < 1e-9, "{k:?}");
    }
}

#[test]
fn original_009_io_block_is_recorded() {
    // 009 原目标 IO 256 帧：256 不整除 FE-M 的 480，只能走任意块长 Pipeline::new。只记录，不断言过线
    for k in ModelKind::ALL {
        let l = full(&pipeline(k, None), IO_BLOCK_009);
        println!(
            "{k:?}（IO 块 {IO_BLOCK_009}，任意块长）合计 {:.2} ms，{}\n{}",
            l.total_ms(),
            if l.check(LATENCY_BUDGET_MS).is_ok() {
                "未超预算"
            } else {
                "超预算"
            },
            l.render()
        );
    }
}
