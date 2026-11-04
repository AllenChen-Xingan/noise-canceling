//! QA 层的评分模型 verifier：V-NRMOS-SCOREQ · V-NRMOS-DISTILL。
//!
//! 主值 = 输出的分 − 带噪原声的分（Δ，越大越好）；过线 = 不比原声差（012 §2.3 第 4 条：L0 感知量只作「不比基线差」的约束）。
//! 只打分，不当训练奖励（018 结论 4）。「两个都说好才算听感过」「分歧送人耳」是成绩单层的规则（`scorecard`），不在这里。
//! 模型在子进程里跑（`vdn-eval nrmos`，一个进程只许一个 ort 环境）；带噪原声的分按（任务, 模型）缓存。

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

use super::QaExtra;
use super::nrmos::{Kind, locate};
use crate::suite::Task;
use crate::synth::Clip;
use crate::thresholds as th;
use crate::verifiers::Params;
use crate::{Better, Score};

pub(super) struct Models {
    paths: BTreeMap<&'static str, Result<PathBuf, String>>,
    mix: BTreeMap<(String, &'static str), f64>,
}

impl Models {
    pub(super) fn locate() -> Models {
        let paths = Kind::ALL
            .into_iter()
            .map(|k| {
                let r = locate(k).ok_or_else(|| {
                    format!(
                        "找不到 {} 模型（设 {} 或放到 ~/{}）",
                        k.as_str(),
                        k.env_var(),
                        k.default_rel()
                    )
                });
                (k.as_str(), r)
            })
            .collect();
        Models {
            paths,
            mix: BTreeMap::new(),
        }
    }
}

/// 起子进程给若干段打分，按顺序返回。
fn run_sub(kind: Kind, model: &PathBuf, wavs: &[PathBuf]) -> Result<Vec<f64>, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let o = std::process::Command::new(exe)
        .arg("nrmos")
        .arg(kind.as_str())
        .arg(model)
        .args(wavs)
        .output()
        .map_err(|e| format!("起不来评分模型子进程：{e}"))?;
    if !o.status.success() {
        return Err(String::from_utf8_lossy(&o.stderr).trim().to_string());
    }
    let text = String::from_utf8_lossy(&o.stdout);
    let v: Vec<f64> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str::<Value>(l)
                .ok()
                .and_then(|j| j["mos"].as_f64())
                .ok_or_else(|| format!("子进程输出读不懂：{l}"))
        })
        .collect::<Result<_, _>>()?;
    if v.len() != wavs.len() {
        return Err(format!("子进程只回了 {} / {} 段", v.len(), wavs.len()));
    }
    Ok(v)
}

/// 该 verifier 的过线值（表里的默认值，清单可覆盖）。
fn lines(kind: Kind) -> &'static th::Lines {
    match kind {
        Kind::Scoreq => &th::NRMOS_SCOREQ,
        Kind::Distill => &th::NRMOS_DISTILL,
    }
}

impl QaExtra {
    pub(super) fn nrmos_score(
        &mut self,
        kind: Kind,
        task: &Task,
        clip: &Clip,
        out: &[f32],
        p: &Params,
    ) -> Score {
        let id = kind.verifier();
        let model = match self.nrmos.paths.get(kind.as_str()) {
            Some(Ok(m)) => m.clone(),
            Some(Err(e)) => return Score::unknown(id, "Δ MOS", Better::Higher, e.clone()),
            None => return Score::unknown(id, "Δ MOS", Better::Higher, "内部错误"),
        };
        let key = (task.id.clone(), kind.as_str());
        let mut wavs = Vec::new();
        let need_mix = !self.nrmos.mix.contains_key(&key);
        let dumped = (|| -> Result<(), String> {
            if need_mix {
                wavs.push(self.dump(&format!("{}-mix", task.id), &clip.mix)?.0);
            }
            wavs.push(self.dump(&format!("{}-out", task.id), out)?.0);
            Ok(())
        })();
        if let Err(e) = dumped {
            return Score::unknown(id, "Δ MOS", Better::Higher, e);
        }
        let got = match run_sub(kind, &model, &wavs) {
            Ok(v) => v,
            Err(e) => return Score::unknown(id, "Δ MOS", Better::Higher, e),
        };
        if need_mix {
            self.nrmos.mix.insert(key.clone(), got[0]);
        }
        let mix = self.nrmos.mix[&key];
        let o = *got.last().unwrap_or(&f64::NAN);
        let l = lines(kind);
        Score::judged(
            id,
            o - mix,
            "Δ MOS",
            Better::Higher,
            Some(p.f_or("min", l.def("min"))),
            &[],
        )
        .with_extra("mos_out", o)
        .with_extra("mos_mix", mix)
        .with_note("Δ = 输出 − 带噪原声；过线 = 不比原声差（L0 约束，不当奖励）")
    }
}
