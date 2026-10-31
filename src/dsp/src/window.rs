//! 窗函数。与 exp003 `enhance.py` 逐值一致：
//! `sqrt_hann_periodic(n) = sqrt(0.5 - 0.5 cos(2πi/n))`（周期窗，分析与合成都用它）。

/// 周期 sqrt-Hann 窗（GTCRN：n = 512，hop = 256 时 w² 重叠相加恒为 1）。
pub fn sqrt_hann_periodic(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            // 用 f64 算再转 f32：numpy 在 float64 里算完才 astype(float32)
            let v = 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos();
            v.sqrt() as f32
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squared_window_overlap_adds_to_one_at_half_hop() {
        let n = 512;
        let w = sqrt_hann_periodic(n);
        for i in 0..n / 2 {
            let s = w[i] * w[i] + w[i + n / 2] * w[i + n / 2];
            assert!((s - 1.0).abs() < 1e-6, "i={i} s={s}");
        }
    }
}
