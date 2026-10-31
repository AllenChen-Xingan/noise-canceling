//! GTCRN simple（`resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`，MIT）适配器。
//!
//! 口径同 exp003 `enhance.py::Gtcrn`：周期 sqrt-Hann 512/256，分析与合成同窗；缓存全零起步、每帧回灌。
//! 输入 `mix [1,257,1,2]`（实部、虚部交错），输出 `enh` 同形；三个缓存 conv / tra / inter。
//! 输出用 `OutputSelector::preallocate` 预分配，缓存每帧拷回输入张量。

use std::path::Path;

use ort::session::{HasSelectedOutputs, OutputSelector, RunOptions, Session};
use ort::value::Tensor;
use vdn_dsp::{StreamingStft, window};
use vdn_engine::{Model, ModelError};

use crate::session::{self, BoxError};

const N_FFT: usize = 512;
const HOP: usize = 256;
const BINS: usize = N_FFT / 2 + 1;
const CACHE_SHAPES: [&[usize]; 3] = [&[2, 1, 16, 16, 33], &[2, 3, 1, 1, 16], &[2, 1, 33, 16]];
const OUT_NAMES: [&str; 4] = ["enh", "conv_cache_out", "tra_cache_out", "inter_cache_out"];

pub struct Gtcrn {
    session: Session,
    run: RunOptions<HasSelectedOutputs>,
    stft: StreamingStft,
    mix: Tensor<f32>,
    caches: [Tensor<f32>; 3],
}

impl Gtcrn {
    pub fn open(path: &Path) -> Result<Self, BoxError> {
        let session = session::open(path)?;
        let names: Vec<&str> = session.inputs().iter().map(|i| i.name()).collect();
        if names != ["mix", "conv_cache", "tra_cache", "inter_cache"] {
            return Err(format!("GTCRN 输入不符：{names:?}").into());
        }
        let mut sel = OutputSelector::default();
        sel = sel.preallocate(OUT_NAMES[0], session::zeros(&[1, BINS, 1, 2])?);
        for (name, shape) in OUT_NAMES[1..].iter().zip(CACHE_SHAPES) {
            sel = sel.preallocate(*name, session::zeros(shape)?);
        }
        Ok(Self {
            session,
            run: RunOptions::new()?.with_outputs(sel),
            stft: StreamingStft::new(N_FFT, HOP, window::sqrt_hann_periodic(N_FFT)),
            mix: session::zeros(&[1, BINS, 1, 2])?,
            caches: [
                session::zeros(CACHE_SHAPES[0])?,
                session::zeros(CACHE_SHAPES[1])?,
                session::zeros(CACHE_SHAPES[2])?,
            ],
        })
    }
}

impl Model for Gtcrn {
    fn name(&self) -> &'static str {
        "gtcrn"
    }
    fn sample_rate(&self) -> u32 {
        16000
    }
    fn hop(&self) -> usize {
        HOP
    }
    fn latency_samples(&self) -> usize {
        self.stft.latency_samples()
    }
    fn reset(&mut self) {
        self.stft.reset();
        for c in &mut self.caches {
            c.extract_tensor_mut().1.fill(0.0);
        }
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        let spec = self.stft.analyze(input);
        {
            let (_, m) = self.mix.extract_tensor_mut();
            for (k, c) in spec.iter().enumerate() {
                m[2 * k] = c.re;
                m[2 * k + 1] = c.im;
            }
        }
        let [c0, c1, c2] = &self.caches;
        let outs = self
            .session
            .run_with_options(ort::inputs![&self.mix, c0, c1, c2], &self.run)
            .map_err(|_| ModelError::Backend)?;
        {
            let (_, enh) = outs[0]
                .try_extract_tensor::<f32>()
                .map_err(|_| ModelError::Shape)?;
            if enh.len() != 2 * BINS {
                return Err(ModelError::Shape);
            }
            let spec = self.stft.spectrum_mut();
            for (k, c) in spec.iter_mut().enumerate() {
                c.re = enh[2 * k];
                c.im = enh[2 * k + 1];
            }
        }
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
        drop(outs);
        self.stft.synthesize(output);
        Ok(())
    }
}
