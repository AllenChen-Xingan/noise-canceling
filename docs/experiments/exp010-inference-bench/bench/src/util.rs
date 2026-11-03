//! 通用工具：参数解析、调度等级、节奏等待、睡眠检测、计时统计。

use crate::*;

/// 常驻内存（MB）：`ri_phys_footprint`，即活动监视器「内存」列（与 applebe 的 BE-MEM 同口径）。
pub(crate) fn footprint_mb() -> f64 {
    let mut u: libc::rusage_info_v4 = unsafe { std::mem::zeroed() };
    let r = unsafe {
        libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            (&mut u as *mut libc::rusage_info_v4).cast(),
        )
    };
    if r != 0 {
        f64::NAN
    } else {
        u.ri_phys_footprint as f64 / (1024.0 * 1024.0)
    }
}

/// 墙钟 Unix 秒（耗电脚本按计时窗截取 powermetrics 样本用；applebe 的 power_parse.py v2 口径）
pub(crate) fn unix_now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

/// 节奏等待：第 n 帧的计划时刻 = w0 + n·pace；还没到就睡到那一刻，已过就不睡。返回计划时刻。
pub(crate) fn wait_due(w0: Instant, n: usize, pace_ms: f64) -> Instant {
    let due = w0 + std::time::Duration::from_secs_f64(n as f64 * pace_ms * 1e-3);
    if let Some(d) = due.checked_duration_since(Instant::now()) {
        std::thread::sleep(d);
    }
    due
}

/// 系统睡眠检测（exp012 20261102-v2 踩坑：拔电后空闲睡眠 42 分钟，单调时钟停走，进程看着像卡在 sleep 里）。
/// macOS 的单调时钟（Instant）在系统睡眠时不走，墙钟走；两者之差 > 1 秒就判这一轮作废。
pub(crate) struct SleepGuard {
    pub(crate) unix0: f64,
    pub(crate) mono0: Instant,
}
impl SleepGuard {
    pub(crate) fn start() -> Self {
        Self {
            unix0: unix_now(),
            mono0: Instant::now(),
        }
    }
    pub(crate) fn gap_s(&self) -> f64 {
        (unix_now() - self.unix0) - self.mono0.elapsed().as_secs_f64()
    }
    pub(crate) fn check(&self) -> Result<(), Err> {
        let g = self.gap_s();
        if g > 1.0 {
            return Err(format!("计时期间系统睡眠了约 {g:.1} 秒（墙钟比单调时钟多走），本轮作废").into());
        }
        Ok(())
    }
}

pub(crate) fn cpu_seconds() -> f64 {
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let t = |v: libc::timeval| v.tv_sec as f64 + v.tv_usec as f64 * 1e-6;
    t(u.ru_utime) + t(u.ru_stime)
}

pub(crate) fn pct(sorted: &[f64], p: f64) -> f64 {
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

pub(crate) fn arg_usize(args: &[String], flag: &str, d: usize) -> usize {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(d)
}

/// 计时线程的调度等级：ui = QOS_CLASS_USER_INTERACTIVE（默认，尽量留在性能核；实时音频线程只会更高），
/// rt = THREAD_TIME_CONSTRAINT_POLICY（period = 帧长、computation 1 ms、constraint 3 ms，可抢占），
/// bg = QOS_CLASS_BACKGROUND（系统倾向放到能效核，研究 020 量能效核用），default = 不设。
/// 审稿发现不设时 tract 的 p50 在一次运行里分两档。
pub(crate) fn set_thread_qos(qos: &str, frame_ms: f64) -> Result<(), Err> {
    if qos == "rt" {
        let mut tb = libc::mach_timebase_info { numer: 0, denom: 0 };
        unsafe { libc::mach_timebase_info(&mut tb) };
        let ms = |v: f64| (v * 1e6 * tb.denom as f64 / tb.numer as f64) as u32;
        let mut pol = libc::thread_time_constraint_policy {
            period: ms(frame_ms),
            computation: ms(1.0),
            constraint: ms(3.0),
            preemptible: 1,
        };
        let r = unsafe {
            libc::thread_policy_set(
                libc::pthread_mach_thread_np(libc::pthread_self()),
                libc::THREAD_TIME_CONSTRAINT_POLICY as _,
                (&mut pol as *mut libc::thread_time_constraint_policy).cast(),
                libc::THREAD_TIME_CONSTRAINT_POLICY_COUNT,
            )
        };
        if r != 0 {
            return Err(format!("设实时线程策略失败：{r}").into());
        }
    } else if qos == "ui" || qos == "bg" {
        let class = if qos == "ui" {
            libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE
        } else {
            libc::qos_class_t::QOS_CLASS_BACKGROUND
        };
        let r = unsafe { libc::pthread_set_qos_class_self_np(class, 0) };
        if r != 0 {
            return Err(format!("设 QoS 失败：{r}").into());
        }
    } else if qos != "default" {
        return Err("--qos 只收 ui / rt / bg / default".into());
    }
    Ok(())
}


/// JSON 数字：非有限值写 null（codex 第 1 轮 #5）
pub(crate) fn jnum(v: f64, prec: usize) -> String {
    if v.is_finite() {
        format!("{v:.prec$}")
    } else {
        "null".into()
    }
}

pub(crate) fn arg_str<'a>(args: &'a [String], flag: &str, d: &'a str) -> &'a str {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map_or(d, String::as_str)
}
