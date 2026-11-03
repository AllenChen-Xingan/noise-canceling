//! `Pipeline` 的行为测试 + 不分配证明（用假模型，不需要 ONNX Runtime）。

#[cfg(debug_assertions)]
use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_engine::testing::{DelayGain, Identity};
use vdn_engine::{Engine, FaultInjector, FaultKind, Mode, Pipeline, PipelineError, SafetyConfig};
use vdn_types::Model;

// AllocDisabler 只在 debug_assertions 下存在：release 下只跑行为测试
#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

const SR: u32 = 48_000;

/// 通带内（< 7 kHz）的多正弦，宽带到足以让互相关峰唯一。
fn multisine(n: usize) -> Vec<f32> {
    let fs = [220.0f32, 523.0, 1187.0, 2213.0, 3301.0, 4519.0, 6007.0];
    (0..n)
        .map(|i| {
            let t = i as f32 / SR as f32;
            fs.iter()
                .map(|f| 0.08 * (2.0 * std::f32::consts::PI * f * t + f * 0.01).sin())
                .sum()
        })
        .collect()
}

/// 固定种子的块长序列（1..=max）。
fn chunk_lens(seed: u64, total: usize, max: usize) -> Vec<usize> {
    let mut s = seed;
    let mut out = Vec::new();
    let mut left = total;
    while left > 0 {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let n = ((s >> 33) as usize % max + 1).min(left);
        out.push(n);
        left -= n;
    }
    out
}

fn run<M: Model>(p: &mut Pipeline<M>, x: &[f32], lens: &[usize]) -> Vec<f32> {
    let mut y = vec![0.0; x.len()];
    let mut i = 0;
    for &n in lens {
        p.process(&x[i..i + n], &mut y[i..i + n]);
        i += n;
    }
    y
}

fn measured_lag(x: &[f32], y: &[f32], max_lag: usize) -> usize {
    let corr = |lag: usize| -> f64 {
        (max_lag..x.len())
            .map(|i| f64::from(y[i]) * f64::from(x[i - lag]))
            .sum()
    };
    (0..max_lag)
        .max_by(|&a, &b| corr(a).total_cmp(&corr(b)))
        .unwrap()
}

fn identity() -> Pipeline<Identity> {
    Pipeline::new(Engine::new(Identity, SafetyConfig::off()), SR).unwrap()
}

#[test]
fn rejects_non_triple_rate() {
    let e = Engine::new(Identity, SafetyConfig::off());
    assert!(matches!(
        Pipeline::new(e, 44_100),
        Err(PipelineError::UnsupportedRate {
            io_rate: 44_100,
            model_rate: 16_000,
        })
    ));
}

#[test]
fn identity_declared_latency_matches_measured() {
    let mut p = identity();
    assert_eq!(p.latency_samples(), 767 + 96);
    let x = multisine(SR as usize);
    let y = run(&mut p, &x, &chunk_lens(1, x.len(), 500));
    assert_eq!(measured_lag(&x, &y, 3000), p.latency_samples());
    // 通带内信号：输出 ≈ 输入延迟 L（只差重采样滤波的通带纹波）
    let l = p.latency_samples();
    let err = (l + 2000..x.len())
        .map(|i| (y[i] - x[i - l]).abs())
        .fold(0.0f32, f32::max);
    assert!(err < 0.01, "Identity 链路误差 {err}");
}

#[test]
fn model_latency_counts_three_times() {
    // 模型延迟 100 个 16k 样本 → 300 个 48k 样本
    let e = Engine::new(DelayGain::new(256, 100, 1.0), SafetyConfig::off());
    let mut p = Pipeline::new(e, SR).unwrap();
    assert_eq!(p.latency_samples(), 767 + 96 + 300);
    let x = multisine(SR as usize);
    let y = run(&mut p, &x, &chunk_lens(2, x.len(), 96));
    assert_eq!(measured_lag(&x, &y, 3000), p.latency_samples());
    // 互相关相邻 ±1 的峰值差不大，再用残差钉死：在声明延迟处逐样本接近
    let l = p.latency_samples();
    let err = (l + 2000..x.len())
        .map(|i| (y[i] - x[i - l]).abs())
        .fold(0.0f32, f32::max);
    assert!(err < 0.01, "残差 {err}");
}

#[test]
fn ledger_total_equals_latency_samples() {
    let e = Engine::new(DelayGain::new(160, 352, 1.0), SafetyConfig::default());
    let p = Pipeline::new(e, SR).unwrap();
    let want_ms = p.latency_samples() as f64 * 1000.0 / f64::from(SR);
    let got = p.latency_ledger().total_ms();
    assert!(
        (got - want_ms).abs() < 1e-9,
        "账 {got} ms ≠ 声明 {want_ms} ms"
    );
    assert_eq!(p.latency_samples(), 480 - 1 + 96 + 352 * 3);
}

