//! 评测阈值比对（issue #62）：正本 `src/eval/src/thresholds.rs` ↔ 012 / rubric 文档 ↔ ci.json / qa.json ↔ verifiers.rs。

use std::path::PathBuf;

use vdn_eval::Better;
use vdn_eval::thresholds::{
    ALL, Cause, Docs, Lines, PENDING, Param, Pend, Src, What, compare_pending, mismatches,
};
use vdn_eval::verifiers::info;

/// 真仓库的好坏方向：查 verifiers 登记表
fn registry_dir(id: &str) -> Option<bool> {
    info(id).map(|i| i.better == Better::Lower)
}

/// 测试里造的 verifier 一律「越小越好」
fn lower(_: &str) -> Option<bool> {
    Some(true)
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|e| panic!("读 {rel}：{e}"))
}

/// 真仓库：找到的不一致必须与待定清单完全相等（多出新的 → 红；清单条目修好了 → 也红）
#[test]
fn table_matches_docs_and_suites_except_pending() {
    let (d012, rub) = (
        read("docs/research/012_eval-standards/verifiers.md"),
        read("specs/rubric/denoise.md"),
    );
    let (ci, qa) = (
        read("src/eval/suite/ci.json"),
        read("src/eval/suite/qa.json"),
    );
    let suites = [("ci.json", ci.as_str()), ("qa.json", qa.as_str())];
    let docs = Docs {
        doc012: &d012,
        rubric: &rub,
        suites: &suites,
    };
    let found = mismatches(&ALL, &docs, &registry_dir);
    let errs = compare_pending(&found, &PENDING);
    assert!(errs.is_empty(), "{errs:#?}\n全部不一致：{found:#?}");
}

/// verifiers.rs 及其子模块里用到的 `th::X.def("key")` 在表里都有默认值（否则会拿到 NaN）；
/// 调用处也不许再写数字的默认阈值或分档
#[test]
fn every_default_used_in_code_exists() {
    // verifiers.rs 与它的子模块（physical / perceptual / sim）
    let mut src = read("src/eval/src/verifiers.rs");
    for m in ["physical", "perceptual", "sim"] {
        src.push_str(&read(&format!("src/eval/src/verifiers/{m}.rs")));
    }
    let mut n = 0;
    for (i, _) in src.match_indices(".def(\"") {
        let head = &src[..i];
        let name = head
            .rsplit(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .next()
            .unwrap_or("");
        let key = src[i + 6..].split('"').next().unwrap_or("");
        let l = ALL
            .iter()
            .find(|l| l.id.trim_start_matches("V-").replace('-', "_") == name)
            .unwrap_or_else(|| panic!("表里没有 {name}"));
        assert!(l.def(key).is_finite(), "{name}.def({key:?}) 是 NaN");
        n += 1;
    }
    assert!(n >= 16, "只找到 {n} 处查表");
    for bad in [
        "f_or(\"max\", 0",
        "f_or(\"max\", 1",
        "f_or(\"max\", 2",
        "f_or(\"max\", 3",
        "f_or(\"min\", -",
    ] {
        assert!(
            !src.contains(bad),
            "verifiers.rs 里还有写死的默认阈值：{bad}"
        );
    }
}

const D012: &str = "| **V-X** | … | 档：≤ 1 A · ≤ 2 B · ≤ 3 C |";
const RUB: &str = "| D | > -6 dB |";

fn docs<'a>(suites: &'a [(&'a str, &'a str)]) -> Docs<'a> {
    Docs {
        doc012: D012,
        rubric: RUB,
        suites,
    }
}

/// 负例（会红）：分档与文档对不上、又不在待定清单里 → 报「新的不一致」
#[test]
fn new_mismatch_not_in_pending_is_red() {
    const BAD: Lines = Lines {
        id: "V-X",
        edges: &[1.0, 2.0, 4.0],
        edges_src: Src::Doc012(&["≤ 1 A · ≤ 2 B · ≤ 3 C"]),
        params: &[],
    };
    let found = mismatches(&[&BAD], &docs(&[]), &lower);
    assert_eq!(found.len(), 1, "{found:?}");
    let errs = compare_pending(&found, &[]);
    assert!(errs[0].contains("新的不一致"), "{errs:?}");
    // 登记进待定清单后不红
    assert!(compare_pending(&found, &[("V-X", "edges", Pend::Edges(&[1.0, 2.0, 4.0], Cause::Differ))]).is_empty());
}

/// 负例（会红）：待定清单里的条目已经对上了 → 报「移出 PENDING」
#[test]
fn fixed_pending_entry_is_red() {
    const OK: Lines = Lines {
        id: "V-X",
        edges: &[1.0, 2.0, 3.0],
        edges_src: Src::Doc012(&["≤ 1 A · ≤ 2 B · ≤ 3 C"]),
        params: &[],
    };
    let found = mismatches(&[&OK], &docs(&[]), &lower);
    assert!(found.is_empty(), "{found:?}");
    let errs = compare_pending(&found, &[("V-X", "edges", Pend::Edges(&[1.0, 2.0, 3.0], Cause::Differ))]);
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("移出 PENDING"), "{errs:?}");
}

