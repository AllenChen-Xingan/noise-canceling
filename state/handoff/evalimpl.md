# 现状页 · evalimpl（批次 20261031-b4）

分支 `feat/eval-verifiers` · worktree `noise_canceling-evalimpl` · 大总管 `noise-canceling-69`

## 记录
- 2026-10-31 报到；读完 Anthropic《Demystifying evals for AI agents》、012 verifiers/decision、rubric 第 2/7 节、exp003 答卷与 scores.json、exp005 人耳反馈
- 发现：exp003 的候选音频已不在盘上（只剩 scores.json 与答卷）→ 定标先用 scores.json 现有指标；exp005 三轨素材在主目录 `_tmp/20261031-exp005/src/`（含 owner/interf/noise/mix + 时间线 + 原文）→ 作 QA 层真实任务
- 设计：xtask 不许依赖 vdn crate → `xtask eval ci` 转调 `cargo run -p vdn-eval -- ci`；CI 无模型权重 → CI 层被测系统 = 真 Engine（三件保命）+ crate 内参考降噪器（维纳，用 vdn-dsp STFT）；本机有 GTCRN 权重时自动加跑
- 开写 src/eval/
