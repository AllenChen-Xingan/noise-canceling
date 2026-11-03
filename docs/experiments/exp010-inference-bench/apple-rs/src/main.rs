//! exp010 · 苹果引擎基准（从 Rust 调用，D68）。口径与字段同 rustbe 的 `bench/src/main.rs`（正本在它顶部注释）。
//!
//! 子命令：
//!   bench <coreml|bnns> <模型> <转储前缀> --model gtcrn|fem [选项]
//!       模型：coreml 吃 .mlpackage 或 .mlmodelc；bnns 吃 .mlmodelc（用 `compile` 子命令编出来）
//!       转储前缀：rustbe `--dump-io` 写出的 `<前缀>.in.f32` / `<前缀>.ort.f32`
//!       选项：--units cpu|gpu|ane|all（coreml）· --state（coreml 有状态版）· --multi（bnns 多线程编译，对照用）
//!             --fem-json <convert_fem.json>（fem 的缓存形状）· --frames 6000 · --warmup 200 · --parity-frames 610
//!             --pace-ms 0 · --qos ui|default|rt · --count-malloc（只计数时加；计时的那次别加）
//!             --thread-policy none|tc（tc = THREAD_TIME_CONSTRAINT_POLICY，周期取 pace-ms 或 16，计算 1 ms，截止 3 ms）
//!             --tc-computation 1 · --tc-constraint 3（毫秒，tc 策略的计算量与截止）
//!             --interval-wg <截止 ms>（自建工作组区间，每帧 start/finish，报自己的截止期；D74 之后的重点）
//!             --workgroup（加入默认输出设备的 IO 工作组）· --spin（帧间忙等代替睡眠：只用来排查降频，不是产品做法）
//!   compile <模型.mlpackage> <输出.mlmodelc>     Core ML 编译（给 bnns 用）
//!   selftest                                    检查两种分配计数器真的在数
//!   loadgen [--threads 0] [--passes 400]        固定前台负载（BE-IMPACT，D75）：每个线程对自己的 4 MB 数组做 passes 遍
//!                                               乘加（算力 + 缓存 / 内存带宽两头都压）；threads 0 = 全部逻辑核。输出墙钟 ms
//! 输出一行 JSON（stdout）。
//!
//! 同口径：第 0 帧冷启动，接着预热，再计时；输入不够就循环（转储本身已按 mix.wav 循环排好）。
//! 逐值一致只比前 --parity-frames 帧，算法同 rustbe parity：最大绝对差 · 整段 SNR · 逐帧最小 SNR（只计参考能量
//! ≥ 平均帧能量 1e-4 的帧）· 非有限值个数 · first_bad（差 > 1e-3 或任一边非有限的第一个位置，下标是帧内平铺下标）。
//! 计时只包一次 `Backend::run`（含写入输入、读出输出，不含 STFT）。
//! 按节奏跑时另报「迟到」：每帧做完的时刻 − 该到的时刻（含唤醒迟到），`late_frames_over_3ms` 才是 BE-DEADLINE 的判据。

mod alloc;
mod bnns;
mod coreml;
mod rt;

use std::time::Instant;

#[global_allocator]
static A: alloc::Counting = alloc::Counting;

const BINS: usize = 257;

#[derive(Clone, Copy, PartialEq)]
pub enum ModelKind {
    Gtcrn,
    Fem,
    /// 认人声纹 WeSpeaker ECAPA-TDNN512：一次一个 1.5 s 窗（148×80 fbank）→ 192 维，无缓存
    Ecapa,
}

