//! 假环与假心跳：单测和无设备演示用（非实时：内部用 `VecDeque`，push 可能分配）。
#![allow(clippy::disallowed_methods, clippy::disallowed_types)] // NRT: 测试替身，不进实时路径

use std::cell::Cell;
use std::collections::VecDeque;

use crate::{Beat, Heartbeat, Pop, Push};

/// 有界假环（同一线程内用）。
#[derive(Debug, Default, Clone)]
pub struct FakeRing {
    buf: VecDeque<f32>,
    cap: usize,
}

impl FakeRing {
    pub fn new(cap: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(cap),
            cap,
        }
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
    /// 全部取出
    pub fn drain_all(&mut self) -> Vec<f32> {
        self.buf.drain(..).collect()
    }
}

impl Push for FakeRing {
    fn push(&mut self, x: &[f32]) -> usize {
        let n = x.len().min(self.cap.saturating_sub(self.buf.len()));
        self.buf.extend(x.iter().take(n));
        n
    }
}

impl Pop for FakeRing {
    fn pop(&mut self, out: &mut [f32]) -> usize {
        let n = out.len().min(self.buf.len());
        for (o, v) in out.iter_mut().zip(self.buf.drain(..n)) {
            *o = v;
        }
        n
    }
    fn available(&self) -> usize {
        self.buf.len()
    }
}

/// 假心跳。
#[derive(Debug, Default)]
pub struct FakeBeat(pub Cell<u64>);

impl Heartbeat for FakeBeat {
    fn heartbeat(&self) -> u64 {
        self.0.get()
    }
}

impl Beat for FakeBeat {
    fn beat(&mut self) {
        self.0.set(self.0.get().wrapping_add(1));
    }
}

/// 测试信号：0.5·sin(440 Hz) + 0.3·sin(2 kHz)，从第 `off` 个样本起（docs/plan/010 第十一节 V1）。
pub fn test_tone(rate: u32, n: usize, off: usize) -> Vec<f32> {
    let r = f64::from(rate);
    (0..n)
        .map(|i| {
            let t = (i + off) as f64 / r;
            (0.5 * (2.0 * std::f64::consts::PI * 440.0 * t).sin() + 0.3 * (2.0 * std::f64::consts::PI * 2000.0 * t).sin())
                as f32
        })
        .collect()
}
