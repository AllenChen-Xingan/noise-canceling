//! 音频工作区间（Audio Workgroup，规约 R8）与 QoS。
//!
//! 为什么要它：只提 time-constraint 实时优先级，系统不一定提核心频率，也可能把线程放到能效核上；
//! rs-entry 的 V7 基线里「空闲时推理反而慢」（中位约 3.0 ms，压力下 0.64–1.43 ms）就像这种情况（推断）。
//! 加入工作区间并每块报「开始 + 截止期」后，调度器知道这条线程有截止期，按截止期安排核心与频率。
//!
//! 降噪进程属于 `AudioWorkInterval.h` 头注释的第 3 种情况：实时线程和设备 IO 周期不同步、在另一个进程里，
//! 所以**自建**工作区间（`AudioWorkIntervalCreate`，macOS 11+），不需要设备 IOProc。
//! 设备自己的 IO 工作组（`kAudioDevicePropertyIOThreadOSWorkgroup`）只在送音进程里拿得到，属 vdn-io-macos，本文件不做。
//!
//! 用法（降噪线程）：
//! ```text
//! let wi = WorkInterval::create("vdn-denoise")?;      // 构造期
//! let member = wi.join()?;                            // 在处理线程上调，之后这条线程是成员
//! loop {
//!     bus.wait(..);
//!     member.begin(2_000_000)?;                       // 本块开始，截止期 = 现在 + 2 ms（实时可调）
//!     处理一块;
//!     member.finish()?;                               // 本块结束（实时可调）
//! }
//! // member drop：离开工作区间（必须在加入的那条线程上，类型保证：成员不能 Send）
//! ```

use core::ffi::{c_char, c_int, c_void};
use core::marker::PhantomData;
use core::ptr::NonNull;

use crate::RtError;

/// `os_workgroup_join_token_s`：`uint32_t sig; char opaque[36]`（64 位，SDK `os/workgroup_base.h`）。
#[repr(C)]
struct JoinToken {
    sig: u32,
    opaque: [u8; 36],
}

// 与 SDK 头文件一致：40 字节、按 4 对齐
const _: () = assert!(core::mem::size_of::<JoinToken>() == 40);

#[cfg(target_os = "macos")]
#[link(name = "AudioToolbox", kind = "framework")]
unsafe extern "C" {
    /// `os_workgroup_interval_t AudioWorkIntervalCreate(const char *name, os_clockid_t clock, os_workgroup_attr_t attr)`
    fn AudioWorkIntervalCreate(name: *const c_char, clock: u32, attr: *mut c_void) -> *mut c_void;
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    // libSystem（os/workgroup_object.h · os/workgroup_interval.h · os/object.h）
    fn os_workgroup_join(wg: *mut c_void, token_out: *mut JoinToken) -> c_int;
    fn os_workgroup_leave(wg: *mut c_void, token: *mut JoinToken);
    fn os_workgroup_interval_start(
        wg: *mut c_void,
        start: u64,
        deadline: u64,
        data: *mut c_void,
    ) -> c_int;
    fn os_workgroup_interval_finish(wg: *mut c_void, data: *mut c_void) -> c_int;
    fn os_release(object: *mut c_void);
}

/// `os/clock.h`：Mach 绝对时间（工作区间的时间戳单位）
const OS_CLOCK_MACH_ABSOLUTE_TIME: u32 = 32;

/// 自建的音频工作区间。`Send + Sync`：系统对象本身线程安全；加入 / 离开由 `Member` 绑在线程上。
pub struct WorkInterval {
    wg: NonNull<c_void>,
}

// SAFETY: os_workgroup 对象是引用计数的系统对象，可跨线程传递与共享；join / leave 的线程约束由 `Member`（!Send）保证。
unsafe impl Send for WorkInterval {}
// SAFETY: 同上；&WorkInterval 只用于 join（任意线程可调）。
unsafe impl Sync for WorkInterval {}

/// 当前线程在工作区间里的成员身份。drop 时离开（同一条线程上，类型不许 Send）。
pub struct Member<'a> {
    wi: &'a WorkInterval,
    // join 写进来的 token 地址在离开前不能变：放堆上（加入在构造期，分配不在实时路径上）
    token: Box<JoinToken>,
    /// 毫秒换 Mach 单位用：(numer, denom)
    tb: (u64, u64),
    _not_send: PhantomData<*const ()>,
}

