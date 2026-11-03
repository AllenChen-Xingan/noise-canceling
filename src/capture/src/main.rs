//! `vdn-capture`：送音进程。
//!
//! ```text
//! vdn-capture --source sine --sink file out.wav [--seconds 5]   # 无设备（V4）
//! vdn-capture --source mic  --sink speaker                     # 开麦、出声：先经人同意
//! vdn-capture --source mic  --sink hal                         # Out 总线留给虚拟麦克风插件读
//! ```
//! 两条总线：
//! - **Out 总线**（`--out-bus`，默认 `/vdn-out`）：本进程自己建，只用 Out 环。插件 / 扬声器 / 文件按名字连它；
//!   降噪进程重启不影响它
//! - **降噪总线**（`--bus`，默认 `/vdn-bus`）：降噪进程建。没有它时照常送延迟原声，后台每 100 ms 重试；
//!   出现了、或换了一条（会话 ID 变了）就把新总线交给音频线程（不重开设备）
//!
//! 交给音频线程用 `Mutex::try_lock`：拿不到就下一块再试，从不阻塞；换下来的旧总线 / 旧 Forwarder
//! 由音频线程放回槽里，监视线程负责释放（音频线程里不 drop 堆对象，R7）。

#![forbid(unsafe_code)]
#![allow(
    clippy::disallowed_methods, // NRT: 进程编排（建总线、开设备、正弦节拍、写文件、监视循环）；音频回调只调 Feeder::feed（try_lock 不阻塞）+ lib 里带 deny 的 forward / HalSource::render
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
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, ValueEnum};
use vdn_capture::shm::{OutSource, forward, out_bus_config};
use vdn_capture::testing::fill_tone;
use vdn_capture::{Forwarder, ForwarderConfig, ForwarderStats};
use vdn_control::{Config, IO_RATE};
use vdn_hal::{HalSource, RenderStatus};
use vdn_io_macos::{InputStream, OutputStream};
use vdn_shm::{Bus, OwnedBus, RingId, ShmError};

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
    about = "vdn 送音进程：原声 → 降噪总线 → 处理后 / 延迟原声 → Out 总线 → 出口"
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
    /// 降噪总线名
    #[arg(long)]
    bus: Option<String>,
    /// Out 总线名（本进程建）
    #[arg(long)]
    out_bus: Option<String>,
    /// IO 块长（48 kHz 样本）
    #[arg(long)]
    block: Option<usize>,
    /// 还没有降噪总线时本地延迟线的长度（样本）；不给 = 配置里的 lead。接上降噪总线后改用总线头的 L_total
    #[arg(long)]
    l_total_hint: Option<usize>,
    #[arg(long)]
    input_device: Option<String>,
    #[arg(long)]
    output_device: Option<String>,
    /// 跑多少秒；`--sink file` 默认 5，其余默认一直跑到 Ctrl-C
    #[arg(long, value_parser = vdn_control::parse_seconds)]
    seconds: Option<f64>,
    /// 心跳停了多少秒后，开始看同名总线是不是换了一条（降噪进程重启后会建新的）
    #[arg(long, default_value_t = 1.0, value_parser = vdn_control::parse_seconds)]
    reconnect_after: f64,
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("vdn-capture: {e}");
        std::process::exit(1);
    }
}

/// 监视线程交给音频线程的东西（换总线时）；音频线程拿走后把旧的放回来。
struct Handover {
    bus: Option<Arc<Bus>>,
    /// `None` = L_total 没变，音频线程在原 Forwarder 上 `rebind` 即可（延迟线不断）
    fwd: Option<Forwarder>,
    taken: bool,
}

struct Link {
    epoch: AtomicU64,
    slot: Mutex<Option<Handover>>,
    stop: AtomicBool,
}

/// 音频线程（麦克风回调或正弦线程）持有：一块原声进，推进 Out 总线。
struct Feeder {
    epoch: u64,
    bus: Option<Arc<Bus>>,
    fwd: Forwarder,
    out: Arc<OwnedBus>,
    link: Arc<Link>,
}

impl Feeder {
    fn feed(&mut self, x: &[f32]) {
        let e = self.link.epoch.load(Ordering::Acquire);
        if e != self.epoch
            && let Ok(mut slot) = self.link.slot.try_lock()
        {
            if let Some(h) = slot.as_mut()
                && !h.taken
            {
                core::mem::swap(&mut self.bus, &mut h.bus);
                match h.fwd.as_mut() {
                    Some(f) => core::mem::swap(&mut self.fwd, f),
                    None => self.fwd.rebind(),
                }
                h.taken = true;
            }
            self.epoch = e;
        }
        forward(&mut self.fwd, self.bus.as_deref(), &self.out, x);
    }
}

