//! 正向频谱的唯一实现（D82「所有频谱计算放在一个地方，各自切法」，`docs/plan/013_frontend-once.md` 第 11 节）。
//!
//! 一段「取一帧 → 调用方钩子改这一帧 → 加窗 → 尾部补零到 `n_fft` → 正向 FFT」只写在 [`FftCore::transform`]；
//! 两个入口都用它：
//! - 流式 [`crate::StreamingStft`]（f32，每 hop 一帧，降噪 · 场景 · 修音）
//! - 按整段分帧 [`FrameFft`]（f32 / f64，给整段信号和帧号；认人 fbank，之后评测器）
//!
//! 功率 `re² + im²` 只写在 [`power_into`]。样本类型对 [`Sample`]（f32、f64）泛型，不手抄两份。

use std::sync::Arc;

use realfft::num_complex::Complex;
use realfft::num_traits::Zero;
use realfft::{FftNum, RealFftPlanner, RealToComplex};

/// FFT 的样本类型（f32 / f64）。
pub trait Sample: FftNum {
    /// 从 f64 转过来（窗表先 f64 算再转）。
    fn from_f64(v: f64) -> Self;
}

impl Sample for f32 {
    fn from_f64(v: f64) -> Self {
        v as f32
    }
}

impl Sample for f64 {
    fn from_f64(v: f64) -> Self {
        v
    }
}

/// 窗表 + 一个正向 FFT 计划 + 工作区。`win.len()` ≤ `n_fft`。
pub(crate) struct FftCore<T: Sample> {
    pub(crate) win: Vec<T>,
    time: Vec<T>,
    spec: Vec<Complex<T>>,
    scratch: Vec<Complex<T>>,
    fwd: Arc<dyn RealToComplex<T>>,
    /// 正向 FFT 做过几次（测试用：证明每帧只算一次）
    pub(crate) forward: u64,
}

// 构造：非实时线程（会分配、会 assert）
impl<T: Sample> FftCore<T> {
    pub(crate) fn new(n_fft: usize, win: Vec<T>) -> Self {
        assert!(
            !win.is_empty() && win.len() <= n_fft,
            "窗长必须在 1..=n_fft"
        );
        let fwd = RealFftPlanner::<T>::new().plan_fft_forward(n_fft);
        Self {
            win,
            time: fwd.make_input_vec(),
            spec: fwd.make_output_vec(),
            scratch: fwd.make_scratch_vec(),
            fwd,
            forward: 0,
        }
    }
}

// 实时：每帧调用。不分配、不 panic（specs/rust/architecture.md §5.3、R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<T: Sample> FftCore<T> {
    pub(crate) fn n_fft(&self) -> usize {
        self.time.len()
    }

    pub(crate) fn bins(&self) -> usize {
        self.spec.len()
    }

    /// `fill` 把这一帧的 `win_len` 个样本写进给它的切片（可顺带做预处理）；之后加窗、补零、正向 FFT（计数 +1）。
    pub(crate) fn transform(&mut self, fill: impl FnOnce(&mut [T])) -> &mut [Complex<T>] {
        self.forward += 1;
        let Some((head, tail)) = self.time.split_at_mut_checked(self.win.len()) else {
            // 构造时已保证 win_len ≤ n_fft
            debug_assert!(false, "窗长超过 n_fft");
            return &mut self.spec;
        };
        fill(head);
        for (t, &w) in head.iter_mut().zip(&self.win) {
            *t = *t * w;
        }
        tail.fill(T::zero());
        // 缓冲长度都由同一个计划给定，不会出错；万一出错，这一帧当静音
        if self
            .fwd
            .process_with_scratch(&mut self.time, &mut self.spec, &mut self.scratch)
            .is_err()
        {
            debug_assert!(false, "rfft 长度不匹配");
            self.spec.fill(Complex::new(T::zero(), T::zero()));
        }
        &mut self.spec
    }

    pub(crate) fn spectrum_mut(&mut self) -> &mut [Complex<T>] {
        &mut self.spec
    }
}

/// 按整段分帧的正向频谱：第 k 帧取 `x[k·hop .. k·hop + win_len]`（Kaldi `snip_edges = true` 的切法）。
pub struct FrameFft<T: Sample> {
    core: FftCore<T>,
    hop: usize,
}

impl<T: Sample> FrameFft<T> {
    /// 按口径建（窗由 [`crate::window::for_spec`] 造，长 `win_len`）。口径不自洽时 panic（构造期）。
    pub fn from_spec(spec: &vdn_types::StftSpec) -> Self {
        assert!(spec.is_valid(), "口径不自洽：{spec:?}");
        Self {
            core: FftCore::new(spec.n_fft, crate::window::for_spec(spec)),
            hop: spec.hop,
        }
    }
}

// 每帧调用：不分配、不 panic
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<T: Sample> FrameFft<T> {
    pub fn n_fft(&self) -> usize {
        self.core.n_fft()
    }

    pub fn win_len(&self) -> usize {
        self.core.win.len()
    }

    pub fn hop(&self) -> usize {
        self.hop
    }

    /// 频点数 = n_fft / 2 + 1。
    pub fn bins(&self) -> usize {
        self.core.bins()
    }

    /// 正向 FFT 累计次数。
    pub fn forward_count(&self) -> u64 {
        self.core.forward
    }

    /// 帧完整落在 `n_samples` 个样本里的帧数（`snip_edges = true`）。
    pub fn num_frames(&self, n_samples: usize) -> usize {
        let wl = self.win_len();
        if n_samples < wl {
            0
        } else {
            1 + (n_samples - wl) / self.hop
        }
    }

    /// 第 `index` 帧的频谱。`pre` 在加窗之前拿到这一帧（长 `win_len`，已是原样本的拷贝），可原地改（去直流、预加重……）。
    /// 帧越出 `x` 时返回 `None`、不做 FFT。
    pub fn frame_with(
        &mut self,
        x: &[T],
        index: usize,
        pre: impl FnOnce(&mut [T]),
    ) -> Option<&mut [Complex<T>]> {
        let start = index.checked_mul(self.hop)?;
        let src = x.get(start..start.checked_add(self.win_len())?)?;
        Some(self.core.transform(|t| {
            for (d, &s) in t.iter_mut().zip(src) {
                *d = s;
            }
            pre(t);
        }))
    }

    /// 同 [`Self::frame_with`]，不做预处理。
    pub fn frame(&mut self, x: &[T], index: usize) -> Option<&mut [Complex<T>]> {
        self.frame_with(x, index, |_| {})
    }
}

/// 功率谱 `re² + im²`（不归一、不加下限）写进 `out`；两边长度不同时只写较短的那段。
/// 全 workspace 只在这里写这个公式（GTCRN 借出、场景自算、认人 fbank 都调它）。实时可调，不分配。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
pub fn power_into<T: Sample>(spec: &[Complex<T>], out: &mut [T]) {
    for (p, c) in out.iter_mut().zip(spec) {
        *p = c.re * c.re + c.im * c.im;
    }
}

#[cfg(test)]
#[path = "fft_tests.rs"]
mod tests;
