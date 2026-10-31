//! 不对称门控（003 §2.3、005 decision「门控是不对称的」）：降噪之后乘一个慢变增益 `g ∈ [floor, 1]`。
//!
//! - **快开（只看 VAD，不等声纹）**：检测到「新的人声起始」立刻 `g → 1`（时间常数 `attack_s`，几 ms）。起始有两种：
//!   ① 停顿 ≥ `onset_gap_s` 后 VAD 转有声；② 一直有人声但本 hop 电平比近 300 ms 均值高 `onset_jump_db`
//!   （主人离麦近，插话时电平跳起来——覆盖「邻座还在说、主人接着开口」）
//! - **慢收（要声纹确认）**：只有连续 `confirm_n` 份判决都是 OthersOnly、且判决窗口**覆盖了最近一次起始之后至少
//!   `min_cover_s` 秒**，才按 `release_s` 慢慢收到 `floor`。起始之前算的判决一律不作数——这是「你接着邻桌开口，
//!   第一个字不被吞」的关键
//! - Owner / Unknown 判决 → 回到 1（`reopen_s`，比起始放开慢一点，避免误判来回抽）；NoSpeech → 保持（停顿里不抽动）
//! - 衰减下限 `floor_db`（初值 −12 dB，不压成真空）
//!
//! 零前瞻、零算法延迟。实时安全：`process` 只有算术，不分配。

use vdn_dsp::{db_to_lin, one_pole_coef};

use crate::types::{FrameCtx, SpeakerClass, Stage};
use crate::vad::{Hysteresis, Vad};

#[derive(Clone, Copy, Debug)]
pub struct GateParams {
    pub sr: f32,
    pub floor_db: f32,
    pub attack_s: f32,
    pub reopen_s: f32,
    pub release_s: f32,
    pub onset_gap_s: f32,
    pub onset_jump_db: f32,
    pub min_cover_s: f32,
    pub confirm_n: u32,
    /// 已知主人电平时，起始的电平要 ≥ 主人电平 − 这么多 dB 才算「可能是主人开口」（邻座远场更轻，不触发放开）。
    /// 设成很大（如 200）= 任何起始都放开
    pub onset_level_margin_db: f32,
    pub vad_on: f32,
    pub vad_off: f32,
}

impl Default for GateParams {
    fn default() -> Self {
        Self {
            sr: 16000.0,
            floor_db: -12.0,
            attack_s: 0.005,
            reopen_s: 0.05,
            release_s: 0.7,
            onset_gap_s: 0.2,
            onset_jump_db: 6.0,
            min_cover_s: 1.0,
            confirm_n: 2,
            onset_level_margin_db: 10.0,
            vad_on: 0.5,
            vad_off: 0.35,
        }
    }
}

pub struct AsymGate {
    p: GateParams,
    vad: Vad,
    hyst: Hysteresis,
    floor: f32,
    a_att: f32,
    a_reopen: f32,
    a_rel: f32,
    g: f32,
    silent_samples: u64,
    slow_db: f32,
    last_onset: u64,
    last_verdict_end: u64,
    others_streak: u32,
    closing: bool,
    onsets: u64,
}

impl AsymGate {
    pub fn new(p: GateParams) -> Self {
        Self {
            vad: Vad::new(),
            hyst: Hysteresis::new(p.vad_on, p.vad_off),
            floor: db_to_lin(p.floor_db),
            a_att: one_pole_coef(p.attack_s, p.sr),
            a_reopen: one_pole_coef(p.reopen_s, p.sr),
            a_rel: one_pole_coef(p.release_s, p.sr),
            g: 1.0,
            silent_samples: u64::MAX / 2,
            slow_db: -120.0,
            last_onset: 0,
            last_verdict_end: 0,
            others_streak: 0,
            closing: false,
            onsets: 0,
            p,
        }
    }

    pub fn gain(&self) -> f32 {
        self.g
    }

    pub fn closing(&self) -> bool {
        self.closing
    }

    pub fn onsets(&self) -> u64 {
        self.onsets
    }
}

impl Stage for AsymGate {
    fn reset(&mut self) {
        self.vad.reset();
        self.hyst.reset();
        self.g = 1.0;
        self.silent_samples = u64::MAX / 2;
        self.slow_db = -120.0;
        self.last_onset = 0;
        self.last_verdict_end = 0;
        self.others_streak = 0;
        self.closing = false;
    }

