//! 分片跑成绩单（大总管 2026-11-25 定）：`vdn-eval scorecard --shard i/n` 只跑一部分，各片写自己的 `--out`
//! （识别缓存 `qa/asr_cache.json` 也因此各片一份，不互相覆盖）；跑完 `--merge-shards` 按清单顺序拼回去。
//!
//! - 分法：清单顺序里的每一段（或一条展开不出片段的任务）是一个单位，第 k 个单位归第 `k mod n` 片（[`super::run_shard`]）
//! - 合并：逐段分数的行**原样**拼接（不解析再写出），按单位在清单里的位置排序；一段的各系统都在同一片里，
//!   片内顺序保留，所以和单进程写出的 `clips.jsonl` 逐字节相同。缺段记录同样排序
//! - 合并时拒绝：同一段同一系统出现两次（重复）；清单里有段既没分数也没缺段记录（少了一片）

use std::collections::{BTreeMap, BTreeSet};

use super::load::parse_jsonl;
use super::{Missing, clips_of, standard};
use crate::suite::Suite;

/// 第 `i` 片，共 `n` 片（`0 ≤ i < n`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shard {
    pub i: usize,
    pub n: usize,
}

impl Shard {
    /// 读 `i/n`。
    pub fn parse(s: &str) -> Result<Shard, String> {
        let bad = || format!("--shard 要写成 i/n（0 ≤ i < n），收到 {s}");
        let (a, b) = s.split_once('/').ok_or_else(bad)?;
        let (i, n): (usize, usize) = (a.parse().map_err(|_| bad())?, b.parse().map_err(|_| bad())?);
        if n == 0 || i >= n {
            return Err(bad());
        }
        Ok(Shard { i, n })
    }

    /// 第 `unit` 个单位归不归这一片。
    pub fn owns(&self, unit: usize) -> bool {
        unit % self.n == self.i
    }
}

/// 每片的识别并发：总并发 [`crate::qa::asr_or::ASR_CONCURRENCY`]（8）÷ 片数，向上取整，不低于 1（大总管 2026-11-25）。
pub fn asr_workers(n: usize) -> usize {
    crate::qa::asr_or::ASR_CONCURRENCY.div_ceil(n.max(1)).max(1)
}

/// 清单里每个单位的位置：段 id → 位置；展开不出片段的任务 id → 位置。另给「必须出现的」名字（分数或缺段记录里）。
fn units(suite: &Suite) -> (BTreeMap<String, usize>, BTreeSet<String>) {
    let (mut pos, mut need) = (BTreeMap::new(), BTreeSet::new());
    let mut k = 0usize;
    for task in &suite.tasks {
        match clips_of(task) {
            Ok(c) if !c.is_empty() => {
                for c in &c {
                    pos.insert(c.id.clone(), k);
                    if standard::group_of(c.scene).is_some() {
                        need.insert(c.id.clone());
                    }
                    k += 1;
                }
            }
            _ => {
                pos.insert(task.id.clone(), k);
                need.insert(task.id.clone());
                k += 1;
            }
        }
    }
    (pos, need)
}

/// 合并各片：`parts` = 每片的 (`clips.jsonl` 全文, 缺段记录)，顺序随意。返回 (合并后的 `clips.jsonl` 全文, 缺段记录)。
pub fn merge(
    suite: &Suite,
    parts: &[(String, Vec<Missing>)],
) -> Result<(String, Vec<Missing>), String> {
    let (pos, need) = units(suite);
    let at = |what: &str| {
        pos.get(what)
            .copied()
            .ok_or_else(|| format!("{what} 不在清单里：分片和合并用的不是同一份清单"))
    };
    let mut lines: Vec<(usize, usize, &str)> = Vec::new();
    let mut miss: Vec<(usize, usize, Missing)> = Vec::new();
    let (mut seen, mut miss_seen, mut covered) =
        (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for (pi, (text, m)) in parts.iter().enumerate() {
        let raw: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        let rows = parse_jsonl(text)?;
        for (j, (line, r)) in raw.iter().zip(&rows).enumerate() {
            if !seen.insert((r.clip.clone(), r.system.clone())) {
                return Err(format!(
                    "{} · {} 在两片里重复出现：不合并",
                    r.clip, r.system
                ));
            }
            covered.insert(r.clip.clone());
            lines.push((at(&r.clip)?, pi * 1_000_000 + j, line));
        }
        for (j, x) in m.iter().enumerate() {
            if !miss_seen.insert((x.what.clone(), x.system.clone())) {
                return Err(format!("缺段记录 {} · {} 重复：不合并", x.what, x.system));
            }
            covered.insert(x.what.clone());
            miss.push((at(&x.what)?, pi * 1_000_000 + j, x.clone()));
        }
    }
    let lack: Vec<&String> = need.difference(&covered).collect();
    if !lack.is_empty() {
        return Err(format!(
            "少了 {} 段（如 {:?}）：少了一片，或某片没跑完，不合并",
            lack.len(),
            &lack[..lack.len().min(3)]
        ));
    }
    // 位置相同的（同一段的几个系统）都在同一片里：按片内顺序排，即单进程的系统顺序
    lines.sort_by_key(|(p, o, _)| (*p, *o));
    miss.sort_by_key(|(p, o, _)| (*p, *o));
    let text: String = lines.iter().map(|(_, _, l)| format!("{l}\n")).collect();
    Ok((text, miss.into_iter().map(|(_, _, m)| m).collect()))
}
