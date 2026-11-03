//! 认出「只在 `cargo test` 编译的单测文件」：tidy 不按实时代码查它们（R3），底层 crate 的 R-RATE 也放行。
//!
//! 规则（窄，大总管 8d 定 2026-11-02）：文件是 `tests.rs`、`tests/mod.rs` 或 `tests/` 下的子模块，
//! **并且**它的父模块文件在顶层（花括号深度 0，注释与字符串不算）写了 `#[cfg(test)]` 紧跟 `mod tests;`，
//! **并且**同目录没有别的文件用普通 `mod tests;` 引用它。其余一律照实时代码查；需要例外的文件，
//! 在文件头写整模块 NRT 标注（`src/capture/src/testing.rs` 的先例）。
//!
//! 为什么选窄规则：最初的宽规则认「任何 `#[cfg(test)] mod x;` 引入的文件」，codex 第 2–5 轮
//! 连续挑出 4 个绕过点（注释里的声明、另一个入口的生产引用、生产引用的传递、内联模块路径），
//! 每补一处代码就更复杂。为省一行文件头标注而维护一套 Rust 模块解析不值得（复杂度规矩）。
//! 窄规则只认一个固定名字，误放生产代码的面小到能一眼看完。
//!
//! 边界：`#[path = "…"]` 不认；`#[cfg(test)]` 与 `mod tests;` 之间夹了别的属性也不认（宁可多查）。

use std::fs;
use std::path::{Path, PathBuf};

use crate::single_source::lex_lines;

/// 一行代码（已剥掉注释与字符串）是不是 `mod x;` / `mod x {` 声明；返回模块名与是否内联。
pub fn mod_decl(code: &str) -> Option<(String, bool)> {
    let t = code.trim();
    let rest = t
        .strip_prefix("pub(crate) ")
        .or_else(|| t.strip_prefix("pub(super) "))
        .or_else(|| t.strip_prefix("pub "))
        .unwrap_or(t)
        .strip_prefix("mod ")?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let after = rest[name.len()..].trim_start();
    if name.is_empty() {
        None
    } else if after.starts_with(';') {
        Some((name, false))
    } else if after.starts_with('{') {
        Some((name, true))
    } else {
        None
    }
}

/// 顶层的 `mod tests;` 声明：（带 `#[cfg(test)]` 的有没有，不带的有没有）。
fn tests_decls(src: &str) -> (bool, bool) {
    let (mut gated, mut plain) = (false, false);
    let mut depth = 0i64;
    let mut prev_cfg_test = false;
    for (code, _) in lex_lines(src) {
        let t = code.trim();
        if t.is_empty() {
            continue;
        }
        if depth == 0 && mod_decl(t).is_some_and(|(n, inline)| n == "tests" && !inline) {
            if prev_cfg_test {
                gated = true;
            } else {
                plain = true;
            }
        }
        prev_cfg_test = depth == 0 && t == "#[cfg(test)]";
        depth += t.matches('{').count() as i64 - t.matches('}').count() as i64;
    }
    (gated, plain)
}

/// 声明在 `file` 里的子模块放在哪个目录。
fn child_dir(file: &Path) -> Option<PathBuf> {
    let name = file.file_name()?.to_str()?;
    if matches!(name, "lib.rs" | "main.rs" | "mod.rs") {
        file.parent().map(Path::to_path_buf)
    } else {
        Some(file.parent()?.join(file.file_stem()?))
    }
}

