//! `cargo run -p xtask -- ci`：本机与 CI 共用的一条闸门（规约 §7 G1–G4、010 第十一节 V6）。
//!
//! 依次跑 fmt --check · clippy --workspace --all-targets -D warnings · test --workspace · tidy · check-deps，
//! 再看本机有没有 cargo-deny：有就跑 `cargo deny check`，没有就跳过（不算失败，打印「未装」）。
//! `--platform linux`：平台无关闸门，排除 macOS 专属 crate（Linux runner 用）；`--platform macos`：只跑平台 crate 的
//! clippy（aarch64-apple-darwin）与测试（macOS runner 用，规约 §7「CI 费用」）。不给 = 全部（本机默认）。
//! 每步打印通过 / 失败与耗时；默认跑完全部步骤再汇总，`--fail-fast` 遇到第一步失败就停。任一失败退出码 1。

use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

use crate::root;

enum Outcome {
    Pass,
    Fail,
    Skip(&'static str),
}

fn cargo(args: &[&str]) -> Outcome {
    let bin = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    match Command::new(bin).args(args).current_dir(root()).status() {
        Ok(s) if s.success() => Outcome::Pass,
        Ok(_) => Outcome::Fail,
        Err(e) => {
            eprintln!("起不来 cargo {}：{e}", args.join(" "));
            Outcome::Fail
        }
    }
}

fn cargo_deny() -> Outcome {
    let bin = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let installed = Command::new(&bin)
        .args(["deny", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !installed {
        return Outcome::Skip("本机未装 cargo-deny");
    }
    cargo(&["deny", "check"])
}

fn ok(b: bool) -> Outcome {
    if b { Outcome::Pass } else { Outcome::Fail }
}

/// 只在 macOS 上编得过的 crate（cpal / CoreAudio）：Linux job 排除。还不存在的名字 cargo 只给警告。
const MACOS_ONLY: &[&str] = &["vdn-io-macos", "vdn-capture"];
/// macOS job 要跑的平台相关 crate（规约 §7「CI 费用」：clippy aarch64-apple-darwin + 这些的测试）。
const MACOS_CRATES: &[&str] = &["vdn-io-macos", "vdn-shm", "vdn-rt-thread", "vdn-hal"];

type Step = (&'static str, Box<dyn Fn() -> Outcome>);

fn owned(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn cargo_v(args: Vec<String>) -> Outcome {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    cargo(&a)
}

fn excludes() -> Vec<String> {
    MACOS_ONLY
        .iter()
        .flat_map(|c| ["--exclude".to_string(), c.to_string()])
        .collect()
}

fn packages() -> Vec<String> {
    MACOS_CRATES
        .iter()
        .flat_map(|c| ["-p".to_string(), c.to_string()])
        .collect()
}

/// `--platform`：`linux` = 平台无关闸门（排除 macOS 专属 crate）· `macos` = 平台部分 · 不给 = 全部（本机默认）。
fn steps(platform: &str) -> Vec<Step> {
    let mut v: Vec<Step> = Vec::new();
    match platform {
        "macos" => {
            v.push((
                "clippy（aarch64-apple-darwin，平台 crate）",
                Box::new(|| {
                    let mut a = owned(&[
                        "clippy",
                        "--target",
                        "aarch64-apple-darwin",
                        "--all-targets",
                    ]);
                    a.extend(packages());
                    a.extend(owned(&["--", "-D", "warnings"]));
                    cargo_v(a)
                }),
            ));
            v.push((
                "test（平台 crate）",
                Box::new(|| {
                    let mut a = owned(&["test"]);
                    a.extend(packages());
                    cargo_v(a)
                }),
            ));
        }
        "linux" => {
            v.push(("fmt", Box::new(|| cargo(&["fmt", "--all", "--check"]))));
            v.push((
                "clippy（排除 macOS 专属）",
                Box::new(|| {
                    let mut a = owned(&["clippy", "--workspace", "--all-targets"]);
                    a.extend(excludes());
                    a.extend(owned(&["--", "-D", "warnings"]));
                    cargo_v(a)
                }),
            ));
            v.push((
                "test（排除 macOS 专属）",
                Box::new(|| {
                    let mut a = owned(&["test", "--workspace"]);
                    a.extend(excludes());
                    cargo_v(a)
                }),
            ));
            v.push(("tidy", Box::new(|| ok(crate::run_tidy()))));
            v.push(("check-deps", Box::new(|| ok(crate::run_check_deps()))));
            v.push(("cargo-deny", Box::new(cargo_deny)));
        }
        _ => {
            v.push(("fmt", Box::new(|| cargo(&["fmt", "--all", "--check"]))));
            v.push((
                "clippy",
                Box::new(|| {
                    cargo(&[
                        "clippy",
                        "--workspace",
                        "--all-targets",
                        "--",
                        "-D",
                        "warnings",
                    ])
                }),
            ));
            v.push(("test", Box::new(|| cargo(&["test", "--workspace"]))));
            v.push(("tidy", Box::new(|| ok(crate::run_tidy()))));
            v.push(("check-deps", Box::new(|| ok(crate::run_check_deps()))));
            v.push(("cargo-deny", Box::new(cargo_deny)));
        }
    }
    v
}

pub fn run(args: &[String]) -> ExitCode {
    let fail_fast = args.iter().any(|a| a == "--fail-fast");
    let platform = args
        .iter()
        .position(|a| a == "--platform")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or("all");
    if !["all", "linux", "macos"].contains(&platform) {
        eprintln!("ci：--platform 只认 linux / macos（不给 = 全部）");
        return ExitCode::FAILURE;
    }
    println!("== ci：平台 {platform} ==");
    let mut summary = Vec::new();
    let mut failed = false;
    for (name, f) in steps(platform) {
        println!("== ci：{name} ==");
        let t = Instant::now();
        let out = f();
        let secs = t.elapsed().as_secs_f64();
        let line = match out {
            Outcome::Pass => format!("通过  {name}（{secs:.1} s）"),
            Outcome::Fail => {
                failed = true;
                format!("失败  {name}（{secs:.1} s）")
            }
            Outcome::Skip(why) => format!("跳过  {name}（{why}）"),
        };
        println!("{line}");
        summary.push(line);
        if failed && fail_fast {
            break;
        }
    }
    println!("== ci 汇总 ==");
    for l in &summary {
        println!("  {l}");
    }
    if failed {
        println!("ci：不通过");
        ExitCode::FAILURE
    } else {
        println!("ci：全绿");
        ExitCode::SUCCESS
    }
}
