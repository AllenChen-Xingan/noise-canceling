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
#[schemars(transform = schema_defaults_suite)]
pub struct SuiteFile {
    /// 清单名
    pub name: Option<String>,
    pub doc: Option<String>,
    /// 基础种子
    pub seed: Option<u64>,
    #[serde(default)]
    pub defaults: Defaults,
    pub tasks: Vec<TaskFile>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(transform = schema_defaults_defaults)]
pub struct Defaults {
    /// 合成场景时长（秒）
    pub dur_s: Option<f64>,
    /// 每个任务的试验数
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
#[schemars(transform = schema_defaults_task)]
pub struct TaskFile {
    /// 全清单唯一。惯例 R 回归 / C 能力 / P 探针 / Q 真实片段 + 两位号 + slug
    pub id: String,
    pub set: Set,
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
    /// 合成任务 = 种子数；缺省取清单的 defaults.trials
    pub trials: Option<u64>,
    /// false = 三件保命全关
    pub safety: Option<bool>,
    #[serde(default)]
    pub faults: Vec<FaultFile>,
    pub lang: Option<Lang>,
    /// 已知失败（原因 / 代码位置 / issue）：阻断降为报警；全过时提示转正
    pub xfail: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(transform = schema_defaults_scene)]
pub struct SceneFile {
    /// 主讲人；缺省或 null = 没有主讲人
    pub owner: Option<TalkerFile>,
    /// 主讲人开口时刻（秒）
    pub owner_start_s: Option<f64>,
    /// 停顿长度范围（秒）
    pub pause_s: Option<[f64; 2]>,
    #[serde(default)]
    pub interferers: Vec<InterfFile>,
    pub noise: Option<NoiseKind>,
    pub snr_db: Option<f64>,
    /// 场景时长（秒）；缺省取清单的 defaults.dur_s
    pub dur_s: Option<f64>,
    /// 末尾静音（秒）
    pub tail_silence_s: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(transform = schema_defaults_talker)]
pub struct TalkerFile {
    /// P.56 主动语音电平（dBov）
    pub level_dbov: Option<f64>,
    /// 基频范围（Hz）
    pub f0: Option<[f64; 2]>,
    /// 声调语言式的音节内 f0 变化
    pub tonal: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(transform = schema_defaults_interf)]
pub struct InterfFile {
    pub mode: Option<InterfMode>,
    pub sir_db: Option<f64>,
    /// 基频范围（Hz）
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
    if let Some(Value::Object(v)) = schema.get_mut("properties").and_then(|p| p.get_mut("v")) {
        v.insert("enum".into(), Value::Array(ids));
    }
}

/// 缺省值的正本：`suite.rs` 补缺省值、导出 schema 的 `default` 都读这里（第 1 轮 codex #3）。
pub mod defaults {
    use super::{Lang, Polarity};
    use crate::synth::{InterfMode, NoiseKind};

