# F · 条件因素的自然分布出处（016 bench-synth）

> 调研日期 2026-10-31。只补 015 `B-synth-data.md` 没有的；重叠率 / 换人间隔 / 笑声 / 附和见那份 §2.1，这里不重抄。
> 证据等级：【官方】【论文】【源码】【实测】【二手】【推断】。「论文·摘要」表示只核到摘要或转引，没读全文表格。

## 五行结论

1. **真实生活 SNR 远比训练集乐观**：老年助听器用户 894 段 5 分钟录音里众数约 8 dB，62.9% 在 2–14 dB，**只有 7.5% < 0 dB**（Wu 2018）；Smeds 2015 说「负 SNR 或接近 0 dB 的场景很少」，babble 里平均约 5 dB。DNS/URGENT 训练用的 −5~20 dB 均匀分布 < 0 dB 占 20%，约为 Wu 的 2.7 倍。⚠️ **复核加注**：Wu 的 SNR 是**听者胸前**录音机测的「对方说话 vs 环境」，Smeds 是助听器麦处；都不是「笔记本麦处、用户自己的声音 vs 咖啡馆背景 / 邻座说话」。两者的 SNR 定义（现场功率相减 vs 合成时按数字 RMS 设定）也不同，直接拿 7.5% 对 20% 只能算方向性证据【推断】。
2. **SNR 和噪声级是绑在一起的（Lombard）**：噪声每升 1 dB，说话电平升 0.3–1 dB（常用 0.5–0.6）；拐点在 46–57 dBA。Wu 2018 实测：噪声 40 dBA 时 SNR 约 20 dB，噪声 74 dBA 时 SNR 约 0 dB。**合成时 SNR 不能和噪声级独立抽**。
3. **餐厅 / 咖啡馆级别**：Lebo 27 家 59–80 dBA（峰值 87）；香港餐厅 66.7–82.6 dBA，均值 73.9，SD 3.81；纽约（曼哈顿）晚高峰餐厅均值 78、酒吧 81 dBA。按第 2 条 Wu 的分段线性关系外推（59.3 dBA 处 SNR 6.7 dB，之后每升 1 dB 噪声 SNR 降 0.46 dB），59–80 dBA 对应 SNR 约 **+7 到 −3 dB**，73.9 dBA（香港均值）≈ 0 dB，78 dBA（纽约）≈ −2 dB【推断，复核改正：原写 0–8 dB，高噪端算错了】。也就是说，**咖啡馆/餐厅档恰好落在 Wu 那 7.5% 的负 SNR 尾巴附近**，这和第 1 条「真实 SNR 很乐观」并不矛盾，但说明咖啡馆格不能拿全样本众数 8 dB 当中位。
4. **RT60 / 设备链 / 时间结构**：日常空间的 RT60 有大样本出处（Traer & McDermott 2016 共 271 处），但数值只在图里，正文没找到中位数。笔记本内置麦的频响、macOS Voice Isolation 的行为、mSBC 和 CVSD 的音质对比，**都没找到公开实测**，只能自己录。Opus 有官方「甜点」码率可以直接拿来当档位。
5. **说话人与时间结构**：语速英文 193–214 wpm、中文 228–247 字/分（电话会话）；性别差约 2%。停顿时长近似对数正态，大致分三簇（<200 ms / 200–1000 ms / >1000 ms）。换人间隔跨语言众数 0 ms，中位数 +100 ms。**突发噪声（键盘等）在会议里出现得多频繁，没找到任何统计**。

## 正文表

### ① 真实生活 SNR

