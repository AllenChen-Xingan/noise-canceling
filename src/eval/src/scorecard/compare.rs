//! 配对比较（计划 014 第二节第 0 步 ①，调研 035 缺口 1）：同一批片段上，候选 b 比基线 a 涨了、退了还是说不准。
//!
//! - **配对**：按片段 id（`clips.jsonl` 的 `clip`）配对；每项只用两边都有有限值的段。只在一边出现的段单列
//! - **区间**：每段差值 = b − a，按源片段成组 bootstrap（[`aggregate::bootstrap`]，水平 / 次数 / 种子 / 最少组数取
//!   `SC-AGG`，不另立参数）。种子 = [`aggregate::seed_for`]（项名 + 比较范围），同样输入结果逐位相同
//! - **三态**：差值区间整个在「更好」一侧 = 涨；整个在「更差」一侧 = 退；碰到 0 或组数不够 = 不确定。
//!   比的是 0（有没有变），不是过关线；没有新的线
//! - **否决类**（[`standard::VETO`]）：逐段判这段是否被否决，按片段 id 列出 b 新增 / 消失的否决段。逐段判法：
//!   物理硬事实项看逐段判定；吃字看「比原声多错 ≥ `V-EAT-ASR.fail_chars` 字」；其余（人声被压低）看单段值是否越过
//!   新规则的线（[`rules_v2::thr_v2`]）
//! - **种子噪声底**：同一配置两个种子的一对 `clips.jsonl` 照同样算法比一遍，作为参照列（计划 014 第三节第 2 条）；
//!   只报，不改三态
//! - 不做多重比较校正：每项一个独立的 95% 区间；项多时偶然「涨 / 退」的会有，参照种子噪声底看

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::aggregate::{Ci, bootstrap, seed_for};
use super::{ClipRow, rules_v2, standard};
use crate::thresholds as th;
use crate::{Better, Score, Verdict};

/// 一项的判断。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Call {
    /// b 比 a 好（区间整个在更好一侧）
    Up,
    /// b 比 a 差
    Down,
    /// 区间碰到 0，或有值的源片段组数不够
    Unsure,
}

impl Call {
    pub fn zh(self) -> &'static str {
        match self {
            Call::Up => "涨",
            Call::Down => "退",
            Call::Unsure => "不确定",
        }
    }
    pub const ALL: [Call; 3] = [Call::Up, Call::Down, Call::Unsure];
}

/// 一项的配对差值。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ItemDiff {
    pub v: &'static str,
    pub unit: &'static str,
    pub better: &'static str,
    /// 两边都有有限值的段数
    pub pairs: usize,
    /// 配上了、但只有一边有值的段数（判不了的段不计）
    pub one_sided: usize,
    pub mean_a: Option<f64>,
    pub mean_b: Option<f64>,
    /// 差值（b − a）的均值与区间
    pub diff: Option<Ci>,
    pub call: Call,
    /// 种子噪声底：同配置两个种子的差值区间（只报）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise: Option<Ci>,
}

/// 一个否决项按片段 id 的增减。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct VetoDiff {
    pub v: &'static str,
    pub rule: &'static str,
    pub a: usize,
    pub b: usize,
    /// b 被否决、a 没有的段
    pub new_in_b: Vec<String>,
    /// a 被否决、b 没有的段
    pub gone_in_b: Vec<String>,
}

/// 一个比较范围（全部段，或 `--by-scene` 时的一个场景）。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Compare {
    /// `all`，或场景键
    pub scope: String,
    pub pairs: usize,
    pub only_a: Vec<String>,
    pub only_b: Vec<String>,
    pub items: Vec<ItemDiff>,
    pub vetoes: Vec<VetoDiff>,
}

pub const VETO_PHYS: &str = "逐段判定不过";
pub const VETO_EAT: &str = "比原声多错 ≥ fail_chars 字";
pub const VETO_LINE: &str = "单段值越过新规则的线";

/// 否决项的逐段判法名（规约表与它比对）。
pub fn veto_rule(v: &str) -> &'static str {
    if standard::PHYSICAL.contains(&v) {
        VETO_PHYS
    } else if v == "V-EAT-ASR" {
        VETO_EAT
    } else {
        VETO_LINE
    }
}

