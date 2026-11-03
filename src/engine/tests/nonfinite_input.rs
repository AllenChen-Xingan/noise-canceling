//! 输入里的 NaN / ±Inf（issue #53）：`Engine::process`、`Engine::process_status`、`Pipeline::process` 三条路径。
//!
//! 规则（唯一正本 `vdn_engine::engine::sanitize_input`）：非有限输入样本按 0 处理并计数，模型、原声延迟线、接缝都只见有限值。
//! 所以「注入坏样本」的输出应**逐样本等于**「把这些样本换成 0」的输出——既证明输出有限，也证明注入结束后恢复成正常处理结果
//! （不是永远原声、也不是永远 0）。

#[cfg(debug_assertions)]
use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_engine::testing::DelayGain;
use vdn_engine::{Engine, Pipeline, SafetyConfig, Status, offline};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

const HOP: usize = 256;
const LAT: usize = 256;
const SR_IO: u32 = 48_000;

fn tone(n: usize) -> Vec<f32> {
    (0..n).map(|i| 0.3 * (i as f32 * 0.05).sin()).collect()
}

fn configs() -> [(&'static str, SafetyConfig); 3] {
    [
        ("default", SafetyConfig::default()),
        (
            "fallback_only",
            SafetyConfig {
                fallback: true,
                ..SafetyConfig::off()
            },
        ),
        ("off", SafetyConfig::off()),
    ]
}

/// 单个 NaN、跨 hop 的一段 +Inf、单个 -Inf。返回 (注入后的输入, 坏样本换成 0 的输入, 坏样本数)。
fn inject(x: &[f32]) -> (Vec<f32>, Vec<f32>, u64) {
    let mut bad = x.to_vec();
    let mut idx: Vec<usize> = vec![10 * HOP + 5];
    idx.extend(20 * HOP + 30..20 * HOP + 30 + 300);
    idx.push(30 * HOP + HOP - 1);
    for (j, &t) in idx.iter().enumerate() {
        bad[t] = match j {
            0 => f32::NAN,
            _ if t == 30 * HOP + HOP - 1 => f32::NEG_INFINITY,
            _ => f32::INFINITY,
        };
    }
    let mut zeroed = x.to_vec();
    for &t in &idx {
        zeroed[t] = 0.0;
    }
    (bad, zeroed, idx.len() as u64)
}

fn assert_same(tag: &str, y: &[f32], want: &[f32]) {
    assert!(y.iter().all(|v| v.is_finite()), "{tag}：输出出现 NaN / Inf");
    for (t, (a, b)) in y.iter().zip(want).enumerate() {
        assert!(
            a.to_bits() == b.to_bits(),
            "{tag}：样本 {t} 是 {a}，应为 {b}"
        );
    }
}

/// 逐 hop 喂 `process_status`，返回输出与每 hop 状态。
fn run_status(e: &mut Engine<DelayGain>, x: &[f32]) -> (Vec<f32>, Vec<Status>) {
    let mut y = vec![0.0; x.len()];
    let mut st = Vec::new();
    for (xi, yi) in x
        .as_chunks::<HOP>()
        .0
        .iter()
        .zip(y.as_chunks_mut::<HOP>().0)
    {
        st.push(e.process_status(xi, yi));
    }
    (y, st)
}

#[test]
fn engine_process_nonfinite_input_is_zeroed_and_recovers() {
    let x = tone(HOP * 60);
    let (bad, zeroed, n_bad) = inject(&x);
    for (tag, cfg) in configs() {
        let mut e = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg.clone());
        let y = offline::run(&mut e, &bad, 0).output;
        let mut r = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg);
        let want = offline::run(&mut r, &zeroed, 0).output;
        assert_same(tag, &y, &want);
        let s = e.stats();
        assert_eq!(s.bad_input, n_bad, "{tag}");
        // 输入坏值不是模型故障：不复位模型、不送原声
        assert_eq!((s.fault_nonfinite, s.model_resets), (0, 0), "{tag}");
        assert_eq!(s.bypass_hops, r.stats().bypass_hops, "{tag}");
    }
}

#[test]
fn engine_process_status_nonfinite_input_is_zeroed_and_recovers() {
    let x = tone(HOP * 60);
    let (bad, zeroed, n_bad) = inject(&x);
    for (tag, cfg) in configs() {
        let mut e = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg.clone());
        let (y, st) = run_status(&mut e, &bad);
        let mut r = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg);
        let (want, st_want) = run_status(&mut r, &zeroed);
        assert_same(tag, &y, &want);
        assert_eq!(st, st_want, "{tag}");
        assert!(st.iter().all(|s| *s == Status::Ok), "{tag}：{st:?}");
        assert_eq!(e.stats().bad_input, n_bad, "{tag}");
        // aligned_dry 给后处理级用，也必须有限
        assert!(e.aligned_dry().iter().all(|v| v.is_finite()), "{tag}");
    }
}

