# 027 · 训练数据许可（子代理笔记）

> 2026-11-05 · 会话内子代理（只读网页 / README / 许可文件，未下载任何数据）。
> 这是子代理原文的整理版；**大总管汇总时以 `../sources.md` 与 `../decision.md` 为准**。主会话亲手复核过的条目在 `../sources.md` 里标「复核」，其余仍是子代理读到的。
> 判据：项目许可规矩（D63 · D64）——CC0 / CC BY / Apache / MIT 可训练；BY-SA 可训练（权重按 BY-SA 共享并署名）；NC · ND · 付费 · 仅限学术 不可训练。

## 一、噪声

| 数据集 | 补我们缺的哪些场景 | 许可（原文 · 读自） | 能否训练 | 大小 | 时长 / 条数 |
|---|---|---|---|---|---|
| CochlScene | 公交 · 地铁车厢 · 地铁站 · 咖啡馆 · 餐馆 · 小汽车 · 街道 · 拥挤室内 | "Creative Commons Attribution Share Alike 3.0 Unported"（Zenodo 记录页）[1] | 能（BY-SA） | 53.9 GB | 76,115 条 × 10 s，约 211 小时（推算） |
| QUT-NOISE | 咖啡馆 · 车内 · 街道 · 家居 | "licensed CC-BY-SA"（官网）[2] | 能（BY-SA） | 约 7.7 GB | 未核 |
| SONYC-UST v2 | 街边交通 · 引擎 · 警报 · 喇叭 | CC BY 4.0（Zenodo）[3] | 能 | 13.3 GB | 18,515 条 × 10 s，约 51 小时（推算） |
| TUM 风噪集 | 风声（手机实录 + 合成） | "Creative Commons Attribution 4.0 International"（Zenodo）[4] | 能 | 0.14 GB | 未核 |
| RWTH IKS 风噪库 | 风声（仿手机麦） | "see the included MIT license files"（仅搜索摘要，官网连不上）[5] | 未核 | 未核 | 未核 |
| MUSAN | 混合噪声 | "Attribution 4.0 International (CC BY 4.0)"（OpenSLR 17）[6] | 能 | 11 GB（含音乐、语音） | 未核 |
| FSD50K | 风 · 脚步 · 车辆 · 碗碟 · 人群（按标签） | 整体 "CC-BY"，每条另有许可（Zenodo）[7] | 能但需过滤：只留 CC0 + CC-BY 43,379 条（约 85%），去掉 CC-BY-NC 6,041 条与 Sampling+ 1,777 条 | 24.7 GB | 51,197 条 / 108.3 小时 |
| DNS5 noise | 约 150 类 | 按来源 "original terms"（AudioSet 标 CC BY 4.0 · Freesound 只收 CC0 · DEMAND CC BY-SA 3.0；GitHub README）[8] | 能但需过滤，且**含 DEMAND 整段，和评测集撞源** | 58 GB | 未核 |
| WHAM! noise | 咖啡馆 · 餐馆 · 酒吧 | CC-BY-NC-4.0（audeering 数据卡）[9] | 不能 | 未核 | 约 81.7 小时 |
| TAU Urban 2020 Mobile | 机场 · 地铁站 · 公交 · 电车 · 街道 | "Other (Non-Commercial)"（Zenodo）[10] | 不能 | 27.4 GB | 64 小时 |
| CHiME-3/4 噪声 | 公交 · 咖啡馆 · 步行街 · 街道 | LDC 付费协议 [11] | 不能 | — | 50 小时 |
| SINS | 家居 | Zenodo 字段 CC BY 4.0 与 license.pdf「for academic research purposes only」冲突 [12] | 不能（按较严） | — | — |
| BBC Sound Effects | 各类 | 只限 personal / educational / research（二手转述）[13] | 不能 | — | — |
| Sonniss GDC | 各类 | "Use for AI/ML training is strictly prohibited"（官网）[14] | 不能 | — | — |
| Freesound 本站 | 全部 | 每条单独许可；API 条款商用 "negotiated on a case by case basis"、"Do not make full copies of the database" [15][16] | 不建议直接抓 | — | — |

## 二、房间冲激响应（本机三套 MIT · ACE · McVAMPIRE 已被评测集占用）

