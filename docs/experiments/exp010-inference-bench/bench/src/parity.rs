//! 逐帧对拍（BE-PARITY）、复位检查、给非 Rust 测量程序导出输入与 ORT 输出。

use crate::*;

/// 逐帧对拍：参考 `a`（ORT）对 `b`，同一份输入，比整帧输出。
/// 逐帧对拍：参考 `a`（ORT）对 `b`，同一份输入，比整帧输出。
/// `first_bad`：第一个「差 > 1e-3 或任一边非有限」的位置（NaN 也算坏，审稿 P2-1）。
/// `min_frame_snr_db`：逐帧 SNR 的最小值，只统计参考能量 ≥ 全段平均帧能量 1e-4 的帧，专看安静段（整段 SNR 被响帧主导）。
/// `reset_at`：若给出，在第 k 帧前对 `b` 调 reset，再拿一个新开的 `b2` 从第 k 帧起同喂，报两者最大差（应为 0）。
pub(crate) fn parity(
    model: ModelId,
    vs: &str,
    mut a: Box<dyn Net>,
    mut b: Box<dyn Net>,
    inputs: &[Vec<f32>],
) -> Result<(), Err> {
    if inputs.is_empty() {
        return Err("对拍输入为空（音频短于一个 hop，或 --frames 0）".into());
    }
    let (mut max_abs, mut sig, mut err) = (0.0f64, 0.0f64, 0.0f64);
    let mut worst_frame = 0;
    let mut first_bad = None;
    let mut nonfinite = 0usize;
    let mut per_frame = Vec::with_capacity(inputs.len()); // (ref 能量, 差能量)
    let mut xa = inputs[0].clone();
    let mut xb = inputs[0].clone();
    for (f, x) in inputs.iter().enumerate() {
        xa.copy_from_slice(x);
        xb.copy_from_slice(x);
        a.infer(&mut xa)?;
        b.infer(&mut xb)?;
        let (mut fs, mut fe) = (0.0f64, 0.0f64);
        for (k, (&p, &q)) in xa.iter().zip(&xb).enumerate() {
            let bad_val = !p.is_finite() || !q.is_finite();
            if bad_val {
                nonfinite += 1;
            }
            let d = (p as f64 - q as f64).abs();
            fs += (p as f64).powi(2);
            fe += d * d;
            if d > max_abs {
                max_abs = d;
                worst_frame = f;
            }
            if (bad_val || !(d <= 1e-3)) && first_bad.is_none() {
                first_bad = Some((f, k));
            }
        }
        sig += fs;
        err += fe;
        per_frame.push((fs, fe));
    }
    let mean_e = sig / inputs.len() as f64;
    let min_frame_snr = per_frame
        .iter()
        .filter(|(s, _)| *s >= 1e-4 * mean_e)
        .map(|(s, e)| 10.0 * (s / e.max(1e-300)).log10())
        .fold(f64::INFINITY, f64::min);
    // 参考全零（全静音）时 SNR 没有定义：两边都为零就算通过，SNR 写 null（codex 第 4 轮 #1）
    let zero_ref = sig == 0.0;
    let snr = if zero_ref {
        f64::NAN
    } else {
        10.0 * (sig / err.max(1e-300)).log10()
    };
    // BE-PARITY（standard.md §2）：没有值差 > 1e-3、没有非有限值，且整段 SNR ≥ 60 dB（参考全零时改判「差全为 0」）
    let pass = first_bad.is_none()
        && nonfinite == 0
        && if zero_ref {
            err == 0.0
        } else {
            snr.is_finite() && snr >= 60.0
        };
    println!(
        "{{\"model\":\"{}\",\"vs\":\"{vs}\",\"frames\":{},\"max_abs\":{},\"worst_frame\":{worst_frame},\"snr_db\":{},\"min_frame_snr_db\":{},\"nonfinite_values\":{nonfinite},\"first_bad\":\"{:?}\",\"zero_reference\":{zero_ref},\"be_parity_pass\":{pass}}}",
        model.name(),
        inputs.len(),
        if max_abs.is_finite() {
            format!("{max_abs:.3e}")
        } else {
            "null".into()
        },
        jnum(snr, 2),
        jnum(min_frame_snr, 2),
        first_bad
    );
    // 不过就返回非零退出码，脚本里后续测速不会继续（codex 第 1 轮 #3）；已知错误的对照用 --allow-fail
    if !pass && !ALLOW_FAIL.load(Relaxed) {
        return Err(format!("{vs} 没过 BE-PARITY").into());
    }
    Ok(())
}
pub(crate) static ALLOW_FAIL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// reset 检查：`b` 先跑 `k` 帧再 reset，`b2` 新开；两者从第 k 帧起喂同样输入，比输出（应逐位相同）。
pub(crate) fn reset_check(
    model: ModelId,
    vs: &str,
    b: &mut dyn Net,
    b2: &mut dyn Net,
    inputs: &[Vec<f32>],
    k: usize,
) -> Result<(), Err> {
    // 先查区间，再切片（codex 第 3 轮 #3）
    if inputs.is_empty() || k == 0 || k >= inputs.len() {
        return Err(format!("--reset-at {k} 不在 1..{} 之内，比较区间为空", inputs.len()).into());
    }
    let mut x = inputs[0].clone();
    for f in &inputs[..k] {
        x.copy_from_slice(f);
        b.infer(&mut x)?;
    }
    b.reset();
    let mut y = inputs[0].clone();
    let mut nonfinite = 0usize;
    let mut max_abs = 0.0f64;
    let mut bit_diff = 0usize;
    for f in &inputs[k..] {
        x.copy_from_slice(f);
        y.copy_from_slice(f);
        b.infer(&mut x)?;
        b2.infer(&mut y)?;
        for (p, q) in x.iter().zip(&y) {
            if p.to_bits() != q.to_bits() {
                bit_diff += 1;
            }
            if !p.is_finite() || !q.is_finite() {
                nonfinite += 1; // 两边一样的 NaN 也不算过（codex 第 2 轮 #4）
            }
            let d = (*p as f64 - *q as f64).abs();
            max_abs = if d.is_nan() {
                f64::INFINITY
            } else {
                max_abs.max(d)
            };
        }
    }
    // 位模式比较（codex 第 1 轮抽查：差 0 不等于逐位相同）
    println!(
        "{{\"model\":\"{}\",\"vs\":\"{vs}\",\"check\":\"reset\",\"reset_at\":{k},\"frames_after\":{},\"max_abs_vs_fresh\":{},\"bitwise_diff_values\":{bit_diff},\"nonfinite_values\":{nonfinite}}}",
        model.name(),
        inputs.len() - k,
        if max_abs.is_finite() {
            format!("{max_abs:.3e}")
        } else {
            "null".into()
        },
    );
    if (bit_diff != 0 || nonfinite != 0) && !ALLOW_FAIL.load(Relaxed) {
        return Err("reset 后与新开实例不逐位相同，或出现非有限值".into());
    }
    Ok(())
}

