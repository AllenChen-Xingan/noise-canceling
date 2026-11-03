//! `vdn-eval`：评测命令行。
//!
//! ```text
//! vdn-eval ci    [--system ref,gtcrn] [--filter R0] [--out DIR] [--audio] [--write-baseline] [--record]
//! vdn-eval qa    [--system gtcrn,fastenhancer-m] [--filter Q0] [--out DIR] [--record]
//! vdn-eval calib [--answers JSON] [--scores JSON] [--round N] [--write-levels] [--record]
//! vdn-eval list
//! ```
//! - `ci`：CI 层（纯 DSP、无模型）。门禁不过 → 退出码 1。权重不在本机的系统自动跳过
//! - `qa`：QA 层（本机 Qwen3-ASR + DNSMOS + 真实片段）
//! - `--record`：把报告复制进 `docs/experiments/exp007-eval-suite/results/`（进仓，CLAUDE.md 第五节第 1 条）
//! - `--write-baseline`：把本次回归集的感知量数值写成冻结基线 `src/eval/suite/baseline-ci.json`

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde_json::{Value, json};
use vdn_eval::files::{BaselineFile, LevelEntry, LevelsFile};
use vdn_eval::report::{Meta, to_json, to_markdown};
use vdn_eval::runner::{Baseline, Opts, SuiteResult, baseline_key, run_suite};
use vdn_eval::suite::{
    CI_SUITE, LEVELS, QA_SUITE, Set, parse_levels, parse_suite, repo_root, resolve,
};
use vdn_eval::system::SystemKind;
use vdn_eval::verifiers::{Kind, REGISTRY, info};

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter()
        .position(|a| a == k)
        .and_then(|i| args.get(i + 1).cloned())
}

fn flag(args: &[String], k: &str) -> bool {
    args.iter().any(|a| a == k)
}