/// 这一段在否决项 `v` 上是否被否决（`None` = 判不了）。
pub fn vetoed(v: &str, s: &Score) -> Option<bool> {
    match veto_rule(v) {
        VETO_PHYS => match s.verdict {
            Verdict::Pass => Some(false),
            Verdict::Fail => Some(true),
            Verdict::Unknown(_) => None,
        },
        VETO_EAT => Some(rules_v2::eat_chars(s)? >= th::EAT_ASR.def("fail_chars")),
        _ => {
            let x = s.value.filter(|x| x.is_finite())?;
            let thr = rules_v2::thr_v2(v, s.threshold)?;
            Some(s.better.worse(x, thr))
        }
    }
}

/// 差值区间的三态（比 0）。
pub fn call_of(ci: Option<&Ci>, better: Better) -> Call {
    let Some(c) = ci.filter(|c| c.groups as f64 >= th::AGG.def("min_groups")) else {
        return Call::Unsure;
    };
    let (up, down) = match better {
        Better::Higher => (c.lo > 0.0, c.hi < 0.0),
        Better::Lower => (c.hi < 0.0, c.lo > 0.0),
    };
    if up {
        Call::Up
    } else if down {
        Call::Down
    } else {
        Call::Unsure
    }
}

fn score<'a>(r: &'a ClipRow, v: &str) -> Option<&'a Score> {
    r.scores.iter().find(|(id, _, _)| *id == v).map(|(_, _, s)| s)
}

fn finite(r: &ClipRow, v: &str) -> Option<f64> {
    score(r, v)?.value.filter(|x| x.is_finite())
}

fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// 按片段 id 建索引；同一 id 出现两次（文件里有多个系统、或拼了两个 build）报错。
pub fn index(rows: &[ClipRow], side: &str) -> Result<BTreeMap<&str, &ClipRow>, String> {
    let mut m = BTreeMap::new();
    for r in rows {
        if m.insert(r.clip.as_str(), r).is_some() {
            return Err(format!(
                "{side} 里片段 {} 出现不止一次：一个文件只放一个系统（用 --system-{side} 选）",
                r.clip
            ));
        }
    }
    Ok(m)
}

/// 只留某个系统的行；不给系统且文件里不止一个系统时报错。
pub fn select(rows: Vec<ClipRow>, sys: Option<&str>, side: &str) -> Result<Vec<ClipRow>, String> {
    let names: BTreeSet<&str> = rows.iter().map(|r| r.system.as_str()).collect();
    match sys {
        Some(s) if !names.contains(s) => Err(format!("{side} 里没有系统 {s}（有 {names:?}）")),
        Some(s) => Ok(rows.into_iter().filter(|r| r.system == s).collect()),
        None if names.len() > 1 => Err(format!(
            "{side} 里有多个系统 {names:?}：用 --system-{side} 选一个"
        )),
        None => Ok(rows),
    }
}

/// 项名按在 a 里第一次出现的顺序，再补 b 独有的。
fn item_ids(a: &[&ClipRow], b: &[&ClipRow]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for r in a.iter().chain(b) {
        for (v, _, _) in &r.scores {
            if !out.contains(v) {
                out.push(v);
            }
        }
    }
    out
}

/// 配对差值的区间：`pairs` = (片段 id, a 行, b 行)。
fn diff_ci(pairs: &[(&str, &ClipRow, &ClipRow)], v: &str, better: Better, scope: &str) -> (Option<Ci>, usize, usize) {
    let mut d: Vec<(&str, f64)> = Vec::new();
    let mut one = 0;
    for (id, ra, rb) in pairs {
        match (finite(ra, v), finite(rb, v)) {
            (Some(x), Some(y)) => d.push((id, y - x)),
            (None, None) => {}
            _ => one += 1,
        }
    }
    let n = d.len();
    (bootstrap(&d, 0.0, better, seed_for(v, &format!("compare|{scope}"))), n, one)
}

fn pairs_of<'a>(
    a: &BTreeMap<&'a str, &'a ClipRow>,
    b: &BTreeMap<&'a str, &'a ClipRow>,
) -> Result<Vec<(&'a str, &'a ClipRow, &'a ClipRow)>, String> {
    let mut out = Vec::new();
    for (id, ra) in a {
        if let Some(rb) = b.get(id) {
            if (ra.scene, ra.tier, &ra.device) != (rb.scene, rb.tier, &rb.device) {
                return Err(format!(
                    "片段 {id} 两边的 场景 / 档 / 设备 不一致：两个文件不是同一个 build"
                ));
            }
            out.push((*id, *ra, *rb));
        }
    }
    Ok(out)
}

