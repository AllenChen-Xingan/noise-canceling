//! 评测阈值与分档边界的正本（issue #62 第二段第 6 件）。
//!
//! - `verifiers.rs` 从这里取分档边界和默认阈值，调用处不再写数字
//! - 每个数带出处：012（`docs/research/012_eval-standards/verifiers.md`）或 rubric（`specs/rubric/denoise.md`）
//!   的原文摘录，或「设计值」的理由；摘录必须逐字在文档里，摘录里的数字必须和表里的数对得上
//! - `ci.json` / `qa.json` 里的每个阈值都必须在表里登记
//! - 已知对不上、或没有出处的条目列在 [`PENDING`]（全部是建议值，挂 012 拍板 #8 · 盲听回灌 D44，不新开题）。
//!   比对测试要求「找到的不一致」与 [`PENDING`] **完全相等**：多出新的不一致 → 红；清单里的条目被修好 → 也红，提醒移出清单
//!
//! 比对测试在 `tests/thresholds.rs`；比对逻辑是本文件的 [`mismatches`]（纯函数，便于造负例）。
//! 来历与后果的调查见 `state/handoff/hygiene-thresholds.md`。

use serde_json::Value;

/// 一个数的出处。
#[derive(Clone, Copy, Debug)]
pub enum Src {
    /// 012 verifiers.md 的原文摘录（每段都要在文档里逐字出现）
    Doc012(&'static [&'static str]),
    /// rubric denoise.md 的原文摘录
    Rubric(&'static [&'static str]),
    /// rubric 原文摘录，但文档写的是损失量（正数），代码记成负的 dB：比对前把文档的数取负（显式登记的符号转换）
    RubricLoss(&'static [&'static str]),
    /// 设计值：文档没给数字，理由写在这里（物理量、模拟真值、任务里写明的有意放宽）
    Design(&'static str),
    /// 没有出处：只能出现在 [`PENDING`] 里
    None,
}

/// 一个参数：默认值（代码里用）与清单里允许出现的取值。
#[derive(Clone, Copy, Debug)]
pub struct Param {
    pub key: &'static str,
    /// 清单没给这个参数时代码用的值；`None` = 清单必须给
    pub default: Option<f64>,
    /// 清单里允许的取值（含默认值）与各自出处
    pub values: &'static [(f64, Src)],
}

/// 一个 verifier 的分档边界与参数。
#[derive(Clone, Copy, Debug)]
pub struct Lines {
    pub id: &'static str,
    /// A/B/C 三档边界（`Lower` 升序，`Higher` 降序）；空 = 不分档
    pub edges: &'static [f64],
    pub edges_src: Src,
    pub params: &'static [Param],
}

impl Lines {
    /// 参数 `key` 的默认值；表里没有时是 NaN（`every_default_used_in_code_exists` 测试保证不会发生）。
    pub fn def(&self, key: &str) -> f64 {
        self.params
            .iter()
            .find(|p| p.key == key)
            .and_then(|p| p.default)
            .unwrap_or(f64::NAN)
    }
}

const fn p(key: &'static str, default: Option<f64>, values: &'static [(f64, Src)]) -> Param {
    Param {
        key,
        default,
        values,
    }
}

const NO_EDGES: Src = Src::Design("不分档");
const PHYS: Src = Src::Design("物理量 / 模拟真值：012 只写判法不写数，设计值，未定标");

pub const BGSTAB: Lines = Lines {
    id: "V-BGSTAB",
    edges: &[3.0, 6.0, 10.0],
    edges_src: Src::Doc012(&["≤ 3 / 6 / 10 dB 为 A / B / C"]),
    params: &[
        p(
            "max",
            Some(10.0),
            &[(10.0, Src::Doc012(&["**> 10 dB → D**"]))],
        ),
        p(
            "from_s",
            Some(0.0),
            &[
                (0.0, Src::Design("默认从头算")),
                (5.0, Src::Doc012(&["平稳噪声先放 ≥ 5 s"])),
            ],
        ),
    ],
};

