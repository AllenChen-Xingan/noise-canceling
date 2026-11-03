# 现状页 · 手下 applebe（原名 engine）· 分支 research/019-apple-inference · issue #45 · 实验 exp010

## 批次 20261102-b8（当前）
- 2026-11-02 报到大总管 noise-canceling-e3，计划获同意
- 管的实体：推理后端（苹果）· 模型（降噪）· 评测器（帧耗时 · 逐值一致）
- 机器：Apple M5 · macOS 27.0.1 · 插电
- 起调研 workflow wf_60470c8a-e59（4 调研 + 4 怀疑派），notes 写 `docs/research/019_apple-inference/notes/{coreml,bnns-graph,mlx,realtime-and-bindings}.md`
- 等调研期间：写测量程序 `docs/experiments/exp010-inference-bench/apple/`（不跑；跑要名额）
- rustbe 同步：手写 GTCRN 已与 ORT 逐值对拍（37 段，最大差 1.6e-4），正在加 tract

## 下一步
1. 调研收回 → 申请名额：装 `_tmp/exp010-venv/`（coremltools · mlx）+ 跑基准
2. 写 `decision.md`，对抗自审一轮

## 转交
（无）

## 批次 20261031-b6（旧）
- workflow wf_502b2e03-c65 中断，只写出 `notes/our-models.md`
