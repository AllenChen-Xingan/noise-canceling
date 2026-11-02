//! 对照小实验（009 待拍板 1）：跨进程「一下叫醒」 vs 周期轮询。手动跑，不进 CI：
//!
//! ```text
//! cargo run --release -p vdn-shm --example wake_bench -- all [秒数=10]
//! cargo run --release -p vdn-shm --example wake_bench -- one <wake|poll200|poll500> <秒数> <rt:0|1> <load:0|1>
//! ```
//!
//! 两个真进程：父进程建总线当生产者（提成实时优先级，模拟送音回调），每 2 ms 推 96 帧，
//! 推之前记一次时间戳并编进样本里；`wake` 时推完调 `notify`。子进程当消费者（模拟降噪进程），
//! `wait` 返回后立刻记时间，「从推到醒」= 醒来时间 − 样本里的时间戳。
//! 统计：延迟 p50 / p99 / max（µs）、消费者每秒被调度醒来的次数（主动让出 CPU 的上下文切换）、消费者 CPU 占用。
//! `load = 1`：另起一个进程把（核数 − 2）个核跑满。`rt = 1`：消费者也提成实时优先级。

#![allow(unsafe_code)] // 只用于 clock_gettime / getrusage 两个 libc 调用，每块带 SAFETY
#![allow(clippy::disallowed_methods, clippy::disallowed_macros)] // NRT: 离线实验程序

use std::process::{Command, Stdio};
use std::time::Duration;
use vdn_shm::{Bus, BusConfig, OwnedBus, RingId, WaitStrategy};

const BLOCK: usize = 96;
const PERIOD: Duration = Duration::from_millis(2);
const WARMUP: usize = 100;

fn now_ns() -> u64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: ts 可写；CLOCK_UPTIME_RAW 是全系统单调时钟（两个进程读的是同一个时钟）。
    unsafe { libc::clock_gettime(libc::CLOCK_UPTIME_RAW, &mut ts) };
    ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64
}

/// (用户 + 系统 CPU 秒, 主动上下文切换次数)
fn rusage() -> (f64, i64) {
    // SAFETY: rusage 是纯数据结构，全零合法；getrusage 只写它。
    let mut r: libc::rusage = unsafe { core::mem::zeroed() };
    // SAFETY: RUSAGE_SELF 合法，r 可写。
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r) };
    let t = |tv: libc::timeval| tv.tv_sec as f64 + tv.tv_usec as f64 * 1e-6;
    (t(r.ru_utime) + t(r.ru_stime), r.ru_nvcsw)
}

/// 时间戳拆成 4 个 16 位整数，按 f32 存（整数 < 2^24，f32 精确表示）。
fn encode(buf: &mut [f32; BLOCK], seq: u32, t: u64) {
    buf[0] = (seq & 0xffff) as f32;
    for k in 0..4 {
        buf[1 + k] = ((t >> (16 * k)) & 0xffff) as f32;
    }
}

fn decode(buf: &[f32; BLOCK]) -> u64 {
    (0..4).fold(0u64, |acc, k| acc | ((buf[1 + k] as u64) << (16 * k)))
}

fn promote() -> bool {
    vdn_rt_thread::promote_current(vdn_rt_thread::RtParams::for_block(BLOCK as u32, 48_000, 0.25)).is_ok()
}

fn strategy(mode: &str) -> WaitStrategy {
    match mode {
        "poll200" => WaitStrategy::Poll(Duration::from_micros(200)),
        "poll500" => WaitStrategy::Poll(Duration::from_micros(500)),
        _ => WaitStrategy::Wake,
    }
}

fn consumer(name: &str, mode: &str, rt: bool) {
    let mut bus = loop {
        match Bus::open(name) {
            Ok(b) => break b,
            Err(_) => std::thread::sleep(Duration::from_millis(5)),
        }
    };
    bus.set_wait_strategy(strategy(mode));
    let rt_ok = !rt || promote();
    let ring = bus.ring(RingId::Raw);
    let mut lat_us: Vec<f64> = Vec::with_capacity(20_000);
    let mut buf = [0.0f32; BLOCK];
    let mut started = None;
    loop {
        let w = bus.wait(RingId::Raw, BLOCK, Duration::from_millis(500));
        let woke = now_ns();
        if !w.is_ready() {
            // 生产者结束（或卡住）
            break;
        }
        let mut stop = false;
        while ring.available() >= BLOCK {
            ring.pop(&mut buf);
            if buf[0] < 0.0 {
                stop = true;
                break;
            }
            let t = decode(&buf);
            lat_us.push(woke.saturating_sub(t) as f64 / 1000.0);
            if lat_us.len() == WARMUP {
                started = Some((now_ns(), rusage()));
            }
        }
        if stop {
            break;
        }
    }
    let (t0, (cpu0, cs0)) = started.unwrap_or((now_ns(), rusage()));
    let (cpu1, cs1) = rusage();
    let wall = (now_ns() - t0) as f64 * 1e-9;
    let mut v: Vec<f64> = lat_us.into_iter().skip(WARMUP).collect();
    v.sort_by(f64::total_cmp);
    let q = |p: f64| v.get(((v.len() as f64 - 1.0) * p).round() as usize).copied().unwrap_or(f64::NAN);
    println!(
        "RESULT n={} p50={:.1} p99={:.1} max={:.1} wake_per_s={:.0} cpu_pct={:.2} fallbacks={} rt_ok={}",
        v.len(),
        q(0.5),
        q(0.99),
        v.last().copied().unwrap_or(f64::NAN),
        (cs1 - cs0) as f64 / wall,
        (cpu1 - cpu0) / wall * 100.0,
        ring.wait_fallbacks(),
        rt_ok
    );
}

