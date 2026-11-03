//! vdn-rt-thread：实时线程优先级（规约 R8）。
//!
//! - `promote_current(RtParams)`：把**当前线程**设成 Mach time-constraint 策略（`thread_policy_set` +
//!   `THREAD_TIME_CONSTRAINT_POLICY`），送音回调线程与降噪线程启动时各调一次
//! - `check_current()`：读回当前线程的策略，启动自检用（失败计数并上报，不静默）
//! - `ThreadPolicy::default()`：降噪线程的默认策略（D68：实时优先级开、工作区间不加入）；`RtParams::from_measured`：computation = 每个周期窗口总工作量的 p99.9 × 1.5
//! - `WorkInterval`：降噪进程自建音频工作区间（`AudioWorkIntervalCreate` + `os_workgroup_join`），每块 `begin` / `finish` 报截止期，见 `workgroup.rs`
//! - `set_qos_user_interactive()`：当前线程 QoS 设成「用户交互」（对照用）
//! - 加入**设备**的 IO 工作组（`join_audio_workgroup`）：只在送音进程里拿得到，属 vdn-io-macos，本 crate 返回 `Unimplemented`
//!
//! 时间参数按纳秒给，内部换成 Mach 绝对时间单位（`mach_timebase_info`；Apple Silicon 上 1 单位 = 125/3 ns）。
//! 非 macOS 一律返回 `Unsupported`。

#![allow(unsafe_code)] // 白名单 crate（metadata.vdn.unsafe = true）；unsafe 只在 mach 模块，每块带 SAFETY
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

/// time-constraint 参数（纳秒）。约束：`computation_ns ≤ constraint_ns`，`period_ns` 为 0 或 ≥ `constraint_ns`。
/// 例：48 kHz、块长 512 → period ≈ 10 666 667 ns；computation 按实测处理耗时给，constraint 一般取 period。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtParams {
    pub period_ns: u64,
    pub computation_ns: u64,
    pub constraint_ns: u64,
}

impl RtParams {
    /// 按块长与采样率给一组常用参数：period = constraint = 块时长，computation = 块时长 × `load`（0–1）。
    pub fn for_block(block: u32, rate: u32, load: f64) -> Self {
        let period = if rate == 0 {
            0
        } else {
            u64::from(block) * 1_000_000_000 / u64::from(rate)
        };
        let computation = (period as f64 * load.clamp(0.0, 1.0)) as u64;
        Self {
            period_ns: period,
            computation_ns: computation,
            constraint_ns: period,
        }
    }

    /// 按实测耗时定参数（D68「computation 按实测 p99.9 留余量」）：
    /// computation = **一个周期窗口内总工作量**的 p99.9 × `margin`（建议 [`MEASURED_MARGIN`]），夹在 [1 ns, 周期] 内；
    /// constraint = period。
    ///
    /// 周期按「最长那一块」的节奏报，不按唤醒节奏报（大总管 2026-11-02 二选一，选 A）：
    /// 降噪线程每 2 ms 醒一次，但每 16 ms 里有 1 块推理（约 3 ms）+ 7 块轻处理（各约 0.13 ms）。
    /// - 报 16 ms：`window_p999_ns` 取每 16 ms 窗口总耗时的 p99.9（约 4 ms）→ computation ≈ 6 ms，余量真是 1.5 倍
    /// - 不报 2 ms：推理那一块单独就超过 2 ms 的周期，computation 最多夹到 2 ms，每 16 ms 必超一次，更容易被系统降级
    pub fn from_measured(period_ns: u64, window_p999_ns: u64, margin: f64) -> Self {
        let comp = (window_p999_ns as f64 * margin.max(1.0)) as u64;
        Self {
            period_ns,
            computation_ns: comp.clamp(1, period_ns.max(1)),
            constraint_ns: period_ns,
        }
    }

    fn validate(&self) -> Result<(), RtError> {
        if self.computation_ns == 0 {
            return Err(RtError::InvalidParams("computation 为 0"));
        }
        if self.computation_ns > self.constraint_ns {
            return Err(RtError::InvalidParams("computation 大于 constraint"));
        }
        if self.period_ns != 0 && self.period_ns < self.constraint_ns {
            return Err(RtError::InvalidParams("period 小于 constraint"));
        }
        Ok(())
    }
}

