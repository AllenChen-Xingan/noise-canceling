//! 固定 3 倍重采样：48 kHz ↔ 16 kHz（规约 §2.1「固定比例重采样包装」）。
//!
//! 线性相位 FIR 低通（加窗 sinc，奇数抽头），两个方向共用同一组系数。
//! - 降采样：先滤波再每 3 个取 1 个；取的是每组 3 个里的**第 1 个**（全局下标 ≡ 0 mod 3），只算要留下的那个输出
//! - 升采样：每个样本后插 2 个零再滤波（系数 ×3 补回能量），按多相拆开只算非零项
//!
//! 延迟：每个方向 `(TAPS - 1) / 2` 个 48 kHz 样本；`TAPS = 97` 时各 48 个 = 1 ms，往返合计 96 个 = 2 ms（规划预算 4 ms）。
//! 48 能被 3 整除、又取每组第 1 个，所以降采样后的延迟正好是 16 个 16 kHz 样本，不留小数相位。
//! 往返对齐靠的是：喂给降采样器的流从下标 0 开始、每次块长都是 3 的倍数（`Pipeline` 按 768 一攒，满足）。
//!
//! 历史缓冲存两份（`hist[i]` 与 `hist[i + n]` 同值），任一时刻最近 n 个样本都是一段连续切片，内层循环不取模。
//!
//! 骨架版：系数未针对阻带做优化（截止 7.2 kHz，Blackman 窗；待实现：按 009 选定的重采样器换掉或调参，延迟账同步改）。

/// 抽头数（奇数）。
pub const TAPS: usize = 97;
/// 比例。
pub const FACTOR: usize = 3;
/// 升采样器每一相的抽头数（`ceil(TAPS / 3)`）。
const PHASE_TAPS: usize = TAPS.div_ceil(FACTOR);

/// 每个方向的延迟（高采样率样本）。
pub const fn delay_high() -> usize {
    (TAPS - 1) / 2
}

/// 加窗 sinc 低通，系数和为 1（直流增益 1）。
fn design() -> Vec<f32> {
    // 截止 = 低采样率奈奎斯特的 0.9（7.2 kHz @ 48 kHz 输入），Blackman 窗
    let fc = 0.9 / (2.0 * FACTOR as f64);
    let m = (TAPS - 1) as f64;
    let pi = std::f64::consts::PI;
    let mut h: Vec<f64> = (0..TAPS)
        .map(|n| {
            let x = n as f64 - m / 2.0;
            let sinc = if x == 0.0 {
                2.0 * fc
            } else {
                (2.0 * pi * fc * x).sin() / (pi * x)
            };
            let w = 0.42 - 0.5 * (2.0 * pi * n as f64 / m).cos()
                + 0.08 * (4.0 * pi * n as f64 / m).cos();
            sinc * w
        })
        .collect();
    let sum: f64 = h.iter().sum();
    h.iter_mut().for_each(|v| *v /= sum);
    h.into_iter().map(|v| v as f32).collect()
}

/// 两份历史缓冲：写一个样本，返回最近 `n` 个样本（旧 → 新）的连续切片。
struct History {
    buf: Vec<f32>,
    n: usize,
    /// 下一个写入位置，`0..n`
    pos: usize,
}

