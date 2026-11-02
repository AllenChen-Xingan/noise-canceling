//! `vdn-denoised`：降噪进程。
//!
//! ```text
//! vdn-denoised --passthrough [--bus /vdn-bus] [--block 480] [--lead 1440] [--seconds N]
//! vdn-denoised [--model gtcrn|fastenhancer-m] [--model-path X.onnx] …
//! vdn-denoised --config vdn.json
//! ```
//! 建总线 → 往 Processed 推 lead 个零 → 循环：Raw 够一块就跑 Pipeline → 推 Processed → 心跳。
//! Ctrl-C（或 `--seconds` 到点）干净退出并 unlink 总线。

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use clap::Parser;
use vdn_control::{Config, Mode, ModelChoice};
use vdn_denoised::{IO_RATE, Processor, Step, Worker, identity_pipeline, model_kind, model_pipeline};
use vdn_shm::{Bus, BusConfig, RingId};

#[derive(Parser)]
#[command(name = "vdn-denoised", version, about = "vdn 降噪进程：Raw 环 → Pipeline → Processed 环 + 心跳")]
struct Args {
    /// 配置文件（JSON，见 vdn-control）；命令行参数覆盖其中的同名项
    #[arg(long)]
    config: Option<PathBuf>,
    /// 直通：不加载模型（testing::Identity）
    #[arg(long)]
    passthrough: bool,
    #[arg(long, value_parser = ["gtcrn", "fastenhancer-m"])]
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
    /// 每个环的容量（样本）
    #[arg(long, default_value_t = 48_000)]
    capacity: u32,
    /// 跑多少秒后退出；不给就一直跑到 Ctrl-C
    #[arg(long)]
    seconds: Option<f64>,
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
        cfg.model = if m == "gtcrn" { ModelChoice::Gtcrn } else { ModelChoice::FastenhancerM };
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
    if let Some(l) = a.lead {
        cfg.lead_frames = l;
    }
    cfg.validate().map_err(|e| e.to_string())?;

    match cfg.mode {
        Mode::Passthrough => serve(identity_pipeline()?, &cfg, &a),
        Mode::Denoise => serve(model_pipeline(model_kind(cfg.model), cfg.model_path.as_deref())?, &cfg, &a),
    }
}

fn serve<P: Processor>(pipeline: P, cfg: &Config, a: &Args) -> Result<(), String> {
    let mut worker = Worker::new(pipeline, cfg.block);
    let l_pipe = worker.processor().latency_samples();
    let l_total = worker.l_total(cfg.lead_frames);
    if a.capacity as usize <= l_total + 2 * cfg.block {
        return Err(format!("--capacity {} 太小：至少要 L_total + 2 块 = {}", a.capacity, l_total + 2 * cfg.block));
    }

    // 上次异常退出可能留了同名对象
    let _ = Bus::unlink_name(&cfg.bus_name);
    let bus = Bus::create(
        &cfg.bus_name,
        BusConfig {
            rate: IO_RATE,
            capacity_frames: a.capacity,
            l_total: u32::try_from(l_total).map_err(|_| "L_total 太大")?,
        },
    )
    .map_err(|e| format!("建总线 {}：{e}", cfg.bus_name))?;
    let bus = Arc::new(bus);
    let primed = worker.prime(&bus, cfg.lead_frames);
    if primed != cfg.lead_frames {
        let _ = bus.unlink();
        return Err(format!("Processed 环只放下 {primed} / {} 个零", cfg.lead_frames));
    }

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed)).map_err(|e| format!("装 Ctrl-C 处理：{e}"))?;
    }

    eprintln!(
        "vdn-denoised: 总线 {} 已建 · 模式 {:?} · 块长 {} · L_total {} 样本（{:.2} ms）= Pipeline {} + 环余量 {}",
        cfg.bus_name,
        cfg.mode,
        cfg.block,
        l_total,
        l_total as f64 / 48.0,
        l_pipe,
        cfg.lead_frames
    );

    // 处理线程 = 本线程；提升失败只记数、不退出
    let mut rt_failures = 0u32;
    if !a.no_rt {
        let p = vdn_rt_thread::RtParams::for_block(cfg.block as u32, IO_RATE, 0.5);
        if let Err(e) = vdn_rt_thread::promote_current(p) {
            rt_failures += 1;
            eprintln!("vdn-denoised: 实时优先级提升失败（照常运行）：{e:?}");
        }
    }

    let deadline = a.seconds.map(|s| Instant::now() + Duration::from_secs_f64(s));
    while !stop.load(Ordering::Relaxed) && deadline.is_none_or(|d| Instant::now() < d) {
        if worker.step(&bus) == Step::Idle {
            // B 档：没活就让出 0.5 ms（后续批次换 os_sync_wait 唤醒）
            std::thread::sleep(Duration::from_micros(500));
        }
    }

    let s = worker.stats();
    let unlinked = bus.unlink();
    eprintln!(
        "vdn-denoised: 退出 · 处理 {} 块 · Processed 溢出 {} 样本 · Raw 溢出 {} · 实时提升失败 {} · unlink {}",
        s.blocks,
        s.proc_overflow,
        bus.ring(RingId::Raw).overflows(),
        rt_failures,
        if unlinked.is_ok() { "成功" } else { "失败" }
    );
    unlinked.map_err(|e| format!("unlink：{e}"))
}