impl ModelKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Gtcrn => "gtcrn",
            Self::Fem => "fastenhancer_m",
            Self::Ecapa => "ecapa_tdnn512",
        }
    }
    /// 转储里每帧（ECAPA 是每个窗）输入的 f32 个数
    pub fn in_len(self) -> usize {
        self.in_shape().iter().product()
    }
    pub fn out_len(self) -> usize {
        self.out_shape().iter().product()
    }
    pub fn in_shape(self) -> Vec<usize> {
        match self {
            Self::Gtcrn => vec![1, BINS, 1, 2],
            Self::Fem => vec![1, 160],
            Self::Ecapa => vec![1, 148, 80],
        }
    }
    pub fn out_shape(self) -> Vec<usize> {
        match self {
            Self::Ecapa => vec![1, 192],
            k => k.in_shape(),
        }
    }
    pub fn input_name(self) -> &'static str {
        match self {
            Self::Gtcrn => "mix",
            Self::Fem => "wav_in",
            Self::Ecapa => "feats",
        }
    }
    pub fn output_name(self) -> &'static str {
        match self {
            Self::Gtcrn => "enh",
            Self::Fem => "wav_out",
            Self::Ecapa => "embs",
        }
    }
    pub fn cache_names(self) -> (Vec<String>, Vec<String>) {
        match self {
            Self::Gtcrn => {
                let n = ["conv_cache", "tra_cache", "inter_cache"];
                (n.iter().map(|s| s.to_string()).collect(), n.iter().map(|s| format!("{s}_out")).collect())
            }
            Self::Fem => ((0..6).map(|i| format!("cache_in_{i}")).collect(), (0..6).map(|i| format!("cache_out_{i}")).collect()),
            Self::Ecapa => (vec![], vec![]),
        }
    }
    /// 转储布局 → 模型输入布局。GTCRN：re[257] 再 im[257] → [1,257,1,2] 交错。
    pub fn pack_input(self, src: &[f32], dst: &mut [f32]) {
        match self {
            Self::Gtcrn => {
                for k in 0..BINS {
                    dst[2 * k] = src[k];
                    dst[2 * k + 1] = src[BINS + k];
                }
            }
            Self::Fem | Self::Ecapa => dst.copy_from_slice(src),
        }
    }
    pub fn unpack_output(self, src: &[f32], dst: &mut [f32]) {
        match self {
            Self::Gtcrn => {
                for k in 0..BINS {
                    dst[k] = src[2 * k];
                    dst[BINS + k] = src[2 * k + 1];
                }
            }
            Self::Fem | Self::Ecapa => dst.copy_from_slice(src),
        }
    }
}

/// 一个后端跑一帧：input / output 都是转储布局。实现必须在热路径上不经 Rust 分配器（由 rust_allocs 字段核对）。
pub trait Backend {
    fn run(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), String>;
}

fn read_f32(p: &str) -> Result<Vec<f32>, String> {
    let b = std::fs::read(p).map_err(|e| format!("读不到 {p}：{e}"))?;
    Ok(b.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect())
}

fn cpu_seconds() -> f64 {
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let t = |v: libc::timeval| v.tv_sec as f64 + v.tv_usec as f64 / 1e6;
    t(u.ru_utime) + t(u.ru_stime)
}

/// 进程常驻内存（phys_footprint，活动监视器「内存」一列的同一个量），单位 MB。
fn footprint_mb() -> f64 {
    let mut ri: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::proc_pid_rusage(libc::getpid(), libc::RUSAGE_INFO_V4, (&mut ri as *mut libc::rusage_info_v4).cast()) };
    if rc != 0 { f64::NAN } else { ri.ri_phys_footprint as f64 / 1048576.0 }
}

/// 非有限值（NaN / ±inf）打成 JSON 的 null，避免整行变成非法 JSON（codex 第 2 轮 #8）
fn jnum(x: f64, prec: usize) -> String {
    if x.is_finite() { format!("{x:.prec$}") } else { "null".into() }
}