| 数据集 | 许可（原文 · 读自） | 能否训练 | 大小 | 数量 |
|---|---|---|---|---|
| OpenSLR 28（RIRS_NOISES） | "Apache 2.0"（只给整体许可；内部来源 RWCP / REVERB 未逐个核）[17] | 能（待核内部来源） | 1.3 GB | 未核 |
| OpenSLR 26（模拟） | "Apache 2.0" [18] | 能 | 0.18 GB | 未核 |
| BUT ReverbDB | "CC-BY 4.0" [19] | 能 | 8.7 GB（RIR 部分） | 约 4,700 条 · 9 个房间 |
| MeshRIR | CC BY 4.0 [20] | 能 | 未核 | 18,081 条 · 1 个房间 |
| dEchorate | CC BY 4.0 [21] | 能 | 25.8 GB | 未核 |
| Arni | CC BY 4.0 [22] | 能 | 50.9 GB | 同一房间 |
| Aachen AIR | "MIT"（audeering 数据卡）[23] | 能 | 未核 | 107 个文件 |
| pyroomacoustics（本机模拟） | MIT [24] | 能 | — | — |
| gpuRIR | AGPL-3.0，需 CUDA [25] | Mac 上用不了 | — | — |

## 三、干净人声（本机 LibriSpeech test-clean 已被评测集占用）

| 数据集 | 许可（原文 · 读自） | 能否训练 | 大小 | 时长 / 说话人 |
|---|---|---|---|---|
| LibriSpeech train-clean-100 / 360 | "CC BY 4.0"（OpenSLR 12）[26] | 能 | 6.3 / 23 GB | 100 / 360 小时 |
| LibriTTS-R | "CC BY 4.0"（OpenSLR 141）[27] | 能 | 8.1 / 28 GB | 全集约 585 小时 |
| VCTK 0.92 | CC BY 4.0 全文（license_text.txt）；DNS README 写 ODC-By，都允许商用 [28][8] | 能 | 10.94 GB | 110 人 |
| AISHELL-1 | "Apache License v.2.0" [29] | 能 | 15 GB | 400 人 |
| AISHELL-3 | "Apache License v.2.0" [30] | 能 | 19 GB | 约 85 小时 · 218 人 |
| MAGICDATA | "CC BY-NC-ND 4.0" [31] | 不能 | 52 GB | — |
| ST-CMDS | "BY-NC-ND 4.0" [32] | 不能 | 8.2 GB | — |
| Common Voice 中文 | 转到 Mozilla Data Collective [33] | 未核 | — | — |

## 四、子代理提的风险

1. DNS noise 内含 DEMAND 原始文件（整段），和评测集撞源；要用就按文件名剔掉全部 DEMAND
2. DNS 的 AudioSet 部分：AudioSet 页面的 CC BY 4.0 只覆盖标签与特征，不覆盖 YouTube 音频本身 [34]（推断有版权风险）
3. CochlScene 是众包手机录音，可能夹带可懂人声，当噪声用前要筛（推断）
4. 开放办公 / 联合办公没有专门的可训练数据，只能用 FSD50K 标签 + CochlScene 拥挤室内拼（推断）

## URL

1. https://zenodo.org/records/7080122 ；类别表 https://arxiv.org/html/2211.02289
2. https://research.qut.edu.au/saivt/databases/qut-noise-databases-and-protocols/
3. https://zenodo.org/records/3966543
4. https://zenodo.org/records/6687982
5. https://www.iks.rwth-aachen.de/forschung/tools-downloads/databases/wind-noise-database/
6. https://www.openslr.org/17/
7. https://zenodo.org/records/4060432
8. https://github.com/microsoft/DNS-Challenge
9. https://audeering.github.io/datasets/datasets/wham.html
10. https://zenodo.org/records/3819968
11. https://catalog.ldc.upenn.edu/LDC2017S24
12. https://zenodo.org/records/2555143
13. https://github.com/bbcarchdev/Remarc
14. https://sonniss.com/gameaudiogdc
15. https://freesound.org/docs/api/resources_apiv2.html
16. https://freesound.org/help/tos_api/
17. https://www.openslr.org/28/
18. https://www.openslr.org/26/
19. https://speech.fit.vut.cz/software/but-speech-fit-reverb-database
20. https://www.sh01.org/MeshRIR/
21. https://zenodo.org/records/6576203
22. https://zenodo.org/records/6985104
23. https://audeering.github.io/audbcards/audb-public/air.html
24. https://github.com/LCAV/pyroomacoustics
25. https://github.com/DavidDiazGuerra/gpuRIR
26. https://www.openslr.org/12/
27. https://www.openslr.org/141/
28. https://datashare.ed.ac.uk/handle/10283/3443
29. https://www.openslr.org/33/
30. https://www.openslr.org/93/
31. https://www.openslr.org/68/
32. https://www.openslr.org/38/
33. https://huggingface.co/datasets/mozilla-foundation/common_voice_17_0
34. https://research.google.com/audioset/download.html
