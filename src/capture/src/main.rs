//! `vdn-capture`：送音进程。
//!
//! ```text
//! vdn-capture --source sine --sink file out.wav [--seconds 5]   # 无设备（V4）
//! vdn-capture --source mic  --sink speaker                     # 开麦、出声：先经人同意
//! vdn-capture --source mic  --sink hal                         # Out 环留给虚拟麦克风插件读
//! ```
//! 打开降噪进程建好的总线（等它最多 `--wait` 秒）→ 每块原声走 `Forwarder` → Out 环 → 出口。

#![forbid(unsafe_code)]
#![allow(
    clippy::disallowed_methods, // NRT: 进程编排（等总线、开设备、正弦节拍、写文件）；回调闭包只调 lib 里带 deny 的 forward / HalSource::render
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
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use vdn_capture::shm::{OutSource, forward_same_bus};
use vdn_capture::testing::fill_tone;
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
    /// 心跳停了多少秒后，开始试着接同名的新总线（降噪进程重启后会建新的）
    #[arg(long, default_value_t = 1.0, value_parser = vdn_control::parse_seconds)]
    reconnect_after: f64,
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

/// 送音端和它当前接着的那条总线。降噪进程重启后换成新总线，`epoch` 加一。
struct Link {
    bus: Mutex<Arc<Bus>>,
    epoch: AtomicU64,
    /// 新总线的 Out 流从第几个样本起（正弦线程换总线时写；读方在这个位置换，前后不错位）。
    /// `usize::MAX` = 还没定（读方先别换）
    switch_at: AtomicUsize,
    stop: AtomicBool,
}

impl Link {
    fn current(&self) -> (u64, Arc<Bus>) {
        let g = self.epoch.load(Ordering::Acquire);
        let b = self
            .bus
            .lock()
            .map(|b| b.clone())
            .unwrap_or_else(|p| p.into_inner().clone());
        (g, b)
    }
}

fn forwarder_for(bus: &Bus, block: usize, timeout: usize) -> Forwarder {
    let mut fcfg = ForwarderConfig::new(bus.l_total() as usize, block);
    fcfg.timeout = timeout;
    Forwarder::new(fcfg)
}

fn open_mic(
    dev: Option<&str>,
    bus: Arc<Bus>,
    block: usize,
    timeout: usize,
) -> Result<InputStream, String> {
    let mut fwd = forwarder_for(&bus, block, timeout);
    let s = InputStream::open(dev, IO_RATE, block, move |m: &[f32]| {
        forward_same_bus(&mut fwd, &bus, m);
    })
    .map_err(|e| format!("开麦克风：{e}"))?;
    eprintln!("vdn-capture: 麦克风 {:?}", s.info());
    Ok(s)
}

