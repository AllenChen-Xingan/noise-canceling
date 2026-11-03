# Pipeline 加后处理级后的签名（rs-core2，2026-11-02，已编译通过、vdn-engine 原有测试全过）

## 不变（rs-entry 在用）
- `Pipeline::new(engine, io_rate) -> Result<Pipeline<M>, PipelineError>`
- `Pipeline::with_block(engine, io_rate, block) -> Result<Pipeline<M>, PipelineError>`
- `process(&mut self, input, output)`、`latency_samples()`、`set_mode`、`reset`、`stats` 等
- 类型 `Pipeline<M>` 照写即可：多了第二个类型参数 `S`，默认 `()`（不带级）

## 新增
- `pub struct Pipeline<M: Model, S: StageChain = ()>`
- `Pipeline::with_stages(engine, io_rate, block: Option<usize>, stages: S) -> Result<Pipeline<M, S>, PipelineError>`
  - `stages` 用 `vdn_types::stages![a, b]` 拼（= `(a, (b, ()))`），按顺序跑在模型之后、升采样之前（16 kHz）
- `process_ctx(&mut self, input, output, ctx: FrameCtx)`：给后处理级声纹判决等上下文；`ctx.now` 由 Pipeline 填
- `set_stage_params(&mut self, &ParamSet)` · `stages(&self) -> &S` · `last_status(&self) -> Status`
- `PipelineError::StageLatency { stage, samples }`（新变体；对 `PipelineError` 穷举 match 的要补一支）
- vdn-types：`trait StageChain`（`()` 与 `(S, Rest)` 已实现）、宏 `stages!`
- vdn-engine：`Engine::process_status(input, output) -> Status`、`Engine::aligned_dry()`、`Engine::config()`；`Engine::process` 签名与输出不变
- vdn-dsp：`FtzGuard`、`ftz::active()`、`ftz::SUPPORTED`

## 行为变化（rs-entry 需知）
- 模型级送原声改在 Pipeline 用 48 kHz 原声做（原来 Engine 在 16 kHz 做，原声缺 8 kHz 以上）。延迟不变
- Pipeline 输出遇到 NaN / Inf 一律换原声（原来只在淡化中换）

## 自审后补充（2026-11-02）
- `PipelineStats` 多两个计数：`stage_faults`（后处理级输出坏值、整 hop 送原声）、`bad_input`（输入 NaN / Inf 按 0 处理的样本数）。用结构体字面量构造 `PipelineStats` 的要补字段（本 worktree 与 rs-entry 里都没有）
- `FtzGuard::off()`：反向守卫；Engine 用它包住推理
- `reset()` 不再把 `FrameCtx.now` 的流时间归零（单调）
