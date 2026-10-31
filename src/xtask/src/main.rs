//! xtask：项目规矩的机器检查（specs/rust/architecture.md §5.1「静态·依赖闭包」「静态·项目约定」）。
//!
//! - `cargo run -p xtask -- check-deps`：每个 crate 的**普通依赖闭包**与它 `[package.metadata.vdn].forbid` 求交，非空即红（R12）
//! - `cargo run -p xtask -- tidy`：文本 / 轻语法检查（R3 · R4 · R10 · R14 与 §5.3 的 NRT 例外写法）
//!
//! 两样都随 `cargo test -p xtask` 跑（照 rust-analyzer：架构规则即测试）。不依赖任何 vdn crate。

#![forbid(unsafe_code)]

mod deps;
mod tidy;

use std::path::PathBuf;
use std::process::ExitCode;

/// workspace 根（xtask 在 `src/xtask/`）。
pub fn root() -> PathBuf {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.ancestors().nth(2).map(PathBuf::from).unwrap_or(here)
}

fn report(name: &str, problems: &[String]) -> ExitCode {
    if problems.is_empty() {
        println!("{name}: 通过");
        ExitCode::SUCCESS
    } else {
        eprintln!("{name}: {} 处问题", problems.len());
        for p in problems {
            eprintln!("  - {p}");
        }
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    match cmd.as_str() {
        "check-deps" => match deps::check() {
            Ok(p) => report("check-deps", &p),
            Err(e) => {
                eprintln!("check-deps 跑不起来：{e}");
                ExitCode::FAILURE
            }
        },
        "tidy" => match tidy::check() {
            Ok(t) => {
                println!("NRT 例外（allow）共 {} 处", t.nrt_allows);
                for n in &t.notes {
                    println!("注：{n}");
                }
                report("tidy", &t.problems)
            }
            Err(e) => {
                eprintln!("tidy 跑不起来：{e}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("用法：cargo run -p xtask -- <check-deps | tidy>");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_deps_passes() {
        let p = super::deps::check().expect("cargo metadata");
        assert!(p.is_empty(), "{p:#?}");
    }

    #[test]
    fn tidy_passes() {
        let t = super::tidy::check().expect("tidy");
        assert!(t.problems.is_empty(), "{:#?}", t.problems);
    }
}
