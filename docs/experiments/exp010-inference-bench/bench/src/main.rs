//! exp010 推理后端基准：同口径比 ORT CPU（基线）、手写 Rust、tract（纯 Rust ONNX 推理）。
//!
//! 子命令：
//!   gtcrn-parity <权重前缀> <onnx> <wav> [--vs native|tract]   逐帧对拍增强后频谱（ORT 对 --vs，默认 native；两边吃同一份 STFT 频谱）
//!   gtcrn-bench  <ort|native|tract> <权重前缀|onnx> <wav> [--frames N] [--warmup N]
//!   fem-parity   <onnx> <wav>                                   FastEnhancer-M：ORT 对 tract，逐帧对拍输出波形
//!   fem-bench    <ort|tract> <onnx> <wav> [--frames N] [--warmup N]
//! 输出一行 JSON（stdout）。一帧 = 模型的一个 hop（GTCRN 256 样本 / 16 ms，FE-M 160 样本 / 10 ms）。

mod gtcrn_native;
mod tract_be;

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

use gtcrn_native::GtcrnNative;
use tract_be::TractNet;
use ort::session::builder::{AutoDevicePolicy, GraphOptimizationLevel};
use ort::session::{HasSelectedOutputs, OutputSelector, RunOptions, Session};
use ort::value::Tensor;
use vdn_dsp::{StreamingStft, window};

type Err = Box<dyn std::error::Error + Send + Sync>;

// ── 分配计数 ──
struct Counting;
static ALLOCS: AtomicU64 = AtomicU64::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static A: Counting = Counting;

const N_FFT: usize = 512;
const HOP: usize = 256;
const BINS: usize = 257;

fn read_wav(p: &Path) -> Result<Vec<f32>, Err> {
    let mut r = hound::WavReader::open(p)?;
    let s = r.spec();
    if s.sample_rate != 16000 || s.channels != 1 {
        return Err("只收 16 kHz 单声道".into());
    }
    Ok(match s.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let m = (1u64 << (s.bits_per_sample - 1)) as f32;
            r.samples::<i32>().map(|v| v.map(|v| v as f32 / m)).collect::<Result<_, _>>()?
        }
    })
}

