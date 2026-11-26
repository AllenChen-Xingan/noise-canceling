//! `vdn-eval accept`：把计划 014 第三节的验收线落到配对比较与两份成绩单上，逐条判「过 / 不过 / 判不了」。
//!
//! ```text
//! vdn-eval accept --vs <compare.json> [--vs …]        # 候选对对照，一个种子一份；第一个 = 报告种子
//!                 --noise <compare.json> [--noise …]  # 同配置两个种子互比（种子噪声底）
//!                 --scenes s1,s2,…                     # 目标场景（D113），必给
//!                 --card-a <对照 scorecard.json> --card-b <候选（报告种子）scorecard.json>
//!                 [--out DIR]                          # 写 accept.md + accept.json（只新建，不覆盖）
//! ```
//!
//! 输入全是现成命令的产物：`--vs` / `--noise` 来自 `vdn-eval compare --by-scene`（没有逐场景的数就报错），
//! `--card-*` 来自 `vdn-eval scorecard`。本命令只读它们，不跑音频、不调模型。
//!
//! 每条线一行，结论只有三态；整份结论：每行都过 = 过，有一行不过 = 不过，其余 = 判不了。
//!
//! 判定前先核三件事，缺了只判「判不了」（身份不符直接报错），绝不当「过」：
//! - **种子齐**：`--vs` 必须是 `SC-ACCEPT.seeds`（3）个不同种子，候选身份 = 成绩单 sha256 + 系统名，
//!   三份的对照必须是同一份；`--noise` 必须恰好是这三个候选的全部两两互比（`judge::seed_shortage`）
//! - **有证据**：「不适用」只按成绩单标准的适用范围算（`standard::items`）；适用场景没有数、区间组数不够、
//!   否决项一段都没判过，都是证据不足 → 判不了（未知不能证明「不新增否决段」）
//! - **卡片对得上**：`--card-a` / `--card-b` 必须是第一份 `--vs` 那两边的成绩单（系统名相同；成绩单自己记了
//!   从哪份逐段文件汇总时，sha256 也要相同），不符直接报错不判（`check_binding`）；覆盖按对照成绩单的
//!   场景与片段核，整场景漏跑或片段少配 → 判不了（`judge::coverage_row`）
//!
//! 阈值全部从 [`crate::thresholds::ACCEPT`] 取（正本 `SC-ACCEPT`），本文件不写数字；
//! 延迟的线是基线成绩单量出来的 V-LAT 本身（014 第三节第 4 条），不是另一个阈值。

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::scene::MeetingPlace;

mod judge;

use judge::evaluate;

/// 「人耳未测」：本结论没做人耳盲听（D99 第 3 条）。
pub const NOT_LISTENED: &str = "人耳未测：本结论没做人耳盲听（D99 第 3 条）。";
/// 「只在合成集上判」：评测集是合成场景集，真实会议集未定（014 第三节第 6 条）。
pub const SYNTH_ONLY: &str =
    "只在合成集上判：评测集是合成场景集，真实会议集未定（014 第三节第 6 条）。";

/// 「延迟等于基线」的容差（ms）：同一个延迟账算出来的数逐位相同，容差只吸收浮点误差。
pub const LAT_SAME_MS: f64 = 1e-3;

/// 要涨的项与它用的线：`(评分项, SC-ACCEPT 的键)`。范围 = 目标场景（人定第五节第 1 题：目标场景）。
pub const GAINS: [(&str, &str); 3] = [
    ("V-ESTOI", "estoi_gain"),
    ("V-NRMOS-SCOREQ", "nrmos_gain"),
    ("V-NRMOS-DISTILL", "nrmos_gain"),
];

/// 不许退的项与它用的线：报告种子的差值区间上界 ≤ 线。
pub const REGRESS: [(&str, &str); 3] = [
    ("V-ONSET", "onset_regress"),
    ("V-BGSTAB", "bgstab_regress"),
    ("V-TIMBRE", "timbre_regress"),
];

/// 否决类：报告种子的「b 新增否决段」必须是 0（「一边判不了」只列数，不挡）。
pub const VETO: [&str; 2] = ["V-EAT-ASR", "V-SPLVL"];

/// 写出的两个文件（只新建）。
pub const FILES: [&str; 2] = ["accept.md", "accept.json"];

