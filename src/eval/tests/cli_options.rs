//! #191：命令行遇到不认识的选项要报错退出，不能悄悄忽略、按默认值跑（把 --system 写成 --systems 会打错系统）。

use std::process::Command;

fn run(args: &[&str]) -> (bool, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_vdn-eval"))
        .args(args)
        .output()
        .unwrap();
    (
        o.status.success(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

/// 拼错的选项：非 0 退出，stderr 点出选项名；在做任何事之前就拒（清单路径不存在也先报选项）。
#[test]
fn misspelled_option_is_refused_by_name() {
    for args in [
        &[
            "scorecard",
            "--suite",
            "/nonexistent/suite.json",
            "--systems",
            "gtcrn",
        ][..],
        &["compare", "--a", "x", "--b", "y", "--by-sceen"][..],
        &[
            "asr-probe",
            "--wav-dir",
            ".",
            "--cache",
            "x",
            "--out",
            "y",
            "--temprature",
            "0",
        ][..],
        &["ci", "--sytem", "ref"][..],
    ] {
        let (ok, err) = run(args);
        let bad = args
            .iter()
            .rev()
            .find(|a| {
                a.starts_with("--")
                    && ["--systems", "--by-sceen", "--temprature", "--sytem"].contains(a)
            })
            .unwrap();
        assert!(!ok, "{args:?} 应该失败");
        assert!(err.contains(bad), "{args:?} 的报错要点出 {bad}：{err}");
    }
}

/// 多出来的位置参数同样拒（`scorecard gtcrn` 不是在选系统）。
#[test]
fn stray_positional_is_refused() {
    let (ok, err) = run(&["scorecard", "gtcrn", "--suite", "/nonexistent/suite.json"]);
    assert!(!ok);
    assert!(err.contains("gtcrn"), "{err}");
}
