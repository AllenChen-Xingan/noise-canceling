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

/// 假密钥：故意不像任何真密钥的样子（不以 `sk-` 开头），免得被外发前的密钥扫描当真密钥拦下，
/// 也免得有人误以为仓里有真密钥。真调用的那一条测试从 `.env.local` 读，不在这里。
/// 测试只核「请求头里带了它」和「出错信息里抹掉了它」。
const KEY: &str = "placeholder-test-credential-0001";

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
    let w = pcm16_wav(&[0.0, 1.0, -1.0, 2.0], vdn_types::MODEL_RATE);
    assert_eq!(&w[..4], b"RIFF");
    assert_eq!(&w[8..16], b"WAVEfmt ");
    assert_eq!(u16::from_le_bytes([w[22], w[23]]), 1);
    assert_eq!(
        u32::from_le_bytes([w[24], w[25], w[26], w[27]]),
        vdn_types::MODEL_RATE
    );
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
    let (url, seen) = fake(vec![ok("hello world", 1e-7)]);
    let mut c = client(&url, &d);
    assert_eq!(
        c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap(),
        "hello world"
    );
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
    assert_eq!(
        wav,
        pcm16_wav(&audio(), vdn_types::MODEL_RATE),
        "送的是 16 位整型 WAV"
    );
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
    // 费用要低于预留（0.1 秒音频 × 标价 × 1.5 ≈ 1.1e-6），否则会被当成「超过预留」停发
    let (url, _) = fake(vec![ok("a", 2.5e-7), ok("b", 5e-7)]);
    let mut c = client(&url, &d);
    c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap();
    c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap();
    assert!((c.ledger.spent_usd - 7.5e-7).abs() < 1e-15);
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
    let e = c
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
    assert!(e.contains("上限"), "{e}");
    assert_eq!(c.requests, 0);
    assert!(seen.lock().unwrap().is_empty(), "到上限不能再发");
}

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
        ok("done", 1e-7),
    ]);
    let mut c = client(&url, &d);
    let t0 = std::time::Instant::now();
    assert_eq!(
        c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").unwrap(),
        "done"
    );
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
    let e = c
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
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
        let e = c
            .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
            .unwrap_err();
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
    let e = c
        .transcribe(&audio(), vdn_types::MODEL_RATE, "en")
        .unwrap_err();
    assert!(!e.contains(KEY), "出错信息里不许有密钥：{e}");
    assert!(e.contains("<密钥>"));
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

#[test]
fn error_inside_200_is_an_error_and_not_billed() {
    let d = tmpdir("err200");
    let (url, _) = fake(vec![(
        200,
        "",
        json!({"error": {"code": 502, "message": "upstream"}}).to_string(),
    )]);
    let mut c = client(&url, &d);
    assert!(c.transcribe(&audio(), vdn_types::MODEL_RATE, "en").is_err());
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
/// 要密钥、会花钱，外发按 D104。音频用 `VDN_ASR_REAL_WAV` 指定（16 kHz 单声道 wav）。人工跑：
/// `VDN_ASR_REAL_WAV=<wav> cargo test -p vdn-eval --lib qa::asr_or::tests::real_call_three_times -- --ignored --nocapture`
#[test]
#[ignore = "要 OPENROUTER_API_KEY、会花钱；人工跑"]
fn real_call_three_times() {
    let root = crate::suite::main_root();
    let key = load_key(&root).expect("没有 OPENROUTER_API_KEY");
    let wav = std::env::var("VDN_ASR_REAL_WAV").expect("用 VDN_ASR_REAL_WAV 指定一条 wav");
    let (x, sr) = crate::wav::read(Path::new(&wav)).unwrap();
    let mut c = OpenRouter::new(OR_URL, key, root.join(LEDGER_REL));
    let mut texts = Vec::new();
    for i in 1..=3 {
        let before = c.ledger.spent_usd;
        let t0 = std::time::Instant::now();
        let t = c
            .transcribe(&x, sr, "en")
            .unwrap_or_else(|e| panic!("第 {i} 次真调用失败：{e}"));
        println!(
            "第 {i} 次：{:.2} 秒，花费 {:.8} 美元，请求 {} 个，文本：{t}",
            t0.elapsed().as_secs_f64(),
            c.ledger.spent_usd - before,
            c.requests
        );
        texts.push(t);
    }
    println!(
        "三次逐字相同：{}；账本累计 {:.8} 美元 / {} 次",
        texts.iter().all(|t| t == &texts[0]),
        c.ledger.spent_usd,
        c.ledger.calls
    );
}
