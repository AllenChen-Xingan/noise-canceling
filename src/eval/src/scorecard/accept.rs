//! `vdn-eval accept`：把计划 014 第三节的验收线落到配对比较与两份成绩单上，逐条判「过 / 不过 / 判不了」。
//!
//! ```text
//! vdn-eval accept --suite <评测清单.json>              # 必给：成绩单就是用它跑的，预期场景 / 片段从它来
//!                 --vs <compare.json> [--vs …]        # 候选对对照，一个种子一份；第一个 = 报告种子
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
//! - **卡片对得上**：`--card-a` / `--card-b` 旁边必须有 `clips.jsonl`，它的 sha256 必须等于第一份 `--vs`
//!   那一边记的 sha256；系统名、评测清单指纹也要对上；不符直接报错不判（`inputs::check_binding`）
//! - **覆盖齐**：预期场景 / 切片 / 片段只从 `--suite` 来（独立于成绩单）；两份逐段文件、两份成绩单、
//!   compare 里少一段就判不了（`Expected::coverage`）——不能让成绩单自己说自己齐了
//!
//! 阈值全部从 [`crate::thresholds::ACCEPT`] 取（正本 `SC-ACCEPT`），本文件不写数字；
//! 延迟的线是基线成绩单量出来的 V-LAT 本身（014 第三节第 4 条），不是另一个阈值。

use std::path::PathBuf;

use serde_json::{Value, json};

use crate::scene::MeetingPlace;

mod inputs;
mod judge;

pub use inputs::*;
use judge::evaluate;

/// 「人耳未测」：本结论没做人耳盲听（D99 第 3 条）。
pub const NOT_LISTENED: &str = "人耳未测：本结论没做人耳盲听（D99 第 3 条）。";
/// 「只在合成集上判」：评测集是合成场景集，真实会议集未定（014 第三节第 6 条）。
pub const SYNTH_ONLY: &str =
    "只在合成集上判：评测集是合成场景集，真实会议集未定（014 第三节第 6 条）。";
/// 「两个评分模型之一只抓得到 0.20 的坏段」：听感没露头的坏段仍可能漏判（014 第三节第 5 条 · 调研 031）。
pub const MODEL_RECALL: &str = "评分模型之一只抓得到 0.20 的坏段（014 第三节第 5 条 · 调研 031）：听感没露头的坏段仍可能漏判。";

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
        "# 验收判定（计划 014 第三节）\n\n{NOT_LISTENED}\n\n{SYNTH_ONLY}\n\n{MODEL_RECALL}\n\n**结论：{}**\n\n",
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
        "notes": [NOT_LISTENED, SYNTH_ONLY, MODEL_RECALL],
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
    let suite_path = need("--suite")?;
    let (expected, fp) = Expected::load(&suite_path)?;
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
    let card_a = load_card(&pa)?;
    let card_b = load_card(&pb)?;
    if let Some(rep) = cmps.first() {
        check_binding(rep, &card_a, &card_b, &fp)?;
    }
    let report = evaluate(&cmps, &nois, &scenes, &expected, &card_a, &card_b)?;
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
