//! 流式 STFT / iSTFT（重叠相加）。口径与 exp003 `enhance.py::stream` 一致：
//! - 分析：`rfft(frame * win)`，不归一化（同 numpy）
//! - 合成：`irfft(spec, n) * win` 重叠相加；`irfft` 带 1/n（同 numpy），DC 与 Nyquist 的虚部按 numpy 的 c2r 约定忽略
//! - 每喂一个 hop 输出一个 hop；输出比输入晚 `n_fft - hop` 个样本
//!
//! 输入缓冲初始为全零，所以第一次调用分析的是「`n_fft - hop` 个零 + 第一个 hop」。

use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

pub struct StreamingStft {
    n_fft: usize,
    hop: usize,
    win: Vec<f32>,
    input: Vec<f32>,
    ola: Vec<f32>,
    time: Vec<f32>,
    spec: Vec<Complex32>,
    scratch_fwd: Vec<Complex32>,
    scratch_inv: Vec<Complex32>,
    fwd: Arc<dyn RealToComplex<f32>>,
    inv: Arc<dyn ComplexToReal<f32>>,
}

impl StreamingStft {
    /// `win` 长度必须等于 `n_fft`，且 `hop <= n_fft`。
    pub fn new(n_fft: usize, hop: usize, win: Vec<f32>) -> Self {
        assert_eq!(win.len(), n_fft, "窗长必须等于 n_fft");
        assert!(hop > 0 && hop <= n_fft, "hop 必须在 1..=n_fft");
        let mut planner = RealFftPlanner::<f32>::new();
        let fwd = planner.plan_fft_forward(n_fft);
        let inv = planner.plan_fft_inverse(n_fft);
        Self {
            n_fft,
            hop,
            win,
            input: vec![0.0; n_fft],
            ola: vec![0.0; n_fft],
            time: vec![0.0; n_fft],
            spec: fwd.make_output_vec(),
            scratch_fwd: fwd.make_scratch_vec(),
            scratch_inv: inv.make_scratch_vec(),
            fwd,
            inv,
        }
    }

    pub fn n_fft(&self) -> usize {
        self.n_fft
    }

    pub fn hop(&self) -> usize {
        self.hop
    }

    /// 频点数 = n_fft / 2 + 1。
    pub fn bins(&self) -> usize {
        self.spec.len()
    }

    /// 输出比输入晚的样本数。
    pub fn latency_samples(&self) -> usize {
        self.n_fft - self.hop
    }

    /// 清零输入与重叠相加缓冲（模型状态复位时一起调）。
    pub fn reset(&mut self) {
        self.input.fill(0.0);
        self.ola.fill(0.0);
    }

    /// 喂一个 hop 的新样本，返回这一帧的频谱（可原地改写，再调 [`Self::synthesize`]）。
    pub fn analyze(&mut self, hop_in: &[f32]) -> &mut [Complex32] {
        assert_eq!(hop_in.len(), self.hop);
        self.input.copy_within(self.hop.., 0);
        let tail = self.n_fft - self.hop;
        self.input[tail..].copy_from_slice(hop_in);
        for ((t, x), w) in self.time.iter_mut().zip(&self.input).zip(&self.win) {
            *t = x * w;
        }
        // 长度都由 planner 给定，不会出错
        self.fwd
            .process_with_scratch(&mut self.time, &mut self.spec, &mut self.scratch_fwd)
            .expect("rfft 长度不匹配");
        &mut self.spec
    }

    /// 当前帧频谱（`analyze` 之后、`synthesize` 之前可读写）。
    pub fn spectrum_mut(&mut self) -> &mut [Complex32] {
        &mut self.spec
    }

    /// 把当前频谱做逆变换、加窗、重叠相加，输出一个 hop 的完整样本。
    pub fn synthesize(&mut self, hop_out: &mut [f32]) {
        assert_eq!(hop_out.len(), self.hop);
        // numpy irfft 忽略 DC 与 Nyquist 的虚部；realfft 遇到非零虚部会报错，这里先清掉
        let last = self.spec.len() - 1;
        self.spec[0].im = 0.0;
        if self.n_fft % 2 == 0 {
            self.spec[last].im = 0.0;
        }
        self.inv
            .process_with_scratch(&mut self.spec, &mut self.time, &mut self.scratch_inv)
            .expect("irfft 长度不匹配");
        let norm = 1.0 / self.n_fft as f32;
        for ((o, t), w) in self.ola.iter_mut().zip(&self.time).zip(&self.win) {
            *o += t * norm * w;
        }
        hop_out.copy_from_slice(&self.ola[..self.hop]);
        self.ola.copy_within(self.hop.., 0);
        let tail = self.n_fft - self.hop;
        self.ola[tail..].fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::sqrt_hann_periodic;

    /// 频谱原样送回 → 输出 = 输入晚 n_fft - hop 个样本（周期 sqrt-Hann、50% 重叠可完美重建）。
    #[test]
    fn identity_reconstructs_with_latency() {
        let (n, h) = (512, 256);
        let mut s = StreamingStft::new(n, h, sqrt_hann_periodic(n));
        let x: Vec<f32> = (0..h * 40)
            .map(|i| ((i as f32) * 0.013).sin() * 0.5)
            .collect();
        let mut y = vec![0.0; x.len()];
        for (xi, yi) in x.chunks(h).zip(y.chunks_mut(h)) {
            s.analyze(xi);
            s.synthesize(yi);
        }
        let lat = s.latency_samples();
        for i in lat..x.len() {
            assert!((y[i] - x[i - lat]).abs() < 1e-5, "i={i}");
        }
    }
}
