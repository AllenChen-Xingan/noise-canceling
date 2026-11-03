//! 电平统计：峰值 · 均方根 · 连续全零长度。回调泵在实时线程上写，任何线程读。
//!
//! 实时安全：只用原子量，不分配、不加锁。**单写者**（一条流只有一个回调线程），所以「读 → 加 → 写回」
//! 不会丢更新；读者可能看到几个字段之间差一次回调（统计用，可以接受）。
//!
//! 「全零」按样本值**恰好等于 0.0** 判：真麦克风总有底噪，不会连续 1 s 恰好是 0；
//! macOS 在程序没有麦克风权限时送的是全零静音（推断，依据：常见报告与 ffmpeg 的行为，未在本机复现）。

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// 连续这么多个样本全是 0.0，就判「疑似没有麦克风权限或设备静音」：1 s（按链路采样率 [`crate::IO_RATE`]）。
pub const SILENT_RUN_SAMPLES: u64 = crate::IO_RATE as u64;

/// 累计电平计数（只增不减，`zero_run` 除外）。
#[derive(Debug, Default)]
pub struct LevelMeter {
    samples: AtomicU64,
    /// |x| 的最大值，存 f32 的位：非负 f32 的位序与数值序一致，所以能直接比大小
    peak_bits: AtomicU32,
    /// 平方和，存 f64 的位（单写者）
    sum_sq_bits: AtomicU64,
    /// 当前连续全零的样本数
    zero_run: AtomicU64,
    max_zero_run: AtomicU64,
    /// 非有限值（NaN / ∞）样本数；不计入峰值与均方根
    nonfinite: AtomicU64,
}

/// 某一时刻的电平读数。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Level {
    pub samples: u64,
    /// 峰值（线性，满刻度 = 1.0）
    pub peak: f32,
    /// 均方根（线性）；没有样本时为 0
    pub rms: f64,
    pub zero_run: u64,
    pub max_zero_run: u64,
    pub nonfinite: u64,
}

impl Level {
    /// 峰值（dBFS）；全零为 −∞
    pub fn peak_dbfs(&self) -> f64 {
        dbfs(f64::from(self.peak))
    }
    /// 均方根（dBFS）；全零为 −∞
    pub fn rms_dbfs(&self) -> f64 {
        dbfs(self.rms)
    }
    /// 现在正连续全零满 1 s（[`SILENT_RUN_SAMPLES`]）
    pub fn silent_now(&self) -> bool {
        self.zero_run >= SILENT_RUN_SAMPLES
    }
    /// 收到过样本，且全部是 0
    pub fn all_zero(&self) -> bool {
        self.samples > 0 && self.peak == 0.0
    }
}

/// 线性幅度 → dBFS（20·log10）；0 或负数给 −∞。
pub fn dbfs(x: f64) -> f64 {
    if x > 0.0 {
        20.0 * x.log10()
    } else {
        f64::NEG_INFINITY
    }
}

