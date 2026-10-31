//! 任务集清单（`suite/ci.json`、`suite/qa.json`）与定标级别表（`suite/levels.json`）的读取。
//!
//! 清单与参数进仓、音频不进仓：CI 任务全是合成场景（参数 + 种子即全部）；QA 任务引用仓外音频，按 sha256 核对。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vdn_engine::FaultKind;

use crate::Level;
use crate::probes::Probe;
use crate::synth::{InterfMode, InterfSpec, NoiseKind, SceneSpec, TalkerSpec};
use crate::verifiers::{Params, info};

pub const CI_SUITE: &str = include_str!("../suite/ci.json");
pub const QA_SUITE: &str = include_str!("../suite/qa.json");
pub const LEVELS: &str = include_str!("../suite/levels.json");

/// 仓库根。
pub fn repo_root() -> PathBuf {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.ancestors().nth(2).map(PathBuf::from).unwrap_or(here)
}

/// 放 `resources/`、`_tmp/` 实物的主目录（worktree 里按 git 公共目录找回主目录；找不到就用仓库根）。
pub fn main_root() -> PathBuf {
    let root = repo_root();
    let out = std::process::Command::new("git")
        .args(["-C"])
        .arg(&root)
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .output();
    if let Ok(o) = out
        && o.status.success()
    {
        let common = PathBuf::from(String::from_utf8_lossy(&o.stdout).trim());
        if let Some(main) = common.parent() {
            return main.to_path_buf();
        }
    }
    root
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Set {
    /// 回归集：已经做好的不许退步，应接近 100% 通过
    Regression,
    /// 能力集：攻坚场景，起点通过率低，给爬坡空间；不阻断 CI
    Capability,
    /// 防作弊探针：verifier 的自测
    Probe,
}

impl Set {
    pub fn parse(s: &str) -> Option<Set> {
        Some(match s {
            "regression" => Set::Regression,
            "capability" => Set::Capability,
            "probe" => Set::Probe,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Set::Regression => "regression",
            Set::Capability => "capability",
            Set::Probe => "probe",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Grader {
    pub v: String,
    pub params: Params,
    /// 只报，不计入通过与门禁（例如探针里展示 V-NR 被刷）
    pub info_only: bool,
}

/// 真实片段的来源（QA）。
#[derive(Clone, Debug)]
pub enum RealSource {
    /// exp005 三轨素材：`clips` 是 clips.json 的仓内路径，`audio_dir` 是主目录下的音频目录
    Exp005 {
        clips: String,
        audio_dir: String,
        clip: String,
    },
    /// exp003 现成的 noisy + clean（`resources/` 下）
    Exp003 { clips: String, clip: String },
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub set: Set,
    /// "suppress"（该压的）· "preserve"（不该压的）· "both" · "robust"（保命 / 物理）
    pub polarity: String,
    pub origin: String,
    pub desc: String,
    pub scene: Option<SceneSpec>,
    pub real: Option<RealSource>,
    pub lang: String,
    pub safety: bool,
    pub faults: Vec<(u64, FaultKind)>,
    pub graders: Vec<Grader>,
    pub trials: usize,
    /// 探针：(基础任务 id, 劣化)
    pub probe: Option<(String, Probe)>,
    /// 已知失败（写原因 / issue）：阻断降为报警；全部试验意外通过时报「可转正」
    pub xfail: Option<String>,
}

pub struct Suite {
    pub name: String,
    pub base_seed: u64,
    pub tasks: Vec<Task>,
}

fn f(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(Value::as_f64)
}

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str)
}

fn pair(v: &Value, k: &str, d: (f64, f64)) -> (f64, f64) {
    match v.get(k).and_then(Value::as_array) {
        Some(a) if a.len() == 2 => (a[0].as_f64().unwrap_or(d.0), a[1].as_f64().unwrap_or(d.1)),
        _ => d,
    }
}

fn parse_scene(v: &Value, dur_default: f64) -> Result<SceneSpec, String> {
    let owner = match v.get("owner") {
        Some(o) if !o.is_null() => {
            let (lo, hi) = pair(o, "f0", (100.0, 140.0));
            Some(TalkerSpec {
                level_dbov: f(o, "level_dbov").unwrap_or(-26.0),
                f0_lo: lo,
                f0_hi: hi,
                tonal: o.get("tonal").and_then(Value::as_bool).unwrap_or(false),
            })
        }
        _ => None,
    };
    let mut interferers = Vec::new();
    for i in v
        .get("interferers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let mode = s(i, "mode").unwrap_or("alternate");
        let (lo, hi) = pair(i, "f0", (180.0, 230.0));
        interferers.push(InterfSpec {
            mode: InterfMode::parse(mode).ok_or_else(|| format!("未知干扰模式 {mode}"))?,
            sir_db: f(i, "sir_db").unwrap_or(6.0),
            f0_lo: lo,
            f0_hi: hi,
        });
    }
    let noise = s(v, "noise").unwrap_or("pink");
    Ok(SceneSpec {
        dur_s: f(v, "dur_s").unwrap_or(dur_default),
        seed: 0,
        owner,
        owner_start_s: f(v, "owner_start_s").unwrap_or(0.5),
        pause_s: pair(v, "pause_s", (0.4, 1.2)),
        interferers,
        noise: NoiseKind::parse(noise).ok_or_else(|| format!("未知噪音 {noise}"))?,
        snr_db: f(v, "snr_db").unwrap_or(10.0),
        tail_silence_s: f(v, "tail_silence_s").unwrap_or(0.0),
    })
}

fn parse_fault(v: &Value) -> Result<(u64, FaultKind), String> {
    let hop = v.get("hop").and_then(Value::as_u64).ok_or("故障缺 hop")?;
    let kind = match s(v, "kind") {
        Some("nan") => FaultKind::Nan,
        Some("error") => FaultKind::Error,
        Some("explode") => FaultKind::Explode,
        k => {
            return Err(format!(
                "未知故障 {k:?}（CI 不注入 stall：会 sleep、计时不稳）"
            ));
        }
    };
    Ok((hop, kind))
}

pub fn parse_suite(text: &str) -> Result<Suite, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| format!("清单不是合法 JSON：{e}"))?;
    let defaults = root.get("defaults").cloned().unwrap_or(Value::Null);
    let dur_default = f(&defaults, "dur_s").unwrap_or(10.0);
    let trials_default = defaults.get("trials").and_then(Value::as_u64).unwrap_or(3) as usize;
    let mut tasks = Vec::new();
    for t in root
        .get("tasks")
        .and_then(Value::as_array)
        .ok_or("清单缺 tasks")?
    {
        let id = s(t, "id").ok_or("任务缺 id")?.to_string();
        let set_s = s(t, "set").unwrap_or("");
        let set = Set::parse(set_s).ok_or_else(|| format!("{id}：未知 set {set_s}"))?;
        let mut graders = Vec::new();
        for (key, info_only) in [("graders", false), ("info", true)] {
            for g in t.get(key).and_then(Value::as_array).into_iter().flatten() {
                let v = s(g, "v").ok_or_else(|| format!("{id}：grader 缺 v"))?;
                if info(v).is_none() {
                    return Err(format!("{id}：没有登记的 verifier {v}"));
                }
                let mut params = g.as_object().cloned().unwrap_or_default();
                params.remove("v");
                graders.push(Grader {
                    v: v.to_string(),
                    params: Params(params),
                    info_only,
                });
            }
        }
        let scene = match t.get("scene") {
            Some(sc) => Some(parse_scene(sc, dur_default).map_err(|e| format!("{id}：{e}"))?),
            None => None,
        };
        let real = match t.get("real") {
            Some(r) => Some(match s(r, "kind") {
                Some("exp005") => RealSource::Exp005 {
                    clips: s(r, "clips").ok_or("exp005 缺 clips")?.into(),
                    audio_dir: s(r, "audio_dir").ok_or("exp005 缺 audio_dir")?.into(),
                    clip: s(r, "clip").ok_or("exp005 缺 clip")?.into(),
                },
                Some("exp003") => RealSource::Exp003 {
                    clips: s(r, "clips").ok_or("exp003 缺 clips")?.into(),
                    clip: s(r, "clip").ok_or("exp003 缺 clip")?.into(),
                },
                k => return Err(format!("{id}：未知真实来源 {k:?}")),
            }),
            None => None,
        };
        let probe = match t.get("probe") {
            Some(p) => {
                let base = s(p, "base").ok_or_else(|| format!("{id}：探针缺 base"))?;
                let kind = s(p, "kind").unwrap_or("");
                Some((
                    base.to_string(),
                    Probe::parse(kind).ok_or_else(|| format!("{id}：未知探针 {kind}"))?,
                ))
            }
            None => None,
        };
        if set == Set::Probe && probe.is_none() {
            return Err(format!("{id}：probe 集的任务要写 probe"));
        }
        if probe.is_none() && scene.is_none() && real.is_none() {
            return Err(format!("{id}：既没有 scene 也没有 real"));
        }
        let mut faults = Vec::new();
        for fv in t
            .get("faults")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            faults.push(parse_fault(fv).map_err(|e| format!("{id}：{e}"))?);
        }
        tasks.push(Task {
            id: id.clone(),
            set,
            polarity: s(t, "polarity").unwrap_or("both").into(),
            origin: s(t, "origin").unwrap_or("").into(),
            desc: s(t, "desc").unwrap_or("").into(),
            scene,
            real,
            lang: s(t, "lang").unwrap_or("en").into(),
            safety: t.get("safety").and_then(Value::as_bool).unwrap_or(true),
            faults,
            graders,
            trials: t
                .get("trials")
                .and_then(Value::as_u64)
                .map_or(trials_default, |v| v as usize),
            probe,
            xfail: s(t, "xfail").map(String::from),
        });
    }
    // 探针的基础任务必须存在
    for t in &tasks {
        if let Some((b, _)) = &t.probe
            && !tasks.iter().any(|x| &x.id == b && x.scene.is_some())
        {
            return Err(format!("{}：探针的基础任务 {b} 不存在或不是合成场景", t.id));
        }
    }
    let mut ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    ids.sort_unstable();
    let n = ids.len();
    ids.dedup();
    if ids.len() != n {
        return Err("任务 id 有重复".into());
    }
    Ok(Suite {
        name: s(&root, "name").unwrap_or("suite").into(),
        base_seed: root
            .get("seed")
            .and_then(Value::as_u64)
            .unwrap_or(20_261_031),
        tasks,
    })
}