/// 构造（非实时）
impl WorkInterval {
    /// 建一个工作区间。非 macOS 返回 `Unsupported`；系统返回空返回 `Unavailable`。
    pub fn create(name: &str) -> Result<Self, RtError> {
        #[cfg(target_os = "macos")]
        {
            let cname =
                std::ffi::CString::new(name).map_err(|_| RtError::InvalidParams("名字含 NUL"))?;
            // SAFETY: cname 以 NUL 结尾、调用期间有效；attr 头文件要求传 NULL；返回已 retain 的对象或 NULL。
            let wg = unsafe {
                AudioWorkIntervalCreate(
                    cname.as_ptr(),
                    OS_CLOCK_MACH_ABSOLUTE_TIME,
                    core::ptr::null_mut(),
                )
            };
            NonNull::new(wg)
                .map(|wg| Self { wg })
                .ok_or(RtError::Unavailable("AudioWorkIntervalCreate 返回空"))
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = name;
            Err(RtError::Unsupported)
        }
    }

    /// 当前线程加入。同一线程重复加入、或线程已在别的工作组里，系统返回错误（`Kernel { op: "os_workgroup_join" }`）。
    pub fn join(&self) -> Result<Member<'_>, RtError> {
        #[cfg(target_os = "macos")]
        {
            let tb = crate::mach_timebase()?;
            let mut token = Box::new(JoinToken {
                sig: 0,
                opaque: [0; 36],
            });
            // SAFETY: wg 是 create 拿到的有效对象；token 指向可写的 40 字节（与 SDK 结构同布局），离开前地址不变。
            let r = unsafe { os_workgroup_join(self.wg.as_ptr(), &mut *token) };
            if r != 0 {
                return Err(RtError::Kernel {
                    op: "os_workgroup_join",
                    code: r,
                });
            }
            Ok(Member {
                wi: self,
                token,
                tb,
                _not_send: PhantomData,
            })
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(RtError::Unsupported)
        }
    }
}

impl Drop for WorkInterval {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        // SAFETY: wg 由 AudioWorkIntervalCreate 返回（已 retain），只释放这一次；成员借用了 &self，drop 时已全部离开。
        unsafe {
            os_release(self.wg.as_ptr())
        };
    }
}

/// 实时可调：不分配、不加锁；每次一个非阻塞系统调用（头文件写明 interval_start 实时安全）。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Member<'_> {
    /// 本块开始：开始时间 = 现在，截止期 = 现在 + `deadline_ns`。
    pub fn begin(&self, deadline_ns: u64) -> Result<(), RtError> {
        #[cfg(target_os = "macos")]
        {
            // SAFETY: mach_absolute_time 无参数、无副作用。
            let now = unsafe { mach2::mach_time::mach_absolute_time() };
            let (numer, denom) = self.tb;
            let add = (u128::from(deadline_ns) * u128::from(denom) / u128::from(numer)) as u64;
            // SAFETY: wg 有效；当前线程是成员（Member 只能在加入的线程上存在）；data 头文件要求 NULL。
            let r = unsafe {
                os_workgroup_interval_start(
                    self.wi.wg.as_ptr(),
                    now,
                    now.saturating_add(add),
                    core::ptr::null_mut(),
                )
            };
            if r != 0 {
                return Err(RtError::Kernel {
                    op: "os_workgroup_interval_start",
                    code: r,
                });
            }
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = deadline_ns;
            Err(RtError::Unsupported)
        }
    }

    /// 本块结束。没先 `begin` 时系统返回错误。
    pub fn finish(&self) -> Result<(), RtError> {
        #[cfg(target_os = "macos")]
        {
            // SAFETY: wg 有效；当前线程是成员；data 头文件要求 NULL。
            let r =
                unsafe { os_workgroup_interval_finish(self.wi.wg.as_ptr(), core::ptr::null_mut()) };
            if r != 0 {
                return Err(RtError::Kernel {
                    op: "os_workgroup_interval_finish",
                    code: r,
                });
            }
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err(RtError::Unsupported)
        }
    }
}

impl Drop for Member<'_> {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        // SAFETY: 与 join 同一条线程（Member 不能 Send）、同一个 wg 与 token，只离开这一次。
        unsafe {
            os_workgroup_leave(self.wi.wg.as_ptr(), &mut *self.token)
        };
        let _ = &self.token.sig;
    }
}

/// 把当前线程的 QoS 设成「用户交互」（最高档；能效核 / 性能核的分配倾向性能核，推断）。
/// 与 time-constraint 是两套机制：对照实验用，看哪种能让空闲时推理不慢。
pub fn set_qos_user_interactive() -> Result<(), RtError> {
    #[cfg(target_os = "macos")]
    {
        // SAFETY: 只改当前线程；相对优先级 0 在合法范围 [-15, 0]。
        let r = unsafe {
            libc::pthread_set_qos_class_self_np(libc::qos_class_t::QOS_CLASS_USER_INTERACTIVE, 0)
        };
        if r != 0 {
            return Err(RtError::Kernel {
                op: "pthread_set_qos_class_self_np",
                code: r,
            });
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(RtError::Unsupported)
    }
}