/// compare.json 里「全部段」的范围名；逐场景的范围名是场景键。
const ALL_SCOPE: &str = "all";

/// 三态结论。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Undecided,
}

impl Verdict {
    pub fn zh(self) -> &'static str {
        match self {
            Verdict::Pass => "过",
            Verdict::Fail => "不过",
            Verdict::Undecided => "判不了",
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::Undecided => "undecided",
        }
    }
}

/// 一行：判什么、结论、用到的数。
#[derive(Clone, Debug)]
struct Row {
    what: String,
    verdict: Verdict,
    detail: String,
}

impl Row {
    fn new(what: String, verdict: Verdict, detail: String) -> Row {
        Row {
            what,
            verdict,
            detail,
        }
    }
    fn json(&self) -> Value {
        json!({"what": self.what, "verdict": self.verdict.as_str(),
               "verdict_zh": self.verdict.zh(), "detail": self.detail})
    }
}

/// 整份判定。
struct Report {
    rows: Vec<Row>,
    verdict: Verdict,
}

/// compare 的三态（名字同 `compare.json` 的 `call`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Call {
    Up,
    Down,
    Unsure,
}

impl Call {
    fn parse(s: &str) -> Call {
        match s {
            "up" => Call::Up,
            "down" => Call::Down,
            _ => Call::Unsure,
        }
    }
    fn zh(self) -> &'static str {
        match self {
            Call::Up => "涨",
            Call::Down => "退",
            Call::Unsure => "不确定",
        }
    }
}

/// 差值区间（`compare.json` 的 `diff`）。
#[derive(Clone, Copy, Debug)]
struct Ci {
    mean: f64,
    hi: f64,
    /// 参与抽样的源片段组数（不足 `SC-AGG.min_groups` 时这个区间判不了）
    groups: usize,
}

/// 一项在一个比较范围里的读数。
#[derive(Clone, Copy, Debug)]
struct Item {
    call: Call,
    pairs: usize,
    diff: Option<Ci>,
}

/// 一个否决项在一个比较范围里的增减。
#[derive(Clone, Copy, Debug)]
struct Veto {
    a: usize,
    b: usize,
    new_in_b: usize,
    undecided: usize,
}

/// 一个比较范围（全部段，或 `--by-scene` 的一个场景）。
#[derive(Debug)]
struct Scope {
    /// 这个范围里配上的段数（否决项「两边都判得了」的分母是它减一边判不了）
    pairs: usize,
    items: BTreeMap<String, Item>,
    vetoes: BTreeMap<String, Veto>,
}

/// 一份 `compare.json`。
/// 一份成绩单的身份：内容 sha256 + 系统名（同一份文件里的两个系统是两个不同候选）。
type Ident = (String, String);

#[derive(Debug)]
struct Cmp {
    path: String,
    scopes: BTreeMap<String, Scope>,
    /// compare 时两边成绩单的身份（sha256 + 系统）；用来核「三个不同种子」与「噪声底恰好是两两互比」
    a_id: Option<Ident>,
    b_id: Option<Ident>,
}

impl Cmp {
    /// 某个范围里某一项；只有真有配对值（`pairs > 0`）才算。
    fn item(&self, scope: &str, v: &str) -> Option<Item> {
        self.scopes
            .get(scope)?
            .items
            .get(v)
            .copied()
            .filter(|i| i.pairs > 0)
    }
    /// 逐场景的范围（不含「全部段」）。
    fn scenes(&self) -> impl Iterator<Item = (&str, &Scope)> {
        self.scopes
            .iter()
            .filter(|(k, _)| k.as_str() != ALL_SCOPE)
            .map(|(k, s)| (k.as_str(), s))
    }
    /// 全部配上的段数（「全部段」那个范围的数；没有该范围时按逐场景求和）。
    fn pairs(&self) -> usize {
        self.scopes
            .get(ALL_SCOPE)
            .map_or_else(|| self.scenes().map(|(_, s)| s.pairs).sum(), |s| s.pairs)
    }
    fn name(&self) -> String {
        short(&self.path)
    }
}

/// 成绩单里的一张卡（场景 × 档 × 设备 × 系统）。
#[derive(Debug)]
struct CardRow {
    scene: String,
    tier: String,
    device: String,
    system: String,
    status: String,
    /// 这张卡上打分了几个片段（核「比较里的片段覆盖」用）
    clips: usize,
}

