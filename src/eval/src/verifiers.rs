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
        _ => Score::unknown("V-?", "", Better::Lower, format!("CI 层没有 verifier {id}")),
    }
}

fn v_lat(c: &Ctx) -> Score {
    const ID: &str = "V-LAT";
    let Some(r) = c.run else {
        return Score::unknown(ID, "ms", Better::Lower, "没有系统运行记录");
    };
    let sr = c.clip.sr as f64;
    let Some(lag) = xcorr_lag(&c.clip.mix, &r.raw, 2048, 32_000) else {
        return Score::unknown(ID, "ms", Better::Lower, "输入太弱，互相关找不到峰");
    };
    let ok_algo = lag.abs_diff(r.latency) <= 1;
    let mut s = Score::judged(
        ID,
        r.ledger_total_ms,
        "ms",
        Better::Lower,
        Some(BUDGET_MS),
        &[],
    )
    .with_extra("measured_algo_ms", lag as f64 * 1000.0 / sr)
    .with_extra("declared_algo_ms", r.latency as f64 * 1000.0 / sr);
    if !ok_algo {
        s.verdict = Verdict::Fail;
        s.note = format!(
            "实测算法延迟 {lag} 样本 ≠ 声明 {} 样本（延迟账与实际不符）",
            r.latency
        );
    } else {
        s.note =
            "账面 = 引擎延迟账 + 平台预算（IO 块 96 帧）；平台项是预算，P3 实测前只证明账面不超"
                .into();
    }
    s
}

fn v_frame(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-FRAME";
    let Some(r) = c.run else {
        return Score::unknown(ID, "×周期", Better::Lower, "没有系统运行记录");
    };
    if r.engine_times.is_empty() {
        return Score::unknown(ID, "×周期", Better::Lower, "没有计时");
    }
    let period = r.hop as f64 / c.clip.sr as f64;
    let t: Vec<f64> = r
        .engine_times
        .iter()
        .map(|d| d.as_secs_f64() / period)
        .collect();
    let max = t.iter().cloned().fold(0.0, f64::max);
    let s = Score::judged(
        ID,
        max,
        "×周期",
        Better::Lower,
        Some(p.f_or("max", th::FRAME.def("max"))),
        &[],
    )
    .with_extra("p999", quantile(&t, 0.999));
    if s.verdict.is_fail() && (c.ci || cfg!(debug_assertions)) {
        let why = if c.ci {
            "CI 虚拟机计时不可信（规约 §7 G10 放 L4）"
        } else {
            "debug 构建计时偏慢，用 --release 重跑"
        };
        return s.demote_unknown(why);
    }
    s
}

fn v_nan(c: &Ctx) -> Score {
    let bad = c.out.iter().filter(|v| !v.is_finite()).count();
    let huge = c
        .out
        .iter()
        .filter(|v| v.is_finite() && v.abs() > 4.0)
        .count();
    Score::judged(
        "V-NAN",
        (bad + huge) as f64,
        "个",
        Better::Lower,
        Some(0.0),
        &[],
    )
    .with_extra("nonfinite", bad as f64)
    .with_extra("over_4", huge as f64)
}

fn v_mute(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-MUTE";
    let x = &c.clip.mix;
    let mut start = x.len();
    while start > 0 && x[start - 1] == 0.0 {
        start -= 1;
    }
    let settle = (p.f_or("settle_s", th::MUTE.def("settle_s")) * c.clip.sr as f64) as usize;
    let from = start + settle;
    if x.len() < from + c.clip.sr as usize / 4 {
        return Score::unknown(
            ID,
            "dBFS",
            Better::Lower,
            "输入末尾没有 ≥ 0.25 s 的数字静音",
        );
    }
    let seg = &c.out[from..x.len().min(c.out.len())];
    let (mn, mx) = seg
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), &v| {
            (a.min(v), b.max(v))
        });
    let p2p = (mx - mn) as f64;
    let v = if p2p.is_finite() {
        20.0 * p2p.max(1e-12).log10()
    } else {
        f64::INFINITY
    };
    Score::judged(
        ID,
        v,
        "dBFS",
        Better::Lower,
        Some(p.f_or("max_dbfs", th::MUTE.def("max_dbfs"))),
        &[],
    )
    .with_extra("silence_s", (x.len() - start) as f64 / c.clip.sr as f64)
}

