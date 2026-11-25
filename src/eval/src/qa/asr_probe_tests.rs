//! 钉温度探针（大总管 2026-11-25）：打本机假服务，不发真请求。

use serde_json::{Value, json};

use super::super::asr_or::tests::{audio, client, fake, tmpdir};
use super::super::asr_probe::*;
#[allow(unused_imports)]
use super::*;

/// 设了温度就带上 `temperature`，没设就不带（不改原来的请求）。
#[test]
fn temperature_is_sent_only_when_set() {
    let d = tmpdir("probe-temp");
    let ok = |t: &str| {
        (
            200,
            "",
            json!({"text": t, "usage": {"seconds": 0.1, "cost": 1e-12}}).to_string(),
        )
    };
    let (url, seen) = fake(vec![ok("a"), ok("b")]);
    let mut c = client(&url, &d);
    c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap();
    c.temperature = Some(0.0);
    c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap();
    let bodies: Vec<Value> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|(_, b)| serde_json::from_slice(b).unwrap())
        .collect();
    assert!(bodies[0].get("temperature").is_none());
    assert_eq!(bodies[1]["temperature"], 0.0);
    assert_eq!(
        bodies[1]["provider"]["data_collection"], "deny",
        "钉温度不改数据政策"
    );
}

/// 每条调 `repeats` 次；分词后有一次不同就算不一致；不一致率 ≤ PROBE_MAX_UNSTABLE 才过。
#[test]
fn probe_counts_unstable_and_judges_against_line() {
    assert_eq!((PROBE_REPEATS, PROBE_MAX_UNSTABLE), (3, 0.02));
    let d = tmpdir("probe-run");
    let ok = |t: &str| {
        (
            200,
            "",
            json!({"text": t, "usage": {"seconds": 0.1, "cost": 1e-12}}).to_string(),
        )
    };
    // 两条音频 × 3 次：第一条三次只差标点（稳定），第二条第三次字词不同（不稳定）
    let (url, _) = fake(vec![
        ok("Hi, there."),
        ok("hi there"),
        ok("hi there!"),
        ok("a b"),
        ok("a b"),
        ok("a c"),
    ]);
    let mut c = client(&url, &d);
    let items = vec![
        (
            "one.wav".to_string(),
            "aa".to_string(),
            "en".to_string(),
            audio(),
        ),
        (
            "two.wav".to_string(),
            "bb".to_string(),
            "en".to_string(),
            audio(),
        ),
    ];
    let r = probe(&mut c, &items, 3, vdn_types::MODEL_RATE);
    assert_eq!(
        (r["audio"].as_u64(), r["unstable"].as_u64()),
        (Some(2), Some(1))
    );
    assert_eq!(r["unstable_rate"], 0.5);
    assert_eq!(r["pass"], false);
    assert_eq!(r["items"][1]["texts"].as_array().unwrap().len(), 3);
    assert_eq!(r["items"][0]["stable"], true);
}

/// codex 本分支第 2 轮 P2 #3：连调次数不是 PROBE_REPEATS 时只出数，不给过不过的结论。
#[test]
fn probe_with_other_repeats_gives_no_verdict() {
    let d = tmpdir("probe-two");
    let ok = |t: &str| {
        (
            200,
            "",
            json!({"text": t, "usage": {"seconds": 0.1, "cost": 1e-12}}).to_string(),
        )
    };
    let (url, _) = fake(vec![ok("a b"), ok("a b")]);
    let mut c = client(&url, &d);
    let items = vec![(
        "one.wav".to_string(),
        "aa".to_string(),
        "en".to_string(),
        audio(),
    )];
    let r = probe(&mut c, &items, 2, vdn_types::MODEL_RATE);
    assert_eq!(r["unstable"], 0);
    assert!(r["pass"].is_null(), "{r}");
}

/// 语言从旧缓存的键（`sha|型号|语言|次`）按音频 sha256 查；查不到的音频不进探针，单列。
#[test]
fn langs_come_from_cache_keys() {
    let cache: std::collections::BTreeMap<String, String> =
        [("s1|m|zh|1", "x"), ("s1|m|zh|2", "x"), ("s2|m|en|1", "y")]
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
    let m = langs_by_sha(&cache);
    assert_eq!(
        (
            m.get("s1").map(String::as_str),
            m.get("s2").map(String::as_str)
        ),
        (Some("zh"), Some("en"))
    );
    assert!(!m.contains_key("s3"));
}

/// codex 本分支第 3 轮 P1 #2：遇到永久错误（如数据政策 403）整套停发，剩下的不再发，标「没跑」。
#[test]
fn probe_stops_on_permanent_error() {
    let d = tmpdir("probe-403");
    let (url, seen) = fake(vec![(
        403,
        "",
        json!({"error": {"code": 403, "message": "no provider"}}).to_string(),
    )]);
    let mut c = client(&url, &d);
    let items = vec![
        (
            "one.wav".to_string(),
            "aa".to_string(),
            "en".to_string(),
            audio(),
        ),
        (
            "two.wav".to_string(),
            "bb".to_string(),
            "en".to_string(),
            audio(),
        ),
    ];
    let r = probe(&mut c, &items, 3, vdn_types::MODEL_RATE);
    assert_eq!(seen.lock().unwrap().len(), 1, "403 之后一条都不再发");
    assert!(
        r["halted"].as_str().is_some_and(|h| h.contains("403")),
        "{r}"
    );
    assert_eq!(r["pass"], false);
    assert_eq!(r["items"][1]["ran"], false);
}

/// codex 本分支第 4 轮 P1 #2：前面都稳定、最后一条遇到永久错误停发，也不能判「钉住了」。
#[test]
fn halted_probe_never_passes() {
    // 60 条：前 59 条三次都一样，最后一条第一次就 403。不一致率 1/60 ≈ 1.7% 在 2% 线以下，但停发了，不能判过
    let d = tmpdir("probe-last403");
    let ok = |t: &str| {
        (
            200,
            "",
            json!({"text": t, "usage": {"seconds": 0.1, "cost": 1e-12}}).to_string(),
        )
    };
    let mut script: Vec<(u16, &'static str, String)> = (0..59 * 3).map(|_| ok("a b")).collect();
    script.push((403, "", json!({"error": {"code": 403}}).to_string()));
    let (url, _) = fake(script);
    let mut c = client(&url, &d);
    let items: Vec<_> = (0..60)
        .map(|k| (format!("w{k}"), format!("s{k}"), "en".to_string(), audio()))
        .collect();
    let r = probe(&mut c, &items, 3, vdn_types::MODEL_RATE);
    assert!(r["halted"].is_string());
    assert!(
        r["unstable_rate"].as_f64().unwrap() <= PROBE_MAX_UNSTABLE,
        "{}",
        r["unstable_rate"]
    );
    assert_eq!(r["pass"], false, "停发了就不算钉住");
}
