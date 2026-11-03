//! 唤醒：`notify` / `wait`（heap 版与真 POSIX 共享内存两份映射）· 新头部字段 · `OwnedBus`。
#![cfg(not(feature = "loom"))] // loom 的原子量只能在 loom::model 里用；loom 模式只跑 tests/loom.rs

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use vdn_shm::{Bus, BusConfig, RingId, ShmError, WaitResult, WaitStrategy};

const CFG: BusConfig = BusConfig {
    rate: vdn_types::IO_RATE,
    capacity_frames: 4096,
    l_total: 960,
    lead: 144,
    denoise_block: 96,
};

#[cfg(all(unix, feature = "posix"))]
fn unique_name() -> String {
    static N: AtomicU32 = AtomicU32::new(0);
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let n = N.fetch_add(1, Ordering::Relaxed);
    format!("/vdn-w{:x}-{n}-{:x}", std::process::id(), t & 0xffff)
}

#[cfg(all(unix, feature = "posix"))]
struct Cleanup(String);
#[cfg(all(unix, feature = "posix"))]
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = Bus::unlink_name(&self.0);
    }
}

#[test]
fn ready_immediately_when_data_is_there() {
    let b = Bus::heap(CFG).unwrap();
    b.ring(RingId::Raw).push(&[0.0; 96]);
    let t = Instant::now();
    assert_eq!(
        b.wait(RingId::Raw, 96, Duration::from_secs(5)),
        WaitResult::Ready { available: 96 }
    );
    assert!(t.elapsed() < Duration::from_millis(50));
    assert!(
        b.wait(RingId::Raw, 0, Duration::ZERO).is_ready(),
        "要 0 个立刻好"
    );
}

#[test]
#[cfg_attr(miri, ignore = "依赖真实时钟的超时上限，Miri 的虚拟时钟下不准")]
fn times_out_and_returns() {
    let b = Bus::heap(CFG).unwrap();
    for (strategy, timeout) in [
        (WaitStrategy::Wake, Duration::from_millis(20)),
        (
            WaitStrategy::Poll(Duration::from_micros(500)),
            Duration::from_millis(20),
        ),
        (WaitStrategy::Wake, Duration::ZERO),
    ] {
        let mut b2 = Bus::heap(CFG).unwrap();
        b2.set_wait_strategy(strategy);
        let t = Instant::now();
        let r = b2.wait(RingId::Raw, 1, timeout);
        let el = t.elapsed();
        assert_eq!(r, WaitResult::TimedOut { available: 0 }, "{strategy:?}");
        assert!(el >= timeout, "{strategy:?} 早退 {el:?}");
        assert!(
            el < timeout + Duration::from_millis(200),
            "{strategy:?} 晚退 {el:?}"
        );
    }
    // 别的 notify（数据不够）叫醒后接着等，最后仍按超时返回
    b.ring(RingId::Raw).push(&[0.0; 10]);
    std::thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..20 {
                b.notify(RingId::Raw);
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        let r = b.wait(RingId::Raw, 96, Duration::from_millis(40));
        assert_eq!(r, WaitResult::TimedOut { available: 10 });
    });
    if Bus::wake_supported() {
        assert_eq!(b.ring(RingId::Raw).wait_fallbacks(), 0);
    }
}

#[test]
fn min_above_capacity_is_clamped() {
    let b = Bus::heap(BusConfig {
        capacity_frames: 8,
        lead: 0,
        denoise_block: 0,
        ..CFG
    })
    .unwrap();
    b.ring(RingId::Raw).push(&[0.0; 8]);
    assert!(
        b.wait(RingId::Raw, 1000, Duration::from_millis(10))
            .is_ready()
    );
}

/// 生产方每块 push + notify，消费方每块 wait 再 pop；任何一次唤醒丢了，那次 wait 就会等满 1 s 超时。
fn ping_pong(producer: &Bus, consumer: &Bus, blocks: u32) {
    const BLOCK: usize = 96;
    let max_wait = std::sync::Mutex::new(Duration::ZERO);
    std::thread::scope(|s| {
        s.spawn(|| {
            let r = producer.ring(RingId::Raw);
            let mut buf = [0.0f32; BLOCK];
            for i in 0..blocks {
                buf[0] = i as f32;
                while r.free_space() < BLOCK {
                    std::thread::yield_now();
                }
                assert_eq!(r.push(&buf), BLOCK);
                producer.notify(RingId::Raw);
                if i % 7 == 0 {
                    std::thread::sleep(Duration::from_micros(50)); // 让消费方真睡下去
                }
            }
        });
        let r = consumer.ring(RingId::Raw);
        let mut buf = [0.0f32; BLOCK];
        for i in 0..blocks {
            let t = Instant::now();
            let w = consumer.wait(RingId::Raw, BLOCK, Duration::from_secs(1));
            let el = t.elapsed();
            assert!(w.is_ready(), "第 {i} 块唤醒丢了：{w:?}");
            let mut m = max_wait.lock().unwrap();
            *m = (*m).max(el);
            assert_eq!(r.pop(&mut buf), BLOCK);
            assert_eq!(buf[0], i as f32);
        }
    });
    let m = *max_wait.lock().unwrap();
    assert!(m < Duration::from_millis(500), "最长一次等了 {m:?}");
    if Bus::wake_supported() {
        assert_eq!(consumer.ring(RingId::Raw).wait_fallbacks(), 0);
    }
}

