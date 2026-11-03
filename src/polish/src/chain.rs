//! 整条修音链：频谱节点 + AGC，外加参数的平滑过渡。
//!
//! 用法（每个 hop）：
//! 1. 模型适配器算出带噪频谱 `dry` 与模型输出频谱 `wet` → [`PolishChain::process_spectrum`]
//! 2. 适配器 iSTFT 出时域 hop → [`PolishChain::process_time`]
//!
//! 参数：判断回路调 [`PolishChain::set_target`] 送一组新参数；链在 `ramp_s` 秒内从当前值线性走过去（D40「慢慢切」）。

use realfft::num_complex::Complex32;

use crate::agc::Agc;
use crate::params::PolishParams;
use crate::spectral::SpectralPolish;
use crate::spectral_stage::SpectralStage;
use vdn_types::{FrameCtx, NodeKind, Stage};

pub struct PolishChain {
    spectral: SpectralPolish,
    agc: Agc,
    from: PolishParams,
    to: PolishParams,
    cur: PolishParams,
    t: f32,
    step: f32,
    ramp_s: f32,
    fps: f32,
    last_vp: Option<f32>,
    /// 本 hop 频谱那一路跑过没有（只走时域的模型，如 FE-M wav2wav，没有频谱）
    spec_ran: bool,
}

impl PolishChain {
    /// `initial` 先按描述表夹进范围（[`PolishParams::clamped`]）。
    pub fn new(n_fft: usize, hop: usize, sr: f32, initial: PolishParams) -> Self {
        let fps = sr / hop as f32;
        let initial = initial.clamped();
        Self {
            spectral: SpectralPolish::new(n_fft, hop, sr),
            agc: Agc::new(hop, sr),
            from: initial,
            to: initial,
            cur: initial,
            t: 1.0,
            step: 1.0,
            ramp_s: vdn_types::desc::POLISH_RAMP_S.default,
            fps,
            last_vp: None,
            spec_ran: false,
        }
    }

    /// 参数过渡时长（秒），默认与范围见 `vdn_types::desc::POLISH_RAMP_S`；越界夹住。
    pub fn set_ramp_s(&mut self, s: f32) {
        self.ramp_s = vdn_types::desc::POLISH_RAMP_S.clamp(s);
    }

    /// 送一组新目标参数（判断回路调用，实时侧只是记下来，下一帧开始过渡）。
    /// 参数先按描述表夹进范围，实时侧不会见到越界值。
    pub fn set_target(&mut self, p: PolishParams) {
        let p = p.clamped();
        if p == self.to {
            return;
        }
        self.from = self.cur;
        self.to = p;
        self.t = 0.0;
        self.step = if self.ramp_s <= 0.0 {
            1.0
        } else {
            1.0 / (self.ramp_s * self.fps)
        };
    }

    /// 当前生效的参数（过渡中的插值）。
    pub fn current(&self) -> PolishParams {
        self.cur
    }

    pub fn spectral(&self) -> &SpectralPolish {
        &self.spectral
    }

    pub fn agc(&self) -> &Agc {
        &self.agc
    }

    fn advance(&mut self) {
        if self.t < 1.0 {
            self.t = (self.t + self.step).min(1.0);
            self.cur = PolishParams::lerp(&self.from, &self.to, self.t);
        }
    }

    pub fn process_spectrum(&mut self, wet: &mut [Complex32], dry: &[Complex32], ctx: FrameCtx) {
        self.advance();
        self.spec_ran = true;
        self.last_vp = ctx.voice_prob;
        let p = self.cur;
        self.spectral.process(dry, wet, &p, ctx.voice_prob);
    }

    /// 时域部分（频谱那一路之后调；或者没有频谱时单独调）。人声概率依次取：外部 VAD → 频谱节点的估计 → AGC 自己的能量粗判。
    /// 每个 hop 参数只推进一次：频谱那一路跑过就不再推进。
    pub fn process_time(&mut self, wet: &mut [f32]) {
        if !self.spec_ran {
            self.advance();
        }
        let vp = match (self.last_vp, self.spec_ran) {
            (Some(v), _) => v,
            (None, true) => self.spectral.last_speech_presence(),
            (None, false) => self.agc.energy_voice_prob(wet),
        };
        let p = self.cur;
        self.agc.process(wet, &p, vp);
        self.spec_ran = false;
        self.last_vp = None;
    }

    /// 只有时域、带外部上下文时用（例如 FE-M wav2wav 那一路）。
    pub fn process_time_ctx(&mut self, wet: &mut [f32], ctx: FrameCtx) {
        if ctx.voice_prob.is_some() {
            self.last_vp = ctx.voice_prob;
        }
        self.process_time(wet);
    }

    pub fn reset(&mut self) {
        self.spectral.reset();
        self.agc.reset();
        self.cur = self.to;
        self.t = 1.0;
    }
}

impl SpectralStage for PolishChain {
    fn kind(&self) -> &'static NodeKind {
        &vdn_types::graph::POLISH_SPECTRAL
    }
    fn reset(&mut self) {
        PolishChain::reset(self);
    }
    fn process_spectrum(&mut self, wet: &mut [Complex32], dry: &[Complex32], ctx: FrameCtx) {
        PolishChain::process_spectrum(self, wet, dry, ctx);
    }
}

/// 时域 Stage 只做 AGC（频谱部分要先经 `SpectralStage` 那一路）。
/// 注意：同一个实例同时实现两个 trait，engine 接入时要把它当「一个节点、两个挂点」，不能当两个独立节点各放一份；
/// 两个 trait 的 `reset` 都是整链重置。
impl Stage for PolishChain {
    fn kind(&self) -> &'static NodeKind {
        &vdn_types::graph::POLISH_AGC
    }
    fn reset(&mut self) {
        PolishChain::reset(self);
    }
    fn process(&mut self, wet: &mut [f32], _dry: &[f32], ctx: FrameCtx) {
        self.process_time_ctx(wet, ctx);
    }
}
