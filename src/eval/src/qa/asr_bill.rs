//! 识别费入账规则与逐笔明细的行格式（从 asr_or.rs 拆出，≤600 行；大总管 2026-11-25）。

use serde_json::{Value, json};

use super::asr_ledger as ledger;
use super::asr_or::{PRICE_PER_SECOND, Resp};

/// 一次成功转写入账多少：返回的 `usage.cost` 与「音频秒数 × 标价」取大（大总管 2026-11-25：返回报的费用不一定是实扣）。
pub fn booked_cost(reported: f64, secs: f64) -> f64 {
    reported.max(secs * PRICE_PER_SECOND)
}

/// 读 curl `-D` 写下的响应头：只取最后一段（前面可能有 100 Continue），名字转小写；set-cookie 不留。
pub(super) fn parse_headers(text: &str) -> serde_json::Map<String, Value> {
    let last = text.rsplit("HTTP/").next().unwrap_or("");
    last.lines()
        .skip(1)
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), Value::from(v.trim())))
        .filter(|(k, _)| k != "set-cookie" && k != "authorization")
        .collect()
}

/// 一次 HTTP 尝试的明细行：时间 · 第几次 · 耗时 · 音频秒数 · 状态码 · 生成 id · usage 全文 · 入账金额 · 响应头；
/// 连不上时状态码 0 并带错误（`scrub` 抹掉密钥）。不含转写原文。
pub(super) fn detail_row(
    attempt: usize,
    ms: u128,
    secs: f64,
    r: &Result<Resp, String>,
    booked: Option<f64>,
    scrub: impl Fn(&str) -> String,
) -> Value {
    let at = ledger::utc_iso(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    );
    match r {
        Err(e) => json!({"at": at, "attempt": attempt + 1, "ms": ms, "audio_s": secs, "status": 0,
            "error": scrub(e)}),
        Ok((code, _, body, h)) => {
            let v: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
            let id = v["id"]
                .as_str()
                .or_else(|| h.get("x-generation-id").and_then(Value::as_str));
            json!({"at": at, "attempt": attempt + 1, "ms": ms, "audio_s": secs, "status": code,
                "id": id, "usage": v["usage"], "booked_usd": booked, "headers": h})
        }
    }
}
