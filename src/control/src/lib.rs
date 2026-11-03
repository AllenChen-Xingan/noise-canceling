//! vdn-control：配置文件（`Config`）与控制消息（`Cmd`）。
//!
//! - 档：C（非实时）。serde 只许出现在这里和 bin 里（specs/rust/architecture.md §2.2）
//! - `Cmd` 是 `Copy` 小枚举，可以经无锁通道交给实时线程；`Config` 只在启动时读
//! - 不认识引擎、模型、设备库：模式与模型在这里只是名字，由 bin 翻成各 crate 的类型

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

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// IO 采样率固定 48 kHz（docs/plan/010 第十节）。
pub const IO_RATE: u32 = 48_000;

/// 处理模式。与 `vdn_engine::Mode` 一一对应，由 bin 翻译。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    #[default]
    Denoise,
    Passthrough,
}

/// 用哪个模型。与 `vdn_models::ModelKind` 一一对应，由 bin 翻译。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ModelChoice {
    #[default]
    Gtcrn,
    FastenhancerM,
}

/// 控制消息：小、`Copy`，可放进无锁队列交给实时线程。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    SetMode(Mode),
    Shutdown,
}

/// 配置文件（JSON）。缺的字段取默认值；多出的字段报错（防拼错键名静默失效）。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// 输入设备名；`None` = 系统默认
    pub input_device: Option<String>,
    /// 输出设备名；`None` = 系统默认
    pub output_device: Option<String>,
    /// 块长（48 kHz 样本数）
    pub block: usize,
    pub mode: Mode,
    pub model: ModelChoice,
    /// 模型文件；`None` = 按 vdn-models 的默认路径查找
    pub model_path: Option<PathBuf>,
    /// 降噪总线名（降噪进程建；Raw / Processed / 心跳）。规则见 `validate_bus_name`
    pub bus_name: String,
    /// Out 总线名（送音进程建，只用 Out 环；虚拟麦克风插件按这个名字连）
    pub out_bus_name: String,
    /// 心跳多久不动就切原声（ms）
    pub heartbeat_timeout_ms: u32,
    /// 环余量（48 kHz 样本）：降噪进程开头往 Processed 先推这么多个零，让送音端每块都取得到。
    /// 链路总延迟 `L_total` = 管线延迟 + 本值（默认见 `DEFAULT_LEAD`）
    pub lead_frames: usize,
    /// 降噪进程每次从 Raw 取多少样本跑一次 Pipeline（48 kHz）；须整除模型 hop × 3（GTCRN 768），
    /// 不整除由 vdn-denoised 启动时报错（人定省 2 ms，2026-11-02）
    pub denoise_block: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_device: None,
            output_device: None,
            block: Self::DEFAULT_BLOCK,
            mode: Mode::Denoise,
            model: ModelChoice::Gtcrn,
            model_path: None,
            bus_name: Self::DEFAULT_BUS.to_owned(),
            out_bus_name: Self::DEFAULT_OUT_BUS.to_owned(),
            heartbeat_timeout_ms: Self::DEFAULT_HEARTBEAT_MS,
            lead_frames: Self::DEFAULT_LEAD,
            denoise_block: Self::DEFAULT_DENOISE_BLOCK,
        }
    }
}

impl Config {
    /// 2 ms @ 48 kHz（规约提议的 IO 块；480 时环余量至少要盖住 10 ms，人定修延迟 2026-11-02）
    pub const DEFAULT_BLOCK: usize = 96;
    pub const DEFAULT_BUS: &'static str = "/vdn-bus";
    pub const DEFAULT_OUT_BUS: &'static str = "/vdn-out";
    /// 192 = 4 ms：**D68：低频运行，V7 过线值**（人定，不追提频；2026-11-02）。release、有唤醒时
    /// 空闲 / 压力各 10 次 × 10 s 无欠载；144 空闲 10/10 有欠载（`docs/experiments/exp-v7-latency/`）
    pub const DEFAULT_LEAD: usize = 192;
    /// 2 ms @ 48 kHz
    pub const DEFAULT_DENOISE_BLOCK: usize = 96;
    /// 环余量上限（0.5 s）
    pub const LEAD_MAX: usize = 24_000;
    /// 共享内存名前缀（vdn-shm 的规则）
    pub const BUS_PREFIX: &'static str = "/vdn-";
    /// docs/plan/010 第十节：心跳超过 100 ms 不动就切原声
    pub const DEFAULT_HEARTBEAT_MS: u32 = 100;
    /// 块长范围：32 样本（0.67 ms）到 4800 样本（100 ms）
    pub const BLOCK_RANGE: (usize, usize) = (32, 4800);
    /// 心跳超时范围（ms）
    pub const HEARTBEAT_RANGE_MS: (u32, u32) = (10, 2000);
    /// macOS 的 POSIX 共享内存名上限（PSHMNAMLEN）
    pub const BUS_NAME_MAX: usize = 31;

