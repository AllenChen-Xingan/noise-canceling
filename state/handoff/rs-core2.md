# 现状页 · 手下 rs-core2（批次 20261102-b8-rust）

> 管的实体：`vdn-types` · `vdn-dsp` · `vdn-engine` · `vdn-models`
> 分支 `feat/rust-skeleton-core`，worktree `noise_canceling-rs-core`
> 交代：`../noise_canceling-skeleton/_tmp/20261102-b8-rust/rs-core2.md`

## 手上
- 进行中：Stage 串进 Pipeline · 送原声改 48 kHz · 延迟断言 · FTZ 守卫

## 做到哪
1. Stage 串进 Pipeline：完成。签名见 `state/handoff/rs-core2-pipeline-signature.md`，已发大总管与 rs-entry，大总管认可
2. 送原声改 48 kHz：完成，故障注入测试过；变异检查（让 Pipeline 忽略 Status）该测试会红
3. 延迟断言改为以 Pipeline 为正本：完成
4. FTZ 守卫：完成，本机 arm64 实测守卫内冲 0、守卫外不冲
5. 回归：`vdn denoise` 4 个场景字节一致；四 crate 测试全过，clippy 无警告，tidy / check-deps 通过

## 下一步
- 三视角自审（子代理跑着），修完收尾

## 已定（大总管 2026-11-02）
- dsp 进 unsafe 白名单，原因 FTZ（arm64 读写 FPCR 只能用内联汇编）。lib.rs 改 `#![deny(unsafe_code)]`，只在 ftz 模块放开；非 aarch64 / x86_64 与 `cfg(miri)` 下守卫是空操作。规约 R10 白名单由大总管收尾时更新

## 流水
- 出改动前基准：`vdn denoise` 4 份（默认 / --no-safety / 故障注入 / --no-fast-open），放 `_tmp/20261102-b8-rust/core2/`
- vdn-types 加 `StageChain` + `stages!`（只加不改）；单测 2 条过
- Engine 拆 `run`，加 `process_status`；老 `process` 输出不变
- 写 `vdn-dsp/src/ftz.rs`，dsp 改 unsafe = true
- 重写 Pipeline（后处理级 · 48 kHz 送原声 · FTZ）；原有 24 条测试过；发签名
- 新测试 `src/engine/tests/stages.rs` 11 条过；变异检查有效
- 改 `src/models/tests/latency.rs`：不抄数字，从 Pipeline 取
- 回归对拍 4 份字节一致；fmt、四 crate 测试、clippy、tidy、check-deps 全过
- 起三个子代理自审
- 报到；读材料
- 请示 vdn-dsp unsafe；大总管同意自写
