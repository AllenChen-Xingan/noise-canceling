//! 监视线程的判断（非实时）：什么时候接上 / 换到同名的新降噪总线。
//! - 还没有降噪总线：每次 `poll` 都试着 `Bus::open`，开到了就接上
//! - 有：心跳在动就不管；心跳停了超过 `reconnect_after`，再看同名的是不是**另一条**（会话 ID 不同）——
//!   是就换（降噪进程重启后建的），不是就不动（同一个降噪进程卡住了，送音端照常送原声）
//! - 新总线经 `Control::handover`（rtrb）交给音频线程；旧对象从 `Control::retired` 收回、在这里释放
#![allow(
    clippy::disallowed_methods, // NRT: 监视线程，非实时（开共享内存、建 Forwarder、Arc）
    clippy::disallowed_types,
    clippy::disallowed_macros
)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use vdn_shm::Bus;

use crate::feeder::{Control, Handover};
use crate::{Forwarder, ForwarderConfig};

/// 一次 `poll` 发生了什么（给 bin 打日志、给测试断言）
pub enum Event {
    /// 接上 / 换到一条降噪总线
    Switched {
        bus: Arc<Bus>,
        /// 之前有没有总线（false = 第一次接上）
        replaced: bool,
        /// L_total 变了、交了新 Forwarder（true）还是原 Forwarder `rebind`（false）
        new_forwarder: bool,
    },
    /// 交接队列满（音频线程没在跑？），下一次再试
    QueueFull,
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Switched {
                bus,
                replaced,
                new_forwarder,
            } => write!(
                f,
                "Switched {{ session: {:016x}, replaced: {replaced}, new_forwarder: {new_forwarder} }}",
                bus.session_id()
            ),
            Self::QueueFull => write!(f, "QueueFull"),
        }
    }
}

pub struct Supervisor {
    name: String,
    cur: Option<Arc<Bus>>,
    cur_l_total: usize,
    last_beat: u64,
    beat_at: Instant,
    reconnect_after: Duration,
    block: usize,
    timeout: usize,
    switches: u32,
}

impl Supervisor {
    /// `l_total` = 音频线程手上那个 Forwarder 的 L_total
    pub fn new(
        name: &str,
        first: Option<Arc<Bus>>,
        l_total: usize,
        block: usize,
        timeout: usize,
        reconnect_after: Duration,
        now: Instant,
    ) -> Self {
        let last_beat = first.as_ref().map_or(0, |b| b.heartbeat());
        Self {
            name: name.to_owned(),
            cur: first,
            cur_l_total: l_total,
            last_beat,
            beat_at: now,
            reconnect_after,
            block,
            timeout,
            switches: 0,
        }
    }

    pub fn current(&self) -> Option<&Arc<Bus>> {
        self.cur.as_ref()
    }
    pub fn switches(&self) -> u32 {
        self.switches
    }