fn sh(cmd: &str, a: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(a)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn meta() -> Meta {
    let root = repo_root();
    let r = root.to_string_lossy().to_string();
    let mut commit =
        sh("git", &["-C", &r, "rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "nogit".into());
    if sh(
        "git",
        &["-C", &r, "status", "--porcelain", "--untracked-files=no"],
    )
    .is_some_and(|s| !s.is_empty())
    {
        commit.push_str("-dirty");
    }
    let stamp = sh("date", &["+%Y%m%d-%H%M%S"]).unwrap_or_else(|| "00000000-000000".into());
    Meta {
        run_id: format!("{stamp}-{commit}"),
        commit,
        profile: if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        ci: std::env::var("CI").is_ok_and(|v| v == "true")
            || std::env::var_os("GITHUB_ACTIONS").is_some(),
    }
}

fn systems(args: &[String], default: &[SystemKind]) -> Result<Vec<SystemKind>, String> {
    match arg(args, "--system") {
        None => Ok(default.to_vec()),
        Some(s) => s
            .split(',')
            .map(|x| SystemKind::parse(x.trim()).ok_or_else(|| format!("未知系统 {x}")))
            .collect(),
    }
}

fn baseline_path() -> PathBuf {
    repo_root().join("src/eval/suite/baseline-ci.json")
}

/// 读冻结基线。文件不存在 = 还没有基线（空）；存在但格式不对 → 报错，不当成空基线静默放过（第 1 轮 codex #2）。
fn load_baseline() -> Result<Baseline, String> {
    let p = baseline_path();
    match std::fs::read_to_string(&p) {
        Ok(t) => BaselineFile::parse(&t).map(|b| b.values),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Baseline::new()),
        Err(e) => Err(format!("{}：{e}", p.display())),
    }
}

fn write_baseline(r: &SuiteResult, m: &Meta) -> Result<usize, String> {
    let mut vals = BTreeMap::new();
    for t in &r.tasks {
        if t.set != Set::Regression {
            continue;
        }
        for tr in &t.trials {
            for g in &tr.graders {
                // V-VACUUM 与 V-NR 是同一个数、方向相反：进基线会把「多压噪声」判成变差，只按 −40 dB 线报警
                if g.info_only
                    || g.v == "V-VACUUM"
                    || info(&g.v).is_none_or(|i| i.kind != Kind::Perceptual)
                {
                    continue;
                }
                if let Some(v) = g.score.value {
                    vals.insert(
                        baseline_key(t.system.as_str(), &t.id, tr.trial, &g.v),
                        (v * 1e4).round() / 1e4,
                    );
                }
            }
        }
    }
    let n = vals.len();
    let doc = BaselineFile {
        doc: "CI 层冻结基线：回归集上 L0/L1 感知量的数值。门禁规则：比它差容差以上即阻断（012：L0 只作「不许比基线差」的约束）。有意改变行为时用 `vdn-eval ci --write-baseline` 重写，并在 PR 里说明为什么变。".into(),
        commit: m.commit.clone(),
        run_id: m.run_id.clone(),
        values: vals,
    };
    std::fs::write(
        baseline_path(),
        serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(n)
}

fn save(
    r: &SuiteResult,
    m: &Meta,
    levels: &BTreeMap<String, (vdn_eval::Level, String)>,
    out: &Path,
    record: bool,
    extra: Option<Value>,
) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut j = to_json(r, m, levels);
    let mut md = to_markdown(r, m, levels);
    if let Some(x) = extra {
        md.push_str("\n## 吃字（集合层，rubric V7）\n\n```json\n");
        md.push_str(&serde_json::to_string_pretty(&x).unwrap_or_default());
        md.push_str("\n```\n");
        j["eat_summary"] = x;
    }
    let js = serde_json::to_string_pretty(&j).map_err(|e| e.to_string())? + "\n";
    std::fs::write(out.join("report.json"), &js).map_err(|e| e.to_string())?;
    std::fs::write(out.join("report.md"), &md).map_err(|e| e.to_string())?;
    println!("报告：{}", out.join("report.md").display());
    if record {
        let d = repo_root().join("docs/experiments/exp007-eval-suite/results");
        std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
        let stem = format!("{}-{}", m.run_id, r.layer);
        std::fs::write(d.join(format!("{stem}.json")), &js).map_err(|e| e.to_string())?;
        std::fs::write(d.join(format!("{stem}.md")), &md).map_err(|e| e.to_string())?;
        println!("已记录：{}", d.join(format!("{stem}.md")).display());
    }
    Ok(())
}

fn print_summary(r: &SuiteResult) {
    for ((sys, set), s) in vdn_eval::report::summarize(r) {
        println!(
            "  {sys:<15} {:<10} 任务 {:>2} · pass@k {}/{} · pass^k {}/{} · 跳过 {}",
            set.as_str(),
            s.tasks,
            s.pass_at_k,
            s.tasks - s.skipped,
            s.pass_pow_k,
            s.tasks - s.skipped,
            s.skipped
        );
    }
    for (k, w) in &r.skipped_systems {
        println!("  跳过系统 {}：{w}", k.as_str());
    }
    let b = r.blocks();
    if b.is_empty() {
        println!("门禁：通过");
    } else {
        println!("门禁：不通过（{} 处）", b.len());
        for x in b {
            println!("  ⛔ {x}");
        }
    }
}

fn cmd_eval(args: &[String], layer: &'static str) -> Result<bool, String> {
    let m = meta();
    let levels = parse_levels(LEVELS)?;
    let (suite, defaults) = if layer == "ci" {
        (
            parse_suite(CI_SUITE)?,
            vec![SystemKind::Ref, SystemKind::Gtcrn],
        )
    } else {
        (
            parse_suite(QA_SUITE)?,
            vec![SystemKind::Gtcrn, SystemKind::FastEnhancerM],
        )
    };
    let sys = systems(args, &defaults)?;
    let out = arg(args, "--out").map(PathBuf::from).unwrap_or_else(|| {
        repo_root()
            .join("_tmp/exp007")
            .join(format!("{}-{layer}", m.run_id))
    });
    let audio = (flag(args, "--audio") || (layer == "qa" && !m.ci)).then(|| out.join("audio"));
    let opts = Opts {
        ci: m.ci,
        audio_dir: audio,
        baseline: if flag(args, "--write-baseline") {
            Baseline::new()
        } else {
            load_baseline()?
        },
        levels: levels.clone(),
    };
    println!(
        "vdn-eval {layer} · {} 个任务 · 系统 {:?} · 基线 {} 项 · {}",
        suite.tasks.len(),
        sys.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        opts.baseline.len(),
        m.profile
    );
    let filter = arg(args, "--filter");
    let (r, extra) = if layer == "qa" {
        let mut x = vdn_eval::qa::QaExtra::new(repo_root().join("_tmp/exp007"));
        if x.asr.is_none() {
            println!("  注意：ASR 服务不在，V-EAT-ASR 全部判「不知道」");
        }
        if let Some(e) = &x.dnsmos_err {
            println!("  注意：{e}");
        }
        let r = run_suite(&suite, layer, &sys, &opts, Some(&mut x), filter.as_deref());
        let calls = x.asr.as_ref().map_or(0, |a| a.calls);
        println!("  ASR 实际调用 {calls} 次（其余命中缓存）");
        (r, Some(x.eat_summary()))
    } else {
        (
            run_suite(&suite, layer, &sys, &opts, None, filter.as_deref()),
            None,
        )
    };
    print_summary(&r);
    if flag(args, "--write-baseline") {
        let n = write_baseline(&r, &m)?;
        println!("已写冻结基线 {} 项：{}", n, baseline_path().display());
    }
    save(&r, &m, &levels, &out, flag(args, "--record"), extra)?;
    Ok(r.gate_ok())
}

fn cmd_calib(args: &[String]) -> Result<(), String> {
    let answers = arg(args, "--answers").unwrap_or_else(|| {
        "docs/experiments/exp003-offline-ab/results/20261030-human-build-20261030110755-f8c3.json".into()
    });
    let scores = arg(args, "--scores")
        .unwrap_or_else(|| "docs/experiments/exp003-offline-ab/scores.json".into());
    let round: u32 = arg(args, "--round")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let read = |p: &str| -> Result<Value, String> {
        let path = resolve(p).ok_or_else(|| format!("找不到 {p}"))?;
        serde_json::from_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    };
    let c = vdn_eval::calib::calibrate(&read(&answers)?, &read(&scores)?, round);
    let src = format!("答卷 `{answers}` · 指标 `{scores}`");
    let md = vdn_eval::calib::to_markdown(&c, &src);
    println!("{md}");
    if flag(args, "--record") {
        let d = repo_root().join("docs/experiments/exp007-eval-suite/calib");
        std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
        let stem = arg(args, "--name").unwrap_or_else(|| format!("round{round}"));
        std::fs::write(
            d.join(format!("{stem}.json")),
            serde_json::to_string_pretty(&c).map_err(|e| e.to_string())? + "\n",
        )
        .map_err(|e| e.to_string())?;
        std::fs::write(d.join(format!("{stem}.md")), &md).map_err(|e| e.to_string())?;
        println!("已记录：{}", d.join(format!("{stem}.md")).display());
    }
    if flag(args, "--write-levels") {
        let path = repo_root().join("src/eval/suite/levels.json");
        let mut lv: LevelsFile =
            serde_json::from_str(LEVELS).map_err(|e| format!("levels.json：{e}"))?;
        for r in c["metrics"].as_array().into_iter().flatten() {
            let id = r["verifier"].as_str().unwrap_or("");
            if info(id).is_none() {
                continue; // 未登记的指标（ESTOI、SI-SDR、DNSMOS 子项）不进级别表
            }
            let level = serde_json::from_value(r["level"].clone())
                .map_err(|e| format!("{id} 的级别不对：{e}"))?;
            lv.verifiers.insert(id.to_string(), LevelEntry {
                level,
                basis: format!(
                    "定标第 {} 轮（{}）：配对 {}、一致率 {:.2} [{:.2}, {:.2}]、片段内 τ {:.2}；{}",
                    c["round"], c["set"].as_str().unwrap_or(""), r["pairs"],
                    r["agreement"].as_f64().unwrap_or(f64::NAN),
                    r["agreement_ci95"][0].as_f64().unwrap_or(f64::NAN),
                    r["agreement_ci95"][1].as_f64().unwrap_or(f64::NAN),
                    r["tau_within_clip_mean"].as_f64().unwrap_or(f64::NAN),
                    r["why"].as_str().unwrap_or("")
                ),
            });
        }
        if let Some(d) = sh("date", &["+%Y-%m-%d"]) {
            lv.updated = d;
        }
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&lv).map_err(|e| e.to_string())? + "\n",
        )
        .map_err(|e| e.to_string())?;
        println!("已回写级别表：{}（重新编译后生效）", path.display());
    }
    Ok(())
}

