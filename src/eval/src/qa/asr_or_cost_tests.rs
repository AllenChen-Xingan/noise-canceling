//! OpenRouter 后端「费用与停发」那一组测试（预留与上限、待核、停发、费用未知、退避中的停发、密钥截断）。
//! 从 `asr_or_tests.rs` 拆出来（守 600 行）；假服务与小工具在那边。全部打本机假服务，不发真请求。

use serde_json::json;

use super::tests::{KEY, audio, client, fake, ok, tmpdir};
use super::*;

/// codex 第 5 轮 P1 #2：余额只差一点时，整批都不能发（每条先预留，放不下上限就不发）。
#[test]
fn near_budget_batch_sends_nothing_that_would_cross_the_cap() {
    let d = tmpdir("near");
    std::fs::write(
        d.join("ledger.json"),
        json!({"spent_usd": BUDGET_USD - 0.0001, "calls": 1}).to_string(),
    )
    .unwrap();
    let (url, seen) = fake(vec![ok("x", 0.00009); 6]);
    let mut c = client(&url, &d);
    let twelve_s: Vec<f32> = vec![0.1; 12 * vdn_types::MODEL_RATE as usize];
    let jobs: Vec<Job> = (0..6)
        .map(|_| (twelve_s.clone(), vdn_types::MODEL_RATE, "en".to_string()))
        .collect();
    let r = c.transcribe_many(&jobs, 8);
    assert!(
        r.iter()
            .all(|x| x.as_ref().is_err_and(|e| e.contains("上限"))),
        "{r:?}"
    );
    assert!(seen.lock().unwrap().is_empty(), "一条都不能发");
    assert!(c.ledger.spent_usd <= BUDGET_USD);
}

/// codex 第 5 轮 P1 #1：账本坏了拒发，坏文件原样留着。
#[test]
fn corrupt_ledger_refuses_to_send() {
    let d = tmpdir("corrupt");
    std::fs::write(d.join("ledger.json"), "garbage").unwrap();
    let (url, seen) = fake(vec![ok("x", 0.001)]);
    let mut c = client(&url, &d);
    let e = c
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
    assert!(e.contains("坏了"), "{e}");
    assert!(seen.lock().unwrap().is_empty());
    assert_eq!(
        std::fs::read_to_string(d.join("ledger.json")).unwrap(),
        "garbage"
    );
}

/// codex 第 6 轮 P1 #1：单线程跑一批，第一条遇到永久错误（404）后，后面的不再派发。
#[test]
fn permanent_error_stops_rest_of_batch() {
    let d = tmpdir("stopbatch");
    let deny = (
        404,
        "",
        json!({"error": {"code": 404, "message": "No endpoints found matching your data policy"}})
            .to_string(),
    );
    let (url, seen) = fake(vec![deny.clone(), deny.clone(), deny]);
    let mut c = client(&url, &d);
    let jobs: Vec<Job> = (0..3)
        .map(|_| (audio(), vdn_types::MODEL_RATE, "en".to_string()))
        .collect();
    let r = c.transcribe_many(&jobs, 1);
    assert_eq!(seen.lock().unwrap().len(), 1, "第一条 404 之后不再发");
    assert!(r.iter().all(Result::is_err));
    assert!(r[1].as_ref().unwrap_err().contains("停发"), "{:?}", r[1]);
}

