//! `vdn` 命令行（P1 离线原型）。
//!
//! ```text
//! vdn denoise in.wav out.wav [--model gtcrn|fastenhancer-m] [--floor-db -12] [--no-safety] …
//! vdn bench [--model …] --input mix.wav [--warmup 200] [--steps 6000] [--json out.json]
//! vdn devices                                         # 列设备（不开流）
//! vdn run --passthrough|--denoise [--seconds N]       # 单进程实时：麦克风 → Pipeline → 扬声器（开麦、出声）
//! vdn chain-test [--mode passthrough|denoise] [--kill-denoiser-at 2]   # 无设备链路测试，按判据给退出码
//! ```
//! denoise / bench 只读写 16 kHz 单声道 WAV、不碰音频设备。

#![forbid(unsafe_code)]

mod chain;
mod live;
mod verify;

use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;
use vdn_engine::{Engine, FaultInjector, FaultKind, Model, SafetyConfig, offline};
use vdn_models::{AnyModel, BoxError, ModelKind};

/// 计数分配器：bench 里量「每 hop 在 Rust 侧分配几次」（ORT C++ 内部的分配绕过它，量不到）。
#[global_allocator]
static GLOBAL: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

#[derive(Parser)]
#[command(name = "vdn", version, about = "Mac 开源实时语音降噪 · P1 离线原型")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// WAV → 逐帧流式降噪 + 三件保命 → WAV（32-bit float）
    Denoise(DenoiseArgs),
    /// 单线程每帧耗时（口径同 exp001：热身后计 N 步）
    Bench(BenchArgs),
    /// 列输入 / 输出设备与采样率（不开流、不弹权限框）
    Devices,
    /// 单进程实时开发模式：麦克风 → Pipeline → 扬声器（会开麦、出声）
    Run(RunArgs),
    /// 无设备链路测试：进程内三线程 + 真 POSIX 共享内存，按 V1 / V2 / V3 判据打印并给退出码
    ChainTest(ChainArgs),
    /// 按 V1 判据检查一个 48 kHz WAV（V4：两个真进程写出的文件）：应是测试正弦延迟 L_total
    ChainCheck(ChainCheckArgs),
}

#[derive(Args)]
struct ChainCheckArgs {
    wav: PathBuf,
    /// 期望的总延迟（48 kHz 样本；vdn-capture 启动时打印）
    #[arg(long)]
    l_total: usize,
    /// 从第几秒起判（重接之后那段）
    #[arg(long, default_value_t = 0.0, value_parser = vdn_control::parse_seconds)]
    from: f64,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct RunMode {
    /// 直通（testing::Identity，不需要模型权重）
    #[arg(long)]
    passthrough: bool,
    /// 降噪（真模型）
    #[arg(long)]
    denoise: bool,
}

#[derive(Args)]
struct RunArgs {
    #[command(flatten)]
    mode: RunMode,
    #[command(flatten)]
    m: ModelOpts,
    /// 跑多少秒；不给就一直跑到 Ctrl-C
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    seconds: Option<f64>,
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_BLOCK)]
    block: usize,
    /// 环余量（48 kHz 样本）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_LEAD)]
    lead: usize,
    #[arg(long)]
    input_device: Option<String>,
    #[arg(long)]
    output_device: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
enum ChainModeArg {
    Passthrough,
    Denoise,
}

#[derive(Args)]
struct ChainArgs {
    #[arg(long, value_enum, default_value = "passthrough")]
    mode: ChainModeArg,
    #[command(flatten)]
    m: ModelOpts,
    /// 第几秒让降噪线程退出（测 V2）
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    kill_denoiser_at: Option<f64>,
    #[arg(long, default_value_t = 5.0, value_parser = vdn_control::parse_seconds)]
    seconds: f64,
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_BLOCK)]
    block: usize,
    /// 环余量（48 kHz 样本）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_LEAD)]
    lead: usize,
    /// 降噪块长（48 kHz 样本，须整除模型 hop × 3）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_DENOISE_BLOCK)]
    denoise_block: usize,
    /// 共享内存名（默认 /vdn-chaintest）
    #[arg(long)]
    bus: Option<String>,
    /// 心跳超时（ms）
    #[arg(long, default_value_t = vdn_control::Config::DEFAULT_HEARTBEAT_MS)]
    heartbeat_ms: u32,
    /// 把输入 / 输出写成 WAV（48 kHz）方便人听：<前缀>-in.wav / <前缀>-out.wav
    #[arg(long)]
    dump: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum ModelArg {
    Gtcrn,
    #[value(name = "fastenhancer-m")]
    FastEnhancerM,
}

