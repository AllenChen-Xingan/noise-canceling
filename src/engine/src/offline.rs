//! 离线驱动：把一整段波形逐 hop 喂给 [`Engine`]，和实时线程走同一个 `process`（`.42cog/cog.md`「eval 和实时用同一份处理代码」）。
//!
//! 对齐口径同 exp003：开头先喂 `lead_in` 个零（GTCRN 对拍时取一个 hop，让第一次推理也是全零帧，和 Python 一致），
//! 结尾补零把延迟冲出来，再丢掉前 `lead_in + latency` 个样本，输出与输入等长、逐样本对齐。

use std::time::Duration;

use crate::engine::Engine;
use crate::model::Model;

pub struct OfflineRun {
    /// 与输入等长、已对齐的输出
    pub output: Vec<f32>,
    /// 每个 hop 里模型本身的耗时（含 lead-in 与冲尾的 hop）
    pub model_times: Vec<Duration>,
    /// 每个 hop 整个 `Engine::process` 的耗时
    pub engine_times: Vec<Duration>,
}

pub fn run<M: Model>(engine: &mut Engine<M>, x: &[f32], lead_in: usize) -> OfflineRun {
    let hop = engine.hop();
    let lat = engine.latency_samples();
    let total = lead_in + x.len() + lat;
    let n_hops = total.div_ceil(hop);
    let mut padded = vec![0.0f32; n_hops * hop];
    padded[lead_in..lead_in + x.len()].copy_from_slice(x);
    let mut y = vec![0.0f32; n_hops * hop];
    let mut model_times = Vec::with_capacity(n_hops);
    let mut engine_times = Vec::with_capacity(n_hops);
    for (xi, yi) in padded.chunks_exact(hop).zip(y.chunks_exact_mut(hop)) {
        let t0 = std::time::Instant::now();
        engine.process(xi, yi);
        engine_times.push(t0.elapsed());
        model_times.push(engine.last_model_time());
    }
    let start = lead_in + lat;
    OfflineRun {
        output: y[start..start + x.len()].to_vec(),
        model_times,
        engine_times,
    }
}