    /// 从 JSON 文本读，并校验。
    pub fn from_json(text: &str) -> Result<Self, ConfigError> {
        let c: Self = serde_json::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        c.validate()?;
        Ok(c)
    }

    /// 从文件读，并校验。
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Io(format!("{}：{e}", path.display())))?;
        Self::from_json(&text)
    }

    /// 写成带缩进的 JSON。
    pub fn to_json(&self) -> String {
        // 只含字符串、数字、枚举，序列化不会失败；万一失败返回空对象
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_owned())
    }

    /// 心跳超时换成 48 kHz 样本数。
    pub fn heartbeat_timeout_samples(&self) -> usize {
        self.heartbeat_timeout_ms as usize * IO_RATE as usize / 1000
    }

    /// 校验取值范围。
    pub fn validate(&self) -> Result<(), ConfigError> {
        let (lo, hi) = Self::BLOCK_RANGE;
        if !(lo..=hi).contains(&self.block) {
            return Err(ConfigError::Invalid(format!(
                "block = {}，须在 {lo}..={hi}",
                self.block
            )));
        }
        let (lo, hi) = Self::HEARTBEAT_RANGE_MS;
        if !(lo..=hi).contains(&self.heartbeat_timeout_ms) {
            return Err(ConfigError::Invalid(format!(
                "heartbeat_timeout_ms = {}，须在 {lo}..={hi}",
                self.heartbeat_timeout_ms
            )));
        }
        if !(1..=hi_block()).contains(&self.denoise_block) {
            return Err(ConfigError::Invalid(format!(
                "denoise_block = {}，须在 1..={}",
                self.denoise_block,
                hi_block()
            )));
        }
        check_lead(self.lead_frames, self.block)?;
        if self.lead_frames > Self::LEAD_MAX {
            return Err(ConfigError::Invalid(format!(
                "lead_frames = {}，须 ≤ {}",
                self.lead_frames,
                Self::LEAD_MAX
            )));
        }
        validate_bus_name(&self.bus_name)?;
        validate_bus_name(&self.out_bus_name)?;
        if self.bus_name == self.out_bus_name {
            return Err(ConfigError::Invalid(format!(
                "bus_name 和 out_bus_name 不能相同（{}）",
                self.bus_name
            )));
        }
        for (k, v) in [
            ("input_device", &self.input_device),
            ("output_device", &self.output_device),
        ] {
            if v.as_deref().is_some_and(|s| s.trim().is_empty()) {
                return Err(ConfigError::Invalid(format!(
                    "{k} 是空字符串；要用系统默认请删掉这个键"
                )));
            }
        }
        Ok(())
    }
}

fn hi_block() -> usize {
    Config::BLOCK_RANGE.1
}

/// 环余量的下限（正本；Config 校验、chain-test、送音端读总线头都调它）：**至少一块 IO**。
/// 送音端每个回调「推一块原声、立刻取一块处理后的」，lead 少于一块时每块的后段都取不到，必欠载
/// （实测：IO 块 480 配 lead 288，每块都用原声补）。不要求再加一块降噪：V7 基线里 IO 块 96、降噪块 96、
/// lead 96 / 144 在压力组 9/10、10/10 次无欠载，空闲时欠载的原因是处理变慢，不是这条线
pub fn check_lead(lead: usize, io_block: usize) -> Result<(), ConfigError> {
    if lead < io_block {
        return Err(ConfigError::Invalid(format!(
            "lead_frames = {lead} 小于 IO 块长 {io_block}：送音端每块都会取不到处理后的；lead 至少要 {io_block}"
        )));
    }
    Ok(())
}

/// 命令行里的秒数：有限、≥ 0、≤ 1 000 000（`Duration::from_secs_f64` 遇负数 / NaN 会 panic）。给 clap 的 `value_parser` 用。
pub fn parse_seconds(s: &str) -> Result<f64, String> {
    let v: f64 = s.trim().parse().map_err(|_| format!("{s:?} 不是数字"))?;
    if v.is_finite() && (0.0..=1_000_000.0).contains(&v) {
        Ok(v)
    } else {
        Err(format!("{s:?}：须在 0 到 1000000 秒之间"))
    }
}

