//! 云端语音识别后端：OpenRouter 上的 Qwen3-ASR 1.7B（D103；型号由大总管 2026-11-25 更正为 OpenRouter 实际有的那个）。
//!
//! - 接口：`POST /api/v1/audio/transcriptions`，JSON 体 `{model, input_audio: {data: <base64>, format: "wav"}, language, provider}`
//!   （OpenRouter 文档 STT 一节，2026-11-25 读）。返回 `{text, usage: {seconds, cost}}`
//! - **每个请求都带 `provider.data_collection = "deny"`**（大总管硬要求）：路由不到不留存的提供方就当失败，不改请求重试，
//!   不退回允许留存的提供方
//! - 密钥：环境变量 `OPENROUTER_API_KEY`，没有就读仓库根 `.env.local` 里同名那一行。**绝不打印、不写日志、不进命令行参数**：
//!   经 `curl -K -` 从标准输入传给 curl，出错信息里若含密钥也先抹掉
//! - 费用：发之前在账本里预留估算、返回后按 `usage.cost` 实际金额结算；`已花 + 在途预留 + 这一条` 超过 [`BUDGET_USD`] 就不发
//!   （大总管 2026-11-25：上限 15 美元）。账本跨进程加锁、原子落盘、坏了拒跑（[`super::asr_ledger`]）
//! - 出错：408 · 429 · 5xx 按 `Retry-After`（没有就按指数退避）重试，最多 [`MAX_ATTEMPTS`] 次；400 · 401 · 402 · 403 · 404 不重试
//! - 送的是 16 位整型 WAV（本机 42model 收浮点 WAV 会挂死，云上也用整型，统一）

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

pub use super::asr_bill::booked_cost;

/// 接口地址。
pub const OR_URL: &str = "https://openrouter.ai/api/v1/audio/transcriptions";
/// 型号（正本）：OpenRouter 模型列表上的带日期正式名（canonical_slug，2026-11-25 03:36 UTC 读公开列表）。
pub const OR_MODEL: &str = "qwen/qwen3-asr-1.7b-20260813";
/// 标价（美元 / 秒音频）：模型页 2026-11-25 读，两家提供方（Prime Intellect · DeepInfra）都是 0.000008。
/// 入账取「返回的 usage.cost」与「秒数 × 标价」中大的（[`booked_cost`]），预留也按它估。
pub const PRICE_PER_SECOND: f64 = 0.000_008;

/// 识别费上限（美元，按调用累计）。
pub const BUDGET_USD: f64 = 15.0;
/// 一次转写最多发几次请求（含第一次）。
pub const MAX_ATTEMPTS: usize = 5;
/// 账本文件（相对主目录）：全部真调用共用一个，上限按它累计。
pub const LEDGER_REL: &str = "_tmp/asr-openrouter-ledger.json";
/// 这个错误是不是永久的：再发也一样，要停下（不重试的 4xx、到上限、账本坏或写不了、费用未知）。
pub fn is_permanent(e: &str) -> bool {
    ["不重试", "上限", "账本", "费用未知", "已停发", "待核"]
        .iter()
        .any(|k| e.contains(k))
}

/// 识别调用默认几路并发（大总管 2026-11-25 定 8 路；遇到 429 各自退避）。
pub const ASR_CONCURRENCY: usize = 8;
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

use super::asr_ledger as ledger;
/// 账本（跨进程加锁、原子落盘、先预留后结算），见 [`super::asr_ledger`]。
pub use super::asr_ledger::Ledger;

/// 发请求前按 `音频秒数 × 标价 × 它` 预留：实际金额略高于标价时也不至于越过上限。
pub const RESERVE_FACTOR: f64 = 1.5;

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

/// 一次连接需要的东西（不可变，多个线程共用）。
struct Conn<'a> {
    url: &'a str,
    key: &'a str,
    backoff_ms: u64,
    requests: &'a std::sync::atomic::AtomicU64,
    /// 本批共享的停止状态：每次真发请求前都查（含重试，codex 第 7 轮 P1 #3）
    stop: &'a std::sync::Mutex<Option<String>>,
    /// 共用账本：每次真发请求前看一眼它的待核标记（别的进程可能刚标上；codex 第 11 轮 P1 #1）
    ledger_path: &'a Path,
}

