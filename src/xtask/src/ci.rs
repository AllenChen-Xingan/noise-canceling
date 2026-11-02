//! `cargo run -p xtask -- ci`：本机与 CI 共用的一条闸门（规约 §7 G1–G4、010 第十一节 V6）。
//!
//! 依次跑 fmt --check · clippy --workspace --all-targets -D warnings · test --workspace · tidy · check-deps，
//! 再看本机有没有 cargo-deny：有就跑 `cargo deny check`，没有就跳过（不算失败，打印「未装」）。
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

pub fn run(args: &[String]) -> ExitCode {
    let fail_fast = args.iter().any(|a| a == "--fail-fast");
    type Step = (&'static str, fn() -> Outcome);
    let steps: [Step; 6] = [
        ("fmt", || cargo(&["fmt", "--all", "--check"])),
        ("clippy", || {
            cargo(&[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ])
        }),
        ("test", || cargo(&["test", "--workspace"])),
        ("tidy", || ok(crate::run_tidy())),
        ("check-deps", || ok(crate::run_check_deps())),
        ("cargo-deny", cargo_deny),
    ];
    let mut summary = Vec::new();
    let mut failed = false;
    for (name, f) in steps {
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
