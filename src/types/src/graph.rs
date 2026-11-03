//! 处理图描述（D72 · 005 第三节 A「统一表示」）：一次降噪 = 模型节点 + 挂在模型里的频域级 + 模型之后按顺序的时域级。
//!
//! **图只作描述数据，运行时仍是固定的级串**（009 §6 · 010 表二最后一行）：真正跑的是编译期定死类型的
//! [`StageChain`]（`stages![a, b]`，静态分发、不分配）。这里的描述是它的第二处表示，所以必须机器比对：
//! `Pipeline::with_stages` 在构造期（非实时）调 [`match_chain`]，把级串 `describe()` 报的种类 · 顺序 · 延迟
//! 与描述逐级比，不一致返回错误。
//!
//! 正本分工（CLAUDE.md 第五节第 5 条）：
//! - **节点种类**（稳定编号 · 名字 · 版本 · 挂点 · 延迟 · 用哪些参数）：正本是本文件的 [`KINDS`] 登记表。
//!   每个 `Stage` / `SpectralStage` 实现只返回它的种类（`fn kind() -> &'static NodeKind`），名字从种类取，
//!   「级名 ↔ 种类」的对应只写在这一处
//! - **参数**：节点只引用 [`crate::desc`] 里的条目，不另抄范围；参数值不进图（值归场景参数表 / 我的配置，各有版本）
//! - **图的身份**：[`ProcessingGraph::content_id`] = 规范文本的 FNV-1a（[`ContentId`]）；会话记录头记它
//!
//! 不列 Pipeline 的固定段（重分块 · 降采样 · 升采样 · 混合）：它们不随配置变，没有可选项；
//! 改固定段是改程序本身，不是换一张图。
//!
//! 规矩：编号永不复用；改了节点行为（算法、默认用的参数、延迟）升 `version`，内容 ID 随之变；
//! `0xF000` 以上留给测试里的假级（[`TEST_ID_MIN`]），不进登记表。

use core::fmt;
use core::fmt::Write as _;

use crate::desc::{self, ParamDesc};
use crate::params::ModelId;
use crate::stage::StageChain;
use crate::version::ContentId;

/// 节点挂在哪。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hook {
    /// 模型之后、升采样之前的时域级（`Stage`，跑在模型采样率）
    Time,
    /// 模型内部「模型输出频谱 → iSTFT」之间的频域级（`vdn-polish` 的 `SpectralStage`，模型适配器调用）
    Spectral,
}

impl Hook {
    pub fn as_str(self) -> &'static str {
        match self {
            Hook::Time => "time",
            Hook::Spectral => "spectral",
        }
    }
}

/// 一种节点。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeKind {
    /// 稳定编号（永不复用；`>= TEST_ID_MIN` 只给测试）
    pub id: u16,
    /// 短名（进延迟账与决策日志）；全局唯一
    pub name: &'static str,
    /// 行为版本：改算法 / 延迟 / 参数清单就加 1
    pub version: u16,
    pub hook: Hook,
    /// 声明延迟（模型采样率样本）；与运行时 `Stage::latency_samples()` 比对
    pub latency_samples: usize,
    /// 本节点读哪些参数（引用 `desc.rs` 的正本）
    pub params: &'static [&'static ParamDesc],
    pub doc: &'static str,
}

/// 测试里假级的编号下限（`0xF000..`）；登记表里的种类一律小于它。
pub const TEST_ID_MIN: u16 = 0xF000;

/// `NodeKind` 的字段名（导出 schema 用）。穷举解构：加了字段而这里没加，编译失败。
pub const NODE_FIELDS: [&str; 7] = {
    #[allow(dead_code)]
    fn exhaustive(k: &NodeKind) {
        let NodeKind {
            id: _,
            name: _,
            version: _,
            hook: _,
            latency_samples: _,
            params: _,
            doc: _,
        } = k;
    }
    [
        "id",
        "name",
        "version",
        "hook",
        "latency_samples",
        "params",
        "doc",
    ]
};

const ENVELOPE_PARAMS: [&ParamDesc; 7] = [
    &desc::FLOOR_DB,
    &desc::OPEN_DB,
    &desc::ATTACK_S,
    &desc::RELEASE_S,
    &desc::HANGOVER_S,
    &desc::VAD_ON,
    &desc::VAD_OFF,
];

/// 快开慢关包络，开合判据用人声概率（`vdn-speaker::VadEnvelopeStage`）。
pub const VAD_ENVELOPE: NodeKind = NodeKind {
    id: 1,
    name: "vad-envelope",
    version: 1,
    hook: Hook::Time,
    latency_samples: 0,
    params: &ENVELOPE_PARAMS,
    doc: "快开慢关包络，按人声概率开合",
};

