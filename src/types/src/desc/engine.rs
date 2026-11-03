//! 参数描述 · engine 保命（500–599）。对外路径 `vdn_types::desc::X`，见 [`super`]。

use super::{ParamDesc, Unit, p};

// ───────────── 500–599：engine 保命（`vdn_engine::SafetyConfig`，只引用，结构不动） ─────────────

pub const ENGINE_MAX_ABS: ParamDesc = p(
    501,
    "engine.max_abs",
    Unit::Amplitude,
    1.0,
    100.0,
    4.0,
    "输出幅度超过它当状态异常",
);
pub const ENGINE_RECOVER_HOLD_S: ParamDesc = p(
    502,
    "engine.recover_hold_s",
    Unit::Seconds,
    0.0,
    5.0,
    0.1,
    "状态类故障复位后继续送原声多久",
);
pub const ENGINE_CROSSFADE_S: ParamDesc = p(
    503,
    "engine.crossfade_s",
    Unit::Seconds,
    0.0,
    1.0,
    0.005,
    "切回模型的交叉淡化 / 切到原声的接缝衰减时长",
);
