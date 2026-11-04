# DDS（Device-Degraded Speech）设备劣化语音集

- **名称**：DDS: Device-Degraded Speech Dataset（DDS v0.8，Interspeech 2022）
- **链接**：https://zenodo.org/record/5464104 （DAPS 部分）; https://zenodo.org/record/5499506 ; https://zenodo.org/record/5501697 （VCTK 部分两包）
- **许可**：DAPS 部分 Zenodo 许可字段 `cc-by-nc-4.0`，包内 README 原句「This repository is made available under Creative Commons Attribution Non Commercial 4.0 International.」；VCTK 两包许可字段 `odc-by`，包内 README 原句「This repository is made available under Open Data Commons Attribution License (ODC-By) v1.0」→ 结论：只评测（DAPS 部分）；VCTK 部分按署名型口径可训练【官方文档】【推断：ODC-By 不在口径列表里，与 CC BY 同属只要求署名】
- **大小**：1,944 h（3 设备 × 9 环境 × 6 位置 × 12 h 源语音）；只发布 16 kHz 版；DAPS 包 9 个环境各约 5.3–5.7 GB，VCTK 两包合计约 90.9 GB【论文 §1】【官方文档：Zenodo 文件表】
- **场景键**：meeting-room, open-office, home-family, other（两间办公室、两间会议室、三间工作室、一间客厅、一间候客室）
- **设备键**：non-laptop-tablet（iPad Air，MEMS 麦）, non-laptop-studio（两只电容麦：论文图注为 Uber Mic 与 MPM1000）
- **笔记本段**：没有。唯一消费设备是 iPad Air；按设备（`Marantz` / `Uber` / `iPad`）、环境、位置（ch1–ch6 对应 F/D/B/A/C/E，A 为 50 cm 正前方，F 为 200 cm、75°）分目录，文件名 `{uttid}_{env}_{device}_{channel}.flac`，可逐条筛【论文 §3.3、图 1】【官方文档：包内 README】
- **干净参考**：有（与录音棚源语音逐条对齐）【官方文档】
- **转写**：有（`text.zip`，源语料朗读文本；无时间戳）【官方文档：Zenodo 文件表】【推断：内容为 DAPS / VCTK 脚本】
- **语种**：英语
- **语体**：朗读（DAPS 4 h、VCTK 8 h 28 人）；无对话、笑声、附和、情绪
- **获取方式**：直接下载（Zenodo，开放获取）
- **与 D62 的关系**：回放重录：录音棚语音经监听音箱播放，三台设备在 6 个位置（最近 50 cm 正前方）同时重录；环境噪声与混响是真实的。开发阶段若拿来评测：取 iPad Air 通道 + 一两个环境（如 confroom1、office1）的 DAPS 部分做「真实房间 + MEMS 麦 + 有参考」的侵入式指标回归；单环境包约 5.5 GB
- **出处**：https://www.isca-archive.org/interspeech_2022/li22e_interspeech.pdf §1、§3.2–§3.3、图 1【论文】; https://arxiv.org/abs/2109.07931【论文】; https://zenodo.org/api/records/5464104 、/5499506 、/5501697（许可字段与文件表）【官方文档】; https://zenodo.org/records/5464104/files/README 与 https://zenodo.org/records/5499506/files/README（COPYING 一节、目录结构）【官方文档】

## 备注
- 与 014 的 DAPS（`daps.md`）关系：DDS 用的是 DAPS 的源语音，但录音设备、环境都是新的，是另一份数据。
- 与 DR-VCTK 互补：DR-VCTK 有 MacBook Air 内置麦但只有一个办公室；DDS 环境多但没有笔记本麦。
- 怀疑派复核：重读三条 Zenodo 记录的许可字段与文件表（DAPS 包 9 个环境各 5.35–5.69 GB；VCTK 两包合计 90.86 GB，与原卡一致），并读了包内 README 的 COPYING 一节（5.7 KB，许可与字段一致）；回 arXiv 2109.07931 核 1,944 h、三设备、9 环境、最近 50 cm，一致。改动：许可补 README 原句；笔记本段补目录与文件名格式、位置编号；出处补 README。
