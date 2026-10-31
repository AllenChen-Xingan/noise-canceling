# A · URGENT Challenge 2024 / 2025 / 2026：合成配方、指标套件、合成与真实上的排名

> 016 调研线 A 篇。只在 015 B 篇（`../noise_canceling-dfnhush/docs/research/015_dfn-hush-synth/notes/B-synth-data.md:27,50`）已写内容之上**增补**。
> 证据等级：【官方】挑战赛官网与官方 PDF ·【论文】arXiv / ISCA ·【源码】`resources/<仓库>/<文件>:<行>` ·【实测】本篇用论文表格或官方小文件**自己算的数**（脚本只在本机跑过，没有进仓）·【推断】我的推论 ·【二手】转述。
> 「没找到」就写没找到。

## 开头 5 行结论

1. **系统级指标与 MOS 的相关**（URGENT 2024 盲测，21 队 + 基线 + 带噪输入，共 23 行）：UTMOS 和 SCOREQ 最高，KRCC 都是 0.75。DNSMOS 的 KRCC 只有 0.46，LSD 只有 0.34，WADA-SNR 只有 0.24。多指标综合排名的 KRCC 是 0.73，强于任何单一挑战赛指标。【论文】arXiv 2506.01611 §3.2 图 4；【源码】`urgent2024_analysis/mos/mos_correlations.png`
2. **只比前几名时，侵入式指标基本不保序**。我按论文原表重算：URGENT 2024 前 10 名里，PESQ 与 MOS 的 KRCC 只有 0.19，LSD 是 −0.16，NISQA 还有 0.61。URGENT 2025 前 12 名里，PESQ 0.13，ESTOI 0.21，NISQA 0.66。URGENT 2026 前 6 名的客观排名与主观排名只有 KRCC 0.47，客观第 1 名在主观里排第 2，客观第 4 名拿了主观第 1。【实测】数据源见 §3
3. **非侵入指标会被「刷分」和「幻觉」骗**。2024 年 T17 的 DNSMOS 排第 1，MOS 只排第 18。2025 年纯生成式系统 T13 的 DNSMOS、NISQA、UTMOS 都排第 1，但 ESTOI 和 SDR 垫底，日语 CAcc 只有 36.8%。CHiME-7 UDASE 上 DNSMOS 最好的系统，在 P.835 真实会话听测里垫底；在同一份数据上，DNSMOS_OVRL 的系统级 SRCC 是 −0.10。【论文】2506.01611 §3.1；2505.23212 表 2–3；2402.01413 §6.2.5；【源码】`urgent2026_challenge_track2/BENCHMARK.md:20`
4. **训练数据：质量胜过数量**。官方认定为「干净」的语料里也有噪声：约 2000 条 WADA-SNR 为负，CommonVoice 的标称带宽近 100% 不实。Track 2 用约 6 万小时数据，没有一支队伍超过 Track 1（约 2500 小时）的最佳系统。从 2500 小时里按非侵入分数挑出 700 小时，训练出的模型多数非侵入指标持平或更好。2026 年官方直接把这 700 小时作为基线训练集。【论文】2506.01611 §2.1；2505.23212 §4.1；2506.23859 表 III；2601.13531 §3
5. **合成测试格系统性缺「多人说话」**。2024 年盲测的 500 条真实录音里 132 条有说话重叠，500 条合成样本里只有 3 条。重叠样本的「难样本率」是 0.56，在所有标签里仅低于强混响（0.88）和低可懂度（0.85）。真实录音来源含 CHiME-6、AISHELL-4（会议）、VoiceHome-2 等。【实测】由 `urgent2024_analysis/tagging/*.tsv` 统计；【论文】2507.11306 §3.1 引文 [35]–[41]

---

## 1. 多失真合成配方（对 015 B 篇的增补）

015 B 篇已写：SNR U(−5,20)，风噪 5%，风噪 SNR −10~15，混响概率 0.5，限带、削波、mp3/ogg 编码、丢包 5–25%，增广个数 0/1/2/3 的概率为 0.25/0.40/0.20/0.15。下表只列 015 没写的项。