impl Conn<'_> {
    /// 抹掉文字里的密钥（出错信息里万一带上）。
    fn scrub(&self, s: &str) -> String {
        if self.key.is_empty() {
            s.to_string()
        } else {
            s.replace(self.key, "<密钥>")
        }
    }

    /// 发一次 HTTP 请求：返回 (状态码, Retry-After 秒数, 响应体, 响应头)。密钥经标准输入给 curl，不进命令行参数。
    fn post(&self, body_path: &Path) -> Result<Resp, String> {
        let (out, hdr) = (tmp_path("body"), tmp_path("hdr"));
        self.requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut child = Command::new("curl")
            .args([
                "-sS",
                "-m",
                "120",
                "-X",
                "POST",
                self.url,
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
        let headers =
            super::asr_bill::parse_headers(&std::fs::read_to_string(&hdr).unwrap_or_default());
        let retry = headers
            .get("retry-after")
            .and_then(|v| v.as_str()?.parse::<u64>().ok());
        let _ = std::fs::remove_file(&out);
        let _ = std::fs::remove_file(&hdr);
        if code == 0 {
            return Err(self.scrub(&format!(
                "连不上识别服务：{}",
                String::from_utf8_lossy(&o.stderr).trim()
            )));
        }
        // 有状态码但 curl 没正常退出（收到头以后超时、响应体不完整）：不当完整响应，按连不上处理 = 费用未知（codex 本分支第 3 轮 P1 #1）
        if !o.status.success() {
            return Err(self.scrub(&format!(
                "连不上识别服务：HTTP {code} 之后传输中断（curl 退出码 {}）：{}",
                o.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&o.stderr).trim()
            )));
        }
        Ok((code, retry, body, headers))
    }

    /// 记一行逐笔明细；写不了就设本批停止（明细缺了就核不了账），这次响应照常处理。
    fn log_attempt(
        &self,
        attempt: usize,
        ms: u128,
        secs: f64,
        r: &Result<Resp, String>,
        booked: Option<f64>,
    ) -> Option<String> {
        let row = super::asr_bill::detail_row(attempt, ms, secs, r, booked, |e| self.scrub(e));
        // 写不了就持久标待核（明细缺了就核不了账，之后任何进程都拒发；codex 本分支第 1 轮 P1 #1）
        if let Err(e) = ledger::append_detail(self.ledger_path, &row) {
            let why = format!("识别费明细写不了（{e}）");
            let _ = ledger::flag_review(self.ledger_path, &why);
            *self.stop.lock().unwrap_or_else(|e| e.into_inner()) = Some(why.clone());
            return Some(why);
        }
        None
    }

    /// 转写一条：成功返回 (文本, 最后一次响应带的费用（没带为 `None` = 费用未知）, 前几次已记账的费用)；
    /// 失败返回 [`Failure`]。`est` = 这一条在账本里的预留，重试途中累计费用超过它就当场停（codex 第 14 轮）。
    fn transcribe(
        &self,
        x: &[f32],
        sr: u32,
        lang: &str,
        est: f64,
    ) -> Result<(String, Option<f64>, f64), Failure> {
        let free = |msg: String| Failure {
            msg,
            cost: Cost::Free,
            carried: 0.0,
        };
        let body = serde_json::to_vec(&request_body(&pcm16_wav(x, sr), lang))
            .map_err(|e| free(e.to_string()))?;
        let body_path = tmp_path("json");
        std::fs::write(&body_path, &body).map_err(|e| free(e.to_string()))?;
        let secs = x.len() as f64 / f64::from(sr);
        let r = self.send_with_retry(&body_path, est, secs);
        let _ = std::fs::remove_file(&body_path);
        r
    }

    fn send_with_retry(
        &self,
        body_path: &Path,
        est: f64,
        secs: f64,
    ) -> Result<(String, Option<f64>, f64), Failure> {
        let mut last = String::new();
        // 前几次（非 200）响应带回的费用：收到就当场记账，之后的出口不再管它（codex 第 13 轮 P1 #1 · 第 14 轮）
        let mut carried = 0.0f64;
        let fail = |msg: String, cost: Cost, carried: f64| Failure {
            msg: self.scrub(&msg),
            cost,
            carried,
        };
        for attempt in 0..MAX_ATTEMPTS {
            match ledger::read(self.ledger_path) {
                Ok(l) if l.review.is_some() => {
                    let why = format!("识别费账本待核（{}）：停发", l.review.unwrap_or_default());
                    return Err(fail(why, Cost::Free, carried));
                }
                Ok(_) => {}
                Err(e) => return Err(fail(e, Cost::Free, carried)),
            }
            if let Some(why) = self.stop.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                return Err(fail(format!("已停发：{why}"), Cost::Free, carried));
            }
            // 连不上、超时：请求可能已送到、可能已计费，不知道 → 费用未知，停发（偏保守）
            let t0 = std::time::Instant::now();
            let sent = self.post(body_path);
            // 入账金额：200 且带 usage.cost → 与标价估算取大（秒数取音频长与返回秒数中大的）；非 200 照返回的记
            let booked = sent.as_ref().ok().and_then(|(code, _, body, _)| {
                let v: Value = serde_json::from_slice(body).ok()?;
                let c = v["usage"]["cost"].as_f64()?;
                // 标价估算只按实际音频秒数；返回的 seconds 只进明细（codex 本分支第 1 轮 P1 #2）
                Some(if *code == 200 && v.get("error").is_none() {
                    booked_cost(c, secs)
                } else {
                    c
                })
            });
            let detail_err =
                self.log_attempt(attempt, t0.elapsed().as_millis(), secs, &sent, booked);
            let (code, retry_after, body, _) =
                sent.map_err(|e| fail(format!("费用未知，停发：{e}"), Cost::Unknown, carried))?;
            if code == 200 {
                let Ok(v) = serde_json::from_slice::<Value>(&body) else {
                    return Err(fail(
                        "费用未知，停发：识别返回 200 但不是 JSON".into(),
                        Cost::Unknown,
                        carried,
                    ));
                };
                let cost = booked;
                let tag = if cost.is_some() {
                    ""
                } else {
                    "费用未知，停发："
                };
                let c = cost.map_or(Cost::Unknown, Cost::Known);
                if let Some(e) = v.get("error") {
                    // 内嵌错误码按 HTTP 状态码同一套分类：不该重试的（4xx 里除了 408 · 429）当永久错误，整套停发（codex 第 15 轮）
                    let inner = e["code"].as_u64().and_then(|x| u16::try_from(x).ok());
                    let perm = inner.is_some_and(|x| (400..500).contains(&x) && !retryable(x));
                    let kind = if perm { "，不重试" } else { "" };
                    return Err(fail(
                        format!("{tag}识别返回 200 但带错误{kind}：{e}"),
                        c,
                        carried,
                    ));
                }
                let Some(t) = v["text"].as_str() else {
                    return Err(fail(format!("{tag}识别返回缺 text：{v}"), c, carried));
                };
                // 明细写不了：这一次也报错（费用照这次的结算），账本已标待核（codex 本分支第 2 轮 P1 #1）
                if let Some(e) = detail_err {
                    return Err(fail(format!("{e}：账本已标待核，停发"), c, carried));
                }
                // 文本到手但返回里没有 usage.cost：文本照用，费用未知（调用方标「待核」、停发；codex 第 7 轮 P1 #1）
                return Ok((
                    t.split_whitespace().collect::<Vec<_>>().join(" "),
                    cost,
                    carried,
                ));
            }
            if let Some(c) = serde_json::from_slice::<Value>(&body)
                .ok()
                .and_then(|v| v["usage"]["cost"].as_f64())
            {
                carried += c;
                ledger::update(self.ledger_path, |l| {
                    l.spent_usd += c;
                    l.calls += 1;
                    Ok(())
                })
                .map_err(|e| fail(e, Cost::Free, carried))?;
                if carried > est {
                    let why = format!(
                        "实际费用 {carried:.6} 美元超过预留 {est:.6} 美元：估算不准（重试途中）"
                    );
                    ledger::flag_review(self.ledger_path, &why)
                        .map_err(|e| fail(e, Cost::Free, carried))?;
                    return Err(fail(format!("已停发：{why}"), Cost::Free, carried));
                }
            }
            // 先对完整响应抹密钥再截断：密钥跨过截断点时也不会漏出半截（codex 第 8 轮 P2 #1）
            let clean = self.scrub(&String::from_utf8_lossy(&body));
            last = format!(
                "HTTP {code}：{}",
                clean.chars().take(300).collect::<String>()
            );
            if !retryable(code) {
                let hint = match code {
                    402 => "（余额不足）",
                    401 => "（密钥无效）",
                    403 | 404 => "（多半是没有不留存数据的提供方肯接：按要求不退回，停下）",
                    _ => "",
                };
                return Err(fail(
                    format!("识别请求失败，不重试{hint}：{last}"),
                    Cost::Free,
                    carried,
                ));
            }
            if attempt + 1 < MAX_ATTEMPTS {
                let wait_ms = retry_after.map_or(self.backoff_ms << attempt, |s| s.min(60) * 1000);
                std::thread::sleep(std::time::Duration::from_millis(wait_ms));
            }
        }
        Err(fail(
            format!("识别请求重试 {MAX_ATTEMPTS} 次仍失败：{last}"),
            Cost::Free,
            carried,
        ))
    }
}