fn v_switch(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SWITCH";
    let Some(r) = c.run else {
        return Score::unknown(ID, "×", Better::Lower, "没有系统运行记录");
    };
    let mut points = Vec::new();
    for h in 1..r.bypass.len() {
        if r.bypass[h] != r.bypass[h - 1] {
            // 切换发生在原始时间轴 h·hop，对齐后减去 latency
            if let Some(t) = (h * r.hop).checked_sub(r.latency) {
                points.push(t);
            }
        }
    }
    if points.is_empty() {
        return Score::unknown(ID, "×", Better::Lower, "本次没有发生送原声切换");
    }
    let dx: Vec<f64> = c
        .clip
        .mix
        .windows(2)
        .map(|w| (w[1] - w[0]).abs() as f64)
        .collect();
    let refd = quantile(&dx, 0.999).max(1e-6);
    let mut worst = 0.0f64;
    for &t in &points {
        let a = t.saturating_sub(64).max(1);
        let b = (t + 64).min(c.out.len());
        for i in a..b {
            worst = worst.max((c.out[i] - c.out[i - 1]).abs() as f64);
        }
    }
    Score::judged(
        ID,
        worst / refd,
        "×",
        Better::Lower,
        Some(p.f_or("max", th::SWITCH.def("max"))),
        &[],
    )
    .with_extra("switches", points.len() as f64)
}

/// 主讲人与干扰者都不在（扩 `guard_frames` 帧）的帧。
fn quiet_frames(c: &Ctx, guard_frames: usize) -> Vec<bool> {
    let nf = c.masks.noise_only.len();
    let mut q = c.masks.noise_only.clone();
    for f in 0..nf {
        if c.masks.owner[f] || c.masks.interf[f] {
            let a = f.saturating_sub(guard_frames);
            let b = (f + guard_frames + 1).min(nf);
            for v in q.iter_mut().take(b).skip(a) {
                *v = false;
            }
        }
    }
    q
}

fn v_bgstab(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-BGSTAB";
    let Some(nout) = c.noise_out else {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            "没有「只有噪音」的对照处理（真实录音的噪声不能逐样本重放）",
        );
    };
    let q = quiet_frames(c, 5);
    let a = frame_power(c.out, MASK_FRAME);
    let b = frame_power(nout, MASK_FRAME);
    let start = (p.f_or("from_s", th::BGSTAB.def("from_s")) * 100.0) as usize;
    let d: Vec<Option<f64>> = (0..q.len().min(a.len()).min(b.len()))
        .map(|f| (q[f] && f >= start && b[f] > 1e-14).then(|| db10(a[f]) - db10(b[f])))
        .collect();
    let mut worst = 0.0f64;
    let mut n = 0usize;
    for w in d.windows(3) {
        if let [Some(x), Some(y), Some(z)] = w {
            let m = (x + y + z) / 3.0;
            if m.abs() > worst.abs() {
                worst = m;
            }
            n += 1;
        }
    }
    if n < 30 {
        return Score::unknown(ID, "dB", Better::Lower, format!("可比的停顿帧只有 {n} 个"));
    }
    Score::judged(
        ID,
        worst.abs(),
        "dB",
        Better::Lower,
        Some(p.f_or("max", th::BGSTAB.def("max"))),
        th::BGSTAB.edges,
    )
    .with_extra("signed_db", worst)
    .with_extra("frames", n as f64)
}

