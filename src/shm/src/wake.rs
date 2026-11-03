//! 跨进程「一下叫醒」：`os_sync_wait_on_address_with_timeout` / `os_sync_wake_by_address_all`（macOS 14.4+）。
//! 叫醒用 `_all` 不用 `_any`：同一环按约定只有一个等待者时两者代价相同；万一有残留的第二个等待者（旧进程、违反约定），
//! `_any` 可能只叫醒它而漏掉真消费方（自审 P2）。
//!
//! 最低系统 macOS 14.4（人定 D67，2026-11-02）：两个函数直接链接（libc 的声明），不再按系统版本退回。
//! 只保留一种兜底：系统调用返回不可恢复的错（EINVAL 等）时，`Bus::wait` 改用 200 µs 轮询并计 `wait_fallbacks`。
//! 非 macOS（Linux CI）、关了 feature `posix`（没有 libc）、Miri 与 loom 下没有这两个函数，`wait` 一律轮询。
//! 本文件有 unsafe（FFI），每块带 SAFETY。

#![allow(unsafe_code)] // 白名单 crate（metadata.vdn.unsafe = true）；每个 unsafe 块带 SAFETY

/// 一次等待的结果。
// 不支持的平台（Linux · Miri · loom）只构造 Failed，其余变体只有 macOS 真实现用到
#[cfg_attr(
    not(all(
        target_os = "macos",
        feature = "posix",
        not(miri),
        not(feature = "loom")
    )),
    allow(dead_code)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// 被叫醒，或进去时值已经变了（立刻返回）
    Woken,
    /// 等到超时
    TimedOut,
    /// 可重试的提前返回（EINTR · ENOMEM · EFAULT 的暂时情况）
    Retry,
    /// 不可恢复（EINVAL 等）：本进程以后改用轮询
    Failed,
}

#[cfg(all(
    target_os = "macos",
    feature = "posix",
    not(miri),
    not(feature = "loom")
))]
mod imp {
    use super::Outcome;
    use crate::sync::AtomicU32;
    use core::ffi::c_void;
    use core::sync::atomic::AtomicBool;
    use core::sync::atomic::Ordering::{Acquire, Release};

    /// 只给测试用：让每次等待都当作「系统调用返回不可恢复的错」（验证退回轮询那条路）。
    static FORCE_ERROR: AtomicBool = AtomicBool::new(false);

    pub(crate) fn init() {}

    pub(crate) fn supported() -> bool {
        true
    }

    pub(crate) fn force_error() {
        FORCE_ERROR.store(true, Release);
    }

    fn wake_flags(shared: bool) -> u32 {
        if shared {
            libc::OS_SYNC_WAKE_BY_ADDRESS_SHARED
        } else {
            libc::OS_SYNC_WAKE_BY_ADDRESS_NONE
        }
    }

    /// 两端必须一致（头文件 EINVAL 条款）：SHARED 的 wait 配 SHARED 的 wake
    fn wait_flags(shared: bool) -> u32 {
        if shared {
            libc::OS_SYNC_WAIT_ON_ADDRESS_SHARED
        } else {
            libc::OS_SYNC_WAIT_ON_ADDRESS_NONE
        }
    }

    /// 实时可调：一次非阻塞系统调用；不分配、不加锁。没有等待者时系统返回 ENOENT，忽略。
    pub(crate) fn wake_one(addr: &AtomicU32, shared: bool) {
        // SAFETY: addr 来自 &AtomicU32，非空、按 4 对齐、在映射存续期内有效；size = 4 与等待端一致；
        // 系统只把它当作内核等待队列的键，不写这块内存。返回值（0 / -1+ENOENT）不影响正确性，忽略。
        let _ = unsafe {
            libc::os_sync_wake_by_address_all(addr.as_ptr().cast::<c_void>(), 4, wake_flags(shared))
        };
    }

    /// 非实时：`*addr == expected` 时睡，最多 `timeout_ns`（≥ 1）。
    pub(crate) fn wait(addr: &AtomicU32, expected: u32, shared: bool, timeout_ns: u64) -> Outcome {
        if FORCE_ERROR.load(Acquire) {
            return Outcome::Failed;
        }
        // SAFETY: addr 同 wake_one；size = 4（只比较 expected 的低 32 位）；timeout_ns ≥ 1（0 会 EINVAL）；
        // 时钟用 Mach 绝对时间（头文件 os/clock.h 唯一的取值）。
        let r = unsafe {
            libc::os_sync_wait_on_address_with_timeout(
                addr.as_ptr().cast::<c_void>(),
                u64::from(expected),
                4,
                wait_flags(shared),
                libc::OS_CLOCK_MACH_ABSOLUTE_TIME,
                timeout_ns.max(1),
            )
        };
        if r >= 0 {
            return Outcome::Woken;
        }
        match std::io::Error::last_os_error().raw_os_error().unwrap_or(0) {
            libc::ETIMEDOUT => Outcome::TimedOut,
            libc::EINTR | libc::ENOMEM | libc::EFAULT => Outcome::Retry,
            _ => Outcome::Failed,
        }
    }
}

// 非 macOS、关了 feature posix（没有 libc）、Miri / loom 下（不能调 dlsym / 系统调用）：一律「不支持」，wait 走轮询
#[cfg(not(all(
    target_os = "macos",
    feature = "posix",
    not(miri),
    not(feature = "loom")
)))]
mod imp {
    use super::Outcome;
    use crate::sync::AtomicU32;

    pub(crate) fn init() {}
    pub(crate) fn supported() -> bool {
        false
    }
    pub(crate) fn force_error() {}
    pub(crate) fn wake_one(_addr: &AtomicU32, _shared: bool) {}
    pub(crate) fn wait(
        _addr: &AtomicU32,
        _expected: u32,
        _shared: bool,
        _timeout_ns: u64,
    ) -> Outcome {
        Outcome::Failed
    }
}

pub(crate) use imp::{force_error, init, supported, wait, wake_one};
