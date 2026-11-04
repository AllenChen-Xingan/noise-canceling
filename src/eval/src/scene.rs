//! 场景测试集的片段清单（`clips.jsonl` 每行一段，exp013 scenesuite 生成）：字段的**正本**。
//!
//! - 由 `vdn-schema` 导出 `specs/schema/eval-scene-clip.schema.json`；scenesuite 的 `manifest.py` 读那份 schema 校验
//! - 场景键、设备键、系统处理模式的正本是 021 `catalog.md` 主表、022 `devices.md` 第一 / 二节；这里的枚举是第二处表示，
//!   由 `tests/scene_keys.rs` 逐个比对原表（多一个、少一个都红）
//! - 声音类别 `cls` 的正本是 scenesuite 的 `sounds.json`，这里只当字符串读，不抄一份
//! - 时间一律 16 kHz 样本号，区间左闭右开；电平 dBFS；距离米

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::files::Lang;

/// 021 主表的地点键（含兜底键 `other`，它不进成绩单）。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum MeetingPlace {
    HomeQuiet,
    HomeFamily,
    OpenOffice,
    PhoneBooth,
    MeetingRoom,
    Cowork,
    Cafe,
    Canteen,
    TransitHub,
    CarPassenger,
    BusMetroCabin,
    RailAirCabin,
    StreetOutdoor,
    HotelRoom,
    LibraryClassroom,
    Other,
}

impl MeetingPlace {
    pub const ALL: [MeetingPlace; 16] = [
        MeetingPlace::HomeQuiet,
        MeetingPlace::HomeFamily,
        MeetingPlace::OpenOffice,
        MeetingPlace::PhoneBooth,
        MeetingPlace::MeetingRoom,
        MeetingPlace::Cowork,
        MeetingPlace::Cafe,
        MeetingPlace::Canteen,
        MeetingPlace::TransitHub,
        MeetingPlace::CarPassenger,
        MeetingPlace::BusMetroCabin,
        MeetingPlace::RailAirCabin,
        MeetingPlace::StreetOutdoor,
        MeetingPlace::HotelRoom,
        MeetingPlace::LibraryClassroom,
        MeetingPlace::Other,
    ];

    /// 键名（由 serde 定；一致性由测试核对）。
    pub fn as_str(self) -> String {
        crate::files::serde_name(&self)
    }

    /// 成绩单里算不算一个开会场景（`other` 不算，021 主表原文「不算开会场景，不进覆盖矩阵」）。
    pub fn is_meeting(self) -> bool {
        self != MeetingPlace::Other
    }
}

/// 022 第一节的设备键。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceKey {
    MbaBuiltin,
    MbpBuiltin,
    MacIntelBuiltin,
    ImacBuiltin,
    StudioDisplay,
    AirpodsHfp,
    BtHfpGeneric,
    UsbHeadset,
    ExternalMic,
    IphoneContinuity,
    MacUnknown,
    LaptopUnknown,
    NonLaptop,
    NonLaptopPhone,
    NonLaptopTablet,
    NonLaptopDesktop,
    NonLaptopConfDevice,
    NonLaptopMicArray,
    NonLaptopCloseTalk,
    NonLaptopStudio,
    NonLaptopUnknown,
}

impl DeviceKey {
    pub const ALL: [DeviceKey; 21] = [
        DeviceKey::MbaBuiltin,
        DeviceKey::MbpBuiltin,
        DeviceKey::MacIntelBuiltin,
        DeviceKey::ImacBuiltin,
        DeviceKey::StudioDisplay,
        DeviceKey::AirpodsHfp,
        DeviceKey::BtHfpGeneric,
        DeviceKey::UsbHeadset,
        DeviceKey::ExternalMic,
        DeviceKey::IphoneContinuity,
        DeviceKey::MacUnknown,
        DeviceKey::LaptopUnknown,
        DeviceKey::NonLaptop,
        DeviceKey::NonLaptopPhone,
        DeviceKey::NonLaptopTablet,
        DeviceKey::NonLaptopDesktop,
        DeviceKey::NonLaptopConfDevice,
        DeviceKey::NonLaptopMicArray,
        DeviceKey::NonLaptopCloseTalk,
        DeviceKey::NonLaptopStudio,
        DeviceKey::NonLaptopUnknown,
    ];

    pub fn as_str(self) -> String {
        crate::files::serde_name(&self)
    }
}

/// 022 第二节：系统处理模式。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MicMode {
    Standard,
    VoiceIsolation,
    WideSpectrum,
    Unknown,
}

impl MicMode {
    pub const ALL: [MicMode; 4] = [
        MicMode::Standard,
        MicMode::VoiceIsolation,
        MicMode::WideSpectrum,
        MicMode::Unknown,
    ];
}

/// 档：常见档 / 压力档。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Common,
    Stress,
}

/// 只留主说话人 / 近处几人都留（只有 `meeting-room`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClipPolarity {
    Suppress,
    PreserveAll,
}

/// 声音分组：底噪 · 有规律 · 突发 · 旁人语音（含电视、广播）。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum SoundGroup {
    Bed,
    Pattern,
    Event,
    Speech,
}

/// 样本区间 `[起, 止)`。
pub type Iv = [u64; 2];

/// 每条轨一个值（路径或哈希），键与 `audio` 一致；`stems` 的键是声音实例 id。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Tracks {
    pub mix: String,
    /// 混响 + 设备链之后的主说话人：**评测用的干净参考**
    pub owner: String,
    pub owner_dry: String,
    pub floor: String,
    /// 全部非语音分轨 + `floor` 之和（削波前）
    pub noise: String,
    /// 全部旁人语音分轨之和（削波前）
    pub interf: String,
    pub stems: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub spk: String,
    pub dist_m: f64,
    pub azimuth_deg: f64,
    pub level_dbfs: f64,
    pub iv: Vec<Iv>,
    /// 素材出处（说话人 id + 句子列表；防泄漏检查用，016 S10⑦；形状由 scenesuite 定）
    pub src: serde_json::Map<String, serde_json::Value>,
}

