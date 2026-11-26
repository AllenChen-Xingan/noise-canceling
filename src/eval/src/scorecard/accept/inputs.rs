//! `vdn-eval accept` 的输入解析与绑定（从 `accept.rs` 拆出，R-SIZE）：compare.json · scorecard.json ·
//! 评测清单（`--suite`）→ 独立于成绩单的预期集合，以及「每份输入都属于这次比较」的三道绑定检查。
//!
//! 结构上的做法（codex 第 4 轮，同类问题第三次出现 → 改结构而不打补丁）：预期集合只从评测清单来，
//! 所有输入在读入处一次绑定到报告种子，判定函数里不再各自猜"这份数据对不对"。

use std::collections::BTreeMap;

use serde_json::Value;

use crate::files::serde_name;
use crate::suite::{Suite, parse_suite, resolve};

/// compare.json 里「全部段」的范围名；逐场景的范围名是场景键。
pub const ALL_SCOPE: &str = "all";

/// compare 的三态（名字同 `compare.json` 的 `call`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    Up,
    Down,
    Unsure,
}

impl Call {
    pub fn parse(s: &str) -> Call {
        match s {
            "up" => Call::Up,
            "down" => Call::Down,
            _ => Call::Unsure,
        }
    }
    pub fn zh(self) -> &'static str {
        match self {
            Call::Up => "涨",
            Call::Down => "退",
            Call::Unsure => "不确定",
        }
    }
}

/// 差值区间（`compare.json` 的 `diff`）。
#[derive(Clone, Copy, Debug)]
pub struct Ci {
    pub mean: f64,
    pub hi: f64,
    /// 参与抽样的源片段组数（不足 `SC-AGG.min_groups` 时这个区间判不了）
    pub groups: usize,
}

/// 一项在一个比较范围里的读数。
#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub call: Call,
    pub pairs: usize,
    pub diff: Option<Ci>,
}

/// 一个否决项在一个比较范围里的增减。
#[derive(Clone, Copy, Debug)]
pub struct Veto {
    pub a: usize,
    pub b: usize,
    pub new_in_b: usize,
    pub undecided: usize,
}

/// 一个比较范围（全部段，或 `--by-scene` 的一个场景）。
#[derive(Debug)]
pub struct Scope {
    /// 这个范围里配上的段数（否决项「两边都判得了」的分母是它减一边判不了）
    pub pairs: usize,
    /// 只在对照 / 只在候选的段数（核片段覆盖：配上的 + 只在一边的 = compare 见到的段）
    pub only_a: usize,
    pub only_b: usize,
    pub items: BTreeMap<String, Item>,
    pub vetoes: BTreeMap<String, Veto>,
}

/// 一份 `compare.json`。
/// 一份成绩单的身份：内容 sha256 + 系统名（同一份文件里的两个系统是两个不同候选）。
pub type Ident = (String, String);

#[derive(Debug)]
pub struct Cmp {
    pub path: String,
    pub scopes: BTreeMap<String, Scope>,
    /// compare 时两边成绩单的身份（sha256 + 系统）；用来核「三个不同种子」与「噪声底恰好是两两互比」
    pub a_id: Option<Ident>,
    pub b_id: Option<Ident>,
}

impl Cmp {
    /// 某个范围里某一项；只有真有配对值（`pairs > 0`）才算。
    pub fn item(&self, scope: &str, v: &str) -> Option<Item> {
        self.scopes
            .get(scope)?
            .items
            .get(v)
            .copied()
            .filter(|i| i.pairs > 0)
    }
    /// 逐场景的范围（不含「全部段」）。
    pub fn scenes(&self) -> impl Iterator<Item = (&str, &Scope)> {
        self.scopes
            .iter()
            .filter(|(k, _)| k.as_str() != ALL_SCOPE)
            .map(|(k, s)| (k.as_str(), s))
    }
    /// 全部配上的段数（「全部段」那个范围的数；没有该范围时按逐场景求和）。
    pub fn pairs(&self) -> usize {
        self.scopes
            .get(ALL_SCOPE)
            .map_or_else(|| self.scenes().map(|(_, s)| s.pairs).sum(), |s| s.pairs)
    }
    pub fn name(&self) -> String {
        short(&self.path)
    }
}

