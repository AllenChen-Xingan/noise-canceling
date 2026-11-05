//! DNSMOS 与官方 `dnsmos_local.py` 对拍（调研 025 F1）。要官方 ONNX 模型、官方脚本的参考输出，本机 CI 不带，标 ignore。
//!
//! 人工步骤：
//! 1. 用 DNS-Challenge 仓库的 `DNSMOS/dnsmos_local.py`（非个性化 `sig_bak_ovr.onnx`）给一个目录里的 16 kHz wav 打分，
//!    把结果整理成 JSON：`[{"wav": "<相对路径>", "sig": .., "bak": .., "ovrl": ..}, …]`
//! 2. `VDN_DNSMOS_MODEL=<sig_bak_ovr.onnx> VDN_DNSMOS_REF=<上一步 JSON> VDN_DNSMOS_DIR=<wav 目录> \
//!    cargo test -p vdn-eval --test dnsmos_parity -- --ignored`
//! 3. 三项都要 |Rust − 官方| ≤ 0.01（与 SCOREQ / Distill-MOS 对拍同一容差）

use std::path::PathBuf;

#[test]
#[ignore = "要官方 DNSMOS 模型与 dnsmos_local.py 的参考输出（不进仓），人工跑"]
fn dnsmos_matches_official_script() {
    let env = |k: &str| PathBuf::from(std::env::var(k).unwrap_or_else(|_| panic!("要设 {k}")));
    let (model, refp, dir) = (
        env("VDN_DNSMOS_MODEL"),
        env("VDN_DNSMOS_REF"),
        env("VDN_DNSMOS_DIR"),
    );
    let refs: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(refp).unwrap()).unwrap();
    let mut d = vdn_eval::qa::dnsmos::Dnsmos::open(&model).unwrap();
    let mut n = 0;
    for r in refs.as_array().unwrap() {
        let (x, sr) = vdn_eval::wav::read(&dir.join(r["wav"].as_str().unwrap())).unwrap();
        assert_eq!(sr, vdn_types::MODEL_RATE);
        let m = d.score(&x).unwrap();
        for (k, got) in [("sig", m.sig), ("bak", m.bak), ("ovrl", m.ovrl)] {
            let want = r[k].as_f64().unwrap();
            assert!(
                (got - want).abs() <= 0.01,
                "{} · {k}：Rust {got:.4} ↔ 官方 {want:.4}",
                r["wav"]
            );
        }
        n += 1;
    }
    assert!(n > 0, "参考里没有条目");
}
