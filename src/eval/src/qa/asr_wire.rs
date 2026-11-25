//! OpenRouter 识别请求的编码：16 位 WAV、base64、请求体（从 asr_or.rs 拆出，≤600 行）。

use serde_json::{Value, json};

use super::asr_or::OR_MODEL;

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
pub fn request_body(wav: &[u8], lang: &str, temperature: Option<f64>) -> Value {
    let mut v = json!({
        "model": OR_MODEL,
        "input_audio": {"data": b64(wav), "format": "wav"},
        "language": lang,
        "provider": {"data_collection": "deny"},
    });
    // 钉温度探针用（大总管 2026-11-25）：只在设了时带上，不改原来的请求
    if let Some(t) = temperature {
        v["temperature"] = json!(t);
    }
    v
}
