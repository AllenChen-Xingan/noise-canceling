//! 起一个线程，提成 time-constraint 后读回参数一致。只改测试线程自己的调度策略，不出声、不弹权限框。

use vdn_rt_thread::{
    MAX_COMPUTATION_NS, MIN_COMPUTATION_NS, RtError, RtParams, check_current, join_audio_workgroup,
    promote_current,
};

#[test]
fn invalid_params_are_rejected() {
    let p = |period_ns, computation_ns, constraint_ns| RtParams {
        period_ns,
        computation_ns,
        constraint_ns,
    };
    assert!(matches!(
        promote_current(p(10_000_000, 0, 10_000_000)),
        Err(RtError::InvalidParams(_))
    ));
    assert!(matches!(
        promote_current(p(10_000_000, 6_000_000, 5_000_000)),
        Err(RtError::InvalidParams(_))
    ));
    assert!(matches!(
        promote_current(p(4_000_000, 1_000_000, 5_000_000)),
        Err(RtError::InvalidParams(_))
    ));
    // 内核下限 / 上限之外：先在本 crate 拦下（不到内核那里变成 code 4）
    assert!(matches!(
        promote_current(p(16_000_000, MIN_COMPUTATION_NS - 1, 16_000_000)),
        Err(RtError::InvalidParams(_))
    ));
    assert!(matches!(
        promote_current(p(0, MAX_COMPUTATION_NS + 1, 100_000_000)),
        Err(RtError::InvalidParams(_))
    ));
}

#[test]
fn for_block_math() {
    let p = RtParams::for_block(480, 48_000, 0.5);
    assert_eq!(p.period_ns, 10_000_000);
    assert_eq!(p.constraint_ns, 10_000_000);
    assert_eq!(p.computation_ns, 5_000_000);
}

#[test]
fn workgroup_is_unimplemented() {
    assert!(matches!(
        join_audio_workgroup(),
        Err(RtError::Unimplemented(_))
    ));
}

#[cfg(target_os = "macos")]
#[test]
fn promote_then_read_back() {
    let want = RtParams::for_block(512, 48_000, 0.5);
    let got = std::thread::spawn(move || {
        let before = check_current().unwrap();
        promote_current(want).unwrap();
        (before, check_current().unwrap())
    })
    .join()
    .unwrap();
    let (before, after) = got;
    assert!(!before.time_constraint, "新线程默认不是实时策略");
    assert!(after.time_constraint);
    assert!(after.preemptible);
    let p = after.params.unwrap();
    // ns → Mach 单位 → ns 有取整；Apple Silicon 一个单位约 41.7 ns，放 100 ns
    let close = |a: u64, b: u64| a.abs_diff(b) <= 100;
    assert!(close(p.period_ns, want.period_ns), "{p:?} vs {want:?}");
    assert!(
        close(p.computation_ns, want.computation_ns),
        "{p:?} vs {want:?}"
    );
    assert!(
        close(p.constraint_ns, want.constraint_ns),
        "{p:?} vs {want:?}"
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn unsupported_off_macos() {
    assert_eq!(check_current().err(), Some(RtError::Unsupported));
}

/// 实时优先级对醒来延迟的影响（rs-wake 对照实验的结论落成测试：消费者不提优先级时有几毫秒的尖峰，提了之后最长几十微秒）。
/// 后台起（核数 − 2）个空转线程，再分别让「普通线程」和「time-constraint 线程」每 2 ms 睡醒 1500 次，比较最长迟到。
/// 结果随机器负载变化，不进 CI。人工步骤：
/// `cargo test --release -p vdn-rt-thread --test promote -- --ignored --nocapture`，看打印的两行数字，测试过 = 实时线程最长迟到 < 1 ms 且小于普通线程。
#[cfg(target_os = "macos")]
#[test]
#[ignore = "人工跑：依赖机器负载、要占满多个核约 6 s，步骤见注释"]
fn promotion_cuts_wakeup_lateness() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};
    static STOP: AtomicBool = AtomicBool::new(false);
    let n = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .saturating_sub(2)
        .max(1);
    let load: Vec<_> = (0..n)
        .map(|_| {
            std::thread::spawn(|| {
                let mut x = 1u64;
                while !STOP.load(Ordering::Relaxed) {
                    x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(1));
                }
            })
        })
        .collect();
    let measure = |rt: bool| {
        std::thread::spawn(move || {
            if rt {
                promote_current(RtParams::for_block(96, 48_000, 0.25)).unwrap();
            }
            let period = Duration::from_millis(2);
            let start = Instant::now();
            let mut worst = Duration::ZERO;
            for i in 1..=1500u32 {
                let due = start + period * i;
                if let Some(d) = due.checked_duration_since(Instant::now()) {
                    std::thread::sleep(d);
                }
                worst = worst.max(Instant::now().saturating_duration_since(due));
            }
            worst
        })
        .join()
        .unwrap()
    };
    let normal = measure(false);
    let rt = measure(true);
    STOP.store(true, Ordering::Relaxed);
    for h in load {
        h.join().unwrap();
    }
    println!("普通线程最长迟到 {normal:?}；实时线程最长迟到 {rt:?}（后台 {n} 个空转线程）");
    assert!(rt < Duration::from_millis(1), "实时线程最长迟到 {rt:?}");
    assert!(rt < normal, "实时 {rt:?} 不比普通 {normal:?} 好");
}