| 因素 | 典型范围 | 分布形状 | 出处 · 等级 |
|---|---|---|---|
| 日常语音场景 SNR（老年轻中度听损，20 人 × 5–6 周，894 个场景） | 62.9% 在 2–14 dB；众数约 8 dB；7.5% < 0 dB；5.5% > 20 dB | 单峰，右尾长（见 Fig. 4） | Wu et al. 2018, Ear Hear 39(2):293–304，Results 与 Fig. 4、Discussion【论文】 |
| 按场景聚类的 SNR | 安静类 12.2 dB（SD 6）；扩散 / 非扩散噪声类 7.4 dB（SD 6.0）；nPLS 4.2 dB（SD 3.8） | — | 同上，Table 1–2【论文】 |
| 按环境的中位 SNR | 家中 9.9；室内其他 9.3；户外 9.7；交通 5.6；室内人群 5.3 dB | — | 同上，Fig. 5 正文【论文】 |
| 负 SNR 占比的跨研究对照 | Pearsons 1977：15.5%；Smeds 2015：好耳 4.2%、差耳 13.7%；Wu 2018：7.5% | — | Wu 2018 Discussion「Distribution of SNR」转引【论文 / 二手（对 Pearsons、Smeds 来说是转引）】 |
| 助听器用户日常录音 SNR（20 人，18–81 岁，3–4 天） | babble 中平均约 5 dB；负或接近 0 dB 的场景「很少」 | 范围大；在强波动噪声和极低 SNR 时估计不确定性高 | Smeds, Wolters & Rung 2015, JAAA 26(2):183–196，摘要（PMID 25690777）【论文·摘要】 |
| Smeds 2015 分场景均值 | 安静约 22 dB；嘈杂场景 4.6–7.6 dB | — | Hearing Review 2018 转述【二手，未核原文表】 |
| 对照：公开合成配方 | DNS/pDNS SNR −5~20 dB（`resources/DNS-Challenge/pdns_synthesizer_icassp2023.cfg:43-45`）；URGENT 2026 −5~20 dB，风噪 −10~15 dB（`resources/urgent2026_challenge_track1/baseline_code/dataset.py:21-34`） | 均匀 | 【源码】 |

### ② 背景声级与 Lombard 效应

| 因素 | 典型范围 | 分布形状 | 出处 · 等级 |
|---|---|---|---|
| 餐厅声级（旧金山湾区 27 家） | 59–80 dBA，峰值 87 dBA | — | Lebo et al.（年份没核实），CDC stacks 文件转引【二手】 |
| 餐厅声级（香港，LAeq,1h） | 66.7–82.6 dBA；均值 73.9，SD 3.81 | 近正态【推断，只有均值和 SD】；占座密度是唯一显著的预测因子；越满，波动越小 | To & Chung 2015, Noise Vib. Worldwide 46(8):11–17，摘要【论文·摘要】 |
| 纽约餐厅 / 酒吧（众包，>1800 家，晚 7–9 点） | 餐厅均值 78 dBA，酒吧 81；约 71% 的餐厅「不利于交谈」 | — | Scott（SoundPrint），ASA 174th 3aAA1 通俗稿【二手 / 会议摘要】 |
| 家中噪声级 | 中位约 53.6 dBA（同处语音中位 63.7 dBA） | — | Wu 2018 Fig. 5 正文【论文】 |
| 安静类 / 扩散噪声类噪声级 | 安静 50.4–51.0 dBA；扩散噪声 59.4–64.0 dBA | — | Wu 2018 Table 1【论文】 |
| Lombard 斜率（文献区间） | 0.3–1 dB/dB；拐点 46–57 dBA | 分段线性 | PMC7316514 Introduction，综述 Lane & Tranel 1971、Pearsons 1977、Bottalico 2018 等【论文（综述句）】 |
| Lombard 斜率（日常实测） | 拐点 59.3 dBA 以下 0.34 dB/dB，以上 0.54 dB/dB；拐点处语音 66.0 dBA；噪声 40–50 dBA 时语音约 60 dBA；噪声超过约 74 dBA 后语音低于噪声 | 分段线性 | Wu 2018 Fig. 3A 正文【论文】 |
| Lombard 斜率（Pearsons 1977） | 0.6 dB/dB | 线性 | Wu 2018 Introduction 转引【二手】；Pearsons 原报告 EPA-600/1-77-025 没读到 |
| Pearsons 1977 说话电平 | 学校、家、医院、商店等大多数场景下，对话距离处 55–66 dBA；火车 / 飞机上 73–77 dBA，SNR 为负 | — | Olsen 1998 转述【二手】 |
| 餐厅情境的 Lombard 拐点 | 发声费力拐点 57.3 dBA（拐点以上增速约翻倍）；主观干扰拐点 52.2 dBA | 分段 | Bottalico 2018 JASA 144，ASA 通俗稿【论文·摘要】 |
| 由此推出的 SNR–噪声关系 | 噪声 40 dBA → SNR 约 20 dB；74 dBA → 约 0 dB | 单调递减，非线性（含二次项） | Wu 2018 Fig. 3B【论文】 |