#[test]
fn pipeline_nonfinite_input_is_zeroed_and_recovers() {
    let hop_io = HOP * 3;
    let x: Vec<f32> = (0..hop_io * 40)
        .map(|i| 0.3 * (i as f32 * 0.02).sin())
        .collect();
    let mut bad = x.clone();
    let mut zeroed = x.clone();
    let mut idx = vec![5 * hop_io + 7];
    idx.extend(12 * hop_io + 100..12 * hop_io + 100 + 900);
    idx.push(20 * hop_io);
    for (j, &t) in idx.iter().enumerate() {
        bad[t] = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY][j % 3];
        zeroed[t] = 0.0;
    }
    for (tag, cfg) in configs() {
        let mut p = Pipeline::new(
            Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg.clone()),
            SR_IO,
        )
        .unwrap();
        let mut r = Pipeline::new(Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg), SR_IO).unwrap();
        let mut y = vec![0.0; x.len()];
        let mut want = vec![0.0; x.len()];
        for ((xi, yi), (zi, wi)) in bad
            .chunks(96)
            .zip(y.chunks_mut(96))
            .zip(zeroed.chunks(96).zip(want.chunks_mut(96)))
        {
            p.process(xi, yi);
            r.process(zi, wi);
        }
        assert_same(tag, &y, &want);
        assert_eq!(p.stats().bad_input, idx.len() as u64, "{tag}");
        // 48 kHz 入口已清 0，降采样后 Engine 不应再见到坏值
        assert_eq!(p.engine().stats().bad_input, 0, "{tag}");
    }
}

#[test]
fn sustained_nonfinite_input_stays_finite_then_recovers() {
    // 前 50 个 hop 全是 NaN / Inf，之后 30 个 hop 正常：全程有限、不 panic；之后等于「前段是 0」的处理结果
    for fill in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut x = vec![fill; HOP * 50];
        x.extend(tone(HOP * 30));
        let mut zeroed = vec![0.0; HOP * 50];
        zeroed.extend(tone(HOP * 30));
        for (tag, cfg) in configs() {
            let mut e = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg.clone());
            let y = offline::run(&mut e, &x, 0).output;
            let mut r = Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg.clone());
            let want = offline::run(&mut r, &zeroed, 0).output;
            assert_same(tag, &y, &want);
            assert_eq!(e.stats().bad_input, (HOP * 50) as u64, "{tag}");

            let xs: Vec<f32> = x.iter().flat_map(|&v| [v; 3]).collect();
            let zs: Vec<f32> = zeroed.iter().flat_map(|&v| [v; 3]).collect();
            let mut p = Pipeline::new(
                Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg.clone()),
                SR_IO,
            )
            .unwrap();
            let mut q =
                Pipeline::new(Engine::new(DelayGain::new(HOP, LAT, 0.5), cfg), SR_IO).unwrap();
            let mut yp = vec![0.0; xs.len()];
            let mut wp = vec![0.0; xs.len()];
            p.process(&xs, &mut yp);
            q.process(&zs, &mut wp);
            assert_same(tag, &yp, &wp);
            // 恢复后不是全 0：最后 10 个 hop 有声音
            let tail = &yp[yp.len() - 10 * HOP * 3..];
            assert!(tail.iter().any(|v| v.abs() > 0.05), "{tag}：尾段全静音");
        }
    }
}

#[cfg(debug_assertions)]
#[test]
fn nonfinite_input_path_does_not_allocate() {
    let mut x = tone(HOP);
    x[3] = f32::NAN;
    x[100] = f32::INFINITY;
    let mut y = vec![0.0; HOP];
    let mut e = Engine::new(DelayGain::new(HOP, LAT, 0.5), SafetyConfig::default());
    let mut p = Pipeline::new(
        Engine::new(DelayGain::new(HOP, LAT, 0.5), SafetyConfig::default()),
        SR_IO,
    )
    .unwrap();
    assert_no_alloc(|| {
        for _ in 0..20 {
            e.process(&x, &mut y);
            let _ = e.process_status(&x, &mut y);
            p.process(&x, &mut y);
        }
    });
    assert!(y.iter().all(|v| v.is_finite()));
}

#[test]
fn sanitize_input_zeroes_only_nonfinite() {
    let mut v = [
        1.0,
        f32::NAN,
        -0.5,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::MIN_POSITIVE,
    ];
    assert_eq!(vdn_engine::sanitize_input(&mut v), 3);
    assert_eq!(v, [1.0, 0.0, -0.5, 0.0, 0.0, f32::MIN_POSITIVE]);
}
