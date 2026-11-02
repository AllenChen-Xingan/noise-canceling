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
                    let outcome = if usable && retries < MAX_RETRIES {
                        let ns = u64::try_from(left.as_nanos()).unwrap_or(u64::MAX);
                        wake::wait(word, seen, self.shared, ns)
                    } else {
                        Outcome::Failed
                    };
                    match outcome {
                        Outcome::Woken | Outcome::TimedOut => retries = 0,
                        Outcome::Retry => retries += 1,
                        Outcome::Failed => {
                            if usable && retries < MAX_RETRIES {
                                self.wake_broken.store(true, Relaxed);
                            }
                            retries = 0;
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
