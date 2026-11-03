//! 逐帧调度 + 三件保命（设计稿 §16「最先做的三件事」、006 decision 第 3 件）：
//!
//! 1. **出事送原声**：模型报错、输出含 NaN/Inf、输出幅度异常（状态坏了）、超过截止时间 →
//!    这一 hop 输出按模型延迟对齐的原声；状态类故障还会复位模型并多送一会儿原声，等它稳下来再交叉淡化回去。
//!    切到原声那一刻没法和坏掉的输出做交叉淡化（它不能用），改用「接缝偏移衰减」：
//!    把上一样本的输出与原声之差在几毫秒内衰减到零，波形连续、不爆音。切回模型用线性交叉淡化。
//! 2. **衰减下限**：`out = (1 - m)·降噪 + m·原声`，停顿时 `m = 10^(floor_db/20)`，背景最多降 `-floor_db` dB
//! 3. **快开慢关**：有声时 `m` 立刻升到 `10^(open_db/20)`，人声停了先保持 `hangover`，再按 `release` 慢慢回到下限
//!
//! 输入里的 NaN / ±Inf 在入口按 0 处理并计数（[`sanitize_input`]，issue #53），不算模型故障。
//!
//! `process` 不分配、不加锁、不做 I/O（`tests/no_alloc.rs` 证明）。

use std::time::{Duration, Instant};

use vdn_dsp::{DelayLine, OpenCloseEnvelope, OpenCloseParams, db_to_lin};
use vdn_types::{Basis, BypassReason, LatencyItem, LatencyLedger, Model, ParamSet, Status};

/// 输入里的非有限样本（NaN / ±Inf）就地换成 0，返回换了几个（issue #53）。
///
/// 唯一正本：`Engine`（16 kHz 入口）与 `Pipeline`（48 kHz 入口）都只调它，不各写一份。
/// 换成 0 而不是送原声：坏的就是原声本身；0 让模型、原声延迟线、接缝偏移都只见有限值，坏样本过去后自然恢复。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
pub fn sanitize_input(buf: &mut [f32]) -> u64 {
    let mut n = 0;
    for v in buf.iter_mut() {
        if !v.is_finite() {
            *v = 0.0;
            n += 1;
        }
    }
    n
}

/// 三件保命的开关与参数。默认值是 P1 的初值，待 AB 听测与吃字率标定（006 decision 第 3 件）。
#[derive(Clone, Debug)]
pub struct SafetyConfig {
    /// 出事送原声
    pub fallback: bool,
    /// 模型一次 `process` 的截止时间；离线时用它模拟「来不及」
    pub deadline: Option<Duration>,
    /// 输出幅度超过它就当状态异常
    pub max_abs: f32,
    /// 状态类故障复位后，继续送原声多久（秒）
    pub recover_hold_s: f32,
    /// 切回模型的交叉淡化时长、切到原声的接缝衰减时长（秒）
    pub crossfade_s: f32,
    /// 衰减下限
    pub floor: bool,
    pub floor_db: f32,
    /// 快开慢关（建在衰减下限之上：`floor` 关时它也不生效）
    pub fast_open: bool,
    pub open_db: f32,
    pub attack_s: f32,
    pub release_s: f32,
    pub hangover_s: f32,
    pub detect_db: f32,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            fallback: true,
            deadline: None,
            max_abs: 4.0,
            recover_hold_s: 0.1,
            crossfade_s: 0.005,
            floor: true,
            floor_db: -12.0,
            fast_open: true,
            open_db: -6.0,
            attack_s: 0.002,
            release_s: 0.3,
            hangover_s: 0.15,
            detect_db: 9.0,
        }
    }
}

impl SafetyConfig {
    /// 三件全关：输出 = 模型原样输出（对拍用）。
    pub fn off() -> Self {
        Self {
            fallback: false,
            floor: false,
            fast_open: false,
            ..Self::default()
        }
    }
}

impl SafetyConfig {
    /// 把策略给的参数组套进来（判断回路 → 新建 / 重配 Engine 时用；构造期，非实时）。
    /// `strength`、`model`、`tone` 不归 Engine 管：strength 由 Pipeline 混合时用，模型切换与修音链在外层（规约 §3.5）。
    pub fn apply_params(&mut self, p: &ParamSet) {
        self.floor_db = p.floor_db;
        self.open_db = p.open_db;
        self.detect_db = p.detect_db;
        self.release_s = p.release_s;
        self.hangover_s = p.hangover_s;
    }
}