/// 设备链第 2 步对一个声源三个频段的方位衰减（dB）。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DirectionDb {
    pub low: f64,
    pub mid: f64,
    pub high: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sound {
    /// 段内唯一；同时是 `audio.stems` 的键
    pub inst: String,
    /// 声音类别键（正本 scenesuite `sounds.json`）
    pub cls: String,
    pub group: SoundGroup,
    pub is_speech: bool,
    pub iv: Vec<Iv>,
    pub level_dbfs: f64,
    pub rel_db: f64,
    /// 扩散声（底噪、babble）为 null
    pub dist_m: Option<f64>,
    pub azimuth_deg: Option<f64>,
    /// 方位衰减；扩散声没有这一项
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction_db: Option<DirectionDb>,
    /// 素材出处（`kind` 加各自字段，形状由 scenesuite 定）
    pub src: serde_json::Map<String, serde_json::Value>,
    pub license: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Nominal {
    pub snr_db: f64,
    /// 无旁人时为 null
    pub sir_db: Option<f64>,
    pub t60_s: f64,
    pub owner_dist_m: f64,
    pub owner_level_dbfs: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Measured {
    /// 只算稳态干扰（底噪 + 有规律的声音 + babble + 设备底噪），同名义值的口径
    pub snr_db: f64,
    /// 把突发声也算进去的 SNR
    pub snr_all_db: f64,
    /// 主说话人实测活动电平（P.56；名义值用 gated 电平，两者最多差约 1.5 dB，按声学条件分档时用名义值）
    pub owner_level_dbfs: f64,
    pub sir_db: Option<f64>,
    pub t60_s: f64,
    pub drr_db: f64,
    pub overlap_ratio: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClipHash {
    /// 样本内容 sha256（float32 小端字节，016 S1）
    pub samples: Tracks,
    /// wav 文件 sha256，`vdn-eval` 逐条核对
    pub wav_sha256: Tracks,
}

/// `clips.jsonl` 的一行。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneClip {
    /// build 内唯一，形如 `cafe-c03`
    pub id: String,
    pub scene: MeetingPlace,
    pub tier: Tier,
    pub device: DeviceKey,
    pub mic_mode: MicMode,
    /// 同一份素材另一设备版本的 id
    pub pair: String,
    pub lang: Lang,
    pub polarity: ClipPolarity,
    pub dur_samples: u64,
    pub sr: u32,
    /// 各轨相对主目录的路径
    pub audio: Tracks,
    pub has_clean_ref: bool,
    /// 分轨相加是否等于 mix（削过波的段为假）
    pub additive: bool,
    pub targets: Vec<Target>,
    pub sounds: Vec<Sound>,
    pub nominal: Nominal,
    pub measured: Measured,
    pub owner_iv: Vec<Iv>,
    pub interf_iv: Vec<Iv>,
    /// 设备链各步实际抽到的参数（形状由 scenesuite 定）
    pub chain: serde_json::Map<String, serde_json::Value>,
    pub seed: u64,
    pub hash: ClipHash,
}

impl SceneClip {
    /// 读一份 `clips.jsonl`（空行跳过）。出错时报行号。
    pub fn parse_jsonl(text: &str) -> Result<Vec<SceneClip>, String> {
        let mut out = Vec::new();
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let c: SceneClip =
                serde_json::from_str(line).map_err(|e| format!("第 {} 行：{e}", i + 1))?;
            c.check()
                .map_err(|e| format!("第 {} 行（{}）：{e}", i + 1, c.id))?;
            out.push(c);
        }
        let mut ids: Vec<&str> = out.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        if let Some(w) = ids.windows(2).find(|w| w[0] == w[1]) {
            return Err(format!("片段 id 重复：{}", w[0]));
        }
        Ok(out)
    }

    /// schema 表达不了的约束：实例 id 唯一且与 `stems` 的键一一对应；区间合法；采样率 16 kHz。
    pub fn check(&self) -> Result<(), String> {
        if self.sr != vdn_types::MODEL_RATE {
            return Err(format!("采样率 {} ≠ {}", self.sr, vdn_types::MODEL_RATE));
        }
        let mut inst: Vec<&str> = self.sounds.iter().map(|s| s.inst.as_str()).collect();
        inst.sort_unstable();
        let stems: Vec<&str> = self.audio.stems.keys().map(String::as_str).collect();
        if inst != stems {
            return Err(format!(
                "sounds 的实例 {inst:?} ≠ audio.stems 的键 {stems:?}"
            ));
        }
        for t in [&self.hash.samples, &self.hash.wav_sha256] {
            if t.stems.keys().map(String::as_str).collect::<Vec<_>>() != stems {
                return Err("hash 的 stems 键与 audio.stems 不一致".into());
            }
        }
        let n = self.dur_samples;
        let all_iv = self
            .owner_iv
            .iter()
            .chain(&self.interf_iv)
            .chain(self.sounds.iter().flat_map(|s| &s.iv))
            .chain(self.targets.iter().flat_map(|t| &t.iv));
        for iv in all_iv {
            if iv[0] >= iv[1] || iv[1] > n {
                return Err(format!("区间 {iv:?} 不合法（段长 {n}）"));
            }
        }
        if self.polarity == ClipPolarity::Suppress && self.targets.len() != 1 {
            return Err(format!(
                "suppress 段要 1 个 target，实际 {}",
                self.targets.len()
            ));
        }
        Ok(())
    }
}
