# 现状页 · evaltool（批次 20261029-b1）

- **状态**：已收尾（待大总管验收）
- **手上**：004 评测工具链用哪些现成的（三把尺子各用什么工具和数据量）
- **交代**：`_tmp/20261029-b1/evaltool.md`
- **做到哪**（2026-10-29）：四份文件填完；exp002 冒烟做完并验收；三视角自审（怀疑派 / 实现派 / 产品派）意见已逐条核实并修完
- **下一步**：等大总管验收；被打回就按缺口补

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| docs/research/004_eval-toolchain/must-know.md | 三条命门、决定一句话、排除项、预判表加核实结果（AI 起草，待人改） |
| docs/research/004_eval-toolchain/sources.md | 自有积累 + 他人积累登记（MOS / ASR / 有参考指标 / 管线 / 数据，含许可与大小） |
| docs/research/004_eval-toolchain/gap.md | 三条命门差距、URGENT 2024 相关表、候选横比、吃字率配方、附加延迟四种量法、旁路交叉淡化评法、它们没做的（含 babble、附和声、编码后、回归门禁） |
| docs/research/004_eval-toolchain/decision.md | 取舍卡：每把尺子主指标 + 交叉、统一前处理、真实录音怎么评、回归门禁形状、11 题待人拍板 |
| docs/research/004_eval-toolchain/README.md | 五份文件状态勾选 |
| docs/experiments/exp002-eval-smoke/{README.md,smoke.py,results.json} | DNSMOS/PESQ/STOI/SI-SDR/LUFS 本机冒烟（含 PESQ 更正） |
| state/handoff/evaltool.md | 本页 |
| 仓外：`resources/` 下 19 个浅克隆（git 忽略）；`_tmp/exp002-venv`（406 MB）、`_tmp/exp002-16k/` | 只读参考 / 实验环境 |

## 发现
- 大总管会话名为 noise-canceling-69（交代里的 5-app-89 已作废）
- resources/P.808 浅克隆后 1.1 GB（克隆前 API 显示 510 MB）；大总管：先留着，收尾再定 → **请大总管定删不删**
- DNSMOS ONNX（约 1 MB）直接在 DNS-Challenge 仓里，不是 LFS 指针
- dnsmos_local.py:53 不兼容 librosa≥0.10（非 16 kHz 输入报 TypeError），绕法：先自己重采样；它建 InferenceSession 不指定执行器，本机实际用 CPU，CoreML 没用上
- python-pesq 标 MIT，但内嵌 ITU C 代码禁止修改、分发（dsp.c:34-75）
- URGENT 2024 的逐句级相关数字其实已在 urgent2026_challenge_track2/BENCHMARK.md:2-21
- SCOREQ 默认走 ONNX，不需要 PyTorch；权重 CC BY 4.0
- DNS 测试集下载脚本有没注释掉的 wget/curl，照跑会真下载

## 待人拍板
见 `docs/research/004_eval-toolchain/decision.md` 末尾 11 题：①麦克风 ②旁人声音怎么存 ③NISQA/PESQ 许可处理 ④中文评测集 ⑤评测数据存放 ⑥要不要加会议编码 ⑦自录集要不要人工转写 ⑧DNSMOS ONNX 与 CI 样本进不进仓 ⑨门禁数值 ⑩SA 数据能否进训练 ⑪收集会议对方反馈的通道

## 转交（范围外，留给大总管）
- 回归门禁的抖动实测、各打分器 Mac 计时（exp002 第二轮）、附加延迟 ①、旁路切换点评测：都排进下一批原型
- 许可白名单检查脚本（scripts/ 下）：下一批

## 流水
- 读交代、共同规矩、手下协议、意向书、设计稿相关章节、六份模板 → 完成
- 报到发 noise-canceling-69 → 成功
- 克隆 19 个候选仓库进 resources/ → 完成（P.808 超 1 GB，已报）
- must-know.md 起草 → 完成
- 人新规矩（经大总管）：调研用 workflow → 计划获批，wf_14ba77e5-34d（10 代理，0 失败），怀疑派推翻 12 条原结论
- exp002：排队拿名额 → 第一次 librosa 报错，改为自己重采样后通过，0.66 s → 名额归还 → 大总管验收通过
- 补查 SCOREQ、AMI、OpenCC、libopus 许可
- 大总管转 pipeline 须知：附加延迟、旁路交叉淡化评法 → 列入 gap.md
- 三视角自审：产品派 P0（缺回归门禁）、怀疑派 P1（逐句数已有、SCOREQ 走 ONNX、PESQ 口径矛盾）、实现派 P1（对齐、真实录音、CI 样本、ONNX 进仓）→ 逐条核实并修完；回归门禁形状按大总管要求写进 decision.md
- 已收尾
