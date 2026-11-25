//! `vdn-eval scorecard` 子命令（从 main.rs 拆出，≤600 行）：场景成绩单、`--from-clips` 重新汇总、
//! `--shard i/n` 分片跑、`--merge-shards` 合并分片（大总管 2026-11-25）。

use std::path::{Path, PathBuf};

use serde_json::json;
use vdn_eval::runner::{Baseline, Opts};
use vdn_eval::suite::{LEVELS, parse_levels, parse_suite, resolve};
use vdn_eval::system::SystemKind;

use super::{arg, flag, meta, systems};

/// 场景成绩单：读场景任务清单 → 逐段打分 → `scorecard.md` + `scorecard.json`。
/// 有不过的场景不算命令失败（成绩单是报告，不是门禁）；清单读不了才失败。
pub(crate) fn cmd_scorecard(args: &[String]) -> Result<bool, String> {
    let m = meta();
    let rules = match arg(args, "--rules") {
        None => vdn_eval::scorecard::card::Rules::V2,
        Some(r) => vdn_eval::scorecard::card::Rules::parse(&r)
            .ok_or_else(|| format!("--rules 只认 v1 或 v2，收到 {r}"))?,
    };
    let default_out = || {
        vdn_eval::suite::main_root()
            .join("_tmp/scorecard")
            .join(&m.run_id)
    };
    if let Some(src) = arg(args, "--from-clips") {
        let out = arg(args, "--out")
            .map(PathBuf::from)
            .unwrap_or_else(default_out);
        let (card, sha) = vdn_eval::scorecard::load::resummarize(Path::new(&src), rules)?;
        let meta_v = json!({"run_id": m.run_id, "commit": m.commit, "from_clips": src,
            "from_clips_sha256": sha, "rules": rules.as_str()});
        let js =
            serde_json::to_string_pretty(&card.to_json(meta_v)).map_err(|e| e.to_string())? + "\n";
        vdn_eval::scorecard::card::write_new(
            &out,
            &js,
            &card.to_markdown("场景成绩单（重新汇总）"),
        )?;
        println!(
            "成绩单：{}（{} 段，规则 {}）",
            out.join("scorecard.md").display(),
            card.clips,
            rules.as_str()
        );
        return Ok(true);
    }
    if let Some(list) = arg(args, "--merge-shards") {
        return merge_shards(args, &list, &m);
    }
    let path = arg(args, "--suite").unwrap_or_else(|| "src/eval/suite/scenes.json".into());
    let p = resolve(&path).ok_or_else(|| format!("找不到 {path}"))?;
    let mut suite = parse_suite(&std::fs::read_to_string(&p).map_err(|e| format!("{path}：{e}"))?)?;
    vdn_eval::scorecard::check_suite(&suite)?;
    // 「不回混」口径：全部任务关保护处理（exp021 已核与原路径逐段相同）
    let no_safety = flag(args, "--no-safety");
    if no_safety {
        suite.without_safety();
    }
    let sys = systems(args, &[SystemKind::Ref])?;
    let opts = Opts {
        ci: m.ci,
        audio_dir: None,
        baseline: Baseline::new(),
        levels: parse_levels(LEVELS)?,
    };
    let out = arg(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(default_out);
    vdn_eval::scorecard::card::ensure_fresh(&out)?;
    // 识别费账本：--asr-ledger 指路径（同 VDN_ASR_LEDGER）；--init-ledger（或 VDN_ASR_INIT_LEDGER=1）才新建
    let asr_ledger = vdn_eval::suite::option_value(args, "--asr-ledger")?.map(PathBuf::from);
    if flag(args, "--qa") {
        let lp = asr_ledger.clone().unwrap_or_else(|| {
            vdn_eval::qa::asr_ledger::ledger_path(&vdn_eval::suite::main_root())
        });
        let env = std::env::var("VDN_ASR_INIT_LEDGER").ok();
        if vdn_eval::qa::asr_ledger::init_if_requested(&lp, flag(args, "--init-ledger"), env)? {
            eprintln!("已新建识别费账本");
        }
    }
    let mut qa = flag(args, "--qa")
        .then(|| vdn_eval::qa::QaExtra::with_ledger(out.join("qa"), asr_ledger.clone()));
    // 分片：--shard i/n 只跑一部分；识别并发 --asr-workers，不给时分片 = 8 ÷ n 向上取整，不分片 = 8（大总管 2026-11-25）
    let shard = vdn_eval::suite::option_value(args, "--shard")?
        .map(|s| vdn_eval::scorecard::shard::Shard::parse(&s))
        .transpose()?;
    let workers = match vdn_eval::suite::option_value(args, "--asr-workers")? {
        Some(w) => w
            .parse::<usize>()
            .ok()
            .filter(|&w| w >= 1)
            .ok_or(format!("--asr-workers 要是 ≥ 1 的整数，收到 {w}"))?,
        None => shard.map_or(vdn_eval::qa::asr_or::ASR_CONCURRENCY, |s| {
            vdn_eval::scorecard::shard::asr_workers(s.n)
        }),
    };
    if let Some(a) = qa.as_mut().and_then(|q| q.asr.as_mut()) {
        a.workers = workers;
    }
    let extra = qa.as_mut().map(|x| x as &mut dyn vdn_eval::runner::Extra);
    let run = vdn_eval::scorecard::run_shard(&suite, &sys, &opts, extra, None, rules, shard);
    let (rows, skipped) = (run.rows, run.missing);
    let total: f64 = run.timing.values().sum();
    eprintln!(
        "分步计时（秒，共 {total:.1}）：{}",
        run.timing
            .iter()
            .map(|(k, v)| format!("{k} {v:.1}"))
            .collect::<Vec<_>>()
            .join(" · ")
    );
    let card = vdn_eval::scorecard::card::Card::build_with(&rows, skipped, rules);
    let meta_v = json!({"run_id": m.run_id, "commit": m.commit, "suite": path,
        "systems": sys.iter().map(|s| s.as_str()).collect::<Vec<_>>(), "qa": qa.is_some(),
        "safety": if no_safety { "off" } else { "task" }, "asr": qa.as_ref().map(|q| q.asr_meta()),
        "rules": rules.as_str(), "power": vdn_eval::power::source(),
        "external": sys.iter().filter_map(|s| s.provenance()).collect::<Vec<_>>(),
        "shard": shard.map(|s| format!("{}/{}", s.i, s.n)), "asr_workers": qa.is_some().then_some(workers),
        "timing": run.timing});
    let js = serde_json::to_string_pretty(&card.to_json(meta_v)).map_err(|e| e.to_string())? + "\n";
    vdn_eval::scorecard::card::write_new(&out, &js, &card.to_markdown("场景成绩单"))?;
    // 逐段分数一并写出（只新建），以后改规则可以不重跑音频、用 --from-clips 重新汇总
    let jsonl = vdn_eval::scorecard::load::to_jsonl(&rows)?;
    vdn_eval::scorecard::card::write_new_file(&out.join("clips.jsonl"), &jsonl)?;
    println!(
        "成绩单：{}（{} 段）",
        out.join("scorecard.md").display(),
        card.clips
    );
    Ok(true)
}

/// `vdn-eval scorecard --merge-shards <目录1>,<目录2>,… [--suite …] --out <新目录>`：把各片的输出合成一份成绩单。
/// 合并后的 `clips.jsonl` 与单进程逐字节相同（`tests/scorecard_shard.rs`）；成绩单从合并后的行重新汇总。
fn merge_shards(args: &[String], list: &str, m: &vdn_eval::report::Meta) -> Result<bool, String> {
    let path = arg(args, "--suite").unwrap_or_else(|| "src/eval/suite/scenes.json".into());
    let p = resolve(&path).ok_or_else(|| format!("找不到 {path}"))?;
    let suite = parse_suite(&std::fs::read_to_string(&p).map_err(|e| format!("{path}：{e}"))?)?;
    let dirs: Vec<PathBuf> = list
        .split(',')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();
    let (text, missing, mut meta) = vdn_eval::scorecard::shard::merge_dirs(&suite, &dirs)?;
    let rules = meta["rules"]
        .as_str()
        .and_then(vdn_eval::scorecard::card::Rules::parse)
        .ok_or("各片 meta 里没有规则版本")?;
    let out = arg(args, "--out")
        .map(PathBuf::from)
        .ok_or("--merge-shards 要带 --out <新目录>")?;
    vdn_eval::scorecard::card::ensure_fresh(&out)?;
    // 只记目录名（不记完整路径：本机路径含用户名，成绩单进公开仓库）
    let names: Vec<String> = dirs
        .iter()
        .map(|d| {
            d.file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
        })
        .collect();
    meta["run_id"] = json!(m.run_id);
    meta["merged_by_commit"] = json!(m.commit);
    meta["merged_from"] = json!(names);
    let rows = vdn_eval::scorecard::load::parse_jsonl(&text)?;
    let card = vdn_eval::scorecard::card::Card::build_with(&rows, missing, rules);
    let js = serde_json::to_string_pretty(&card.to_json(meta)).map_err(|e| e.to_string())? + "\n";
    vdn_eval::scorecard::card::write_new(&out, &js, &card.to_markdown("场景成绩单"))?;
    vdn_eval::scorecard::card::write_new_file(&out.join("clips.jsonl"), &text)?;
    println!(
        "合并 {} 片 → {}（{} 段）",
        dirs.len(),
        out.join("scorecard.md").display(),
        card.clips
    );
    Ok(true)
}
