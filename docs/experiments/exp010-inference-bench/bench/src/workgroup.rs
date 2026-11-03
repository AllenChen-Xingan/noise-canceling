//! 计时线程加入默认输出设备的 IO 工作组（`kAudioDevicePropertyIOThreadOSWorkgroup`，macOS 11+）。
//! 用途：查「按实时节奏跑时每帧变慢」是不是调度问题。和 applebe 的 `apple-rs/src/rt.rs` 调同一组系统接口，
//! JSON 字段同名（`workgroup`），两边可以合表。
//! 注意：设备没在跑音频 IO 时，加入工作组可能拿不到调度好处；这里没效果不等于产品里没效果。

use std::ffi::c_void;

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
    fn AudioObjectGetPropertyData(
        id: u32,
        addr: *const PropAddr,
        qsize: u32,
        qdata: *const c_void,
        size: *mut u32,
        data: *mut c_void,
    ) -> i32;
}
unsafe extern "C" {
    fn os_workgroup_join(wg: *mut c_void, token: *mut c_void) -> i32;
}

/// 加入默认输出设备的 IO 工作组。返回的令牌缓冲必须活到计时结束（这里随进程退出释放，不 leave）。
pub fn join_output() -> Result<Box<[u64; 8]>, String> {
    const SYSTEM_OBJECT: u32 = 1;
    let glob = fourcc(b"glob");
    let mut dev: u32 = 0;
    let mut sz = 4u32;
    let a = PropAddr { selector: fourcc(b"dOut"), scope: glob, element: 0 };
    let st = unsafe {
        AudioObjectGetPropertyData(SYSTEM_OBJECT, &a, 0, std::ptr::null(), &mut sz, (&mut dev as *mut u32).cast())
    };
    if st != 0 {
        return Err(format!("取默认输出设备失败：{st}"));
    }
    let mut wg: *mut c_void = std::ptr::null_mut();
    let mut sz = std::mem::size_of::<*mut c_void>() as u32;
    let a = PropAddr { selector: fourcc(b"oswg"), scope: glob, element: 0 };
    let st = unsafe {
        AudioObjectGetPropertyData(dev, &a, 0, std::ptr::null(), &mut sz, (&mut wg as *mut *mut c_void).cast())
    };
    if st != 0 || wg.is_null() {
        return Err(format!("取 IO 工作组失败：{st}"));
    }
    // os_workgroup_join_token_s 是 40 字节的不透明结构，给 64 字节
    let mut token = Box::new([0u64; 8]);
    let rc = unsafe { os_workgroup_join(wg, token.as_mut_ptr().cast()) };
    if rc != 0 { Err(format!("os_workgroup_join 失败：{rc}")) } else { Ok(token) }
}
