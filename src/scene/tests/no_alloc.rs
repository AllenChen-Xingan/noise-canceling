//! 证明场景分析的逐帧（自算与借谱两个入口）与判断路径都不分配（只在 debug 下有分配器钩子）。
#![cfg(debug_assertions)]

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_scene::SceneAnalyzer;
use vdn_types::MODEL_RATE;

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn push_and_judge_do_not_allocate() {
    let mut sa = SceneAnalyzer::new(MODEL_RATE, 256);
    let x: Vec<f32> = (0..256).map(|i| (i as f32 * 0.03).sin() * 0.1).collect();
    assert_no_alloc(|| {
        for k in 0..400 {
            sa.push_hop(&x);
            if k % 31 == 30 {
                let j = sa.judge();
                assert!(j.raw.iter().all(|p| p.is_finite()));
            }
        }
    });
}

#[test]
fn borrowed_spectrum_entry_does_not_allocate() {
    let mut sa = SceneAnalyzer::new(MODEL_RATE, 256);
    let mut fe = vdn_dsp::StreamingStft::from_spec(&vdn_types::FRONTEND);
    let mut pow = vec![0.0f32; vdn_types::FRONTEND.bins()];
    let x: Vec<f32> = (0..256).map(|i| (i as f32 * 0.03).sin() * 0.1).collect();
    assert_no_alloc(|| {
        for k in 0..400 {
            vdn_dsp::power_into(fe.analyze(&x), &mut pow);
            sa.push_hop_with_power(&x, &vdn_types::FRONTEND, &pow);
            if k % 31 == 30 {
                let j = sa.judge();
                assert!(j.raw.iter().all(|p| p.is_finite()));
            }
        }
    });
}