/// 子进程入口：`vdn-eval dnsmos <模型.onnx> <音频.wav>` → 一行 JSON。
fn cmd_dnsmos(args: &[String]) -> Result<bool, String> {
    let (Some(model), Some(wav)) = (args.get(1), args.get(2)) else {
        return Err("用法：vdn-eval dnsmos <模型.onnx> <音频.wav>".into());
    };
    let (x, sr) = vdn_eval::wav::read(Path::new(wav))?;
    if sr != 16_000 {
        return Err(format!("只支持 16 kHz，实际 {sr}"));
    }
    let mut d = vdn_eval::qa::dnsmos::Dnsmos::open(Path::new(model)).map_err(|e| e.to_string())?;
    let m = d.score(&x).map_err(|e| e.to_string())?;
    println!(
        "{}",
        json!({"sig": m.sig, "bak": m.bak, "ovrl": m.ovrl, "windows": m.windows})
    );
    Ok(true)
}

fn cmd_list() {
    for (name, text) in [("ci", CI_SUITE), ("qa", QA_SUITE)] {
        match parse_suite(text) {
            Ok(s) => {
                println!("[{name}] {} 个任务", s.tasks.len());
                for t in &s.tasks {
                    println!(
                        "  {:<36} {:<10} {:<8} {}",
                        t.id,
                        t.set.as_str(),
                        t.polarity,
                        t.desc
                    );
                }
            }
            Err(e) => println!("[{name}] 清单错误：{e}"),
        }
    }
    println!("verifier：");
    for i in REGISTRY {
        println!(
            "  {:<12} {:<3} {:<10} {}",
            i.id,
            i.layer,
            i.kind.as_str(),
            i.name
        );
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().cloned().unwrap_or_default();
    let r = match cmd.as_str() {
        "ci" => cmd_eval(&args, "ci"),
        "qa" => cmd_eval(&args, "qa"),
        "calib" => cmd_calib(&args).map(|()| true),
        "dnsmos" => cmd_dnsmos(&args),
        "list" => {
            cmd_list();
            Ok(true)
        }
        _ => {
            eprintln!("用法：vdn-eval <ci | qa | calib | list> [选项]（见源码头注释）");
            return ExitCode::FAILURE;
        }
    };
    match r {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("vdn-eval 出错：{e}");
            ExitCode::FAILURE
        }
    }
}
