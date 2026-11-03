//! 参数描述表：每个可调参数的**正本**（稳定编号 · 键名 · 单位 · 最小 · 最大 · 默认 · 说明）。
//!
//! CLAUDE.md 第五节第 5 条「唯一真相源」：参数的范围与默认值只在这里写一次。各 crate 的 `Default`、范围校验、
//! 命令行默认值都从这里取，不再手写数字；`xtask tidy` 禁止在别处写参数的范围字面量（`tidy.rs` 的 R-PARAM）。
//!
//! 设计成纯数据常量，第二阶段可直接导出成 schema：范围不藏在函数逻辑里，每条的字段就是 schema 的字段。
//! 零依赖、不引 serde（规约 §2.2 规则 1）。
//!
//! 规矩：
//! - **编号永不复用**：删掉一个参数，它的编号也作废；改键名不改编号（日志与配置按编号对得上）
//! - 键名全局唯一；`ParamSet` 的六个字段键名不带前缀（它们就是文本格式里的键，见 [`crate::ParamField`]）
//! - 区间是闭区间，`min ≤ default ≤ max`，都是有限数（`tests` 里逐条查）
//! - 范围与默认值是 AI 按物理意义与现有实验取值定的（2026-11-02，single 线）；默认值沿用各 crate 原有初值，
//!   **范围待标定**，改了要同步说明
//!
//! 同一含义的参数在多处使用时共用一条（例如衰减下限 [`FLOOR_DB`]：`ParamSet` · engine 的 `SafetyConfig` ·
//! speaker 的门控与包络）；名字相同但含义不同的分开（例如修音链的增益下限 [`POLISH_FLOOR_DB`]）。

use core::fmt;

// 各组参数分文件写（每个代码文件 ≤ 600 行）；对外一律经本模块 `vdn_types::desc::X` 取，登记都在下面的 `ALL`
mod engine;
mod polish;
mod scene;
mod speaker;

pub use engine::*;
pub use polish::*;
pub use scene::*;
pub use speaker::*;

/// 单位（导出 schema 时原样写出 [`Unit::as_str`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Unit {
    /// 0..=1 的比例 / 混合量
    Ratio,
    /// 0..=1 的概率或置信度
    Probability,
    /// 相对电平（dB）
    Db,
    /// 绝对电平（dB 满刻度）
    Dbfs,
    /// 秒
    Seconds,
    /// dB 每秒
    DbPerSecond,
    /// 线性幅度（满刻度 = 1）
    Amplitude,
    /// 整数个数（以 f32 存，范围内都是精确整数）
    Count,
    /// 余弦相似度（-1..=1）
    Similarity,
    /// 倍数（无量纲，≥ 1 的比值）
    Factor,
}

impl Unit {
    pub fn as_str(self) -> &'static str {
        match self {
            Unit::Ratio => "ratio",
            Unit::Probability => "probability",
            Unit::Db => "dB",
            Unit::Dbfs => "dBFS",
            Unit::Seconds => "s",
            Unit::DbPerSecond => "dB/s",
            Unit::Amplitude => "amplitude",
            Unit::Count => "count",
            Unit::Similarity => "cosine",
            Unit::Factor => "x",
        }
    }
}

/// 一个参数的描述。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamDesc {
    /// 稳定编号（永不复用）
    pub id: u16,
    /// 键名（全局唯一；配置文本、命令行、日志用它）
    pub key: &'static str,
    pub unit: Unit,
    /// 闭区间下限
    pub min: f32,
    /// 闭区间上限
    pub max: f32,
    /// 默认值。`optional` 为真时，出厂值是「缺省（功能关）」，`default` 是打开时的建议值
    pub default: f32,
    /// 可缺省（使用方是 `Option<f32>`）
    pub optional: bool,
    /// 一句话说明
    pub doc: &'static str,
}

/// 参数越界或非有限。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeError {
    pub desc: &'static ParamDesc,
    pub value: f32,
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = self.desc;
        write!(
            f,
            "{} = {} 不在 [{}, {}]（{}）",
            d.key,
            self.value,
            d.min,
            d.max,
            d.unit.as_str()
        )
    }
}

impl std::error::Error for RangeError {}