/// 计数。全是整数，`process` 里只做加法。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EngineStats {
    pub hops: u64,
    pub fault_backend: u64,
    pub fault_nonfinite: u64,
    pub fault_abnormal: u64,
    pub fault_deadline: u64,
    pub model_resets: u64,
    /// 输出完全是原声的 hop 数
    pub bypass_hops: u64,
    /// 调用方给的长度不等于 hop、原样送原声的次数
    pub bad_len: u64,
    /// 判为有声（含 hangover）的 hop 数
    pub voiced_hops: u64,
    /// 输入里非有限（NaN / Inf）、按 0 处理的样本数（[`sanitize_input`]）
    pub bad_input: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    Backend,
    NonFinite,
    Abnormal,
    Deadline,
}

impl Fault {
    fn reason(self) -> BypassReason {
        match self {
            Fault::Backend => BypassReason::Backend,
            Fault::NonFinite => BypassReason::NonFinite,
            Fault::Abnormal => BypassReason::Abnormal,
            Fault::Deadline => BypassReason::Deadline,
        }
    }
}

pub struct Engine<M: Model> {
    model: M,
    cfg: SafetyConfig,
    hop: usize,
    delay: DelayLine,
    /// 本 hop 输入的副本，非有限样本已清 0（模型与原声延迟线都只读它）
    inp: Vec<f32>,
    dry: Vec<f32>,
    wet: Vec<f32>,
    mix: Vec<f32>,
    env: Option<OpenCloseEnvelope>,
    floor_lin: f32,
    /// 原声占比 0..=1（1 = 全原声）
    bypass: f32,
    bypass_step: f32,
    hold_hops_left: u32,
    hold_hops: u32,
    seam: f32,
    seam_decay: f32,
    last_out: f32,
    last_dry: f32,
    last_model_time: Duration,
    stats: EngineStats,
}

// 构造：非实时线程（会分配）
impl<M: Model> Engine<M> {
    pub fn new(model: M, cfg: SafetyConfig) -> Self {
        let hop = model.hop();
        let sr = model.sample_rate() as f32;
        assert!(hop > 0 && sr > 0.0, "模型的 hop 与采样率必须为正");
        debug_assert!(cfg.crossfade_s >= 0.0 && cfg.recover_hold_s >= 0.0 && cfg.max_abs > 0.0);
        let floor_lin = if cfg.floor {
            db_to_lin(cfg.floor_db)
        } else {
            0.0
        };
        let env = (cfg.fast_open && cfg.floor).then(|| {
            OpenCloseEnvelope::new(OpenCloseParams {
                sr,
                closed: floor_lin,
                open: db_to_lin(cfg.open_db).max(floor_lin),
                attack_s: cfg.attack_s,
                release_s: cfg.release_s,
                hangover_s: cfg.hangover_s,
                detect_db: cfg.detect_db,
                abs_min_db: -70.0,
                noise_rise_db_per_s: 6.0,
            })
        });
        let xf = (cfg.crossfade_s * sr).max(1.0);
        let hold_hops = (cfg.recover_hold_s * sr / hop as f32).ceil() as u32;
        Self {
            delay: DelayLine::new(model.latency_samples()),
            inp: vec![0.0; hop],
            dry: vec![0.0; hop],
            wet: vec![0.0; hop],
            mix: vec![floor_lin; hop],
            env,
            floor_lin,
            bypass: 0.0,
            bypass_step: 1.0 / xf,
            hold_hops_left: 0,
            hold_hops,
            seam: 0.0,
            // 接缝偏移在 crossfade_s 内衰减到约 1%
            seam_decay: (0.01f32.ln() / xf).exp(),
            last_out: 0.0,
            last_dry: 0.0,
            last_model_time: Duration::ZERO,
            stats: EngineStats::default(),
            hop,
            cfg,
            model,
        }
    }

