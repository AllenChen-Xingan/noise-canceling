//! 云端语音识别后端：OpenRouter 上的 Qwen3-ASR 1.7B（D103；型号由大总管 2026-11-25 更正为 OpenRouter 实际有的那个）。
//!
//! - 接口：`POST /api/v1/audio/transcriptions`，JSON 体 `{model, input_audio: {data: <base64>, format: "wav"}, language, provider}`
//!   （OpenRouter 文档 STT 一节，2026-11-25 读）。返回 `{text, usage: {seconds, cost}}`
//! - **每个请求都带 `provider.data_collection = "deny"`**（大总管硬要求）：路由不到不留存的提供方就当失败，不改请求重试，
//!   不退回允许留存的提供方
//! - 密钥：环境变量 `OPENROUTER_API_KEY`，没有就读仓库根 `.env.local` 里同名那一行。**绝不打印、不写日志、不进命令行参数**：
//!   经 `curl -K -` 从标准输入传给 curl，出错信息里若含密钥也先抹掉
//! - 费用：按每次返回的 `usage.cost` 累计进账本文件，到 [`BUDGET_USD`] 就拒绝再发（大总管 2026-11-25：上限 15 美元）
//! - 出错：408 · 429 · 5xx 按 `Retry-After`（没有就按指数退避）重试，最多 [`MAX_ATTEMPTS`] 次；400 · 401 · 402 · 403 · 404 不重试
//! - 送的是 16 位整型 WAV（本机 42model 收浮点 WAV 会挂死，云上也用整型，统一）

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

/// 接口地址。
pub const OR_URL: &str = "https://openrouter.ai/api/v1/audio/transcriptions";
/// 型号（正本）：OpenRouter 模型列表上的带日期正式名（canonical_slug，2026-11-25 03:36 UTC 读公开列表）。
pub const OR_MODEL: &str = "qwen/qwen3-asr-1.7b-20260813";
/// 列表上的标价（美元 / 秒音频）。只在返回里没有 `usage.cost` 时用来估算，正常按实际金额记账。
pub const PRICE_PER_SECOND: f64 = 0.000_007_5;
/// 识别费上限（美元，按调用累计）。
pub const BUDGET_USD: f64 = 15.0;
/// 一次转写最多发几次请求（含第一次）。
pub const MAX_ATTEMPTS: usize = 5;
/// 密钥的环境变量名。
pub const KEY_VAR: &str = "OPENROUTER_API_KEY";

/// 读密钥：环境变量优先，其次 `<root>/.env.local` 里 `OPENROUTER_API_KEY=…` 那一行（可带引号）。都没有返回 `None`。
pub fn load_key(root: &Path) -> Option<String> {
    if let Ok(k) = std::env::var(KEY_VAR)
        && !k.trim().is_empty()
    {
        return Some(k.trim().to_string());
    }
    let text = std::fs::read_to_string(root.join(".env.local")).ok()?;
    text.lines().find_map(|l| {
        let l = l.trim().strip_prefix("export ").unwrap_or(l.trim());
        let v = l
            .strip_prefix(KEY_VAR)?
            .trim_start()
            .strip_prefix('=')?
            .trim();
        let v = v.trim_matches(|c| c == '"' || c == '\'');
        (!v.is_empty()).then(|| v.to_string())
    })
}

/// 16 位整型单声道 WAV 的完整字节（含 44 字节文件头）。超出 [-1, 1] 的样本截断。
pub fn pcm16_wav(x: &[f32], sr: u32) -> Vec<u8> {
    let data_len = (x.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + x.len() * 2);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // 单声道
    b.extend_from_slice(&sr.to_le_bytes());
    b.extend_from_slice(&(sr * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for v in x {
        let s = (v.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        b.extend_from_slice(&s.to_le_bytes());
    }
    b
}

/// 标准 base64（RFC 4648，带补位）。
pub fn b64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        for (i, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            if i <= c.len() {
                s.push(T[((n >> shift) & 63) as usize] as char);
            } else {
                s.push('=');
            }
        }
    }
    s
}

/// 请求体。`provider.data_collection = "deny"` 一律带上（大总管硬要求，测试核）。
pub fn request_body(wav: &[u8], lang: &str) -> Value {
    json!({
        "model": OR_MODEL,
        "input_audio": {"data": b64(wav), "format": "wav"},
        "language": lang,
        "provider": {"data_collection": "deny"},
    })
}

/// 账本：累计花费与调用次数（JSON 文件，按调用即时写）。
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ledger {
    pub spent_usd: f64,
    pub calls: u64,
}

/// 一个 OpenRouter 客户端。`url` 可换成本机假服务（测试用）。
pub struct OpenRouter {
    url: String,
    key: String,
    ledger_path: PathBuf,
    pub ledger: Ledger,
    /// 没有 `Retry-After` 时第一次退避的毫秒数（之后翻倍）
    pub backoff_ms: u64,
    /// 本进程发出的 HTTP 请求数（含重试）
    pub requests: u64,
}

/// 临时文件名的进程内唯一序号：同一进程里可能有多个客户端（测试并行、按段分后端），各自从 0 数会撞名。
fn tmp_path(ext: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("vdn-asr-or-{}-{n}.{ext}", std::process::id()))
}

fn retryable(code: u16) -> bool {
    matches!(code, 408 | 429 | 500 | 502 | 503 | 504)
}