/// 实测 p99.9 之上留的余量倍数（`RtParams::from_measured` 用）。
/// 取 1.5：2026-11-02 耗电对照里，同一块计算 p99.9 比 p50 只多 1–6 %，max 比 p99.9 多 0.1–5 %；
/// 1.5 倍盖得住偶发慢块，又小于周期（GTCRN：每 16 ms 窗口总工作量约 4 ms → computation 约 6 ms）。
pub const MEASURED_MARGIN: f64 = 1.5;

/// 降噪处理线程的默认调度策略（人定 D68，2026-11-02）：
/// - `time_constraint = true`：实时优先级默认开。实测把醒来迟到中位从 2.6 ms 降到 22 µs，看不出额外耗电
/// - `work_interval = false`：音频工作区间默认**不**加入，保留开关。实测功耗中位多约 27 mW、没有提速（截止期宽时系统不提频）
///
/// 依据：`docs/experiments/exp-v7-latency/results/20261102-power/README.md`（单次实测）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreadPolicy {
    pub time_constraint: bool,
    pub work_interval: bool,
}

impl Default for ThreadPolicy {
    fn default() -> Self {
        Self {
            time_constraint: true,
            work_interval: false,
        }
    }
}

/// 读回的线程状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RtStatus {
    /// 当前线程是否处在 time-constraint 策略。
    /// 注意：这是**读回声明的策略**，未必看得到系统内部的临时降级——2026-11-02 耗电对照里声明 computation 2.1 ms、
    /// 实际每块 3.0 ms，每秒读回 60 次都还是 time-constraint（推断：降级不反映在读回值上，或系统容忍了；未核实）。
    /// 所以「降级计数」只能当下限看，醒来迟到与处理超时才是直接判据
    pub time_constraint: bool,
    /// 读回的参数（纳秒，由 Mach 单位换回，可能差 1 个单位的取整）
    pub params: Option<RtParams>,
    pub preemptible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RtError {
    InvalidParams(&'static str),
    /// 内核调用返回非 KERN_SUCCESS（参数超内核允许范围也会走这里）
    Kernel {
        op: &'static str,
        code: i32,
    },
    /// 非 macOS
    Unsupported,
    /// 本批未实现
    Unimplemented(&'static str),
    /// 系统没给出要的对象（如 `AudioWorkIntervalCreate` 返回空）
    Unavailable(&'static str),
}

impl core::fmt::Display for RtError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidParams(why) => write!(f, "实时参数非法：{why}"),
            Self::Kernel { op, code } => write!(f, "{op} 失败（kern_return {code}）"),
            Self::Unsupported => write!(f, "本平台不支持 Mach time-constraint"),
            Self::Unimplemented(what) => write!(f, "待实现：{what}"),
            Self::Unavailable(what) => write!(f, "系统不可用：{what}"),
        }
    }
}

impl std::error::Error for RtError {}

/// 把当前线程设成 time-constraint 实时策略（可抢占）。
pub fn promote_current(p: RtParams) -> Result<(), RtError> {
    p.validate()?;
    #[cfg(target_os = "macos")]
    {
        mach::promote(p)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(RtError::Unsupported)
    }
}

/// 读回当前线程的策略。
pub fn check_current() -> Result<RtStatus, RtError> {
    #[cfg(target_os = "macos")]
    {
        mach::check()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(RtError::Unsupported)
    }
}

/// 把当前线程加入音频设备 IO 线程的 workgroup（`os_workgroup_join`，跨进程用 `os_workgroup_copy_port`）。
/// 设备的 IO 工作组要从 `kAudioDevicePropertyIOThreadOSWorkgroup` 拿，只在开着设备 IOProc 的送音进程里有，属 vdn-io-macos；
/// 本 crate 不依赖 CoreAudio，所以返回 `Unimplemented`。降噪进程用 [`WorkInterval`]（自建工作区间）。
pub fn join_audio_workgroup() -> Result<(), RtError> {
    Err(RtError::Unimplemented(
        "设备 IO 工作组只在送音进程里拿得到（属 vdn-io-macos）；降噪进程用 WorkInterval",
    ))
}

mod workgroup;
pub use workgroup::{Member, WorkInterval, set_qos_user_interactive};

/// (numer, denom)：1 个 Mach 单位 = numer / denom 纳秒。
#[cfg(target_os = "macos")]
pub(crate) fn mach_timebase() -> Result<(u64, u64), RtError> {
    mach::timebase()
}

