//! 阈值与分档边界的表（正本数据）。类型与比对逻辑在上一级 `thresholds.rs`。

use super::{Cause, Lines, Param, Pend, Src};

const fn p(key: &'static str, default: Option<f64>, values: &'static [(f64, Src)]) -> Param {
    Param {
        key,
        default,
        values,
    }
}

const NO_EDGES: Src = Src::Design("不分档");
const PHYS: Src = Src::Design("物理量 / 模拟真值：012 只写判法不写数，设计值，未定标");

pub const BGSTAB: Lines = Lines {
    id: "V-BGSTAB",
    edges: &[3.0, 6.0, 10.0],
    edges_src: Src::Doc012(&["≤ 3 / 6 / 10 dB 为 A / B / C"]),
    params: &[
        p(
            "max",
            Some(10.0),
            &[(10.0, Src::Doc012(&["**> 10 dB → D**"]))],
        ),
        p(
            "from_s",
            Some(0.0),
            &[
                (0.0, Src::Design("默认从头算")),
                (5.0, Src::Doc012(&["平稳噪声先放 ≥ 5 s"])),
            ],
        ),
    ],
};

pub const ONSET: Lines = Lines {
    id: "V-ONSET",
    edges: &[1.0, 2.0, 3.0],
    edges_src: Src::Doc012(&["> 3 dB → 「吞字头」"]),
    params: &[p(
        "max",
        Some(3.0),
        &[(3.0, Src::Doc012(&["> 3 dB → 「吞字头」"]))],
    )],
};

/// 量法与 rubric C3 接近：代码按主讲人有声帧把输出投影到干净参考上算能量损失（`verifiers.rs` 登记：不用 P.56），
/// C3 按参考有声帧能量损失；差在投影去噪。分档照 C3（铁律 5：标准文档是正本；codex 第 2 轮查出量法描述错，
/// 据此把 C/D 线从 −3 改为 −4，大总管 8d 定 2026-11-02）
pub const SPLVL: Lines = Lines {
    id: "V-SPLVL",
    edges: &[-1.0, -2.0, -4.0],
    edges_src: Src::RubricLoss(&["损失 > 4 dB", "损失 > 2 dB", "损失 ≤ 1 dB"]),
    params: &[p(
        "min",
        Some(-3.0),
        &[
            (-3.0, Src::None),
            // 人定 D85 题 5（G-5）：新规则（v2）用 4 dB，判在组合均值的区间上（G.160 参考下限）
            (-4.0, Src::RubricLoss(&["损失 > 4 dB"])),
        ],
    )],
};

pub const MUSIC: Lines = Lines {
    id: "V-MUSIC",
    edges: &[1.2, 1.5, 2.0],
    edges_src: Src::Doc012(&["峰度比 ≤ 1.2 A · ≤ 1.5 B · ≤ 2 C · > 2 D"]),
    params: &[p(
        "max",
        Some(2.0),
        &[(2.0, Src::Doc012(&["≤ 2 C · > 2 D"]))],
    )],
};

/// 量法与 rubric N4 不同：代码 100–1k / 1–2k / 2–4k / 4–7.9k Hz、512 点跳 256；N4 100–300 / 300–500 / 2–5k / 5–8k Hz、1024 点跳 512
pub const TIMBRE: Lines = Lines {
    id: "V-TIMBRE",
    edges: &[2.0, 4.0, 6.0],
    edges_src: Src::Rubric(&["D > 6 · C ≤ 6 · B ≤ 3 · A ≤ 1.5"]),
    params: &[p(
        "max",
        Some(6.0),
        &[(6.0, Src::Rubric(&["D > 6 · C ≤ 6"]))],
    )],
};

pub const NR: Lines = Lines {
    id: "V-NR",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "max",
        None,
        &[
            (-6.0, Src::Rubric(&["| D | > −6 dB | 跟原声差不多 |"])),
            (-8.0, Src::None),
            (-10.0, Src::None),
            (-20.0, Src::Rubric(&["| A | ≤ −20 dB |"])),
        ],
    )],
};

pub const INTF: Lines = Lines {
    id: "V-INTF",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "max",
        None,
        &[(-10.0, Src::Rubric(&["| B | −15 < x ≤ −10 |"]))],
    )],
};

pub const VACUUM: Lines = Lines {
    id: "V-VACUUM",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "min",
        Some(-40.0),
        &[(-40.0, Src::Rubric(&["`noise_att_db` ≤ −40 dB"]))],
    )],
};