impl OpenRouter {
    pub fn new(url: impl Into<String>, key: String, ledger_path: PathBuf) -> OpenRouter {
        let ledger = std::fs::read_to_string(&ledger_path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        OpenRouter {
            url: url.into(),
            key,
            ledger_path,
            ledger,
            backoff_ms: 1000,
            requests: 0,
        }
    }

    /// 抹掉文字里的密钥（出错信息里万一带上）。
    fn scrub(&self, s: &str) -> String {
        if self.key.is_empty() {
            s.to_string()
        } else {
            s.replace(&self.key, "<密钥>")
        }
    }

    /// 发一次 HTTP 请求：返回 (状态码, Retry-After 秒数, 响应体)。密钥经标准输入给 curl。
    fn post(&mut self, body_path: &Path) -> Result<(u16, Option<u64>, Vec<u8>), String> {
        let (out, hdr) = (tmp_path("body"), tmp_path("hdr"));
        self.requests += 1;
        let mut child = Command::new("curl")
            .args([
                "-sS",
                "-m",
                "120",
                "-X",
                "POST",
                &self.url,
                "-H",
                "Content-Type: application/json",
                "--data-binary",
            ])
            .arg(format!("@{}", body_path.display()))
            .arg("-o")
            .arg(&out)
            .arg("-D")
            .arg(&hdr)
            .args(["-w", "%{http_code}", "-K", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("调不起 curl：{e}"))?;
        {
            use std::io::Write;
            let mut stdin = child.stdin.take().ok_or("curl 没有标准输入")?;
            writeln!(stdin, "header = \"Authorization: Bearer {}\"", self.key)
                .map_err(|e| e.to_string())?;
        }
        let o = child.wait_with_output().map_err(|e| e.to_string())?;
        let code: u16 = String::from_utf8_lossy(&o.stdout)
            .trim()
            .parse()
            .unwrap_or(0);
        let body = std::fs::read(&out).unwrap_or_default();
        let retry = std::fs::read_to_string(&hdr).ok().and_then(|h| {
            h.lines().find_map(|l| {
                let (k, v) = l.split_once(':')?;
                k.trim()
                    .eq_ignore_ascii_case("retry-after")
                    .then(|| v.trim().parse::<u64>().ok())?
            })
        });
        let _ = std::fs::remove_file(&out);
        let _ = std::fs::remove_file(&hdr);
        if code == 0 {
            return Err(self.scrub(&format!(
                "连不上识别服务：{}",
                String::from_utf8_lossy(&o.stderr).trim()
            )));
        }
        Ok((code, retry, body))
    }

    /// 转写一条音频（16 kHz 浮点样本）。成功后按 `usage.cost` 记账；到上限、出错都返回 `Err`。
    pub fn transcribe(&mut self, x: &[f32], sr: u32, lang: &str) -> Result<String, String> {
        if self.ledger.spent_usd >= BUDGET_USD {
            return Err(format!(
                "识别费已累计 {:.4} 美元，到上限 {BUDGET_USD} 美元，停发",
                self.ledger.spent_usd
            ));
        }
        let body = serde_json::to_vec(&request_body(&pcm16_wav(x, sr), lang))
            .map_err(|e| e.to_string())?;
        let body_path = tmp_path("json");
        std::fs::write(&body_path, &body).map_err(|e| e.to_string())?;
        let r = self.send_with_retry(&body_path, x.len() as f64 / f64::from(sr));
        let _ = std::fs::remove_file(&body_path);
        r
    }

    fn send_with_retry(&mut self, body_path: &Path, seconds: f64) -> Result<String, String> {
        let mut last = String::new();
        for attempt in 0..MAX_ATTEMPTS {
            let (code, retry_after, body) = self.post(body_path)?;
            let text = String::from_utf8_lossy(&body).to_string();
            if code == 200 {
                let v: Value =
                    serde_json::from_slice(&body).map_err(|e| format!("识别返回不是 JSON：{e}"))?;
                if let Some(e) = v.get("error") {
                    return Err(self.scrub(&format!("识别返回 200 但带错误：{e}")));
                }
                let t = v["text"]
                    .as_str()
                    .ok_or_else(|| self.scrub(&format!("识别返回缺 text：{v}")))?;
                let cost = v["usage"]["cost"]
                    .as_f64()
                    .unwrap_or(seconds * PRICE_PER_SECOND);
                self.ledger.spent_usd += cost;
                self.ledger.calls += 1;
                if let Some(d) = self.ledger_path.parent() {
                    let _ = std::fs::create_dir_all(d);
                }
                let _ = std::fs::write(
                    &self.ledger_path,
                    serde_json::to_string_pretty(&self.ledger).unwrap_or_default(),
                );
                return Ok(t.split_whitespace().collect::<Vec<_>>().join(" "));
            }
            last = self.scrub(&format!(
                "HTTP {code}：{}",
                text.chars().take(300).collect::<String>()
            ));
            if !retryable(code) {
                let hint = match code {
                    402 => "（余额不足）",
                    401 => "（密钥无效）",
                    403 | 404 => "（多半是没有不留存数据的提供方肯接：按要求不退回，停下）",
                    _ => "",
                };
                return Err(format!("识别请求失败，不重试{hint}：{last}"));
            }
            if attempt + 1 < MAX_ATTEMPTS {
                let wait_ms = retry_after.map_or(self.backoff_ms << attempt, |s| s.min(60) * 1000);
                std::thread::sleep(std::time::Duration::from_millis(wait_ms));
            }
        }
        Err(format!("识别请求重试 {MAX_ATTEMPTS} 次仍失败：{last}"))
    }
}

#[cfg(test)]
#[path = "asr_or_tests.rs"]
mod tests;