#[cfg(target_os = "macos")]
mod mach {
    use super::{RtError, RtParams, RtStatus};
    use mach2::kern_return::KERN_SUCCESS;
    use mach2::mach_time::{mach_timebase_info, mach_timebase_info_data_t};
    use mach2::thread_policy::{
        THREAD_TIME_CONSTRAINT_POLICY, THREAD_TIME_CONSTRAINT_POLICY_COUNT, thread_policy_get,
        thread_policy_set, thread_policy_t, thread_time_constraint_policy_data_t,
    };

    /// (numer, denom)：1 个 Mach 单位 = numer / denom 纳秒。
    pub(super) fn timebase() -> Result<(u64, u64), RtError> {
        let mut tb = mach_timebase_info_data_t { numer: 0, denom: 0 };
        // SAFETY: tb 是可写的 mach_timebase_info 结构，函数只写它。
        let kr = unsafe { mach_timebase_info(&mut tb) };
        if kr != KERN_SUCCESS || tb.numer == 0 || tb.denom == 0 {
            return Err(RtError::Kernel {
                op: "mach_timebase_info",
                code: kr,
            });
        }
        Ok((u64::from(tb.numer), u64::from(tb.denom)))
    }

    fn to_abs(ns: u64, (numer, denom): (u64, u64)) -> Result<u32, RtError> {
        let abs = u128::from(ns) * u128::from(denom) / u128::from(numer);
        u32::try_from(abs).map_err(|_| RtError::InvalidParams("时间超出 Mach 单位的 u32 范围"))
    }

    fn to_ns(abs: u32, (numer, denom): (u64, u64)) -> u64 {
        (u128::from(abs) * u128::from(numer) / u128::from(denom)) as u64
    }

    fn this_thread() -> mach2::mach_types::thread_t {
        // SAFETY: pthread_self 总是有效；pthread_mach_thread_np 不增加端口引用，不需要释放。
        unsafe { libc::pthread_mach_thread_np(libc::pthread_self()) }
    }

    pub(super) fn promote(p: RtParams) -> Result<(), RtError> {
        let tb = timebase()?;
        let mut policy = thread_time_constraint_policy_data_t {
            period: to_abs(p.period_ns, tb)?,
            computation: to_abs(p.computation_ns, tb)?,
            constraint: to_abs(p.constraint_ns, tb)?,
            preemptible: 1,
        };
        // SAFETY: policy 是 THREAD_TIME_CONSTRAINT_POLICY 对应的 repr(C) 结构，count 与之匹配；线程端口是当前线程。
        let kr = unsafe {
            thread_policy_set(
                this_thread(),
                THREAD_TIME_CONSTRAINT_POLICY,
                (&mut policy as *mut thread_time_constraint_policy_data_t).cast(),
                THREAD_TIME_CONSTRAINT_POLICY_COUNT,
            )
        };
        if kr != KERN_SUCCESS {
            return Err(RtError::Kernel {
                op: "thread_policy_set",
                code: kr,
            });
        }
        Ok(())
    }

    pub(super) fn check() -> Result<RtStatus, RtError> {
        let tb = timebase()?;
        let mut policy = thread_time_constraint_policy_data_t {
            period: 0,
            computation: 0,
            constraint: 0,
            preemptible: 0,
        };
        let mut count = THREAD_TIME_CONSTRAINT_POLICY_COUNT;
        // 传入 0：要当前值；返回后为 1 表示线程没设这个策略、给的是默认值
        let mut get_default: mach2::boolean::boolean_t = 0;
        let info: thread_policy_t =
            (&mut policy as *mut thread_time_constraint_policy_data_t).cast();
        // SAFETY: policy / count / get_default 都可写，count 与结构大小匹配；线程端口是当前线程。
        let kr = unsafe {
            thread_policy_get(
                this_thread(),
                THREAD_TIME_CONSTRAINT_POLICY,
                info,
                &mut count,
                &mut get_default,
            )
        };
        if kr != KERN_SUCCESS {
            return Err(RtError::Kernel {
                op: "thread_policy_get",
                code: kr,
            });
        }
        let on = get_default == 0;
        Ok(RtStatus {
            time_constraint: on,
            params: on.then(|| RtParams {
                period_ns: to_ns(policy.period, tb),
                computation_ns: to_ns(policy.computation, tb),
                constraint_ns: to_ns(policy.constraint, tb),
            }),
            preemptible: policy.preemptible != 0,
        })
    }
}