/// 同 rustbe：下标 = round((n-1)·p)
fn pct(sorted: &[f64], p: f64) -> f64 {
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

struct Opts {
    pos: Vec<String>,
    flags: std::collections::HashMap<String, String>,
}
impl Opts {
    fn parse(a: &[String]) -> Self {
        let (mut pos, mut flags) = (Vec::new(), std::collections::HashMap::new());
        let mut i = 0;
        while i < a.len() {
            if let Some(f) = a[i].strip_prefix("--") {
                if matches!(f, "state" | "multi" | "workgroup" | "spin" | "count-malloc") {
                    flags.insert(f.to_string(), "1".to_string());
                    i += 1;
                } else {
                    flags.insert(f.to_string(), a.get(i + 1).cloned().unwrap_or_default());
                    i += 2;
                }
            } else {
                pos.push(a[i].clone());
                i += 1;
            }
        }
        Self { pos, flags }
    }
    fn get(&self, k: &str, d: &str) -> String {
        self.flags.get(k).cloned().unwrap_or_else(|| d.to_string())
    }
    fn num<T: std::str::FromStr>(&self, k: &str, d: T) -> T {
        self.flags.get(k).and_then(|v| v.parse().ok()).unwrap_or(d)
    }
}

/// 缓存形状：GTCRN 固定；FE-M 从 convert_fem.json 的 shapes 读。
fn cache_shapes(kind: ModelKind, fem_json: &str) -> Result<Vec<Vec<usize>>, String> {
    match kind {
        ModelKind::Gtcrn => Ok(vec![vec![2, 1, 16, 16, 33], vec![2, 3, 1, 1, 16], vec![2, 1, 33, 16]]),
        ModelKind::Ecapa => Ok(vec![]),
        ModelKind::Fem => {
            let j: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(fem_json).map_err(|e| format!("FE-M 要 --fem-json：{e}"))?)
                    .map_err(|e| e.to_string())?;
            (0..6)
                .map(|i| {
                    j["shapes"][format!("cache_in_{i}")]
                        .as_array()
                        .map(|a| a.iter().map(|v| v.as_u64().unwrap() as usize).collect())
                        .ok_or(format!("convert_fem.json 里没有 cache_in_{i}"))
                })
                .collect()
        }
    }
}