pub const ONSET: Lines = Lines {
    id: "V-ONSET",
    edges: &[1.0, 2.0, 3.0],
    edges_src: Src::Doc012(&["> 3 dB → 「吞字头」"]),
    params: &[p(
        "max",
        Some(3.0),
        &[(3.0, Src::Doc012(&["> 3 dB → 「吞字头」"]))],
    )],
};

/// 量法与 rubric C3 接近：代码按主讲人有声帧把输出投影到干净参考上算能量损失（`verifiers.rs` 登记：不用 P.56），
/// C3 按参考有声帧能量损失；差在投影去噪。分档照 C3（铁律 5：标准文档是正本；codex 第 2 轮查出量法描述错，
/// 据此把 C/D 线从 −3 改为 −4，大总管 8d 定 2026-11-02）
pub const SPLVL: Lines = Lines {
    id: "V-SPLVL",
    edges: &[-1.0, -2.0, -4.0],
    edges_src: Src::RubricLoss(&["损失 > 4 dB", "损失 > 2 dB", "损失 ≤ 1 dB"]),
    params: &[p("min", Some(-3.0), &[(-3.0, Src::None)])],
};

pub const MUSIC: Lines = Lines {
    id: "V-MUSIC",
    edges: &[1.2, 1.5, 2.0],
    edges_src: Src::Doc012(&["峰度比 ≤ 1.2 A · ≤ 1.5 B · ≤ 2 C · > 2 D"]),
    params: &[p(
        "max",
        Some(2.0),
        &[(2.0, Src::Doc012(&["≤ 2 C · > 2 D"]))],
    )],
};

/// 量法与 rubric N4 不同：代码 100–1k / 1–2k / 2–4k / 4–7.9k Hz、512 点跳 256；N4 100–300 / 300–500 / 2–5k / 5–8k Hz、1024 点跳 512
pub const TIMBRE: Lines = Lines {
    id: "V-TIMBRE",
    edges: &[2.0, 4.0, 6.0],
    edges_src: Src::Rubric(&["D > 6 · C ≤ 6 · B ≤ 3 · A ≤ 1.5"]),
    params: &[p(
        "max",
        Some(6.0),
        &[(6.0, Src::Rubric(&["D > 6 · C ≤ 6"]))],
    )],
};

pub const NR: Lines = Lines {
    id: "V-NR",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "max",
        None,
        &[
            (-6.0, Src::Rubric(&["| D | > −6 dB | 跟原声差不多 |"])),
            (-8.0, Src::None),
            (-10.0, Src::None),
            (-20.0, Src::Rubric(&["| A | ≤ −20 dB |"])),
        ],
    )],
};

pub const INTF: Lines = Lines {
    id: "V-INTF",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "max",
        None,
        &[(-10.0, Src::Rubric(&["| B | −15 < x ≤ −10 |"]))],
    )],
};

pub const VACUUM: Lines = Lines {
    id: "V-VACUUM",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "min",
        Some(-40.0),
        &[(-40.0, Src::Rubric(&["`noise_att_db` ≤ −40 dB"]))],
    )],
};

pub const EAT_ASR: Lines = Lines {
    id: "V-EAT-ASR",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p(
        "max",
        None,
        &[(1.0, Src::Rubric(&["（> +1 时带 `v7_suspect`）"]))],
    )],
};

pub const FRAME: Lines = Lines {
    id: "V-FRAME",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(0.5), &[(0.5, PHYS)])],
};

pub const MUTE: Lines = Lines {
    id: "V-MUTE",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p("max_dbfs", Some(-90.3), &[(-90.3, PHYS)]),
        p("settle_s", Some(0.3), &[(0.3, PHYS)]),
    ],
};

pub const SWITCH: Lines = Lines {
    id: "V-SWITCH",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(2.0), &[(2.0, PHYS)])],
};

