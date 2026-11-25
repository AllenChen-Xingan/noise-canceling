//! 识别费入账与逐笔明细（大总管 2026-11-25：后台用量与账本差约 10 倍，人说密钥别处也用过）。
//! 入账取「返回的 usage.cost」与「音频秒数 × 标价」中大的；每次 HTTP 尝试都记一行明细，以后逐笔核。

use serde_json::{Value, json};

use super::tests::{KEY, audio, client, fake, tmpdir};
use super::*;

fn details(d: &std::path::Path) -> Vec<Value> {
    let p = ledger::details_path(&d.join("ledger.json"));
    std::fs::read_to_string(p)
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// 返回的费用低于标价估算时按估算记；高于时照返回记。
#[test]
fn booked_cost_is_larger_of_reported_and_list() {
    let secs = audio().len() as f64 / f64::from(vdn_types::MODEL_RATE);
    let list = secs * PRICE_PER_SECOND;
    assert_eq!(booked_cost(1e-12, secs), list);
    assert_eq!(booked_cost(list * 1.2, secs), list * 1.2);
    let d = tmpdir("bill-max");
    let ok = |t: &str, c: f64| {
        (
            200,
            "",
            json!({"text": t, "usage": {"seconds": secs, "cost": c}}).to_string(),
        )
    };
    let (url, _) = fake(vec![ok("a", 1e-12), ok("b", list * 1.2)]);
    let mut c = client(&url, &d);
    c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap();
    c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap();
    assert!(
        (c.ledger.spent_usd - list * 2.2).abs() < 1e-15,
        "{}",
        c.ledger.spent_usd
    );
}

/// 每次尝试一行：状态码、生成 id（返回体 `id` 或响应头 `x-generation-id`）、usage 全文（含 tokens）、
/// 响应头（去掉 set-cookie）、入账金额；不含密钥、不含转写原文。重试也各记一行。
#[test]
fn every_attempt_is_logged_with_id_usage_and_headers() {
    let d = tmpdir("bill-detail");
    let usage = json!({"seconds": 0.1, "cost": 1e-12, "input_tokens": 5, "output_tokens": 2, "total_tokens": 7});
    let (url, _) = fake(vec![
        (
            503,
            "X-Generation-Id: gen-retry-1\r\nSet-Cookie: s=secret\r\n",
            json!({"error": {"code": 503}}).to_string(),
        ),
        (
            200,
            "X-Generation-Id: gen-ok-2\r\n",
            json!({"id": "gen-ok-2", "text": "private words", "usage": usage}).to_string(),
        ),
    ]);
    let mut c = client(&url, &d);
    c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap();
    let rows = details(&d);
    assert_eq!(rows.len(), 2, "重试也记：{rows:?}");
    assert_eq!(
        (rows[0]["attempt"].as_u64(), rows[0]["status"].as_u64()),
        (Some(1), Some(503))
    );
    assert_eq!(rows[0]["id"], "gen-retry-1");
    assert!(rows[0]["headers"].get("set-cookie").is_none());
    assert_eq!(rows[1]["status"], 200);
    assert_eq!(rows[1]["id"], "gen-ok-2");
    assert_eq!(rows[1]["usage"], usage);
    let secs = audio().len() as f64 / f64::from(vdn_types::MODEL_RATE);
    assert_eq!(
        rows[1]["booked_usd"].as_f64(),
        Some(booked_cost(1e-12, secs))
    );
    assert!(rows[1]["at"].as_str().is_some_and(|s| s.ends_with('Z')));
    let text = std::fs::read_to_string(ledger::details_path(&d.join("ledger.json"))).unwrap();
    assert!(
        !text.contains(KEY) && !text.contains("secret") && !text.contains("private words"),
        "{text}"
    );
}

/// 连不上（超时同理）也记一行：状态 0，错误原因（抹掉密钥）。
#[test]
fn unreachable_attempt_is_logged() {
    let d = tmpdir("bill-down");
    let mut c = client("http://127.0.0.1:9/api/v1/audio/transcriptions", &d);
    assert!(c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").is_err());
    let rows = details(&d);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["status"], 0);
    assert!(
        rows[0]["error"]
            .as_str()
            .is_some_and(|e| e.contains("连不上"))
    );
}
