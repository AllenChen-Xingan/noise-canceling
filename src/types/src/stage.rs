//! 后处理级（门控、衰减下限、音色链节点），跑在模型采样率（规约 §3.3）。

use crate::frame::FrameCtx;
use crate::params::ParamSet;

pub trait Stage: Send {
    /// 短名，进延迟账与决策日志
    fn name(&self) -> &'static str;
    /// 状态清回初值；可在实时线程调
    fn reset(&mut self);
    /// 原地处理一个 hop。`wet` 是模型输出，`dry` 是对齐后的原声。不分配、不 panic
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
    /// 本级引入的延迟（模型采样率样本）。规约 §3.3：在「每级 dry 对齐」做出来之前必须为 0；
    /// 修音链零前瞻，带宽扩展例外（≤ 1 ms，本批硬约束 1）
    fn latency_samples(&self) -> usize {
        0
    }
    /// 策略下发的参数（engine 平滑后逐 hop 调用，实时线程，不分配）。用不到的级默认忽略
    fn set_params(&mut self, _p: &ParamSet) {}
}

/// 一串按固定顺序静态调用的后处理级（处理图第一步，规约 §3.3「后处理链按固定顺序静态调用」· D66）。
///
/// 写法是嵌套二元组的「链表」：`()` 是空串，`(S, Rest)` 表示先跑 `S` 再跑 `Rest`，可嵌套任意长；
/// 用 [`stages!`](crate::stages) 宏写成 `stages![a, b, c]` = `(a, (b, (c, ())))`。
/// 选它而不选 `Vec<Box<dyn Stage>>` 或定长数组：类型在编译期定死，逐级静态分发、可内联，实时路径不分配、无虚调用，
/// 也允许每级是不同类型（门控、衰减下限、修音链节点各是各的 struct）。代价：换一串级要换类型（新 Pipeline，在非实时线程建）。
///
/// 实时方法（`process_all` / `reset_all` / `set_params_all`）不分配、不 panic；`describe` 是构造期用的。
pub trait StageChain: Send {
    /// 级数
    const LEN: usize;
    /// 按顺序原地处理一个 hop（模型采样率）
    fn process_all(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
    /// 每级状态清回初值
    fn reset_all(&mut self);
    /// 每级参数下发
    fn set_params_all(&mut self, p: &ParamSet);
    /// 各级延迟之和（模型采样率样本）
    fn latency_samples_all(&self) -> usize;
    /// 按顺序把每级的短名与延迟交给 `f`（建延迟账、写会话记录用；构造期）
    fn describe(&self, f: &mut dyn FnMut(&'static str, usize));
}

impl StageChain for () {
    const LEN: usize = 0;
    fn process_all(&mut self, _wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {}
    fn reset_all(&mut self) {}
    fn set_params_all(&mut self, _p: &ParamSet) {}
    fn latency_samples_all(&self) -> usize {
        0
    }
    fn describe(&self, _f: &mut dyn FnMut(&'static str, usize)) {}
}

impl<S: Stage, R: StageChain> StageChain for (S, R) {
    const LEN: usize = 1 + R::LEN;
    fn process_all(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx) {
        self.0.process(wet, dry, ctx);
        self.1.process_all(wet, dry, ctx);
    }
    fn reset_all(&mut self) {
        self.0.reset();
        self.1.reset_all();
    }
    fn set_params_all(&mut self, p: &ParamSet) {
        self.0.set_params(p);
        self.1.set_params_all(p);
    }
    fn latency_samples_all(&self) -> usize {
        self.0.latency_samples() + self.1.latency_samples_all()
    }
    fn describe(&self, f: &mut dyn FnMut(&'static str, usize)) {
        f(self.0.name(), self.0.latency_samples());
        self.1.describe(f);
    }
}

/// 把几级拼成 [`StageChain`]：`stages![a, b]` = `(a, (b, ()))`，`stages![]` = `()`。
#[macro_export]
macro_rules! stages {
    () => { () };
    ($head:expr $(, $rest:expr)* $(,)?) => { ($head, $crate::stages!($($rest),*)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Add(f32, &'static str);
    impl Stage for Add {
        fn name(&self) -> &'static str {
            self.1
        }
        fn reset(&mut self) {}
        fn process(&mut self, wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {
            wet.iter_mut().for_each(|v| *v = *v * 2.0 + self.0);
        }
    }

    #[test]
    fn chain_runs_in_order() {
        // (x·2 + 1)·2 + 10，顺序反了结果不同
        let mut c = crate::stages![Add(1.0, "a"), Add(10.0, "b")];
        let mut w = [1.0f32, 0.0];
        c.process_all(&mut w, &[0.0, 0.0], FrameCtx::default());
        assert_eq!(w, [16.0, 12.0]);
        assert_eq!(<(Add, (Add, ())) as StageChain>::LEN, 2);
        let mut names = Vec::new();
        c.describe(&mut |n, l| names.push((n, l)));
        assert_eq!(names, [("a", 0), ("b", 0)]);
        assert_eq!(c.latency_samples_all(), 0);
    }

    #[test]
    fn empty_chain_is_noop() {
        let mut c = crate::stages![];
        let mut w = [3.0f32];
        c.process_all(&mut w, &[0.0], FrameCtx::default());
        assert_eq!(w, [3.0]);
        assert_eq!(<() as StageChain>::LEN, 0);
    }
}
