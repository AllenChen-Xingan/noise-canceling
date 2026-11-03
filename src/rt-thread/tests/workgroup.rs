//! 音频工作区间：建 · 加入 · 每块开始 / 结束 · 错误路径 · QoS。只改测试线程自己的调度属性，不出声、不弹权限框。
#![cfg(target_os = "macos")]

use vdn_rt_thread::{
    RtError, RtParams, WorkInterval, check_current, promote_current, set_qos_user_interactive,
};

#[test]
fn create_join_begin_finish() {
    std::thread::spawn(|| {
        let wi = WorkInterval::create("vdn-test").expect("AudioWorkIntervalCreate");
        // AudioWorkInterval.h 26–27 行：有实时优先级的辅助线程应加入工作组（建议，不是加入的前提）
        promote_current(RtParams::for_block(96, vdn_types::IO_RATE, 0.5)).unwrap();
        wi.with_member(|m| {
            for _ in 0..100 {
                m.begin(2_000_000).unwrap();
                std::hint::black_box((0..10_000u64).sum::<u64>());
                m.finish().unwrap();
            }
            // 加入工作区间不会撤掉 time-constraint
            assert!(check_current().unwrap().time_constraint);
        })
        .expect("os_workgroup_join");
    })
    .join()
    .unwrap();
}

#[test]
fn errors_are_reported_not_silent() {
    std::thread::spawn(|| {
        let wi = WorkInterval::create("vdn-test-err").unwrap();
        wi.with_member(|m| {
            // 没 begin 就 finish（workgroup_interval.h 写明的错误）
            assert!(matches!(
                m.finish(),
                Err(RtError::Kernel {
                    op: "os_workgroup_interval_finish",
                    ..
                })
            ));
            // 连着两次 begin（workgroup_interval.h 写明的错误）
            m.begin(2_000_000).unwrap();
            assert!(matches!(
                m.begin(2_000_000),
                Err(RtError::Kernel {
                    op: "os_workgroup_interval_start",
                    ..
                })
            ));
            m.finish().unwrap();
            // 同一线程再加入同一个工作区间：本机实测报错（头文件没写这一条）
            assert!(matches!(
                wi.with_member(|_| ()),
                Err(RtError::Kernel {
                    op: "os_workgroup_join",
                    ..
                })
            ));
        })
        .unwrap();
    })
    .join()
    .unwrap();
}

#[test]
fn bad_name_is_rejected() {
    assert!(matches!(
        WorkInterval::create("a\0b"),
        Err(RtError::InvalidParams(_))
    ));
}

#[test]
fn nested_intervals_leave_in_reverse_order() {
    // 两个工作区间嵌套加入：闭包作用域保证先离开里层（头文件要求逆序离开）。
    // 若系统不许同时在两个工作组里（EALREADY），里层返回错误，外层不受影响
    std::thread::spawn(|| {
        let a = WorkInterval::create("vdn-test-a").unwrap();
        let b = WorkInterval::create("vdn-test-b").unwrap();
        a.with_member(|ma| {
            let inner = b.with_member(|mb| {
                mb.begin(1_000_000).unwrap();
                mb.finish().unwrap();
            });
            assert!(
                inner.is_ok()
                    || matches!(
                        inner,
                        Err(RtError::Kernel {
                            op: "os_workgroup_join",
                            ..
                        })
                    )
            );
            ma.begin(1_000_000).unwrap();
            ma.finish().unwrap();
        })
        .unwrap();
        // 离开后可以再加入
        a.with_member(|_| ()).unwrap();
    })
    .join()
    .unwrap();
}

#[test]
fn shared_across_threads() {
    // WorkInterval 是 Send + Sync：几条线程各自加入同一个工作区间
    let wi = WorkInterval::create("vdn-test-shared").unwrap();
    std::thread::scope(|s| {
        for _ in 0..3 {
            s.spawn(|| {
                wi.with_member(|m| m.begin(1_000_000).is_ok() && m.finish().is_ok())
                    .unwrap()
            });
        }
    });
}

#[test]
fn qos_user_interactive() {
    std::thread::spawn(|| set_qos_user_interactive().unwrap())
        .join()
        .unwrap();
}
