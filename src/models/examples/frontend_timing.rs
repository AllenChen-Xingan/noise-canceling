//! 每帧耗时：「GTCRN + 场景自算频谱」（改前的走法）vs「GTCRN + 场景借 GTCRN 的谱」（013）。
//! 两种走法在同一份源码上跑，交替轮次以抵消机器状态漂移；每帧分别记模型、场景两段耗时。
//! 场景自算入口与改前逐位相同（见 `vdn-scene` 的对拍测试），所以「自算」一栏就是改前的场景开销。
//!
//! 用法：VDN_GTCRN_ONNX=<模型> caffeinate -i cargo run --release -p vdn-models --example frontend_timing -- [轮数]
//! 要插电跑；结果打到标准输出（Markdown 表）。
//! 看「场景」两列：借谱省掉的是场景那次 512 点正向 FFT 加窗，预期每帧省几百纳秒到一两微秒；
//! 「合计」与每帧预算（一个 hop 的时长）比，省下的占比才是这次改动在算力上的意义。
use std::time::Instant;

use vdn_models::{Gtcrn, ModelKind};
use vdn_scene::SceneAnalyzer;
use vdn_types::{FRONTEND, MODEL_RATE, Model, SpectrumTap};

const HOPS: usize = 1875; // 30 s

fn signal() -> Vec<f32> {
    let mut rng: u32 = 3;
    (0..FRONTEND.hop * HOPS)
        .map(|i| {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let w = (rng >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
            w * if (i / 4000) % 2 == 0 { 0.2 } else { 0.02 }
        })
        .collect()
}

/// 跑一遍，返回每帧（模型 µs，场景 µs）
fn run(m: &mut Gtcrn, x: &[f32], borrow: bool) -> Vec<(f64, f64)> {
    m.reset();
    let mut sa = SceneAnalyzer::new(MODEL_RATE, FRONTEND.hop);
    let mut out = vec![0.0f32; FRONTEND.hop];
    let mut t = Vec::with_capacity(HOPS);
    let spec = m.spectrum_spec();
    for h in x.chunks(FRONTEND.hop) {
        let t0 = Instant::now();
        m.process(h, &mut out).expect("GTCRN 推理");
        let t1 = Instant::now();
        match (borrow, m.input_power()) {
            (true, Some(p)) => sa.push_hop_with_power(h, &spec, p),
            _ => sa.push_hop(h),
        }
        let t2 = Instant::now();
        t.push(((t1 - t0).as_secs_f64() * 1e6, (t2 - t1).as_secs_f64() * 1e6));
    }
    if borrow {
        assert_eq!(sa.own_fft_count(), 0, "借谱走法里场景不该自己做 FFT");
    }
    t
}

fn pct(v: &mut [f64], q: f64) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[((v.len() - 1) as f64 * q).round() as usize]
}

fn main() {
    let rounds: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    let path = ModelKind::Gtcrn
        .default_path()
        .expect("找不到 GTCRN 模型文件（设 VDN_GTCRN_ONNX）");
    let mut m = Gtcrn::open(&path).expect("打开 GTCRN");
    let x = signal();
    run(&mut m, &x, false); // 预热
    let (mut own, mut bor) = (Vec::new(), Vec::new());
    for r in 0..rounds {
        // 交替先后，抵消漂移
        let order = if r % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        };
        for b in order {
            let t = run(&mut m, &x, b);
            if b { bor.extend(t) } else { own.extend(t) }
        }
    }
    println!(
        "| 走法 | 帧数 | 模型 中位 µs | 场景 中位 µs | 场景 p99 µs | 合计 中位 µs | 合计 p99 µs |"
    );
    println!("|---|---|---|---|---|---|---|");
    for (name, t) in [("场景自算（改前）", &own), ("场景借谱（013）", &bor)] {
        let mut mo: Vec<f64> = t.iter().map(|p| p.0).collect();
        let mut sc: Vec<f64> = t.iter().map(|p| p.1).collect();
        let mut to: Vec<f64> = t.iter().map(|p| p.0 + p.1).collect();
        println!(
            "| {name} | {} | {:.1} | {:.2} | {:.2} | {:.1} | {:.1} |",
            t.len(),
            pct(&mut mo, 0.5),
            pct(&mut sc, 0.5),
            pct(&mut sc, 0.99),
            pct(&mut to, 0.5),
            pct(&mut to, 0.99)
        );
    }
    let budget = FRONTEND.hop as f64 * 1e6 / f64::from(FRONTEND.sample_rate);
    println!("\n每帧预算 {budget:.0} µs（一个 hop）；{rounds} 轮 × {HOPS} 帧，两种走法交替先后。");
}
