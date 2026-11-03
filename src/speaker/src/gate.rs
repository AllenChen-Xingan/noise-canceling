//! 不对称门控（003 §2.3、005 decision「门控是不对称的」）：降噪之后乘一个慢变增益 `g ∈ [floor, 1]`。
//!
//! - **快开（只看 VAD，不等声纹）**：检测到「新的人声起始」立刻 `g → 1`（时间常数 `attack_s`，几 ms）。起始有两种：
//!   ① 停顿 ≥ `onset_gap_s` 后 VAD 转有声；② 一直有人声但短时电平（最近 3 个 hop ≈ 48 ms）比近 1 s 均值高
//!   `onset_jump_db`（主人离麦近，插话时电平跳起来——覆盖「邻座还在说、主人接着开口」）。
//!   **已知主人电平时，两种起始都还要短时电平 ≥ 主人电平 − `onset_level_margin_db`**：邻座远场更轻，他每句话之间
//!   的停顿不该把门重新打开（exp005 第一轮：不加这条时每段 80 s 触发 600–980 次起始，门控根本收不住）。
//!   停顿后 VAD 刚转有声时，起始判据保持 `onset_arm_s`（100 ms）有效——句首软起音头一个 hop 不够响也不错过；
//!   电平跳起的起始有 `onset_refractory_s` 不应期，免得同一个人的重音峰反复刷新起始
//! - **电平口径**：短时电平量的是 `dry`（原声，与判断回路记主人电平的信号相同，见 `judge.rs`），不量包络混过原声的 wet——
//!   两边同一个信号，余量才不随信噪比漂（自审 P1-2）
//! - **慢收（要声纹确认）**：只有 `confirm_n` 份作数的 Other 判决（中间夹的 NoSpeech、起始前的 Other 不打断也不计数）、且判决窗**整个落在最近一次起始之后**
//!   （`SpeakerVerdict::valid_after`），才按 `release_s` 慢慢收到 `floor`。起始之前算的判决一律不作数——
//!   这是「你接着邻桌开口，第一个字不被吞」的关键
//! - Target / Unknown 判决 → 回到 1（`reopen_s`，比起始放开慢一点，避免误判来回抽）；NoSpeech → 保持（停顿里不抽动）
//! - 衰减下限 `floor_db`（初值 −12 dB，不压成真空）
//!
//! 零前瞻、零算法延迟。实时安全：`process` 只有算术，不分配。

use vdn_dsp::{db_to_lin, one_pole_coef};

use crate::vad::{Hysteresis, Vad};
use vdn_types::{FrameCtx, SampleClock, SpeakerClass, Stage};

#[derive(Clone, Copy, Debug)]
pub struct GateParams {
    /// 本级处理的采样率（模型采样率）
    pub sr: f32,
    /// `ctx.now` 的时钟频率（vdn-types：IO 采样率）。所有「秒 → 时钟」的换算用它；离线 = sr
    pub clock_rate: f32,
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
    /// **可选快路径（默认关 = None）**：已知主人电平时，连续 `level_close_hold_s` 秒「有人声但短时电平 < 主人电平 − 它」
    /// 就先开始慢收，不等声纹确认。偏离 003 §2.3「声纹确认才收」，作为待拍板项单独评测（exp005）
    pub level_close_db: Option<f32>,
    pub level_close_hold_s: f32,
    /// 停顿后 VAD 转有声起，起始判据保持有效的时长
    pub onset_arm_s: f32,
    /// 电平跳起型起始的不应期
    pub onset_refractory_s: f32,
    pub vad_on: f32,
    pub vad_off: f32,
}

