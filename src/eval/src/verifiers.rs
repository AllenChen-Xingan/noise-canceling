//! CI 层 verifier：纯 DSP、确定性、无模型。清单与口径来自 `docs/research/012_eval-standards/verifiers.md` §3。
//!
//! 每条在 [`REGISTRY`] 里登记：定义 · 算法 · 锚点 · 默认级别 · 已知失效。**阈值凡标〔建议值〕的都是 AI 估的，没有出处**；
//! 感知类定标前一律 L0（`suite/levels.json` 是级别正本，定标脚本回写）。
//! 任何一条在前提不满足时返回 [`Verdict::Unknown`] 并写明原因，不硬给分。

use serde_json::Value;

use crate::analysis::{
    MASK_FRAME, Masks, Spectrogram, frame_power, kurtosis, mean, quantile, truth_from_clean,
    xcorr_lag,
};
use crate::synth::Clip;
use crate::system::{BUDGET_MS, RunOut};
use crate::thresholds as th;
use crate::{Better, Level, Score, Verdict, db10};

pub mod estoi;
mod perceptual;
mod physical;
mod sim;
mod speech;
use perceptual::*;
use physical::*;
pub use sim::GATE_TOLERANCE_S;
use sim::*;
use speech::*;

/// 类别：决定置信度从哪来（012 §1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// 物理量：判事实，直接 L3
    Physical,
    /// 模拟真值：只在模拟内 L3（012 §3.4）
    Sim,
    /// 感知量：置信度只能来自人耳，定标前 L0
    Perceptual,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Physical => "physical",
            Kind::Sim => "sim",
            Kind::Perceptual => "perceptual",
        }
    }
}

/// 一条 verifier 的登记。
pub struct Info {
    pub id: &'static str,
    pub name: &'static str,
    /// "ci" 或 "qa"
    pub layer: &'static str,
    pub kind: Kind,
    pub default_level: Level,
    pub unit: &'static str,
    pub better: Better,
    pub definition: &'static str,
    pub algorithm: &'static str,
    pub anchor: &'static str,
    pub failure: &'static str,
}

