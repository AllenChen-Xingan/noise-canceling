//! WAV 读写与文件哈希（离线工具，C 档）。

use std::path::Path;

/// 读单声道 WAV，转成 f32（整型按满幅归一）。返回 (样本, 采样率)。多声道取第一声道。
pub fn read(path: &Path) -> Result<(Vec<f32>, u32), String> {
    let mut r = hound::WavReader::open(path).map_err(|e| format!("读 {}：{e}", path.display()))?;
    let spec = r.spec();
    let ch = spec.channels.max(1) as usize;
    let all: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r
            .samples::<f32>()
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?,
        hound::SampleFormat::Int => {
            let scale = 1.0 / (1u64 << (spec.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|s| s.map(|v| v as f32 * scale))
                .collect::<Result<_, _>>()
                .map_err(|e| e.to_string())?
        }
    };
    Ok((all.into_iter().step_by(ch).collect(), spec.sample_rate))
}

/// 写 32-bit float 单声道 WAV。
pub fn write(path: &Path, x: &[f32], sr: u32) -> Result<(), String> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sr,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec).map_err(|e| e.to_string())?;
    for &v in x {
        w.write_sample(v).map_err(|e| e.to_string())?;
    }
    w.finalize().map_err(|e| e.to_string())
}

/// 文件 sha256（调系统 `shasum -a 256`，macOS 自带；CI 层不用它）。
pub fn sha256(path: &Path) -> Result<String, String> {
    let o = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .map_err(|e| format!("调不起 shasum：{e}"))?;
    if !o.status.success() {
        return Err(format!("shasum 失败：{}", path.display()));
    }
    String::from_utf8_lossy(&o.stdout)
        .split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| "shasum 没有输出".into())
}