#[test]
fn chunking_does_not_change_output() {
    // 规约 G6：同一段音频随机切块喂 = 整段喂，逐样本一致（带三件保命的默认配置，模式切换也在同一样本处）
    let x = multisine(SR as usize * 2);
    let make = || {
        let e = Engine::new(DelayGain::new(256, 256, 0.5), SafetyConfig::default());
        Pipeline::new(e, SR).unwrap()
    };
    let whole = run(&mut make(), &x, &[x.len()]);
    for seed in [3u64, 4, 5] {
        for max in [1usize, 7, 96, 1000, 5000] {
            let lens = chunk_lens(seed, x.len(), max);
            assert_eq!(run(&mut make(), &x, &lens), whole, "seed={seed} max={max}");
        }
    }
}

#[test]
fn passthrough_is_exactly_delayed_dry() {
    // 模型输出非零且很大（增益 1e6）：直通也必须逐样本等于延迟原声，不能被 o + (d - o) 的舍入吃掉
    let mut p = Pipeline::new(
        Engine::new(DelayGain::new(256, 256, 1e6), SafetyConfig::off()),
        SR,
    )
    .unwrap();
    p.set_mode(Mode::Passthrough);
    p.reset(); // 不淡化，直接落到原声
    let x = multisine(20_000);
    let y = run(&mut p, &x, &chunk_lens(6, x.len(), 300));
    let l = p.latency_samples();
    assert!(y[..l].iter().all(|v| *v == 0.0));
    assert_eq!(&y[l..], &x[..x.len() - l]);
}

#[test]
fn mode_switch_fades_over_one_hop_and_keeps_alignment() {
    // 模型输出全零：Denoise = 0，Passthrough = 延迟原声；切换处按 768 样本线性淡化
    let mut p = Pipeline::new(
        Engine::new(DelayGain::new(256, 256, 0.0), SafetyConfig::off()),
        SR,
    )
    .unwrap();
    let l = p.latency_samples();
    let hop = p.hop_io();
    let x = vec![1.0f32; 12_000];
    let mut y = vec![0.0; x.len()];
    let sw = 5_000;
    p.process(&x[..sw], &mut y[..sw]);
    p.set_mode(Mode::Passthrough);
    p.process(&x[sw..], &mut y[sw..]);
    assert!(y[..sw].iter().all(|v| *v == 0.0));
    // 第 k 个切换后样本的原声占比 = (k+1)/hop
    for k in [0usize, 100, 383, 700, hop - 1] {
        let want = ((k + 1) as f32 / hop as f32).min(1.0);
        assert!(
            (y[sw + k] - want).abs() < 1e-4,
            "k={k} y={} want={want}",
            y[sw + k]
        );
    }
    assert!(y[sw + hop..].iter().all(|v| *v == 1.0));
    assert!(sw > l); // 切换时原声已经到了
    // 切回去：同样按一个 hop 淡出
    let mut z = vec![0.0; 2000];
    p.set_mode(Mode::Denoise);
    p.process(&x[..2000], &mut z);
    assert!((z[0] - (1.0 - 1.0 / hop as f32)).abs() < 1e-4);
    assert!(z[hop..].iter().all(|v| *v == 0.0));
}

#[test]
fn reset_reproduces_fresh_output() {
    let x = multisine(10_000);
    let make = || {
        Pipeline::new(
            Engine::new(DelayGain::new(256, 256, 0.7), SafetyConfig::default()),
            SR,
        )
        .unwrap()
    };
    let fresh = run(&mut make(), &x, &[x.len()]);
    let mut p = make();
    run(&mut p, &x[..3333], &[3333]);
    p.reset();
    assert_eq!(run(&mut p, &x, &[x.len()]), fresh);
}

#[cfg(debug_assertions)]
#[test]
fn realtime_path_does_not_allocate() {
    // 带故障的模型（NaN、后端报错交替），走 Engine 送原声与 Pipeline 淡化的降级路径
    let plan = (0..400)
        .step_by(7)
        .map(|h| {
            (
                h,
                if h % 2 == 0 {
                    FaultKind::Nan
                } else {
                    FaultKind::Error
                },
            )
        })
        .collect();
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.8), plan);
    let mut p = Pipeline::new(Engine::new(m, SafetyConfig::default()), SR).unwrap();
    // 小块随机（含 1、7 等不是 3 的倍数的块长） + 一次 8192 大块（块长无上限，大块也不能分配）
    let x = multisine(4096 + 8192);
    let mut y = vec![0.0; x.len()];
    let mut lens = chunk_lens(9, 4096, 1000);
    lens.push(8192);
    assert_no_alloc(|| {
        for round in 0..20 {
            let mut i = 0;
            for &n in &lens {
                p.process(&x[i..i + n], &mut y[i..i + n]);
                i += n;
            }
            p.set_mode(if round % 2 == 0 {
                Mode::Passthrough
            } else {
                Mode::Denoise
            });
            let _ = p.latency_samples();
            let _ = p.stats();
        }
        p.reset();
    });
    assert!(p.stats().hops > 0);
}

