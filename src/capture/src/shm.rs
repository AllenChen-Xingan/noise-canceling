//! 把 `vdn-shm` 的环和心跳接到本 crate 的小 trait 上；把 Out 环接成 `vdn-hal` 的 `SampleSource`。
//! shm 与 hal 互不依赖，这层小包装归送音端（大总管定，2026-11-02）。

use std::sync::Arc;

use vdn_shm::{Bus, Ring, RingId};

use crate::{Heartbeat, Pop, Push};

// 实时（A 档）：全部转调 vdn-shm 的实时方法
#[deny(clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros)]
impl Push for Ring<'_> {
    fn push(&mut self, x: &[f32]) -> usize {
        Ring::push(self, x)
    }
}

#[deny(clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros)]
impl Pop for Ring<'_> {
    fn pop(&mut self, out: &mut [f32]) -> usize {
        Ring::pop(self, out)
    }
    fn available(&self) -> usize {
        Ring::available(self)
    }
    /// O(1)：只挪读指针
    fn discard(&mut self, n: usize) -> usize {
        Ring::discard(self, n)
    }
}

#[deny(clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros)]
impl Heartbeat for Bus {
    fn heartbeat(&self) -> u64 {
        Bus::heartbeat(self)
    }
}

/// Out 环的消费端，给 `HalSource` 用（开发期也给扬声器输出回调、文件写出线程用）。
/// 欠载：`HalSource` 补零后调 `note_underrun`，这里转记进 Out 环头部的计数器。
pub struct OutSource {
    bus: Arc<Bus>,
}

impl OutSource {
    pub fn new(bus: Arc<Bus>) -> Self {
        Self { bus }
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl vdn_hal::SampleSource for OutSource {
    fn pop(&mut self, out: &mut [f32]) -> usize {
        self.bus.ring(RingId::Out).pop(out)
    }
    fn note_underrun(&mut self, missing: usize) {
        self.bus.ring(RingId::Out).note_underrun(missing);
    }
}

/// 一块原声走一遍 `Forwarder`，三个环都取自同一条总线。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
pub fn forward(fwd: &mut crate::Forwarder, bus: &Bus, input: &[f32]) {
    let (mut raw, mut processed, mut out) = (
        bus.ring(RingId::Raw),
        bus.ring(RingId::Processed),
        bus.ring(RingId::Out),
    );
    fwd.process(input, &mut raw, &mut processed, &mut out, bus);
}
