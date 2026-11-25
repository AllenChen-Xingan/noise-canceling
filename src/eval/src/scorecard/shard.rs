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

/// 各片 meta 里必须相同的键（不同就不是同一次运行的几片）。
const SAME_KEYS: [&str; 7] = [
    "commit", "suite", "systems", "qa", "safety", "rules", "external",
];

/// 读几个分片输出目录（各有 `clips.jsonl` 与 `scorecard.json`），核对它们是同一次运行的 `0/n … n-1/n`，合并。
/// 返回 (合并后的 `clips.jsonl`, 缺段记录, 合并后的 meta)。meta：共同的键照抄；`shards` = n；
/// `timing` 各片相加、`shard_timing` 逐片列；`asr` 的 calls · checked · unstable 相加并重算不一致率（同一段音频只在一片里，不会重复计）。
pub fn merge_dirs(
    suite: &Suite,
    dirs: &[std::path::PathBuf],
) -> Result<(String, Vec<Missing>, serde_json::Value), String> {
    use serde_json::{Value, json};
    let mut parts = Vec::new();
    let mut metas: Vec<Value> = Vec::new();
    for d in dirs {
        let rd = |f: &str| {
            std::fs::read_to_string(d.join(f)).map_err(|e| format!("读不了分片输出 {f}：{e}"))
        };
        let card: Value =
            serde_json::from_str(&rd("scorecard.json")?).map_err(|e| e.to_string())?;
        let missing: Vec<Missing> =
            serde_json::from_value(card["missing"].clone()).map_err(|e| e.to_string())?;
        parts.push((rd("clips.jsonl")?, missing));
        metas.push(card["meta"].clone());
    }
    let shards: Vec<Shard> = metas
        .iter()
        .map(|m| {
            Shard::parse(m["shard"].as_str().unwrap_or(""))
                .map_err(|_| "有一个输出目录不是分片跑的（meta 里没有 shard）".to_string())
        })
        .collect::<Result<_, _>>()?;
    let n = shards.first().map_or(0, |s| s.n);
    let mut got: Vec<usize> = shards.iter().map(|s| s.i).collect();
    got.sort_unstable();
    if shards.iter().any(|s| s.n != n) || got != (0..n).collect::<Vec<_>>() {
        return Err(format!(
            "分片不齐：要 0/{n} … {}/{n} 各一份，收到 {got:?}",
            n.saturating_sub(1)
        ));
    }
    for k in SAME_KEYS {
        if metas.iter().any(|m| m[k] != metas[0][k]) {
            return Err(format!("各片 meta 的 {k} 不同：不是同一次运行，不合并"));
        }
    }
    let (text, missing) = merge(suite, &parts)?;
    let mut meta = serde_json::Map::new();
    for k in SAME_KEYS {
        meta.insert(k.into(), metas[0][k].clone());
    }
    let mut timing: BTreeMap<String, f64> = BTreeMap::new();
    for m in &metas {
        for (k, v) in m["timing"].as_object().into_iter().flatten() {
            *timing.entry(k.clone()).or_default() += v.as_f64().unwrap_or(0.0);
        }
    }
    let sum = |k: &str| {
        metas
            .iter()
            .map(|m| m["asr"][k].as_u64().unwrap_or(0))
            .sum::<u64>()
    };
    let asr = (!metas[0]["asr"].is_null()).then(|| {
        let mut a = metas[0]["asr"].clone();
        let (checked, unstable) = (sum("checked"), sum("unstable"));
        a["calls"] = json!(sum("calls"));
        a["checked"] = json!(checked);
        a["unstable"] = json!(unstable);
        a["unstable_rate"] = json!((checked > 0).then(|| unstable as f64 / checked as f64));
        a["per_shard"] = json!(metas.iter().map(|m| m["asr"].clone()).collect::<Vec<_>>());
        a
    });
    meta.insert("shards".into(), json!(n));
    meta.insert("timing".into(), json!(timing));
    meta.insert(
        "shard_timing".into(),
        json!(
            metas
                .iter()
                .map(|m| m["timing"].clone())
                .collect::<Vec<_>>()
        ),
    );
    meta.insert("asr".into(), json!(asr));
    Ok((text, missing, Value::Object(meta)))
}
