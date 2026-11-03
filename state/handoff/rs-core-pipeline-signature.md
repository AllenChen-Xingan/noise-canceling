# Pipeline 最终签名（rs-core，2026-11-02，已编译通过）

crate `vdn-engine`，`use vdn_engine::{Pipeline, Mode, PipelineError, PipelineStats, Engine, SafetyConfig};`

```rust
impl<M: Model> Pipeline<M> {
    // 构造（非实时，会分配）
    pub fn new(engine: Engine<M>, io_rate: u32) -> Result<Self, PipelineError>; // io_rate 必须 = 模型采样率 × 3（48000）
    pub fn latency_ledger(&self) -> LatencyLedger;                              // 48 kHz 口径，合计 = latency_samples()
    // 实时（不分配、不加锁、不 panic）
    pub fn process(&mut self, input: &[f32], output: &mut [f32]);               // 任意块长，**无上限**；输出与输入等长
    pub fn set_mode(&mut self, mode: Mode);                                     // 按一个 48k hop（768 样本 = 16 ms）线性淡化
    pub fn mode(&self) -> Mode;
    pub fn latency_samples(&self) -> usize;                                     // 48 kHz 样本，两模式相同
    pub fn io_rate(&self) -> u32;
    pub fn hop_io(&self) -> usize;                                              // 模型 hop × 3
    pub fn stats(&self) -> PipelineStats;                                       // { hops, bad_len }
    pub fn engine(&self) -> &Engine<M>;
    pub fn reset(&mut self);
}
pub enum Mode { Denoise /*默认*/, Passthrough }
pub enum PipelineError { UnsupportedRate { io_rate: u32, model_rate: u32 } }   // 实现 Display + Error

// 无权重时：
let p = Pipeline::new(Engine::new(vdn_engine::testing::Identity, SafetyConfig::off()), 48_000)?;
```

延迟（48 kHz 样本）= 重分块 (hop×3 − 1) + 重采样 96 + 模型延迟×3。
- Identity：767 + 96 + 0 = **863**（17.98 ms）
- GTCRN：767 + 96 + 768 = **1631**（33.98 ms）

和 010 第十节的差别：没有 `max_block`（内部缓冲只按 hop 定，任意块长都行，是约定的超集）。
`process` 不收 `FrameCtx`（与 010 第十节一致；规约 §3.3 示意里的 ctx 归后续批次）。
