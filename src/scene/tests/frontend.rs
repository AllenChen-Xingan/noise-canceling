//! 借谱 vs 自算（013 第 8 节第 2、3 条）。谱源用 `StreamingStft::from_spec(FRONTEND)` + `power_into`——
//! 与 GTCRN 适配器借出功率谱是同一段代码，所以这里不需要模型文件，CI 上能跑；真 GTCRN 的同一检查在 `vdn-models` 的测试里。
mod common;

use common::{HOP, TICK_HOPS};
use vdn_dsp::{StreamingStft, power_into};
use vdn_polish::{PolishChain, PolishParams};
use vdn_scene::{Judgement, SceneAnalyzer};
use vdn_types::{FRONTEND, FrameCtx, MODEL_RATE};

/// 借谱跑一遍：外面一个 STFT 当降噪模型的前端，场景吃它借出的功率谱。返回判断与前端正向 FFT 次数。
fn run_borrowed(x: &[f32]) -> (Vec<Judgement>, u64, u64) {
    let mut fe = StreamingStft::from_spec(&FRONTEND);
    let mut pow = vec![0.0f32; FRONTEND.bins()];
    let mut own = 0;
    let js = common::run(x, |sa, h| {
        power_into(fe.analyze(h), &mut pow);
        sa.push_hop_with_power(h, &pow);
        own = sa.own_fft_count();
    });
    (js, fe.forward_count(), own)
}

fn max_feature_diff(a: &[Judgement], b: &[Judgement]) -> f32 {
    a.iter()
        .zip(b)
        .flat_map(|(x, y)| x.features.iter().zip(&y.features).map(|(p, q)| (p - q).abs()))
        .fold(0.0, f32::max)
}

/// 35 维最大绝对差 ≤ 1e-5（预期 0），每次判断的原始细场景与生效细场景都相同。
#[test]
fn borrowed_equals_self_compute() {
    let x = common::signal();
    let own = common::run(&x, |sa, h| sa.push_hop(h));
    let (borrowed, _, _) = run_borrowed(&x);
    assert_eq!(own.len(), borrowed.len());
    assert!(own.len() >= 20, "判断次数太少，测不出东西");
    let d = max_feature_diff(&own, &borrowed);
    eprintln!("借谱 vs 自算：35 维最大绝对差 {d:e}（阈值 1e-5），判断 {} 次", own.len());
    assert!(d <= 1e-5, "35 维最大绝对差 {d:e} > 1e-5");
    for (k, (a, b)) in own.iter().zip(&borrowed).enumerate() {
        assert_eq!(a.label.fine, b.label.fine, "第 {k} 次判断的原始细场景不同");
        assert_eq!(a.committed, b.committed, "第 {k} 次判断的生效场景不同");
    }
    // 信号确实让判断动起来了（否则上面的比较没意义）
    assert!(own.windows(2).any(|w| w[0].label.fine != w[1].label.fine), "测试信号没让场景变化");
}

/// 借谱时每帧正向 FFT 恰好 1 次：前端每帧 1 次，场景自己 0 次。
#[test]
fn one_forward_fft_per_frame_when_borrowing() {
    let x = common::signal();
    let hops = (x.len() / HOP) as u64;
    let (_, fe_count, own) = run_borrowed(&x);
    assert_eq!(fe_count, hops);
    assert_eq!(own, 0);
}

/// 组合路径（前端 + 场景借谱 + 测试里手动接修音频谱级）每帧正向 FFT 恰好 1 次。
#[test]
fn combined_path_counts_one_fft_per_frame() {
    let x = common::signal();
    let mut fe = StreamingStft::from_spec(&FRONTEND);
    let mut sa = SceneAnalyzer::new(MODEL_RATE, HOP);
    let mut chain = PolishChain::new(FRONTEND.n_fft, FRONTEND.hop, MODEL_RATE as f32, PolishParams::preset(2));
    let mut pow = vec![0.0f32; FRONTEND.bins()];
    let mut dry = vec![Default::default(); FRONTEND.bins()];
    let mut out = vec![0.0f32; HOP];
    let mut frames = 0u64;
    for (k, h) in x.chunks_exact(HOP).enumerate() {
        let before = fe.forward_count() + sa.own_fft_count();
        let spec = fe.analyze(h);
        power_into(spec, &mut pow);
        dry.copy_from_slice(spec);
        // 假模型：输出谱 = 输入谱减半
        for c in spec.iter_mut() {
            *c *= 0.5;
        }
        sa.push_hop_with_power(h, &pow);
        chain.process_spectrum(fe.spectrum_mut(), &dry, FrameCtx::default());
        fe.synthesize(&mut out);
        chain.process_time(&mut out);
        if (k + 1) % TICK_HOPS == 0 {
            sa.judge();
        }
        assert_eq!(fe.forward_count() + sa.own_fft_count() - before, 1, "第 {k} 帧正向 FFT 不是 1 次");
        frames += 1;
    }
    assert_eq!(fe.forward_count(), frames);
    assert_eq!(sa.own_fft_count(), 0);
}

/// 先借谱、中途换成自算（换到 FE-M）：结果与全程自算相同——借谱时自算缓冲也在同步推进。
#[test]
fn switching_to_self_compute_mid_stream_is_seamless() {
    let x = common::signal();
    let own = common::run(&x, |sa, h| sa.push_hop(h));
    let mut fe = StreamingStft::from_spec(&FRONTEND);
    let mut pow = vec![0.0f32; FRONTEND.bins()];
    let mut k = 0usize;
    let half = x.len() / HOP / 2;
    let mixed = common::run(&x, |sa, h| {
        if k < half {
            power_into(fe.analyze(h), &mut pow);
            sa.push_hop_with_power(h, &pow);
        } else {
            sa.push_hop(h);
        }
        k += 1;
    });
    let d = max_feature_diff(&own, &mixed);
    assert!(d <= 1e-5, "切换后 35 维最大绝对差 {d:e} > 1e-5");
    for (a, b) in own.iter().zip(&mixed) {
        assert_eq!(a.committed, b.committed);
    }
}

/// 借来的谱长度不对：退回自算，结果仍与自算相同（不 panic、不算错）。
#[test]
fn wrong_length_power_falls_back_to_self_compute() {
    let x = common::signal();
    let own = common::run(&x, |sa, h| sa.push_hop(h));
    let short = [0.0f32; 10];
    let fb = common::run(&x, |sa, h| sa.push_hop_with_power(h, &short));
    assert_eq!(common::digest(&own), common::digest(&fb));
}
