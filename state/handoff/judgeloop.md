# 现状页 · judgeloop（018 本机评分 + 调节模型闭环）

- 分支：`research/018-ondevice-judge` · issue #44 · 大总管：noise-canceling-e3（批次 20261102-b7）
- 状态：进行中

## 做到哪
10-31 tmux 断掉，旧 workflow 中断（只写出 r3、r4）。11-02 重派：补 r1 · r2 · r5 workflow 在跑（wf_3d6391bc-bd2，各配怀疑派）；之后写 decision.md，exp011 等 benchsynth/ears 与名额（排第三）。

## 流水
- 2026-10-31 读交代 · common · CLAUDE.md · intent/cog · pending · D51–D53；向 c4 报到

## 待人拍板
（暂无）

## 发现 / 转交
- common.md 写大总管是 noise-canceling-69，首条消息为 c4，按 c4
- `specs/rubric/preferences.md` 在 main 不存在（prefs PR #43 未合）
- 2026-10-31 c4 给实验编号 exp011（docs/experiments/exp011-ondevice-loop/）；向 c4 报 workflow 计划（7 代理）
- 2026-10-31 起 workflow wf_ff431f0b-3ee：R1 评分模型小型化 · R2 评分当奖励先例 · R3 声音→参数 / 助听器偏好学习 · R4 护栏 · R5 本机盘点；R1–R4 各配一个怀疑派
- 2026-10-31 问 ears（评分模型起点 / 本机现成权重）· benchsynth（exp008 合成格位置、真值、好坏配对）；读 exp006 offline_ab.py、route.rs、polish 预设，exp011 初步思路：GTCRN 跑一次，参数（strength · tone 预设 · 下限）后处理叠加，逐窗 bandit 用评分模型当奖励，另一评分模型留出防讨好，红线 verifier 一票否决，对照 = 路由表 / 固定默认 / 事后最优
- 2026-10-31 ears 回：起点倾向 SCOREQ（URGENT24 语句 SRCC 0.789 / 系统 0.926；CHiME-7 真实 0.744 / 0.90），DNSMOS 在 CHiME-7 语句 0.232、系统 −0.10；本机现成 = DNSMOS P.835（resources/DNS-Challenge…，exp002-venv）· NISQA v2（要 torch）；SCOREQ 待 ears 装（_tmp/exp009-venv）。出处：ears worktree docs/experiments/exp009-more-ears/results/crowd_ears.md
- 2026-10-31 benchsynth 回：exp008 = 348 段（2 生成器 × 29 格档 × 6），mix/owner/interf/noise 分轨，_tmp/exp008/20261031-102406-0c67/；owner 轨是过 RIR+设备链后的主讲人；逐帧标签要自己重算加 ±40 ms guard；A 部分与 exp006 训练集不隔离 → 查表对照给「真值场景」（对查表最有利的上界），或只用 exp006 留出 build fed2；好坏配对未实现
- 2026-11-02 新会话接续（批次 b7）：读 judgeloop.md / common.md / 旧交代 / D56–D65；向 e3 报到，e3 同意计划；起 workflow wf_3d6391bc-bd2（r1 答 D60 SCOREQ 蒸馏 · r2 · r5，各配怀疑派，共 6 代理）
