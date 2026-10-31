# 现状页 · benchsynth（research/016-bench-synth · issue #41）

大总管：noise-canceling-c4（旧名 noise-canceling-69）

- 2026-10-31 报到；读交代与 common.md
- 2026-10-31 读 D44–D50；开始核实四条诊断
- 2026-10-31 诊断 1–4 核实成立；新发现：全部带人耳答卷的片段（exp003 12 段、exp005 8 段）都是加法混合，真实锚为零
- 2026-10-31 起调研 workflow wf_4b3f7b2d-afd（6 路 + 6 怀疑派），笔记写 docs/research/016_bench-synth/notes/
- 2026-10-31 大总管给编号 exp008；人新指示「可以多弄几个来对比」→ 系统阶梯 ≥ 12（独立模型为主）；派子代理盘点可跑模型 → notes/G-system-ladder.md
- 2026-10-31 exp008 数据侧检查完成（build 20261031-102406-0c67，348 段，不占名额）：S1/S2/S3 全过；S6 合成杯碟、合成音乐「不过」，合成 babble 临界/过，稳态无参照 D0。中途修两处：float wav 文件头含时间戳 → 改样本哈希；电平在设备链前定 → 实测 SNR 偏 6.3 dB → 改到设备链后
- 2026-10-31 调研 workflow 完成（12 代理，6 篇笔记各带怀疑派复核）；名额已拿到（第一轮 12 系统 + 理想掩码），rank.py 全量在跑（后台）；② cargo fetch ③ 小下载 已批，① Hush dylib ④ torch 大总管在问人
- 2026-10-31 benchmark.md、synth-standard.md 正文写完（D0–D4 分级、S1–S12）；待 rank 结果写 mvp.md、decision.md