#[test]
fn default_policy_follows_d68() {
    let p = vdn_rt_thread::ThreadPolicy::default();
    assert!(p.time_constraint, "D68：实时优先级默认开");
    assert!(!p.work_interval, "D68：工作区间默认不加入");
}

#[test]
fn from_measured_adds_margin_and_clamps() {
    use vdn_rt_thread::MEASURED_MARGIN;
    // GTCRN：周期报 16 ms；每 16 ms 窗口 = 1 块推理（约 3 ms）+ 7 块轻处理（各约 0.13 ms），总工作量 p99.9 约 4 ms
    // → computation = 6 ms（不是单块推理的 3.05 ms × 1.5）
    let p = RtParams::from_measured(16_000_000, 4_000_000, MEASURED_MARGIN);
    assert_eq!(p.period_ns, 16_000_000);
    assert_eq!(p.constraint_ns, 16_000_000);
    assert_eq!(p.computation_ns, 6_000_000);
    // 余量小于 1 按 1 算；乘出来超过周期就夹到周期
    assert_eq!(
        RtParams::from_measured(2_000_000, 800_000, 0.5).computation_ns,
        800_000
    );
    assert_eq!(
        RtParams::from_measured(2_000_000, 1_500_000, 2.0).computation_ns,
        2_000_000
    );
    // 实测 0 也给出内核接受的参数（computation 抬到 50 µs 下限）
    let z = RtParams::from_measured(2_000_000, 0, MEASURED_MARGIN);
    assert_eq!(z.computation_ns, MIN_COMPUTATION_NS);
    assert!(z.validate().is_ok());
    #[cfg(target_os = "macos")]
    std::thread::spawn(move || promote_current(p).unwrap())
        .join()
        .unwrap();
}

/// 回归（2026-11-02 `vdn run --passthrough` 报 code 4）：直通管线预热每周期只有约 20 µs，
/// × 1.5 = 30 µs < 内核下限 50 µs。修前 `promote_current` 返回 `Kernel { code: 4 }`
#[test]
fn tiny_measurement_gives_params_the_kernel_accepts() {
    use vdn_rt_thread::MEASURED_MARGIN;
    let p = RtParams::from_measured(16_000_000, 20_000, MEASURED_MARGIN);
    assert_eq!(p.computation_ns, MIN_COMPUTATION_NS);
    assert!(p.validate().is_ok(), "{p:?}");
    #[cfg(target_os = "macos")]
    {
        let r = std::thread::spawn(move || promote_current(p))
            .join()
            .unwrap();
        assert_eq!(r, Ok(()), "{p:?}");
    }
}

/// 实测极大（比周期和内核上限 50 ms 都大）：夹到 min(50 ms, 周期)，内核接受
#[test]
fn huge_measurement_is_clamped_to_kernel_max_and_period() {
    // 周期 100 ms，实测 80 ms × 1.5 = 120 ms → 内核上限 50 ms
    let p = RtParams::from_measured(100_000_000, 80_000_000, 1.5);
    assert_eq!(p.computation_ns, MAX_COMPUTATION_NS);
    assert!(p.validate().is_ok(), "{p:?}");
    // 周期 16 ms，实测 30 ms → 夹到周期
    let q = RtParams::from_measured(16_000_000, 30_000_000, 1.5);
    assert_eq!(q.computation_ns, 16_000_000);
    assert!(q.validate().is_ok(), "{q:?}");
    #[cfg(target_os = "macos")]
    for x in [p, q] {
        let r = std::thread::spawn(move || promote_current(x))
            .join()
            .unwrap();
        assert_eq!(r, Ok(()), "{x:?}");
    }
}

/// `for_block` 负载极小也给出合法值（chain-test 的送音 / 读出线程走它）
#[test]
fn for_block_with_tiny_load_is_valid() {
    let p = RtParams::for_block(96, 48_000, 0.001);
    assert_eq!(p.computation_ns, MIN_COMPUTATION_NS);
    assert!(p.validate().is_ok());
    assert_eq!(
        RtParams::for_block(96, 48_000, 0.0).computation_ns,
        MIN_COMPUTATION_NS
    );
}

/// 夹取不改周期：周期比内核下限还短时不悄悄改，交给校验报错
#[test]
fn period_shorter_than_kernel_min_is_rejected_not_rewritten() {
    let p = RtParams::from_measured(10_000, 5_000, 1.5);
    assert_eq!((p.period_ns, p.constraint_ns), (10_000, 10_000));
    assert!(matches!(p.validate(), Err(RtError::InvalidParams(_))));
    assert!(matches!(promote_current(p), Err(RtError::InvalidParams(_))));
}

/// 读回：内核会把小于 constraint / 2 的 computation 抬到 constraint / 2（xnu thread_policy.c，读源码）。
/// 这条测试把这个行为钉住，免得以后读回的数字被误判为「被降级」
#[cfg(target_os = "macos")]
#[test]
fn kernel_raises_small_computation_to_half_constraint() {
    let want = RtParams::from_measured(16_000_000, 1_000_000, 1.5); // computation 1.5 ms < 8 ms
    let st = std::thread::spawn(move || {
        promote_current(want).unwrap();
        check_current().unwrap()
    })
    .join()
    .unwrap();
    let got = st.params.unwrap();
    println!("给 {want:?}，读回 {got:?}");
    let close = |a: u64, b: u64| a.abs_diff(b) <= 100;
    assert!(
        close(got.computation_ns, want.computation_ns) || close(got.computation_ns, 8_000_000),
        "读回 computation 既不是给的值也不是 constraint/2：{got:?}"
    );
}