| 项 | 内容（原文数字） | 出处 | 等级 |
|---|---|---|---|
| 2024 版失真种类 | 4 类：加性噪声、混响、限带、削波 | arXiv 2406.04660 §2；2505.23212 §1 | 【论文】 |
| 2024 版参数 | SNR −5~20 dB，混响概率 0.5 | 2406.04660 §4.1（pdftotext 第 269 行附近） | 【论文】 |
| 2025 版新增 | 编解码损失（MP3/OGG）、丢包、风噪，共 7 类；每条最多叠 5 种失真 | 2505.23212 §3.2 | 【论文】 |
| 四种增广等权抽 | bandwidth_limitation / clipping / codec / packet_loss 的 `weight` 都是 1.0 | `urgent2025_challenge/conf/simulation_train.yaml:82-103` | 【源码】 |
| 削波分位 | min_quantile [0.0, 0.1]，max_quantile [0.9, 1.0] | 同上 `:88-89` | 【源码】 |
| 丢包连续上限 | 每包 20 ms，最多连续丢 10 包 | 同上 `:101-102` | 【源码】 |
| OGG qscale 2025→2026 | 2025 是 [−1, 10]，2026 改为 [1, 10]；其余概率与参数两届一致 | `urgent2025_challenge/conf/simulation_train.yaml:98` vs `urgent2026_challenge_track1/conf/simulation_train.yaml:59`（`diff` 只有这一处数值差） | 【源码】 |
| 风噪自身削波 | `clipping_threshold` [0.85, 1.0]，`clipping_chance` 0.75；阵风度 gustiness [3, 10]；每个采样率生成 200 条（训练） | `urgent2025_challenge/conf/simulation_train.yaml:72-73`；`wind_noise_simulation_train.yaml:2,9-10` | 【源码】 |
| 风噪生成器 | SC-Wind-Noise-Generator（经 sp-uhh/storm 封装） | 2505.23212 §3.2 脚注 3 | 【论文】 |
| 采样率 | 8 / 16 / 22.05 / 24 / 32 / 44.1 / 48 kHz 七档；先估有效带宽，再重采样到「能覆盖它的最低档」 | 官网 urgent2025/data；2506.01611 §2.1 | 【官方】【论文】 |
| 盲测才有的失真 | 2025 年盲测加入训练没见过的神经编解码（Encodec、DAC）；真实录音缺丢包这类失真，所以在真实录音上**人工叠加**削波、丢包、限带、编码 | 2505.23212 §3.3.3 | 【论文】 |
| 2026 年说话人多样性 | 新增儿童（NNCES）、老人（SeniorTalk）、歌唱（VocalSet）、情感（ESD）、耳语等；噪声改用 AudioSet / WHAM! / FSD50K / FMA | 2601.13531 §2–3 | 【论文】 |
| 2026 年训练集 | 官方 700 小时精选集（从 2500 小时筛出），清单在 `urgent2026_challenge_track1/meta/train_selected_700h` | 2601.13531 §3；源码目录 | 【论文】【源码】 |
| **竞争说话人** | 三届配方都**没有**把第二说话人当干扰源合成 | 读三份 `simulation_train.yaml` 都没有该项 | 【源码】 |

## 2. 指标套件与各指标和主观分的相关

### 2.1 套件演变

| 届 | 非侵入式 | 侵入式 | 下游·与任务无关 | 下游·与任务相关 | 主观 | 出处 |
|---|---|---|---|---|---|---|
| 2024 | DNSMOS, NISQA | POLQA, PESQ, ESTOI, SDR, MCD, LSD | LPS, SpeechBERTScore | SpkSim, WAcc | P.808 ACR MOS；只测 300 条（一半合成、一半真实），每条 8 人 | 2506.01611 §3.1 | 
| 2025 | + UTMOS | 同上 | LPS, SBS（mHuBERT-147） | SpkSim（RawNet3）, CAcc（OWSM v3.1） | P.808 MOS；比赛期间只测英语，赛后补了德语、汉语、日语 | 2505.23212 §3.5；2507.11306 |
| 2026 T1 | DNSMOS, NISQA, UTMOS, SCOREQ | PESQ, ESTOI, POLQA | SBS, LPS | 说话人相似度、情感相似度、语种识别准确率、CAcc | 客观前 6 名进入 P.808 **ACR + CCR**，最终名次看 MOS / CMOS 加显著性检验 | 2601.13531 §5；官方 PDF |

