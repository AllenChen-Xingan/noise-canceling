//! 报告：机读 JSON + 人读 Markdown。每条 verifier 的分数 · 级别 · 是否过线；回归 / 能力 / 探针分开汇总；
//! 附抽查样本路径（Anthropic：「读原始记录」——分数之外一定要能听到那段音频）。

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::{Value, json};

use crate::runner::{Gate, SuiteResult, TaskResult};
use crate::suite::Set;
use crate::verifiers::REGISTRY;
use crate::{Level, Verdict};

pub struct Meta {
    pub run_id: String,
    pub commit: String,
    pub profile: &'static str,
    pub ci: bool,
}

fn gate_json(g: &Gate) -> Value {
    match g {
        Gate::Ok => json!({"kind": "ok"}),
        Gate::Info => json!({"kind": "info"}),
        Gate::Warn(w) => json!({"kind": "warn", "why": w}),
        Gate::Block(w) => json!({"kind": "block", "why": w}),
    }
}

fn r4(v: f64) -> Value {
    if v.is_finite() {
        json!((v * 10_000.0).round() / 10_000.0)
    } else {
        Value::Null
    }
}

fn task_json(t: &TaskResult) -> Value {
    let trials: Vec<Value> = t
        .trials
        .iter()
        .map(|tr| {
            let graders: Vec<Value> = tr
                .graders
                .iter()
                .map(|g| {
                    let s = &g.score;
                    let extra: serde_json::Map<String, Value> =
                        s.extra.iter().map(|(k, v)| (k.to_string(), r4(*v))).collect();
                    json!({
                        "v": g.v,
                        "level": g.level.as_str(),
                        "info_only": g.info_only,
                        "value": s.value.map(r4),
                        "unit": s.unit,
                        "better": s.better.as_str(),
                        "threshold": s.threshold,
                        "grade": s.grade.map(|c| c.to_string()),
                        "verdict": s.verdict.as_str(),
                        "unknown_reason": match &s.verdict { Verdict::Unknown(w) => Some(w.clone()), _ => None },
                        "extra": extra,
                        "note": s.note,
                        "probe_base_value": g.base.as_ref().and_then(|b| b.value).map(r4),
                        "baseline": g.baseline.map(r4),
                        "gate": gate_json(&g.gate),
                    })
                })
                .collect();
            json!({
                "trial": tr.trial,
                "seed": tr.seed,
                "pass": tr.pass,
                "samples": tr.samples.iter().map(|p| rel(p)).collect::<Vec<_>>(),
                "graders": graders,
            })
        })
        .collect();
    json!({
        "id": t.id,
        "set": t.set.as_str(),
        "polarity": t.polarity,
        "system": t.system.as_str(),
        "origin": t.origin,
        "desc": t.desc,
        "skipped": t.skipped,
        "xfail": t.xfail,
        "xfail_now_passes": t.xfail_now_passes(),
        "k": t.k(),
        "passes": t.passes(),
        "pass_at_k": t.pass_at_k(),
        "pass_pow_k": t.pass_pow_k(),
        "trials": trials,
    })
}

/// 一个系统、一个集合的汇总。
#[derive(Default, Clone, Copy)]
pub struct SetSummary {
    pub tasks: usize,
    pub skipped: usize,
    pub pass_at_k: usize,
    pub pass_pow_k: usize,
    pub trials: usize,
    pub trial_passes: usize,
}

pub fn summarize(r: &SuiteResult) -> BTreeMap<(String, Set), SetSummary> {
    let mut m: BTreeMap<(String, Set), SetSummary> = BTreeMap::new();
    for t in &r.tasks {
        let e = m.entry((t.system.as_str().to_string(), t.set)).or_default();
        e.tasks += 1;
        if t.skipped.is_some() || t.k() == 0 {
            e.skipped += 1;
            continue;
        }
        e.pass_at_k += usize::from(t.pass_at_k() == Some(true));
        e.pass_pow_k += usize::from(t.pass_pow_k() == Some(true));
        e.trials += t.k();
        e.trial_passes += t.passes();
    }
    m
}

