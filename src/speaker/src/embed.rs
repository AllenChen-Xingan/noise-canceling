//! 声纹提取：WeSpeaker 官方 ONNX（输入 `feats [1, T, 80]`，输出 `embs [1, D]`）。跑在判断回路（非实时线程）。
//!
//! 默认模型 `voxceleb_ECAPA512.onnx`（192 维，HF `Wespeaker/wespeaker-voxceleb-ecapa-tdnn512`，见 `src/speaker/README.md`）。
//! 同接口的 WeSpeaker CAM++（`voxceleb_CAM++.onnx`，512 维）可直接替换，前端相同。
//! 3D-Speaker 的 CAM++ 前端不同（povey 窗、输入不放大）：换 [`FbankConfig`] 即可，模型输入名不同时在 `open` 里报错。

use std::path::{Path, PathBuf};

use ort::session::Session;
use ort::session::builder::{AutoDevicePolicy, GraphOptimizationLevel};
use ort::value::Tensor;

use crate::fbank::{Fbank, FbankConfig, cmn};

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// 默认查找：环境变量 `VDN_SPEAKER_ONNX` → `~/vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx`。
pub fn default_model_path() -> Option<PathBuf> {
    let mut c = Vec::new();
    if let Some(p) = std::env::var_os("VDN_SPEAKER_ONNX") {
        c.push(PathBuf::from(p));
    }
    if let Some(h) = std::env::var_os("HOME") {
        c.push(Path::new(&h).join("vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx"));
    }
    c.into_iter().find(|p| p.is_file())
}

pub struct Embedder {
    session: Session,
    fbank: Fbank,
    feats: Vec<f32>,
    dim: usize,
    input_name: String,
}

static ENV: std::sync::OnceLock<ort::environment::Environment> = std::sync::OnceLock::new();

fn env() -> Result<&'static ort::environment::Environment, BoxError> {
    if let Some(e) = ENV.get() {
        return Ok(e);
    }
    let e = ort::init().with_name("vdn-speaker").build()?;
    Ok(ENV.get_or_init(|| e))
}

fn open_session(path: &Path) -> Result<Session, BoxError> {
    if !path.is_file() {
        return Err(format!("找不到声纹模型：{}", path.display()).into());
    }
    let b = |e: ort::Error<ort::session::builder::SessionBuilder>| -> BoxError { e.to_string().into() };
    // 单线程：声纹线程放能效核，不和降噪线程抢（003 §1）；CPU 执行器，同 vdn-models 的理由（CoreML 自动挂载不稳）
    Ok(Session::builder(env()?)?
        .with_auto_device(AutoDevicePolicy::PreferCPU)
        .map_err(b)?
        .with_optimization_level(GraphOptimizationLevel::All)
        .map_err(b)?
        .with_intra_threads(1)
        .map_err(b)?
        .with_inter_threads(1)
        .map_err(b)?
        .with_parallel_execution(false)
        .map_err(b)?
        .with_intra_op_spinning(false)
        .map_err(b)?
        .commit_from_file(path)?)
}

impl Embedder {
    /// `max_samples`：一次最长送多少样本（决定特征缓冲大小）。
    pub fn open(path: &Path, fbank: FbankConfig, max_samples: usize) -> Result<Self, BoxError> {
        let session = open_session(path)?;
        let inputs: Vec<&str> = session.inputs().iter().map(|i| i.name()).collect();
        if inputs.len() != 1 {
            return Err(format!("声纹模型应只有一个输入，实际 {inputs:?}").into());
        }
        let input_name = inputs[0].to_string();
        let mut e = Self {
            fbank: Fbank::new(fbank),
            feats: Vec::new(),
            dim: 0,
            input_name,
            session,
        };
        let nf = e.fbank.num_frames(max_samples);
        e.feats = vec![0.0; nf * fbank.num_mel_bins];
        // 用一段 1 s 的低电平正弦探出输出维度，顺带预热
        let probe: Vec<f32> = (0..16000).map(|i| 0.01 * (i as f32 * 0.07).sin()).collect();
        e.dim = e.embed(&probe)?.len();
        Ok(e)
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// 一段 16 kHz 波形 → L2 归一化后的声纹向量。
    pub fn embed(&mut self, x: &[f32]) -> Result<Vec<f32>, BoxError> {
        let nb = self.fbank.config().num_mel_bins;
        let need = self.fbank.num_frames(x.len()) * nb;
        if need > self.feats.len() {
            self.feats.resize(need, 0.0);
        }
        let nf = self.fbank.compute_into(x, &mut self.feats);
        if nf < 10 {
            return Err("片段太短，不足 10 帧".into());
        }
        cmn(&mut self.feats, nf, nb);
        let t = Tensor::from_array(([1usize, nf, nb], self.feats[..nf * nb].to_vec()))?;
        let outs = self
            .session
            .run(ort::inputs![self.input_name.as_str() => t])?;
        let (_, v) = outs[0].try_extract_tensor::<f32>()?;
        let mut v = v.to_vec();
        l2_normalize(&mut v);
        Ok(v)
    }

    /// 只算 fbank（含 CMN 前），给对拍用。
    pub fn fbank_only(&mut self, x: &[f32]) -> (usize, Vec<f32>) {
        let nb = self.fbank.config().num_mel_bins;
        let mut out = vec![0.0; self.fbank.num_frames(x.len()) * nb];
        let nf = self.fbank.compute_into(x, &mut out);
        (nf, out)
    }
}

pub fn l2_normalize(v: &mut [f32]) {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 && n.is_finite() {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

/// 两个已归一化向量的余弦相似度。
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