/// 输出投影到干净参考上的能量 ⟨o, c⟩² / ‖c‖²：只留下与参考相关的部分，不受不相关的残留噪声影响
///（怀疑派自审 P1：停顿帧估的残留噪声比说话时小约 6 dB——门控开着时混回的原声更多——扣不干净会把人声损失藏掉）。
fn proj_energy(o: &[f32], c: &[f32]) -> (f64, f64) {
    let (mut oc, mut cc) = (0.0f64, 0.0f64);
    for (a, b) in o.iter().zip(c) {
        oc += *a as f64 * *b as f64;
        cc += *b as f64 * *b as f64;
    }
    let p = if cc > 0.0 { oc * oc / cc } else { 0.0 };
    (p, cc)
}

fn v_onset(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-ONSET";
    const F: usize = 320;
    const W: usize = 800; // 50 ms
    let ivs = truth_from_clean(&c.clip.clean);
    let mut onsets = Vec::new();
    let mut last_end: Option<usize> = None;
    for &(a, b) in &ivs {
        if last_end.is_none_or(|e| a >= e + 5 * F) && a + W <= c.out.len() {
            onsets.push(a);
        }
        last_end = Some(b);
    }
    if onsets.len() < 3 {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            format!("字头只有 {} 个", onsets.len()),
        );
    }
    let loss = |eo: f64, ec: f64| -> f64 { (db10(ec) - db10(eo.max(ec * 1e-4))).min(40.0) };
    let per: Vec<f64> = onsets
        .iter()
        .map(|&s| {
            let (po, pc) = proj_energy(&c.out[s..s + W], &c.clip.clean[s..s + W]);
            loss(po, pc)
        })
        .collect();
    // 有声段：逐 20 ms 帧投影后合计（容许慢变的增益）
    let (mut ec, mut eo) = (0.0, 0.0);
    for &(a, b) in &ivs {
        let b = b.min(c.out.len());
        let mut t = a;
        while t + F <= b {
            let (po, pc) = proj_energy(&c.out[t..t + F], &c.clip.clean[t..t + F]);
            eo += po;
            ec += pc;
            t += F;
        }
    }
    let avg = loss(eo, ec);
    let excess = mean(&per) - avg;
    Score::judged(
        ID,
        excess,
        "dB",
        Better::Lower,
        Some(p.f_or("max", th::ONSET.def("max"))),
        th::ONSET.edges,
    )
    .with_extra("onsets", per.len() as f64)
    .with_extra("p90_onset_loss_db", quantile(&per, 0.9))
    .with_extra("voiced_loss_db", avg)
}

fn tnlr(c: &Ctx) -> Option<(f64, usize)> {
    let (po, n) = power_over(c.out, &c.masks.noise_only)?;
    let (pi, _) = power_over(&c.clip.mix, &c.masks.noise_only)?;
    (n >= 50 && pi > 1e-14).then(|| (db10(po) - db10(pi), n))
}

fn v_nr(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-NR";
    let Some((t, n)) = tnlr(c) else {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            "纯噪声帧不足 0.5 s，或输入停顿是数字静音",
        );
    };
    let mut s =
        Score::judged(ID, t, "dB", Better::Lower, p.f("max"), &[]).with_extra("frames", n as f64);
    let own: Vec<bool> = (0..c.masks.owner.len())
        .map(|f| c.masks.owner[f] && !c.masks.interf[f])
        .collect();
    if let (Some((so, _)), Some((si, _)), Some((no, _)), Some((ni, _))) = (
        power_over(c.out, &own),
        power_over(&c.clip.mix, &own),
        power_over(c.out, &c.masks.noise_only),
        power_over(&c.clip.mix, &c.masks.noise_only),
    ) {
        let snr = |s: f64, n: f64| db10((s - n).max(s * 1e-4)) - db10(n.max(1e-14));
        s = s.with_extra("snri_db", snr(so, no) - snr(si, ni));
    }
    s
}

