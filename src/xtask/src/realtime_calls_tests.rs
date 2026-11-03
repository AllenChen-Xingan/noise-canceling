//! R3 的测试：012 第五节第 3 表 S1–S4 与旧文本规则认的写法。

use super::*;

fn n(src: &str) -> usize {
    hits("src/dsp/src/a.rs", src).unwrap().len()
}

/// 旧规则认的三种写法照样认（含 use 行）
#[test]
fn old_forms_still_caught() {
    for src in [
        "fn f() { let _ = std::fs::read(\"a\"); }\n",
        "use std::fs;\n",
        "fn f() { log::info!(\"x\"); }\n",
        "fn f() { std::thread::sleep(d); }\n",
        "use std::thread;\nfn f() { thread::sleep(d); }\n",
    ] {
        assert!(n(src) >= 1, "{src}");
    }
}

/// S1：`use … as` 改名后的调用；`use` 分组与 self
#[test]
fn s1_aliases() {
    let src = "use std::thread::sleep as nap;\nfn f() {\n    nap(d);\n}\n";
    let h = hits("src/dsp/src/a.rs", src).unwrap();
    assert_eq!(
        h.iter().map(|(l, _)| *l).collect::<Vec<_>>(),
        [1, 3],
        "{h:?}"
    );
    assert_eq!(
        n("use std::{fs::{self as f}, io};\nfn g() { f::read(p); }\n"),
        2
    );
    assert_eq!(n("use log::info;\nfn g() { info!(\"x\"); }\n"), 2);
}

/// S2：把禁用函数当值引用
#[test]
fn s2_function_as_value() {
    assert_eq!(
        n("fn f() {\n    let s = std::thread::sleep;\n    s(d);\n}\n"),
        1
    );
}

/// S3：宏参数里的调用；`::std::fs` 前导冒号
#[test]
fn s3_inside_macros_and_leading_colons() {
    assert_eq!(n("fn f() {\n    dbg!(std::fs::read(p));\n}\n"), 1);
    assert_eq!(n("fn f() { let _ = ::std::fs::read(p); }\n"), 1);
}

/// S4（正例）：测试代码不算——测试模块、`#[test]` 函数、测试模块之后的生产代码照查
#[test]
fn s4_test_code_is_exempt_but_code_after_is_not() {
    let src = "#[cfg(test)]\nmod tests {\n    fn g() { std::thread::sleep(d); }\n}\nfn after() { std::thread::sleep(d); }\n";
    let h = hits("src/dsp/src/a.rs", src).unwrap();
    assert_eq!(h, [(5, "thread::sleep")]);
    assert_eq!(n("#[test]\nfn t() { std::fs::read(p); }\n"), 0);
}

/// 正例：注释、字符串、文档注释不算；别的 `sleep` / `fs` 不算
#[test]
fn comments_strings_and_other_paths_pass() {
    for src in [
        "// std::fs::read\nconst A: u8 = 1;\n",
        "const S: &str = \"std::thread::sleep\";\n",
        "/// 不许 std::fs\nconst A: u8 = 1;\n",
        "fn f() { tokio::time::sleep(d); my::fs::x(); }\n",
        "use std::thread;\nfn f() { thread::yield_now(); }\n",
        // 叫 log / fs 的局部变量（speaker/judge.rs 的误报，迁移比对时发现）
        "fn f() { let mut log = L::new(); log.x = 1; let fs = 2; }\n",
    ] {
        assert_eq!(n(src), 0, "{src}");
    }
}

/// 解析不了的文件报错，不静默跳过
#[test]
fn parse_errors_are_reported() {
    assert!(hits("src/dsp/src/a.rs", "fn broken( {\n").is_err());
}

/// 交叉校验：根 clippy.toml 里属于 R3 的禁用项（sleep）在 BANNED 里（CLAUDE.md 第五节第 5 条）
#[test]
fn banned_list_matches_clippy_toml() {
    let toml = std::fs::read_to_string(crate::root().join("clippy.toml")).unwrap();
    for line in toml
        .lines()
        .filter(|l| l.contains("（R3）") && l.contains("path ="))
    {
        let path = line.split('"').nth(1).unwrap_or("");
        let segs: Vec<&str> = path.split("::").collect();
        // 宏（println 等）由 clippy 的 disallowed-macros 管，不在 R3 的路径前缀里
        if toml[..toml.find(line).unwrap_or(0)].rfind("disallowed-macros")
            > toml[..toml.find(line).unwrap_or(0)].rfind("disallowed-methods")
        {
            continue;
        }
        assert!(
            BANNED
                .iter()
                .any(|(p, _)| segs.len() >= p.len()
                    && segs.iter().zip(p.iter()).all(|(a, b)| a == b)),
            "clippy.toml 的 R3 禁用项 {path} 不在 BANNED 里"
        );
    }
}
