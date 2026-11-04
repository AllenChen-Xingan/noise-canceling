//! GTCRN 借谱（013）：真模型借出的输入功率谱与自算逐位相同；没模型文件时同 `smoke.rs` 跳过。
use vdn_dsp::{StreamingStft, power_into};
use vdn_models::{Gtcrn, ModelKind};
use vdn_types::{FRONTEND, Model, SpectrumTap};

/// 2 s：LCG 噪声 + 开关调制（只用乘加，跨平台一致）
fn input() -> Vec<f32> {
    let mut rng: u32 = 11;
    (0..FRONTEND.hop * 125)
        .map(|i| {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let w = (rng >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
            w * if (i / 1500) % 2 == 0 { 0.2 } else { 0.02 }
        })
        .collect()
}

/// 真 GTCRN 借出的功率谱 == 另起一个同口径 STFT 自算的功率谱（逐位）：场景借谱与自算等价的前提在真模型上也成立。
/// 推理出错时借出的谱仍是本帧输入的（取自推理前）。
#[test]
fn gtcrn_tap_equals_self_compute() {
    let Some(path) = ModelKind::Gtcrn.default_path() else {
        assert!(
            std::env::var_os("VDN_REQUIRE_MODELS").is_none(),
            "VDN_REQUIRE_MODELS 已设，但找不到模型文件"
        );
        eprintln!("跳过：找不到 GTCRN 模型文件（设 VDN_GTCRN_ONNX）");
        return;
    };
    let mut m = Gtcrn::open(&path).expect("打开 GTCRN");
    assert_eq!(m.spectrum_spec(), FRONTEND);
    assert!(m.input_power().is_none(), "还没处理过帧就借出了谱");
    let mut own = StreamingStft::from_spec(&FRONTEND);
    let mut want = vec![0.0f32; FRONTEND.bins()];
    let x = input();
    let mut y = vec![0.0f32; m.hop()];
    for (k, h) in x.chunks_exact(m.hop()).enumerate() {
        m.process(h, &mut y).expect("GTCRN 处理");
        power_into(own.analyze(h), &mut want);
        let got = m.input_power().expect("处理过帧后应借得出谱");
        assert!(
            got.iter().zip(&want).all(|(a, b)| a.to_bits() == b.to_bits()),
            "第 {k} 帧借出的功率谱与自算不同"
        );
    }
    // 输入长度不对：不做分析，也不借出旧谱
    assert!(m.process(&x[..10], &mut y[..10]).is_err());
    assert!(m.input_power().is_none());
    m.reset();
    assert!(m.input_power().is_none());
}
