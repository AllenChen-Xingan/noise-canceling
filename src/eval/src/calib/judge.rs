//! 定标的第二种量法（调研 031 第 1 条）：评估器按成绩单的**过关线**判「过 / 不过」时，和人耳的混淆矩阵。
//!
//! 排序一致（`calib.rs` 主体）只说明「A 比 B 好」的方向对不对；成绩单真正用的是过关线。
//! 这里分开报（Hamel《evals FAQ》的口径，「阳性」= 不过 = 有缺陷，人耳是真值）：
//! - 真阳性率：人说不过的里面，评估器也判不过（抓到坏的）
//! - 真阴性率：人说过的里面，评估器也判过（不冤枉好的）
//! - 总一致率与「一律判过」时的总一致率并列：类别不平衡时总一致率会被后者撑高
//!
//! 人耳真值：「开会能用吗」宽切（no = 不过）与严切（no 或 marginal = 不过）；「吃字」勾选 = 不过。
//! 过关线一律从 `crate::thresholds` 正本取，这里不写数字。只报不判：级别判据不看这里（改判据要人定）。
//! 区间：Wilson 95%；另给按屏重采样的区间（同一屏几个候选不独立），种子与次数同排序一致那部分。
//! 另一份独立实现在 `docs/experiments/exp022-judge-tpr-tnr/`（Python），本文件测试逐格比对两边的计数。

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::analysis::quantile;
use crate::rng::Rng;
use crate::thresholds as th;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum V {
    Pass,
    Fail,
}

/// 人耳真值的切法。
#[derive(Clone, Copy, Debug)]
pub enum Human {
    UsableLoose,
    UsableStrict,
    Eaten,
}

impl Human {
    pub fn id(self) -> &'static str {
        match self {
            Human::UsableLoose => "usable_loose",
            Human::UsableStrict => "usable_strict",
            Human::Eaten => "eaten",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Human::UsableLoose => "开会能用：no = 不过",
            Human::UsableStrict => "开会能用：no 或 marginal = 不过",
            Human::Eaten => "人勾吃字 = 不过",
        }
    }
}

/// 一条（屏, 候选）的输入：答卷、指标表、可选的评分模型表。
pub struct Item<'a> {
    pub clip: &'a str,
    pub model: &'a str,
    pub usable: Option<&'a str>,
    pub eaten: bool,
    pub scores: &'a Value,
    pub noisy: &'a Value,
    /// `{scoreq, distill_mos}`；没给评分模型表时为 Null
    pub pred: &'a Value,
    pub pred_noisy: &'a Value,
}

impl Item<'_> {
    pub fn truth(&self, h: Human) -> Option<V> {
        let u = self.usable;
        match h {
            Human::Eaten => Some(if self.eaten { V::Fail } else { V::Pass }),
            Human::UsableLoose | Human::UsableStrict => match u? {
                "no" => Some(V::Fail),
                "marginal" if matches!(h, Human::UsableStrict) => Some(V::Fail),
                "ok" | "marginal" => Some(V::Pass),
                _ => None,
            },
        }
    }
}

fn f(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64)
}

/// 吃字：候选比带噪原声多错（删 + 替）几个词 / 字。
fn extra_tokens(it: &Item) -> Option<f64> {
    let ds = |v: &Value| Some(f(v, "del")? + f(v, "sub")?);
    Some(ds(it.scores)? - ds(it.noisy)?)
}

fn delta(it: &Item, k: &str) -> Option<f64> {
    Some(f(it.pred, k)? - f(it.pred_noisy, k)?)
}

fn at_least(x: f64, line: f64) -> V {
    if x >= line - 1e-12 { V::Pass } else { V::Fail }
}

/// 参数的第一个登记取值（`default` 为 `None` 的参数，如 V-EAT-ASR 的 v1 线 `max`）。
fn registered(l: &th::Lines, key: &str) -> f64 {
    l.params
        .iter()
        .find(|p| p.key == key)
        .and_then(|p| p.default.or_else(|| p.values.first().map(|v| v.0)))
        .unwrap_or(f64::NAN)
}

/// 一个评估器在单段上的判定。
pub struct Judge {
    pub id: &'static str,
    pub verifier: &'static str,
    pub label: &'static str,
    pub humans: &'static [Human],
    pub judge: fn(&Item) -> Option<V>,
}

const USABLE: &[Human] = &[Human::UsableLoose, Human::UsableStrict];