pub const SIM_GATE: Lines = Lines {
    id: "V-SIM-GATE",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[
        p(
            "max",
            Some(0.05),
            &[
                (0.05, Src::None),
                (
                    0.4,
                    Src::Design("R01 / R03 的 note：放宽的护栏线，严格线在能力集 C10"),
                ),
            ],
        ),
        p(
            "onset_max",
            Some(0.2),
            &[
                (0.2, Src::None),
                (
                    0.5,
                    Src::Design("R01 / R03 的 note：放宽的护栏线，严格线在能力集 C10"),
                ),
            ],
        ),
    ],
};

pub const SIM_FA: Lines = Lines {
    id: "V-SIM-FA",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(0.1), &[(0.1, PHYS)])],
};

pub const SIM_QUIET: Lines = Lines {
    id: "V-SIM-QUIET",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(0.1), &[(0.1, PHYS)])],
};

pub const SIM_SWLAT: Lines = Lines {
    id: "V-SIM-SWLAT",
    edges: &[],
    edges_src: NO_EDGES,
    params: &[p("max", Some(32.0), &[(32.0, PHYS)])],
};

/// 表里全部 verifier。
pub const ALL: [&Lines; 16] = [
    &BGSTAB, &ONSET, &SPLVL, &MUSIC, &TIMBRE, &NR, &INTF, &VACUUM, &EAT_ASR, &FRAME, &MUTE,
    &SWITCH, &SIM_GATE, &SIM_FA, &SIM_QUIET, &SIM_SWLAT,
];

/// 不一致的原因（codex 第 3 轮 P1 #1：待定项要连原因一起对上，免得数值差异掩盖新的出处错误）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// 摘录在文档里找不到
    QuoteMissing,
    /// 没有出处（`Src::None`）
    NoSource,
    /// 摘录在，但数对不上
    Differ,
    /// 分档顺序不对，或登记表里查不到好坏方向
    Order,
    /// 清单里出现了表里没登记的值
    Unregistered,
    /// 默认值没登记在取值里
    DefaultMissing,
    /// 清单不是合法 JSON
    BadSuite,
}

/// 待定清单里一条的具体内容：精确到那组分档或那个数，以及原因。
#[derive(Clone, Copy, Debug)]
pub enum Pend {
    Edges(&'static [f64], Cause),
    Value(f64, Cause),
}

/// 待定清单：已知不一致或没有出处的条目。全部是建议值，挂 012 拍板 #8 · D44。
pub const PENDING: [(&str, &str, Pend); 7] = [
    // 量法与 rubric N4 不同，数也不同（代码 2 / 4 / 6，N4 1.5 / 3 / 6）
    ("V-TIMBRE", "edges", Pend::Edges(&[2.0, 4.0, 6.0], Cause::Differ)),
    // 过线值 −3 dB 没有出处
    ("V-SPLVL", "min", Pend::Value(-3.0, Cause::NoSource)),
    // 只有 3 dB 有出处，1 / 2 dB 没有
    ("V-ONSET", "edges", Pend::Edges(&[1.0, 2.0, 3.0], Cause::Differ)),
    // −8 / −10 dB 没有出处（−6 / −20 是 rubric C2 档线）
    ("V-NR", "max", Pend::Value(-8.0, Cause::NoSource)),
    ("V-NR", "max", Pend::Value(-10.0, Cause::NoSource)),
    // 门控误关 0.05、词首 0.2 没有出处
    ("V-SIM-GATE", "max", Pend::Value(0.05, Cause::NoSource)),
    ("V-SIM-GATE", "onset_max", Pend::Value(0.2, Cause::NoSource)),
];

/// 一处不一致。
#[derive(Clone, Debug, PartialEq)]
pub enum What {
    /// 分档的数与文档对不上（附代码里的分档）
    Edges(Vec<f64>),
    /// 分档顺序不对（`Lower` 应升序、`Higher` 应降序），或登记表里查不到方向
    EdgesOrder,
    /// 某个取值没有出处、摘录不对、或清单里出现了表里没登记的值
    Value(f64),
    /// 清单不是合法 JSON
    Suite,
}

#[derive(Clone, Debug)]
pub struct Mismatch {
    pub id: String,
    pub key: String,
    pub what: What,
    pub cause: Cause,
    pub why: String,
}

/// 比对时用的三份文本。
pub struct Docs<'a> {
    pub doc012: &'a str,
    pub rubric: &'a str,
    /// 清单文件名与原文（`ci.json` · `qa.json`）
    pub suites: &'a [(&'a str, &'a str)],
}