### ③ 房间混响、使用距离、DRR

| 因素 | 典型范围 | 分布形状 | 出处 · 等级 |
|---|---|---|---|
| 日常空间 IR（7 名志愿者 × 14 天，每小时随机短信采样，测了 271 处） | 正文没给全样本的 RT60 中位数 / 区间（只在 Fig. 4C/D、Fig. S1A 里）；从调查中挑出的 10 间房 T = 0.51–1.19 s，DRR 中位 32–45 dB（**这是挑出的子集，不是分布**） | 200–2000 Hz 衰减最慢；户外比室内短 | Traer & McDermott 2016, PNAS（PMC5137703），Results、SI【论文】；**全样本数值没找到** |
| 会议室 / 办公室 RT60 | 会议室目标 0.5–0.7 s；玻璃墙和视频会议室常见 0.8–1.2 s；未处理的开放办公 0.9–1.2 s | — | 厂商指南（commercial-acoustics.com 等）【二手，不是测量分布】 |
| 对照：DNS 合成的 T60 | 0.3–1.3 s | 按 RIR 表筛选后均匀抽 | `resources/DNS-Challenge/pdns_synthesizer_icassp2023.cfg:110-112`、`noisyspeech_synthesizer.cfg:81-83`【源码】 |
| 笔记本使用距离 | **没找到实测** | — | — |
| DRR 随距离的变化 | 直达声每倍距衰减 6 dB，混响声近似不变（实测常见 4–5 dB）；临界距离 dc ≈ 0.057·√(V/T60) | 物理模型 | Sengpiel 讲义、Wikipedia「Critical distance」【二手】 |
| 笔记本距离处的 DRR | **没找到实测**。按上面的公式：30 m³、T60 0.5 s 时 dc ≈ 0.44 m，所以 50–70 cm 的使用距离 DRR 约 −1~−4 dB【推断】 | — | 【推断】 |

### ④ 笔记本内置麦链路

| 因素 | 现状 | 出处 · 等级 |
|---|---|---|
| MacBook 内置麦频响 | **没有公开的消声室曲线**。DXOMARK 在半消声室测过 MacBook Pro 14 (M3 Pro)，1 m，1/12 倍频程图，数值没读到；论坛意见两头都有 | DXOMARK 测评页【二手】 |
| 系统 AGC | Apple 语音处理 I/O 的 `isVoiceProcessingAGCEnabled` 默认开启；有开发者反映关掉以后增益行为没有变化 | developer.apple.com AVAudioInputNode 文档【官方】；Apple 开发者论坛 thread 732823【二手】 |
| macOS 麦克风模式 | Standard / Voice Isolation / Wide Spectrum 三档；需要 2018 年以后的机型和 Monterey 以上；**没有延迟、频响、伪影方面的公开测量** | Apple Mac 使用手册 mchle82b42f0【官方】 |
| 蓝牙 HFP | CVSD 窄带 8 kHz 采样；mSBC 宽带 16 kHz 采样；帧长：mSBC 7.5 ms，CVSD 3.75 ms；**没找到 PESQ/POLQA 的直接对比** | NTi Audio 说明【二手】；US11083031 专利【二手】；HEAD acoustics DAGA 2014（方法，结果没读到）【论文·摘要】 |
| Opus 码率 | 20 ms 帧的「甜点」：窄带语音 8–12 kbit/s，宽带 16–20，全带 28–40 | RFC 7587 §3.1.1【官方】 |
| 对照：URGENT 2026 的编解码 / 丢包 | 编解码只用 mp3 / ogg-vorbis（qscale 1–10），没用 Opus；丢包每包 20 ms，丢包率 U(0.05, 0.25)，最多连丢 10 包 | `resources/urgent2026_challenge_track1/baseline_code/dataset.py:53-76`【源码】 |
| Zoom / Teams / Meet 实际码率 | **没找到官方数** | — |

### ⑤ 说话人

