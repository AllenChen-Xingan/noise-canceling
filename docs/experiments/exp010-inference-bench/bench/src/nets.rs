//! 各后端（ORT · 手写 · tract）的统一接口 `Net`、模型打开与输入准备（读 wav、STFT 频谱）。

use crate::*;

pub(crate) const N_FFT: usize = 512;
pub(crate) const HOP: usize = 256;
pub(crate) const BINS: usize = 257;

pub(crate) fn read_wav(p: &Path) -> Result<Vec<f32>, Err> {
    let mut r = hound::WavReader::open(p)?;
    let s = r.spec();
    if s.sample_rate != 16000 || s.channels != 1 {
        return Err("只收 16 kHz 单声道".into());
    }
    Ok(match s.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let m = (1u64 << (s.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|v| v.map(|v| v as f32 / m))
                .collect::<Result<_, _>>()?
        }
    })
}

/// 把整段音频切成逐帧频谱（re, im），与 vdn-models 同一个 STFT。
pub(crate) fn spectra(wav: &[f32], frames: usize) -> Vec<([f32; BINS], [f32; BINS])> {
    let mut stft = StreamingStft::new(N_FFT, HOP, window::sqrt_hann_periodic(N_FFT));
    let mut out = Vec::with_capacity(frames);
    let n = wav.len() / HOP;
    for i in 0..frames {
        let k = i % n;
        let s = stft.analyze(&wav[k * HOP..(k + 1) * HOP]);
        let mut re = [0.0; BINS];
        let mut im = [0.0; BINS];
        for (j, c) in s.iter().enumerate() {
            re[j] = c.re;
            im[j] = c.im;
        }
        out.push((re, im));
    }
    out
}

/// ORT 会话：设置同 src/models/src/session.rs（单线程、不自旋、PreferCPU、ORT_ENABLE_ALL）。
pub(crate) fn ort_session(path: &Path) -> Result<Session, Err> {
    let b = |e: ort::Error<ort::session::builder::SessionBuilder>| -> Err { e.to_string().into() };
    static ENV: std::sync::OnceLock<ort::environment::Environment> = std::sync::OnceLock::new();
    if ENV.get().is_none() {
        let e = ort::init().with_name("exp010").build()?;
        let _ = ENV.set(e);
    }
    let s = Session::builder(ENV.get().ok_or("env")?)?
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
        .commit_from_file(path)?;
    Ok(s)
}

// ── ORT 基线（设置同 src/models/src/session.rs：单线程、不自旋、PreferCPU、ORT_ENABLE_ALL） ──
pub(crate) struct OrtGtcrn {
    pub(crate) session: Session,
    pub(crate) run: RunOptions<HasSelectedOutputs>,
    pub(crate) mix: Tensor<f32>,
    pub(crate) caches: [Tensor<f32>; 3],
}
pub(crate) const CACHE_SHAPES: [&[usize]; 3] = [&[2, 1, 16, 16, 33], &[2, 3, 1, 1, 16], &[2, 1, 33, 16]];
pub(crate) const OUT_NAMES: [&str; 4] = ["enh", "conv_cache_out", "tra_cache_out", "inter_cache_out"];

/// GTCRN 的手抄形状（mix + CACHE_SHAPES）必须和 ONNX 文件读出的一致，不一致就报错
pub(crate) fn check_gtcrn_shapes(onnx: &[Vec<usize>]) -> Result<(), Err> {
    let mut want = vec![vec![1, BINS, 1, 2]];
    want.extend(CACHE_SHAPES.iter().map(|s| s.to_vec()));
    if onnx != want.as_slice() {
        return Err(format!("GTCRN 输入形状和 ONNX 不一致：手抄 {want:?}，ONNX {onnx:?}").into());
    }
    Ok(())
}

pub(crate) fn zeros(shape: &[usize]) -> Result<Tensor<f32>, Err> {
    Ok(Tensor::from_array((
        shape.to_vec(),
        vec![0.0f32; shape.iter().product()],
    ))?)
}

