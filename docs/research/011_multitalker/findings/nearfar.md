# 问题 3 · 近讲 / 远讲检测（不靠认人）

## 一句话结论

**单麦能做的是「场景层」的粗分，做不到「逐帧把近讲者的声音挑出来」。** 能粗分的情形：一个人贴近麦克风（≤0.5 m），远处是扩散的 babble（≥2 m）。可用的特征有混响（DRR/C50）、包络调制深度、电平与电平起伏，用 2–4 s 滑窗、每 0.5 s 更新一次。但在 exp003 那种「邻桌 babble 比你还响（SNR −5 dB）」的情形下，盲估计器会一起失效：它们都假设只有一个主导声源，在低 SNR 和 babble 下误差最大。另外 Mac 内置麦是全向 MEMS，近场低频增强基本不存在。**双麦（拿得到原始声道时）** 能多出逐时频点、10–20 ms 级、计算量很小的「直达 / 扩散」判别（相干性 / CDR），这是唯一能落到逐帧门控上的物理线索。可开源复用且许可干净的现成实现很少；神经网络距离估计都不是因果模型，或者算力超预算。

## 证据表

| # | 方法 / 特征 | 关键数字（测试集与条件） | 帧长 / 延迟 · 算力 | 现成实现 · 许可 | 标记 |
|---|---|---|---|---|---|
| 1 | **ACE Challenge 盲估 DRR**（全频带，所有噪声与 SNR 合并） | 相关系数最好的是「beamspace PSD 估计 + 偏差补偿」（手机 3 麦）：bias 1.07 dB，MSE 8.14 dB²（RMSE≈2.85 dB），ρ=0.577。**单通道**最好的是 NIRAv2：bias −1.85 dB，MSE 14.8（RMSE≈3.85 dB），ρ=0.558。多通道 EM32 粒子速度法 ρ=0.449 | 逐句（整句）估计；报告里没有逐帧数字 | 多数参赛方法未开源 | 【论文】arXiv 1606.03365（Tables 7–8，全频带汇总） |
| 2 | ACE 盲估 T60（参照） | 单通道最好的是 QA Reverb，ρ=0.778。babble 下 ρ：18 dB 时 0.873，−1 dB 时降到 0.759 | 整句 | — | 【论文】1606.03365（Tables 9–20） |
| 3 | NIRA（BLSTM 估 DRR） | 在 ACE 上 DRR 的 RMSD 是 3.84 dB；在开发集里留出的 10% 上只有 0.63 dB，作者自认可能过拟合 | 整句 | 未查到代码 | 【论文】arXiv 1510.04616 |
| 4 | **盲估 C50**（CNN-LSTM，约 74k 参数） | 自建评测集 RMSE≈3 dB，ρ≥0.96。**ACE 实录**上 ρ 只有 0.6–0.77，8 个模型里只有 3 个胜过「永远输出均值」的哑估计器。SNR 2 dB 时 RMSE 升到约 4–5 dB，babble 最差 | 1 s 样本是基准，可变长；帧长 128 最好 | 未查到代码 | 【论文】Gamper MMSP 2020（Table IV，Fig. 4–8） |
| 5 | Mack/Deng/Habets 2020：单通道 DRR，用掩模提取直达 / 混响两路 | 摘要称在 ACE 上优于单通道和多通道的 SOTA；**具体 dB 未查到** | 未查到 | 未查到 | 【论文】Interspeech 2020-2171 |
| 6 | **SRMR**（调制谱，语音对混响调制能量比） | 改进版 SRMR 估 DRR，误差比原版最多降 18% | 原定义是 256 ms 窗、整句平均（【推断】按原论文惯例，本次未逐页核对） | SRMRpy（GitHub license = NOASSERTION/Other，**许可不明**）；torchmetrics 内的 SRMR（Apache-2.0，但依赖 PyTorch） | 【论文】arXiv 1510.04707；【官方】api.github.com/repos/jfsantos/SRMRpy |
| 7 | **Neri 等单通道说话人距离估计**（CRNN + 注意力，约 650k 参数，**双向 GRU**） | 合成无噪 MAE 0.11 m。合成加噪：30 dB 时 1.14 m，0 dB 时 1.39 m。混合数据 QMULTIMIT 1.48 m；VoiceHome-2 实录 0.63 m；STARSS 0.42 m | 输入 10 s（STARSS 是 2 s），32 ms 窗；**不是因果模型**，论文没谈实时 | github.com/michaelneri/audio-distance-estimation，**MIT** | 【论文】arXiv 2403.17514（Tables I–VII）；【官方】GitHub API |
| 8 | **Distance-Based Sound Separation**（Google，单麦，近 / 远分离） | 1 近 + 4 远、阈值 1.5 m：近讲 SI-SDRi **4.4 dB**，远处 6.8 dB。阈值 0.8 m 时 4.0 / 8.3 dB；3.0 m 时 1.9 / 2.7 dB。近讲有 2 人时只剩 2.2 dB | 单向 LSTM，32 ms 窗、16 ms 跳。**纯仿真**，仿真器**不含近场效应**；RT60 没有单独考察 | **没有代码**（只有 demo） | 【论文】arXiv 2207.00562（Tables 1–2） |
| 9 | NS-Extractor（距离线索 + 自注册的 TSE，Interspeech 2023） | LibriSpeech 仿真：2 人 SI-SDRi 8.75 dB，3 人 6.82 dB，4 人 5.34 dB。阈值内混进了第二个说话人时，去掉说话人编码器降到 3.53 dB | 基于 TF-GridNet + **BLSTM**，非因果；16 ms 窗，8 ms 跳 | 无代码（005 已记） | 【论文】Interspeech 2023-218（Tables 1–3，p.2491） |
| 10 | 单通道距离分离，面向手机 GPU（ICASSP 2025） | 阈值 0.5 m，1 近 1 远：SI-SDRi 5.5 / 12.0 dB | **1.3M 参数，25.7 GMAC/s**；S23 的 CPU 上 RTF 1.55（跑不实时），GPU 上 0.16；未说明是否因果 | 无代码；论文 CC BY-NC-ND | 【论文】arXiv 2501.03045（Tables I–III） |
| 11 | **Hush**「只留最响的人」 | 训练时背景说话人比主讲**低 12–24 dB SIR**；README 自述「very loud competing speakers may not be fully suppressed」 | 约 20 ms 算法延迟，每 10 ms 帧 <1 ms | Apache-2.0（训练数据含 ESC-50 NC，005 已记） | 【源码】`resources/Hush/README.md:76,334,429` |
| 12 | 前景 / 主讲 VAD（无注册），商业参照 ai-coustics VAD Voice Focus 2.0 | 内部基准（未公开）：准确率 92.4%，误报 5.5%；对照组普通 VAD 是 71.0% / 28.5%。原文明说**判主讲靠的不是距离** | 30 ms 延迟，CPU 实时 | 闭源 SDK，不可用 | 【传言】ai-coustics.com/blog/vad-voice-focus-2.0 |
| 13 | **近场效应（proximity effect）** | 只出现在压差 / 指向性麦上（8 字形 > 心形 > 全向几乎没有）；有 AES 论文做「近场效应检测」，但没有用它判别语音距离的公开准确率 | 实时分频带对比，很便宜 | 无 | 【传言】厂商博客；【论文】Clifford & Reiss AES 2011（repository.uwl.ac.uk/id/eprint/505，只读到摘要） |
| 14 | **双麦 CDR**（相干与扩散功率比） | 只要知道目标 DOA **或** 噪声场相干性之一，就能得到无偏估计；可逐帧实时运行 | 逐时频点闭式解，FFT + 递归平滑即可，算力极低 | **没有找到开源实现**，需按论文公式自己写 | 【论文】arXiv 1502.03784（Schwarz & Kellermann TASLP 2015）；DAGA 2015 实时版 |
| 15 | babble 人数估计 | 由 babble 模型估计「人数区间」，在 Switchboard 110 h 上评测；**准确率未查到** | — | — | 【论文】Krishnamurthy & Hansen, TASLP 17(7) 2009 |
| 16 | 专利思路（DRR / 早晚能量比阈值判近场与远场） | 近场 / 远场的边界取 0.5–3 m，因设备而异；做法是逐帧输出 → 平滑 → 阈值 | — | 专利，不可抄 | 【传言】USPTO 12126971 等 |

