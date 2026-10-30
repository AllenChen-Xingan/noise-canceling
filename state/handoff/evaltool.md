# 现状页 · evaltool（批次 20261029-b1）

- **状态**：进行中
- **手上**：004 评测工具链用哪些现成的（三把尺子各用什么工具和数据量）
- **交代**：`_tmp/20261029-b1/evaltool.md`
- **做到哪**（2026-10-29）：must-know 已起草；19 个候选仓库已浅克隆；Workflow（5 路检索 + 5 路对抗核实，wf_14ba77e5-34d）在跑
- **下一步**：Workflow 回来后综合进 sources/gap/decision；自审

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| state/handoff/evaltool.md | 新建现状页 |
| docs/experiments/exp002-eval-smoke/{README.md,smoke.py,results.json} | DNSMOS/PESQ/STOI/SI-SDR/LUFS 本机冒烟 |
| docs/research/004_eval-toolchain/must-know.md | 起草三条命门、决定一句话、排除项、预判表（AI 起草，待人改） |

## 发现
- 大总管会话名为 noise-canceling-69（交代里的 5-app-89 已作废）

- resources/P.808 浅克隆后 1.1 GB（克隆前 API 显示 510 MB）；大总管：先留着，收尾再定
- DNSMOS ONNX 模型（约 1 MB）直接在 DNS-Challenge 仓里，不是 LFS 指针
- dnsmos_local.py:53 不兼容 librosa≥0.10（非 16 kHz 输入报 TypeError），绕法：先自己重采样
- 小信号：含噪样例的 DNSMOS SIG 高于干净样例，SIG 对伤人声可能不灵敏（每类 n=1）

## 待人拍板
（暂无）

## 转交（范围外，留给大总管）
（暂无）

## 流水
- 读交代、共同规矩、手下协议、意向书、设计稿相关章节、六份模板 → 完成
- 报到发 noise-canceling-69 → 成功，大总管已确认会话名
- 克隆 19 个候选仓库进 resources/ → 完成
- must-know.md 起草 → 完成
- 人新规矩（经大总管）：调研用 workflow → 计划报大总管并获批，wf_14ba77e5-34d 启动
- 申请名额做 exp002 冒烟 → 排队后获批
- exp002：venv 406 MB 在 _tmp/；第一次跑 librosa 报错，改为自己重采样后通过，总耗时 0.66 s → 名额已归还