    fn process(&mut self, wet: &mut [f32], _dry: &[f32], ctx: FrameCtx) {
        let n = wet.len();
        if n == 0 {
            return;
        }
        // 1) 起始检测（判据用门控前的信号）
        let p = self.vad.push(wet);
        let voiced = self.hyst.update(p);
        let e = wet.iter().map(|v| v * v).sum::<f32>() / n as f32;
        let e_db = 10.0 * (e + 1e-12).log10();
        let gap = (self.p.onset_gap_s * self.p.sr) as u64;
        let mut onset = false;
        let ol = ctx.speaker.owner_level_db;
        let loud_enough = !ol.is_finite() || e_db >= ol - self.p.onset_level_margin_db;
        if voiced {
            if loud_enough
                && (self.silent_samples >= gap || e_db > self.slow_db + self.p.onset_jump_db)
            {
                onset = true;
            }
            self.silent_samples = 0;
        } else {
            self.silent_samples = self.silent_samples.saturating_add(n as u64);
        }
        // 慢均值（约 300 ms），只跟有声 hop
        if voiced {
            let a = one_pole_coef(0.3, self.p.sr / n as f32);
            self.slow_db += (e_db - self.slow_db) * a;
        }
        if onset {
            self.onsets += 1;
            self.last_onset = ctx.sample_pos;
            self.others_streak = 0;
            self.closing = false;
        }
        // 2) 吸收新判决（每份只算一次）
        let v = ctx.speaker;
        if v.window_end != self.last_verdict_end {
            self.last_verdict_end = v.window_end;
            let cover = (self.p.min_cover_s * self.p.sr) as u64;
            match v.class {
                SpeakerClass::OthersOnly if v.window_end >= self.last_onset + cover => {
                    self.others_streak += 1;
                    if self.others_streak >= self.p.confirm_n {
                        self.closing = true;
                    }
                }
                SpeakerClass::OthersOnly => {}
                SpeakerClass::NoSpeech => {}
                SpeakerClass::Owner | SpeakerClass::Unknown => {
                    self.others_streak = 0;
                    self.closing = false;
                }
            }
        }
        // 3) 增益：收 = 慢；起始后 100 ms 内放开 = 快；其余放开 = 中速
        let fast = ctx.sample_pos < self.last_onset + (0.1 * self.p.sr) as u64 && self.onsets > 0;
        let (target, a) = if self.closing {
            (self.floor, self.a_rel)
        } else if fast {
            (1.0, self.a_att)
        } else {
            (1.0, self.a_reopen)
        };
        for w in wet.iter_mut() {
            self.g += (target - self.g) * a;
            *w *= self.g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SpeakerVerdict;

    fn verdict(class: SpeakerClass, end: u64) -> SpeakerVerdict {
        SpeakerVerdict {
            class,
            window_start: end.saturating_sub(24000),
            window_end: end,
            owner_sim: 0.0,
            owner_level_db: f32::NAN,
        }
    }

    #[test]
    fn closes_only_after_confirmed_and_reopens_on_onset() {
        let mut g = AsymGate::new(GateParams::default());
        let hop = 256u64;
        let mut wet = vec![0.0f32; hop as usize];
        let dry = wet.clone();
        let mut pos = 0u64;
        let mut v = SpeakerVerdict::UNKNOWN;
        // 4 s，判决每 0.5 s 一份 OthersOnly
        for _ in 0..250 {
            if pos % 8000 < hop && pos >= 24000 {
                v = verdict(SpeakerClass::OthersOnly, pos);
            }
            wet.fill(0.0);
            g.process(&mut wet, &dry, FrameCtx { speaker: v, sample_pos: pos });
            pos += hop;
        }
        assert!(g.closing());
        assert!(g.gain() < 0.4, "应已收到接近下限：{}", g.gain());
    }

    #[test]
    fn stale_verdict_before_onset_is_ignored() {
        let mut g = AsymGate::new(GateParams::default());
        // 起始发生在 pos=100000；之前算的 OthersOnly（窗尾 100500 < 起始 + 1 s）不作数
        g.last_onset = 100_000;
        let mut wet = vec![0.0f32; 256];
        let dry = wet.clone();
        for k in 0..3 {
            let v = verdict(SpeakerClass::OthersOnly, 100_500 + k);
            g.process(&mut wet, &dry, FrameCtx { speaker: v, sample_pos: 100_256 + k * 256 });
        }
        assert!(!g.closing());
    }
}