#[test]
fn no_lost_wakeups_heap_two_threads() {
    let b = Bus::heap(CFG).unwrap();
    // Miri 很慢：只跑 200 块（Miri 下没有唤醒，走 200 µs 轮询那一支）
    ping_pong(&b, &b, if cfg!(miri) { 200 } else { 20_000 });
}

#[cfg(all(unix, feature = "posix"))]
#[test]
#[cfg_attr(miri, ignore = "Miri 不支持 shm_open / mmap")]
fn no_lost_wakeups_posix_two_mappings() {
    let name = unique_name();
    let _c = Cleanup(name.clone());
    let p = Bus::create(&name, CFG).unwrap();
    let c = Bus::open(&name).unwrap();
    ping_pong(&p, &c, 20_000);
}

/// 一来一回锁步：生产方推一块 + notify，然后等消费方在 Processed 环上回一个应答，才推下一块。
/// 每一次 wait 前对方都还没推，所以消费方一定真睡下去；任何一次唤醒丢了，那次 wait 就要等满 1 s 超时。
/// 注意：两次读之间的窗口很窄，「读序写反」这类错本测试未必抓得到（反向验证时没抓到）；
/// 那一类由 `src/wait.rs` 里撑大窗口的单元测试确定地抓。
fn lockstep(a: &Bus, b: &Bus, rounds: u32) {
    let slow = AtomicU32::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut x = [0.0f32; 96];
            for i in 0..rounds {
                x[0] = i as f32;
                assert_eq!(a.push_notify(RingId::Raw, &x), 96);
                let t = Instant::now();
                let w = a.wait(RingId::Processed, 1, Duration::from_secs(1));
                assert!(w.is_ready(), "第 {i} 轮应答的唤醒丢了");
                if t.elapsed() > Duration::from_millis(100) {
                    slow.fetch_add(1, Ordering::Relaxed);
                }
                let mut ack = [0.0f32; 1];
                assert_eq!(a.ring(RingId::Processed).pop(&mut ack), 1);
                assert_eq!(ack[0], i as f32);
            }
        });
        let mut x = [0.0f32; 96];
        for i in 0..rounds {
            let t = Instant::now();
            let w = b.wait(RingId::Raw, 96, Duration::from_secs(1));
            assert!(w.is_ready(), "第 {i} 块的唤醒丢了");
            if t.elapsed() > Duration::from_millis(100) {
                slow.fetch_add(1, Ordering::Relaxed);
            }
            assert_eq!(b.ring(RingId::Raw).pop(&mut x), 96);
            assert_eq!(x[0], i as f32);
            b.push_notify(RingId::Processed, &[i as f32]);
        }
    });
    // 偶发调度慢（机器忙）可以有几次；丢唤醒会是 1 s 一次，上面已断言为 Ready
    // Miri 下慢几百倍，计时没意义：只查「没有一次等满 1 s」（上面的 is_ready）
    if !cfg!(miri) {
        assert!(slow.load(Ordering::Relaxed) <= rounds / 100, "慢的轮数太多");
    }
}

#[test]
fn lockstep_heap() {
    let b = Bus::heap(CFG).unwrap();
    lockstep(&b, &b, if cfg!(miri) { 50 } else { 5_000 });
}

#[cfg(all(unix, feature = "posix"))]
#[test]
#[cfg_attr(miri, ignore = "Miri 不支持 shm_open / mmap")]
fn lockstep_posix_two_mappings() {
    let name = unique_name();
    let _c = Cleanup(name.clone());
    let p = Bus::create(&name, CFG).unwrap();
    let c = Bus::open(&name).unwrap();
    lockstep(&p, &c, 5_000);
}