- **哪些指标只在合成那一半上算**：2024 年盲测中，侵入式指标、下游·与任务无关指标和 SpkSim 只在合成的一半上算；非侵入式指标和 WAcc 在全集上算（2506.01611 表 1 注）。2025 年真实录音只算 DNSMOS / NISQA / UTMOS / MOS / CER（官网 urgent2025/data）。【论文】【官方】
- **排名方法**：先算每个指标上的名次，再在类别内平均，最后在类别间平均（受 Friedman 检验启发）（2505.23212 §3.5）。2024 年按类别加权求和，权重见 `urgent2024_analysis/tagging/README.md:60-100`（非侵入每项 1/8–1/10，侵入每项 1/20–1/30，MOS 1/5）。【论文】【源码】

### 2.2 与主观分的相关（系统级）

| 场景 | 指标 → 与 MOS 的相关 | 出处 | 等级 |
|---|---|---|---|
| 2024 盲测 23 行（含带噪输入和基线），KRCC / SRCC / LCC | DNSMOS 0.46/0.57/0.83 · NISQA 0.64/0.71/0.87 · DNSMOS Pro 0.57/0.71/0.88 · **UTMOS 0.75/0.91/0.94** · WV-MOS 0.54/0.74/0.87 · **SCOREQ 0.75/0.91/0.91** · VQScore 0.49/0.65/0.87 · WADA-SNR 0.24/0.35/0.11 · PESQ 0.57/0.77/0.72 · ESTOI 0.61/0.80/0.63 · SDR 0.59/0.77/0.54 · MCD 0.52/0.71/0.72 · LSD 0.34/0.49/0.42 · POLQA 0.64/0.82/0.75 · SBS 0.53/0.71/0.75 · LPS 0.57/0.76/0.69 · SpkSim 0.52/0.71/0.42 · WAcc 0.50/0.71/0.30 · 综合排名（含 MOS）0.73/0.90/0.76 · 综合排名（不含 MOS）0.67/0.84/0.72 | 2506.01611 图 4；精修版 `urgent2024_analysis/mos/mos_correlations.png`（`README.md` 的 MOS correlation 节） | 【论文】【源码】 |
| 同上，复核 | 我用论文表 1 的原始数值重算 KRCC（tau-b）：DNSMOS 0.47、NISQA 0.64、PESQ 0.56、ESTOI 0.61、LSD 0.34、POLQA 0.64、综合排名 0.73，与图 4 一致，最多差 0.01 | 2506.01611 表 1 | 【实测】 |
| 2024，去掉带噪输入和基线（21 队） | DNSMOS 0.40 · NISQA 0.57 · PESQ 0.53 · ESTOI 0.58 · LSD 0.28 · POLQA 0.62 · 综合排名 0.71 | 同上表 | 【实测】 |
| **2024 前 10 名** | DNSMOS 0.55 · NISQA 0.61 · **PESQ 0.19 · ESTOI 0.35 · SDR 0.31 · LSD −0.16** · POLQA 0.36 · LPS 0.26 · WAcc 0.27 · 综合排名 0.52 | 同上表 | 【实测】 |
| **2025 盲测 13 系统**（多语 MOS，英/德/汉/日四语平均）KRCC / PCC | PESQ −0.03/−0.21 · ESTOI 0.04/−0.27 · LPS 0.16/0.08 · DNSMOS 0.63/0.64 · NISQA 0.68/0.73 · 官方名次 0.31 | 2507.11306 表 3 | 【实测】 |
| 2025，去掉纯生成式 T13（12 系统） | PESQ 0.13 · ESTOI 0.21 · LPS 0.35 · DNSMOS 0.59 · NISQA 0.66 · 官方名次 0.52 | 同上 | 【实测】 |
| 2025 Track 1 表 2 中可见的 12 行（只有英语 MOS），用名次算 KRCC | UTMOS 0.73 · NISQA 0.58 · DNSMOS 0.45 · POLQA 0.21 · PESQ 0.18 · ESTOI 0.21 · CAcc 0.06 · MCD −0.30 · 综合分 0.42；去掉 T13 和 T22 后：UTMOS 0.60、综合分 0.60 | 2505.23212 表 2（只公开 22 行中的 12 行） | 【实测】（行不全，仅供参考） |
| 2026 前 6 名：客观名次 vs 初始主观名次（9 语 ACR + CCR 平均） | KRCC 0.47，SRCC 0.60（n=6）。客观 1→主观 2，客观 4→主观 1，客观 3→主观 4。正式排名的 4 队（另 2 队因违规不计）KRCC 0.33 | 官方 PDF `urgent2026/assets/files/FinalRankingDetails_URGENT2026.pdf` p.1–2 | 【官方】数据，【实测】计算 |
| 2026 年 Track 2 质量评估基准，**系统级 SRCC** | urgent2024-sqa（SE 输出）：UTMOS 0.90 · SCOREQ 0.93 · DNSMOS_OVRL 0.56 · NISQA 0.70 · Distill-MOS 0.75 · Uni-VERSA-Ext 0.96。**CHiME-7 UDASE（真实会话录音，5 个系统）**：SCOREQ 0.90 · Uni-VERSA-Ext 0.90 · Distill-MOS 0.70 · NISQA 0.13 · **DNSMOS_OVRL −0.10** · **UTMOS 0.20** | `urgent2026_challenge_track2/BENCHMARK.md:1-21` | 【源码】 |
| CHiME-7 UDASE 原论文 | 只有 DNSMOS BAK 与主观 BAK 的 PCC 达到 0.73；其余非侵入指标与主观分的 PCC 都 ≤ 0.50。DNSMOS 最好的 CMGAN 系统在 P.835 听测里最差（推测是它在训练里直接优化了 DNSMOS） | arXiv 2402.01413 §6.2.5、图 5 | 【论文】 |
| URGENT-PK（成对排序模型） | 2026 年 Track 2 中，场外参赛的 URGENT-PK 系统级相关最高；论文中的具体数字我没读到 | 2601.13531 §6；arXiv 2506.23874 | 【论文】（数字没找到） |

