//! exp021-remix：重混外部离线输出目录，派生关掉保命的测试集。
//!
//! 用法：
//!   exp021-remix remix --src <外部目录> --out <新目录> --prefix <名字前缀> [--open-db X] [--floor-db Y] [--off]
//!   exp021-remix raw --like <外部目录> --out <新目录> --model gtcrn [--limit N]   # 生成混回前的原始输出
//!   exp021-remix suite --out <JSON>        # 由 src/eval/suite/scenes.json 派生，全部任务 safety = false
//!
//! 目录可以写相对主目录的路径。新目录已存在就报错（不覆盖旧 build）。

mod raw;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use exp021_remix::{MixParams, remix_one, suite_safety_off, system_name};
use vdn_eval::scene::SceneClip;
use vdn_eval::scorecard::load_scene_clip;
use vdn_eval::suite::{main_root, resolve};
use vdn_eval::system::external::Kind;
use vdn_eval::wav;

fn arg(args: &[String], k: &str) -> Option<String> {
    args.iter()
        .position(|a| a == k)
        .and_then(|i| args.get(i + 1).cloned())
}

fn abs(p: &str) -> PathBuf {
    let p = Path::new(p);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        main_root().join(p)
    }
}

fn num(args: &[String], k: &str) -> Result<Option<f32>, String> {
    arg(args, k)
        .map(|v| v.parse::<f32>().map_err(|e| format!("{k} {v}：{e}")))
        .transpose()
}

fn cmd_remix(args: &[String]) -> Result<(), String> {
    let src = abs(&arg(args, "--src").ok_or("缺 --src")?);
    let out = abs(&arg(args, "--out").ok_or("缺 --out")?);
    let prefix = arg(args, "--prefix").ok_or("缺 --prefix")?;
    let p = MixParams {
        open_db: num(args, "--open-db")?,
        floor_db: num(args, "--floor-db")?,
        off: args.iter().any(|a| a == "--off"),
    };
    if out.exists() {
        return Err(format!("{} 已存在，不覆盖", out.display()));
    }
    let rj_path = src.join("run.json");
    let mut rj: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&rj_path).map_err(|e| format!("{}：{e}", rj_path.display()))?,
    )
    .map_err(|e| e.to_string())?;
    let lat = rj["latency_samples"].as_u64().ok_or("run.json 缺 latency_samples")? as usize;
    let clips_rel = rj["clips"].as_str().ok_or("run.json 缺 clips")?.to_string();
    let cp = resolve(&clips_rel).ok_or_else(|| format!("找不到 {clips_rel}"))?;
    let clips = SceneClip::parse_jsonl(&std::fs::read_to_string(&cp).map_err(|e| e.to_string())?)?;
    let root = main_root();
    let kinds: Vec<Kind> = rj["inputs"]
        .as_array()
        .ok_or("run.json 缺 inputs")?
        .iter()
        .map(|k| k.as_str().and_then(Kind::parse).ok_or("inputs 里有未知种类"))
        .collect::<Result<_, _>>()?;
    let mut n_files = 0usize;
    for c in clips.iter().filter(|c| c.has_clean_ref) {
        let clip = load_scene_clip(c, &root)?;
        for &k in &kinds {
            let f = src.join(k.as_str()).join(format!("{}.wav", c.id));
            if !f.exists() {
                continue;
            }
            let (y, sr) = wav::read(&f)?;
            if sr != exp021_remix::SR {
                return Err(format!("{} 采样率 {sr}", f.display()));
            }
            let x = k.input(&clip);
            if x.len() != y.len() {
                return Err(format!("{} 长度 {} ≠ 输入 {}", f.display(), y.len(), x.len()));
            }
            let o = remix_one(&x, &y, lat, p.config());
            wav::write(&out.join(k.as_str()).join(format!("{}.wav", c.id)), &o, sr)?;
            n_files += 1;
        }
    }
    let c = p.config();
    let name = system_name(&prefix, &p);
    let src_name = rj["name"].clone();
    rj["name"] = name.clone().into();
    let model = rj["model"].take();
    rj["model"] = serde_json::json!({
        "exp021_remix": {
            "from": src.strip_prefix(&root).unwrap_or(&src).to_string_lossy(),
            "from_name": src_name,
            "from_run_json_sha256": wav::sha256(&rj_path)?,
            "off": p.off,
            "open_db": c.open_db,
            "floor_db": c.floor_db,
            "files": n_files,
            "note": "已用真引擎按上面的参数混回原声；打分时须用 safety=false 的派生测试集，免得混两次",
        },
        "upstream": model,
    });
    std::fs::write(
        out.join("run.json"),
        serde_json::to_string_pretty(&rj).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| e.to_string())?;
    println!("{name}：{n_files} 个文件 → {}", out.display());
    Ok(())
}

fn cmd_suite(args: &[String]) -> Result<(), String> {
    let out = abs(&arg(args, "--out").ok_or("缺 --out")?);
    let src = resolve("src/eval/suite/scenes.json").ok_or("找不到 scenes.json")?;
    let s: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&src).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let o = suite_safety_off(&s)?;
    if out.exists() {
        return Err(format!("{} 已存在，不覆盖", out.display()));
    }
    std::fs::write(&out, serde_json::to_string_pretty(&o).map_err(|e| e.to_string())? + "\n")
        .map_err(|e| e.to_string())?;
    println!("派生测试集 → {}（源 sha256 {}）", out.display(), wav::sha256(&src)?);
    Ok(())
}

fn cmd_raw(args: &[String]) -> Result<(), String> {
    let like = abs(&arg(args, "--like").ok_or("缺 --like")?);
    let out = abs(&arg(args, "--out").ok_or("缺 --out")?);
    let model = arg(args, "--model").unwrap_or_else(|| "gtcrn".into());
    let limit = arg(args, "--limit")
        .map(|v| v.parse::<usize>().map_err(|e| format!("--limit {v}：{e}")))
        .transpose()?;
    raw::run(&like, &out, &model, limit)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("remix") => cmd_remix(&args),
        Some("suite") => cmd_suite(&args),
        Some("raw") => cmd_raw(&args),
        _ => Err("用法：exp021-remix <remix | raw | suite> …".into()),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("exp021-remix 出错：{e}");
            ExitCode::FAILURE
        }
    }
}
