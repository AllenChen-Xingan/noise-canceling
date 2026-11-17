# 并行 WaveGAN 与多分辨率 STFT 损失（Parallel WaveGAN: A fast waveform generation model based on generative adversarial networks with multi-resolution spectrogram）

- 出处：Ryuichi Yamamoto, Eunwoo Song, Jae-Min Kim · 2019（v2 2020）· LINE / NAVER · arXiv:1910.11480v2（p.1）。会议：文中自带参考链接指向 ICASSP 2020 演示页（p.3 脚注 5）
- 文件：`resources/papers/training/2019-yamamoto-parallel-wavegan-mrstft.pdf` · sha256 `f91b87aef598450f590a22b08aef92e5ffac03eb172001493dc6c8a9ba2452c4`
- 是否读完全文：是（5 页；p.4 看了原图核对表 2–4）
- 注意：这是声码器（语音合成）论文，不是降噪论文。

## 模型与参数量
- 生成器：非因果、非自回归的 WaveNet，30 层膨胀残差卷积，3 个膨胀周期，残差与跳连通道 64，卷积核 3（p.3）。
- 判别器：10 层非因果膨胀 1-D 卷积，leaky ReLU（α=0.2）（p.3）。
- 卷积层全用 weight normalization（p.3）。
- 参数量：Parallel WaveGAN 1.44 M；ClariNet 2.78 M；WaveNet 3.81 M（p.4 表 2）。

## 训练数据与混音做法
- 日语女声专业录音，24 kHz，16 bit（p.3）。
- 训练 11,449 句（23.09 h），验证 250 句（0.35 h），测试 250 句（0.34 h）（p.3）。
- 条件特征：80 维对数梅尔谱，70–8000 Hz，帧长 50 ms、帧移 12.5 ms（p.3）。
- 无混音，无噪声。

## 损失
- 单分辨率 STFT 损失 = 谱收敛 L_sc + 对数幅度 L_mag（p.2 式 3–5）：
  - L_sc = ‖|STFT(x)|−|STFT(x̂)|‖_F / ‖|STFT(x)|‖_F
  - L_mag = (1/N)‖log|STFT(x)| − log|STFT(x̂)|‖_1
- 多分辨率：M 个分辨率的单分辨率损失取平均（p.2 式 6）。
- 三组设置，Hanning 窗（p.3 表 1）：

| 组 | FFT 点数 | 窗长 | 帧移 |
|---|---|---|---|
| 1 | 1024 | 600（25 ms） | 120（5 ms） |
| 2 | 2048 | 1200（50 ms） | 240（10 ms） |
| 3 | 512 | 240（10 ms） | 50（约 2 ms） |

- 作者给的理由：多分辨率兼顾时间与频率分辨率，并防止生成器过拟合到某一个固定 STFT 表示（p.2）。
- 生成器总损失 L_G = L_aux + λ_adv L_adv，λ_adv = 4.0（p.2 式 7，p.3）。
- 对抗损失用最小二乘 GAN（p.2 脚注 3）。

## 优化器、学习率、批量、步数
- RAdam，ε=1e-6（p.3）。
- 学习率：生成器 1e-4，判别器 5e-5；每 200 K 步减半（p.3）。
- 批量 8，每段 24 K 样本点（1.0 s）（p.3）。
- 共 400 K 步；前 100 K 步判别器固定，之后联合训练（p.3）。

## 训练硬件与时长
- 两张 NVIDIA Tesla V100（p.3）。
- 训练时长：Parallel WaveGAN 2.8 天；WaveNet 7.4 天；ClariNet 12.7 天；ClariNet-GAN 13.5 天（p.4 表 3）。

## 评测方法与主要数字
- MOS：18 名日语母语听者，随机取测试集 20 句，5 分制（p.3）。
- 推理速度：单张 V100 上比实时快 28.68 倍（p.4 表 2）。
- 分析-合成 MOS（p.4 表 2，95% 置信区间）：
  - 录音原声 4.46±0.08；
  - ClariNet + 多分辨率 3.88→4.21（单分辨率 3.88±0.11，多分辨率 4.21±0.09）；
  - Parallel WaveGAN 单分辨率 1.36±0.07，多分辨率 4.06±0.10。
- 接 Transformer TTS 后（p.4 表 4）：Parallel WaveGAN 4.16±0.09；ClariNet-GAN 4.14±0.10。

## 消融里哪些做法真有效（带数字）
- 多分辨率 vs 只用第 1 组单分辨率（p.4 表 2）：
  - ClariNet：3.88 → 4.21 MOS（+0.33）；
  - Parallel WaveGAN：1.36 → 4.06 MOS（+2.70）。
  - 单分辨率的 Parallel WaveGAN 基本不可用。
- 加 STFT 损失的系统都优于不加的自回归 WaveNet 3.61（p.3–4）。作者解释：WaveNet 只靠限带梅尔，高频有噪声。
- 对抗损失：分析-合成时 ClariNet 加不加都是 4.21（p.4 表 2）；接 TTS 时 4.00→4.14（p.4 表 4）。
- 样本量：每个系统 20 句 × 18 人。

## 对 GTCRN 加训：能借什么（【推断】）
- 多分辨率 STFT 可当「第 5 项损失」的候选，FFT 组可按 16 kHz 换算：
  例如 {512/240/50, 1024/600/120, 2048/1200/240} 已被 DEMUCS 在 16 kHz 降噪上直接沿用。
- 它的长处是约束「只看一个 STFT 时不可见」的时域细节，对应 GTCRN 单一 512/256 分辨率的盲区。
  对「字头被吞」这种短时瞬态，短窗那一组（10 ms 窗、约 2 ms 帧移）可能有用。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 本文任务是声码器，生成器从零生成波形；单分辨率失败（1.36）的情形对降噪不一定成立。
  降噪模型有含噪输入兜底，单分辨率损失通常已可用。
- 对数幅度 L_mag 在静音频点上误差会被放大（Braun 2020 损失综述也提到 LSD 的同一问题）。
  对「停顿时背景」可能起反作用，需截断或加权。
- 对抗损失不建议进第一轮监督微调（与已定 D81 精神一致，且本文显示其收益只在 TTS 链路上出现）。
- 本文数字是 MOS，样本只有 20 句，不是降噪指标。
