//! 参数描述 · speaker（门控 100–199 · 判断回路 600–699 · 零设置注册 700–799）。对外路径 `vdn_types::desc::X`，见 [`super`]。

use super::{ParamDesc, Unit, p};

// ───────────── 100–199：speaker 不对称门控（`vdn_speaker::gate::GateParams`） ─────────────

pub const GATE_ATTACK_S: ParamDesc = p(
    101,
    "gate.attack_s",
    Unit::Seconds,
    0.0,
    0.5,
    0.005,
    "门控：新人声起始时放开的时间常数",
);
pub const GATE_REOPEN_S: ParamDesc = p(
    102,
    "gate.reopen_s",
    Unit::Seconds,
    0.0,
    2.0,
    0.05,
    "门控：Target / Unknown 判决后回到全开的时间常数",
);
pub const GATE_RELEASE_S: ParamDesc = p(
    103,
    "gate.release_s",
    Unit::Seconds,
    0.0,
    5.0,
    0.5,
    "门控：确认是别人后慢收到下限的时间常数",
);
pub const GATE_ONSET_GAP_S: ParamDesc = p(
    104,
    "gate.onset_gap_s",
    Unit::Seconds,
    0.0,
    5.0,
    0.3,
    "门控：停顿至少这么久后转有声才算新起始",
);
pub const GATE_ONSET_JUMP_DB: ParamDesc = p(
    105,
    "gate.onset_jump_db",
    Unit::Db,
    0.0,
    40.0,
    8.0,
    "门控：短时电平比近 1 s 均值高这么多算起始",
);
pub const GATE_CONFIRM_N: ParamDesc = p(
    106,
    "gate.confirm_n",
    Unit::Count,
    1.0,
    20.0,
    2.0,
    "门控：要几份作数的 Other 判决才慢收",
);
pub const GATE_ONSET_LEVEL_MARGIN_DB: ParamDesc = p(
    107,
    "gate.onset_level_margin_db",
    Unit::Db,
    0.0,
    240.0,
    3.0,
    "门控：起始电平须 ≥ 主人电平 − 它；设很大 = 任何起始都放开",
);
/// 可缺省：出厂关（偏离 003 §2.3，待拍板项，exp005）
pub const GATE_LEVEL_CLOSE_DB: ParamDesc = ParamDesc {
    optional: true,
    ..p(
        108,
        "gate.level_close_db",
        Unit::Db,
        0.0,
        60.0,
        10.0,
        "门控快路径：有人声但比主人电平低这么多，持续一段就先慢收",
    )
};
pub const GATE_LEVEL_CLOSE_HOLD_S: ParamDesc = p(
    109,
    "gate.level_close_hold_s",
    Unit::Seconds,
    0.0,
    5.0,
    0.3,
    "门控快路径：轻人声要持续多久",
);
pub const GATE_ONSET_ARM_S: ParamDesc = p(
    110,
    "gate.onset_arm_s",
    Unit::Seconds,
    0.0,
    2.0,
    0.1,
    "门控：停顿后转有声起，起始判据保持有效的时长",
);
pub const GATE_ONSET_REFRACTORY_S: ParamDesc = p(
    111,
    "gate.onset_refractory_s",
    Unit::Seconds,
    0.0,
    5.0,
    0.3,
    "门控：电平跳起型起始的不应期",
);

// ───────────── 600–699：speaker 判断回路（`vdn_speaker::judge::JudgeParams`） ─────────────

