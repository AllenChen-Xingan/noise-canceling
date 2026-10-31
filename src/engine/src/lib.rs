//! vdn-engine：模型适配器接口、逐帧调度、三件保命、离线驱动。
//!
//! 不认识任何具体模型（那在 `vdn-models`），也不链接 ONNX Runtime：engine 的逻辑可以用假模型单独测。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing
    )
)]
pub mod engine;
pub mod fault;
pub mod model;
pub mod offline;

pub use engine::{Engine, EngineStats, SafetyConfig};
pub use fault::{FaultInjector, FaultKind};
pub use vdn_types::{FrameCtx, Model, ModelError, Stage};

/// 测试与示例用的假模型。
pub mod testing {
    use vdn_types::{Model, ModelError};

    /// 输出 = 输入延迟 `latency` 个样本再乘 `gain`（`latency` 必须是 `hop` 的整数倍以内的任意值）。
    pub struct DelayGain {
        pub hop: usize,
        pub gain: f32,
        line: vdn_dsp::DelayLine,
    }

    impl DelayGain {
        pub fn new(hop: usize, latency: usize, gain: f32) -> Self {
            Self {
                hop,
                gain,
                line: vdn_dsp::DelayLine::new(latency),
            }
        }
    }

    impl Model for DelayGain {
        fn name(&self) -> &'static str {
            "delay_gain"
        }
        fn sample_rate(&self) -> u32 {
            16000
        }
        fn hop(&self) -> usize {
            self.hop
        }
        fn latency_samples(&self) -> usize {
            self.line.delay_samples()
        }
        fn reset(&mut self) {
            self.line.reset();
        }
        fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
            self.line.process(input, output);
            output.iter_mut().for_each(|v| *v *= self.gain);
            Ok(())
        }
    }
}
