//! 成绩单的汇总规则（说明见 `specs/rubric/scorecard.md` 第四至六节）：
//!
//! 1. **一项在一个场景里**：任一段不过 → 不过（最差一段决定）；没有不过、至少一段过 → 过；全都判不了 → 待测。
//!    均值与最差值只作参考
//! 2. **听感**（两个评分模型）：每段两个 Δ 都不比原声差 → 过；都差 → 不过；一个说好一个说差 → 送人耳（待测）；
//!    两个都过但 Δ 相差 ≥ 分歧线（rubric 0.3，建议值）→ 仍算过，也列入送人耳
//! 3. **场景**：否决项任一不过 → 不过（后面的分只记数）；其余任一项或听感不过 → 不过；
//!    都没不过但有待测 → 待测；全过 → 过
//! 4. **拖后腿的声音**：场景里每个不过的项，按「段里有没有这类声音」分两组比均值，差最大的列出来；
//!    两组各少于 [`MIN_SPLIT`] 段时只列「样本不够」，不下结论
//! 5. **全局**：不给总均值当结论；按「不过 → 待测 → 过」、再按不过项数排，列最差的场景

use std::collections::{BTreeMap, BTreeSet};

use super::{ClipRow, Missing};
use crate::scene::{MeetingPlace, SoundGroup, Tier};
use crate::thresholds as th;
use crate::{Better, Score, Verdict};

