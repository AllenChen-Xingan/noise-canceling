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
//! ./target/release/examples/rt_bench all [秒=20]
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
        let mut setup = String::from("ok");
        let wi = if mode == "tc_wg" { WorkInterval::create("vdn-rt-bench").ok() } else { None };
        if (mode == "tc" || mode == "tc_wg")
            && let Err(e) = promote_current(params) {
                setup = format!("promote 失败：{e}");
            }
        if mode == "qos"
            && let Err(e) = set_qos_user_interactive() {
                setup = format!("qos 失败：{e}");
            }
        if mode == "tc_wg" && wi.is_none() {
            setup = "拿不到工作区间，已跳过加入".into();
        }
        // 一块一块地干；member = 工作区间成员（tc_wg 档才有）
        let lp = |member: Option<&Member<'_>>| {
            let blocks = (Duration::from_secs(secs).as_nanos() / period.as_nanos()) as u64;
            let mut dur = Vec::with_capacity(blocks as usize);
            let mut late = Vec::with_capacity(blocks as usize);
            let mut degraded = 0u32;
            let mut wg_err = 0u32;
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
                    && m.begin(period.as_nanos() as u64 / 2).is_err() {
                        wg_err += 1;
                    }
                std::hint::black_box(work(iters));
                if let Some(m) = &member
                    && m.finish().is_err() {
                        wg_err += 1;
                    }
                dur.push(t0.elapsed().as_secs_f64() * 1e6);
                if (mode == "tc" || mode == "tc_wg") && i % per_sec.max(1) == 0
                    && !check_current().map(|s| s.time_constraint).unwrap_or(false) {
                        degraded += 1;
                    }
            }
            (start, dur, late, degraded, wg_err, cpu0)
        };
        let (start, mut dur, mut late, degraded, wg_err, cpu0) = match wi.as_ref() {
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
            "RESULT mode={mode} n={} dur_p50={:.0} dur_p999={:.0} dur_max={:.0} late_p50={:.0} late_max={:.0} degraded={degraded} wg_err={wg_err} cpu_pct={cpu:.2} setup={}",
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
    let period = Duration::from_micros(a.get(3).and_then(|s| s.parse().ok()).unwrap_or(16_000));
    let iters: u64 = a
        .get(4)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| calibrate(Duration::from_micros(700)));
    println!(
        "# 计算量 {iters} 次 · 周期 {} µs · 每种 {secs} s",
        period.as_micros()
    );
    let modes: Vec<&str> = if mode == "all" {
        vec!["none", "tc", "tc_wg", "qos"]
    } else {
        vec![mode.as_str()]
    };
    if mode == "all" {
        println!(
            "| 设置 | 块数 | 块耗时 p50 µs | p99.9 µs | max µs | 醒来迟到 p50 µs | max µs | 降级次数 | 工作区间报错 | CPU % | 备注 |"
        );
        println!("|---|---|---|---|---|---|---|---|---|---|---|");
    }
    for m in modes {
        let l = run(m, secs, period, iters);
        if mode == "all" {
            println!(
                "| {m} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                field(&l, "n"),
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