impl History {
    fn new(n: usize) -> Self {
        Self {
            buf: vec![0.0; 2 * n],
            n,
            pos: 0,
        }
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl History {
    fn reset(&mut self) {
        self.buf.fill(0.0);
        self.pos = 0;
    }

    fn push(&mut self, x: f32) {
        let n = self.n;
        if let Some(s) = self.buf.get_mut(self.pos) {
            *s = x;
        }
        if let Some(s) = self.buf.get_mut(self.pos + n) {
            *s = x;
        }
        self.pos += 1;
        if self.pos == n {
            self.pos = 0;
        }
    }

    /// 最近 n 个样本，旧 → 新。
    fn window(&self) -> &[f32] {
        self.buf.get(self.pos..self.pos + self.n).unwrap_or(&[])
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// 48 kHz → 16 kHz。
pub struct Decimator3 {
    /// 倒序系数：`hr[i] = h[TAPS - 1 - i]`，与旧 → 新的历史窗逐项相乘
    hr: Vec<f32>,
    hist: History,
    /// 下一个输入样本的全局下标 mod 3；为 0 时出一个样本
    phase: usize,
}

/// 16 kHz → 48 kHz。
pub struct Interpolator3 {
    /// 3 相，每相 `PHASE_TAPS` 个倒序系数（不足补零）：
    /// 第 p 个输出 = Σ_j 3·h[p + 3j] · x[m - j] = dot(`poly[p]`, 旧 → 新的历史窗)
    poly: Vec<f32>,
    hist: History,
}

// 构造：非实时线程（会分配）
impl Decimator3 {
    pub fn new() -> Self {
        let mut hr = design();
        hr.reverse();
        Self {
            hr,
            hist: History::new(TAPS),
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
        let h = design();
        let mut poly = vec![0.0f32; FACTOR * PHASE_TAPS];
        for p in 0..FACTOR {
            for j in 0..PHASE_TAPS {
                // 窗里下标 PHASE_TAPS-1-j 是 x[m-j]
                if let (Some(&hk), Some(slot)) = (
                    h.get(p + FACTOR * j),
                    poly.get_mut(p * PHASE_TAPS + (PHASE_TAPS - 1 - j)),
                ) {
                    *slot = hk * FACTOR as f32;
                }
            }
        }
        Self {
            poly,
            hist: History::new(PHASE_TAPS),
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
        self.hist.reset();
        self.phase = 0;
    }

    /// 本次调用会输出几个样本（与当前相位有关；从 0 相开始、`n` 是 3 的倍数时正好 `n / 3`）。
    pub fn output_len(&self, n: usize) -> usize {
        n.saturating_sub((FACTOR - self.phase) % FACTOR)
            .div_ceil(FACTOR)
    }

    /// `output.len()` 应等于 [`Self::output_len`]`(input.len())`（调用方的 bug：debug 下 panic；
    /// release 下多出的输出丢掉、不足的输出清零，不 panic）。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert_eq!(output.len(), self.output_len(input.len()));
        let mut out = output.iter_mut();
        for &x in input {
            self.hist.push(x);
            if self.phase == 0 {
                let y = dot(&self.hr, self.hist.window());
                if let Some(o) = out.next() {
                    *o = y;
                }
            }
            self.phase += 1;
            if self.phase == FACTOR {
                self.phase = 0;
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
        self.hist.reset();
    }

    /// `output.len() == input.len() * 3`（调用方的 bug：debug 下 panic；release 下多出的输出清零，不 panic）。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        debug_assert_eq!(input.len() * FACTOR, output.len());
        let (triples, rest) = output.as_chunks_mut::<FACTOR>();
        rest.fill(0.0);
        let (phases, _) = self.poly.as_chunks::<PHASE_TAPS>();
        let mut triples = triples.iter_mut();
        for &x in input {
            let Some(chunk) = triples.next() else { break };
            self.hist.push(x);
            let w = self.hist.window();
            for (o, h) in chunk.iter_mut().zip(phases) {
                *o = dot(h, w);
            }
        }
        for c in triples {
            c.fill(0.0);
        }
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

    fn round_trip(x: &[f32], block: usize) -> Vec<f32> {
        assert_eq!(block % 3, 0);
        let mut d = Decimator3::new();
        let mut u = Interpolator3::new();
        let mut lo = vec![0.0; block / 3];
        let mut y = vec![0.0; x.len()];
        for (xi, yi) in x.chunks(block).zip(y.chunks_mut(block)) {
            let l = &mut lo[..xi.len() / 3];
            d.process(xi, l);
            u.process(l, yi);
        }
        y
    }

    #[test]
    fn coefficients_are_symmetric_and_unit_dc() {
        let h = design();
        assert_eq!(h.len(), TAPS);
        let sum: f32 = h.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
        for i in 0..TAPS {
            assert!((h[i] - h[TAPS - 1 - i]).abs() < 1e-7);
        }
    }

    #[test]
    fn round_trip_keeps_in_band_sine_with_declared_delay() {
        // 440 Hz 与 5 kHz 都在通带里；块长 96（IO 提议块）
        for f in [440.0, 5000.0] {
            let x = sine(f, vdn_types::IO_RATE as f32, 9600);
            let y = round_trip(&x, 96);
            let lag = 2 * delay_high();
            let err = (2000..9000)
                .map(|i| (y[i] - x[i - lag]).abs())
                .fold(0.0, f32::max);
            assert!(err < 0.01, "{f} Hz 往返误差 {err}");
        }
    }

    #[test]
    fn round_trip_lag_is_exactly_declared() {
        // 宽带（多正弦）信号：互相关峰在 2 × delay_high，相邻 ±3 都更小
        let x: Vec<f32> = (0..19200)
            .map(|i| {
                let t = i as f32 / vdn_types::IO_RATE as f32;
                [300.0f32, 1234.0, 2717.0, 4100.0, 6300.0]
                    .iter()
                    .map(|f| (2.0 * std::f32::consts::PI * f * t + f * 0.001).sin())
                    .sum::<f32>()
            })
            .collect();
        let y = round_trip(&x, 768);
        let corr = |lag: usize| -> f32 { (4000..18000).map(|i| y[i] * x[i - lag]).sum() };
        let best = (80..112)
            .max_by(|&a, &b| corr(a).total_cmp(&corr(b)))
            .unwrap();
        assert_eq!(best, 2 * delay_high());
    }

    #[test]
    fn decimator_rejects_above_new_nyquist() {
        for f in [9000.0, 12000.0, 20000.0] {
            let x = sine(f, vdn_types::IO_RATE as f32, 9600);
            let mut d = Decimator3::new();
            let mut lo = vec![0.0; x.len() / 3];
            d.process(&x, &mut lo);
            let peak = lo[1000..].iter().fold(0.0f32, |m, v| m.max(v.abs()));
            assert!(peak < 0.01, "{f} Hz 漏过来 {peak}");
        }
    }

    #[test]
    fn decimator_output_on_first_of_each_triple() {
        // 冲激在下标 0 → 降采样后第 m 个输出 = h[3m]（取每组第 1 个）
        let mut d = Decimator3::new();
        let mut x = vec![0.0; 300];
        x[0] = 1.0;
        let mut lo = vec![0.0; 100];
        d.process(&x, &mut lo);
        let h = design();
        for m in 0..33 {
            assert!((lo[m] - h[3 * m]).abs() < 1e-7, "m={m}");
        }
    }

    #[test]
    fn chunking_does_not_change_output() {
        let x = sine(1000.0, vdn_types::IO_RATE as f32, 4800);
        let mut whole = vec![0.0; 1600];
        Decimator3::new().process(&x, &mut whole);
        // 不是 3 的倍数的块长也要一致
        let mut d = Decimator3::new();
        let mut parts = Vec::new();
        let mut i = 0;
        for (k, n) in [1usize, 2, 7, 96, 100, 511, 3].iter().cycle().enumerate() {
            if i >= x.len() || k > 1000 {
                break;
            }
            let end = (i + n).min(x.len());
            let mut o = vec![0.0; d.output_len(end - i)];
            d.process(&x[i..end], &mut o);
            parts.extend(o);
            i = end;
        }
        assert_eq!(parts, whole);

        let lo = &whole;
        let mut up_whole = vec![0.0; lo.len() * 3];
        Interpolator3::new().process(lo, &mut up_whole);
        let mut u = Interpolator3::new();
        let mut up_parts = Vec::new();
        for c in lo.chunks(37) {
            let mut o = vec![0.0; c.len() * 3];
            u.process(c, &mut o);
            up_parts.extend(o);
        }
        assert_eq!(up_parts, up_whole);
    }

    #[test]
    fn reset_returns_to_initial_state() {
        let x = sine(700.0, vdn_types::IO_RATE as f32, 960);
        let mut d = Decimator3::new();
        let mut a = vec![0.0; 320];
        d.process(&x, &mut a);
        d.process(&x[..1], &mut [0.0]);
        d.reset();
        let mut b = vec![0.0; 320];
        d.process(&x, &mut b);
        assert_eq!(a, b);
    }
}