pub const JUDGE_WINDOW_S: ParamDesc = p(
    601,
    "judge.window_s",
    Unit::Seconds,
    0.1,
    10.0,
    1.5,
    "判断回路：声纹窗长",
);
pub const JUDGE_HOP_S: ParamDesc = p(
    602,
    "judge.hop_s",
    Unit::Seconds,
    0.05,
    5.0,
    0.5,
    "判断回路：多久判一次",
);
pub const JUDGE_MIN_VOICED_FRAC: ParamDesc = p(
    603,
    "judge.min_voiced_frac",
    Unit::Ratio,
    0.0,
    1.0,
    0.3,
    "窗里有声帧比例低于它 → NoSpeech",
);
pub const JUDGE_ENROLL_VOICED_FRAC: ParamDesc = p(
    604,
    "judge.enroll_voiced_frac",
    Unit::Ratio,
    0.0,
    1.0,
    0.6,
    "有声帧比例至少这么多才喂注册器",
);
pub const JUDGE_CONSIST_SIM: ParamDesc = p(
    605,
    "judge.consist_sim",
    Unit::Similarity,
    -1.0,
    1.0,
    0.6,
    "与上一窗相似度至少这么多才算清楚的单人片段",
);
pub const JUDGE_OWNER_HI: ParamDesc = p(
    606,
    "judge.owner_hi",
    Unit::Similarity,
    -1.0,
    1.0,
    0.45,
    "与主人相似度高于它 → Target",
);
pub const JUDGE_OTHERS_LO: ParamDesc = p(
    607,
    "judge.others_lo",
    Unit::Similarity,
    -1.0,
    1.0,
    0.25,
    "与主人相似度低于它 → Other",
);
pub const JUDGE_VAD_THR: ParamDesc = p(
    608,
    "judge.vad_thr",
    Unit::Probability,
    0.0,
    1.0,
    0.5,
    "判断回路数有声帧用的 VAD 硬门限",
);

// ───────────── 700–799：speaker 零设置注册（`vdn_speaker::enroll::EnrollParams`） ─────────────

pub const ENROLL_ASSIGN_SIM: ParamDesc = p(
    701,
    "enroll.assign_sim",
    Unit::Similarity,
    -1.0,
    1.0,
    0.5,
    "与最近簇中心相似度至少这么多就并入",
);
pub const ENROLL_MAX_CLUSTERS: ParamDesc = p(
    702,
    "enroll.max_clusters",
    Unit::Count,
    1.0,
    64.0,
    8.0,
    "最多保留几个簇",
);
pub const ENROLL_MIN_OWNER_S: ParamDesc = p(
    703,
    "enroll.min_owner_s",
    Unit::Seconds,
    0.0,
    600.0,
    15.0,
    "主人簇至少累计这么多秒才锁定",
);
pub const ENROLL_SINGLE_CLUSTER_MIN_S: ParamDesc = p(
    704,
    "enroll.single_cluster_min_s",
    Unit::Seconds,
    0.0,
    600.0,
    30.0,
    "只有一个簇时至少累计这么多秒才锁定",
);
pub const ENROLL_LOCK_MARGIN: ParamDesc = p(
    705,
    "enroll.lock_margin",
    Unit::Factor,
    1.0,
    10.0,
    1.5,
    "主人簇得分要比第二名高这么多倍才锁定",
);
pub const ENROLL_STABLE_UPDATES: ParamDesc = p(
    706,
    "enroll.stable_updates",
    Unit::Count,
    1.0,
    100.0,
    6.0,
    "领先者连续保持这么多次更新才锁定",
);
pub const ENROLL_MIN_COHESION: ParamDesc = p(
    707,
    "enroll.min_cohesion",
    Unit::Similarity,
    -1.0,
    1.0,
    0.6,
    "簇内聚度至少这么多才可当主人（应高于 assign_sim）",
);
pub const ENROLL_RECENCY_HALF_LIFE_S: ParamDesc = p(
    708,
    "enroll.recency_half_life_s",
    Unit::Seconds,
    1.0,
    86400.0,
    300.0,
    "簇得分的时间衰减半衰期",
);
pub const ENROLL_LEVEL_HALVING_DB: ParamDesc = p(
    709,
    "enroll.level_halving_db",
    Unit::Db,
    1.0,
    60.0,
    6.0,
    "电平每低这么多 dB，近讲系数减半",
);
pub const ENROLL_RELOCK_RATIO: ParamDesc = p(
    710,
    "enroll.relock_ratio",
    Unit::Factor,
    1.0,
    10.0,
    2.0,
    "别的簇得分超过主人这么多倍才改判",
);
pub const ENROLL_CENTROID_CAP: ParamDesc = p(
    711,
    "enroll.centroid_cap",
    Unit::Count,
    1.0,
    10000.0,
    200.0,
    "簇中心按至多这么多个样本平均，之后转指数平均",
);
