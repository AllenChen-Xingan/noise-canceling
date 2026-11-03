//! 空闲时「推理块」耗时对照：不提优先级 / time-constraint / time-constraint + 工作区间 / QoS 用户交互。
//!
//! 起因：rs-entry 的 V7 基线里，空闲时推理一块中位约 3.0 ms，后台压力下反而 0.64–1.43 ms（推断：空闲时降频或上能效核）。
//! 本程序用**固定计算量**代替模型推理（不依赖 ort、结果可重复），每 `period` 做一块，记：
//! - 块耗时 p50 / p99.9 / max（µs）：同样的活，越快说明核心越快 / 频率越高
//! - 醒来迟到 p50 / max（µs）：到点后多久才开始干活
//! - 实时策略被系统降级的次数：每秒读回一次，time-constraint 没了就计 1
//! - CPU 时间占比（用户 + 系统 / 墙钟）
//!
//! ```text
//! cargo build --release -p vdn-rt-thread --example rt_bench
//! ./target/release/examples/rt_bench <none|tc|tc_wg|qos> [秒=20] [周期 µs=16000] [计算量=自动]
//! ./target/release/examples/rt_bench all [秒=20]           # 四档：none · tc · tc+wg · qos
//! ./target/release/examples/rt_bench qos_set [秒=20]       # QoS 那一轮：8 档，含 1 ms 紧截止期（D68 定不追性能核后取消，未跑；保留备查）
//! 本程序是手动实验，不进 ci（examples 只编译、不运行）
//! 单档设置可组合：qos · tc · wg · wg@<截止期 µs>，用 + 连，如 qos+tc+wg@1000
//! ./target/release/examples/rt_bench calibrate        # 只打印标定出的计算量
//! ```
//! 计算量默认 = 启动时在本线程上标定「约 700 µs 的活」（标定前先把本线程提成 QoS 用户交互、空转 200 ms 热身），
//! 四种设置用**同一个**计算量。输出一行 `RESULT ...`，`all` 时打印 Markdown 表。

#![allow(clippy::disallowed_methods, clippy::disallowed_macros)] // NRT: 离线实验程序

use std::time::{Duration, Instant};
use vdn_rt_thread::{
    Member, RtParams, WorkInterval, check_current, promote_current, set_qos_user_interactive,
};

fn work(iters: u64) -> u64 {
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..iters {
        x = std::hint::black_box(x.rotate_left(5) ^ i).wrapping_mul(0x2545_F491_4F6C_DD1D);
    }
    x
}

fn calibrate(target: Duration) -> u64 {
    std::thread::spawn(move || {
        let _ = set_qos_user_interactive();
        let t = Instant::now();
        while t.elapsed() < Duration::from_millis(200) {
            work(10_000);
        }
        let mut iters = 100_000u64;
        loop {
            let t = Instant::now();
            work(iters);
            let el = t.elapsed();
            if el >= Duration::from_millis(5) {
                return (iters as f64 * target.as_secs_f64() / el.as_secs_f64()) as u64;
            }
            iters *= 2;
        }
    })
    .join()
    .unwrap_or(1_000_000)
}

fn cpu_secs() -> f64 {
    // getrusage 只为读本进程 CPU 时间
    #[allow(unsafe_code)]
    // SAFETY: rusage 是纯数据结构，全零合法；getrusage 只写它，RUSAGE_SELF 合法。
    let r = unsafe {
        let mut r: libc::rusage = core::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut r);
        r
    };
    let t = |tv: libc::timeval| tv.tv_sec as f64 + tv.tv_usec as f64 * 1e-6;
    t(r.ru_utime) + t(r.ru_stime)
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(((v.len() as f64 - 1.0) * p).round() as usize)
        .copied()
        .unwrap_or(f64::NAN)
}