/// 成绩单里的一张卡（场景 × 档 × 设备 × 系统）。
#[derive(Debug)]
pub struct ClipKey {
    pub id: String,
    pub scene: String,
    pub tier: String,
    pub device: String,
}

/// 评测清单给的预期集合（独立于成绩单）：清单里每条任务的片段清单 → 应有哪几段、各属哪个
/// 场景 × 档 × 设备。它是「该评什么」的唯一来源，成绩单只提供「评了什么」。
pub struct Expected {
    pub keys: Vec<ClipKey>,
}

impl Expected {
    /// 读评测清单（`--suite`）并展开每个任务的片段清单。返回（预期集合, 清单指纹）。
    pub fn load(path: &str) -> Result<(Expected, String), String> {
        let p = resolve(path).ok_or_else(|| format!("找不到评测清单 {path}"))?;
        let text = std::fs::read_to_string(&p).map_err(|e| format!("{path}：{e}"))?;
        let suite: Suite = parse_suite(&text).map_err(|e| format!("{path}：{e}"))?;
        crate::scorecard::check_suite(&suite).map_err(|e| format!("{path}：{e}"))?;
        let fp = crate::scorecard::shard::suite_fingerprint(&suite);
        let mut keys = Vec::new();
        for t in &suite.tasks {
            let clips =
                crate::scorecard::clips_of(t).map_err(|e| format!("清单任务 {}：{e}", t.id))?;
            for c in clips {
                keys.push(ClipKey {
                    id: c.id,
                    scene: c.scene.as_str(),
                    tier: serde_name(&c.tier),
                    device: serde_name(&c.device),
                });
            }
        }
        if keys.is_empty() {
            return Err(format!("{path} 里没有片段：评测清单没展开出要评的段"));
        }
        Ok((Expected { keys }, fp))
    }

    /// 预期场景（去重，保持清单顺序）。
    pub fn scenes(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for k in &self.keys {
            if !out.contains(&k.scene) {
                out.push(k.scene.clone());
            }
        }
        out
    }

    /// 预期切片（场景 × 档 × 设备，去重）。
    pub fn slices(&self) -> Vec<(String, String, String)> {
        let mut out: Vec<(String, String, String)> = Vec::new();
        for k in &self.keys {
            let s = (k.scene.clone(), k.tier.clone(), k.device.clone());
            if !out.contains(&s) {
                out.push(s);
            }
        }
        out
    }

    fn clips_in(&self, scene: &str) -> usize {
        self.keys.iter().filter(|k| k.scene == scene).count()
    }

    /// 覆盖：清单要的每个场景 / 切片 / 片段，两份逐段文件与 compare 里都得在。返回问题清单（空 = 齐）。
    pub fn coverage(&self, a: &CardIn, b: &CardIn, rep: &Cmp) -> Vec<String> {
        let mut hole = Vec::new();
        let has = |c: &CardIn, scene: &str, tier: &str, device: &str| {
            c.keys
                .iter()
                .any(|k| k.scene == scene && k.tier == tier && k.device == device)
        };
        for (scene, tier, device) in self.slices() {
            for (side, c) in [("对照", a), ("候选", b)] {
                if !has(c, &scene, &tier, &device) {
                    hole.push(format!(
                        "{side}的逐段文件里没有 {scene} · {tier} · {device}"
                    ));
                }
            }
            for (side, c) in [("对照", a), ("候选", b)] {
                if !c
                    .rows
                    .iter()
                    .any(|r| r.scene == scene && r.tier == tier && r.device == device)
                {
                    hole.push(format!(
                        "{side}成绩单里没有 {scene} · {tier} · {device} 这张卡"
                    ));
                }
            }
        }
        for scene in self.scenes() {
            let want = self.clips_in(&scene);
            match rep.scopes.get(&scene) {
                None => hole.push(format!("compare 里没有场景 {scene}（清单有 {want} 段）")),
                Some(s) => {
                    let got = s.pairs + s.only_a + s.only_b;
                    if got < want {
                        hole.push(format!("{scene} 只见到 {got} 段，清单有 {want} 段"));
                    }
                }
            }
        }
        hole
    }
}