fn bench(o: &Opts) -> Result<(), String> {
    let (be_name, model_path, prefix) = (o.pos[1].as_str(), o.pos[2].as_str(), o.pos[3].as_str());
    let kind = match o.get("model", "").as_str() {
        "gtcrn" => ModelKind::Gtcrn,
        "fem" => ModelKind::Fem,
        "ecapa" => ModelKind::Ecapa,
        _ => return Err("--model gtcrn|fem|ecapa".into()),
    };
    let frames: usize = o.num("frames", 6000);
    let warmup: usize = o.num("warmup", 200);
    // 逐值一致默认比转储里的全部帧（转储本身就是 ORT 从零状态顺序跑的全部 1 + 预热 + 计时帧）；输出先存下，跑完再比，帧间不做额外运算
    let parity_n: usize = o.num("parity-frames", usize::MAX);
    // 分配计数会给每次 malloc 多加一次回调，拖慢分配多的后端（Core ML）；同 rustbe，计数和计时分开跑
    let count_malloc = o.flags.contains_key("count-malloc");
    let pace: f64 = o.num("pace-ms", 0.0);
    let qos = o.get("qos", "ui");
    if qos == "ui" {
        let rc = unsafe { libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE, 0) };
        if rc != 0 {
            return Err(format!("设 QoS 失败 rc={rc}"));
        }
    } else if qos != "default" && qos != "rt" {
        return Err("--qos 只收 ui / default / rt".into());
    }

    // 同 rustbe：`--qos rt` = THREAD_TIME_CONSTRAINT_POLICY（period = 帧长，computation 1 ms，constraint 3 ms，preemptible 1）
    let policy = if qos == "rt" { "tc".to_string() } else { o.get("thread-policy", "none") };
    match policy.as_str() {
        "none" => {}
        "tc" => rt::time_constraint(if pace > 0.0 { pace } else { 16.0 }, o.num("tc-computation", 1.0), o.num("tc-constraint", 3.0))?,
        _ => return Err("--thread-policy 只收 none / tc".into()),
    }
    let _wg = if o.flags.contains_key("workgroup") { Some(rt::join_output_workgroup()?) } else { None };
    let spin = o.flags.contains_key("spin");
    let ready_file = o.flags.get("ready-file").cloned(); // impact.sh 等它出现才起前台负载（codex 第 2 轮 #2）
    let iwg_ms: f64 = o.num("interval-wg", 0.0);
    let mut iwg = if iwg_ms > 0.0 { Some(rt::IntervalWg::create_and_join(iwg_ms)?) } else { None };

    let (per, per_out) = (kind.in_len(), kind.out_len());
    let input = read_f32(&format!("{prefix}.in.f32"))?;
    let refo = read_f32(&format!("{prefix}.ort.f32"))?;
    if input.is_empty() || input.len() % per != 0 || refo.len() != input.len() / per * per_out {
        return Err("in/ort 帧数不对".into());
    }
    let n_in = input.len() / per;
    let shapes = cache_shapes(kind, &o.get("fem-json", ""))?;

    if count_malloc {
        alloc::install_process_hook();
    }
    // 基准自己的缓冲（帧耗时、留存输出、迟到）先分配并逐页写一遍，再取「加载前」内存基线，
    // 这样 BE-MEM 只量到后端和模型，不混进基准自己的约 12 MiB（codex 第 3 轮 #1）
    let total = 1 + warmup + frames;
    let pn = parity_n.min(n_in).min(total);
    let mut times: Vec<f64> = Vec::with_capacity(frames);
    let mut saved = vec![0f32; pn * per_out]; // 先存输出，跑完再对拍
    let mut late: Vec<f64> = Vec::with_capacity(if pace > 0.0 { frames } else { 0 });
    for v in [&mut times, &mut late] {
        v.resize(v.capacity(), 0.0);
        for i in (0..v.len()).step_by(512) {
            v[i] = std::hint::black_box(1.0);
        }
        v.clear();
    }
    for i in (0..saved.len()).step_by(1024) {
        saved[i] = std::hint::black_box(0.0);
    }
    let fp0 = footprint_mb();
    let t0 = Instant::now();
    let (mut be, compile_ms, backend, extra): (Box<dyn Backend>, f64, String, String) = match be_name {
        "coreml" => {
            let unit = o.get("units", "cpu");
            let st = o.flags.contains_key("state");
            let m = coreml::CoreMl::open(model_path, &unit, kind, &shapes, st)?;
            let c = m.compile_ms;
            (Box::new(m), c, format!("coreml-{unit}{}", if st { "-state" } else { "" }), "{}".into())
        }
        "bnns" => {
            let multi = o.flags.contains_key("multi");
            let sizes: Vec<usize> = shapes.iter().map(|s| s.iter().product()).collect();
            let m = bnns::Bnns::open(model_path, kind, &sizes, !multi)?;
            let (c, info) = (m.compile_ms, m.info.clone());
            (Box::new(m), c, format!("bnns-graph{}", if multi { "-multi" } else { "" }), info)
        }
        _ => return Err("后端只收 coreml / bnns（mlx 见 README）".into()),
    };
    let open_ms = t0.elapsed().as_secs_f64() * 1e3 - compile_ms;

    let t0w = 1 + warmup;
    let mut out = vec![0f32; per_out];
    let (mut first_ms, mut nonfinite_frames) = (0.0, 0usize);
    let mut first_warm_ms = 0.0;
    let mut warmup_ms = 0.0; // BE-COLD：第 0 帧（冷）+ 全部预热帧的总时长，从开始跑第 0 帧算到进入计时窗
    let (mut max_abs, mut sig, mut err) = (0.0f64, 0.0f64, 0.0f64);
    let (mut worst, mut first_bad, mut nonfinite_values) = (0usize, None, 0usize);
    let mut per_frame = Vec::with_capacity(pn);
    let unix = || std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
    let mut win_start = 0.0;
    // BE-DEADLINE 要含唤醒迟到：按节奏跑时，一帧「做完的时刻 − 该到的时刻」才是这一帧离截止还有多少余量

    let (mut ra0, mut pa0, mut c0) = (0, 0, 0.0);
    let mut w0 = Instant::now();
    let tw0 = Instant::now(); // 第 0 帧开始的时刻
    let mut fp1 = f64::NAN;
    for f in 0..total {
        if f == t0w {
            fp1 = footprint_mb();
            ra0 = alloc::rust_allocs();
            pa0 = alloc::process_allocs();
            c0 = cpu_seconds();
            w0 = Instant::now();
            win_start = unix();
            warmup_ms = tw0.elapsed().as_secs_f64() * 1e3;
            if let Some(p) = &ready_file {
                let _ = std::fs::write(p, b"ready\n"); // 进入计时窗（加载、预热都已过）才发就绪信号
            }
        }
        // 父进程没了（脚本被 SIGKILL）就自己退出，不当孤儿一直跑
        if f % 64 == 0 && unsafe { libc::getppid() } == 1 {
            return Err("父进程已退出，停止".into());
        }
        if pace > 0.0 && f >= t0w {
            let due = (f - t0w) as f64 * pace;
            let now = w0.elapsed().as_secs_f64() * 1e3;
            if due > now {
                if spin {
                    while w0.elapsed().as_secs_f64() * 1e3 < due {
                        std::hint::spin_loop();
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_secs_f64((due - now) / 1e3));
                }
            }
        }
        let x = &input[(f % n_in) * per..(f % n_in + 1) * per];
        if let Some(w) = iwg.as_mut() {
            w.begin();
        }
        let t = Instant::now();
        be.run(x, &mut out)?;
        let dt = t.elapsed().as_secs_f64() * 1e3;
        if pace > 0.0 && f >= t0w {
            late.push(w0.elapsed().as_secs_f64() * 1e3 - (f - t0w) as f64 * pace);
        }
        if let Some(w) = iwg.as_mut() {
            w.end();
        }
        if f == 0 {
            first_ms = dt;
        }
        if f == t0w {
            first_warm_ms = dt; // BE-COLD：预热后第 1 帧
        }
        if f >= t0w {
            times.push(dt);
            if !out.iter().all(|v| v.is_finite()) {
                nonfinite_frames += 1;
            }
        }
        if f < pn {
            saved[f * per_out..(f + 1) * per_out].copy_from_slice(&out);
        }
    }
    let wall = w0.elapsed().as_secs_f64();
    let win_end = unix();
    // 计时窗一结束就读 CPU 与分配计数（对拍在后面做，不能算进推理占用）
    let cpu = cpu_seconds() - c0;
    let rust_n = alloc::rust_allocs() - ra0;
    let proc_n = alloc::process_allocs() - pa0;
    for f in 0..pn {
        {
            let out = &saved[f * per_out..(f + 1) * per_out];
            let r = &refo[f * per_out..(f + 1) * per_out];
            let (mut fs, mut fe) = (0.0f64, 0.0f64);
            for (k, (&p, &q)) in r.iter().zip(out).enumerate() {
                let bad_val = !p.is_finite() || !q.is_finite();
                if bad_val {
                    nonfinite_values += 1;
                }
                let d = (p as f64 - q as f64).abs();
                fs += (p as f64).powi(2);
                fe += d * d;
                if d > max_abs || d.is_nan() {
                    max_abs = if d.is_nan() { f64::NAN } else { d };
                    worst = f;
                }
                if (bad_val || !(d <= 1e-3)) && first_bad.is_none() {
                    first_bad = Some((f, k));
                }
            }
            sig += fs;
            err += fe;
            per_frame.push((fs, fe));
        }
    }
    let fp2 = footprint_mb();
    let rust_allocs = rust_n as f64 / frames as f64;
    let proc_allocs = if count_malloc { format!("{:.3}", proc_n as f64 / frames as f64) } else { "null".into() };
    let proc_total = if count_malloc { proc_n.to_string() } else { "null".into() };
    let late_over3 = late.iter().filter(|&&t| t > 3.0).count();
    let late_max = late.iter().cloned().fold(f64::NAN, f64::max);
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    let over3 = times.iter().filter(|&&t| t > 3.0).count(); // BE-DEADLINE：单帧 > 3.0 ms（延迟账余量）
    times.sort_by(|a, b| a.total_cmp(b));
    let mean_e = sig / pn.max(1) as f64;
    let min_frame_snr = per_frame
        .iter()
        .filter(|(s, _)| *s >= 1e-4 * mean_e)
        .map(|(s, e)| 10.0 * (s / e.max(1e-300)).log10())
        .fold(f64::INFINITY, f64::min);
    println!(
        "{{\"model\":\"{}\",\"backend\":\"{backend}\",\"artifact\":\"{}\",\"frames\":{frames},\"warmup\":{warmup},\"pace_ms\":{pace},\"qos\":\"{qos}\",\"thread_policy\":\"{policy}\",\"workgroup\":{},\"spin\":{spin},\"tc_computation_ms\":{},\"tc_constraint_ms\":{},\"interval_wg_deadline_ms\":{iwg_ms},\"interval_wg_errors\":{},\
\"p10_ms\":{:.4},\"p50_ms\":{:.4},\"p90_ms\":{:.4},\"p99_ms\":{:.4},\"p999_ms\":{:.4},\"max_ms\":{:.4},\"mean_ms\":{mean:.4},\"frames_over_3ms\":{over3},\
\"ort_env_ms\":0.00,\"compile_ms\":{compile_ms:.2},\"open_ms\":{open_ms:.2},\"first_frame_ms\":{first_ms:.3},\"warmup_ms\":{warmup_ms:.2},\"first_warm_frame_ms\":{first_warm_ms:.3},\"cpu_over_wall\":{:.3},\
\"rust_allocs_per_frame\":{rust_allocs:.3},\"rust_allocs_total\":{rust_n},\"process_allocs_total\":{proc_total},\"late_frames_over_3ms\":{},\"late_max_ms\":{},\"process_allocs_per_frame\":{proc_allocs},\"win_start_unix\":{win_start:.3},\"win_end_unix\":{win_end:.3},\"wall_s\":{wall:.3},\"nonfinite_frames\":{nonfinite_frames},\
\"footprint_before_mb\":{fp0:.2},\"footprint_warm_mb\":{fp1:.2},\"footprint_end_mb\":{fp2:.2},\"parity_frames\":{pn},\"parity_max_abs\":{},\"parity_worst_frame\":{worst},\"parity_snr_db\":{},\
\"parity_min_frame_snr_db\":{},\"parity_nonfinite_values\":{nonfinite_values},\"parity_first_bad\":\"{first_bad:?}\",\"extra\":{extra}}}",
        kind.name(),
        std::path::Path::new(model_path).file_name().map(|s| s.to_string_lossy()).unwrap_or_default(),
        _wg.is_some(),
        if policy == "tc" { o.num("tc-computation", 1.0) } else { 0.0 },
        if policy == "tc" { o.num("tc-constraint", 3.0) } else { 0.0 },
        iwg.as_ref().map(|w| w.errors).unwrap_or(0),
        pct(&times, 0.10),
        pct(&times, 0.50),
        pct(&times, 0.90),
        pct(&times, 0.99),
        pct(&times, 0.999),
        times[times.len() - 1],
        cpu / wall,
        if pace > 0.0 { late_over3.to_string() } else { "null".into() },
        jnum(late_max, 4),
        if max_abs.is_finite() { format!("{max_abs:.3e}") } else { "null".into() },
        jnum(10.0 * (sig / err.max(1e-300)).log10(), 2),
        jnum(min_frame_snr, 2),
    );
    Ok(())
}