/// 一次 HTTP 响应：(状态码, Retry-After 秒数, 响应体, 响应头)。
pub(super) type Resp = (u16, Option<u64>, Vec<u8>, serde_json::Map<String, Value>);

/// 结算最后一次响应带的已知金额（成功、失败两条路共用；codex 第 9、10、14 轮）：照记；这条转写的总花费
/// （前几次已记的 `carried` + 这次的 `c`）超过预留就给账本标待核、设本批停止状态，之后任何进程都拒发。
fn settle_known(
    ledger_path: &Path,
    est: f64,
    c: f64,
    carried: f64,
    stop: &std::sync::Mutex<Option<String>>,
) -> Result<(), String> {
    if c + carried > est {
        let why = format!(
            "实际费用 {:.6} 美元超过预留 {est:.6} 美元：估算不准",
            c + carried
        );
        ledger::flag_review(ledger_path, &why)?;
        let mut st = stop.lock().unwrap_or_else(|e| e.into_inner());
        if st.is_none() {
            *st = Some(why);
        }
    }
    ledger::settle(ledger_path, est, Some(c))
}

/// 一次请求失败时最后那次响应的费用情况（codex 第 6 轮 P1 #2：费用结算和转写成败分开）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cost {
    /// HTTP 错误码：没有计费，退预留
    Free,
    /// 返回里有 `usage.cost`：照记这笔钱
    Known(f64),
    /// 不知道算没算钱（连不上、超时、返回读不出）：预留不退，停发等人核对
    Unknown,
}

