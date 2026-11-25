//! `vdn-eval compare` 的命令行与输出（Markdown + JSON，只新建不覆盖）。算法在 [`super::compare`]。
//!
//! ```text
//! vdn-eval compare --a <基线 clips.jsonl> --b <候选 clips.jsonl> [--by-scene]
//!                  [--system-a S] [--system-b S] [--noise-a <clips.jsonl> --noise-b <clips.jsonl>] [--out DIR]
//! ```

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::compare::{Call, Compare, compare, select};
use super::load::parse_jsonl;

pub const FILES: [&str; 2] = ["compare.json", "compare.md"];

fn f(v: Option<f64>) -> String {
    v.map_or("—".into(), |x| format!("{x:.3}"))
}

fn ci_text(c: Option<&super::aggregate::Ci>) -> String {
    c.map_or("—".into(), |c| {
        format!(
            "{:+.3} [{:+.3}, {:+.3}]（{} 组）",
            c.mean, c.lo, c.hi, c.groups
        )
    })
}

/// Markdown 里最多列出的片段 id 数；全部 id 在 compare.json。
pub const MAX_IDS: usize = 10;

/// 片段 id 列表：空写「—」，超过 [`MAX_IDS`] 只列前几个并写明总数。
fn ids(v: &[String]) -> String {
    match v.len() {
        0 => "—".into(),
        n if n <= MAX_IDS => v.join(" "),
        n => format!(
            "{} …共 {n} 段（全部见 compare.json）",
            v[..MAX_IDS].join(" ")
        ),
    }
}

/// 人读的比较报告。
pub fn to_markdown(cmp: &[Compare], a: &str, b: &str, noise: bool) -> String {
    let mut s = format!(
        "# 配对比较\n\n基线 a：`{a}`\n候选 b：`{b}`\n\n差值 = b − a；区间 = 按源片段成组 bootstrap 的 95% 区间（SC-AGG）。\
         区间整个在更好一侧 = 涨，整个在更差一侧 = 退，碰到 0 或组数不够 = 不确定。没有做多重比较校正。\n"
    );
    if noise {
        s += "「种子噪声底」= 同一配置两个种子之间的差值区间，只作参照，不改判断。\n";
    }
    for c in cmp {
        let title = if c.scope == "all" {
            "全部段".to_string()
        } else {
            format!("场景 {}", c.scope)
        };
        s += &format!(
            "\n## {title}（配对 {} 段；只在 a {} 段，只在 b {} 段）\n\n",
            c.pairs,
            c.only_a.len(),
            c.only_b.len()
        );
        let tally: Vec<String> = Call::ALL
            .iter()
            .map(|k| {
                format!(
                    "{} {}",
                    k.zh(),
                    c.items.iter().filter(|i| i.call == *k).count()
                )
            })
            .collect();
        s += &format!("判断：{}\n\n", tally.join(" · "));
        s += if noise {
            "| 项 | 越大 / 越小越好 | 配对段 | a 均值 | b 均值 | 差值 [95% 区间] | 判断 | 种子噪声底 |\n|---|---|---|---|---|---|---|---|\n"
        } else {
            "| 项 | 越大 / 越小越好 | 配对段 | a 均值 | b 均值 | 差值 [95% 区间] | 判断 |\n|---|---|---|---|---|---|---|\n"
        };
        for i in &c.items {
            let dir = if i.better == "higher" {
                "越大越好"
            } else {
                "越小越好"
            };
            s += &format!(
                "| {} | {dir} | {}{} | {} | {} | {} {} | {} |",
                i.v,
                i.pairs,
                if i.one_sided > 0 {
                    format!("（另 {} 段只一边有值）", i.one_sided)
                } else {
                    String::new()
                },
                f(i.mean_a),
                f(i.mean_b),
                ci_text(i.diff.as_ref()),
                i.unit,
                i.call.zh()
            );
            if noise {
                s += &format!(" {} |", ci_text(i.noise.as_ref()));
            }
            s += "\n";
        }
        s += "\n| 否决项 | 逐段判法 | a 否决段 | b 否决段 | b 新增 | b 消失 | 一边判不了 |\n|---|---|---|---|---|---|---|\n";
        for v in &c.vetoes {
            s += &format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                v.v,
                v.rule,
                v.a,
                v.b,
                ids(&v.new_in_b),
                ids(&v.gone_in_b),
                ids(&v.undecided),
            );
        }
        if !c.only_a.is_empty() || !c.only_b.is_empty() {
            s += &format!("\n只在 a：{}\n只在 b：{}\n", ids(&c.only_a), ids(&c.only_b));
        }
    }
    s
}

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter()
        .position(|a| a == k)
        .and_then(|i| args.get(i + 1).cloned())
}

fn read(p: &str, sys: Option<&str>, side: &str) -> Result<(Vec<super::ClipRow>, String), String> {
    let path = Path::new(p);
    let text = std::fs::read_to_string(path).map_err(|e| format!("{p}：{e}"))?;
    let rows = select(
        parse_jsonl(&text).map_err(|e| format!("{p}：{e}"))?,
        sys,
        side,
    )?;
    if rows.is_empty() {
        return Err(format!("{p} 里没有段"));
    }
    Ok((rows, crate::wav::sha256(path)?))
}

/// 选系统之后一个文件里只剩一个系统（[`select`] 保证）：取它的名字。
fn system_of(rows: &[super::ClipRow]) -> &str {
    rows.first().map_or("", |r| r.system.as_str())
}

