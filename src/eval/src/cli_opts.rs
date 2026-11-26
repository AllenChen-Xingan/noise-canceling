//! 每个子命令认哪些选项（正本，#191）：命令行在做任何事之前先按这张表核一遍，不认识的选项、带值选项缺值、
//! 位置参数多了或少了，一律报错退出（退出码 2），不悄悄忽略、按默认值跑（把 `--system` 写成 `--systems`
//! 会打错系统，成绩单看着正常、结论却错）。
//!
//! 限制（有意为之）：带值选项的值不能以 `--` 开头（如目录名 `--results`），一律当缺值拒；宁可拒一个怪名字，也不让
//! 「下一个选项被吞成值、下游又把它当开关」那种两义的写法走下去（codex 本分支第 1、2 轮）。
//!
//! 和代码对得上靠测试 `table_matches_sources`：每个子命令入口那几个函数里实际读到的 `--选项`，必须和表里那一行完全相同。

/// (子命令, 带值的选项, 开关, 位置参数个数范围 (最少, 最多；None = 不限))
type Entry = (
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    (usize, Option<usize>),
);

/// 不收位置参数。
const NONE: (usize, Option<usize>) = (0, Some(0));
const EVAL_VALUES: &[&str] = &["--system", "--filter", "--out"];
const EVAL_FLAGS: &[&str] = &["--audio", "--write-baseline", "--record"];

