//! OpenRouter 后端的测试：全部打本机假服务（127.0.0.1 随机端口），不发真请求。
//! 真调用的一条标 `#[ignore]`，等密钥到了人工跑。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use super::*;

/// 一次收到的请求：(请求头原文, 请求体)。
type Seen = Arc<Mutex<Vec<(String, Vec<u8>)>>>;

/// 起一个假服务，按顺序回 `script` 里的 (状态码, 额外响应头, 响应体)；回完就退出。返回 (地址, 收到的请求)。
fn fake(script: Vec<(u16, &'static str, String)>) -> (String, Seen) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://{}/api/v1/audio/transcriptions",
        l.local_addr().unwrap()
    );
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let s2 = seen.clone();
    std::thread::spawn(move || {
        for (code, extra, body) in script {
            let Ok((mut c, _)) = l.accept() else { return };
            let mut r = BufReader::new(c.try_clone().unwrap());
            let (mut head, mut len) = (String::new(), 0usize);
            loop {
                let mut line = String::new();
                if r.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap_or(0);
                }
                head += &line;
            }
            let mut b = vec![0u8; len];
            r.read_exact(&mut b).unwrap();
            s2.lock().unwrap().push((head, b));
            let resp = format!(
                "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = c.write_all(resp.as_bytes());
        }
    });
    (url, seen)
}

const KEY: &str = "sk-or-test-not-a-real-key-123456";

fn client(url: &str, dir: &Path) -> OpenRouter {
    let mut c = OpenRouter::new(url, KEY.into(), dir.join("ledger.json"));
    c.backoff_ms = 5;
    c
}

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vdn-asr-or-test-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn ok(text: &str, cost: f64) -> (u16, &'static str, String) {
    (
        200,
        "",
        json!({"text": text, "usage": {"seconds": 1.0, "cost": cost}}).to_string(),
    )
}

fn audio() -> Vec<f32> {
    (0..1600).map(|i| (i as f32 * 0.05).sin() * 0.5).collect()
}

/// 反解 base64（只给测试核对请求体用）。
fn unb64(s: &str) -> Vec<u8> {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let v: Vec<u32> = s
        .bytes()
        .filter(|&c| c != b'=')
        .map(|c| T.iter().position(|&t| t == c).unwrap() as u32)
        .collect();
    let mut out = Vec::new();
    for c in v.chunks(4) {
        let n = c
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, x)| n | (x << (18 - 6 * i)));
        for i in 0..c.len().saturating_sub(1) {
            out.push((n >> (16 - 8 * i)) as u8);
        }
    }
    out
}

#[test]
fn base64_matches_rfc4648_vectors() {
    for (i, o) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(b64(i.as_bytes()), o, "{i}");
        assert_eq!(unb64(o), i.as_bytes());
    }
}