/// codex 第 6 轮 P1 #2：200 带了 usage.cost 但缺 text → 照记这笔钱；响应读不出（费用未知）→ 预留不退、之后停发。
#[test]
fn known_cost_on_failure_is_booked_and_unknown_cost_keeps_reservation() {
    let d = tmpdir("costfail");
    let (url, _) = fake(vec![(
        200,
        "",
        json!({"usage": {"cost": 0.02}}).to_string(),
    )]);
    let mut c = client(&url, &d);
    assert!(c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").is_err());
    assert!((c.ledger.spent_usd - 0.02).abs() < 1e-12, "已知费用照记");
    assert_eq!(c.ledger.reserved_usd, 0.0);
    // codex 第 10 轮 P1 #1：失败分支的已知费用同样要核「超过预留」：0.02 美元远超 0.1 秒音频的预留 → 待核、停发、重启也拒
    assert!(
        c.ledger
            .review
            .as_deref()
            .is_some_and(|r| r.contains("超过预留")),
        "{:?}",
        c.ledger.review
    );
    assert!(c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").is_err());
    assert!(
        client(&url, &d)
            .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
            .is_err()
    );
    let d2 = tmpdir("costunknown");
    let (url2, seen2) = fake(vec![
        (200, "", "not json at all".to_string()),
        ok("never", 0.001),
    ]);
    let mut c2 = client(&url2, &d2);
    let e = c2
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
    assert!(e.contains("费用未知"), "{e}");
    assert!(c2.ledger.reserved_usd > 0.0, "费用未知：预留不退，等人核对");
    assert!(crate::qa::asr::is_permanent(&e), "费用未知要停发");
    assert_eq!(seen2.lock().unwrap().len(), 1);
}

/// codex 第 7 轮 P1 #1 · #2：有 text 没 usage.cost → 这条照用，但费用未知：预留不退、账本标「待核」；
/// 换个客户端（重启）也拒发，直到人清掉。
#[test]
fn text_without_cost_flags_ledger_for_review_across_restarts() {
    let d = tmpdir("review");
    let (url, seen) = fake(vec![
        (200, "", json!({"text": "hello"}).to_string()),
        ok("never", 0.001),
    ]);
    let mut c = client(&url, &d);
    assert_eq!(
        c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap(),
        "hello"
    );
    assert!(c.ledger.reserved_usd > 0.0, "预留不退");
    assert!(
        c.ledger
            .review
            .as_deref()
            .is_some_and(|r| r.contains("费用未知")),
        "{:?}",
        c.ledger.review
    );
    let e = c
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
    assert!(e.contains("待核"), "{e}");
    let mut c2 = client(&url, &d); // 重启
    assert!(
        c2.transcribe(&audio(), vdn_types::MODEL_RATE, "en")
            .unwrap_err()
            .contains("待核")
    );
    assert_eq!(seen.lock().unwrap().len(), 1, "待核期间一个请求都不发");
}

/// codex 第 7 轮 P1 #3：一条在 429 退避，另一条遇到 404：退避醒来不能再发。
#[test]
fn backoff_wakeup_checks_stop_before_resending() {
    let d = tmpdir("wake");
    // 第一个到的请求回 429（等 1 秒），第二个到的回 404；不管哪条先到，最终只能收到 2 个请求
    let (url, seen) = fake(vec![
        (429, "Retry-After: 1\r\n", json!({"error": {"code": 429, "message": "slow"}}).to_string()),
        (404, "", json!({"error": {"code": 404, "message": "No endpoints found matching your data policy"}}).to_string()),
        ok("must-not-arrive", 0.001),
    ]);
    let mut c = client(&url, &d);
    let jobs: Vec<Job> = (0..2)
        .map(|_| (audio(), vdn_types::MODEL_RATE, "en".to_string()))
        .collect();
    let r = c.transcribe_many(&jobs, 2);
    assert!(r.iter().all(Result::is_err), "{r:?}");
    assert_eq!(
        seen.lock().unwrap().len(),
        2,
        "404 之后，退避中的那条醒来不再发"
    );
}

/// codex 第 11 轮 P1 #1：A 在 429 退避时，别的进程把共用账本标了待核；A 醒来要先看账本，不再发。
#[test]
fn backoff_wakeup_checks_shared_ledger_review() {
    let d = tmpdir("xproc");
    let (url, seen) = fake(vec![
        (
            429,
            "Retry-After: 1\r\n",
            json!({"error": {"code": 429, "message": "slow"}}).to_string(),
        ),
        ok("must-not-arrive", 1e-7),
    ]);
    let ledger = d.join("ledger.json");
    let l2 = ledger.clone();
    let other = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        crate::qa::asr_ledger::flag_review(&l2, "别的进程：费用未知").unwrap();
    });
    let mut a = client(&url, &d);
    let e = a
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
    other.join().unwrap();
    assert!(e.contains("待核"), "{e}");
    assert_eq!(seen.lock().unwrap().len(), 1, "退避醒来不再发");
    assert_eq!(
        crate::qa::asr_ledger::read(&ledger).unwrap().reserved_usd,
        0.0,
        "没发出去的那次退掉预留"
    );
}

