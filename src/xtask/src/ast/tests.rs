//! 模块树与测试判定的测试：012 第五节第 1 表的负例 T1–T14（必须判成生产或报错）与正例。

use std::fs;
use std::path::{Path, PathBuf};

use super::modtree::{Scope, add_crate};

/// 临时 crate：`files` 是（相对 crate 目录的路径，内容）；返回（仓库根，模块树）。
fn tree(tag: &str, files: &[(&str, &str)]) -> (PathBuf, Scope) {
    let root = std::env::temp_dir().join(format!("xtask-ast-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let c = root.join("c");
    for (p, s) in files {
        let f = c.join(p);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, s).unwrap();
    }
    let mut scope = Scope::default();
    add_crate(&root, &c, &mut scope);
    (root, scope)
}

fn done(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

/// 正例：`#[cfg(test)] mod tests;` 引入的文件、它声明的子模块、生产文件里 `#[cfg(test)] mod wt;` 都是测试；
/// `#[path]` 在顶层相对文件目录；`#![cfg(test)]` 整文件
#[test]
fn test_modules_are_found() {
    let (r, s) = tree(
        "pos",
        &[
            (
                "src/lib.rs",
                "mod worker;\nmod policy;\n#[cfg(test)]\nmod tests;\n",
            ),
            ("src/tests.rs", "mod more;\n"),
            ("src/tests/more.rs", "fn f() {}\n"),
            ("src/worker.rs", "#[cfg(test)]\nmod wt;\nfn prod() {}\n"),
            ("src/worker/wt.rs", "fn t() {}\n"),
            (
                "src/policy.rs",
                "#[cfg(test)]\n#[path = \"policy_tests.rs\"]\nmod tests;\n",
            ),
            ("src/policy_tests.rs", "fn t() {}\n"),
            ("tests/it.rs", "mod common;\n"),
            ("tests/common.rs", "fn c() {}\n"),
        ],
    );
    assert!(s.problems.is_empty(), "{:?}", s.problems);
    assert_eq!(
        s.test_files(),
        [
            "c/src/policy_tests.rs",
            "c/src/tests.rs",
            "c/src/tests/more.rs",
            "c/src/worker/wt.rs",
            "c/tests/common.rs",
            "c/tests/it.rs",
        ]
    );
    assert!(!s.is_test_file("c/src/worker.rs"));
    assert!(s.orphans.is_empty(), "{:?}", s.orphans);
    done(&r);
}

/// 正例：生产文件里 `#[cfg(test)]` / `#[test]` 项所占的行是测试行，之后的代码不是；impl 里的测试函数、
/// 生产函数体里带 `#[cfg(test)]` 的语句也认
#[test]
fn test_lines_in_prod_file() {
    let lib = "fn a() {}\n#[cfg(test)]\nfn helper() {\n    let _ = 1;\n}\nfn b() {}\nstruct S;\nimpl S {\n    #[cfg(test)]\n    fn t(&self) {}\n    fn p(&self) {}\n}\n#[cfg(all(test, unix))]\nmod inline {\n    fn x() {}\n}\nfn c() {\n    let a = 1;\n    #[cfg(test)]\n    spy();\n    let b = 2;\n}\n";
    let (r, s) = tree("lines", &[("src/lib.rs", lib)]);
    let t: Vec<usize> = (1..=22)
        .filter(|l| s.is_test_line("c/src/lib.rs", *l))
        .collect();
    assert_eq!(t, [2, 3, 4, 5, 9, 10, 13, 14, 15, 16, 19, 20]);
    done(&r);
}

/// T1 · T2：块注释、字符串里写的声明不算，worker.rs 不是测试（是孤儿，只报注释）
#[test]
fn t1_t2_comment_and_string_declarations() {
    for lib in [
        "/*\n#[cfg(test)]\nmod worker;\n*/\n",
        "const S: &str = \"\n#[cfg(test)]\nmod worker;\n\";\n",
    ] {
        let (r, s) = tree(
            "t12",
            &[("src/lib.rs", lib), ("src/worker.rs", "fn f() {}\n")],
        );
        assert!(!s.is_test_file("c/src/worker.rs"), "{lib}");
        assert_eq!(s.orphans, ["c/src/worker.rs"], "{lib}");
        done(&r);
    }
}

/// T3：内联模块里的 `#[cfg(test)] mod inner;` 指到 outer/inner.rs，不是 inner.rs
#[test]
fn t3_inline_module_path() {
    let (r, s) = tree(
        "t3",
        &[
            (
                "src/lib.rs",
                "mod outer {\n    #[cfg(test)]\n    mod inner;\n}\n",
            ),
            ("src/outer/inner.rs", "fn t() {}\n"),
            ("src/inner.rs", "fn p() {}\n"),
        ],
    );
    assert!(s.is_test_file("c/src/outer/inner.rs"));
    assert!(!s.is_test_file("c/src/inner.rs"));
    done(&r);
}

/// T4–T11：另一个入口的生产引用（含传递、内联路径、`r#`、带属性、同一行、cfg(test) 后隔了别的项）→ 生产
#[test]
fn t4_to_t11_production_references_win() {
    type Case<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a str);
    let cases: [Case; 8] = [
        (
            "t4",
            &[
                ("src/main.rs", "mod worker;\nfn main() {}\n"),
                ("src/worker.rs", ""),
            ],
            "c/src/worker.rs",
        ),
        (
            "t5",
            &[
                ("src/main.rs", "mod worker;\nfn main() {}\n"),
                ("src/worker.rs", "mod child;\n"),
                ("src/worker/child.rs", ""),
            ],
            "c/src/worker/child.rs",
        ),
        (
            "t6",
            &[
                ("src/main.rs", "mod worker;\nfn main() {}\n"),
                ("src/worker.rs", "mod inner {\n    mod child;\n}\n"),
                ("src/worker/inner/child.rs", ""),
            ],
            "c/src/worker/inner/child.rs",
        ),
        (
            "t7",
            &[
                (
                    "src/main.rs",
                    "mod tests {\n    mod child;\n}\nfn main() {}\n",
                ),
                ("src/tests/child.rs", ""),
            ],
            "c/src/tests/child.rs",
        ),
        (
            "t8",
            &[("src/main.rs", "mod r#tests;\nfn main() {}\n")],
            "c/src/tests.rs",
        ),
        (
            "t9",
            &[(
                "src/main.rs",
                "#[allow(dead_code)] mod tests;\nfn main() {}\n",
            )],
            "c/src/tests.rs",
        ),
        (
            "t10",
            &[("src/main.rs", "fn main() {} mod tests;\n")],
            "c/src/tests.rs",
        ),
        (
            "t11",
            &[(
                "src/main.rs",
                "#[cfg(test)]\nfn x() {}\nmod tests;\nfn main() {}\n",
            )],
            "c/src/tests.rs",
        ),
    ];
    for (tag, extra, target) in cases {
        let mut files: Vec<(&str, &str)> = vec![
            (
                "src/lib.rs",
                "#[cfg(test)]\nmod worker;\n#[cfg(test)]\nmod tests;\n",
            ),
            ("src/tests.rs", "mod child;\n"),
        ];
        if !extra.iter().any(|(p, _)| *p == "src/tests/child.rs") {
            files.push(("src/tests/child.rs", ""));
        }
        files.extend_from_slice(extra);
        if !files.iter().any(|(p, _)| *p == "src/worker.rs") {
            files.push(("src/worker.rs", ""));
        }
        let (r, s) = tree(tag, &files);
        assert!(!s.is_test_file(target), "{tag}: {target} 应按生产查");
        done(&r);
    }
}

/// T12：`cfg(any(test, …))` 可能进生产 → 照查
#[test]
fn t12_cfg_any_is_production() {
    let (r, s) = tree(
        "t12b",
        &[
            (
                "src/lib.rs",
                "#[cfg(any(test, feature = \"x\"))]\nmod helper;\n#[cfg(not(test))]\nfn p() {}\n",
            ),
            ("src/helper.rs", ""),
        ],
    );
    assert!(!s.is_test_file("c/src/helper.rs"));
    assert!(!s.is_test_line("c/src/lib.rs", 4));
    done(&r);
}

/// T13：`#[path]` 跳出 crate 目录 → 红；找不到模块文件 → 红；同时有 x.rs 与 x/mod.rs → 红
#[test]
fn t13_bad_module_files_are_errors() {
    let (r, s) = tree(
        "t13",
        &[
            (
                "src/lib.rs",
                "#[path = \"../../outside.rs\"]\nmod x;\nmod missing;\nmod both;\n",
            ),
            ("src/both.rs", ""),
            ("src/both/mod.rs", ""),
        ],
    );
    assert_eq!(s.problems.len(), 3, "{:?}", s.problems);
    assert!(s.problems.iter().any(|p| p.contains("跳出了 crate")));
    assert!(s.problems.iter().any(|p| p.contains("找不到模块 missing")));
    assert!(
        s.problems
            .iter()
            .any(|p| p.contains("both.rs 与 both/mod.rs"))
    );
    done(&r);
}

/// T14：没有任何声明指到的 `.rs` 是孤儿（只报注释）；带别的 cfg 的缺失文件不报红
#[test]
fn t14_orphans_and_cfg_gated_missing() {
    let (r, s) = tree(
        "t14",
        &[
            ("src/lib.rs", "#[cfg(target_os = \"windows\")]\nmod win;\n"),
            ("src/stray.rs", "fn s() {}\n"),
        ],
    );
    assert!(s.problems.is_empty(), "{:?}", s.problems);
    assert_eq!(s.orphans, ["c/src/stray.rs"]);
    done(&r);
}

/// 语法错的文件报红，不静默跳过
#[test]
fn parse_errors_are_reported() {
    let (r, s) = tree("perr", &[("src/lib.rs", "fn broken( {\n")]);
    assert_eq!(s.problems.len(), 1, "{:?}", s.problems);
    assert!(s.problems[0].contains("语法树解析失败"));
    done(&r);
}

/// codex #71 第 1 轮 P1 #1：同一行里测试项后面的生产代码不算测试（按列判）
#[test]
fn same_line_production_after_test_item() {
    let lib = "#[cfg(test)] fn t() {} fn p() { prod(); }\n";
    let (r, s) = tree("sameline", &[("src/lib.rs", lib)]);
    let file = syn::parse_file(lib).unwrap();
    let ranges = super::modtree::file_test_ranges("c/src/lib.rs", &file);
    let col = lib.find("prod").unwrap();
    assert!(!super::modtree::in_ranges(&ranges, (1, col)), "{ranges:?}");
    assert!(super::modtree::in_ranges(
        &ranges,
        (1, lib.find("fn t").unwrap())
    ));
    let _ = s;
    done(&r);
}

/// codex #71 第 1 轮 P1 #2：内联模块自己的 `#[path]` 决定子模块目录
#[test]
fn inline_module_path_attr() {
    let (r, s) = tree(
        "inlinepath",
        &[
            (
                "src/main.rs",
                "#[path = \"other\"]\nmod outer {\n    mod x;\n}\nfn main() {}\n",
            ),
            (
                "src/lib.rs",
                "#[cfg(test)]\n#[path = \"other/x.rs\"]\nmod x;\n",
            ),
            ("src/other/x.rs", "fn f() {}\n"),
        ],
    );
    assert!(s.problems.is_empty(), "{:?}", s.problems);
    assert!(
        !s.is_test_file("c/src/other/x.rs"),
        "生产引用经 #[path] 内联模块到 other/x.rs"
    );
    done(&r);
}

/// codex #71 第 1 轮 P1 #3：建树错误进 tidy 的问题（红），孤儿进注释——测完整入口 `report`
#[test]
fn report_surfaces_problems_and_orphans() {
    let root = std::env::temp_dir().join(format!("xtask-ast-report-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src/c/src")).unwrap();
    fs::write(root.join("src/c/Cargo.toml"), "[package]\nname = \"c\"\n").unwrap();
    fs::write(root.join("src/c/src/lib.rs"), "mod missing;\n").unwrap();
    fs::write(root.join("src/c/src/stray.rs"), "fn s() {}\n").unwrap();
    let (mut p, mut n) = (Vec::new(), Vec::new());
    super::modtree::report(&root, &mut p, &mut n);
    assert!(p.iter().any(|x| x.contains("找不到模块 missing")), "{p:?}");
    assert!(
        n.iter()
            .any(|x| x.contains("stray.rs") && x.contains("孤儿")),
        "{n:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

/// codex #71 第 2 轮 P1 #1：函数体里带 `#[path]` 的模块也是生产引用
#[test]
fn block_module_with_path_is_production() {
    let (r, s) = tree(
        "blockmod",
        &[
            (
                "src/lib.rs",
                "#[cfg(test)]\nmod shared;\nfn f() {\n    #[path = \"shared.rs\"]\n    mod p;\n}\n",
            ),
            ("src/shared.rs", "fn s() {}\n"),
        ],
    );
    assert!(s.problems.is_empty(), "{:?}", s.problems);
    assert!(!s.is_test_file("c/src/shared.rs"));
    done(&r);
}

/// codex #71 第 2 轮 P1 #3：生产代码里的 `include!` 报红；测试代码里的不报
#[test]
fn production_include_is_an_error() {
    let (r, s) = tree(
        "include",
        &[(
            "src/lib.rs",
            "include!(\"gen.rs\");\nfn f() { include!(\"x.rs\"); }\n#[cfg(test)]\nmod t {\n    include!(\"y.rs\");\n}\n",
        )],
    );
    assert_eq!(s.problems.len(), 2, "{:?}", s.problems);
    assert!(s.problems.iter().all(|p| p.contains("include!")));
    done(&r);
}

/// codex #71 第 3 轮 P1 #1：常量初始化块、trait 默认方法里的模块与 include! 也走到
#[test]
fn modules_and_include_anywhere() {
    let (r, s) = tree(
        "anywhere",
        &[
            (
                "src/lib.rs",
                "#[cfg(test)]\nmod shared;\nconst _: () = {\n    #[path = \"shared.rs\"]\n    mod p;\n};\ntrait T {\n    fn d() {\n        include!(\"x.rs\");\n    }\n}\n",
            ),
            ("src/shared.rs", "fn s() {}\n"),
        ],
    );
    assert!(!s.is_test_file("c/src/shared.rs"));
    assert_eq!(s.problems.len(), 1, "{:?}", s.problems);
    assert!(s.problems[0].contains("include!"));
    done(&r);
}

/// codex #71 第 3 轮 P1 #2（第 4 轮 P2 #4 重写）：同一文件以两种目录上下文、**同一测试状态**加载，两边的子模块都要展开
///（旧去重键（文件，是否测试）会跳过第二次，那边的子模块报「找不到」或成孤儿）；两种声明顺序都测
#[test]
fn same_file_two_contexts() {
    for lib in [
        "mod a;\n#[path = \"a.rs\"]\nmod b;\n",
        "#[path = \"a.rs\"]\nmod b;\nmod a;\n",
    ] {
        let (r, s) = tree(
            "twoctx",
            &[
                ("src/lib.rs", lib),
                ("src/a.rs", "mod child;\n"),
                ("src/a/child.rs", "fn c() {}\n"),
                // `#[path]` 引入的文件按 mod-rs 处理：经 b 加载时它的子模块在文件所在目录
                ("src/child.rs", "fn c() {}\n"),
            ],
        );
        assert!(s.problems.is_empty(), "{lib}: {:?}", s.problems);
        assert!(s.orphans.is_empty(), "{lib}: 孤儿 {:?}", s.orphans);
        assert!(!s.is_test_file("c/src/a/child.rs"));
        assert!(!s.is_test_file("c/src/child.rs"));
        done(&r);
    }
}

/// codex #71 第 4 轮 P1 #1：内联模块的 `cfg_attr` path 也展开，生产引用优先
#[test]
fn inline_module_cfg_attr_path() {
    let (r, s) = tree(
        "inlinecfgattr",
        &[
            (
                "src/lib.rs",
                "#[cfg_attr(not(test), path = \"other\")]\nmod outer {\n    mod x;\n}\n#[cfg(test)]\n#[path = \"other/x.rs\"]\nmod t;\n",
            ),
            ("src/outer/x.rs", "fn a() {}\n"),
            ("src/other/x.rs", "fn b() {}\n"),
        ],
    );
    assert!(
        !s.is_test_file("c/src/other/x.rs"),
        "经 cfg_attr 路径的生产引用"
    );
    assert!(!s.is_test_file("c/src/outer/x.rs"));
    done(&r);
}

/// codex #71 第 3 轮 P1 #3：`cfg_attr` 里的 path 也当引用（保守）
#[test]
fn cfg_attr_path_is_a_reference() {
    let (r, s) = tree(
        "cfgattr",
        &[
            (
                "src/lib.rs",
                "#[cfg(test)]\nmod shared;\n#[cfg_attr(not(test), path = \"shared.rs\")]\nmod p;\n",
            ),
            ("src/shared.rs", "fn s() {}\n"),
            ("src/p.rs", "fn p() {}\n"),
        ],
    );
    assert!(s.problems.is_empty(), "{:?}", s.problems);
    assert!(!s.is_test_file("c/src/shared.rs"));
    assert!(!s.is_test_file("c/src/p.rs"));
    done(&r);
}
