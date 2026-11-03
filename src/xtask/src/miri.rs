//! `cargo run -p xtask -- miri [crate…]`：用 nightly 的 Miri 跑无锁结构的测试（规约 R6）。
//!
//! - 默认只跑 `vdn-shm`（自写无锁环只许在这里）；可另列 crate，例如 `miri vdn-shm vdn-dsp`
//! - 只跑 heap 版测试：POSIX 共享内存 / 跨进程 / 依赖真实时钟的测试在 Miri 下标了 `cfg_attr(miri, ignore)`
//! - 慢（几分钟），**不进默认 `ci`**，CI 里也先不加；本机要先装：`rustup toolchain install nightly --component miri`
//! - 不改 `rust-toolchain.toml`：用 `cargo +nightly` 临时切

use std::process::{Command, ExitCode, Stdio};

use crate::root;

fn nightly_miri_installed() -> bool {
    Command::new("cargo")
        .args(["+nightly", "miri", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn run(args: &[String]) -> ExitCode {
    if !nightly_miri_installed() {
        eprintln!(
            "miri: 本机没有 nightly 的 Miri；先跑 `rustup toolchain install nightly --component miri`"
        );
        return ExitCode::FAILURE;
    }
    let crates: Vec<&str> = if args.is_empty() {
        vec!["vdn-shm"]
    } else {
        args.iter().map(String::as_str).collect()
    };
    let mut ok = true;
    for c in crates {
        let t = std::time::Instant::now();
        // 用 rustup 的 cargo 代理（不用 $CARGO：那是 stable 的 cargo 本体，认不得 +nightly）
        let s = Command::new("cargo")
            .args(["+nightly", "miri", "test", "-p", c])
            .current_dir(root())
            .status();
        let pass = s.is_ok_and(|s| s.success());
        println!(
            "miri {c}: {}（{:.0} s）",
            if pass { "通过" } else { "失败" },
            t.elapsed().as_secs_f64()
        );
        ok &= pass;
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