#[derive(Clone, Debug)]
pub struct CardRow {
    pub scene: String,
    pub tier: String,
    pub device: String,
    pub system: String,
    pub status: String,
    /// 这张卡上打分了几个片段（核「比较里的片段覆盖」用）
    pub clips: usize,
    /// 这张卡 V-LAT 的均值（逐卡比延迟用；没有读数时 `None`）
    pub lat_ms: Option<f64>,
}

/// 一份 `scorecard.json`。
#[derive(Debug)]
pub struct CardIn {
    pub path: String,
    /// 这份成绩单唯一的系统（[`parse_card`] 保证只有一个）
    pub system: String,
    /// 外部系统（延迟只是声明值）
    pub external: bool,
    /// 成绩单自己记的「从哪份逐段文件汇总」（`meta.from_clips_sha256`；现跑的成绩单没有，为 `None`）
    pub clips_sha: Option<String>,
    /// 成绩单记的评测清单指纹（`meta.suite_fingerprint`）
    pub fingerprint: Option<String>,
    /// 旁边 `clips.jsonl` 的 sha256（[`load_card`] 填）
    pub clips_actual_sha: String,
    /// 旁边 `clips.jsonl` 里的片段（[`load_card`] 填）
    pub keys: Vec<ClipKey>,
    pub rows: Vec<CardRow>,
    /// 各卡 V-LAT 均值的平均；没有读数时为 `None`
    pub lat_ms: Option<f64>,
}

pub fn num(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64).filter(|x| x.is_finite())
}

pub fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// 报告里写路径：相对路径照写；绝对路径只留文件名（本机绝对路径含用户名，报告可能进仓库）。
pub fn short(p: &str) -> String {
    let q = std::path::Path::new(p);
    if q.is_absolute() {
        q.file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
    } else {
        p.to_string()
    }
}

pub fn ci_of(v: &Value) -> Option<Ci> {
    v.as_object()?;
    Some(Ci {
        mean: num(v, "mean")?,
        hi: num(v, "hi")?,
        groups: num(v, "groups").unwrap_or(0.0).max(0.0) as usize,
    })
}

/// 读一份 `compare.json`：不是 `--by-scene` 生成的直接报错（没有逐场景的数，判不了）。
pub fn parse_compare(text: &str, path: &str) -> Result<Cmp, String> {
    let v: Value =
        serde_json::from_str(text).map_err(|e| format!("{path}：不是合法 JSON（{e}）"))?;
    if v["meta"]["by_scene"].as_bool() != Some(true) {
        return Err(format!(
            "{path} 不是 --by-scene 生成的 compare.json（meta.by_scene 不是 true）：没有逐场景的数，判不了。\
             重跑 `vdn-eval compare --by-scene`"
        ));
    }
    let mut scopes = BTreeMap::new();
    for s in v["scopes"].as_array().into_iter().flatten() {
        let name = s["scope"].as_str().unwrap_or_default().to_string();
        if name.is_empty() {
            return Err(format!("{path} 里有一个比较范围没有 scope 名"));
        }
        let mut items = BTreeMap::new();
        for i in s["items"].as_array().into_iter().flatten() {
            let id = i["v"].as_str().unwrap_or_default().to_string();
            items.insert(
                id,
                Item {
                    call: Call::parse(i["call"].as_str().unwrap_or_default()),
                    pairs: num(i, "pairs").unwrap_or(0.0).max(0.0) as usize,
                    diff: i.get("diff").and_then(ci_of),
                },
            );
        }
        let mut vetoes = BTreeMap::new();
        for x in s["vetoes"].as_array().into_iter().flatten() {
            let count = |k: &str| x[k].as_array().map_or(0, Vec::len);
            vetoes.insert(
                x["v"].as_str().unwrap_or_default().to_string(),
                Veto {
                    a: num(x, "a").unwrap_or(0.0).max(0.0) as usize,
                    b: num(x, "b").unwrap_or(0.0).max(0.0) as usize,
                    new_in_b: count("new_in_b"),
                    undecided: count("undecided"),
                },
            );
        }
        scopes.insert(
            name,
            Scope {
                pairs: num(s, "pairs").unwrap_or(0.0).max(0.0) as usize,
                only_a: s["only_a"].as_array().map_or(0, Vec::len),
                only_b: s["only_b"].as_array().map_or(0, Vec::len),
                items,
                vetoes,
            },
        );
    }
    if scopes.is_empty() {
        return Err(format!("{path} 里没有 scopes：不是 compare 的输出"));
    }
    let ident = |sha: &str, system: &str| -> Option<Ident> {
        let s = v["meta"][sha].as_str().filter(|x| !x.is_empty())?;
        let y = v["meta"][system].as_str().filter(|x| !x.is_empty())?;
        Some((s.to_string(), y.to_string()))
    };
    Ok(Cmp {
        path: path.to_string(),
        scopes,
        a_id: ident("a_sha256", "a_system"),
        b_id: ident("b_sha256", "b_system"),
    })
}

