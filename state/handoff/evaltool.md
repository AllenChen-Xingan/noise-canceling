# 现状页 · evaltool（批次 20261029-b1）

- **状态**：进行中
- **手上**：004 评测工具链用哪些现成的（三把尺子各用什么工具和数据量）
- **交代**：`_tmp/20261029-b1/evaltool.md`
- **做到哪**（2026-10-29）：四份文件（must-know/sources/gap/decision）已填完，取舍卡草稿已报大总管；3 个对抗视角子代理（怀疑派/实现派/产品派）自审中
- **下一步**：按自审意见修 → 报「evaltool 可以收尾」

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| state/handoff/evaltool.md | 新建现状页 |
| docs/experiments/exp002-eval-smoke/{README.md,smoke.py,results.json} | DNSMOS/PESQ/STOI/SI-SDR/LUFS 本机冒烟 |
| docs/research/004_eval-toolchain/{sources,gap,decision,README}.md | 来源登记、差距表（含候选横比与附加延迟四种量法）、取舍卡（6 题待人拍板）、状态勾选 |
| docs/research/004_eval-toolchain/must-know.md | 起草三条命门、决定一句话、排除项、预判表（AI 起草，待人改） |

## 发现
- 大总管会话名为 noise-canceling-69（交代里的 5-app-89 已作废）

- resources/P.808 浅克隆后 1.1 GB（克隆前 API 显示 510 MB）；大总管：先留着，收尾再定
- DNSMOS ONNX 模型（约 1 MB）直接在 DNS-Challenge 仓里，不是 LFS 指针
- dnsmos_local.py:53 不兼容 librosa≥0.10（非 16 kHz 输入报 TypeError），绕法：先自己重采样
- 小信号：含噪样例的 DNSMOS SIG 高于干净样例，SIG 对伤人声可能不灵敏（每类 n=1）

## 待人拍板
见 `docs/research/004_eval-toolchain/decision.md` 末尾 6 题：①麦克风 ②真实录音里旁人声音怎么存 ③NISQA/PESQ 许可处理 ④中文评测集 ⑤评测数据（约 15–20 GB）下载与存放 ⑥SA 数据能否进训练

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
- Workflow wf_14ba77e5-34d 完成（10 代理、0 失败），怀疑派推翻 12 条原结论（如漏了 SCOREQ、DTW 须加 -nfa、jiwer 计数法）
- 补查 SCOREQ（代码 MIT 无 LICENSE 文件、训练集 NISQA TRAIN SIM）、AMI（CC BY 4.0）
- 大总管转 pipeline 须知：附加延迟列入工具链 → gap.md「轻快尺子」四种量法，标缺口
- sources/gap/decision 写完，草稿报大总管 → 自审中
