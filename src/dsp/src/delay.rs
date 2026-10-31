//! 固定长度延迟线：把原声推迟到和模型输出同一时刻（混回原声必须逐样本对齐，否则梳状失真，
//! `docs/research/006_sota-landscape/decision.md` 第 3 件）。

pub struct DelayLine {
    buf: Vec<f32>,
    pos: usize,
}

// 构造：非实时线程
impl DelayLine {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buf: vec![0.0; delay_samples],
            pos: 0,
        }
    }
}

// 实时（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl DelayLine {
    pub fn delay_samples(&self) -> usize {
        self.buf.len()
    }

    pub fn reset(&mut self) {
        self.buf.fill(0.0);
        self.pos = 0;
    }

    /// `out[i] = input[i - delay]`（开头补零）。`input` 与 `out` 应等长（调用方的 bug：debug 下 panic；
    /// release 下只处理较短的那段，`out` 多出的部分清零，不 panic）。
    pub fn process(&mut self, input: &[f32], out: &mut [f32]) {
        debug_assert_eq!(input.len(), out.len());
        let n = input.len().min(out.len());
        let (head, tail) = out.split_at_mut(n);
        tail.fill(0.0);
        if self.buf.is_empty() {
            for (y, x) in head.iter_mut().zip(input) {
                *y = *x;
            }
            return;
        }
        let len = self.buf.len();
        for (x, y) in input.iter().zip(head.iter_mut()) {
            if let Some(slot) = self.buf.get_mut(self.pos) {
                *y = core::mem::replace(slot, *x);
            }
            self.pos += 1;
            if self.pos == len {
                self.pos = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_by_exact_samples() {
        let mut d = DelayLine::new(5);
        let x: Vec<f32> = (1..=12).map(|v| v as f32).collect();
        let mut y = [0.0; 12];
        d.process(&x[..7], &mut y[..7]);
        d.process(&x[7..], &mut y[7..]);
        assert_eq!(&y[..5], &[0.0; 5]);
        assert_eq!(&y[5..], &x[..7]);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn mismatched_lengths_do_not_panic_in_release() {
        let mut d = DelayLine::new(2);
        let mut y = [9.0; 4];
        d.process(&[1.0, 2.0, 3.0], &mut y);
        assert_eq!(y, [0.0, 0.0, 1.0, 0.0]);
    }
}
