//! 门禁判定：冻结基线、定标级别、每个 grader 的「过 / 报警 / 阻断」（规则见 `runner.rs` 文件头）。

use super::*;

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

#[derive(Clone, Debug, PartialEq)]
pub enum Gate {
    Ok,
    /// 只报，不计入
    Info,
    Warn(String),
    Block(String),
}

pub(super) fn level_of(opts: &Opts, v: &str, synthetic: bool) -> Level {
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

pub(super) fn decide(
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
            // 按级别判阻断（模拟真值指标在真实片段上已降为 L0，不能再阻断）
            let hard = level >= Level::L2;
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

pub(super) fn fmt_val(s: &Score) -> String {
    match (s.value, s.threshold) {
        (Some(v), Some(t)) => format!("{v:.2} {}（线 {t}）", s.unit),
        (Some(v), None) => format!("{v:.2} {}", s.unit),
        _ => "—".into(),
    }
}
