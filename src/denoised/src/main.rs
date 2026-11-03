//! `vdn-denoised`：降噪进程。
//!
//! ```text
//! vdn-denoised --passthrough [--bus /vdn-bus] [--block 480] [--lead 1440] [--seconds N]
//! vdn-denoised [--model gtcrn|fastenhancer-m] [--model-path X.onnx] …
//! vdn-denoised --config vdn.json
//! ```
//! 运行中切模式：往标准输入写一行 `passthrough` / `denoise` / `quit`（控制线程 → rtrb → 降噪线程，不重启）。
//! 建总线 → 往 Processed 推 lead 个零 → 循环：Raw 够一块就跑 Pipeline → 推 Processed → 心跳。
//! Ctrl-C / SIGTERM / SIGHUP（或 `--seconds` 到点）干净退出并 unlink 总线；同名总线已有活着的建方时拒绝启动。

#![forbid(unsafe_code)]
#![allow(
    clippy::disallowed_methods, // NRT: 进程编排（建总线、信号、日志、空闲让出 sleep）；实时部分在 lib 的 Worker::step。空闲 sleep 轮询待换跨进程唤醒（backlog）
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use clap::Parser;
use vdn_control::{Config, Mode, ModelChoice};
use vdn_denoised::{
    Processor, Step, Worker, denoise_bus_config, identity_pipeline, model_kind, model_pipeline,
};
use vdn_shm::{OwnedBus, RingId, ShmError};

#[derive(Parser)]
#[command(
    name = "vdn-denoised",
    version,
    about = "vdn 降噪进程：Raw 环 → Pipeline → Processed 环 + 心跳"
)]
struct Args {
    /// 配置文件（JSON，见 vdn-control）；命令行参数覆盖其中的同名项
    #[arg(long)]
    config: Option<PathBuf>,
    /// 直通：不加载模型（testing::Identity）
    #[arg(long)]
    passthrough: bool,
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(ModelId::NAMES))]
    model: Option<String>,
    #[arg(long)]
    model_path: Option<PathBuf>,
    /// 共享内存名（/vdn- 开头）
    #[arg(long)]
    bus: Option<String>,
    /// 块长（48 kHz 样本）
    #[arg(long)]
    block: Option<usize>,
    /// 环余量（48 kHz 样本）
    #[arg(long)]
    lead: Option<usize>,
    /// 降噪块长（48 kHz 样本，须整除模型 hop × 3，默认 96）
    #[arg(long)]
    denoise_block: Option<usize>,
    /// 每个环的容量（样本）
    #[arg(long, default_value_t = 48_000)]
    capacity: u32,
    /// 跑多少秒后退出；不给就一直跑到 Ctrl-C
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    seconds: Option<f64>,
    /// 启动预热多少块（喂零、丢输出，让推理库第一次的慢调用不落在服务期）
    #[arg(long, default_value_t = 64)]
    warmup_blocks: usize,
    /// 不提升实时优先级
    #[arg(long)]
    no_rt: bool,
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("vdn-denoised: {e}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let a = Args::parse();
    let mut cfg = match &a.config {
        Some(p) => Config::load(p).map_err(|e| e.to_string())?,
        None => Config::default(),
    };
    if a.passthrough {
        cfg.mode = Mode::Passthrough;
    }
    if let Some(m) = &a.model {
        let id = ModelId::parse(m).ok_or("未知的模型")?;
        cfg.model = model_choice(ModelKind::from_id(id));
    }
    if let Some(p) = &a.model_path {
        cfg.model_path = Some(p.clone());
    }
    if let Some(b) = &a.bus {
        cfg.bus_name = b.clone();
    }
    if let Some(b) = a.block {
        cfg.block = b;
    }
    if let Some(b) = a.denoise_block {
        cfg.denoise_block = b;
    }
    if let Some(l) = a.lead {
        cfg.lead_frames = l;
    }
    cfg.validate().map_err(|e| e.to_string())?;

    match cfg.mode {
        Mode::Passthrough => serve(identity_pipeline(Some(cfg.denoise_block))?, &cfg, &a),
        Mode::Denoise => serve(
            model_pipeline(
                model_kind(cfg.model),
                cfg.model_path.as_deref(),
                Some(cfg.denoise_block),
            )?,
            &cfg,
            &a,
        ),
    }
}