impl OrtGtcrn {
    pub(crate) fn open(path: &Path) -> Result<Self, Err> {
        let session = ort_session(path)?;
        check_gtcrn_shapes(&input_shapes(&session)?)?;
        let mut sel = OutputSelector::default();
        sel = sel.preallocate(OUT_NAMES[0], zeros(&[1, BINS, 1, 2])?);
        for (name, shape) in OUT_NAMES[1..].iter().zip(CACHE_SHAPES) {
            sel = sel.preallocate(*name, zeros(shape)?);
        }
        Ok(Self {
            session,
            run: RunOptions::new()?.with_outputs(sel),
            mix: zeros(&[1, BINS, 1, 2])?,
            caches: [
                zeros(CACHE_SHAPES[0])?,
                zeros(CACHE_SHAPES[1])?,
                zeros(CACHE_SHAPES[2])?,
            ],
        })
    }

    pub(crate) fn infer(&mut self, re: &mut [f32], im: &mut [f32]) -> Result<(), Err> {
        {
            let (_, m) = self.mix.extract_tensor_mut();
            for k in 0..BINS {
                m[2 * k] = re[k];
                m[2 * k + 1] = im[k];
            }
        }
        let [c0, c1, c2] = &self.caches;
        let outs = self
            .session
            .run_with_options(ort::inputs![&self.mix, c0, c1, c2], &self.run)?;
        let mut vals = outs.values();
        let v0 = vals.next().ok_or("无输出")?;
        let (_, enh) = v0.try_extract_tensor::<f32>()?;
        for k in 0..BINS {
            re[k] = enh[2 * k];
            im[k] = enh[2 * k + 1];
        }
        for cache in self.caches.iter_mut() {
            let v = vals.next().ok_or("无缓存输出")?;
            let (_, src) = v.try_extract_tensor::<f32>()?;
            cache.extract_tensor_mut().1.copy_from_slice(src);
        }
        Ok(())
    }
}

/// 一帧原地推理：`x` 进、`x` 出。GTCRN 的一帧是 [re 257 | im 257]，FE-M 的一帧是 160 个样本。
pub(crate) trait Net {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err>;
    /// 清回零状态。只有手写版实现了（`--reset-at` 检查用）。
    fn reset(&mut self) {}
}

impl Net for OrtGtcrn {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
        let (re, im) = x.split_at_mut(BINS);
        OrtGtcrn::infer(self, re, im)
    }
}
impl Net for GtcrnNative {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
        let (re, im) = x.split_at_mut(BINS);
        GtcrnNative::infer(self, re, im);
        Ok(())
    }
    fn reset(&mut self) {
        GtcrnNative::reset(self);
    }
}

/// tract 跑 GTCRN：输入 mix 是 [1,257,1,2]（re、im 交错），其余三个缓存，形状同 ORT。
pub(crate) struct TractGtcrn {
    pub(crate) net: TractNet,
    pub(crate) out: Vec<f32>,
}
impl TractGtcrn {
    pub(crate) fn open(path: &Path) -> Result<Self, Err> {
        // 和 ONNX 里的输入形状交叉校验（CLAUDE.md 第五节第 5 条）：正本是 ONNX 文件，CACHE_SHAPES 是手抄的第二处
        check_gtcrn_shapes(&input_shapes(&ort_session(path)?)?)?;
        let mut shapes = vec![vec![1, BINS, 1, 2]];
        shapes.extend(CACHE_SHAPES.iter().map(|s| s.to_vec()));
        Ok(Self {
            net: TractNet::open(path, &shapes)?,
            out: vec![0.0; 2 * BINS],
        })
    }
}
impl Net for TractGtcrn {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
        let m = self.net.input0();
        for k in 0..BINS {
            m[2 * k] = x[k];
            m[2 * k + 1] = x[BINS + k];
        }
        self.net.step(&mut self.out)?;
        for k in 0..BINS {
            x[k] = self.out[2 * k];
            x[BINS + k] = self.out[2 * k + 1];
        }
        Ok(())
    }
}