pub fn judges() -> Vec<Judge> {
    vec![
        Judge {
            id: "scoreq_v1",
            verifier: "V-NRMOS-SCOREQ",
            label: "SCOREQ 单段 Δ ≥ min（v1）",
            humans: USABLE,
            judge: |it| Some(at_least(delta(it, "scoreq")?, th::NRMOS_SCOREQ.def("min"))),
        },
        Judge {
            id: "distill_v1",
            verifier: "V-NRMOS-DISTILL",
            label: "Distill-MOS 单段 Δ ≥ min（v1）",
            humans: USABLE,
            judge: |it| {
                Some(at_least(
                    delta(it, "distill_mos")?,
                    th::NRMOS_DISTILL.def("min"),
                ))
            },
        },
        Judge {
            id: "scoreq_slack",
            verifier: "V-NRMOS-SCOREQ",
            label: "SCOREQ 单段 Δ ≥ −SC-NRMOS-DELTA（v2 的线用在单段上）",
            humans: USABLE,
            judge: |it| Some(at_least(delta(it, "scoreq")?, -th::NRMOS_DELTA.def("max"))),
        },
        Judge {
            id: "distill_slack",
            verifier: "V-NRMOS-DISTILL",
            label: "Distill-MOS 单段 Δ ≥ −SC-NRMOS-DELTA",
            humans: USABLE,
            judge: |it| {
                Some(at_least(
                    delta(it, "distill_mos")?,
                    -th::NRMOS_DELTA.def("max"),
                ))
            },
        },
        Judge {
            id: "eat_tokens",
            verifier: "V-EAT-ASR",
            label: "吃字：多错 ≥ fail_chars 个（v2）",
            humans: &[Human::Eaten],
            judge: |it| {
                let lim = th::EAT_ASR.def("fail_chars");
                Some(if extra_tokens(it)? >= lim {
                    V::Fail
                } else {
                    V::Pass
                })
            },
        },
        Judge {
            id: "eat_pp",
            verifier: "V-EAT-ASR",
            label: "吃字：多错 > max 个百分点（v1 · rubric V7）",
            humans: &[Human::Eaten],
            judge: |it| {
                let pp = extra_tokens(it)? / f(it.scores, "ref_len")?.max(1.0) * 100.0;
                let lim = registered(&th::EAT_ASR, "max");
                Some(if pp > lim + 1e-12 { V::Fail } else { V::Pass })
            },
        },
    ]
}

/// 混淆矩阵：tp = 人不过且评估器不过 · fn_ = 人不过评估器过（漏判）· tn · fp = 人过评估器不过（冤枉）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Confusion {
    pub tp: usize,
    pub fn_: usize,
    pub tn: usize,
    pub fp: usize,
}

impl Confusion {
    pub fn of(pairs: &[(V, V)]) -> Self {
        let mut c = Confusion::default();
        for (h, j) in pairs {
            match (h, j) {
                (V::Fail, V::Fail) => c.tp += 1,
                (V::Fail, V::Pass) => c.fn_ += 1,
                (V::Pass, V::Pass) => c.tn += 1,
                (V::Pass, V::Fail) => c.fp += 1,
            }
        }
        c
    }

    fn ratio(k: usize, n: usize) -> f64 {
        if n == 0 {
            f64::NAN
        } else {
            k as f64 / n as f64
        }
    }

    pub fn tpr(&self) -> f64 {
        Self::ratio(self.tp, self.tp + self.fn_)
    }

    pub fn tnr(&self) -> f64 {
        Self::ratio(self.tn, self.tn + self.fp)
    }
}

/// 二项比例的 Wilson 95% 区间。
pub fn wilson(k: usize, n: usize) -> (f64, f64) {
    if n == 0 {
        return (f64::NAN, f64::NAN);
    }
    let z = 1.959_964f64;
    let (nf, p) = (n as f64, k as f64 / n as f64);
    let den = 1.0 + z * z / nf;
    let mid = (p + z * z / (2.0 * nf)) / den;
    let half = z * (p * (1.0 - p) / nf + z * z / (4.0 * nf * nf)).sqrt() / den;
    ((mid - half).max(0.0), (mid + half).min(1.0))
}

/// 按屏重采样的 95% 区间（跳过 NaN）。
fn cluster_ci(
    by: &BTreeMap<&str, Vec<(V, V)>>,
    rate: fn(&Confusion) -> f64,
    seed: u64,
    reps: usize,
) -> (f64, f64) {
    let keys: Vec<&&str> = by.keys().collect();
    let mut rng = Rng::new(seed);
    let mut vals = Vec::with_capacity(reps);
    for _ in 0..reps {
        let mut s = Vec::new();
        for _ in 0..keys.len() {
            s.extend_from_slice(&by[keys[rng.below(keys.len())]]);
        }
        let v = rate(&Confusion::of(&s));
        if v.is_finite() {
            vals.push(v);
        }
    }
    (quantile(&vals, 0.025), quantile(&vals, 0.975))
}

