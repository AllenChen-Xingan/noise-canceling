//! 认出「`#[cfg(test)] mod x;` 引入的文件」：这些文件只在 `cargo test` 编译，tidy 不按实时代码查（R3）。
//!
//! 起因（2026-11-02，split 线）：denoised 的单测搬到 `src/tests.rs` 后，R3 把它当实时代码扫，
//! 只能在文件头加整模块 NRT 标注绕过。这里按模块声明认：
//! - `#[cfg(test)]` 的下一条非空行是 `mod x;`（可带 `pub` / `pub(crate)`）→ 测试模块
//! - 模块文件按 Rust 规则找：声明它的文件是 `lib.rs` / `main.rs` / `mod.rs` 时在同目录，否则在 `<文件名>/` 下；
//!   取 `x.rs` 或 `x/mod.rs`
//! - 测试模块的子模块（`x/` 目录下的全部文件）也算测试代码
//!
//! 边界：`#[path = "…"]` 指定的路径不认；`#[cfg(test)]` 与 `mod` 之间夹了别的属性也不认；
//! 只认文件顶层的声明，内联模块里的不认；注释与字符串里的写法不算（宁可多查）。

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
        return None;
    }
    if after.starts_with(';') {
        Some((name, false))
    } else if after.starts_with('{') {
        Some((name, true))
    } else {
        None
    }
}

/// 花括号净增量（代码已剥掉注释、字符串与字符字面量）。
pub fn brace_delta(code: &str) -> i64 {
    code.chars().fold(0, |d, c| match c {
        '{' => d + 1,
        '}' => d - 1,
        _ => d,
    })
}

/// 文件顶层（花括号深度 0）`#[cfg(test)]` 后面紧跟的 `mod x;` 的模块名。
/// 注释与字符串里的写法不算（codex 第 2 轮 P1 #2）；内联模块里的声明不认（路径不同，宁可多查）。
#[cfg(test)]
fn cfg_test_mod_names(src: &str) -> Vec<String> {
    mod_names(src).0
}

/// （测试模块名，其他模块名）。其他 = 顶层不带 `#[cfg(test)]` 的 `mod x;`，以及内联模块里的任何 `mod x;`
/// （路径没解析，按同名保守算生产引用）。
fn mod_names(src: &str) -> (Vec<String>, Vec<String>) {
    let (mut test, mut other) = (Vec::new(), Vec::new());
    let mut depth = 0i64;
    let mut prev_cfg_test = false;
    for (code, _) in lex_lines(src) {
        let t = code.trim();
        if t.is_empty() {
            continue;
        }
        if let Some((name, false)) = mod_decl(t) {
            if depth == 0 && prev_cfg_test {
                test.push(name);
            } else {
                other.push(name);
            }
        }
        prev_cfg_test = depth == 0 && t == "#[cfg(test)]";
        depth += brace_delta(t);
    }
    (test, other)
}

/// 声明在 `file` 里的子模块放在哪个目录。
fn child_dir(file: &Path) -> Option<PathBuf> {
    let parent = file.parent()?;
    let name = file.file_name()?.to_str()?;
    if matches!(name, "lib.rs" | "main.rs" | "mod.rs") {
        Some(parent.to_path_buf())
    } else {
        Some(parent.join(file.file_stem()?))
    }
}

