//! 实时 VAD：earshot（纯 Rust，状态全是定长数组，`predict` 不分配；009 §5 选型）。
//!
//! earshot 只收 16 kHz、每帧 256 样本（16 ms），正好等于 GTCRN 的 hop。别的 hop（FastEnhancer-M 是 160）
//! 用内部定长缓冲重分块：攒满 256 个样本才出一个新概率，其间沿用上一个。**只影响判据的时效（最多晚 16 ms），
//! 不延迟声音**——声音照常一个 hop 进一个 hop 出。
//!
//! 输入先夹到 [-1, 1]：earshot 在 debug 构建里对越界样本 `debug_assert!`（crates.io 1.2.2 `predict_f32` 里的断言；`resources/earshot` 的 HEAD 已改名 `predict`，本 crate 钉 1.2.2）。

/// earshot 1.2.2 写死的采样率与帧长（`Detector::predict_f32` 文档）
pub const VAD_SR: u32 = vdn_types::MODEL_RATE;
pub const VAD_FRAME: usize = 256;

pub struct Vad {
    det: earshot::Detector,
    buf: [f32; VAD_FRAME],
    fill: usize,
    prob: f32,
    /// 自上次 `take_frames` 以来新出的帧概率个数（给判断回路记逐帧概率用）
    new_frames: usize,
}

impl Default for Vad {
    fn default() -> Self {
        Self::new()
    }
}

// 实时：每 hop 在降噪线程（B 档）上调用。不分配、不加锁、不 panic（specs/rust/architecture.md §5.3、R1–R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Vad {
    pub const fn new() -> Self {
        Self {
            det: earshot::Detector::const_default(),
            buf: [0.0; VAD_FRAME],
            fill: 0,
            prob: 0.0,
            new_frames: 0,
        }
    }

    pub fn reset(&mut self) {
        self.det.reset();
        self.buf = [0.0; VAD_FRAME];
        self.fill = 0;
        self.prob = 0.0;
        self.new_frames = 0;
    }

    /// 喂任意长度的一段（16 kHz）；返回喂完后的最新人声概率。
    pub fn push(&mut self, x: &[f32]) -> f32 {
        for &s in x {
            // fill < VAD_FRAME 恒成立（满即清零），get_mut 只为不留 panic 路径（R4）
            if let Some(b) = self.buf.get_mut(self.fill) {
                *b = if s.is_finite() {
                    s.clamp(-1.0, 1.0)
                } else {
                    0.0
                };
            }
            self.fill += 1;
            if self.fill == VAD_FRAME {
                self.prob = self.det.predict_f32(&self.buf);
                self.fill = 0;
                self.new_frames += 1;
            }
        }
        self.prob
    }

    /// 最新概率。
    pub fn prob(&self) -> f32 {
        self.prob
    }

    /// 取走并清零「新出帧数」。
    pub fn take_new_frames(&mut self) -> usize {
        std::mem::take(&mut self.new_frames)
    }
}

/// 概率 → 有声 / 无声的滞回判定：高于 `on` 进入有声，低于 `off` 才退出。
// tidy: 同名不同物：这是 VAD 概率的施密特触发；vdn_types::Hysteresis 是场景切换的确认 / 停留时长
#[derive(Clone, Copy, Debug)]
pub struct Hysteresis {
    pub on: f32,
    pub off: f32,
    state: bool,
}

// 实时：每 hop 在降噪线程（B 档）上调用。不分配、不加锁、不 panic（specs/rust/architecture.md §5.3、R1–R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Hysteresis {
    pub const fn new(on: f32, off: f32) -> Self {
        Self {
            on,
            off,
            state: false,
        }
    }

    pub fn reset(&mut self) {
        self.state = false;
    }

    pub fn update(&mut self, p: f32) -> bool {
        if self.state {
            if p < self.off {
                self.state = false;
            }
        } else if p > self.on {
            self.state = true;
        }
        self.state
    }

    pub fn state(&self) -> bool {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_is_not_voice_and_rechunks() {
        let mut v = Vad::new();
        let z = [0.0f32; 160];
        for _ in 0..50 {
            v.push(&z);
        }
        assert!(v.prob() < 0.5);
        // 50 × 160 = 8000 样本 = 31 帧，余 64
        assert_eq!(v.take_new_frames(), 8000 / VAD_FRAME);
    }

    #[test]
    fn out_of_range_and_nan_do_not_panic() {
        let mut v = Vad::new();
        let mut x = [3.0f32; VAD_FRAME];
        x[3] = f32::NAN;
        x[4] = f32::NEG_INFINITY;
        let p = v.push(&x);
        assert!((0.0..=1.0).contains(&p));
    }

    #[test]
    fn hysteresis() {
        let mut h = Hysteresis::new(0.6, 0.3);
        assert!(!h.update(0.5));
        assert!(h.update(0.7));
        assert!(h.update(0.4));
        assert!(!h.update(0.2));
    }
}
