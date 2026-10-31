//! AGC + 零前瞻限幅（时域，模型采样率，一个 hop 进出）。治「音量忽大忽小」。
//!
//! - **电平**：只在人声帧（`voice_prob ≥ 0.5`）更新语音电平估计（τ≈0.6 s 的一阶平滑，dB 域）；无人声时冻结，
//!   所以停顿里不会把底噪越抬越高（AGC 抽吸）
//! - **增益**：目标 = `agc_target_dbfs − 电平`，限在 ±`agc_range_db`；增益变化限速（升 ≤ 4 dB/s，降 ≤ 10 dB/s），
//!   hop 内逐样本线性插值，不出台阶
//! - **限幅**（照 sonora AGC2 Limiter 的原理自写，`resources/sonora/crates/sonora-agc2/src/limiter.rs:12-40`，铁律 3 不复制代码）：
//!   把本 hop 分成 8 个子帧，按子帧峰值算「不超过 −1 dBFS 所需增益」，攻击立即、释放 τ≈60 ms，子帧之间线性插值。
//!   用到的只是本 hop 已经在手的样本（hop 本来就要攒满才处理），**不另加延迟**
//!
//! 参考：009 选型表「限幅 / 轻压缩 → 学思路：照 sonora AGC2 Limiter 自写」（`docs/research/009_rust-ecosystem/decision.md:50`）。

use crate::params::PolishParams;

const SUB: usize = 8;
const CEIL: f32 = 0.891; // −1 dBFS

pub struct Agc {
    hop: usize,
    fps: f32,
    level_db: f32,
    have_level: bool,
    gain_db: f32,
    lim_gain: f32,
    lim_release: f32,
    a_level: f32,
}

impl Agc {
    pub fn new(hop: usize, sr: f32) -> Self {
        let fps = sr / hop as f32;
        let sub_rate = fps * SUB as f32;
        Self {
            hop,
            fps,
            level_db: -100.0,
            have_level: false,
            gain_db: 0.0,
            lim_gain: 1.0,
            lim_release: 1.0 - (-1.0 / (0.06 * sub_rate)).exp(),
            a_level: 1.0 - (-1.0 / (0.6 * fps)).exp(),
        }
    }

    pub fn reset(&mut self) {
        self.level_db = -100.0;
        self.have_level = false;
        self.gain_db = 0.0;
        self.lim_gain = 1.0;
    }

    /// 当前 AGC 增益（dB，不含限幅）。
    pub fn gain_db(&self) -> f32 {
        self.gain_db
    }

    /// 原地处理一个 hop。`x.len() == hop`。
    pub fn process(&mut self, x: &mut [f32], p: &PolishParams, voice_prob: f32) {
        let n = x.len().min(self.hop);
        if n == 0 {
            return;
        }
        let on = p.agc >= 0.5;
        // 电平（用增益前的信号）
        let ms = x[..n].iter().map(|v| v * v).sum::<f32>() / n as f32;
        let e_db = 10.0 * ms.max(1e-12).log10();
        if voice_prob >= 0.5 && e_db > -70.0 {
            if self.have_level {
                self.level_db += self.a_level * (e_db - self.level_db);
            } else {
                self.level_db = e_db;
                self.have_level = true;
            }
        }
        let target = if on && self.have_level {
            (p.agc_target_dbfs - self.level_db).clamp(-p.agc_range_db, p.agc_range_db)
        } else {
            0.0
        };
        let up = 4.0 / self.fps;
        let down = 10.0 / self.fps;
        let g0 = self.gain_db;
        let g1 = g0 + (target - g0).clamp(-down, up);
        self.gain_db = g1;
        let l0 = 10f32.powf(g0 / 20.0);
        let l1 = 10f32.powf(g1 / 20.0);
        let inv = 1.0 / n as f32;
        for (i, v) in x[..n].iter_mut().enumerate() {
            *v *= l0 + (l1 - l0) * i as f32 * inv;
        }
        // 限幅：子帧峰值 → 所需增益 → 立即攻击 / 慢释放 → 子帧间插值
        let sub = n.div_ceil(SUB);
        let mut start = 0;
        while start < n {
            let end = (start + sub).min(n);
            let peak = x[start..end].iter().fold(0.0f32, |m, v| m.max(v.abs()));
            let need = if peak > CEIL { CEIL / peak } else { 1.0 };
            let g_prev = self.lim_gain;
            let g_next = if need < g_prev { need } else { g_prev + self.lim_release * (1.0 - g_prev) };
            // 攻击时整段用 min(prev, next)，保证本子帧峰值不超；释放时线性插值
            let len = (end - start) as f32;
            for (i, v) in x[start..end].iter_mut().enumerate() {
                let g = if g_next < g_prev { g_next } else { g_prev + (g_next - g_prev) * i as f32 / len };
                *v *= g;
            }
            self.lim_gain = g_next;
            start = end;
        }
    }
}
