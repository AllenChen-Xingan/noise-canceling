//! 开流（非实时，构造期）：选设备 → 查采样率 → 建 cpal 流 → 播放。回调本体在 `pump.rs`。
//!
//! 设备拔掉、流作废：cpal 调错误回调（不在实时线程），这里只计数并记下最后一次错误的种类，
//! 不 panic、不重建（重建归后续批次，003「重建入口」）。调用方读 `health()` 自己决定怎么办。
//!
//! 用户回调跑在 Core Audio 的实时线程上：不许分配、加锁、阻塞、panic。万一 panic，回调泵接住并静音（见 `pump.rs`）。
//!
//! 副作用（cpal 行为，读源码确认）：开流会把**设备的全局**采样率设成 `rate`，`block` 被设备接受时还会改设备全局的
//! 硬件缓冲帧数——同一设备上的其他程序（会议软件）也受影响；开流失败或 drop 后不还原。还原留到自写 IOProc 那批。
//! `device = None` 打开的默认输出流会跟着系统默认输出设备切换（cpal 报 `DeviceChanged`），之后 `info().device` 是旧名字；
//! 按名字打开的流不跟随。

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::level::{Level, LevelMeter};
use crate::pump::{InputPump, OutputPump};
use crate::{Direction, IoError, RateRange};

/// 块长上限（帧）：1 秒 @ 48 kHz。超过说明调用方传错了。
pub const MAX_BLOCK: usize = crate::IO_RATE as usize;

/// 流的健康计数：错误回调与回调泵写，任何线程读。全部是原子量：cpal 有时在实时线程上调错误回调。
#[derive(Debug, Default)]
pub struct StreamHealth {
    errors: AtomicU64,
    xruns: AtomicU64,
    device_lost: AtomicBool,
    needs_reopen: AtomicBool,
    device_changed: AtomicBool,
    callback_panicked: AtomicBool,
    /// 回调泵看到的单声道电平（输入：取第 1 声道之后；输出：用户回调写完之后、复制到各声道之前）
    level: LevelMeter,
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl StreamHealth {
    /// 错误回调被调用的总次数（含 xrun）。
    pub fn errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
    /// 其中 xrun（处理器过载、丢块）的次数。xrun 后流仍在跑。
    pub fn xruns(&self) -> u64 {
        self.xruns.load(Ordering::Relaxed)
    }
    /// 设备已断开（拔掉、被系统收回）。
    pub fn device_lost(&self) -> bool {
        self.device_lost.load(Ordering::Relaxed)
    }
    /// 流已停、要重新 `open`：设备断开，或设备采样率被别的程序改了（cpal 报 `StreamInvalidated` 并暂停流）。
    pub fn needs_reopen(&self) -> bool {
        self.needs_reopen.load(Ordering::Relaxed)
    }
    /// 默认设备流跟着系统默认设备换了设备；`info().device` 已过时。
    pub fn device_changed(&self) -> bool {
        self.device_changed.load(Ordering::Relaxed)
    }
    /// 用户回调 panic 过；之后不再调用它（输入丢弃、输出静音）。
    pub fn callback_panicked(&self) -> bool {
        self.callback_panicked.load(Ordering::Relaxed)
    }
    /// 电平读数（峰值 · 均方根 · 连续全零长度）。
    pub fn level(&self) -> Level {
        self.level.snapshot()
    }
    /// 现在正连续全零满 1 s。输入流上的意思是「疑似没有麦克风权限或设备静音」
    /// （系统设置 › 隐私与安全性 › 麦克风）；输出流上只表示一直在放静音
    pub fn silent_now(&self) -> bool {
        self.level.snapshot().silent_now()
    }
    pub(crate) fn meter(&self) -> &LevelMeter {
        &self.level
    }
    pub(crate) fn mark_panicked(&self) {
        self.callback_panicked.store(true, Ordering::Relaxed);
    }
    pub(crate) fn record(&self, kind: cpal::ErrorKind) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        match kind {
            cpal::ErrorKind::Xrun => {
                self.xruns.fetch_add(1, Ordering::Relaxed);
            }
            cpal::ErrorKind::DeviceNotAvailable => {
                self.device_lost.store(true, Ordering::Relaxed);
                self.needs_reopen.store(true, Ordering::Relaxed);
            }
            cpal::ErrorKind::StreamInvalidated => {
                self.needs_reopen.store(true, Ordering::Relaxed);
            }
            cpal::ErrorKind::DeviceChanged => {
                self.device_changed.store(true, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

/// 麦克风输入流。drop 即停流。
pub struct InputStream {
    _stream: cpal::Stream,
    info: OpenedInfo,
    health: Arc<StreamHealth>,
}

/// 扬声器输出流。drop 即停流。
pub struct OutputStream {
    _stream: cpal::Stream,
    info: OpenedInfo,
    health: Arc<StreamHealth>,
}

/// 实际打开的设备与参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedInfo {
    pub device: String,
    pub rate: u32,
    /// 设备声道数（输入取第 1 声道；输出复制到全部声道）
    pub channels: u16,
    /// 用户回调每次最多拿到的帧数
    pub block: usize,
    /// 设备硬件缓冲帧数：`Some(block)` = 设成了 block；`None` = 设备不接受，用了设备默认
    pub device_buffer: Option<u32>,
}

impl fmt::Debug for InputStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputStream")
            .field("info", &self.info)
            .finish()
    }
}
impl fmt::Debug for OutputStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutputStream")
            .field("info", &self.info)
            .finish()
    }
}

