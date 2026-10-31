//! 定标（D44 · 012 §2 与 §5）：读人耳答卷 + 每个候选的指标表，算每条感知 verifier 与人耳的
//! 成对方向一致率（按屏聚类 bootstrap 区间）与片段内 Kendall τ，再按 L0–L3 判据定级。
//!
//! 规则照 012（全部〔建议值〕，拍板 #8）：
//! - 配对：AB 页被选「最 X」者对同屏其余各一对；H3（能用 / 勉强 / 不能用）同屏差 ≥ 1 级的两个候选一对。
//!   **只按人耳分差筛**，不按 verifier 自己的分差筛
//! - L1：累计**验证集**配对 ≥ 20 且一致率 95% 下界 > 0.5；L2：再加验证集片段 ≥ 30、片段内 τ 下界 ≥ 0.2
//! - 奇数轮进定标集、偶数轮进验证集；判级只看验证集 → 第 1 轮无论数字多好都是 L0
//! - 统计全部确定性：种子 20261030，bootstrap 10000 次

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::Level;
use crate::analysis::quantile;
use crate::rng::Rng;

/// 一个指标在答卷候选上的取值来源。
pub struct Metric {
    /// 对应的 verifier（或「未登记」指标的名字）
    pub verifier: &'static str,
    pub label: &'static str,
    pub higher_better: bool,
    /// 从 scores.json 的 per_clip[clip][model] 取值
    pub get: fn(&Value, &Value) -> Option<f64>,
}

fn field(name: &'static str) -> impl Fn(&Value) -> Option<f64> {
    move |v: &Value| v.get(name).and_then(Value::as_f64)
}

/// exp003 scores.json 里能拿来定标的指标。
pub fn exp003_metrics() -> Vec<Metric> {
    vec![
        Metric {
            verifier: "V-DNSMOS",
            label: "DNSMOS OVRL",
            higher_better: true,
            get: |c, _| field("OVRL")(c),
        },
        Metric {
            verifier: "V-DNSMOS.SIG",
            label: "DNSMOS SIG",
            higher_better: true,
            get: |c, _| field("SIG")(c),
        },
        Metric {
            verifier: "V-DNSMOS.BAK",
            label: "DNSMOS BAK",
            higher_better: true,
            get: |c, _| field("BAK")(c),
        },
        Metric {
            verifier: "V-NR",
            label: "停顿衰减 noise_att_db（越低 = 压得越多）",
            higher_better: false,
            get: |c, _| field("noise_att_db")(c),
        },
        Metric {
            verifier: "V-EAT-ASR",
            label: "Δ(del + sub) 相对原声（越低越好）",
            higher_better: false,
            get: |c, noisy| {
                let ds = |v: &Value| Some(field("del")(v)? + field("sub")(v)?);
                let l = field("ref_len")(c)?.max(1.0);
                Some((ds(c)? - ds(noisy)?) / l * 100.0)
            },
        },
        Metric {
            verifier: "ESTOI（未登记）",
            label: "ESTOI",
            higher_better: true,
            get: |c, _| field("ESTOI")(c),
        },
        Metric {
            verifier: "SI-SDR（未登记）",
            label: "SI-SDR",
            higher_better: true,
            get: |c, _| field("SI_SDR_dB")(c),
        },
    ]
}

/// 一对人耳判断：同屏 winner 优于 loser。
#[derive(Clone, Debug)]
pub struct Pair {
    pub screen: String,
    pub winner: String,
    pub loser: String,
    /// "clearer" · "natural" · "h3"
    pub source: &'static str,
}

fn grade(s: &str) -> Option<i32> {
    match s {
        "ok" => Some(2),
        "marginal" => Some(1),
        "no" => Some(0),
        _ => None,
    }
}

