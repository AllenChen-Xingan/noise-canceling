//! `Bus::wait`：消费方在**非实时线程**上等数据（会睡、会做系统调用）。整个文件是非实时模块。

#![allow(clippy::disallowed_methods)] // NRT: wait 只给非实时线程用（降噪进程处理线程空闲时），要 sleep 退回轮询

use core::sync::atomic::Ordering::{Acquire, Relaxed};
use core::time::Duration;

use crate::wake::{self, Outcome};
use crate::{Bus, RingId, WaitResult, WaitStrategy};

/// 系统不支持唤醒或调用出错时，`wait` 退回的轮询间隔。
pub const FALLBACK_POLL: Duration = Duration::from_micros(200);
/// `wait` 连续几次「可重试的提前返回」之后，这一轮改睡一个轮询间隔（防 EFAULT 一直重复时空转）。
const MAX_RETRIES: u32 = 3;
/// 单次系统调用最多睡这么久（防极大 timeout 换算成 Mach 时间时溢出；自审 P2）；到点醒来重新看一眼再睡。
const MAX_SLICE: Duration = Duration::from_secs(3600);

/// 等待（非实时：会睡、会做系统调用；**只许在非实时线程调**，例如降噪进程的处理线程在空闲时）
impl Bus {
    /// 等到 `id` 环里至少有 `min_available` 个样本可读，或过了 `timeout`。
    ///
    /// - 数据已经够：立刻返回 `Ready`，不做系统调用
    /// - 不够：`WaitStrategy::Wake` 下睡在该环的唤醒计数字上，生产方 `notify` 一下就醒；
    ///   系统不支持（低于 macOS 14.4）或调用出错，改用 200 µs 间隔轮询，并给该环的 `wait_fallbacks` 加 1（每次调用最多 1）
    /// - `WaitStrategy::Poll(间隔)`：按给定间隔轮询（对照实验用；不计 `wait_fallbacks`）
    /// - `min_available` 大于环容量时按容量算（否则永远等不到）；为 0 时立刻返回
    /// - 一定在 `timeout` 之后不久返回（不会永远睡死）：生产方崩了也只是超时。`timeout` 为 0 = 只看一眼
    /// - 被别的 `notify`（数据还不够）叫醒会接着等，不算返回
    pub fn wait(&self, id: RingId, min_available: usize, timeout: Duration) -> WaitResult {
        let ring = self.ring(id);
        let want = min_available.min(self.cap);
        let word = &ring.hdr.prod.wake;
        let start = std::time::Instant::now();
        let deadline = start.checked_add(timeout);
        let mut fell_back = false;
        let mut retries = 0u32;
        loop {
            // 先读计数字、再看数据：生产方是「先 push 再加计数字」，所以看漏的数据一定伴随计数字已变，
            // 系统调用比较时发现值不同会立刻返回——唤醒不会丢
            let seen = word.load(Acquire);
            #[cfg(test)]
            tests::gap();
            let available = ring.available();
            if available >= want {
                return WaitResult::Ready { available };
            }
            let now = std::time::Instant::now();
            let left = match deadline {
                Some(d) if now >= d => return WaitResult::TimedOut { available },
                Some(d) => d - now,
                None => Duration::from_secs(3600), // timeout 大到溢出：按一小时一轮继续等
            };
            let poll = |iv: Duration| std::thread::sleep(iv.min(left));
            match self.strategy {
                WaitStrategy::Poll(iv) => poll(iv.max(Duration::from_micros(1))),
                WaitStrategy::Wake => {
                    let usable = wake::supported() && !self.wake_broken.load(Relaxed);
                    if usable && retries >= MAX_RETRIES {
                        // 连续几次暂时性错误（EINTR 等）：这一轮睡一个轮询间隔再试，不算退回轮询
                        retries = 0;
                        poll(FALLBACK_POLL);
                        continue;
                    }
                    let outcome = if usable {
                        let ns = u64::try_from(left.min(MAX_SLICE).as_nanos()).unwrap_or(u64::MAX);
                        wake::wait(word, seen, self.shared, ns)
                    } else {
                        Outcome::Failed
                    };
                    match outcome {
                        Outcome::Woken | Outcome::TimedOut => retries = 0,
                        Outcome::Retry => retries += 1,
                        Outcome::Failed => {
                            if usable {
                                // 不可恢复（EINVAL 等）：本句柄以后都轮询
                                self.wake_broken.store(true, Relaxed);
                            }
                            if !fell_back {
                                fell_back = true;
                                ring.hdr.cons.wait_fallbacks.fetch_add(1, Relaxed);
                            }
                            poll(FALLBACK_POLL);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BusConfig;
    use core::sync::atomic::AtomicU64;

    /// 单元测试专用：在「读计数字」与「看可读量」之间停这么多微秒，把丢唤醒的窗口撑大到必然命中。
    static GAP_US: AtomicU64 = AtomicU64::new(0);

    pub(super) fn gap() {
        let g = GAP_US.load(Relaxed);
        if g > 0 {
            std::thread::sleep(Duration::from_micros(g));
        }
    }

    /// 消费方读完计数字后停 3 ms；生产方在第 1 ms 推 + notify。
    /// 顺序对（先读计数字）：看可读量时数据已在，立刻 Ready。
    /// 顺序反（先看可读量）：数据看漏、计数字读到的是新值，睡下去没人再叫醒，要等满 500 ms 超时。
    /// 反向验证（2026-11-02，rs-wake）：把两次读对调后本测试失败；改回后通过。
    #[test]
    fn widened_race_window_does_not_lose_wakeup() {
        GAP_US.store(3_000, Relaxed);
        let b = Bus::heap(BusConfig {
            rate: 48_000,
            capacity_frames: 1024,
            l_total: 0,
            lead: 0,
            denoise_block: 0,
        })
        .unwrap();
        for _ in 0..5 {
            std::thread::scope(|s| {
                s.spawn(|| {
                    std::thread::sleep(Duration::from_millis(1));
                    b.push_notify(RingId::Raw, &[0.0; 96]);
                });
                let t = std::time::Instant::now();
                let r = b.wait(RingId::Raw, 96, Duration::from_millis(500));
                assert!(r.is_ready(), "{r:?}");
                assert!(
                    t.elapsed() < Duration::from_millis(100),
                    "{:?}",
                    t.elapsed()
                );
            });
            b.ring(RingId::Raw).discard(96);
        }
        GAP_US.store(0, Relaxed);
    }
}