pub(crate) const OPTIONS: &[Entry] = &[
    ("ci", EVAL_VALUES, EVAL_FLAGS, NONE),
    ("qa", EVAL_VALUES, EVAL_FLAGS, NONE),
    (
        "calib",
        &["--answers", "--scores", "--round", "--predictors", "--name"],
        &["--record", "--write-levels"],
        NONE,
    ),
    // dnsmos <模型.onnx> <音频.wav>：正好两个
    ("dnsmos", &[], &[], (2, Some(2))),
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
        NONE,
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
        NONE,
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
        NONE,
    ),
    // accept：验收判定（计划 014 第三节）。--vs 与 --noise 可以给多次
    (
        "accept",
        &[
            "--suite", "--vs", "--noise", "--scenes", "--card-a", "--card-b", "--out",
        ],
        &[],
        NONE,
    ),
    // nrmos <scoreq|distill> <模型.onnx> (<音频.wav>… | --serve)：至少两个，其余由它自己核
    ("nrmos", &[], &["--serve"], (2, None)),
    ("list", &[], &[], NONE),
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
    let mut pos = 0usize;
    while i < args.len() {
        let a = args[i].as_str();
        if values.contains(&a) {
            // 带值选项缺值（在末尾，或下一项又是 -- 开头）：报错，不当成没给、也不把下一个选项吞成值
            // （codex 本分支第 1 轮 P1 #1 · #2）
            match args.get(i + 1) {
                Some(v) if !v.starts_with("--") => i += 2,
                _ => return Err(format!("vdn-eval {cmd} 的 {a} 后面缺值")),
            }
        } else if flags.contains(&a) {
            i += 1;
        } else if a.starts_with("--") {
            return Err(format!(
                "vdn-eval {cmd} 不认识选项 {a}（认的有：{}）",
                known()
            ));
        } else {
            pos += 1;
            if positional.1.is_some_and(|max| pos > max) {
                return Err(format!(
                    "vdn-eval {cmd} 多出一个参数 {a}（选项要带 --；认的有：{}）",
                    known()
                ));
            }
            i += 1;
        }
    }
    if pos < positional.0 {
        return Err(format!(
            "vdn-eval {cmd} 少参数：要至少 {} 个，给了 {pos} 个",
            positional.0
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

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
        assert!(check(&a(&["asr-probe", "--temperature", "0", "--out", "r.json"])).is_ok());
        assert!(check(&a(&["nrmos", "scoreq", "m.onnx", "a.wav", "--serve"])).is_ok());
        assert!(check(&a(&["dnsmos", "m.onnx", "a.wav"])).is_ok());
        assert!(
            check(&a(&["nosuch", "--whatever"])).is_ok(),
            "不认识的子命令交给分发处报"
        );
    }

    /// codex 本分支第 1 轮：缺值、把下一个选项当值、位置参数超量或不够，都拒。
    #[test]
    fn missing_values_and_positional_counts() {
        assert!(check(&a(&["ci", "--system"])).unwrap_err().contains("缺值"));
        assert!(
            check(&a(&["ci", "--filter", "--write-baseline"]))
                .unwrap_err()
                .contains("缺值")
        );
        assert!(
            check(&a(&["dnsmos", "m.onnx", "a.wav", "extra"]))
                .unwrap_err()
                .contains("extra")
        );
        assert!(
            check(&a(&["dnsmos", "m.onnx"]))
                .unwrap_err()
                .contains("少参数")
        );
        assert!(
            check(&a(&["nrmos", "scoreq"]))
                .unwrap_err()
                .contains("少参数")
        );
    }

    /// 一段代码读到的 `--选项`：只认读命令行的写法 arg/flag/option_value(args, "--…")、need("--…")、== "--…"
    /// （不把 git 之类的参数算进来）。
    fn read_opts(s: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for pre in ["(args, \"--", "need(\"--", "== \"--"] {
            for (i, _) in s.match_indices(pre) {
                let rest = &s[i + pre.len() - 2..];
                let opt = &rest[..rest.find('"').unwrap_or(0)];
                let ok = opt.len() > 2
                    && opt[2..]
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
                if ok {
                    out.insert(opt.to_string());
                }
            }
        }
        out
    }

    /// 一个文件里从 `fn 名(` 到下一个顶层函数或测试模块之间的代码。
    fn func<'a>(src: &'a str, name: &str) -> &'a str {
        let i = src
            .find(&format!("fn {name}("))
            .unwrap_or_else(|| panic!("找不到 fn {name}"));
        let rest = &src[i + 3..];
        let end = ["\nfn ", "\npub fn ", "\npub(crate) fn ", "\n#[cfg(test)]"]
            .iter()
            .filter_map(|m| rest.find(m))
            .min();
        &rest[..end.unwrap_or(rest.len())]
    }

    /// 表和代码按子命令交叉校验（codex 本分支第 1 轮 P2 #4）：每个子命令入口那几个函数里实际读的选项，
    /// 必须和表里那一行完全相同。删掉某个子命令表里的一项、或代码里新读一个没进表的选项，都会红。
    #[test]
    fn table_matches_sources() {
        let main = include_str!("main.rs");
        let systems = func(main, "systems");
        let code: &[(&str, Vec<&str>)] = &[
            ("ci", vec![func(main, "cmd_eval"), systems]),
            ("qa", vec![func(main, "cmd_eval"), systems]),
            ("calib", vec![func(main, "cmd_calib")]),
            ("dnsmos", vec![func(main, "cmd_dnsmos")]),
            ("scorecard", vec![include_str!("cli_scorecard.rs"), systems]),
            ("asr-probe", vec![include_str!("cli_probe.rs")]),
            ("compare", vec![include_str!("scorecard/compare_out.rs")]),
            ("accept", vec![include_str!("scorecard/accept.rs")]),
            ("nrmos", vec![func(include_str!("qa/nrmos.rs"), "cli")]),
            ("list", vec![func(main, "cmd_list")]),
        ];
        assert_eq!(
            code.len(),
            OPTIONS.len(),
            "每个子命令都要在这里对上入口代码"
        );
        for (cmd, parts) in code {
            let read: BTreeSet<String> = parts.iter().flat_map(|p| read_opts(p)).collect();
            let e = OPTIONS.iter().find(|e| e.0 == *cmd).unwrap();
            let table: BTreeSet<String> =
                e.1.iter()
                    .chain(e.2.iter())
                    .map(|s| s.to_string())
                    .collect();
            assert_eq!(read, table, "{cmd}：左 = 代码里读的，右 = 表里的");
        }
    }
}