/// 一份 `scorecard.json`。
#[derive(Debug)]
struct CardIn {
    path: String,
    /// 这份成绩单唯一的系统（[`parse_card`] 保证只有一个）
    system: String,
    /// 外部系统（延迟只是声明值）
    external: bool,
    /// 成绩单自己记的「从哪份逐段文件汇总」（`meta.from_clips_sha256`；现跑的成绩单没有，为 `None`）
    clips_sha: Option<String>,
    rows: Vec<CardRow>,
    /// 各卡 V-LAT 均值的平均；没有读数时为 `None`
    lat_ms: Option<f64>,
}

fn num(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64).filter(|x| x.is_finite())
}

fn mean(v: &[f64]) -> Option<f64> {
    (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
}

/// 报告里写路径：相对路径照写；绝对路径只留文件名（本机绝对路径含用户名，报告可能进仓库）。
fn short(p: &str) -> String {
    let q = std::path::Path::new(p);
    if q.is_absolute() {
        q.file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
    } else {
        p.to_string()
    }
}

fn ci_of(v: &Value) -> Option<Ci> {
    v.as_object()?;
    Some(Ci {
        mean: num(v, "mean")?,
        hi: num(v, "hi")?,
        groups: num(v, "groups").unwrap_or(0.0).max(0.0) as usize,
    })
}

/// 读一份 `compare.json`：不是 `--by-scene` 生成的直接报错（没有逐场景的数，判不了）。
fn parse_compare(text: &str, path: &str) -> Result<Cmp, String> {
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
fn parse_card(text: &str, path: &str) -> Result<CardIn, String> {
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
        rows,
        lat_ms: mean(&lats),
    })
}

/// 两张卡必须就是报告种子那一边的成绩单：系统名相同；成绩单自己记了逐段文件 sha256 时也要相同。
///
/// 不核这一条，误传旧候选或原版的卡就能掩盖新增「不过」，外部候选的声明延迟也会被当成实测
/// （codex 第 3 轮 P1 #1）。
fn check_binding(rep: &Cmp, a: &CardIn, b: &CardIn) -> Result<(), String> {
    let (Some(aid), Some(bid)) = (rep.a_id.as_ref(), rep.b_id.as_ref()) else {
        return Err(format!(
            "{} 没记两边成绩单的 sha256 与系统名（meta.a_sha256 · a_system · b_sha256 · b_system）：             核不了两张卡是哪一边的，不判",
            rep.name()
        ));
    };
    for (flag, card, id, side) in [
        ("--card-a", a, aid, "对照"),
        ("--card-b", b, bid, "候选（报告种子）"),
    ] {
        if card.system != id.1 {
            return Err(format!(
                "{flag} 的系统是 {}，第一份 --vs 的{side}系统是 {}：不是同一边的成绩单，不判",
                card.system, id.1
            ));
        }
        if let Some(sha) = &card.clips_sha
            && sha != &id.0
        {
            return Err(format!(
                "{flag} 是从 {} 的逐段文件汇总的，第一份 --vs 的{side}是 {}：不是同一份逐段文件，不判",
                &sha[..sha.len().min(12)],
                &id.0[..id.0.len().min(12)]
            ));
        }
    }
    Ok(())
}

fn to_markdown(r: &Report, vs: &[String], noise: &[String], scenes: &[String]) -> String {
    let names = |v: &[String]| {
        if v.is_empty() {
            "—".to_string()
        } else {
            v.iter()
                .map(|p| format!("`{}`", short(p)))
                .collect::<Vec<_>>()
                .join(" · ")
        }
    };
    let mut s = format!(
        "# 验收判定（计划 014 第三节）\n\n{NOT_LISTENED}\n\n{SYNTH_ONLY}\n\n**结论：{}**\n\n",
        r.verdict.zh()
    );
    s += &format!(
        "目标场景：{}\n\n报告种子（第一份 `--vs`）：`{}`\n\n其他种子：{}\n\n种子噪声底：{}\n\n",
        scenes.join(" · "),
        short(&vs[0]),
        names(&vs[1..]),
        names(noise)
    );
    s += "| 判什么 | 结论 | 用到的数 / 依据 |\n|---|---|---|\n";
    for row in &r.rows {
        s += &format!("| {} | {} | {} |\n", row.what, row.verdict.zh(), row.detail);
    }
    s += "\n阈值正本：`src/eval/src/thresholds/accept.rs`（`SC-ACCEPT`）；延迟的线是基线成绩单量出来的 V-LAT，不是另一个数。\n";
    s
}