impl From<ModelArg> for ModelKind {
    fn from(m: ModelArg) -> Self {
        match m {
            ModelArg::Gtcrn => ModelKind::Gtcrn,
            ModelArg::FastEnhancerM => ModelKind::FastEnhancerM,
        }
    }
}

#[derive(Args)]
struct ModelOpts {
    /// 用哪个模型
    #[arg(long, value_enum, default_value = "gtcrn")]
    model: ModelArg,
    /// 模型文件；不给就按 环境变量 VDN_GTCRN_ONNX / VDN_FASTENHANCER_ONNX → ~/vdn-data/models/… → ./resources/… 查找
    #[arg(long)]
    model_path: Option<PathBuf>,
}

#[derive(Args)]
struct DenoiseArgs {
    input: PathBuf,
    output: PathBuf,
    #[command(flatten)]
    m: ModelOpts,
    /// 三件保命全关（输出 = 模型原样输出，对拍用）
    #[arg(long)]
    no_safety: bool,
    /// 关「出事送原声」
    #[arg(long)]
    no_fallback: bool,
    /// 关衰减下限（同时关快开慢关）
    #[arg(long)]
    no_floor: bool,
    /// 关快开慢关（只留固定的衰减下限）
    #[arg(long)]
    no_fast_open: bool,
    /// 衰减下限（dB）：停顿时背景最多降这么多
    #[arg(long, default_value_t = -12.0, allow_hyphen_values = true)]
    floor_db: f32,
    /// 有声时的原声混回（dB），须 ≥ floor-db
    #[arg(long, default_value_t = -6.0, allow_hyphen_values = true)]
    open_db: f32,
    /// 人声停后保持放开多久（ms）
    #[arg(long, default_value_t = 150.0)]
    hangover_ms: f32,
    /// 收回下限的时间常数（ms）
    #[arg(long, default_value_t = 300.0)]
    release_ms: f32,
    /// 模型每次 process 的截止时间（ms），超时那一帧送原声；默认不设
    #[arg(long)]
    deadline_ms: Option<f32>,
    /// 开头先喂多少个零样本（对拍 GTCRN 时用 256，与 exp003 Python 口径一致）
    #[arg(long, default_value_t = 0)]
    lead_in: usize,
    /// 故障注入（实验用）：逗号分隔的 `<hop>:<nan|error|explode|stall=<ms>>`
    #[arg(long)]
    inject: Option<String>,
    /// 把统计与每帧耗时写成 JSON
    #[arg(long)]
    json: Option<PathBuf>,
}

#[derive(Args)]
struct BenchArgs {
    #[command(flatten)]
    m: ModelOpts,
    /// 输入 WAV（16 kHz 单声道），不够长就循环
    #[arg(long)]
    input: PathBuf,
    #[arg(long, default_value_t = 200)]
    warmup: usize,
    #[arg(long, default_value_t = 6000)]
    steps: usize,
    /// 结果 JSON
    #[arg(long)]
    json: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    // flush-to-zero 暂不开：P1 要和 Python 逐值对拍，且 no_denormals 是 unsafe；留到 P3 实时线程入口（exp004 README「已知局限」）
    let res = match cli.cmd {
        Cmd::Denoise(a) => denoise(a),
        Cmd::Bench(a) => bench(a),
        Cmd::Devices => {
            live::list_devices();
            Ok(())
        }
        Cmd::Run(a) => run(a),
        Cmd::ChainCheck(a) => match chain_check(a) {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(e) => Err(e),
        },
        Cmd::ChainTest(a) => match chain_test(a) {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(e) => Err(e),
        },
    };
    if let Err(e) = res {
        eprintln!("vdn: {e}");
        std::process::exit(1);
    }
}

