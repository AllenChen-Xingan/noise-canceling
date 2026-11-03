//! 被测系统：真 `vdn_engine::Engine`（三件保命）包一个模型，逐 hop 驱动，记下每 hop 的门控状态与耗时。
//!
//! 模型有四种：
//! - `passthrough`：输出 = 输入（延迟 0）。量「什么都不做」
//! - `ref`：[`RefWiener`]，本 crate 的参考降噪器（判决引导维纳 + 最小值跟踪噪声估计，`vdn-dsp` 的 STFT 512/256，
//!   与 GTCRN 同 hop、同延迟）。**它不是产品模型**，是 CI 没有权重时的替身，让 Engine 的保命逻辑在真信号上被检验
//! - `gtcrn` / `fastenhancer-m`：真模型（`vdn-models`），找得到权重才跑
//!
//! 驱动和 `vdn_engine::offline::run` 同一口径（开头不补零、结尾补零把延迟冲出来、丢掉前 latency 个样本），
//! 另外记下**原始未对齐输出**（V-LAT 用它量延迟）和每 hop 的 `voiced`（看 `EngineStats::voiced_hops` 的增量）。

use std::time::{Duration, Instant};

use vdn_dsp::StreamingStft;
use vdn_engine::testing::DelayGain;
use vdn_engine::{Engine, EngineStats, FaultInjector, FaultKind, Model, ModelError, SafetyConfig};
use vdn_models::{AnyModel, ModelId, ModelKind};
use vdn_types::latency::{IO_BLOCK_PROPOSED, LATENCY_BUDGET_MS, planned_platform};

/// 参考降噪器（CI 替身）。
pub struct RefWiener {
    stft: StreamingStft,
    noise: Vec<f32>,
    smooth: Vec<f32>,
    prev_clean: Vec<f32>,
    frames: u32,
}

impl RefWiener {
    pub const N_FFT: usize = 512;
    pub const HOP: usize = 256;
    /// 增益下限（−25 dB）
    const GMIN: f32 = 0.056;
    /// 判决引导系数
    const ALPHA: f32 = 0.98;
    /// 噪声估计每 hop 最多上涨的倍数（约 +3 dB/s）
    const RISE: f32 = 1.011;
    /// 开头多少帧用平均值初始化噪声估计
    const INIT_FRAMES: u32 = 8;

    pub fn new() -> Self {
        let bins = Self::N_FFT / 2 + 1;
        RefWiener {
            stft: StreamingStft::new(
                Self::N_FFT,
                Self::HOP,
                vdn_dsp::window::sqrt_hann_periodic(Self::N_FFT),
            ),
            noise: vec![0.0; bins],
            smooth: vec![0.0; bins],
            prev_clean: vec![0.0; bins],
            frames: 0,
        }
    }
}

impl Default for RefWiener {
    fn default() -> Self {
        Self::new()
    }
}

impl Model for RefWiener {
    fn name(&self) -> &'static str {
        "ref_wiener"
    }
    fn sample_rate(&self) -> u32 {
        16_000
    }
    fn hop(&self) -> usize {
        Self::HOP
    }
    fn latency_samples(&self) -> usize {
        Self::N_FFT - Self::HOP
    }
    fn reset(&mut self) {
        self.stft.reset();
        self.noise.fill(0.0);
        self.smooth.fill(0.0);
        self.prev_clean.fill(0.0);
        self.frames = 0;
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        if input.len() != Self::HOP || output.len() != Self::HOP {
            return Err(ModelError::Shape);
        }
        let k = self.frames;
        self.frames = self.frames.saturating_add(1);
        let spec = self.stft.analyze(input);
        for (((z, n), s), pc) in spec
            .iter_mut()
            .zip(self.noise.iter_mut())
            .zip(self.smooth.iter_mut())
            .zip(self.prev_clean.iter_mut())
        {
            let p = z.norm_sqr();
            *s = 0.7 * *s + 0.3 * p;
            if k < Self::INIT_FRAMES {
                *n += p / Self::INIT_FRAMES as f32;
            } else if *s < *n {
                *n = *s;
            } else {
                *n *= Self::RISE;
            }
            let nn = n.max(1e-12);
            let post = (p / nn - 1.0).max(0.0);
            let xi = Self::ALPHA * *pc / nn + (1.0 - Self::ALPHA) * post;
            let g = (xi / (1.0 + xi)).max(Self::GMIN);
            *z *= g;
            *pc = g * g * p;
        }
        self.stft.synthesize(output);
        Ok(())
    }
}

/// 本 crate 能驱动的模型（静态分发）。
pub enum EvalModel {
    Passthrough(DelayGain),
    Ref(Box<RefWiener>),
    Real(Box<AnyModel>),
}

