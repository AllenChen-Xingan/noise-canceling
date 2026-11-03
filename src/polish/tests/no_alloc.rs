//! 证明修音链每帧路径不分配内存（assert_no_alloc；只在 debug 下有分配器钩子，release 测试跳过本文件）。
#![cfg(debug_assertions)]

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use realfft::num_complex::Complex32;
use vdn_types::FrameCtx;
use vdn_polish::{PolishChain, PolishParams};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn per_frame_paths_do_not_allocate() {
    let mut chain = PolishChain::new(512, 256, 16000.0, PolishParams::preset(2));
    let dry: Vec<Complex32> = (0..257)
        .map(|k| Complex32::new((k as f32 * 0.37).sin(), (k as f32 * 0.11).cos()))
        .collect();
    let mut wet = dry.clone();
    let mut hop = vec![0.1f32; 256];
    assert_no_alloc(|| {
        for i in 0..200 {
            if i == 50 {
                chain.set_target(PolishParams::preset(13));
            }
            for (w, d) in wet.iter_mut().zip(&dry) {
                *w = *d * 0.3;
            }
            chain.process_spectrum(
                &mut wet,
                &dry,
                FrameCtx {
                    voice_prob: Some((i % 3) as f32 * 0.5),
                    ..FrameCtx::default()
                },
            );
            chain.process_time(&mut hop);
        }
    });
}