/// `files` 里哪些是单测文件（规则见文件头）。
pub fn test_module_files(files: &[PathBuf]) -> Vec<PathBuf> {
    // 每个父模块目录：有没有带 cfg(test) 的 `mod tests;`、有没有普通的
    let mut dirs: Vec<(PathBuf, bool, bool)> = Vec::new();
    for f in files {
        let (Ok(src), Some(dir)) = (fs::read_to_string(f), child_dir(f)) else {
            continue;
        };
        let (g, p) = tests_decls(&src);
        match dirs.iter_mut().find(|(d, ..)| *d == dir) {
            Some(e) => (e.1, e.2) = (e.1 || g, e.2 || p),
            None => dirs.push((dir, g, p)),
        }
    }
    let mut out: Vec<PathBuf> = files
        .iter()
        .filter(|f| {
            dirs.iter().any(|(d, gated, plain)| {
                *gated && !*plain && (**f == d.join("tests.rs") || f.starts_with(d.join("tests")))
            })
        })
        .cloned()
        .collect();
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("xtask-tm-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("src/tests")).unwrap();
        d
    }

    fn files(d: &Path) -> Vec<PathBuf> {
        let mut v = Vec::new();
        crate::tidy::rs_files(&d.join("src"), &mut v);
        v
    }

    const SLEEP: &str = "fn f() { std::thread::sleep(std::time::Duration::ZERO); }\n";

    /// 跑一遍 R3：测试清单里的文件按非实时查，其余按实时查；返回问题数。
    /// 传给 check_file 的路径去掉 `/tests/`，免得它按「集成测试目录」另行放过
    fn r3(d: &Path) -> usize {
        let all = files(d);
        let t = test_module_files(&all);
        let mut tidy = crate::tidy::Tidy::default();
        for f in &all {
            let rel = f.display().to_string().replace("/tests/", "/t/");
            crate::tidy::check_file(f, &rel, !t.contains(f), &mut tidy);
        }
        tidy.problems.len()
    }

    /// 正例：`#[cfg(test)] mod tests;` 引入的 tests.rs 与 tests/ 下的子模块算单测，R3 不报
    #[test]
    fn cfg_test_mod_tests_is_exempt() {
        let d = tmp("pos");
        fs::write(d.join("src/lib.rs"), "#[cfg(test)]\nmod tests;\n").unwrap();
        fs::write(d.join("src/tests.rs"), SLEEP).unwrap();
        fs::write(d.join("src/tests/more.rs"), SLEEP).unwrap();
        assert_eq!(
            test_module_files(&files(&d)),
            [d.join("src/tests/more.rs"), d.join("src/tests.rs")]
        );
        assert_eq!(r3(&d), 0);
        let _ = fs::remove_dir_all(&d);
    }

    /// 负例（会红）：名字不是 tests 的 `#[cfg(test)] mod helper;` 照实时代码查
    #[test]
    fn other_name_is_checked() {
        let d = tmp("name");
        fs::write(d.join("src/lib.rs"), "#[cfg(test)]\nmod helper;\n").unwrap();
        fs::write(d.join("src/helper.rs"), SLEEP).unwrap();
        assert!(test_module_files(&files(&d)).is_empty());
        assert_eq!(r3(&d), 1);
        let _ = fs::remove_dir_all(&d);
    }

    /// 负例（会红）：没有 `#[cfg(test)]` 的 `mod tests;` 照实时代码查
    #[test]
    fn plain_mod_tests_is_checked() {
        let d = tmp("plain");
        fs::write(d.join("src/lib.rs"), "mod tests;\n").unwrap();
        fs::write(d.join("src/tests.rs"), SLEEP).unwrap();
        assert!(test_module_files(&files(&d)).is_empty());
        assert_eq!(r3(&d), 1);
        let _ = fs::remove_dir_all(&d);
    }

    /// 负例：注释、字符串、内联模块里的写法不算；另一个入口普通引用 tests 时也不放行
    #[test]
    fn fake_or_shared_declarations_are_checked() {
        for lib in [
            "/*\n#[cfg(test)]\nmod tests;\n*/\n",
            "const S: &str = \"\n#[cfg(test)]\nmod tests;\n\";\n",
            "mod outer {\n    #[cfg(test)]\n    mod tests;\n}\n",
        ] {
            let d = tmp("fake");
            fs::write(d.join("src/lib.rs"), lib).unwrap();
            fs::write(d.join("src/tests.rs"), SLEEP).unwrap();
            assert!(test_module_files(&files(&d)).is_empty(), "{lib}");
            let _ = fs::remove_dir_all(&d);
        }
        let d = tmp("dual");
        fs::write(d.join("src/lib.rs"), "#[cfg(test)]\nmod tests;\n").unwrap();
        fs::write(d.join("src/main.rs"), "mod tests;\nfn main() {}\n").unwrap();
        fs::write(d.join("src/tests.rs"), SLEEP).unwrap();
        assert!(test_module_files(&files(&d)).is_empty());
        let _ = fs::remove_dir_all(&d);
    }
}
