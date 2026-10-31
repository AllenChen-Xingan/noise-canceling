# 派活记录 · 013 RL 环境与分场景训练

> 2026-10-31 · rlenv。完整提示词存在会话目录 `workflows/scripts/rlenv-013-research-wf_113bb80a-4ec.js`，这里只记要点。

## 第一轮：Workflow `wf_113bb80a-4ec`（10 代理，pipeline：每路调研 → 该路怀疑派）

| 路 | 范围 | 原文 · 核对 |
|---|---|---|
| A 环境 | exp006 缺口 · 域随机化范围 · 内置麦与系统处理近似 · 数据许可 · sim-to-real 校准 · 机器人 / 自驾先例 | `notes/A-env.md` · `notes/A-env-skeptic.md` |
| B 奖励 | 指标可微性 / 成本 / 参考 / 被讨好方式 · 奖励形态 · Goodhart 案例与对策 · ASR 当奖励 | `notes/B-reward.md` · `notes/B-reward-skeptic.md` |
| C 策略 | 黑盒优化 / bandit / RL 的适用边界 · 助听器先例 · 离线评估 · 工具许可 | `notes/C-policy.md` · `notes/C-policy-skeptic.md` |
| D 微调 | RL-SE / MetricGAN 系 / DPO-GRPO SE · GTCRN 训练代码 · MPS · 专家 vs 条件 · 吃字感知损失 | `notes/D-finetune.md` · `notes/D-finetune-skeptic.md` |
| E 蒸馏 | 非因果老师 → 因果学生 · 学生结构 · 算力与价格 · 数据许可与存储 | `notes/E-distill.md` · `notes/E-distill-skeptic.md` |

## 第二轮：单代理（D47 后补）

| 范围 | 原文 |
|---|---|
| Personal VAD（论文 / 实现 / 参数量 / 数据 / MPS）与最小实验草案 | `notes/F-pvad.md` |

## 共用护栏

- 子代理**不许构建、安装或运行任何程序**，不许写文件、不许克隆；只读本机与上网
- 证据分级：【官方】【论文】【源码 文件:行号】【二手】【推算】；查不到写「未能核实」
- 调研 ≤ 2500 字、怀疑派 ≤ 1800 字；必须给判断、必须写推翻了哪条预判（`must-know.md` 第四节）
- **冲突时以怀疑派核对为准**；合成进 `gap.md` · `decision.md` 时再逐条复核关键数字
