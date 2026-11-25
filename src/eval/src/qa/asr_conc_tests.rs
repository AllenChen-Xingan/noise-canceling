//! 识别并发的测试（大总管 2026-11-25：默认 8 路；并发与串行的结果必须逐字节相同）。全部打本机假服务。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};

use super::*;
use crate::qa::asr_or::{self, OpenRouter};

/// 并发假服务：每个连接一个线程；转写 = 音频 base64 的长度与校验和（不同音频不同、同一音频永远相同）；
/// 每次回复前睡 `delay_ms`，好让请求在服务端重叠。`fail_first_429`：前这么多个请求先回 429。
/// 返回 (地址, 同时在处理的最大请求数, 收到的请求总数)。
fn echo_server(delay_ms: u64, fail_first_429: usize) -> (String, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/api/v1/audio/transcriptions", l.local_addr().unwrap());
    let (live, peak, total) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (p2, t2) = (peak.clone(), total.clone());
    std::thread::spawn(move || {
        for c in l.incoming() {
            let Ok(mut c) = c else { return };
            let (live, peak, total) = (live.clone(), p2.clone(), t2.clone());
            std::thread::spawn(move || {
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
                let n = total.fetch_add(1, Ordering::SeqCst);
                let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(now, Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                live.fetch_sub(1, Ordering::SeqCst);
                let (code, body) = if n < fail_first_429 {
                    (429, json!({"error": {"code": 429, "message": "slow down"}}).to_string())
                } else {
                    let v: Value = serde_json::from_slice(&b).unwrap();
                    let d = v["input_audio"]["data"].as_str().unwrap();
                    let sum: u64 = d.bytes().map(u64::from).sum();
                    (200, json!({"text": format!("t{} s{sum}", d.len()), "usage": {"cost": 0.0001}}).to_string())
                };
                let extra = if code == 429 { "Retry-After: 0\r\n" } else { "" };
                let resp = format!(
                    "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = c.write_all(resp.as_bytes());
            });
        }
    });
    (url, peak, total)
}

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vdn-asr-conc-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// 12 条互不相同的短音频。
fn jobs() -> Vec<asr_or::Job> {
    (0..12)
        .map(|k| ((0..800).map(|i| ((i * (k + 1)) as f32 * 0.01).sin() * 0.3).collect(), 16_000, "en".to_string()))
        .collect()
}

fn client(url: &str, dir: &std::path::Path) -> OpenRouter {
    let mut c = OpenRouter::new(url, "sk-test".into(), dir.join("ledger.json"));
    c.backoff_ms = 1;
    c
}

#[test]
fn concurrent_batch_equals_serial_and_really_overlaps() {
    let (url, peak, _) = echo_server(40, 0);
    let d = tmpdir("batch");
    let serial = client(&url, &d.join("s")).transcribe_many(&jobs(), 1);
    assert_eq!(peak.load(Ordering::SeqCst), 1, "1 路时不该重叠");
    let (url8, peak8, _) = echo_server(40, 0);
    let mut c8 = client(&url8, &d.join("c"));
    let conc = c8.transcribe_many(&jobs(), asr_or::ASR_CONCURRENCY);
    assert_eq!(serial, conc, "结果按输入顺序，和并发与否无关");
    assert!(conc.iter().all(Result::is_ok));
    let p = peak8.load(Ordering::SeqCst);
    assert!(p > 1 && p <= asr_or::ASR_CONCURRENCY, "同时在途 {p} 路");
    assert_eq!(c8.ledger.calls, 12);
    assert!((c8.ledger.spent_usd - 12.0 * 0.0001).abs() < 1e-12, "每条只记一次账");
}

#[test]
fn rate_limited_requests_back_off_and_batch_still_matches() {
    let (url, _, total) = echo_server(5, 3);
    let d = tmpdir("429");
    let got = client(&url, &d).transcribe_many(&jobs(), asr_or::ASR_CONCURRENCY);
    let (url1, _, _) = echo_server(0, 0);
    let want = client(&url1, &d.join("ref")).transcribe_many(&jobs(), 1);
    assert_eq!(got, want);
    assert_eq!(total.load(Ordering::SeqCst), 12 + 3, "3 个 429 各重发了一次");
}

/// 成绩单那一层：先预取（并发）再逐条转写，与不预取、逐条串行转写，转写结果与缓存文件逐字节相同。
#[test]
fn prefetch_then_transcribe_equals_plain_serial() {
    let d = tmpdir("prefetch");
    let mut items = Vec::new();
    for (k, (x, sr, lang)) in jobs().into_iter().enumerate() {
        let p = d.join(format!("a{k}.wav"));
        crate::wav::write(&p, &x, sr).unwrap();
        let sha = crate::wav::sha256(&p).unwrap();
        items.push((p, sha, lang, format!("cafe-c{k:02}-mba#00")));
    }
    let run = |tag: &str, prefetch: bool| -> (Vec<(String, bool)>, String) {
        let (url, _, _) = echo_server(10, 0);
        let dir = d.join(tag);
        let or = client(&url, &dir);
        let cache = dir.join("asr_cache.json");
        let mut a = Asr::with_backends(cache.clone(), vec![Backend::OpenRouter(or)], OPENROUTER, Route::default());
        if prefetch {
            a.prefetch(&items);
        }
        let out = items.iter().map(|(p, sha, lang, clip)| a.transcribe(p, sha, lang, clip).unwrap()).collect();
        (out, std::fs::read_to_string(cache).unwrap())
    };
    let (plain, cache_plain) = run("plain", false);
    let (pre, cache_pre) = run("pre", true);
    assert_eq!(plain, pre);
    assert_eq!(cache_plain, cache_pre, "缓存文件逐字节相同（按 sha256 排序写，和写的先后无关）");
    assert!(plain.iter().all(|(_, stable)| *stable));
}

#[test]
fn spot_check_sampling_is_deterministic_and_about_ten_percent() {
    assert_eq!(repeats_for("00000000abc", false), REPEATS);
    assert_eq!(repeats_for("00000000abc", true), REPEATS, "sha 很小 → 抽中");
    assert_eq!(repeats_for("ffffffffabc", true), 1);
    let n = 10_000u64;
    let hit = (0..n)
        .map(|i| format!("{:08x}", (i.wrapping_mul(2_654_435_761)) as u32))
        .filter(|s| repeats_for(s, true) == REPEATS)
        .count() as f64;
    assert!((hit / n as f64 - SPOT_FRACTION).abs() < 0.01, "抽中比例 {}", hit / n as f64);
    assert_eq!(MAX_UNSTABLE_RATE, 0.0);
}