/// dBFS 的显示：保留 1 位小数，−∞ 写成「−∞」。
pub fn fmt_dbfs(db: f64) -> String {
    if db.is_finite() {
        format!("{db:.1} dBFS")
    } else {
        "−∞ dBFS".to_string()
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl LevelMeter {
    /// 记一段单声道样本（回调线程调用；单写者）。
    pub fn observe(&self, x: &[f32]) {
        let mut peak = 0.0f32;
        let mut sq = 0.0f64;
        let mut bad = 0u64;
        let mut run = self.zero_run.load(Ordering::Relaxed);
        let mut max_run = self.max_zero_run.load(Ordering::Relaxed);
        for &s in x {
            if !s.is_finite() {
                bad += 1;
                run = 0;
                continue;
            }
            let a = s.abs();
            if a > peak {
                peak = a;
            }
            sq += f64::from(s) * f64::from(s);
            if s == 0.0 {
                run += 1;
                if run > max_run {
                    max_run = run;
                }
            } else {
                run = 0;
            }
        }
        self.peak_bits.fetch_max(peak.to_bits(), Ordering::Relaxed);
        let total = f64::from_bits(self.sum_sq_bits.load(Ordering::Relaxed)) + sq;
        self.sum_sq_bits.store(total.to_bits(), Ordering::Relaxed);
        self.zero_run.store(run, Ordering::Relaxed);
        self.max_zero_run.store(max_run, Ordering::Relaxed);
        if bad > 0 {
            self.nonfinite.fetch_add(bad, Ordering::Relaxed);
        }
        self.samples.fetch_add(x.len() as u64, Ordering::Relaxed);
    }

    /// 读一次（任何线程）。
    pub fn snapshot(&self) -> Level {
        let samples = self.samples.load(Ordering::Relaxed);
        let nonfinite = self.nonfinite.load(Ordering::Relaxed);
        let finite = samples.saturating_sub(nonfinite);
        let sum_sq = f64::from_bits(self.sum_sq_bits.load(Ordering::Relaxed));
        Level {
            samples,
            peak: f32::from_bits(self.peak_bits.load(Ordering::Relaxed)),
            rms: if finite == 0 {
                0.0
            } else {
                (sum_sq / finite as f64).sqrt()
            },
            zero_run: self.zero_run.load(Ordering::Relaxed),
            max_zero_run: self.max_zero_run.load(Ordering::Relaxed),
            nonfinite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_level_matches_theory() {
        // 振幅 0.5 的正弦：峰值 0.5（−6.02 dBFS），均方根 0.5/√2（−9.03 dBFS）
        let m = LevelMeter::default();
        let x: Vec<f32> = (0..48_000)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 48_000.0).sin())
            .collect();
        for c in x.chunks(96) {
            m.observe(c);
        }
        let l = m.snapshot();
        assert_eq!(l.samples, 48_000);
        assert!((l.peak - 0.5).abs() < 1e-3, "{l:?}");
        assert!((l.rms - 0.5 / 2f64.sqrt()).abs() < 1e-3, "{l:?}");
        assert!((l.peak_dbfs() + 6.02).abs() < 0.02);
        assert!((l.rms_dbfs() + 9.03).abs() < 0.02);
        assert!(!l.silent_now() && !l.all_zero());
        assert_eq!(l.max_zero_run, 1, "正弦过零点恰好是 0 的只有第 0 个样本");
    }

    #[test]
    fn zero_run_spans_callbacks_and_resets_on_sound() {
        let m = LevelMeter::default();
        let z = [0.0f32; 96];
        for _ in 0..(SILENT_RUN_SAMPLES / 96 - 1) {
            m.observe(&z);
        }
        assert!(!m.snapshot().silent_now(), "差一块不到 1 s");
        m.observe(&z);
        let l = m.snapshot();
        assert!(l.silent_now() && l.all_zero(), "{l:?}");
        assert_eq!(l.peak_dbfs(), f64::NEG_INFINITY);
        // 有声音就清零当前连续长度，但保留最长纪录
        m.observe(&[0.0, 1e-6, 0.0]);
        let l = m.snapshot();
        assert_eq!(l.zero_run, 1);
        assert!(l.max_zero_run >= SILENT_RUN_SAMPLES);
        assert!(!l.silent_now() && !l.all_zero());
    }

    #[test]
    fn negative_zero_counts_as_zero_and_nonfinite_is_skipped() {
        let m = LevelMeter::default();
        m.observe(&[-0.0, f32::NAN, f32::INFINITY, -0.25]);
        let l = m.snapshot();
        assert_eq!((l.samples, l.nonfinite), (4, 2));
        assert_eq!(l.peak, 0.25);
        // 有限的两个：0 和 0.25 → 均方根 0.25/√2
        assert!((l.rms - 0.25 / 2f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn empty_meter_reads_minus_infinity() {
        let l = LevelMeter::default().snapshot();
        assert_eq!(l.samples, 0);
        assert!(!l.all_zero(), "一个样本都没收到不算全零");
        assert_eq!(fmt_dbfs(l.rms_dbfs()), "−∞ dBFS");
        assert_eq!(fmt_dbfs(-6.0206), "-6.0 dBFS");
    }
}
