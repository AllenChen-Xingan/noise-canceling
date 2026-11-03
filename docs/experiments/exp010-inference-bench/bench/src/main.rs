//! exp010 推理后端基准：同口径比 ORT CPU（基线）、手写 Rust、tract（纯 Rust ONNX 推理）。
//!
//! 子命令：
//!   gtcrn-parity <权重前缀> <onnx> <wav> [--vs native|tract] [--frames N] [--reset-at K]
//!                逐帧对拍增强后频谱（ORT 对 --vs，默认 native；两边吃同一份 STFT 频谱）；--frames 循环音频跑长；
//!                --reset-at：被测一侧跑 K 帧后 reset，对比新开实例（只 native 实现了 reset）
//!   gtcrn-bench  <ort|native|tract> <权重前缀|onnx> <wav> [--frames N] [--warmup N]
//!   fem-parity   <onnx> <wav> [--vs tract|ort] [--b-onnx <被测侧的 onnx>] [--frames N]
//!                FastEnhancer-M：ORT 跑 <onnx> 对 --vs 跑 --b-onnx（默认同一份），逐帧对拍输出波形
//!                （--vs ort --b-onnx 改写图 = 单独证明改写等价；旧参数 --tract-onnx 等同 --b-onnx）
//!   fem-bench    <ort|tract> <onnx> <wav> [--frames N] [--warmup N]
//!   常驻内存：footprint_before_mb（加载前）/ footprint_warm_mb（预热完）/ footprint_end_mb（跑完），读 ri_phys_footprint。
//!   两个 bench 都可加 `--count-malloc`：挂 malloc_logger 数全进程 malloc（process_allocs_per_frame；不加时为 null）。
//!   这个钩子会拖慢分配，所以计数和计时分开跑。
//!   两个 bench 都可加 `--qos ui|rt|default`：计时线程的调度等级，默认 ui（QOS_CLASS_USER_INTERACTIVE）；
//!   rt = THREAD_TIME_CONSTRAINT_POLICY（period = 帧长、computation 1 ms、constraint 3 ms）。
//!   `--workgroup`：计时线程加入默认输出设备的 IO 工作组（字段 workgroup:true/false）。
//!   frames_over_3ms：计时帧里纯推理 > 3.0 ms 的个数。实时节奏下另报 late_*（醒来迟到）、release_*（计划时刻到完成，
//!   含迟到）与 release_over_3ms——BE-DEADLINE 应看 release_over_3ms（产品里帧是否赶上截止）。
//!   open_to_warm_ms：从开始打开到预热完（BE-COLD 直接量）。`--ready-file <路径>`：预热完写这个文件（耗电脚本用）。
//!   对拍输出 be_parity_pass；不过时退出码非零（--allow-fail 例外，给已知错误的对照用）；非有限指标写 null。
//!   两个 bench 都可加 `--pace-ms <毫秒>`：按实时节奏每隔这么久喂一帧（测耗电、测睡醒后的帧耗时用；默认 0 = 连续跑）。
//!   两个 bench 都可加 `--dump-io <前缀>`（此时第 2 个参数须是 onnx）：计时之后另起一个 ORT 会话，从零状态把
//!   全部 1 + warmup + frames 帧再跑一遍，写 `<前缀>.in.f32` / `<前缀>.ort.f32`（小端 f32，按帧拼接；
//!   GTCRN 每帧 re[257] 再 im[257]，FE-M 每帧 160 个样本）和 `<前缀>.json`（帧数、每帧长度、wav 路径与 sha256）。
//!   给非 Rust 的测量程序（applebe 的 Swift / Python）吃同一份输入、对同一份 ORT 输出。
//!   gtcrn-layers <权重前缀> <wav> [--frames N] [--warmup N] [--qos rt|ui|bg] [--pace-ms X] [--variant native|native-fast]
//!                研究 020：手写版逐层计时（cntvct 打点）+ 每层工作量（乘加、非线性、字节、关键路径），见 src/layers.rs
//!   hw-peak [--qos rt|ui|bg] [--reps N]  研究 020：本机 NEON 乘加吞吐 / 延迟、非线性延迟、各级缓存读写带宽，见 src/hwpeak.rs
//!   dsp-bench <wav16k> [--frames N] [--qos rt|ui|bg] [--pace-ms X]
//!                研究 020：前后处理（vdn-dsp 的 48↔16 kHz 重采样、STFT / iSTFT）每帧计时，对照「只改点积求和方式」的 NEON 版，见 src/dsp.rs
//!   gtcrn-bench 的后端另有 native-v3（研究 020 第二批，按能效核写，见 src/gtcrn_native/v3.rs）；后缀 -pf = 每帧预取权重；以及 native-v2 / native-v2-fast（研究 020：同一结构与权重，算子换 NEON 重写，见 src/gtcrn_native/v2.rs）
//!   amx-probe                                                   小矩阵上 Accelerate（AMX / SME）对手写循环，形状取自 GTCRN
//! 输出一行 JSON（stdout）。一帧 = 模型的一个 hop（GTCRN 256 样本 / 16 ms，FE-M 160 样本 / 10 ms）。

mod amx_probe;
mod dsp;
mod gtcrn_native;
mod hwpeak;
mod layers;
mod tract_be;
mod workgroup;

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