## 3. 非盲测（合成）与盲测（含真实）上的排名是否一致

| 问题 | 发现 | 出处 | 等级 |
|---|---|---|---|
| 官方是否报告了非盲测与盲测排名的相关 | **没找到**。三届总结论文都没给，2024 年的官网排行榜也已关闭 | 2506.01611；2505.23212；2601.13531；urgent2024/leaderboard 页 | 【论文】【官方】 |
| 盲测内部合成那一半与真实那一半的分项排名 | **没找到**。官方没有分开报告 | 同上 | — |
| 间接证据 1：在合成那一半上算的侵入指标，对只含一半真实录音的 MOS 排名的预测力 | 拉开全部 23 行时还行（KRCC 0.56–0.64），只看前 10 名时降到 0.19–0.36。在全集（含真实录音）上算的 NISQA 仍有 0.61 | §2.2 复算 | 【实测】→【推断】：**合成格可以分清好坏两大档，分不清最前面那几名** |
| 间接证据 2：难样本率 | 2024 年非盲测（全合成）中 93/1000 是难样本（0.09）；盲测中，合成那一半 155/500（0.31），真实那一半 173/500（0.35）。**同为合成，盲测比非盲测难 3 倍多**（来源换成了没见过的语料） | `urgent2024_analysis/tagging/README.md`（难样本清单）+ `*_test_tags.tsv` | 【实测】 |
| 间接证据 3：真实数据上的质量评估器 | 在 SE 合成数据上系统级 SRCC 有 0.90 的 UTMOS，到 CHiME-7 真实会话上只剩 0.20；只有 SCOREQ 和 Uni-VERSA-Ext 保持在 0.90 | `BENCHMARK.md:10,17,21` | 【源码】（CHiME 只有 5 个系统，统计量很粗） |

## 4. 训练数据：质量、数量与过滤

