//! 判断回路（约 0.5 s 一次，非实时线程）：最近 1.5 s 的降噪输出 → 声纹 → 喂注册器 / 对主人打分 → 发布 [`SpeakerVerdict`]。
//!
//! **只改参数、不碰声音、不占实时回路延迟**：实时线程每 hop 只把音频和 VAD 概率送过来（实时版用 rtrb 环），
//! 判决经 triple_buffer 回去（规约 §3.4）。本模块是线程无关的纯逻辑，离线实验里由驱动按时间顺序调用。
//!
//! 片段怎么挑（注册用）：窗里 VAD 有声帧 ≥ `enroll_voiced_frac`，且与上一个窗（重叠 1 s）声纹相似度 ≥ `consist_sim`
//! ——连续两个窗像同一个人，说明窗里没有换人、没有明显重叠（003 §2.2 第 2 条「清楚的单人片段」）。
//!
//! 判决（相似度 = 与主人声纹的余弦）：≥ `owner_hi` → Target；≤ `others_lo` → Other；两者之间 → Unknown（不压）。
//! 滞回：上一份「有人声」的判决是 Other 且相似度仍 < 两档中点，维持 Other（中间的 NoSpeech 不打断）。
//! 电平（喂注册器的近讲线索、发给门控的主人电平）在**原声**上量，与门控量起始电平同一口径。

#![allow(clippy::indexing_slicing)] // NRT: 判断回路（C 档线程，约 0.5 s 一次），不在降噪线程上跑

use std::time::{Duration, Instant};

use crate::enroll::{EnrollOutcome, Enroller};
use crate::vad::VAD_FRAME;
use crate::vecmath::{BoxError, all_finite, cosine};
use vdn_types::{Confidence, SampleClock, SpeakerClass, SpeakerVerdict};

/// 声纹来源（真模型或测试用的假模型）。
pub trait EmbedSource {
    fn embed(&mut self, x: &[f32]) -> Result<Vec<f32>, BoxError>;
}

