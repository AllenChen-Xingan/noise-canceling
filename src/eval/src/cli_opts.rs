//! 每个子命令认哪些选项（正本，#191）：命令行在做任何事之前先按这张表核一遍，不认识的选项、多出来的位置参数
//! 一律报错退出，不悄悄忽略、按默认值跑（把 `--system` 写成 `--systems` 会打错系统，成绩单看着正常、结论却错）。
//!
//! 和代码对得上靠测试 `table_matches_sources`：扫各命令行源文件里读到的每个 `--选项`，必须在表里；表里的也必须在源文件里出现。

/// (子命令, 带值的选项, 开关, 收不收位置参数)
type Entry = (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    bool,
);

const EVAL_VALUES: &[&str] = &["--system", "--filter", "--out"];
const EVAL_FLAGS: &[&str] = &["--audio", "--write-baseline", "--record"];

pub(crate) const OPTIONS: &[Entry] = &[
    ("ci", EVAL_VALUES, EVAL_FLAGS, false),
    ("qa", EVAL_VALUES, EVAL_FLAGS, false),
    (
        "calib",
        &["--answers", "--scores", "--round", "--predictors", "--name"],
        &["--record", "--write-levels"],
        false,
    ),
    ("dnsmos", &[], &[], true),
    (
        "scorecard",
        &[
            "--suite",
            "--system",
            "--rules",
            "--asr-ledger",
            "--shard",
            "--asr-workers",
            "--out",
            "--from-clips",
            "--merge-shards",
        ],
        &["--qa", "--no-safety", "--init-ledger"],
        false,
    ),
    (
        "asr-probe",
        &[
            "--wav-dir",
            "--cache",
            "--out",
            "--temperature",
            "--repeats",
            "--asr-ledger",
        ],
        &[],
        false,
    ),
    (
        "compare",
        &[
            "--a",
            "--b",
            "--system-a",
            "--system-b",
            "--noise-a",
            "--noise-b",
            "--out",
        ],
        &["--by-scene"],
        false,
    ),
    ("nrmos", &[], &["--serve"], true),
    ("list", &[], &[], false),
];

/// 核 `args`（含子命令名）。表里没有的子命令不管（由分发那里报「不认识的子命令」）。
pub(crate) fn check(args: &[String]) -> Result<(), String> {
    let Some(cmd) = args.first() else {
        return Ok(());
    };
    let Some((_, values, flags, positional)) = OPTIONS.iter().find(|e| e.0 == cmd) else {
        return Ok(());
    };
    let known = || {
        values
            .iter()
            .chain(flags.iter())
            .copied()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        if values.contains(&a) {
            i += 2;
        } else if flags.contains(&a) {
            i += 1;
        } else if a.starts_with("--") {
            return Err(format!(
                "vdn-eval {cmd} 不认识选项 {a}（认的有：{}）",
                known()
            ));
        } else if *positional {
            i += 1;
        } else {
            return Err(format!(
                "vdn-eval {cmd} 多出一个参数 {a}（选项要带 --；认的有：{}）",
                known()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn known_pass_unknown_fail() {
        assert!(
            check(&a(&[
                "scorecard",
                "--system",
                "gtcrn",
                "--qa",
                "--out",
                "/x"
            ]))
            .is_ok()
        );
        assert!(
            check(&a(&["scorecard", "--systems", "gtcrn"]))
                .unwrap_err()
                .contains("--systems")
        );
        assert!(
            check(&a(&["scorecard", "gtcrn"]))
                .unwrap_err()
                .contains("gtcrn")
        );
        // 带值选项的值可以长得像什么都行（不会被当成选项再查一遍）
        assert!(check(&a(&["asr-probe", "--temperature", "0", "--out", "r.json"])).is_ok());
        assert!(check(&a(&["nrmos", "scoreq", "m.onnx", "a.wav", "--serve"])).is_ok());
        assert!(check(&a(&["dnsmos", "m.onnx", "a.wav"])).is_ok());
        assert!(
            check(&a(&["nosuch", "--whatever"])).is_ok(),
            "不认识的子命令交给分发处报"
        );
    }

    /// 表和源文件里实际读的选项交叉校验：源文件读了表里没有的 → 红；表里有、源文件从没读 → 红。
    #[test]
    fn table_matches_sources() {
        let sources = [
            include_str!("main.rs"),
            include_str!("cli_scorecard.rs"),
            include_str!("cli_probe.rs"),
            include_str!("scorecard/compare_out.rs"),
            include_str!("qa/nrmos.rs"),
        ];
        let mut read = std::collections::BTreeSet::new();
        for s in sources {
            for (i, _) in s.match_indices("\"--") {
                let rest = &s[i + 1..];
                let end = rest.find('"').unwrap_or(0);
                let opt = &rest[..end];
                if opt.len() > 2
                    && opt[2..]
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                {
                    read.insert(opt.to_string());
                }
            }
        }
        let table: std::collections::BTreeSet<String> = OPTIONS
            .iter()
            .flat_map(|e| e.1.iter().chain(e.2.iter()))
            .map(|s| s.to_string())
            .collect();
        let not_in_table: Vec<_> = read.difference(&table).collect();
        let not_read: Vec<_> = table.difference(&read).collect();
        assert!(
            not_in_table.is_empty(),
            "源文件读了、表里没有：{not_in_table:?}"
        );
        assert!(not_read.is_empty(), "表里有、源文件没读：{not_read:?}");
    }
}