fn to_json(r: &Report, vs: &[String], noise: &[String], scenes: &[String], run_id: &str) -> Value {
    json!({
        "doc": "验收判定（vdn-eval accept）：按计划 014 第三节逐条判过 / 不过 / 判不了；阈值正本 SC-ACCEPT",
        "run_id": run_id,
        "verdict": r.verdict.as_str(),
        "verdict_zh": r.verdict.zh(),
        "notes": [NOT_LISTENED, SYNTH_ONLY],
        "inputs": {"vs": vs, "noise": noise, "scenes": scenes},
        "thresholds": {"line": "SC-ACCEPT", "src": "src/eval/src/thresholds/accept.rs"},
        "rows": r.rows.iter().map(Row::json).collect::<Vec<_>>(),
    })
}

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter()
        .position(|a| a == k)
        .and_then(|i| args.get(i + 1).cloned())
}

/// 可以给多次的选项（`--vs a --vs b`）。
fn many(args: &[String], k: &str) -> Vec<String> {
    args.iter()
        .enumerate()
        .filter(|(_, a)| *a == k)
        .filter_map(|(i, _)| args.get(i + 1).cloned())
        .collect()
}

/// 命令行入口（`args` 不含子命令名）。返回报告目录。
pub fn cli(args: &[String], run_id: &str, default_out: PathBuf) -> Result<PathBuf, String> {
    let vs = many(args, "--vs");
    if vs.is_empty() {
        return Err(
            "缺 --vs（至少一份 compare.json；用法见 src/eval/src/scorecard/accept.rs 头注释）"
                .into(),
        );
    }
    let noise = many(args, "--noise");
    let scenes: Vec<String> = arg(args, "--scenes")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    if scenes.is_empty() {
        return Err(
            "缺 --scenes（目标场景，逗号分隔，如 rail-air-cabin,canteen,meeting-room,transit-hub）"
                .into(),
        );
    }
    for s in &scenes {
        if !MeetingPlace::ALL.iter().any(|p| p.as_str() == *s) {
            return Err(format!(
                "--scenes 里的 {s} 不是场景键（例：rail-air-cabin · canteen · meeting-room · transit-hub）"
            ));
        }
    }
    let need = |k: &str| {
        arg(args, k)
            .ok_or_else(|| format!("缺 {k}（用法见 src/eval/src/scorecard/accept.rs 头注释）"))
    };
    let (pa, pb) = (need("--card-a")?, need("--card-b")?);
    let read = |p: &str| std::fs::read_to_string(p).map_err(|e| format!("{p}：{e}"));
    let mut cmps = Vec::new();
    for p in &vs {
        cmps.push(parse_compare(&read(p)?, p)?);
    }
    let mut nois = Vec::new();
    for p in &noise {
        nois.push(parse_compare(&read(p)?, p)?);
    }
    let card_a = parse_card(&read(&pa)?, &pa)?;
    let card_b = parse_card(&read(&pb)?, &pb)?;
    if let Some(rep) = cmps.first() {
        check_binding(rep, &card_a, &card_b)?;
    }
    let report = evaluate(&cmps, &nois, &scenes, &card_a, &card_b)?;
    let out = arg(args, "--out").map(PathBuf::from).unwrap_or(default_out);
    for f in FILES {
        if out.join(f).exists() {
            return Err(format!(
                "{} 已存在，不覆盖：换一个 --out 目录",
                out.join(f).display()
            ));
        }
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let md = to_markdown(&report, &vs, &noise, &scenes);
    let js = serde_json::to_string_pretty(&to_json(&report, &vs, &noise, &scenes, run_id))
        .map_err(|e| e.to_string())?
        + "\n";
    for (f, t) in FILES.iter().zip([md, js]) {
        super::card::write_new_file(&out.join(f), &t)?;
    }
    Ok(out)
}