fn v_intf(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-INTF";
    let (Some((po, n)), Some((pi, _))) = (
        power_over(c.out, &c.masks.interf_only),
        power_over(&c.clip.mix, &c.masks.interf_only),
    ) else {
        return Score::unknown(ID, "dB", Better::Lower, "没有只有干扰者说话的帧");
    };
    if n < 50 {
        return Score::unknown(
            ID,
            "dB",
            Better::Lower,
            format!("只有干扰者的帧只有 {n} 个"),
        );
    }
    Score::judged(
        ID,
        db10(po) - db10(pi),
        "dB",
        Better::Lower,
        p.f("max"),
        &[],
    )
    .with_extra("frames", n as f64)
}

fn v_splvl(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SPLVL";
    let own: Vec<bool> = (0..c.masks.owner.len())
        .map(|f| c.masks.owner[f] && !c.masks.interf[f])
        .collect();
    let (mut ps, mut pc, mut n) = (0.0, 0.0, 0usize);
    for (f, &m) in own.iter().enumerate() {
        let (a, b) = (f * MASK_FRAME, (f + 1) * MASK_FRAME);
        if m && b <= c.out.len() {
            let (po, cc) = proj_energy(&c.out[a..b], &c.clip.clean[a..b]);
            ps += po;
            pc += cc;
            n += 1;
        }
    }
    if n < 50 || pc <= 1e-14 {
        return Score::unknown(ID, "dB", Better::Higher, "主讲人单独说话的帧不足 0.5 s");
    }
    let ps = ps.max(pc * 1e-4);
    let d = (db10(ps) - db10(pc)).min(0.0);
    Score::judged(
        ID,
        d,
        "dB",
        Better::Higher,
        Some(p.f_or("min", th::SPLVL.def("min"))),
        th::SPLVL.edges,
    )
}

fn noise_spec_frames(c: &Ctx, sg: &Spectrogram) -> Vec<usize> {
    let q = &c.masks.noise_only;
    (0..sg.frames.len())
        .filter(|&k| {
            sg.mask_frame(k)
                .is_some_and(|m| m >= 2 && m + 2 < q.len() && (m - 2..=m + 2).all(|f| q[f]))
        })
        .collect()
}

fn v_music(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-MUSIC";
    let so = Spectrogram::new(c.out, 512, 256);
    let si = Spectrogram::new(&c.clip.mix, 512, 256);
    let ks = noise_spec_frames(c, &so);
    if ks.len() < 30 {
        return Score::unknown(
            ID,
            "×",
            Better::Lower,
            format!("纯噪声谱帧只有 {} 个", ks.len()),
        );
    }
    let (b0, b1) = (4usize, 192usize);
    // 每个频点先除以它在停顿帧上的均值再合并：否则合并后的峰度量到的主要是谱倾斜（怀疑派自审 P1：白噪 9、粉噪 112）
    let collect = |sg: &Spectrogram| -> Vec<f64> {
        let mut out = Vec::with_capacity(ks.len() * (b1 - b0 + 1));
        for b in b0..=b1 {
            let m = ks.iter().map(|&k| sg.frames[k][b] as f64).sum::<f64>() / ks.len() as f64;
            if m > 0.0 {
                out.extend(ks.iter().map(|&k| sg.frames[k][b] as f64 / m));
            }
        }
        out
    };
    let ki = kurtosis(&collect(&si));
    let ko = kurtosis(&collect(&so));
    if !ki.is_finite() || !ko.is_finite() || ki <= 0.0 {
        return Score::unknown(
            ID,
            "×",
            Better::Lower,
            "峰度算不出（输入或输出在停顿里全零）",
        );
    }
    // 孤立亮点
    let db = |k: usize, b: usize| 10.0 * (so.frames[k][b] as f64).max(1e-20).log10();
    let mut peaks = 0usize;
    let set: std::collections::HashSet<usize> = ks.iter().copied().collect();
    for &k in &ks {
        if !(set.contains(&(k.wrapping_sub(1))) && set.contains(&(k + 1))) {
            continue;
        }
        for b in b0 + 1..b1 {
            let v = db(k, b);
            let nb = [db(k - 1, b), db(k + 1, b), db(k, b - 1), db(k, b + 1)];
            if nb.iter().any(|&x| x > v - 6.0) {
                continue;
            }
            let mut hood: Vec<f64> = Vec::with_capacity(9);
            for dk in [k - 1, k, k + 1] {
                for dbin in [b - 1, b, b + 1] {
                    hood.push(db(dk, dbin));
                }
            }
            if v >= quantile(&hood, 0.5) + 12.0 {
                peaks += 1;
            }
        }
    }
    let secs = ks.len() as f64 * 256.0 / c.clip.sr as f64;
    Score::judged(
        ID,
        ko / ki,
        "×",
        Better::Lower,
        Some(p.f_or("max", th::MUSIC.def("max"))),
        th::MUSIC.edges,
    )
    .with_extra("kurt_in", ki)
    .with_extra("kurt_out", ko)
    .with_extra("isolated_peaks_per_s", peaks as f64 / secs)
}

