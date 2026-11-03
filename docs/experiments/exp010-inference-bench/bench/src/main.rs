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

enum Backend {
    Ort(Box<OrtGtcrn>),
    Native(Box<GtcrnNative>),
}
impl Backend {
    fn open(kind: &str, path: &Path) -> Result<Self, Err> {
        Ok(match kind {
            "ort" => Backend::Ort(Box::new(OrtGtcrn::open(path)?)),
            "native" => Backend::Native(Box::new(GtcrnNative::load(path)?)),
            _ => return Err(format!("未知后端 {kind}").into()),
        })
    }
    #[inline]
    fn infer(&mut self, re: &mut [f32], im: &mut [f32]) -> Result<(), Err> {
        match self {
            Backend::Ort(m) => m.infer(re, im),
            Backend::Native(m) => {
                m.infer(re, im);
                Ok(())
            }
        }
    }
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

fn bench(args: &[String]) -> Result<(), Err> {
    let (kind, model, wav) = (&args[0], Path::new(&args[1]), Path::new(&args[2]));
    let frames = arg_usize(args, "--frames", 6000);
    let warm = arg_usize(args, "--warmup", 200);
    let audio = read_wav(wav)?;
    let specs = spectra(&audio, frames + warm + 1);
    // 冷启动：打开 + 第一帧
    let t0 = Instant::now();
    let mut m = Backend::open(kind, model)?;
    let t_open = t0.elapsed().as_secs_f64() * 1e3;
    let (mut re, mut im) = specs[0];
    let t1 = Instant::now();
    m.infer(&mut re, &mut im)?;
    let t_first = t1.elapsed().as_secs_f64() * 1e3;
    for (r, i) in &specs[1..=warm] {
        let (mut re, mut im) = (*r, *i);
        m.infer(&mut re, &mut im)?;
    }
    let mut times = Vec::with_capacity(frames);
    let mut nan = 0usize;
    let a0 = ALLOCS.load(Relaxed);
    let c0 = cpu_seconds();
    let w0 = Instant::now();
    for (r, i) in &specs[warm + 1..] {
        let (mut re, mut im) = (*r, *i);
        let t = Instant::now();
        m.infer(&mut re, &mut im)?;
        times.push(t.elapsed().as_secs_f64() * 1e3);
        if !re.iter().chain(im.iter()).all(|v| v.is_finite()) {
            nan += 1;
        }
    }
    let wall = w0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    let allocs = ALLOCS.load(Relaxed) - a0;
    times.sort_by(|a, b| a.total_cmp(b));
    println!(
        "{{\"model\":\"gtcrn\",\"backend\":\"{kind}\",\"frames\":{frames},\"p50_ms\":{:.4},\"p99_ms\":{:.4},\"p999_ms\":{:.4},\"max_ms\":{:.4},\"mean_ms\":{:.4},\"open_ms\":{t_open:.2},\"first_frame_ms\":{t_first:.3},\"cpu_over_wall\":{:.3},\"rust_allocs_per_frame\":{:.3},\"nonfinite_frames\":{nan}}}",
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

fn parity(args: &[String]) -> Result<(), Err> {
    let (wp, onnx, wav) = (Path::new(&args[0]), Path::new(&args[1]), Path::new(&args[2]));
    let audio = read_wav(wav)?;
    let n = audio.len() / HOP;
    let specs = spectra(&audio, n);
    let mut a = OrtGtcrn::open(onnx)?;
    let mut b = GtcrnNative::load(wp)?;
    let (mut max_abs, mut sig, mut err) = (0.0f64, 0.0f64, 0.0f64);
    let mut worst_frame = 0;
    let mut first_bad = None;
    for (f, (r, i)) in specs.iter().enumerate() {
        let (mut ra, mut ia) = (*r, *i);
        let (mut rb, mut ib) = (*r, *i);
        a.infer(&mut ra, &mut ia)?;
        b.infer(&mut rb, &mut ib);
        for k in 0..BINS {
            for (x, y) in [(ra[k], rb[k]), (ia[k], ib[k])] {
                let d = (x as f64 - y as f64).abs();
                sig += (x as f64).powi(2);
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
    }
    println!(
        "{{\"model\":\"gtcrn\",\"frames\":{n},\"max_abs\":{max_abs:.3e},\"worst_frame\":{worst_frame},\"snr_db\":{:.2},\"first_bad\":\"{:?}\"}}",
        10.0 * (sig / err.max(1e-300)).log10(),
        first_bad
    );
    Ok(())
}

fn main() -> Result<(), Err> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("gtcrn-bench") => bench(&args[1..]),
        Some("gtcrn-parity") => parity(&args[1..]),
        _ => Err("用法见 src/main.rs 顶部".into()),
    }
}
