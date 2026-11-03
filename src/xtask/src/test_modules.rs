//! 认出「`#[cfg(test)] mod x;` 引入的文件」：这些文件只在 `cargo test` 编译，tidy 不按实时代码查（R3）。
//!
//! 起因（2026-11-02，split 线）：denoised 的单测搬到 `src/tests.rs` 后，R3 把它当实时代码扫，
//! 只能在文件头加整模块 NRT 标注绕过。这里按模块声明认：
//! - `#[cfg(test)]` 的下一条非空行是 `mod x;`（可带 `pub` / `pub(crate)`）→ 测试模块
//! - 模块文件按 Rust 规则找：声明它的文件是 `lib.rs` / `main.rs` / `mod.rs` 时在同目录，否则在 `<文件名>/` 下；
//!   取 `x.rs` 或 `x/mod.rs`
//! - 测试模块的子模块（`x/` 目录下的全部文件）也算测试代码
//!
//! 边界：`#[path = "…"]` 指定的路径不认；`#[cfg(test)]` 与 `mod` 之间夹了别的属性也不认（宁可多查）。

use std::fs;
use std::path::{Path, PathBuf};

/// `#[cfg(test)]` 后面紧跟的 `mod x;` 的模块名。
pub fn cfg_test_mod_names(src: &str) -> Vec<String> {
    let lines: Vec<&str> = src
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    lines
        .windows(2)
        .filter(|w| w[0] == "#[cfg(test)]")
        .filter_map(|w| {
            let rest = w[1]
                .strip_prefix("pub(crate) ")
                .or_else(|| w[1].strip_prefix("pub "))
                .unwrap_or(w[1]);
            let name = rest.strip_prefix("mod ")?.strip_suffix(';')?.trim();
            (!name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_'))
                .then(|| name.to_string())
        })
        .collect()
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
    let mut out: Vec<PathBuf> = Vec::new();
    for f in files {
        let Ok(src) = fs::read_to_string(f) else {
            continue;
        };
        let Some(dir) = child_dir(f) else { continue };
        for name in cfg_test_mod_names(&src) {
            let flat = dir.join(format!("{name}.rs"));
            let sub = dir.join(&name);
            for g in files {
                if *g == flat || g.starts_with(&sub) {
                    out.push(g.clone());
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_after_cfg_test() {
        let src = "mod a;\n#[cfg(test)]\nmod tests;\n#[cfg(test)]\n\npub(crate) mod helpers;\n#[cfg(test)]\nmod inline {\n}\n";
        assert_eq!(cfg_test_mod_names(src), ["tests", "helpers"]);
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
        fs::write(d.join("src/lib.rs"), "mod worker;\n#[cfg(test)]\nmod tests;\n").unwrap();
        fs::write(d.join("src/tests.rs"), sleep).unwrap();
        fs::write(d.join("src/tests/more.rs"), sleep).unwrap();
        fs::write(d.join("src/worker.rs"), "#[cfg(test)]\nmod wt;\n").unwrap();
        fs::write(d.join("src/worker/wt.rs"), sleep).unwrap();
        let all = files(&d);
        let t = test_module_files(&all);
        assert_eq!(
            t,
            [
                d.join("src/tests.rs"),
                d.join("src/tests/more.rs"),
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