| 发现 | 原文数字 | 出处 | 等级 |
|---|---|---|---|
| 标称带宽不实 | 约 25% 的 LibriTTS、约 100% 的 DNS5 LibriVox、约 100% 的 CommonVoice 11.0 英语，有效带宽低于标称采样率的奈奎斯特频率 | 2506.01611 §2.1 图 1 | 【论文】 |
| 「干净」标签带噪 | 约 2000 条训练标签的 WADA-SNR 为负，人工抽检大多确实带噪；VAD + DNSMOS 过滤没能去掉它们；基线增强后 WADA-SNR 分布几乎不变 | 2506.01611 §2.1 图 2；`urgent2024_analysis/README.md` WADA-SNR 节 | 【论文】 |
| 官方故意保留部分带噪样本 | 2024 年「决定不过滤这些样本，让模型学会应对」 | 2406.04660 §3（pdftotext 第 186 行附近） | 【论文】 |
| 数据越多不一定越好 | Track 2 约 6 万小时，没有系统超过 Track 1（约 2500 小时）的最佳系统，原文：「扩大数据不一定带来更好的 SE，当新增数据含有带噪语音时」 | 2505.23212 §4.1 | 【论文】 |
| 精选 700 小时 vs 全量 2500 小时 | 两阶段筛选：① 按阈值过滤，DNSMOS / SigMOS / UTMOS 默认 3.0，NISQA 4.0，SQUIM_SDR 20（EARS 放宽到 2.5 / 2.5 / 2.5 / 3.0 / 0）；② 五项指标做 z 归一化后求和，取前 100 / 350 / 700 小时。BSRNN 结果：DNSMOS 2.80→2.85，NISQA 2.83→2.96，DNSMOS Pro 3.22→3.43；侵入指标基本持平（SDR 10.89→10.71，PESQ 2.39→2.36）。CommonVoice 从约 1300 小时砍到约 250 小时，EARS 从约 107 小时砍到约 16 小时 | arXiv 2506.23859 §II-C 表 I–III | 【论文】 |
| 噪声量扩展 | 噪声从 50 小时加到 200 小时，没有出现明显的规模曲线 | 2506.23859 §III-E 图 4 | 【论文】 |
| 2026 年前排队伍的做法 | 基于 MOS 的过滤、用预训练修复模型清洗数据、定制增广 | 2601.13531 §6 | 【论文】（无数字） |
| 官方验证集 | 1000 条，CommonVoice **人工挑选**干净样本 | 2505.23212 §3.3.2 | 【论文】 |

## 5. 真实测试集怎么构成

| 届 | 构成 | 真实录音来源 | 出处 | 等级 |
|---|---|---|---|---|
| 2024 非盲测 | 1000 条全合成：641 条匹配域（CommonVoice / LibriTTS / VCTK 测试分区），359 条 YouTube CC 播客；噪声来自 WHAM! 测试集、DEMAND、TUT 2018；真实 RIR 来自 SLR28 和 MYRiAD，RT60 2.43±0.96 s；标签由人工听过去噪 | — | 2506.01611 §2.2 | 【论文】 |
| 2024 盲测 | 500 条合成 + 500 条真实；真实那一半的标签分布：重叠 132、多人 121、自发风格 316、瞬态噪声 282、强混响 44、音乐 78、削波 0 | 来源未公开（「identities won't be revealed」） | 2506.01611 §2.3；官网 urgent2024/data；【实测】标签统计 | 【论文】【实测】 |
| 2025 盲测 | 6 种语言，每种 150 条，共 900 条，合成与真实各一半；日语为训练未见语言 | CHiME-6、COVYT、FLEURS、VoiceHome-2、**AISHELL-4（会议场景）**、YODAS、ICASSP 2024 SSI 挑战 | 2507.11306 §3.1 引文 [35]–[41]；2505.23212 §3.3.3 | 【论文】 |
| 2026 盲测 | 360 条合成 + 480 条真实（共 840 条）；新增 5 种未见语言（HI / KR / AR / JP / IT） | 只写了「collected from public sources」 | 2601.13531 §3 | 【论文】 |
| 2026 Track 2 测试 | 8000 条带 MOS 的样本，来自 16 个系统（取自 2025 年盲测阶段） | — | 2601.13531 §3 | 【论文】 |

**难样本统计（2024 年盲测，「难」= 至少 2 队处理差）**【实测】由 `urgent2024_analysis/tagging/` 统计：

| 标签 | 全盲测 n / 难样本率 | 真实那一半 n / 难样本率 | 合成那一半 n / 难样本率 |
|---|---|---|---|
| speech_overlap | 135 / 0.56 | 132 / 0.56 | 3 / 0.33 |
| Nspk（多人） | 131 / 0.32 | 121 / 0.33 | 10 / 0.20 |
| strong_reverberation | 176 / 0.88 | 44 / 0.86 | 132 / 0.88 |
| low_intelligibility | 230 / 0.85 | 94 / 0.86 | 136 / 0.84 |
| instantaneous_noise | 712 / 0.32 | 282 / 0.33 | 430 / 0.32 |
| bandwidth_limitation | 262 / 0.29 | 40 / 0.17 | 222 / 0.32 |
| clipping | 154 / 0.30 | 0 / — | 154 / 0.30 |
| 全部 | 1000 / 0.33 | 500 / 0.35 | 500 / 0.31 |

