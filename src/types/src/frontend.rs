//! 频谱前端参数的正本（D71「前端只算一次」，`docs/plan/013_frontend-once.md`）：
//! 降噪模型（GTCRN）、场景特征、修音频谱级用的是同一套 STFT，参数只在这里写一次。
//! 认人 fbank 用自己的切法（400 / 160、Kaldi 窗，D82），口径同样写成一份 [`StftSpec`]，实现同在 `vdn-dsp`。
//!
//! 别处一律引用 [`FRONTEND`]：`vdn_dsp::StreamingStft::from_spec` 按它建 STFT；
//! 按这组参数训练过的东西（场景分类器权重、GTCRN 的 `mix` 形状）在各自 crate 里用编译期断言或测试与它比对。
//! 窗种类 → 窗函数的对应只写在 `vdn_dsp::window::for_spec` 一处。

use crate::rate::MODEL_RATE;

/// 窗种类（长度 n = `win_len`，下标 i = 0 … n−1）。公式只写在 `vdn_dsp::window` 一处；
/// 每种照抄原调用方的运算顺序（顺序不同末位就不同），都先用 f64 算。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Window {
    /// 周期 sqrt-Hann：`sqrt(0.5 - 0.5 cos(2πi/n))`；n = 2·hop 时 w² 重叠相加恒为 1（降噪 · 场景 · 修音）
    SqrtHannPeriodic,
    /// 周期 Hann：`0.5 - 0.5 cos(2πi/n)`（评测器人声细节）
    HannPeriodic,
    /// numpy `hanning(n + 2)[1:-1]`：`0.5 - 0.5 cos(2π(i+1)/(n+1))`，两端不为 0（评测器 ESTOI）
    HannNumpyInner,
    /// Kaldi 汉明（对称）：`0.54 - 0.46 cos((2π/(n−1))·i)`（认人 WeSpeaker）
    KaldiHamming,
    /// Kaldi 汉宁（对称）：`0.5 - 0.5 cos((2π/(n−1))·i)`
    KaldiHanning,
    /// Kaldi povey：`(0.5 - 0.5 cos((2π/(n−1))·i))^0.85`（认人 3D-Speaker CAM++）
    KaldiPovey,
}

/// 一套 STFT 的口径（流式与按整段分帧共用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StftSpec {
    /// FFT 点数
    pub n_fft: usize,
    /// 窗长（≤ `n_fft`；每帧取 `win_len` 个样本加窗，尾部补零到 `n_fft`）
    pub win_len: usize,
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

    /// 口径自洽：`0 < win_len ≤ n_fft`、`0 < hop`。
    pub const fn is_valid(&self) -> bool {
        self.win_len > 0 && self.win_len <= self.n_fft && self.hop > 0
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
    win_len: 512,
    hop: 256,
    sample_rate: MODEL_RATE,
    window: Window::SqrtHannPeriodic,
};
const _: () = assert!(FRONTEND.is_valid() && FRONTEND.win_len == FRONTEND.n_fft);

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
        // 流式重叠相加要求不补零
        assert_eq!(FRONTEND.win_len, FRONTEND.n_fft);
    }

    #[test]
    fn validity() {
        let ok = StftSpec {
            n_fft: 512,
            win_len: 400,
            hop: 160,
            ..FRONTEND
        };
        assert!(ok.is_valid());
        assert!(!StftSpec { win_len: 513, ..ok }.is_valid());
        assert!(!StftSpec { win_len: 0, ..ok }.is_valid());
        assert!(!StftSpec { hop: 0, ..ok }.is_valid());
    }
}
