//! 把 `vdn-shm` 的环和心跳接到本 crate 的小 trait 上；把 Out 环接成 `vdn-hal` 的 `SampleSource`。
//! shm 与 hal 互不依赖，这层小包装归送音端（大总管定，2026-11-02）。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use vdn_shm::{Bus, Ring, RingId};

use crate::{Heartbeat, Pop, Push};

// 实时（A 档）：全部转调 vdn-shm 的实时方法
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Push for Ring<'_> {
    fn push(&mut self, x: &[f32]) -> usize {
        Ring::push(self, x)
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
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

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Heartbeat for Bus {
    fn heartbeat(&self) -> u64 {
        Bus::heartbeat(self)
    }
}

/// Out 环的消费端，给 `HalSource` 用（开发期也给扬声器输出回调、文件写出线程用）。
/// - 欠载：`HalSource` 补零后调 `note_underrun`，这里转记进 Out 环头部的计数器
/// - 积压上限：读方停过一阵再回来时，Out 里会积着旧声音，照读会让用户这一侧永久多出那么多延迟。
///   可读量超过 `max_backlog + 本次要的量` 就先丢掉最旧的多余部分（计入 `dropped()`）
pub struct OutSource {
    bus: Arc<Bus>,
    max_backlog: usize,
    dropped: Arc<AtomicU64>,
}

impl OutSource {
    /// 默认积压上限 200 ms（9600 样本）
    pub const DEFAULT_MAX_BACKLOG: usize = 9600;

    pub fn new(bus: Arc<Bus>) -> Self {
        Self::with_max_backlog(bus, Self::DEFAULT_MAX_BACKLOG)
    }
    pub fn with_max_backlog(bus: Arc<Bus>, max_backlog: usize) -> Self {
        Self {
            bus,
            max_backlog,
            dropped: Arc::new(AtomicU64::new(0)),
        }
    }
    /// 因积压超限丢掉的样本数（可在别的线程读）
    pub fn dropped(&self) -> Arc<AtomicU64> {
        self.dropped.clone()
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl vdn_hal::SampleSource for OutSource {
    fn pop(&mut self, out: &mut [f32]) -> usize {
        let ring = self.bus.ring(RingId::Out);
        let limit = self.max_backlog.saturating_add(out.len());
        let avail = ring.available();
        if avail > limit {
            let d = ring.discard(avail - limit);
            self.dropped.fetch_add(d as u64, Ordering::Relaxed);
        }
        ring.pop(out)
    }
    fn note_underrun(&mut self, missing: usize) {
        self.bus.ring(RingId::Out).note_underrun(missing);
    }
}

/// 一块原声走一遍 `Forwarder`：Raw / Processed / 心跳取自降噪总线 `bus`（`None` = 还没有降噪进程，整段送延迟原声），
/// 结果推进送音端自己建的 Out 总线 `out`（插件只连它，降噪进程重启不影响它）。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
pub fn forward(fwd: &mut crate::Forwarder, bus: Option<&Bus>, out: &Bus, input: &[f32]) {
    let mut o = out.ring(RingId::Out);
    match bus {
        Some(b) => {
            let (mut raw, mut processed) = (b.ring(RingId::Raw), b.ring(RingId::Processed));
            fwd.process(input, &mut raw, &mut processed, &mut o, b);
            // 原声已推进 Raw：叫醒降噪进程（一次非阻塞系统调用，规约 §4 A 档唯一例外）
            b.notify(RingId::Raw);
        }
        None => fwd.process(
            input,
            &mut crate::NullRing,
            &mut crate::NullRing,
            &mut o,
            &crate::NoBeat,
        ),
    }
}

/// 旧写法（Out 也在降噪总线上）：单进程 `vdn run`、测试用。
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
pub fn forward_same_bus(fwd: &mut crate::Forwarder, bus: &Bus, input: &[f32]) {
    let (mut raw, mut processed, mut out) = (
        bus.ring(RingId::Raw),
        bus.ring(RingId::Processed),
        bus.ring(RingId::Out),
    );
    fwd.process(input, &mut raw, &mut processed, &mut out, bus);
    bus.notify(RingId::Raw);
}

/// 送音端自己建的 Out 总线的配置：只用 Out 环，L_total / lead / 降噪块长无意义（填 0）。
pub fn out_bus_config(capacity: u32) -> vdn_shm::BusConfig {
    vdn_shm::BusConfig {
        rate: vdn_control::IO_RATE,
        capacity_frames: capacity,
        l_total: 0,
        lead: 0,
        denoise_block: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_hal::SampleSource;

    #[test]
    fn out_backlog_is_capped() {
        let bus = Arc::new(Bus::heap(out_bus_config(48_000)).unwrap());
        let out = bus.ring(RingId::Out);
        let x: Vec<f32> = (0..20_000).map(|i| i as f32).collect();
        assert_eq!(out.push(&x), 20_000);
        let mut src = OutSource::with_max_backlog(bus.clone(), 1000);
        let mut y = [0.0f32; 480];
        assert_eq!(src.pop(&mut y), 480);
        // 留下的是最新的 1000 + 480 个：第一个读到的是 20000 - 1480
        assert_eq!(y[0], (20_000 - 1480) as f32);
        assert_eq!(src.dropped().load(Ordering::Relaxed), 20_000 - 1480);
        assert_eq!(bus.ring(RingId::Out).available(), 1000);
    }
}