/// 快开慢关包络，开合判据用能量（`vdn-speaker::EnergyEnvelopeStage`）。
pub const ENERGY_ENVELOPE: NodeKind = NodeKind {
    id: 2,
    name: "energy-envelope",
    version: 1,
    hook: Hook::Time,
    latency_samples: 0,
    params: &[
        &desc::FLOOR_DB,
        &desc::OPEN_DB,
        &desc::ATTACK_S,
        &desc::RELEASE_S,
        &desc::HANGOVER_S,
        &desc::VAD_ON,
        &desc::VAD_OFF,
        &desc::DETECT_DB,
    ],
    doc: "快开慢关包络，按能量开合",
};

/// 固定衰减下限（`vdn-speaker::FloorStage`）。
pub const FLOOR: NodeKind = NodeKind {
    id: 3,
    name: "floor",
    version: 1,
    hook: Hook::Time,
    latency_samples: 0,
    params: &[&desc::FLOOR_DB],
    doc: "衰减下限：原声按下限混回",
};

/// 非对称门控（`vdn-speaker::AsymGate`）。
pub const ASYM_GATE: NodeKind = NodeKind {
    id: 4,
    name: "asym-gate",
    version: 1,
    hook: Hook::Time,
    latency_samples: 0,
    params: &[
        &desc::FLOOR_DB,
        &desc::GATE_ATTACK_S,
        &desc::GATE_REOPEN_S,
        &desc::GATE_RELEASE_S,
        &desc::GATE_ONSET_GAP_S,
        &desc::GATE_ONSET_JUMP_DB,
        &desc::GATE_ONSET_LEVEL_MARGIN_DB,
        &desc::GATE_LEVEL_CLOSE_HOLD_S,
        &desc::GATE_CONFIRM_N,
        &desc::GATE_LEVEL_CLOSE_DB,
        &desc::GATE_ONSET_ARM_S,
        &desc::GATE_ONSET_REFRACTORY_S,
        &desc::OPEN_DB,
    ],
    doc: "声纹门控：非主人说话时压到下限",
};

/// 修音链的时域部分：AGC（`vdn-polish::PolishChain` 的 `Stage` 实现）。
/// 与 [`POLISH_SPECTRAL`] 是同一个实例的两个挂点（`chain.rs` 注释），图里两处都要列。
pub const POLISH_AGC: NodeKind = NodeKind {
    id: 5,
    name: "polish-agc",
    version: 1,
    hook: Hook::Time,
    latency_samples: 0,
    params: &[
        &desc::POLISH_AGC,
        &desc::POLISH_AGC_TARGET_DBFS,
        &desc::POLISH_AGC_RANGE_DB,
        &desc::POLISH_RAMP_S,
    ],
    doc: "修音链：自动增益",
};

/// 修音链的频域部分（`vdn-polish::PolishChain` 的 `SpectralStage` 实现）。
pub const POLISH_SPECTRAL: NodeKind = NodeKind {
    id: 6,
    name: "polish-spectral",
    version: 1,
    hook: Hook::Spectral,
    latency_samples: 0,
    params: &[
        &desc::POLISH_SMOOTH,
        &desc::POLISH_FLOOR_DB,
        &desc::POLISH_COMFORT_DB,
        &desc::POLISH_DEMUFFLE_MAX_DB,
        &desc::POLISH_RAMP_S,
    ],
    doc: "修音链：频谱平滑 · 增益下限 · 舒适噪声 · 去发闷",
};

/// 全部节点种类（导出、查重、按编号查都走它）。加一种 = 写常量 + 加进这里。
pub const KINDS: [&NodeKind; 6] = [
    &VAD_ENVELOPE,
    &ENERGY_ENVELOPE,
    &FLOOR,
    &ASYM_GATE,
    &POLISH_AGC,
    &POLISH_SPECTRAL,
];

pub fn kind_by_id(id: u16) -> Option<&'static NodeKind> {
    KINDS.into_iter().find(|k| k.id == id)
}

/// 一张处理图。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProcessingGraph {
    pub model: ModelId,
    /// 挂在模型节点里的频域级，按调用顺序
    pub spectral: &'static [&'static NodeKind],
    /// 模型之后的时域级，按顺序（= `Pipeline` 的级串）
    pub stages: &'static [&'static NodeKind],
}

impl ProcessingGraph {
    /// 规范文本的格式号（文本本身的写法变了才加；内容 ID 随之全变）
    pub const FORMAT: u32 = 1;
    const HEADER: &'static str = "vdn-graph";

    /// 只有模型、不带任何级的图。
    pub const fn bare(model: ModelId) -> Self {
        Self {
            model,
            spectral: &[],
            stages: &[],
        }
    }

