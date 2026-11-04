//! 修音频谱级吃借来的带噪谱 vs 自算的带噪谱（013 第 8 节第 4 条，测试里手动接；模型里的槽不做，见 013 第 4 节）。
//! 借：降噪模型前端分析时复制出的 dry（GTCRN 推理前那份）；自算：另起一个同口径 STFT 分析同一段输入。
//! 两条路的修音输出（改写后的谱与时域 hop）必须逐位相同。
use realfft::num_complex::Complex32;
use vdn_dsp::StreamingStft;
use vdn_polish::{PolishChain, PolishParams};
use vdn_types::{FRONTEND, FrameCtx, MODEL_RATE};

fn input() -> Vec<f32> {
    let mut rng: u32 = 7;
    (0..FRONTEND.hop * 300)
        .map(|i| {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let w = (rng >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
            0.05 * w + 0.2 * (i as f32 * 0.06).sin() * if (i / 3000) % 2 == 0 { 1.0 } else { 0.0 }
        })
        .collect()
}

fn chain() -> PolishChain {
    PolishChain::new(FRONTEND.n_fft, FRONTEND.hop, MODEL_RATE as f32, PolishParams::preset(2))
}

#[test]
fn borrowed_dry_equals_self_computed_dry() {
    let x = input();
    let hop = FRONTEND.hop;
    // 借：模型前端（分析 → 复制 dry → 假模型改写成输出谱 → 修音 → 合成）
    let mut fe = StreamingStft::from_spec(&FRONTEND);
    let mut dry_b = vec![Complex32::default(); FRONTEND.bins()];
    // 自算：修音自己另做一次 STFT 拿 dry
    let mut own = StreamingStft::from_spec(&FRONTEND);
    let (mut ca, mut cb) = (chain(), chain());
    let mut wet_s = vec![Complex32::default(); FRONTEND.bins()];
    let (mut ya, mut yb) = (vec![0.0f32; hop], vec![0.0f32; hop]);
    let mut syn_b = StreamingStft::from_spec(&FRONTEND);
    for h in x.chunks_exact(hop) {
        let spec = fe.analyze(h);
        dry_b.copy_from_slice(spec);
        for (k, c) in spec.iter_mut().enumerate() {
            *c *= 0.2 + 0.6 * (k % 5) as f32 / 5.0;
        }
        wet_s.copy_from_slice(spec);
        let dry_s = own.analyze(h);
        assert_eq!(&dry_b[..], &dry_s[..], "借来的 dry 与自算的 dry 不同");

        ca.process_spectrum(fe.spectrum_mut(), &dry_b, FrameCtx::default());
        let wb = syn_b.analyze(h); // 只为拿一块同口径的谱缓冲，内容马上被覆盖
        wb.copy_from_slice(&wet_s);
        cb.process_spectrum(wb, own.spectrum_mut(), FrameCtx::default());
        assert_eq!(fe.spectrum_mut().to_vec(), syn_b.spectrum_mut().to_vec(), "修音改写后的谱不同");

        fe.synthesize(&mut ya);
        ca.process_time(&mut ya);
        syn_b.synthesize(&mut yb);
        cb.process_time(&mut yb);
        assert!(ya.iter().zip(&yb).all(|(a, b)| a.to_bits() == b.to_bits()), "修音输出不同");
    }
}
