//! 评分模型对拍：`vdn-eval nrmos`（Rust + ort）与上游 Python（scoreq 1.0.1 · distillmos 0.9.1）在同 6 段公式信号上的分差 ≤ 0.01。
//! 参考值由 `tools/gen_nrmos_ref.py` 用上游算出后进仓（`tests/fixtures/nrmos-upstream.json`）；信号公式两边必须一字不差。
//!
//! 要本机模型（SCOREQ 378 MB · Distill-MOS 导出物），CI 上没有 → 标 ignore。人工跑：
//! `cargo test -p vdn-eval --release --test nrmos_parity -- --ignored --nocapture`

use std::path::{Path, PathBuf};

use vdn_eval::qa::nrmos::{Kind, locate};

const SR: u32 = 16_000;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    }
}

fn clean(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let t = i as f64 / SR as f64;
            let on = if ((t * 2.5) as i64) % 2 == 0 { 1.0 } else { 0.0 };
            let f0 = 120.0 * (1.0 + 0.08 * (2.0 * std::f64::consts::PI * 0.7 * t).sin());
            let v: f64 = (1..25)
                .map(|k| (2.0 * std::f64::consts::PI * k as f64 * f0 * t).sin() / k as f64)
                .sum();
            let am = 0.6 + 0.4 * (2.0 * std::f64::consts::PI * 4.0 * t).sin();
            0.1 * v * am * on
        })
        .collect()
}

/// 同 `gen_nrmos_ref.py` 的 `signal()`：a < 0 只有噪声；a > 0 干净 + 噪声。
fn signal(n: usize, a: f64, seed: u64) -> Vec<f32> {
    let mut g = Lcg(seed);
    let x: Vec<f64> = if a < 0.0 {
        (0..n).map(|_| 0.1 * g.next()).collect()
    } else {
        let c = clean(n);
        if a > 0.0 {
            c.iter().map(|v| v + a * g.next()).collect()
        } else {
            c
        }
    };
    x.iter().map(|v| *v as f32).collect()
}

fn fixture() -> serde_json::Value {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/nrmos-upstream.json");
    serde_json::from_str(&std::fs::read_to_string(p).expect("缺参考值：先跑 tools/gen_nrmos_ref.py"))
        .unwrap()
}

fn score_all(kind: Kind, wavs: &[PathBuf]) -> Vec<f64> {
    let model = locate(kind).unwrap_or_else(|| panic!("找不到 {} 模型", kind.as_str()));
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_vdn-eval"))
        .arg("nrmos")
        .arg(kind.as_str())
        .arg(&model)
        .args(wavs)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap()["mos"].as_f64().unwrap())
        .collect()
}

#[test]
#[ignore = "要本机评分模型（SCOREQ 378 MB、Distill-MOS 导出物），人工跑：见文件头"]
fn nrmos_matches_upstream_within_0_01() {
    let v = fixture();
    let cases = v["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 6);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("nrmos-parity");
    std::fs::create_dir_all(&dir).unwrap();
    let mut wavs = Vec::new();
    for c in cases {
        let x = signal(
            c["n"].as_u64().unwrap() as usize,
            c["noise"].as_f64().unwrap(),
            c["seed"].as_u64().unwrap(),
        );
        let p = dir.join(format!("{}.wav", c["case"].as_str().unwrap()));
        vdn_eval::wav::write(&p, &x, SR).unwrap();
        wavs.push(p);
    }
    for kind in Kind::ALL {
        let got = score_all(kind, &wavs);
        let mut worst = 0.0f64;
        for (c, g) in cases.iter().zip(&got) {
            let want = c[kind.as_str()].as_f64().unwrap();
            let d = (g - want).abs();
            worst = worst.max(d);
            assert!(
                d <= 0.01,
                "{} · {}：Rust {g:.4} ↔ 上游 {want:.4}（差 {d:.4}）",
                kind.as_str(),
                c["case"]
            );
        }
        eprintln!("{} 对拍最大差 {worst:.5}", kind.as_str());
    }
}

/// 模型文件哈希与 `nrmos.rs` 登记的正本一致（不打分，只核身份）。
#[test]
#[ignore = "要本机评分模型，人工跑"]
fn local_models_match_registered_hash() {
    for kind in Kind::ALL {
        let p = locate(kind).unwrap_or_else(|| panic!("找不到 {} 模型", kind.as_str()));
        vdn_eval::qa::nrmos::verify(kind, &p).unwrap();
    }
}