impl InputStream {
    /// 打开麦克风。`device = None` 用系统默认输入；`Some(名)` 先精确匹配，再找唯一一个包含该名（不分大小写）的设备。
    /// 回调拿到单声道 f32，每次 ≤ `block` 帧（设备一次给得多就切成几段）。
    ///
    /// ⚠️ 会触发 macOS 麦克风权限框。
    pub fn open(
        device: Option<&str>,
        rate: u32,
        block: usize,
        cb: impl FnMut(&[f32]) + Send + 'static,
    ) -> Result<Self, IoError> {
        validate(rate, block)?;
        let host = cpal::default_host();
        let dev = pick(&host, Direction::Input, device)?;
        let (info, config) = plan(&dev, Direction::Input, rate, block)?;
        let mut pump = InputPump::new(block, usize::from(info.channels), cb);
        let health = pump.health();
        let h = pump.health();
        let stream = dev
            .build_input_stream::<f32, _, _>(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| pump.on_data(data),
                move |e: cpal::Error| h.record(e.kind()),
                None,
            )
            .map_err(|e| IoError::Backend(e.to_string()))?;
        stream.play().map_err(|e| IoError::Backend(e.to_string()))?;
        Ok(Self {
            _stream: stream,
            info,
            health,
        })
    }

    pub fn info(&self) -> &OpenedInfo {
        &self.info
    }
    pub fn health(&self) -> &StreamHealth {
        &self.health
    }
}

impl OutputStream {
    /// 打开扬声器。回调拿到清零的单声道块（≤ `block` 帧）去写，写进去的复制到设备每个声道。
    ///
    /// ⚠️ 会出声。
    pub fn open(
        device: Option<&str>,
        rate: u32,
        block: usize,
        cb: impl FnMut(&mut [f32]) + Send + 'static,
    ) -> Result<Self, IoError> {
        validate(rate, block)?;
        let host = cpal::default_host();
        let dev = pick(&host, Direction::Output, device)?;
        let (info, config) = plan(&dev, Direction::Output, rate, block)?;
        let mut pump = OutputPump::new(block, usize::from(info.channels), cb);
        let health = pump.health();
        let h = pump.health();
        let stream = dev
            .build_output_stream::<f32, _, _>(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| pump.on_data(data),
                move |e: cpal::Error| h.record(e.kind()),
                None,
            )
            .map_err(|e| IoError::Backend(e.to_string()))?;
        stream.play().map_err(|e| IoError::Backend(e.to_string()))?;
        Ok(Self {
            _stream: stream,
            info,
            health,
        })
    }

    pub fn info(&self) -> &OpenedInfo {
        &self.info
    }
    pub fn health(&self) -> &StreamHealth {
        &self.health
    }
}

/// 不碰设备的参数校验。
pub fn validate(rate: u32, block: usize) -> Result<(), IoError> {
    if rate == 0 {
        return Err(IoError::InvalidParam("采样率不能为 0".into()));
    }
    if block == 0 || block > MAX_BLOCK {
        return Err(IoError::InvalidParam(format!(
            "块长 {block} 帧不在 1..={MAX_BLOCK} 内"
        )));
    }
    Ok(())
}

