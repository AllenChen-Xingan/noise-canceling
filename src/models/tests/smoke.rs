//! 真模型冒烟：能加载、逐帧跑、输出有限、延迟符合声明。模型文件在仓外（`~/vdn-data/models/…` 或环境变量），
//! 找不到就打印一行跳过——CI 上没有权重，这条测试只在本机有意义。逐值对拍在 exp004 的脚本里做。

use vdn_engine::{Engine, Model, SafetyConfig, offline};
use vdn_models::ModelKind;

fn run(kind: ModelKind, hop: usize, latency: usize) {
    let Some(path) = kind.default_path() else {
        eprintln!("跳过 {kind:?}：找不到模型文件");
        return;
    };
    let m = kind.open(&path).expect("打开模型");
    assert_eq!((m.hop(), m.latency_samples()), (hop, latency));
    let x: Vec<f32> = (0..16000)
        .map(|i| 0.1 * (i as f32 * 0.07).sin() + 0.01 * ((i * 7919 % 101) as f32 / 50.0 - 1.0))
        .collect();
    let mut e = Engine::new(m, SafetyConfig::off());
    let y = offline::run(&mut e, &x, 0).output;
    assert_eq!(y.len(), x.len());
    assert!(y.iter().all(|v| v.is_finite()));
    assert!(y.iter().any(|v| v.abs() > 1e-4), "输出全是零");
    let s = e.stats();
    assert_eq!(s.fault_backend + s.fault_nonfinite + s.fault_abnormal, 0);
}

#[test]
fn gtcrn_runs() {
    run(ModelKind::Gtcrn, 256, 256);
}

#[test]
fn fastenhancer_m_runs() {
    run(ModelKind::FastEnhancerM, 160, 352);
}