/// 任务第 `trial` 次试验的种子（FNV-1a(id) ⊕ 基础种子 ⊕ 试验号）。
pub fn trial_seed(base: u64, id: &str, trial: usize) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^ base ^ ((trial as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15))
}

/// 定标级别表：verifier id → (级别, 依据)。
pub fn parse_levels(text: &str) -> Result<BTreeMap<String, (Level, String)>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("levels.json：{e}"))?;
    let mut out = BTreeMap::new();
    for (k, e) in v
        .get("verifiers")
        .and_then(Value::as_object)
        .ok_or("levels.json 缺 verifiers")?
    {
        let l = s(e, "level")
            .and_then(Level::parse)
            .ok_or_else(|| format!("{k}：级别不对"))?;
        out.insert(k.clone(), (l, s(e, "basis").unwrap_or("").to_string()));
    }
    Ok(out)
}

/// 解析路径：先仓库根，再主目录，再同级的 p2 worktree（exp005 未合并前）。
pub fn resolve(rel: &str) -> Option<PathBuf> {
    let p = Path::new(rel);
    if p.is_absolute() {
        return p.exists().then(|| p.to_path_buf());
    }
    let main = main_root();
    let mut cands = vec![repo_root().join(rel), main.join(rel)];
    if let Some(parent) = main.parent() {
        cands.push(parent.join("noise_canceling-p2").join(rel));
    }
    cands.into_iter().find(|c| c.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_suites_parse() {
        let ci = parse_suite(CI_SUITE).unwrap();
        let qa = parse_suite(QA_SUITE).unwrap();
        let n = ci.tasks.len() + qa.tasks.len();
        assert!(
            (20..=50).contains(&n),
            "任务数 {n}（Anthropic：起步 20–50 个）"
        );
        for set in [Set::Regression, Set::Capability, Set::Probe] {
            assert!(
                ci.tasks.iter().any(|t| t.set == set),
                "CI 缺 {}",
                set.as_str()
            );
        }
        // 正反两面：既有「该压」也有「不该压」
        assert!(ci.tasks.iter().any(|t| t.polarity == "suppress"));
        assert!(ci.tasks.iter().any(|t| t.polarity == "preserve"));
        // 每个任务都写了来源
        for t in ci.tasks.iter().chain(&qa.tasks) {
            assert!(!t.origin.is_empty(), "{} 没写 origin", t.id);
        }
    }

    #[test]
    fn levels_cover_registry() {
        let l = parse_levels(LEVELS).unwrap();
        for i in crate::verifiers::REGISTRY {
            assert!(l.contains_key(i.id), "levels.json 缺 {}", i.id);
        }
    }

    #[test]
    fn seeds_differ_by_trial_and_task() {
        assert_ne!(trial_seed(1, "a", 0), trial_seed(1, "a", 1));
        assert_ne!(trial_seed(1, "a", 0), trial_seed(1, "b", 0));
    }
}