pub(crate) fn dump_io(
    model: ModelId,
    onnx: &Path,
    wav: &Path,
    inputs: &[Vec<f32>],
    prefix: &str,
) -> Result<(), Err> {
    use std::io::Write;
    let mut net = open_net(model, "ort", onnx)?;
    let mut fi = std::io::BufWriter::new(std::fs::File::create(format!("{prefix}.in.f32"))?);
    let mut fo = std::io::BufWriter::new(std::fs::File::create(format!("{prefix}.ort.f32"))?);
    let mut x = inputs[0].clone();
    for f in inputs {
        x.copy_from_slice(f);
        for v in &x {
            fi.write_all(&v.to_le_bytes())?;
        }
        net.infer(&mut x)?;
        for v in &x {
            fo.write_all(&v.to_le_bytes())?;
        }
    }
    let sha = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(wav)
        .output()?;
    let sha = String::from_utf8_lossy(&sha.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    let onnx_sha = std::process::Command::new("shasum")
        .args(["-a", "256"])
        .arg(onnx)
        .output()?;
    let onnx_sha = String::from_utf8_lossy(&onnx_sha.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    std::fs::write(
        format!("{prefix}.json"),
        format!(
            "{{\"model\":\"{}\",\"frames_total\":{},\"frame_len\":{},\"layout\":\"{}\",\"note\":\"第 0 帧是冷启动帧，接着 warmup 帧，再接计时帧；ORT 从零状态顺序跑\",\"wav\":\"{}\",\"wav_sha256\":\"{sha}\",\"onnx\":\"{}\",\"onnx_sha256\":\"{onnx_sha}\"}}\n",
            model.name(),
            inputs.len(),
            inputs[0].len(),
            if model == ModelId::Gtcrn {
                "re[257] then im[257]"
            } else {
                "wav hop samples"
            },
            wav.display(),
            onnx.display()
        ),
    )?;
    Ok(())
}
