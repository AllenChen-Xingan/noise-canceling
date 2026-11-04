# DEMAND: Diverse Environments Multichannel Acoustic Noise Database

- **名称**：Diverse Environments Multichannel Acoustic Noise Database（DEMAND）
- **链接**：https://zenodo.org/records/1227121
- **许可**：包内 DEMAND.pdf §5 原文「licensed under the Creative Commons Attribution-ShareAlike 3.0 Unported License」；Zenodo「Rights」字段另写 CC BY 4.0，两处冲突，按严者（SA）算，SA 按 D64 仍可训练 → 结论：训练可（按 D64：SA 可训练，训出的权重按相同条款共享并署名）【官方文档】
- **大小**：18 个环境 × 300 s ≈ 1.5 h（每环境 16 通道）；Zenodo 全部文件 7.4 GB【官方文档】；小时数由 18×300 s 算得【推断】
- **场景键**：home-family, open-office, meeting-room, cafe, canteen, transit-hub, bus-metro-cabin, street-outdoor, car-passenger, other（自然：田野 / 河边 / 公园）
- **设备键**：non-laptop-mic-array
- **笔记本段**：没有。16 支 Sony ECM-C10 驻极体麦组成平面阵，架在 1.4 m 三脚架上；笔记本只用来接 USB 声卡存盘，不是收音设备【官方文档】
- **干净参考**：没有（纯噪声录音，没有目标语音）
- **转写**：没有
- **语种**：无目标语音；背景里有法国雷恩当地人声（OMEETING「讨论麦阵」、OOFFICE 三人用电脑、PCAFETER / PRESTO 食堂、SPSQUARE 游客），语种未标【官方文档】【推断：背景人声多为法语】
- **语体**：无目标语体；背景含会议室讨论声、食堂 babble；笑声、附和、情绪语音均未标注
- **获取方式**：直接下载（Zenodo，按环境分 zip，有 16 kHz 与 48 kHz 两版）
- **与 D62 的关系**：真实录音（噪声），不含目标说话人，属于「合成数据」的噪声原料而非真实会议录音。若开发阶段拿来评测：用 PCAFETER / PRESTO / SCAFE（咖啡馆、食堂）、OOFFICE / OMEETING（办公、会议室）、DLIVING / DKITCHEN（家里）、PSTATION / TMETRO / TBUS（交通）、TCAR（车内）、STRAFFIC（街边）当混合用噪声；取单通道约 18×5 min，16 kHz 单通道约 0.15 GB【推断】
- **出处**：https://zenodo.org/records/1227121【官方文档】; https://zenodo.org/records/1227121/files/DEMAND.pdf §2–§5【官方文档】

## 备注
- 每个地点只有 5 min，录于 2012–2013 年雷恩；量小但地点覆盖全，适合当「地点 → 噪声谱」的对照样本。
- Zenodo 描述写「15 recordings」，文件列表却有 18 个前缀（含 DWASHING、NFIELD、NRIVER 等），以 PDF 为准（6 类 × 3）。
- TCAR 阵列在副驾胸口高度，其余在 1.4 m 三脚架上。
- 怀疑派复核：无改动。已读 DEMAND.pdf 原文：license 节写 CC BY-SA 3.0 Unported，1.4 m 三脚架、TCAR 在副驾胸口高度、每段剪成 300 s、阵列接笔记本存盘均属实；Zenodo Rights 字段确为 CC BY 4.0，冲突属实。
- 主控复核（2026-11-03，codex 第 2 轮 #1）：许可结论按 D64 由「只评测」改为「训练可」。
