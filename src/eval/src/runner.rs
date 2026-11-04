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

mod gate;
mod real;
pub use gate::{Baseline, Gate, baseline_delta, baseline_key};
use gate::{decide, level_of};
pub use real::load_real;

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

pub struct Opts {
    pub ci: bool,
    pub audio_dir: Option<PathBuf>,
    pub baseline: Baseline,
    pub levels: BTreeMap<String, (Level, String)>,
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

fn scene_clip(task: &Task, seed: u64) -> Option<Clip> {
    task.scene.as_ref().map(|s| {
        let mut s = s.clone();
        s.seed = seed;
        render(&s)
    })
}

pub(crate) struct Prepared {
    pub(crate) clip: Clip,
    pub(crate) run: RunOut,
    pub(crate) noise_out: Option<Vec<f32>>,
    pub(crate) masks: Masks,
}

fn prepare(task: &Task, sys: SystemKind, clip: Clip) -> Result<Prepared, String> {
    // 只在合成场景上做「只有噪音」的对照处理：真实录音的噪音轨不是逐样本可重放的同一段输入
    prepare_clip(task, sys, clip, task.scene.is_some())
}

/// `replayable`：噪音 + 干扰轨能逐样本重放（合成场景；场景测试集里分轨相加等于混合的段）。
pub(crate) fn prepare_clip(
    task: &Task,
    sys: SystemKind,
    clip: Clip,
    replayable: bool,
) -> Result<Prepared, String> {
    let spec = SystemSpec {
        kind: sys,
        safety: task.safety,
        faults: task.faults.clone(),
    };
    let r = run(&spec, &clip.mix)?;
    let needs_noise = task.graders.iter().any(|g| g.v == "V-BGSTAB");
    let noise_out = if needs_noise && replayable {
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

pub(crate) fn score_one(
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
                gate = Gate::Warn(
                    "没有冻结基线项（任务改名 / 新 verifier？重写基线并在 PR 里说明）".into(),
                );
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
