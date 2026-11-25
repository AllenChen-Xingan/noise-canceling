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