fn forwarder_with(l_total: usize, block: usize, timeout: usize) -> Forwarder {
    let mut c = ForwarderConfig::new(l_total, block);
    c.timeout = timeout;
    Forwarder::new(c)
}

fn open_speaker(dev: Option<&str>, out_name: &str, block: usize) -> Result<OutputStream, String> {
    let bus = Arc::new(Bus::open(out_name).map_err(|e| format!("扬声器端打开 Out 总线：{e}"))?);
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
    if let Some(b) = &a.out_bus {
        cfg.out_bus_name = b.clone();
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

    // Out 总线：本进程建（同名第二个送音进程会被锁挡住）
    let out = Arc::new(
        OwnedBus::create(&cfg.out_bus_name, out_bus_config(IO_RATE)).map_err(|e| match e {
            ShmError::Busy => format!("Out 总线 {} 正被另一个送音进程使用", cfg.out_bus_name),
            e => format!("建 Out 总线 {}：{e}", cfg.out_bus_name),
        })?,
    );

    // 降噪总线：有就接，没有就先送延迟原声
    let first = Bus::open(&cfg.bus_name).ok().map(Arc::new);
    let l_total0 = first
        .as_ref()
        .map_or(a.l_total_hint.unwrap_or(cfg.lead_frames), |b| b.l_total() as usize);
    match &first {
        Some(b) => {
            check_header(b, &cfg);
            eprintln!(
                "vdn-capture: 已接上降噪总线 {} · L_total {} 样本（{:.2} ms）· IO 块长 {block}",
                cfg.bus_name,
                l_total0,
                l_total0 as f64 / 48.0
            );
        }
        None => eprintln!(
            "vdn-capture: 还没有降噪总线 {}，先送延迟 {} 样本的原声，后台重试",
            cfg.bus_name, l_total0
        ),
    }
    eprintln!("vdn-capture: Out 总线 {} 已建", cfg.out_bus_name);

    let link = Arc::new(Link {
        epoch: AtomicU64::new(0),
        slot: Mutex::new(None),
        stop: AtomicBool::new(false),
    });
    let feeder = Feeder {
        epoch: 0,
        bus: first.clone(),
        fwd: forwarder_with(l_total0, block, timeout),
        out: out.clone(),
        link: link.clone(),
    };

    let start = Instant::now() + Duration::from_millis(20);
    let block_dur = Duration::from_secs_f64(block as f64 / f64::from(IO_RATE));
    let total = seconds.map(|s| ((s * f64::from(IO_RATE)) as usize).div_ceil(block) * block);

    // 入口
    let mut _mic = None;
    let sine = match a.source {
        Source::Sine => {
            let n = total.unwrap_or(usize::MAX / 2);
            let mut feeder = feeder;
            Some(thread::spawn(move || -> ForwarderStats {
                let mut buf = vec![0.0f32; block];
                let (mut off, mut i) = (0usize, 0u32);
                while off < n && !feeder.link.stop.load(Ordering::Relaxed) {
                    sleep_until(start + block_dur * i);
                    let k = block.min(n - off);
                    let x = buf.get_mut(..k).unwrap_or(&mut []);
                    fill_tone(x, IO_RATE, off);
                    feeder.feed(x);
                    off += k;
                    i += 1;
                }
                feeder.fwd.stats()
            }))
        }
        Source::Mic => {
            let mut feeder = feeder;
            let s = InputStream::open(cfg.input_device.as_deref(), IO_RATE, block, move |m: &[f32]| {
                feeder.feed(m);
            })
            .map_err(|e| format!("开麦克风：{e}"))?;
            eprintln!("vdn-capture: 麦克风 {:?}", s.info());
            _mic = Some(s);
            None
        }
    };

    // 出口：都按名字连 Out 总线（和插件一样）
    let mut _speaker = None;
    let file = match a.sink {
        Sink::Hal => {
            eprintln!("vdn-capture: Out 总线留给虚拟麦克风插件读（本批插件未装，读方不在时只计溢出）");
            None
        }
        Sink::Speaker => {
            _speaker = Some(open_speaker(cfg.output_device.as_deref(), &cfg.out_bus_name, block)?);
            None
        }
        Sink::File => {
            let n = total.unwrap_or(0);
            let bus = Arc::new(Bus::open(&cfg.out_bus_name).map_err(|e| format!("文件端打开 Out 总线：{e}"))?);
            Some(thread::spawn(move || {
                let mut src = HalSource::new(OutSource::new(bus));
                let mut y = vec![0.0f32; n];
                for (i, yb) in y.chunks_mut(block).enumerate() {
                    // 晚 1.5 块读，模拟插件 IO 线程
                    sleep_until(start + block_dur * i as u32 + block_dur * 3 / 2);
                    let _: RenderStatus = src.render(yb);
                }
                (y, src.counters().underrun_samples())
            }))
        }
    };

    // 监视循环
    let until = seconds.map(|s| start + Duration::from_secs_f64(s));
    let reconnect_after = Duration::from_secs_f64(a.reconnect_after);
    let mut cur = first;
    let mut cur_l_total = l_total0;
    let (mut last_beat, mut beat_at) = (cur.as_ref().map_or(0, |b| b.heartbeat()), Instant::now());
    let mut switches = 0u32;
    loop {
        thread::sleep(Duration::from_millis(100));
        if until.is_some_and(|u| Instant::now() >= u) || sine.as_ref().is_some_and(|t| t.is_finished()) {
            break;
        }
        // 收回音频线程换下来的旧对象（在这里释放）
        if let Ok(mut slot) = link.slot.lock()
            && slot.as_ref().is_some_and(|h| h.taken)
        {
            drop(slot.take());
        }
        if let Some(b) = &cur {
            let hb = b.heartbeat();
            if hb != last_beat {
                (last_beat, beat_at) = (hb, Instant::now());
                continue;
            }
            if beat_at.elapsed() < reconnect_after {
                continue;
            }
        }
        // 没有降噪总线，或心跳停太久：看同名的是不是一条新的（会话 ID 不同）
        let Ok(new) = Bus::open(&cfg.bus_name) else {
            continue;
        };
        if cur.as_ref().is_some_and(|b| b.session_id() == new.session_id()) || new.rate() != IO_RATE {
            continue;
        }
        check_header(&new, &cfg);
        let new = Arc::new(new);
        let l_new = new.l_total() as usize;
        eprintln!(
            "vdn-capture: {}降噪总线（会话 {:016x}）· L_total {l_new}{}",
            if cur.is_some() { "换到新的" } else { "接上" },
            new.session_id(),
            if l_new == cur_l_total { "（不变，延迟线接着用）" } else { "（变了，换新 Forwarder，会有一次接缝）" }
        );
        let fwd = (l_new != cur_l_total).then(|| forwarder_with(l_new, block, timeout));
        if let Ok(mut slot) = link.slot.lock() {
            // 上一次的还没被拿走就直接替换（旧的在这里释放）
            *slot = Some(Handover {
                bus: Some(new.clone()),
                fwd,
                taken: false,
            });
        }
        link.epoch.fetch_add(1, Ordering::AcqRel);
        cur = Some(new);
        cur_l_total = l_new;
        (last_beat, beat_at) = (cur.as_ref().map_or(0, |b| b.heartbeat()), Instant::now());
        switches += 1;
    }
    link.stop.store(true, Ordering::Relaxed);
    drop(_mic);
    drop(_speaker);

    if let Some(s) = sine {
        let st = s.join().map_err(|_| "送音线程 panic")?;
        eprintln!("vdn-capture: 计数 {st:?}");
    }
    if let Some(f) = file {
        let (y, underrun) = f.join().map_err(|_| "写文件线程 panic")?;
        write_wav(a.out.as_deref(), &y)?;
        eprintln!("vdn-capture: 读出欠载 {underrun} 样本");
    }
    eprintln!(
        "vdn-capture: 退出 · 接上 / 换降噪总线 {switches} 次 · Out 溢出 {} · Out 欠载 {} · Raw 溢出 {} · 心跳 {}",
        out.ring(RingId::Out).overflows(),
        out.ring(RingId::Out).underruns(),
        cur.as_ref().map_or(0, |b| b.ring(RingId::Raw).overflows()),
        cur.as_ref().map_or(0, |b| b.heartbeat())
    );
    Ok(())
}

/// 读降噪总线头核对：lead 至少要盖住一块 IO + 一块降噪，不然送音端会经常取不到处理后的。只告警。
fn check_header(b: &Bus, cfg: &Config) {
    let c = b.config();
    let need = cfg.block + c.denoise_block as usize;
    if (c.lead as usize) < need {
        eprintln!(
            "vdn-capture: 警告：降噪总线的 lead {} < IO 块 {} + 降噪块 {} = {need}，会经常用原声补",
            c.lead, cfg.block, c.denoise_block
        );
    }
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
    let mut w = hound::WavWriter::create(path, spec).map_err(|e| format!("{}：{e}", path.display()))?;
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
