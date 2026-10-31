//! 证明每帧路径不分配内存：在 assert_no_alloc 里跑 analyze / synthesize / delay / envelope，分配就 panic。

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_dsp::{DelayLine, OpenCloseEnvelope, OpenCloseParams, StreamingStft, window};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn per_frame_paths_do_not_allocate() {
    let (n, h) = (512, 256);
    let mut stft = StreamingStft::new(n, h, window::sqrt_hann_periodic(n));
    let mut delay = DelayLine::new(256);
    let mut env = OpenCloseEnvelope::new(OpenCloseParams {
        sr: 16000.0,
        closed: 0.25,
        open: 0.5,
        attack_s: 0.002,
        release_s: 0.3,
        hangover_s: 0.15,
        detect_db: 9.0,
        abs_min_db: -70.0,
        noise_rise_db_per_s: 6.0,
    });
    let x: Vec<f32> = (0..h).map(|i| (i as f32 * 0.1).sin()).collect();
    let mut y = vec![0.0; h];
    let mut d = vec![0.0; h];
    let mut m = vec![0.0; h];
    assert_no_alloc(|| {
        for _ in 0..50 {
            let spec = stft.analyze(&x);
            spec[3] *= 0.5;
            stft.synthesize(&mut y);
            delay.process(&x, &mut d);
            env.process(&y, &mut m);
        }
    });
}