#[cfg(feature = "onnx")]
impl EmbedSource for crate::embed::Embedder {
    fn embed(&mut self, x: &[f32]) -> Result<Vec<f32>, BoxError> {
        crate::embed::Embedder::embed(self, x)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JudgeParams {
    pub sr: u32,
    pub window_s: f32,
    pub hop_s: f32,
    /// 窗里有声帧比例低于它 → NoSpeech
    pub min_voiced_frac: f32,
    pub enroll_voiced_frac: f32,
    pub consist_sim: f32,
    pub owner_hi: f32,
    pub others_lo: f32,
    /// VAD 帧算「有声」的概率门限（硬门限；实时级用 0.5/0.35 滞回，口径不同：这里只为数「窗里多少帧有声」）
    pub vad_thr: f32,
    /// 一个模型样本对应几个时钟单位（时钟 = vdn-types 的 SampleClock，IO 采样率；离线 16 kHz 时为 1，48 kHz IO 时为 3）
    pub clock_per_sample: u64,
}

impl Default for JudgeParams {
    /// 可调参数的默认值取自 `vdn_types::desc`（`judge.*`）；`sr` / `clock_per_sample` 是接线配置。
    fn default() -> Self {
        use vdn_types::desc::*;
        Self {
            sr: vdn_types::MODEL_RATE,
            window_s: JUDGE_WINDOW_S.default,
            hop_s: JUDGE_HOP_S.default,
            min_voiced_frac: JUDGE_MIN_VOICED_FRAC.default,
            enroll_voiced_frac: JUDGE_ENROLL_VOICED_FRAC.default,
            consist_sim: JUDGE_CONSIST_SIM.default,
            owner_hi: JUDGE_OWNER_HI.default,
            others_lo: JUDGE_OTHERS_LO.default,
            vad_thr: JUDGE_VAD_THR.default,
            clock_per_sample: 1,
        }
    }
}

impl JudgeParams {
    /// 每个可调参数都在描述表范围内（越界拒收）。穷举解构：加了字段没登记描述表，编译失败。
    pub fn validate(&self) -> Result<(), vdn_types::RangeError> {
        use vdn_types::desc::*;
        let JudgeParams {
            sr: _,
            window_s,
            hop_s,
            min_voiced_frac,
            enroll_voiced_frac,
            consist_sim,
            owner_hi,
            others_lo,
            vad_thr,
            clock_per_sample: _,
        } = *self;
        JUDGE_WINDOW_S.check(window_s)?;
        JUDGE_HOP_S.check(hop_s)?;
        JUDGE_MIN_VOICED_FRAC.check(min_voiced_frac)?;
        JUDGE_ENROLL_VOICED_FRAC.check(enroll_voiced_frac)?;
        JUDGE_CONSIST_SIM.check(consist_sim)?;
        JUDGE_OWNER_HI.check(owner_hi)?;
        JUDGE_OTHERS_LO.check(others_lo)?;
        JUDGE_VAD_THR.check(vad_thr)?;
        Ok(())
    }

    /// 夹进描述表范围（NaN 当默认值）；[`Judge::new`] 先夹再用。
    pub fn clamped(self) -> Self {
        use vdn_types::desc::*;
        Self {
            window_s: JUDGE_WINDOW_S.clamp(self.window_s),
            hop_s: JUDGE_HOP_S.clamp(self.hop_s),
            min_voiced_frac: JUDGE_MIN_VOICED_FRAC.clamp(self.min_voiced_frac),
            enroll_voiced_frac: JUDGE_ENROLL_VOICED_FRAC.clamp(self.enroll_voiced_frac),
            consist_sim: JUDGE_CONSIST_SIM.clamp(self.consist_sim),
            owner_hi: JUDGE_OWNER_HI.clamp(self.owner_hi),
            others_lo: JUDGE_OTHERS_LO.clamp(self.others_lo),
            vad_thr: JUDGE_VAD_THR.clamp(self.vad_thr),
            ..self
        }
    }
}

/// 每次判断的记录（离线分析用）。
#[derive(Clone, Debug)]
pub struct JudgeLog {
    pub window_end: u64,
    pub voiced_frac: f32,
    pub level_db: f32,
    pub owner_sim: f32,
    pub prev_sim: f32,
    /// 这个窗被当作「清楚的单人片段」喂给了注册器
    pub enrolled: bool,
    pub event: EnrollOutcome,
    pub class: SpeakerClass,
    pub compute: Duration,
}

pub struct Judge<E: EmbedSource> {
    p: JudgeParams,
    emb: E,
    pub enroller: Enroller,
    /// 最近一个窗的音频（环形，按时间顺序复制出来再算）
    ring: Vec<f32>,
    ring_pos: usize,
    filled: usize,
    vad_ring: Vec<f32>,
    vad_pos: usize,
    vad_filled: usize,
    window: Vec<f32>,
    /// 判电平用的原声（与门控量起始电平的信号相同，自审 P1-2）
    lvl_ring: Vec<f32>,
    lvl_window: Vec<f32>,
    pos: u64,
    /// 最近一次送进来的音频块末尾对应的时钟（由调用方给，不自己数——实时版环满会丢块）
    clock_end: u64,
    /// 上一份「有人声」的判决类别（滞回只看有人声的判决，NoSpeech 不打断）
    last_speech_class: SpeakerClass,
    next_tick: u64,
    prev_emb: Option<Vec<f32>>,
    last: SpeakerVerdict,
    /// 是否允许发 Other（关掉 = 只注册、不个性化）
    pub personalize: bool,
    pub logs: Vec<JudgeLog>,
}

impl<E: EmbedSource> Judge<E> {
    pub fn new(p: JudgeParams, emb: E, enroller: Enroller) -> Self {
        let p = p.clamped();
        let wn = (p.window_s * p.sr as f32) as usize;
        let vn = wn / VAD_FRAME;
        Self {
            ring: vec![0.0; wn],
            ring_pos: 0,
            filled: 0,
            vad_ring: vec![0.0; vn.max(1)],
            vad_pos: 0,
            vad_filled: 0,
            window: vec![0.0; wn],
            lvl_ring: vec![0.0; wn],
            lvl_window: vec![0.0; wn],
            pos: 0,
            clock_end: 0,
            last_speech_class: SpeakerClass::Unknown,
            next_tick: (p.hop_s * p.sr as f32) as u64,
            prev_emb: None,
            last: SpeakerVerdict::default(),
            personalize: true,
            logs: Vec::new(),
            p,
            emb,
            enroller,
        }
    }

    pub fn params(&self) -> JudgeParams {
        self.p
    }

    pub fn last(&self) -> SpeakerVerdict {
        self.last
    }

    /// 送一段音频：`wet` = 降噪输出（算声纹），`dry` = 同一段原声（量电平），`start` = 这段首样本的时钟。
    pub fn push_audio(&mut self, wet: &[f32], dry: &[f32], start: SampleClock) {
        let n = self.ring.len();
        for (i, &s) in wet.iter().enumerate() {
            self.ring[self.ring_pos] = s;
            self.lvl_ring[self.ring_pos] = dry.get(i).copied().unwrap_or(0.0);
            self.ring_pos = (self.ring_pos + 1) % n;
        }
        self.filled = (self.filled + wet.len()).min(n);
        self.pos += wet.len() as u64;
        self.clock_end = start.0 + wet.len() as u64 * self.p.clock_per_sample;
    }

    /// 送一个 VAD 帧概率（每 16 ms 一个）。
    pub fn push_vad(&mut self, prob: f32) {
        let n = self.vad_ring.len();
        self.vad_ring[self.vad_pos] = prob;
        self.vad_pos = (self.vad_pos + 1) % n;
        self.vad_filled = (self.vad_filled + 1).min(n);
    }

    /// 到点了就判断一次，返回新判决。
    pub fn tick(&mut self) -> Option<SpeakerVerdict> {
        if self.pos < self.next_tick {
            return None;
        }
        self.next_tick += (self.p.hop_s * self.p.sr as f32) as u64;
        if self.filled < self.ring.len() {
            return None;
        }
        let t0 = Instant::now();
        let n = self.ring.len();
        for i in 0..n {
            self.window[i] = self.ring[(self.ring_pos + i) % n];
            self.lvl_window[i] = self.lvl_ring[(self.ring_pos + i) % n];
        }
        // 判决窗的时钟：由调用方给的时钟推回窗长
        let end = self.clock_end;
        let start = end.saturating_sub(n as u64 * self.p.clock_per_sample);
        // 有声帧比例、有声帧电平
        let vn = self.vad_filled;
        let voiced: Vec<bool> = (0..vn)
            .map(|i| {
                self.vad_ring[(self.vad_pos + self.vad_ring.len() - vn + i) % self.vad_ring.len()]
                    > self.p.vad_thr
            })
            .collect();
        let vf = voiced.iter().filter(|&&v| v).count() as f32 / vn.max(1) as f32;
        // VAD 帧与窗尾对齐：最后 vn 帧对应窗的最后 vn·256 个样本
        let off = n.saturating_sub(vn * VAD_FRAME);
        let (mut e, mut cnt) = (0.0f64, 0usize);
        for (i, &v) in voiced.iter().enumerate() {
            if v {
                let a = off + i * VAD_FRAME;
                let fr = &self.lvl_window[a..(a + VAD_FRAME).min(n)];
                e += fr.iter().map(|&x| (x as f64) * (x as f64)).sum::<f64>();
                cnt += fr.len();
            }
        }
        let level_db = if cnt > 0 {
            10.0 * ((e / cnt as f64) + 1e-12).log10() as f32
        } else {
            f32::NEG_INFINITY
        };
        let mut log = JudgeLog {
            window_end: self.pos,
            voiced_frac: vf,
            level_db,
            owner_sim: f32::NAN,
            prev_sim: f32::NAN,
            enrolled: false,
            event: EnrollOutcome::None,
            class: SpeakerClass::NoSpeech,
            compute: Duration::ZERO,
        };
        let class = if vf < self.p.min_voiced_frac {
            self.prev_emb = None;
            SpeakerClass::NoSpeech
        } else {
            match self.emb.embed(&self.window) {
                Ok(v) if !all_finite(&v) => {
                    self.prev_emb = None;
                    SpeakerClass::Unknown
                }
                Err(_) => {
                    self.prev_emb = None;
                    SpeakerClass::Unknown
                }
                Ok(v) => {
                    let ps = self
                        .prev_emb
                        .as_ref()
                        .map(|p| cosine(p, &v))
                        .unwrap_or(f32::NAN);
                    log.prev_sim = ps;
                    if vf >= self.p.enroll_voiced_frac && ps >= self.p.consist_sim {
                        // 新增人声时长 = hop × 最近一个 hop 里的有声帧比例（不是固定 hop，自审 P2-6）
                        let k = ((self.p.hop_s * self.p.sr as f32) as usize / VAD_FRAME)
                            .clamp(1, voiced.len().max(1));
                        let recent = &voiced[voiced.len().saturating_sub(k)..];
                        let dur =
                            self.p.hop_s * recent.iter().filter(|&&v| v).count() as f32 / k as f32;
                        log.event = self.enroller.observe(
                            &v,
                            self.pos as f64 / self.p.sr as f64,
                            dur,
                            level_db,
                        );
                        log.enrolled = true;
                    }
                    let c = match self.enroller.owner() {
                        None => SpeakerClass::Unknown,
                        Some(o) => {
                            let s = cosine(o, &v);
                            log.owner_sim = s;
                            let mid = 0.5 * (self.p.owner_hi + self.p.others_lo);
                            if s >= self.p.owner_hi {
                                SpeakerClass::Target
                            } else if s <= self.p.others_lo
                                || (self.last_speech_class == SpeakerClass::Other && s < mid)
                            {
                                SpeakerClass::Other
                            } else {
                                SpeakerClass::Unknown
                            }
                        }
                    };
                    self.prev_emb = Some(v);
                    c
                }
            }
        };
        let class = if !self.personalize && class == SpeakerClass::Other {
            SpeakerClass::Unknown
        } else {
            class
        };
        if class != SpeakerClass::NoSpeech {
            self.last_speech_class = class;
        }
        log.class = class;
        log.compute = t0.elapsed();
        self.last = SpeakerVerdict {
            class,
            confidence: Confidence::new(log.owner_sim),
            window_start: SampleClock(start),
            window_end: SampleClock(end),
            owner_level_db: self
                .enroller
                .owner_index()
                .map(|i| self.enroller.clusters()[i].level_db)
                .filter(|l| l.is_finite()),
        };
        self.logs.push(log);
        Some(self.last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 描述表是正本：默认值逐条同表；越界 validate 拒收（报的是那一条）、clamped 夹到边界。
    #[test]
    fn judge_params_follow_desc_table() {
        use vdn_types::desc::{self, ParamDesc};
        let d = JudgeParams::default();
        d.validate().unwrap();
        type Set = fn(&mut JudgeParams, f32);
        type Get = fn(&JudgeParams) -> f32;
        let fields: [(&ParamDesc, Get, Set); 8] = [
            (&desc::JUDGE_WINDOW_S, |p| p.window_s, |p, v| p.window_s = v),
            (&desc::JUDGE_HOP_S, |p| p.hop_s, |p, v| p.hop_s = v),
            (
                &desc::JUDGE_MIN_VOICED_FRAC,
                |p| p.min_voiced_frac,
                |p, v| p.min_voiced_frac = v,
            ),
            (
                &desc::JUDGE_ENROLL_VOICED_FRAC,
                |p| p.enroll_voiced_frac,
                |p, v| p.enroll_voiced_frac = v,
            ),
            (
                &desc::JUDGE_CONSIST_SIM,
                |p| p.consist_sim,
                |p, v| p.consist_sim = v,
            ),
            (&desc::JUDGE_OWNER_HI, |p| p.owner_hi, |p, v| p.owner_hi = v),
            (
                &desc::JUDGE_OTHERS_LO,
                |p| p.others_lo,
                |p, v| p.others_lo = v,
            ),
            (&desc::JUDGE_VAD_THR, |p| p.vad_thr, |p, v| p.vad_thr = v),
        ];
        for (dsc, get, set) in fields {
            assert_eq!(get(&d), dsc.default, "{} 默认值", dsc.key);
            for bad in [dsc.min - 0.5, dsc.max + 0.5] {
                let mut p = d;
                set(&mut p, bad);
                assert_eq!(p.validate().unwrap_err().desc.id, dsc.id, "{}", dsc.key);
                assert_eq!(get(&p.clamped()), dsc.clamp(bad), "{}", dsc.key);
            }
        }
    }
}