/// 一次失败：原因 + 最后那次的费用情况 + 前几次已记账的费用。
#[derive(Clone, Debug)]
pub struct Failure {
    pub msg: String,
    pub cost: Cost,
    pub carried: f64,
}

/// 一条待转写的音频：(16 kHz 样本, 采样率, 语言)。
pub type Job = (Vec<f32>, u32, String);
/// 并发时每条的结果槽：(文本, 实际花费) 或出错原因。
type Slot = std::sync::Mutex<Option<Result<String, String>>>;

impl OpenRouter {
    /// 新客户端。账本快照只给人看；真正的上限判断每次都在锁里重读账本（坏账本那时就拒发）。
    pub fn new(url: impl Into<String>, key: String, ledger_path: PathBuf) -> OpenRouter {
        let ledger = ledger::read(&ledger_path).unwrap_or_default();
        OpenRouter {
            url: url.into(),
            key,
            ledger_path,
            ledger,
            backoff_ms: 1000,
            requests: 0,
        }
    }

    /// 转写一条音频（16 kHz 浮点样本）。成功后按 `usage.cost` 记账；到上限、出错都返回 `Err`。
    pub fn transcribe(&mut self, x: &[f32], sr: u32, lang: &str) -> Result<String, String> {
        self.transcribe_many(&[(x.to_vec(), sr, lang.to_string())], 1)
            .remove(0)
    }

