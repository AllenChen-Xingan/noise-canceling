//! 固定长度延迟线：把原声推迟到和模型输出同一时刻（混回原声必须逐样本对齐，否则梳状失真，
//! `docs/research/006_sota-landscape/decision.md` 第 3 件）。

pub struct DelayLine {
    buf: Vec<f32>,
    pos: usize,
}

impl DelayLine {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buf: vec![0.0; delay_samples],
            pos: 0,
        }
    }

    pub fn delay_samples(&self) -> usize {
        self.buf.len()
    }

    pub fn reset(&mut self) {
        self.buf.fill(0.0);
        self.pos = 0;
    }

    /// `out[i] = input[i - delay]`（开头补零）。`input` 与 `out` 等长。
    pub fn process(&mut self, input: &[f32], out: &mut [f32]) {
        assert_eq!(input.len(), out.len());
        if self.buf.is_empty() {
            out.copy_from_slice(input);
            return;
        }
        for (x, y) in input.iter().zip(out.iter_mut()) {
            *y = self.buf[self.pos];
            self.buf[self.pos] = *x;
            self.pos += 1;
            if self.pos == self.buf.len() {
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
        let mut y = vec![0.0; 12];
        d.process(&x[..7], &mut y[..7]);
        d.process(&x[7..], &mut y[7..]);
        assert_eq!(&y[..5], &[0.0; 5]);
        assert_eq!(&y[5..], &x[..7]);
    }
}
