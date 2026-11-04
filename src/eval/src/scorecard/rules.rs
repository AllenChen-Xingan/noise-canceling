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

use super::ClipRow;
use crate::scene::{SceneKey, SoundGroup, Tier};
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
pub enum Status {
    Fail,
    Unknown,
    Pass,
}

impl Status {
    pub fn zh(self) -> &'static str {
        match self {
            Status::Pass => "过",
            Status::Fail => "不过",
            Status::Unknown => "待测",
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ItemCell {
    pub v: &'static str,
    pub veto: bool,
    pub unit: &'static str,
    pub better: &'static str,
    pub pass: usize,
    pub fail: usize,
    pub unknown: usize,
    pub mean: Option<f64>,
    pub worst: Option<f64>,
    pub worst_clip: Option<String>,
    pub threshold: Option<f64>,
    pub status: Status,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct ListenCell {
    pub pass: usize,
    pub fail: usize,
    pub unknown: usize,
    /// 送人耳的段：(段 id, 原因)
    pub to_human: Vec<(String, String)>,
    pub status: Option<Status>,
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
    pub scene: SceneKey,
    pub tier: Tier,
    pub device: String,
    pub system: String,
    pub clips: usize,
    pub status: Status,
    pub veto_failed: Vec<&'static str>,
    pub items: Vec<ItemCell>,
    pub listen: ListenCell,
    pub drags: Vec<Drag>,
}

fn status_of(pass: usize, fail: usize) -> Status {
    if fail > 0 {
        Status::Fail
    } else if pass > 0 {
        Status::Pass
    } else {
        Status::Unknown
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
pub fn listen_one(scores: &[(&'static str, bool, Score)]) -> (Status, Option<String>) {
    let get = |id: &str| scores.iter().find(|(v, _, _)| *v == id).map(|(_, _, s)| s);
    let (Some(a), Some(b)) = (get(LISTEN[0]), get(LISTEN[1])) else {
        return (Status::Unknown, None);
    };
    let (Some(da), Some(db)) = (a.value, b.value) else {
        return (Status::Unknown, None);
    };
    let split = th::NRMOS_SPLIT.def("max");
    match (a.verdict.is_pass(), b.verdict.is_pass()) {
        (true, true) if (da - db).abs() >= split => (
            Status::Pass,
            Some(format!("两个评分模型都说不比原声差，但 Δ 相差 {:.2} ≥ {split}", (da - db).abs())),
        ),
        (true, true) => (Status::Pass, None),
        (false, false) if a.verdict.is_fail() && b.verdict.is_fail() => (Status::Fail, None),
        _ if a.verdict.is_fail() || b.verdict.is_fail() => (
            Status::Unknown,
            Some(format!("一个说好一个说差：SCOREQ Δ {da:+.2} · Distill-MOS Δ {db:+.2}")),
        ),
        _ => (Status::Unknown, None),
    }
}

fn listen_cell(rows: &[&ClipRow]) -> ListenCell {
    let mut c = ListenCell::default();
    for r in rows {
        let (st, why) = listen_one(&r.scores);
        match st {
            Status::Pass => c.pass += 1,
            Status::Fail => c.fail += 1,
            Status::Unknown => c.unknown += 1,
        }
        if let Some(w) = why {
            c.to_human.push((r.clip.clone(), w));
        }
    }
    c.status = Some(status_of(c.pass, c.fail));
    c
}

/// 不过的项按声音类别分组比较。
fn drags(items: &[ItemCell], rows: &[&ClipRow]) -> Vec<Drag> {
    let mut out = Vec::new();
    let classes: BTreeSet<(String, SoundGroup)> =
        rows.iter().flat_map(|r| r.sounds.iter().cloned()).collect();
    for it in items.iter().filter(|i| i.status == Status::Fail) {
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
            let gap = (w.len() >= MIN_SPLIT && wo.len() >= MIN_SPLIT).then(|| match better {
                Better::Lower => mw - mo,
                Better::Higher => mo - mw,
            });
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
        per.sort_by(|a, b| {
            b.gap
                .unwrap_or(f64::NEG_INFINITY)
                .total_cmp(&a.gap.unwrap_or(f64::NEG_INFINITY))
        });
        out.extend(per.into_iter().take(3));
    }
    out
}

/// 按 场景 × 档 × 设备 × 系统 汇总。
pub fn summarize(rows: &[ClipRow]) -> Vec<SceneCard> {
    let mut by: BTreeMap<(SceneKey, Tier, String, String), Vec<&ClipRow>> = BTreeMap::new();
    for r in rows {
        by.entry((r.scene, r.tier, r.device.clone(), r.system.clone()))
            .or_default()
            .push(r);
    }
    by.into_iter()
        .map(|((scene, tier, device, system), rs)| {
            let order: Vec<(&'static str, bool)> = rs[0]
                .scores
                .iter()
                .filter(|(v, _, _)| !LISTEN.contains(v))
                .map(|(v, veto, _)| (*v, *veto))
                .collect();
            let items: Vec<ItemCell> = order.iter().map(|(v, veto)| item_cell(v, *veto, &rs)).collect();
            let listen = listen_cell(&rs);
            let veto_failed: Vec<&'static str> = items
                .iter()
                .filter(|i| i.veto && i.status == Status::Fail)
                .map(|i| i.v)
                .collect();
            let any_fail = items.iter().any(|i| i.status == Status::Fail)
                || listen.status == Some(Status::Fail);
            let any_unknown = items.iter().any(|i| i.status == Status::Unknown)
                || listen.status == Some(Status::Unknown);
            let status = if any_fail {
                Status::Fail
            } else if any_unknown {
                Status::Unknown
            } else {
                Status::Pass
            };
            let drags = drags(&items, &rs);
            SceneCard {
                scene,
                tier,
                device,
                system,
                clips: rs.len(),
                status,
                veto_failed,
                items,
                listen,
                drags,
            }
        })
        .collect()
}

/// 最差的几个场景（下标）：不过在前；同为不过时否决项多的在前，再按不过项数。
pub fn worst(cards: &[SceneCard]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..cards.len()).collect();
    let fails = |c: &SceneCard| c.items.iter().filter(|i| i.status == Status::Fail).count();
    idx.sort_by_key(|&i| {
        let c = &cards[i];
        (c.status, std::cmp::Reverse(c.veto_failed.len()), std::cmp::Reverse(fails(c)))
    });
    idx.into_iter()
        .filter(|&i| cards[i].status != Status::Pass)
        .take(WORST_N)
        .collect()
}