/// 负例：摘录不在文档里、清单里有没登记的阈值、没有出处的取值 → 都算不一致
#[test]
fn bad_quote_unregistered_value_and_unsourced_are_mismatches() {
    const L: Lines = Lines {
        id: "V-X",
        edges: &[],
        edges_src: Src::Design("不分档"),
        params: &[Param {
            key: "max",
            default: Some(-6.0),
            values: &[(-6.0, Src::Rubric(&["| D | > -6 dB |"])), (-8.0, Src::None)],
        }],
    };
    let suite = r#"{"tasks":[{"id":"t","graders":[{"v":"V-X","max":-7}]}]}"#;
    let found = mismatches(&[&L], &docs(&[("ci.json", suite)]), &lower);
    let keys: Vec<_> = found
        .iter()
        .map(|m| (m.key.as_str(), m.why.as_str()))
        .collect();
    assert!(
        keys.iter().any(|(_, w)| w.contains("-8 没有出处")),
        "{keys:?}"
    );
    assert!(
        keys.iter().any(|(_, w)| w.contains("没登记在表里")),
        "{keys:?}"
    );
    const Q: Lines = Lines {
        id: "V-Y",
        edges: &[1.0],
        edges_src: Src::Rubric(&["这段话不在文档里 1"]),
        params: &[],
    };
    let f = mismatches(&[&Q], &docs(&[]), &lower);
    assert!(f[0].why.contains("摘录不在文档里"), "{f:?}");
}

/// 负例（codex 第 2 轮 P1 #1）：已登记的待定项不吞同一参数上的新错误
#[test]
fn pending_does_not_swallow_new_value_on_same_param() {
    const L: Lines = Lines {
        id: "V-X",
        edges: &[],
        edges_src: Src::Design("不分档"),
        params: &[Param {
            key: "max",
            default: None,
            values: &[(-8.0, Src::None)],
        }],
    };
    let suite = r#"{"tasks":[{"id":"t","graders":[{"v":"V-X","max":999}]}]}"#;
    let found = mismatches(&[&L], &docs(&[("ci.json", suite)]), &lower);
    let errs = compare_pending(&found, &[("V-X", "max", Pend::Value(-8.0, Cause::NoSource))]);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].contains("999"), "{errs:?}");
}

/// 负例（codex 第 2 轮 P1 #3）：符号反了要报（文档 −6，表里 +6）；分档顺序反了要报
#[test]
fn sign_and_order_are_checked() {
    const SIGN: Lines = Lines {
        id: "V-X",
        edges: &[],
        edges_src: Src::Design("不分档"),
        params: &[Param {
            key: "max",
            default: Some(6.0),
            values: &[(6.0, Src::Rubric(&["| D | > -6 dB |"]))],
        }],
    };
    let f = mismatches(&[&SIGN], &docs(&[]), &lower);
    assert!(f.iter().any(|m| m.what == What::Value(6.0)), "{f:?}");
    const ORDER: Lines = Lines {
        id: "V-X",
        edges: &[3.0, 2.0, 1.0],
        edges_src: Src::Doc012(&["≤ 1 A · ≤ 2 B · ≤ 3 C"]),
        params: &[],
    };
    let f = mismatches(&[&ORDER], &docs(&[]), &lower);
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].what, What::EdgesOrder);
    // 显式登记的符号转换：文档写损失量 1 / 2 / 3，代码 −1 / −2 / −3（越大越好、降序）
    const LOSS: Lines = Lines {
        id: "V-X",
        edges: &[-1.0, -2.0, -3.0],
        edges_src: Src::RubricLoss(&["> -6 dB"]),
        params: &[],
    };
    let f = mismatches(&[&LOSS], &docs(&[]), &|_| Some(false));
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(matches!(f[0].what, What::Edges(_)), "{f:?}");
}

/// 负例（codex 第 3 轮 P1 #1）：同一组分档、同一个数，原因变了（数对不上 → 摘录不在文档里）也要红
#[test]
fn same_value_different_cause_is_red() {
    const BROKEN: Lines = Lines {
        id: "V-X",
        edges: &[1.0, 2.0, 4.0],
        edges_src: Src::Doc012(&["这段摘录已经不在文档里"]),
        params: &[],
    };
    let found = mismatches(&[&BROKEN], &docs(&[]), &lower);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].cause, Cause::QuoteMissing);
    let errs = compare_pending(
        &found,
        &[("V-X", "edges", Pend::Edges(&[1.0, 2.0, 4.0], Cause::Differ))],
    );
    assert_eq!(errs.len(), 2, "新原因要报、旧待定项要提示移出：{errs:?}");
}
