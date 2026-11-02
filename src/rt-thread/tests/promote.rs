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
