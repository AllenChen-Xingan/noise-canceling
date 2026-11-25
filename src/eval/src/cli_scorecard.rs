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
    let extra = qa.as_mut().map(|x| x as &mut dyn vdn_eval::runner::Extra);
    let (rows, skipped) = vdn_eval::scorecard::run_with(&suite, &sys, &opts, extra, None, rules);
    let card = vdn_eval::scorecard::card::Card::build_with(&rows, skipped, rules);
    let meta_v = json!({"run_id": m.run_id, "commit": m.commit, "suite": path,
        "systems": sys.iter().map(|s| s.as_str()).collect::<Vec<_>>(), "qa": qa.is_some(),
        "safety": if no_safety { "off" } else { "task" }, "asr": qa.as_ref().map(|q| q.asr_meta()),
        "rules": rules.as_str(), "power": vdn_eval::power::source(),
        "external": sys.iter().filter_map(|s| s.provenance()).collect::<Vec<_>>()});
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

