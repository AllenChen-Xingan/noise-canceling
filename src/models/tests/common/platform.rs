//! 对拍基准的平台标签：`<CPU 架构>-<系统>-<系统主版本>`，如 `aarch64-macos-27`。
//! ln / exp / powf 等由系统数学库算，换系统大版本末位就可能不同，所以逐位对拍只在生成基准的那个平台上成立。
//! 本文件在 `vdn-scene` 与 `vdn-models` 的测试里各有一份（测试代码不能跨 crate 共用），
//! 两份必须逐字相同：比对在 `src/models/tests/golden.rs::platform_helper_matches_scene`。

/// 本机的平台标签；取不到系统版本时主版本记 `unknown`（与任何基准都不同 → 跳过，不误报）。
pub fn platform() -> String {
    let ver = os_version().unwrap_or_default();
    let major = os_major(&ver).unwrap_or("unknown");
    format!(
        "{}-{}-{major}",
        std::env::consts::ARCH,
        std::env::consts::OS
    )
}

/// 系统版本号原文（macOS 如 `27.0.1`）；别的系统暂不取。
fn os_version() -> Option<String> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let out = std::process::Command::new("sysctl")
        .args(["-n", "kern.osproductversion"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

/// 取主版本：第一个点之前、且全是数字。
fn os_major(ver: &str) -> Option<&str> {
    let m = ver.trim().split('.').next()?;
    (!m.is_empty() && m.bytes().all(|b| b.is_ascii_digit())).then_some(m)
}

#[test]
fn os_major_parses_version() {
    assert_eq!(os_major("27.0.1\n"), Some("27"));
    assert_eq!(os_major("15"), Some("15"));
    assert_eq!(os_major(""), None);
    assert_eq!(os_major("abc.1"), None);
}

#[test]
fn platform_tag_has_three_parts() {
    let p = platform();
    assert_eq!(
        p.split('-').count(),
        3,
        "平台标签应是 架构-系统-主版本：{p}"
    );
    if cfg!(target_os = "macos") {
        assert!(!p.ends_with("-unknown"), "macOS 上应取到系统主版本：{p}");
    }
}