// 实时线程可调（clamp / contains）：不分配、不 panic
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl ParamDesc {
    /// `(min, max)`
    pub const fn range(&self) -> (f32, f32) {
        (self.min, self.max)
    }

    /// 有限且在闭区间内。
    pub fn contains(&self, v: f32) -> bool {
        v.is_finite() && v >= self.min && v <= self.max
    }

    /// 合法就原样返回，否则报错（加载配置、解析命令行时用：越界拒收）。
    pub fn check(&'static self, v: f32) -> Result<f32, RangeError> {
        if self.contains(v) {
            Ok(v)
        } else {
            Err(RangeError {
                desc: self,
                value: v,
            })
        }
    }

    /// 可缺省参数的校验：`None` 合法。
    pub fn check_opt(&'static self, v: Option<f32>) -> Result<Option<f32>, RangeError> {
        match v {
            None => Ok(None),
            Some(x) => self.check(x).map(Some),
        }
    }

    /// 夹进范围；NaN 当默认值（构造期防御、实时侧收参数用）。
    pub fn clamp(&self, v: f32) -> f32 {
        if v.is_nan() {
            self.default
        } else {
            v.clamp(self.min, self.max)
        }
    }

    /// 可缺省参数的夹值：`None` 原样。
    pub fn clamp_opt(&self, v: Option<f32>) -> Option<f32> {
        v.map(|x| self.clamp(x))
    }

    /// 整数参数（[`Unit::Count`]）的默认值。
    pub fn default_count(&self) -> u32 {
        self.default as u32
    }

    /// 整数参数的校验。
    pub fn check_count(&'static self, v: u32) -> Result<u32, RangeError> {
        self.check(v as f32).map(|_| v)
    }

    /// 整数参数的夹值。
    pub fn clamp_count(&self, v: u32) -> u32 {
        (v as f32).clamp(self.min, self.max) as u32
    }
}

const fn p(
    id: u16,
    key: &'static str,
    unit: Unit,
    min: f32,
    max: f32,
    default: f32,
    doc: &'static str,
) -> ParamDesc {
    ParamDesc {
        id,
        key,
        unit,
        min,
        max,
        default,
        optional: false,
        doc,
    }
}

// ───────────── 1–99：ParamSet 的连续字段，及与之同义、多处共用的参数 ─────────────

/// 降噪强度（湿干比，D36 给用户的旋钮）：1 = 全量
pub const STRENGTH: ParamDesc = p(
    1,
    "strength",
    Unit::Ratio,
    0.0,
    1.0,
    1.0,
    "降噪强度：输出 = 原声 + strength·(降噪 − 原声)",
);
/// 衰减下限：停顿时背景最多压到这里（即「背景保留」）
pub const FLOOR_DB: ParamDesc = p(
    2,
    "floor_db",
    Unit::Db,
    -60.0,
    0.0,
    -12.0,
    "衰减下限：停顿时混回原声的比例",
);
/// 有声时混回原声的比例（快开）
pub const OPEN_DB: ParamDesc = p(
    3,
    "open_db",
    Unit::Db,
    -60.0,
    0.0,
    -6.0,
    "有声时混回原声的比例",
);
/// 门控灵敏度：高出底噪多少判为有声
pub const DETECT_DB: ParamDesc = p(
    4,
    "detect_db",
    Unit::Db,
    0.0,
    40.0,
    9.0,
    "高出底噪多少判为有声",
);
/// 人声停后回到下限的时间常数（慢关）
pub const RELEASE_S: ParamDesc = p(
    5,
    "release_s",
    Unit::Seconds,
    0.0,
    5.0,
    0.3,
    "人声停后回到下限的时间常数",
);
/// 人声停后先保持多久
pub const HANGOVER_S: ParamDesc = p(
    6,
    "hangover_s",
    Unit::Seconds,
    0.0,
    5.0,
    0.15,
    "人声停后保持放开多久",
);
/// 快开的时间常数（engine 的能量包络、speaker 的 VAD 包络共用）
pub const ATTACK_S: ParamDesc = p(
    7,
    "attack_s",
    Unit::Seconds,
    0.0,
    0.5,
    0.002,
    "快开的时间常数",
);
/// VAD 概率滞回：高于它转有声
pub const VAD_ON: ParamDesc = p(
    8,
    "vad_on",
    Unit::Probability,
    0.0,
    1.0,
    0.5,
    "VAD 概率高于它转有声",
);
/// VAD 概率滞回：低于它转无声（应 ≤ `vad_on`）
pub const VAD_OFF: ParamDesc = p(
    9,
    "vad_off",
    Unit::Probability,
    0.0,
    1.0,
    0.35,
    "VAD 概率低于它转无声",
);

