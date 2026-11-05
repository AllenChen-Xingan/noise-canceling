//! 新规则（v2，人定 D85）：在旧规则逐段计数的基础上，改判每一项、听感与场景（说明见 `specs/rubric/scorecard.md` 第四节）。
//!
//! 1. **物理硬事实项**（[`standard::PHYSICAL`]）：照旧逐段判，任一段不过即否决
//! 2. **吃字**（D85 题 3 = C）：单段比原声多错（删 + 替）≥ 5 字才算这段不过；任一段不过即否决
//! 3. **只报不判**（[`standard::V2_REPORT_ONLY`]，以及没在干净人声上算的 [`standard::CLEAN_PASS`] 三项）：照算区间，不进过不过
//! 4. **其余项**（D85 题 1 = B）：组合内均值 + 按源片段成组的 95% 区间三态。逐段「判不了」的段不计
//!    （多是不适用，如没有旁人说话的段不量压旁人）；有值的源片段少于 `SC-AGG.min_groups` 组 → 待测
//! 5. **听感**（题 1、题 6）：两个评分模型各自 Δ 均值区间对 −δ（δ = 0.1）三态；都过 → 过，都不过 → 不过，其余待测
//! 6. **Holm**（X4）：每项每个系统一族，判「不过」的要过 Holm 校正，没过的改判待测
//! 7. **场景**：同旧规则——否决项或任一判定项不过 → 不过；有待测或缺段 → 待测；全过 → 过

use std::collections::BTreeMap;

use super::aggregate::{Ci, bootstrap, holm, seed_for, tri};
use super::rules::{self, CardStatus, ItemCell, SceneCard, status_of};
use super::{ClipRow, Missing, standard};
use crate::thresholds as th;
use crate::{Better, Score};

pub const RULE_PHYS: &str = "逐段：物理硬事实，任一段不过即否决";
pub const RULE_EAT: &str = "逐段：比原声多错 ≥ 5 字的段即否决（人定 D85 题 3）";
pub const RULE_MEAN: &str = "均值 + 95% 区间三态，不过经 Holm 校正（人定 D85 题 1）";
pub const RULE_REPORT: &str = "只报不判（人定 D85）";
pub const RULE_OLD_INPUT: &str =
    "只报不判：旧口径（在带噪输出上算），D85 题 2 要在干净人声单独过一遍的输出上重跑";

/// 送人耳的最差听感段数。
pub const LISTEN_WORST: usize = 3;

fn key_of(c: &SceneCard) -> String {
    format!("{:?}|{:?}|{}|{}", c.scene, c.tier, c.device, c.system)
}

fn rows_of<'a>(c: &SceneCard, rows: &'a [ClipRow]) -> Vec<&'a ClipRow> {
    rows.iter()
        .filter(|r| {
            r.scene == c.scene && r.tier == c.tier && r.device == c.device && r.system == c.system
        })
        .collect()
}

fn score_in<'a>(r: &'a ClipRow, v: &str) -> Option<&'a Score> {
    r.scores
        .iter()
        .find(|(id, _, _)| *id == v)
        .map(|(_, _, s)| s)
}

fn extra(s: &Score, k: &str) -> Option<f64> {
    s.extra.iter().find(|(x, _)| *x == k).map(|(_, v)| *v)
}

/// v2 的过关线：[`standard::V2_PARAMS`] 换掉的用新值，其余沿用分数里记的线。
fn thr_v2(v: &str, row_thr: Option<f64>) -> Option<f64> {
    standard::V2_PARAMS
        .iter()
        .find(|(id, _, _)| *id == v)
        .map(|(_, _, x)| *x)
        .or(row_thr)
}

/// 比原声多错几个字（删 + 替）：分数是百分点，乘参考字数还原；四舍五入消掉浮点误差。
pub fn eat_chars(s: &Score) -> Option<f64> {
    Some((s.value? / 100.0 * extra(s, "ref_len")?).round())
}

fn ci_of(v: &str, rs: &[&ClipRow], thr: f64, better: Better, key: &str) -> Option<Ci> {
    let vals: Vec<(&str, f64)> = rs
        .iter()
        .filter_map(|r| Some((r.clip.as_str(), score_in(r, v)?.value?)))
        .collect();
    bootstrap(&vals, thr, better, seed_for(v, key))
}

fn redo_item(it: &mut ItemCell, rs: &[&ClipRow], key: &str) {
    let better = crate::verifiers::info(it.v).map_or(Better::Lower, |i| i.better);
    if standard::PHYSICAL.contains(&it.v) {
        it.rule = RULE_PHYS;
        return;
    }
    if it.v == "V-EAT-ASR" {
        let lim = th::EAT_ASR.def("fail_chars");
        let (mut p, mut f, mut u) = (0, 0, 0);
        for s in rs.iter().filter_map(|r| score_in(r, it.v)) {
            match eat_chars(s) {
                Some(n) if n >= lim => f += 1,
                Some(_) => p += 1,
                None => u += 1,
            }
        }
        (it.pass, it.fail, it.unknown) = (p, f, u);
        it.status = status_of(p, f);
        it.rule = RULE_EAT;
        it.ci = it.threshold.and_then(|t| ci_of(it.v, rs, t, better, key));
        return;
    }
    let old_input = standard::CLEAN_PASS.contains(&it.v)
        && rs
            .iter()
            .filter_map(|r| score_in(r, it.v))
            .any(|s| extra(s, standard::CLEAN_PASS_MARK) != Some(1.0));
    it.threshold = thr_v2(it.v, it.threshold);
    let Some(thr) = it.threshold else {
        it.status = CardStatus::Unknown;
        it.rule = RULE_MEAN;
        return;
    };
    it.ci = ci_of(it.v, rs, thr, better, key);
    it.status = judge(it.ci.as_ref(), thr, better);
    if standard::V2_REPORT_ONLY.contains(&it.v) {
        it.report_only = true;
        it.rule = RULE_REPORT;
    } else if old_input {
        it.report_only = true;
        it.rule = RULE_OLD_INPUT;
    } else {
        it.rule = RULE_MEAN;
    }
}

