//! 分类器权重与阈值。**由 `docs/experiments/exp006-scene-sim/train.py` 生成，别手改**；重训 = 重跑脚本 = 新提交。
//! 占位版：全零权重（分类器恒判第一类），只为让 crate 先编译。
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
/// 特征 8（hf_p90）低于它判为窄带蓝牙
pub const BT_HF_P90_MAX: f32 = -30.0;
/// 特征 21（log10 平顶占比）高于它判为削波
pub const CLIP_MIN: f32 = -3.0;
/// 后验平滑时间常数（秒）
pub const EMA_S: f32 = 1.5;
pub const HYSTERESIS: Hysteresis = Hysteresis { confirm_s: 2.0, min_confidence: 0.6, min_dwell_s: 5.0 };