    pub const NAME: &str = "suite";
    pub const SEED: u64 = 20_261_031;
    pub const DUR_S: f64 = 10.0;
    pub const TRIALS: u64 = 3;
    pub const POLARITY: Polarity = Polarity::Both;
    pub const SAFETY: bool = true;
    pub const LANG: Lang = Lang::En;
    pub const OWNER_START_S: f64 = 0.5;
    pub const PAUSE_S: [f64; 2] = [0.4, 1.2];
    pub const NOISE: NoiseKind = NoiseKind::Pink;
    pub const SNR_DB: f64 = 10.0;
    pub const TAIL_SILENCE_S: f64 = 0.0;
    pub const LEVEL_DBOV: f64 = -26.0;
    pub const OWNER_F0: [f64; 2] = [100.0, 140.0];
    pub const TONAL: bool = false;
    pub const INTERF_MODE: InterfMode = InterfMode::Alternate;
    pub const SIR_DB: f64 = 6.0;
    pub const INTERF_F0: [f64; 2] = [180.0, 230.0];
}

/// 往 schema 的 `properties.<键>` 里写 `default`。
fn put_defaults(schema: &mut schemars::Schema, pairs: &[(&str, Value)]) {
    let Some(Value::Object(props)) = schema.get_mut("properties") else {
        return;
    };
    for (k, v) in pairs {
        if let Some(Value::Object(p)) = props.get_mut(*k) {
            p.insert("default".into(), v.clone());
        }
    }
}

fn j<T: Serialize>(v: T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

fn schema_defaults_suite(s: &mut schemars::Schema) {
    use defaults as d;
    put_defaults(s, &[("name", j(d::NAME)), ("seed", j(d::SEED))]);
}

fn schema_defaults_defaults(s: &mut schemars::Schema) {
    use defaults as d;
    put_defaults(s, &[("dur_s", j(d::DUR_S)), ("trials", j(d::TRIALS))]);
}

fn schema_defaults_task(s: &mut schemars::Schema) {
    use defaults as d;
    put_defaults(
        s,
        &[
            ("polarity", j(d::POLARITY)),
            ("safety", j(d::SAFETY)),
            ("lang", j(d::LANG)),
        ],
    );
}

fn schema_defaults_scene(s: &mut schemars::Schema) {
    use defaults as d;
    put_defaults(
        s,
        &[
            ("owner_start_s", j(d::OWNER_START_S)),
            ("pause_s", j(d::PAUSE_S)),
            ("noise", j(d::NOISE)),
            ("snr_db", j(d::SNR_DB)),
            ("tail_silence_s", j(d::TAIL_SILENCE_S)),
        ],
    );
}

fn schema_defaults_talker(s: &mut schemars::Schema) {
    use defaults as d;
    put_defaults(
        s,
        &[
            ("level_dbov", j(d::LEVEL_DBOV)),
            ("f0", j(d::OWNER_F0)),
            ("tonal", j(d::TONAL)),
        ],
    );
}

fn schema_defaults_interf(s: &mut schemars::Schema) {
    use defaults as d;
    put_defaults(
        s,
        &[
            ("mode", j(d::INTERF_MODE)),
            ("sir_db", j(d::SIR_DB)),
            ("f0", j(d::INTERF_F0)),
        ],
    );
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

/// 补进级别表的新项的依据（`calib --write-levels` 写）。
pub const NEW_ENTRY_BASIS: &str =
    "新登记，未定标：取登记表的默认级别（感知量 L0、物理量 L3，012 §2.1）";

impl LevelsFile {
    /// 把 `registry`（id, 默认级别）里表上还没有的项补上，已有条目一个字不动。返回补了哪些。
    pub fn fill_missing<'a>(
        &mut self,
        registry: impl IntoIterator<Item = (&'a str, Level)>,
    ) -> Vec<String> {
        let mut added = Vec::new();
        for (id, level) in registry {
            if !self.verifiers.contains_key(id) {
                self.verifiers.insert(
                    id.to_string(),
                    LevelEntry {
                        basis: NEW_ENTRY_BASIS.to_string(),
                        level,
                    },
                );
                added.push(id.to_string());
            }
        }
        added
    }
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

impl BaselineFile {
    /// 读基线文件；格式不对（缺键、多键、值不是数）就报错。
    pub fn parse(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| format!("baseline-ci.json 格式不对：{e}"))
    }
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

    /// 补缺项：表上没有的按默认级别补上，已有条目原样不动（含依据文字）。
    #[test]
    fn fill_missing_adds_only_new_entries() {
        let mut lv: LevelsFile = serde_json::from_str(crate::suite::LEVELS).unwrap();
        let before = lv.verifiers.clone();
        let removed = lv.verifiers.remove("V-WEAK");
        assert!(removed.is_some(), "测试前提：表里有 V-WEAK");
        let reg = crate::verifiers::REGISTRY
            .iter()
            .map(|i| (i.id, i.default_level));
        let added = lv.fill_missing(reg);
        assert_eq!(added, vec!["V-WEAK".to_string()]);
        assert_eq!(lv.verifiers["V-WEAK"].level, Level::L0);
        assert_eq!(lv.verifiers["V-WEAK"].basis, NEW_ENTRY_BASIS);
        for (k, v) in &before {
            if k != "V-WEAK" {
                assert_eq!(lv.verifiers[k].basis, v.basis, "{k} 的依据被改了");
                assert_eq!(lv.verifiers[k].level, v.level, "{k} 的级别被改了");
            }
        }
        // 再补一次：什么都不加
        let reg = crate::verifiers::REGISTRY
            .iter()
            .map(|i| (i.id, i.default_level));
        assert!(lv.fill_missing(reg).is_empty());
    }

    /// `ALL` 列全了：靠穷举 match（加变体不加进 ALL，这里编译不过）。
    #[test]
    fn all_lists_are_exhaustive() {
        let n = |p: Probe| match p {
            Probe::OnsetCut
            | Probe::NonSpeechMinus40
            | Probe::Lowpass1k
            | Probe::Identity
            | Probe::Mute
            | Probe::WeakDrop30
            | Probe::WeakDrop40
            | Probe::GainWobble
            | Probe::Clicks5
            | Probe::Noise0db => 10,
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
        assert_eq!(
            (o.level_dbov, o.f0_lo, o.f0_hi, o.tonal),
            (-26.0, 100.0, 140.0, false)
        );
        assert_eq!(sc.tail_silence_s, 0.0);
        // 干扰者的缺省值（第 1 轮 codex #3：原测试没覆盖）；scene 里不写 owner = 没有主讲人
        let s = crate::suite::parse_suite(
            r#"{"tasks":[{"id":"R1","set":"regression","scene":{"interferers":[{}]}}]}"#,
        )
        .unwrap();
        let sc = s.tasks[0].scene.as_ref().unwrap();
        let i = &sc.interferers[0];
        assert_eq!(
            (i.mode, i.sir_db, i.f0_lo, i.f0_hi),
            (InterfMode::Alternate, 6.0, 180.0, 230.0)
        );
        assert!(sc.owner.is_none());
    }
}
