//! CFPlugIn 外壳：coreaudiod 加载 `.driver` 包时，按 `Info.plist` 的 `CFPlugInFactories` 找到这个导出符号并调用。
//!
//! 真驱动接口待实现（D05）：将来这里返回一个 `AudioServerPlugInDriverInterface` 的 IUnknown 指针，
//! 驱动的 IO 回调里调 [`crate::HalSource::render`]。**本批返回空指针**：CFPlugIn 把空指针当作「工厂造不出实例」，
//! 宿主跳过这个插件，不会把一个半成品设备挂进系统（安全）。
//!
//! 本批也不组装 `.driver` 包、不安装：cargo 只产出 `target/` 下的 `libvdn_hal.dylib`，系统不会去那里找插件。

use std::ffi::c_void;

/// `Info.plist` 里 `CFPlugInFactories` 写的工厂 UUID（本项目生成后写死，别改：改了已装的包会对不上）。
pub const FACTORY_UUID: &str = "74085B86-80AC-46AB-88B0-331E157E7A4F";

/// Apple 定义的 `kAudioServerPlugInTypeUUID`（`AudioServerPlugIn.h`）。
pub const AUDIO_SERVER_PLUGIN_TYPE_UUID: &str = "443ABAB8-E7B3-491A-B985-BEB9187030DB";

/// 导出的工厂函数名，必须与 `Info.plist` 的 `CFPlugInFactories` 值一致。
pub const FACTORY_SYMBOL: &str = "VdnHalFactory";

/// CFPlugIn 工厂函数。签名照 `CFPlugInFactoryFunction`：`void *(*)(CFAllocatorRef, CFUUIDRef)`。
///
/// 真驱动接口待实现（D05）。现在不看参数、不解引用任何指针，直接返回空指针 = 不加载。
#[allow(unsafe_code)]
// R10：导出 C 符号要 #[unsafe(no_mangle)]；本 crate 在 unsafe 白名单（metadata.vdn.unsafe = true）
#[unsafe(no_mangle)]
pub extern "C" fn VdnHalFactory(
    _allocator: *const c_void,
    _requested_type: *const c_void,
) -> *mut c_void {
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLIST: &str = include_str!("../bundle/Info.plist");

    #[test]
    fn factory_returns_null_so_host_skips_plugin() {
        assert!(VdnHalFactory(std::ptr::null(), std::ptr::null()).is_null());
    }

    #[test]
    fn plist_names_our_factory_and_audio_server_type() {
        // 工厂表：UUID → 函数名
        let factory_entry =
            format!("<key>{FACTORY_UUID}</key>\n\t\t<string>{FACTORY_SYMBOL}</string>");
        assert!(
            PLIST.contains(&factory_entry),
            "CFPlugInFactories 要把 {FACTORY_UUID} 映射到 {FACTORY_SYMBOL}"
        );
        // 类型表：AudioServerPlugIn 类型 → [工厂 UUID]
        let type_entry = format!(
            "<key>{AUDIO_SERVER_PLUGIN_TYPE_UUID}</key>\n\t\t<array>\n\t\t\t<string>{FACTORY_UUID}</string>"
        );
        assert!(
            PLIST.contains(&type_entry),
            "CFPlugInTypes 要把类型 UUID 映射到工厂 UUID"
        );
    }

    #[test]
    fn uuids_are_well_formed_and_distinct() {
        for u in [FACTORY_UUID, AUDIO_SERVER_PLUGIN_TYPE_UUID] {
            let parts: Vec<usize> = u.split('-').map(str::len).collect();
            assert_eq!(parts, [8, 4, 4, 4, 12], "{u}");
            assert!(u.chars().all(|c| c == '-' || c.is_ascii_hexdigit()), "{u}");
        }
        assert_ne!(FACTORY_UUID, AUDIO_SERVER_PLUGIN_TYPE_UUID);
    }
}