fn v_vacuum(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-VACUUM";
    let Some((t, _)) = tnlr(c) else {
        return Score::unknown(ID, "dB", Better::Higher, "纯噪声帧不足 0.5 s");
    };
    Score::judged(ID, t, "dB", Better::Higher, Some(p.f_or("min", th::VACUUM.def("min"))), &[])
}

fn v_timbre(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-TIMBRE";
    const BANDS: [(f64, f64); 4] = [
        (100.0, 1000.0),
        (1000.0, 2000.0),
        (2000.0, 4000.0),
        (4000.0, 7900.0),
    ];
    let so = Spectrogram::new(c.out, 512, 256);
    let sc = Spectrogram::new(&c.clip.clean, 512, 256);
    let own = |m: usize| m < c.masks.owner.len() && c.masks.owner[m] && !c.masks.interf[m];
    let speech: Vec<usize> = (0..so.frames.len())
        .filter(|&k| so.mask_frame(k).is_some_and(own))
        .collect();
    let quiet = noise_spec_frames(c, &so);
    if speech.len() < 50 {
        return Score::unknown(ID, "dB", Better::Lower, "主讲人有声谱帧不足");
    }
    let sr = c.clip.sr;
    let band_pow = |sg: &Spectrogram, ks: &[usize]| -> [f64; 4] {
        let mut out = [0.0; 4];
        if ks.is_empty() {
            return out;
        }
        for &k in ks {
            for (b, v) in sg.frames[k].iter().enumerate() {
                let hz = sg.bin_hz(b, sr);
                for (i, (lo, hi)) in BANDS.iter().enumerate() {
                    if hz >= *lo && hz < *hi {
                        out[i] += *v as f64;
                    }
                }
            }
        }
        out.map(|v| v / ks.len() as f64)
    };
    let pc = band_pow(&sc, &speech);
    let po = band_pow(&so, &speech);
    let pn = band_pow(&so, &quiet);
    let po: Vec<f64> = po
        .iter()
        .zip(&pn)
        .map(|(o, n)| (o - n).max(0.1 * o).max(1e-20))
        .collect();
    let tot_c: f64 = pc.iter().sum();
    let tot_o: f64 = po.iter().sum();
    if tot_c <= 0.0 {
        return Score::unknown(ID, "dB", Better::Lower, "干净参考全零");
    }
    let dev: Vec<f64> = (0..4)
        .map(|i| (db10(po[i]) - db10(tot_o)) - (db10(pc[i]) - db10(tot_c)))
        .collect();
    let worst = dev
        .iter()
        .cloned()
        .fold(0.0f64, |a, b| if b.abs() > a.abs() { b } else { a });
    Score::judged(
        ID,
        worst.abs(),
        "dB",
        Better::Lower,
        Some(p.f_or("max", th::TIMBRE.def("max"))),
        th::TIMBRE.edges,
    )
    .with_extra("dev_0_1k", dev[0])
    .with_extra("dev_1_2k", dev[1])
    .with_extra("dev_2_4k", dev[2])
    .with_extra("dev_4_8k", dev[3])
}