fn one_scope(
    scope: &str,
    a: &BTreeMap<&str, &ClipRow>,
    b: &BTreeMap<&str, &ClipRow>,
    noise: Option<&[(&str, &ClipRow, &ClipRow)]>,
) -> Result<Compare, String> {
    let pairs = pairs_of(a, b)?;
    let ra: Vec<&ClipRow> = pairs.iter().map(|p| p.1).collect();
    let rb: Vec<&ClipRow> = pairs.iter().map(|p| p.2).collect();
    let mut items = Vec::new();
    for v in item_ids(&ra, &rb) {
        let info = crate::verifiers::info(v);
        let better = info.map_or(Better::Lower, |i| i.better);
        let (diff, n, one) = diff_ci(&pairs, v, better, scope);
        let both = |side: &[&ClipRow], other: &[&ClipRow]| -> Vec<f64> {
            side.iter()
                .zip(other)
                .filter_map(|(x, y)| finite(y, v).and(finite(x, v)))
                .collect()
        };
        items.push(ItemDiff {
            v,
            unit: info.map_or("", |i| i.unit),
            better: better.as_str(),
            pairs: n,
            one_sided: one,
            mean_a: mean(&both(&ra, &rb)),
            mean_b: mean(&both(&rb, &ra)),
            call: call_of(diff.as_ref(), better),
            diff,
            noise: noise.and_then(|np| diff_ci(np, v, better, &format!("noise|{scope}")).0),
        });
    }
    let vetoes = standard::VETO
        .iter()
        .map(|i| {
            let mut d = VetoDiff {
                v: i.v,
                rule: veto_rule(i.v),
                a: 0,
                b: 0,
                new_in_b: Vec::new(),
                gone_in_b: Vec::new(),
            };
            for (id, x, y) in &pairs {
                let va = score(x, i.v).and_then(|s| vetoed(i.v, s)) == Some(true);
                let vb = score(y, i.v).and_then(|s| vetoed(i.v, s)) == Some(true);
                d.a += usize::from(va);
                d.b += usize::from(vb);
                if vb && !va {
                    d.new_in_b.push((*id).to_string());
                }
                if va && !vb {
                    d.gone_in_b.push((*id).to_string());
                }
            }
            d
        })
        .collect();
    Ok(Compare {
        scope: scope.to_string(),
        pairs: pairs.len(),
        only_a: a.keys().filter(|k| !b.contains_key(*k)).map(|k| k.to_string()).collect(),
        only_b: b.keys().filter(|k| !a.contains_key(*k)).map(|k| k.to_string()).collect(),
        items,
        vetoes,
    })
}

/// 比较 a（基线）与 b（候选）。`noise` = 同配置两个种子的一对（种子噪声底）。`by_scene` 时先出全部，再逐场景各出一份。
pub fn compare(
    a: &[ClipRow],
    b: &[ClipRow],
    noise: Option<(&[ClipRow], &[ClipRow])>,
    by_scene: bool,
) -> Result<Vec<Compare>, String> {
    let (ia, ib) = (index(a, "a")?, index(b, "b")?);
    if pairs_of(&ia, &ib)?.is_empty() {
        return Err("a 与 b 没有共同的片段 id：不是同一批片段".into());
    }
    let nidx = match noise {
        Some((x, y)) => Some((index(x, "noise-a")?, index(y, "noise-b")?)),
        None => None,
    };
    let npairs = match &nidx {
        Some((x, y)) => Some(pairs_of(x, y)?),
        None => None,
    };
    let mut out = vec![one_scope("all", &ia, &ib, npairs.as_deref())?];
    if by_scene {
        let scenes: BTreeSet<String> = a.iter().chain(b).map(|r| r.scene.as_str()).collect();
        let of_scene = |m: &BTreeMap<&'a str, &'a ClipRow>, s: &str| -> BTreeMap<&'a str, &'a ClipRow> {
            m.iter()
                .filter(|(_, r)| r.scene.as_str() == s)
                .map(|(k, r)| (*k, *r))
                .collect()
        };
        for s in scenes {
            let (sa, sb) = (of_scene(&ia, &s), of_scene(&ib, &s));
            let sn: Option<Vec<_>> = npairs
                .as_ref()
                .map(|v| v.iter().filter(|p| p.1.scene.as_str() == s).copied().collect());
            out.push(one_scope(&s, &sa, &sb, sn.as_deref())?);
        }
    }
    Ok(out)
}

#[cfg(test)]
#[path = "compare_tests.rs"]
mod tests;