论文原话：重叠语音、强宽带噪声或瞬态噪声、强混响最难；「SNR 不能衡量难度」，盲测里大量 SNR > 10 dB 的样本仍是难样本（2506.01611 §2.3）。【论文】

---

## 对 016 的启示（可转成 benchmark 格子或合成数据检查项）

**benchmark 的指标层**

1. **不用 DNSMOS 单独做裁判**。URGENT 2024 上它的 KRCC 是 0.46，真实会话上系统级 SRCC 是 −0.10，与我们自己测到的 0.48 一致。主力非侵入指标改用 **SCOREQ**（2024 年 KRCC 0.75，CHiME-7 真实会话 SRCC 0.90），UTMOS 只作辅助：它在 CHiME 真实会话上只有 0.20。→ verifier 候选：`scoreq_sys_rank`。【推断】
2. **非侵入指标必须配一个「内容保真」项**（LPS 或 CAcc）。这样能抓住 T13 那种「听着顺、字是错的」幻觉，也能防止我们自己的模型钻 DNSMOS 的空子，就像 CMGAN 那样。→ verifier：`lps_drop ≤ 阈值`，阈值尚无判据，待测。【推断】
3. **报告相关要分档**：全量系统（含带噪输入）与「前几名」各算一次 KRCC。我们的场景是在几个相近候选里挑一个，属于前几名区间。URGENT 的数据显示，侵入指标在这个区间降到 0.2–0.35。验收应以「前几名区间的 KRCC」为准。【推断】
4. **多指标综合排名比任何单项都稳**（2024 年 0.73 / 0.67）。我们的三把尺子（清楚 / 自然 / 轻快）可以学它：先按类别内平均名次合成一个分，再看 MOS 与这个合成分的 KRCC。【论文 → 推断】

**benchmark 的条件格子**

5. **必须有「竞争说话人 / 重叠」这一轴**。URGENT 三届的合成配方都没有这一项，2024 年合成测试只有 3/500 条带重叠，而真实录音有 132/500 条，且这类样本的难样本率是 0.56。这正是「邻座说话」场景，合成格不覆盖它，就测不出真实世界里的排名。→ 格子：重叠比例 × 干扰说话人相对电平。【实测 → 推断】
6. **SNR 不能当难度轴的唯一刻度**（SNR > 10 dB 的样本仍有大量难样本）。要加「瞬态噪声」「混响强度」「可懂度」这几个轴。→ 格子维度：SNR × 噪声时间结构（稳态 / 瞬态） × RT60。【论文 → 推断】
7. **测试集 = 合成 + 真实各半**是 URGENT 三届一直沿用的做法。真实录音没有参考信号，只能算非侵入指标、ASR 指标和 MOS。可以直接借来的公开真实会议源：**AISHELL-4**、CHiME-6、VoiceHome-2、ICASSP 2024 SSI（许可要回 014 decision 核对）。【论文 → 推断】
8. **加一道合成 vs 真实的排名一致性检查**：同一批系统分别在合成格和真实录音上排名，算 KRCC。URGENT 自己都没公开这个数，我们应该把它做成 verifier（`sim_real_rank_krcc`），过线值待定。【推断】

**合成数据检查项**

9. **干净源语音要过滤**：① 估有效带宽，假宽带要降档或剔除；② 用 WADA-SNR < 0 抓带噪「干净」样本；③ 对 DNSMOS / SigMOS / UTMOS / NISQA 设阈值（默认 3.0 / 3.0 / 3.0 / 4.0），并做 z 分数排序。→ 检查项：`clean_src_bandwidth_ok`、`clean_src_wada_snr>0`、`clean_src_nisqa≥4.0`。【论文 → 推断】
10. **数据量不追大**：URGENT 的证据是 2500 小时挑出 700 小时就够，6 万小时也没打赢 2500 小时；噪声量从 50 小时加到 200 小时也没有规模收益。我们的合成数据「一般般」，先查干净源是否干净，再考虑扩量。【论文 → 推断】
11. **编码 / 丢包 / 限带只进评测链，不进训练默认**（同意 015 B 篇的做法）。URGENT 2024 发现，现代系统对限带和削波本来就比较稳健。【论文】
12. **测试源要换未见过的语料**：URGENT 同为合成数据，换成未见来源后难样本率从 0.09 涨到 0.31。我们的单类和多类格子应该有一份「来源未见」版，否则会高估效果。【实测 → 推断】

