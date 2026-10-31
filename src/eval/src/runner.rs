//! 评测编排：任务 × 系统 × 试验 → 分数 → 门禁判定。
//!
//! 门禁规则（012 §2.1「能拿来做什么」+ Anthropic「能力集与回归集分开」）：
//! - **回归集**：L3（物理 / 模拟真值）不过 → 阻断；≥ L2 的感知量不过 → 阻断；L0/L1 感知量不过线 → 只报警，
//!   但**比冻结基线差 δ 以上 → 阻断**（012：L0 只能作「不许比基线差」的约束）
//! - **能力集**：从不阻断，只报通过率（爬坡用）
//! - **探针**：劣化版必须被判「不过」且比原输出差，否则阻断（verifier 被骗过 = verifier 坏了）
//! - 「不知道」不算过也不算不过：回归集上的 L3 判不了 → 报警

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

use crate::analysis::{Masks, masks, truth_from_clean};
use crate::probes::Probe;
use crate::suite::{Grader, RealSource, Set, Suite, Task, resolve, trial_seed};
use crate::synth::{Clip, SR, Truth, render};
use crate::system::{RunOut, SystemKind, SystemSpec, run};
use crate::verifiers::{Ctx, Kind, Params, eval, info};
use crate::{Level, Score, Verdict, wav};

/// QA 层的额外评分器（ASR、DNSMOS）。CI 层传 `None`。
pub trait Extra {
    fn score(
        &mut self,
        v: &str,
        sys: &str,
        task: &Task,
        clip: &Clip,
        out: &[f32],
        p: &Params,
    ) -> Option<Score>;
}

/// 冻结基线：(系统, 任务, 试验, verifier) → 数值。
pub type Baseline = BTreeMap<String, f64>;

pub fn baseline_key(system: &str, task: &str, trial: usize, v: &str) -> String {
    format!("{system}/{task}/{trial}/{v}")
}

/// 「比基线差」的容差，按单位〔建议值〕。
pub fn baseline_delta(unit: &str) -> f64 {
    match unit {
        "dB" => 1.0,
        "×" => 0.2,
        "比例" => 0.02,
        "ms" => 8.0,
        "MOS" => 0.1,
        "百分点" => 1.0,
        _ => 0.0,
    }
}

