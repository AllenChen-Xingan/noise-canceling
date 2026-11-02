//! `vdn-capture`：送音进程。
//!
//! ```text
//! vdn-capture --source sine --sink file out.wav [--seconds 5]   # 无设备（V4）
//! vdn-capture --source mic  --sink speaker                     # 开麦、出声：先经人同意
//! vdn-capture --source mic  --sink hal                         # Out 环留给虚拟麦克风插件读
//! ```
//! 打开降噪进程建好的总线（等它最多 `--wait` 秒）→ 每块原声走 `Forwarder` → Out 环 → 出口。

#![forbid(unsafe_code)]
#![allow(clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros)] // NRT: 进程编排（等总线、开设备、正弦节拍、写文件）；回调闭包只调 lib 里带 deny 的 forward / HalSource::render
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
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use vdn_capture::shm::{OutSource, forward};
use vdn_capture::testing::test_tone;
use vdn_capture::{Forwarder, ForwarderConfig};
use vdn_control::Config;
use vdn_hal::{HalSource, RenderStatus};
use vdn_io_macos::{IO_RATE, InputStream, OutputStream};
use vdn_shm::{Bus, RingId, ShmError};

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Source {
    /// 麦克风（会弹麦克风权限框）
    Mic,
    /// 合成正弦（440 Hz + 2 kHz），按实时节拍产生
    Sine,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Sink {
    /// 不读 Out：留给虚拟麦克风插件（本批插件未装）
    Hal,
    /// 扬声器 / 耳机（会出声）
    Speaker,
    /// 写 WAV（48 kHz 单声道 32 位浮点）
    File,
}

#[derive(Parser)]
#[command(
    name = "vdn-capture",
    version,
    about = "vdn 送音进程：原声 → 共享内存 → 处理后 / 延迟原声 → 出口"
)]
struct Args {
    #[arg(long, value_enum)]
    source: Source,
    #[arg(long, value_enum)]
    sink: Sink,
    /// `--sink file` 的输出路径
    #[arg(required_if_eq("sink", "file"))]
    out: Option<PathBuf>,
    /// 配置文件（JSON，见 vdn-control）
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    bus: Option<String>,
    #[arg(long)]
    block: Option<usize>,
    #[arg(long)]
    input_device: Option<String>,
    #[arg(long)]
    output_device: Option<String>,
    /// 跑多少秒；`--sink file` 默认 5，其余默认一直跑到 Ctrl-C
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    seconds: Option<f64>,
    /// 等降噪进程建总线最多多少秒
    #[arg(long, default_value_t = 10.0, value_parser = vdn_control::parse_seconds)]
    wait: f64,
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("vdn-capture: {e}");
        std::process::exit(1);
    }
}

fn open_bus(name: &str, wait: f64) -> Result<Bus, String> {
    let until = Instant::now() + Duration::from_secs_f64(wait);
    loop {
        match Bus::open(name) {
            Ok(b) => return Ok(b),
            Err(ShmError::NotFound | ShmError::NotReady) if Instant::now() < until => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("打开总线 {name}：{e}（降噪进程起了吗？）")),
        }
    }
}

