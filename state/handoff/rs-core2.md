# 现状页 · 手下 rs-core2（批次 20261102-b8-rust）

> 管的实体：`vdn-types` · `vdn-dsp` · `vdn-engine` · `vdn-models`
> 分支 `feat/rust-skeleton-core`，worktree `noise_canceling-rs-core`
> 交代：`../noise_canceling-skeleton/_tmp/20261102-b8-rust/rs-core2.md`

## 手上
- 进行中：Stage 串进 Pipeline · 送原声改 48 kHz · 延迟断言 · FTZ 守卫

## 做到哪
- 读完交代、规约 §3.3 / §3.6 / §3.7.6 / §5.2、审查报告第 5 节、前任现状页

## 下一步
- 写 Stage 串（静态分发）与 Pipeline 接入

## 已定（大总管 2026-11-02）
- dsp 进 unsafe 白名单，原因 FTZ（arm64 读写 FPCR 只能用内联汇编）。lib.rs 改 `#![deny(unsafe_code)]`，只在 ftz 模块放开；非 aarch64 / x86_64 与 `cfg(miri)` 下守卫是空操作。规约 R10 白名单由大总管收尾时更新

## 流水
- 报到；读材料
- 请示 vdn-dsp unsafe；大总管同意自写