/// 固定前台负载：量「开着降噪时，用户自己的活慢了多少」（BE-IMPACT）。工作量固定，只看墙钟时间。
fn loadgen(o: &Opts) -> Result<(), String> {
    let mut threads: usize = o.num("threads", 0);
    if threads == 0 {
        threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
    }
    let passes: usize = o.num("passes", 400);
    let t = Instant::now();
    let hs: Vec<_> = (0..threads)
        .map(|k| {
            std::thread::spawn(move || {
                let n = 1 << 20; // 每线程 4 MB：超过单核 L1/L2 份额，会和降噪线程抢缓存与带宽
                let mut a: Vec<f32> = (0..n).map(|i| ((i + k) % 97) as f32 * 1e-3).collect();
                let b: Vec<f32> = (0..n).map(|i| 1.0 - ((i * 7 + k) % 89) as f32 * 1e-4).collect();
                for _ in 0..passes {
                    for (x, y) in a.iter_mut().zip(&b) {
                        *x = x.mul_add(*y, 1e-3);
                    }
                }
                a.iter().map(|&v| v as f64).sum::<f64>()
            })
        })
        .collect();
    let chk: f64 = hs.into_iter().map(|h| h.join().unwrap()).sum();
    println!("{{\"loadgen_ms\":{:.1},\"threads\":{threads},\"passes\":{passes},\"checksum\":{chk:.3}}}", t.elapsed().as_secs_f64() * 1e3);
    Ok(())
}