/// 读 ONNX 输入形状（动态维取 1），给 FE-M 两个后端共用。
pub(crate) fn input_shapes(session: &Session) -> Result<Vec<Vec<usize>>, Err> {
    session
        .inputs()
        .iter()
        .map(|i| {
            i.dtype()
                .tensor_shape()
                .map(|s| s.iter().map(|&d| d.max(1) as usize).collect())
                .ok_or_else(|| -> Err { "输入不是张量".into() })
        })
        .collect()
}

/// FastEnhancer-M（官方 onnx-dns-v1.0.0 wav2wav 版）ORT 基线，口径同 src/models/src/fastenhancer.rs：
/// wav_in + 6 个缓存，输出按位置回灌。
pub(crate) struct OrtFem {
    pub(crate) session: Session,
    pub(crate) run: RunOptions<HasSelectedOutputs>,
    pub(crate) ins: Vec<Tensor<f32>>,
}
impl OrtFem {
    pub(crate) fn open(path: &Path) -> Result<Self, Err> {
        let session = ort_session(path)?;
        let shapes = input_shapes(&session)?;
        if shapes.len() != 7 || session.outputs().len() != 7 {
            return Err("FE-M 应有 7 个输入 7 个输出".into());
        }
        let mut sel = OutputSelector::default();
        for (o, s) in session.outputs().iter().zip(&shapes) {
            sel = sel.preallocate(o.name(), zeros(s)?);
        }
        let ins = shapes
            .iter()
            .map(|s| zeros(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            run: RunOptions::new()?.with_outputs(sel),
            ins,
            session,
        })
    }
}
impl Net for OrtFem {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
        self.ins[0].extract_tensor_mut().1.copy_from_slice(x);
        let [a, b, c, d, e, f, g] = self.ins.as_slice() else {
            return Err("输入个数".into());
        };
        let outs = self
            .session
            .run_with_options(ort::inputs![a, b, c, d, e, f, g], &self.run)?;
        let mut vals = outs.values();
        let v0 = vals.next().ok_or("无输出")?;
        x.copy_from_slice(v0.try_extract_tensor::<f32>()?.1);
        for cache in self.ins[1..].iter_mut() {
            let v = vals.next().ok_or("无缓存输出")?;
            cache
                .extract_tensor_mut()
                .1
                .copy_from_slice(v.try_extract_tensor::<f32>()?.1);
        }
        Ok(())
    }
}

pub(crate) struct TractFem {
    pub(crate) net: TractNet,
    pub(crate) out: Vec<f32>,
}
impl TractFem {
    /// `shapes` 由调用方在计时前读好（不把 ORT 建会话算进 tract 的打开时间）。
    pub(crate) fn open(path: &Path, shapes: &[Vec<usize>]) -> Result<Self, Err> {
        let shapes = shapes.to_vec();
        let hop = *shapes[0].last().ok_or("wav_in 形状空")?;
        Ok(Self {
            net: TractNet::open(path, &shapes)?,
            out: vec![0.0; hop],
        })
    }
}
impl Net for TractFem {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
        self.net.input0().copy_from_slice(x);
        self.net.step(&mut self.out)?;
        x.copy_from_slice(&self.out);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ModelId {
    Gtcrn,
    Fem,
}
impl ModelId {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ModelId::Gtcrn => "gtcrn",
            ModelId::Fem => "fastenhancer_m",
        }
    }
}

/// 计时前的准备：初始化 ORT 环境，并用一个 ORT 会话读出输入形状（会话随即丢掉）。
/// 对所有后端、两个模型都做，这样「打开」一律不含 ORT 环境初始化，模型文件也都已在页缓存里。
pub(crate) fn prepare(path_onnx: &Path) -> Result<(Vec<Vec<usize>>, f64), Err> {
    let t = Instant::now();
    let s = ort_session(path_onnx)?;
    let ms = t.elapsed().as_secs_f64() * 1e3;
    Ok((input_shapes(&s)?, ms))
}