    /// 挂点对不对：`spectral` 里只能是频域级，`stages` 里只能是时域级。
    pub fn check(&self) -> Result<(), ChainMismatch> {
        for (list, hook) in [(self.spectral, Hook::Spectral), (self.stages, Hook::Time)] {
            for (index, k) in list.iter().enumerate() {
                if k.hook != hook {
                    return Err(ChainMismatch::WrongHook {
                        index,
                        stage: k.name,
                        hook: k.hook,
                    });
                }
            }
        }
        Ok(())
    }

    /// 规范文本：内容 ID 就是它的哈希。一行一个节点：`<编号> <名字> <版本> <挂点> <延迟> <参数编号,…>`。
    /// 不含 `doc`（改说明不算换图）。构造期用（会分配）。
    pub fn canonical_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "{} = {}", Self::HEADER, Self::FORMAT);
        let _ = writeln!(s, "model = {}", self.model.as_str());
        for (section, list) in [("spectral", self.spectral), ("stages", self.stages)] {
            let _ = writeln!(s, "[{section}]");
            for k in list {
                let params: Vec<String> = k.params.iter().map(|p| p.id.to_string()).collect();
                let _ = writeln!(
                    s,
                    "node = {} {} {} {} {} {}",
                    k.id,
                    k.name,
                    k.version,
                    k.hook.as_str(),
                    k.latency_samples,
                    params.join(",")
                );
            }
        }
        s
    }

    pub fn content_id(&self) -> ContentId {
        ContentId::of(self.canonical_text().as_bytes())
    }
}

/// 描述与实际级串对不上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainMismatch {
    /// 描述里有、级串里没有（级串少了）
    Missing { index: usize, expected: &'static str },
    /// 级串里有、描述里没有（级串多了）
    Extra { index: usize, actual: &'static str },
    /// 同一位置种类不同（顺序反、名字错、版本不同都落在这）
    Kind {
        index: usize,
        expected: &'static str,
        actual: &'static str,
    },
    /// 种类对，运行时延迟与描述的声明不同
    Latency {
        index: usize,
        stage: &'static str,
        declared: usize,
        actual: usize,
    },
    /// 时域位置放了频域级，或反过来
    WrongHook {
        index: usize,
        stage: &'static str,
        hook: Hook,
    },
}

impl fmt::Display for ChainMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainMismatch::Missing { index, expected } => {
                write!(f, "第 {index} 级：描述里是 {expected}，级串里没有这一级")
            }
            ChainMismatch::Extra { index, actual } => {
                write!(f, "第 {index} 级：级串里多出 {actual}，描述里没有")
            }
            ChainMismatch::Kind {
                index,
                expected,
                actual,
            } => write!(
                f,
                "第 {index} 级：描述里是 {expected}，级串里是 {actual}（种类 · 名字 · 版本须全同）"
            ),
            ChainMismatch::Latency {
                index,
                stage,
                declared,
                actual,
            } => write!(
                f,
                "第 {index} 级 {stage}：描述声明延迟 {declared} 个样本，运行时报 {actual} 个"
            ),
            ChainMismatch::WrongHook { index, stage, hook } => write!(
                f,
                "第 {index} 级 {stage} 是 {} 挂点的节点，放错了位置",
                hook.as_str()
            ),
        }
    }
}

impl std::error::Error for ChainMismatch {}

/// 把实际级串与描述逐级比对：级数 · 每级种类（编号、名字、版本、挂点、参数清单全同）· 每级延迟。
/// 构造期用（非实时）；返回第一处不一致。
pub fn match_chain<C: StageChain + ?Sized>(
    expected: &[&'static NodeKind],
    chain: &C,
) -> Result<(), ChainMismatch> {
    let mut err = None;
    let mut index = 0;
    chain.describe(&mut |actual, latency| {
        if err.is_some() {
            return;
        }
        err = match expected.get(index) {
            None => Some(ChainMismatch::Extra {
                index,
                actual: actual.name,
            }),
            Some(&want) if want.hook != Hook::Time => Some(ChainMismatch::WrongHook {
                index,
                stage: want.name,
                hook: want.hook,
            }),
            Some(&want) if want != actual => Some(ChainMismatch::Kind {
                index,
                expected: want.name,
                actual: actual.name,
            }),
            Some(&want) if want.latency_samples != latency => Some(ChainMismatch::Latency {
                index,
                stage: want.name,
                declared: want.latency_samples,
                actual: latency,
            }),
            Some(_) => None,
        };
        index += 1;
    });
    if let Some(e) = err {
        return Err(e);
    }
    match expected.get(index) {
        Some(want) => Err(ChainMismatch::Missing {
            index,
            expected: want.name,
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