pub struct Opts {
    pub ci: bool,
    pub audio_dir: Option<PathBuf>,
    pub baseline: Baseline,
    pub levels: BTreeMap<String, (Level, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Gate {
    Ok,
    /// 只报，不计入
    Info,
    Warn(String),
    Block(String),
}

#[derive(Clone, Debug)]
pub struct GraderResult {
    pub v: String,
    pub info_only: bool,
    pub level: Level,
    pub score: Score,
    /// 探针：未劣化的原输出上的分数
    pub base: Option<Score>,
    pub baseline: Option<f64>,
    pub gate: Gate,
}

#[derive(Clone, Debug)]
pub struct TrialResult {
    pub trial: usize,
    pub seed: u64,
    pub graders: Vec<GraderResult>,
    /// 所有非「只报」grader 都过（「不知道」不计）；全都判不了时为 `None`
    pub pass: Option<bool>,
    pub samples: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct TaskResult {
    pub id: String,
    pub set: Set,
    pub polarity: String,
    pub origin: String,
    pub desc: String,
    pub system: SystemKind,
    pub trials: Vec<TrialResult>,
    pub skipped: Option<String>,
    /// 已知失败的原因（任务清单 `xfail`）
    pub xfail: Option<String>,
}

impl TaskResult {
    pub fn k(&self) -> usize {
        self.trials.iter().filter(|t| t.pass.is_some()).count()
    }
    pub fn passes(&self) -> usize {
        self.trials.iter().filter(|t| t.pass == Some(true)).count()
    }
    /// pass@k：k 次里至少过一次
    pub fn pass_at_k(&self) -> Option<bool> {
        (self.k() > 0).then(|| self.passes() > 0)
    }
    /// pass^k：k 次全过
    pub fn pass_pow_k(&self) -> Option<bool> {
        (self.k() > 0).then(|| self.passes() == self.k())
    }
    /// 已知失败的任务全部试验都过了：可以转正（去掉 xfail）。
    pub fn xfail_now_passes(&self) -> bool {
        self.xfail.is_some() && self.pass_pow_k() == Some(true)
    }
    pub fn blocks(&self) -> Vec<String> {
        let mut out = Vec::new();
        for t in &self.trials {
            for g in &t.graders {
                if let Gate::Block(why) = &g.gate {
                    out.push(format!(
                        "{} [{}] 试验 {}：{} — {why}",
                        self.id,
                        self.system.as_str(),
                        t.trial,
                        g.v
                    ));
                }
            }
        }
        out
    }
}

pub struct SuiteResult {
    pub suite: String,
    pub layer: &'static str,
    pub tasks: Vec<TaskResult>,
    pub systems: Vec<SystemKind>,
    pub skipped_systems: Vec<(SystemKind, String)>,
}

impl SuiteResult {
    pub fn blocks(&self) -> Vec<String> {
        self.tasks.iter().flat_map(TaskResult::blocks).collect()
    }
    pub fn gate_ok(&self) -> bool {
        self.blocks().is_empty()
    }
}

fn level_of(opts: &Opts, v: &str, synthetic: bool) -> Level {
    let l = opts
        .levels
        .get(v)
        .map(|e| e.0)
        .or_else(|| info(v).map(|i| i.default_level))
        .unwrap_or(Level::L0);
    // 模拟真值指标只在模拟内 L3（012 §3.4 限定 1）
    if !synthetic && info(v).is_some_and(|i| i.kind == Kind::Sim) {
        Level::L0
    } else {
        l
    }
}

fn decide(
    set: Set,
    g: &Grader,
    level: Level,
    kind: Kind,
    score: &Score,
    base: Option<&Score>,
    baseline: Option<f64>,
) -> Gate {
    if g.info_only {
        return Gate::Info;
    }
    match set {
        Set::Capability => Gate::Ok,
        Set::Probe => {
            let Some(b) = base else {
                return Gate::Block("探针缺原输出分数".into());
            };
            match (score.value, b.value) {
                (Some(v), Some(bv)) if score.verdict.is_fail() && score.better.worse(v, bv) => {
                    Gate::Ok
                }
                (Some(v), Some(bv)) => Gate::Block(format!(
                    "劣化版没被抓到：劣化 {v:.2} [{}] vs 原输出 {bv:.2} [{}]",
                    score.verdict.as_str(),
                    b.verdict.as_str()
                )),
                _ => Gate::Block(format!(
                    "探针判不了（劣化 {} / 原输出 {}）",
                    score.verdict.as_str(),
                    b.verdict.as_str()
                )),
            }
        }
        Set::Regression => {
            let hard = kind != Kind::Perceptual || level >= Level::L2;
            if let (Some(b), Some(v)) = (baseline, score.value)
                && kind == Kind::Perceptual
                && score.better.worse_by(v, b, baseline_delta(score.unit))
            {
                return Gate::Block(format!(
                    "比冻结基线差：{v:.2} vs 基线 {b:.2}（容差 {} {}）",
                    baseline_delta(score.unit),
                    score.unit
                ));
            }
            match (&score.verdict, hard) {
                (Verdict::Fail, true) => {
                    Gate::Block(format!("{} 级不过线：{}", level.as_str(), fmt_val(score)))
                }
                (Verdict::Fail, false) => {
                    Gate::Warn(format!("{} 级只报警：{}", level.as_str(), fmt_val(score)))
                }
                (Verdict::Unknown(why), true) => Gate::Warn(format!("判不了：{why}")),
                _ => Gate::Ok,
            }
        }
    }
}

fn fmt_val(s: &Score) -> String {
    match (s.value, s.threshold) {
        (Some(v), Some(t)) => format!("{v:.2} {}（线 {t}）", s.unit),
        (Some(v), None) => format!("{v:.2} {}", s.unit),
        _ => "—".into(),
    }
}

/// 读一个真实片段（QA）。
pub fn load_real(src: &RealSource) -> Result<Clip, String> {
    let check = |p: &std::path::Path, want: Option<&str>| -> Result<(), String> {
        if let Some(w) = want {
            let got = wav::sha256(p)?;
            if got != w {
                return Err(format!(
                    "{} 的 sha256 对不上（清单 {w}，实际 {got}）",
                    p.display()
                ));
            }
        }
        Ok(())
    };
    let load = |p: &std::path::Path| -> Result<Vec<f32>, String> {
        let (x, sr) = wav::read(p)?;
        if sr != SR {
            return Err(format!("{} 采样率 {sr}，只支持 16 kHz", p.display()));
        }
        Ok(x)
    };
    match src {
        RealSource::Exp005 {
            clips,
            audio_dir,
            clip,
        } => {
            let cp = resolve(clips).ok_or_else(|| format!("找不到 {clips}（exp005 未合并？）"))?;
            let dir = resolve(audio_dir).ok_or_else(|| format!("找不到音频目录 {audio_dir}"))?;
            let j: Value =
                serde_json::from_str(&std::fs::read_to_string(&cp).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let c = j["clips"]
                .as_array()
                .and_then(|a| a.iter().find(|c| c["id"] == clip.as_str()))
                .ok_or_else(|| format!("{clips} 里没有 {clip}"))?;
            let mut tr = BTreeMap::new();
            for k in ["mix", "owner", "interf", "noise"] {
                let f = &c["files"][k];
                let p = dir.join(f["file"].as_str().ok_or("缺文件名")?);
                check(&p, f["sha256"].as_str())?;
                tr.insert(k, load(&p)?);
            }
            let n = tr.values().map(Vec::len).min().unwrap_or(0);
            let take = |k: &str| tr.get(k).map(|v| v[..n].to_vec()).unwrap_or_default();
            let (mix, clean, interf, noise) =
                (take("mix"), take("owner"), take("interf"), take("noise"));
            Ok(Clip {
                sr: SR,
                truth: Truth {
                    owner: truth_from_clean(&clean),
                    interf: truth_from_clean(&interf),
                },
                clean,
                interf,
                noise,
                mix,
                tracks: true,
            })
        }
        RealSource::Exp003 { clips, clip } => {
            let cp = resolve(clips).ok_or_else(|| format!("找不到 {clips}"))?;
            let j: Value =
                serde_json::from_str(&std::fs::read_to_string(&cp).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let c = j["clips"]
                .as_array()
                .and_then(|a| a.iter().find(|c| c["id"] == clip.as_str()))
                .ok_or_else(|| format!("{clips} 里没有 {clip}"))?;
            let get = |k: &str, h: &str| -> Result<Vec<f32>, String> {
                let rel = c[k]
                    .as_str()
                    .ok_or_else(|| format!("{clip} 缺 {k}（不是现成片段）"))?;
                let p = resolve(rel).ok_or_else(|| format!("找不到 {rel}"))?;
                check(&p, c[h].as_str())?;
                load(&p)
            };
            let mix = get("noisy_src", "noisy_sha256")?;
            let clean = get("clean_src", "clean_sha256")?;
            let n = mix.len().min(clean.len());
            let (mix, clean) = (mix[..n].to_vec(), clean[..n].to_vec());
            let noise: Vec<f32> = mix.iter().zip(&clean).map(|(a, b)| a - b).collect();
            Ok(Clip {
                sr: SR,
                truth: Truth {
                    owner: truth_from_clean(&clean),
                    interf: Vec::new(),
                },
                interf: vec![0.0; n],
                clean,
                noise,
                mix,
                tracks: false,
            })
        }
    }
}

fn scene_clip(task: &Task, seed: u64) -> Option<Clip> {
    task.scene.as_ref().map(|s| {
        let mut s = s.clone();
        s.seed = seed;
        render(&s)
    })
}

struct Prepared {
    clip: Clip,
    run: RunOut,
    noise_out: Option<Vec<f32>>,
    masks: Masks,
}

fn prepare(task: &Task, sys: SystemKind, clip: Clip) -> Result<Prepared, String> {
    let spec = SystemSpec {
        kind: sys,
        safety: task.safety,
        faults: task.faults.clone(),
    };
    let r = run(&spec, &clip.mix)?;
    let needs_noise = task.graders.iter().any(|g| g.v == "V-BGSTAB");
    // 只在合成场景上做「只有噪音」的对照处理：真实录音的噪音轨不是逐样本可重放的同一段输入
    let noise_out = if needs_noise && task.scene.is_some() {
        let bg: Vec<f32> = clip
            .noise
            .iter()
            .zip(&clip.interf)
            .map(|(a, b)| a + b)
            .collect();
        let spec = SystemSpec {
            faults: Vec::new(),
            ..spec
        };
        Some(run(&spec, &bg)?.out)
    } else {
        None
    };
    let m = masks(&clip.truth, &clip.mix, 0.03);
    Ok(Prepared {
        clip,
        run: r,
        noise_out,
        masks: m,
    })
}

fn score_one(
    g: &Grader,
    sys: SystemKind,
    task: &Task,
    p: &Prepared,
    out: &[f32],
    opts: &Opts,
    extra: &mut Option<&mut dyn Extra>,
) -> Score {
    let layer = info(&g.v).map_or("ci", |i| i.layer);
    if layer == "qa" {
        if let Some(x) = extra.as_deref_mut()
            && let Some(s) = x.score(&g.v, sys.as_str(), task, &p.clip, out, &g.params)
        {
            return s;
        }
        let i = info(&g.v);
        return Score::unknown(
            i.map_or("V-?", |i| i.id),
            i.map_or("", |i| i.unit),
            i.map_or(crate::Better::Lower, |i| i.better),
            "QA 层 verifier，本次没有接模型（用 vdn-eval qa）",
        );
    }
    let ctx = Ctx {
        clip: &p.clip,
        out,
        run: Some(&p.run),
        masks: &p.masks,
        noise_out: p.noise_out.as_deref(),
        ci: opts.ci,
    };
    eval(&g.v, &ctx, &g.params)
}

fn write_samples(
    opts: &Opts,
    sys: SystemKind,
    task: &Task,
    trial: usize,
    p: &Prepared,
    degraded: Option<&[f32]>,
) -> Vec<PathBuf> {
    let Some(dir) = &opts.audio_dir else {
        return Vec::new();
    };
    let base = dir.join(sys.as_str()).join(format!("{}-t{trial}", task.id));
    let mut out = Vec::new();
    let mut put = |name: &str, x: &[f32]| {
        let path = base.with_file_name(format!("{}-t{trial}-{name}.wav", task.id));
        if wav::write(&path, x, SR).is_ok() {
            out.push(path);
        }
    };
    put("mix", &p.clip.mix);
    put("out", &p.run.out);
    put("clean", &p.clip.clean);
    if let Some(d) = degraded {
        put("probe", d);
    }
    out
}

/// 跑一个任务在一个系统上的全部试验。
pub fn run_task(
    suite: &Suite,
    task: &Task,
    sys: SystemKind,
    opts: &Opts,
    mut extra: Option<&mut dyn Extra>,
    sample_this: bool,
) -> TaskResult {
    let mut res = TaskResult {
        id: task.id.clone(),
        set: task.set,
        polarity: task.polarity.clone(),
        origin: task.origin.clone(),
        desc: task.desc.clone(),
        system: sys,
        trials: Vec::new(),
        skipped: None,
        xfail: task.xfail.clone(),
    };
    // 探针借用基础任务的场景
    let (scene_task, probe): (&Task, Option<Probe>) = match &task.probe {
        Some((b, pr)) => match suite.tasks.iter().find(|t| &t.id == b) {
            Some(bt) => (bt, Some(*pr)),
            None => {
                res.skipped = Some(format!("基础任务 {b} 不存在"));
                return res;
            }
        },
        None => (task, None),
    };
    let trials = if task.real.is_some() {
        1
    } else {
        task.trials.max(1)
    };
    for i in 0..trials {
        let seed = trial_seed(suite.base_seed, &scene_task.id, i);
        let clip = match &task.real {
            Some(src) => match load_real(src) {
                Ok(c) => c,
                Err(e) => {
                    res.skipped = Some(e);
                    return res;
                }
            },
            None => match scene_clip(scene_task, seed) {
                Some(c) => c,
                None => {
                    res.skipped = Some("没有场景".into());
                    return res;
                }
            },
        };
        let prep = match prepare(scene_task, sys, clip) {
            Ok(p) => p,
            Err(e) => {
                res.skipped = Some(e);
                return res;
            }
        };
        let degraded = probe.map(|pr| pr.apply(&prep.clip, &prep.run.out));
        let out: &[f32] = degraded.as_deref().unwrap_or(&prep.run.out);
        let synthetic = task.real.is_none();
        let mut graders = Vec::new();
        for g in &task.graders {
            let level = level_of(opts, &g.v, synthetic);
            let kind = info(&g.v).map_or(Kind::Perceptual, |i| i.kind);
            let score = score_one(g, sys, task, &prep, out, opts, &mut extra);
            let base = degraded
                .is_some()
                .then(|| score_one(g, sys, task, &prep, &prep.run.out, opts, &mut extra));
            let baseline = opts
                .baseline
                .get(&baseline_key(sys.as_str(), &task.id, i, &g.v))
                .copied();
            let mut gate = decide(task.set, g, level, kind, &score, base.as_ref(), baseline);
            // 回归集的感知量没有基线项（任务改名、新 verifier、改了试验次数）→ 报警，提示重写基线
            if gate == Gate::Ok
                && task.set == Set::Regression
                && kind == Kind::Perceptual
                && !g.info_only
                && baseline.is_none()
                && !opts.baseline.is_empty()
                && score.value.is_some()
            {
                gate = Gate::Warn("没有冻结基线项（任务改名 / 新 verifier？重写基线并在 PR 里说明）".into());
            }
            // 已知失败：阻断降为报警，留给修复线
            if let (Some(x), Gate::Block(w)) = (&task.xfail, &gate) {
                gate = Gate::Warn(format!("已知失败（{x}）：{w}"));
            }
            graders.push(GraderResult {
                v: g.v.clone(),
                info_only: g.info_only,
                level,
                score,
                base,
                baseline,
                gate,
            });
        }
        let counted: Vec<&GraderResult> = graders.iter().filter(|g| !g.info_only).collect();
        let pass = if task.set == Set::Probe {
            // 探针的「过」= verifier 抓到了劣化
            Some(counted.iter().all(|g| g.gate == Gate::Ok))
        } else if counted
            .iter()
            .all(|g| matches!(g.score.verdict, Verdict::Unknown(_)))
        {
            None
        } else {
            Some(counted.iter().all(|g| !g.score.verdict.is_fail()))
        };
        let samples = if pass == Some(false) && res.trials.iter().all(|t| t.pass != Some(false))
            || (sample_this && i == 0)
        {
            write_samples(opts, sys, task, i, &prep, degraded.as_deref())
        } else {
            Vec::new()
        };
        res.trials.push(TrialResult {
            trial: i,
            seed,
            graders,
            pass,
            samples,
        });
    }
    res
}

/// 跑整个任务集。`systems` 里权重不在本机的会被跳过并记下原因。
pub fn run_suite(
    suite: &Suite,
    layer: &'static str,
    systems: &[SystemKind],
    opts: &Opts,
    mut extra: Option<&mut dyn Extra>,
    filter: Option<&str>,
) -> SuiteResult {
    let mut tasks = Vec::new();
    let mut ran = Vec::new();
    let mut skipped = Vec::new();
    for &sys in systems {
        if !sys.available() {
            skipped.push((sys, "权重不在本机".to_string()));
            continue;
        }
        ran.push(sys);
        for (k, t) in suite.tasks.iter().enumerate() {
            if filter.is_some_and(|f| !t.id.contains(f)) {
                continue;
            }
            // 抽查样本：每 5 个任务固定抽 1 个通过的（失败的一律抽）
            let sample = k % 5 == 0;
            eprintln!("  [{}] {}", sys.as_str(), t.id);
            let e: Option<&mut dyn Extra> = match &mut extra {
                Some(x) => Some(&mut **x),
                None => None,
            };
            tasks.push(run_task(suite, t, sys, opts, e, sample));
        }
    }
    SuiteResult {
        suite: suite.name.clone(),
        layer,
        tasks,
        systems: ran,
        skipped_systems: skipped,
    }
}
