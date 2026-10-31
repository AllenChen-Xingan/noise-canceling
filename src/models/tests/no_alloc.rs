//! 真模型路径的 Rust 侧不分配证明（ORT C++ 内部分配不经 Rust 分配器，这里抓不到，见 exp004 README）。
//! - GTCRN：4 个输入 / 4 个输出，落在 ort 的栈上小数组里，加上预分配输出 → Rust 侧零分配，这里断言
//! - FastEnhancer-M：7 个输入 / 7 个输出超过 ort 的栈上容量（`ort src/util/stack.rs:13-15`），每帧仍分配 7 次，
//!   bench 里计数（`rust_allocs_per_hop`），这里不断言
//!
//! 模型文件不在就跳过。

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_engine::{Engine, SafetyConfig};
use vdn_models::ModelKind;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn gtcrn_engine_process_does_not_allocate_in_rust() {
    let Some(path) = ModelKind::Gtcrn.default_path() else {
        eprintln!("跳过：找不到 GTCRN 模型文件（设 VDN_GTCRN_ONNX）");
        return;
    };
    let m = ModelKind::Gtcrn.open(&path).expect("打开模型");
    let mut e = Engine::new(m, SafetyConfig::default());
    let x: Vec<f32> = (0..256).map(|i| 0.1 * (i as f32 * 0.07).sin()).collect();
    let mut y = vec![0.0; 256];
    // 热身：第一次推理 ORT 可能做惰性初始化
    for _ in 0..5 {
        e.process(&x, &mut y);
    }
    assert_no_alloc(|| {
        for _ in 0..50 {
            e.process(&x, &mut y);
        }
    });
    assert_eq!(e.stats().fault_backend, 0);
}