fn open_speaker(dev: Option<&str>, bus: Arc<Bus>, block: usize) -> Result<OutputStream, String> {
    let mut src = HalSource::new(OutSource::new(bus));
    let s = OutputStream::open(dev, IO_RATE, block, move |o: &mut [f32]| {
        let _ = src.render(o);
    })
    .map_err(|e| format!("开扬声器：{e}"))?;
    eprintln!("vdn-capture: 扬声器 {:?}", s.info());
    Ok(s)
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
    let block = cfg.block;
    let timeout = cfg.heartbeat_timeout_samples();

    let first = Arc::new(open_bus(&cfg.bus_name, a.wait)?);
    if first.rate() != IO_RATE {
        return Err(format!(
            "总线采样率 {} Hz，本程序只支持 {IO_RATE}",
            first.rate()
        ));
    }
    eprintln!(
        "vdn-capture: 已接上总线 {} · L_total {} 样本（{:.2} ms）· 块长 {block}",
        cfg.bus_name,
        first.l_total(),
        f64::from(first.l_total()) / 48.0
    );
    let link = Arc::new(Link {
        bus: Mutex::new(first.clone()),
        epoch: AtomicU64::new(0),
        switch_at: AtomicUsize::new(0),
        stop: AtomicBool::new(false),
    });

    let start = Instant::now() + Duration::from_millis(20);
    let block_dur = Duration::from_secs_f64(block as f64 / f64::from(IO_RATE));
    let total = seconds.map(|s| ((s * f64::from(IO_RATE)) as usize).div_ceil(block) * block);

    // 入口：正弦线程自己跟着 epoch 换总线；麦克风流由下面的监视循环重开
    let mut mic = None;
    let sine = match a.source {
        Source::Sine => {
            let link = link.clone();
            let n = total.unwrap_or(usize::MAX / 2);
            Some(thread::spawn(move || {
                let (mut epoch, mut bus) = link.current();
                let mut fwd = forwarder_for(&bus, block, timeout);
                let mut buf = vec![0.0f32; block];
                let (mut off, mut i) = (0usize, 0u32);
                while off < n && !link.stop.load(Ordering::Relaxed) {
                    sleep_until(start + block_dur * i);
                    let (g, b) = link.current();
                    if g != epoch {
                        (epoch, bus) = (g, b);
                        if bus.l_total() as usize == fwd.config().l_total {
                            fwd.rebind(); // 原声不断
                        } else {
                            fwd = forwarder_for(&bus, block, timeout);
                        }
                        link.switch_at.store(off, Ordering::Release);
                    }
                    let k = block.min(n - off);
                    let x = buf.get_mut(..k).unwrap_or(&mut []);
                    fill_tone(x, IO_RATE, off);
                    forward_same_bus(&mut fwd, &bus, x);
                    off += k;
                    i += 1;
                }
                fwd.stats()
            }))
        }
        Source::Mic => {
            mic = Some(open_mic(
                cfg.input_device.as_deref(),
                first.clone(),
                block,
                timeout,
            )?);
            None
        }
    };

    // 出口
    let mut speaker = None;
    let file = match a.sink {
        Sink::Hal => {
            eprintln!("vdn-capture: Out 环留给虚拟麦克风插件读（本批插件未装，环满后计溢出）");
            None
        }
        Sink::Speaker => {
            speaker = Some(open_speaker(
                cfg.output_device.as_deref(),
                first.clone(),
                block,
            )?);
            None
        }
        Sink::File => {
            let n = total.unwrap_or(0);
            let link = link.clone();
            Some(thread::spawn(move || {
                let (mut epoch, bus) = link.current();
                let mut src = HalSource::new(OutSource::new(bus));
                let mut y = vec![0.0f32; n];
                let mut underrun = 0u64;
                for (i, yb) in y.chunks_mut(block).enumerate() {
                    // 晚 1.5 块读，模拟插件 IO 线程
                    sleep_until(start + block_dur * i as u32 + block_dur * 3 / 2);
                    let (g, b) = link.current();
                    if g != epoch && i * block >= link.switch_at.load(Ordering::Acquire) {
                        underrun += src.counters().underrun_samples();
                        epoch = g;
                        src = HalSource::new(OutSource::new(b));
                    }
                    let _: RenderStatus = src.render(yb);
                }
                (y, underrun + src.counters().underrun_samples())
            }))
        }
    };

    // 监视循环：到点 / 正弦放完就停；心跳停了超过 --reconnect-after 秒，而同名能打开一条心跳不同的总线 → 换过去
    let until = seconds.map(|s| start + Duration::from_secs_f64(s));
    let reconnect_after = Duration::from_secs_f64(a.reconnect_after);
    let (mut last_beat, mut beat_at) = (first.heartbeat(), Instant::now());
    let mut reconnects = 0u32;
    loop {
        thread::sleep(Duration::from_millis(100));
        if until.is_some_and(|u| Instant::now() >= u)
            || sine.as_ref().is_some_and(|t| t.is_finished())
        {
            break;
        }
        let (_, cur) = link.current();
        let hb = cur.heartbeat();
        if hb != last_beat {
            (last_beat, beat_at) = (hb, Instant::now());
            continue;
        }
        if beat_at.elapsed() < reconnect_after {
            continue;
        }
        let Ok(new) = Bus::open(&cfg.bus_name) else {
            continue;
        };
        if new.heartbeat() == hb || new.rate() != IO_RATE {
            continue; // 多半还是同一条（降噪进程卡住而不是重启）
        }
        let new = Arc::new(new);
        eprintln!(
            "vdn-capture: 降噪进程换了一条总线（心跳 {hb} → {}），重接 · 新 L_total {}",
            new.heartbeat(),
            new.l_total()
        );
        // 先停旧流再开新流：中间会断一小段（开流耗时，未量化）
        if mic.take().is_some() {
            mic = Some(open_mic(
                cfg.input_device.as_deref(),
                new.clone(),
                block,
                timeout,
            )?);
        }
        if speaker.take().is_some() {
            speaker = Some(open_speaker(
                cfg.output_device.as_deref(),
                new.clone(),
                block,
            )?);
        }
        // 正弦入口会自己写换总线的位置；麦克风入口已重开，读方立刻换
        link.switch_at.store(
            if sine.is_some() { usize::MAX } else { 0 },
            Ordering::Release,
        );
        if let Ok(mut slot) = link.bus.lock() {
            *slot = new.clone();
        }
        link.epoch.fetch_add(1, Ordering::AcqRel);
        (last_beat, beat_at) = (new.heartbeat(), Instant::now());
        reconnects += 1;
    }
    link.stop.store(true, Ordering::Relaxed);
    drop(mic);
    drop(speaker);

    if let Some(s) = sine {
        let st = s.join().map_err(|_| "送音线程 panic")?;
        eprintln!("vdn-capture: 计数（最后一条总线）{st:?}");
    }
    if let Some(f) = file {
        let (y, underrun) = f.join().map_err(|_| "写文件线程 panic")?;
        write_wav(a.out.as_deref(), &y)?;
        eprintln!("vdn-capture: 读出欠载 {underrun} 样本");
    }
    let (_, bus) = link.current();
    eprintln!(
        "vdn-capture: 退出 · 重接 {reconnects} 次 · Raw 溢出 {} · Out 溢出 {} · Out 欠载 {} · 心跳 {}",
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