/// 从答卷抽配对与 H3 等级。只用 `kind == "real"` 的屏（陷阱屏不进定标）。
pub fn extract(answers: &Value) -> (Vec<Pair>, BTreeMap<String, BTreeMap<String, i32>>) {
    let mut pairs = Vec::new();
    let mut h3: BTreeMap<String, BTreeMap<String, i32>> = BTreeMap::new();
    for t in answers["trials"].as_array().into_iter().flatten() {
        if t["kind"] != "real" {
            continue;
        }
        let Some(clip) = t["clip"].as_str() else {
            continue;
        };
        let models: Vec<String> = t["slots"]
            .as_object()
            .map(|o| {
                o.values()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        for q in ["clearer", "natural"] {
            if let Some(w) = t["by_model"][q].as_str()
                && models.iter().any(|m| m == w)
            {
                for l in models.iter().filter(|m| *m != w) {
                    pairs.push(Pair {
                        screen: clip.into(),
                        winner: w.into(),
                        loser: l.clone(),
                        source: if q == "clearer" { "clearer" } else { "natural" },
                    });
                }
            }
        }
        let mut g = BTreeMap::new();
        if let Some(u) = t["by_model"]["usable"].as_object() {
            for (m, v) in u {
                if let Some(x) = v.as_str().and_then(grade) {
                    g.insert(m.clone(), x);
                }
            }
        }
        let ms: Vec<(&String, &i32)> = g.iter().collect();
        for i in 0..ms.len() {
            for j in 0..ms.len() {
                if ms[i].1 - ms[j].1 >= 1 {
                    pairs.push(Pair {
                        screen: clip.into(),
                        winner: ms[i].0.clone(),
                        loser: ms[j].0.clone(),
                        source: "h3",
                    });
                }
            }
        }
        h3.insert(clip.into(), g);
    }
    (pairs, h3)
}

fn kendall_tau_b(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len();
    let (mut c, mut d, mut tx, mut ty) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for i in 0..n {
        for j in i + 1..n {
            let a = (x[i] - x[j]).signum();
            let b = (y[i] - y[j]).signum();
            if a == 0.0 && b == 0.0 {
                continue;
            } else if a == 0.0 {
                tx += 1.0;
            } else if b == 0.0 {
                ty += 1.0;
            } else if a == b {
                c += 1.0;
            } else {
                d += 1.0;
            }
        }
    }
    let den = ((c + d + tx) * (c + d + ty)).sqrt();
    (den > 0.0).then(|| (c - d) / den)
}

/// 定标一轮。`scores` 是 exp003 scores.json（`per_clip[clip][model]`，原声为 `noisy`）。
pub fn calibrate(answers: &Value, scores: &Value, round: u32) -> Value {
    let (pairs, h3) = extract(answers);
    let per = &scores["per_clip"];
    let is_validation = round % 2 == 0;
    let discounted = answers["summary"]["traps"]
        .as_array()
        .is_some_and(|a| a.iter().any(|t| t["pass"] == false));
    let mut rows = Vec::new();
    for m in exp003_metrics() {
        let val = |clip: &str, model: &str| -> Option<f64> {
            (m.get)(&per[clip][model], &per[clip]["noisy"])
        };
        // 每对：1 = 指标与耳朵同向，0 = 反向，0.5 = 指标打平
        let scored: Vec<(String, &'static str, f64)> = pairs
            .iter()
            .filter_map(|p| {
                let (w, l) = (val(&p.screen, &p.winner)?, val(&p.screen, &p.loser)?);
                let agree = if (w - l).abs() < 1e-9 {
                    0.5
                } else if (w > l) == m.higher_better {
                    1.0
                } else {
                    0.0
                };
                Some((p.screen.clone(), p.source, agree))
            })
            .collect();
        let screens: Vec<String> = {
            let mut s: Vec<String> = scored.iter().map(|x| x.0.clone()).collect();
            s.sort();
            s.dedup();
            s
        };
        let rate = |idx: &[usize]| -> f64 {
            let (mut a, mut n) = (0.0, 0.0);
            for &i in idx {
                for x in scored.iter().filter(|x| x.0 == screens[i]) {
                    a += x.2;
                    n += 1.0;
                }
            }
            if n > 0.0 { a / n } else { f64::NAN }
        };
        let all: Vec<usize> = (0..screens.len()).collect();
        let point = rate(&all);
        let mut rng = Rng::new(20_261_030);
        let bs: Vec<f64> = (0..10_000)
            .map(|_| {
                let idx: Vec<usize> = (0..screens.len())
                    .map(|_| rng.below(screens.len()))
                    .collect();
                rate(&idx)
            })
            .collect();
        let (lo, hi) = (quantile(&bs, 0.025), quantile(&bs, 0.975));
        // 片段内 Kendall τ（指标 vs H3 等级）
        let mut taus = Vec::new();
        for (clip, g) in &h3 {
            let mut xs = Vec::new();
            let mut ys = Vec::new();
            for (model, gr) in g {
                if let Some(v) = val(clip, model) {
                    xs.push(if m.higher_better { v } else { -v });
                    ys.push(*gr as f64);
                }
            }
            if xs.len() >= 2
                && let Some(t) = kendall_tau_b(&xs, &ys)
            {
                taus.push(t);
            }
        }
        let tau_mean = crate::analysis::mean(&taus);
        let mut rng = Rng::new(20_261_030);
        let tbs: Vec<f64> = (0..10_000)
            .map(|_| {
                let s: Vec<f64> = (0..taus.len())
                    .map(|_| taus[rng.below(taus.len())])
                    .collect();
                crate::analysis::mean(&s)
            })
            .collect();
        let tau_lo = if taus.is_empty() {
            f64::NAN
        } else {
            quantile(&tbs, 0.025)
        };
        let n = scored.len();
        // 若这些数据是验证集，会是哪一级（只作参考）
        let as_if = if n >= 20 && lo > 0.5 {
            if taus.len() >= 30 && tau_lo >= 0.2 {
                Level::L2
            } else {
                Level::L1
            }
        } else {
            Level::L0
        };
        let formal = if is_validation { as_if } else { Level::L0 };
        let by_src: BTreeMap<&str, (usize, f64)> = ["clearer", "natural", "h3"]
            .iter()
            .map(|s| {
                let v: Vec<f64> = scored.iter().filter(|x| x.1 == *s).map(|x| x.2).collect();
                (*s, (v.len(), crate::analysis::mean(&v)))
            })
            .collect();
        rows.push(json!({
            "verifier": m.verifier,
            "metric": m.label,
            "pairs": n,
            "screens": screens.len(),
            "agreement": point,
            "agreement_ci95": [lo, hi],
            "by_source": by_src.iter().map(|(k, (n, r))| json!({"source": k, "pairs": n, "agreement": r})).collect::<Vec<_>>(),
            "tau_within_clip_mean": tau_mean,
            "tau_lo95": tau_lo,
            "tau_clips": taus.len(),
            "level_if_validation": as_if.as_str(),
            "level": formal.as_str(),
            "why": if is_validation {
                "验证轮：按 012 §2.1 判级".to_string()
            } else {
                format!("第 {round} 轮是定标轮（奇数轮），验证集为空 → 正式级别 L0（012 §2.3 第 1 条）")
            },
        }));
    }
    json!({
        "round": round,
        "set": if is_validation { "validation" } else { "calibration" },
        "session_discounted": discounted,
        "pairs_total": pairs.len(),
        "screens": h3.len(),
        "rules": "配对：最清楚 / 最自然被选者对同屏其余各一对 + H3 同屏差 ≥ 1 级；按屏聚类 bootstrap 10000 次（种子 20261030）；L1：配对 ≥ 20 且下界 > 0.5；L2：再加片段 ≥ 30 且片段内 τ 下界 ≥ 0.2〔建议值，012 拍板 #8〕",
        "metrics": rows,
    })
}

pub fn to_markdown(c: &Value, src: &str) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# 定标 · 第 {} 轮（{}）\n",
        c["round"],
        c["set"].as_str().unwrap_or("")
    );
    let _ = writeln!(s, "数据：{src}\n");
    let _ = writeln!(
        s,
        "配对 {} 对、{} 屏；会话{}。规则：{}\n",
        c["pairs_total"],
        c["screens"],
        if c["session_discounted"] == true {
            "**打折**（陷阱 trap-identical 没过）"
        } else {
            "有效"
        },
        c["rules"].as_str().unwrap_or("")
    );
    let _ = writeln!(
        s,
        "| verifier | 指标 | 配对 | 一致率 | 95% 区间（按屏聚类） | 片段内 τ 均值 / 下界（片段数） | 若作验证集 | **正式级别** |"
    );
    let _ = writeln!(s, "|---|---|---|---|---|---|---|---|");
    for r in c["metrics"].as_array().into_iter().flatten() {
        let f = |v: &Value| v.as_f64().map_or("—".into(), |x| format!("{x:.2}"));
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | [{}, {}] | {} / {}（{}） | {} | **{}** |",
            r["verifier"].as_str().unwrap_or(""),
            r["metric"].as_str().unwrap_or(""),
            r["pairs"],
            f(&r["agreement"]),
            f(&r["agreement_ci95"][0]),
            f(&r["agreement_ci95"][1]),
            f(&r["tau_within_clip_mean"]),
            f(&r["tau_lo95"]),
            r["tau_clips"],
            r["level_if_validation"].as_str().unwrap_or(""),
            r["level"].as_str().unwrap_or("")
        );
    }
    let _ = writeln!(
        s,
        "\n一致率 0.5 = 和抛硬币一样；低于 0.5 = 指标与耳朵**反向**。片段内 τ > 0 = 同一片段里指标排序与 H3 同向。"
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tau_basic() {
        assert_eq!(kendall_tau_b(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]), Some(1.0));
        assert_eq!(
            kendall_tau_b(&[1.0, 2.0, 3.0], &[3.0, 2.0, 1.0]),
            Some(-1.0)
        );
        assert_eq!(kendall_tau_b(&[1.0, 2.0], &[1.0, 1.0]), None);
    }

    #[test]
    fn extract_pairs_from_toy_answers() {
        let a = json!({"trials": [
            {"kind": "real", "clip": "c01", "slots": {"A": "m1", "B": "m2", "C": "m3"},
             "by_model": {"clearer": "m1", "natural": "same", "usable": {"m1": "ok", "m2": "no", "m3": "no"}}},
            {"kind": "trap", "clip": "c02", "slots": {"A": "m1"}, "by_model": {"clearer": "m1"}}
        ]});
        let (p, h3) = extract(&a);
        // clearer：m1 胜 m2、m3；h3：m1 胜 m2、m3
        assert_eq!(p.len(), 4);
        assert_eq!(h3.len(), 1);
    }
}
