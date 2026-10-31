//! 故障注入：包住任意模型，在指定的 hop 上制造故障，用来离线验证「出事送原声」。
//! 只用于测试与实验；`Stall` 会让线程睡眠，绝不能出现在真实的实时路径上。

#![allow(
    clippy::indexing_slicing,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)] // NRT: 故障注入器只用于测试与实验（Stall 会 sleep），绝不进实时路径

use std::time::Duration;

use crate::model::{Model, ModelError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FaultKind {
    /// 输出整段 NaN
    Nan,
    /// 正常算完后再睡这么久（模拟卡顿 / 来不及）
    Stall(Duration),
    /// 后端报错
    Error,
    /// 输出放大成异常幅度（模拟状态发散）
    Explode,
}

pub struct FaultInjector<M: Model> {
    inner: M,
    /// (hop 序号, 故障)，按 hop 升序
    plan: Vec<(u64, FaultKind)>,
    next: usize,
    hop_index: u64,
}

impl<M: Model> FaultInjector<M> {
    pub fn new(inner: M, mut plan: Vec<(u64, FaultKind)>) -> Self {
        plan.sort_by_key(|p| p.0);
        Self {
            inner,
            plan,
            next: 0,
            hop_index: 0,
        }
    }
}

impl<M: Model> Model for FaultInjector<M> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn hop(&self) -> usize {
        self.inner.hop()
    }
    fn latency_samples(&self) -> usize {
        self.inner.latency_samples()
    }
    fn reset(&mut self) {
        // 只复位被包住的模型；注入计划按全局 hop 序号走，不受复位影响
        self.inner.reset();
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        let idx = self.hop_index;
        self.hop_index += 1;
        let r = self.inner.process(input, output);
        let mut out = r;
        while self.next < self.plan.len() && self.plan[self.next].0 <= idx {
            let (at, kind) = self.plan[self.next];
            self.next += 1;
            if at != idx {
                continue;
            }
            match kind {
                FaultKind::Nan => output.fill(f32::NAN),
                FaultKind::Stall(d) => std::thread::sleep(d),
                FaultKind::Error => out = Err(ModelError::Backend),
                FaultKind::Explode => output.iter_mut().for_each(|v| *v += 1e3),
            }
        }
        out
    }
}
