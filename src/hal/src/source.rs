//! 读出端：插件 IO 线程每次要一块声音，从源里取；取不够补零并计欠载。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// 单消费者样本源。将来由 `vdn-shm` 的 `Out` 环消费端实现；本批测试用假源。
///
/// 实现者在 `pop` 里不许分配、加锁、阻塞（A 档）。
pub trait SampleSource {
    /// 尽量填满 `out`，返回实际写入的样本数（≤ `out.len()`）。没有数据返回 0，不阻塞。
    fn pop(&mut self, out: &mut [f32]) -> usize;
}

/// 一次 `render` 的结果。
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderStatus {
    /// 整块都是源里的声音
    Full,
    /// 源里只有 `got` 个样本，其余补了零
    Underrun { got: usize },
}

/// 欠载计数：实时线程写，任何线程读。
#[derive(Debug, Default)]
pub struct HalCounters {
    underruns: AtomicU64,
    underrun_samples: AtomicU64,
    renders: AtomicU64,
}

impl HalCounters {
    /// 发生欠载的块数
    pub fn underruns(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }
    /// 补零的样本总数
    pub fn underrun_samples(&self) -> u64 {
        self.underrun_samples.load(Ordering::Relaxed)
    }
    /// `render` 调用次数
    pub fn renders(&self) -> u64 {
        self.renders.load(Ordering::Relaxed)
    }
}

/// 虚拟麦克风的读出端。
pub struct HalSource<S> {
    source: S,
    counters: Arc<HalCounters>,
}

// 构造期：可以分配。
impl<S: SampleSource> HalSource<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            counters: Arc::new(HalCounters::default()),
        }
    }

    /// 计数器句柄（可交给别的线程读）。
    pub fn counters(&self) -> Arc<HalCounters> {
        Arc::clone(&self.counters)
    }

    /// 取回源（测试用）。
    pub fn into_inner(self) -> S {
        self.source
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl<S: SampleSource> HalSource<S> {
    /// 填满 `out`。源给多少用多少，剩下补零；空块（`out` 长 0）算 `Full`。
    pub fn render(&mut self, out: &mut [f32]) -> RenderStatus {
        self.counters.renders.fetch_add(1, Ordering::Relaxed);
        let mut got = 0;
        // 源可能分段给（环形缓冲绕回），循环到取不到为止
        while got < out.len() {
            let Some(rest) = out.get_mut(got..) else {
                break;
            };
            let n = self.source.pop(rest).min(rest.len());
            if n == 0 {
                break;
            }
            got += n;
        }
        if got == out.len() {
            return RenderStatus::Full;
        }
        if let Some(rest) = out.get_mut(got..) {
            rest.fill(0.0);
        }
        self.counters.underruns.fetch_add(1, Ordering::Relaxed);
        self.counters
            .underrun_samples
            .fetch_add((out.len() - got) as u64, Ordering::Relaxed);
        RenderStatus::Underrun { got }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 假源：手里有一串样本，每次最多给 `chunk` 个（模拟环绕回）。
    struct Fake {
        data: Vec<f32>,
        pos: usize,
        chunk: usize,
    }

    impl SampleSource for Fake {
        fn pop(&mut self, out: &mut [f32]) -> usize {
            let n = out.len().min(self.chunk).min(self.data.len() - self.pos);
            out[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            n
        }
    }

    fn fake(n: usize, chunk: usize) -> Fake {
        Fake {
            data: (1..=n).map(|i| i as f32).collect(),
            pos: 0,
            chunk,
        }
    }

    #[test]
    fn full_block_when_source_has_enough_even_in_pieces() {
        let mut h = HalSource::new(fake(10, 3));
        let mut out = [0.0; 8];
        assert_eq!(h.render(&mut out), RenderStatus::Full);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        assert_eq!(h.counters().underruns(), 0);
    }

    #[test]
    fn underrun_pads_zero_and_counts() {
        let mut h = HalSource::new(fake(10, 4));
        let mut out = [9.0; 8];
        assert_eq!(h.render(&mut out), RenderStatus::Full);
        let mut out = [9.0; 8];
        assert_eq!(h.render(&mut out), RenderStatus::Underrun { got: 2 });
        assert_eq!(out, [9.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let mut out = [9.0; 4];
        assert_eq!(h.render(&mut out), RenderStatus::Underrun { got: 0 });
        assert_eq!(out, [0.0; 4]);
        let c = h.counters();
        assert_eq!(
            (c.underruns(), c.underrun_samples(), c.renders()),
            (2, 10, 3)
        );
    }

    #[test]
    fn empty_block_is_full() {
        let mut h = HalSource::new(fake(0, 1));
        assert_eq!(h.render(&mut []), RenderStatus::Full);
    }

    /// 源实现有 bug、报的数比给的空间大：不越界、不 panic。
    #[test]
    fn lying_source_does_not_overrun() {
        struct Liar;
        impl SampleSource for Liar {
            fn pop(&mut self, out: &mut [f32]) -> usize {
                out.fill(1.0);
                out.len() + 100
            }
        }
        let mut h = HalSource::new(Liar);
        let mut out = [0.0; 4];
        assert_eq!(h.render(&mut out), RenderStatus::Full);
    }
}