/// 读一份 `scorecard.json`。
/// 读一份成绩单：`scorecard.json` + 旁边的 `clips.jsonl`（它的 sha256 是这张卡的「出身」）。
/// 缺 `clips.jsonl`、缺清单指纹 → 报错（没有这两样就核不了它是不是这次比较的那一边）。
pub fn load_card(path: &str) -> Result<CardIn, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}：{e}"))?;
    let mut card = parse_card(&text, path)?;
    let clips = std::path::Path::new(path).with_file_name("clips.jsonl");
    if !clips.exists() {
        return Err(format!(
            "{} 旁边没有 clips.jsonl：accept 要用它核这份成绩单是哪一份逐段文件跑出来的",
            card.path
        ));
    }
    card.clips_actual_sha = crate::wav::sha256(&clips)?;
    let text = std::fs::read_to_string(&clips).map_err(|e| format!("{}：{e}", clips.display()))?;
    let rows = crate::scorecard::load::parse_jsonl(&text)
        .map_err(|e| format!("{}：{e}", clips.display()))?;
    card.keys = rows
        .iter()
        .map(|r| ClipKey {
            id: r.clip.clone(),
            scene: r.scene.as_str(),
            tier: serde_name(&r.tier),
            device: r.device.clone(),
        })
        .collect();
    if card.fingerprint.is_none() {
        return Err(format!(
            "{} 没记评测清单指纹（meta.suite_fingerprint）：用 `vdn-eval scorecard --suite …` 重跑一份，\
             或用 `--merge-shards` 合并过的成绩单",
            card.path
        ));
    }
    Ok(card)
}