| 因素 | 典型范围 | 分布形状 | 出处 · 等级 |
|---|---|---|---|
| 会话 F0（美式英语，Buckeye） | 全体中位 145 Hz，范围 75–339 Hz；女声均值约 191 Hz（摘要片段，没核实） | 右偏、尖峰 | Yang 2024（koreascience JAKO202412343227078）【论文·摘要】 |
| 会话 F0（韩语对话，2740 人） | 男中位 114 Hz，女中位 199 Hz；总跨度约 238 Hz | — | Yang 2023（koreascience JAKO202330043236925）【论文·摘要】 |
| 中文会话 F0 | **没找到大样本会话统计** | — | — |
| 发声力度档位（1 m 处总声级） | normal 62.35 / raised 68.34 / loud 74.85 / shout 82.30 dB | 离散 4 档 | ANSI S3.5-1997，经 R 包 `SII` 文档转引【二手】 |
| 轻声 / 正常 / 大声的差距 | normal 比 soft 高 9.2 dB，loud 比 normal 再高 7.6 dB（loud 比 soft 高 16.8 dB） | — | Bottalico et al. 2017，检索结果转引【二手】 |
| 耳语电平 | **没找到可靠的 dB 数**（Traunmüller & Eriksson 2000 只读到转引） | — | — |
| 语速（电话会话） | 英文：熟人 214±6.73 wpm，陌生人 193±0.71 wpm；中文：熟人 247±10.2 字/分，陌生人 228±2.85 字/分；英文 Fisher 总体 196 wpm（说话人 111–291） | 男略快，约 2%（4–5 词或字/分）；1–7 词的短段语速陡升 | Yuan, Liberman & Cieri 2006, Interspeech pp.541–544，Table 2、§3【论文，已读 PDF 全文】 |
| 附和 / 笑声 | 见 015 `B-synth-data.md` §2.1（笑声约占 10%，附和约 19%） | — | 不重复 |

### ⑥ 会议中的时间结构

| 因素 | 典型范围 | 分布形状 | 出处 · 等级 |
|---|---|---|---|
| 句内 / 句间停顿 | 三簇：短（<200 ms）· 中（200–1000 ms，峰在约 500 ms）· 长（>1000 ms，自发语里在约 1.5 s 另有一簇） | 对数正态，三峰；用几何均值或中位数，不要用算术均值 | Campione & Véronis 2002, Speech Prosody（5 种欧洲语言，约 6000 个停顿）【论文】；三簇的切点来自 MDPI 综述【二手】 |
| 换人间隔（turn offset） | 各语言的峰都在 0~+200 ms；跨语言众数 0 ms，中位 +100 ms；日语中位 0，丹麦语中位约 +300 ms；各语言均值离跨语言均值都在 250 ms 以内 | 单峰，略偏正 | Stivers et al. 2009, PNAS 106(26):10587（摘要）；Language Log 引原文【论文·摘要 / 二手】 |
| 重叠率 / 40% 换人为重叠 | 见 015 §2.1 | — | 不重复 |
| 突发噪声（键盘、杯子、门）的出现频率 | **没找到任何统计**。专利只定性说「常见」；DCASE 2016 有 20 种键盘的素材 | — | US11217262 等专利【二手】 |

## 对 016 的启示（可直接变成格子或检查项）

**建议网格（单因素扫，其余固定在中位）**

