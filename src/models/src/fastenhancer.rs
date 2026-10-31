//! FastEnhancer（官方 release `onnx-dns-v1.0.0` 的 wav2wav 版，STFT/iSTFT/OLA 都在图里）适配器。
//!
//! 口径同 exp003 `enhance.py::FastEnhancer`：每次喂一个 hop（M 档 160 样本），缓存原样回灌；
//! 输出比输入晚 `n_fft − hop = 512 − hop` 个样本（官方 `docs/docs/onnx.md`）。
//! 权重许可待核（003 技术方案 §5），发布前必须定。

use std::path::Path;

use ort::session::{HasSelectedOutputs, OutputSelector, RunOptions, Session};
use ort::value::Tensor;
use vdn_engine::{Model, ModelError};

use crate::session::{self, BoxError};

const N_FFT: usize = 512;

pub struct FastEnhancer {
    session: Session,
    run: RunOptions<HasSelectedOutputs>,
    hop: usize,
    wav_in: Tensor<f32>,
    caches: Vec<Tensor<f32>>,
}

impl FastEnhancer {
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

impl Model for FastEnhancer {
    fn name(&self) -> &'static str {
        "fastenhancer"
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
        self.wav_in.extract_tensor_mut().1.copy_from_slice(input);
        let c = &self.caches;
        // 输入个数固定为 7（wav_in + 6 个缓存）；用定长数组避免拼 Vec
        if c.len() != 6 {
            return Err(ModelError::Shape);
        }
        let outs = self
            .session
            .run_with_options(
                ort::inputs![&self.wav_in, &c[0], &c[1], &c[2], &c[3], &c[4], &c[5]],
                &self.run,
            )
            .map_err(|_| ModelError::Backend)?;
        let (_, y) = outs[0]
            .try_extract_tensor::<f32>()
            .map_err(|_| ModelError::Shape)?;
        if y.len() != output.len() {
            return Err(ModelError::Shape);
        }
        output.copy_from_slice(y);
        for (i, cache) in self.caches.iter_mut().enumerate() {
            let (_, src) = outs[i + 1]
                .try_extract_tensor::<f32>()
                .map_err(|_| ModelError::Shape)?;
            let dst = cache.extract_tensor_mut().1;
            if src.len() != dst.len() {
                return Err(ModelError::Shape);
            }
            dst.copy_from_slice(src);
        }
        Ok(())
    }
}