/// 把整段音频切成逐帧频谱（re, im），与 vdn-models 同一个 STFT。
fn spectra(wav: &[f32], frames: usize) -> Vec<([f32; BINS], [f32; BINS])> {
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
fn ort_session(path: &Path) -> Result<Session, Err> {
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
struct OrtGtcrn {
    session: Session,
    run: RunOptions<HasSelectedOutputs>,
    mix: Tensor<f32>,
    caches: [Tensor<f32>; 3],
}
const CACHE_SHAPES: [&[usize]; 3] = [&[2, 1, 16, 16, 33], &[2, 3, 1, 1, 16], &[2, 1, 33, 16]];
const OUT_NAMES: [&str; 4] = ["enh", "conv_cache_out", "tra_cache_out", "inter_cache_out"];

fn zeros(shape: &[usize]) -> Result<Tensor<f32>, Err> {
    Ok(Tensor::from_array((shape.to_vec(), vec![0.0f32; shape.iter().product()]))?)
}

impl OrtGtcrn {
    fn open(path: &Path) -> Result<Self, Err> {
        let session = ort_session(path)?;
        let mut sel = OutputSelector::default();
        sel = sel.preallocate(OUT_NAMES[0], zeros(&[1, BINS, 1, 2])?);
        for (name, shape) in OUT_NAMES[1..].iter().zip(CACHE_SHAPES) {
            sel = sel.preallocate(*name, zeros(shape)?);
        }
        Ok(Self {
            session,
            run: RunOptions::new()?.with_outputs(sel),
            mix: zeros(&[1, BINS, 1, 2])?,
            caches: [zeros(CACHE_SHAPES[0])?, zeros(CACHE_SHAPES[1])?, zeros(CACHE_SHAPES[2])?],
        })
    }

    fn infer(&mut self, re: &mut [f32], im: &mut [f32]) -> Result<(), Err> {
        {
            let (_, m) = self.mix.extract_tensor_mut();
            for k in 0..BINS {
                m[2 * k] = re[k];
                m[2 * k + 1] = im[k];
            }
        }
        let [c0, c1, c2] = &self.caches;
        let outs = self.session.run_with_options(ort::inputs![&self.mix, c0, c1, c2], &self.run)?;
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
trait Net {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err>;
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
}

/// tract 跑 GTCRN：输入 mix 是 [1,257,1,2]（re、im 交错），其余三个缓存，形状同 ORT。
struct TractGtcrn {
    net: TractNet,
    out: Vec<f32>,
}
impl TractGtcrn {
    fn open(path: &Path) -> Result<Self, Err> {
        let mut shapes = vec![vec![1, BINS, 1, 2]];
        shapes.extend(CACHE_SHAPES.iter().map(|s| s.to_vec()));
        Ok(Self { net: TractNet::open(path, &shapes)?, out: vec![0.0; 2 * BINS] })
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
fn input_shapes(session: &Session) -> Result<Vec<Vec<usize>>, Err> {
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
struct OrtFem {
    session: Session,
    run: RunOptions<HasSelectedOutputs>,
    ins: Vec<Tensor<f32>>,
}
impl OrtFem {
    fn open(path: &Path) -> Result<Self, Err> {
        let session = ort_session(path)?;
        let shapes = input_shapes(&session)?;
        if shapes.len() != 7 || session.outputs().len() != 7 {
            return Err("FE-M 应有 7 个输入 7 个输出".into());
        }
        let mut sel = OutputSelector::default();
        for (o, s) in session.outputs().iter().zip(&shapes) {
            sel = sel.preallocate(o.name(), zeros(s)?);
        }
        let ins = shapes.iter().map(|s| zeros(s)).collect::<Result<Vec<_>, _>>()?;
        Ok(Self { run: RunOptions::new()?.with_outputs(sel), ins, session })
    }
}
impl Net for OrtFem {
    fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
        self.ins[0].extract_tensor_mut().1.copy_from_slice(x);
        let [a, b, c, d, e, f, g] = self.ins.as_slice() else { return Err("输入个数".into()) };
        let outs = self.session.run_with_options(ort::inputs![a, b, c, d, e, f, g], &self.run)?;
        let mut vals = outs.values();
        let v0 = vals.next().ok_or("无输出")?;
        x.copy_from_slice(v0.try_extract_tensor::<f32>()?.1);
        for cache in self.ins[1..].iter_mut() {
            let v = vals.next().ok_or("无缓存输出")?;
            cache.extract_tensor_mut().1.copy_from_slice(v.try_extract_tensor::<f32>()?.1);
        }
        Ok(())
    }
}

struct TractFem {
    net: TractNet,
    out: Vec<f32>,
}
impl TractFem {
    fn open(path: &Path) -> Result<Self, Err> {
        let shapes = input_shapes(&ort_session(path)?)?; // 只借 ORT 读形状，会话随即丢掉
        let hop = *shapes[0].last().ok_or("wav_in 形状空")?;
        Ok(Self { net: TractNet::open(path, &shapes)?, out: vec![0.0; hop] })
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
enum ModelId {
    Gtcrn,
    Fem,
}
impl ModelId {
    fn name(self) -> &'static str {
        match self {
            ModelId::Gtcrn => "gtcrn",
            ModelId::Fem => "fastenhancer_m",
        }
    }
}

fn open_net(model: ModelId, kind: &str, path: &Path) -> Result<Box<dyn Net>, Err> {
    Ok(match (model, kind) {
        (ModelId::Gtcrn, "ort") => Box::new(OrtGtcrn::open(path)?),
        (ModelId::Gtcrn, "native") => Box::new(GtcrnNative::load(path)?),
        (ModelId::Gtcrn, "tract") => Box::new(TractGtcrn::open(path)?),
        (ModelId::Fem, "ort") => Box::new(OrtFem::open(path)?),
        (ModelId::Fem, "tract") => Box::new(TractFem::open(path)?),
        _ => return Err(format!("{} 没有后端 {kind}", model.name()).into()),
    })
}

/// 逐帧输入：GTCRN 是同一个 STFT 的 [re|im]；FE-M 是逐 hop 的波形。超出音频长度就循环。
fn frames_for(model: ModelId, audio: &[f32], frames: usize, onnx_or_any: &Path) -> Result<Vec<Vec<f32>>, Err> {
    Ok(match model {
        ModelId::Gtcrn => spectra(audio, frames)
            .into_iter()
            .map(|(re, im)| re.iter().chain(im.iter()).copied().collect())
            .collect(),
        ModelId::Fem => {
            let hop = *input_shapes(&ort_session(onnx_or_any)?)?[0].last().ok_or("wav_in 形状空")?;
            let n = audio.len() / hop;
            (0..frames).map(|i| audio[(i % n) * hop..(i % n + 1) * hop].to_vec()).collect()
        }
    })
}

fn cpu_seconds() -> f64 {
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let t = |v: libc::timeval| v.tv_sec as f64 + v.tv_usec as f64 * 1e-6;
    t(u.ru_utime) + t(u.ru_stime)
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

fn arg_usize(args: &[String], flag: &str, d: usize) -> usize {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(d)
}

fn bench(model: ModelId, args: &[String]) -> Result<(), Err> {
    let (kind, path, wav) = (&args[0], Path::new(&args[1]), Path::new(&args[2]));
    let frames = arg_usize(args, "--frames", 6000);
    let warm = arg_usize(args, "--warmup", 200);
    let audio = read_wav(wav)?;
    // FE-M 的帧形状要从 ONNX 读；GTCRN 不用 path
    let inputs = frames_for(model, &audio, frames + warm + 1, path)?;
    let mut x = inputs[0].clone();
    // 冷启动：打开 + 第一帧
    let t0 = Instant::now();
    let mut m = open_net(model, kind, path)?;
    let t_open = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = Instant::now();
    m.infer(&mut x)?;
    let t_first = t1.elapsed().as_secs_f64() * 1e3;
    for f in &inputs[1..=warm] {
        x.copy_from_slice(f);
        m.infer(&mut x)?;
    }
    let mut times = Vec::with_capacity(frames);
    let mut nan = 0usize;
    let a0 = ALLOCS.load(Relaxed);
    let c0 = cpu_seconds();
    let w0 = Instant::now();
    for f in &inputs[warm + 1..] {
        x.copy_from_slice(f);
        let t = Instant::now();
        m.infer(&mut x)?;
        times.push(t.elapsed().as_secs_f64() * 1e3);
        if !x.iter().all(|v| v.is_finite()) {
            nan += 1;
        }
    }
    let wall = w0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    let allocs = ALLOCS.load(Relaxed) - a0;
    times.sort_by(|a, b| a.total_cmp(b));
    println!(
        "{{\"model\":\"{}\",\"backend\":\"{kind}\",\"frames\":{frames},\"p50_ms\":{:.4},\"p99_ms\":{:.4},\"p999_ms\":{:.4},\"max_ms\":{:.4},\"mean_ms\":{:.4},\"open_ms\":{t_open:.2},\"first_frame_ms\":{t_first:.3},\"cpu_over_wall\":{:.3},\"rust_allocs_per_frame\":{:.3},\"nonfinite_frames\":{nan}}}",
        model.name(),
        pct(&times, 0.5),
        pct(&times, 0.99),
        pct(&times, 0.999),
        times[times.len() - 1],
        times.iter().sum::<f64>() / times.len() as f64,
        cpu / wall,
        allocs as f64 / frames as f64,
    );
    Ok(())
}

/// 逐帧对拍：参考 `a`（ORT）对 `b`，同一份输入，比整帧输出。
fn parity(model: ModelId, vs: &str, mut a: Box<dyn Net>, mut b: Box<dyn Net>, inputs: &[Vec<f32>]) -> Result<(), Err> {
    let (mut max_abs, mut sig, mut err) = (0.0f64, 0.0f64, 0.0f64);
    let mut worst_frame = 0;
    let mut first_bad = None;
    let mut xa = inputs[0].clone();
    let mut xb = inputs[0].clone();
    for (f, x) in inputs.iter().enumerate() {
        xa.copy_from_slice(x);
        xb.copy_from_slice(x);
        a.infer(&mut xa)?;
        b.infer(&mut xb)?;
        for (k, (&p, &q)) in xa.iter().zip(&xb).enumerate() {
            let d = (p as f64 - q as f64).abs();
            sig += (p as f64).powi(2);
            err += d * d;
            if d > max_abs {
                max_abs = d;
                worst_frame = f;
            }
            if d > 1e-3 && first_bad.is_none() {
                first_bad = Some((f, k));
            }
        }
    }
    println!(
        "{{\"model\":\"{}\",\"vs\":\"{vs}\",\"frames\":{},\"max_abs\":{max_abs:.3e},\"worst_frame\":{worst_frame},\"snr_db\":{:.2},\"first_bad\":\"{:?}\"}}",
        model.name(),
        inputs.len(),
        10.0 * (sig / err.max(1e-300)).log10(),
        first_bad
    );
    Ok(())
}

fn arg_str<'a>(args: &'a [String], flag: &str, d: &'a str) -> &'a str {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map_or(d, String::as_str)
}

fn main() -> Result<(), Err> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or(&[]);
    match args.first().map(String::as_str) {
        Some("gtcrn-bench") => bench(ModelId::Gtcrn, rest),
        Some("fem-bench") => bench(ModelId::Fem, rest),
        Some("gtcrn-parity") => {
            // 帧数 = 音频长度 / hop（不循环），与旧版一致
            let (wp, onnx, wav) = (Path::new(&rest[0]), Path::new(&rest[1]), Path::new(&rest[2]));
            let vs = arg_str(rest, "--vs", "native");
            let audio = read_wav(wav)?;
            let inputs = frames_for(ModelId::Gtcrn, &audio, audio.len() / HOP, onnx)?;
            let b = open_net(ModelId::Gtcrn, vs, if vs == "native" { wp } else { onnx })?;
            parity(ModelId::Gtcrn, vs, open_net(ModelId::Gtcrn, "ort", onnx)?, b, &inputs)
        }
        Some("fem-parity") => {
            let (onnx, wav) = (Path::new(&rest[0]), Path::new(&rest[1]));
            let audio = read_wav(wav)?;
            let hop = *input_shapes(&ort_session(onnx)?)?[0].last().ok_or("wav_in 形状空")?;
            let inputs = frames_for(ModelId::Fem, &audio, audio.len() / hop, onnx)?;
            parity(ModelId::Fem, "tract", open_net(ModelId::Fem, "ort", onnx)?, open_net(ModelId::Fem, "tract", onnx)?, &inputs)
        }
        _ => Err("用法见 src/main.rs 顶部".into()),
    }
}