pub(crate) fn open_net(model: ModelId, kind: &str, path: &Path) -> Result<Box<dyn Net>, Err> {
    open_net_with(model, kind, path, None)
}

pub(crate) fn open_net_with(
    model: ModelId,
    kind: &str,
    path: &Path,
    shapes: Option<&[Vec<usize>]>,
) -> Result<Box<dyn Net>, Err> {
    Ok(match (model, kind) {
        (ModelId::Gtcrn, "ort") => Box::new(OrtGtcrn::open(path)?),
        (ModelId::Gtcrn, k @ ("native" | "native-blas" | "native-fast" | "native-blas-fast")) => {
            // 两个全局开关只在构造时设；同一进程里先后开两个不同 native 变体时，以后开的为准（对拍、基准都只开一种）
            gtcrn_native::BLAS.store(k.contains("blas"), Relaxed);
            gtcrn_native::FAST.store(k.contains("fast"), Relaxed);
            Box::new(GtcrnNative::load(path)?)
        }
        // 研究 020：同一份权重与结构，算子换成 NEON 重写（v2 不读 BLAS / FAST 两个全局开关；BLAS 关掉以免 DPGRNN 走 sgemm）
        // 后缀 -pf：每帧开头预取全部权重进 L2（D79「唤醒时预取权重」）
        (
            ModelId::Gtcrn,
            k @ ("native-v2" | "native-v2-fast" | "native-v2-pf" | "native-v2-fast-pf" | "native-v3" | "native-v3-pf" | "native-v3i" | "native-v3i-pf" | "native-v4" | "native-v4i" | "native-v4-pf" | "native-v4i-pf"),
        ) => {
            gtcrn_native::BLAS.store(false, Relaxed);
            let mut n = GtcrnNative::load(path)?;
            n.set_kind(gtcrn_native::Kind::from_backend(k).ok_or("未知后端")?);
            n.set_prefetch(k.ends_with("-pf"));
            Box::new(n)
        }
        (ModelId::Gtcrn, "tract") => Box::new(TractGtcrn::open(path)?),
        (ModelId::Fem, "ort") => Box::new(OrtFem::open(path)?),
        (ModelId::Fem, "tract") => match shapes {
            Some(s) => Box::new(TractFem::open(path, s)?),
            None => Box::new(TractFem::open(path, &input_shapes(&ort_session(path)?)?)?),
        },
        _ => return Err(format!("{} 没有后端 {kind}", model.name()).into()),
    })
}

/// 逐帧输入：GTCRN 是同一个 STFT 的 [re|im]；FE-M 是逐 hop 的波形。超出音频长度就循环。
pub(crate) fn frames_for(
    model: ModelId,
    audio: &[f32],
    frames: usize,
    shapes: &[Vec<usize>],
) -> Result<Vec<Vec<f32>>, Err> {
    // 音频至少一个 hop、帧数 > 0，否则 spectra / 循环取模会除零或越界（codex 第 4 轮 #2）
    let hop = if model == ModelId::Gtcrn {
        HOP
    } else {
        *shapes
            .first()
            .and_then(|s| s.last())
            .ok_or("FE-M 需要输入形状")?
    };
    if frames == 0 || audio.len() < hop {
        return Err(format!(
            "帧数 {frames} 为 0，或音频 {} 个样本短于一个 hop（{hop}）",
            audio.len()
        )
        .into());
    }
    Ok(match model {
        ModelId::Gtcrn => spectra(audio, frames)
            .into_iter()
            .map(|(re, im)| re.iter().chain(im.iter()).copied().collect())
            .collect(),
        ModelId::Fem => {
            let hop = *shapes
                .first()
                .and_then(|s| s.last())
                .ok_or("FE-M 需要输入形状")?;
            let n = audio.len() / hop;
            (0..frames)
                .map(|i| audio[(i % n) * hop..(i % n + 1) * hop].to_vec())
                .collect()
        }
    })
}