/// 统一写法：全角负号 → `-`，连续空白 → 一个空格。
fn norm(s: &str) -> String {
    s.replace('−', "-")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 文本里的数，带正负号：`-` 紧贴数字、且前面不是字母数字时算负号（`V-1` 不算）。
fn numbers(s: &str) -> Vec<f64> {
    let cs: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < cs.len() {
        let neg = cs[i] == '-'
            && cs.get(i + 1).is_some_and(char::is_ascii_digit)
            && !(i > 0 && cs[i - 1].is_alphanumeric());
        let start = if neg { i + 1 } else { i };
        if cs.get(start).is_some_and(char::is_ascii_digit)
            && !(start > 0 && !neg && cs[start - 1].is_ascii_digit())
        {
            let mut j = start;
            while j < cs.len()
                && (cs[j].is_ascii_digit()
                    || (cs[j] == '.' && cs.get(j + 1).is_some_and(char::is_ascii_digit)))
            {
                j += 1;
            }
            let t: String = cs[start..j].iter().collect();
            if let Ok(v) = t.parse::<f64>() {
                out.push(if neg { -v } else { v });
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

fn same(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// 摘录都在文档里吗；返回摘录里的数（带符号，`RubricLoss` 取负）。摘录不在文档里 → `Err(哪段)`。
fn quoted_numbers(src: Src, d: &Docs) -> Result<Option<Vec<f64>>, String> {
    let (doc, quotes, sign) = match src {
        Src::Doc012(q) => (d.doc012, q, 1.0),
        Src::Rubric(q) => (d.rubric, q, 1.0),
        Src::RubricLoss(q) => (d.rubric, q, -1.0),
        Src::Design(_) => return Ok(Some(Vec::new())),
        Src::None => return Ok(None),
    };
    let nd = norm(doc);
    let mut nums = Vec::new();
    for q in quotes {
        if !nd.contains(&norm(q)) {
            return Err(format!("摘录不在文档里：{q}"));
        }
        nums.extend(numbers(&norm(q)).into_iter().map(|x| x * sign));
    }
    Ok(Some(nums))
}

/// 分档顺序对不对：`lower` 时严格升序，否则严格降序。
fn ordered(edges: &[f64], lower: bool) -> bool {
    edges
        .windows(2)
        .all(|w| if lower { w[0] < w[1] } else { w[0] > w[1] })
}

/// 找出全部不一致。`lower_is_better(id)` 给分档方向（真仓库用 verifiers 登记表；测试可注入）。
pub fn mismatches(
    table: &[&Lines],
    d: &Docs,
    lower_is_better: &dyn Fn(&str) -> Option<bool>,
) -> Vec<Mismatch> {
    let mut out = Vec::new();
    let mut push = |id: &str, key: &str, what: What, cause: Cause, why: String| {
        out.push(Mismatch {
            id: id.to_string(),
            key: key.to_string(),
            what,
            cause,
            why,
        });
    };
    for l in table {
        if !l.edges.is_empty() {
            match lower_is_better(l.id) {
                Some(lower) if ordered(l.edges, lower) => {}
                Some(_) => push(
                    l.id,
                    "edges",
                    What::EdgesOrder,
                    Cause::Order,
                    format!("分档 {:?} 顺序不对", l.edges),
                ),
                None => push(
                    l.id,
                    "edges",
                    What::EdgesOrder,
                    Cause::Order,
                    "登记表里查不到好坏方向".into(),
                ),
            }
        }
        // 分档：摘录里的数（带符号）与边界作为集合相等
        let edges = What::Edges(l.edges.to_vec());
        match quoted_numbers(l.edges_src, d) {
            Err(e) => push(l.id, "edges", edges, Cause::QuoteMissing, e),
            Ok(None) => push(l.id, "edges", edges, Cause::NoSource, "分档没有出处".into()),
            Ok(Some(n)) if !matches!(l.edges_src, Src::Design(_)) => {
                let all_in = l.edges.iter().all(|e| n.iter().any(|x| same(*x, *e)));
                let back = n.iter().all(|x| l.edges.iter().any(|e| same(*x, *e)));
                if !(all_in && back) {
                    push(
                        l.id,
                        "edges",
                        edges,
                        Cause::Differ,
                        format!("代码 {:?} ↔ 文档 {n:?}", l.edges),
                    );
                }
            }
            Ok(Some(_)) => {}
        }
        // 参数：每个取值（带符号）要出现在它的摘录里
        for p in l.params {
            if let Some(dv) = p.default
                && !p.values.iter().any(|(v, _)| same(*v, dv))
            {
                push(
                    l.id,
                    p.key,
                    What::Value(dv),
                    Cause::DefaultMissing,
                    format!("默认值 {dv} 没登记在取值里"),
                );
            }
            for (v, s) in p.values {
                match quoted_numbers(*s, d) {
                    Err(e) => push(l.id, p.key, What::Value(*v), Cause::QuoteMissing, e),
                    Ok(None) => push(l.id, p.key, What::Value(*v), Cause::NoSource, format!("{v} 没有出处")),
                    Ok(Some(n))
                        if !matches!(s, Src::Design(_)) && !n.iter().any(|x| same(*x, *v)) =>
                    {
                        push(
                            l.id,
                            p.key,
                            What::Value(*v),
                            Cause::Differ,
                            format!("{v} 不在摘录的数 {n:?} 里"),
                        );
                    }
                    Ok(Some(_)) => {}
                }
            }
        }
    }
    // 清单：每个阈值都要在表里登记
    for (name, text) in d.suites {
        let Ok(v) = serde_json::from_str::<Value>(text) else {
            push(name, "-", What::Suite, Cause::BadSuite, "清单不是合法 JSON".into());
            continue;
        };
        let tasks = v["tasks"].as_array().cloned().unwrap_or_default();
        for t in &tasks {
            for gs in ["graders", "info"] {
                for g in t[gs].as_array().into_iter().flatten() {
                    let id = g["v"].as_str().unwrap_or("");
                    for (k, val) in g.as_object().into_iter().flatten() {
                        let Some(x) = val.as_f64() else { continue };
                        let ok = table.iter().find(|l| l.id == id).is_some_and(|l| {
                            l.params
                                .iter()
                                .any(|p| p.key == k && p.values.iter().any(|(v, _)| same(*v, x)))
                        });
                        if !ok {
                            let why = format!("{name} 任务 {} 的 {k} = {x} 没登记在表里", t["id"]);
                            push(id, k, What::Value(x), Cause::Unregistered, why);
                        }
                    }
                }
            }
        }
    }
    out
}

fn pend_matches(m: &Mismatch, (id, key, p): &(&str, &str, Pend)) -> bool {
    m.id == *id
        && m.key == *key
        && match (&m.what, p) {
            (What::Edges(a), Pend::Edges(b, c)) => {
                m.cause == *c && a.len() == b.len() && a.iter().zip(*b).all(|(x, y)| same(*x, *y))
            }
            (What::Value(a), Pend::Value(b, c)) => m.cause == *c && same(*a, *b),
            _ => false,
        }
}

/// 找到的不一致 ↔ 待定清单：多出的（新不一致）与少了的（已修好，要移出清单）都报。
pub fn compare_pending(found: &[Mismatch], pending: &[(&str, &str, Pend)]) -> Vec<String> {
    let mut errs = Vec::new();
    for m in found {
        if !pending.iter().any(|p| pend_matches(m, p)) {
            errs.push(format!(
                "新的不一致（不在待定清单里）：{} · {}：{}",
                m.id, m.key, m.why
            ));
        }
    }
    for p in pending {
        if !found.iter().any(|m| pend_matches(m, p)) {
            errs.push(format!(
                "待定清单里的 {} · {} · {:?} 已经对上了：把它移出 PENDING",
                p.0, p.1, p.2
            ));
        }
    }
    errs
}