pub fn to_json(r: &SuiteResult, meta: &Meta, levels: &BTreeMap<String, (Level, String)>) -> Value {
    let summary: Vec<Value> = summarize(r)
        .iter()
        .map(|((sys, set), s)| {
            json!({
                "system": sys, "set": set.as_str(), "tasks": s.tasks, "skipped": s.skipped,
                "pass_at_k": s.pass_at_k, "pass_pow_k": s.pass_pow_k,
                "trials": s.trials, "trial_passes": s.trial_passes,
            })
        })
        .collect();
    let verifiers: Vec<Value> = REGISTRY
        .iter()
        .map(|i| {
            let (l, basis) = levels
                .get(i.id)
                .cloned()
                .unwrap_or((i.default_level, String::new()));
            json!({"id": i.id, "name": i.name, "layer": i.layer, "kind": i.kind.as_str(),
                   "level": l.as_str(), "basis": basis, "unit": i.unit, "better": i.better.as_str()})
        })
        .collect();
    json!({
        "run_id": meta.run_id,
        "commit": meta.commit,
        "layer": r.layer,
        "suite": r.suite,
        "profile": meta.profile,
        "ci": meta.ci,
        "systems": r.systems.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        "skipped_systems": r.skipped_systems.iter().map(|(s, w)| json!({"system": s.as_str(), "why": w})).collect::<Vec<_>>(),
        "gate": {"ok": r.gate_ok(), "blocks": r.blocks()},
        "summary": summary,
        "verifiers": verifiers,
        "tasks": r.tasks.iter().map(task_json).collect::<Vec<_>>(),
    })
}

/// 仓库内路径写成相对路径（报告进仓，不留本机绝对路径——铁律 12）。
fn rel(p: &std::path::Path) -> String {
    let root = crate::suite::repo_root();
    p.strip_prefix(&root)
        .map(|r| r.display().to_string())
        .unwrap_or_else(|_| {
            p.file_name()
                .map_or_else(String::new, |f| format!("<仓外>/{}", f.to_string_lossy()))
        })
}

fn yn(b: Option<bool>) -> &'static str {
    match b {
        Some(true) => "✓",
        Some(false) => "✗",
        None => "?",
    }
}