/// 计数器自检：若钩子没生效，所有「每帧 0 次分配」的结论都不可信。
fn selftest() -> Result<(), String> {
    alloc::install_process_hook();
    let (r0, p0) = (alloc::rust_allocs(), alloc::process_allocs());
    for _ in 0..1000 {
        let p = unsafe { libc::malloc(64) };
        std::hint::black_box(p);
        unsafe { libc::free(p) };
    }
    let p1 = alloc::process_allocs();
    let v: Vec<Box<u64>> = (0..1000).map(Box::new).collect();
    std::hint::black_box(&v);
    let (r2, p2) = (alloc::rust_allocs(), alloc::process_allocs());
    let ok = p1 - p0 >= 1000 && r2 - r0 >= 1000 && p2 - p1 >= 1000;
    println!(
        "{{\"selftest\":\"{}\",\"libc_malloc_seen_by_process_hook\":{},\"rust_allocs_seen\":{},\"rust_allocs_seen_by_process_hook\":{}}}",
        if ok { "pass" } else { "FAIL" },
        p1 - p0,
        r2 - r0,
        p2 - p1
    );
    if ok { Ok(()) } else { Err("分配计数器自检失败".into()) }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = Opts::parse(&args);
    let r = match o.pos.first().map(String::as_str) {
        Some("bench") if o.pos.len() >= 4 => bench(&o),
        Some("compile") if o.pos.len() >= 3 => coreml::compile(&o.pos[1]).and_then(|(p, ms)| {
            let dst = std::path::Path::new(&o.pos[2]);
            if dst.exists() {
                return Err(format!("{} 已存在，不覆盖（换个 build 目录）", dst.display()));
            }
            let st = std::process::Command::new("cp").arg("-R").arg(&p).arg(dst).status().map_err(|e| e.to_string())?;
            if !st.success() {
                return Err("复制编译产物失败".into());
            }
            println!("{{\"compiled\":\"{}\",\"compile_ms\":{ms:.2}}}", dst.display());
            Ok(())
        }),
        Some("selftest") => selftest(),
        Some("loadgen") => loadgen(&o),
        _ => Err("用法见 src/main.rs 顶部".into()),
    };
    if let Err(e) = r {
        eprintln!("出错：{e}");
        std::process::exit(2);
    }
}