| 因素 | 网格档位 | 中位（固定值） | 依据 |
|---|---|---|---|
| SNR | −5 / 0 / 5 / 10 / 15 / 20 dB | 5 dB（咖啡馆格）· 10 dB（家中格） | Wu 2018 众数约 8、7.5% < 0；Smeds babble 约 5 |
| 背景声级 | 50 / 60 / 70 / 78 dBA | 70 dBA（咖啡馆） | Wu 安静类约 51 · 扩散约 60–64；To & Chung 73.9；纽约 78 |
| 说话人电平（随噪声变） | 按 Lombard 自动推：L_speech = 60 dBA（噪声 ≤ 50 dBA 时）；50–59.3 dBA 斜率 0.34；> 59.3 dBA 斜率 0.54 | — | Wu 2018 Fig. 3A |
| 发声力度 | soft / normal / raised / loud（相对 normal −9 / 0 / +6 / +12.5 dB） | normal | ANSI S3.5 + Bottalico 2017（二手） |
| RT60 | 0.2 / 0.4 / 0.6 / 0.9 / 1.2 s | 0.5 s | 会议室目标 0.5–0.7 s；视频会议室 0.8–1.2 s；DNS 0.3–1.3 s |
| 距离 / DRR | 0.3 / 0.5 / 0.8 / 1.2 m（或 DRR +6 / 0 / −3 / −6 dB） | 0.5 m【假设】 | 物理推算，没有实测 |
| 编解码 | 无 / Opus 12k NB / Opus 20k WB / Opus 32k FB / mSBC / CVSD | 无 | RFC 7587 §3.1.1 |
| 丢包 | 0 / 5% / 15% / 25%（每包 20 ms） | 0 | URGENT 2026 源码区间 |
| F0 | 男 ~110 Hz / 女 ~200 Hz / 高女声、童声 ~300 Hz | 按性别 | Yang 2023/2024 |
| 语速 | 慢 160 / 中 200 / 快 250 wpm（中文对应 190 / 230 / 280 字/分【推断】） | 中 | Yuan 2006 |
| 停顿 | 对数正态，中位约 500 ms；档位 150 / 500 / 1500 ms | 500 ms | Campione & Véronis 2002 |
| 突发噪声密度 | 0 / 2 / 10 / 30 次/分【假设，没有出处】 | 2 次/分 | 没有统计，必须标为假设 |

**合成数据检查项（可写成 verifier）**

1. **SNR 边缘分布检查**：「整体」层的数据，SNR < 0 dB 的占比应在 5–15%（Wu 7.5%、Smeds 好耳 4.2% / 差耳 13.7%、Pearsons 15.5%）；众数落在 5–10 dB。用均匀 −5~20 dB 抽出来的会有约 20% < 0 dB，不合格。
2. **SNR–噪声级联动检查**：对每段数据，回归「说话人电平 ~ 噪声级」的斜率，噪声 > 60 dBA 段应在 0.3–0.6 dB/dB 之间；斜率为 0（独立抽）就判不合格。这也是「单类 / 多类」层最容易漏的一条。
3. **说话人绝对电平检查**：在设定的标定下，噪声 ≤ 50 dBA 时说话人应在 55–66 dBA 范围（Pearsons / Olsen 转述、Wu）。
4. **停顿分布检查**：每段的停顿时长做对数直方图，应可见 200 ms 以下和 500 ms 左右的两个峰；只有一种定长停顿就不合格。
5. **换人间隔检查**：换人间隔中位数应在 0~+300 ms 内（Stivers）；按 015 §2.1，约 40% 是重叠。
6. **语速检查**：字幕对齐后每段语速应在英文 150–290 wpm、中文 180–300 字/分以内；超出就是拼接错误或变速增强过头。
7. **设备链是空白**：笔记本内置麦频响、AGC 时间常数、Voice Isolation 都没有公开数，必须**自录一次 sweep 和对讲**来定标（属于实效验证层，交给人）。在这之前，benchmark 里「设备链」这个格子只能标「尚无判据」。
8. **突发噪声密度没有出处**：任何对「键盘 / 杯子出现频率」的设定都只能是【假设】。建议从 AMI / ICSI 的非语音事件标注里统计一次（014 的许可清单里如果有 AMI，就可以做）。

## 空白（没查到）

- Traer & McDermott 2016 全样本 RT60 / DRR 的中位数和分位数（只在图里）
- Pearsons 1977 原报告（EPA-600/1-77-025）正文；Smeds 2015 正文各场景表
- Lebo 餐厅研究的原始出处与年份
- 笔记本使用距离、笔记本麦处 DRR 的实测
- MacBook 内置麦频响曲线数值；macOS Voice Isolation 的延迟、频响、伪影测量；AGC 时间常数
- mSBC 和 CVSD 的 PESQ/POLQA 直接对比；Zoom / Teams / Meet 实际 Opus 码率
- 中文会话 F0 大样本；耳语电平 dB
- 会议中突发噪声（键盘、杯子、门）的出现频率
- 咖啡馆（区别于餐厅）的专门声级测量

## 来源清单