## 对本项目的含义

**1. 0.5 s 判断回路能用什么（单麦，只改参数）**

| 特征 | 取窗 | 能判什么 | 算力（推断） | 落点 |
|---|---|---|---|---|
| 电平（近讲者比 babble 高多少）+ 短时电平起伏（4 Hz 包络调制深度、峰谷比） | 1–2 s 滑窗，每 0.5 s 更新 | 是「一个近讲者主导」还是「平滑的多人 babble」：人数越多，包络越平。这是最便宜也最稳的一维 | 能量包络 + 一个带通，≪0.1% 核 | 场景层：选「多人嘈杂」场景，调 PSE 门控强度 |
| SRMR 类调制谱比 / 混响度 | 2–4 s | 近讲干、远讲湿 | 一个小滤波器组，<1% 核【推断】 | 同上，作为确认量 |
| 神经 C50/DRR 估计（约 74k 参数级） | 1–4 s | 当前主导声源的混响量 | 一次 1 s 推理约 0.01 GMAC 量级【推断】，每 0.5 s 一次可忽略 | 可选，要先实测 |
| 频谱倾斜 | 1 s | 远讲和混响高频衰减大；但受人声、口型、麦克风 EQ 干扰大 | 极低 | 只作辅助量 |
| 近场效应 | — | **Mac 内置麦和多数耳机都是全向 MEMS，基本不出现**；只有外接心形麦有用 | — | 不建议作主特征 |