impl Model for EvalModel {
    fn name(&self) -> &'static str {
        match self {
            EvalModel::Passthrough(m) => m.name(),
            EvalModel::Ref(m) => m.name(),
            EvalModel::Real(m) => m.name(),
        }
    }
    fn sample_rate(&self) -> u32 {
        match self {
            EvalModel::Passthrough(m) => m.sample_rate(),
            EvalModel::Ref(m) => m.sample_rate(),
            EvalModel::Real(m) => m.sample_rate(),
        }
    }
    fn hop(&self) -> usize {
        match self {
            EvalModel::Passthrough(m) => m.hop(),
            EvalModel::Ref(m) => m.hop(),
            EvalModel::Real(m) => m.hop(),
        }
    }
    fn latency_samples(&self) -> usize {
        match self {
            EvalModel::Passthrough(m) => m.latency_samples(),
            EvalModel::Ref(m) => m.latency_samples(),
            EvalModel::Real(m) => m.latency_samples(),
        }
    }
    fn reset(&mut self) {
        match self {
            EvalModel::Passthrough(m) => m.reset(),
            EvalModel::Ref(m) => m.reset(),
            EvalModel::Real(m) => m.reset(),
        }
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        match self {
            EvalModel::Passthrough(m) => m.process(input, output),
            EvalModel::Ref(m) => m.process(input, output),
            EvalModel::Real(m) => m.process(input, output),
        }
    }
}

/// 系统名（任务清单与报告里用）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemKind {
    Passthrough,
    Ref,
    Gtcrn,
    FastEnhancerM,
}

impl SystemKind {
    pub fn parse(s: &str) -> Option<SystemKind> {
        Some(match s {
            "passthrough" => SystemKind::Passthrough,
            "ref" => SystemKind::Ref,
            _ => match ModelId::parse(s)? {
                ModelId::Gtcrn => SystemKind::Gtcrn,
                ModelId::FastEnhancerM => SystemKind::FastEnhancerM,
            },
        })
    }
    /// 真模型的名字取自正本 `ModelId::as_str`（结果文件、门禁基线的键都用它）。
    pub fn as_str(self) -> &'static str {
        match self {
            SystemKind::Passthrough => "passthrough",
            SystemKind::Ref => "ref",
            SystemKind::Gtcrn => ModelId::Gtcrn.as_str(),
            SystemKind::FastEnhancerM => ModelId::FastEnhancerM.as_str(),
        }
    }
    fn model_kind(self) -> Option<ModelKind> {
        match self {
            SystemKind::Gtcrn => Some(ModelKind::Gtcrn),
            SystemKind::FastEnhancerM => Some(ModelKind::FastEnhancerM),
            _ => None,
        }
    }
    /// 真模型的权重在不在本机（不在就跳过，不算失败）。
    pub fn available(self) -> bool {
        self.model_kind().is_none_or(|k| k.default_path().is_some())
    }
}

/// 一次运行的设置。
#[derive(Clone, Debug)]
pub struct SystemSpec {
    pub kind: SystemKind,
    /// false = 三件保命全关（`SafetyConfig::off`）
    pub safety: bool,
    pub faults: Vec<(u64, FaultKind)>,
}

/// 一次运行的结果。
#[derive(Clone, Debug)]
pub struct RunOut {
    /// 与输入等长、逐样本对齐
    pub out: Vec<f32>,
    /// 原始输出（未丢前 latency 个样本），长度 = hop 整数倍
    pub raw: Vec<f32>,
    pub hop: usize,
    pub latency: usize,
    /// 每 hop 门控是否判有声（engine 的快开慢关包络；按原始输出时间轴）
    pub voiced: Vec<bool>,
    /// 每 hop 输出是否全是原声（送原声）
    pub bypass: Vec<bool>,
    pub engine_times: Vec<Duration>,
    pub stats: EngineStats,
    /// 引擎延迟账 + 平台预算（IO 块取提议值 96 帧），ms
    pub ledger_total_ms: f64,
    pub ledger_text: String,
    pub model_name: &'static str,
}

impl RunOut {
    /// 对齐后第 `t` 个样本时门控是否开着。
    pub fn gate_at(&self, t: usize) -> bool {
        self.voiced
            .get((t + self.latency) / self.hop)
            .copied()
            .unwrap_or(false)
    }
}