    /// 并发转写一批（最多 `workers` 路同时发；429 / 5xx 各自退避）。结果按输入顺序返回，和完成先后无关。
    /// 每条发之前在账本里预留估算金额（放不下上限就不发、返回 `Err`），返回后按实际金额结算（codex 第 5 轮 P1 #2）。
    pub fn transcribe_many(&mut self, jobs: &[Job], workers: usize) -> Vec<Result<String, String>> {
        let requests = std::sync::atomic::AtomicU64::new(0);
        let next = std::sync::atomic::AtomicUsize::new(0);
        // 遇到永久错误后，本批里还没派出去的、正在退避的都不再发（codex 第 6 轮 P1 #1 · 第 7 轮 P1 #3）
        let stop: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
        let conn = Conn {
            url: &self.url,
            key: &self.key,
            backoff_ms: self.backoff_ms,
            requests: &requests,
            stop: &stop,
            ledger_path: &self.ledger_path,
        };
        let ledger_path = &self.ledger_path;
        let slots: Vec<Slot> = jobs.iter().map(|_| std::sync::Mutex::new(None)).collect();
        std::thread::scope(|sc| {
            for _ in 0..workers.clamp(1, jobs.len().max(1)) {
                sc.spawn(|| {
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some((x, sr, lang)) = jobs.get(i) else {
                            break;
                        };
                        if let Some(why) = stop.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                            *slots[i].lock().unwrap_or_else(|e| e.into_inner()) =
                                Some(Err(format!("已停发：{why}")));
                            continue;
                        }
                        let est =
                            x.len() as f64 / f64::from(*sr) * PRICE_PER_SECOND * RESERVE_FACTOR;
                        // 结算失败（写不了账本）也当这一条失败：不能确认记上了账
                        let r = ledger::reserve(ledger_path, est, BUDGET_USD).and_then(|()| {
                            match conn.transcribe(x, *sr, lang, est) {
                                Ok((t, Some(c), carried)) => {
                                    settle_known(ledger_path, est, c, carried, &stop).map(|()| t)
                                }
                                Ok((t, None, _)) => {
                                    // 文本照用；预留不退、账本标待核，本批与以后都停发
                                    let why = "费用未知：返回有文本但没带 usage.cost";
                                    ledger::flag_review(ledger_path, why)?;
                                    let mut st = stop.lock().unwrap_or_else(|e| e.into_inner());
                                    if st.is_none() {
                                        *st = Some(why.to_string());
                                    }
                                    Ok(t)
                                }
                                Err(f) => {
                                    match f.cost {
                                        Cost::Free => ledger::settle(ledger_path, est, None)?,
                                        Cost::Known(c) => {
                                            settle_known(ledger_path, est, c, f.carried, &stop)?
                                        }
                                        // 预留不退：可能已计费；账本标待核，重启也停发（前几次的费用已当场记账，不会丢）
                                        Cost::Unknown => ledger::flag_review(ledger_path, &f.msg)?,
                                    }
                                    Err(f.msg)
                                }
                            }
                        });
                        if let Err(e) = &r
                            && is_permanent(e)
                        {
                            let mut st = stop.lock().unwrap_or_else(|e| e.into_inner());
                            if st.is_none() {
                                *st = Some(e.clone());
                            }
                        }
                        *slots[i].lock().unwrap_or_else(|e| e.into_inner()) = Some(r);
                    }
                });
            }
        });
        self.requests += requests.into_inner();
        self.ledger = ledger::read(&self.ledger_path).unwrap_or_default();
        slots
            .into_iter()
            .map(|m| {
                m.into_inner()
                    .unwrap_or_else(|e| e.into_inner())
                    .unwrap_or(Err("内部错误：没跑到".into()))
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "asr_or_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "asr_or_cost_tests.rs"]
mod cost_tests;

#[cfg(test)]
#[path = "asr_or_bill_tests.rs"]
mod bill_tests;
