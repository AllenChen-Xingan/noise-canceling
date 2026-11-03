//! 任务集清单（`suite/ci.json`、`suite/qa.json`）与定标级别表（`suite/levels.json`）的读取。
//!
//! 清单与参数进仓、音频不进仓：CI 任务全是合成场景（参数 + 种子即全部）；QA 任务引用仓外音频，按 sha256 核对。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vdn_engine::FaultKind;

use crate::Level;
use crate::files::{
    FaultFile, FaultFileKind, GraderFile, Lang, LevelsFile, Polarity, SceneFile, SuiteFile,
    serde_name,
};
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

/// 清单里的名字由 serde 定（小写）；`as_str` 与它的一致性由测试核对。
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Set {
    /// 回归集：已经做好的不许退步，应接近 100% 通过
    Regression,
    /// 能力集：攻坚场景，起点通过率低，给爬坡空间；不阻断 CI
    Capability,
    /// 防作弊探针：verifier 的自测
    Probe,
}

impl Set {
    pub const ALL: [Set; 3] = [Set::Regression, Set::Capability, Set::Probe];
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

/// 真实片段的来源（QA）。清单里写成 `{"kind": "exp005", …}`，格式由 serde 定。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
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

fn pair(v: Option<[f64; 2]>, d: (f64, f64)) -> (f64, f64) {
    v.map_or(d, |[a, b]| (a, b))
}

fn scene(v: SceneFile, dur_default: f64) -> SceneSpec {
    let owner = v.owner.map(|o| {
        let (f0_lo, f0_hi) = pair(o.f0, (100.0, 140.0));
        TalkerSpec {
            level_dbov: o.level_dbov.unwrap_or(-26.0),
            f0_lo,
            f0_hi,
            tonal: o.tonal.unwrap_or(false),
        }
    });
    let interferers = v
        .interferers
        .into_iter()
        .map(|i| {
            let (f0_lo, f0_hi) = pair(i.f0, (180.0, 230.0));
            InterfSpec {
                mode: i.mode.unwrap_or(InterfMode::Alternate),
                sir_db: i.sir_db.unwrap_or(6.0),
                f0_lo,
                f0_hi,
            }
        })
        .collect();
    SceneSpec {
        dur_s: v.dur_s.unwrap_or(dur_default),
        seed: 0,
        owner,
        owner_start_s: v.owner_start_s.unwrap_or(0.5),
        pause_s: pair(v.pause_s, (0.4, 1.2)),
        interferers,
        noise: v.noise.unwrap_or(NoiseKind::Pink),
        snr_db: v.snr_db.unwrap_or(10.0),
        tail_silence_s: v.tail_silence_s.unwrap_or(0.0),
    }
}

fn fault(f: &FaultFile) -> (u64, FaultKind) {
    let kind = match f.kind {
        FaultFileKind::Nan => FaultKind::Nan,
        FaultFileKind::Error => FaultKind::Error,
        FaultFileKind::Explode => FaultKind::Explode,
    };
    (f.hop, kind)
}

fn graders(id: &str, list: Vec<GraderFile>, info_only: bool) -> Result<Vec<Grader>, String> {
    list.into_iter()
        .map(|g| {
            if info(&g.v).is_none() {
                return Err(format!("{id}：没有登记的 verifier {}", g.v));
            }
            let params = g.params.into_iter().map(|(k, v)| (k, Value::from(v))).collect();
            Ok(Grader {
                v: g.v,
                params: Params(params),
                info_only,
            })
        })
        .collect()
}

/// 读任务清单。格式正本是 `files::SuiteFile`（未知的键报错）；缺省值在这里补；跨任务的规则（id 唯一、探针的基础任务存在）也在这里查。
pub fn parse_suite(text: &str) -> Result<Suite, String> {
    let root: SuiteFile = serde_json::from_str(text).map_err(|e| format!("清单格式不对：{e}"))?;
    let dur_default = root.defaults.dur_s.unwrap_or(10.0);
    let trials_default = root.defaults.trials.unwrap_or(3) as usize;
    let mut tasks = Vec::new();
    for t in root.tasks {
        let id = t.id;
        let mut gs = graders(&id, t.graders, false)?;
        gs.extend(graders(&id, t.info, true)?);
        let probe = t.probe.map(|p| (p.base, p.kind));
        if t.set == Set::Probe && probe.is_none() {
            return Err(format!("{id}：probe 集的任务要写 probe"));
        }
        if probe.is_none() && t.scene.is_none() && t.real.is_none() {
            return Err(format!("{id}：既没有 scene 也没有 real"));
        }
        tasks.push(Task {
            set: t.set,
            polarity: serde_name(&t.polarity.unwrap_or(Polarity::Both)),
            origin: t.origin.unwrap_or_default(),
            desc: t.desc.unwrap_or_default(),
            scene: t.scene.map(|sc| scene(sc, dur_default)),
            real: t.real,
            lang: serde_name(&t.lang.unwrap_or(Lang::En)),
            safety: t.safety.unwrap_or(true),
            faults: t.faults.iter().map(fault).collect(),
            graders: gs,
            trials: t.trials.map_or(trials_default, |v| v as usize),
            probe,
            xfail: t.xfail,
            id,
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
        name: root.name.unwrap_or_else(|| "suite".into()),
        base_seed: root.seed.unwrap_or(20_261_031),
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

/// 定标级别表：verifier id → (级别, 依据)。格式正本是 `files::LevelsFile`。
pub fn parse_levels(text: &str) -> Result<BTreeMap<String, (Level, String)>, String> {
    let f: LevelsFile = serde_json::from_str(text).map_err(|e| format!("levels.json：{e}"))?;
    Ok(f
        .verifiers
        .into_iter()
        .map(|(k, e)| (k, (e.level, e.basis)))
        .collect())
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
