# 面向音素重建的时频加权损失（Time–Frequency Weighted Losses for Phoneme Reconstruction in DNN-Based Speech Enhancement）

- 出处：Nasser-Eddine Monir, Paul Magron, Romain Serizel · 2026 · Université de Lorraine / Inria / LORIA · arXiv:2606.21635v1（p.1）。p.5 有 ISCA 生成式 AI 声明，投稿方向为 ISCA 会议，具体名文中未写
- 文件：`resources/papers/training/2026-monir-tf-weighted-loss.pdf` · sha256 `387d3d62b193911a0016e0edca0c52883d21f67fefe9d9fd1c6b95aebbb1dda6`
- 是否读完全文：是（5 页；p.2–3 看了原图核对表 1、表 2 的显著性加粗）
- 代码：https://github.com/Nasseredd/fw-se-loss（p.2 脚注）

## 模型与参数量
- 降噪算法：FaSNet，端到端时域两级自适应波束形成，多通道（p.2）。
- 4 通道双耳助听器麦克风配置；评测用左耳（p.2）。
- 参数量：文中未写。

## 训练数据与混音做法
- 沿用作者前作 [15] 的数据流程（p.2）。
- 语音：LibriSpeech；噪声：语音谱形噪声 SSN + Disconoise 生态噪声（p.2）。
- 混响：Pyroomacoustics 模拟 RIR，随机房间尺寸与 RT60（p.2）。
- 训练 SIR 范围 [-10, 10] dB（p.2）。
- 测试：白噪声 WN 与 SSN 两套，10 名说话人（5 男 5 女）；目标 0°，干扰 45°；实测 RIR（Binaurec）；SIR -8 到 8 dB（p.2）。

## 损失
- 基线 L_T：时域 SI-SDR 损失（p.1）。
- 时频加权 SDR（p.1 式 1）：在梅尔带幅度上，把估计投影到干净语音，算加权的「投影能量 / 残差能量」，取 -10 log10。
- 三种权重 w(f,t)：
  - L_SIR·SP（p.2 式 2–4）：w = σ(−SIR + τ1) · σ(|S|^γ + τ2)。τ1、τ2 可学。
    意思是「语音在、但被噪声压着」的格子权重高。
  - L_SIR·SP·SF（p.2 式 5–6）：再乘 (1 + k·σ(SF(t)))。
    SF 是逐帧谱通量：本帧比上一帧增加的能量 / 上一帧总能量。k 在验证集上调到 0.2（p.2）。
    目的：突出爆破音等瞬态。
  - L_learn（p.2 式 7）：只随频率变的可学权重 softmax(θ_f)，用 ANSI 1997 频带重要性初始化。
- 前作的 L_logSIR：w = softmax(−log SIR)（p.1）。

## 优化器、学习率、批量、步数
文中未写（只说沿用前作 [15] 的训练流程，用 Asteroid 工具包）（p.2）。

## 训练硬件与时长
- 硬件：Grid'5000 计算平台（p.5 致谢），型号文中未写。
- 时长：文中未写。

## 评测方法与主要数字
- 指标：SI-SDR / SIR / SAR，及其频率加权版（权重 ∝ |S|^0.2）；STOI；Wav2Vec2 的 WER 与音素准确率 PA（p.2）。
- 显著性：每种方法对基线做 Wilcoxon 符号秩检验，α=5%，显著者加粗（p.2）。
- 句级平均，-8 到 8 dB（p.2 表 1）：
  - WN：SIR 13.3→17.2，FW-SIR 5.1→8.4，FW-SDR 4.7→5.2，STOI 0.63→0.64（L_T → L_SIR·SP·SF）。
    但 SAR 2.4→1.8、SDR 1.7→1.5，基线显著更好。
  - SSN：STOI 0.63→0.61，基线显著更好；FW-SDR 4.4→4.9。
  - L_logSIR 在 SSN 上 SDR -1.6，STOI 0.57，最差。

## 消融里哪些做法真有效（带数字）
- 音素准确率 PA（p.3 表 2，L_T → L_SIR·SP·SF，全部显著）：
  - 辅音：WN 34.0→36.1；SSN 43.6→45.5。
  - 元音：WN 43.7→45.5；SSN 46.4→47.9。
- 爆破音 PA（WN，p.4 图 3）：从 0 dB 起 L_SIR·SP·SF 显著优于基线；-8 dB 附近基线略高。
- WER（p.3 图 1）：WN 下加权损失在多数 SIR 档 WER 更低；SSN 下 0 dB 以下基线 WER 最低，L_learn 全档更差。
- 爆破音频谱（p.4 图 2）：0 dB 与 8 dB 时 L_SIR·SP·SF 在中频（梅尔带 6–13）更贴近干净谱；-8 dB 时反而更远。
- 只加 SIR 门（L_SIR·SP、L_logSIR）在 SSN 上不稳；加谱通量后最稳（p.3）。
- 代价：SAR / SDR 小幅下降（0.2–0.6 dB），即残留伪影略多（p.2 表 1）。

## 对 GTCRN 加训：能借什么（【推断】）
- 谱通量权重直接对着「字头被吞」：辅音起始是谱通量高的帧，给这些帧加权能让模型少压它们。
  做法便宜：只用干净语音算 SF(t)，乘到 HybridLoss 的逐帧项上，k 从 0.2 起扫。
- 「语音在且 SIR 低」的门 σ(−SIR+τ) 可以用在低 SNR 场景。
- 用 Wilcoxon 符号秩检验对基线做逐句配对比较，可借到我们的 verifier 判「更好」。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 本文是 4 通道波束形成、时域模型、助听器场景；GTCRN 单通道、频域、复数掩码。迁移效果未知。
- 测试噪声只有白噪声和 SSN 两种平稳噪声，没有真实非平稳噪声。
- -8 dB 等很低 SIR 时，加权损失不如基线（爆破音 PA、频谱、SSN 的 WER）。
  我们的「低 SNR」弱场景可能反而变差，需单独看。
- STOI 在 SSN 上下降、SAR 下降：加权会多留伪影，可能与「停顿时背景稳定」相冲。
- 优化器、学习率等全未写，需要去读前作或代码仓。