#[cfg(not(debug_assertions))]
#[test]
fn mismatched_lengths_do_not_panic_in_release() {
    let mut p = identity();
    let mut y = [9.0f32; 10];
    p.process(&[0.5; 7], &mut y);
    assert_eq!(&y[7..], &[0.0; 3]);
    assert_eq!(p.stats().bad_len, 1);
}

#[test]
fn passthrough_blocks_non_finite_model_output() {
    // 保命全关、模型每帧出 NaN：降噪模式会漏 NaN（Engine 不拦），直通模式必须是干净的延迟原声（R11）
    let plan = (0..200).map(|h| (h, FaultKind::Nan)).collect();
    let m = FaultInjector::new(DelayGain::new(256, 256, 1.0), plan);
    let mut p = Pipeline::new(Engine::new(m, SafetyConfig::off()), SR).unwrap();
    p.set_mode(Mode::Passthrough);
    p.reset();
    let x = multisine(20_000);
    let y = run(&mut p, &x, &chunk_lens(8, x.len(), 700));
    let l = p.latency_samples();
    assert_eq!(&y[l..], &x[..x.len() - l]);
    // 淡化途中遇到 NaN 也送原声：切回降噪的第一个 hop 里不出现非有限值
    p.set_mode(Mode::Denoise);
    let mut z = vec![0.0; p.hop_io() - 1];
    p.process(&x[..z.len()], &mut z);
    assert!(z.iter().all(|v| v.is_finite()));
}

#[test]
fn wet_and_dry_are_aligned_through_a_switch() {
    // Identity、保命全关：降噪路 ≈ 原声路，中途切换前后都应 ≈ x[t - L]（没有错位、没有梳状）
    let mut p = identity();
    let l = p.latency_samples();
    let x = multisine(30_000);
    let mut y = vec![0.0; x.len()];
    let (a, b) = (9_001, 17_777);
    p.process(&x[..a], &mut y[..a]);
    p.set_mode(Mode::Passthrough);
    p.process(&x[a..b], &mut y[a..b]);
    p.set_mode(Mode::Denoise);
    p.process(&x[b..], &mut y[b..]);
    let err = (l + 2000..x.len())
        .map(|i| (y[i] - x[i - l]).abs())
        .fold(0.0f32, f32::max);
    assert!(err < 0.01, "切换前后误差 {err}");
}

// ---- 定块长省延迟（人定 2026-11-02）----

/// GTCRN 的形状：模型 hop 256、算法延迟 256（16 kHz）→ 48 kHz hop 768、模型延迟 768。
fn gtcrn_like(gain: f32, safety: SafetyConfig) -> Engine<DelayGain> {
    Engine::new(DelayGain::new(256, 256, gain), safety)
}

#[test]
fn with_block_declared_latency_matches_measured() {
    // GTCRN 形状、块长 96：672 + 96 + 768 = 1536（32.00 ms）
    let mut p = Pipeline::with_block(gtcrn_like(1.0, SafetyConfig::off()), SR, 96).unwrap();
    assert_eq!(p.latency_samples(), 768 - 96 + 96 + 768);
    assert_eq!(p.latency_samples(), 1536);
    let got_ms = p.latency_ledger().total_ms();
    assert!((got_ms - 32.0).abs() < 1e-9, "账 {got_ms} ms");
    let x = multisine(SR as usize);
    let lens = vec![96; x.len() / 96];
    let x = &x[..lens.len() * 96];
    let y = run(&mut p, x, &lens);
    assert_eq!(measured_lag(x, &y, 3000), p.latency_samples());
    let l = p.latency_samples();
    let err = (l + 2000..x.len())
        .map(|i| (y[i] - x[i - l]).abs())
        .fold(0.0f32, f32::max);
    assert!(err < 0.01, "残差 {err}");
    let s = p.stats();
    assert_eq!((s.misaligned_blocks, s.queue_faults), (0, 0));

    // Identity、块长 96：672 + 96 + 0 = 768（16.00 ms）
    let mut p = Pipeline::with_block(Engine::new(Identity, SafetyConfig::off()), SR, 96).unwrap();
    assert_eq!(p.latency_samples(), 768);
    assert!((p.latency_ledger().total_ms() - 16.0).abs() < 1e-9);
    let y = run(&mut p, x, &lens);
    assert_eq!(measured_lag(x, &y, 3000), 768);
}