fn run(mode: &str, secs: u64, period: Duration, iters: u64) -> String {
    let mode = mode.to_string();
    std::thread::spawn(move || {
        // computation = 块计算量的 3 倍（按标定 700 µs 估），不超过周期的一半
        let comp = (Duration::from_micros(2100)).min(period / 2);
        let params = RtParams {
            period_ns: period.as_nanos() as u64,
            computation_ns: comp.as_nanos() as u64,
            constraint_ns: period.as_nanos() as u64,
        };
        let f = Flags::parse(&mode, period);
        let mut setup = String::from("ok");
        let wi = if f.wg.is_some() { WorkInterval::create("vdn-rt-bench").ok() } else { None };
        // 先设 QoS 再提 time-constraint（两者都要时）
        if f.qos
            && let Err(e) = set_qos_user_interactive() {
                setup = format!("qos 失败：{e}");
            }
        if f.tc
            && let Err(e) = promote_current(params) {
                setup = format!("promote 失败：{e}");
            }
        if f.wg.is_some() && wi.is_none() {
            setup = "拿不到工作区间，已跳过加入".into();
        }
        let deadline_ns = f.wg.unwrap_or(period / 2).as_nanos() as u64;
        let ecores = ecore_count();
        // 一块一块地干；member = 工作区间成员（tc_wg 档才有）
        let lp = |member: Option<&Member<'_>>| {
            let blocks = (Duration::from_secs(secs).as_nanos() / period.as_nanos()) as u64;
            let mut dur = Vec::with_capacity(blocks as usize);
            let mut late = Vec::with_capacity(blocks as usize);
            let mut degraded = 0u32;
            let mut wg_err = 0u32;
            let mut on_p = 0u64;
            let per_sec = (1_000_000_000 / period.as_nanos().max(1)) as u64;
            let cpu0 = cpu_secs();
            let start = Instant::now();
            for i in 0..blocks {
                let due = start + period * (i as u32 + 1);
                if let Some(d) = due.checked_duration_since(Instant::now()) {
                    std::thread::sleep(d);
                }
                let t0 = Instant::now();
                late.push(t0.saturating_duration_since(due).as_secs_f64() * 1e6);
                if let Some(m) = &member
                    && m.begin(deadline_ns).is_err() {
                        wg_err += 1;
                    }
                std::hint::black_box(work(iters));
                if let Some(m) = &member
                    && m.finish().is_err() {
                        wg_err += 1;
                    }
                dur.push(t0.elapsed().as_secs_f64() * 1e6);
                if current_cpu().is_some_and(|c| c >= ecores) {
                    on_p += 1;
                }
                if f.tc && i % per_sec.max(1) == 0
                    && !check_current().map(|s| s.time_constraint).unwrap_or(false) {
                        degraded += 1;
                    }
            }
            (start, dur, late, degraded, wg_err, cpu0, on_p)
        };
        let (start, mut dur, mut late, degraded, wg_err, cpu0, on_p) = match wi.as_ref() {
            Some(w) => match w.with_member(|m| lp(Some(m))) {
                Ok(r) => r,
                Err(e) => {
                    setup = format!("加入工作区间失败：{e}");
                    lp(None)
                }
            },
            None => lp(None),
        };
        let wall = start.elapsed().as_secs_f64();
        let cpu = (cpu_secs() - cpu0) / wall * 100.0;
        format!(
            "RESULT mode={mode} p_core_pct={:.1} n={} dur_p50={:.0} dur_p999={:.0} dur_max={:.0} late_p50={:.0} late_max={:.0} degraded={degraded} wg_err={wg_err} cpu_pct={cpu:.2} setup={}",
            on_p as f64 / dur.len().max(1) as f64 * 100.0,
            dur.len(),
            pct(&mut dur.clone(), 0.5),
            pct(&mut dur.clone(), 0.999),
            pct(&mut dur, 1.0),
            pct(&mut late.clone(), 0.5),
            pct(&mut late, 1.0),
            setup.replace(' ', "_"),
        )
    })
    .join()
    .unwrap_or_else(|_| "RESULT 失败".into())
}

/// 设置：`qos` · `tc` · `wg`（截止期 = 半个周期）· `wg@<µs>`，用 `+` 组合；`none` = 都不要。
/// 旧名照认：`tc_wg` = `tc+wg`。
struct Flags {
    qos: bool,
    tc: bool,
    /// 工作区间每块报的截止期
    wg: Option<Duration>,
}