/// 共享内存名：`/vdn-` 开头、后面非空且不再含 `/`、总长 ≤ 31（与 vdn-shm 同一规则）。
pub fn validate_bus_name(name: &str) -> Result<(), ConfigError> {
    let ok = name.len() <= Config::BUS_NAME_MAX
        && name
            .strip_prefix(Config::BUS_PREFIX)
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('/') && !rest.contains('\0'));
    if ok {
        Ok(())
    } else {
        Err(ConfigError::Invalid(format!(
            "bus_name = {name:?}：须以 {} 开头、只含一个 /、总长 ≤ {}",
            Config::BUS_PREFIX,
            Config::BUS_NAME_MAX
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    Invalid(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(s) => write!(f, "读配置失败：{s}"),
            Self::Parse(s) => write!(f, "配置不是合法 JSON：{s}"),
            Self::Invalid(s) => write!(f, "配置取值不对：{s}"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_valid_and_roundtrips() {
        let c = Config::default();
        c.validate().unwrap();
        assert_eq!(Config::from_json(&c.to_json()).unwrap(), c);
        assert_eq!(c.heartbeat_timeout_samples(), 4800);
    }

    #[test]
    fn missing_fields_take_defaults() {
        // block 取得比默认 lead 小（lead ≥ 一块 IO 的规则）
        let c = Config::from_json(r#"{"mode":"passthrough","block":128}"#).unwrap();
        assert_eq!(c.mode, Mode::Passthrough);
        assert_eq!(c.block, 128);
        assert_eq!(c.lead_frames, Config::DEFAULT_LEAD);
        assert_eq!(c.bus_name, Config::DEFAULT_BUS);
        assert_eq!(c.model, ModelChoice::Gtcrn);
    }

    #[test]
    fn unknown_key_rejected() {
        assert!(matches!(
            Config::from_json(r#"{"blok":256}"#),
            Err(ConfigError::Parse(_))
        ));
    }

    #[test]
    fn model_names() {
        let c = Config::from_json(r#"{"model":"fastenhancer-m"}"#).unwrap();
        assert_eq!(c.model, ModelChoice::FastenhancerM);
    }

    #[test]
    fn ranges_checked() {
        for bad in [
            r#"{"block":0}"#,
            r#"{"block":100000}"#,
            r#"{"heartbeat_timeout_ms":1}"#,
            r#"{"bus_name":"vdn"}"#,
            r#"{"bus_name":"/"}"#,
            r#"{"bus_name":"/vdn-"}"#,
            r#"{"bus_name":"/other"}"#,
            r#"{"lead_frames":100000}"#,
            r#"{"denoise_block":0}"#,
            r#"{"out_bus_name":"/vdn-bus"}"#,
            r#"{"out_bus_name":"out"}"#,
            r#"{"bus_name":"/vdn-a/b"}"#,
            r#"{"bus_name":"/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
            r#"{"input_device":"  "}"#,
        ] {
            assert!(
                matches!(Config::from_json(bad), Err(ConfigError::Invalid(_))),
                "{bad}"
            );
        }
    }

    /// lead 小于一块 IO 在配置这一层就挡住（以前要等测出欠载才发现）
    #[test]
    fn lead_below_one_io_block_is_rejected() {
        let e = Config::from_json(r#"{"block":480,"lead_frames":288}"#);
        assert!(
            matches!(&e, Err(ConfigError::Invalid(m)) if m.contains("小于 IO 块长")),
            "{e:?}"
        );
        assert!(Config::from_json(r#"{"lead_frames":64}"#).is_err());
        // 刚好一块可以；默认值满足
        assert!(Config::from_json(r#"{"block":96,"lead_frames":96}"#).is_ok());
        let d = Config::default();
        assert!(check_lead(d.lead_frames, d.block).is_ok());
    }

    #[test]
    fn seconds_parser() {
        assert_eq!(parse_seconds("2.5"), Ok(2.5));
        for bad in ["-1", "NaN", "inf", "x", "1e9"] {
            assert!(parse_seconds(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn cmd_is_copy() {
        fn need_copy<T: Copy>(_: T) {}
        need_copy(Cmd::SetMode(Mode::Passthrough));
        need_copy(Cmd::Shutdown);
    }
}