pub const REGISTRY: &[Info] = &[
    Info {
        id: "V-LAT",
        name: "附加延迟",
        layer: "ci",
        kind: Kind::Physical,
        default_level: Level::L3,
        unit: "ms",
        better: Better::Lower,
        definition: "端到端附加延迟：引擎延迟账（hop 等待 + 模型算法延迟，代码声明）+ 平台预算（IO 块取提议值 96 帧）；并实测算法延迟核对声明",
        algorithm: "输入与未对齐原始输出的互相关峰（前 2 s、0–2048 样本）= 实测算法延迟；|实测 − Model::latency_samples| ≤ 1 样本且账面合计 ≤ 45 ms 才过。5 个种子各量一次（Teams §6.35 放 5 次的口径；本实现确定性，5 次一致）",
        anchor: "D32：≤ 45 ms；rubric L1 / V2",
        failure: "平台项是预算（Basis::Budget）不是实测，P3 脉冲法之前只证明「账面不超」；互相关对强非线性输出可能偏峰",
    },
    Info {
        id: "V-FRAME",
        name: "最坏一帧耗时",
        layer: "ci",
        kind: Kind::Physical,
        default_level: Level::L3,
        unit: "×周期",
        better: Better::Lower,
        definition: "每 hop 整个 Engine::process 的耗时 ÷ hop 周期，取最大（旁报 p99.9）",
        algorithm: "逐 hop Instant 计时",
        anchor: "rubric V3：最坏一帧 ≤ 周期一半",
        failure: "CI 虚拟机与 debug 构建计时不可信（规约 §7 G10 放 L4）→ 在这两种环境下不过线时判「不知道」而非「不过」",
    },
    Info {
        id: "V-NAN",
        name: "数值失效",
        layer: "ci",
        kind: Kind::Physical,
        default_level: Level::L3,
        unit: "个",
        better: Better::Lower,
        definition: "输出里 NaN / Inf 的个数 + 幅度 > 4（engine max_abs）的个数",
        algorithm: "逐样本计数",
        anchor: "rubric V6：任何 > 0 → 否决",
        failure: "只看输出，不看中间状态",
    },
    Info {
        id: "V-MUTE",
        name: "静音全零",
        layer: "ci",
        kind: Kind::Physical,
        default_level: Level::L3,
        unit: "dBFS",
        better: Better::Lower,
        definition: "输入数字静音时，输出（过 0.3 s 稳定期后）的峰峰值",
        algorithm: "找输入末尾最长的全零段，量输出同段的 max − min，换成 dBFS（全零记 −240）",
        anchor: "[S35] Works With Meet：全零或 (max − min) ≤ 0.0028 dBFS——原文单位口径存疑，这里取〔建议值〕1 LSB@16 bit = −90.3 dBFS",
        failure: "稳定期 0.3 s 是〔建议值〕",
    },
    Info {
        id: "V-SWITCH",
        name: "送原声切换跳变",
        layer: "ci",
        kind: Kind::Physical,
        default_level: Level::L3,
        unit: "×",
        better: Better::Lower,
        definition: "进出送原声的那几个 hop 附近，输出一阶差分的最大值 ÷ 输入一阶差分的 p99.9",
        algorithm: "由每 hop 的 bypass 计数增量找切换点，±64 样本窗",
        anchor: "012：跳变 = 0；这里容许不超过信号自身陡度的 2 倍〔建议值〕",
        failure: "没有发生切换时判「不知道」；输入本身有瞬态时分母变大，会放松",
    },
    Info {
        id: "V-BGSTAB",
        name: "背景跳变 / 抽吸",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Lower,
        definition: "同一噪声「插入语音」与「不插语音」两次处理，停顿帧里输出噪声电平之差的最大绝对值",
        algorithm: "P.1120 §11.14.5 式：先放 ≥ 5 s 平稳噪声再插入语音。改动：用 10 ms 帧功率、只取主讲人与干扰者都不在的帧（掩码自带 30 ms + 再扩 50 ms），帧间 3 帧滑动平均，不用 35 ms 一阶平滑（那会把语音能量拖进停顿帧）",
        anchor: "> 10 dB → D（P.1120 原文线）；≤ 3 / 6 / 10 为 A / B / C〔建议值〕",
        failure: "把噪声换成恒定假底噪也能拿满分 → 与 V-VACUUM、V-TIMBRE 制衡；要求噪声逐样本可重放（合成场景满足，真实录音不满足 → 判「不知道」）",
    },
    Info {
        id: "V-ONSET",
        name: "吞字头",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Lower,
        definition: "字头 50 ms 的能量损失 − 全部有声帧的平均能量损失",
        algorithm: "clean 20 ms 帧、比最大帧低 35 dB 以内算有声；前面 ≥ 100 ms 无声的有声帧为起点；字头 50 ms 与有声段逐 20 ms 帧都用输出对干净参考的投影能量（不受残留噪声影响）",
        anchor: "> 3 dB → 吞字头（rubric C3）",
        failure: "原声起音被噪声盖住；VAD 门限对低电平说话人失准；起点少于 3 个判「不知道」",
    },
    Info {
        id: "V-NR",
        name: "残留噪声电平（TNLR）",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Lower,
        definition: "G.160 附录 II 式 TNLR：纯噪声帧里输出功率 ÷ 输入功率；旁报 SNRI",
        algorithm: "主讲人与干扰者都不在的 10 ms 帧（扩 30 ms）上求均值；SNRI = SNR_out − SNR_in，SNR 用有声帧功率减噪声帧功率",
        anchor: "rubric C2；每个任务给自己的过线值（「该压」的任务要求 ≤ −6 dB〔建议值〕）",
        failure: "过度抑制也拿高分 → 必须与 V-SPLVL、V-ONSET、V-VACUUM 成对看（探针 P04/P05 证明它单独会被刷）",
    },
    Info {
        id: "V-INTF",
        name: "邻座压低",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Lower,
        definition: "只有干扰者说话的帧里输出功率 ÷ 输入功率",
        algorithm: "干扰者有声、主讲人（扩 30 ms）不在的 10 ms 帧",
        anchor: "rubric C4；能力集要求 ≤ −10 dB〔建议值〕；外部参照 Teams §6.26（测法不同，不可直接比）",
        failure: "只量「压了多少」，不量「主讲人有没有被一起压」→ 与 V-SPLVL 成对",
    },
    Info {
        id: "V-SPLVL",
        name: "人声被压低",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Higher,
        definition: "主讲人有声帧（无干扰者）里，输出语音功率（减去残留噪声）÷ 干净参考功率；增益为正记 0",
        algorithm: "逐 10 ms 帧把输出投影到干净参考上（⟨o,c⟩²/‖c‖²），合计后比干净参考能量；不用 P.56（输出含噪声时 P.56 会把噪声算成语音），也不用停顿帧估噪声再扣（门控开合使两者差约 6 dB）",
        anchor: "≥ −3 dB 过〔建议值〕；rubric C3 损失档",
        failure: "后级 AGC 会掩盖；残留噪声估计偏大时会高估损失",
    },
    Info {
        id: "V-MUSIC",
        name: "音乐噪声（残留频谱峰度）",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "×",
        better: Better::Lower,
        definition: "纯噪声帧上，输出功率谱值的峰度 ÷ 输入功率谱值的峰度；旁报孤立亮点密度",
        algorithm: "STFT 512/256（vdn-dsp），125 Hz–6 kHz，帧中心 ±2 个 10 ms 帧都是纯噪声帧才计；每个频点先除以它在停顿帧上的均值再合并（去掉谱倾斜）；亮点：比 3×3 邻域中位数高 ≥ 12 dB 且上下左右都低于它 6 dB 以上，每秒个数",
        anchor: "峰度比 ≤ 1.2 A · ≤ 1.5 B · ≤ 2 C · > 2 D〔建议值〕",
        failure: "峰度与感知只弱相关（arXiv 2105.13079）；杯碟等瞬态本身峰度高（比值能部分抵消）；纯噪声帧 < 30 判「不知道」",
    },
    Info {
        id: "V-VACUUM",
        name: "真空感",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Higher,
        definition: "纯噪声帧的衰减（同 V-NR 的 TNLR），衰减深过 −40 dB 报警",
        algorithm: "同 V-NR；012 的「底噪频谱缺口」条件本实现未做",
        anchor: "rubric N3：noise_att_db ≤ −40 → 真空（只报警）；exp003 c09 有反例（−72 dB 作者说「非常干净」）",
        failure: "用户偏好不一：有人就要全静 → 只报警，不阻断",
    },
    Info {
        id: "V-TIMBRE",
        name: "音色偏移（发闷 / 齿音）",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Lower,
        definition: "有声帧长时平均谱四频带（0.1–1 · 1–2 · 2–4 · 4–7.9 kHz）相对总电平的偏差，输出 vs 干净参考，取最大绝对值",
        algorithm: "STFT 512/256；输出的频带功率先减去停顿帧估出的残留噪声频带功率（下限为原值的 10%）",
        anchor: "rubric N4；≤ 2 / 4 / 6 dB 为 A / B / C，> 6 dB 不过〔建议值〕",
        failure: "麦克风本身频响；低信噪比的高频带残留噪声扣不干净",
    },
    Info {
        id: "V-SIM-GATE",
        name: "门控漏开（模拟真值）",
        layer: "ci",
        kind: Kind::Sim,
        default_level: Level::L3,
        unit: "比例",
        better: Better::Lower,
        definition: "主讲人在说话的 hop 里门控没开的比例；字头 200 ms 单列",
        algorithm: "真值 = 合成器的音节区间（hop 与音节重叠 ≥ 半个 hop 算在说话）；门控 = engine 快开慢关包络的 voiced（EngineStats 增量）；字头 = 主讲人静默 ≥ 300 ms 后的音节起点",
        anchor: "漏开 ≤ 5%、字头漏开 ≤ 20%〔建议值〕",
        failure: "只在模拟内 L3（012 §3.4 限定 1）；门控对 ≠ 听着好（限定 2）；真实录音没有真值",
    },
    Info {
        id: "V-SIM-FA",
        name: "门控误开（只有邻座说话时）",
        layer: "ci",
        kind: Kind::Sim,
        default_level: Level::L3,
        unit: "比例",
        better: Better::Lower,
        definition: "只有干扰者说话（主讲人 ±200 ms 内都不在）的 hop 里门控开着的比例",
        algorithm: "同 V-SIM-GATE",
        anchor: "≤ 10%〔建议值〕。现在的门控只看能量，不认人——这是能力集，P2 的活",
        failure: "同 V-SIM-GATE",
    },
    Info {
        id: "V-SIM-SWLAT",
        name: "门控跟上的延迟",
        layer: "ci",
        kind: Kind::Sim,
        default_level: Level::L3,
        unit: "ms",
        better: Better::Lower,
        definition: "主讲人开口（此前门控关着）到门控打开的时间，取 p90",
        algorithm: "同 V-SIM-GATE；开口时门控已经开着（被干扰者或拖尾打开）的不计入，单报个数",
        anchor: "≤ 32 ms（两个 hop）〔建议值〕",
        failure: "同 V-SIM-GATE；可测起点 < 3 个判「不知道」",
    },
    Info {
        id: "V-SIM-QUIET",
        name: "没人说话时门控要关（模拟真值）",
        layer: "ci",
        kind: Kind::Sim,
        default_level: Level::L3,
        unit: "比例",
        better: Better::Lower,
        definition: "主讲人（±200 ms 拖尾容许）与干扰者都不在的 hop 里，门控开着的比例",
        algorithm: "同 V-SIM-GATE。门控开着时 engine 混回 open_db（−6 dB）原声而不是 floor_db（−12 dB），停顿就压不下去",
        anchor: "≤ 10%〔建议值〕。exp007 首跑：GTCRN 在粉噪 10 dB 下约 99%（engine 底噪跟踪「往下立刻跟」被模型输出的低谷钉住，见 README）",
        failure: "同 V-SIM-GATE；拖尾容许 200 ms 是〔建议值〕",
    },
    Info {
        id: "V-WEAK",
        name: "轻声字被吞",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB",
        better: Better::Lower,
        definition: "500 Hz–4 kHz 各 1/3 倍频程带上，干净轨与输出有声帧电平第 30 百分位之差（L30 损失）的均值；旁报动态范围变化 Δ(P99 − P30)",
        algorithm: "012 §3.1 V-WEAK（IEC 60118-15 式分析）：125 ms Hann 窗、50% 重叠，帧中心落在主讲人掩码里才算；不先扣整体增益",
        anchor: "012：L30 ≤ 1 / 2 / 4 dB 为 A / B / C，> 4 D；Δ(P99 − P30) < −3 dB 另标「被压缩」（原文标〔建议值〕）",
        failure: "灵敏度低：合成人声上把最弱 30% 的段压 −10 / −20 dB 只读到约 1.8 / 3.7 dB，4 dB 线只抓得住整段吞掉（scorecard 2026-11-03 实测）；残留噪声会抬高低分位；策略可以加压缩把 L30 抬回来 → 看 Δ(P99 − P30)",
    },
    Info {
        id: "V-PUMPV",
        name: "人声忽大忽小",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "dB²",
        better: Better::Lower,
        definition: "有声 100 ms 窗上，输出投影到干净轨的能量 ÷ 干净轨能量（dB）的方差",
        algorithm: "012 §3.2 V-PUMPV；用投影能量（同 V-ONSET）去掉不相关的残留噪声；一窗里有声 10 ms 帧 ≥ 80% 才算",
        anchor: "012：方差 ≤ 1 dB² 为 A，其余档未写；过线值暂定（待定清单）",
        failure: "原声本身的 Lombard 变化不在干净轨里时会误报；慢变的整体增益不算抽动",
    },
    Info {
        id: "V-CLICK",
        name: "爆音 / 咔嗒",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "个/分钟",
        better: Better::Lower,
        definition: "输出一阶差分超过局部 20 ms 中位绝对偏差 k 倍、且带噪输入 ±2 ms 内没有的点；5 ms 内的算一个",
        algorithm: "012 §3.2 V-CLICK（k = 8）；按每分钟计",
        anchor: "012：每分钟 0 个为 A",
        failure: "原声里的真实瞬态（杯碟）被降噪后变陡也会被计；帧边界不连续另需专查",
    },
    Info {
        id: "V-ESTOI",
        name: "可懂度变化（ΔESTOI）",
        layer: "ci",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "Δ",
        better: Better::Higher,
        definition: "ESTOI(干净, 输出) − ESTOI(干净, 带噪原声)",
        algorithm: "pystoi 0.4.1 口径的 ESTOI，Rust 重写（`verifiers/estoi.rs`），与上游对拍差 ≤ 0.01（`tests/estoi_parity.rs`）",
        anchor: "rubric C3：ΔESTOI ≥ +0.05 A · ≥ 0 B · < 0 C · < −0.05 D",
        failure: "对生成式伪影不敏感（012）；要求输出与干净轨逐样本对齐",
    },
    Info {
        id: "V-EAT-ASR",
        name: "吃字（ASR 删除 + 替换差）",
        layer: "qa",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "百分点",
        better: Better::Lower,
        definition: "Δ(del + sub) = (候选删除 + 替换 − 原声删除 + 替换) ÷ 参考字数；参考 = 干净参考的 ASR 转写",
        algorithm: "本机 Qwen3-ASR（127.0.0.1:11520），temperature 0，每个文件转 2 次（多次试验）、按 sha256 缓存；两次不一致记 unstable 并剔除；集合层按片段配对 bootstrap 10000 次（种子 20261030）",
        anchor: "rubric V7：Δdel > +1 个百分点记 v7_suspect；片段 ≥ 30 且区间下界 > 0 才否决；本实现另报 Δsub（012 R13）",
        failure: "ASR 对伪影敏感而非人耳；LLM 解码器脑补（看 Δins）；exp003 里人勾吃字远多于 ASR",
    },
    Info {
        id: "V-DNSMOS",
        name: "DNSMOS P.835（SIG / BAK / OVRL）",
        layer: "qa",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "MOS",
        better: Better::Higher,
        definition: "DNSMOS sig_bak_ovr.onnx 的三项，非个性化多项式映射；主值报 OVRL",
        algorithm: "9.01 s 窗、1 s 跳，不够长时自我重复（同官方 dnsmos_local.py 的口径，Rust 自写）；ort 单线程 CPU",
        anchor: "无外部锚点；只作 L0 约束（不比基线差）",
        failure: "exp003：机器分领先的 DPDFNet-2 人耳最差；007 已见 BAK 被刷高；不建议作奖励",
    },
    Info {
        id: "V-NRMOS-SCOREQ",
        name: "听感分 · SCOREQ（主）",
        layer: "qa",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "Δ MOS",
        better: Better::Higher,
        definition: "SCOREQ（natural · NR）给输出的分 − 给带噪原声的分",
        algorithm: "ort 跑上游 ONNX（adapt_nr_telephone.onnx，sha256 登记在 qa/nrmos.rs）；16 kHz 尾部补零到 320 倍数、一次前向；与上游 Python 对拍差 ≤ 0.01（tests/nrmos_parity.rs）",
        anchor: "过线 = 不比原声差（012 §2.3 第 4 条 L0 约束）；公开众人数据上配对一致率 0.70–0.77（017 decision 第 0 节）；在我们片段上尚无判据",
        failure: "当奖励时被钻空子最狠（018 结论 4）→ 只打分不优化；TMHINT 上系统级 −0.714（018 第二节）；exp009 里与作者一致率 0.375–0.40",
    },
    Info {
        id: "V-NRMOS-DISTILL",
        name: "听感分 · Distill-MOS（交叉）",
        layer: "qa",
        kind: Kind::Perceptual,
        default_level: Level::L0,
        unit: "Δ MOS",
        better: Better::Higher,
        definition: "Distill-MOS v7 给输出的分 − 给带噪原声的分",
        algorithm: "上游 PyTorch 权重导出的 ONNX（不分段前向，tools/export_distillmos.py）；分段与求均值照上游 sqa.py 在 Rust 里做；与上游 Python 对拍差 ≤ 0.01",
        anchor: "过线 = 不比原声差；公开众人数据上配对一致率 0.68–0.69（017 decision 第 3 节）",
        failure: "与 SCOREQ 同为众包 MOS 训的，彼此一致不算独立旁证（017 第 0 节第 4 条）；系统级相关 0.772 低于 SCOREQ",
    },
];

