//! FastEnhancer（官方 release `onnx-dns-v1.0.0` 的 wav2wav 版，STFT/iSTFT/OLA 都在图里）适配器。
//!
//! 口径同 exp003 `enhance.py::FastEnhancer`：每次喂一个 hop（M 档 160 样本），缓存原样回灌；
//! 输出比输入晚 `n_fft − hop = 512 − hop` 个样本（官方 `docs/docs/onnx.md`）。
//! 权重许可待核（003 技术方案 §5），发布前必须定。

use std::path::Path;

use ort::session::{HasSelectedOutputs, OutputSelector, RunOptions, Session};
use ort::value::Tensor;
use vdn_backend_ort::ort;
use vdn_types::{Model, ModelError};

use vdn_backend_ort::{self as session, BoxError};

use crate::interface::{self, Interface, check_session, dims};

const N_FFT: usize = 512;
/// 输入输出的声明（名字 · 顺序 · 形状）正本在 `interface::FASTENHANCER_M`。
const IO: Interface = interface::FASTENHANCER_M;

pub struct FastEnhancer {
    session: Session,
    run: RunOptions<HasSelectedOutputs>,
    hop: usize,
    wav_in: Tensor<f32>,
    caches: Vec<Tensor<f32>>,
}

// 构造：非实时线程
impl FastEnhancer {
    pub fn open(path: &Path) -> Result<Self, BoxError> {
        let session = session::open(path)?;
        // 名字、顺序、形状逐个对声明（输入输出按位置配对：wav_in/wav_out、cache_in_i/cache_out_i；
        // 形状相同时顺序乱了不会报错，只会悄悄算错）
        check_session(&IO, &session).map_err(|e| format!("FastEnhancer {e}"))?;
        let mut sel = OutputSelector::default();
        for o in IO.outputs {
            sel = sel.preallocate(o.name, session::zeros(&dims(o.shape))?);
        }
        let mut ins = IO.inputs.iter().map(|d| dims(d.shape));
        let wav = ins.next().ok_or("FastEnhancer 声明缺 wav_in")?;
        let hop = *wav.last().ok_or("wav_in 形状为空")?;
        let caches = ins
            .map(|s| session::zeros(&s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            run: RunOptions::new()?.with_outputs(sel),
            hop,
            wav_in: session::zeros(&wav)?,
            caches,
            session,
        })
    }
}

// 实时（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Model for FastEnhancer {
    fn name(&self) -> &'static str {
        vdn_types::ModelId::FastEnhancerM.as_str()
    }
    fn sample_rate(&self) -> u32 {
        vdn_types::MODEL_RATE
    }
    fn hop(&self) -> usize {
        self.hop
    }
    fn latency_samples(&self) -> usize {
        N_FFT - self.hop
    }
    fn reset(&mut self) {
        for c in &mut self.caches {
            c.extract_tensor_mut().1.fill(0.0);
        }
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        let r = self.infer(input, output);
        if r.is_err() {
            // 缓存可能只回拷了一半：清回初值，不带着坏状态进下一帧（align 自审 P2）
            self.reset();
            output.fill(0.0);
        }
        r
    }
}

// 实时（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl FastEnhancer {
    fn infer(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        let wav = self.wav_in.extract_tensor_mut().1;
        if wav.len() != input.len() {
            return Err(ModelError::Shape);
        }
        wav.copy_from_slice(input);
        // 输入个数固定为 7（wav_in + 6 个缓存）；用定长数组避免拼 Vec
        let [c0, c1, c2, c3, c4, c5] = self.caches.as_slice() else {
            return Err(ModelError::Shape);
        };
        let outs = self
            .session
            .run_with_options(
                ort::inputs![&self.wav_in, c0, c1, c2, c3, c4, c5],
                &self.run,
            )
            .map_err(|_| ModelError::Backend)?;
        // 全部输出都预分配了，values() 按会话输出顺序给出（ort session/output.rs Values）
        let mut vals = outs.values();
        let first = vals.next().ok_or(ModelError::Shape)?;
        session::copy_out(&first, output).map_err(|_| ModelError::Shape)?;
        for cache in self.caches.iter_mut() {
            let v = vals.next().ok_or(ModelError::Shape)?;
            session::copy_out(&v, cache.extract_tensor_mut().1).map_err(|_| ModelError::Shape)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelKind;

    /// 声明里 `wav_in` 的长度就是 hop；延迟 = n_fft − hop（官方口径）；与 `ModelKind::declared` 一致。
    #[test]
    fn declaration_agrees_with_declared_hop_and_latency() {
        let d = ModelKind::FastEnhancerM.declared();
        let wav = IO.inputs.first().unwrap();
        assert_eq!(wav.name, "wav_in");
        assert_eq!(*wav.shape.last().unwrap() as usize, d.hop);
        assert_eq!(N_FFT - d.hop, d.latency_samples);
    }
}
