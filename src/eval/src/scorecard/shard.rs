//! 分片跑成绩单（大总管 2026-11-25 定）：`vdn-eval scorecard --shard i/n` 只跑一部分，各片写自己的 `--out`
//! （识别缓存 `qa/asr_cache.json` 也因此各片一份，不互相覆盖）；跑完 `--merge-shards` 按清单顺序拼回去。
//!
//! - 分法：清单顺序里的每一段（或一条展开不出片段的任务）是一个单位，第 k 个单位归第 `k mod n` 片（[`super::run_shard`]）
//! - 每片在 meta 里记下每行分数、每条缺段记录来自第几个单位（`shard_units`）：合并按「单位位置 × 系统顺序」排，
//!   不靠片段 id 定位（id 只在一个 build 内唯一；codex 本分支第 1 轮 P1 #3）
//! - 合并：逐段分数的行**原样**拼接（不解析再写出），所以和单进程写出的 `clips.jsonl` 逐字节相同。缺段记录同样排序
//! - 合并时拒绝：片不齐（要 0…n−1 各一份）；单位不归这一片；同一单位同一系统有两条记录（分数或缺段）；
//!   清单里该有的「单位 × 系统」缺了（codex 本分支第 1 轮 P1 #1）；清单内容和各片跑时不同（指纹，P1 #2）

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

/// 一片的产物：第几片、逐段分数全文、缺段记录，以及每行分数 / 每条缺段记录来自第几个单位。
#[derive(Clone, Debug)]
pub struct Part {
    pub shard: Shard,
    pub jsonl: String,
    pub missing: Vec<Missing>,
    pub row_units: Vec<usize>,
    pub missing_units: Vec<usize>,
}

/// 清单里的一个单位（一段，或一条展开不出片段的任务）：要不要有记录、归哪一片看的下标、应该是谁。
#[derive(Clone, Debug)]
pub struct Unit {
    /// 有片段且分组认得 / 展开不出片段的任务 = 要有记录；分组不认得 = 不要
    pub need: bool,
    /// 分片看的下标：同一段（id + 混合轨哈希）在清单里出现几次，都按它第一次出现的位置分，落在同一片里
    /// （单进程会复用它的参考转写，分到两片就要重新识别、结果可能不同；codex 本分支第 2 轮 P1 #1）
    pub home: usize,
    /// 记录里应有的身份：(片段或任务 id, 场景, 档, 设备, 混合轨哈希；任务单位没有哈希)
    pub expect: (
        String,
        crate::scene::MeetingPlace,
        crate::scene::Tier,
        String,
        Option<String>,
    ),
}

/// 单位清单（正本；[`super::run_shard`] 与 [`merge`] 共用，顺序就是单进程的顺序）。
pub fn unit_plan(suite: &Suite) -> Vec<Unit> {
    let mut v: Vec<Unit> = Vec::new();
    let mut first: BTreeMap<(String, String), usize> = BTreeMap::new();
    for task in &suite.tasks {
        match clips_of(task) {
            Ok(c) if !c.is_empty() => {
                for c in &c {
                    let k = v.len();
                    let mix = c.hash.wav_sha256.mix.clone();
                    let home = *first.entry((c.id.clone(), mix.clone())).or_insert(k);
                    let dev = crate::files::serde_name(&c.device);
                    v.push(Unit {
                        need: standard::group_of(c.scene).is_some(),
                        home,
                        expect: (c.id.clone(), c.scene, c.tier, dev, Some(mix)),
                    });
                }
            }
            _ => {
                let Some(crate::suite::RealSource::Scene {
                    scene,
                    tier,
                    device,
                    ..
                }) = &task.real
                else {
                    continue;
                };
                let k = v.len();
                let dev = crate::files::serde_name(device);
                v.push(Unit {
                    need: true,
                    home: k,
                    expect: (task.id.clone(), *scene, *tier, dev, None),
                });
            }
        }
    }
    v
}