/// 区间三态；没有区间或源片段组数不够 → 待测。
fn judge(ci: Option<&Ci>, thr: f64, better: Better) -> CardStatus {
    match ci {
        Some(c) if c.groups as f64 >= th::AGG.def("min_groups") => tri(c, thr, better),
        _ => CardStatus::Unknown,
    }
}

fn redo_listen_model(m: &mut ItemCell, rs: &[&ClipRow], key: &str) {
    let d = th::NRMOS_DELTA.def("max");
    m.threshold = Some(-d);
    m.ci = ci_of(m.v, rs, -d, Better::Higher, key);
    m.status = judge(m.ci.as_ref(), -d, Better::Higher);
    m.rule = RULE_MEAN;
}

/// Holm：同一项 × 同一系统的所有组合是一族；区间判不过、但 Holm 不拒绝的改判待测。
fn apply_holm(cells: &mut [&mut ItemCell]) {
    let ps: Vec<f64> = cells
        .iter()
        .map(|c| c.ci.as_ref().map_or(1.0, |x| x.p_fail))
        .collect();
    let rej = holm(&ps);
    for (c, r) in cells.iter_mut().zip(rej) {
        if c.status == CardStatus::Fail && !c.report_only && !r && c.rule == RULE_MEAN {
            c.status = CardStatus::Unknown;
        }
    }
}

/// 场景状态、否决、拖后腿的声音：在每项改判之后重算（同旧规则的合成方式）。
fn finish(c: &mut SceneCard, rs: &[&ClipRow]) {
    let la = c.listen.models.first().map(|m| m.status);
    let lb = c.listen.models.get(1).map(|m| m.status);
    c.listen.status = Some(match (la, lb) {
        (Some(CardStatus::Pass), Some(CardStatus::Pass)) => CardStatus::Pass,
        (Some(CardStatus::Fail), Some(CardStatus::Fail)) => CardStatus::Fail,
        _ => CardStatus::Unknown,
    });
    c.veto_failed = c
        .items
        .iter()
        .filter(|i| i.veto && i.status == CardStatus::Fail)
        .map(|i| i.v)
        .collect();
    let judged = || c.items.iter().filter(|i| !i.report_only);
    let any_fail =
        judged().any(|i| i.status == CardStatus::Fail) || c.listen.status == Some(CardStatus::Fail);
    let any_unknown = judged().any(|i| i.status == CardStatus::Unknown)
        || c.listen.status == Some(CardStatus::Unknown);
    c.status = if any_fail {
        CardStatus::Fail
    } else if any_unknown || !c.missing.is_empty() || rs.is_empty() {
        CardStatus::Unknown
    } else {
        CardStatus::Pass
    };
    let mut for_drag = c.items.clone();
    if c.listen.status == Some(CardStatus::Fail) {
        for_drag.extend(c.listen.models.iter().cloned());
    }
    c.drags = rules::drags(&for_drag, rs);
    // 听感最差的几段送人耳（X1「最差 1–3 段送人耳」）：按两个模型里较差的 Δ 排
    let mut w: Vec<(f64, String)> = rs
        .iter()
        .filter_map(|r| {
            let a = score_in(r, rules::LISTEN[0])?.value?;
            let b = score_in(r, rules::LISTEN[1])?.value?;
            Some((a.min(b), r.clip.clone()))
        })
        .collect();
    w.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (d, clip) in w.into_iter().take(LISTEN_WORST) {
        if !c.listen.to_human.iter().any(|(x, _)| *x == clip) {
            c.listen
                .to_human
                .push((clip, format!("听感最差段之一（较差的 Δ {d:+.2}）")));
        }
    }
}

/// 新规则汇总。先按旧规则建表（逐段计数、均值、最差段照报），再逐项改判。
pub fn summarize(rows: &[ClipRow], missing: &[Missing]) -> Vec<SceneCard> {
    let mut cards = rules::summarize(rows, missing);
    for c in &mut cards {
        let rs = rows_of(c, rows);
        let key = key_of(c);
        for it in &mut c.items {
            redo_item(it, &rs, &key);
        }
        for m in &mut c.listen.models {
            redo_listen_model(m, &rs, &key);
        }
    }
    // Holm：按 (项, 系统) 分族
    let mut fam: BTreeMap<(&'static str, String), Vec<&mut ItemCell>> = BTreeMap::new();
    for c in &mut cards {
        let sys = c.system.clone();
        for it in c.items.iter_mut().chain(c.listen.models.iter_mut()) {
            fam.entry((it.v, sys.clone())).or_default().push(it);
        }
    }
    for cells in fam.values_mut() {
        apply_holm(cells);
    }
    for c in &mut cards {
        let rs = rows_of(c, rows);
        finish(c, &rs);
    }
    cards
}

#[cfg(test)]
#[path = "rules_v2_tests.rs"]
mod tests;
