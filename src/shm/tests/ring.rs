//! 进程内 heap 版：顺序 · 回绕 · 满 · 空 · 计数器 · 头部。Miri 也跑这个文件（本机暂无 Miri，见现状页）。

use vdn_shm::{Bus, BusConfig, RingId, ShmError};

fn bus(cap: u32) -> Bus {
    Bus::heap(BusConfig {
        rate: 48_000,
        capacity_frames: cap,
        l_total: 123,
        lead: 0,
        denoise_block: 0,
    })
    .unwrap()
}

#[test]
fn header_reads_back() {
    let b = bus(8);
    assert_eq!(b.rate(), 48_000);
    assert_eq!(b.l_total(), 123);
    assert_eq!(b.capacity(), 8);
    assert_eq!(b.config().capacity_frames, 8);
    assert_eq!(b.name(), None);
    assert_eq!(b.heartbeat(), 0);
    b.beat();
    b.beat();
    assert_eq!(b.heartbeat(), 2);
    assert!(b.unlink().is_ok(), "heap 版 unlink 是空操作");
}

#[test]
fn bad_config_is_rejected() {
    let c = |rate, cap| BusConfig {
        rate,
        capacity_frames: cap,
        l_total: 0,
        lead: 0,
        denoise_block: 0,
    };
    assert!(matches!(
        Bus::heap(c(48_000, 0)),
        Err(ShmError::InvalidConfig(_))
    ));
    assert!(matches!(
        Bus::heap(c(0, 8)),
        Err(ShmError::InvalidConfig(_))
    ));
    assert!(matches!(
        Bus::heap(c(48_000, (1 << 24) + 1)),
        Err(ShmError::InvalidConfig(_))
    ));
}

#[test]
fn order_and_empty() {
    let b = bus(8);
    let r = b.ring(RingId::Raw);
    let mut out = [9.0f32; 4];
    assert_eq!(r.pop(&mut out), 0, "空环读不到");
    assert_eq!(out, [9.0; 4], "读不到时 dst 不动");
    assert_eq!(r.push(&[1.0, 2.0, 3.0]), 3);
    assert_eq!(r.available(), 3);
    assert_eq!(r.pop(&mut out), 3, "只读到有的部分");
    assert_eq!(out, [1.0, 2.0, 3.0, 9.0]);
    assert_eq!(r.available(), 0);
}

#[test]
fn full_drops_tail_and_counts_overflow() {
    let b = bus(4);
    let r = b.ring(RingId::Processed);
    assert_eq!(r.push(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]), 4);
    assert_eq!(r.overflows(), 2);
    assert_eq!(r.free_space(), 0);
    assert_eq!(r.push(&[7.0]), 0);
    assert_eq!(r.take_overflows(), 3);
    assert_eq!(r.overflows(), 0);
    let mut out = [0.0; 6];
    assert_eq!(r.pop(&mut out), 4);
    assert_eq!(&out[..4], &[1.0, 2.0, 3.0, 4.0], "满时不覆盖旧数据");
}

#[test]
fn wraps_around_many_times() {
    let b = bus(5);
    let r = b.ring(RingId::Out);
    let mut next_in = 0.0f32;
    let mut next_out = 0.0f32;
    for round in 0..200 {
        let k = 1 + round % 4; // 1..=4，与容量 5 互素，回绕点到处跑
        let src: Vec<f32> = (0..k).map(|i| next_in + i as f32).collect();
        assert_eq!(r.push(&src), k);
        next_in += k as f32;
        let mut dst = vec![0.0; k];
        assert_eq!(r.pop(&mut dst), k);
        for x in dst {
            assert_eq!(x, next_out);
            next_out += 1.0;
        }
    }
    assert_eq!(r.overflows(), 0);
}

#[test]
fn rings_are_independent() {
    let b = bus(4);
    b.ring(RingId::Raw).push(&[1.0]);
    b.ring(RingId::Processed).push(&[2.0, 2.0]);
    assert_eq!(b.ring(RingId::Raw).available(), 1);
    assert_eq!(b.ring(RingId::Processed).available(), 2);
    assert_eq!(b.ring(RingId::Out).available(), 0);
}

#[test]
fn discard_and_underrun_counter() {
    let b = bus(8);
    let r = b.ring(RingId::Raw);
    r.push(&[1.0, 2.0, 3.0]);
    assert_eq!(r.discard(2), 2);
    assert_eq!(r.discard(5), 1, "只丢得到有的");
    r.note_underrun(7);
    assert_eq!(r.underruns(), 7);
    assert_eq!(r.take_underruns(), 7);
    assert_eq!(r.underruns(), 0);
}

#[test]
fn special_floats_roundtrip_bit_exact() {
    let b = bus(8);
    let r = b.ring(RingId::Raw);
    let src = [f32::NAN, -0.0, f32::INFINITY, f32::MIN_POSITIVE / 2.0];
    r.push(&src);
    let mut dst = [0.0; 4];
    r.pop(&mut dst);
    for (a, b) in src.iter().zip(dst) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
}

#[test]
fn pop_or_zero_fills_and_counts() {
    let b = bus(8);
    let r = b.ring(RingId::Out);
    r.push(&[1.0, 2.0]);
    let mut dst = [9.0f32; 5];
    assert_eq!(r.pop_or_zero(&mut dst), 2);
    assert_eq!(
        dst,
        [1.0, 2.0, 0.0, 0.0, 0.0],
        "不够的部分填 0，不留上一块的声音"
    );
    assert_eq!(r.underruns(), 3);
}

#[test]
fn drift_full_ring_realigned_by_discard() {
    // 生产方快于消费方：环停在满、多出来的被丢；消费方按目标积压 discard 回去
    let b = bus(16);
    let r = b.ring(RingId::Processed);
    for _ in 0..10 {
        r.push(&[0.5; 4]);
    }
    assert_eq!(r.available(), 16);
    assert!(r.overflows() > 0);
    let target = 4;
    r.discard(r.available() - target);
    assert_eq!(r.available(), target);
}

// loom 模型检查已在 tests/loom.rs（cargo test -p vdn-shm --no-default-features --features loom --test loom --release）