/// 全部描述（导出、查重、按编号或键名查找都走它）。加一条 = 写常量 + 加进这里（`all_listed` 测试查漏）。
pub const ALL: [&ParamDesc; 58] = [
    &STRENGTH,
    &FLOOR_DB,
    &OPEN_DB,
    &DETECT_DB,
    &RELEASE_S,
    &HANGOVER_S,
    &ATTACK_S,
    &VAD_ON,
    &VAD_OFF,
    &GATE_ATTACK_S,
    &GATE_REOPEN_S,
    &GATE_RELEASE_S,
    &GATE_ONSET_GAP_S,
    &GATE_ONSET_JUMP_DB,
    &GATE_CONFIRM_N,
    &GATE_ONSET_LEVEL_MARGIN_DB,
    &GATE_LEVEL_CLOSE_DB,
    &GATE_LEVEL_CLOSE_HOLD_S,
    &GATE_ONSET_ARM_S,
    &GATE_ONSET_REFRACTORY_S,
    &POLISH_SMOOTH,
    &POLISH_FLOOR_DB,
    &POLISH_COMFORT_DB,
    &POLISH_DEMUFFLE_MAX_DB,
    &POLISH_AGC,
    &POLISH_AGC_TARGET_DBFS,
    &POLISH_AGC_RANGE_DB,
    &POLISH_RAMP_S,
    &HYST_CONFIRM_S,
    &HYST_MIN_CONFIDENCE,
    &HYST_MIN_DWELL_S,
    &SCENE_EMA_S,
    &TRANS_GLIDE_S,
    &TRANS_MODEL_CROSSFADE_S,
    &TRANS_MAX_DB_PER_S,
    &TRANS_MANUAL_RAMP_S,
    &ENGINE_MAX_ABS,
    &ENGINE_RECOVER_HOLD_S,
    &ENGINE_CROSSFADE_S,
    &JUDGE_WINDOW_S,
    &JUDGE_HOP_S,
    &JUDGE_MIN_VOICED_FRAC,
    &JUDGE_ENROLL_VOICED_FRAC,
    &JUDGE_CONSIST_SIM,
    &JUDGE_OWNER_HI,
    &JUDGE_OTHERS_LO,
    &JUDGE_VAD_THR,
    &ENROLL_ASSIGN_SIM,
    &ENROLL_MAX_CLUSTERS,
    &ENROLL_MIN_OWNER_S,
    &ENROLL_SINGLE_CLUSTER_MIN_S,
    &ENROLL_LOCK_MARGIN,
    &ENROLL_STABLE_UPDATES,
    &ENROLL_MIN_COHESION,
    &ENROLL_RECENCY_HALF_LIFE_S,
    &ENROLL_LEVEL_HALVING_DB,
    &ENROLL_RELOCK_RATIO,
    &ENROLL_CENTROID_CAP,
];

pub fn by_id(id: u16) -> Option<&'static ParamDesc> {
    ALL.into_iter().find(|d| d.id == id)
}

