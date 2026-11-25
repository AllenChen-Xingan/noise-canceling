//! 有分歧才调第 3 次、取多数（A，正本 `asr_vote.rs`）接进 [`Asr`] 后的行为：串行取值与预取都一样，
//! 只在前两次不同时多发 1 次，已缓存的不再发。全部打本机假服务（按脚本依次回复）。

use std::path::PathBuf;

use super::*;
use crate::qa::asr_or::tests::{audio, client, fake, ok, tmpdir};
use crate::qa::asr_vote::TIE_BREAK_REP;

const M: &str = crate::qa::asr_or::OR_MODEL;

/// 一个目录里 n 条互不相同的音频（路径, sha256, 语言, 片段 id）。
fn items(d: &std::path::Path, n: usize) -> Vec<(PathBuf, String, String, String)> {
    (0..n)
        .map(|k| {
            let x: Vec<f32> = audio().iter().map(|v| v * (1.0 + k as f32 * 0.1)).collect();
            let p = d.join(format!("v{k}.wav"));
            crate::wav::write(&p, &x, vdn_types::MODEL_RATE).unwrap();
            let sha = crate::wav::sha256(&p).unwrap();
            (p, sha, "en".to_string(), format!("cafe-c{k:02}-mba#00"))
        })
        .collect()
}

/// 预置缓存：每条音频给出已有的几次（第 1 次起）。
fn seed(cache: &std::path::Path, it: &[(PathBuf, String, String, String)], texts: &[&[&str]]) {
    let mut m = BTreeMap::new();
    for ((_, sha, lang, _), ts) in it.iter().zip(texts) {
        for (r, t) in ts.iter().enumerate() {
            m.insert(format!("{sha}|{M}|{lang}|{}", r + 1), t.to_string());
        }
    }
    std::fs::write(cache, serde_json::to_string(&m).unwrap()).unwrap();
}

fn asr(url: &str, d: &std::path::Path) -> Asr {
    Asr::with_backends(
        d.join("cache.json"),
        vec![Backend::OpenRouter(Box::new(client(url, d)))],
        OPENROUTER,
        Route::default(),
    )
}

/// 串行取值：前两次不同 → 补调第 3 次；第 3 次同第 2 次就用第 2 次的文本，判一致；同第 1 次就用第 1 次。
#[test]
fn transcribe_breaks_tie_with_third_call() {
    let d = tmpdir("vote-serial");
    let it = items(&d, 2);
    seed(
        &d.join("cache.json"),
        &it,
        &[&["a b", "a c"], &["x y", "x z"]],
    );
    let (url, seen) = fake(vec![ok("A c.", 1e-9), ok("x y", 1e-9)]);
    let mut a = asr(&url, &d);
    let (p, sha, lang, clip) = &it[0];
    assert_eq!(
        a.transcribe(p, sha, lang, clip).unwrap(),
        ("a c".to_string(), true)
    );
    let (p, sha, lang, clip) = &it[1];
    assert_eq!(
        a.transcribe(p, sha, lang, clip).unwrap(),
        ("x y".to_string(), true)
    );
    assert_eq!(seen.lock().unwrap().len(), 2, "每条只多发 1 次");
    let cache: BTreeMap<String, String> =
        serde_json::from_str(&std::fs::read_to_string(d.join("cache.json")).unwrap()).unwrap();
    assert_eq!(
        cache[&format!("{}|{M}|en|{TIE_BREAK_REP}", it[0].1)],
        "A c.",
        "第 3 次照常进缓存"
    );
    // 前两次的不一致照旧计（这个率决定以后能不能改 1 次）；补调与结果另计
    assert_eq!((a.checked, a.unstable), (2, 2));
    let s = a.summary();
    assert_eq!(
        (s["tie_breaks"].as_u64(), s["no_majority"].as_u64()),
        (Some(2), Some(0))
    );
}

/// 三次都不同：判不一致（评分那边转成「不知道」）；前两次相同：一个请求都不多发。
#[test]
fn no_majority_is_unstable_and_agreement_sends_nothing() {
    let d = tmpdir("vote-none");
    let it = items(&d, 2);
    seed(
        &d.join("cache.json"),
        &it,
        &[&["a b", "a c"], &["same", "Same."]],
    );
    let (url, seen) = fake(vec![ok("a d", 1e-9)]);
    let mut a = asr(&url, &d);
    let (p, sha, lang, clip) = &it[0];
    assert!(!a.transcribe(p, sha, lang, clip).unwrap().1);
    let (p, sha, lang, clip) = &it[1];
    assert_eq!(
        a.transcribe(p, sha, lang, clip).unwrap(),
        ("same".to_string(), true)
    );
    assert_eq!(seen.lock().unwrap().len(), 1);
    assert_eq!(a.summary()["no_majority"].as_u64(), Some(1));
}

/// 预取：先发缺的前两次，前两次不同的再并发补第 3 次；之后串行取值全部命中缓存，结果与不预取相同。
#[test]
fn prefetch_sends_third_call_only_on_disagreement() {
    let d = tmpdir("vote-pre");
    let it = items(&d, 3);
    // 第 1 条两次不同、第 2 条两次相同、第 3 条只缓存了第 1 次
    seed(
        &d.join("cache.json"),
        &it,
        &[&["a b", "a c"], &["k", "k"], &["m n"]],
    );
    // 预取第一批只发第 3 条的第 2 次（回「m o」，与第 1 次不同）；第二批补第 1、3 条的第 3 次
    let (url, seen) = fake(vec![ok("m o", 1e-9), ok("a b", 1e-9), ok("m o", 1e-9)]);
    let mut a = asr(&url, &d);
    a.workers = 1; // 假服务按到达顺序回，单路才对得上脚本
    a.prefetch(&it);
    assert_eq!(seen.lock().unwrap().len(), 3);
    let got: Vec<(String, bool)> = it
        .iter()
        .map(|(p, sha, lang, clip)| a.transcribe(p, sha, lang, clip).unwrap())
        .collect();
    assert_eq!(seen.lock().unwrap().len(), 3, "取值时一个都不再发");
    assert!(got.iter().all(|g| g.1), "{got:?}");
    assert_eq!(got[0].0, "a b");
    assert_eq!(got[1].0, "k");
    assert_eq!(got[2].0, "m o", "第 2、3 次相同，用第 2 次");
    assert_eq!((a.checked, a.unstable), (3, 2));
    assert_eq!(a.summary()["tie_breaks"].as_u64(), Some(2));
}

/// 三次都已缓存（补跑后重跑）：一个请求都不发，判法照旧。
#[test]
fn fully_cached_votes_send_nothing() {
    let d = tmpdir("vote-cached");
    let it = items(&d, 1);
    seed(&d.join("cache.json"), &it, &[&["a b", "a c", "a c"]]);
    let (url, seen) = fake(vec![]);
    let mut a = asr(&url, &d);
    a.prefetch(&it);
    let (p, sha, lang, clip) = &it[0];
    assert_eq!(
        a.transcribe(p, sha, lang, clip).unwrap(),
        ("a c".to_string(), true)
    );
    assert!(seen.lock().unwrap().is_empty());
}