use gtcrn_native::GtcrnNative;
use ort::session::builder::{AutoDevicePolicy, GraphOptimizationLevel};
use ort::session::{HasSelectedOutputs, OutputSelector, RunOptions, Session};
use ort::value::Tensor;
use tract_be::TractNet;
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
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static A: Counting = Counting;

// ── 全进程 malloc 计数（含 C / C++ / 系统框架）：挂 libmalloc 的 malloc_logger 钩子 ──
// 只在 `--count-malloc` 时挂上；钩子会让每次 malloc 多走一次回调，计时会略偏慢，所以和计时分开跑。
type MallocLogger = unsafe extern "C" fn(u32, usize, usize, usize, usize, u32);
unsafe extern "C" {
    static mut malloc_logger: Option<MallocLogger>;
}
static PROC_ALLOCS: AtomicU64 = AtomicU64::new(0);
const MALLOC_LOG_TYPE_ALLOCATE: u32 = 2;
// 调用栈采样：STACK_ARMED 为真时，把接下来最多 STACK_N 次分配的返回地址存进静态表（backtrace 不分配；
// 用 IN_HOOK 防重入）。只给 amx-probe 查「Accelerate 在哪个调用里分配」用。
pub(crate) static STACK_ARMED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static IN_HOOK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub(crate) const STACK_N: usize = 4;
pub(crate) const STACK_DEPTH: usize = 16;
pub(crate) static mut STACKS: [[usize; STACK_DEPTH]; STACK_N] = [[0; STACK_DEPTH]; STACK_N];
pub(crate) static STACK_COUNT: AtomicU64 = AtomicU64::new(0);
unsafe extern "C" {
    fn backtrace(buf: *mut *mut libc::c_void, size: libc::c_int) -> libc::c_int;
}
unsafe extern "C" fn count_malloc(ty: u32, _: usize, _: usize, _: usize, _: usize, _: u32) {
    if ty & MALLOC_LOG_TYPE_ALLOCATE != 0 {
        PROC_ALLOCS.fetch_add(1, Relaxed);
        if STACK_ARMED.load(Relaxed) && !IN_HOOK.swap(true, Relaxed) {
            let i = STACK_COUNT.load(Relaxed) as usize;
            if i < STACK_N {
                unsafe {
                    let slot = &mut (*std::ptr::addr_of_mut!(STACKS))[i];
                    backtrace(slot.as_mut_ptr().cast(), STACK_DEPTH as libc::c_int);
                }
                STACK_COUNT.fetch_add(1, Relaxed);
            }
            IN_HOOK.store(false, Relaxed);
        }
    }
}
pub(crate) fn proc_allocs() -> u64 {
    PROC_ALLOCS.load(Relaxed)
}
pub(crate) fn install_malloc_counter() {
    unsafe { malloc_logger = Some(count_malloc) };
}

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
            r.samples::<i32>()
                .map(|v| v.map(|v| v as f32 / m))
                .collect::<Result<_, _>>()?
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

/// GTCRN 的手抄形状（mix + CACHE_SHAPES）必须和 ONNX 文件读出的一致，不一致就报错
fn check_gtcrn_shapes(onnx: &[Vec<usize>]) -> Result<(), Err> {
    let mut want = vec![vec![1, BINS, 1, 2]];
    want.extend(CACHE_SHAPES.iter().map(|s| s.to_vec()));
    if onnx != want.as_slice() {
        return Err(format!("GTCRN 输入形状和 ONNX 不一致：手抄 {want:?}，ONNX {onnx:?}").into());
    }
    Ok(())
}

fn zeros(shape: &[usize]) -> Result<Tensor<f32>, Err> {
    Ok(Tensor::from_array((
        shape.to_vec(),
        vec![0.0f32; shape.iter().product()],
    ))?)
}

