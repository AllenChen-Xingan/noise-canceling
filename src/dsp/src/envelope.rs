//! 「快开慢关」包络：人声一出现立刻放开，停顿后先保持一会儿（hangover）再慢慢收。
//!
//! 输出是逐样本的一个数 `m`，在 `closed..=open` 之间。engine 拿它当「混回多少原声」：
//! `out = (1 - m)·降噪 + m·原声`。`closed` 就是衰减下限（模型把噪音压到零时，背景只降 `20·log10(closed)` dB）。
//!
//! 人声判据只看能量（设计稿 003 §2.3：放开不等声纹）：本 hop 能量比「底噪跟踪值」高出 `detect_db`，
//! 且高于绝对门限 `abs_min_db`，就算有声。底噪跟踪：初值取第一个 hop，往下立刻跟，往上每秒最多涨 `noise_rise_db_per_s`。

use crate::one_pole_coef;

#[derive(Clone, Copy, Debug)]
pub struct OpenCloseParams {
    /// 采样率
    pub sr: f32,
    /// 停顿时的混回比例（线性）
    pub closed: f32,
    /// 有声时的混回比例（线性），`>= closed`
    pub open: f32,
    /// 放开的时间常数（秒），应远小于一个 hop
    pub attack_s: f32,
    /// 收的时间常数（秒）
    pub release_s: f32,
    /// 人声结束后保持放开的时长（秒）
    pub hangover_s: f32,
    /// 高于底噪多少 dB 算有声
    pub detect_db: f32,
    /// 绝对门限（dBFS），低于它一律算无声
    pub abs_min_db: f32,
    /// 底噪跟踪值每秒最多上涨多少 dB
    pub noise_rise_db_per_s: f32,
}

pub struct OpenCloseEnvelope {
    p: OpenCloseParams,
    a_up: f32,
    a_down: f32,
    m: f32,
    noise_db: f32,
    /// 剩余保持样本数（整数计，避免浮点逐 hop 相减在整除时多开一个 hop）
    hang_left: usize,
    hang_total: usize,
    voiced: bool,
}

// 构造：非实时线程
impl OpenCloseEnvelope {
    pub fn new(p: OpenCloseParams) -> Self {
        assert!(p.open >= p.closed, "open 必须 >= closed");
        Self {
            a_up: one_pole_coef(p.attack_s, p.sr),
            a_down: one_pole_coef(p.release_s, p.sr),
            m: p.closed,
            noise_db: f32::INFINITY,
            hang_left: 0,
            hang_total: (p.hangover_s * p.sr).round().max(0.0) as usize,
            voiced: false,
            p,
        }
    }
}

// 实时：每 hop 调用（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl OpenCloseEnvelope {
    pub fn reset(&mut self) {
        self.m = self.p.closed;
        self.noise_db = f32::INFINITY;
        self.hang_left = 0;
        self.voiced = false;
    }

    /// 本 hop 是否判为有声（含 hangover）。
    pub fn voiced(&self) -> bool {
        self.voiced
    }

    /// 用 `detect`（一个 hop 的判据信号）更新状态，把逐样本混回比例写进 `mix`（与 `detect` 等长）。
    pub fn process(&mut self, detect: &[f32], mix: &mut [f32]) {
        debug_assert_eq!(detect.len(), mix.len());
        let n = detect.len().min(mix.len());
        let hop_s = n as f32 / self.p.sr;
        let e = detect.iter().take(n).map(|v| v * v).sum::<f32>() / n.max(1) as f32;
        let e_db = 10.0 * (e + 1e-12).log10();
        // 底噪跟踪：往下立刻跟，往上慢慢涨。低于绝对门限的 hop（数字静音、开头补的零、模型延迟里的零输出）
        // 不参与跟踪——否则底噪会被钉在 −120 dB，之后要十几秒才涨回来，期间什么都判「有声」（exp004 自审 P0）
        if e_db > self.p.abs_min_db {
            self.noise_db = if e_db < self.noise_db {
                e_db
            } else {
                (self.noise_db + self.p.noise_rise_db_per_s * hop_s).min(e_db)
            };
        }
        let speech = e_db > self.p.abs_min_db && e_db > self.noise_db + self.p.detect_db;
        if speech {
            self.hang_left = self.hang_total;
        } else {
            self.hang_left = self.hang_left.saturating_sub(n);
        }
        self.voiced = speech || self.hang_left > 0;
        let target = if self.voiced {
            self.p.open
        } else {
            self.p.closed
        };
        for m in mix.iter_mut().take(n) {
            let a = if target > self.m {
                self.a_up
            } else {
                self.a_down
            };
            self.m += (target - self.m) * a;
            *m = self.m;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> OpenCloseParams {
        OpenCloseParams {
            sr: vdn_types::MODEL_RATE as f32,
            closed: 0.25,
            open: 0.5,
            attack_s: 0.002,
            release_s: 0.3,
            hangover_s: 0.15,
            detect_db: 9.0,
            abs_min_db: -70.0,
            noise_rise_db_per_s: 6.0,
        }
    }

    /// 开头的数字静音不能把底噪钉死：零之后来的稳定底噪应当判为无声。
    #[test]
    fn leading_silence_does_not_pin_noise_floor() {
        let mut env = OpenCloseEnvelope::new(params());
        let hop = 256;
        let zeros = vec![0.0f32; hop];
        let noise = vec![3e-3f32; hop];
        let mut mix = vec![0.0; hop];
        for _ in 0..5 {
            env.process(&zeros, &mut mix);
        }
        env.process(&noise, &mut mix);
        for _ in 0..20 {
            env.process(&noise, &mut mix);
            assert!(!env.voiced(), "稳定底噪被判成有声");
        }
    }

    #[test]
    fn opens_fast_and_closes_slowly() {
        let mut env = OpenCloseEnvelope::new(params());
        let hop = 256;
        let quiet = vec![1e-3f32; hop];
        let loud = vec![0.1f32; hop];
        let mut mix = vec![0.0; hop];
        for _ in 0..20 {
            env.process(&quiet, &mut mix);
        }
        assert!((mix[hop - 1] - 0.25).abs() < 1e-3);
        // 一个 hop 内放开到接近 open
        env.process(&loud, &mut mix);
        assert!(mix[hop - 1] > 0.49, "开得不够快：{}", mix[hop - 1]);
        // 停顿：hangover 150 ms ≈ 9 个 hop 内保持放开
        for _ in 0..8 {
            env.process(&quiet, &mut mix);
            assert!(mix[hop - 1] > 0.49);
        }
        // hangover 之后慢慢收：再过 1 个 hop 还远没收到底
        for _ in 0..2 {
            env.process(&quiet, &mut mix);
        }
        assert!(mix[hop - 1] > 0.4 && mix[hop - 1] < 0.5, "{}", mix[hop - 1]);
        for _ in 0..200 {
            env.process(&quiet, &mut mix);
        }
        assert!((mix[hop - 1] - 0.25).abs() < 1e-3);
    }
}