pub fn to_markdown(
    r: &SuiteResult,
    meta: &Meta,
    levels: &BTreeMap<String, (Level, String)>,
) -> String {
    let mut s = String::new();
    let blocks = r.blocks();
    let _ = writeln!(s, "# 评测报告 · {} · {}\n", r.suite, meta.run_id);
    let _ = writeln!(
        s,
        "**门禁：{}**（阻断 {} 处）· 层 `{}` · 提交 `{}` · 构建 `{}`{}\n",
        if blocks.is_empty() {
            "通过"
        } else {
            "不通过"
        },
        blocks.len(),
        r.layer,
        meta.commit,
        meta.profile,
        if meta.ci { " · CI" } else { "" }
    );
    let sys: Vec<&str> = r.systems.iter().map(|s| s.as_str()).collect();
    let _ = writeln!(s, "被测系统：{}", sys.join("、"));
    for (k, w) in &r.skipped_systems {
        let _ = writeln!(s, "- 跳过 `{}`：{w}", k.as_str());
    }
    if r.skipped_systems.iter().any(|(k, _)| {
        matches!(
            k,
            crate::system::SystemKind::Gtcrn | crate::system::SystemKind::FastEnhancerM
        )
    }) {
        let _ = writeln!(
            s,
            "\n**注意：本次没测产品模型**（权重不在本机），门禁只覆盖 Engine + 参考降噪器。"
        );
    }
    let xf: Vec<&TaskResult> = r.tasks.iter().filter(|t| t.xfail.is_some()).collect();
    if !xf.is_empty() {
        let _ = writeln!(s, "\n已知失败（xfail，阻断降为报警，留给修复线）：");
        for t in xf {
            let _ = writeln!(
                s,
                "- {} [{}]：{}{}",
                t.id,
                t.system.as_str(),
                t.xfail.as_deref().unwrap_or(""),
                if t.xfail_now_passes() {
                    " —— **本次全过，可以转正**"
                } else {
                    ""
                }
            );
        }
    }
    let _ = writeln!(
        s,
        "\n`ref` = 真 Engine（三件保命）+ 参考降噪器（维纳，CI 替身，**不是产品模型**）；`gtcrn` / `fastenhancer-m` = 真模型。\n"
    );

    let _ = writeln!(s, "## 汇总（回归 / 能力 / 探针分开）\n");
    let _ = writeln!(
        s,
        "| 系统 | 集合 | 任务 | pass@k | pass^k | 试验通过 | 跳过 |"
    );
    let _ = writeln!(s, "|---|---|---|---|---|---|---|");
    for ((sy, set), v) in summarize(r) {
        let _ = writeln!(
            s,
            "| {sy} | {} | {} | {}/{} | {}/{} | {}/{} | {} |",
            set.as_str(),
            v.tasks,
            v.pass_at_k,
            v.tasks - v.skipped,
            v.pass_pow_k,
            v.tasks - v.skipped,
            v.trial_passes,
            v.trials,
            v.skipped
        );
    }
    let _ = writeln!(
        s,
        "\npass@k = k 次试验至少过一次；pass^k = k 次全过（Anthropic 文章的两种口径）。合成任务的 k 次试验 = k 个种子；探针的「过」= verifier 抓住了劣化。\n"
    );

    if !blocks.is_empty() {
        let _ = writeln!(s, "## 阻断\n");
        for b in &blocks {
            let _ = writeln!(s, "- {b}");
        }
        let _ = writeln!(s);
    }
    let mut warns: BTreeMap<String, usize> = BTreeMap::new();
    for t in &r.tasks {
        for tr in &t.trials {
            for g in &tr.graders {
                if let Gate::Warn(w) = &g.gate {
                    *warns
                        .entry(format!("{} [{}] {}：{w}", t.id, t.system.as_str(), g.v))
                        .or_default() += 1;
                }
            }
        }
    }
    if !warns.is_empty() {
        let _ = writeln!(s, "## 报警（不阻断）\n");
        for (w, n) in &warns {
            let _ = writeln!(s, "- {w}（{n} 次）");
        }
        let _ = writeln!(s);
    }

    let _ = writeln!(s, "## 逐任务\n");
    let _ = writeln!(
        s,
        "| 任务 | 集合 | 正反 | 系统 | 过/k | pass^k | 各 verifier（第 1 次试验：值 · 判 · 级） |"
    );
    let _ = writeln!(s, "|---|---|---|---|---|---|---|");
    for t in &r.tasks {
        if let Some(w) = &t.skipped {
            let _ = writeln!(
                s,
                "| {} | {} | {} | {} | — | — | 跳过：{w} |",
                t.id,
                t.set.as_str(),
                t.polarity,
                t.system.as_str()
            );
            continue;
        }
        let cells: Vec<String> = t
            .trials
            .first()
            .map(|tr| {
                tr.graders
                    .iter()
                    .map(|g| {
                        let v = g.score.value.map_or("?".to_string(), |v| format!("{v:.2}"));
                        let mark = match (&g.gate, &g.score.verdict) {
                            (Gate::Info, _) => "ⓘ",
                            (Gate::Block(_), _) => "⛔",
                            (_, Verdict::Pass) => "✓",
                            (_, Verdict::Fail) => "✗",
                            (_, Verdict::Unknown(_)) => "?",
                        };
                        let base = g
                            .base
                            .as_ref()
                            .and_then(|b| b.value)
                            .map_or(String::new(), |b| format!("（原 {b:.2}）"));
                        format!(
                            "{} {v}{}{base} {mark} {}",
                            g.v,
                            g.score.unit,
                            g.level.as_str()
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {}/{} | {} | {} |",
            t.id,
            t.set.as_str(),
            t.polarity,
            t.system.as_str(),
            t.passes(),
            t.k(),
            yn(t.pass_pow_k()),
            cells.join("<br>")
        );
    }
    let _ = writeln!(
        s,
        "\n记号：✓ 过 · ✗ 不过 · ? 判不了 · ⛔ 阻断 · ⓘ 只报。「原」= 探针未劣化时的分数。\n"
    );

    let _ = writeln!(s, "## verifier 与级别\n");
    let _ = writeln!(s, "| id | 名称 | 层 | 类 | 级别 | 门禁用法 |");
    let _ = writeln!(s, "|---|---|---|---|---|---|");
    for i in REGISTRY {
        let l = levels.get(i.id).map_or(i.default_level, |e| e.0);
        let usage = match (i.kind, l) {
            (crate::verifiers::Kind::Perceptual, Level::L0 | Level::L1) => {
                "回归集只报警 + 不许比冻结基线差"
            }
            (crate::verifiers::Kind::Perceptual, _) => "回归集阻断",
            (crate::verifiers::Kind::Sim, _) => "回归集阻断（仅模拟片段）",
            (crate::verifiers::Kind::Physical, _) => "回归集阻断",
        };
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {usage} |",
            i.id,
            i.name,
            i.layer,
            i.kind.as_str(),
            l.as_str()
        );
    }

    let mut samples = Vec::new();
    for t in &r.tasks {
        for tr in &t.trials {
            if !tr.samples.is_empty() {
                samples.push((t, tr));
            }
        }
    }
    let _ = writeln!(s, "\n## 抽查样本（读原始记录）\n");
    if samples.is_empty() {
        let _ = writeln!(s, "本次没写音频（CI 默认不写；本机加 `--audio`）。");
    } else {
        let _ = writeln!(
            s,
            "每个失败任务的第一次失败试验 + 每 5 个任务固定抽 1 个。音频在仓外，不进仓。\n"
        );
        for (t, tr) in samples {
            let _ = writeln!(
                s,
                "- {} [{}] 试验 {}（{}）：{}",
                t.id,
                t.system.as_str(),
                tr.trial,
                yn(tr.pass),
                tr.samples
                    .iter()
                    .map(|p| format!("`{}`", rel(p)))
                    .collect::<Vec<_>>()
                    .join(" · ")
            );
        }
    }
    s
}
