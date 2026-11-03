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
    pub const fn range(&'static self) -> (f32, f32) {
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
pub const STRENGTH: ParamDesc = p(1, "strength", Unit::Ratio, 0.0, 1.0, 1.0, "降噪强度：输出 = 原声 + strength·(降噪 − 原声)");
/// 衰减下限：停顿时背景最多压到这里（即「背景保留」）
pub const FLOOR_DB: ParamDesc = p(2, "floor_db", Unit::Db, -60.0, 0.0, -12.0, "衰减下限：停顿时混回原声的比例");
/// 有声时混回原声的比例（快开）
pub const OPEN_DB: ParamDesc = p(3, "open_db", Unit::Db, -60.0, 0.0, -6.0, "有声时混回原声的比例");
/// 门控灵敏度：高出底噪多少判为有声
pub const DETECT_DB: ParamDesc = p(4, "detect_db", Unit::Db, 0.0, 40.0, 9.0, "高出底噪多少判为有声");
/// 人声停后回到下限的时间常数（慢关）
pub const RELEASE_S: ParamDesc = p(5, "release_s", Unit::Seconds, 0.0, 5.0, 0.3, "人声停后回到下限的时间常数");
/// 人声停后先保持多久
pub const HANGOVER_S: ParamDesc = p(6, "hangover_s", Unit::Seconds, 0.0, 5.0, 0.15, "人声停后保持放开多久");
/// 快开的时间常数（engine 的能量包络、speaker 的 VAD 包络共用）
pub const ATTACK_S: ParamDesc = p(7, "attack_s", Unit::Seconds, 0.0, 0.5, 0.002, "快开的时间常数");
/// VAD 概率滞回：高于它转有声
pub const VAD_ON: ParamDesc = p(8, "vad_on", Unit::Probability, 0.0, 1.0, 0.5, "VAD 概率高于它转有声");
/// VAD 概率滞回：低于它转无声（应 ≤ `vad_on`）
pub const VAD_OFF: ParamDesc = p(9, "vad_off", Unit::Probability, 0.0, 1.0, 0.35, "VAD 概率低于它转无声");

// ───────────── 100–199：speaker 不对称门控（`vdn_speaker::gate::GateParams`） ─────────────

pub const GATE_ATTACK_S: ParamDesc = p(101, "gate.attack_s", Unit::Seconds, 0.0, 0.5, 0.005, "门控：新人声起始时放开的时间常数");
pub const GATE_REOPEN_S: ParamDesc = p(102, "gate.reopen_s", Unit::Seconds, 0.0, 2.0, 0.05, "门控：Target / Unknown 判决后回到全开的时间常数");
pub const GATE_RELEASE_S: ParamDesc = p(103, "gate.release_s", Unit::Seconds, 0.0, 5.0, 0.5, "门控：确认是别人后慢收到下限的时间常数");
pub const GATE_ONSET_GAP_S: ParamDesc = p(104, "gate.onset_gap_s", Unit::Seconds, 0.0, 5.0, 0.3, "门控：停顿至少这么久后转有声才算新起始");
pub const GATE_ONSET_JUMP_DB: ParamDesc = p(105, "gate.onset_jump_db", Unit::Db, 0.0, 40.0, 8.0, "门控：短时电平比近 1 s 均值高这么多算起始");
pub const GATE_CONFIRM_N: ParamDesc = p(106, "gate.confirm_n", Unit::Count, 1.0, 20.0, 2.0, "门控：要几份作数的 Other 判决才慢收");
pub const GATE_ONSET_LEVEL_MARGIN_DB: ParamDesc = p(107, "gate.onset_level_margin_db", Unit::Db, 0.0, 240.0, 3.0, "门控：起始电平须 ≥ 主人电平 − 它；设很大 = 任何起始都放开");
/// 可缺省：出厂关（偏离 003 §2.3，待拍板项，exp005）
pub const GATE_LEVEL_CLOSE_DB: ParamDesc = ParamDesc {
    optional: true,
    ..p(108, "gate.level_close_db", Unit::Db, 0.0, 60.0, 10.0, "门控快路径：有人声但比主人电平低这么多，持续一段就先慢收")
};
pub const GATE_LEVEL_CLOSE_HOLD_S: ParamDesc = p(109, "gate.level_close_hold_s", Unit::Seconds, 0.0, 5.0, 0.3, "门控快路径：轻人声要持续多久");
pub const GATE_ONSET_ARM_S: ParamDesc = p(110, "gate.onset_arm_s", Unit::Seconds, 0.0, 2.0, 0.1, "门控：停顿后转有声起，起始判据保持有效的时长");
pub const GATE_ONSET_REFRACTORY_S: ParamDesc = p(111, "gate.onset_refractory_s", Unit::Seconds, 0.0, 5.0, 0.3, "门控：电平跳起型起始的不应期");

// ───────────── 200–299：修音链（`vdn_polish::PolishParams` · `PolishChain`） ─────────────

pub const POLISH_SMOOTH: ParamDesc = p(201, "polish.smooth", Unit::Ratio, 0.0, 1.0, 0.6, "频谱平滑强度（治金属声 / 音乐噪声）");
/// 修音链频谱增益下限；与 [`FLOOR_DB`]（门控衰减下限）不是一回事。`min` = 关
pub const POLISH_FLOOR_DB: ParamDesc = p(202, "polish.floor_db", Unit::Db, -100.0, 0.0, -24.0, "频谱增益下限；-100 = 不设下限");
pub const POLISH_COMFORT_DB: ParamDesc = p(203, "polish.comfort_db", Unit::Db, -100.0, 0.0, -10.0, "舒适噪声电平（相对稳态底噪）；-100 = 关");
pub const POLISH_DEMUFFLE_MAX_DB: ParamDesc = p(204, "polish.demuffle_max_db", Unit::Db, 0.0, 12.0, 4.0, "去发闷的最大补偿；0 = 关");
pub const POLISH_AGC: ParamDesc = p(205, "polish.agc", Unit::Ratio, 0.0, 1.0, 1.0, "AGC 开关（≥ 0.5 视为开，f32 便于插值）");
pub const POLISH_AGC_TARGET_DBFS: ParamDesc = p(206, "polish.agc_target_dbfs", Unit::Dbfs, -60.0, 0.0, -26.0, "AGC 目标语音电平");
pub const POLISH_AGC_RANGE_DB: ParamDesc = p(207, "polish.agc_range_db", Unit::Db, 0.0, 30.0, 10.0, "AGC 最大增益 / 最大衰减（对称）");
pub const POLISH_RAMP_S: ParamDesc = p(208, "polish.ramp_s", Unit::Seconds, 0.0, 30.0, 1.0, "修音链换参数的线性过渡时长");

// ───────────── 300–399：场景判断（`Hysteresis` · `vdn_scene::SceneAnalyzer`） ─────────────
// 默认值 = exp006 held-out 标定结果；`vdn_scene::weights` 里训练脚本写出的同一组数由测试比对

pub const HYST_CONFIRM_S: ParamDesc = p(301, "hysteresis.confirm_s", Unit::Seconds, 0.0, 30.0, 1.0, "新场景要连续被判中这么久才换");
pub const HYST_MIN_CONFIDENCE: ParamDesc = p(302, "hysteresis.min_confidence", Unit::Probability, 0.0, 1.0, 0.4, "低于这个置信度的判断不计入确认");
pub const HYST_MIN_DWELL_S: ParamDesc = p(303, "hysteresis.min_dwell_s", Unit::Seconds, 0.0, 60.0, 3.0, "换过一次后至少停留这么久");
pub const SCENE_EMA_S: ParamDesc = p(304, "scene.ema_s", Unit::Seconds, 0.0, 10.0, 0.5, "场景后验的平滑时间常数");

// ───────────── 400–499：平滑过渡（`Transition`） ─────────────

pub const TRANS_GLIDE_S: ParamDesc = p(401, "transition.glide_s", Unit::Seconds, 0.0, 10.0, 1.0, "连续参数的一阶平滑时间常数");
pub const TRANS_MODEL_CROSSFADE_S: ParamDesc = p(402, "transition.model_crossfade_s", Unit::Seconds, 0.0, 1.0, 0.05, "换模型的交叉淡化时长");
pub const TRANS_MAX_DB_PER_S: ParamDesc = p(403, "transition.max_db_per_s", Unit::DbPerSecond, 0.0, 120.0, 1.0, "dB 类字段每秒最多变多少");
pub const TRANS_MANUAL_RAMP_S: ParamDesc = p(404, "transition.manual_ramp_s", Unit::Seconds, 0.0, 0.5, 0.03, "用户手动改动的最短斜坡");

// ───────────── 500–599：engine 保命（`vdn_engine::SafetyConfig`，只引用，结构不动） ─────────────

pub const ENGINE_MAX_ABS: ParamDesc = p(501, "engine.max_abs", Unit::Amplitude, 0.0, 100.0, 4.0, "输出幅度超过它当状态异常");
pub const ENGINE_RECOVER_HOLD_S: ParamDesc = p(502, "engine.recover_hold_s", Unit::Seconds, 0.0, 5.0, 0.1, "状态类故障复位后继续送原声多久");
pub const ENGINE_CROSSFADE_S: ParamDesc = p(503, "engine.crossfade_s", Unit::Seconds, 0.0, 1.0, 0.005, "切回模型的交叉淡化 / 切到原声的接缝衰减时长");

/// 全部描述（导出、查重、按编号或键名查找都走它）。加一条 = 写常量 + 加进这里（`all_listed` 测试查漏）。
pub const ALL: [&ParamDesc; 42] = [
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
    // 留位：ALL 长度与常量个数由测试核对
    &STRENGTH,
    &STRENGTH,
    &STRENGTH,
];

pub fn by_id(id: u16) -> Option<&'static ParamDesc> {
    ALL.into_iter().find(|d| d.id == id)
}

pub fn by_key(key: &str) -> Option<&'static ParamDesc> {
    ALL.into_iter().find(|d| d.key == key)
}
