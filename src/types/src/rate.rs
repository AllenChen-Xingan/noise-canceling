//! 采样率的正本（issue #62）：全 workspace 只在这里写 48 kHz 与 16 kHz 两个数。
//!
//! 别处一律引用这两个常量；不能依赖 `vdn-types` 的底层 crate 若在非测试代码里留了自己的常量，
//! 必须有一条测试断言它与这里相等（CLAUDE.md 第五节第 5 条）。`xtask tidy` 的 R-RATE 挡新增字面量。

/// 链路 IO 采样率（Hz）：麦克风、扬声器、共享内存、虚拟麦克风（docs/plan/010 第十节）。
pub const IO_RATE: u32 = 48_000;

/// 降噪模型与说话人模型的采样率（Hz）：GTCRN · FastEnhancer · VAD · 声纹。
pub const MODEL_RATE: u32 = 16_000;

#[cfg(test)]
mod tests {
    use super::*;

    /// 两者是整数倍：重采样按 3:1 抽取 / 插值（vdn-dsp）
    #[test]
    fn io_rate_is_integer_multiple_of_model_rate() {
        assert_eq!(IO_RATE % MODEL_RATE, 0);
        assert_eq!(IO_RATE / MODEL_RATE, 3);
    }
}
