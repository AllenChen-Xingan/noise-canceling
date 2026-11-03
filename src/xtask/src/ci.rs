//! `cargo run -p xtask -- ci`：本机与 CI 共用的一条闸门（规约 §7 G1–G4、010 第十一节 V6）。
//!
//! 依次跑 fmt --check · clippy --workspace --all-targets -D warnings · test --workspace · tidy · check-deps，
//! 再跑 `cargo deny check`；没装 cargo-deny 算失败（2026-11-02 起不再跳过）。
//! 本机与 CI 跑同一套全量步骤：CI 只跑 macOS（人定 D76，2026-11-02），不再按平台拆分。
//! 每步打印通过 / 失败与耗时；默认跑完全部步骤再汇总，`--fail-fast` 遇到第一步失败就停。任一失败退出码 1。

use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

use crate::root;

enum Outcome {
    Pass,
    Fail,
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
        // 人定（2026-11-02）：不再跳过。CI 由 ci.yml 先装；本机装一次即可
        eprintln!("cargo-deny 未装：先跑 `cargo install cargo-deny --locked`");
        return Outcome::Fail;
    }
    cargo(&["deny", "check"])
}

fn ok(b: bool) -> Outcome {
    if b { Outcome::Pass } else { Outcome::Fail }
}

type Step = (&'static str, Box<dyn Fn() -> Outcome>);

fn steps() -> Vec<Step> {
    vec![
        ("fmt", Box::new(|| cargo(&["fmt", "--all", "--check"]))),
        (
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
        ),
        ("test", Box::new(|| cargo(&["test", "--workspace"]))),
        ("tidy", Box::new(|| ok(crate::run_tidy()))),
        ("check-deps", Box::new(|| ok(crate::run_check_deps()))),
        ("cargo-deny", Box::new(cargo_deny)),
    ]
}

pub fn run(args: &[String]) -> ExitCode {
    let fail_fast = args.iter().any(|a| a == "--fail-fast");
    let mut summary = Vec::new();
    let mut failed = false;
    for (name, f) in steps() {
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