fn load() {
    let n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).saturating_sub(2).max(1);
    let hs: Vec<_> = (0..n)
        .map(|_| {
            std::thread::spawn(|| {
                let mut x = 1u64;
                loop {
                    x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(1));
                }
            })
        })
        .collect();
    for h in hs {
        let _ = h.join();
    }
}

/// 跑一组，返回子进程那一行结果。
fn one(mode: &str, secs: u64, rt: bool, with_load: bool) -> String {
    let exe = std::env::current_exe().expect("current_exe");
    let name = format!("/vdn-wb{:x}", std::process::id());
    let cfg = BusConfig {
        rate: 48_000,
        capacity_frames: 4096,
        l_total: 0,
        lead: 0,
        denoise_block: BLOCK as u32,
    };
    let bus = OwnedBus::create(&name, cfg).expect("建总线");
    let mut loader = with_load.then(|| {
        Command::new(&exe).arg("load").stdout(Stdio::null()).spawn().expect("起压力进程")
    });
    let child = Command::new(&exe)
        .args(["consumer", &name, mode, if rt { "1" } else { "0" }])
        .stdout(Stdio::piped())
        .spawn()
        .expect("起消费者");
    std::thread::sleep(Duration::from_millis(200)); // 等它 open
    let notify = mode == "wake";
    let ring = bus.ring(RingId::Raw);
    let mut buf = [0.0f32; BLOCK];
    let producer_rt = promote();
    let blocks = secs * 1000 / PERIOD.as_millis() as u64;
    let start = std::time::Instant::now();
    for i in 0..blocks {
        let due = start + PERIOD * i as u32;
        if let Some(d) = due.checked_duration_since(std::time::Instant::now()) {
            std::thread::sleep(d);
        }
        encode(&mut buf, i as u32, now_ns());
        ring.push(&buf);
        if notify {
            bus.notify(RingId::Raw);
        }
    }
    buf[0] = -1.0;
    ring.push(&buf);
    bus.notify(RingId::Raw);
    let out = child.wait_with_output().expect("等消费者");
    if let Some(l) = loader.as_mut() {
        let _ = l.kill();
        let _ = l.wait();
    }
    let line = String::from_utf8_lossy(&out.stdout).lines().find(|l| l.starts_with("RESULT")).unwrap_or("RESULT 失败").to_string();
    format!("{line} producer_rt={producer_rt}")
}

fn field<'a>(line: &'a str, k: &str) -> &'a str {
    line.split_whitespace().find_map(|t| t.strip_prefix(k).and_then(|r| r.strip_prefix('='))).unwrap_or("?")
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("consumer") => consumer(&a[2], &a[3], a[4] == "1"),
        Some("load") => load(),
        Some("one") => println!("{}", one(&a[2], a[3].parse().unwrap_or(10), a[4] == "1", a[5] == "1")),
        Some("all") => {
            let secs: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(10);
            println!("| 方式 | 消费者实时优先级 | 后台压力 | 块数 | p50 µs | p99 µs | max µs | 每秒醒来次数 | 消费者 CPU % | 退回轮询 |");
            println!("|---|---|---|---|---|---|---|---|---|---|");
            for with_load in [false, true] {
                for rt in [false, true] {
                    for mode in ["wake", "poll200", "poll500"] {
                        let l = one(mode, secs, rt, with_load);
                        println!(
                            "| {mode} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                            if rt { "是" } else { "否" },
                            if with_load { "有" } else { "无" },
                            field(&l, "n"),
                            field(&l, "p50"),
                            field(&l, "p99"),
                            field(&l, "max"),
                            field(&l, "wake_per_s"),
                            field(&l, "cpu_pct"),
                            field(&l, "fallbacks"),
                        );
                    }
                }
            }
        }
        _ => eprintln!("用法见文件头注释"),
    }
}
