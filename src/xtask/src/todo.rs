//! `cargo run -p xtask -- todo`：统计未实现的地方（common 规矩第 6 条的两种写法），按 crate 列出。
//!
//! - `#[ignore = "待实现…"]` 的测试
//! - 非测试代码里**构造** `…::Unimplemented` 错误的地方（注释、枚举定义、`=>` 匹配臂、`(_)` 模式、`tests/` 下的断言都不算）
//!
//! 只是盘点，不判对错，退出码恒为 0。xtask 自身不统计（它的源码里有这两个字样）。

use std::fs;
use std::path::Path;

use crate::deps::metadata;
use crate::root;

const IGNORE_MARK: &str = "#[ignore = \"待实现";

fn is_marker(line: &str, in_tests_dir: bool) -> bool {
    if line.contains(IGNORE_MARK) {
        return true;
    }
    let code = line.split_once("//").map_or(line, |(c, _)| c);
    !in_tests_dir
        && code.contains("::Unimplemented")
        && !code.contains("=>")
        && !code.contains("Unimplemented(_)")
}

pub struct Hit {
    pub krate: String,
    pub at: String,
    pub text: String,
}

fn scan(dir: &Path, krate: &str, root: &Path, out: &mut Vec<Hit>) {
    let mut files = Vec::new();
    crate::tidy::rs_files(dir, &mut files);
    for f in files {
        let Ok(src) = fs::read_to_string(&f) else {
            continue;
        };
        let rel = f.strip_prefix(root).unwrap_or(&f).display().to_string();
        let in_tests_dir = f.components().any(|c| c.as_os_str() == "tests");
        for (i, l) in src.lines().enumerate() {
            if is_marker(l, in_tests_dir) {
                out.push(Hit {
                    krate: krate.to_string(),
                    at: format!("{rel}:{}", i + 1),
                    text: l.trim().to_string(),
                });
            }
        }
    }
}

pub fn collect() -> Result<Vec<Hit>, String> {
    let root = root();
    let meta = metadata()?;
    let mut hits = Vec::new();
    let mut members = meta.members();
    members.sort_by(|a, b| a.1.cmp(&b.1));
    for (_, name, manifest, _) in members {
        if name == "xtask" {
            continue;
        }
        let Some(dir) = Path::new(&manifest).parent() else {
            continue;
        };
        for sub in ["src", "tests", "benches", "examples"] {
            scan(&dir.join(sub), &name, &root, &mut hits);
        }
    }
    Ok(hits)
}

pub fn run() -> std::process::ExitCode {
    match collect() {
        Ok(hits) => {
            let mut cur = "";
            let mut per: Vec<(String, usize)> = Vec::new();
            for h in &hits {
                if h.krate != cur {
                    println!("{}：", h.krate);
                    per.push((h.krate.clone(), 0));
                }
                cur = &h.krate;
                if let Some(last) = per.last_mut() {
                    last.1 += 1;
                }
                println!("  {}  {}", h.at, h.text);
            }
            println!("== 待实现共 {} 处 ==", hits.len());
            for (k, n) in per {
                println!("  {k}: {n}");
            }
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("todo 跑不起来：{e}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_marker;

    #[test]
    fn only_real_markers_count() {
        assert!(is_marker("#[ignore = \"待实现：loom\"]", true));
        assert!(is_marker("    Err(RtError::Unimplemented(\"x\"))", false));
        assert!(
            !is_marker("    Unimplemented(&'static str),", false),
            "枚举定义"
        );
        assert!(
            !is_marker("//! 返回 `RtError::Unimplemented`", false),
            "注释"
        );
        assert!(
            !is_marker("Self::Unimplemented(w) => write!(f, \"{w}\"),", false),
            "匹配臂"
        );
        assert!(
            !is_marker("matches!(e, Err(RtError::Unimplemented(_)))", false),
            "模式"
        );
        assert!(
            !is_marker("Err(RtError::Unimplemented(\"x\"))", true),
            "tests/ 下的断言"
        );
    }
}
