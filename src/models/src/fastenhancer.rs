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

const N_FFT: usize = 512;

pub struct FastEnhancer {
    session: Session,
    run: RunOptions<HasSelectedOutputs>,
    hop: usize,
    wav_in: Tensor<f32>,
    caches: Vec<Tensor<f32>>,
}

// 构造：非实时线程
impl FastEnhancer {
    #[allow(clippy::indexing_slicing)] // NRT: 构造期一次性读形状；shapes 非空已由上面的 wav_in 检查保证
    pub fn open(path: &Path) -> Result<Self, BoxError> {
        let session = session::open(path)?;
        let ins = session.inputs();
        if ins.first().map(|i| i.name()) != Some("wav_in") {
            return Err("FastEnhancer 第一个输入应是 wav_in".into());
        }
        let shapes: Vec<Vec<usize>> = ins
            .iter()
            .map(|i| {
                i.dtype()
                    .tensor_shape()
                    .map(|s| s.iter().map(|&d| d.max(1) as usize).collect())
                    .ok_or("FastEnhancer 输入不是张量")
            })
            .collect::<Result<_, _>>()?;
        let hop = *shapes[0].last().ok_or("wav_in 形状为空")?;
        let mut sel = OutputSelector::default();
        let outs = session.outputs();
        if outs.len() != ins.len() {
            return Err("FastEnhancer 输入输出个数不符".into());
        }
        // 输入输出按位置配对，名字必须是 wav_in/wav_out、cache_in_i/cache_out_i（形状相同时顺序乱了不会报错，只会悄悄算错）
        for (i, (a, b)) in ins.iter().zip(outs.iter()).enumerate() {
            let (want_in, want_out) = if i == 0 {
                ("wav_in".to_string(), "wav_out".to_string())
            } else {
                (
                    format!("cache_in_{}", i - 1),
                    format!("cache_out_{}", i - 1),
                )
            };
            if a.name() != want_in || b.name() != want_out {
                return Err(format!(
                    "FastEnhancer 第 {i} 对输入输出名不符：{} / {}",
                    a.name(),
                    b.name()
                )
                .into());
            }
        }
        for (o, shape) in outs.iter().zip(&shapes) {
            sel = sel.preallocate(o.name(), session::zeros(shape)?);
        }
        let caches = shapes[1..]
            .iter()
            .map(|s| session::zeros(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            run: RunOptions::new()?.with_outputs(sel),
            hop,
            wav_in: session::zeros(&shapes[0])?,
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
        16000
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