fn open_model(m: &ModelOpts) -> Result<(AnyModel, PathBuf), BoxError> {
    let kind = ModelKind::from(m.model);
    let path = match &m.model_path {
        Some(p) => p.clone(),
        None => kind
            .default_path()
            .ok_or("找不到模型文件：用 --model-path 指定，或设环境变量 VDN_GTCRN_ONNX / VDN_FASTENHANCER_ONNX")?,
    };
    Ok((kind.open(&path)?, path))
}

fn read_wav(path: &Path) -> Result<Vec<f32>, BoxError> {
    let mut r = hound::WavReader::open(path)?;
    let spec = r.spec();
    if spec.channels != 1 || spec.sample_rate != 16000 {
        return Err(format!(
            "{}：只收 16 kHz 单声道（这份是 {} Hz、{} 声道）",
            path.display(),
            spec.sample_rate,
            spec.channels
        )
        .into());
    }
    let x = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (spec.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|s| s.map(|v| v as f32 * scale))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    Ok(x)
}

fn write_wav(path: &Path, y: &[f32]) -> Result<(), BoxError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for &v in y {
        w.write_sample(v)?;
    }
    w.finalize()?;
    Ok(())
}

fn parse_inject(s: &str) -> Result<Vec<(u64, FaultKind)>, BoxError> {
    s.split(',')
        .filter(|t| !t.is_empty())
        .map(|t| {
            let (hop, kind) = t.split_once(':').ok_or("--inject 格式：<hop>:<kind>")?;
            let hop: u64 = hop.trim().parse()?;
            let kind = match kind.trim() {
                "nan" => FaultKind::Nan,
                "error" => FaultKind::Error,
                "explode" => FaultKind::Explode,
                k => match k.strip_prefix("stall=") {
                    Some(ms) => FaultKind::Stall(Duration::from_secs_f32(ms.parse::<f32>()? / 1e3)),
                    None => return Err(format!("未知故障：{k}").into()),
                },
            };
            Ok((hop, kind))
        })
        .collect()
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn percentiles(t: &[Duration]) -> serde_json::Value {
    let mut v: Vec<f64> = t.iter().map(|d| ms(*d)).collect();
    v.sort_by(|a, b| a.total_cmp(b));
    // 线性插值，同 numpy.percentile 默认
    let p = |q: f64| {
        if v.is_empty() {
            return f64::NAN;
        }
        let pos = q / 100.0 * (v.len() - 1) as f64;
        let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
        v[lo] + (v[hi] - v[lo]) * (pos - lo as f64)
    };
    json!({"n": v.len(), "p50": p(50.0), "p99": p(99.0), "p999": p(99.9), "max": v.last().copied().unwrap_or(f64::NAN)})
}

fn denoise(a: DenoiseArgs) -> Result<(), BoxError> {
    let x = read_wav(&a.input)?;
    let (model, path) = open_model(&a.m)?;
    let mut cfg = if a.no_safety {
        SafetyConfig::off()
    } else {
        SafetyConfig::default()
    };
    if !a.no_safety {
        cfg.fallback = !a.no_fallback;
        cfg.floor = !a.no_floor;
        cfg.fast_open = !a.no_fast_open && !a.no_floor;
        cfg.floor_db = a.floor_db;
        cfg.open_db = a.open_db;
        cfg.hangover_s = a.hangover_ms / 1e3;
        cfg.release_s = a.release_ms / 1e3;
        cfg.deadline = a.deadline_ms.map(|v| Duration::from_secs_f32(v / 1e3));
        if cfg.open_db < cfg.floor_db {
            return Err("--open-db 必须 ≥ --floor-db".into());
        }
    }
    let plan = match &a.inject {
        Some(s) => parse_inject(s)?,
        None => Vec::new(),
    };
    let model = FaultInjector::new(model, plan);
    let mut engine = Engine::new(model, cfg.clone());
    let run = offline::run(&mut engine, &x, a.lead_in);
    write_wav(&a.output, &run.output)?;
    let s = engine.stats();
    let nonfinite = run.output.iter().filter(|v| !v.is_finite()).count();
    eprintln!(
        "vdn: {} → {}（{}，{} 个 hop，送原声 {} hop，复位 {} 次，有声 {} hop，非有限样本 {}）",
        a.input.display(),
        a.output.display(),
        engine.model().name(),
        s.hops,
        s.bypass_hops,
        s.model_resets,
        s.voiced_hops,
        nonfinite
    );
    if let Some(j) = &a.json {
        let out = json!({
            "input": a.input, "output": a.output, "model": engine.model().name(), "model_path": path,
            "hop": engine.hop(), "latency_samples": engine.latency_samples(), "lead_in": a.lead_in,
            "safety": format!("{cfg:?}"),
            "stats": {
                "hops": s.hops, "fault_backend": s.fault_backend, "fault_nonfinite": s.fault_nonfinite,
                "fault_abnormal": s.fault_abnormal, "fault_deadline": s.fault_deadline,
                "model_resets": s.model_resets, "bypass_hops": s.bypass_hops, "voiced_hops": s.voiced_hops,
                "nonfinite_samples": nonfinite,
            },
            "model_ms": percentiles(&run.model_times),
            "engine_ms": percentiles(&run.engine_times),
        });
        std::fs::write(j, serde_json::to_string_pretty(&out)?)?;
    }
    Ok(())
}

fn bench(a: BenchArgs) -> Result<(), BoxError> {
    let x = read_wav(&a.input)?;
    if x.is_empty() {
        return Err("输入为空".into());
    }
    let (model, path) = open_model(&a.m)?;
    let mut raw = Engine::new(model, SafetyConfig::off());
    let hop = raw.hop();
    let need = (a.warmup + a.steps) * hop;
    let xs: Vec<f32> = x.iter().copied().cycle().take(need).collect();
    let mut y = vec![0.0f32; hop];
    let mut t_model = Vec::with_capacity(a.steps);
    let mut t_engine = Vec::with_capacity(a.steps);
    let mut nonfinite = 0usize;
    let (mut alloc_total, mut alloc_max, mut alloc_hops) = (0usize, 0usize, 0usize);
    for (k, xi) in xs.chunks_exact(hop).enumerate() {
        let region = stats_alloc::Region::new(GLOBAL);
        let t0 = std::time::Instant::now();
        raw.process(xi, &mut y);
        let te = t0.elapsed();
        let ch = region.change();
        if k >= a.warmup {
            let n = ch.allocations + ch.reallocations;
            alloc_total += n;
            alloc_max = alloc_max.max(n);
            alloc_hops += usize::from(n > 0);
            t_model.push(raw.last_model_time());
            t_engine.push(te);
            nonfinite += y.iter().filter(|v| !v.is_finite()).count();
        }
    }
    // 同一模型再跑一遍「三件保命全开」，看保命本身加了多少
    let (model2, _) = open_model(&a.m)?;
    let mut safe = Engine::new(model2, SafetyConfig::default());
    let mut t_safe = Vec::with_capacity(a.steps);
    for (k, xi) in xs.chunks_exact(hop).enumerate() {
        let t0 = std::time::Instant::now();
        safe.process(xi, &mut y);
        if k >= a.warmup {
            t_safe.push(t0.elapsed());
        }
    }
    let period_ms = hop as f64 / 16.0;
    let out = json!({
        "model": raw.model().name(), "model_path": path, "input": a.input,
        "hop": hop, "period_ms": period_ms, "warmup": a.warmup, "steps": a.steps,
        "threads": 1, "nonfinite_samples": nonfinite,
        "rust_allocs_per_hop": {
            "mean": alloc_total as f64 / a.steps.max(1) as f64, "max": alloc_max,
            "hops_with_alloc": alloc_hops, "note": "只计 Rust 全局分配器；ORT C++ 内部分配不在内",
        },
        "model_ms": percentiles(&t_model),
        "engine_raw_ms": percentiles(&t_engine),
        "engine_safety_ms": percentiles(&t_safe),
    });
    let text = serde_json::to_string_pretty(&out)?;
    println!("{text}");
    if let Some(j) = &a.json {
        std::fs::write(j, text)?;
    }
    Ok(())
}

fn run(a: RunArgs) -> Result<(), BoxError> {
    let o = live::RunOpts {
        seconds: a.seconds,
        block: a.block,
        lead: a.lead,
        input_device: a.input_device,
        output_device: a.output_device,
    };
    if a.mode.passthrough {
        // 单进程：cpal 回调块长不固定，用任意块长的 Pipeline::new
        live::run(vdn_denoised::identity_pipeline(None)?, &o)?;
    } else {
        let kind = ModelKind::from(a.m.model);
        live::run(
            vdn_denoised::model_pipeline(kind, a.m.model_path.as_deref(), None)?,
            &o,
        )?;
    }
    Ok(())
}

/// 返回 Ok(true) = 全部判据过。
fn chain_test(a: ChainArgs) -> Result<bool, BoxError> {
    let o = chain::ChainOpts {
        mode: match a.mode {
            ChainModeArg::Passthrough => chain::ChainMode::Passthrough,
            ChainModeArg::Denoise => chain::ChainMode::Denoise,
        },
        seconds: a.seconds,
        block: a.block,
        lead: a.lead,
        denoise_block: a.denoise_block,
        kill_at: a.kill_denoiser_at,
        bus_name: a
            .bus
            // 固定默认名：上次被 Ctrl-C 打断留下的残留，下次拿到锁时会清掉；同时跑两个会报「正被占用」
            .unwrap_or_else(|| "/vdn-chaintest".to_owned()),
        heartbeat_timeout: a.heartbeat_ms as usize * 48,
    };
    vdn_control::validate_bus_name(&o.bus_name)?;
    let r = match o.mode {
        chain::ChainMode::Passthrough => {
            chain::run(vdn_denoised::identity_pipeline(Some(o.denoise_block))?, &o)?
        }
        chain::ChainMode::Denoise => {
            let kind = ModelKind::from(a.m.model);
            chain::run(
                vdn_denoised::model_pipeline(
                    kind,
                    a.m.model_path.as_deref(),
                    Some(o.denoise_block),
                )?,
                &o,
            )?
        }
    };
    if let Some(prefix) = &a.dump {
        let p = prefix.display().to_string();
        write_wav48(Path::new(&format!("{p}-in.wav")), &r.x)?;
        write_wav48(Path::new(&format!("{p}-out.wav")), &r.y)?;
    }
    println!(
        "chain-test：模式 {} · {} s · IO 块长 {} · 降噪块长 {} · 心跳超时 {} ms{}",
        match o.mode {
            chain::ChainMode::Passthrough => "直通",
            chain::ChainMode::Denoise => "降噪",
        },
        o.seconds,
        o.block,
        o.denoise_block,
        a.heartbeat_ms,
        o.kill_at
            .map_or(String::new(), |k| format!(" · 第 {k} s 让降噪线程退出"))
    );
    println!("{}", chain::latency_line(&r));
    println!("{}", chain::counters_line(&r));
    let mut verdicts = Vec::new();
    match (o.mode, o.kill_at) {
        (chain::ChainMode::Passthrough, None) => verdicts.push(chain::judge_v1(&r)),
        (_, Some(_)) => verdicts.push(chain::judge_v2(&r)),
        (chain::ChainMode::Denoise, None) => verdicts.push(chain::judge_v3(&r)),
    }
    let mut ok = true;
    for v in &verdicts {
        println!("{v}");
        ok &= v.pass;
    }
    Ok(ok)
}

fn write_wav48(path: &Path, y: &[f32]) -> Result<(), BoxError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for &v in y {
        w.write_sample(v)?;
    }
    w.finalize()?;
    Ok(())
}

fn chain_check(a: ChainCheckArgs) -> Result<bool, BoxError> {
    let mut r = hound::WavReader::open(&a.wav)?;
    let spec = r.spec();
    if spec.channels != 1
        || spec.sample_rate != 48_000
        || spec.sample_format != hound::SampleFormat::Float
    {
        return Err(format!("{}：只收 48 kHz 单声道 32 位浮点", a.wav.display()).into());
    }
    let y = r.samples::<f32>().collect::<Result<Vec<_>, _>>()?;
    let v = chain::judge_file(&y, a.l_total, (a.from * 48_000.0) as usize);
    println!("{} · 文件 {}（{} 样本）", v, a.wav.display(), y.len());
    Ok(v.pass)
}
