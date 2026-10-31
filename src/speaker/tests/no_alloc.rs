//! 实时级（VAD 包络、不对称门控）每 hop 不分配：在 assert_no_alloc 里跑，分配就 panic。
// assert_no_alloc 的 AllocDisabler 只在 debug_assertions 下存在：release 测试跳过这个文件
#![cfg(debug_assertions)]

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_speaker::envelope::{EnvelopeParams, VadEnvelopeStage};
use vdn_speaker::gate::{AsymGate, GateParams};
use vdn_speaker::types::{SampleClock, SpeakerClass, SpeakerVerdict};
use vdn_speaker::{FrameCtx, Stage};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn realtime_stages_do_not_allocate() {
    let mut env = VadEnvelopeStage::new(EnvelopeParams::p1_like(16000.0));
    let mut gate = AsymGate::new(GateParams::default());
    // FE-M 的 hop 160 也走一遍（VAD 内部重分块）
    for hop in [256usize, 160] {
        let mut wet = vec![0.0f32; hop];
        let dry = vec![0.0f32; hop];
        let mut pos = 0u64;
        assert_no_alloc(|| {
            for k in 0..400u64 {
                for (i, w) in wet.iter_mut().enumerate() {
                    *w = 0.2 * ((pos + i as u64) as f32 * 0.05).sin();
                }
                let class = if k % 3 == 0 {
                    SpeakerClass::Other
                } else {
                    SpeakerClass::NoSpeech
                };
                let v = SpeakerVerdict {
                    class,
                    window_start: SampleClock(pos),
                    window_end: SampleClock(pos + k),
                    owner_level_db: Some(-25.0),
                    ..SpeakerVerdict::UNKNOWN
                };
                let ctx = FrameCtx {
                    now: SampleClock(pos),
                    speaker: v,
                    voice_prob: None,
                };
                env.process(&mut wet, &dry, ctx);
                gate.process(&mut wet, &dry, ctx);
                pos += hop as u64;
            }
        });
    }
}
