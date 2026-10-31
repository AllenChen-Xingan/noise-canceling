//! 不对称门控（003 §2.3、005 decision「门控是不对称的」）：降噪之后乘一个慢变增益 `g ∈ [floor, 1]`。
//!
//! - **快开（只看 VAD，不等声纹）**：检测到「新的人声起始」立刻 `g → 1`（时间常数 `attack_s`，几 ms）。起始有两种：
//!   ① 停顿 ≥ `onset_gap_s` 后 VAD 转有声；② 一直有人声但本 hop 电平比近 300 ms 均值高 `onset_jump_db`
//!   （主人离麦近，插话时电平跳起来——覆盖「邻座还在说、主人接着开口」）
//! - **慢收（要声纹确认）**：只有连续 `confirm_n` 份判决都是 Other、且判决窗**整个落在最近一次起始之后**
//!   （`SpeakerVerdict::valid_after`），才按 `release_s` 慢慢收到 `floor`。起始之前算的判决一律不作数——
//!   这是「你接着邻桌开口，第一个字不被吞」的关键
//! - Owner / Unknown 判决 → 回到 1（`reopen_s`，比起始放开慢一点，避免误判来回抽）；NoSpeech → 保持（停顿里不抽动）
//! - 衰减下限 `floor_db`（初值 −12 dB，不压成真空）
//!
//! 零前瞻、零算法延迟。实时安全：`process` 只有算术，不分配。

use vdn_dsp::{db_to_lin, one_pole_coef};

use crate::types::{FrameCtx, SampleClock, SpeakerClass, Stage};
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
    last_onset: SampleClock,
    last_verdict_end: SampleClock,
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
            last_onset: SampleClock(0),
            last_verdict_end: SampleClock(0),
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
    fn name(&self) -> &'static str {
        "asym-gate"
    }

    fn reset(&mut self) {
        self.vad.reset();
        self.hyst.reset();
        self.g = 1.0;
        self.silent_samples = u64::MAX / 2;
        self.slow_db = -120.0;
        self.last_onset = SampleClock(0);
        self.last_verdict_end = SampleClock(0);
        self.others_streak = 0;
        self.closing = false;
    }

    fn process(&mut self, wet: &mut [f32], _dry: &[f32], ctx: FrameCtx) {
        let n = wet.len();
        if n == 0 {
            return;
        }
        // 1) 起始检测：人声概率优先用上游给的（与包络同一个 VAD），没有就自己算
        let p = match ctx.voice_prob {
            Some(p) => p,
            None => self.vad.push(wet),
        };
        let voiced = self.hyst.update(p);
        let e = wet.iter().map(|v| v * v).sum::<f32>() / n as f32;
        let e_db = 10.0 * (e + 1e-12).log10();
        let gap = (self.p.onset_gap_s * self.p.sr) as u64;
        let loud_enough = match ctx.speaker.owner_level_db {
            Some(ol) => e_db >= ol - self.p.onset_level_margin_db,
            None => true,
        };
        let mut onset = false;
        if voiced {
            if loud_enough
                && (self.silent_samples >= gap || e_db > self.slow_db + self.p.onset_jump_db)
            {
                onset = true;
            }
            self.silent_samples = 0;
            // 慢均值（约 300 ms），只跟有声 hop
            let a = one_pole_coef(0.3, self.p.sr / n as f32);
            self.slow_db += (e_db - self.slow_db) * a;
        } else {
            self.silent_samples = self.silent_samples.saturating_add(n as u64);
        }
        if onset {
            self.onsets += 1;
            self.last_onset = ctx.now;
            self.others_streak = 0;
            self.closing = false;
        }
        // 2) 吸收新判决（每份只算一次）
        let v = ctx.speaker;
        if v.window_end != self.last_verdict_end {
            self.last_verdict_end = v.window_end;
            match v.class {
                SpeakerClass::Other if v.valid_after(self.last_onset) => {
                    self.others_streak += 1;
                    if self.others_streak >= self.p.confirm_n {
                        self.closing = true;
                    }
                }
                SpeakerClass::Other | SpeakerClass::NoSpeech => {}
                SpeakerClass::Target | SpeakerClass::Unknown => {
                    self.others_streak = 0;
                    self.closing = false;
                }
            }
        }
        // 3) 增益：收 = 慢；起始后 100 ms 内放开 = 快；其余放开 = 中速
        let fast = self.onsets > 0 && ctx.now.since(self.last_onset) < (0.1 * self.p.sr) as u64;
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
    use crate::types::{Confidence, SpeakerVerdict};

    fn verdict(class: SpeakerClass, end: u64) -> SpeakerVerdict {
        SpeakerVerdict {
            class,
            confidence: Confidence::ZERO,
            window_start: SampleClock(end.saturating_sub(24000)),
            window_end: SampleClock(end),
            owner_level_db: None,
        }
    }

    fn ctx(v: SpeakerVerdict, pos: u64, p: f32) -> FrameCtx {
        FrameCtx {
            now: SampleClock(pos),
            speaker: v,
            voice_prob: Some(p),
        }
    }

    /// 只有别人在说（持续有声、无起始）→ 收到下限；然后停顿 0.3 s 再来一个人声 → 立刻放开
    #[test]
    fn closes_after_confirmed_then_reopens_on_onset() {
        let mut g = AsymGate::new(GateParams::default());
        let hop = 256u64;
        let mut wet = vec![0.01f32; hop as usize];
        let dry = wet.clone();
        let mut pos = 0u64;
        let mut v = SpeakerVerdict::UNKNOWN;
        for _ in 0..400 {
            if pos % 8000 < hop && pos >= 24000 {
                v = verdict(SpeakerClass::Other, pos);
            }
            wet.fill(0.01);
            g.process(&mut wet, &dry, ctx(v, pos, 0.9));
            pos += hop;
        }
        assert!(g.closing());
        assert!(g.gain() < 0.3, "应已收到接近下限：{}", g.gain());
        // 停顿 0.3 s（判决不变）
        for _ in 0..19 {
            wet.fill(0.0);
            g.process(&mut wet, &dry, ctx(v, pos, 0.0));
            pos += hop;
        }
        // 新人声：一个 hop 内放开
        wet.fill(0.01);
        g.process(&mut wet, &dry, ctx(v, pos, 0.9));
        assert!(!g.closing());
        assert!(g.gain() > 0.9, "起始没有立刻放开：{}", g.gain());
        // 起始之前算的旧 Other 判决（同一份或窗起点早于起始）不能再把它收回去
        for k in 0..60 {
            pos += hop;
            let stale = verdict(SpeakerClass::Other, pos - 100 + k);
            g.process(
                &mut wet,
                &dry,
                ctx(
                    SpeakerVerdict {
                        window_start: SampleClock(pos - 30000),
                        ..stale
                    },
                    pos,
                    0.9,
                ),
            );
        }
        assert!(!g.closing());
    }

    /// 已知主人电平时，比主人轻 10 dB 以上的起始（远场邻座）不触发放开
    #[test]
    fn quiet_onset_does_not_reopen_when_owner_level_known() {
        let mut g = AsymGate::new(GateParams::default());
        g.closing = true;
        g.g = g.floor;
        g.silent_samples = 100_000;
        let mut wet = vec![0.003f32; 256]; // ≈ −50 dB
        let dry = wet.clone();
        let v = SpeakerVerdict {
            owner_level_db: Some(-25.0),
            ..verdict(SpeakerClass::Other, 1000)
        };
        g.process(&mut wet, &dry, ctx(v, 50_000, 0.9));
        assert!(g.closing());
    }
}
