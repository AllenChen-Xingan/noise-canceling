//! 确定性场景合成器：主讲人 · 干扰者 · 噪音三轨分存，外加逐音节的真值时间线。
//!
//! 为什么自己合成：CI 不能带音频、不能联网，而 V-SIM-* 要「哪一帧谁在说话」的事实（012 §3.4）。
//! 合成语音是**类语音**信号（声门脉冲 + 三个共振峰 + 音节包络 + 擦音 + 爆破起音），不是可懂的话——
//! 它够格测能量门控、字头、电平、背景稳定、音乐噪声这些物理 / 信号层的事，**不够格测「吃字」「好不好听」**，
//! 那些在 QA 层用真实录音测。scene 线的模拟器合并后，这里换成它（接口：同样产出三轨 + 时间线）。
//!
//! 真值规则（写死、进仓，决定了「真值」是什么）：一个音节从包络起点到包络终点算「在说话」；
//! 不加 hangover。门控类 verifier 自己声明它容许多长的拖尾（`verifiers.rs` 的 `GATE_TOLERANCE_S`）。

use crate::analysis::active_level_db;
use crate::rng::Rng;

pub const SR: u32 = vdn_types::MODEL_RATE;

/// 噪音种类。清单里的名字由 serde 定（小写），格式正本见 `files.rs`。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum NoiseKind {
    None,
    White,
    Pink,
    /// 风扇 / 空调：低频为主的平稳噪声 + 100 Hz 嗡声
    Hum,
    /// 一大片人：6 个合成说话人叠加
    Babble,
    /// 咖啡馆：粉噪 + 杯碟碰撞（衰减正弦，泊松到达）
    Cafe,
    /// 背景音乐：和弦序列
    Music,
}

/// 干扰者怎么说话。清单里的名字由 serde 定（snake_case）。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum InterfMode {
    /// 只在主讲人的停顿里说（轮流）
    Alternate,
    /// 自顾自说，和主讲人重叠
    Overlap,
    /// 主讲人开口前先说一大段，之后轮流（exp005 c04 `other_first`）
    OtherFirst,
}

#[derive(Clone, Debug)]
pub struct TalkerSpec {
    /// P.56 主动语音电平（dBov）
    pub level_dbov: f64,
    pub f0_lo: f64,
    pub f0_hi: f64,
    /// 声调语言式的音节内 f0 变化（中文类）
    pub tonal: bool,
}

#[derive(Clone, Debug)]
pub struct InterfSpec {
    pub mode: InterfMode,
    /// 主讲人电平 − 干扰者电平（dB）
    pub sir_db: f64,
    pub f0_lo: f64,
    pub f0_hi: f64,
}

#[derive(Clone, Debug)]
pub struct SceneSpec {
    pub dur_s: f64,
    pub seed: u64,
    pub owner: Option<TalkerSpec>,
    /// 主讲人第一次开口的时刻（之前只有噪音 / 干扰者；P.1120 §11.14.5 要求先放 ≥ 5 s 噪声）
    pub owner_start_s: f64,
    /// 主讲人两句之间的停顿（秒，区间）
    pub pause_s: (f64, f64),
    pub interferers: Vec<InterfSpec>,
    pub noise: NoiseKind,
    /// 主讲人主动电平 − 噪音 RMS 电平（dB）；没有主讲人时为噪音的绝对 dBov
    pub snr_db: f64,
    /// 结尾的数字静音（所有轨全零）
    pub tail_silence_s: f64,
}

/// 合成结果：三轨分存 + 混合 + 真值。
#[derive(Clone, Debug)]
pub struct Clip {
    pub sr: u32,
    /// 主讲人（期望输出）
    pub clean: Vec<f32>,
    pub interf: Vec<f32>,
    pub noise: Vec<f32>,
    pub mix: Vec<f32>,
    pub truth: Truth,
    /// 有没有逐轨分存（真实录音只有 clean + mix 时为 false，noise = mix − clean）
    pub tracks: bool,
}

/// 逐段真值（样本区间，左闭右开）。
#[derive(Clone, Debug, Default)]
pub struct Truth {
    pub owner: Vec<(usize, usize)>,
    pub interf: Vec<(usize, usize)>,
}

impl Clip {
    pub fn len(&self) -> usize {
        self.mix.len()
    }
    pub fn is_empty(&self) -> bool {
        self.mix.is_empty()
    }
}

