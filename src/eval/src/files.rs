//! `suite/` 下 4 份 JSON 的文件格式。**这里的类型是格式正本**：
//! - 读：`suite.rs` 先反序列化成这些类型，再换成运行用的 `Task` / `Suite`（默认值在那里补）
//! - 写：`main.rs` 写基线、级别表时序列化这些类型，不再手拼 JSON
//! - schema：`vdn-schema` 从这些类型派生 `specs/schema/eval-*.schema.json`，测试逐字比对、并拿 4 份文件过 schema
//!
//! 多出的键一律报错（`deny_unknown_fields`），防拼错键名静默失效。

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Level;
use crate::probes::Probe;
use crate::suite::{RealSource, Set};
use crate::synth::{InterfMode, NoiseKind};

/// 任务清单（`ci.json`、`qa.json`）。
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SuiteFile {
    /// 清单名；缺省 "suite"
    pub name: Option<String>,
    pub doc: Option<String>,
    /// 基础种子；缺省 20261031
    pub seed: Option<u64>,
    #[serde(default)]
    pub defaults: Defaults,
    pub tasks: Vec<TaskFile>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    /// 合成场景时长（秒）；缺省 10
    pub dur_s: Option<f64>,
    /// 每个任务的试验数；缺省 3
    pub trials: Option<u64>,
}

/// 该压的 · 不该压的 · 两面 · 保命与物理量。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Polarity {
    Suppress,
    Preserve,
    Both,
    Robust,
}