fn build(spec: &SystemSpec) -> Result<Engine<FaultInjector<EvalModel>>, String> {
    let model = match spec.kind {
        SystemKind::Passthrough => EvalModel::Passthrough(DelayGain::new(256, 0, 1.0)),
        SystemKind::Ref => EvalModel::Ref(Box::default()),
        k => {
            let mk = k.model_kind().ok_or("内部错误：未知模型")?;
            let path = mk
                .default_path()
                .ok_or_else(|| format!("{} 的权重不在本机", k.as_str()))?;
            EvalModel::Real(Box::new(
                mk.open(&path)
                    .map_err(|e| format!("打开 {}：{e}", path.display()))?,
            ))
        }
    };
    let cfg = if spec.safety {
        SafetyConfig::default()
    } else {
        SafetyConfig::off()
    };
    Ok(Engine::new(
        FaultInjector::new(model, spec.faults.clone()),
        cfg,
    ))
}

/// 跑一段。
pub fn run(spec: &SystemSpec, x: &[f32]) -> Result<RunOut, String> {
    let mut eng = build(spec)?;
    let hop = eng.hop();
    let lat = eng.latency_samples();
    let mut ledger = eng.latency_ledger();
    ledger.extend(planned_platform(IO_BLOCK_PROPOSED));
    let n_hops = (x.len() + lat).div_ceil(hop);
    let mut padded = vec![0.0f32; n_hops * hop];
    padded[..x.len()].copy_from_slice(x);
    let mut raw = vec![0.0f32; n_hops * hop];
    let mut voiced = Vec::with_capacity(n_hops);
    let mut bypass = Vec::with_capacity(n_hops);
    let mut times = Vec::with_capacity(n_hops);
    let mut prev = eng.stats();
    for (xi, yi) in padded.chunks_exact(hop).zip(raw.chunks_exact_mut(hop)) {
        let t0 = Instant::now();
        eng.process(xi, yi);
        times.push(t0.elapsed());
        let s = eng.stats();
        voiced.push(s.voiced_hops > prev.voiced_hops);
        bypass.push(s.bypass_hops > prev.bypass_hops);
        prev = s;
    }
    let out = raw[lat..lat + x.len()].to_vec();
    Ok(RunOut {
        out,
        raw,
        hop,
        latency: lat,
        voiced,
        bypass,
        engine_times: times,
        stats: eng.stats(),
        ledger_total_ms: ledger.total_ms(),
        ledger_text: ledger.render(),
        model_name: eng.model().name(),
    })
}

/// D32 的预算（ms），给报告用。
pub const BUDGET_MS: f64 = LATENCY_BUDGET_MS;

#[cfg(test)]
mod tests {
    use super::*;

    /// 真模型系统的名字 = 它的模型的正本名；两种 FE-M 写法解析到同一个系统；名字往返不丢。
    #[test]
    fn system_names_follow_model_id() {
        let all = [
            SystemKind::Passthrough,
            SystemKind::Ref,
            SystemKind::Gtcrn,
            SystemKind::FastEnhancerM,
        ];
        for k in all {
            assert_eq!(SystemKind::parse(k.as_str()), Some(k));
            if let Some(m) = k.model_kind() {
                assert_eq!(k.as_str(), m.id().as_str());
            }
        }
        assert_eq!(
            SystemKind::parse("fastenhancer-m"),
            Some(SystemKind::FastEnhancerM)
        );
        assert_eq!(
            SystemKind::parse("fastenhancer_m"),
            Some(SystemKind::FastEnhancerM)
        );
        assert_eq!(SystemKind::parse("nope"), None);
    }

    fn spec(kind: SystemKind) -> SystemSpec {
        SystemSpec {
            kind,
            safety: true,
            faults: vec![],
        }
    }

    #[test]
    fn passthrough_with_safety_off_is_identity() {
        let mut r = crate::rng::Rng::new(1);
        let x: Vec<f32> = (0..16000).map(|_| 0.1 * r.normal() as f32).collect();
        let s = SystemSpec {
            safety: false,
            ..spec(SystemKind::Passthrough)
        };
        let o = run(&s, &x).unwrap();
        assert_eq!(o.out, x);
    }

    #[test]
    fn ref_wiener_attenuates_stationary_noise() {
        let mut r = crate::rng::Rng::new(2);
        let x: Vec<f32> = (0..48000).map(|_| 0.05 * r.normal() as f32).collect();
        let s = SystemSpec {
            safety: false,
            ..spec(SystemKind::Ref)
        };
        let o = run(&s, &x).unwrap();
        let p = |v: &[f32]| v.iter().map(|a| (*a as f64).powi(2)).sum::<f64>();
        let att = 10.0 * (p(&o.out[16000..]) / p(&x[16000..])).log10();
        assert!(att < -15.0, "参考降噪器对白噪声只压了 {att:.1} dB");
    }

    #[test]
    fn ref_latency_is_one_hop() {
        assert_eq!(RefWiener::new().latency_samples(), 256);
    }
}
