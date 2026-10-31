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
- 2026-10-31 Workflow 完成，10 份笔记落 notes/；写完 assign · sources · gap；等 PVAD 代理后写 decision
- 2026-10-31 写完 decision · README，登记研究索引；开始自审（3 个对抗视角）
- 2026-10-31 evalstd 更新：L2 改为片段内 Kendall τ 下界 ≥0.2；同策略审计（每轮听 3 段奖励最高输出）；吃字否决用 Δ(del+sub)、段数不足视为不通过；感知类到 L2 约 5–6 轮——待自审后一并改 decision
- 2026-10-31 evalstd 同意模拟真值指标（V-SIM-GATE / V-SIM-FA / V-SIM-SWLAT，L3 仅限模拟），三限定：真实上换人耳/教师并量落差；只作门控子模块目标，输出仍过吃字否决 + V-ONSET + V-BGSTAB + 同策略审计；真值规则写死进仓且与模型输入不同源
- 2026-10-31 012 已合并 main（PR #27）；注意：V-SIM-GATE/FA/SWLAT（§3.4）是合并后 evalstd worktree 的提交 5f9085f，main 上还没有——收尾时告诉大总管
- 2026-10-31 三视角自审（事实 · 实现 · 符合度）完成，P0 5 条、P1 十余条已改进 decision；must-know · gap · sources 同步
- 2026-10-31 已发 scene（模拟器前置与域随机化清单）、p2（babble 下快开不触发的吃字漏洞；辅助模型复用声纹线程）

## 待人拍板（详见 docs/research/013_rl-environment/decision.md 末尾）
1 训练在哪（建议本机）· 2 数据下载存放（建议先 train-clean-100 约 6 GB）· 3 许可灰区与 ECAPA（建议只作推理条件输入，与 005 #2 同定）· 4a 回放重录（建议做）· 4b 咖啡馆实录（建议做，本机 30 天删）· 5 模拟真值指标沿用 012 §3.4（确认）· 6 成功标准先冻结方向 · 7 训练触发条件 · 8 D47 强化学习做到哪（建议先黑盒调门控，平台期再 RL 微调权重）· 9 人耳听测节奏（5–6 轮 × 35 分钟）

## 已收尾（2026-10-31）
交付：docs/research/013_rl-environment/（README · must-know · sources · assign · gap · decision · notes/）
- 2026-10-31 p2 回复：已有持续 babble 快开（48 ms +8 dB、≥使用者电平 −3 dB），c02 句首仍误压 2.5–11%；接口 Enroller::owner() 192 维、判决滞后 1.5–2 s——已写进 decision §4.1
