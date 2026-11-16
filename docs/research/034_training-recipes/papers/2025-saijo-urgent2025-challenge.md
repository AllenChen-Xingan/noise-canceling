# URGENT 2025 通用降噪挑战赛总结（Interspeech 2025 URGENT Speech Enhancement Challenge）

- 出处：Kohei Saijo, Wangyou Zhang, Samuele Cornell 等 12 人 · 2025 · Interspeech 2025 挑战赛 · arXiv:2505.23212v2（p.1）
- 文件：`resources/papers/training/2025-saijo-urgent2025-challenge.pdf` · sha256 `14f90c34339062882767ef4fa41d89a8f5765e425537e8eb22a0ace7c0b1c674`
- 是否读完全文：是（5 页；p.4 看了原图核对表 2、表 3）
- 性质：挑战赛总结，不是单一方法论文；参赛方细节只有简短描述。

## 模型与参数量
- 官方基线 T10：TF-GridNet，约 8.5 M 参数（p.3）。
- 第一名 T1：纯判别式子带 RNN，约 102 M 参数（p.3）。
- 前 9 名多数是「判别式损失 + 对抗损失」的混合型（T2、T4、T5、T6、T9）（p.3）。
- T13：纯生成式（潜扩散 + 声码器）（p.3）。

## 训练数据与混音做法
- 7 种失真：加性噪声、混响、削波、带宽限制、编解码（MP3 / OGG）、丢包、风噪（p.2）。
- 每条含噪语音最多同时叠 5 种失真（p.2）。
- 风噪用 Lemercier 等人的模拟器（SC-Wind-Noise-Generator）（p.2 脚注 3）。
- 采样率 8–48 kHz 共 7 种（p.2）。
- 语料（p.2 表 1）：语音含 DNS5 LibriVox 约 350 h、LibriTTS 约 200 h、MLS-HQ 约 450 h（不限时 48,600 h）、CommonVoice 约 1300 h（不限时 9500 h）等；噪声含 DNS5 约 180 h、WHAM! 约 70 h、FSD50K 约 100 h、FMA 约 200 h；RIR 约 6 万条模拟。
- 预处理（p.2）：重采样到能覆盖有效频带的最低采样率；用 webrtcvad 去掉非语音和静音为主的样本；按 DNSMOS 分数滤掉含噪样本。
- 作者明说：VAD + DNSMOS 过滤不完善，过滤后仍有许多样本带可闻背景噪声；故意保留，鼓励参赛者研究怎么用（p.2）。
- 两个赛道（p.3）：
  - 赛道 1：限制 MLS 与 CommonVoice 时长，约 2.5 k 小时语音。
  - 赛道 2：不限，约 6 万小时语音。
- 验证集 1000 条；CommonVoice 部分人工挑干净样本（p.2）。
- 盲测 900 条：一半模拟、一半真实录音，含未见语言日语，每种语言 150 条；还含未见的神经编解码失真（p.2–3）。

## 损失
- 参赛方损失细节文中未逐一写，只有类别：判别式损失、对抗损失、去噪分数匹配（p.3）。
- 基线的损失：文中未写（指向官网）。

## 优化器、学习率、批量、步数
- 基线：用第一届预训练权重初始化，以节省训练时间（p.3）。
- 其余：文中未写。

## 训练硬件与时长
- 基线：单张 RTX A6000，约 2.5 天，ESPnet-SE 流程（p.3）。

## 评测方法与主要数字
- 14 个指标分 5 类：无参考（DNSMOS、NISQA、UTMOS）、有参考（POLQA、PESQ、ESTOI、SDR、MCD、LSD）、任务无关（SpeechBERTScore、音素相似度 LPS）、任务相关（说话人相似度、字准确率 CAcc）、主观 P.808 MOS（只英语）（p.3）。
- 排名：每个指标排名 → 类内平均 → 类间平均（p.3）。
- 赛道 1 部分结果（p.4 表 2）：

| 系统 | 类型 | PESQ | CAcc % | MOS | 总排名分 |
|---|---|---|---|---|---|
| T1 | 判别 | 2.64 | 79.80 | 3.24 | 2.97（第 1） |
| T3 | 混合 | 2.45 | 77.09 | 3.44 | 4.47 |
| T10 基线 | 判别 | 2.24 | 75.60 | 2.96 | 9.63 |
| T13 | 生成 | 1.34 | 67.87 | 3.69 | 12.53 |
| T22 含噪 | – | 1.31 | 73.41 | 2.13 | 20.50 |

- T13 主观 MOS 最高（3.69），但 CAcc 低于含噪输入（67.87 vs 73.41）（p.4 表 2）。
- 语言（p.4 表 3）：T13 日语 CAcc 36.8、中文 20.1，含噪输入分别 74.5、69.3。判别式 T1、T10 跨语言稳定。

## 消融里哪些做法真有效（带数字）
本文没有受控消融。与「数据量 vs 质量」相关的是赛道对比：
- 原文（p.3）：「Although larger-scale training data was available in Track2, no systems in Track2 outperform the best system in Track1, which suggests that scaling up the data does not necessarily lead to better performance in SE when the additional data contain some noisy speech.」
- 结论原文（p.4）：「more but possibly noisy data does not necessarily lead to better performance」。
- 前言原文（p.1）：「simple filtering methods (e.g., using DNSMOS scores) did not remove noisy samples very effectively」。
- 同队两赛道对照（p.4 表 2，赛道 1 → 赛道 2，数据约 2.5 k → 约 6 万 h）：
  - T3：PESQ 2.45→2.44；CAcc 77.09→77.51。
  - T5：PESQ 2.48→2.49；CAcc 78.35→78.40。
  - T6：PESQ 2.29→2.31；CAcc 77.15→76.77。
  - 变化都在 ±0.02 PESQ、±0.4 CAcc 以内。
- 证据强度：跨队比较，第一名 T1 没参加赛道 2；同队对照只 3 组，且各队如何用多出的数据不清楚。只能说「24 倍数据没换来可见提升」，不能说「更多数据有害」。
- 生成式模型在低 SNR 下会「编内容」：未见语言日语时输出像英语（p.4）。
- DNSMOS 在出现幻觉时仍给高分（p.4）。

## 对 GTCRN 加训：能借什么（【推断】）
- 加训数据要「干净优先」，不是「越多越好」：24 倍含噪数据在多支队伍上没涨分。
  对我们：宁可几十小时人工核过的干净语音，也别直接灌大规模爬取语料。
- DNSMOS 过滤不可靠，作者已实测；我们筛干净语音不能只靠 DNSMOS 阈值。
- 失真清单（削波、带宽限制、编解码、丢包、风噪）可当场景表的参照。风噪用开源模拟器即可生成。
- 用 webrtcvad 去掉静音为主的样本，是可直接借的预处理。
- 评测侧：只看 DNSMOS / MOS 会漏掉吞字和幻觉，必须配有参考的字准确率类指标。这和已定 D81、「验收靠 verifier」一致。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 参赛模型 8.5 M–102 M 参数，比 GTCRN 大 2–3 个数量级。「大数据配大模型」的结论不能直接套到 4.8 万参数。
- 小模型容量小，数据多了更可能学不动；这一点本文没测，未知。
- 赛道对比混了「数据更多」与「含噪样本更多」两个因素，无法分开。
- 对抗损失在本赛很流行，但违反第一轮只做监督微调的已定方向，不借。