- **延迟**：这些量都在旁路计算，不进音频主路径，**附加延迟 0 ms**；判断响应时间是 1–4 s 窗口加平滑，所以只适合切场景或调参数，不适合逐帧门控。
- **致命局限**：exp003 的 −5 dB babble 正好落在所有盲估计器最差的区间（C50 在 2 dB 时 RMSE 约 4–5 dB；ACE 的 babble、−1 dB 一档最差）。盲估计器估的是混合信号整体，babble 比你响时，估出来的是 babble 的混响。【推断】所以只有在近讲者说话的片段里，这些特征才有区分力；需要先用 VAD 能量门槛挑出「本地峰值帧」再统计（这和 Hush / VoxLock 自动注册取「比底噪高 10 dB 的帧」是同一个思路）。
- **神经距离估计**（Neri，MIT 许可可借鉴）：双向 GRU、10 s 输入，不能直接流式；改成因果版、2 s 窗需要重训。实录误差 0.4–0.6 m，**区分「0.5 m 对 3 m」够用，区分「0.5 m 对 1 m」不够**。放在 M2 之后作可选研究。
- **端到端近 / 远分离模型**（Google DSS、NS-Extractor、2501.03045）：没有可用代码，或者非因果、算力是 GTCRN 的几百倍（25.7 GMAC/s），**不进主路径**。DSS 的数字也说明上限不高：1 近 4 远时近讲 SI-SDRi 只有 4.4 dB。
- **Hush 的「最响」假设**：训练时 SIR 是 12–24 dB，**正好不覆盖 −5 dB 邻桌 babble**，和 exp003 的失败是同一个原因。
- **许可**：Neri 代码 MIT 和 torchmetrics Apache-2.0 都与 GPL-3.0 兼容；SRMRpy 许可不明，不要直接用（照论文自己实现）；2501.03045 是 CC BY-NC-ND，只能读。

**2. 双麦多出什么**

- **相干性 / CDR**：逐时频点、10–20 ms 帧，直接给出「直达（近、相干）对扩散（远处 babble、混响）」的软掩模。可以在每 10 ms 里调 GTCRN 输出的增益下限，或作 PSE 门控的辅助输入；算力极低，延迟只多 0–1 帧（复用同一个 STFT）。
- **局限**：远处单个说话人在近距离内也是相干的，CDR 只把「扩散的 babble」和「近讲」分开，分不开「1.5 m 外一个清楚说话的人」。麦间距决定了可用频带（间距小，低频相干性都接近 1）。两麦电平差（近场 ILD）在间距 2 cm、距离 0.5 m 时只有约 0.34 dB（20·log10(0.52/0.5)，【推断】几何计算），实际不可用。
- **落点**：M2 之后的「双麦增强」选项，**前提是 macOS 能拿到内置阵列的原始多声道**（由另一条线 · macOS 多声道调研回答）。

