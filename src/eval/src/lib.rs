//! vdn-eval：降噪的评测——verifier、任务集、定标、报告。
//!
//! 设计纲领是 Anthropic《Demystifying evals for AI agents》（逐条对照见 `docs/experiments/exp007-eval-suite/README.md`）：
//! 任务取自真实失败、正反两面都测；评分器优先确定性代码；评结果不评过程；能力集与回归集分开；
//! 多次试验报 pass@k 与 pass^k；附抽查样本（读原始记录）；防作弊探针；放进 CI。
//!
//! 两层：
//! - **CI 层**（`vdn-eval ci`）：纯 DSP、确定性、无模型权重、几分钟内跑完。被测系统是真 `vdn_engine::Engine`
//!   （三件保命）+ 本 crate 的参考降噪器（[`system::RefWiener`]，用 `vdn-dsp` 的 STFT）；本机有 GTCRN 权重时加跑
//! - **QA 层**（`vdn-eval qa`）：要本机模型（Qwen3-ASR HTTP 服务、DNSMOS ONNX），跑真实片段
//!
//! 每个 verifier 的结论只有三种：过、不过、**不知道**（[`Verdict::Unknown`]，带原因）。
//! 定标级别（L0–L3，`docs/research/012_eval-standards/verifiers.md` §2）由 `suite/levels.json` 给出，定标脚本回写。

#![forbid(unsafe_code)]

pub mod analysis;
pub mod calib;
pub mod files;
pub mod power;
pub mod probes;
pub mod qa;
pub mod report;
pub mod rng;
pub mod runner;
pub mod scene;
pub mod scorecard;
pub mod suite;
pub mod synth;
pub mod system;
pub mod thresholds;
pub mod verifiers;
pub mod wav;

use std::fmt;

/// 定标级别（012 §2.1）。物理量直接 L3；模拟真值指标只在模拟内 L3；感知量没有人耳数据一律 L0。
/// 级别表里的名字由 serde 定；`as_str` 与它的一致性由 `files_tests` 核对。
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
pub enum Level {
    L0,
    L1,
    L2,
    L3,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::L0 => "L0",
            Level::L1 => "L1",
            Level::L2 => "L2",
            Level::L3 => "L3",
        }
    }

    pub const ALL: [Level; 4] = [Level::L0, Level::L1, Level::L2, Level::L3];
}

/// 一次判定。
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Pass,
    Fail,
    /// 判不了：前提不满足、数据不够、环境不可信。原因写清楚，**不算过也不算不过**
    Unknown(String),
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::Unknown(_) => "unknown",
        }
    }
    pub fn is_pass(&self) -> bool {
        matches!(self, Verdict::Pass)
    }
    pub fn is_fail(&self) -> bool {
        matches!(self, Verdict::Fail)
    }
}

/// 数值越小越好还是越大越好。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Better {
    Lower,
    Higher,
}

impl Better {
    pub fn as_str(self) -> &'static str {
        match self {
            Better::Lower => "lower",
            Better::Higher => "higher",
        }
    }
    /// `a` 是否比 `b` 差 `delta` 以上。
    pub fn worse_by(self, a: f64, b: f64, delta: f64) -> bool {
        match self {
            Better::Lower => a > b + delta,
            Better::Higher => a < b - delta,
        }
    }
    /// `a` 是否严格比 `b` 差。
    pub fn worse(self, a: f64, b: f64) -> bool {
        self.worse_by(a, b, 0.0)
    }
}

/// 一个 verifier 在一次试验上的分数。
#[derive(Clone, Debug)]
pub struct Score {
    pub id: &'static str,
    /// 主数值；判不了时为 `None`
    pub value: Option<f64>,
    pub unit: &'static str,
    pub better: Better,
    /// 过线值（`value` 不比它差即过）
    pub threshold: Option<f64>,
    /// A–D 档（有档位锚点时）
    pub grade: Option<char>,
    pub verdict: Verdict,
    /// 旁证数值（名字 → 值）
    pub extra: Vec<(&'static str, f64)>,
    pub note: String,
}

impl Score {
    /// 按阈值与档位边界判。`edges` 是 A/B/C 三档的边界（`Lower` 时升序，`Higher` 时降序），可为空。
    pub fn judged(
        id: &'static str,
        value: f64,
        unit: &'static str,
        better: Better,
        threshold: Option<f64>,
        edges: &[f64],
    ) -> Score {
        if !value.is_finite() {
            return Score::unknown(id, unit, better, format!("数值不是有限数（{value}）"));
        }
        let verdict = match threshold {
            Some(t) if better.worse(value, t) => Verdict::Fail,
            Some(_) => Verdict::Pass,
            None => Verdict::Unknown("本任务没给阈值，只记数".into()),
        };
        let grade = if edges.is_empty() {
            None
        } else {
            let pos = edges
                .iter()
                .position(|&e| !better.worse(value, e))
                .unwrap_or(edges.len());
            ['A', 'B', 'C', 'D'].get(pos).copied()
        };
        Score {
            id,
            value: Some(value),
            unit,
            better,
            threshold,
            grade,
            verdict,
            extra: Vec::new(),
            note: String::new(),
        }
    }

    pub fn unknown(
        id: &'static str,
        unit: &'static str,
        better: Better,
        reason: impl Into<String>,
    ) -> Score {
        Score {
            id,
            value: None,
            unit,
            better,
            threshold: None,
            grade: None,
            verdict: Verdict::Unknown(reason.into()),
            extra: Vec::new(),
            note: String::new(),
        }
    }

    pub fn with_extra(mut self, name: &'static str, v: f64) -> Score {
        self.extra.push((name, v));
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Score {
        self.note = note.into();
        self
    }

    /// 把判定改成「不知道」（例如 CI 虚拟机计时不可信时），数值保留。
    pub fn demote_unknown(mut self, reason: impl Into<String>) -> Score {
        self.verdict = Verdict::Unknown(reason.into());
        self
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Some(v) => write!(
                f,
                "{} = {:.3} {} [{}]",
                self.id,
                v,
                self.unit,
                self.verdict.as_str()
            ),
            None => write!(f, "{} = ? [{}]", self.id, self.verdict.as_str()),
        }
    }
}

/// 10·log10，带下限，避免 −inf。
pub fn db10(p: f64) -> f64 {
    10.0 * p.max(1e-20).log10()
}