fn real_main() -> Result<(), String> {
    let a = Args::parse();
    let mut cfg = match &a.config {
        Some(p) => Config::load(p).map_err(|e| e.to_string())?,
        None => Config::default(),
    };
    if let Some(b) = &a.bus {
        cfg.bus_name = b.clone();
    }
    if let Some(b) = a.block {
        cfg.block = b;
    }
    if a.input_device.is_some() {
        cfg.input_device = a.input_device.clone();
    }
    if a.output_device.is_some() {
        cfg.output_device = a.output_device.clone();
    }
    cfg.validate().map_err(|e| e.to_string())?;
    let seconds = a.seconds.or((a.sink == Sink::File).then_some(5.0));

    let bus = Arc::new(open_bus(&cfg.bus_name, a.wait)?);
    if bus.rate() != IO_RATE {
        return Err(format!(
            "总线采样率 {} Hz，本程序只支持 {IO_RATE}",
            bus.rate()
        ));
    }
    let l_total = bus.l_total() as usize;
    let block = cfg.block;
    let mut fcfg = ForwarderConfig::new(l_total, block);
    fcfg.timeout = cfg.heartbeat_timeout_samples();
    let mut fwd = Forwarder::new(fcfg);
    eprintln!(
        "vdn-capture: 已接上总线 {} · L_total {} 样本（{:.2} ms）· 块长 {block}",
        cfg.bus_name,
        l_total,
        l_total as f64 / 48.0
    );

    let start = Instant::now() + Duration::from_millis(20);
    let block_dur = Duration::from_secs_f64(block as f64 / f64::from(IO_RATE));
    let total = seconds.map(|s| ((s * f64::from(IO_RATE)) as usize).div_ceil(block) * block);

    // 入口
    let mut _input = None;
    let sine = match a.source {
        Source::Sine => {
            let bus = bus.clone();
            let n = total.unwrap_or(usize::MAX / 2);
            Some(thread::spawn(move || {
                let mut off = 0usize;
                let mut i = 0u32;
                while off < n {
                    sleep_until(start + block_dur * i);
                    let x = test_tone(IO_RATE, block.min(n - off), off);
                    forward(&mut fwd, &bus, &x);
                    off += x.len();
                    i += 1;
                }
                fwd.stats()
            }))
        }
        Source::Mic => {
            let bus = bus.clone();
            let s = InputStream::open(
                cfg.input_device.as_deref(),
                IO_RATE,
                block,
                move |m: &[f32]| {
                    forward(&mut fwd, &bus, m);
                },
            )
            .map_err(|e| format!("开麦克风：{e}"))?;
            eprintln!("vdn-capture: 麦克风 {:?}", s.info());
            _input = Some(s);
            None
        }
    };

    // 出口
    let mut _output = None;
    let file = match a.sink {
        Sink::Hal => {
            eprintln!("vdn-capture: Out 环留给虚拟麦克风插件读（本批插件未装，环满后计溢出）");
            None
        }
        Sink::Speaker => {
            let mut src = HalSource::new(OutSource::new(bus.clone()));
            let s = OutputStream::open(
                cfg.output_device.as_deref(),
                IO_RATE,
                block,
                move |o: &mut [f32]| {
                    let _ = src.render(o);
                },
            )
            .map_err(|e| format!("开扬声器：{e}"))?;
            eprintln!("vdn-capture: 扬声器 {:?}", s.info());
            _output = Some(s);
            None
        }
        Sink::File => {
            let n = total.unwrap_or(0);
            let bus = bus.clone();
            Some(thread::spawn(move || {
                let mut src = HalSource::new(OutSource::new(bus));
                let mut y = vec![0.0f32; n];
                for (i, yb) in y.chunks_mut(block).enumerate() {
                    // 晚 1.5 块读，模拟插件 IO 线程
                    sleep_until(start + block_dur * i as u32 + block_dur * 3 / 2);
                    let _: RenderStatus = src.render(yb);
                }
                (y, src.counters())
            }))
        }
    };

    // 等结束
    match (sine, file) {
        (Some(s), Some(f)) => {
            let st = s.join().map_err(|_| "送音线程 panic")?;
            let (y, c) = f.join().map_err(|_| "写文件线程 panic")?;
            write_wav(a.out.as_deref(), &y)?;
            eprintln!(
                "vdn-capture: 计数 {st:?} · 读出欠载 {} 样本",
                c.underrun_samples()
            );
        }
        (s, f) => {
            match seconds {
                Some(sec) => thread::sleep(Duration::from_secs_f64(sec)),
                None => loop {
                    thread::sleep(Duration::from_secs(3600));
                },
            }
            if let Some(f) = f {
                let (y, _) = f.join().map_err(|_| "写文件线程 panic")?;
                write_wav(a.out.as_deref(), &y)?;
            }
            if let Some(s) = s {
                let st = s.join().map_err(|_| "送音线程 panic")?;
                eprintln!("vdn-capture: 计数 {st:?}");
            }
        }
    }
    eprintln!(
        "vdn-capture: 退出 · Raw 溢出 {} · Out 溢出 {} · Out 欠载 {} · 心跳 {}",
        bus.ring(RingId::Raw).overflows(),
        bus.ring(RingId::Out).overflows(),
        bus.ring(RingId::Out).underruns(),
        bus.heartbeat()
    );
    Ok(())
}

fn write_wav(path: Option<&std::path::Path>, y: &[f32]) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: IO_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w =
        hound::WavWriter::create(path, spec).map_err(|e| format!("{}：{e}", path.display()))?;
    for &v in y {
        w.write_sample(v).map_err(|e| e.to_string())?;
    }
    w.finalize().map_err(|e| e.to_string())?;
    eprintln!("vdn-capture: 写出 {}（{} 样本）", path.display(), y.len());
    Ok(())
}

/// 睡到某个时刻（已过就立刻返回）。
fn sleep_until(t: Instant) {
    let now = Instant::now();
    if t > now {
        thread::sleep(t - now);
    }
}