pub const EAT_ASR: Lines = Lines {
    id: "V-EAT-ASR",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p(
            "max",
            None,
            &[(1.0, Src::Rubric(&["（> +1 时带 `v7_suspect`）"]))],
        ),
        // 人定 D85 题 3（G-3 = C）：新规则（v2）单段比原声多错（删 + 替）≥ 5 字才否决
        p(
            "fail_chars",
            Some(5.0),
            &[(5.0, Src::Design("人定 D85 题 3 选 C：单段多错 ≥ 5 字才否决"))],
        ),
    ],
};

pub const FRAME: Lines = Lines {
    id: "V-FRAME",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(0.5), &[(0.5, PHYS)])],
};

pub const MUTE: Lines = Lines {
    id: "V-MUTE",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p("max_dbfs", Some(-90.3), &[(-90.3, PHYS)]),
        p("settle_s", Some(0.3), &[(0.3, PHYS)]),
    ],
};

pub const SWITCH: Lines = Lines {
    id: "V-SWITCH",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(2.0), &[(2.0, PHYS)])],
};

pub const SIM_GATE: Lines = Lines {
    id: "V-SIM-GATE",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p(
            "max",
            Some(0.05),
            &[
                (0.05, Src::None),
                (
                    0.4,
                    Src::Design("R01 / R03 的 note：放宽的护栏线，严格线在能力集 C10"),
                ),
            ],
        ),
        p(
            "onset_max",
            Some(0.2),
            &[
                (0.2, Src::None),
                (
                    0.5,
                    Src::Design("R01 / R03 的 note：放宽的护栏线，严格线在能力集 C10"),
                ),
            ],
        ),
    ],
};

pub const SIM_FA: Lines = Lines {
    id: "V-SIM-FA",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(0.1), &[(0.1, PHYS)])],
};

pub const SIM_QUIET: Lines = Lines {
    id: "V-SIM-QUIET",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(0.1), &[(0.1, PHYS)])],
};

pub const SIM_SWLAT: Lines = Lines {
    id: "V-SIM-SWLAT",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(32.0), &[(32.0, PHYS)])],
};

/// 012 V-WEAK 原文标〔建议值〕：按惯例 012 摘录算出处，成绩单标准里另注「原文是建议值」
pub const WEAK: Lines = Lines {
    id: "V-WEAK",
    edges: &[1.0, 2.0, 4.0],
    edges_src: Src::Doc012(&["≤ 1 dB A · ≤ 2 B · ≤ 4 C · > 4 D"]),
    params: &[
        p("max", Some(4.0), &[(4.0, Src::Doc012(&["≤ 4 C · > 4 D"]))]),
        p(
            "compress_min",
            Some(-3.0),
            &[(-3.0, Src::Doc012(&["< −3 dB 另标「被压缩」"]))],
        ),
    ],
};

/// 012 只给了 A 档（「≤ 1 dB² A …」），B / C / D 没写：过线值暂定，进待定清单
pub const PUMPV: Lines = Lines {
    id: "V-PUMPV",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(1.0), &[(1.0, Src::None)])],
};

pub const CLICK: Lines = Lines {
    id: "V-CLICK",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p(
            "max",
            Some(0.0),
            &[(0.0, Src::Doc012(&["每分钟 0 个为 A"]))],
        ),
        p("k", Some(8.0), &[(8.0, Src::Doc012(&["k = 8"]))]),
        p(
            "resid_frac",
            Some(0.5),
            &[(
                0.5,
                Src::Design(
                    "跳变里干净人声解释不了的部分 ≥ 一半才算咔嗒（判「是不是人声自己的陡变」，不是过关线）",
                ),
            )],
        ),
        p(
            "floor_db",
            Some(-34.0),
            &[(
                -34.0,
                Src::Design(
                    "跳变至少到人声峰值以下 34 dB 才算（滤掉底噪毛刺，检测器参数，不是过关线；未经人耳定）",
                ),
            )],
        ),
    ],
};

/// ΔESTOI = 输出 − 原声（rubric C3）；过线 = 不落 D 档
pub const ESTOI: Lines = Lines {
    id: "V-ESTOI",
    edges: &[0.05, 0.0, -0.05],
    edges_src: Src::Rubric(&["ΔESTOI ≥ +0.05", "ΔESTOI ≥ 0", "ΔESTOI < −0.05"]),
    params: &[p(
        "min",
        Some(-0.05),
        &[(-0.05, Src::Rubric(&["ΔESTOI < −0.05"]))],
    )],
};