/// 从答卷抽（屏, 候选）条目：只用 `kind == "real"` 的屏，与 [`super::extract`] 同一口径。
/// `scores` 是指标表（`per_clip[clip][model]`，原声为 `noisy`）；`preds` 是评分模型表（`[clip][model]`，可为 Null）。
pub fn items<'a>(answers: &'a Value, scores: &'a Value, preds: &'a Value) -> Vec<Item<'a>> {
    let per = &scores["per_clip"];
    let mut out = Vec::new();
    for t in answers["trials"].as_array().into_iter().flatten() {
        if t["kind"] != "real" {
            continue;
        }
        let Some(clip) = t["clip"].as_str() else {
            continue;
        };
        let eaten: Vec<&str> = t["by_model"]["eaten"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        for m in t["slots"].as_object().into_iter().flat_map(|o| o.values()) {
            let Some(model) = m.as_str() else { continue };
            out.push(Item {
                clip,
                model,
                usable: t["by_model"]["usable"][model].as_str(),
                eaten: eaten.contains(&model),
                scores: &per[clip][model],
                noisy: &per[clip]["noisy"],
                pred: &preds[clip][model],
                pred_noisy: &preds[clip]["noisy"],
            });
        }
    }
    out
}

/// 对每个评估器 × 每种人耳切法出一行。没有评分模型表时，听感两项的条目全部「判不了」，行照出、条数为 0。
pub fn table(items: &[Item], seed: u64, reps: usize) -> Vec<Value> {
    let mut rows = Vec::new();
    for j in judges() {
        for &h in j.humans {
            let mut by: BTreeMap<&str, Vec<(V, V)>> = BTreeMap::new();
            let mut skipped = 0usize;
            for it in items {
                match (it.truth(h), (j.judge)(it)) {
                    (Some(t), Some(v)) => by.entry(it.clip).or_default().push((t, v)),
                    _ => skipped += 1,
                }
            }
            let all: Vec<(V, V)> = by.values().flatten().copied().collect();
            let c = Confusion::of(&all);
            let (nf, np) = (c.tp + c.fn_, c.tn + c.fp);
            let n = nf + np;
            let r = |k: usize| {
                if n == 0 {
                    f64::NAN
                } else {
                    k as f64 / n as f64
                }
            };
            rows.push(json!({
                "judge": j.id, "verifier": j.verifier, "label": j.label,
                "human": h.id(), "human_label": h.label(),
                "n": n, "skipped": skipped, "human_fail": nf, "human_pass": np,
                "confusion": {"tp": c.tp, "fn": c.fn_, "tn": c.tn, "fp": c.fp},
                "tpr": c.tpr(), "tpr_ci95": wilson(c.tp, nf),
                "tpr_ci95_cluster": cluster_ci(&by, Confusion::tpr, seed, reps),
                "tnr": c.tnr(), "tnr_ci95": wilson(c.tn, np),
                "tnr_ci95_cluster": cluster_ci(&by, Confusion::tnr, seed, reps),
                "raw_agreement": r(c.tp + c.tn),
                "always_pass_agreement": r(np),
                "youden_j": c.tpr() + c.tnr() - 1.0,
            }));
        }
    }
    rows
}

/// 定标报告里「按过关线判」一节。
pub fn to_markdown(rows: &[Value]) -> String {
    let f = |v: &Value| v.as_f64().map_or("—".into(), |x| format!("{x:.2}"));
    let ci = |v: &Value| format!("[{}, {}]", f(&v[0]), f(&v[1]));
    let mut s = String::from(
        "\n## 按成绩单过关线判：和人耳的混淆矩阵（只报不判）\n\n\
         「阳性」= 不过。真阳性率 = 人说不过的里面评估器也判不过（抓到坏的）；\
         真阴性率 = 人说过的里面评估器也判过（不冤枉好的）。\
         总一致率不高于「一律判过时的一致率」，说明总一致率没有信息量。区间：Wilson · 按屏重采样。\n\n\
         | 评估器（单段线） | 人耳真值 | 条数（判不了） | 人不过 / 人过 | TP FN TN FP | 真阳性率 | 真阴性率 | 总一致率 / 一律判过时 | TPR+TNR−1 |\n\
         |---|---|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let c = &r["confusion"];
        s += &format!(
            "| {} | {} | {}（{}） | {} / {} | {} {} {} {} | {} {} {} | {} {} {} | {} / {} | {} |\n",
            r["label"].as_str().unwrap_or(""),
            r["human_label"].as_str().unwrap_or(""),
            r["n"],
            r["skipped"],
            r["human_fail"],
            r["human_pass"],
            c["tp"],
            c["fn"],
            c["tn"],
            c["fp"],
            f(&r["tpr"]),
            ci(&r["tpr_ci95"]),
            ci(&r["tpr_ci95_cluster"]),
            f(&r["tnr"]),
            ci(&r["tnr_ci95"]),
            ci(&r["tnr_ci95_cluster"]),
            f(&r["raw_agreement"]),
            f(&r["always_pass_agreement"]),
            f(&r["youden_j"]),
        );
    }
    s
}

#[cfg(test)]
#[path = "judge_tests.rs"]
mod tests;