pub fn info(id: &str) -> Option<&'static Info> {
    REGISTRY.iter().find(|i| i.id == id)
}

/// 任务清单里给一个 grader 的参数（JSON 对象）。
#[derive(Clone, Debug, Default)]
pub struct Params(pub serde_json::Map<String, Value>);

impl Params {
    pub fn f(&self, k: &str) -> Option<f64> {
        self.0.get(k).and_then(Value::as_f64)
    }
    pub fn f_or(&self, k: &str, d: f64) -> f64 {
        self.f(k).unwrap_or(d)
    }
}

/// 一次评分的输入。
pub struct Ctx<'a> {
    pub clip: &'a Clip,
    /// 被评的输出（探针时是劣化版）
    pub out: &'a [f32],
    pub run: Option<&'a RunOut>,
    pub masks: &'a Masks,
    /// 同一系统在「只有噪音」输入上的输出（V-BGSTAB 用；没有时为 `None`）
    pub noise_out: Option<&'a [f32]>,
    /// 在 CI 虚拟机上跑（计时不可信）
    pub ci: bool,
}

fn power_over(x: &[f32], mask: &[bool]) -> Option<(f64, usize)> {
    let p = frame_power(x, MASK_FRAME);
    let mut s = 0.0;
    let mut n = 0usize;
    for (v, &m) in p.iter().zip(mask) {
        if m {
            s += v;
            n += 1;
        }
    }
    (n > 0).then(|| (s / n as f64, n))
}

