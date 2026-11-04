//! 窗函数：窗种类 → 公式只在 [`coef`] 一处（种类正本 `vdn_types::Window`，D82「所有频谱计算放在一个地方」）。
//!
//! 每种窗照抄原调用方的运算顺序（如 Kaldi 先算 `2π/(n−1)` 再乘 i，周期窗是 `2π·i/n`）：顺序不同，末位就不同。
//! 一律先用 f64 算，再转成样本类型（numpy 在 float64 里算完才 astype；认人 fbank 原来也是先 f64 再转 f32）。

use std::f64::consts::PI;

use vdn_types::Window;

use crate::fft::Sample;

/// 长度 `n` 的窗第 `i` 个系数（f64）。
pub fn coef(window: Window, i: usize, n: usize) -> f64 {
    match window {
        Window::SqrtHannPeriodic => (0.5 - 0.5 * (2.0 * PI * i as f64 / n as f64).cos()).sqrt(),
        Window::HannPeriodic => 0.5 - 0.5 * (2.0 * PI * i as f64 / n as f64).cos(),
        Window::HannNumpyInner => {
            // numpy hanning(m)：0.5 − 0.5 cos(2πk/(m−1))，m = n + 2，去掉两端 → k = i + 1
            let m = (n + 2) as f64;
            0.5 - 0.5 * (2.0 * PI * (i + 1) as f64 / (m - 1.0)).cos()
        }
        Window::KaldiHamming | Window::KaldiHanning | Window::KaldiPovey => {
            let a = 2.0 * PI / (n as f64 - 1.0);
            let x = i as f64;
            match window {
                Window::KaldiHamming => 0.54 - 0.46 * (a * x).cos(),
                Window::KaldiPovey => (0.5 - 0.5 * (a * x).cos()).powf(0.85),
                _ => 0.5 - 0.5 * (a * x).cos(),
            }
        }
    }
}

/// 长度 `n` 的整张窗。
pub fn table<T: Sample>(window: Window, n: usize) -> Vec<T> {
    (0..n).map(|i| T::of_f64(coef(window, i, n))).collect()
}

/// 按口径造窗：长度 `win_len`（不是 `n_fft`；补零在 FFT 那一步做）。
pub fn for_spec<T: Sample>(spec: &vdn_types::StftSpec) -> Vec<T> {
    table(spec.window, spec.win_len)
}

/// 周期 sqrt-Hann 窗（GTCRN：n = 512，hop = 256 时 w² 重叠相加恒为 1）。与 exp003 `enhance.py` 逐值一致。
pub fn sqrt_hann_periodic(n: usize) -> Vec<f32> {
    table(Window::SqrtHannPeriodic, n)
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;
