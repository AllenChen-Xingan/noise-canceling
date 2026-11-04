//! 流式 STFT / iSTFT（重叠相加）。口径与 exp003 `enhance.py::stream` 一致：
//! - 分析：`rfft(frame * win)`，不归一化（同 numpy）
//! - 合成：`irfft(spec, n) * win` 重叠相加；`irfft` 带 1/n（同 numpy），DC 与 Nyquist 的虚部按 numpy 的 c2r 约定忽略
//! - 每喂一个 hop 输出一个 hop；输出比输入晚 `n_fft - hop` 个样本
//!
//! 输入缓冲初始为全零，所以第一次调用分析的是「`n_fft - hop` 个零 + 第一个 hop」。
//!
//! 全链路共用的口径是 `vdn_types::FRONTEND`，用 [`StreamingStft::from_spec`] 建；功率谱一律用 [`crate::power_into`] 算
//! （GTCRN 借出与场景自算是同一段代码，逐位相同靠构造保证，`docs/plan/013_frontend-once.md`）。
//! 正向那一段（加窗 → FFT）与按整段分帧的 [`crate::FrameFft`] 共用 [`FftCore`]（`fft.rs`）。

use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner};

use crate::fft::FftCore;

pub struct StreamingStft {
    n_fft: usize,
    hop: usize,
    /// 正向：窗表、正向计划、频谱缓冲、计数
    core: FftCore<f32>,
    input: Vec<f32>,
    ola: Vec<f32>,
    time: Vec<f32>,
    scratch_inv: Vec<Complex32>,
    inv: Arc<dyn ComplexToReal<f32>>,
}

// 构造：非实时线程（会分配、会 assert）
impl StreamingStft {
    /// `win` 长度必须等于 `n_fft`，且 `hop <= n_fft`。
    pub fn new(n_fft: usize, hop: usize, win: Vec<f32>) -> Self {
        assert_eq!(win.len(), n_fft, "窗长必须等于 n_fft");
        assert!(hop > 0 && hop <= n_fft, "hop 必须在 1..=n_fft");
        // 逆变换只有流式合成要；正向计划在 FftCore 里
        let inv = RealFftPlanner::<f32>::new().plan_fft_inverse(n_fft);
        Self {
            n_fft,
            hop,
            core: FftCore::new(n_fft, win),
            input: vec![0.0; n_fft],
            ola: vec![0.0; n_fft],
            time: vec![0.0; n_fft],
            scratch_inv: inv.make_scratch_vec(),
            inv,
        }
    }

    /// 按口径正本建（窗由 [`crate::window::for_spec`] 造）。流式重叠相加要求 `win_len == n_fft`（不补零）。
    pub fn from_spec(spec: &vdn_types::StftSpec) -> Self {
        assert!(spec.is_valid(), "口径不自洽：{spec:?}");
        assert_eq!(
            spec.win_len, spec.n_fft,
            "流式 STFT 不支持补零（win_len 必须等于 n_fft）"
        );
        Self::new(spec.n_fft, spec.hop, crate::window::for_spec(spec))
    }
}