/// 按名字选设备。名字匹配规则见 [`match_name`]。
fn pick(host: &cpal::Host, dir: Direction, wanted: Option<&str>) -> Result<cpal::Device, IoError> {
    let Some(wanted) = wanted else {
        let d = match dir {
            Direction::Input => host.default_input_device(),
            Direction::Output => host.default_output_device(),
        };
        return d.ok_or(IoError::NoDefaultDevice(dir));
    };
    let devs: Vec<(String, cpal::Device)> = host
        .devices()
        .map_err(|e| IoError::Backend(e.to_string()))?
        .filter(|d| match dir {
            Direction::Input => d.supports_input(),
            Direction::Output => d.supports_output(),
        })
        .map(|d| (crate::device_name(&d), d))
        .collect();
    let names: Vec<&str> = devs.iter().map(|(n, _)| n.as_str()).collect();
    match match_name(&names, wanted) {
        Ok(i) => devs
            .into_iter()
            .nth(i)
            .map(|(_, d)| d)
            .ok_or_else(|| IoError::Backend("设备下标越界".into())),
        Err(candidates) => Err(IoError::DeviceNotFound {
            wanted: wanted.to_string(),
            direction: dir,
            candidates,
        }),
    }
}

/// 名字匹配：空名不匹配；先精确相等（恰好一个）；否则找不分大小写包含 `wanted` 的，恰好一个才算。
/// 失败返回「供人挑的候选名」：有多个包含匹配时列这几个，一个都没有时列全部。
pub fn match_name(names: &[&str], wanted: &str) -> Result<usize, Vec<String>> {
    if wanted.trim().is_empty() {
        return Err(names.iter().map(|s| s.to_string()).collect());
    }
    let exact: Vec<usize> = names
        .iter()
        .enumerate()
        .filter(|(_, n)| **n == wanted)
        .map(|(i, _)| i)
        .collect();
    match exact.as_slice() {
        [i] => return Ok(*i),
        [] => {}
        // 两台同名设备（如两个同型号 USB 麦克风）：不猜
        many => {
            return Err(many
                .iter()
                .filter_map(|&i| names.get(i))
                .map(|s| s.to_string())
                .collect());
        }
    }
    let w = wanted.to_lowercase();
    let hits: Vec<usize> = names
        .iter()
        .enumerate()
        .filter(|(_, n)| n.to_lowercase().contains(&w))
        .map(|(i, _)| i)
        .collect();
    match hits.as_slice() {
        [i] => Ok(*i),
        [] => Err(names.iter().map(|s| s.to_string()).collect()),
        many => Err(many
            .iter()
            .filter_map(|&i| names.get(i))
            .map(|s| s.to_string())
            .collect()),
    }
}

/// 查设备支持的配置：要有覆盖 `rate` 的一档，声道数取默认配置的声道数（取不到就取那一档的）。
fn plan(
    dev: &cpal::Device,
    dir: Direction,
    rate: u32,
    block: usize,
) -> Result<(OpenedInfo, cpal::StreamConfig), IoError> {
    let name = crate::device_name(dev);
    let ranges: Vec<cpal::SupportedStreamConfigRange> = match dir {
        Direction::Input => dev.supported_input_configs().map(|it| it.collect()),
        Direction::Output => dev.supported_output_configs().map(|it| it.collect()),
    }
    .map_err(|e| IoError::Backend(e.to_string()))?;
    let default_channels = match dir {
        Direction::Input => dev.default_input_config(),
        Direction::Output => dev.default_output_config(),
    }
    .ok()
    .map(|c| c.channels());
    let supported: Vec<RateRange> = ranges
        .iter()
        .map(|r| RateRange {
            min: r.min_sample_rate(),
            max: r.max_sample_rate(),
            channels: r.channels(),
        })
        .collect();
    let channels = choose_channels(&supported, rate, default_channels).ok_or_else(|| {
        IoError::UnsupportedRate {
            device: name.clone(),
            wanted: rate,
            supported: supported.clone(),
        }
    })?;
    let frames =
        u32::try_from(block).map_err(|_| IoError::InvalidParam(format!("块长 {block} 太大")))?;
    // 设备硬件缓冲能设成 block 就设；设不了用设备默认（回调泵照样按 ≤ block 切段），不因此开流失败
    let buf_range = ranges
        .iter()
        .find(|r| {
            r.channels() == channels && r.min_sample_rate() <= rate && rate <= r.max_sample_rate()
        })
        .and_then(|r| match r.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => Some((*min, *max)),
            cpal::SupportedBufferSize::Unknown => None,
        });
    let buffer_size = choose_buffer(buf_range, frames);
    let config = cpal::StreamConfig {
        channels,
        sample_rate: rate,
        buffer_size,
    };
    Ok((
        OpenedInfo {
            device: name,
            rate,
            channels,
            block,
            device_buffer: match buffer_size {
                cpal::BufferSize::Fixed(f) => Some(f),
                cpal::BufferSize::Default => None,
            },
        },
        config,
    ))
}

/// 设备硬件缓冲长度：`frames` 落在设备报的范围里就固定为它，否则（或范围未知）用设备默认。
pub fn choose_buffer(range: Option<(u32, u32)>, frames: u32) -> cpal::BufferSize {
    match range {
        Some((min, max)) if min <= frames && frames <= max => cpal::BufferSize::Fixed(frames),
        _ => cpal::BufferSize::Default,
    }
}