/// 命令行入口（`args` 不含子命令名）。返回报告目录。
pub fn cli(args: &[String], run_id: &str, default_out: PathBuf) -> Result<PathBuf, String> {
    let need =
        |k: &str| arg(args, k).ok_or_else(|| format!("缺 {k}（用法见 compare_out.rs 头注释）"));
    let (pa, pb) = (need("--a")?, need("--b")?);
    let (a, sha_a) = read(&pa, arg(args, "--system-a").as_deref(), "a")?;
    let (b, sha_b) = read(&pb, arg(args, "--system-b").as_deref(), "b")?;
    let noise = match (arg(args, "--noise-a"), arg(args, "--noise-b")) {
        (Some(x), Some(y)) => Some((read(&x, None, "noise-a")?, read(&y, None, "noise-b")?, x, y)),
        (None, None) => None,
        _ => return Err("--noise-a 与 --noise-b 要一起给".into()),
    };
    let by_scene = args.iter().any(|x| x == "--by-scene");
    let cmp = compare(
        &a,
        &b,
        noise
            .as_ref()
            .map(|((x, _), (y, _), _, _)| (x.as_slice(), y.as_slice())),
        by_scene,
    )?;
    let out = arg(args, "--out").map(PathBuf::from).unwrap_or(default_out);
    let meta: Value = json!({
        "run_id": run_id, "a": pa, "a_sha256": sha_a, "a_system": system_of(&a), "b": pb, "b_sha256": sha_b,
        "b_system": system_of(&b), "by_scene": by_scene,
        "noise": noise.as_ref().map(|((_, sx), (_, sy), x, y)| json!({"a": x, "a_sha256": sx, "b": y, "b_sha256": sy})),
    });
    let js = serde_json::to_string_pretty(&json!({"meta": meta, "scopes": cmp}))
        .map_err(|e| e.to_string())?
        + "\n";
    // 报告里写明两边实际比的系统（codex 第 2 轮 P2 #1：同一文件选不同系统时，只看路径分不出谁是基线）
    let md = to_markdown(
        &cmp,
        &format!("{pa}（系统 {}）", system_of(&a)),
        &format!("{pb}（系统 {}）", system_of(&b)),
        noise.is_some(),
    );
    for f in FILES {
        if out.join(f).exists() {
            return Err(format!(
                "{} 已存在，不覆盖：换一个 --out 目录",
                out.join(f).display()
            ));
        }
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    for (f, t) in FILES.iter().zip([js, md]) {
        super::card::write_new_file(&out.join(f), &t)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strs(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("c{i}")).collect()
    }

    #[test]
    fn long_id_lists_are_capped() {
        assert_eq!(ids(&[]), "—");
        assert_eq!(ids(&strs(3)), "c0 c1 c2");
        let t = ids(&strs(MAX_IDS + 5));
        assert!(t.contains(&format!("共 {} 段", MAX_IDS + 5)), "{t}");
        assert!(!t.contains(&format!("c{MAX_IDS} ")), "{t}");
    }

    #[test]
    fn cli_errors_and_refuses_overwrite() {
        let dir = std::env::temp_dir().join(format!("vdn-compare-cli-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let s = |x: &[&str]| x.iter().map(|v| v.to_string()).collect::<Vec<_>>();
        // 缺 --b
        assert!(cli(&s(&["--a", "x"]), "t", dir.clone()).is_err());
        // --noise-a 不配 --noise-b
        let jl = dir.join("a.jsonl");
        let row = super::super::ClipRow {
            clip: "cafe-c00-ext".into(),
            scene: crate::scene::MeetingPlace::Cafe,
            tier: crate::scene::Tier::Common,
            device: "mba-builtin".into(),
            system: "gtcrn".into(),
            sounds: Vec::new(),
            scores: vec![(
                "V-ESTOI",
                false,
                crate::Score::judged("V-ESTOI", 0.5, "Δ", crate::Better::Higher, Some(0.0), &[]),
            )],
        };
        std::fs::write(&jl, super::super::load::to_jsonl(&[row]).unwrap()).unwrap();
        let p = jl.to_string_lossy().to_string();
        assert!(
            cli(
                &s(&["--a", &p, "--b", &p, "--noise-a", &p]),
                "t",
                dir.join("o1")
            )
            .is_err()
        );
        // 正常写出；同一目录第二次拒绝覆盖
        let out = dir.join("o2");
        cli(&s(&["--a", &p, "--b", &p]), "t", out.clone()).unwrap();
        assert!(out.join("compare.md").exists() && out.join("compare.json").exists());
        assert!(cli(&s(&["--a", &p, "--b", &p]), "t", out).is_err());
        // 同一个多系统文件，两边选不同系统：JSON 与 Markdown 都写明谁是基线、谁是候选；交换后跟着换
        let mut two = row.clone();
        two.system = "ref".into();
        let jl2 = dir.join("two.jsonl");
        std::fs::write(&jl2, super::super::load::to_jsonl(&[row.clone(), two]).unwrap()).unwrap();
        let p2 = jl2.to_string_lossy().to_string();
        for (sa, sb, tag) in [("gtcrn", "ref", "o3"), ("ref", "gtcrn", "o4")] {
            let o = dir.join(tag);
            let args = s(&["--a", &p2, "--b", &p2, "--system-a", sa, "--system-b", sb]);
            cli(&args, "t", o.clone()).unwrap();
            let js: Value = serde_json::from_str(&std::fs::read_to_string(o.join("compare.json")).unwrap()).unwrap();
            assert_eq!((js["meta"]["a_system"].as_str(), js["meta"]["b_system"].as_str()), (Some(sa), Some(sb)));
            let md = std::fs::read_to_string(o.join("compare.md")).unwrap();
            assert!(md.contains(&format!("（系统 {sa}）`").replace("`", "")) || md.contains(&format!("系统 {sa}")), "{md}");
            let ia = md.find(&format!("系统 {sa}")).unwrap();
            let ib = md.find(&format!("系统 {sb}")).unwrap();
            assert!(ia < ib, "基线在前");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