#[test]
fn with_block_saves_block_minus_one_and_output_is_shifted_new() {
    // 带三件保命的默认配置：定块长输出 = 任意块长版输出提前 95 个样本，逐样本一致（只省等待，不改内容）
    let x = multisine(SR as usize);
    let n = x.len() / 96 * 96;
    let x = &x[..n];
    let lens = vec![96; n / 96];
    let mut fixed = Pipeline::with_block(gtcrn_like(0.5, SafetyConfig::default()), SR, 96).unwrap();
    let a = run(&mut fixed, x, &lens);
    let mut any = Pipeline::new(gtcrn_like(0.5, SafetyConfig::default()), SR).unwrap();
    let b = run(&mut any, x, &lens);
    assert_eq!(any.latency_samples() - fixed.latency_samples(), 95);
    assert_eq!(&a[..n - 95], &b[95..]);
    // 同一块长 96：分多次调用（中途 reset 前后各一段）与一口气按 96 切喂一致
    let mut p = Pipeline::with_block(gtcrn_like(0.5, SafetyConfig::default()), SR, 96).unwrap();
    let mut c = run(&mut p, &x[..96 * 100], &lens[..100]);
    c.extend(run(&mut p, &x[96 * 100..], &lens[100..]));
    assert_eq!(c, a);
    assert_eq!(fixed.stats().misaligned_blocks, 0);
}

#[test]
fn with_block_rejects_unaligned_block() {
    let bad = |b: usize| Pipeline::with_block(gtcrn_like(1.0, SafetyConfig::off()), SR, b).err();
    assert_eq!(
        bad(480),
        Some(PipelineError::BlockNotAligned {
            block: 480,
            hop_io: 768
        })
    );
    assert!(matches!(
        bad(0),
        Some(PipelineError::BlockNotAligned { block: 0, .. })
    ));
    assert!(matches!(
        bad(1536),
        Some(PipelineError::BlockNotAligned { .. })
    ));
    for ok in [1usize, 96, 256, 384, 768] {
        assert!(bad(ok).is_none(), "块长 {ok} 应当可以");
    }
    // 采样率错仍先报采样率
    assert!(matches!(
        Pipeline::with_block(gtcrn_like(1.0, SafetyConfig::off()), 44_100, 96),
        Err(PipelineError::UnsupportedRate { .. })
    ));
}

#[test]
fn with_block_misaligned_input_does_not_panic() {
    let mut p = Pipeline::with_block(gtcrn_like(1.0, SafetyConfig::default()), SR, 96).unwrap();
    let x = multisine(20_000);
    // 先对齐几块，再随机块长（含 1、7、500），再回到 96
    let mut lens = vec![96; 20];
    let mid = chunk_lens(11, 20_000 - 96 * 40, 500);
    let n_mid = mid.iter().filter(|&&n| n != 96).count() as u64;
    lens.extend(mid);
    lens.extend(vec![96; 20]);
    let y = run(&mut p, &x, &lens);
    assert!(y.iter().all(|v| v.is_finite()));
    let s = p.stats();
    assert_eq!(s.misaligned_blocks, n_mid);
    assert!(n_mid > 0);
    // 欠载补零的样本数不超过 block - 1（之后队列水位等同于任意块长版，不再欠载）
    assert!(s.queue_faults <= 95, "欠载 {}", s.queue_faults);
    assert!(y[x.len() - 5000..].iter().any(|v| *v != 0.0));
}

#[cfg(debug_assertions)]
#[test]
fn with_block_realtime_path_does_not_allocate() {
    let plan = (0..400)
        .step_by(5)
        .map(|h| {
            (
                h,
                if h % 2 == 0 {
                    FaultKind::Nan
                } else {
                    FaultKind::Error
                },
            )
        })
        .collect();
    let m = FaultInjector::new(DelayGain::new(256, 256, 0.8), plan);
    let mut p = Pipeline::with_block(Engine::new(m, SafetyConfig::default()), SR, 96).unwrap();
    let x = multisine(96 * 64);
    let mut y = vec![0.0; x.len()];
    assert_no_alloc(|| {
        for round in 0..20 {
            for (xi, yi) in x.chunks(96).zip(y.chunks_mut(96)) {
                p.process(xi, yi);
            }
            // 不对齐块也走一遍（欠载补零路径）
            p.process(&x[..7], &mut y[..7]);
            p.set_mode(if round % 2 == 0 {
                Mode::Passthrough
            } else {
                Mode::Denoise
            });
            let _ = p.latency_samples();
            let _ = p.stats();
        }
        p.reset();
    });
    assert!(p.stats().hops > 0);
    assert_eq!(p.stats().misaligned_blocks, 20);
}
