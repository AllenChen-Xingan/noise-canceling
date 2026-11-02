//! `vdn chain-test` 的判据计算（纯函数，docs/plan/010 第十一节 V1–V4）。
//!
//! - 相关：皮尔逊相关系数（与增益无关），`y[k]` 对 `x[k - lag]`
//! - 找延迟：先用「起声点」粗定（周期信号的互相关有周期模糊：440 Hz + 2 kHz 合成周期 25 ms = 1200 样本），
//!   再在粗值 ± 半周期内用互相关细定
//! - 静音：`|y| < 阈值` 的最长连续段

/// `y[k]` 与 `x[k - lag]` 的皮尔逊相关，`k` 从 `lag + skip` 到两者重叠的末尾。重叠不足 2 个样本返回 `None`。
pub fn pearson_at_lag(x: &[f32], y: &[f32], lag: usize, skip: usize) -> Option<f64> {
    let start = lag + skip;
    if start >= y.len() {
        return None;
    }
    let n = (y.len() - start).min(x.len().saturating_sub(skip));
    if n < 2 {
        return None;
    }
    let xs = &x[skip..skip + n];
    let ys = &y[start..start + n];
    let (mut sx, mut sy) = (0.0f64, 0.0f64);
    for (&a, &b) in xs.iter().zip(ys) {
        sx += f64::from(a);
        sy += f64::from(b);
    }
    let (mx, my) = (sx / n as f64, sy / n as f64);
    let (mut sxy, mut sxx, mut syy) = (0.0f64, 0.0f64, 0.0f64);
    for (&a, &b) in xs.iter().zip(ys) {
        let (da, db) = (f64::from(a) - mx, f64::from(b) - my);
        sxy += da * db;
        sxx += da * da;
        syy += db * db;
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return Some(0.0);
    }
    Some(sxy / (sxx * syy).sqrt())
}

/// 在 `lo..=hi` 里找相关最大的延迟；只用 `skip` 之后 `win` 个样本算（控制耗时）。
pub fn best_lag(x: &[f32], y: &[f32], lo: usize, hi: usize, skip: usize, win: usize) -> Option<(usize, f64)> {
    let xw = &x[..x.len().min(skip + win)];
    let mut best: Option<(usize, f64)> = None;
    for lag in lo..=hi {
        let yw = &y[..y.len().min(lag + skip + win)];
        if let Some(r) = pearson_at_lag(xw, yw, lag, skip)
            && best.is_none_or(|(_, b)| r > b)
        {
            best = Some((lag, r));
        }
    }
    best
}

/// 第一个 `|y| ≥ thresh` 的样本下标。
pub fn onset(y: &[f32], thresh: f32) -> Option<usize> {
    y.iter().position(|v| v.abs() >= thresh)
}

/// `y[from..]` 里 `|y| < thresh` 的最长连续段：(起点, 长度)。没有返回 (from, 0)。
pub fn longest_silence(y: &[f32], thresh: f32, from: usize) -> (usize, usize) {
    let (mut best, mut cur_start, mut cur) = ((from, 0usize), from, 0usize);
    for (i, v) in y.iter().enumerate().skip(from) {
        if v.abs() < thresh {
            if cur == 0 {
                cur_start = i;
            }
            cur += 1;
            if cur > best.1 {
                best = (cur_start, cur);
            }
        } else {
            cur = 0;
        }
    }
    best
}

/// 非有限值（NaN / ±∞）个数。
pub fn nonfinite(y: &[f32]) -> usize {
    y.iter().filter(|v| !v.is_finite()).count()
}

/// 均方根。
pub fn rms(y: &[f32]) -> f64 {
    if y.is_empty() {
        return 0.0;
    }
    (y.iter().map(|&v| f64::from(v) * f64::from(v)).sum::<f64>() / y.len() as f64).sqrt()
}

pub use vdn_capture::testing::test_tone;

/// 样本数 → 毫秒（48 kHz）。
pub fn ms48(samples: usize) -> f64 {
    samples as f64 / 48.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn delayed(x: &[f32], d: usize) -> Vec<f32> {
        let mut y = vec![0.0; d];
        y.extend_from_slice(x);
        y
    }

    #[test]
    fn pearson_exact_delay_is_one_and_gain_invariant() {
        let x = test_tone(48_000, 9600, 0);
        let y: Vec<f32> = delayed(&x, 2303).iter().map(|v| v * 0.3).collect();
        let r = pearson_at_lag(&x, &y, 2303, 480).unwrap();
        assert!(r > 0.999_999, "{r}");
        let off = pearson_at_lag(&x, &y, 2303 + 7, 480).unwrap();
        assert!(off < 0.99, "{off}");
    }

    #[test]
    fn pearson_degenerate_cases() {
        assert_eq!(pearson_at_lag(&[1.0; 10], &[1.0; 10], 20, 0), None);
        assert_eq!(pearson_at_lag(&[1.0; 10], &[1.0; 10], 0, 0), Some(0.0));
    }

    #[test]
    fn onset_plus_xcorr_find_delay() {
        let x = test_tone(48_000, 24_000, 0);
        let d = 2303;
        let y = delayed(&x, d);
        let on = onset(&y, 0.05).unwrap();
        assert!(on.abs_diff(d) < 50, "{on}");
        let (lag, r) = best_lag(&x, &y, on.saturating_sub(600), on + 600, 2400, 4800).unwrap();
        assert_eq!(lag, d);
        assert!(r > 0.999_999);
    }

    #[test]
    fn silence_runs() {
        let mut y = test_tone(48_000, 4800, 0);
        for v in &mut y[1000..1500] {
            *v = 0.0;
        }
        let (s, n) = longest_silence(&y, 1e-3, 0);
        assert_eq!(s, 1000);
        assert!((500..510).contains(&n), "{n}");
        // 正弦本身过零只有零星几个样本
        let (_, n2) = longest_silence(&test_tone(48_000, 48_000, 0), 1e-3, 0);
        assert!(n2 < 5, "{n2}");
        assert_eq!(longest_silence(&[1.0; 4], 1e-3, 0), (0, 0));
    }

    #[test]
    fn misc() {
        assert_eq!(nonfinite(&[0.0, f32::NAN, f32::INFINITY]), 2);
        assert!((rms(&[1.0, -1.0]) - 1.0).abs() < 1e-12);
        assert_eq!(ms48(480), 10.0);
    }
}
