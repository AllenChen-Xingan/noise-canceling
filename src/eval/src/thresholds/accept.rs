//! 成绩单层：开训第一批判「过 / 不过」的验收线（计划 014 第三节第 3–4 条）。
//!
//! 正本在这里，别处只许引用：
//! - `docs/plan/014_training-kickoff.md` 第三节在每个数后面标 `（SC-ACCEPT.<键>）`，
//!   `src/eval/tests/accept_doc.rs` 把那些数与这里的默认值逐个比对，对不上即红
//! - 判读命令 `vdn-eval accept`（`src/eval/src/scorecard/accept.rs`）从这里取线，不写数字
//!
//! 延迟不登记成新数：014 要的是「候选 V-LAT 仍等于基线」，线是基线成绩单量出来的那个数本身
//! （`scorecard::accept` 从对照成绩单读），不是另立一条阈值。

use super::{Lines, Param, Src};

const fn p(key: &'static str, default: Option<f64>, values: &'static [(f64, Src)]) -> Param {
    Param {
        key,
        default,
        values,
    }
}

const NO_EDGES: Src = Src::Design("不分档：这一族是验收线，不是成绩单分档");

/// 014 第三节的验收线。每个数的出处都是 014 原文，`Src::Design` 的说明里抄了那句话。
///
/// `seeds` 不是 014 里的数：种子数来自 D99 第 1 条（每组 3 个种子），014 第三节没写这个数，
/// 所以 014 第三节不标它的键（`src/eval/tests/accept_doc.rs` 只比对 014 写了的那几个键）。
pub const ACCEPT: Lines = Lines {
    id: "SC-ACCEPT",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p(
            "estoi_gain",
            Some(0.01),
            &[(
                0.01,
                Src::Design("014 第三节第 3 条：目标场景可懂度均值 ≥ +0.01，且区间下界 > 0"),
            )],
        ),
        p(
            "nrmos_gain",
            Some(0.05),
            &[(
                0.05,
                Src::Design(
                    "014 第三节第 3 条：两个评分模型（V-NRMOS-SCOREQ · V-NRMOS-DISTILL）各 ≥ +0.05 才算听感变好",
                ),
            )],
        ),
        p(
            "onset_regress",
            Some(1.0),
            &[(
                1.0,
                Src::Design("014 第三节第 4 条：字头退步区间上界 ≤ 1 dB"),
            )],
        ),
        p(
            "bgstab_regress",
            Some(2.0),
            &[(
                2.0,
                Src::Design("014 第三节第 4 条：背景跳变退步区间上界 ≤ 2 dB"),
            )],
        ),
        p(
            "timbre_regress",
            Some(1.0),
            &[(
                1.0,
                Src::Design("014 第三节第 4 条：音色退步区间上界 ≤ 1 dB"),
            )],
        ),
        p(
            "seeds",
            Some(3.0),
            &[(3.0, Src::Design("人定 D99 第 1 条：每组 3 个种子"))],
        ),
    ],
};