impl Default for GateParams {
    /// 可调参数的默认值取自 `vdn_types::desc`（正本）；`sr` / `clock_rate` 是接线配置，不是可调参数。
    fn default() -> Self {
        use vdn_types::desc::*;
        Self {
            sr: vdn_types::MODEL_RATE as f32,
            clock_rate: vdn_types::MODEL_RATE as f32,
            floor_db: FLOOR_DB.default,
            attack_s: GATE_ATTACK_S.default,
            reopen_s: GATE_REOPEN_S.default,
            release_s: GATE_RELEASE_S.default,
            onset_gap_s: GATE_ONSET_GAP_S.default,
            onset_jump_db: GATE_ONSET_JUMP_DB.default,
            // 1 次确认在 exp005 上多压约 0.4 dB，但 c02（邻座近）句首误压 2.5% → 11.4%；按「宁可漏噪音也不吃字」取 2
            confirm_n: GATE_CONFIRM_N.default_count(),
            onset_level_margin_db: GATE_ONSET_LEVEL_MARGIN_DB.default,
            // 可缺省参数：出厂关
            level_close_db: None,
            level_close_hold_s: GATE_LEVEL_CLOSE_HOLD_S.default,
            onset_arm_s: GATE_ONSET_ARM_S.default,
            onset_refractory_s: GATE_ONSET_REFRACTORY_S.default,
            vad_on: VAD_ON.default,
            vad_off: VAD_OFF.default,
        }
    }
}

impl GateParams {
    /// 每个可调参数都在描述表范围内（解析命令行 / 配置时调；越界拒收）。
    /// 穷举解构：结构加了字段而这里没登记描述表，编译失败（防漏，照 schema 线 `vdn_shm::desc` 的办法）。
    pub fn validate(&self) -> Result<(), vdn_types::RangeError> {
        use vdn_types::desc::*;
        let GateParams {
            sr: _,
            clock_rate: _,
            floor_db,
            attack_s,
            reopen_s,
            release_s,
            onset_gap_s,
            onset_jump_db,
            confirm_n,
            onset_level_margin_db,
            level_close_db,
            level_close_hold_s,
            onset_arm_s,
            onset_refractory_s,
            vad_on,
            vad_off,
        } = *self;
        FLOOR_DB.check(floor_db)?;
        GATE_ATTACK_S.check(attack_s)?;
        GATE_REOPEN_S.check(reopen_s)?;
        GATE_RELEASE_S.check(release_s)?;
        GATE_ONSET_GAP_S.check(onset_gap_s)?;
        GATE_ONSET_JUMP_DB.check(onset_jump_db)?;
        GATE_CONFIRM_N.check_count(confirm_n)?;
        GATE_ONSET_LEVEL_MARGIN_DB.check(onset_level_margin_db)?;
        GATE_LEVEL_CLOSE_DB.check_opt(level_close_db)?;
        GATE_LEVEL_CLOSE_HOLD_S.check(level_close_hold_s)?;
        GATE_ONSET_ARM_S.check(onset_arm_s)?;
        GATE_ONSET_REFRACTORY_S.check(onset_refractory_s)?;
        VAD_ON.check(vad_on)?;
        VAD_OFF.check(vad_off)?;
        Ok(())
    }

