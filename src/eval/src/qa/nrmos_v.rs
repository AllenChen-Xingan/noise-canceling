//! QA 层的评分模型 verifier：V-NRMOS-SCOREQ · V-NRMOS-DISTILL。
//!
//! 主值 = 输出的分 − 带噪原声的分（Δ，越大越好）；过线 = 不比原声差（012 §2.3 第 4 条：L0 感知量只作「不比基线差」的约束）。
//! 只打分，不当训练奖励（018 结论 4）。「两个都说好才算听感过」「分歧送人耳」是成绩单层的规则（`scorecard`），不在这里。
//! 模型在常驻子进程里跑（`vdn-eval nrmos <kind> <模型> --serve`，一个进程只许一个 ort 环境；模型只载一次）；
//! 带噪原声的分按（任务, 模型）缓存。

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

/// 一个常驻的评分子进程（`vdn-eval nrmos <kind> <模型> --serve`）。
struct Server {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: std::io::BufReader<std::process::ChildStdout>,
}

impl Server {
    fn spawn(kind: Kind, model: &PathBuf) -> Result<Server, String> {
        use std::process::{Command, Stdio};
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let mut child = Command::new(exe)
            .arg("nrmos")
            .arg(kind.as_str())
            .arg(model)
            .arg("--serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("起不来评分模型子进程：{e}"))?;
        let stdin = child.stdin.take().ok_or("子进程没有标准输入")?;
        let stdout = std::io::BufReader::new(child.stdout.take().ok_or("子进程没有标准输出")?);
        Ok(Server {
            child,
            stdin,
            stdout,
        })
    }

    fn score(&mut self, wav: &std::path::Path) -> Result<f64, String> {
        use std::io::{BufRead, Write};
        writeln!(self.stdin, "{}", wav.display()).map_err(|e| format!("写子进程：{e}"))?;
        self.stdin.flush().map_err(|e| e.to_string())?;
        let mut line = String::new();
        let n = self
            .stdout
            .read_line(&mut line)
            .map_err(|e| format!("读子进程：{e}"))?;
        if n == 0 {
            return Err("评分模型子进程退出了（哈希不对或模型打不开，见上面的报错）".into());
        }
        let v: Value = serde_json::from_str(&line).map_err(|e| format!("子进程输出读不懂：{e}"))?;
        if let Some(e) = v["error"].as_str() {
            return Err(e.to_string());
        }
        v["mos"]
            .as_f64()
            .ok_or_else(|| format!("子进程输出缺 mos：{line}"))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(super) struct Models {
    paths: BTreeMap<&'static str, Result<PathBuf, String>>,
    /// 带噪原声的分，按（混合音频内容哈希, 模型）缓存：片段 id 只在一个 build 内唯一，不能当键（codex 第 2 轮 P1 #1）
    mix: BTreeMap<(u64, &'static str), f64>,
    servers: BTreeMap<&'static str, Server>,
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
            servers: BTreeMap::new(),
        }
    }

    /// 用常驻子进程给若干段打分；子进程坏了就丢掉，下次重起。
    fn run(&mut self, kind: Kind, model: &PathBuf, wavs: &[PathBuf]) -> Result<Vec<f64>, String> {
        if !self.servers.contains_key(kind.as_str()) {
            self.servers
                .insert(kind.as_str(), Server::spawn(kind, model)?);
        }
        let srv = self.servers.get_mut(kind.as_str()).ok_or("内部错误")?;
        let r: Result<Vec<f64>, String> = wavs.iter().map(|w| srv.score(w)).collect();
        if r.is_err() {
            self.servers.remove(kind.as_str());
        }
        r
    }
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
        let key = (super::fnv(&clip.mix), kind.as_str());
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
        let got = match self.nrmos.run(kind, &model, &wavs) {
            Ok(v) => v,
            Err(e) => return Score::unknown(id, "Δ MOS", Better::Higher, e),
        };
        if need_mix {
            self.nrmos.mix.insert(key, got[0]);
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