/// 分组比较时每组至少几段。
pub const MIN_SPLIT: usize = 3;
/// 全局列几个最差场景。
pub const WORST_N: usize = 3;
/// 听感两项（成绩单层合成一条，不单独判）。
pub const LISTEN: [&str; 2] = ["V-NRMOS-SCOREQ", "V-NRMOS-DISTILL"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CardStatus {
    Fail,
    Unknown,
    Pass,
}

impl CardStatus {
    pub fn zh(self) -> &'static str {
        match self {
            CardStatus::Pass => "过",
            CardStatus::Fail => "不过",
            CardStatus::Unknown => "待测",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ItemCell {
    pub v: &'static str,
    pub veto: bool,
    /// 只报不判：照常列出，不进场景的过不过、不做「拖后腿的声音」
    pub report_only: bool,
    pub unit: &'static str,
    pub better: &'static str,
    pub pass: usize,
    pub fail: usize,
    pub unknown: usize,
    pub mean: Option<f64>,
    pub worst: Option<f64>,
    pub worst_clip: Option<String>,
    pub threshold: Option<f64>,
    pub status: CardStatus,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ListenCell {
    /// 两个评分模型各自的逐项汇总（过 / 不过 / 待测段数、均值、最差段）；不单独决定场景，只给人看、给分组比较用
    pub models: Vec<ItemCell>,
    /// 两个评分模型各自的 Δ 均值（输出 − 原声；判不了的段不计）
    pub mean_scoreq: Option<f64>,
    pub mean_distill: Option<f64>,
    pub pass: usize,
    pub fail: usize,
    pub unknown: usize,
    /// 送人耳的段：(段 id, 原因)
    pub to_human: Vec<(String, String)>,
    pub status: Option<CardStatus>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Drag {
    pub v: &'static str,
    pub cls: String,
    pub group: SoundGroup,
    pub n_with: usize,
    pub n_without: usize,
    pub mean_with: f64,
    pub mean_without: f64,
    /// 「有这类声音」比「没有」差多少（正数 = 更差）；样本不够时为 None
    pub gap: Option<f64>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SceneCard {
    pub scene: MeetingPlace,
    pub tier: Tier,
    pub device: String,
    pub system: String,
    pub clips: usize,
    /// 没打分的段（或整条任务）：(id, 原因)
    pub missing: Vec<(String, String)>,
    pub status: CardStatus,
    pub veto_failed: Vec<&'static str>,
    pub items: Vec<ItemCell>,
    pub listen: ListenCell,
    pub drags: Vec<Drag>,
}

fn status_of(pass: usize, fail: usize) -> CardStatus {
    if fail > 0 {
        CardStatus::Fail
    } else if pass > 0 {
        CardStatus::Pass
    } else {
        CardStatus::Unknown
    }
}

fn worse(b: Better, a: f64, c: f64) -> bool {
    b.worse(a, c)
}

fn item_cell(v: &'static str, veto: bool, rows: &[&ClipRow]) -> ItemCell {
    let info = crate::verifiers::info(v);
    let better = info.map_or(Better::Lower, |i| i.better);
    let (mut pass, mut fail, mut unknown) = (0, 0, 0);
    let mut vals = Vec::new();
    let mut worst: Option<(f64, String)> = None;
    let mut threshold = None;
    for r in rows {
        let Some((_, _, s)) = r.scores.iter().find(|(id, _, _)| *id == v) else {
            continue;
        };
        match s.verdict {
            Verdict::Pass => pass += 1,
            Verdict::Fail => fail += 1,
            Verdict::Unknown(_) => unknown += 1,
        }
        threshold = threshold.or(s.threshold);
        if let Some(x) = s.value {
            vals.push(x);
            if worst.as_ref().is_none_or(|(w, _)| worse(better, x, *w)) {
                worst = Some((x, r.clip.clone()));
            }
        }
    }
    ItemCell {
        v,
        veto,
        report_only: super::standard::is_report_only(v),
        unit: info.map_or("", |i| i.unit),
        better: better.as_str(),
        pass,
        fail,
        unknown,
        mean: (!vals.is_empty()).then(|| vals.iter().sum::<f64>() / vals.len() as f64),
        worst: worst.as_ref().map(|w| w.0),
        worst_clip: worst.map(|w| w.1),
        threshold,
        status: status_of(pass, fail),
    }
}

/// 一段的听感判定。
pub fn listen_one(scores: &[(&'static str, bool, Score)]) -> (CardStatus, Option<String>) {
    let get = |id: &str| scores.iter().find(|(v, _, _)| *v == id).map(|(_, _, s)| s);
    let (Some(a), Some(b)) = (get(LISTEN[0]), get(LISTEN[1])) else {
        return (CardStatus::Unknown, None);
    };
    let (Some(da), Some(db)) = (a.value, b.value) else {
        return (CardStatus::Unknown, None);
    };
    let split = th::NRMOS_SPLIT.def("max");
    let (pa, pb) = (a.verdict.is_pass(), b.verdict.is_pass());
    let (fa, fb) = (a.verdict.is_fail(), b.verdict.is_fail());
    if pa && pb {
        let gap = (da - db).abs();
        let why = (gap >= split)
            .then(|| format!("两个评分模型都说不比原声差，但 Δ 相差 {gap:.2} ≥ {split}"));
        (CardStatus::Pass, why)
    } else if fa && fb {
        (CardStatus::Fail, None)
    } else if (pa && fb) || (fa && pb) {
        (
            CardStatus::Unknown,
            Some(format!(
                "一个说好一个说差：SCOREQ Δ {da:+.2} · Distill-MOS Δ {db:+.2}"
            )),
        )
    } else {
        (CardStatus::Unknown, None)
    }
}

fn listen_cell(rows: &[&ClipRow]) -> ListenCell {
    let mut c = ListenCell::default();
    for r in rows {
        let (st, why) = listen_one(&r.scores);
        match st {
            CardStatus::Pass => c.pass += 1,
            CardStatus::Fail => c.fail += 1,
            CardStatus::Unknown => c.unknown += 1,
        }
        if let Some(w) = why {
            c.to_human.push((r.clip.clone(), w));
        }
    }
    // 有一段判不了（模型出错、两个模型一个说好一个说差）、又没有不过的 → 待测，不是过（codex 第 2 轮 P1 #2）
    c.status = Some(if c.fail > 0 {
        CardStatus::Fail
    } else if c.unknown > 0 || c.pass == 0 {
        CardStatus::Unknown
    } else {
        CardStatus::Pass
    });
    let mean_of = |id: &str| {
        let v: Vec<f64> = rows
            .iter()
            .filter_map(|r| r.scores.iter().find(|(x, _, _)| *x == id))
            .filter_map(|(_, _, s)| s.value)
            .collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    };
    c.mean_scoreq = mean_of(LISTEN[0]);
    c.mean_distill = mean_of(LISTEN[1]);
    c
}

/// 不过的项按声音类别分组比较。
fn drags(items: &[ItemCell], rows: &[&ClipRow]) -> Vec<Drag> {
    let mut out = Vec::new();
    let classes: BTreeSet<(String, SoundGroup)> =
        rows.iter().flat_map(|r| r.sounds.iter().cloned()).collect();
    for it in items
        .iter()
        .filter(|i| i.status == CardStatus::Fail && !i.report_only)
    {
        let better = crate::verifiers::info(it.v).map_or(Better::Lower, |i| i.better);
        let mut per: Vec<Drag> = Vec::new();
        for (cls, group) in &classes {
            let (mut w, mut wo) = (Vec::new(), Vec::new());
            for r in rows {
                let Some(x) = r
                    .scores
                    .iter()
                    .find(|(id, _, _)| *id == it.v)
                    .and_then(|(_, _, s)| s.value)
                else {
                    continue;
                };
                if r.sounds.iter().any(|(c, _)| c == cls) {
                    w.push(x);
                } else {
                    wo.push(x);
                }
            }
            let m = |v: &[f64]| {
                if v.is_empty() {
                    f64::NAN
                } else {
                    v.iter().sum::<f64>() / v.len() as f64
                }
            };
            let (mw, mo) = (m(&w), m(&wo));
            let d = match better {
                Better::Lower => mw - mo,
                Better::Higher => mo - mw,
            };
            let gap = (w.len() >= MIN_SPLIT && wo.len() >= MIN_SPLIT).then_some(d);
            per.push(Drag {
                v: it.v,
                cls: cls.clone(),
                group: *group,
                n_with: w.len(),
                n_without: wo.len(),
                mean_with: mw,
                mean_without: mo,
                gap,
            });
        }
        // 只列真拖后腿的（有这类声音的段更差，gap > 0），按差距从大到小取 3 个；
        // 分不出组的（样本不够）最多再列 2 个，提醒「这类声音没法判」
        let (mut hit, short): (Vec<Drag>, Vec<Drag>) = per
            .into_iter()
            .filter(|d| d.gap.is_none_or(|g| g > 0.0))
            .partition(|d| d.gap.is_some());
        hit.sort_by(|a, b| b.gap.unwrap_or(0.0).total_cmp(&a.gap.unwrap_or(0.0)));
        out.extend(hit.into_iter().take(3));
        out.extend(short.into_iter().take(2));
    }
    out
}

type Key = (MeetingPlace, Tier, String, String);

/// 按 场景 × 档 × 设备 × 系统 汇总。`missing` 里的段没打分：该组合有缺段就不能判「过」（最多待测）；
/// 一段都没打成的组合也出一行（待测），不会从成绩单里消失。
pub fn summarize(rows: &[ClipRow], missing: &[Missing]) -> Vec<SceneCard> {
    let mut by: BTreeMap<Key, (Vec<&ClipRow>, Vec<&Missing>)> = BTreeMap::new();
    for r in rows {
        by.entry((r.scene, r.tier, r.device.clone(), r.system.clone()))
            .or_default()
            .0
            .push(r);
    }
    for m in missing {
        by.entry((m.scene, m.tier, m.device.clone(), m.system.clone()))
            .or_default()
            .1
            .push(m);
    }
    by.into_iter()
        .map(|((scene, tier, device, system), (rs, ms))| {
            let order: Vec<(&'static str, bool)> = rs
                .first()
                .map(|r| {
                    r.scores
                        .iter()
                        .filter(|(v, _, _)| !LISTEN.contains(v))
                        .map(|(v, veto, _)| (*v, *veto))
                        .collect()
                })
                .unwrap_or_default();
            let items: Vec<ItemCell> = order
                .iter()
                .map(|(v, veto)| item_cell(v, *veto, &rs))
                .collect();
            let mut listen = listen_cell(&rs);
            listen.models = LISTEN.iter().map(|v| item_cell(v, false, &rs)).collect();
            let veto_failed: Vec<&'static str> = items
                .iter()
                .filter(|i| i.veto && i.status == CardStatus::Fail)
                .map(|i| i.v)
                .collect();
            let any_fail = items
                .iter()
                .filter(|i| !i.report_only)
                .any(|i| i.status == CardStatus::Fail)
                || listen.status == Some(CardStatus::Fail);
            let any_unknown = items
                .iter()
                .filter(|i| !i.report_only)
                .any(|i| i.status == CardStatus::Unknown)
                || listen.status == Some(CardStatus::Unknown);
            let status = if any_fail {
                CardStatus::Fail
            } else if any_unknown || !ms.is_empty() || rs.is_empty() {
                CardStatus::Unknown
            } else {
                CardStatus::Pass
            };
            // 听感不过时，对说「比原声差」的那个评分模型也做声音分组（codex 第 2 轮 P2 #3）
            let mut for_drag: Vec<ItemCell> = items.clone();
            if listen.status == Some(CardStatus::Fail) {
                for_drag.extend(listen.models.iter().cloned());
            }
            let drags = drags(&for_drag, &rs);
            SceneCard {
                scene,
                tier,
                device,
                system,
                clips: rs.len(),
                missing: ms.iter().map(|m| (m.what.clone(), m.why.clone())).collect(),
                status,
                veto_failed,
                items,
                listen,
                drags,
            }
        })
        .collect()
}

impl SceneCard {
    /// 不过的项（不含只报不判的）；听感不过算一项，记作「听感」（codex 第 3 轮 P2 #3）。
    pub fn failed_items(&self) -> Vec<&'static str> {
        let mut v: Vec<&'static str> = self
            .items
            .iter()
            .filter(|i| i.status == CardStatus::Fail && !i.report_only)
            .map(|i| i.v)
            .collect();
        if self.listen.status == Some(CardStatus::Fail) {
            v.push("听感");
        }
        v
    }
}

/// 最差的几个场景（下标）：不过在前；同为不过时否决项多的在前，再按不过项数。
pub fn worst(cards: &[SceneCard]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..cards.len()).collect();
    let fails = |c: &SceneCard| c.failed_items().len();
    idx.sort_by_key(|&i| {
        let c = &cards[i];
        (
            c.status,
            std::cmp::Reverse(c.veto_failed.len()),
            std::cmp::Reverse(fails(c)),
        )
    });
    idx.into_iter()
        .filter(|&i| cards[i].status != CardStatus::Pass)
        .take(WORST_N)
        .collect()
}

#[cfg(test)]
mod tests;