    pub fn poll(&mut self, ctl: &mut Control, now: Instant) -> Option<Event> {
        // 收回音频线程换下来的旧对象（在这里释放）
        while let Ok(old) = ctl.retired.pop() {
            drop(old);
        }
        if let Some(b) = &self.cur {
            let hb = b.heartbeat();
            if hb != self.last_beat {
                (self.last_beat, self.beat_at) = (hb, now);
                return None;
            }
            if now.duration_since(self.beat_at) < self.reconnect_after {
                return None;
            }
        }
        let new = Bus::open(&self.name).ok()?;
        if self
            .cur
            .as_ref()
            .is_some_and(|b| b.session_id() == new.session_id())
            || new.rate() != vdn_control::IO_RATE
        {
            return None;
        }
        let new = Arc::new(new);
        let l_new = new.l_total() as usize;
        let fwd = (l_new != self.cur_l_total).then(|| {
            let mut c = ForwarderConfig::new(l_new, self.block);
            c.timeout = self.timeout;
            Forwarder::new(c)
        });
        let new_forwarder = fwd.is_some();
        if ctl
            .handover
            .push(Handover {
                bus: Some(new.clone()),
                fwd,
            })
            .is_err()
        {
            return Some(Event::QueueFull);
        }
        let replaced = self.cur.is_some();
        self.cur = Some(new.clone());
        self.cur_l_total = l_new;
        (self.last_beat, self.beat_at) = (new.heartbeat(), now);
        self.switches += 1;
        Some(Event::Switched {
            bus: new,
            replaced,
            new_forwarder,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feeder::Feeder;
    use crate::shm::out_bus_config;
    use vdn_shm::{BusConfig, OwnedBus, RingId};

    fn den_cfg(l_total: u32) -> BusConfig {
        BusConfig {
            rate: 48_000,
            capacity_frames: 48_000,
            l_total,
            lead: 288,
            denoise_block: 96,
        }
    }

    struct Rig {
        feeder: Feeder,
        ctl: Control,
        sup: Supervisor,
        t0: Instant,
        _out: Arc<OwnedBus>,
    }

    fn rig(tag: &str, first: Option<Arc<Bus>>, l_total: usize) -> Rig {
        let out = Arc::new(
            OwnedBus::create(
                &format!("/vdn-tso{tag}-{}", std::process::id()),
                out_bus_config(48_000),
            )
            .unwrap(),
        );
        let (feeder, ctl) = Feeder::new(
            first.clone(),
            Forwarder::new(ForwarderConfig::new(l_total, 96)),
            out.clone(),
        );
        let t0 = Instant::now();
        let name = format!("/vdn-tsv{tag}-{}", std::process::id());
        let sup = Supervisor::new(&name, first, l_total, 96, 4800, Duration::from_secs(1), t0);
        Rig {
            feeder,
            ctl,
            sup,
            t0,
            _out: out,
        }
    }

    fn name(tag: &str) -> String {
        format!("/vdn-tsv{tag}-{}", std::process::id())
    }

    const X: [f32; 96] = [0.25; 96];

    /// 送音端先起、降噪总线后出现：接上（L_total 变了 → 新 Forwarder），原声开始进新总线的 Raw
    #[test]
    fn connects_when_denoiser_appears() {
        let mut r = rig("a", None, 288);
        assert!(r.sup.poll(&mut r.ctl, r.t0).is_none());
        let den = OwnedBus::create(&name("a"), den_cfg(1056)).unwrap();
        let ev = r.sup.poll(&mut r.ctl, r.t0 + Duration::from_millis(100));
        assert!(
            matches!(
                ev,
                Some(Event::Switched {
                    replaced: false,
                    new_forwarder: true,
                    ..
                })
            ),
            "{ev:?}"
        );
        r.feeder.feed(&X);
        assert_eq!(r.feeder.switches(), 1);
        assert_eq!(r.feeder.forwarder().config().l_total, 1056);
        assert_eq!(den.ring(RingId::Raw).available(), 96);
        // 旧 Forwarder 回到监视线程，下一次 poll 里释放
        assert!(
            r.sup
                .poll(&mut r.ctl, r.t0 + Duration::from_millis(200))
                .is_none()
        );
    }

    /// 降噪进程重启（旧的被杀、新的建了同名总线）：心跳停够 1 s 后换过去；L_total 不变 → rebind（延迟线不断）
    #[test]
    fn switches_to_new_session_after_restart() {
        let a = OwnedBus::create(&name("b"), den_cfg(1056)).unwrap();
        let first = Arc::new(Bus::open(&name("b")).unwrap());
        let mut r = rig("b", Some(first), 1056);
        // 心跳不动但没到 1 s：不动
        assert!(
            r.sup
                .poll(&mut r.ctl, r.t0 + Duration::from_millis(500))
                .is_none()
        );
        drop(a); // 旧降噪进程退出（名字已删，送音端手上的映射还在）
        let b = OwnedBus::create(&name("b"), den_cfg(1056)).unwrap();
        let ev = r.sup.poll(&mut r.ctl, r.t0 + Duration::from_millis(1500));
        assert!(
            matches!(
                ev,
                Some(Event::Switched {
                    replaced: true,
                    new_forwarder: false,
                    ..
                })
            ),
            "{ev:?}"
        );
        r.feeder.feed(&X);
        assert_eq!(r.feeder.switches(), 1);
        assert_eq!(b.ring(RingId::Raw).available(), 96, "原声进了新总线");
    }

    /// 同一个降噪进程卡住（心跳不动、会话 ID 没变）：不换
    #[test]
    fn same_session_hung_is_not_switched() {
        let _a = OwnedBus::create(&name("c"), den_cfg(1056)).unwrap();
        let first = Arc::new(Bus::open(&name("c")).unwrap());
        let mut r = rig("c", Some(first), 1056);
        assert!(
            r.sup
                .poll(&mut r.ctl, r.t0 + Duration::from_secs(5))
                .is_none()
        );
        assert_eq!(r.sup.switches(), 0);
    }

    /// 交接队列满（音频线程不在跑）：报 QueueFull，状态不变，下次再试
    #[test]
    fn queue_full_is_retried() {
        let mut r = rig("d", None, 288);
        let _den = OwnedBus::create(&name("d"), den_cfg(1056)).unwrap();
        // 音频线程一直不 feed：队列 4 个
        for _ in 0..4 {
            let _ = r.ctl.handover.push(Handover {
                bus: None,
                fwd: None,
            });
        }
        assert!(matches!(
            r.sup.poll(&mut r.ctl, r.t0),
            Some(Event::QueueFull)
        ));
        assert!(r.sup.current().is_none());
        for _ in 0..4 {
            r.feeder.feed(&X);
        }
        assert!(matches!(
            r.sup.poll(&mut r.ctl, r.t0),
            Some(Event::Switched { .. })
        ));
    }
}
