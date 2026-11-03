//! 实时线程策略（给「实时节奏下每帧慢约 5 倍」的排查用，019）。
//! - `time_constraint`：mach `THREAD_TIME_CONSTRAINT_POLICY`（Core Audio 的 IO 线程用的同一种），周期 / 计算量 / 截止都按毫秒给
//! - `join_output_workgroup`：加入默认输出设备的 IO 线程工作组（`kAudioDevicePropertyIOThreadOSWorkgroup`）。
//!   注意：设备没在跑 IO 时，工作组的调度收益可能拿不到；结果要和 `time_constraint` 对照着看
//! 绑性能核：Apple Silicon 上 macOS 没有公开的核亲和接口（affinity tag 被忽略），只能靠 QoS / 实时策略 / 工作组。
use std::ffi::c_void;

fn ms_to_abs(ms: f64) -> u32 {
    let mut tb = libc::mach_timebase_info { numer: 0, denom: 0 };
    unsafe { libc::mach_timebase_info(&mut tb) };
    (ms * 1e6 * tb.denom as f64 / tb.numer as f64) as u32
}

pub fn time_constraint(period_ms: f64, computation_ms: f64, constraint_ms: f64) -> Result<(), String> {
    let mut p = libc::thread_time_constraint_policy {
        period: ms_to_abs(period_ms),
        computation: ms_to_abs(computation_ms),
        constraint: ms_to_abs(constraint_ms),
        preemptible: 1,
    };
    let count = (std::mem::size_of::<libc::thread_time_constraint_policy>() / std::mem::size_of::<libc::integer_t>()) as u32;
    #[allow(deprecated)]
    let th = unsafe { libc::mach_thread_self() };
    let kr = unsafe {
        libc::thread_policy_set(th, libc::THREAD_TIME_CONSTRAINT_POLICY as u32, (&mut p as *mut libc::thread_time_constraint_policy).cast(), count)
    };
    if kr == 0 { Ok(()) } else { Err(format!("thread_policy_set 失败 kr={kr}")) }
}

#[repr(C)]
struct PropAddr {
    selector: u32,
    scope: u32,
    element: u32,
}
const fn fourcc(s: &[u8; 4]) -> u32 {
    ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | s[3] as u32
}

#[link(name = "CoreAudio", kind = "framework")]
unsafe extern "C" {
    fn AudioObjectGetPropertyData(id: u32, addr: *const PropAddr, qsize: u32, qdata: *const c_void, size: *mut u32, data: *mut c_void) -> i32;
}
unsafe extern "C" {
    fn os_workgroup_join(wg: *mut c_void, token: *mut c_void) -> i32;
}

/// 加入默认输出设备的 IO 工作组。返回的 token 缓冲要一直留着（离开时应 leave，这里进程结束即释放）。
pub fn join_output_workgroup() -> Result<Box<[u64; 8]>, String> {
    unsafe {
        let glob = fourcc(b"glob");
        let mut dev: u32 = 0;
        let mut sz = 4u32;
        let a = PropAddr { selector: fourcc(b"dOut"), scope: glob, element: 0 };
        let st = AudioObjectGetPropertyData(1, &a, 0, std::ptr::null(), &mut sz, (&mut dev as *mut u32).cast());
        if st != 0 {
            return Err(format!("取默认输出设备失败 {st}"));
        }
        let mut wg: *mut c_void = std::ptr::null_mut();
        let mut sz = std::mem::size_of::<*mut c_void>() as u32;
        let a = PropAddr { selector: fourcc(b"oswg"), scope: glob, element: 0 };
        let st = AudioObjectGetPropertyData(dev, &a, 0, std::ptr::null(), &mut sz, (&mut wg as *mut *mut c_void).cast());
        if st != 0 || wg.is_null() {
            return Err(format!("取 IO 工作组失败 {st}"));
        }
        let mut token = Box::new([0u64; 8]); // os_workgroup_join_token_s 是 40 字节的不透明结构，这里给 64 字节
        let rc = os_workgroup_join(wg, token.as_mut_ptr().cast());
        if rc != 0 { Err(format!("os_workgroup_join 失败 {rc}")) } else { Ok(token) }
    }
}