/// 评分模型（QA 层）：Δ = 输出 − 带噪原声。过线 0 是 012 的约束口径，不是外部锚点
const NOT_WORSE: Src =
    Src::Design("012 §2.3 第 4 条：L0 感知量只作「不比基线差」的约束；基线 = 带噪原声，所以 Δ ≥ 0");

pub const NRMOS_SCOREQ: Lines = Lines {
    id: "V-NRMOS-SCOREQ",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("min", Some(0.0), &[(0.0, NOT_WORSE)])],
};

pub const NRMOS_DISTILL: Lines = Lines {
    id: "V-NRMOS-DISTILL",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("min", Some(0.0), &[(0.0, NOT_WORSE)])],
};

/// 成绩单层：两个评分模型的 Δ 相差多少算「分歧，送人耳」。rubric 第 1 节第 3 条（原文标建议值）
pub const NRMOS_SPLIT: Lines = Lines {
    id: "SC-NRMOS-SPLIT",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "max",
        Some(0.3),
        &[(0.3, Src::Rubric(&["SCOREQ 比原声高 ≥ 0.3"]))],
    )],
};

/// 成绩单层（新规则 v2，人定 D85 题 1 = B）：组合内均值的 bootstrap 区间与 Holm 校正（调研 025 X1 · X4）。
pub const AGG: Lines = Lines {
    id: "SC-AGG",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p(
            "level",
            Some(0.95),
            &[(0.95, Src::Design("人定 D85 题 1：95% 区间三态"))],
        ),
        p(
            "reps",
            Some(2000.0),
            &[(2000.0, Src::Design("重采样次数，同调研 025 重算脚本"))],
        ),
        p(
            "seed",
            Some(20_261_030.0),
            &[(20_261_030.0, Src::Design("固定种子，同调研 025 重算脚本"))],
        ),
        p(
            "alpha",
            Some(0.025),
            &[(0.025, Src::Design("Holm 单侧 α：与 95% 双侧区间的「不过」一端对齐"))],
        ),
    ],
};

/// 成绩单层（v2）：听感 Δ 允许比原声差多少分（人定 D85 题 6 = B）。
pub const NRMOS_DELTA: Lines = Lines {
    id: "SC-NRMOS-DELTA",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "max",
        Some(0.1),
        &[(0.1, Src::Design("人定 D85 题 6 选 B：允许比原声差 0.1 分"))],
    )],
};

/// 表里全部 verifier。
pub const ALL: [&Lines; 23] = [
    &BGSTAB,
    &ONSET,
    &SPLVL,
    &MUSIC,
    &TIMBRE,
    &NR,
    &INTF,
    &VACUUM,
    &EAT_ASR,
    &FRAME,
    &MUTE,
    &SWITCH,
    &SIM_GATE,
    &SIM_FA,
    &SIM_QUIET,
    &SIM_SWLAT,
    &WEAK,
    &PUMPV,
    &CLICK,
    &ESTOI,
    &NRMOS_SCOREQ,
    &NRMOS_DISTILL,
    &NRMOS_SPLIT,
    &AGG,
    &NRMOS_DELTA,
];

/// 待定清单：已知不一致或没有出处的条目。全部是建议值，挂 012 拍板 #8 · D44。
pub const PENDING: [(&str, &str, Pend); 8] = [
    // 量法与 rubric N4 不同，数也不同（代码 2 / 4 / 6，N4 1.5 / 3 / 6）
    (
        "V-TIMBRE",
        "edges",
        Pend::Edges(&[2.0, 4.0, 6.0], Cause::Differ),
    ),
    // 过线值 −3 dB 没有出处
    ("V-SPLVL", "min", Pend::Value(-3.0, Cause::NoSource)),
    // 只有 3 dB 有出处，1 / 2 dB 没有
    (
        "V-ONSET",
        "edges",
        Pend::Edges(&[1.0, 2.0, 3.0], Cause::Differ),
    ),
    // −8 / −10 dB 没有出处（−6 / −20 是 rubric C2 档线）
    ("V-NR", "max", Pend::Value(-8.0, Cause::NoSource)),
    ("V-NR", "max", Pend::Value(-10.0, Cause::NoSource)),
    // 门控误关 0.05、词首 0.2 没有出处
    ("V-SIM-GATE", "max", Pend::Value(0.05, Cause::NoSource)),
    ("V-SIM-GATE", "onset_max", Pend::Value(0.2, Cause::NoSource)),
    // 012 只给了 A 档 1 dB²，过线值没有出处
    ("V-PUMPV", "max", Pend::Value(1.0, Cause::NoSource)),
];
