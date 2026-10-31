# 现状页 · evalstd（批次 20261031-b4）

- **状态**：已收尾（待大总管验收）
- **手上**：012 工业级评测标准调研（ITU-T / ETSI / 3GPP / ITU-R / IEC / 认证 / 测量公司 / 比赛协议 / 助听器）+ 对 004 与 007 的差距表 + 升级清单
- **交代**：主目录 `_tmp/20261031-b4/evalstd.md`
- **交付**：`docs/research/012_eval-standards/`（六份）

## 流水
- 2026-10-30 读交代、common.md、意向书、004 decision、`specs/rubric/denoise.md`、007 sources → 完成
- 报到 noise-canceling-69 → 成功；大总管回同批会话名（scene 对齐 ES 202 396-1）
- 建 `docs/research/012_eval-standards/`（编号按交代用 012，research.sh 会编成 010，故手建）
- Workflow wf_3e12ea68-f47 起（12 代理：6 路调研 × 怀疑派），大总管已批
- Workflow 完成：12/12，0 失败；怀疑派纠正了约 30 处（版本、条款号、证据等级），原始报告在会话 scratchpad `wf/`（不进仓，结论已折进 sources/gap）
- 自己补核：ITU-T G.160 (06/12) 附录 II（SNRI/TNLR/DSN，有推荐值和 ANSI-C 参考码，数值未读到——PDF 下载 500）；3GPP TS 26.077 现行 19.0.0
- 下载 Microsoft Teams V5 规范 ZIP（19 MB）到主目录 `resources/teams-v5-spec/`（git 忽略；sha256 5a6863cd…d7df）；读了 Audio Spec 的通过线表、§6.21/6.26/6.35/8.1
- 写 sources.md → 进行中
- D44（大总管转人）：加 `verifiers.md`（主观项拆解 + verifier 清单 + 人耳回灌校准）；回 rlenv：文件路径、参考/成本/可微三栏、过线阈值（L2 = Spearman 95% 下界 ≥ 0.5 且 ≥ 30 段，另需 L1）
- 写完 gap.md · verifiers.md · decision.md · README.md；Teams 规范 README 写进 `resources/teams-v5-spec/README.md`
- 自审 Workflow wf_3cac798e-29c（3 代理：怀疑派 / 实现派 / 产品派）→ 进行中

## 待人拍板
见 `docs/research/012_eval-standards/decision.md` 末尾 9 题：①自研 TS 103 281 前先核许可 ②人耳加 P.835 三量表 ③加听者（含众包与 D39 的关系）④ETSI 噪声短样本进仓 ⑤对外报告并列 Teams / TS 26.131 数字 ⑥是否设法读 P.835 (07/26) 正文 ⑦「工业级」在本项目指什么 ⑧verifier 判据建议值 ⑨rlenv 过渡期能否用 L1 当弱奖励
- 自审 3 视角回来：怀疑派 0 P0 / 3 P1 / 5 P2；实现派 2 P0（单轮降级会把 verifier 全降光；跨片段相关被片段难度带偏）；产品派 1 P0（RL 刷分查不出）→ 逐条核实，全部采纳并修完（唯一不采纳：V-SNG「要三路输入」——D 路怀疑派已核 TS 103 281 §6.2 只要两路；改为补「全带模型 vs 16 kHz」提醒）
- 已收尾

## 交付
`docs/research/012_eval-standards/`：README · must-know · sources（[S1]–[S44]）· assign · gap · **verifiers**（D44）· decision（升级清单 16 条、rubric 修改清单 R1–R14 交大总管、待人拍板 9 题）

## 转交大总管
- rubric 修改清单 R1–R14（`decision.md`），本线没改 `specs/rubric/denoise.md`
- 第一个该做的实验：用 exp003 现有数据算第一张 verifier 定标表（纯计算）
- `resources/teams-v5-spec/`（19 MB，git 忽略，README 记来源与哈希）
- 应 rlenv 提议，verifiers.md 新增 3.4「模拟真值指标」（L3 仅限模拟，附三个限定）；原 3.4 改为 3.5
