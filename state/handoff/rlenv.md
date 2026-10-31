# 现状页 · rlenv（013 强化学习环境与分场景训练方案）

> 分支 `research/013-rl-environment` · worktree `../noise_canceling-rlenv` · 大总管 `noise-canceling-69`

- 2026-10-31 报到；读完交代、common.md、intent、005、D29–D43、exp006（scene worktree）、011、010、007、012（进行中）
- 2026-10-31 建 `docs/research/013_rl-environment/`，写 must-know（决策点）
- 2026-10-31 起 Workflow wf_113bb80a-4ec（5 路调研 × 怀疑派，10 代理）；收到 D44（奖励只用 012 置信度过线的 verifier），写进 must-know 命门 2，已问 evalstd 清单
- 2026-10-31 scene 回复：本批不加内置麦设备模型 / 空间分布；make_clip(job) 纯函数可在线采样（需把 recipes 当参数传）；无真实录音校准计划
- 2026-10-31 收到 D45（能用算法 / 工程解决的先做，训练是慢线）：decision 排序改为 策略/参数搜索最前 → 最小实验选本机最便宜 → 专家训练与蒸馏写触发条件
- 2026-10-31 evalstd 回复：清单将落 evalstd worktree docs/research/012_eval-standards/verifiers.md；过线 = 人耳系统级 SRCC 95% 下界 ≥0.6 且成对一致率 ≥70%（攒 20–30 对前一律「未定标」，只作约束/报警）；吃字一票否决、延迟算力作可行域与我一致；MOS 预测器不能判小差距
- 2026-10-31 读 evalstd verifiers.md：L0–L3 四级；今天感知类全 L0 → 第一版奖励 = 物理量可行域 + V-EAT-ASR 一票否决 + 其余「不比基线差」约束；§2.3 第 3 条（训练用过的 verifier 不参与其定标）照守
- 2026-10-31 收到 D47：第一个最小实验 = Personal VAD 式辅助小模型（模拟真值 + 教师蒸馏 + 按过线 verifier 用 RL 调；对照算法版辅助判断）；派 1 个代理查 PVAD 论文/实现/MPS 可行性
