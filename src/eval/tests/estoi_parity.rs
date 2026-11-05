//! ESTOI 对拍：Rust 重写（`verifiers::estoi`）与上游 pystoi 0.4.1 在同一组公式信号上的结果差 ≤ 1e−4
//!（025 E3：原容差 0.01 太松，实测最大差 < 1e−5）。含「输出整段数字零」样例。
//! 参考值由 `tests/fixtures/gen_estoi_ref.py` 用上游算出后进仓；信号公式两边必须一字不差。

use vdn_eval::verifiers::estoi::estoi;

const SR: u32 = vdn_types::MODEL_RATE;

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
            let on = if ((t * 2.5) as usize).is_multiple_of(2) {
                1.0
            } else {
                0.0
            };
            let f0 = 120.0 * (1.0 + 0.08 * (2.0 * std::f64::consts::PI * 0.7 * t).sin());
            let v: f64 = (1..25)
                .map(|k| (2.0 * std::f64::consts::PI * k as f64 * f0 * t).sin() / k as f64)
                .sum();
            let am = 0.6 + 0.4 * (2.0 * std::f64::consts::PI * 4.0 * t).sin();
            0.1 * v * am * on
        })
        .collect()
}

fn case(kind: &str, n: usize, seed: u64) -> (Vec<f32>, Vec<f32>) {
    let x = clean(n);
    let mut g = Lcg(seed);
    let y: Vec<f64> = if let Some(a) = kind.strip_prefix("noise-") {
        let a: f64 = a.parse().unwrap();
        x.iter().map(|v| v + a * g.next()).collect()
    } else if kind == "zero" {
        vec![0.0; n]
    } else if kind == "gate" {
        x.iter()
            .enumerate()
            .map(|(i, v)| v * if (i / 1600) % 3 == 0 { 0.1 } else { 1.0 })
            .collect()
    } else {
        x.clone()
    };
    // 上游用 float64 算；这里接口是 f32，量化误差 ~1e−7，远小于容差 1e−4
    (
        x.iter().map(|v| *v as f32).collect(),
        y.iter().map(|v| *v as f32).collect(),
    )
}

#[test]
fn estoi_matches_pystoi_within_1e_4() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/estoi-pystoi.json");
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let n = v["n"].as_u64().unwrap() as usize;
    let cases = v["cases"].as_array().unwrap();
    assert!(cases.len() >= 6);
    assert!(cases.iter().any(|c| c["case"] == "zero"), "缺整段数字零样例");
    let mut worst = 0.0f64;
    for c in cases {
        let kind = c["case"].as_str().unwrap();
        let (x, y) = case(kind, n, c["seed"].as_u64().unwrap());
        let got = estoi(&x, &y, SR).unwrap_or_else(|| panic!("{kind}: Rust 没出分"));
        let want = c["estoi"].as_f64().unwrap();
        let d = (got - want).abs();
        worst = worst.max(d);
        assert!(
            d <= 1e-4,
            "{kind}: Rust {got:.4} ↔ pystoi {want:.4}（差 {d:.4}）"
        );
    }
    eprintln!("ESTOI 对拍最大差 {worst:.5}");
}
