//! exp014 驱动：和 `vdn-eval scorecard` 走同一条路（同一组库函数、同一份成绩单），
//! 另外把每段每项的分数写成 `clips.jsonl`，供 `analyze.py` 分设备、分「同房间」比较。
//! 不改评测器：成绩单本身由 `Card::build` 生成，和命令行版逐字相同（见 README「怎么跑」的核对）。
//!
//! 用法：exp014-driver scorecard --suite <JSON> --system passthrough,ref,gtcrn [--qa] [--out DIR]
//! 子进程入口（评分模型要用当前可执行文件起子进程）：`dnsmos` · `nrmos`，转给库里同名实现。

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde_json::{Value, json};
use vdn_eval::runner::{Baseline, Opts};
use vdn_eval::scorecard::{ClipRow, card};
use vdn_eval::suite::{LEVELS, main_root, parse_levels, parse_suite, repo_root, resolve};
use vdn_eval::system::SystemKind;

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter()
        .position(|a| a == k)
        .and_then(|i| args.get(i + 1).cloned())
}

fn sh(cmd: &str, a: &[&str]) -> Option<String> {
    let o = Command::new(cmd).args(a).output().ok()?;
    o.status
        .success()
        .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// run id 与命令行版同规则：时间戳 + 短提交号（工作区有改动加 -dirty）。
fn run_id() -> (String, String) {
    let r = repo_root().to_string_lossy().to_string();
    let mut commit =
        sh("git", &["-C", &r, "rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "nogit".into());
    if sh("git", &["-C", &r, "status", "--porcelain", "--untracked-files=no"])
        .is_some_and(|s| !s.is_empty())
    {
        commit.push_str("-dirty");
    }
    let stamp = sh("date", &["+%Y%m%d-%H%M%S"]).unwrap_or_else(|| "00000000-000000".into());
    (format!("{stamp}-{commit}"), commit)
}

/// 一段一行：段 · 场景 · 档 · 设备 · 系统 · 声音 · 每项（分数、判定、旁证）。
pub fn row_json(r: &ClipRow) -> Value {
    let sounds: Vec<Value> = r
        .sounds
        .iter()
        .map(|(c, g)| json!({"cls": c, "group": format!("{g:?}").to_lowercase()}))
        .collect();
    let scores: Vec<Value> = r
        .scores
        .iter()
        .map(|(v, veto, s)| {
            let extra: serde_json::Map<String, Value> = s
                .extra
                .iter()
                .map(|(k, x)| ((*k).to_string(), json!(x.is_finite().then_some(*x))))
                .collect();
            json!({
                "v": v, "veto": veto,
                "value": s.value.filter(|x| x.is_finite()),
                "threshold": s.threshold,
                "verdict": s.verdict.as_str(),
                "extra": extra,
            })
        })
        .collect();
    json!({
        "clip": r.clip, "scene": r.scene, "tier": r.tier, "device": r.device,
        "system": r.system, "sounds": sounds, "scores": scores,
    })
}

fn cmd_scorecard(args: &[String]) -> Result<(), String> {
    let path = arg(args, "--suite").unwrap_or_else(|| "src/eval/suite/scenes.json".into());
    let p = resolve(&path).ok_or_else(|| format!("找不到 {path}"))?;
    let suite = parse_suite(&std::fs::read_to_string(&p).map_err(|e| format!("{path}：{e}"))?)?;
    vdn_eval::scorecard::check_suite(&suite)?;
    let sys: Vec<SystemKind> = arg(args, "--system")
        .unwrap_or_else(|| "ref".into())
        .split(',')
        .map(|x| SystemKind::parse(x.trim()).ok_or_else(|| format!("未知系统 {x}")))
        .collect::<Result<_, _>>()?;
    let opts = Opts {
        ci: false,
        audio_dir: None,
        baseline: Baseline::new(),
        levels: parse_levels(LEVELS)?,
    };
    let (rid, commit) = run_id();
    let out = arg(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| main_root().join("_tmp/scorecard").join(&rid));
    card::ensure_fresh(&out)?;
    let qa_on = args.iter().any(|a| a == "--qa");
    let mut qa = qa_on.then(|| vdn_eval::qa::QaExtra::new(out.join("qa")));
    let extra = qa.as_mut().map(|x| x as &mut dyn vdn_eval::runner::Extra);
    let (rows, skipped) = vdn_eval::scorecard::run(&suite, &sys, &opts, extra, None);
    let c = card::Card::build(&rows, skipped);
    let meta = json!({"run_id": rid, "commit": commit, "suite": path,
        "systems": sys.iter().map(|s| s.as_str()).collect::<Vec<_>>(), "qa": qa_on});
    let js = serde_json::to_string_pretty(&c.to_json(meta)).map_err(|e| e.to_string())? + "\n";
    card::write_new(&out, &js, &c.to_markdown("场景成绩单"))?;
    let mut lines = String::new();
    for r in &rows {
        lines += &serde_json::to_string(&row_json(r)).map_err(|e| e.to_string())?;
        lines.push('\n');
    }
    std::fs::write(out.join("clips.jsonl"), lines).map_err(|e| e.to_string())?;
    println!("成绩单：{}（{} 段）", out.join("scorecard.md").display(), c.clips);
    Ok(())
}

/// 子进程入口，与 `vdn-eval dnsmos` 相同。
fn cmd_dnsmos(args: &[String]) -> Result<(), String> {
    let (Some(model), Some(wav)) = (args.get(1), args.get(2)) else {
        return Err("用法：dnsmos <模型.onnx> <音频.wav>".into());
    };
    let (x, sr) = vdn_eval::wav::read(Path::new(wav))?;
    if sr != vdn_types::MODEL_RATE {
        return Err(format!("只支持 16 kHz，实际 {sr}"));
    }
    let mut d = vdn_eval::qa::dnsmos::Dnsmos::open(Path::new(model)).map_err(|e| e.to_string())?;
    let m = d.score(&x).map_err(|e| e.to_string())?;
    println!(
        "{}",
        json!({"sig": m.sig, "bak": m.bak, "ovrl": m.ovrl, "windows": m.windows})
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("scorecard") => cmd_scorecard(&args),
        Some("dnsmos") => cmd_dnsmos(&args),
        Some("nrmos") => vdn_eval::qa::nrmos::cli(&args[1..]).map(|rows| {
            for r in rows {
                println!("{r}");
            }
        }),
        _ => Err("用法：exp014-driver <scorecard | dnsmos | nrmos> …".into()),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("exp014-driver 出错：{e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_eval::scene::{MeetingPlace, SoundGroup, Tier};
    use vdn_eval::{Better, Score};

    #[test]
    fn row_json_keeps_every_score_and_drops_nan() {
        let mut s = Score::judged("V-NR", -7.0, "dB", Better::Lower, Some(-6.0), &[]);
        s.extra.push(("bad", f64::NAN));
        let r = ClipRow {
            clip: "cafe-c00-mba".into(),
            scene: MeetingPlace::Cafe,
            tier: Tier::Common,
            device: "mba-builtin".into(),
            system: "gtcrn".into(),
            sounds: vec![("music".into(), SoundGroup::Pattern)],
            scores: vec![("V-NR", false, s)],
        };
        let j = row_json(&r);
        assert_eq!(j["scene"], "cafe");
        assert_eq!(j["tier"], "common");
        assert_eq!(j["sounds"][0]["group"], "pattern");
        assert_eq!(j["scores"][0]["v"], "V-NR");
        assert_eq!(j["scores"][0]["value"], -7.0);
        assert!(j["scores"][0]["extra"]["bad"].is_null());
    }
}
