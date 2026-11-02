//! `cargo run -p xtask -- todo`：统计未实现的地方（common 规矩第 6 条的两种写法），按 crate 列出。
//!
//! - `#[ignore = "待实现…"]` 的测试
//! - 代码里出现 `Unimplemented`（返回明确的未实现错误）
//!
//! 只是盘点，不判对错，退出码恒为 0。xtask 自身不统计（它的源码里有这两个字样）。

use std::fs;
use std::path::Path;

use crate::deps::metadata;
use crate::root;

const MARKS: [&str; 2] = ["#[ignore = \"待实现", "Unimplemented"];

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
        for (i, l) in src.lines().enumerate() {
            if MARKS.iter().any(|m| l.contains(m)) {
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
