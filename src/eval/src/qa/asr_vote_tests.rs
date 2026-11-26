//! 有分歧才调第 3 次、取多数（A，正本 `asr_vote.rs`）接进 [`Asr`] 后的行为：串行取值与预取都一样，
//! 只在前两次不同时多发 1 次，已缓存的不再发。全部打本机假服务（按脚本依次回复）。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use super::*;
use crate::qa::asr_or::MAX_ATTEMPTS;
use crate::qa::asr_or::tests::{audio, client, fake, ok, tmpdir};
use crate::qa::asr_vote::TIE_BREAK_REP;

const M: &str = crate::qa::asr_or::OR_MODEL;

/// 读一个请求（请求头 + 正文），返回正文。假服务共用。
fn read_req(c: &mut TcpStream) -> Vec<u8> {
    let mut r = BufReader::new(c.try_clone().unwrap());
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            len = v.trim().parse().unwrap_or(0);
        }
    }
    let mut b = vec![0u8; len];
    r.read_exact(&mut b).unwrap();
    b
}

/// 一次回的响应（正文已拼好）。假服务共用。
fn respond(c: &mut TcpStream, code: u16, body: &str) {
    let resp = format!(
        "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = c.write_all(resp.as_bytes());
}

/// 固定回同一个错误的假服务（照 `asr_conc_tests` 的 `error_server`）。返回 (地址, 收到的请求数)。
fn error_server(code: u16, msg: &'static str) -> (String, Arc<AtomicUsize>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/api/v1/audio/transcriptions",
        l.local_addr().unwrap()
    );
    let total = Arc::new(AtomicUsize::new(0));
    let t2 = total.clone();
    std::thread::spawn(move || {
        for c in l.incoming() {
            let Ok(mut c) = c else { return };
            let total = t2.clone();
            std::thread::spawn(move || {
                let _ = read_req(&mut c);
                total.fetch_add(1, Ordering::SeqCst);
                respond(
                    &mut c,
                    code,
                    &json!({"error": {"code": code, "message": msg}}).to_string(),
                );
            });
        }
    });
    (url, total)
}

/// 按音频内容 + 这条音频已被问过几次回话：第 1 次 `a k…`、第 2 次 `b k…`、第 3 次起 `a k…`（多数总是 `a`）。
/// 返回 (地址, 收到的请求数)。
fn cycle_server() -> (String, Arc<AtomicUsize>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/api/v1/audio/transcriptions",
        l.local_addr().unwrap()
    );
    let total = Arc::new(AtomicUsize::new(0));
    let per: Arc<Mutex<BTreeMap<String, usize>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let (t2, p2) = (total.clone(), per.clone());
    std::thread::spawn(move || {
        for c in l.incoming() {
            let Ok(mut c) = c else { return };
            let (total, per) = (t2.clone(), p2.clone());
            std::thread::spawn(move || {
                let b = read_req(&mut c);
                total.fetch_add(1, Ordering::SeqCst);
                let v: Value = serde_json::from_slice(&b).unwrap();
                let d = v["input_audio"]["data"].as_str().unwrap().to_string();
                let n = {
                    let mut m = per.lock().unwrap();
                    let e = m.entry(d.clone()).or_insert(0);
                    let n = *e;
                    *e += 1;
                    n
                };
                std::thread::sleep(std::time::Duration::from_millis(2));
                let tag = if n == 1 { "b" } else { "a" };
                let sum: u64 = d.bytes().map(u64::from).sum();
                respond(
                    &mut c,
                    200,
                    &json!({"text": format!("{tag} k{sum}"),
                        "usage": {"seconds": 0.1, "cost": 1e-9}})
                    .to_string(),
                );
            });
        }
    });
    (url, total)
}

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

/// 串行取值时第 3 次撞上不重试的错误（4xx）：`transcribe` 报错，但「该补调」这条要留下（`tie_break_missing`），
/// 不能因为提前返回就不算（评审发现：补调条数少计）。
#[test]
fn third_call_failure_serial_is_counted_as_missing() {
    let d = tmpdir("vote-3err");
    let it = items(&d, 1);
    seed(&d.join("cache.json"), &it, &[&["a b", "a c"]]);
    let (url, seen) = fake(vec![(
        400,
        "",
        json!({"error": {"code": 400, "message": "bad audio"}}).to_string(),
    )]);
    let mut a = asr(&url, &d);
    let (p, sha, lang, clip) = &it[0];
    assert!(a.transcribe(p, sha, lang, clip).is_err());
    assert_eq!(seen.lock().unwrap().len(), 1, "只发第 3 次，4xx 不重试");
    let s = a.summary();
    assert_eq!(
        (
            s["tie_breaks"].as_u64(),
            s["tie_break_missing"].as_u64(),
            s["no_majority"].as_u64()
        ),
        (Some(1), Some(1), Some(0))
    );
}

/// 同上，第 3 次是 5xx、重试用尽：照样留 `tie_break_missing`。
#[test]
fn third_call_retries_exhausted_is_counted_as_missing() {
    let d = tmpdir("vote-3busy");
    let it = items(&d, 1);
    seed(&d.join("cache.json"), &it, &[&["a b", "a c"]]);
    let busy = || {
        (
            503,
            "",
            json!({"error": {"code": 503, "message": "busy"}}).to_string(),
        )
    };
    let (url, seen) = fake((0..MAX_ATTEMPTS).map(|_| busy()).collect());
    let mut a = asr(&url, &d);
    let (p, sha, lang, clip) = &it[0];
    let e = a.transcribe(p, sha, lang, clip).unwrap_err();
    assert!(e.contains(&format!("{MAX_ATTEMPTS} 次")), "{e}");
    assert_eq!(seen.lock().unwrap().len(), MAX_ATTEMPTS);
    assert_eq!(a.summary()["tie_break_missing"].as_u64(), Some(1));
}

