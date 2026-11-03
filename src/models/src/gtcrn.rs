//! GTCRN simple（`resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`，MIT）适配器。
//!
//! 口径同 exp003 `enhance.py::Gtcrn`：周期 sqrt-Hann 512/256，分析与合成同窗；缓存全零起步、每帧回灌。
//! 输入 `mix [1,257,1,2]`（实部、虚部交错），输出 `enh` 同形；三个缓存 conv / tra / inter。
//! 输出用 `OutputSelector::preallocate` 预分配，缓存每帧拷回输入张量。张量名字与形状的正本是 `interface::GTCRN`。

use std::path::Path;

use ort::session::{HasSelectedOutputs, OutputSelector, RunOptions, Session};
use ort::value::Tensor;
use vdn_backend_ort::ort;
use vdn_dsp::{StreamingStft, window};
use vdn_types::{Model, ModelError};

use vdn_backend_ort::{self as session, BoxError};

use crate::interface::{self, Interface, TensorDecl, check_session, dims};

const N_FFT: usize = 512;
const HOP: usize = 256;
const BINS: usize = N_FFT / 2 + 1;
/// 输入输出的声明（名字 · 顺序 · 形状）正本在 `interface::GTCRN`；`mix` 的频点数由测试与 `BINS` 比对。
const IO: Interface = interface::GTCRN;

pub struct Gtcrn {
    session: Session,
    run: RunOptions<HasSelectedOutputs>,
    stft: StreamingStft,
    mix: Tensor<f32>,
    caches: [Tensor<f32>; 3],
}

// 构造：非实时线程
impl Gtcrn {
    pub fn open(path: &Path) -> Result<Self, BoxError> {
        let session = session::open(path)?;
        // 名字、顺序、形状逐个对声明；输出按会话顺序取（outs.values()），顺序错了会悄悄算错
        check_session(&IO, &session).map_err(|e| format!("GTCRN {e}"))?;
        let mut sel = OutputSelector::default();
        for o in IO.outputs {
            sel = sel.preallocate(o.name, session::zeros(&dims(o.shape))?);
        }
        let zeros = |d: Option<&TensorDecl>| -> Result<Tensor<f32>, BoxError> {
            session::zeros(&dims(d.ok_or("GTCRN 声明缺输入")?.shape))
        };
        Ok(Self {
            session,
            run: RunOptions::new()?.with_outputs(sel),
            stft: StreamingStft::new(N_FFT, HOP, window::sqrt_hann_periodic(N_FFT)),
            mix: zeros(IO.inputs.first())?,
            caches: [
                zeros(IO.inputs.get(1))?,
                zeros(IO.inputs.get(2))?,
                zeros(IO.inputs.get(3))?,
            ],
        })
    }
}

// 实时（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Model for Gtcrn {
    fn name(&self) -> &'static str {
        vdn_types::ModelId::Gtcrn.as_str()
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
        if input.len() != HOP || output.len() != HOP {
            // 调用方的 bug：不静默产出错音频（align 自审 P1）
            output.fill(0.0);
            return Err(ModelError::Shape);
        }
        let r = self.infer(input);
        if r.is_err() {
            // 出错也要照常合成一帧（频谱清零），否则重叠相加缓冲不移位，之后整体错位一个 hop（自审 P2）
            self.stft.spectrum_mut().fill(Default::default());
            // 缓存可能只回拷了一半：清回初值，不带着坏状态进下一帧（align 自审 P2）
            for c in &mut self.caches {
                c.extract_tensor_mut().1.fill(0.0);
            }
        }
        self.stft.synthesize(output);
        r
    }
}

// 实时（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Gtcrn {
    /// 分析一帧 → 推理 → 把增强后的频谱写回 STFT、缓存拷回输入。不做合成。
    fn infer(&mut self, input: &[f32]) -> Result<(), ModelError> {
        let spec = self.stft.analyze(input);
        {
            let (_, m) = self.mix.extract_tensor_mut();
            if m.len() != 2 * spec.len() {
                return Err(ModelError::Shape);
            }
            for ([re, im], c) in m.as_chunks_mut::<2>().0.iter_mut().zip(spec.iter()) {
                *re = c.re;
                *im = c.im;
            }
        }
        let [c0, c1, c2] = &self.caches;
        let outs = self
            .session
            .run_with_options(ort::inputs![&self.mix, c0, c1, c2], &self.run)
            .map_err(|_| ModelError::Backend)?;
        // 全部输出都预分配了，values() 按会话输出顺序给出，名字已在 open() 里逐个核对
        let mut vals = outs.values();
        {
            let first = vals.next().ok_or(ModelError::Shape)?;
            let (_, enh) = first
                .try_extract_tensor::<f32>()
                .map_err(|_| ModelError::Shape)?;
            if enh.len() != 2 * BINS {
                return Err(ModelError::Shape);
            }
            let spec = self.stft.spectrum_mut();
            for (c, [re, im]) in spec.iter_mut().zip(enh.as_chunks::<2>().0) {
                c.re = *re;
                c.im = *im;
            }
        }
        for cache in self.caches.iter_mut() {
            let v = vals.next().ok_or(ModelError::Shape)?;
            session::copy_out(&v, cache.extract_tensor_mut().1).map_err(|_| ModelError::Shape)?;
        }
        Ok(())
    }
}