impl Flags {
    fn parse(mode: &str, period: Duration) -> Self {
        let mode = if mode == "tc_wg" { "tc+wg" } else { mode };
        let mut f = Flags {
            qos: false,
            tc: false,
            wg: None,
        };
        for t in mode.split('+') {
            match t {
                "qos" => f.qos = true,
                "tc" => f.tc = true,
                "wg" => f.wg = Some(period / 2),
                _ => {
                    if let Some(us) = t.strip_prefix("wg@").and_then(|u| u.parse().ok()) {
                        f.wg = Some(Duration::from_micros(us));
                    }
                }
            }
        }
        f
    }
}

/// 能效核个数（`hw.perflevel1.logicalcpu`）。本机 powermetrics 里 CPU 0–5 是能效簇、6–9 是性能簇，
/// 所以「核号 ≥ 能效核个数」= 性能核（按本机观察的编号规律，换机器要先核对）。
fn ecore_count() -> usize {
    let mut v: libc::c_int = 0;
    let mut len = core::mem::size_of::<libc::c_int>();
    #[allow(unsafe_code)]
    // SAFETY: 名字以 NUL 结尾；v / len 可写且大小匹配；不写新值（newp = NULL）。
    let r = unsafe {
        libc::sysctlbyname(
            c"hw.perflevel1.logicalcpu".as_ptr(),
            (&mut v as *mut libc::c_int).cast(),
            &mut len,
            core::ptr::null_mut(),
            0,
        )
    };
    if r == 0 { v.max(0) as usize } else { 0 }
}

/// 当前线程此刻在哪个核上（macOS 11+）。
fn current_cpu() -> Option<usize> {
    let mut n: libc::size_t = 0;
    #[allow(unsafe_code)]
    // SAFETY: n 可写；只读当前线程状态。
    let r = unsafe { libc::pthread_cpu_number_np(&mut n) };
    (r == 0).then_some(n)
}

fn field<'a>(l: &'a str, k: &str) -> &'a str {
    l.split_whitespace()
        .find_map(|t| t.strip_prefix(k).and_then(|r| r.strip_prefix('=')))
        .unwrap_or("?")
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mode = a.get(1).cloned().unwrap_or_else(|| "all".into());
    if mode == "calibrate" {
        // 只打印标定出的计算量（给 power/run.sh 用：几种设置分开跑时用同一个数）
        println!("{}", calibrate(Duration::from_micros(700)));
        return;
    }
    let secs: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(20);
    let period = Duration::from_micros(a.get(3).and_then(|s| s.parse().ok()).unwrap_or(16_000)); // tidy: 非采样率：默认周期 16 ms（微秒）
    let iters: u64 = a
        .get(4)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| calibrate(Duration::from_micros(700)));
    println!(
        "# 计算量 {iters} 次 · 周期 {} µs · 每种 {secs} s",
        period.as_micros()
    );
    // all = 上一轮的四档；qos = 本轮（QoS 单独 / 加 tc / 加 tc + 工作区间，工作区间分宽截止期与 1 ms 紧截止期）
    let modes: Vec<&str> = match mode.as_str() {
        "all" => vec!["none", "tc", "tc+wg", "qos"],
        "qos_set" => vec![
            "none",
            "qos",
            "tc",
            "qos+tc",
            "tc+wg",
            "tc+wg@1000",
            "qos+tc+wg",
            "qos+tc+wg@1000",
        ],
        _ => vec![mode.as_str()],
    };
    let table = modes.len() > 1;
    if table {
        println!(
            "| 设置 | 块数 | 在性能核上 % | 块耗时 p50 µs | p99.9 µs | max µs | 醒来迟到 p50 µs | max µs | 降级次数 | 工作区间报错 | CPU % | 备注 |"
        );
        println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
    }
    for m in modes {
        let l = run(m, secs, period, iters);
        if table {
            println!(
                "| {m} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                field(&l, "n"),
                field(&l, "p_core_pct"),
                field(&l, "dur_p50"),
                field(&l, "dur_p999"),
                field(&l, "dur_max"),
                field(&l, "late_p50"),
                field(&l, "late_max"),
                field(&l, "degraded"),
                field(&l, "wg_err"),
                field(&l, "cpu_pct"),
                field(&l, "setup"),
            );
        } else {
            println!("{l}");
        }
    }
}