/// 在支持表里挑声道数：优先默认声道数（且该声道数有一档覆盖 `rate`），否则第一档覆盖 `rate` 的。
/// 没有任何一档覆盖 `rate` 返回 `None`（本批不重采样）。
pub fn choose_channels(
    supported: &[RateRange],
    rate: u32,
    default_channels: Option<u16>,
) -> Option<u16> {
    let covers = |r: &&RateRange| r.min <= rate && rate <= r.max && r.channels > 0;
    if let Some(dc) = default_channels
        && supported.iter().filter(covers).any(|r| r.channels == dc)
    {
        return Some(dc);
    }
    supported.iter().find(covers).map(|r| r.channels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IO_RATE;

    #[test]
    fn health_latches_by_error_kind() {
        let h = StreamHealth::default();
        h.record(cpal::ErrorKind::Xrun);
        assert_eq!((h.errors(), h.xruns(), h.needs_reopen()), (1, 1, false));
        h.record(cpal::ErrorKind::StreamInvalidated);
        assert!(h.needs_reopen() && !h.device_lost());
        h.record(cpal::ErrorKind::DeviceChanged);
        assert!(h.device_changed());
        h.record(cpal::ErrorKind::DeviceNotAvailable);
        assert!(h.device_lost() && h.needs_reopen());
        assert_eq!(h.errors(), 4);
    }

    #[test]
    fn validate_rejects_bad_params() {
        assert!(validate(IO_RATE, 480).is_ok());
        assert!(matches!(validate(0, 480), Err(IoError::InvalidParam(_))));
        assert!(matches!(
            validate(IO_RATE, 0),
            Err(IoError::InvalidParam(_))
        ));
        assert!(matches!(
            validate(IO_RATE, MAX_BLOCK + 1),
            Err(IoError::InvalidParam(_))
        ));
    }

    #[test]
    fn choose_buffer_falls_back_to_default() {
        use cpal::BufferSize::{Default, Fixed};
        assert_eq!(choose_buffer(Some((15, 4096)), 480), Fixed(480));
        assert_eq!(choose_buffer(Some((512, 4096)), 480), Default);
        assert_eq!(choose_buffer(None, 480), Default);
    }

    #[test]
    fn match_name_exact_then_unique_substring() {
        let names = ["MacBook Pro Microphone", "MacBook Pro Speakers", "AirPods"];
        assert_eq!(match_name(&names, "AirPods"), Ok(2));
        assert_eq!(match_name(&names, "microphone"), Ok(0));
        // 两个都含 "macbook"：不猜，列出这两个
        assert_eq!(
            match_name(&names, "macbook"),
            Err(vec![
                "MacBook Pro Microphone".to_string(),
                "MacBook Pro Speakers".to_string()
            ])
        );
        // 一个都没有：列全部
        assert_eq!(match_name(&names, "zzz").map_err(|v| v.len()), Err(3));
        // 空名不匹配任何设备
        assert!(match_name(&names, "  ").is_err());
        // 两台同名：不挑第一个
        let dup = ["USB Mic", "USB Mic"];
        assert_eq!(match_name(&dup, "USB Mic").map_err(|v| v.len()), Err(2));
    }

    #[test]
    fn choose_channels_prefers_default_and_needs_rate() {
        let sup = [
            RateRange {
                min: 44_100,
                max: 44_100,
                channels: 2,
            },
            RateRange {
                min: IO_RATE,
                max: IO_RATE,
                channels: 1,
            },
            RateRange {
                min: 8_000,
                max: 96_000,
                channels: 2,
            },
        ];
        assert_eq!(choose_channels(&sup, IO_RATE, Some(2)), Some(2));
        assert_eq!(choose_channels(&sup, IO_RATE, Some(6)), Some(1));
        assert_eq!(choose_channels(&sup, IO_RATE, None), Some(1));
        let only44 = [RateRange {
            min: 44_100,
            max: 44_100,
            channels: 2,
        }];
        assert_eq!(choose_channels(&only44, IO_RATE, Some(2)), None);
    }

    #[test]
    fn unsupported_rate_error_lists_what_device_supports() {
        let e = IoError::UnsupportedRate {
            device: "某麦克风".into(),
            wanted: IO_RATE,
            supported: vec![RateRange {
                min: 44_100,
                max: 44_100,
                channels: 1,
            }],
        };
        let s = e.to_string();
        assert!(
            s.contains("48000") && s.contains("44100") && s.contains("某麦克风"),
            "{s}"
        );
    }
}