impl OrtGtcrn {
    fn open(path: &Path) -> Result<Self, Err> {
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

    fn infer(&mut self, re: &mut [f32], im: &mut [f32]) -> Result<(), Err> {
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
trait Net {
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
struct TractGtcrn {
    net: TractNet,
    out: Vec<f32>,
}
impl TractGtcrn {
    fn open(path: &Path) -> Result<Self, Err> {
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

struct TractFem {
    net: TractNet,
    out: Vec<f32>,
}
impl TractFem {
    /// `shapes` 由调用方在计时前读好（不把 ORT 建会话算进 tract 的打开时间）。
    fn open(path: &Path, shapes: &[Vec<usize>]) -> Result<Self, Err> {
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

/// 计时前的准备：初始化 ORT 环境，并用一个 ORT 会话读出输入形状（会话随即丢掉）。
/// 对所有后端、两个模型都做，这样「打开」一律不含 ORT 环境初始化，模型文件也都已在页缓存里。
fn prepare(path_onnx: &Path) -> Result<(Vec<Vec<usize>>, f64), Err> {
    let t = Instant::now();
    let s = ort_session(path_onnx)?;
    let ms = t.elapsed().as_secs_f64() * 1e3;
    Ok((input_shapes(&s)?, ms))
}

fn open_net(model: ModelId, kind: &str, path: &Path) -> Result<Box<dyn Net>, Err> {
    open_net_with(model, kind, path, None)
}

fn open_net_with(
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
            k @ ("native-v2" | "native-v2-fast" | "native-v2-pf" | "native-v2-fast-pf" | "native-v3" | "native-v3-pf" | "native-v3i" | "native-v3i-pf"),
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
fn frames_for(
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

/// 常驻内存（MB）：`ri_phys_footprint`，即活动监视器「内存」列（与 applebe 的 BE-MEM 同口径）。
fn footprint_mb() -> f64 {
    let mut u: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
    let r = unsafe {
        libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            (&mut u as *mut libc::rusage_info_v4).cast(),
        )
    };
    if r != 0 {
        f64::NAN
    } else {
        u.ri_phys_footprint as f64 / (1024.0 * 1024.0)
    }
}

/// 墙钟 Unix 秒（耗电脚本按计时窗截取 powermetrics 样本用；applebe 的 power_parse.py v2 口径）
fn unix_now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

/// 节奏等待：第 n 帧的计划时刻 = w0 + n·pace；还没到就睡到那一刻，已过就不睡。返回计划时刻。
pub(crate) fn wait_due(w0: Instant, n: usize, pace_ms: f64) -> Instant {
    let due = w0 + std::time::Duration::from_secs_f64(n as f64 * pace_ms * 1e-3);
    if let Some(d) = due.checked_duration_since(Instant::now()) {
        std::thread::sleep(d);
    }
    due
}

/// 系统睡眠检测（exp012 20261102-v2 踩坑：拔电后空闲睡眠 42 分钟，单调时钟停走，进程看着像卡在 sleep 里）。
/// macOS 的单调时钟（Instant）在系统睡眠时不走，墙钟走；两者之差 > 1 秒就判这一轮作废。
pub(crate) struct SleepGuard {
    unix0: f64,
    mono0: Instant,
}
impl SleepGuard {
    pub(crate) fn start() -> Self {
        Self {
            unix0: unix_now(),
            mono0: Instant::now(),
        }
    }
    pub(crate) fn gap_s(&self) -> f64 {
        (unix_now() - self.unix0) - self.mono0.elapsed().as_secs_f64()
    }
    pub(crate) fn check(&self) -> Result<(), Err> {
        let g = self.gap_s();
        if g > 1.0 {
            return Err(format!("计时期间系统睡眠了约 {g:.1} 秒（墙钟比单调时钟多走），本轮作废").into());
        }
        Ok(())
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
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(d)
}

/// 计时线程的调度等级：ui = QOS_CLASS_USER_INTERACTIVE（默认，尽量留在性能核；实时音频线程只会更高），
/// rt = THREAD_TIME_CONSTRAINT_POLICY（period = 帧长、computation 1 ms、constraint 3 ms，可抢占），
/// bg = QOS_CLASS_BACKGROUND（系统倾向放到能效核，研究 020 量能效核用），default = 不设。
/// 审稿发现不设时 tract 的 p50 在一次运行里分两档。
pub(crate) fn set_thread_qos(qos: &str, frame_ms: f64) -> Result<(), Err> {
    if qos == "rt" {
        let mut tb = libc::mach_timebase_info { numer: 0, denom: 0 };
        unsafe { libc::mach_timebase_info(&mut tb) };
        let ms = |v: f64| (v * 1e6 * tb.denom as f64 / tb.numer as f64) as u32;
        let mut pol = libc::thread_time_constraint_policy {
            period: ms(frame_ms),
            computation: ms(1.0),
            constraint: ms(3.0),
            preemptible: 1,
        };
        let r = unsafe {
            libc::thread_policy_set(
                libc::pthread_mach_thread_np(libc::pthread_self()),
                libc::THREAD_TIME_CONSTRAINT_POLICY as _,
                (&mut pol as *mut libc::thread_time_constraint_policy).cast(),
                libc::THREAD_TIME_CONSTRAINT_POLICY_COUNT,
            )
        };
        if r != 0 {
            return Err(format!("设实时线程策略失败：{r}").into());
        }
    } else if qos == "ui" || qos == "bg" {
        let class = if qos == "ui" {
            libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE
        } else {
            libc::qos_class_t::QOS_CLASS_BACKGROUND
        };
        let r = unsafe { libc::pthread_set_qos_class_self_np(class, 0) };
        if r != 0 {
            return Err(format!("设 QoS 失败：{r}").into());
        }
    } else if qos != "default" {
        return Err("--qos 只收 ui / rt / bg / default".into());
    }
    Ok(())
}

fn bench(model: ModelId, args: &[String]) -> Result<(), Err> {
    let (kind, path, wav) = (&args[0], Path::new(&args[1]), Path::new(&args[2]));
    let frames = arg_usize(args, "--frames", 6000);
    let warm = arg_usize(args, "--warmup", 200);
    let pace: f64 = arg_str(args, "--pace-ms", "0").parse()?;
    // 入口校验（codex 第 5 轮 #1 #2）：计时帧数 > 0；节奏必须有限、非负，且不超过 10 秒一帧
    if frames == 0 {
        return Err("--frames 必须 > 0".into());
    }
    let total = frames
        .checked_add(warm)
        .and_then(|v| v.checked_add(1))
        .ok_or("--frames + --warmup 溢出")?;
    if !pace.is_finite() || !(0.0..=10_000.0).contains(&pace) {
        return Err(format!("--pace-ms {pace} 无效：要有限、非负、≤ 10000").into());
    }
    let qos = arg_str(args, "--qos", "ui");
    set_thread_qos(qos, if model == ModelId::Gtcrn { 16.0 } else { 10.0 })?;
    // 加入默认输出设备的 IO 工作组（在调度等级之后、计时之前；令牌留到函数结束）
    let wg_on = args.iter().any(|a| a == "--workgroup");
    let _wg_token = if wg_on {
        Some(workgroup::join_output()?)
    } else {
        None
    };
    let count_malloc = args.iter().any(|a| a == "--count-malloc");
    if count_malloc {
        install_malloc_counter();
    }
    let audio = read_wav(wav)?;
    // 计时前：ORT 环境初始化 + 读输入形状。native 吃的是权重前缀、不碰 ORT，跳过（它的「打开」本来就不含 ORT）
    let (shapes, ort_env_ms) = if kind.starts_with("native") {
        (Vec::new(), 0.0)
    } else {
        prepare(path)?
    };
    let inputs = frames_for(model, &audio, total, &shapes)?;
    let mut x = inputs[0].clone();
    let fp_before = footprint_mb(); // 加载前（输入帧、ORT 环境已在内存里）
    // 冷启动：打开 + 第一帧
    let t0 = Instant::now();
    let mut m = open_net_with(model, kind, path, Some(&shapes))?;
    let t_open = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = Instant::now();
    m.infer(&mut x)?;
    let t_first = t1.elapsed().as_secs_f64() * 1e3;
    for f in &inputs[1..=warm] {
        x.copy_from_slice(f);
        m.infer(&mut x)?;
    }
    // BE-COLD 直接量：从开始打开到预热完（codex 第 1 轮 #6）
    let open_to_warm = t0.elapsed().as_secs_f64() * 1e3;
    let fp_warm = footprint_mb();
    // 预热完发就绪信号（耗电脚本据此只采稳态窗口，codex 第 4 轮 #4）
    if let Some(i) = args.iter().position(|a| a == "--ready-file") {
        std::fs::write(args.get(i + 1).ok_or("--ready-file 缺路径")?, b"ready\n")?;
    }
    let mut times = Vec::with_capacity(frames);
    let mut nan = 0usize;
    // 实时节奏下另记：醒来迟到 = 实际开始 − 计划时刻；释放到完成 = 完成 − 计划时刻（codex 第 1 轮 #4）
    // 所有测量缓冲都在计数起点之前分配好（codex 第 2 轮 #1）
    let mut late = Vec::with_capacity(if pace > 0.0 { frames } else { 0 });
    let mut release = Vec::with_capacity(if pace > 0.0 { frames } else { 0 });
    let a0 = ALLOCS.load(Relaxed);
    let pa0 = PROC_ALLOCS.load(Relaxed);
    let c0 = cpu_seconds();
    let win_start = unix_now();
    let slept = SleepGuard::start();
    let w0 = Instant::now();
    for (n, f) in inputs[warm + 1..].iter().enumerate() {
        // 实时节奏：第 n 帧在 w0 + n·pace 时刻开始（睡眠不计入帧耗时）；连续跑时计划时刻 = w0，不睡
        let due = wait_due(w0, n, pace);
        x.copy_from_slice(f);
        let t = Instant::now();
        m.infer(&mut x)?;
        times.push(t.elapsed().as_secs_f64() * 1e3);
        if pace > 0.0 {
            late.push(t.saturating_duration_since(due).as_secs_f64() * 1e3);
            release.push(Instant::now().saturating_duration_since(due).as_secs_f64() * 1e3);
        }
        if !x.iter().all(|v| v.is_finite()) {
            nan += 1;
        }
    }
    // 循环一结束就截取两种分配计数，之后的排序、格式化不算进去（codex 第 2 轮 #1）
    let allocs = ALLOCS.load(Relaxed) - a0;
    let proc_allocs = PROC_ALLOCS.load(Relaxed) - pa0;
    let wall = w0.elapsed().as_secs_f64();
    let win_end = unix_now();
    let cpu = cpu_seconds() - c0;
    let over_3ms = times.iter().filter(|t| **t > 3.0).count(); // 纯推理 > 3 ms 的帧（同 applebe 字段）
    // 释放到完成 > 3 ms 的帧（含醒来迟到），只在实时节奏下有意义；不按节奏跑时为 null
    let release_over_3ms = if pace > 0.0 {
        release.iter().filter(|t| **t > 3.0).count().to_string()
    } else {
        "null".into()
    };
    late.sort_by(|a, b| a.total_cmp(b));
    release.sort_by(|a, b| a.total_cmp(b));
    let q = |v: &Vec<f64>, p: f64| {
        if v.is_empty() {
            "null".to_string()
        } else {
            format!("{:.4}", pct(v, p))
        }
    };
    let pace_extra = format!(
        "\"open_to_warm_ms\":{open_to_warm:.2},\"late_p50_ms\":{},\"late_p999_ms\":{},\"late_max_ms\":{},\"release_p999_ms\":{},\"release_max_ms\":{},\"release_over_3ms\":{release_over_3ms},",
        q(&late, 0.5),
        q(&late, 0.999),
        q(&late, 1.0),
        q(&release, 0.999),
        q(&release, 1.0)
    );
    let fp_end = footprint_mb();
    times.sort_by(|a, b| a.total_cmp(b));
    if let Some(i) = args.iter().position(|a| a == "--dump-io") {
        dump_io(
            model,
            path,
            wav,
            &inputs,
            args.get(i + 1).ok_or("--dump-io 缺前缀")?,
        )?;
    }
    println!(
        "{{\"model\":\"{}\",\"backend\":\"{kind}\",\"frames\":{frames},\"pace_ms\":{pace},\"qos\":\"{qos}\",\"workgroup\":{wg_on},\"p10_ms\":{:.4},\"p50_ms\":{:.4},\"p90_ms\":{:.4},\"p99_ms\":{:.4},\"p999_ms\":{:.4},\"max_ms\":{:.4},\"mean_ms\":{:.4},\"ort_env_ms\":{ort_env_ms:.2},\"open_ms\":{t_open:.2},\"first_frame_ms\":{t_first:.3},\"cpu_over_wall\":{:.3},\"rust_allocs_per_frame\":{:.3},\"rust_allocs_total\":{allocs},\"process_allocs_total\":{},\"nonfinite_frames\":{nan},\"frames_over_3ms\":{over_3ms},{pace_extra}\"process_allocs_per_frame\":{},\"footprint_before_mb\":{fp_before:.2},\"footprint_warm_mb\":{fp_warm:.2},\"footprint_end_mb\":{fp_end:.2},\"win_start_unix\":{win_start:.3},\"win_end_unix\":{win_end:.3},\"wall_s\":{wall:.3}}}",
        model.name(),
        pct(&times, 0.1),
        pct(&times, 0.5),
        pct(&times, 0.9),
        pct(&times, 0.99),
        pct(&times, 0.999),
        times[times.len() - 1],
        times.iter().sum::<f64>() / times.len() as f64,
        cpu / wall,
        allocs as f64 / frames as f64,
        if count_malloc {
            proc_allocs.to_string()
        } else {
            "null".into()
        },
        if count_malloc {
            format!("{:.3}", proc_allocs as f64 / frames as f64)
        } else {
            "null".into()
        },
    );
    // 有非有限输出：结果已打印（nonfinite_frames 字段可见），但这一轮性能作废，返回错误让脚本停下（codex 第 3 轮 #5）
    if nan > 0 {
        return Err(format!("{nan} 帧输出含非有限值，本轮性能结果无效").into());
    }
    slept.check()
}

/// 逐帧对拍：参考 `a`（ORT）对 `b`，同一份输入，比整帧输出。
/// 逐帧对拍：参考 `a`（ORT）对 `b`，同一份输入，比整帧输出。
/// `first_bad`：第一个「差 > 1e-3 或任一边非有限」的位置（NaN 也算坏，审稿 P2-1）。
/// `min_frame_snr_db`：逐帧 SNR 的最小值，只统计参考能量 ≥ 全段平均帧能量 1e-4 的帧，专看安静段（整段 SNR 被响帧主导）。
/// `reset_at`：若给出，在第 k 帧前对 `b` 调 reset，再拿一个新开的 `b2` 从第 k 帧起同喂，报两者最大差（应为 0）。
fn parity(
    model: ModelId,
    vs: &str,
    mut a: Box<dyn Net>,
    mut b: Box<dyn Net>,
    inputs: &[Vec<f32>],
) -> Result<(), Err> {
    if inputs.is_empty() {
        return Err("对拍输入为空（音频短于一个 hop，或 --frames 0）".into());
    }
    let (mut max_abs, mut sig, mut err) = (0.0f64, 0.0f64, 0.0f64);
    let mut worst_frame = 0;
    let mut first_bad = None;
    let mut nonfinite = 0usize;
    let mut per_frame = Vec::with_capacity(inputs.len()); // (ref 能量, 差能量)
    let mut xa = inputs[0].clone();
    let mut xb = inputs[0].clone();
    for (f, x) in inputs.iter().enumerate() {
        xa.copy_from_slice(x);
        xb.copy_from_slice(x);
        a.infer(&mut xa)?;
        b.infer(&mut xb)?;
        let (mut fs, mut fe) = (0.0f64, 0.0f64);
        for (k, (&p, &q)) in xa.iter().zip(&xb).enumerate() {
            let bad_val = !p.is_finite() || !q.is_finite();
            if bad_val {
                nonfinite += 1;
            }
            let d = (p as f64 - q as f64).abs();
            fs += (p as f64).powi(2);
            fe += d * d;
            if d > max_abs {
                max_abs = d;
                worst_frame = f;
            }
            if (bad_val || !(d <= 1e-3)) && first_bad.is_none() {
                first_bad = Some((f, k));
            }
        }
        sig += fs;
        err += fe;
        per_frame.push((fs, fe));
    }
    let mean_e = sig / inputs.len() as f64;
    let min_frame_snr = per_frame
        .iter()
        .filter(|(s, _)| *s >= 1e-4 * mean_e)
        .map(|(s, e)| 10.0 * (s / e.max(1e-300)).log10())
        .fold(f64::INFINITY, f64::min);
    // 参考全零（全静音）时 SNR 没有定义：两边都为零就算通过，SNR 写 null（codex 第 4 轮 #1）
    let zero_ref = sig == 0.0;
    let snr = if zero_ref {
        f64::NAN
    } else {
        10.0 * (sig / err.max(1e-300)).log10()
    };
    // BE-PARITY（standard.md §2）：没有值差 > 1e-3、没有非有限值，且整段 SNR ≥ 60 dB（参考全零时改判「差全为 0」）
    let pass = first_bad.is_none()
        && nonfinite == 0
        && if zero_ref {
            err == 0.0
        } else {
            snr.is_finite() && snr >= 60.0
        };
    println!(
        "{{\"model\":\"{}\",\"vs\":\"{vs}\",\"frames\":{},\"max_abs\":{},\"worst_frame\":{worst_frame},\"snr_db\":{},\"min_frame_snr_db\":{},\"nonfinite_values\":{nonfinite},\"first_bad\":\"{:?}\",\"zero_reference\":{zero_ref},\"be_parity_pass\":{pass}}}",
        model.name(),
        inputs.len(),
        if max_abs.is_finite() {
            format!("{max_abs:.3e}")
        } else {
            "null".into()
        },
        jnum(snr, 2),
        jnum(min_frame_snr, 2),
        first_bad
    );
    // 不过就返回非零退出码，脚本里后续测速不会继续（codex 第 1 轮 #3）；已知错误的对照用 --allow-fail
    if !pass && !ALLOW_FAIL.load(Relaxed) {
        return Err(format!("{vs} 没过 BE-PARITY").into());
    }
    Ok(())
}
static ALLOW_FAIL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// reset 检查：`b` 先跑 `k` 帧再 reset，`b2` 新开；两者从第 k 帧起喂同样输入，比输出（应逐位相同）。
fn reset_check(
    model: ModelId,
    vs: &str,
    b: &mut dyn Net,
    b2: &mut dyn Net,
    inputs: &[Vec<f32>],
    k: usize,
) -> Result<(), Err> {
    // 先查区间，再切片（codex 第 3 轮 #3）
    if inputs.is_empty() || k == 0 || k >= inputs.len() {
        return Err(format!("--reset-at {k} 不在 1..{} 之内，比较区间为空", inputs.len()).into());
    }
    let mut x = inputs[0].clone();
    for f in &inputs[..k] {
        x.copy_from_slice(f);
        b.infer(&mut x)?;
    }
    b.reset();
    let mut y = inputs[0].clone();
    let mut nonfinite = 0usize;
    let mut max_abs = 0.0f64;
    let mut bit_diff = 0usize;
    for f in &inputs[k..] {
        x.copy_from_slice(f);
        y.copy_from_slice(f);
        b.infer(&mut x)?;
        b2.infer(&mut y)?;
        for (p, q) in x.iter().zip(&y) {
            if p.to_bits() != q.to_bits() {
                bit_diff += 1;
            }
            if !p.is_finite() || !q.is_finite() {
                nonfinite += 1; // 两边一样的 NaN 也不算过（codex 第 2 轮 #4）
            }
            let d = (*p as f64 - *q as f64).abs();
            max_abs = if d.is_nan() {
                f64::INFINITY
            } else {
                max_abs.max(d)
            };
        }
    }
    // 位模式比较（codex 第 1 轮抽查：差 0 不等于逐位相同）
    println!(
        "{{\"model\":\"{}\",\"vs\":\"{vs}\",\"check\":\"reset\",\"reset_at\":{k},\"frames_after\":{},\"max_abs_vs_fresh\":{},\"bitwise_diff_values\":{bit_diff},\"nonfinite_values\":{nonfinite}}}",
        model.name(),
        inputs.len() - k,
        if max_abs.is_finite() {
            format!("{max_abs:.3e}")
        } else {
            "null".into()
        },
    );
    if (bit_diff != 0 || nonfinite != 0) && !ALLOW_FAIL.load(Relaxed) {
        return Err("reset 后与新开实例不逐位相同，或出现非有限值".into());
    }
    Ok(())
}

fn dump_io(
    model: ModelId,
    onnx: &Path,
    wav: &Path,
    inputs: &[Vec<f32>],
    prefix: &str,
) -> Result<(), Err> {
    use std::io::Write;
    let mut net = open_net(model, "ort", onnx)?;
    let mut fi = std::io::BufWriter::new(std::fs::File::create(format!("{prefix}.in.f32"))?);
    let mut fo = std::io::BufWriter::new(std::fs::File::create(format!("{prefix}.ort.f32"))?);
    let mut x = inputs[0].clone();
    for f in inputs {
        x.copy_from_slice(f);
        for v in &x {
            fi.write_all(&v.to_le_bytes())?;
        }
        net.infer(&mut x)?;
        for v in &x {
            fo.write_all(&v.to_le_bytes())?;
        }
    }
    let sha = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(wav)
        .output()?;
    let sha = String::from_utf8_lossy(&sha.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    let onnx_sha = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(onnx)
        .output()?;
    let onnx_sha = String::from_utf8_lossy(&onnx_sha.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    std::fs::write(
        format!("{prefix}.json"),
        format!(
            "{{\"model\":\"{}\",\"frames_total\":{},\"frame_len\":{},\"layout\":\"{}\",\"note\":\"第 0 帧是冷启动帧，接着 warmup 帧，再接计时帧；ORT 从零状态顺序跑\",\"wav\":\"{}\",\"wav_sha256\":\"{sha}\",\"onnx\":\"{}\",\"onnx_sha256\":\"{onnx_sha}\"}}\n",
            model.name(),
            inputs.len(),
            inputs[0].len(),
            if model == ModelId::Gtcrn {
                "re[257] then im[257]"
            } else {
                "wav hop samples"
            },
            wav.display(),
            onnx.display()
        ),
    )?;
    Ok(())
}

/// JSON 数字：非有限值写 null（codex 第 1 轮 #5）
fn jnum(v: f64, prec: usize) -> String {
    if v.is_finite() {
        format!("{v:.prec$}")
    } else {
        "null".into()
    }
}

fn arg_str<'a>(args: &'a [String], flag: &str, d: &'a str) -> &'a str {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map_or(d, String::as_str)
}

fn main() -> Result<(), Err> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or(&[]);
    ALLOW_FAIL.store(rest.iter().any(|a| a == "--allow-fail"), Relaxed);
    match args.first().map(String::as_str) {
        Some("amx-probe") => {
            amx_probe::run();
            Ok(())
        }
        Some("gtcrn-bench") => bench(ModelId::Gtcrn, rest),
        Some("hw-peak") => hwpeak::run(rest),
        Some("dsp-bench") => {
            let audio = read_wav(Path::new(rest.first().ok_or("缺 wav")?))?;
            dsp::run(rest, &audio)
        }
        Some("gtcrn-layers") => {
            let audio = read_wav(Path::new(rest.get(1).ok_or("缺 wav")?))?;
            layers::run(rest, |n| frames_for(ModelId::Gtcrn, &audio, n, &[]))
        }
        Some("fem-bench") => bench(ModelId::Fem, rest),
        Some("gtcrn-parity") => {
            // 帧数默认 = 音频长度 / hop（不循环）；--frames N 时循环音频（STFT 不重置），查长时间跑会不会漂
            let (wp, onnx, wav) = (
                Path::new(&rest[0]),
                Path::new(&rest[1]),
                Path::new(&rest[2]),
            );
            let vs = arg_str(rest, "--vs", "native");
            let audio = read_wav(wav)?;
            let n = arg_usize(rest, "--frames", audio.len() / HOP);
            let inputs = frames_for(ModelId::Gtcrn, &audio, n, &[])?;
            let pb = if vs.starts_with("native") { wp } else { onnx };
            parity(
                ModelId::Gtcrn,
                vs,
                open_net(ModelId::Gtcrn, "ort", onnx)?,
                open_net(ModelId::Gtcrn, vs, pb)?,
                &inputs,
            )?;
            if let Some(i) = rest.iter().position(|a| a == "--reset-at") {
                let k: usize = rest.get(i + 1).ok_or("--reset-at 缺帧号")?.parse()?;
                let (mut b, mut b2) = (
                    open_net(ModelId::Gtcrn, vs, pb)?,
                    open_net(ModelId::Gtcrn, vs, pb)?,
                );
                reset_check(ModelId::Gtcrn, vs, b.as_mut(), b2.as_mut(), &inputs, k)?;
            }
            Ok(())
        }
        Some("fem-parity") => {
            // --vs tract（默认）| ort；--b-onnx 给被测一侧用的图（如改写图），默认同 <onnx>
            let (onnx, wav) = (Path::new(&rest[0]), Path::new(&rest[1]));
            let vs = arg_str(rest, "--vs", "tract");
            let bo = Path::new(arg_str(
                rest,
                "--b-onnx",
                arg_str(rest, "--tract-onnx", &rest[0]),
            ));
            let audio = read_wav(wav)?;
            let shapes = input_shapes(&ort_session(onnx)?)?;
            let hop = *shapes[0].last().ok_or("wav_in 形状空")?;
            let n = arg_usize(rest, "--frames", audio.len() / hop);
            let inputs = frames_for(ModelId::Fem, &audio, n, &shapes)?;
            let label = format!(
                "{vs}:{}",
                bo.file_name().and_then(|s| s.to_str()).unwrap_or("?")
            );
            parity(
                ModelId::Fem,
                &label,
                open_net(ModelId::Fem, "ort", onnx)?,
                open_net_with(ModelId::Fem, vs, bo, Some(&shapes))?,
                &inputs,
            )
        }
        _ => Err("用法见 src/main.rs 顶部".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 恒等或输出 NaN 的假模型，用来测对拍和 reset 检查本身
    struct Fake {
        nan: bool,
    }
    impl Net for Fake {
        fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
            if self.nan {
                x.fill(f32::NAN);
            }
            Ok(())
        }
    }

    /// 计时窗字段：applebe 的耗电解析（power_parse.py v2）与 apple-rs/tests/fields.rs 依赖这三个键（#58）
    #[test]
    fn bench_emits_window_fields() {
        let src = include_str!("main.rs");
        for k in ["win_start_unix", "win_end_unix", "wall_s"] {
            assert!(src.contains(&format!("\\\"{k}\\\":{{")), "bench 输出缺 {k}");
        }
        let t = unix_now();
        assert!(t > 1.7e9 && t < 1e10, "unix_now 应是 Unix 秒：{t}");
    }

    /// 节奏等待：按 2 ms 节奏跑 20 帧，总时长在 [38 ms, 20 × 2 × 1.2 + 10 ms] 之内（大总管要的回归测试）
    #[test]
    fn wait_due_keeps_pace() {
        let w0 = Instant::now();
        for n in 0..20 {
            wait_due(w0, n, 2.0);
        }
        let ms = w0.elapsed().as_secs_f64() * 1e3;
        assert!((38.0..=58.0).contains(&ms), "20 帧 × 2 ms 用了 {ms:.1} ms");
        // 连续跑（pace 0）不睡
        let t = Instant::now();
        for n in 0..1000 {
            wait_due(t, n, 0.0);
        }
        assert!(t.elapsed().as_millis() < 5);
    }

    #[test]
    fn sleep_guard_flags_wall_clock_jump() {
        let g = SleepGuard::start();
        assert!(g.check().is_ok() && g.gap_s().abs() < 0.5);
        let jumped = SleepGuard {
            unix0: g.unix0 - 5.0, // 假装墙钟多走了 5 秒
            mono0: g.mono0,
        };
        assert!(jumped.check().is_err());
    }

    #[test]
    fn jnum_and_pct() {
        assert_eq!(jnum(f64::NAN, 2), "null");
        assert_eq!(jnum(f64::NEG_INFINITY, 2), "null");
        assert_eq!(jnum(1.234, 2), "1.23");
        let v: Vec<f64> = (0..11).map(|i| i as f64).collect();
        assert_eq!((pct(&v, 0.0), pct(&v, 0.5), pct(&v, 1.0)), (0.0, 5.0, 10.0));
    }

    #[test]
    fn frames_for_rejects_empty_and_short() {
        let audio = vec![0.1f32; HOP * 3];
        assert!(frames_for(ModelId::Gtcrn, &audio, 0, &[]).is_err());
        assert!(frames_for(ModelId::Gtcrn, &audio[..HOP - 1], 5, &[]).is_err());
        assert_eq!(frames_for(ModelId::Gtcrn, &audio, 5, &[]).unwrap().len(), 5);
        assert!(
            frames_for(ModelId::Fem, &audio, 5, &[]).is_err(),
            "FE-M 没给形状要报错"
        );
    }

    #[test]
    fn parity_identical_passes_nan_fails_silence_passes() {
        let inputs: Vec<Vec<f32>> = (0..4).map(|i| vec![i as f32 + 1.0; 8]).collect();
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: false }),
                &inputs
            )
            .is_ok()
        );
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: true }),
                &inputs
            )
            .is_err()
        );
        let silent = vec![vec![0.0f32; 8]; 4];
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: false }),
                &silent
            )
            .is_ok(),
            "全静音且两边一致要通过"
        );
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: false }),
                &[]
            )
            .is_err()
        );
    }

    #[test]
    fn reset_check_bounds_and_nan() {
        let inputs: Vec<Vec<f32>> = (0..4).map(|i| vec![i as f32; 8]).collect();
        let (mut a, mut b) = (Fake { nan: false }, Fake { nan: false });
        assert!(reset_check(ModelId::Gtcrn, "t", &mut a, &mut b, &inputs, 2).is_ok());
        assert!(
            reset_check(ModelId::Gtcrn, "t", &mut a, &mut b, &inputs, 4).is_err(),
            "区间为空要报错"
        );
        assert!(reset_check(ModelId::Gtcrn, "t", &mut a, &mut b, &inputs, 0).is_err());
        let (mut c, mut d) = (Fake { nan: true }, Fake { nan: true });
        assert!(
            reset_check(ModelId::Gtcrn, "t", &mut c, &mut d, &inputs, 2).is_err(),
            "两边一样的 NaN 也不能过"
        );
    }

    #[test]
    fn gtcrn_shape_check() {
        let mut ok = vec![vec![1, BINS, 1, 2]];
        ok.extend(CACHE_SHAPES.iter().map(|s| s.to_vec()));
        assert!(check_gtcrn_shapes(&ok).is_ok());
        ok[1][0] = 3;
        assert!(check_gtcrn_shapes(&ok).is_err());
    }

    /// 端到端：手写版（含三个变体）对 ORT 逐值一致 + reset。需要仓外的权重与模型，CI 上没有，人工跑：
    ///   EXP010_WEIGHTS=<主目录>/_tmp/20261031-exp010/gtcrn_dns3 \
    ///   EXP010_GTCRN_ONNX=<主目录>/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx \
    ///   EXP010_WAV=<主目录>/resources/gtcrn/test_wavs/mix.wav \
    ///   cargo test --release -- --ignored --test-threads 1
    /// （--test-threads 1：native 变体靠全局开关切换，不能并行）
    #[test]
    #[ignore = "需要仓外的 GTCRN 权重、ONNX 和 wav，人工跑（见注释）"]
    fn native_variants_match_ort() {
        let v = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("要设 {k}"));
        let (w, onnx, wav) = (v("EXP010_WEIGHTS"), v("EXP010_GTCRN_ONNX"), v("EXP010_WAV"));
        let audio = read_wav(Path::new(&wav)).unwrap();
        let inputs = frames_for(ModelId::Gtcrn, &audio, audio.len() / HOP, &[]).unwrap();
        for k in ["native", "native-fast", "native-blas", "native-v2", "native-v2-fast", "native-v2-fast-pf", "native-v3", "native-v3-pf", "native-v3i"] {
            let ort = open_net(ModelId::Gtcrn, "ort", Path::new(&onnx)).unwrap();
            let nat = open_net(ModelId::Gtcrn, k, Path::new(&w)).unwrap();
            parity(ModelId::Gtcrn, k, ort, nat, &inputs).unwrap_or_else(|e| panic!("{k}: {e}"));
            let (mut a, mut b) = (
                open_net(ModelId::Gtcrn, k, Path::new(&w)).unwrap(),
                open_net(ModelId::Gtcrn, k, Path::new(&w)).unwrap(),
            );
            reset_check(
                ModelId::Gtcrn,
                k,
                a.as_mut(),
                b.as_mut(),
                &inputs,
                inputs.len() / 2,
            )
            .unwrap_or_else(|e| panic!("{k} reset: {e}"));
        }
        // 手抄形状与 ONNX 一致（OrtGtcrn::open 里也会查，这里单独断言一次）
        check_gtcrn_shapes(&input_shapes(&ort_session(Path::new(&onnx)).unwrap()).unwrap())
            .unwrap();
    }
}
