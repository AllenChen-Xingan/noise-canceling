//! 修音链行为测试：关掉时原样透传；下限与舒适噪声补「真空」；平滑压住增益乱跳；AGC 往目标走；限幅不超 −1 dBFS。
use realfft::num_complex::Complex32;
use vdn_polish::{Agc, PolishParams, SpectralPolish};

const BINS: usize = 257;

fn noise_spec(seed: &mut u64, amp: f32) -> Vec<Complex32> {
    (0..BINS)
        .map(|_| {
            *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let a = ((*seed >> 33) as f32 / (1u64 << 31) as f32) * std::f32::consts::TAU;
            Complex32::new(amp * a.cos(), amp * a.sin())
        })
        .collect()
}

#[test]
fn off_is_identity() {
    let mut sp = SpectralPolish::new(512, 256, 16000.0);
    let mut s = 1u64;
    for _ in 0..50 {
        let dry = noise_spec(&mut s, 1.0);
        let wet0: Vec<Complex32> = dry.iter().enumerate().map(|(k, d)| *d * (0.1 + 0.8 * (k % 7) as f32 / 7.0)).collect();
        let mut wet = wet0.clone();
        sp.process(&dry, &mut wet, &PolishParams::OFF, None);
        for (a, b) in wet.iter().zip(&wet0) {
            assert!((a - b).norm() < 1e-5 * (1.0 + b.norm()));
        }
    }
    let mut agc = Agc::new(256, 16000.0);
    let mut x: Vec<f32> = (0..256).map(|i| (i as f32 * 0.05).sin() * 0.1).collect();
    let x0 = x.clone();
    agc.process(&mut x, &PolishParams::OFF, 1.0);
    assert_eq!(x, x0);
}

#[test]
fn floor_and_comfort_fill_the_vacuum() {
    let mut sp = SpectralPolish::new(512, 256, 16000.0);
    let p = PolishParams { floor_db: -20.0, comfort_db: -100.0, smooth: 0.0, ..PolishParams::OFF };
    let mut s = 7u64;
    let mut ratio = 0.0;
    for _ in 0..200 {
        let dry = noise_spec(&mut s, 1.0);
        let mut wet = vec![Complex32::new(0.0, 0.0); BINS]; // 模型把背景全抽空
        sp.process(&dry, &mut wet, &p, Some(0.0));
        ratio = wet.iter().map(|c| c.norm_sqr()).sum::<f32>() / dry.iter().map(|c| c.norm_sqr()).sum::<f32>();
    }
    assert!((10.0 * ratio.log10() + 20.0).abs() < 1.0, "下限 −20 dB，实得 {:.1} dB", 10.0 * ratio.log10());

    let mut sp = SpectralPolish::new(512, 256, 16000.0);
    let p = PolishParams { comfort_db: -10.0, ..PolishParams::OFF };
    for _ in 0..200 {
        let dry = noise_spec(&mut s, 1.0);
        let mut wet = vec![Complex32::new(0.0, 0.0); BINS];
        sp.process(&dry, &mut wet, &p, Some(0.0));
        ratio = wet.iter().map(|c| c.norm_sqr()).sum::<f32>() / dry.iter().map(|c| c.norm_sqr()).sum::<f32>();
    }
    // 底噪跟踪是「快降慢升」的最小值，会比平均功率低几 dB；舒适噪声在 −10 dB 下方几 dB 内即可
    let r = 10.0 * ratio.log10();
    assert!((-18.0..=-9.0).contains(&r), "舒适噪声 {r:.1} dB");
}

#[test]
fn smoothing_calms_gain_flicker() {
    // 无人声帧里每个频点的增益在 0 与 1 之间乱跳（音乐噪声的来源）；平滑后逐帧增益的变化应明显变小
    let run = |smooth: f32| {
        let mut sp = SpectralPolish::new(512, 256, 16000.0);
        let p = PolishParams { smooth, ..PolishParams::OFF };
        let mut s = 3u64;
        let mut prev = vec![0.0f32; BINS];
        let mut flick = 0.0f32;
        for f in 0..300 {
            let dry = noise_spec(&mut s, 1.0);
            let mut wet: Vec<Complex32> = dry.iter().enumerate().map(|(k, d)| if (k * 31 + f * 17) % 5 == 0 { *d } else { *d * 0.02 }).collect();
            sp.process(&dry, &mut wet, &p, Some(0.0));
            let g: Vec<f32> = wet.iter().zip(&dry).map(|(w, d)| w.norm() / d.norm()).collect();
            if f > 50 {
                flick += g.iter().zip(&prev).map(|(a, b)| (a - b).abs()).sum::<f32>();
            }
            prev = g;
        }
        flick
    };
    let (raw, smooth) = (run(0.0), run(1.0));
    assert!(smooth < 0.5 * raw, "平滑前 {raw:.1}，平滑后 {smooth:.1}");
}

#[test]
fn agc_moves_toward_target_and_limiter_caps() {
    let mut agc = Agc::new(256, 16000.0);
    let p = PolishParams { agc: 1.0, agc_target_dbfs: -26.0, agc_range_db: 10.0, ..PolishParams::OFF };
    for _ in 0..62 * 8 {
        // −40 dBFS 的「人声」：需要 +14 dB，被限在 +10 dB
        let mut x: Vec<f32> = (0..256).map(|i| (i as f32 * 0.07).sin() * 0.01414).collect();
        agc.process(&mut x, &p, 1.0);
    }
    assert!((agc.gain_db() - 10.0).abs() < 0.2, "增益 {:.2}", agc.gain_db());
    let mut agc = Agc::new(256, 16000.0);
    for _ in 0..20 {
        let mut x: Vec<f32> = (0..256).map(|i| if i % 50 == 0 { 3.0 } else { 0.0 }).collect();
        agc.process(&mut x, &PolishParams::OFF, 0.0);
        assert!(x.iter().all(|v| v.abs() <= 0.8913), "限幅后超了 −1 dBFS");
    }
}
