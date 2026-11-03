//! 起一个线程，提成 time-constraint 后读回参数一致。只改测试线程自己的调度策略，不出声、不弹权限框。

use vdn_rt_thread::{RtError, RtParams, check_current, join_audio_workgroup, promote_current};

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