/// 两张卡必须就是报告种子那一边的成绩单：旁边的 `clips.jsonl` sha256、系统名、清单指纹都要对上
/// （codex 第 3 · 4 轮 P1：误传旧候选 / 原版卡会掩盖新增「不过」，外部候选的声明延迟也会被当成实测）。
pub fn check_binding(rep: &Cmp, a: &CardIn, b: &CardIn, fp: &str) -> Result<(), String> {
    let (Some(aid), Some(bid)) = (rep.a_id.as_ref(), rep.b_id.as_ref()) else {
        return Err(format!(
            "{} 没记两边成绩单的 sha256 与系统名（meta.a_sha256 · a_system · b_sha256 · b_system）：\
             核不了两张卡是哪一边的，不判",
            rep.name()
        ));
    };
    for (flag, card, id, side) in [
        ("--card-a", a, aid, "对照"),
        ("--card-b", b, bid, "候选（报告种子）"),
    ] {
        if card.clips_actual_sha != id.0 {
            return Err(format!(
                "{flag} 旁边的 clips.jsonl 是 {}…，第一份 --vs 的{side}是 {}…：不是同一份逐段文件，不判",
                &card.clips_actual_sha[..card.clips_actual_sha.len().min(12)],
                &id.0[..id.0.len().min(12)]
            ));
        }
        if let Some(sha) = &card.clips_sha
            && sha != &id.0
        {
            return Err(format!(
                "{flag} 记的是从 {}… 的逐段文件汇总的，第一份 --vs 的{side}是 {}…：不是同一份，不判",
                &sha[..sha.len().min(12)],
                &id.0[..id.0.len().min(12)]
            ));
        }
        if card.system != id.1 {
            return Err(format!(
                "{flag} 的系统是 {}，第一份 --vs 的{side}系统是 {}：不是同一边的成绩单，不判",
                card.system, id.1
            ));
        }
        match card.fingerprint.as_deref() {
            Some(x) if x == fp => {}
            Some(x) => {
                return Err(format!(
                    "{flag} 的评测清单指纹是 {x}，--suite 的指纹是 {fp}：不是同一份清单跑的成绩单，不判"
                ));
            }
            None => {
                return Err(format!(
                    "{flag} 没记评测清单指纹（meta.suite_fingerprint）：核不了它是不是 --suite 这份清单跑的"
                ));
            }
        }
    }
    Ok(())
}

pub fn parse_card(text: &str, path: &str) -> Result<CardIn, String> {
    let v: Value =
        serde_json::from_str(text).map_err(|e| format!("{path}：不是合法 JSON（{e}）"))?;
    let mut rows = Vec::new();
    let mut lats = Vec::new();
    for c in v["scenes"].as_array().into_iter().flatten() {
        let status = c["status"].as_str().unwrap_or_default().to_string();
        if status.is_empty() {
            return Err(format!("{path} 里有一张卡没有 status：不是成绩单"));
        }
        if let Some(x) = c["items"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|x| x["v"] == "V-LAT")
            && let Some(m) = num(x, "mean")
        {
            lats.push(m);
        }
        rows.push(CardRow {
            scene: c["scene"].as_str().unwrap_or_default().to_string(),
            tier: c["tier"].as_str().unwrap_or_default().to_string(),
            device: c["device"].as_str().unwrap_or_default().to_string(),
            system: c["system"].as_str().unwrap_or_default().to_string(),
            status,
            clips: num(c, "clips").unwrap_or(0.0).max(0.0) as usize,
            lat_ms: c["items"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|x| x["v"] == "V-LAT")
                .and_then(|x| num(x, "mean")),
        });
    }
    if rows.is_empty() {
        return Err(format!("{path} 里没有场景卡：不是成绩单"));
    }
    // 一份成绩单只能一个系统：多个系统时同一 场景 × 档 × 设备 的卡会互相覆盖（退步被遮住），
    // 延迟也会把几个系统混在一起求均值（codex 第 1 轮 P1 #3）
    let systems: std::collections::BTreeSet<&str> =
        rows.iter().map(|r| r.system.as_str()).collect();
    if systems.contains("") {
        return Err(format!("{path} 里有一张卡没有 system：不是成绩单"));
    }
    if systems.len() > 1 {
        return Err(format!(
            "{path} 里有 {} 个系统（{}）：accept 一次只比一个系统。用 `vdn-eval scorecard --system <一个系统>` \
             分别出成绩单，或按系统把这几个系统拆成几份成绩单",
            systems.len(),
            systems.iter().copied().collect::<Vec<_>>().join(" · ")
        ));
    }
    let ext = v["meta"]["external"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    Ok(CardIn {
        path: path.to_string(),
        system: systems.iter().next().copied().unwrap_or("").to_string(),
        external: !ext.is_empty(),
        clips_sha: v["meta"]["from_clips_sha256"]
            .as_str()
            .filter(|x| !x.is_empty())
            .map(str::to_string),
        fingerprint: v["meta"]["suite_fingerprint"]
            .as_str()
            .filter(|x| !x.is_empty())
            .map(str::to_string),
        clips_actual_sha: String::new(),
        keys: Vec::new(),
        rows,
        lat_ms: mean(&lats),
    })
}