---

## 没查到的空白

- 官方没有公开非盲测与盲测、盲测内部合成与真实两半的**队伍排名相关系数**。原始数据也许在 HuggingFace 的 `urgent-challenge/urgent2024_mos`（带 MOS）和 urgent-challenge.com 排行榜（`competitions/13`、`/15`）里，本篇按规定没有下载数据集。
- 2024 年盲测真实录音的具体来源（官方说不公开）；2026 年的来源只写了「public sources」。
- 2025 年 Track 1 表 2 只公开了 22 行中的 12 行，§2.2 那组 KRCC 的行不全。
- URGENT-PK（arXiv 2506.23874）的系统级相关具体数字没读。
- 2026 年 Track 2 各队的 LCC / SRCC 数字，在总结论文里只有定性描述（看排行榜 `competitions/17`）。
- 2024 年训练合成配方的完整概率表（论文只给了 SNR 和混响概率，2024 仓库没有克隆）。

## 来源清单

**论文**
- Zhang et al., *Lessons Learned from the URGENT 2024 Speech Enhancement Challenge*, Interspeech 2025, arXiv 2506.01611（§2.1–2.3、§3.1–3.2、表 1、图 3–4）
- Zhang et al., *URGENT Challenge: Universality, Robustness, and Generalizability for Speech Enhancement*, Interspeech 2024, arXiv 2406.04660
- Saijo et al., *Interspeech 2025 URGENT Speech Enhancement Challenge*, arXiv 2505.23212（§3.2–3.5、§4.1–4.2、表 2–3）
- *P.808 Multilingual Speech Enhancement Testing: Approach and Results of URGENT 2025 Challenge*, arXiv 2507.11306（§3.1、表 1–3、引文 [35]–[41]）
- Li et al., *Less is More: Data Curation Matters in Scaling Speech Enhancement*, arXiv 2506.23859（§II-C、表 I–III、图 3–4）
- *ICASSP 2026 URGENT Speech Enhancement Challenge*, arXiv 2601.13531（§2–3、§5–6）
- Leglaive et al., *Objective and subjective evaluation of speech enhancement methods in the UDASE task of the 7th CHiME challenge*, arXiv 2402.01413（§6.2.5、图 5）
- Wang et al., *URGENT-PK*, arXiv 2506.23874（只读了 README）

**官方页**
- https://urgent-challenge.github.io/urgent2024/data/ · https://urgent-challenge.github.io/urgent2025/data/ · https://urgent-challenge.github.io/urgent2026/ranking/
- https://urgent-challenge.github.io/urgent2026/assets/files/FinalRankingDetails_URGENT2026.pdf（2025-12-19，前 6 名的 ACR / CCR 结果）

**源码（`/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`）**
- `urgent2024_analysis/README.md`、`mos/mos_correlations.png`、`tagging/README.md`、`tagging/blind_test_tags.tsv`、`tagging/nonblind_test_tags.tsv`
- `urgent2025_challenge/conf/simulation_train.yaml:58-110`、`wind_noise_simulation_train.yaml:1-11`、`validation_data_selection.yaml:1-30`
- `urgent2026_challenge_track1/conf/simulation_train.yaml:19-71`、`meta/train_selected_700h`
- `urgent2026_challenge_track2/BENCHMARK.md:1-98`、`README.md:195-240`
- `URGENT-PK/README.md`

**【实测】的计算方法**：KRCC 用 Kendall tau-b，是我用 python3 按论文表格原值手算的（2024 年表 1、2025 年 arXiv 2507.11306 表 3、2025 年表 2 中的名次），复核 2024 年图 4 时误差 ≤ 0.01。难样本率按 `tagging/README.md` 里的难样本清单与 `*_tags.tsv` 交叉统计。脚本只在本机跑过，没有进仓。
