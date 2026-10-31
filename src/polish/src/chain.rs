//! 整条修音链：频谱节点 + AGC，外加参数的平滑过渡。
//!
//! 用法（每个 hop）：
//! 1. 模型适配器算出带噪频谱 `dry` 与模型输出频谱 `wet` → [`PolishChain::process_spectrum`]
//! 2. 适配器 iSTFT 出时域 hop → [`PolishChain::process_time`]
//!
//! 参数：判断回路调 [`PolishChain::set_target`] 送一组新参数；链在 `ramp_s` 秒内从当前值线性走过去（D40「慢慢切」）。

use realfft::num_complex::Complex32;

use crate::agc::Agc;
use crate::mirror::{FrameCtx, SpectralStage, Stage};
use crate::params::PolishParams;
use crate::spectral::SpectralPolish;

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
}

impl PolishChain {
    pub fn new(n_fft: usize, hop: usize, sr: f32, initial: PolishParams) -> Self {
        let fps = sr / hop as f32;
        Self {
            spectral: SpectralPolish::new(n_fft, hop, sr),
            agc: Agc::new(hop, sr),
            from: initial,
            to: initial,
            cur: initial,
            t: 1.0,
            step: 1.0,
            ramp_s: 1.0,
            fps,
            last_vp: None,
        }
    }

    /// 参数过渡时长（秒），默认 1 s。
    pub fn set_ramp_s(&mut self, s: f32) {
        self.ramp_s = s.max(0.0);
    }

    /// 送一组新目标参数（判断回路调用，实时侧只是记下来，下一帧开始过渡）。
    pub fn set_target(&mut self, p: PolishParams) {
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
        self.last_vp = ctx.voice_prob;
        let p = self.cur;
        self.spectral.process(dry, wet, &p, ctx.voice_prob);
    }

    /// 时域部分。没有外部 VAD 时借用频谱节点的人声存在度。
    pub fn process_time(&mut self, wet: &mut [f32]) {
        let vp = self
            .last_vp
            .unwrap_or_else(|| self.spectral.last_speech_presence());
        let p = self.cur;
        self.agc.process(wet, &p, vp);
    }

    pub fn reset(&mut self) {
        self.spectral.reset();
        self.agc.reset();
        self.cur = self.to;
        self.t = 1.0;
    }
}

impl SpectralStage for PolishChain {
    fn name(&self) -> &'static str {
        "polish-spectral"
    }
    fn reset(&mut self) {
        PolishChain::reset(self);
    }
    fn process_spectrum(&mut self, wet: &mut [Complex32], dry: &[Complex32], ctx: FrameCtx) {
        PolishChain::process_spectrum(self, wet, dry, ctx);
    }
}

/// 时域 Stage 只做 AGC（频谱部分要先经 `SpectralStage` 那一路）。
impl Stage for PolishChain {
    fn name(&self) -> &'static str {
        "polish-agc"
    }
    fn reset(&mut self) {
        PolishChain::reset(self);
    }
    fn process(&mut self, wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {
        self.process_time(wet);
    }
}