#[test]
fn pcm16_wav_header_and_samples() {
    let w = pcm16_wav(&[0.0, 1.0, -1.0, 2.0], 16_000);
    assert_eq!(&w[..4], b"RIFF");
    assert_eq!(&w[8..16], b"WAVEfmt ");
    assert_eq!(u16::from_le_bytes([w[22], w[23]]), 1);
    assert_eq!(u32::from_le_bytes([w[24], w[25], w[26], w[27]]), 16_000);
    assert_eq!(u16::from_le_bytes([w[34], w[35]]), 16);
    assert_eq!(w.len(), 44 + 8);
    let s: Vec<i16> = w[44..]
        .chunks(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();
    assert_eq!(s, vec![0, 32767, -32767, 32767], "超出 [-1, 1] 截断");
}

/// 大总管硬要求：请求里必须带 provider.data_collection = "deny"；没带就红。
#[test]
fn request_carries_model_audio_language_and_deny() {
    let d = tmpdir("req");
    let (url, seen) = fake(vec![ok("hello world", 0.001)]);
    let mut c = client(&url, &d);
    assert_eq!(c.transcribe(&audio(), 16_000, "en").unwrap(), "hello world");
    let reqs = seen.lock().unwrap();
    assert_eq!(reqs.len(), 1);
    let (head, body) = &reqs[0];
    let v: Value = serde_json::from_slice(body).unwrap();
    assert_eq!(
        v["provider"]["data_collection"], "deny",
        "必须带 data_collection = deny"
    );
    assert_eq!(v["model"], OR_MODEL);
    assert_eq!(v["language"], "en");
    assert_eq!(v["input_audio"]["format"], "wav");
    let wav = unb64(v["input_audio"]["data"].as_str().unwrap());
    assert_eq!(wav, pcm16_wav(&audio(), 16_000), "送的是 16 位整型 WAV");
    assert!(head.to_ascii_lowercase().contains(&format!(
        "authorization: bearer {}",
        KEY.to_ascii_lowercase()
    )));
    assert!(
        head.to_ascii_lowercase()
            .contains("content-type: application/json")
    );
    // 函数本身也核一遍
    assert_eq!(
        request_body(b"x", "zh")["provider"]["data_collection"],
        "deny"
    );
}

#[test]
fn cost_is_accumulated_into_ledger_and_survives_restart() {
    let d = tmpdir("ledger");
    let (url, _) = fake(vec![ok("a", 0.25), ok("b", 0.5)]);
    let mut c = client(&url, &d);
    c.transcribe(&audio(), 16_000, "en").unwrap();
    c.transcribe(&audio(), 16_000, "en").unwrap();
    assert!((c.ledger.spent_usd - 0.75).abs() < 1e-12);
    assert_eq!(c.ledger.calls, 2);
    let again = OpenRouter::new("http://127.0.0.1:9/", KEY.into(), d.join("ledger.json"));
    assert_eq!(again.ledger, c.ledger, "账本落盘，重开接着累计");
}

#[test]
fn budget_reached_refuses_before_sending() {
    let d = tmpdir("budget");
    std::fs::write(
        d.join("ledger.json"),
        json!({"spent_usd": BUDGET_USD, "calls": 9}).to_string(),
    )
    .unwrap();
    let (url, seen) = fake(vec![ok("never", 0.0)]);
    let mut c = client(&url, &d);
    let e = c.transcribe(&audio(), 16_000, "en").unwrap_err();
    assert!(e.contains("上限"), "{e}");
    assert_eq!(c.requests, 0);
    assert!(seen.lock().unwrap().is_empty(), "到上限不能再发");
}

#[test]
fn rate_limit_429_backs_off_with_retry_after_then_succeeds() {
    let d = tmpdir("429");
    let (url, seen) = fake(vec![
        (
            429,
            "Retry-After: 1\r\n",
            json!({"error": {"code": 429, "message": "rate limited"}}).to_string(),
        ),
        (
            503,
            "",
            json!({"error": {"code": 503, "message": "busy"}}).to_string(),
        ),
        ok("done", 0.001),
    ]);
    let mut c = client(&url, &d);
    let t0 = std::time::Instant::now();
    assert_eq!(c.transcribe(&audio(), 16_000, "en").unwrap(), "done");
    assert!(
        t0.elapsed() >= std::time::Duration::from_millis(1000),
        "照 Retry-After 等了 1 秒"
    );
    assert_eq!(seen.lock().unwrap().len(), 3);
    // 每次重试的请求体相同：不会改成允许留存
    let reqs = seen.lock().unwrap();
    assert!(reqs.iter().all(|(_, b)| b == &reqs[0].1));
}

#[test]
fn server_errors_give_up_after_max_attempts() {
    let d = tmpdir("5xx");
    let e500 = || {
        (
            500,
            "",
            json!({"error": {"code": 500, "message": "x"}}).to_string(),
        )
    };
    let (url, seen) = fake((0..MAX_ATTEMPTS).map(|_| e500()).collect());
    let mut c = client(&url, &d);
    let e = c.transcribe(&audio(), 16_000, "en").unwrap_err();
    assert!(e.contains(&format!("{MAX_ATTEMPTS} 次")), "{e}");
    assert_eq!(seen.lock().unwrap().len(), MAX_ATTEMPTS);
    assert_eq!(c.ledger.calls, 0, "失败不记账");
}

#[test]
fn credit_and_data_policy_failures_do_not_retry_or_fall_back() {
    for (code, msg, hint) in [
        (402, "Insufficient credits", "余额不足"),
        (
            404,
            "No endpoints found matching your data policy",
            "不留存",
        ),
        (401, "Invalid key", "密钥无效"),
    ] {
        let d = tmpdir(&format!("hard{code}"));
        let (url, seen) = fake(vec![(
            code,
            "",
            json!({"error": {"code": code, "message": msg}}).to_string(),
        )]);
        let mut c = client(&url, &d);
        let e = c.transcribe(&audio(), 16_000, "en").unwrap_err();
        assert!(e.contains(hint) && e.contains("不重试"), "{code}：{e}");
        assert_eq!(seen.lock().unwrap().len(), 1, "{code} 不重试");
    }
}

#[test]
fn errors_never_contain_the_key() {
    let d = tmpdir("scrub");
    let (url, _) = fake(vec![(
        400,
        "",
        format!("{{\"error\":{{\"code\":400,\"message\":\"bad key {KEY}\"}}}}"),
    )]);
    let mut c = client(&url, &d);
    let e = c.transcribe(&audio(), 16_000, "en").unwrap_err();
    assert!(!e.contains(KEY), "出错信息里不许有密钥：{e}");
    assert!(e.contains("<密钥>"));
}

#[test]
fn error_inside_200_is_an_error_and_not_billed() {
    let d = tmpdir("err200");
    let (url, _) = fake(vec![(
        200,
        "",
        json!({"error": {"code": 502, "message": "upstream"}}).to_string(),
    )]);
    let mut c = client(&url, &d);
    assert!(c.transcribe(&audio(), 16_000, "en").is_err());
    assert_eq!(c.ledger.calls, 0);
}

#[test]
fn load_key_from_env_local_variants() {
    let d = tmpdir("key");
    assert_eq!(
        load_key(&d).filter(|_| std::env::var(KEY_VAR).is_err()),
        None
    );
    if std::env::var(KEY_VAR).is_ok() {
        return; // 环境里已有密钥时不测文件那一路（不动真环境）
    }
    for line in [
        "OPENROUTER_API_KEY=abc",
        "export OPENROUTER_API_KEY=\"abc\"",
        "OPENROUTER_API_KEY = 'abc'",
    ] {
        std::fs::write(d.join(".env.local"), format!("OTHER=1\n{line}\n")).unwrap();
        assert_eq!(load_key(&d).as_deref(), Some("abc"), "{line}");
    }
    std::fs::write(d.join(".env.local"), "OPENROUTER_API_KEY=\n").unwrap();
    assert_eq!(load_key(&d), None);
}

/// 真调用：同一条音频连调 3 次，看是否逐字相同（大总管要求，结果写进 exp025 baseline README）。
/// 要密钥、要花钱（约 3 × 1 秒 × 0.0000075 美元），并且外发许可要先定。人工跑：
/// `cargo test -p vdn-eval --lib qa::asr_or::tests::real_call_three_times -- --ignored --nocapture`
#[test]
#[ignore = "要 OPENROUTER_API_KEY、会花钱、外发许可待人定；人工跑"]
fn real_call_three_times() {
    let root = crate::suite::main_root();
    let key = load_key(&root).expect("没有 OPENROUTER_API_KEY");
    let d = root.join("_tmp/exp025-asr-real");
    let mut c = OpenRouter::new(OR_URL, key, d.join("ledger.json"));
    // 合成的哼声（自造，不涉及任何外部素材的许可）
    let x: Vec<f32> = (0..16_000).map(|i| (i as f32 * 0.02).sin() * 0.3).collect();
    let t: Vec<String> = (0..3)
        .map(|_| c.transcribe(&x, 16_000, "en").expect("真调用失败"))
        .collect();
    println!("三次转写：{t:?}；累计 {:.6} 美元", c.ledger.spent_usd);
}