/// 按 id 评分。
pub fn eval(id: &str, c: &Ctx, p: &Params) -> Score {
    match id {
        "V-LAT" => v_lat(c),
        "V-FRAME" => v_frame(c, p),
        "V-NAN" => v_nan(c),
        "V-MUTE" => v_mute(c, p),
        "V-SWITCH" => v_switch(c, p),
        "V-BGSTAB" => v_bgstab(c, p),
        "V-ONSET" => v_onset(c, p),
        "V-NR" => v_nr(c, p),
        "V-INTF" => v_intf(c, p),
        "V-SPLVL" => v_splvl(c, p),
        "V-MUSIC" => v_music(c, p),
        "V-VACUUM" => v_vacuum(c, p),
        "V-TIMBRE" => v_timbre(c, p),
        "V-SIM-GATE" => v_sim_gate(c, p),
        "V-SIM-FA" => v_sim_fa(c, p),
        "V-SIM-SWLAT" => v_sim_swlat(c, p),
        "V-SIM-QUIET" => v_sim_quiet(c, p),
        "V-WEAK" => v_weak(c, p),
        "V-PUMPV" => v_pumpv(c, p),
        "V-CLICK" => v_click(c, p),
        "V-ESTOI" => v_estoi(c, p),
        _ => Score::unknown("V-?", "", Better::Lower, format!("CI 层没有 verifier {id}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_ids_unique_and_documented() {
        let mut ids: Vec<&str> = REGISTRY.iter().map(|i| i.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), REGISTRY.len());
        for i in REGISTRY {
            assert!(!i.definition.is_empty() && !i.algorithm.is_empty());
            assert!(!i.anchor.is_empty() && !i.failure.is_empty());
            // 感知类定标前一律 L0
            if i.kind == Kind::Perceptual {
                assert_eq!(i.default_level, Level::L0, "{}", i.id);
            }
        }
    }
}
