//! 固定 3 倍重采样：48 kHz ↔ 16 kHz（规约 §2.1「固定比例重采样包装」）。
//!
//! 线性相位 FIR 低通（加窗 sinc，奇数抽头），两个方向共用同一组系数。
//! - 降采样：先滤波再每 3 个取 1 个，只算要留下的那个输出
//! - 升采样：每个样本后插 2 个零再滤波（系数 ×3 补回能量），按多相拆开只算非零项
//!
//! 延迟：每个方向 `(TAPS - 1) / 2` 个 48 kHz 样本；`TAPS = 97` 时各 48 个 = 1 ms，两次合计 2 ms（规划预算 4 ms）。
//! 48 能被 3 整除，所以降采样后的延迟正好是 16 个 16 kHz 样本，不留小数相位。
//!
//! 骨架版：系数未针对阻带做优化（待实现：按 009 选定的重采样器换掉或调参，延迟账同步改）。

/// 抽头数（奇数）。
pub const TAPS: usize = 97;
/// 比例。
pub const FACTOR: usize = 3;

/// 每个方向的延迟（高采样率样本）。
pub const fn delay_high() -> usize {
    (TAPS - 1) / 2
}

fn design() -> Vec<f32> {
    // 截止 = 低采样率奈奎斯特的 0.9（7.2 kHz @ 16 kHz），Blackman 窗
    let fc = 0.9 / (2.0 * FACTOR as f64);
    let m = (TAPS - 1) as f64;
    let mut h: Vec<f64> = (0..TAPS)
        .map(|n| {
            let x = n as f64 - m / 2.0;
            let sinc = if x == 0.0 {
                2.0 * fc
            } else {
                (2.0 * std::f64::consts::PI * fc * x).sin() / (std::f64::consts::PI * x)
            };
            let w = 0.42 - 0.5 * (2.0 * std::f64::consts::PI * n as f64 / m).cos()
                + 0.08 * (4.0 * std::f64::consts::PI * n as f64 / m).cos();
            sinc * w
        })
        .collect();
    let sum: f64 = h.iter().sum();
    h.iter_mut().for_each(|v| *v /= sum);
    h.into_iter().map(|v| v as f32).collect()
}

/// 48 kHz → 16 kHz。
pub struct Decimator3 {
    h: Vec<f32>,
    /// 最近 TAPS 个输入（环形）
    hist: Vec<f32>,
    pos: usize,
    phase: usize,
}

/// 16 kHz → 48 kHz。
pub struct Interpolator3 {
    h: Vec<f32>,
    /// 最近 ceil(TAPS/3) 个低采样率输入（环形）
    hist: Vec<f32>,
    pos: usize,
}

// 构造：非实时线程
impl Decimator3 {
    pub fn new() -> Self {
        Self {
            h: design(),
            hist: vec![0.0; TAPS],
            pos: 0,
            phase: 0,
        }
    }
}

impl Default for Decimator3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpolator3 {
    pub fn new() -> Self {
        let h = design().into_iter().map(|v| v * FACTOR as f32).collect();
        Self {
            h,
            hist: vec![0.0; TAPS.div_ceil(FACTOR)],
            pos: 0,
        }
    }
}

impl Default for Interpolator3 {
    fn default() -> Self {
        Self::new()
    }
}

// 实时（规约 §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Decimator3 {
    pub fn reset(&mut self) {
        self.hist.fill(0.0);
        self.pos = 0;
        self.phase = 0;
    }

    /// `input.len()` 必须是 3 的倍数，`output.len() == input.len() / 3`。长度不符时多出的输出清零。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert_eq!(input.len(), output.len() * FACTOR);
        let n = self.hist.len();
        let mut out = output.iter_mut();
        for &x in input {
            if let Some(s) = self.hist.get_mut(self.pos) {
                *s = x;
            }
            self.pos = (self.pos + 1) % n;
            self.phase += 1;
            if self.phase == FACTOR {
                self.phase = 0;
                // y = Σ h[k] · x[t-k]；pos 指向最旧的样本
                let mut acc = 0.0f32;
                for (k, hk) in self.h.iter().enumerate() {
                    let idx = (self.pos + n - 1 - k) % n;
                    acc += hk * self.hist.get(idx).copied().unwrap_or(0.0);
                }
                if let Some(o) = out.next() {
                    *o = acc;
                }
            }
        }
        for o in out {
            *o = 0.0;
        }
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Interpolator3 {
    pub fn reset(&mut self) {
        self.hist.fill(0.0);
        self.pos = 0;
    }

    /// `output.len() == input.len() * 3`。长度不符时多出的输出清零。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert_eq!(input.len() * FACTOR, output.len());
        let n = self.hist.len();
        let mut chunks = output.chunks_exact_mut(FACTOR);
        for &x in input {
            if let Some(s) = self.hist.get_mut(self.pos) {
                *s = x;
            }
            self.pos = (self.pos + 1) % n;
            let Some(chunk) = chunks.next() else { break };
            // 第 p 个高采样率输出 = Σ_j h[p + 3j] · x[t - j]
            for (p, o) in chunk.iter_mut().enumerate() {
                let mut acc = 0.0f32;
                let mut j = 0;
                while p + FACTOR * j < TAPS {
                    let idx = (self.pos + n - 1 - j) % n;
                    acc += self.h.get(p + FACTOR * j).copied().unwrap_or(0.0)
                        * self.hist.get(idx).copied().unwrap_or(0.0);
                    j += 1;
                }
                *o = acc;
            }
        }
        for c in chunks.by_ref() {
            c.fill(0.0);
        }
        chunks.into_remainder().fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(f: f32, sr: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * f * i as f32 / sr).sin())
            .collect()
    }

    #[test]
    fn round_trip_keeps_in_band_sine_with_declared_delay() {
        let x = sine(440.0, 48000.0, 9600);
        let mut d = Decimator3::new();
        let mut u = Interpolator3::new();
        let mut lo = vec![0.0; x.len() / 3];
        let mut y = vec![0.0; x.len()];
        // 分块喂，块长 96（IO 提议块）
        for (xi, li) in x.chunks(96).zip(lo.chunks_mut(32)) {
            d.process(xi, li);
        }
        for (li, yi) in lo.chunks(32).zip(y.chunks_mut(96)) {
            u.process(li, yi);
        }
        let lag = 2 * delay_high();
        let err: f32 = (2000..9000)
            .map(|i| (y[i] - x[i - lag]).abs())
            .fold(0.0, f32::max);
        assert!(err < 0.01, "往返误差 {err}");
    }

    #[test]
    fn decimator_rejects_above_new_nyquist() {
        let x = sine(12000.0, 48000.0, 9600);
        let mut d = Decimator3::new();
        let mut lo = vec![0.0; x.len() / 3];
        d.process(&x, &mut lo);
        let peak = lo[1000..].iter().fold(0.0f32, |m, v| m.max(v.abs()));
        assert!(peak < 0.01, "12 kHz 漏过来 {peak}");
    }
}
