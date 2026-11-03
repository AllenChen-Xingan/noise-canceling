//! 回调里跑的部分：多声道交错 ↔ 单声道，按块切给用户回调。
//!
//! 这里是 A 档实时代码：不分配、不加锁、不 panic。暂存缓冲在构造时一次分配好，
//! 设备一次给的帧数超过暂存长度时，分几段交给用户回调（每段 ≤ `block` 帧），不扩容。
//! cpal 自身在回调里的违规（`Vec::resize` 等）不在本文件，见 `state/handoff/rs-io.md`「已知违规清单」。
//!
//! 用户回调 panic：coreaudio-rs 的回调是 `extern "C-unwind"`，panic 穿过 Core Audio 的 C 栈帧是未定义行为。
//! 所以这里用 `catch_unwind` 接住（不 panic 时没有开销）：之后不再调用户回调，输入丢弃、输出送静音，
//! 并在 `StreamHealth::callback_panicked()` 上置位。release 若设 `panic = "abort"`，panic 直接终止进程（也是确定行为）。

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::stream::StreamHealth;

/// 输入侧：交错多声道 → 单声道（取第 1 声道）→ 用户回调。
pub struct InputPump<F> {
    scratch: Vec<f32>,
    channels: usize,
    cb: F,
    health: Arc<StreamHealth>,
}

/// 输出侧：用户回调写单声道 → 复制到各声道交错写回设备。
pub struct OutputPump<F> {
    scratch: Vec<f32>,
    channels: usize,
    cb: F,
    health: Arc<StreamHealth>,
}

// 构造期：可以分配。
impl<F> InputPump<F> {
    /// `block` 与 `channels` 由调用方保证 ≥ 1（`stream::validate` 已查）。
    pub fn new(block: usize, channels: usize, cb: F) -> Self {
        Self {
            scratch: vec![0.0; block.max(1)],
            channels: channels.max(1),
            cb,
            health: Arc::new(StreamHealth::default()),
        }
    }

    /// 健康计数句柄（回调 panic 置位在这里）。
    pub fn health(&self) -> Arc<StreamHealth> {
        Arc::clone(&self.health)
    }
}

impl<F> OutputPump<F> {
    pub fn new(block: usize, channels: usize, cb: F) -> Self {
        Self {
            scratch: vec![0.0; block.max(1)],
            channels: channels.max(1),
            cb,
            health: Arc::new(StreamHealth::default()),
        }
    }