// ───────────────────────── 语音积木 ─────────────────────────

/// 二阶共振器（极点半径由带宽定，增益按峰值约归一）。
struct Reso {
    a1: f64,
    a2: f64,
    g: f64,
    y1: f64,
    y2: f64,
}

impl Reso {
    fn new(f: f64, bw: f64, sr: f64) -> Reso {
        let r = (-std::f64::consts::PI * bw / sr).exp();
        let th = 2.0 * std::f64::consts::PI * f / sr;
        Reso {
            a1: 2.0 * r * th.cos(),
            a2: -r * r,
            g: (1.0 - r) * 2.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
    fn tick(&mut self, x: f64) -> f64 {
        let y = self.g * x + self.a1 * self.y1 + self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

const VOWELS: [(f64, f64, f64); 5] = [
    (730.0, 1090.0, 2440.0),
    (270.0, 2290.0, 3010.0),
    (300.0, 870.0, 2240.0),
    (530.0, 1840.0, 2480.0),
    (570.0, 840.0, 2410.0),
];

/// 升余弦起落包络。
fn env_at(t: f64, dur: f64, attack: f64, release: f64) -> f64 {
    if t < 0.0 || t >= dur {
        0.0
    } else if t < attack {
        0.5 - 0.5 * (std::f64::consts::PI * t / attack).cos()
    } else if t > dur - release {
        let u = (dur - t) / release;
        0.5 - 0.5 * (std::f64::consts::PI * u).cos()
    } else {
        1.0
    }
}

/// 在 `buf[start..]` 上叠加一个音节，返回实际写入的终点。
fn render_syllable(
    rng: &mut Rng,
    t: &TalkerSpec,
    f0_base: f64,
    buf: &mut [f32],
    start: usize,
    dur: usize,
) -> usize {
    let sr = SR as f64;
    let end = (start + dur).min(buf.len());
    if end <= start {
        return start;
    }
    let d = (end - start) as f64 / sr;
    let fricative = rng.uniform() < 0.2;
    if fricative {
        let fc = rng.range(3500.0, 5500.0);
        let mut r = Reso::new(fc, 1800.0, sr);
        let amp = rng.range(0.25, 0.45);
        for (i, s) in buf[start..end].iter_mut().enumerate() {
            let tt = i as f64 / sr;
            let e = env_at(tt, d, 0.012, 0.03);
            *s += (amp * e * r.tick(rng.normal())) as f32;
        }
        return end;
    }
    let (f1, f2, f3) = VOWELS[rng.below(VOWELS.len())];
    let jitter = rng.range(0.92, 1.08);
    let mut r1 = Reso::new(f1 * jitter, 90.0, sr);
    let mut r2 = Reso::new(f2 * jitter, 110.0, sr);
    let mut r3 = Reso::new(f3, 150.0, sr);
    // 声调：0 平 · 1 升 · 2 降 · 3 降升
    let tone = if t.tonal { rng.below(4) } else { 0 };
    let plosive = rng.uniform() < 0.4;
    let mut phase = 0.0f64;
    let mut last_pulse = 0.0f64;
    for (i, s) in buf[start..end].iter_mut().enumerate() {
        let tt = i as f64 / sr;
        let u = tt / d;
        let contour = match tone {
            1 => 0.85 + 0.3 * u,
            2 => 1.15 - 0.3 * u,
            3 => 1.0 - 0.6 * u * (1.0 - u) * 2.0,
            _ => 1.0 - 0.05 * u,
        };
        let f0 = f0_base * contour;
        phase += f0 / sr;
        // 声门脉冲：每周期一个尖脉冲，后接指数衰减（粗略的开相）
        let mut exc = 0.0;
        if phase >= 1.0 {
            phase -= 1.0;
            last_pulse = tt;
            exc += 1.0;
        }
        exc += 0.6 * (-(tt - last_pulse) * 2000.0).exp() * 0.05;
        exc += 0.02 * rng.normal();
        let v = r1.tick(exc) + 0.6 * r2.tick(exc) + 0.3 * r3.tick(exc);
        let mut e = env_at(tt, d, 0.015, 0.04);
        // 爆破起音：前 6 ms 一小段宽带噪声，让字头有个陡沿
        let burst = if plosive && tt < 0.006 {
            0.4 * rng.normal()
        } else {
            0.0
        };
        if plosive && tt < 0.015 {
            e = e.max(0.3);
        }
        *s += (e * v * 6.0 + burst) as f32;
    }
    end
}

/// 在 `[from, to)` 内渲染一句话，音节区间写进 `ivs`。
fn render_utterance(
    rng: &mut Rng,
    t: &TalkerSpec,
    buf: &mut [f32],
    from: usize,
    to: usize,
    ivs: &mut Vec<(usize, usize)>,
) {
    let sr = SR as f64;
    let f0_base = rng.range(t.f0_lo, t.f0_hi);
    let mut pos = from;
    while pos < to {
        let n_syl = 1 + rng.below(3);
        for k in 0..n_syl {
            let dur = (rng.range(0.12, 0.28) * sr) as usize;
            if pos + dur > to {
                return;
            }
            // 句内 f0 缓降
            let decl = 1.0 - 0.15 * (pos - from) as f64 / (to - from).max(1) as f64;
            let end = render_syllable(rng, t, f0_base * decl, buf, pos, dur);
            ivs.push((pos, end));
            pos = end;
            if k + 1 < n_syl {
                pos += (rng.range(0.0, 0.02) * sr) as usize;
            }
        }
        pos += (rng.range(0.05, 0.13) * sr) as usize;
    }
}

/// 一个说话人的「独白」：在 `[from, to)` 里一句接一句，句间停顿 `pause`。返回各句区间。
fn monologue(
    rng: &mut Rng,
    t: &TalkerSpec,
    buf: &mut [f32],
    from: usize,
    to: usize,
    pause: (f64, f64),
    ivs: &mut Vec<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let sr = SR as f64;
    let mut utts = Vec::new();
    let mut pos = from;
    while pos < to {
        let len = (rng.range(1.5, 4.0) * sr) as usize;
        let end = (pos + len).min(to);
        if end - pos > (0.5 * sr) as usize {
            render_utterance(rng, t, buf, pos, end, ivs);
            utts.push((pos, end));
        }
        pos = end + (rng.range(pause.0, pause.1) * sr) as usize;
    }
    utts
}

// ───────────────────────── 噪音 ─────────────────────────

/// 粉噪（Paul Kellet 的经济型滤波，误差约 ±0.05 dB/oct 量级；作测试噪声足够）。
fn pink(rng: &mut Rng, n: usize) -> Vec<f32> {
    let (mut b0, mut b1, mut b2) = (0.0f64, 0.0f64, 0.0f64);
    (0..n)
        .map(|_| {
            let w = rng.normal();
            b0 = 0.99765 * b0 + w * 0.0990460;
            b1 = 0.96300 * b1 + w * 0.2965164;
            b2 = 0.57000 * b2 + w * 1.0526913;
            (b0 + b1 + b2 + w * 0.1848) as f32
        })
        .collect()
}

fn noise_track(kind: NoiseKind, rng: &mut Rng, n: usize) -> Vec<f32> {
    let sr = SR as f64;
    match kind {
        NoiseKind::None => vec![0.0; n],
        NoiseKind::White => (0..n).map(|_| rng.normal() as f32).collect(),
        NoiseKind::Pink => pink(rng, n),
        NoiseKind::Hum => {
            let p = pink(rng, n);
            let mut lp = 0.0f64;
            let a = 1.0 - (-2.0 * std::f64::consts::PI * 400.0 / sr).exp();
            p.iter()
                .enumerate()
                .map(|(i, &v)| {
                    lp += a * (v as f64 - lp);
                    let t = i as f64 / sr;
                    let hum = 0.5 * (2.0 * std::f64::consts::PI * 100.0 * t).sin()
                        + 0.25 * (2.0 * std::f64::consts::PI * 200.0 * t).sin()
                        + 0.12 * (2.0 * std::f64::consts::PI * 300.0 * t).sin();
                    (lp * 3.0 + hum * 0.3) as f32
                })
                .collect()
        }
        NoiseKind::Babble => {
            let mut buf = vec![0.0f32; n];
            for k in 0..6 {
                let mut r = rng.fork(100 + k);
                let t = TalkerSpec {
                    level_dbov: -26.0,
                    f0_lo: if k % 2 == 0 { 95.0 } else { 170.0 },
                    f0_hi: if k % 2 == 0 { 140.0 } else { 240.0 },
                    tonal: false,
                };
                let mut one = vec![0.0f32; n];
                let mut ivs = Vec::new();
                let off = r.below((0.5 * sr) as usize);
                monologue(&mut r, &t, &mut one, off, n, (0.1, 0.5), &mut ivs);
                // 远处说话人：一阶低通，少一点高频
                let a = 1.0 - (-2.0 * std::f64::consts::PI * 2500.0 / sr).exp();
                let mut lp = 0.0f64;
                let g = r.range(0.6, 1.0);
                for (b, &v) in buf.iter_mut().zip(&one) {
                    lp += a * (v as f64 - lp);
                    *b += (lp * g) as f32;
                }
            }
            buf
        }
        NoiseKind::Cafe => {
            let mut buf = pink(rng, n);
            let rms =
                (buf.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / n.max(1) as f64).sqrt();
            let mut t = rng.range(0.1, 0.6);
            while ((t * sr) as usize) < n {
                let at = (t * sr) as usize;
                let f = rng.range(2000.0, 6000.0);
                let tau = rng.range(0.03, 0.08);
                let amp = rms * rng.range(3.0, 6.0);
                let len = ((tau * 6.0) * sr) as usize;
                for (i, b) in buf.iter_mut().skip(at).take(len).enumerate() {
                    let tt = i as f64 / sr;
                    *b += (amp * (-tt / tau).exp() * (2.0 * std::f64::consts::PI * f * tt).sin())
                        as f32;
                }
                // 泊松到达，平均每秒 1.5 次
                t += -(rng.uniform().max(1e-9)).ln() / 1.5;
            }
            buf
        }
        NoiseKind::Music => {
            let mut buf = vec![0.0f32; n];
            let roots = [220.0, 246.94, 261.63, 293.66, 329.63, 349.23, 392.0];
            let mut pos = 0usize;
            while pos < n {
                let len = (rng.range(0.5, 1.0) * sr) as usize;
                let root = roots[rng.below(roots.len())];
                let chord = [root, root * 1.26, root * 1.5];
                for (i, b) in buf.iter_mut().skip(pos).take(len).enumerate() {
                    let tt = i as f64 / sr;
                    let e = (-tt * 2.5).exp() * 0.8 + 0.2;
                    let mut v = 0.0;
                    for f in chord {
                        for h in 1..=4 {
                            v += (2.0 * std::f64::consts::PI * f * h as f64 * tt).sin()
                                / (h * h) as f64;
                        }
                    }
                    *b += (v * e) as f32;
                }
                pos += len;
            }
            buf
        }
    }
}

fn rms_db(x: &[f32]) -> f64 {
    let p = x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len().max(1) as f64;
    crate::db10(p)
}

fn scale(x: &mut [f32], db: f64) {
    let g = 10f64.powf(db / 20.0) as f32;
    x.iter_mut().for_each(|v| *v *= g);
}

/// 合成一个场景。同一个 `spec`（含种子）永远得到逐样本相同的结果。
pub fn render(spec: &SceneSpec) -> Clip {
    let sr = SR as f64;
    let n = (spec.dur_s * sr) as usize;
    let tail = (spec.tail_silence_s * sr) as usize;
    let speech_end = n.saturating_sub(tail);
    let root = Rng::new(spec.seed);
    let mut truth = Truth::default();

    let mut clean = vec![0.0f32; n];
    let mut owner_utts = Vec::new();
    let owner_from = (spec.owner_start_s * sr) as usize;
    if let Some(t) = &spec.owner {
        let mut r = root.fork(1);
        let pause = if spec
            .interferers
            .iter()
            .any(|i| matches!(i.mode, InterfMode::Alternate | InterfMode::OtherFirst))
        {
            // 轮流说话：主讲人的停顿要够长，留给干扰者
            (spec.pause_s.0.max(1.6), spec.pause_s.1.max(2.8))
        } else {
            spec.pause_s
        };
        owner_utts = monologue(
            &mut r,
            t,
            &mut clean,
            owner_from,
            speech_end,
            pause,
            &mut truth.owner,
        );
        let lvl = active_level_db(&clean, SR);
        if lvl.is_finite() {
            scale(&mut clean, t.level_dbov - lvl);
        }
    }
    let owner_level = spec.owner.as_ref().map(|t| t.level_dbov);

    let mut interf = vec![0.0f32; n];
    for (k, is) in spec.interferers.iter().enumerate() {
        let mut r = root.fork(10 + k as u64);
        let t = TalkerSpec {
            level_dbov: owner_level.unwrap_or(-26.0) - is.sir_db,
            f0_lo: is.f0_lo,
            f0_hi: is.f0_hi,
            tonal: false,
        };
        let mut one = vec![0.0f32; n];
        let guard = (0.25 * sr) as usize;
        match is.mode {
            InterfMode::Overlap => {
                monologue(
                    &mut r,
                    &t,
                    &mut one,
                    0,
                    speech_end,
                    (0.3, 1.0),
                    &mut truth.interf,
                );
            }
            InterfMode::Alternate | InterfMode::OtherFirst => {
                let mut gaps = Vec::new();
                if is.mode == InterfMode::OtherFirst && owner_from > 2 * guard {
                    gaps.push((0, owner_from - guard));
                }
                for w in owner_utts.windows(2) {
                    gaps.push((w[0].1 + guard, w[1].0.saturating_sub(guard)));
                }
                if let Some(last) = owner_utts.last() {
                    gaps.push((last.1 + guard, speech_end));
                }
                for (a, b) in gaps {
                    if b > a + (0.6 * sr) as usize {
                        render_utterance(&mut r, &t, &mut one, a, b, &mut truth.interf);
                    }
                }
            }
        }
        let lvl = active_level_db(&one, SR);
        if lvl.is_finite() {
            scale(&mut one, t.level_dbov - lvl);
        }
        for (a, b) in interf.iter_mut().zip(&one) {
            *a += b;
        }
    }
    truth.interf.sort_unstable();

    let mut r = root.fork(2);
    let mut noise = noise_track(spec.noise, &mut r, n);
    if spec.noise != NoiseKind::None {
        let target = match owner_level {
            Some(l) => l - spec.snr_db,
            None => spec.snr_db,
        };
        let cur = rms_db(&noise[..speech_end.max(1).min(n)]);
        scale(&mut noise, target - cur);
    }
    for v in noise.iter_mut().skip(speech_end) {
        *v = 0.0;
    }
    let mix: Vec<f32> = (0..n).map(|i| clean[i] + interf[i] + noise[i]).collect();
    Clip {
        sr: SR,
        clean,
        interf,
        noise,
        mix,
        truth,
        tracks: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> SceneSpec {
        SceneSpec {
            dur_s: 6.0,
            seed: 42,
            owner: Some(TalkerSpec {
                level_dbov: -26.0,
                f0_lo: 100.0,
                f0_hi: 140.0,
                tonal: false,
            }),
            owner_start_s: 0.5,
            pause_s: (0.4, 1.0),
            interferers: vec![InterfSpec {
                mode: InterfMode::Alternate,
                sir_db: 6.0,
                f0_lo: 180.0,
                f0_hi: 230.0,
            }],
            noise: NoiseKind::Pink,
            snr_db: 10.0,
            tail_silence_s: 0.5,
        }
    }

    #[test]
    fn deterministic() {
        let a = render(&spec());
        let b = render(&spec());
        assert_eq!(a.mix, b.mix);
        assert_eq!(a.truth.owner, b.truth.owner);
    }

    #[test]
    fn levels_and_tracks_add_up() {
        let c = render(&spec());
        let lvl = active_level_db(&c.clean, SR);
        assert!((lvl + 26.0).abs() < 0.5, "owner level {lvl}");
        for i in (0..c.len()).step_by(997) {
            let s = c.clean[i] + c.interf[i] + c.noise[i];
            assert!((s - c.mix[i]).abs() < 1e-6);
        }
        assert!(!c.truth.owner.is_empty());
        assert!(!c.truth.interf.is_empty());
        // 结尾数字静音
        assert!(c.mix[c.len() - 100..].iter().all(|v| *v == 0.0));
    }

    #[test]
    fn alternate_interferer_does_not_overlap_owner() {
        let c = render(&spec());
        for &(a, b) in &c.truth.interf {
            for &(oa, ob) in &c.truth.owner {
                assert!(
                    b <= oa || a >= ob,
                    "干扰者 {a}..{b} 与主讲人 {oa}..{ob} 重叠"
                );
            }
        }
    }
}
