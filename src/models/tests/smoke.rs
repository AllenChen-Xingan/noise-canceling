//! 真模型冒烟：能加载、逐帧跑、输出有限、延迟符合声明。模型文件在仓外（`~/vdn-data/models/…` 或环境变量），
//! 找不到就打印一行跳过——CI 上没有权重，这条测试只在本机有意义。逐值对拍在 exp004 的脚本里做。

use vdn_engine::{Engine, Model, SafetyConfig, offline};
use vdn_models::ModelKind;

fn run(kind: ModelKind) {
    let d = kind.declared();
    let (hop, latency) = (d.hop, d.latency_samples);
    let Some(path) = kind.default_path() else {
        // 设了 VDN_REQUIRE_MODELS 就不许跳过（本机验收用）；CI 上没有权重，不设，打印一行跳过
        assert!(
            std::env::var_os("VDN_REQUIRE_MODELS").is_none(),
            "VDN_REQUIRE_MODELS 已设，但找不到模型文件"
        );
        eprintln!("跳过：找不到模型文件（设 VDN_GTCRN_ONNX / VDN_FASTENHANCER_ONNX）");
        return;
    };
    let m = kind.open(&path).expect("打开模型");
    // 声明（延迟账用的）必须和打开后的实际一致
    assert_eq!(
        (m.sample_rate(), m.hop(), m.latency_samples()),
        (d.sample_rate, hop, latency)
    );
    let x: Vec<f32> = (0..vdn_types::MODEL_RATE)
        .map(|i| 0.1 * (i as f32 * 0.07).sin() + 0.01 * ((i * 7919 % 101) as f32 / 50.0 - 1.0))
        .collect();
    // 开着「出事送原声」才会给故障计数
    let cfg = SafetyConfig {
        fallback: true,
        ..SafetyConfig::off()
    };
    let mut e = Engine::new(m, cfg);
    let y = offline::run(&mut e, &x, 0).output;
    assert_eq!(y.len(), x.len());
    assert!(y.iter().all(|v| v.is_finite()));
    assert!(y.iter().any(|v| v.abs() > 1e-4), "输出全是零");
    let s = e.stats();
    assert_eq!(s.fault_backend + s.fault_nonfinite + s.fault_abnormal, 0);
}

#[test]
fn gtcrn_runs() {
    run(ModelKind::Gtcrn);
}

#[test]
fn fastenhancer_m_runs() {
    run(ModelKind::FastEnhancerM);
}

/// 真模型走 48 kHz `Pipeline`：任意块长进出、输出有限、非静音、延迟 = 重分块 + 重采样 + 模型 × 3。
fn run_pipeline(kind: ModelKind) {
    let Some(path) = kind.default_path() else {
        assert!(
            std::env::var_os("VDN_REQUIRE_MODELS").is_none(),
            "VDN_REQUIRE_MODELS 已设，但找不到模型文件"
        );
        eprintln!("跳过：找不到模型文件（设 VDN_GTCRN_ONNX / VDN_FASTENHANCER_ONNX）");
        return;
    };
    let d = kind.declared();
    let m = kind.open(&path).expect("打开模型");
    let mut p = vdn_engine::Pipeline::new(Engine::new(m, SafetyConfig::default()), vdn_types::IO_RATE)
        .expect("48 kHz 应可用");
    assert_eq!(
        p.latency_samples(),
        d.hop * 3 - 1 + 2 * vdn_dsp::resample::delay_high() + d.latency_samples * 3
    );
    let x: Vec<f32> = (0..vdn_types::IO_RATE)
        .map(|i| 0.1 * (i as f32 * 0.02).sin() + 0.01 * ((i * 7919 % 101) as f32 / 50.0 - 1.0))
        .collect();
    let mut y = vec![0.0; x.len()];
    // 块长轮换 96 / 256 / 4096（IO 提议块 · 009 目标块 · 大块）
    let mut i = 0;
    for n in [96usize, 256, 4096].iter().cycle() {
        if i >= x.len() {
            break;
        }
        let end = (i + n).min(x.len());
        p.process(&x[i..end], &mut y[i..end]);
        i = end;
    }
    assert!(y.iter().all(|v| v.is_finite()));
    assert!(
        y[p.latency_samples()..].iter().any(|v| v.abs() > 1e-4),
        "输出全是零"
    );
    let s = p.engine().stats();
    assert_eq!(s.fault_backend + s.fault_nonfinite + s.fault_abnormal, 0);
}

#[test]
fn gtcrn_pipeline_48k_runs() {
    run_pipeline(ModelKind::Gtcrn);
}

#[test]
fn fastenhancer_m_pipeline_48k_runs() {
    run_pipeline(ModelKind::FastEnhancerM);
}
