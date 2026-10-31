//! 按 VAD 驱动的「快开慢关」（修 exp004：只看能量时 94–96% 的帧开着，见 `docs/experiments/exp004-p1-rust-parity/README.md`）。
//!
//! 语义与 p1 的 `vdn_dsp::OpenCloseEnvelope` 完全相同，只换人声判据：
//! `out = (1 − m)·降噪 + m·原声`，有声时 `m → open`，停顿时先保持 `hangover` 再按 `release` 收回 `closed`（= 衰减下限）。
//! 判据 = earshot 在**降噪输出**上的人声概率，过滤滞回。零前瞻、零算法延迟。
//!
//! 同文件还有 [`EnergyEnvelopeStage`]：把 p1 的能量版包成同一个 `Stage`，只为离线对比。

use vdn_dsp::{OpenCloseEnvelope, OpenCloseParams, db_to_lin, one_pole_coef};

use crate::types::{FrameCtx, Stage};
use crate::vad::{Hysteresis, Vad};

#[derive(Clone, Copy, Debug)]
pub struct EnvelopeParams {
    pub sr: f32,
    /// 衰减下限（dB，负数）：停顿时混回原声的比例
    pub floor_db: f32,
    /// 有声时混回原声的比例（dB，负数，`>= floor_db`）
    pub open_db: f32,
    pub attack_s: f32,
    pub release_s: f32,
    pub hangover_s: f32,
    /// VAD 概率滞回
    pub vad_on: f32,
    pub vad_off: f32,
}

impl EnvelopeParams {
    /// 与 p1 默认对齐（open −6、hangover 150 ms、release 300 ms、下限 −12），只换判据。
    pub fn p1_like(sr: f32) -> Self {
        Self {
            sr,
            floor_db: -12.0,
            open_db: -6.0,
            attack_s: 0.002,
            release_s: 0.3,
            hangover_s: 0.15,
            vad_on: 0.5,
            vad_off: 0.35,
        }
    }
}

pub struct VadEnvelopeStage {
    p: EnvelopeParams,
    vad: Vad,
    hyst: Hysteresis,
    closed: f32,
    open: f32,
    a_up: f32,
    a_down: f32,
    m: f32,
    hang_left: usize,
    hang_total: usize,
    voiced: bool,
    raw_voice: bool,
}

impl VadEnvelopeStage {
    pub fn new(p: EnvelopeParams) -> Self {
        let closed = db_to_lin(p.floor_db);
        let open = db_to_lin(p.open_db).max(closed);
        Self {
            vad: Vad::new(),
            hyst: Hysteresis::new(p.vad_on, p.vad_off),
            closed,
            open,
            a_up: one_pole_coef(p.attack_s, p.sr),
            a_down: one_pole_coef(p.release_s, p.sr),
            m: closed,
            hang_left: 0,
            hang_total: (p.hangover_s * p.sr).round().max(0.0) as usize,
            voiced: false,
            raw_voice: false,
            p,
        }
    }

    /// 本 hop 判为有声（含 hangover）
    pub fn voiced(&self) -> bool {
        self.voiced
    }

    /// 本 hop 判为有声（不含 hangover，滞回之后）
    pub fn raw_voice(&self) -> bool {
        self.raw_voice
    }

    pub fn vad_prob(&self) -> f32 {
        self.vad.prob()
    }

    pub fn params(&self) -> EnvelopeParams {
        self.p
    }
}

impl Stage for VadEnvelopeStage {
    fn reset(&mut self) {
        self.vad.reset();
        self.hyst.reset();
        self.m = self.closed;
        self.hang_left = 0;
        self.voiced = false;
        self.raw_voice = false;
    }

    fn process(&mut self, wet: &mut [f32], dry: &[f32], _ctx: FrameCtx) {
        let n = wet.len().min(dry.len());
        let p = self.vad.push(&wet[..n]);
        self.raw_voice = self.hyst.update(p);
        if self.raw_voice {
            self.hang_left = self.hang_total;
        } else {
            self.hang_left = self.hang_left.saturating_sub(n);
        }
        self.voiced = self.raw_voice || self.hang_left > 0;
        let target = if self.voiced { self.open } else { self.closed };
        for (w, &d) in wet[..n].iter_mut().zip(&dry[..n]) {
            let a = if target > self.m { self.a_up } else { self.a_down };
            self.m += (target - self.m) * a;
            *w = (1.0 - self.m) * *w + self.m * d;
        }
    }
}

/// p1 能量版快开慢关（`vdn_dsp::OpenCloseEnvelope`）包成 Stage，参数与 exp004 默认一致。只为对比。
pub struct EnergyEnvelopeStage {
    env: OpenCloseEnvelope,
    mix: Vec<f32>,
}

impl EnergyEnvelopeStage {
    pub fn new(p: EnvelopeParams, max_hop: usize) -> Self {
        let closed = db_to_lin(p.floor_db);
        Self {
            env: OpenCloseEnvelope::new(OpenCloseParams {
                sr: p.sr,
                closed,
                open: db_to_lin(p.open_db).max(closed),
                attack_s: p.attack_s,
                release_s: p.release_s,
                hangover_s: p.hangover_s,
                // 以下三项 = exp004 默认（`src/engine/src/engine.rs`）
                detect_db: 9.0,
                abs_min_db: -70.0,
                noise_rise_db_per_s: 6.0,
            }),
            mix: vec![closed; max_hop],
        }
    }

    pub fn voiced(&self) -> bool {
        self.env.voiced()
    }
}

impl Stage for EnergyEnvelopeStage {
    fn reset(&mut self) {
        self.env.reset();
    }

    fn process(&mut self, wet: &mut [f32], dry: &[f32], _ctx: FrameCtx) {
        let n = wet.len().min(dry.len()).min(self.mix.len());
        self.env.process(&wet[..n], &mut self.mix[..n]);
        for ((w, &d), &m) in wet[..n].iter_mut().zip(&dry[..n]).zip(&self.mix[..n]) {
            *w = (1.0 - m) * *w + m * d;
        }
    }
}

/// 只开衰减下限（固定比例混回原声），作基线。
pub struct FloorStage {
    m: f32,
}

impl FloorStage {
    pub fn new(floor_db: f32) -> Self {
        Self {
            m: db_to_lin(floor_db),
        }
    }
}

impl Stage for FloorStage {
    fn reset(&mut self) {}

    fn process(&mut self, wet: &mut [f32], dry: &[f32], _ctx: FrameCtx) {
        for (w, &d) in wet.iter_mut().zip(dry) {
            *w = (1.0 - self.m) * *w + self.m * d;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SpeakerVerdict;

    fn ctx() -> FrameCtx {
        FrameCtx {
            speaker: SpeakerVerdict::UNKNOWN,
            sample_pos: 0,
        }
    }

    #[test]
    fn stationary_noise_stays_closed() {
        // 白噪声（伪随机）不是人声：VAD 版应保持在下限
        let mut st = VadEnvelopeStage::new(EnvelopeParams::p1_like(16000.0));
        let mut seed = 1u32;
        let mut wet = vec![0.0f32; 256];
        let dry = vec![0.0f32; 256];
        let mut opened = 0;
        for _ in 0..200 {
            for w in wet.iter_mut() {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                *w = ((seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5) * 0.05;
            }
            st.process(&mut wet, &dry, ctx());
            opened += st.voiced() as usize;
        }
        assert!(opened < 20, "白噪声里开了 {opened}/200 个 hop");
    }
}
