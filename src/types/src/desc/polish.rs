//! 参数描述 · 修音链（200–299）。对外路径 `vdn_types::desc::X`，见 [`super`]。

use super::{ParamDesc, Unit, p};

// ───────────── 200–299：修音链（`vdn_polish::PolishParams` · `PolishChain`） ─────────────

pub const POLISH_SMOOTH: ParamDesc = p(
    201,
    "polish.smooth",
    Unit::Ratio,
    0.0,
    1.0,
    0.6,
    "频谱平滑强度（治金属声 / 音乐噪声）",
);
/// 修音链频谱增益下限；与 [`super::FLOOR_DB`]（门控衰减下限）不是一回事。`min` = 关
pub const POLISH_FLOOR_DB: ParamDesc = p(
    202,
    "polish.floor_db",
    Unit::Db,
    -100.0,
    0.0,
    -24.0,
    "频谱增益下限；-100 = 不设下限",
);
pub const POLISH_COMFORT_DB: ParamDesc = p(
    203,
    "polish.comfort_db",
    Unit::Db,
    -100.0,
    0.0,
    -10.0,
    "舒适噪声电平（相对稳态底噪）；-100 = 关",
);
pub const POLISH_DEMUFFLE_MAX_DB: ParamDesc = p(
    204,
    "polish.demuffle_max_db",
    Unit::Db,
    0.0,
    12.0,
    4.0,
    "去发闷的最大补偿；0 = 关",
);
pub const POLISH_AGC: ParamDesc = p(
    205,
    "polish.agc",
    Unit::Ratio,
    0.0,
    1.0,
    1.0,
    "AGC 开关（≥ 0.5 视为开，f32 便于插值）",
);
pub const POLISH_AGC_TARGET_DBFS: ParamDesc = p(
    206,
    "polish.agc_target_dbfs",
    Unit::Dbfs,
    -60.0,
    0.0,
    -26.0,
    "AGC 目标语音电平",
);
pub const POLISH_AGC_RANGE_DB: ParamDesc = p(
    207,
    "polish.agc_range_db",
    Unit::Db,
    0.0,
    30.0,
    10.0,
    "AGC 最大增益 / 最大衰减（对称）",
);
pub const POLISH_RAMP_S: ParamDesc = p(
    208,
    "polish.ramp_s",
    Unit::Seconds,
    0.0,
    30.0,
    1.0,
    "修音链换参数的线性过渡时长",
);
