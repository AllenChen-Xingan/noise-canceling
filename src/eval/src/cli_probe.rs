//! `vdn-eval asr-probe`：钉温度探针（大总管 2026-11-25 定 C），逻辑在 `vdn_eval::qa::asr_probe`。
//!
//! 用法：`vdn-eval asr-probe --wav-dir <旧运行的 qa/asr-wav> --cache <旧运行的 qa/asr_cache.json> --out <报告.json>
//!        [--temperature T] [--repeats N] [--asr-ledger 账本]`
//! - 只探旧缓存里出现过的音频（语言按 sha256 从缓存的键里查）；每条连调 N 次（默认 3），全是真调用
//! - 账本照常：不存在、已封存、待核都拒发；报告只新建，不覆盖

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::arg;

pub(crate) fn cmd_asr_probe(args: &[String]) -> Result<bool, String> {
    use vdn_eval::qa::{asr_ledger, asr_or, asr_probe};
    let need = |k: &str| arg(args, k).ok_or(format!("asr-probe 要带 {k}"));
    let (dir, cache, out) = (
        PathBuf::from(need("--wav-dir")?),
        need("--cache")?,
        PathBuf::from(need("--out")?),
    );
    if out.exists() {
        return Err(format!("{} 已存在：报告只新建，不覆盖", out.display()));
    }
    let repeats = match vdn_eval::suite::option_value(args, "--repeats")? {
        Some(r) => r
            .parse::<usize>()
            .ok()
            .filter(|&r| r >= 2)
            .ok_or(format!("--repeats 要 ≥ 2，收到 {r}"))?,
        None => asr_probe::PROBE_REPEATS,
    };
    let temperature = vdn_eval::suite::option_value(args, "--temperature")?
        .map(|t| {
            t.parse::<f64>()
                .ok()
                .filter(|t| t.is_finite() && *t >= 0.0)
                .ok_or(format!("--temperature 要是 ≥ 0 的数，收到 {t}"))
        })
        .transpose()?;
    let cache: BTreeMap<String, String> = serde_json::from_str(
        &std::fs::read_to_string(&cache).map_err(|e| format!("读不了缓存：{e}"))?,
    )
    .map_err(|e| e.to_string())?;
    let langs = asr_probe::langs_by_sha(&cache);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("读不了 {}：{e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "wav"))
        .collect();
    files.sort();
    let mut items = Vec::new();
    for f in &files {
        let sha = vdn_eval::wav::sha256(f)?;
        let Some(lang) = langs.get(&sha) else {
            continue;
        };
        let (x, sr) = vdn_eval::wav::read(f)?;
        if sr != vdn_types::MODEL_RATE {
            return Err(format!("{} 不是 16 kHz", f.display()));
        }
        let name = f
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        items.push((name, sha, lang.clone(), x));
    }
    if items.is_empty() {
        return Err("目录里没有缓存认得的音频".into());
    }
    let root = vdn_eval::suite::main_root();
    let key =
        asr_or::load_key(&root).ok_or("没有 OPENROUTER_API_KEY（环境变量或仓库根 .env.local）")?;
    let ledger = vdn_eval::suite::option_value(args, "--asr-ledger")?
        .map(PathBuf::from)
        .unwrap_or_else(|| asr_ledger::ledger_path(&root));
    asr_ledger::read(&ledger)?;
    let mut c = asr_or::OpenRouter::new(asr_or::OR_URL, key, ledger);
    c.temperature = temperature;
    eprintln!(
        "探针：{} 条 × {repeats} 次，温度 {temperature:?}",
        items.len()
    );
    // 报告文件在所有检查之后、发第一个请求之前「只新建」地占住（create_new，原子），跑完写进这个已打开的文件；同路径已有就直接拒（codex 本分支第 5 轮 P2 #2）
    let mut report = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out)
        .map_err(|e| format!("{}：报告只新建，不覆盖（{e}）", out.display()))?;
    let r = asr_probe::probe(&mut c, &items, repeats, vdn_types::MODEL_RATE);
    {
        use std::io::Write;
        let text = serde_json::to_string_pretty(&r).map_err(|e| e.to_string())? + "\n";
        report
            .write_all(text.as_bytes())
            .map_err(|e| format!("写不了报告：{e}"))?;
    }
    println!(
        "探针：{} 条里 {} 条不一致（{:.1}%，线 {:.0}%）→ {}；报告 {}",
        r["audio"],
        r["unstable"],
        r["unstable_rate"].as_f64().unwrap_or(1.0) * 100.0,
        asr_probe::PROBE_MAX_UNSTABLE * 100.0,
        match r["pass"].as_bool() {
            Some(true) => "钉住了",
            Some(false) => "没钉住",
            None => "次数不是 3，只作探索、不下结论",
        },
        Path::new(&out).display()
    );
    Ok(r["pass"] == true)
}
