//! 占位（特征改成 35 维后待重训）。由 train.py 覆盖。
use crate::features::NF;
use crate::mirror::Hysteresis;
pub const NC: usize = 7;
pub const NH: usize = 24;
pub const TRAINED_ON: &str = "placeholder";
pub static MEAN: [f32; NF] = [0.0; NF];
pub static STD: [f32; NF] = [1.0; NF];
pub static W1: [[f32; NF]; NH] = [[0.0; NF]; NH];
pub static B1: [f32; NH] = [0.0; NH];
pub static W2: [[f32; NH]; NC] = [[0.0; NH]; NC];
pub static B2: [f32; NC] = [0.0; NC];
pub const BT_HF_P90_MAX: f32 = -14.0;
pub const CLIP_MIN: f32 = -4.0;
pub const EMA_S: f32 = 1.5;
pub const HYSTERESIS: Hysteresis = Hysteresis { confirm_s: 2.0, min_confidence: 0.6, min_dwell_s: 5.0 };
