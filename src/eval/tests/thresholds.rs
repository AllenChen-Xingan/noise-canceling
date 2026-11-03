//! 评测阈值比对（issue #62）：正本 `src/eval/src/thresholds.rs` ↔ 012 / rubric 文档 ↔ ci.json / qa.json ↔ verifiers.rs。

use std::path::PathBuf;

use vdn_eval::thresholds::{ALL, Docs, Lines, PENDING, Param, Src, compare_pending, mismatches};

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
    let found = mismatches(&ALL, &docs);
    let errs = compare_pending(&found, &PENDING);
    assert!(errs.is_empty(), "{errs:#?}\n全部不一致：{found:#?}");
}

/// verifiers.rs 里用到的 `th::X.def("key")` 在表里都有默认值（否则会拿到 NaN）；
/// 调用处也不许再写数字的默认阈值或分档
#[test]
fn every_default_used_in_code_exists() {
    let src = read("src/eval/src/verifiers.rs");
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
    let found = mismatches(&[&BAD], &docs(&[]));
    assert_eq!(found.len(), 1, "{found:?}");
    let errs = compare_pending(&found, &[]);
    assert!(errs[0].contains("新的不一致"), "{errs:?}");
    // 登记进待定清单后不红
    assert!(compare_pending(&found, &[("V-X", "edges")]).is_empty());
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
    let found = mismatches(&[&OK], &docs(&[]));
    assert!(found.is_empty(), "{found:?}");
    let errs = compare_pending(&found, &[("V-X", "edges")]);
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
    let found = mismatches(&[&L], &docs(&[("ci.json", suite)]));
    let keys: Vec<_> = found
        .iter()
        .map(|(_, k, w)| (k.as_str(), w.as_str()))
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
    let f = mismatches(&[&Q], &docs(&[]));
    assert!(f[0].2.contains("摘录不在文档里"), "{f:?}");
}
