//! 系统不支持唤醒（低于 macOS 14.4）时：`wait` 退回 200 µs 轮询，照样不丢数据、能超时，并计 `wait_fallbacks`。
//! 单独一个测试二进制：`force_wake_unsupported_for_test` 对全进程生效。

use std::time::{Duration, Instant};
use vdn_shm::{Bus, BusConfig, RingId, WaitResult};

#[test]
fn falls_back_to_polling_and_counts() {
    Bus::force_wake_unsupported_for_test();
    assert!(!Bus::wake_supported());
    let b = Bus::heap(BusConfig {
        rate: 48_000,
        capacity_frames: 1024,
        l_total: 0,
        lead: 0,
        denoise_block: 0,
    })
    .unwrap();
    let t = Instant::now();
    assert_eq!(
        b.wait(RingId::Raw, 1, Duration::from_millis(10)),
        WaitResult::TimedOut { available: 0 }
    );
    assert!(t.elapsed() >= Duration::from_millis(10));
    assert_eq!(
        b.ring(RingId::Raw).wait_fallbacks(),
        1,
        "每次 wait 最多记 1"
    );
    std::thread::scope(|s| {
        s.spawn(|| {
            std::thread::sleep(Duration::from_millis(5));
            b.ring(RingId::Raw).push(&[1.0; 96]);
            b.notify(RingId::Raw); // 不支持时只做原子加，不出错
        });
        let r = b.wait(RingId::Raw, 96, Duration::from_secs(2));
        assert_eq!(r, WaitResult::Ready { available: 96 });
    });
    assert_eq!(b.ring(RingId::Raw).wait_fallbacks(), 2);
    // 数据已够时不算退回
    let _ = b.wait(RingId::Raw, 96, Duration::from_secs(1));
    assert_eq!(b.ring(RingId::Raw).wait_fallbacks(), 2);
}
