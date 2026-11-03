//! loom 模型检查（规约 R6）：穷举两线程的交错顺序，查环的核心协议与 notify / wait 的「不丢唤醒」性质。
//! 只测协议（同一份 `Ring` / `Bus` 代码，原子类型经 `vdn_shm::sync` 换成 loom 的），不测 mmap 与系统调用。
//!
//! ```text
//! cargo test -p vdn-shm --no-default-features --features loom --test loom --release
//! ```
#![cfg(feature = "loom")]

use loom::sync::Arc;
use loom::thread;
use vdn_shm::{Bus, BusConfig, RingId};

fn bus(cap: u32) -> Bus {
    Bus::heap(BusConfig {
        rate: 48_000,
        capacity_frames: cap,
        l_total: 0,
        lead: 0,
        denoise_block: 0,
    })
    .unwrap()
}

/// 单生产者单消费者：容量 2、写 3 个样本（必然回绕、必然碰到满），消费方按序拿到 1 · 2 · 3，不丢不重、没有旧值。
#[test]
fn spsc_in_order_across_wrap() {
    loom::model(|| {
        let b = Arc::new(bus(2));
        let p = b.clone();
        let h = thread::spawn(move || {
            let r = p.ring(RingId::Raw);
            for x in [1.0f32, 2.0, 3.0] {
                while r.push(&[x]) == 0 {
                    thread::yield_now();
                }
            }
        });
        let r = b.ring(RingId::Raw);
        let mut got = Vec::new();
        let mut one = [0.0f32];
        while got.len() < 3 {
            if r.pop(&mut one) == 1 {
                got.push(one[0]);
            } else {
                thread::yield_now();
            }
        }
        h.join().unwrap();
        assert_eq!(got, [1.0, 2.0, 3.0]);
    });
}

/// 不丢唤醒：消费方照 `wait` 的第一步取快照（先读计数字、再看可读量）；数据不够时模拟内核「计数字仍等于快照值才睡」。
/// 性质：只要消费方睡下去了，之后一定还有一次 notify（计数字最终 ≠ 快照值），所以它会被叫醒。
/// 若把快照里两次读的顺序写反，loom 会找到「睡下去后再没人叫醒」的交错。
#[test]
fn wait_snapshot_never_sleeps_through_a_notify() {
    loom::model(|| {
        let b = Arc::new(bus(4));
        let p = b.clone();
        let h = thread::spawn(move || {
            p.push_notify(RingId::Raw, &[1.0]);
        });
        let (seen, available) = b.wait_snapshot(RingId::Raw);
        let slept = available < 1 && b.wake_word(RingId::Raw) == seen;
        h.join().unwrap();
        if slept {
            assert_ne!(
                b.wake_word(RingId::Raw),
                seen,
                "睡下去之后没有新的 notify：唤醒丢了"
            );
        } else {
            // 没睡：要么已经看到数据，要么计数字已变（内核会立刻返回，下一轮再看）
            assert!(available >= 1 || b.wake_word(RingId::Raw) != seen);
        }
    });
}
