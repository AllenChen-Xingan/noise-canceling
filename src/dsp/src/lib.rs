//! vdn-dsp：纯信号处理积木。
//!
//! 规矩（所有 crate 的样板）：
//! - `new()` 里一次性分配好所有缓冲；每帧调用的方法（`process` / `analyze` / `synthesize` …）不分配、不加锁、不做 I/O，
//!   由 `tests/` 里的 `assert_no_alloc` 测试证明
//! - 这里不认识任何模型，也不认识 WAV 文件
//! - unsafe 只在 [`ftz`] 一个模块（读写浮点控制寄存器，R10 白名单，大总管 2026-11-02 定）；其余模块一律不许

#![deny(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]
pub mod delay;
pub mod envelope;
pub mod fft;
pub mod ftz;
pub mod resample;
pub mod stft;
pub mod window;

pub use delay::DelayLine;
pub use envelope::{OpenCloseEnvelope, OpenCloseParams};
pub use ftz::FtzGuard;
pub use resample::{Decimator3, Interpolator3};
pub use fft::{FrameFft, Sample, power_into};
pub use stft::StreamingStft;

/// 分贝 → 线性幅度。
pub fn db_to_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

/// 一阶平滑系数：时间常数 `tau_s` 秒、采样率 `sr`。`tau_s <= 0` 时返回 1（立刻跟上）。
pub fn one_pole_coef(tau_s: f32, sr: f32) -> f32 {
    if tau_s <= 0.0 {
        1.0
    } else {
        1.0 - (-1.0 / (tau_s * sr)).exp()
    }
}
