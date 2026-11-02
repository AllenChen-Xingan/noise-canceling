//! 真 POSIX 共享内存：两个线程各自映射同一个对象（create 一份、open 一份），各推各取 10 万块，逐样本一致。
#![cfg(all(unix, feature = "posix"))]

use std::sync::atomic::{AtomicU32, Ordering};
use vdn_shm::{Bus, BusConfig, RingId, ShmError};

/// 随机后缀：pid + 计数 + 时间，防止并行测试撞名。总长 ≤ 31。
fn unique_name() -> String {
    static N: AtomicU32 = AtomicU32::new(0);
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let n = N.fetch_add(1, Ordering::Relaxed);
    format!("/vdn-t{:x}-{n}-{:x}", std::process::id(), t & 0xffff)
}

/// 测完一定 unlink（panic 也删）。
struct Cleanup(String);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = Bus::unlink_name(&self.0);
    }
}

const CFG: BusConfig = BusConfig {
    rate: 48_000,
    capacity_frames: 4096,
    l_total: 960,
};

#[test]
fn create_open_validate_unlink() {
    let name = unique_name();
    let _c = Cleanup(name.clone());
    assert_eq!(Bus::open(&name).err(), Some(ShmError::NotFound));
    let a = Bus::create(&name, CFG).unwrap();
    assert_eq!(Bus::create(&name, CFG).err(), Some(ShmError::AlreadyExists));
    let b = Bus::open(&name).unwrap();
    assert_eq!(b.config(), CFG);
    assert_eq!(b.name(), Some(name.as_str()));
    a.beat();
    assert_eq!(b.heartbeat(), 1, "两份映射看到同一个心跳");
    a.ring(RingId::Out).push(&[0.5]);
    let mut x = [0.0];
    assert_eq!(b.ring(RingId::Out).pop(&mut x), 1);
    assert_eq!(x[0], 0.5);
    a.unlink().unwrap();
    assert_eq!(Bus::open(&name).err(), Some(ShmError::NotFound));
    // 已映射的照常用
    a.ring(RingId::Raw).push(&[1.0]);
    assert_eq!(b.ring(RingId::Raw).available(), 1);
}

#[test]
fn bad_names_are_rejected() {
    for n in [
        "vdn-x",
        "/vdn-",
        "/other-x",
        "/vdn-a/b",
        "/vdn-0123456789012345678901234567",
    ] {
        assert_eq!(
            Bus::create(n, CFG).err(),
            Some(ShmError::InvalidName),
            "{n}"
        );
    }
}

#[test]
fn two_threads_100k_blocks_sample_exact() {
    const BLOCKS: u32 = 100_000;
    const BLOCK: usize = 64; // 共 640 万样本 < 2^24，f32 能精确表示每个序号
    let name = unique_name();
    let _c = Cleanup(name.clone());
    let producer_bus = Bus::create(&name, CFG).unwrap();
    let consumer_bus = Bus::open(&name).unwrap();

    let consumer = std::thread::spawn(move || {
        let r = consumer_bus.ring(RingId::Raw);
        let mut buf = [0.0f32; BLOCK];
        let mut expect = 0u32;
        let total = BLOCKS * BLOCK as u32;
        while expect < total {
            let n = r.pop(&mut buf);
            if n == 0 {
                std::thread::yield_now();
                continue;
            }
            for &x in &buf[..n] {
                assert_eq!(x, expect as f32, "第 {expect} 个样本不一致");
                expect += 1;
            }
        }
        consumer_bus.heartbeat()
    });

    let r = producer_bus.ring(RingId::Raw);
    let mut block = [0.0f32; BLOCK];
    let mut next = 0u32;
    for _ in 0..BLOCKS {
        for x in block.iter_mut() {
            *x = next as f32;
            next += 1;
        }
        let mut sent = 0;
        while sent < BLOCK {
            // 只推环里放得下的，避免计溢出丢样本
            let room = r.free_space().min(BLOCK - sent);
            sent += r.push(&block[sent..sent + room]);
            if sent < BLOCK {
                std::thread::yield_now();
            }
        }
        producer_bus.beat();
    }
    let beats_seen = consumer.join().unwrap();
    assert_eq!(r.overflows(), 0);
    assert!(beats_seen <= u64::from(BLOCKS));
    assert_eq!(producer_bus.heartbeat(), u64::from(BLOCKS));
    producer_bus.unlink().unwrap();
}