fn serve<P: Processor>(pipeline: P, cfg: &Config, a: &Args) -> Result<(), String> {
    let mut worker =
        Worker::new(pipeline, cfg.denoise_block).skip_backlog_over_lead(cfg.lead_frames);
    let w = worker.warm_up(a.warmup_blocks);
    eprintln!(
        "vdn-denoised: 预热 {} 块 · 第一块 {:.3} ms · 之后最长 {:.3} ms · 平均 {:.3} ms（一块音频 {:.3} ms）",
        w.blocks,
        w.first_ns as f64 / 1e6,
        w.max_after_first_ns as f64 / 1e6,
        w.mean_ns as f64 / 1e6,
        worker.block_ns() as f64 / 1e6
    );
    let l_pipe = worker.processor().latency_samples();
    let l_total = worker.l_total(cfg.lead_frames);
    let bus_cfg = denoise_bus_config(&worker, cfg.lead_frames, a.capacity as usize)?;

    // lead ≥ 一块 IO 已在 cfg.validate()（vdn_control::check_lead）里查过

    // 先装信号处理（Ctrl-C / SIGTERM / SIGHUP）再建总线：之间收到信号也能走正常退出、删名字
    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed))
            .map_err(|e| format!("装信号处理：{e}"))?;
    }

    // 拿到这个名字的锁才建（同名第二个降噪进程直接报错，不删别人在用的总线）；
    // 之后任何返回路径（含出错、panic 展开）都由守卫删名字
    let bus = OwnedBus::create(&cfg.bus_name, bus_cfg).map_err(|e| match e {
        ShmError::Busy => format!(
            "总线 {} 正被另一个降噪进程（或 chain-test）使用；换个 --bus 名字，或先停掉它",
            cfg.bus_name
        ),
        e => format!("建总线 {}：{e}", cfg.bus_name),
    })?;
    let primed = worker.prime(&bus, cfg.lead_frames);
    if primed != cfg.lead_frames {
        return Err(format!(
            "Processed 环只放下 {primed} / {} 个零",
            cfg.lead_frames
        ));
    }

    eprintln!(
        "vdn-denoised: 总线 {} 已建 · 模式 {:?} · 降噪块长 {} · L_total {} 样本（{:.2} ms）= Pipeline {} + 环余量 {}",
        cfg.bus_name,
        cfg.mode,
        cfg.denoise_block,
        l_total,
        l_total as f64 / 48.0,
        l_pipe,
        cfg.lead_frames
    );

    // 处理线程 = 本线程。调度按 D68 默认（time-constraint 开，参数按预热实测）；提升失败只记数、不退出
    let mut rt_failures = 0u32;
    let policy = vdn_rt_thread::ThreadPolicy::default();
    if !a.no_rt && policy.time_constraint {
        let p = vdn_denoised::denoise_rt_params(&worker, &w);
        eprintln!(
            "vdn-denoised: 实时调度 period {:.3} ms · computation {:.3} ms（每周期 {} 块总耗时 p99.9 {:.3} ms × {}）",
            p.period_ns as f64 / 1e6,
            p.computation_ns as f64 / 1e6,
            w.period_blocks,
            w.p999_period_ns as f64 / 1e6,
            vdn_rt_thread::MEASURED_MARGIN
        );
        if let Err(e) = vdn_rt_thread::promote_current(p) {
            rt_failures += 1;
            eprintln!("vdn-denoised: 实时优先级提升失败（照常运行）：{e:?}");
        }
    }

    let deadline = a
        .seconds
        .map(|s| Instant::now() + Duration::from_secs_f64(s));
    // 控制线程：读标准输入的命令行，经 rtrb 交给本线程（块之间执行）
    let (mut tx, mut rx) = vdn_denoised::command_channel(16);
    std::thread::spawn(move || {
        let mut line = String::new();
        while std::io::stdin().read_line(&mut line).is_ok_and(|n| n > 0) {
            let cmd = match line.trim() {
                "passthrough" | "直通" => {
                    Some(vdn_control::Cmd::SetMode(vdn_control::Mode::Passthrough))
                }
                "denoise" | "降噪" => Some(vdn_control::Cmd::SetMode(vdn_control::Mode::Denoise)),
                "quit" | "退出" => Some(vdn_control::Cmd::Shutdown),
                "" => None,
                other => {
                    eprintln!(
                        "vdn-denoised: 不认识的命令 {other:?}（passthrough / denoise / quit）"
                    );
                    None
                }
            };
            if let Some(c) = cmd
                && tx.push(c).is_err()
            {
                eprintln!("vdn-denoised: 命令队列满，丢掉 {c:?}");
            }
            line.clear();
        }
    });

    let mut next_rt_check = Instant::now();
    while !stop.load(Ordering::Relaxed) && deadline.is_none_or(|d| Instant::now() < d) {
        if !worker.apply_commands(&mut rx) {
            break;
        }
        // 每秒读回一次调度策略：读不到或不是 time-constraint 记一次降级（提升本来就关了就不查）
        if !a.no_rt && policy.time_constraint && Instant::now() >= next_rt_check {
            let ok = vdn_rt_thread::check_current().is_ok_and(|st| st.time_constraint);
            worker.note_rt_check(ok);
            next_rt_check += Duration::from_secs(1);
        }
        // 等送音端推够一块再醒（送音端每推一块 notify 一次）；20 ms 兜底超时，好检查退出标志
        let _ = bus.wait(RingId::Raw, cfg.denoise_block, Duration::from_millis(20));
        while worker.step(&bus) == Step::Block {}
    }

    let s = worker.stats();
    let unlinked = bus.unlink();
    eprintln!(
        "vdn-denoised: 退出 · 处理 {} 块 · 单块最长 {:.3} ms · 平均 {:.3} ms · 超一块时长 {} 块 · 每周期总耗时最长 {:.3} ms · 超周期 {} 次 · 醒来时 Raw 最多积 {} 样本（{:.2} ms）· 调度读回 {} 次、降级 {} 次 · 跳过积压 {} 样本 · Processed 溢出 {} 样本 · Raw 溢出 {} · 实时提升失败 {} · 唤醒 {} · 退回轮询 {} 次 · unlink {}",
        s.blocks,
        s.max_process_ns as f64 / 1e6,
        s.total_process_ns as f64 / 1e6 / s.blocks.max(1) as f64,
        s.over_budget,
        s.max_period_ns as f64 / 1e6,
        s.over_period,
        s.max_raw_backlog,
        s.max_raw_backlog as f64 / 48.0,
        s.rt_checks,
        s.rt_downgrades,
        s.raw_skipped,
        s.proc_overflow,
        bus.ring(RingId::Raw).overflows(),
        rt_failures,
        if vdn_shm::Bus::wake_supported() {
            "可用"
        } else {
            "不可用（本机系统低于 14.4，走轮询）"
        },
        bus.ring(RingId::Raw).wait_fallbacks(),
        if unlinked.is_ok() { "成功" } else { "失败" }
    );
    unlinked.map_err(|e| format!("unlink：{e}"))
}
