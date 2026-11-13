//! `raw` 子命令：用项目里的真模型（`vdn_models`）生成「混回前的原始输出」外部目录。
//! 文件清单照抄一个现成外部目录（`--like`），保证段与输入种类和其他候选一致。
//! 引擎三件保命全关（`SafetyConfig::off`），输出 = 模型原样输出；坏值直接报错，不写进目录。
//! 模型的预热输出（对齐前的前 latency 个样本）另存到 `<out>.head/<种类>/<段>.wav`：外部目录格式不收它，
//! 但重混要带上它才和内置模型路径逐段一致（`remix --head`）。

use std::path::Path;

use exp021_remix::{SR, run_engine_full};
use vdn_engine::SafetyConfig;
use vdn_eval::scene::SceneClip;
use vdn_eval::scorecard::load_scene_clip;
use vdn_eval::suite::{main_root, resolve};
use vdn_eval::system::external::Kind;
use vdn_eval::wav;
use vdn_models::ModelKind;

/// `limit`：只做前 N 段（冒烟用）。
pub fn run(like: &Path, out: &Path, model: &str, limit: Option<usize>) -> Result<(), String> {
    let mk = match model {
        "gtcrn" => ModelKind::Gtcrn,
        _ => return Err(format!("只支持 gtcrn，收到 {model}")),
    };
    let path = mk.default_path().ok_or("GTCRN 权重不在本机")?;
    let head_dir = out.with_extension("head");
    if out.exists() || head_dir.exists() {
        return Err(format!("{} 或其 .head 已存在，不覆盖", out.display()));
    }
    let rj_path = like.join("run.json");
    let like_rj: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&rj_path).map_err(|e| format!("{}：{e}", rj_path.display()))?,
    )
    .map_err(|e| e.to_string())?;
    let clips_rel = like_rj["clips"].as_str().ok_or("run.json 缺 clips")?;
    let cp = resolve(clips_rel).ok_or_else(|| format!("找不到 {clips_rel}"))?;
    let clips = SceneClip::parse_jsonl(&std::fs::read_to_string(&cp).map_err(|e| e.to_string())?)?;
    let kinds: Vec<Kind> = like_rj["inputs"]
        .as_array()
        .ok_or("run.json 缺 inputs")?
        .iter()
        .map(|k| k.as_str().and_then(Kind::parse).ok_or("inputs 里有未知种类"))
        .collect::<Result<_, _>>()?;
    let root = main_root();
    let open = || mk.open(&path).map_err(|e| format!("打开 {}：{e}", path.display()));
    let latency = vdn_engine::Model::latency_samples(&open()?);
    let (mut n_clips, mut n_files) = (0usize, 0usize);
    for c in clips.iter().filter(|c| c.has_clean_ref) {
        if limit.is_some_and(|l| n_clips >= l) {
            break;
        }
        let clip = load_scene_clip(c, &root)?;
        let mut any = false;
        for &k in &kinds {
            if !like.join(k.as_str()).join(format!("{}.wav", c.id)).exists() {
                continue;
            }
            // 每个文件新开模型：和评测器每段新建引擎同口径（状态清零）
            let (head, y) = run_engine_full(open()?, &k.input(&clip), SafetyConfig::off());
            if let Some(i) = head.iter().chain(&y).position(|v| !v.is_finite()) {
                return Err(format!("{} {} 第 {i} 样本不是有限数", c.id, k.as_str()));
            }
            let name = format!("{}.wav", c.id);
            wav::write(&out.join(k.as_str()).join(&name), &y, SR)?;
            wav::write(&head_dir.join(k.as_str()).join(&name), &head, SR)?;
            n_files += 1;
            any = true;
        }
        n_clips += usize::from(any);
    }
    let rj = serde_json::json!({
        "schema": vdn_eval::system::external::SCHEMA,
        "name": format!("{model}-raw"),
        "clips": clips_rel,
        "sr": SR,
        "latency_samples": latency,
        "inputs": like_rj["inputs"],
        "native_rate": SR,
        "model": {
            "exp021_raw": {
                "model": model,
                "weights_sha256": wav::sha256(&path)?,
                "engine": "SafetyConfig::off（三件保命全关，输出 = 模型原样输出）",
                "head_dir": head_dir.strip_prefix(&root).unwrap_or(&head_dir).to_string_lossy(),
                "file_list_like": like.strip_prefix(&root).unwrap_or(like).to_string_lossy(),
                "clips": n_clips,
                "files": n_files,
                "limit": limit,
            }
        },
    });
    std::fs::write(
        out.join("run.json"),
        serde_json::to_string_pretty(&rj).map_err(|e| e.to_string())? + "\n",
    )
    .map_err(|e| e.to_string())?;
    println!("{model}-raw：{n_clips} 段 {n_files} 个文件 → {}", out.display());
    Ok(())
}