- Wu Y-H, Stangl E, Chipara O, et al. 2018. Characteristics of real-world SNRs and speech listening situations of older adults with mild-to-moderate hearing loss. Ear Hear 39(2):293–304. https://pmc.ncbi.nlm.nih.gov/articles/PMC5824438
- Smeds K, Wolters F, Rung M. 2015. Estimation of SNRs in realistic sound scenarios. JAAA 26(2):183–196. https://pubmed.ncbi.nlm.nih.gov/25690777/
- Hearing Review 2018（Smeds 分场景数的转述）. https://hearingreview.com/hearing-loss/patient-care/evaluation/better-speech-noise-testing-demonstration-dual-variable-speed-compression
- Pearsons KS, Bennett RL, Fidell S. 1977. Speech levels in various noise environments. EPA-600/1-77-025. https://nepis.epa.gov （原文没读到；Olsen 1998 转述经 auditory.org 2008/530 帖）
- To WM, Chung AWL. 2015. Restaurant noise: levels and temporal characteristics. Noise Vib. Worldwide 46(8):11–17. https://research.mpu.edu.mo/en/publications/restaurant-noise-levels-and-temporal-characteristics/
- Lebo et al.（餐厅 59–80 dBA），CDC 转引：https://stacks.cdc.gov/view/cdc/38053/cdc_38053_DS1.pdf
- Scott G. ASA 174th 3aAA1, Are restaurants and bars in NYC too loud? https://acoustics.org/3aaa1-are-restaurants-and-bars-in-new-york-city-too-loud-gregory-scott/
- Bottalico P. 2018. Lombard effect in restaurant setting. https://acoustics.org/4ans11-lombard-effect-in-restaurant-setting-how-much-would-you-spend-to-eat-at-this-restaurant-pasquale-bottalico/
- Lombard 斜率综述句：https://pmc.ncbi.nlm.nih.gov/articles/PMC7316514
- Traer J, McDermott JH. 2016. Statistics of natural reverberation… PNAS. https://pmc.ncbi.nlm.nih.gov/articles/PMC5137703/ ；https://mcdermottlab.mit.edu/Reverb/IR_Survey.html
- Critical distance：https://en.wikipedia.org/wiki/Critical_distance ；https://www.SengpielAudio.com/DirectFieldAndReverberantField.pdf
- RFC 7587 §3.1.1. https://www.rfc-editor.org/rfc/rfc7587.html
- Apple AVAudioInputNode isVoiceProcessingAGCEnabled. https://developer.apple.com/documentation/avfaudio/avaudioinputnode/isvoiceprocessingagcenabled ；论坛 https://developer.apple.com/forums/thread/732823
- Apple Mac 使用手册（麦克风模式）. https://support.apple.com/en-au/guide/mac-help/mchle82b42f0/mac
- DXOMARK MacBook Pro 14 M3 Pro. https://www.dxomark.com/apple-macbook-pro-14-m3-pro-2023-laptop-test/
- NTi Audio Bluetooth 测试说明. https://www.nti-audio.com/en/support/know-how/how-to-test-bluetooth-devices ；HEAD acoustics DAGA 2014. https://pub.dega-akustik.de/DAGA_2014/data/articles/000151.pdf
- Yuan J, Liberman M, Cieri C. 2006. Towards an integrated understanding of speaking rate in conversation. Interspeech. https://www.ldc.upenn.edu/sites/default/files/interspeech2006-towards-integrated-understanding.pdf
- Yang B. 2023/2024（韩语对话 F0；Buckeye F0）. https://www.koreascience.kr/article/JAKO202330043236925.page ；https://koreascience.or.kr/article/JAKO202412343227078.pdf
- ANSI S3.5-1997 发声力度档位（经 R 包 SII）. https://rdrr.io/rforge/SII/man/sii.html
- Campione E, Véronis J. 2002. A large-scale multilingual study of silent pause duration. Speech Prosody. https://mail.sprosig.org/sp2002/pdf/campione-veronis.pdf
- Stivers T, et al. 2009. Universals and cultural variation in turn-taking. PNAS 106(26):10587. https://forms.mpi.nl/node/50939 ；https://languagelog.ldc.upenn.edu/nll/?p=7997
- 源码：`resources/DNS-Challenge/pdns_synthesizer_icassp2023.cfg:43-49,110-112`；`resources/urgent2026_challenge_track1/baseline_code/dataset.py:21-76`