**3. 判断（用户只问能否作 0.5 s 回路的特征）**：**能，但只适合调参与切场景，不适合当门控本身。** 建议的最小组合是「电平差 + 包络调制深度 + 一个便宜的混响度量」，只在本地峰值帧上统计，输出「近讲主导 / babble 主导 / 不确定」三态，用来调 PSE 门控强度和 GTCRN 增益下限。

## gap（需另开实验或查不到）

1. **实测**：在 exp003 素材加若干条自录（MacBook 内置麦，0.3 / 0.5 / 1 / 2 / 3 m）上，比较电平、4 Hz 调制深度、SRMR、频谱倾斜的近 / 远 AUC，在 −5 / 0 / +5 dB babble 下各测一次。
2. **实测**：只在本地峰值帧上统计，能否扛住 −5 dB babble。
3. **实测**：Neri 模型（MIT）在 2 s 窗、单向化之后的误差（需要重训，属于实验线）。
4. **查不到**：ACE 里 DRR 分 babble、分 SNR 的单独数字（报告只给全部噪声合并的结果）；Mack 2020 的 dB 误差；Krishnamurthy & Hansen 的人数估计准确率；SRMR 的确切窗长（本次未逐页核对）。
5. **查不到**：任何开源 CDR / 相干性估计实现（C / Rust / Python）。
6. **依赖其他线**：MacBook / AirPods 能否在 CoreAudio 拿到原始多声道、麦间距是多少。
7. **未覆盖**：ai-coustics 是否完全在端上运行、数据集多大，均未公开。

## 来源清单

- ACE Challenge Results Technical Report — arXiv 1606.03365（https://arxiv.org/html/1606.03365v4）
- NIRA on ACE — arXiv 1510.04616
- SRMR variants for blind room acoustics — arXiv 1510.04707
- DRR on ACE with two-channel beamformer (DENBE) — arXiv 1510.07546
- Gamper, Blind C50 estimation, MMSP 2020 — https://www.microsoft.com/en-us/research/uploads/prod/2020/10/Blind_C50_estimation_Gamper_MMSP_2020.pdf
- Mack, Deng, Habets, Single-channel blind DRR estimation using masking, Interspeech 2020 — https://www.isca-archive.org/interspeech_2020/mack20_interspeech.html
- Neri et al., Speaker Distance Estimation in Enclosures from Single-Channel Audio — arXiv 2403.17514；代码 https://github.com/michaelneri/audio-distance-estimation（MIT）
- Patterson et al., Distance-Based Sound Separation — arXiv 2207.00562
- Lin et al., Focus the Sound around You (NS-Extractor), Interspeech 2023 — https://www.isca-archive.org/interspeech_2023/lin23b_interspeech.pdf
- Single-Channel Distance-Based Source Separation for Mobile GPU — arXiv 2501.03045
- Schwarz & Kellermann, Coherent-to-Diffuse Power Ratio Estimation for Dereverberation — arXiv 1502.03784
- Hush — `resources/Hush/README.md:76,334,429`
- ai-coustics VAD Voice Focus 2.0 — https://ai-coustics.com/blog/vad-voice-focus-2.0
- Clifford & Reiss, Proximity effect detection — https://repository.uwl.ac.uk/id/eprint/505
- Krishnamurthy & Hansen, Babble noise — https://personal.utdallas.edu/~jxh052100/Publications/JP-61-IEEE-ASLP-Babble-NitishK-JohnH-Sept09.pdf
- SRMRpy — https://github.com/jfsantos/SRMRpy；torchmetrics SRMR — https://lightning.ai/docs/torchmetrics/stable/audio/speech_reverberation_modulation_energy_ratio.html
- 已有调研（未重复）：`noise_canceling/docs/research/005_pse/`、`006_sota-landscape/`
