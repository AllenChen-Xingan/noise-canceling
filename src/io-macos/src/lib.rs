//! vdn-io-macos：链路的两头——麦克风进、扬声器出（开发期；正式版出口是虚拟麦克风 `vdn-hal`）。
//!
//! 约定（`docs/plan/010_rust-skeleton.md` 第十节）：
//! - 单声道 f32 语义：多声道输入取第 1 声道；输出复制到各声道
//! - IO 采样率固定 48 000 Hz（[`IO_RATE`]）；设备不支持就返回 [`IoError::UnsupportedRate`]，写明设备支持什么。本批不重采样
//! - 返回的流句柄 drop 即停流
//! - 本批底层是 cpal（规约 §4：P3 允许）。cpal 回调里的已知违规见 `state/handoff/rs-io.md`；M1 起换自写 IOProc
//!
//! 实时代码只在 `pump` 模块的 `#[deny(clippy::disallowed_*)]` 块里；其余是构造期。

#![forbid(unsafe_code)]
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

pub mod level;
pub mod pump;
pub mod stream;

use std::fmt;

use cpal::traits::{DeviceTrait, HostTrait};

pub use level::{Level, LevelMeter, SILENT_RUN_SAMPLES, dbfs, fmt_dbfs};
pub use stream::{InputStream, OpenedInfo, OutputStream, StreamHealth};

/// 链路 IO 采样率（Hz）。正本在 `vdn_types::rate`，这里只转出，下游路径不变。
pub use vdn_types::IO_RATE;

/// 设备方向。一个既能收又能放的设备在 [`devices`] 里出现两次，一个方向一行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::Input => "输入",
            Direction::Output => "输出",
        })
    }
}

/// 一个设备在一个方向上的信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub direction: Direction,
    /// 默认配置的采样率（Hz）；取不到为 0
    pub default_rate: u32,
    /// 默认配置的声道数；取不到为 0
    pub channels: u16,
    /// 是否为系统该方向的默认设备
    pub is_default: bool,
    /// 是否有一档配置覆盖 48 kHz（本批开流的前提）
    pub supports_io_rate: bool,
}

/// 设备支持的一档采样率范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateRange {
    pub min: u32,
    pub max: u32,
    pub channels: u16,
}

impl fmt::Display for RateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.min == self.max {
            write!(f, "{} Hz × {} 声道", self.min, self.channels)
        } else {
            write!(f, "{}–{} Hz × {} 声道", self.min, self.max, self.channels)
        }
    }
}

/// 本 crate 的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoError {
    /// 参数不对（采样率 0、块长 0 或过大）
    InvalidParam(String),
    /// 系统没有该方向的默认设备
    NoDefaultDevice(Direction),
    /// 按名字找不到（或不唯一）
    DeviceNotFound {
        wanted: String,
        direction: Direction,
        candidates: Vec<String>,
    },
    /// 设备不支持请求的采样率（本批不重采样，下一批做）
    UnsupportedRate {
        device: String,
        wanted: u32,
        supported: Vec<RateRange>,
    },
    /// cpal / Core Audio 报的错（含权限被拒、设备忙）
    Backend(String),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::InvalidParam(s) => write!(f, "参数不对：{s}"),
            IoError::NoDefaultDevice(d) => write!(f, "系统没有默认{d}设备"),
            IoError::DeviceNotFound {
                wanted,
                direction,
                candidates,
            } => write!(
                f,
                "找不到唯一的{direction}设备「{wanted}」；可选：{}",
                candidates.join("、")
            ),
            IoError::UnsupportedRate {
                device,
                wanted,
                supported,
            } => {
                write!(
                    f,
                    "设备「{device}」不支持 {wanted} Hz（本批不重采样）；它支持："
                )?;
                if supported.is_empty() {
                    f.write_str("（设备没报任何配置）")
                } else {
                    let list: Vec<String> = supported.iter().map(ToString::to_string).collect();
                    f.write_str(&list.join("；"))
                }
            }
            IoError::Backend(s) => write!(f, "音频后端出错：{s}"),
        }
    }
}

impl std::error::Error for IoError {}

/// 列出本机所有输入 / 输出设备（不开流，不弹权限框）。取设备列表失败返回空表；单个设备读不到参数时该项填 0。
pub fn devices() -> Vec<DeviceInfo> {
    let host = cpal::default_host();
    let def_in = host.default_input_device();
    let def_out = host.default_output_device();
    let Ok(all) = host.devices() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for d in all {
        let name = device_name(&d);
        if let Ok(ranges) = d.supported_input_configs() {
            let ranges: Vec<_> = ranges.collect();
            if !ranges.is_empty() {
                let def = d.default_input_config().ok();
                out.push(DeviceInfo {
                    name: name.clone(),
                    direction: Direction::Input,
                    default_rate: def.as_ref().map_or(0, |c| c.sample_rate()),
                    channels: def.as_ref().map_or(0, |c| c.channels()),
                    is_default: def_in.as_ref() == Some(&d),
                    supports_io_rate: ranges
                        .iter()
                        .any(|r| r.min_sample_rate() <= IO_RATE && IO_RATE <= r.max_sample_rate()),
                });
            }
        }
        if let Ok(ranges) = d.supported_output_configs() {
            let ranges: Vec<_> = ranges.collect();
            if !ranges.is_empty() {
                let def = d.default_output_config().ok();
                out.push(DeviceInfo {
                    name,
                    direction: Direction::Output,
                    default_rate: def.as_ref().map_or(0, |c| c.sample_rate()),
                    channels: def.as_ref().map_or(0, |c| c.channels()),
                    is_default: def_out.as_ref() == Some(&d),
                    supports_io_rate: ranges
                        .iter()
                        .any(|r| r.min_sample_rate() <= IO_RATE && IO_RATE <= r.max_sample_rate()),
                });
            }
        }
    }
    out
}

/// 设备显示名。读不到描述时退回 cpal 的 `Display`。
pub(crate) fn device_name(d: &cpal::Device) -> String {
    d.description()
        .map(|desc| desc.name().to_string())
        .unwrap_or_else(|_| d.to_string())
}