/// 预取时第 3 次失败：预取当场记下「该补调」；随后串行取值照旧报错，`tie_break_missing` 仍是 1（不被清掉）。
#[test]
fn third_call_failure_through_prefetch_is_counted_as_missing() {
    let d = tmpdir("vote-3err-pre");
    let it = items(&d, 1);
    seed(&d.join("cache.json"), &it, &[&["a b", "a c"]]);
    let (url, seen) = fake(vec![(
        400,
        "",
        json!({"error": {"code": 400, "message": "bad audio"}}).to_string(),
    )]);
    let mut a = asr(&url, &d);
    a.prefetch(&it);
    assert_eq!(seen.lock().unwrap().len(), 1, "预取只补调第 3 次");
    let (p, sha, lang, clip) = &it[0];
    assert!(a.transcribe(p, sha, lang, clip).is_err());
    let s = a.summary();
    assert_eq!(
        (
            s["tie_breaks"].as_u64(),
            s["tie_break_missing"].as_u64(),
            s["no_majority"].as_u64()
        ),
        (Some(1), Some(1), Some(0))
    );
    assert_eq!(seen.lock().unwrap().len(), 1, "失败的那次不再重发");
}

/// 停发之后：前两次已缓存且不同的音频该补调第 3 次，但一个请求都不许再发；照样留 `tie_break_missing`。
#[test]
fn halted_sending_sends_no_third_request() {
    let d = tmpdir("vote-halt");
    let it = items(&d, 2);
    // 只给第 2 条音频预先写第 1、2 次（不同）
    seed(&d.join("cache.json"), &it[1..], &[&["a b", "a c"]]);
    let (url, total) = error_server(404, "No endpoints found matching your data policy");
    let mut a = asr(&url, &d);
    let (p, sha, lang, clip) = &it[0];
    let e = a.transcribe(p, sha, lang, clip).unwrap_err();
    assert!(e.contains("不重试"), "{e}");
    assert_eq!(total.load(Ordering::SeqCst), 1);
    let (p, sha, lang, clip) = &it[1];
    let e = a.transcribe(p, sha, lang, clip).unwrap_err();
    assert!(e.contains("停发"), "{e}");
    assert_eq!(total.load(Ordering::SeqCst), 1, "停发后第 3 次不发");
    let s = a.summary();
    assert_eq!(
        (
            s["tie_breaks"].as_u64(),
            s["tie_break_missing"].as_u64(),
            s["no_majority"].as_u64()
        ),
        (Some(1), Some(1), Some(0))
    );
}

/// 并发与串行必须逐条相同：假服务按「这条音频第几次被问到」回 `a` · `b` · `a`（多数总是 `a`，与请求到达先后无关），
/// 文本里带音频校验和，好分辨是哪条。串行逐条转写 vs 预取 8 路再逐条取值。
#[test]
fn concurrent_matches_serial_with_content_dependent_replies() {
    let d = tmpdir("vote-conc-eq");
    let it = items(&d, 6);
    let run = |tag: &str, prefetch: bool| {
        let dir = d.join(tag);
        std::fs::create_dir_all(&dir).unwrap();
        let (url, total) = cycle_server();
        let mut a = asr(&url, &dir);
        if prefetch {
            a.workers = 8;
            a.prefetch(&it);
        }
        let got: Vec<(String, bool)> = it
            .iter()
            .map(|(p, sha, lang, clip)| a.transcribe(p, sha, lang, clip).unwrap())
            .collect();
        let s = a.summary();
        let nums = (
            s["checked"].as_u64().unwrap(),
            s["unstable"].as_u64().unwrap(),
            s["tie_breaks"].as_u64().unwrap(),
            s["no_majority"].as_u64().unwrap(),
        );
        (got, nums, total.load(Ordering::SeqCst))
    };
    let (serial, sn, sreq) = run("serial", false);
    let (conc, cn, creq) = run("conc", true);
    assert_eq!(serial, conc, "串行与并发逐条相同");
    assert_eq!(sn, cn, "统计相同");
    assert_eq!(sn, (6, 6, 6, 0), "6 条都前两次不同、取到多数");
    assert!(serial.iter().all(|(t, ok)| *ok && t.starts_with("a k")), "{serial:?}");
    assert_eq!((sreq, creq), (18, 18), "每条 3 个请求");
}

/// 单次模式（`VDN_ASR_REPEATS=1`）：没被抽中的音频只转 1 次，不补调，也不记进补调统计。
#[test]
fn single_mode_one_call_and_no_vote() {
    let d = tmpdir("vote-single");
    let it = items(&d, 20);
    let (p, sha, lang, clip) = it
        .iter()
        .find(|(_, sha, _, _)| repeats_for(sha, true) == 1)
        .expect("20 条音频里应当有没被抽中的");
    let (url, seen) = fake(vec![ok("only once", 1e-9)]);
    let mut a = asr(&url, &d);
    a.single = true;
    assert_eq!(
        a.transcribe(p, sha, lang, clip).unwrap(),
        ("only once".to_string(), true)
    );
    assert_eq!(seen.lock().unwrap().len(), 1, "只转 1 次");
    let s = a.summary();
    assert_eq!(
        (
            s["tie_breaks"].as_u64(),
            s["tie_break_missing"].as_u64()
        ),
        (Some(0), Some(0))
    );
    assert_eq!((a.checked, a.unstable), (0, 0));
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