/// 门控类 verifier 容许的拖尾（主讲人停下后多久内门控开着不算误开）。
pub const GATE_TOLERANCE_S: f64 = 0.2;

struct HopTruth {
    owner: Vec<bool>,
    interf: Vec<bool>,
    /// 主讲人 ±GATE_TOLERANCE_S 内有声
    owner_near: Vec<bool>,
    gate: Vec<bool>,
    hop: usize,
}

fn hop_truth(c: &Ctx, r: &RunOut) -> HopTruth {
    let hop = r.hop;
    let n = c.clip.len() / hop;
    let cover = |ivs: &[(usize, usize)], pad: usize| -> Vec<bool> {
        let mut acc = vec![0usize; n];
        for &(a, b) in ivs {
            let (a, b) = (a.saturating_sub(pad), (b + pad).min(n * hop));
            for (h, v) in acc.iter_mut().enumerate() {
                let (s, e) = (h * hop, (h + 1) * hop);
                let ov = e.min(b).saturating_sub(s.max(a));
                *v += ov;
            }
        }
        acc.iter().map(|&v| v * 2 >= hop).collect()
    };
    let tol = (GATE_TOLERANCE_S * c.clip.sr as f64) as usize;
    HopTruth {
        owner: cover(&c.clip.truth.owner, 0),
        interf: cover(&c.clip.truth.interf, 0),
        owner_near: cover(&c.clip.truth.owner, tol),
        gate: (0..n).map(|h| r.gate_at(h * hop + hop / 2)).collect(),
        hop,
    }
}

/// 主讲人「开口」：静默 ≥ 300 ms 后的音节起点（样本）。
fn owner_onsets(c: &Ctx) -> Vec<usize> {
    let gap = (0.3 * c.clip.sr as f64) as usize;
    let mut out = Vec::new();
    let mut last: Option<usize> = None;
    for &(a, b) in &c.clip.truth.owner {
        if last.is_none_or(|e| a >= e + gap) {
            out.push(a);
        }
        last = Some(b);
    }
    out
}

fn need_sim(c: &Ctx, id: &'static str, unit: &'static str) -> Result<HopTruth, Box<Score>> {
    if !c.clip.tracks {
        return Err(Box::new(Score::unknown(
            id,
            unit,
            Better::Lower,
            "没有逐轨分存，没有逐帧真值",
        )));
    }
    let Some(r) = c.run else {
        return Err(Box::new(Score::unknown(
            id,
            unit,
            Better::Lower,
            "没有系统运行记录",
        )));
    };
    Ok(hop_truth(c, r))
}