/// ASR 分词用的语言。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Zh,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskFile {
    /// 全清单唯一。惯例 R 回归 / C 能力 / P 探针 / Q 真实片段 + 两位号 + slug
    pub id: String,
    pub set: Set,
    /// 缺省 both
    pub polarity: Option<Polarity>,
    /// 取自哪次真实失败（实验号 + 片段 + 人原话或标签）；测试要求非空
    pub origin: Option<String>,
    pub desc: Option<String>,
    /// 给读清单的人看的备注，程序不读
    pub note: Option<String>,
    /// 合成场景（CI）。scene · real · probe 至少一个
    pub scene: Option<SceneFile>,
    /// 仓外真实片段（QA），按 clips.json 里的 sha256 核对
    pub real: Option<RealSource>,
    /// 只在 set = probe 用
    pub probe: Option<ProbeFile>,
    #[serde(default)]
    pub graders: Vec<GraderFile>,
    /// 同 graders，但只报，不计通过与门禁
    #[serde(default)]
    pub info: Vec<GraderFile>,
    /// 合成任务 = 种子数；缺省取 defaults.trials
    pub trials: Option<u64>,
    /// false = 三件保命全关；缺省 true
    pub safety: Option<bool>,
    #[serde(default)]
    pub faults: Vec<FaultFile>,
    /// 缺省 en
    pub lang: Option<Lang>,
    /// 已知失败（原因 / 代码位置 / issue）：阻断降为报警；全过时提示转正
    pub xfail: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneFile {
    /// 主讲人；缺省或 null = 没有主讲人
    pub owner: Option<TalkerFile>,
    /// 缺省 0.5
    pub owner_start_s: Option<f64>,
    /// 停顿长度范围（秒）；缺省 [0.4, 1.2]
    pub pause_s: Option<[f64; 2]>,
    #[serde(default)]
    pub interferers: Vec<InterfFile>,
    /// 缺省 pink
    pub noise: Option<NoiseKind>,
    /// 缺省 10
    pub snr_db: Option<f64>,
    /// 缺省取 defaults.dur_s
    pub dur_s: Option<f64>,
    /// 缺省 0
    pub tail_silence_s: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TalkerFile {
    /// P.56 主动语音电平（dBov）；缺省 −26
    pub level_dbov: Option<f64>,
    /// 基频范围（Hz）；缺省 [100, 140]
    pub f0: Option<[f64; 2]>,
    /// 声调语言式的音节内 f0 变化；缺省 false
    pub tonal: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InterfFile {
    /// 缺省 alternate
    pub mode: Option<InterfMode>,
    /// 缺省 6
    pub sir_db: Option<f64>,
    /// 基频范围（Hz）；缺省 [180, 230]
    pub f0: Option<[f64; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProbeFile {
    /// 基础任务 id（必须是本清单里的合成场景任务）
    pub base: String,
    pub kind: Probe,
}

/// CI 不注入 stall：会 sleep、计时不稳。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FaultFileKind {
    Nan,
    Error,
    Explode,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FaultFile {
    pub hop: u64,
    pub kind: FaultFileKind,
}

/// 一个评分器：`v` 是 verifier id（`verifiers::REGISTRY`），其余键是它的数值参数（`max` / `min` 与各自的参数）。
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(transform = verifier_ids)]
pub struct GraderFile {
    pub v: String,
    #[serde(flatten)]
    pub params: BTreeMap<String, f64>,
}

/// schema 里把 `v` 限定为已登记的 verifier（id 只来自 `REGISTRY`）。
fn verifier_ids(schema: &mut schemars::Schema) {
    let ids: Vec<Value> = crate::verifiers::REGISTRY
        .iter()
        .map(|i| Value::from(i.id))
        .collect();
    if let Some(Value::Object(v)) = schema
        .get_mut("properties")
        .and_then(|p| p.get_mut("v"))
    {
        v.insert("enum".into(), Value::Array(ids));
    }
}

/// 定标级别表（`levels.json`）。只由 `vdn-eval calib --write-levels` 改。
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LevelsFile {
    pub doc: String,
    /// 最后一次回写的日期（YYYY-MM-DD）
    pub updated: String,
    /// verifier id → 级别与依据
    pub verifiers: BTreeMap<String, LevelEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LevelEntry {
    pub basis: String,
    pub level: Level,
}

/// CI 层冻结基线（`baseline-ci.json`）。只由 `vdn-eval ci --write-baseline` 写。
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BaselineFile {
    /// 写基线时的提交（可带 -dirty）
    pub commit: String,
    pub doc: String,
    pub run_id: String,
    /// 键：系统/任务 id/试验号/verifier id
    pub values: BTreeMap<String, f64>,
}

/// serde 给枚举变体写的名字（运行时要字符串的地方用它，不另写一份名字表）。
pub fn serde_name<T: Serialize>(v: &T) -> String {
    match serde_json::to_value(v) {
        Ok(Value::String(s)) => s,
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `as_str` 是名字的第二处表示：逐个变体与 serde 的名字比对。
    #[test]
    fn as_str_matches_serde_names() {
        for p in Probe::ALL {
            assert_eq!(p.as_str(), serde_name(&p));
        }
        for l in Level::ALL {
            assert_eq!(l.as_str(), serde_name(&l));
        }
        for s in Set::ALL {
            assert_eq!(s.as_str(), serde_name(&s));
        }
    }

    /// `ALL` 列全了：靠穷举 match（加变体不加进 ALL，这里编译不过）。
    #[test]
    fn all_lists_are_exhaustive() {
        let n = |p: Probe| match p {
            Probe::OnsetCut
            | Probe::NonSpeechMinus40
            | Probe::Lowpass1k
            | Probe::Identity
            | Probe::Mute => 5,
        };
        assert_eq!(Probe::ALL.len(), n(Probe::Mute));
        let n = |l: Level| match l {
            Level::L0 | Level::L1 | Level::L2 | Level::L3 => 4,
        };
        assert_eq!(Level::ALL.len(), n(Level::L0));
        let n = |s: Set| match s {
            Set::Regression | Set::Capability | Set::Probe => 3,
        };
        assert_eq!(Set::ALL.len(), n(Set::Probe));
    }

    /// 缺省值照旧：只写了必填项的合成任务，补出的场景与改格式前一致。
    #[test]
    fn defaults_unchanged() {
        let s = crate::suite::parse_suite(
            r#"{"tasks":[{"id":"R1","set":"regression","scene":{"owner":{}}}]}"#,
        )
        .unwrap();
        assert_eq!(s.name, "suite");
        assert_eq!(s.base_seed, 20_261_031);
        let t = &s.tasks[0];
        assert_eq!((t.polarity.as_str(), t.lang.as_str()), ("both", "en"));
        assert!(t.safety);
        assert_eq!(t.trials, 3);
        let sc = t.scene.as_ref().unwrap();
        assert_eq!(sc.dur_s, 10.0);
        assert_eq!(sc.owner_start_s, 0.5);
        assert_eq!(sc.pause_s, (0.4, 1.2));
        assert_eq!(sc.noise, NoiseKind::Pink);
        assert_eq!(sc.snr_db, 10.0);
        let o = sc.owner.as_ref().unwrap();
        assert_eq!((o.level_dbov, o.f0_lo, o.f0_hi, o.tonal), (-26.0, 100.0, 140.0, false));
    }
}
