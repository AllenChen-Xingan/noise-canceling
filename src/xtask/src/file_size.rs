//! tidy 的「单文件行数上限」（R-SIZE；CLAUDE.md 第五节第 7 条；issue #62）。
//!
//! | 规则 | 查什么 | 怎么过 |
//! |---|---|---|
//! | R-SIZE | [`SCAN_ROOTS`] 下每个代码文件（[`CODE_EXTS`]）≤ [`MAX_LINES`] 行 | 按职责拆（类型 / 参数解析 / 运行 / 输出 / 测试分文件），不许机械切 part1/part2，不许把多行挤成一行 |
//!
//! 硬上限：不设基线、不留豁免（人定 2026-11-02）。#62 过渡期用过只降不升的基线，存量拆完后删掉了。
//! 读目录或文件出错照实报，不当成「没有文件」。

use std::fs;
use std::path::{Path, PathBuf};

/// 单文件行数上限。
pub const MAX_LINES: usize = 600;
/// R-SIZE 扫描的目录（相对仓库根），唯一一处。
pub const SCAN_ROOTS: [&str; 2] = ["src", "docs/experiments"];
/// 算「代码文件」的扩展名（CLAUDE.md 第五节第 7 条「`.rs` 等」）。加新语言时加在这里。
pub const CODE_EXTS: [&str; 10] = ["rs", "py", "sh", "js", "ts", "swift", "c", "h", "m", "mm"];

/// 一个文件：相对路径、行数。
pub type Sized = (String, usize);

/// 超线的文件。
pub fn size_problems(files: &[Sized]) -> Vec<String> {
    files
        .iter()
        .filter(|(_, n)| *n > MAX_LINES)
        .map(|(rel, n)| {
            format!(
                "{rel}：{n} 行，超过上限 {MAX_LINES} 行（R-SIZE）。按职责拆成几个模块（类型 / 参数解析 / 运行 / 输出 / 测试），每个 ≤ {MAX_LINES} 行"
            )
        })
        .collect()
}

/// 递归列出代码文件；读目录出错就报错（不静默跳过）。
fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let rd = fs::read_dir(dir).map_err(|e| format!("读目录 {}：{e}", dir.display()))?;
    let mut entries = Vec::new();
    for e in rd {
        entries.push(
            e.map_err(|e| format!("读目录 {}：{e}", dir.display()))?
                .path(),
        );
    }
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out)?;
        } else if p
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| CODE_EXTS.contains(&e))
        {
            out.push(p);
        }
    }
    Ok(())
}

/// [`SCAN_ROOTS`] 下全部代码文件的（相对路径，行数）；任何文件读不了都报错。
pub fn scan(root: &Path) -> Result<Vec<Sized>, String> {
    let mut all = Vec::new();
    for r in SCAN_ROOTS {
        walk(&root.join(r), &mut all)?;
    }
    all.into_iter()
        .map(|f| {
            let rel = f
                .strip_prefix(root)
                .map_err(|_| format!("{} 不在仓库根下", f.display()))?
                .display()
                .to_string();
            let n = fs::read_to_string(&f)
                .map_err(|e| format!("读 {rel}：{e}"))?
                .lines()
                .count();
            Ok((rel, n))
        })
        .collect()
}

pub fn check(root: &Path, problems: &mut Vec<String>) {
    match scan(root) {
        Ok(files) => problems.extend(size_problems(&files)),
        Err(e) => problems.push(format!("R-SIZE 扫描失败：{e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(p: &str, n: usize) -> Sized {
        (p.to_string(), n)
    }

    /// 负例（会红）：601 行
    #[test]
    fn file_over_limit_is_flagged() {
        let p = size_problems(&[f("src/a.rs", 601)]);
        assert_eq!(p.len(), 1);
        assert!(p[0].contains("src/a.rs") && p[0].contains("601"), "{p:?}");
    }

    #[test]
    fn file_at_limit_passes() {
        assert!(size_problems(&[f("src/a.rs", 600)]).is_empty());
    }

    /// 负例：docs/experiments 下的代码、非 Rust 的代码文件也查；不是代码的文本文件不查
    #[test]
    fn docs_experiments_and_other_languages_are_scanned() {
        let d = std::env::temp_dir().join(format!("xtask-size-docs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("src")).unwrap();
        fs::create_dir_all(d.join("docs/experiments/e1/src")).unwrap();
        fs::write(d.join("docs/experiments/e1/src/big.rs"), "//\n".repeat(601)).unwrap();
        fs::write(d.join("docs/experiments/e1/src/ok.rs"), "//\n".repeat(600)).unwrap();
        fs::write(d.join("docs/experiments/e1/run.py"), "#\n".repeat(601)).unwrap();
        fs::write(d.join("docs/experiments/e1/notes.md"), "x\n".repeat(900)).unwrap();
        let p = size_problems(&scan(&d).unwrap());
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(p.iter().any(|x| x.contains("docs/experiments/e1/src/big.rs")), "{p:?}");
        assert!(p.iter().any(|x| x.contains("docs/experiments/e1/run.py")), "{p:?}");
        let _ = fs::remove_dir_all(&d);
    }

    /// 读不了的目录报错，不当成「没有文件」
    #[test]
    fn scan_reports_missing_src() {
        let d = std::env::temp_dir().join(format!("xtask-size-{}", std::process::id()));
        let _ = fs::create_dir_all(&d);
        assert!(scan(&d).is_err());
        let _ = fs::remove_dir_all(&d);
    }
}
