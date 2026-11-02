//! 实时路径（ring · push · pop · discard · beat · 计数器）零分配。
// assert_no_alloc 的 AllocDisabler 只在 debug_assertions 下存在：release 测试跳过这个文件
#![cfg(debug_assertions)]

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_shm::{Bus, BusConfig, RingId};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn realtime_path_does_not_allocate() {
    let bus = Bus::heap(BusConfig {
        rate: 48_000,
        capacity_frames: 1024,
        l_total: 0,
    })
    .unwrap();
    let src = [0.25f32; 512];
    let mut dst = [0.0f32; 512];
    assert_no_alloc(|| {
        for _ in 0..100 {
            for id in RingId::ALL {
                let r = bus.ring(id);
                r.push(&src);
                r.push(&src);
                r.push(&src); // 溢出路径
                r.pop(&mut dst);
                r.discard(100);
                r.pop(&mut dst);
                r.pop(&mut dst); // 欠载路径
                r.note_underrun(1);
                let _ = (
                    r.available(),
                    r.free_space(),
                    r.take_overflows(),
                    r.take_underruns(),
                );
            }
            bus.beat();
            let _ = (bus.heartbeat(), bus.l_total(), bus.rate(), bus.capacity());
        }
    });
}