// 实时：每 hop 调用。不分配、不 panic（specs/rust/architecture.md §5.3、R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl StreamingStft {
    pub fn n_fft(&self) -> usize {
        self.n_fft
    }

    pub fn hop(&self) -> usize {
        self.hop
    }

    /// 频点数 = n_fft / 2 + 1。
    pub fn bins(&self) -> usize {
        self.core.bins()
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

    /// 正向 FFT 累计次数（`reset` 不清零）。
    pub fn forward_count(&self) -> u64 {
        self.core.forward
    }

    /// 只把一个 hop 推进输入缓冲、不做 FFT（借别人的谱时让自己的缓冲保持同步，切回自算时第一帧不吃旧数据）。
    /// `hop_in.len()` 应等于 `hop()`（调用方的 bug：debug 下 panic；release 下不足的补零、多的丢掉）。
    pub fn feed(&mut self, hop_in: &[f32]) {
        debug_assert_eq!(hop_in.len(), self.hop);
        self.input.copy_within(self.hop.., 0);
        let tail = self.n_fft - self.hop;
        let mut src = hop_in.iter();
        for d in self.input.iter_mut().skip(tail) {
            *d = src.next().copied().unwrap_or(0.0);
        }
    }

    /// 推进 `min(x.len(), hop)` 个样本（不足一个 hop 时只移这么多、不补零；多的丢掉；空输入不动缓冲）。
    /// 给输入长度不由自己定的吃谱方（场景特征）保留改前的移位语义；整 hop 时与 [`Self::feed`] 逐位相同。
    pub fn feed_partial(&mut self, x: &[f32]) {
        let h = self.hop.min(x.len());
        self.input.copy_within(h.., 0);
        let tail = self.n_fft - h;
        for (d, &s) in self.input.iter_mut().skip(tail).zip(x) {
            *d = s;
        }
    }

    /// 同 [`Self::analyze`]，但按 [`Self::feed_partial`] 推进缓冲。
    pub fn analyze_partial(&mut self, x: &[f32]) -> &mut [Complex32] {
        self.feed_partial(x);
        self.transform()
    }

    /// 喂一个 hop 的新样本，返回这一帧的频谱（可原地改写，再调 [`Self::synthesize`]）。
    /// `hop_in.len()` 同 [`Self::feed`]。
    pub fn analyze(&mut self, hop_in: &[f32]) -> &mut [Complex32] {
        self.feed(hop_in);
        self.transform()
    }

    /// 对当前输入缓冲加窗、做一次正向 FFT（计数 +1）。
    fn transform(&mut self) -> &mut [Complex32] {
        let input = &self.input;
        self.core.transform(|t| {
            for (d, &x) in t.iter_mut().zip(input) {
                *d = x;
            }
        })
    }

    /// 当前帧频谱（`analyze` 之后、`synthesize` 之前可读写）。
    pub fn spectrum_mut(&mut self) -> &mut [Complex32] {
        self.core.spectrum_mut()
    }

    /// 把当前频谱做逆变换、加窗、重叠相加，输出一个 hop 的完整样本。
    /// `hop_out.len()` 应等于 `hop()`（调用方的 bug：debug 下 panic；release 下多出的部分清零）。
    pub fn synthesize(&mut self, hop_out: &mut [f32]) {
        debug_assert_eq!(hop_out.len(), self.hop);
        let even = self.n_fft.is_multiple_of(2);
        let spec = self.core.spectrum_mut();
        // numpy irfft 忽略 DC 与 Nyquist 的虚部；realfft 遇到非零虚部会报错，这里先清掉
        if let Some(dc) = spec.first_mut() {
            dc.im = 0.0;
        }
        if even && let Some(ny) = spec.last_mut() {
            ny.im = 0.0;
        }
        // InputValues（DC / Nyquist 虚部非零）时 realfft 照样算完、结果可用；只有缓冲长度类错误才当静音
        match self
            .inv
            .process_with_scratch(spec, &mut self.time, &mut self.scratch_inv)
        {
            Ok(()) | Err(realfft::FftError::InputValues(..)) => {}
            Err(_) => {
                debug_assert!(false, "irfft 长度不匹配");
                self.time.fill(0.0);
            }
        }
        let norm = 1.0 / self.n_fft as f32;
        for ((o, t), w) in self.ola.iter_mut().zip(&self.time).zip(&self.core.win) {
            *o += t * norm * w;
        }
        let mut src = self.ola.iter().take(self.hop);
        for o in hop_out.iter_mut() {
            *o = src.next().copied().unwrap_or(0.0);
        }
        self.ola.copy_within(self.hop.., 0);
        let tail = self.n_fft - self.hop;
        for o in self.ola.iter_mut().skip(tail) {
            *o = 0.0;
        }
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

    /// `from_spec(FRONTEND)` 与手写 512 / 256 / sqrt-Hann 逐位相同；计数每次 `analyze` 加 1、`feed` 不加。
    #[test]
    fn from_spec_matches_manual_and_counts() {
        let spec = vdn_types::FRONTEND;
        let mut a = StreamingStft::from_spec(&spec);
        let mut b = StreamingStft::new(512, 256, sqrt_hann_periodic(512));
        assert_eq!((a.n_fft(), a.hop(), a.bins()), (512, 256, spec.bins()));
        let x: Vec<f32> = (0..256 * 6).map(|i| ((i as f32) * 0.021).sin()).collect();
        for h in x.chunks(256) {
            let sa: Vec<Complex32> = a.analyze(h).to_vec();
            assert_eq!(sa, b.analyze(h).to_vec());
        }
        assert_eq!(a.forward_count(), 6);
        a.feed(&x[..256]);
        assert_eq!(a.forward_count(), 6);
    }

    /// `feed` 之后再 `analyze` = 一直 `analyze`（缓冲同步）。
    #[test]
    fn feed_keeps_buffer_in_sync() {
        let mut a = StreamingStft::from_spec(&vdn_types::FRONTEND);
        let mut b = StreamingStft::from_spec(&vdn_types::FRONTEND);
        let x: Vec<f32> = (0..256 * 5).map(|i| ((i as f32) * 0.037).cos()).collect();
        let hops: Vec<&[f32]> = x.chunks(256).collect();
        for h in &hops[..4] {
            a.feed(h);
            b.analyze(h);
        }
        assert_eq!(a.analyze(hops[4]).to_vec(), b.analyze(hops[4]).to_vec());
    }

    /// `feed_partial`：整 hop 时与 `feed` 相同；不足一个 hop 只移那么多、不补零；空输入不动；多的丢掉。
    #[test]
    fn feed_partial_shifts_by_input_length() {
        let spec = vdn_types::FRONTEND;
        let x: Vec<f32> = (0..spec.n_fft * 2).map(|i| i as f32 + 1.0).collect();
        let (mut a, mut b) = (
            StreamingStft::from_spec(&spec),
            StreamingStft::from_spec(&spec),
        );
        a.feed(&x[..spec.hop]);
        b.feed_partial(&x[..spec.hop]);
        assert_eq!(a.input, b.input);
        // 手算参照：左移 h、尾部填 x[..h]
        let mut want = b.input.clone();
        for len in [100usize, 0, spec.hop + 50, 1] {
            let h = len.min(spec.hop);
            want.copy_within(h.., 0);
            let tail = spec.n_fft - h;
            want[tail..].copy_from_slice(&x[..h]);
            b.feed_partial(&x[..len]);
            assert_eq!(b.input, want, "len = {len}");
        }
        let n = b.forward_count();
        b.analyze_partial(&x[..10]);
        assert_eq!(b.forward_count(), n + 1);
    }
}
