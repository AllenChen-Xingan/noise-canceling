//! 钉温度探针（大总管 2026-11-25 定 C）：同一批音频每条连调 [`PROBE_REPEATS`] 次，按评分用的分词
//! （[`super::asr::same_text`]）看几次是否一致；不一致率 ≤ [`PROBE_MAX_UNSTABLE`] 才算钉住。
//! 钉不住就改用「有分歧才调第 3 次、取多数」（大总管定的 A），那时再写它的正本常量。
//!
//! 每次调用照常走 [`super::asr_or::OpenRouter`]：预留、按「返回费用与标价估算取大」入账、逐笔明细、`data_collection=deny`。

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::asr::same_text;
use super::asr_or::OpenRouter;

/// 每条音频连调几次（大总管 2026-11-25）。
pub const PROBE_REPEATS: usize = 3;
/// 过线：不一致的音频占比上限（大总管 2026-11-25：≤ 2%）。
pub const PROBE_MAX_UNSTABLE: f64 = 0.02;

/// 旧缓存（`asr_cache.json`）里每段音频的语言：键是 `sha|型号|语言|次`。
pub fn langs_by_sha(cache: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    cache
        .keys()
        .filter_map(|k| {
            let mut it = k.split('|');
            let sha = it.next()?;
            let _model = it.next()?;
            Some((sha.to_string(), it.next()?.to_string()))
        })
        .collect()
}

/// 跑探针。`items` = (文件名, sha256, 语言, 16 kHz 样本)。出错的那次记进 `errors`，整条算不一致（偏保守）。
pub fn probe(
    c: &mut OpenRouter,
    items: &[(String, String, String, Vec<f32>)],
    repeats: usize,
    sr: u32,
) -> Value {
    let mut rows = Vec::new();
    let mut unstable = 0usize;
    // 永久错误（不重试的 4xx、到上限、账本待核、费用未知…）一出现就整套停发，剩下的标「没跑」（codex 本分支第 3 轮 P1 #2）
    let mut halted: Option<String> = None;
    for (name, sha, lang, x) in items {
        if halted.is_some() {
            unstable += 1;
            rows.push(
                json!({"wav": name, "sha256": sha, "lang": lang, "ran": false, "stable": false}),
            );
            continue;
        }
        let mut texts = Vec::new();
        let mut errors = Vec::new();
        for _ in 0..repeats {
            match c.transcribe(x, sr, lang) {
                Ok(t) => texts.push(t),
                Err(e) => {
                    let stop = super::asr_or::is_permanent(&e) || e.contains("HTTP 4");
                    errors.push(e.clone());
                    if stop {
                        halted = Some(e);
                        break;
                    }
                }
            }
        }
        let stable = errors.is_empty() && texts.iter().all(|t| same_text(t, &texts[0], lang));
        unstable += usize::from(!stable);
        rows.push(json!({"wav": name, "sha256": sha, "lang": lang, "ran": true, "texts": texts, "errors": errors, "stable": stable}));
    }
    let n = items.len();
    let rate = if n > 0 {
        unstable as f64 / n as f64
    } else {
        1.0
    };
    json!({"model": super::asr_or::OR_MODEL, "temperature": c.temperature, "repeats": repeats,
        "audio": n, "unstable": unstable, "unstable_rate": rate,
        "max_unstable": PROBE_MAX_UNSTABLE,
        // 正式结论只在连调 PROBE_REPEATS 次时给；别的次数只是探索，pass 为 null（codex 本分支第 2 轮 P2 #3）
        "halted": halted,
        "pass": (repeats == PROBE_REPEATS).then_some(n > 0 && halted.is_none() && rate <= PROBE_MAX_UNSTABLE), "items": rows})
}