#[test]
#[cfg_attr(miri, ignore = "依赖真实时钟的超时上限，Miri 的虚拟时钟下不准")]
fn poll_zero_interval_still_times_out() {
    let mut b = Bus::heap(CFG).unwrap();
    b.set_wait_strategy(WaitStrategy::Poll(Duration::ZERO));
    let t = Instant::now();
    assert!(!b.wait(RingId::Raw, 1, Duration::from_millis(20)).is_ready());
    let el = t.elapsed();
    assert!(
        el >= Duration::from_millis(20) && el < Duration::from_millis(220),
        "{el:?}"
    );
}

#[cfg(all(unix, feature = "posix"))]
#[test]
#[cfg_attr(miri, ignore = "Miri 不支持 shm_open / mmap")]
fn header_v2_fields_and_session() {
    let name = unique_name();
    let _c = Cleanup(name.clone());
    let a = Bus::create(&name, CFG).unwrap();
    let b = Bus::open(&name).unwrap();
    assert_eq!(b.config(), CFG);
    assert_ne!(a.session_id(), 0);
    assert_eq!(a.session_id(), b.session_id());
    drop(b);
    a.unlink().unwrap();
    let a2 = Bus::create(&name, CFG).unwrap();
    assert_ne!(a2.session_id(), a.session_id(), "重建后会话 ID 不同");
    // 缺口计数（两份映射看到同一个）
    let b2 = Bus::open(&name).unwrap();
    a2.ring(RingId::Processed).note_gap(480);
    assert_eq!(b2.ring(RingId::Processed).gaps(), 480);
    assert_eq!(b2.ring(RingId::Processed).take_gaps(), 480);
    assert_eq!(a2.ring(RingId::Processed).gaps(), 0);
    assert_eq!(
        a2.ring(RingId::Processed).underruns(),
        0,
        "缺口与欠载分开计"
    );
}

#[test]
fn bad_lead_or_block_is_rejected() {
    for cfg in [
        BusConfig { lead: 4097, ..CFG },
        BusConfig {
            denoise_block: 4097,
            ..CFG
        },
    ] {
        assert!(matches!(
            Bus::heap(cfg).err(),
            Some(ShmError::InvalidConfig(_))
        ));
    }
}

#[cfg(all(unix, feature = "posix"))]
mod owned {
    use super::*;
    use vdn_shm::OwnedBus;

    #[test]
    #[cfg_attr(miri, ignore = "Miri 不支持 shm_open / mmap")]
    fn second_owner_is_busy_and_drop_unlinks() {
        let name = unique_name();
        let _c = Cleanup(name.clone());
        let a = OwnedBus::create(&name, CFG).unwrap();
        assert_eq!(OwnedBus::create(&name, CFG).err(), Some(ShmError::Busy));
        // 被拒的那次没删掉 a 的总线
        assert!(Bus::open(&name).is_ok());
        assert!(a.lock_path().starts_with(std::env::temp_dir()));
        a.unlink().unwrap();
        a.unlink().unwrap(); // 第二次是空操作
        drop(a);
        assert_eq!(Bus::open(&name).err(), Some(ShmError::NotFound));
        // 锁放了，可以再建
        let b = OwnedBus::create(&name, CFG).unwrap();
        drop(b);
        assert_eq!(
            Bus::open(&name).err(),
            Some(ShmError::NotFound),
            "Drop 删了名字"
        );
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri 不支持 shm_open / mmap")]
    fn legacy_owner_lock_is_respected() {
        // 模拟 rs-entry 旧版建方：只拿旧锁文件
        let name = unique_name();
        let _c = Cleanup(name.clone());
        let old = std::fs::File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(OwnedBus::legacy_lock_path_for(&name))
            .unwrap();
        old.try_lock().unwrap();
        let held = Bus::create(&name, CFG).unwrap();
        assert_eq!(OwnedBus::create(&name, CFG).err(), Some(ShmError::Busy));
        assert!(Bus::open(&name).is_ok(), "旧建方的总线没被删");
        drop(old);
        drop(held);
        assert!(OwnedBus::create(&name, CFG).is_ok(), "旧锁放了就能建");
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri 不支持 shm_open / mmap")]
    fn stale_leftover_is_cleared() {
        let name = unique_name();
        let _c = Cleanup(name.clone());
        let stale = Bus::create(&name, CFG).unwrap(); // 模拟上次异常退出留下的（没拿锁、没 unlink）
        let old = stale.session_id();
        drop(stale);
        let a = OwnedBus::create(&name, CFG).unwrap();
        assert_ne!(a.session_id(), old);
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri 不支持 shm_open / mmap")]
    fn bad_name_creates_no_lock_file() {
        assert_eq!(
            OwnedBus::create("/nope", CFG).err(),
            Some(ShmError::InvalidName)
        );
        assert!(!OwnedBus::lock_path_for("/nope").exists());
    }
}