    /// 延迟账（D32 · 规约 R13）：攒满一个 hop 的等待 + 模型算法延迟；保命（送原声、衰减下限、快开慢关）不加延迟。
    /// 构造期调用（会分配）。平台部分（重采样、IO、环、HAL）由调用方追加，见 `vdn_types::latency::planned_platform`。
    pub fn latency_ledger(&self) -> LatencyLedger {
        let sr = self.model.sample_rate();
        let mut l = LatencyLedger::new();
        l.push(LatencyItem::new(
            "hop-buffer",
            self.hop as u32,
            sr,
            Basis::Declared,
            "攒满一个模型 hop 才能处理",
        ))
        .push(LatencyItem::new(
            self.model.name(),
            self.model.latency_samples() as u32,
            sr,
            Basis::Declared,
            "模型算法延迟（Model::latency_samples）",
        ));
        l
    }
}

// 实时：每 hop 调用。不分配、不加锁、不 panic（specs/rust/architecture.md §5.3、R1–R4）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<M: Model> Engine<M> {
    pub fn hop(&self) -> usize {
        self.hop
    }

    pub fn sample_rate(&self) -> u32 {
        self.model.sample_rate()
    }

    /// 整条链的输出延迟（= 模型延迟；保命不加延迟）。
    pub fn latency_samples(&self) -> usize {
        self.model.latency_samples()
    }

    pub fn stats(&self) -> EngineStats {
        self.stats
    }

    /// 上一次 `process` 里模型本身花的时间。
    pub fn last_model_time(&self) -> Duration {
        self.last_model_time
    }

    pub fn model(&self) -> &M {
        &self.model
    }

    pub fn reset(&mut self) {
        self.model.reset();
        self.delay.reset();
        if let Some(e) = self.env.as_mut() {
            e.reset();
        }
        self.bypass = 0.0;
        self.hold_hops_left = 0;
        self.seam = 0.0;
        self.last_out = 0.0;
        self.last_dry = 0.0;
    }

    /// 处理一个 hop（离线 / 16 kHz 直驱用）：故障时在 16 kHz 自己送原声并淡化。
    /// `input.len() == output.len() == hop()`；长度不符时（调用方的 bug）debug 下 panic，
    /// release 下原样送原声并计数，不在实时线程上 panic。
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let _ = self.run(input, output, true);
    }

    /// 处理一个 hop（`Pipeline` 用，规约 §3.3 规则 2）：只判故障、管保持期并回报 [`Status`]，
    /// **不在 16 kHz 做送原声的切换与淡化**——调用方用 48 kHz 原声做。
    ///
    /// - `Status::Ok`：`output` = 模型输出（含衰减下限 / 快开慢关的 16 kHz 原声混回）
    /// - `Status::Bypassed(_)` / `Status::Holding`：`output` = 按模型延迟对齐的 16 kHz 原声，只给升采样器垫底
    ///   （保证有限、波形连续），调用方应整段换成 48 kHz 原声
    /// - `fallback` 关时恒为 `Ok`（对拍用，输出 = 模型原样输出）
    ///
    /// 推理调用（`Model::process`）在这里面：调用方不要用 FTZ 守卫包住它（R9）。
    pub fn process_status(&mut self, input: &[f32], output: &mut [f32]) -> Status {
        self.run(input, output, false)
    }

    /// 上一个 hop 按模型延迟对齐的 16 kHz 原声（`process` / `process_status` 之后有效）。给后处理级当 `dry`。
    pub fn aligned_dry(&self) -> &[f32] {
        &self.dry
    }

    pub fn config(&self) -> &SafetyConfig {
        &self.cfg
    }

    /// `own_bypass`：true = 在 16 kHz 自己做送原声（`process`）；false = 只回报（`process_status`）。
    fn run(&mut self, input: &[f32], output: &mut [f32], own_bypass: bool) -> Status {
        debug_assert_eq!(input.len(), self.hop);
        debug_assert_eq!(output.len(), self.hop);
        self.stats.hops += 1;
        if input.len() != self.hop || output.len() != self.hop {
            self.stats.bad_len += 1;
            let mut src = input.iter();
            for o in output.iter_mut() {
                *o = src.next().copied().unwrap_or(0.0);
            }
            self.stats.bad_input += sanitize_input(output);
            return Status::Bypassed(BypassReason::BadLen);
        }
        self.inp.copy_from_slice(input);
        self.stats.bad_input += sanitize_input(&mut self.inp);
        self.delay.process(&self.inp, &mut self.dry);

        let t0 = Instant::now();
        let res = {
            // 推理一律在 FTZ 关着时跑（R9）：调用方线程自己开着 FTZ 也不跟着开（DeepFilterNet #713）
            let _no_ftz = vdn_dsp::FtzGuard::off();
            self.model.process(&self.inp, &mut self.wet)
        };
        self.last_model_time = t0.elapsed();

        let fault = self.classify(res.is_err());
        if !self.cfg.fallback {
            if res.is_err() {
                self.wet.fill(0.0);
            }
            self.mix_out(output);
            return Status::Ok;
        }

        if let Some(f) = fault {
            match f {
                Fault::Backend => self.stats.fault_backend += 1,
                Fault::NonFinite => self.stats.fault_nonfinite += 1,
                Fault::Abnormal => self.stats.fault_abnormal += 1,
                Fault::Deadline => self.stats.fault_deadline += 1,
            }
            if f != Fault::Deadline {
                // 状态可能已经坏了：复位，多送一会儿原声
                self.model.reset();
                self.stats.model_resets += 1;
                self.hold_hops_left = self.hold_hops;
            }
        }
        let want_bypass = fault.is_some() || self.hold_hops_left > 0;
        if fault.is_none() && self.hold_hops_left > 0 {
            self.hold_hops_left -= 1;
        }
        let status = match fault {
            Some(f) => Status::Bypassed(f.reason()),
            None if want_bypass => Status::Holding,
            None => Status::Ok,
        };

        if !own_bypass {
            if want_bypass {
                self.stats.bypass_hops += 1;
                output.copy_from_slice(&self.dry);
            } else {
                self.mix_out(output);
            }
            return status;
        }

        if want_bypass {
            if self.bypass < 1.0 {
                // 切到原声：接缝偏移，从上一个输出样本平滑过渡
                self.seam = self.last_out - self.last_dry;
                if !self.seam.is_finite() {
                    // 输入里有 NaN / Inf 时别让接缝锁死成 NaN
                    self.seam = 0.0;
                }
                self.bypass = 1.0;
            }
            self.stats.bypass_hops += 1;
            for (o, d) in output.iter_mut().zip(&self.dry) {
                *o = d + self.seam;
                self.seam *= self.seam_decay;
            }
        } else {
            self.mix_out(output);
            if self.bypass > 0.0 || self.seam != 0.0 {
                for (o, d) in output.iter_mut().zip(&self.dry) {
                    // 切回模型：线性交叉淡化
                    *o += self.bypass * (d - *o) + self.seam;
                    self.bypass = (self.bypass - self.bypass_step).max(0.0);
                    self.seam *= self.seam_decay;
                }
                if self.seam.abs() < 1e-9 {
                    self.seam = 0.0;
                }
            }
        }
        // 接缝只收有限值：dry 已有限；output 只在保命全关、模型出坏值时非有限，那时按「无接缝」记
        self.last_dry = self.dry.last().copied().unwrap_or(0.0);
        self.last_out = match output.last() {
            Some(&v) if v.is_finite() => v,
            _ => self.last_dry,
        };
        if status == Status::Ok && self.bypass > 0.0 {
            Status::Crossfading
        } else {
            status
        }
    }

    fn classify(&self, backend_err: bool) -> Option<Fault> {
        if backend_err {
            return Some(Fault::Backend);
        }
        let mut peak = 0.0f32;
        for v in &self.wet {
            if !v.is_finite() {
                return Some(Fault::NonFinite);
            }
            peak = peak.max(v.abs());
        }
        if peak > self.cfg.max_abs {
            return Some(Fault::Abnormal);
        }
        match self.cfg.deadline {
            Some(d) if self.last_model_time > d => Some(Fault::Deadline),
            _ => None,
        }
    }

    /// 降噪输出 + 衰减下限 / 快开慢关的原声混回。
    fn mix_out(&mut self, output: &mut [f32]) {
        match self.env.as_mut() {
            Some(env) => {
                env.process(&self.wet, &mut self.mix);
                if env.voiced() {
                    self.stats.voiced_hops += 1;
                }
                for ((o, (w, d)), m) in output
                    .iter_mut()
                    .zip(self.wet.iter().zip(&self.dry))
                    .zip(&self.mix)
                {
                    *o = w + m * (d - w);
                }
            }
            None if self.floor_lin == 0.0 => output.copy_from_slice(&self.wet),
            None => {
                let m = self.floor_lin;
                for (o, (w, d)) in output.iter_mut().zip(self.wet.iter().zip(&self.dry)) {
                    *o = w + m * (d - w);
                }
            }
        }
    }
}
