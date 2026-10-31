//! 判断回路（约 0.5 s 一次，非实时线程）：最近 1.5 s 的降噪输出 → 声纹 → 喂注册器 / 对主人打分 → 发布 [`SpeakerVerdict`]。
//!
//! **只改参数、不碰声音、不占实时回路延迟**：实时线程每 hop 只把音频和 VAD 概率送过来（实时版用 rtrb 环），
//! 判决经 triple_buffer 回去（规约 §3.4）。本模块是线程无关的纯逻辑，离线实验里由驱动按时间顺序调用。
//!
//! 片段怎么挑（注册用）：窗里 VAD 有声帧 ≥ `enroll_voiced_frac`，且与上一个窗（重叠 1 s）声纹相似度 ≥ `consist_sim`
//! ——连续两个窗像同一个人，说明窗里没有换人、没有明显重叠（003 §2.2 第 2 条「清楚的单人片段」）。
//!
//! 判决（相似度 = 与主人声纹的余弦）：≥ `owner_hi` → Owner；≤ `others_lo` → OthersOnly；两者之间 → Unknown（不压）。
//! 滞回：上一份是 OthersOnly 且相似度仍 < 两档中点，维持 OthersOnly。

use std::time::{Duration, Instant};

use crate::embed::{BoxError, cosine};
use crate::enroll::{EnrollEvent, Enroller};
use crate::types::{SpeakerClass, SpeakerVerdict};
use crate::vad::VAD_FRAME;

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
    /// VAD 帧算「有声」的概率门限
    pub vad_thr: f32,
}

impl Default for JudgeParams {
    fn default() -> Self {
        Self {
            sr: 16000,
            window_s: 1.5,
            hop_s: 0.5,
            min_voiced_frac: 0.3,
            enroll_voiced_frac: 0.6,
            consist_sim: 0.6,
            owner_hi: 0.45,
            others_lo: 0.25,
            vad_thr: 0.5,
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
    pub enrolled_cluster: Option<usize>,
    pub event: EnrollEvent,
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
    pos: u64,
    next_tick: u64,
    prev_emb: Option<Vec<f32>>,
    last: SpeakerVerdict,
    /// 是否允许发 OthersOnly（关掉 = 只注册、不个性化）
    pub personalize: bool,
    pub logs: Vec<JudgeLog>,
}

impl<E: EmbedSource> Judge<E> {
    pub fn new(p: JudgeParams, emb: E, enroller: Enroller) -> Self {
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
            pos: 0,
            next_tick: (p.hop_s * p.sr as f32) as u64,
            prev_emb: None,
            last: SpeakerVerdict::UNKNOWN,
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

    /// 送一段音频（降噪输出）。
    pub fn push_audio(&mut self, x: &[f32]) {
        let n = self.ring.len();
        for &s in x {
            self.ring[self.ring_pos] = s;
            self.ring_pos = (self.ring_pos + 1) % n;
        }
        self.filled = (self.filled + x.len()).min(n);
        self.pos += x.len() as u64;
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
        }
        let (start, end) = (self.pos - n as u64, self.pos);
        // 有声帧比例、有声帧电平
        let vn = self.vad_filled;
        let voiced: Vec<bool> = (0..vn)
            .map(|i| self.vad_ring[(self.vad_pos + self.vad_ring.len() - vn + i) % self.vad_ring.len()] > self.p.vad_thr)
            .collect();
        let vf = voiced.iter().filter(|&&v| v).count() as f32 / vn.max(1) as f32;
        // VAD 帧与窗尾对齐：最后 vn 帧对应窗的最后 vn·256 个样本
        let off = n.saturating_sub(vn * VAD_FRAME);
        let (mut e, mut cnt) = (0.0f64, 0usize);
        for (i, &v) in voiced.iter().enumerate() {
            if v {
                let a = off + i * VAD_FRAME;
                let fr = &self.window[a..(a + VAD_FRAME).min(n)];
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
            window_end: end,
            voiced_frac: vf,
            level_db,
            owner_sim: f32::NAN,
            prev_sim: f32::NAN,
            enrolled_cluster: None,
            event: EnrollEvent::None,
            class: SpeakerClass::NoSpeech,
            compute: Duration::ZERO,
        };
        let class = if vf < self.p.min_voiced_frac {
            self.prev_emb = None;
            SpeakerClass::NoSpeech
        } else {
            match self.emb.embed(&self.window) {
                Err(_) => {
                    self.prev_emb = None;
                    SpeakerClass::Unknown
                }
                Ok(v) => {
                    let ps = self.prev_emb.as_ref().map(|p| cosine(p, &v)).unwrap_or(f32::NAN);
                    log.prev_sim = ps;
                    if vf >= self.p.enroll_voiced_frac && ps >= self.p.consist_sim {
                        let dur = self.p.hop_s;
                        log.event = self.enroller.observe(&v, end as f64 / self.p.sr as f64, dur, level_db);
                        log.enrolled_cluster = Some(usize::MAX);
                    }
                    let c = match self.enroller.owner() {
                        None => SpeakerClass::Unknown,
                        Some(o) => {
                            let s = cosine(o, &v);
                            log.owner_sim = s;
                            let mid = 0.5 * (self.p.owner_hi + self.p.others_lo);
                            if s >= self.p.owner_hi {
                                SpeakerClass::Owner
                            } else if s <= self.p.others_lo
                                || (self.last.class == SpeakerClass::OthersOnly && s < mid)
                            {
                                SpeakerClass::OthersOnly
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
        let class = if !self.personalize && class == SpeakerClass::OthersOnly {
            SpeakerClass::Unknown
        } else {
            class
        };
        log.class = class;
        log.compute = t0.elapsed();
        self.last = SpeakerVerdict {
            class,
            window_start: start,
            window_end: end,
            owner_sim: log.owner_sim,
        };
        self.logs.push(log);
        Some(self.last)
    }
}
