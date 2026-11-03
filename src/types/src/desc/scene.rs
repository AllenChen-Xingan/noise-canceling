//! 参数描述 · 场景判断（300–399）与平滑过渡（400–499）。对外路径 `vdn_types::desc::X`，见 [`super`]。

use super::{ParamDesc, Unit, p};

// ───────────── 300–399：场景判断（`Hysteresis` · `vdn_scene::SceneAnalyzer`） ─────────────
// 默认值 = exp006 held-out 标定结果；`vdn_scene::weights` 里训练脚本写出的同一组数由测试比对

pub const HYST_CONFIRM_S: ParamDesc = p(
    301,
    "hysteresis.confirm_s",
    Unit::Seconds,
    0.0,
    30.0,
    1.0,
    "新场景要连续被判中这么久才换",
);
pub const HYST_MIN_CONFIDENCE: ParamDesc = p(
    302,
    "hysteresis.min_confidence",
    Unit::Probability,
    0.0,
    1.0,
    0.4,
    "低于这个置信度的判断不计入确认",
);
pub const HYST_MIN_DWELL_S: ParamDesc = p(
    303,
    "hysteresis.min_dwell_s",
    Unit::Seconds,
    0.0,
    60.0,
    3.0,
    "换过一次后至少停留这么久",
);
pub const SCENE_EMA_S: ParamDesc = p(
    304,
    "scene.ema_s",
    Unit::Seconds,
    0.0,
    10.0,
    0.5,
    "场景后验的平滑时间常数",
);

// ───────────── 400–499：平滑过渡（`Transition`） ─────────────

pub const TRANS_GLIDE_S: ParamDesc = p(
    401,
    "transition.glide_s",
    Unit::Seconds,
    0.0,
    10.0,
    1.0,
    "连续参数的一阶平滑时间常数",
);
pub const TRANS_MODEL_CROSSFADE_S: ParamDesc = p(
    402,
    "transition.model_crossfade_s",
    Unit::Seconds,
    0.0,
    1.0,
    0.05,
    "换模型的交叉淡化时长",
);
pub const TRANS_MAX_DB_PER_S: ParamDesc = p(
    403,
    "transition.max_db_per_s",
    Unit::DbPerSecond,
    0.0,
    120.0,
    1.0,
    "dB 类字段每秒最多变多少",
);
pub const TRANS_MANUAL_RAMP_S: ParamDesc = p(
    404,
    "transition.manual_ramp_s",
    Unit::Seconds,
    0.0,
    0.5,
    0.03,
    "用户手动改动的最短斜坡",
);