    /// 夹进描述表范围（NaN 当默认值）。[`AsymGate::new`] 先夹再用，状态里不会出现越界参数。
    pub fn clamped(self) -> Self {
        use vdn_types::desc::*;
        Self {
            floor_db: FLOOR_DB.clamp(self.floor_db),
            attack_s: GATE_ATTACK_S.clamp(self.attack_s),
            reopen_s: GATE_REOPEN_S.clamp(self.reopen_s),
            release_s: GATE_RELEASE_S.clamp(self.release_s),
            onset_gap_s: GATE_ONSET_GAP_S.clamp(self.onset_gap_s),
            onset_jump_db: GATE_ONSET_JUMP_DB.clamp(self.onset_jump_db),
            confirm_n: GATE_CONFIRM_N.clamp_count(self.confirm_n),
            onset_level_margin_db: GATE_ONSET_LEVEL_MARGIN_DB.clamp(self.onset_level_margin_db),
            level_close_db: GATE_LEVEL_CLOSE_DB.clamp_opt(self.level_close_db),
            level_close_hold_s: GATE_LEVEL_CLOSE_HOLD_S.clamp(self.level_close_hold_s),
            onset_arm_s: GATE_ONSET_ARM_S.clamp(self.onset_arm_s),
            onset_refractory_s: GATE_ONSET_REFRACTORY_S.clamp(self.onset_refractory_s),
            vad_on: VAD_ON.clamp(self.vad_on),
            vad_off: VAD_OFF.clamp(self.vad_off),
            ..self
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
    /// 最近 3 个 hop 的能量（环形）
    short_e: [f32; 3],
    short_i: usize,
    quiet_voiced: u64,
    /// 停顿后转有声时，起始判据有效到这个时刻
    armed_until: SampleClock,
    last_onset: SampleClock,
    last_verdict_end: SampleClock,
    others_streak: u32,
    closing: bool,
    onsets: u64,
}

impl AsymGate {
    /// 参数先按描述表夹进范围（[`GateParams::clamped`]）；要拒收越界值，调用方先调 [`GateParams::validate`]。
    pub fn new(p: GateParams) -> Self {
        let p = p.clamped();
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
            short_e: [0.0; 3],
            short_i: 0,
            quiet_voiced: 0,
            armed_until: SampleClock(0),
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

// 实时：每 hop 在降噪线程（B 档）上调用。不分配、不加锁、不 panic（specs/rust/architecture.md §5.3、R1–R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
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
        self.short_e = [0.0; 3];
        self.short_i = 0;
        self.quiet_voiced = 0;
        self.armed_until = SampleClock(0);
        self.last_onset = SampleClock(0);
        self.last_verdict_end = SampleClock(0);
        self.others_streak = 0;
        self.closing = false;
        self.onsets = 0;
    }

    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx) {
        let n = wet.len();
        if n == 0 {
            return;
        }
        // 1) 起始检测：人声概率优先用上游给的（与包络同一个 VAD），没有就自己算
        let p = match ctx.voice_prob {
            Some(p) if p.is_finite() => p,
            _ => self.vad.push(wet),
        };
        let voiced = self.hyst.update(p);
        // 短时电平量原声（与主人电平同一口径）；非有限样本当 0
        let lvl_src = if dry.len() == n { dry } else { &wet[..] };
        let e = lvl_src
            .iter()
            .map(|v| if v.is_finite() { v * v } else { 0.0 })
            .sum::<f32>()
            / n as f32;
        if let Some(slot) = self.short_e.get_mut(self.short_i) {
            *slot = e;
        }
        self.short_i = (self.short_i + 1) % self.short_e.len();
        let st_db =
            10.0 * (self.short_e.iter().sum::<f32>() / self.short_e.len() as f32 + 1e-12).log10();
        let gap = (self.p.onset_gap_s * self.p.sr) as u64;
        let loud_enough = match ctx.speaker.owner_level_db {
            Some(ol) if ol.is_finite() => st_db >= ol - self.p.onset_level_margin_db,
            _ => true,
        };
        let mut onset = false;
        if voiced {
            if self.silent_samples >= gap {
                // 停顿后刚转有声：起始判据保持 onset_arm_s
                self.armed_until =
                    SampleClock(ctx.now.0 + (self.p.onset_arm_s * self.p.clock_rate) as u64);
            }
            let armed = ctx.now < self.armed_until;
            let refractory_over = self.onsets == 0
                || ctx.now.since(self.last_onset)
                    >= (self.p.onset_refractory_s * self.p.clock_rate) as u64;
            let jump = refractory_over && st_db > self.slow_db + self.p.onset_jump_db;
            if loud_enough && (armed || jump) {
                onset = true;
                self.armed_until = SampleClock(0);
            }
            self.silent_samples = 0;
            // 背景电平（约 1 s），只跟有声 hop；第一次有声直接取当前值，免得从 −120 dB 慢慢爬时一路判「电平跳起」
            if self.slow_db <= -119.0 || !self.slow_db.is_finite() {
                self.slow_db = st_db;
            } else {
                let a = one_pole_coef(1.0, self.p.sr / n as f32);
                self.slow_db += (st_db - self.slow_db) * a;
            }
        } else {
            self.silent_samples = self.silent_samples.saturating_add(n as u64);
        }
        if onset {
            self.onsets += 1;
            self.last_onset = ctx.now;
            self.others_streak = 0;
            self.closing = false;
            self.quiet_voiced = 0;
        }
        // 快路径：连续的「轻」人声 → 先收（可选）。停顿、起始、Target/Unknown 判决都让计数归零
        if let (Some(d), Some(ol)) = (self.p.level_close_db, ctx.speaker.owner_level_db) {
            if voiced && !onset && st_db < ol - d {
                self.quiet_voiced += n as u64;
                if self.quiet_voiced >= (self.p.level_close_hold_s * self.p.sr) as u64 {
                    self.closing = true;
                }
            } else {
                self.quiet_voiced = 0;
            }
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
                    self.quiet_voiced = 0;
                }
            }
        }
        // 3) 增益：收 = 慢；起始后 100 ms 内放开 = 快；其余放开 = 中速
        let fast =
            self.onsets > 0 && ctx.now.since(self.last_onset) < (0.1 * self.p.clock_rate) as u64;
        let (target, a) = if self.closing {
            (self.floor, self.a_rel)
        } else if fast {
            (1.0, self.a_att)
        } else {
            (1.0, self.a_reopen)
        };
        if !self.g.is_finite() {
            self.g = 1.0;
        }
        for w in wet.iter_mut() {
            self.g += (target - self.g) * a;
            *w *= self.g;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 描述表是正本：默认值逐条同表；取表的边界外一点造输入，validate 拒收（报的是那一条）、clamped 夹到边界。
    #[test]
    fn gate_params_follow_desc_table() {
        use vdn_types::desc::{self, ParamDesc};
        let d = GateParams::default();
        d.validate().unwrap();
        type Set = fn(&mut GateParams, f32);
        type Get = fn(&GateParams) -> f32;
        let fields: [(&ParamDesc, Get, Set); 13] = [
            (&desc::FLOOR_DB, |p| p.floor_db, |p, v| p.floor_db = v),
            (&desc::GATE_ATTACK_S, |p| p.attack_s, |p, v| p.attack_s = v),
            (&desc::GATE_REOPEN_S, |p| p.reopen_s, |p, v| p.reopen_s = v),
            (
                &desc::GATE_RELEASE_S,
                |p| p.release_s,
                |p, v| p.release_s = v,
            ),
            (
                &desc::GATE_ONSET_GAP_S,
                |p| p.onset_gap_s,
                |p, v| p.onset_gap_s = v,
            ),
            (
                &desc::GATE_ONSET_JUMP_DB,
                |p| p.onset_jump_db,
                |p, v| p.onset_jump_db = v,
            ),
            (
                &desc::GATE_ONSET_LEVEL_MARGIN_DB,
                |p| p.onset_level_margin_db,
                |p, v| p.onset_level_margin_db = v,
            ),
            (
                &desc::GATE_LEVEL_CLOSE_HOLD_S,
                |p| p.level_close_hold_s,
                |p, v| p.level_close_hold_s = v,
            ),
            (
                &desc::GATE_ONSET_ARM_S,
                |p| p.onset_arm_s,
                |p, v| p.onset_arm_s = v,
            ),
            (
                &desc::GATE_ONSET_REFRACTORY_S,
                |p| p.onset_refractory_s,
                |p, v| p.onset_refractory_s = v,
            ),
            (&desc::VAD_ON, |p| p.vad_on, |p, v| p.vad_on = v),
            (&desc::VAD_OFF, |p| p.vad_off, |p, v| p.vad_off = v),
            (
                &desc::GATE_LEVEL_CLOSE_DB,
                |p| p.level_close_db.unwrap_or(f32::NAN),
                |p, v| p.level_close_db = Some(v),
            ),
        ];
        for (dsc, get, set) in fields {
            if !dsc.optional {
                assert_eq!(get(&d), dsc.default, "{} 默认值", dsc.key);
            }
            for bad in [dsc.min - 0.5, dsc.max + 0.5] {
                let mut p = d;
                set(&mut p, bad);
                assert_eq!(p.validate().unwrap_err().desc.id, dsc.id, "{}", dsc.key);
                assert_eq!(get(&p.clamped()), dsc.clamp(bad), "{}", dsc.key);
            }
        }
        // 可缺省参数：出厂关，None 合法
        assert!(desc::GATE_LEVEL_CLOSE_DB.optional && d.level_close_db.is_none());
        // 整数参数
        assert_eq!(d.confirm_n, desc::GATE_CONFIRM_N.default_count());
        let p = GateParams { confirm_n: 0, ..d };
        assert_eq!(p.validate().unwrap_err().desc.id, desc::GATE_CONFIRM_N.id);
        assert_eq!(p.clamped().confirm_n, 1);
        // 接线配置不归描述表管，夹值时原样保留
        let p = GateParams {
            sr: vdn_types::IO_RATE as f32,
            clock_rate: vdn_types::IO_RATE as f32,
            ..d
        };
        assert_eq!(
            (p.clamped().sr, p.clamped().clock_rate),
            (vdn_types::IO_RATE as f32, vdn_types::IO_RATE as f32)
        );
    }

    /// new() 先夹再用：越界的下限不会让增益掉到表的下限之下。
    #[test]
    fn new_clamps_out_of_range_floor() {
        let g = AsymGate::new(GateParams {
            floor_db: -500.0,
            ..GateParams::default()
        });
        assert_eq!(g.floor, vdn_dsp::db_to_lin(vdn_types::desc::FLOOR_DB.min));
    }
    use vdn_types::{Confidence, SpeakerVerdict};

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
        let mut dry = wet.clone();
        let mut pos = 0u64;
        let mut v = SpeakerVerdict::default();
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
            dry.fill(0.0);
            g.process(&mut wet, &dry, ctx(v, pos, 0.0));
            pos += hop;
        }
        // 新人声：一个 hop 内放开
        wet.fill(0.01);
        dry.fill(0.01);
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

