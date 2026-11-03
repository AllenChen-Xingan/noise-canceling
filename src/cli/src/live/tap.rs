//! 输出端的增益与录音分流（扬声器实时线程上用）。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// 输出端的增益与录音分流（实时线程上用）。
pub struct OutputTap {
    gain: f32,
    rec: Option<rtrb::Producer<f32>>,
    /// 录音队列满了丢掉的样本数
    rec_dropped: Arc<AtomicU64>,
    /// 加增益后超过 ±1、被夹住的样本数
    clipped: Arc<AtomicU64>,
}

impl OutputTap {
    pub fn new(gain: f32, rec: Option<rtrb::Producer<f32>>) -> Self {
        Self {
            gain,
            rec,
            rec_dropped: Arc::new(AtomicU64::new(0)),
            clipped: Arc::new(AtomicU64::new(0)),
        }
    }
    pub fn rec_dropped(&self) -> Arc<AtomicU64> {
        self.rec_dropped.clone()
    }
    pub fn clipped(&self) -> Arc<AtomicU64> {
        self.clipped.clone()
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl OutputTap {
    /// 加增益（夹在 ±1 内并计数），再把这块送进录音队列（满了丢、计数，不等）。不分配、不加锁。
    pub fn apply(&mut self, out: &mut [f32]) {
        if self.gain != 1.0 {
            let mut clipped = 0u64;
            for s in out.iter_mut() {
                let v = *s * self.gain;
                if v.abs() > 1.0 {
                    clipped += 1;
                }
                *s = v.clamp(-1.0, 1.0);
            }
            if clipped > 0 {
                self.clipped.fetch_add(clipped, Ordering::Relaxed);
            }
        }
        if let Some(p) = self.rec.as_mut() {
            let mut lost = 0u64;
            for &s in out.iter() {
                if p.push(s).is_err() {
                    lost += 1;
                }
            }
            if lost > 0 {
                self.rec_dropped.fetch_add(lost, Ordering::Relaxed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 录音队列满了：不阻塞、丢掉并计数
    #[test]
    fn record_queue_full_drops_and_counts() {
        let (tx, _rx) = rtrb::RingBuffer::new(10);
        let mut tap = OutputTap::new(1.0, Some(tx));
        let d = tap.rec_dropped();
        tap.apply(&mut [0.1; 25]);
        assert_eq!(d.load(Ordering::Relaxed), 15);
    }
}
