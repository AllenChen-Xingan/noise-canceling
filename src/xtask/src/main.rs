//! xtask：项目规矩的机器检查（specs/rust/architecture.md §5.1「静态·依赖闭包」「静态·项目约定」）。
//!
//! - `cargo run -p xtask -- check-deps`：每个 crate 的**普通依赖闭包**与它 `[package.metadata.vdn].forbid` 求交，非空即红（R12）
//! - `cargo run -p xtask -- tidy`：文本 / 轻语法检查（R3 · R4 · R10 · R14 与 §5.3 的 NRT 例外写法）
//! - `cargo run -p xtask -- eval ci|qa [选项…]`：转调 `cargo run -p vdn-eval -- ci|qa`（xtask 不依赖任何 vdn crate，只起子进程）。
//!   `eval ci` = CI 层（纯 DSP、无模型、门禁不过退出码 1）；`eval qa` = 本机 QA 层（ASR、DNSMOS、真实片段）
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

/// `eval ci|qa …`：起 `cargo run -p vdn-eval -- …`，把退出码原样传回。
fn eval(rest: &[String]) -> ExitCode {
    let Some(layer) = rest.first().filter(|l| *l == "ci" || *l == "qa") else {
        eprintln!("用法：cargo run -p xtask -- eval <ci | qa> [vdn-eval 的选项]");
        return ExitCode::FAILURE;
    };
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    // QA 层要跑模型，用 release；CI 层用 debug，复用 cargo test 已有的构建产物
    let mut cmd = std::process::Command::new(cargo);
    cmd.current_dir(root())
        .args(["run", "-q", "-p", "vdn-eval"]);
    if layer == "qa" {
        cmd.arg("--release");
    }
    cmd.arg("--").args(rest);
    match cmd.status() {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("起不来 vdn-eval：{e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().cloned().unwrap_or_default();
    match cmd.as_str() {
        "eval" => eval(&args[1..]),
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
            eprintln!("用法：cargo run -p xtask -- <check-deps | tidy | eval ci|qa>");
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