fn v_sim_gate(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-GATE";
    let t = match need_sim(c, ID, "比例") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let act: Vec<usize> = (0..t.owner.len()).filter(|&h| t.owner[h]).collect();
    if act.len() < 20 {
        return Score::unknown(ID, "比例", Better::Lower, "主讲人有声 hop 不足 20 个");
    }
    let miss = act.iter().filter(|&&h| !t.gate[h]).count() as f64 / act.len() as f64;
    // 旁报：主讲人（含拖尾容许）与干扰者都不在的 hop 里门控开着的比例（噪声误开）
    let quiet: Vec<usize> = (0..t.owner.len())
        .filter(|&h| !t.owner_near[h] && !t.interf[h])
        .collect();
    let noise_open = if quiet.is_empty() {
        f64::NAN
    } else {
        quiet.iter().filter(|&&h| t.gate[h]).count() as f64 / quiet.len() as f64
    };
    let w = (0.2 * c.clip.sr as f64) as usize;
    let mut on_total = 0usize;
    let mut on_miss = 0usize;
    for s in owner_onsets(c) {
        for h in s / t.hop..((s + w) / t.hop).min(t.owner.len()) {
            if t.owner[h] {
                on_total += 1;
                on_miss += usize::from(!t.gate[h]);
            }
        }
    }
    let onset_miss = if on_total > 0 {
        on_miss as f64 / on_total as f64
    } else {
        0.0
    };
    let mut s = Score::judged(
        ID,
        miss,
        "比例",
        Better::Lower,
        Some(p.f_or("max", th::SIM_GATE.def("max"))),
        &[],
    )
    .with_extra("onset_miss", onset_miss)
    .with_extra("noise_open", noise_open)
    .with_extra("owner_hops", act.len() as f64);
    let omax = p.f_or("onset_max", th::SIM_GATE.def("onset_max"));
    if s.verdict.is_pass() && onset_miss > omax {
        s.verdict = Verdict::Fail;
        s.note = format!(
            "字头 200 ms 漏开 {:.0}% > {:.0}%",
            onset_miss * 100.0,
            omax * 100.0
        );
    }
    s
}

fn v_sim_fa(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-FA";
    let t = match need_sim(c, ID, "比例") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let only: Vec<usize> = (0..t.owner.len())
        .filter(|&h| t.interf[h] && !t.owner_near[h])
        .collect();
    if only.len() < 20 {
        return Score::unknown(ID, "比例", Better::Lower, "只有干扰者说话的 hop 不足 20 个");
    }
    let fa = only.iter().filter(|&&h| t.gate[h]).count() as f64 / only.len() as f64;
    Score::judged(ID, fa, "比例", Better::Lower, Some(p.f_or("max", th::SIM_FA.def("max"))), &[])
        .with_extra("interf_only_hops", only.len() as f64)
}

fn v_sim_quiet(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-QUIET";
    let t = match need_sim(c, ID, "比例") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let quiet: Vec<usize> = (0..t.owner.len())
        .filter(|&h| !t.owner_near[h] && !t.interf[h])
        .collect();
    if quiet.len() < 20 {
        return Score::unknown(ID, "比例", Better::Lower, "没人说话的 hop 不足 20 个");
    }
    let open = quiet.iter().filter(|&&h| t.gate[h]).count() as f64 / quiet.len() as f64;
    Score::judged(
        ID,
        open,
        "比例",
        Better::Lower,
        Some(p.f_or("max", th::SIM_QUIET.def("max"))),
        &[],
    )
    .with_extra("quiet_hops", quiet.len() as f64)
}

fn v_sim_swlat(c: &Ctx, p: &Params) -> Score {
    const ID: &str = "V-SIM-SWLAT";
    let t = match need_sim(c, ID, "ms") {
        Ok(t) => t,
        Err(s) => return *s,
    };
    let ms_per_hop = t.hop as f64 * 1000.0 / c.clip.sr as f64;
    let mut lats = Vec::new();
    let mut already_open = 0usize;
    for s in owner_onsets(c) {
        let h0 = s / t.hop;
        if h0 == 0 || h0 >= t.gate.len() {
            continue;
        }
        if t.gate[h0 - 1] {
            already_open += 1;
            continue;
        }
        let lim = (h0 + 64).min(t.gate.len());
        let first = (h0..lim).find(|&h| t.gate[h]).unwrap_or(lim);
        lats.push((first - h0) as f64 * ms_per_hop);
    }
    if lats.len() < 3 {
        return Score::unknown(
            ID,
            "ms",
            Better::Lower,
            format!("门控关着时开口的起点只有 {} 个", lats.len()),
        );
    }
    Score::judged(
        ID,
        quantile(&lats, 0.9),
        "ms",
        Better::Lower,
        Some(p.f_or("max", th::SIM_SWLAT.def("max"))),
        &[],
    )
    .with_extra("onsets", lats.len() as f64)
    .with_extra("already_open", already_open as f64)
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