    /// 时钟是 48 kHz（IO 采样率）、处理是 16 kHz 时，起始后快放窗仍是 100 ms、起始前的旧判决仍被拒
    #[test]
    fn io_clock_domain() {
        let p = GateParams {
            clock_rate: vdn_types::IO_RATE as f32,
            ..GateParams::default()
        };
        let mut g = AsymGate::new(p);
        let mut wet = vec![0.01f32; 256];
        let dry = wet.clone();
        // 起始发生在时钟 480 000（10 s @48k）
        g.process(&mut wet, &dry, ctx(SpeakerVerdict::default(), 480_000, 0.9));
        assert_eq!(g.onsets(), 1);
        // 窗起点早于起始 → 不作数
        let stale = SpeakerVerdict {
            window_start: SampleClock(470_000),
            window_end: SampleClock(490_000),
            ..verdict(SpeakerClass::Other, 0)
        };
        for k in 0..3u64 {
            let v = SpeakerVerdict {
                window_end: SampleClock(490_000 + k),
                ..stale
            };
            g.process(&mut wet, &dry, ctx(v, 480_768 + k * 768, 0.9));
        }
        assert!(!g.closing());
        // 快放窗：起始后 4 800 个时钟（100 ms @48k）内算快放；用 since 换算检查
        assert!(
            SampleClock(480_000 + 4_799).since(SampleClock(480_000)) < (0.1 * p.clock_rate) as u64
        );
    }

    /// 门已收、主人电平已知：停顿后句首第一个 hop 很轻（软起音），下一个 hop 才够响 → 仍算起始、放开（自审 P1-1）
    #[test]
    fn soft_start_after_pause_still_reopens() {
        let mut g = AsymGate::new(GateParams::default());
        g.closing = true;
        g.g = g.floor;
        g.silent_samples = 100_000;
        g.slow_db = -30.0;
        g.onsets = 1;
        let v = SpeakerVerdict {
            owner_level_db: Some(-25.0),
            ..verdict(SpeakerClass::Other, 1000)
        };
        let mut pos = 200_000u64;
        // 第一个 hop −45 dB（软起音），之后 −22 dB
        for (k, amp) in [0.0056f32, 0.079, 0.079, 0.079].iter().enumerate() {
            let mut wet = vec![*amp; 256];
            let dry = wet.clone();
            g.process(&mut wet, &dry, ctx(v, pos, 0.9));
            pos += 256;
            if k == 0 {
                assert!(g.closing(), "软起音那一 hop 不该放开");
            }
        }
        assert!(!g.closing(), "句首够响后应放开");
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
