//! 音频线程（麦克风回调 / 正弦线程）的入口：一块原声进，经 `Forwarder` 推进 Out 总线；
//! 降噪总线换了时，从监视线程**经 rtrb 接手**新总线（和新 Forwarder），旧的**经回收 rtrb 送回**监视线程释放
//! （规约 §3.4 跨线程原语 · §3.5 换模型同一套；音频线程里不加锁、不 drop 堆对象，R2 / R7）。

use std::sync::Arc;

use rtrb::{Consumer, Producer, PushError, RingBuffer};
use vdn_shm::{Bus, OwnedBus};

use crate::Forwarder;
use crate::shm::forward;

/// 监视线程 → 音频线程：换上这条降噪总线（`None` = 断开，只送原声）。
/// `fwd = None`：L_total 没变，音频线程在原 Forwarder 上 `rebind`（延迟线不断）
pub struct Handover {
    pub bus: Option<Arc<Bus>>,
    pub fwd: Option<Forwarder>,
}

/// 音频线程 → 监视线程：换下来的旧对象（在监视线程里释放）
pub struct Retired {
    pub bus: Option<Arc<Bus>>,
    pub fwd: Option<Forwarder>,
}

/// 监视线程一侧的两个端口
pub struct Control {
    pub handover: Producer<Handover>,
    pub retired: Consumer<Retired>,
}

pub struct Feeder {
    bus: Option<Arc<Bus>>,
    fwd: Forwarder,
    out: Arc<OwnedBus>,
    rx: Consumer<Handover>,
    back: Producer<Retired>,
    /// 回收队列满时先留在这里，下一块再送
    pending: Option<Retired>,
    switches: u64,
}

// 构造：非实时
impl Feeder {
    /// 队列容量：交接 4 个、回收 8 个（换总线是秒级的事，够用）
    pub fn new(bus: Option<Arc<Bus>>, fwd: Forwarder, out: Arc<OwnedBus>) -> (Self, Control) {
        let (handover, rx) = RingBuffer::new(4);
        let (back, retired) = RingBuffer::new(8);
        (
            Self {
                bus,
                fwd,
                out,
                rx,
                back,
                pending: None,
                switches: 0,
            },
            Control { handover, retired },
        )
    }
}

// 实时（A 档）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl Feeder {
    pub fn forwarder(&self) -> &Forwarder {
        &self.fwd
    }
    /// 接手过几次新总线
    pub fn switches(&self) -> u64 {
        self.switches
    }

    pub fn feed(&mut self, x: &[f32]) {
        // 先把上次没送出去的旧对象送回去；送不出就这块先不接新的（不让旧对象在本线程里被覆盖 drop）
        if let Some(r) = self.pending.take()
            && let Err(PushError::Full(r)) = self.back.push(r)
        {
            self.pending = Some(r);
        }
        if self.pending.is_none()
            && let Ok(h) = self.rx.pop()
        {
            let old_bus = core::mem::replace(&mut self.bus, h.bus);
            let old_fwd = match h.fwd {
                Some(f) => Some(core::mem::replace(&mut self.fwd, f)),
                None => {
                    self.fwd.rebind();
                    None
                }
            };
            self.switches += 1;
            if let Err(PushError::Full(r)) = self.back.push(Retired {
                bus: old_bus,
                fwd: old_fwd,
            }) {
                self.pending = Some(r);
            }
        }
        forward(&mut self.fwd, self.bus.as_deref(), &self.out, x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ForwarderConfig;
    use crate::shm::out_bus_config;
    use vdn_shm::RingId;

    #[test]
    fn handover_and_retire_without_dropping_in_audio_thread() {
        let name = format!("/vdn-tfd-{}", std::process::id());
        let out = Arc::new(OwnedBus::create(&name, out_bus_config(48_000)).unwrap());
        let (mut f, mut ctl) = Feeder::new(
            None,
            Forwarder::new(ForwarderConfig::new(96, 96)),
            out.clone(),
        );
        let x = crate::testing::test_tone(48_000, 96, 0);
        f.feed(&x);
        assert_eq!(f.switches(), 0);
        // 交一条新总线（L_total 不同 → 带新 Forwarder）
        let bus = Arc::new(Bus::heap(out_bus_config(48_000)).unwrap());
        assert!(
            ctl.handover
                .push(Handover {
                    bus: Some(bus.clone()),
                    fwd: Some(Forwarder::new(ForwarderConfig::new(500, 96))),
                })
                .is_ok()
        );
        f.feed(&x);
        assert_eq!(f.switches(), 1);
        assert_eq!(f.forwarder().config().l_total, 500);
        // 旧对象回到监视线程：旧总线是 None、旧 Forwarder 的 L_total 是 96
        let r = ctl.retired.pop().unwrap();
        assert!(r.bus.is_none());
        assert_eq!(r.fwd.unwrap().config().l_total, 96);
        // 原声推进了新总线的 Raw
        assert!(bus.ring(RingId::Raw).available() >= 96);
        assert_eq!(out.ring(RingId::Out).available(), 192);
    }

    #[test]
    fn retire_queue_full_keeps_old_objects_locally() {
        let name = format!("/vdn-tfq-{}", std::process::id());
        let out = Arc::new(OwnedBus::create(&name, out_bus_config(48_000)).unwrap());
        let (mut f, mut ctl) = Feeder::new(None, Forwarder::new(ForwarderConfig::new(96, 96)), out);
        let x = [0.0f32; 96];
        // 换 12 次、监视线程一直不收：回收队列 8 个满了以后，音频线程停在本地槽位，不再接新的
        for _ in 0..12 {
            let _ = ctl.handover.push(Handover {
                bus: None,
                fwd: None,
            });
            f.feed(&x);
        }
        assert_eq!(f.switches(), 9, "8 个进队列 + 1 个留在本地");
        // 监视线程收走以后恢复
        while ctl.retired.pop().is_ok() {}
        for _ in 0..4 {
            f.feed(&x);
        }
        assert!(f.switches() > 9);
    }
}
