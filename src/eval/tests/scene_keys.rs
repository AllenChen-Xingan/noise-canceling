//! 场景键 · 设备键 · 系统处理模式：Rust 枚举（`vdn_eval::scene`）与正本原表逐个比对，多一个、少一个都红。
//! 正本：021 `catalog.md` 主表第一列 · 022 `devices.md` 第一节第一列（带反引号的键）· 第二节 `mic_mode` 表第一列。

use std::collections::BTreeSet;
use std::path::PathBuf;

use vdn_eval::files::serde_name;
use vdn_eval::scene::{DeviceKey, MicMode, MeetingPlace};

fn read(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("读不到正本 {}：{e}", p.display()))
}

/// 表格行 `| \`键\` | …` 的键。
fn backtick_keys(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|l| l.strip_prefix("| `"))
        .filter_map(|r| r.split('`').next())
        .map(str::to_string)
        .collect()
}

/// `## 一、` 到 `## 二、` 之间（devices.md 第一节）。
fn section<'a>(text: &'a str, from: &str, to: &str) -> &'a str {
    let a = text
        .find(from)
        .unwrap_or_else(|| panic!("找不到小节 {from}"));
    let rest = &text[a..];
    let b = rest.find(to).unwrap_or(rest.len());
    &rest[..b]
}

#[test]
fn scene_keys_match_021_catalog() {
    let doc = backtick_keys(&read("docs/research/021_meeting-scenes/catalog.md"));
    let code: BTreeSet<String> = MeetingPlace::ALL.iter().map(serde_name).collect();
    assert_eq!(code, doc, "MeetingPlace 与 021 主表第一列不一致");
    assert_eq!(doc.len(), 16, "021 主表现在是 15 个地点键 + other");
}

#[test]
fn device_keys_match_022_section1() {
    let text = read("docs/research/022_mac-mic/devices.md");
    let doc = backtick_keys(section(&text, "## 一、", "## 二、"));
    let code: BTreeSet<String> = DeviceKey::ALL.iter().map(serde_name).collect();
    assert_eq!(code, doc, "DeviceKey 与 022 第一节不一致");
}

#[test]
fn mic_modes_match_022_section2() {
    let text = read("docs/research/022_mac-mic/devices.md");
    let sec = section(&text, "## 二、", "## 三、");
    let doc: BTreeSet<String> = sec
        .lines()
        .filter_map(|l| l.strip_prefix("| "))
        .filter_map(|r| r.split(" |").next())
        .map(str::trim)
        .filter(|k| !k.is_empty() && k.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
        .map(str::to_string)
        .collect();
    let code: BTreeSet<String> = MicMode::ALL.iter().map(serde_name).collect();
    assert_eq!(code, doc, "MicMode 与 022 第二节不一致");
}

#[test]
fn all_lists_are_exhaustive() {
    // 加变体不加进 ALL，这里编译不过
    let n = |k: MeetingPlace| match k {
        MeetingPlace::HomeQuiet
        | MeetingPlace::HomeFamily
        | MeetingPlace::OpenOffice
        | MeetingPlace::PhoneBooth
        | MeetingPlace::MeetingRoom
        | MeetingPlace::Cowork
        | MeetingPlace::Cafe
        | MeetingPlace::Canteen
        | MeetingPlace::TransitHub
        | MeetingPlace::CarPassenger
        | MeetingPlace::BusMetroCabin
        | MeetingPlace::RailAirCabin
        | MeetingPlace::StreetOutdoor
        | MeetingPlace::HotelRoom
        | MeetingPlace::LibraryClassroom
        | MeetingPlace::Other => 16,
    };
    assert_eq!(MeetingPlace::ALL.len(), n(MeetingPlace::Other));
    let n = |m: MicMode| match m {
        MicMode::Standard | MicMode::VoiceIsolation | MicMode::WideSpectrum | MicMode::Unknown => 4,
    };
    assert_eq!(MicMode::ALL.len(), n(MicMode::Unknown));
    let n = |d: DeviceKey| match d {
        DeviceKey::MbaBuiltin
        | DeviceKey::MbpBuiltin
        | DeviceKey::MacIntelBuiltin
        | DeviceKey::ImacBuiltin
        | DeviceKey::StudioDisplay
        | DeviceKey::AirpodsHfp
        | DeviceKey::BtHfpGeneric
        | DeviceKey::UsbHeadset
        | DeviceKey::ExternalMic
        | DeviceKey::IphoneContinuity
        | DeviceKey::MacUnknown
        | DeviceKey::LaptopUnknown
        | DeviceKey::NonLaptop
        | DeviceKey::NonLaptopPhone
        | DeviceKey::NonLaptopTablet
        | DeviceKey::NonLaptopDesktop
        | DeviceKey::NonLaptopConfDevice
        | DeviceKey::NonLaptopMicArray
        | DeviceKey::NonLaptopCloseTalk
        | DeviceKey::NonLaptopStudio
        | DeviceKey::NonLaptopUnknown => 21,
    };
    assert_eq!(DeviceKey::ALL.len(), n(DeviceKey::NonLaptop));
    let set: BTreeSet<_> = DeviceKey::ALL.iter().collect();
    assert_eq!(set.len(), DeviceKey::ALL.len());
}
