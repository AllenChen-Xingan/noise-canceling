//! 音频工作区间：建 · 加入 · 每块开始 / 结束 · 错误路径 · QoS。只改测试线程自己的调度属性，不出声、不弹权限框。
#![cfg(target_os = "macos")]

use vdn_rt_thread::{
    RtError, RtParams, WorkInterval, check_current, promote_current, set_qos_user_interactive,
};

#[test]
fn create_join_begin_finish() {
    std::thread::spawn(|| {
        let wi = WorkInterval::create("vdn-test").expect("AudioWorkIntervalCreate");
        // 官方建议：加入工作区间的线程先提成实时优先级
        promote_current(RtParams::for_block(96, 48_000, 0.5)).unwrap();
        let m = wi.join().expect("os_workgroup_join");
        for _ in 0..100 {
            m.begin(2_000_000).unwrap();
            std::hint::black_box((0..10_000u64).sum::<u64>());
            m.finish().unwrap();
        }
        // 加入工作区间不会撤掉 time-constraint
        assert!(check_current().unwrap().time_constraint);
        drop(m);
        drop(wi);
    })
    .join()
    .unwrap();
}

#[test]
fn errors_are_reported_not_silent() {
    std::thread::spawn(|| {
        let wi = WorkInterval::create("vdn-test-err").unwrap();
        let m = wi.join().unwrap();
        // 没 begin 就 finish
        assert!(matches!(
            m.finish(),
            Err(RtError::Kernel {
                op: "os_workgroup_interval_finish",
                ..
            })
        ));
        // 连着两次 begin
        m.begin(2_000_000).unwrap();
        assert!(matches!(
            m.begin(2_000_000),
            Err(RtError::Kernel {
                op: "os_workgroup_interval_start",
                ..
            })
        ));
        m.finish().unwrap();
        // 同一线程再加入一次（已是成员）
        assert!(matches!(
            wi.join(),
            Err(RtError::Kernel {
                op: "os_workgroup_join",
                ..
            })
        ));
    })
    .join()
    .unwrap();
}

#[test]
fn non_member_thread_cannot_begin() {
    let wi = WorkInterval::create("vdn-test-nm").unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            let m = wi.join().unwrap();
            m.begin(1_000_000).unwrap();
            m.finish().unwrap();
        });
    });
    // 本线程从没加入：拿不到 Member，类型上就调不了 begin。这里只确认 wi 还能再被别的线程加入
    std::thread::scope(|s| {
        s.spawn(|| assert!(wi.join().is_ok()));
    });
}

#[test]
fn qos_user_interactive() {
    std::thread::spawn(|| set_qos_user_interactive().unwrap())
        .join()
        .unwrap();
}
