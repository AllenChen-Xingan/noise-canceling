//! vdn-capture：送音进程的核心逻辑（`Forwarder`）+ 链路两端共用的小 trait。
//!
//! 链路（docs/plan/010 第十节）：输入一块原声 → 推 Raw → 从 Processed 取同样多 →
//! 按心跳决定用处理后的声音还是本地延迟 `L_total` 的原声 → 推 Out。
//!
//! - 档：`Forwarder::process*` 跑在 IO 回调里（A 档）：不分配、不加锁、不 panic
//! - 依赖闭包里**不许有** `vdn-engine`、`vdn-models`、推理库（D03 崩溃隔离，§2.2 规则 3）
//! - `Forwarder` 对 [`Push`] / [`Pop`] / [`Heartbeat`] 泛型：真环由 bin 接 `vdn-shm`，单测用 [`testing`] 里的假环
//!
//! 对齐约定（送原声能无缝接上的前提）：降噪进程建总线后，先往 Processed 推
//! `lead = L_total - 管线延迟` 个零，再逐块推处理结果。于是 Processed 的第 k 个样本
//! = 原声第 `k - L_total` 个样本处理后的结果，和本地延迟线输出逐样本同一时刻。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]

mod config;
pub mod feeder;
mod forwarder;
pub mod shm;
pub mod supervisor;
pub mod testing;

pub use config::{ForwarderConfig, ForwarderStats};
pub use forwarder::Forwarder;

/// 单生产者一端：尽量写入，返回实际写入的样本数（环满时少于 `x.len()`）。
pub trait Push {
    fn push(&mut self, x: &[f32]) -> usize;
}

/// 单消费者一端：尽量读出，返回实际读出的样本数（不够时少于 `out.len()`）。
pub trait Pop {
    fn pop(&mut self, out: &mut [f32]) -> usize;
    /// 现在可读的样本数
    fn available(&self) -> usize;
    /// 丢掉至多 `n` 个样本，返回丢了多少。默认实现读出再扔（栈上 256 样本一段）；
    /// 真环应覆盖成 O(1) 的挪读指针（`vdn_shm::Ring::discard`）。
    fn discard(&mut self, n: usize) -> usize {
        let mut buf = [0.0f32; 256];
        let mut left = n;
        while left > 0 {
            let k = left.min(buf.len());
            let got = self.pop(buf.get_mut(..k).unwrap_or(&mut []));
            left -= got.min(left);
            if got < k {
                break;
            }
        }
        n - left
    }
}

/// 读降噪进程的状态（经总线头，带外）：心跳每处理一块 +1；缺口计数 = 它因落后跳过的 Raw 样本数。
pub trait Heartbeat {
    fn heartbeat(&self) -> u64;
    /// 降噪进程跳过 Raw 积压的累计样本数。涨了 = 接下来一个 L_total 窗口里会有零占位，送音端改送原声
    fn gaps(&self) -> u64 {
        0
    }
}

impl<T: Push + ?Sized> Push for &mut T {
    fn push(&mut self, x: &[f32]) -> usize {
        (**self).push(x)
    }
}
impl<T: Pop + ?Sized> Pop for &mut T {
    fn pop(&mut self, out: &mut [f32]) -> usize {
        (**self).pop(out)
    }
    fn available(&self) -> usize {
        (**self).available()
    }
    fn discard(&mut self, n: usize) -> usize {
        (**self).discard(n)
    }
}
impl<T: Heartbeat + ?Sized> Heartbeat for &T {
    fn heartbeat(&self) -> u64 {
        (**self).heartbeat()
    }
    fn gaps(&self) -> u64 {
        (**self).gaps()
    }
}

/// 没有降噪总线时的替身：推多少收多少（扔掉），取永远是 0。
#[derive(Clone, Copy, Debug, Default)]
pub struct NullRing;

impl Push for NullRing {
    fn push(&mut self, x: &[f32]) -> usize {
        x.len()
    }
}

impl Pop for NullRing {
    fn pop(&mut self, _out: &mut [f32]) -> usize {
        0
    }
    fn available(&self) -> usize {
        0
    }
    fn discard(&mut self, _n: usize) -> usize {
        0
    }
}

/// 没有降噪总线时的心跳：永远不动（= 一直送原声）。
#[derive(Clone, Copy, Debug, Default)]
pub struct NoBeat;

impl Heartbeat for NoBeat {
    fn heartbeat(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod tests;
