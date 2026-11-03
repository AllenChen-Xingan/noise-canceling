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
    /// 纠漂移（写方一侧）：Out 积压连续多少块低于一块
    low_run: usize,
    /// 补过几块（读方时钟比麦克风快，Out 快读空了）
    slips_added: u64,
}

/// 纠漂移：积压连续偏离这么多块（≈ 50 ms @ 96）才动手，免得抖一下就补 / 丢
pub const DRIFT_PATIENCE: usize = 25;

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
                low_run: 0,
                slips_added: 0,
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
    /// 纠漂移补过几块
    pub fn slips_added(&self) -> u64 {
        self.slips_added
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

        // 纠漂移（写方一侧）：读方时钟比麦克风快时 Out 会慢慢读空。刚推完的积压连续 DRIFT_PATIENCE 块
        // 不到两块（= 推之前已不到一块），就把刚推的这块再推一次（重复 1 块，记 Out 环缺口）。
        // 积压过多那一侧由读方 `OutSource` 丢
        let out = self.out.ring(vdn_shm::RingId::Out);
        if out.available() < 2 * x.len() {
            self.low_run += 1;
        } else {
            self.low_run = 0;
        }
        if self.low_run >= DRIFT_PATIENCE {
            let last = self.fwd.last_output();
            let n = out.push(last);
            out.note_gap(n);
            self.slips_added += 1;
            self.low_run = 0;
        }
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

    /// 两个设备时钟差 ±0.1%（麦克风快 / 读方快），模拟 60 s：Out 积压（= 用户侧多出的延迟）不漂，
    /// 始终在 [0, 目标 + 3 块] 里；补 / 丢都记了缺口
    #[test]
    fn drift_is_corrected_both_ways() {
        use crate::shm::OutSource;
        use vdn_hal::{HalSource, RenderStatus};
        for (tag, producer_ppm) in [("fast", 1000i64), ("slow", -1000i64)] {
            let name = format!("/vdn-tdr{tag}-{}", std::process::id());
            let out = Arc::new(OwnedBus::create(&name, out_bus_config(48_000)).unwrap());
            let block = 96usize;
            let target = 2 * block;
            let (mut f, _ctl) = Feeder::new(
                None,
                Forwarder::new(ForwarderConfig::new(96, block)),
                out.clone(),
            );
            let mut src = HalSource::new(
                OutSource::new(Arc::new(Bus::open(&name).unwrap())).with_drift(target, block),
            );
            let x = crate::testing::test_tone(48_000, block, 0);
            let mut y = vec![0.0f32; block];
            // 事件时间以 ns 计：写方周期 = 2 ms / (1 + ppm)，读方周期 = 2 ms
            let base = 2_000_000i64;
            let wp = base * 1_000_000 / (1_000_000 + producer_ppm);
            let (mut tw, mut tr) = (
                0i64,
                i64::try_from(target).unwrap() * 1_000_000_000 / 48_000,
            );
            let end = 60_000_000_000i64;
            let (mut max_backlog, mut underruns) = (0usize, 0u64);
            while tr < end {
                if tw <= tr {
                    f.feed(&x);
                    tw += wp;
                } else {
                    if let RenderStatus::Underrun { .. } = src.render(&mut y) {
                        underruns += 1;
                    }
                    if tr > 10_000_000_000 {
                        max_backlog = max_backlog.max(out.ring(vdn_shm::RingId::Out).available());
                    }
                    tr += base;
                }
            }
            let gaps = out.ring(vdn_shm::RingId::Out).gaps();
            eprintln!(
                "{tag}（写方 {producer_ppm:+} ppm）：10–60 s 最大积压 {max_backlog} 样本 · 缺口 {gaps} 样本 · 补块 {} · 读方欠载 {underruns} 次",
                f.slips_added()
            );
            // 不纠时 60 s 会漂 ±2880 样本（0.1% × 60 s × 48 kHz）
            assert!(
                max_backlog <= target + 3 * block,
                "{tag}：最大积压 {max_backlog}"
            );
            assert!(gaps > 0, "{tag}：没有纠过");
            if producer_ppm < 0 {
                assert!(f.slips_added() > 0, "{tag}：读方快，应补过块");
                assert!(underruns < 30, "{tag}：欠载 {underruns} 次");
            }
        }
    }
}