    /// 健康计数句柄（回调 panic 置位在这里）。
    pub fn health(&self) -> Arc<StreamHealth> {
        Arc::clone(&self.health)
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<F: FnMut(&[f32])> InputPump<F> {
    /// 设备回调：`data` 是交错样本。尾部不成整帧的样本丢弃（不该出现）。
    pub fn on_data(&mut self, data: &[f32]) {
        let Self {
            scratch,
            channels,
            cb,
            health,
        } = self;
        let frame_len = scratch.len() * *channels;
        for chunk in data.chunks(frame_len) {
            if health.callback_panicked() {
                return;
            }
            let n = fold_first_channel(chunk, *channels, scratch);
            if n == 0 {
                continue;
            }
            if let Some(mono) = scratch.get(..n) {
                health.meter().observe(mono);
                if catch_unwind(AssertUnwindSafe(|| cb(mono))).is_err() {
                    health.mark_panicked();
                }
            }
        }
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<F: FnMut(&mut [f32])> OutputPump<F> {
    /// 设备回调：把 `data`（交错）整块写满。用户回调先拿到清零的单声道块。
    pub fn on_data(&mut self, data: &mut [f32]) {
        let Self {
            scratch,
            channels,
            cb,
            health,
        } = self;
        let frame_len = scratch.len() * *channels;
        for chunk in data.chunks_mut(frame_len) {
            let frames = chunk.len() / *channels;
            if frames == 0 || health.callback_panicked() {
                chunk.fill(0.0);
                continue;
            }
            if let Some(mono) = scratch.get_mut(..frames) {
                mono.fill(0.0);
                if catch_unwind(AssertUnwindSafe(|| cb(mono))).is_err() {
                    health.mark_panicked();
                    mono.fill(0.0);
                }
                health.meter().observe(mono);
                fan_out(mono, *channels, chunk);
            }
        }
    }
}

/// 交错 `interleaved`（`channels` 声道）取第 1 声道写进 `out`，返回写了多少帧。
/// 帧数取「整帧数」与 `out.len()` 的较小者。`channels == 0` 时返回 0。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
pub fn fold_first_channel(interleaved: &[f32], channels: usize, out: &mut [f32]) -> usize {
    if channels == 0 {
        return 0;
    }
    let mut n = 0;
    for (o, frame) in out.iter_mut().zip(interleaved.chunks_exact(channels)) {
        if let Some(&s) = frame.first() {
            *o = s;
            n += 1;
        }
    }
    n
}

/// 单声道 `mono` 复制到 `interleaved` 的每个声道。`interleaved` 里多出的样本（不足一帧或超过 `mono`）写 0。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
pub fn fan_out(mono: &[f32], channels: usize, interleaved: &mut [f32]) {
    if channels == 0 {
        interleaved.fill(0.0);
        return;
    }
    let mut frames = interleaved.chunks_exact_mut(channels);
    let mut src = mono.iter();
    for frame in frames.by_ref() {
        let s = src.next().copied().unwrap_or(0.0);
        frame.fill(s);
    }
    frames.into_remainder().fill(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_takes_first_channel() {
        let inter = [1.0, -1.0, 2.0, -2.0, 3.0, -3.0];
        let mut out = [0.0; 4];
        assert_eq!(fold_first_channel(&inter, 2, &mut out), 3);
        assert_eq!(&out[..3], &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn fold_mono_is_copy_and_respects_out_len() {
        let inter = [1.0, 2.0, 3.0];
        let mut out = [0.0; 2];
        assert_eq!(fold_first_channel(&inter, 1, &mut out), 2);
        assert_eq!(out, [1.0, 2.0]);
    }

    #[test]
    fn fold_ignores_partial_frame_and_zero_channels() {
        let mut out = [9.0; 4];
        assert_eq!(fold_first_channel(&[1.0, 2.0, 3.0], 2, &mut out), 1);
        assert_eq!(fold_first_channel(&[1.0], 0, &mut out), 0);
    }

    #[test]
    fn fan_out_copies_to_every_channel_and_zeroes_rest() {
        let mut inter = [7.0; 7];
        fan_out(&[1.0, 2.0], 3, &mut inter);
        assert_eq!(inter, [1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 0.0]);
        let mut inter = [7.0; 6];
        fan_out(&[1.0], 2, &mut inter);
        assert_eq!(inter, [1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn input_pump_splits_into_blocks_no_larger_than_block() {
        let mut seen: Vec<Vec<f32>> = Vec::new();
        {
            let mut pump = InputPump::new(2, 2, |m: &[f32]| seen.push(m.to_vec()));
            // 5 帧立体声：第 1 声道 0..5
            let data: Vec<f32> = (0..5).flat_map(|i| [i as f32, -1.0]).collect();
            pump.on_data(&data);
        }
        assert_eq!(seen, vec![vec![0.0, 1.0], vec![2.0, 3.0], vec![4.0]]);
    }

    #[test]
    fn output_pump_fills_all_channels_in_blocks() {
        let mut k = 0.0;
        let mut pump = OutputPump::new(2, 2, |m: &mut [f32]| {
            assert!(m.iter().all(|&s| s == 0.0), "用户回调拿到的块应先清零");
            for s in m.iter_mut() {
                k += 1.0;
                *s = k;
            }
        });
        let mut data = [9.0; 6]; // 3 帧立体声
        pump.on_data(&mut data);
        assert_eq!(data, [1.0, 1.0, 2.0, 2.0, 3.0, 3.0]);
    }

    #[test]
    fn callback_panic_is_caught_then_silenced() {
        let mut calls = 0;
        let mut pump = OutputPump::new(2, 1, |m: &mut [f32]| {
            calls += 1;
            m.fill(1.0);
            if calls == 2 {
                panic!("测试：回调 panic");
            }
        });
        let h = pump.health();
        let mut data = [9.0; 6]; // 3 段，第 2 段 panic
        pump.on_data(&mut data);
        assert!(h.callback_panicked());
        assert_eq!(data, [1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
        drop(pump);
        assert_eq!(calls, 2, "panic 之后不再调用户回调");

        let mut n = 0;
        let mut pump = InputPump::new(1, 1, |_m: &[f32]| {
            n += 1;
            panic!("测试");
        });
        let h = pump.health();
        pump.on_data(&[0.0; 4]);
        drop(pump);
        assert!(h.callback_panicked());
        assert_eq!(n, 1);
    }
}
