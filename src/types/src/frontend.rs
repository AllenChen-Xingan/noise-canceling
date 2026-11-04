//! 频谱前端参数的正本（D71「前端只算一次」，`docs/plan/013_frontend-once.md`）：
//! 降噪模型（GTCRN）、场景特征、修音频谱级用的是同一套 STFT，参数只在这里写一次。
//!
//! 别处一律引用 [`FRONTEND`]：`vdn_dsp::StreamingStft::from_spec` 按它建 STFT；
//! 按这组参数训练过的东西（场景分类器权重、GTCRN 的 `mix` 形状）在各自 crate 里用编译期断言或测试与它比对。
//! 窗种类 → 窗函数的对应只写在 `vdn_dsp::window::for_spec` 一处。

use crate::rate::MODEL_RATE;

/// 分析窗与合成窗（分析、合成同窗）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Window {
    /// 周期 sqrt-Hann：`sqrt(0.5 - 0.5 cos(2πi/n))`；n = 2·hop 时 w² 重叠相加恒为 1
    SqrtHannPeriodic,
}

/// 一套流式 STFT 的口径。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StftSpec {
    /// FFT 点数（= 窗长）
    pub n_fft: usize,
    /// 帧移（样本）
    pub hop: usize,
    /// 采样率（Hz）
    pub sample_rate: u32,
    pub window: Window,
}

impl StftSpec {
    /// 频点数（0 … 奈奎斯特）。
    pub const fn bins(&self) -> usize {
        self.n_fft / 2 + 1
    }

    /// 相邻频点的间隔（Hz）。
    pub fn bin_hz(&self) -> f32 {
        self.sample_rate as f32 / self.n_fft as f32
    }
}

/// 全链路共用的频谱前端：512 点 · 帧移 256（16 ms）· 16 kHz · 周期 sqrt-Hann。
/// 改它等于改 GTCRN 的输入口径并作废场景分类器权重——不是调参。
pub const FRONTEND: StftSpec = StftSpec {
    n_fft: 512,
    hop: 256,
    sample_rate: MODEL_RATE,
    window: Window::SqrtHannPeriodic,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontend_shape() {
        assert_eq!(FRONTEND.bins(), 257);
        // 50% 重叠：sqrt-Hann 完美重建的前提
        assert_eq!(FRONTEND.n_fft, 2 * FRONTEND.hop);
        assert_eq!(FRONTEND.sample_rate, MODEL_RATE);
        assert!((FRONTEND.bin_hz() - 31.25).abs() < 1e-6);
    }
}