/// 清单指纹：每条任务的定义 + 它引用的片段清单的字节（FNV-64，只用来查「是不是同一份」；codex 本分支第 1 轮 P1 #2）。
pub fn suite_fingerprint(suite: &Suite) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |b: &[u8]| {
        for x in b {
            h ^= u64::from(*x);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for t in &suite.tasks {
        eat(format!("{t:?}").as_bytes());
        if let Some(crate::suite::RealSource::Scene { clips, .. }) = &t.real {
            let bytes = crate::suite::resolve(clips)
                .and_then(|p| std::fs::read(p).ok())
                .unwrap_or_default();
            eat(&bytes);
        }
    }
    format!("{h:016x}")
}

/// 合并各片（顺序随意）。`systems` = 单进程时的系统顺序（`meta.systems`）。返回 (合并后的 `clips.jsonl` 全文, 缺段记录)。
pub fn merge(
    suite: &Suite,
    systems: &[String],
    parts: &[Part],
) -> Result<(String, Vec<Missing>), String> {
    let n = parts.first().map_or(0, |p| p.shard.n);
    let mut got: Vec<usize> = parts.iter().map(|p| p.shard.i).collect();
    got.sort_unstable();
    if parts.iter().any(|p| p.shard.n != n) || got != (0..n).collect::<Vec<_>>() {
        return Err(format!(
            "分片不齐：要 0/{n} … {}/{n} 各一份，收到 {got:?}",
            n.saturating_sub(1)
        ));
    }
    let plan = unit_plan(suite);
    let sys_pos = |s: &str| {
        systems
            .iter()
            .position(|x| x == s)
            .ok_or_else(|| format!("系统 {s} 不在 meta.systems 里"))
    };
    let mut seen: BTreeSet<(usize, usize)> = BTreeSet::new();
    // 每条记录：单位要在清单里、该有记录、归这一片；记录的身份要和清单里这个单位一致（codex 本分支第 2 轮 P1 #2）；同一单位同一系统只许一条
    type Who<'a> = (
        &'a str,
        crate::scene::MeetingPlace,
        crate::scene::Tier,
        &'a str,
        Option<&'a str>,
    );
    let mut claim = |u: usize, sys: &str, who: Who, p: &Part| -> Result<(usize, usize), String> {
        let Some(unit) = plan.get(u).filter(|x| x.need) else {
            return Err(format!(
                "第 {u} 个单位不在清单里（或不该有记录）：分片和合并用的不是同一份清单"
            ));
        };
        if unit.home % n != p.shard.i {
            return Err(format!("第 {u} 个单位不归第 {}/{n} 片", p.shard.i));
        }
        let e = &unit.expect;
        let same = who.0 == e.0
            && who.1 == e.1
            && who.2 == e.2
            && who.3 == e.3
            && (who.4.is_none() || who.4 == e.4.as_deref());
        if !same {
            return Err(format!(
                "第 {}/{n} 片第 {u} 个单位的记录是 {}，清单里这里是 {}：文件配错了，不合并",
                p.shard.i, who.0, e.0
            ));
        }
        let k = (u, sys_pos(sys)?);
        if !seen.insert(k) {
            return Err(format!("第 {u} 个单位的 {sys} 有两条记录（重复）：不合并"));
        }
        Ok(k)
    };
    let mut lines: Vec<((usize, usize), &str)> = Vec::new();
    let mut miss: Vec<((usize, usize), Missing)> = Vec::new();
    for p in parts {
        let raw: Vec<&str> = p.jsonl.lines().filter(|l| !l.trim().is_empty()).collect();
        let rows = parse_jsonl(&p.jsonl)?;
        if rows.len() != p.row_units.len() || p.missing.len() != p.missing_units.len() {
            return Err(format!("第 {}/{n} 片的单位记录和行数对不上", p.shard.i));
        }
        for ((line, r), &u) in raw.iter().zip(&rows).zip(&p.row_units) {
            let who = (
                r.clip.as_str(),
                r.scene,
                r.tier,
                r.device.as_str(),
                r.mix_sha256.as_deref(),
            );
            if who.4.is_none() {
                return Err(format!(
                    "{} 的分数行没有 mix_sha256：旧格式，不能核身份",
                    r.clip
                ));
            }
            lines.push((claim(u, &r.system, who, p)?, line));
        }
        for (m, &u) in p.missing.iter().zip(&p.missing_units) {
            let who = (m.what.as_str(), m.scene, m.tier, m.device.as_str(), None);
            miss.push((claim(u, &m.system, who, p)?, m.clone()));
        }
    }
    let want = plan.iter().filter(|x| x.need).count() * systems.len();
    if seen.len() != want {
        return Err(format!(
            "少了 {} 条「单位 × 系统」的记录：少了一片，或某片没跑完，不合并",
            want - seen.len()
        ));
    }
    lines.sort_by_key(|(k, _)| *k);
    miss.sort_by_key(|(k, _)| *k);
    let text: String = lines.iter().map(|(_, l)| format!("{l}\n")).collect();
    Ok((text, miss.into_iter().map(|(_, m)| m).collect()))
}

/// 各片 meta 里必须相同的键（不同就不是同一次运行的几片）。
const SAME_KEYS: [&str; 8] = [
    "commit",
    "suite",
    "suite_fingerprint",
    "systems",
    "qa",
    "safety",
    "rules",
    "external",
];

/// 读几个分片输出目录（各有 `clips.jsonl` 与 `scorecard.json`），核对它们是同一次运行的 `0/n … n-1/n`、
/// 清单指纹和 `suite` 一致，合并。返回 (合并后的 `clips.jsonl`, 缺段记录, 合并后的 meta)。meta：共同的键照抄；
/// `shards` = n；`timing` 各片相加、`shard_timing` 逐片列；`asr` 的 calls · checked · unstable 相加并重算不一致率。
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
        let m = card["meta"].clone();
        let shard = Shard::parse(m["shard"].as_str().unwrap_or(""))
            .map_err(|_| "有一个输出目录不是分片跑的（meta 里没有 shard）".to_string())?;
        let us = |k: &str| -> Result<Vec<usize>, String> {
            serde_json::from_value(m["shard_units"][k].clone())
                .map_err(|_| "meta 里没有 shard_units".to_string())
        };
        parts.push(Part {
            shard,
            jsonl: rd("clips.jsonl")?,
            missing: serde_json::from_value(card["missing"].clone()).map_err(|e| e.to_string())?,
            row_units: us("rows")?,
            missing_units: us("missing")?,
        });
        metas.push(m);
    }
    if metas.is_empty() {
        return Err("没有给分片目录".into());
    }
    for k in SAME_KEYS {
        if metas.iter().any(|m| m[k] != metas[0][k]) {
            return Err(format!("各片 meta 的 {k} 不同：不是同一次运行，不合并"));
        }
    }
    if metas[0]["suite_fingerprint"] != json!(suite_fingerprint(suite)) {
        return Err("合并用的清单和各片跑时的内容不同（指纹对不上）：不合并".into());
    }
    let systems: Vec<String> =
        serde_json::from_value(metas[0]["systems"].clone()).map_err(|e| e.to_string())?;
    let (text, missing) = merge(suite, &systems, &parts)?;
    let n = parts[0].shard.n;
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