/// `files` 里哪些是测试模块（或其子模块）的文件。
pub fn test_module_files(files: &[PathBuf]) -> Vec<PathBuf> {
    // 模块名 → 它能指到的文件：`x.rs` 与 `x/` 下全部
    let resolve = |dir: &Path, name: &str| -> Vec<PathBuf> {
        let (flat, sub) = (dir.join(format!("{name}.rs")), dir.join(name));
        files
            .iter()
            .filter(|g| **g == flat || g.starts_with(&sub))
            .cloned()
            .collect()
    };
    let decls: Vec<(PathBuf, PathBuf, Vec<String>, Vec<String>)> = files
        .iter()
        .filter_map(|f| {
            let src = fs::read_to_string(f).ok()?;
            let (t, o) = mod_names(&src);
            Some((f.clone(), child_dir(f)?, t, o))
        })
        .collect();
    let mut out: Vec<PathBuf> = decls
        .iter()
        .flat_map(|(_, dir, t, _)| t.iter().flat_map(|n| resolve(dir, n)))
        .collect();
    out.sort();
    out.dedup();
    // 生产引用：不在测试清单里的文件用普通 `mod x;` 指到的文件不算测试代码（codex 第 3 轮 P1 #2：
    // lib.rs 里 `#[cfg(test)] mod worker;`、main.rs 里普通 `mod worker;` 时，worker.rs 照实时代码查）
    let prod: Vec<PathBuf> = decls
        .iter()
        .filter(|(f, ..)| !out.contains(f))
        // 只指到模块文件本身（`x.rs` / `x/mod.rs`）；它的子模块是不是测试由它自己的声明决定
        .flat_map(|(_, dir, _, o)| {
            o.iter()
                .flat_map(|n| [dir.join(format!("{n}.rs")), dir.join(n).join("mod.rs")])
        })
        .collect();
    out.retain(|f| !prod.contains(f));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_after_cfg_test() {
        let src = "mod a;\n#[cfg(test)]\nmod tests;\n#[cfg(test)]\n\npub(crate) mod helpers;\n#[cfg(test)]\nmod inline {\n}\n";
        assert_eq!(cfg_test_mod_names(src), ["tests", "helpers"]);
        // 注释、字符串里的写法不算；内联模块里的声明不算
        let fake = "/*\n#[cfg(test)]\nmod worker;\n*/\nconst S: &str = \"\n#[cfg(test)]\nmod worker2;\n\";\nmod outer {\n    #[cfg(test)]\n    mod inner;\n}\n";
        assert!(
            cfg_test_mod_names(fake).is_empty(),
            "{:?}",
            cfg_test_mod_names(fake)
        );
    }

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("xtask-tm-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("src/tests")).unwrap();
        fs::create_dir_all(d.join("src/worker")).unwrap();
        d
    }

    fn files(d: &Path) -> Vec<PathBuf> {
        let mut v = Vec::new();
        crate::tidy::rs_files(&d.join("src"), &mut v);
        v
    }

    /// 正例：`#[cfg(test)] mod tests;` 引入的文件与它的子模块算测试代码，R3 不再报
    #[test]
    fn cfg_test_module_files_are_exempt_from_r3() {
        let d = tmp("pos");
        let sleep = "fn f() { std::thread::sleep(std::time::Duration::ZERO); }\n";
        fs::write(
            d.join("src/lib.rs"),
            "mod worker;\n#[cfg(test)]\nmod tests;\n",
        )
        .unwrap();
        fs::write(d.join("src/tests.rs"), sleep).unwrap();
        fs::write(d.join("src/tests/more.rs"), sleep).unwrap();
        fs::write(d.join("src/worker.rs"), "#[cfg(test)]\nmod wt;\n").unwrap();
        fs::write(d.join("src/worker/wt.rs"), sleep).unwrap();
        let all = files(&d);
        let t = test_module_files(&all);
        assert_eq!(
            t,
            [
                d.join("src/tests/more.rs"),
                d.join("src/tests.rs"),
                d.join("src/worker/wt.rs")
            ]
        );
        let mut tidy = crate::tidy::Tidy::default();
        for f in &all {
            let realtime = !t.contains(f);
            crate::tidy::check_file(f, &f.display().to_string(), realtime, &mut tidy);
        }
        assert!(tidy.problems.is_empty(), "{:#?}", tidy.problems);
        let _ = fs::remove_dir_all(&d);
    }

    /// 负例（codex 第 3 轮 P1 #2）：lib.rs 测试引用、main.rs 生产引用同一个文件 → 不当测试代码
    #[test]
    fn file_also_used_by_production_entry_is_not_exempt() {
        let d = tmp("dual");
        fs::write(d.join("src/lib.rs"), "#[cfg(test)]\nmod worker;\n").unwrap();
        fs::write(d.join("src/main.rs"), "mod worker;\nfn main() {}\n").unwrap();
        fs::write(d.join("src/worker.rs"), "fn f() {}\n").unwrap();
        assert!(test_module_files(&files(&d)).is_empty());
        let _ = fs::remove_dir_all(&d);
    }

    /// 负例（会红）：同样的文件，但 `mod tests;` 没有 `#[cfg(test)]` → 照实时代码查，R3 报 thread::sleep
    #[test]
    fn plain_module_is_still_checked() {
        let d = tmp("neg");
        fs::write(d.join("src/lib.rs"), "mod tests;\n").unwrap();
        fs::write(
            d.join("src/tests.rs"),
            "fn f() { std::thread::sleep(std::time::Duration::ZERO); }\n",
        )
        .unwrap();
        let all = files(&d);
        assert!(test_module_files(&all).is_empty());
        let mut tidy = crate::tidy::Tidy::default();
        for f in &all {
            crate::tidy::check_file(f, &f.display().to_string(), true, &mut tidy);
        }
        assert_eq!(tidy.problems.len(), 1, "{:#?}", tidy.problems);
        let _ = fs::remove_dir_all(&d);
    }
}