pub fn by_key(key: &str) -> Option<&'static ParamDesc> {
    ALL.into_iter().find(|d| d.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_is_well_formed() {
        for d in ALL {
            assert!(
                d.min.is_finite() && d.max.is_finite() && d.default.is_finite(),
                "{}",
                d.key
            );
            assert!(
                d.min <= d.default && d.default <= d.max,
                "{} 默认值不在范围内",
                d.key
            );
            assert!(!d.key.is_empty() && !d.doc.is_empty(), "{}", d.id);
            assert!(
                d.key
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.'),
                "{} 键名只许小写字母、数字、下划线、点",
                d.key
            );
            if d.unit == Unit::Count {
                assert!(
                    d.min.fract() == 0.0 && d.max.fract() == 0.0 && d.default.fract() == 0.0,
                    "{}",
                    d.key
                );
            }
        }
    }

    #[test]
    fn ids_and_keys_are_unique() {
        for (i, a) in ALL.iter().enumerate() {
            for b in &ALL[i + 1..] {
                assert_ne!(a.id, b.id, "编号重复：{} / {}", a.key, b.key);
                assert_ne!(a.key, b.key, "键名重复：{}", a.key);
            }
        }
    }

    /// 源码里的 `pub const X: ParamDesc` 常量名，以及其中没出现在 `ALL` 那一段里的。
    fn unlisted(srcs: &str, mod_src: &str) -> (usize, Vec<String>) {
        let all = mod_src
            .split_once("pub const ALL")
            .and_then(|(_, r)| r.split_once("];"))
            .map_or("", |(a, _)| a);
        let mut n = 0;
        let mut missing = Vec::new();
        for line in srcs.lines() {
            let Some(rest) = line.trim_start().strip_prefix("pub const ") else {
                continue;
            };
            let Some((name, ty)) = rest.split_once(':') else {
                continue;
            };
            if !ty.trim_start().starts_with("ParamDesc") {
                continue;
            }
            n += 1;
            if !all.contains(&format!("&{name},")) {
                missing.push(name.to_string());
            }
        }
        (n, missing)
    }

    /// 每个 `pub const X: ParamDesc` 都进了 `ALL`：扫本目录下**实际存在的全部** `.rs`（新加的描述文件自动在内，
    /// codex 第 1 轮 #4），漏登记就红。
    #[test]
    fn all_listed() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/desc");
        let mut files: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "rs"))
            .collect();
        files.sort();
        assert!(files.len() >= 2, "{files:?}");
        let srcs: String = files
            .iter()
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        let (n, missing) = unlisted(&srcs, include_str!("mod.rs"));
        assert!(missing.is_empty(), "没登记进 ALL：{missing:?}");
        assert_eq!(n, ALL.len());
    }

    /// 负例：新文件里的常量没进 `ALL` → 报出来；只在别处（比如注释、别的数组）出现也不算登记。
    #[test]
    fn unlisted_detects_missing_entries() {
        let m = "pub const A: ParamDesc = p(1);\npub const ALL: [&ParamDesc; 1] = [\n    &A,\n];\nconst OTHER: [&ParamDesc; 1] = [&B,];\n";
        let extra = "pub const B: ParamDesc = p(2);\n";
        let (n, missing) = unlisted(&format!("{m}{extra}"), m);
        assert_eq!((n, missing), (2, vec!["B".to_string()]));
    }

    #[test]
    fn lookups_roundtrip() {
        for d in ALL {
            assert_eq!(by_id(d.id), Some(d));
            assert_eq!(by_key(d.key), Some(d));
        }
        assert!(by_id(0).is_none() && by_key("nope").is_none());
    }

    #[test]
    fn check_and_clamp_follow_the_table() {
        for d in ALL {
            assert_eq!(d.check(d.default), Ok(d.default));
            assert_eq!(d.check(d.min), Ok(d.min));
            assert_eq!(d.check(d.max), Ok(d.max));
            let below = d.min - 1.0;
            let above = d.max + 1.0;
            assert_eq!(d.check(below).unwrap_err().value, below, "{}", d.key);
            assert!(d.check(above).is_err(), "{}", d.key);
            assert!(d.check(f32::NAN).is_err() && d.check(f32::INFINITY).is_err());
            assert_eq!(d.clamp(below), d.min);
            assert_eq!(d.clamp(above), d.max);
            assert_eq!(d.clamp(f32::NAN), d.default);
            assert_eq!(d.check_opt(None), Ok(None));
            assert!(d.check_opt(Some(above)).is_err());
        }
        assert_eq!(GATE_CONFIRM_N.default_count(), 2);
        assert_eq!(GATE_CONFIRM_N.clamp_count(0), 1);
        assert_eq!(GATE_CONFIRM_N.clamp_count(99), 20);
        assert!(GATE_CONFIRM_N.check_count(0).is_err());
    }

    #[test]
    fn error_names_key_range_and_unit() {
        let e = FLOOR_DB.check(3.0).unwrap_err();
        assert_eq!(e.to_string(), "floor_db = 3 不在 [-60, 0]（dB）");
    }
}