/// codex 第 9 轮 P1 #1：实际费用高于预留（估算不准）→ 照记实际金额，账本标待核、停发，不让后面的请求再把总额推高。
#[test]
fn actual_cost_above_reservation_flags_review_and_stops() {
    let d = tmpdir("overrun");
    let (url, seen) = fake(vec![ok("a", 1.0), ok("never", 0.001)]); // 1 美元远高于 0.1 秒音频的预留
    let mut c = client(&url, &d);
    assert_eq!(
        c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap(),
        "a",
        "文本照用"
    );
    assert!((c.ledger.spent_usd - 1.0).abs() < 1e-12, "实际金额照记");
    assert!(
        c.ledger
            .review
            .as_deref()
            .is_some_and(|r| r.contains("超过预留")),
        "{:?}",
        c.ledger.review
    );
    assert!(c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").is_err());
    assert_eq!(seen.lock().unwrap().len(), 1);
}

/// codex 第 8 轮 P2 #1：密钥跨过 300 字的截断边界时也不能漏出半截（先抹密钥，再截断）。
#[test]
fn key_straddling_truncation_boundary_is_scrubbed() {
    let d = tmpdir("scrub300");
    // 响应体前缀 `{"error":{"code":400,"message":"` 共 32 字；让密钥从第 290 字开始，跨过第 300 字的截断点
    let prefix = r#"{"error":{"code":400,"message":""#;
    let pad = "x".repeat(290 - prefix.len());
    let body = format!("{prefix}{pad}{KEY}\"}}}}");
    assert!(body.find(KEY).unwrap() < 300 && body.find(KEY).unwrap() + KEY.len() > 300);
    let (url, _) = fake(vec![(400, "", body)]);
    let mut c = client(&url, &d);
    let e = c
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
    assert!(!e.contains(&KEY[..10]), "密钥的前半截也不能出现：{e}");
}

/// codex 第 13 轮 P1 #1：非 200 的响应带了 usage.cost，也要照记；重试成功后账本 = 各次之和。
#[test]
fn cost_carried_by_non_200_responses_is_booked() {
    let d = tmpdir("cost503");
    let (url, _) = fake(vec![
        (
            503,
            "",
            json!({"error": {"code": 503, "message": "busy"}, "usage": {"cost": 1e-7}}).to_string(),
        ),
        ok("done", 2e-7),
    ]);
    let mut c = client(&url, &d);
    assert_eq!(
        c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap(),
        "done"
    );
    assert!(
        (c.ledger.spent_usd - 3e-7).abs() < 1e-15,
        "账本 {}",
        c.ledger.spent_usd
    );
    // 重试用完仍失败：前面各次带的费用照记，不当免费
    let d2 = tmpdir("cost503b");
    let busy = (
        503,
        "",
        json!({"error": {"code": 503, "message": "busy"}, "usage": {"cost": 1e-7}}).to_string(),
    );
    let (url2, _) = fake(vec![busy; MAX_ATTEMPTS]);
    let mut c2 = client(&url2, &d2);
    assert!(
        c2.transcribe(&audio(), vdn_types::MODEL_RATE, "en")
            .is_err()
    );
    assert!(
        (c2.ledger.spent_usd - MAX_ATTEMPTS as f64 * 1e-7).abs() < 1e-15,
        "账本 {}",
        c2.ledger.spent_usd
    );
}

/// codex 第 14 轮：503 带了费用、下一次却费用未知（返回没带金额）→ 前面那笔已知费用不能丢。
#[test]
fn earlier_known_cost_survives_a_later_unknown_cost() {
    let d = tmpdir("carrylost");
    let (url, _) = fake(vec![
        (
            503,
            "",
            json!({"error": {"code": 503, "message": "busy"}, "usage": {"cost": 1e-7}}).to_string(),
        ),
        (200, "", json!({"text": "no cost"}).to_string()),
    ]);
    let mut c = client(&url, &d);
    let _ = c.transcribe(&audio(), vdn_types::MODEL_RATE, "en");
    assert!(
        (c.ledger.spent_usd - 1e-7).abs() < 1e-15,
        "503 那笔照记：{}",
        c.ledger.spent_usd
    );
    assert!(c.ledger.review.is_some(), "后一次费用未知 → 待核");
}

/// codex 第 14 轮：重试中途累计费用超过这条的预留 → 当场标待核、停发，不再重试。
#[test]
fn retry_costs_over_reservation_stop_immediately() {
    let d = tmpdir("carryover");
    let pricey = (
        503,
        "",
        json!({"error": {"code": 503, "message": "busy"}, "usage": {"cost": 0.5}}).to_string(),
    );
    let (url, seen) = fake(vec![pricey.clone(), pricey, ok("never", 1e-7)]);
    let mut c = client(&url, &d);
    assert!(c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").is_err());
    assert_eq!(seen.lock().unwrap().len(), 1, "第一笔就超了预留，不再重试");
    assert!((c.ledger.spent_usd - 0.5).abs() < 1e-12);
    assert!(
        c.ledger
            .review
            .as_deref()
            .is_some_and(|r| r.contains("超过预留"))
    );
}
