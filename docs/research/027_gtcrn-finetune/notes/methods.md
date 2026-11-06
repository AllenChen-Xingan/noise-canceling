# 027 · 微调方法文献（子代理笔记）

> 2026-11-05 · 会话内子代理（只读网页、GitHub 文件与 arXiv PDF；未构建、未训练、未装包）。
> 这是子代理原文的整理版；**结论以 `../decision.md` 为准**。主会话亲手复核过的条目在 `../sources.md` 标「复核」。
> 范围：只看「在现有权重上微调」。加注意力头（028）、强化学习（029）、辅助小模型（030）不在这里。

## 1. GTCRN 官方训练配置

GTCRN 没有 arXiv 版；IEEE 正文未读到（未核）。只能拼两处：作者海报 + 上游训练仓库 SEtrain（通用模板，两处有出入）。

- 论文：Rong et al., ICASSP 2024, pp. 971–975, DOI 10.1109/ICASSP48485.2024.10448310（正文未核）
- 海报（SigPort「Datasets」栏）：DNS3 约 2000 小时 + DiDiSpeech 中文；SNR −5～15 dB；每轮 40,000 对 8 s 片段；验证 840 对、测试 800 对。用了 DNS3 哪些子集：未核
- SEtrain `configs/cfg_train.yaml`（提交 3ec437d）：
  - 学习率 1e-3（第 10 行）；线性预热 25k 步后余弦降到 1e-6，到第 250k 步（第 14–17 行），按步更新（第 19 行）
  - 损失：压缩 0.3、λ_ri = 30、λ_mag = 70（第 30–33 行）——与本地 `resources/gtcrn/loss.py:16-30` 一致
  - 片段 10 s，总 72,000 条（注释 200 h），每轮 10,000 条（第 36–38 行）
  - 每卡 batch 8 × 2 卡（第 7、43 行）；200 轮；梯度裁剪 3.0（第 61、63 行）
- 优化器 `train.py:70` 用 Adam；调度器 `scheduler.py:27-48`
- 总步数（推断）：10,000 ÷ (8 × 2) = 625 步 / 轮 × 200 ≈ 12.5 万步
- 混音 `prepare_datasets/gen_DNS3_datasets.py`：名义 SNR −5～15 dB（第 64 行），但第 32 行 `alpha=10**(-snr*1.5/20)`，读源码推断实际约 −7.5～22.5 dB；加 DNS 房间冲激响应，目标取前 100 ms 早期混响（第 100–107 行）；幅度随机缩放 0.01–0.51（第 26 行）

子代理建议（推断）：微调学习率取官方峰值的 1/10～1/100（1e-4～1e-5），短预热，保留裁剪 3.0；混音沿用「早期混响当目标 + 1.5 倍 SNR 缩放」，否则数据分布先变了。

## 2. 防「灾难性遗忘」

已核三篇都表明：直接微调会明显伤原场景；正则、回放、低秩适配器能缓解，不能完全不退。

| 来源 | 做法 | 数字 |
|---|---|---|
| SERIL，arXiv 2005.11760 | EWC 式拉回 λΣF_i(θ_i−θ*_i)²（式 5）+ 路径重要度（式 9） | 表 1 原场景 SDR：未适配 11.75 → 直接微调 6.99 → SERIL 9.31；PESQ 2.708 → 2.406 → 2.461；摘要称遗忘减少 52%；每个新场景 1,800 对 |
| SAFE，Archives of Acoustics 51(2), 2026（SEGAN，§3.3、表 3） | EMA 权重平均（0.995）+ L2-SP（λ = 1e-4）+ 老师一致性（λ = 0.1）+ 原 : 新 = 2 : 1 回放 | 新场景 300 对；原场景 PESQ 约 1.84、STOI 约 90%；消融称 EMA 最管用。期刊较小 |
| 低秩适配器，arXiv 2603.07471（ICASSP 2026，§4.3、表 1–2） | 骨干冻结，只更新 < 1% 参数（512～708 个），秩 1、缩放 64 | lr 1e-3 / 5e-4；batch 24 × 2 s；每场景 20 次更新；SI-SDR 平均 +1.51 dB；连续换场景不漂移 |
| arXiv 2607.11630（只读到摘要） | 降噪 → 歌声分离，全量微调 vs LoRA | 全量微调伤降噪，LoRA 保住；原文数字未核 |

子代理建议（推断）：回放原领域 30–50%；L2-SP 从 1e-4 起扫；原检查点当老师加一致性项（λ ≈ 0.1）；维护 EMA 权重并用它评估。48K 参数的模型只冻层 / 只加 LoRA 能不能学到够：未量化。

## 3. 保字头 / 防过度压制的可微损失

每篇都给出「压得更狠」与「语音不受损」的权衡数字，没有一篇两者同时变好。

| 来源 | 做法 | 数字 |
|---|---|---|
| VoiceFilter-Lite，arXiv 2009.04323（式 2–3、表 1） | 压过头的残差（x > 0）乘 α，原文 α = 10 | 安静 WER 9.1 → 8.8；但语音噪声加性条件 25.5 → 35.5，降噪收益丢一部分 |
| 加权语音失真，arXiv 2001.10601（式 12、§2、图 3、表 2） | αL_speech + (1−α)L_noise；语音段按能量门限判（300–5000 Hz，比峰值低 30 dB 以上不算） | 客观最优 α = 0.65，作者选 0.35；α < 0.35 听到语音失真 |
| 按语音段加权，arXiv 2302.11768（§4.1） | 语音帧权重 μ = 0.5 / 0.75 / 0.9 | μ = 0.5 过度压制最多；μ 越大残噪越多，总质量 0.9 最好 |
| 字头 / 瞬态加权，arXiv 2606.21635（表 1） | SIR × 语音存在 × 频谱通量加权 | 白噪声下 SIR 13.3 → 17.2，SAR 2.4 → 1.8，STOI 0.63 → 0.64；多通道、白噪声，离我们较远 |
| 综述，arXiv 2009.12286（§3.4、表 1） | 压缩指数 | 0.3 最常用；GTCRN 也用 0.3 |

子代理建议（推断）：HybridLoss 上叠「只在语音帧上的不对称幅度项」（α 从 2～4 起扫，原文 10 是给识别用的，对听感太猛）+ 频谱通量加权项照顾字头；验收同时看降噪量与保字头。

## 4. 小模型微调要多少数据 / 步数

- 10 万参数以下因果模型的全量微调实例：**未找到**
- 适配器级：arXiv 2603.07471，DPRNN 骨干 89k 参数，每场景 20 次更新 × 24 段 × 2 s（480 s 音频），SI-SDR +1.51 dB
- SERIL：每场景 1,800 对，模型约 3 层 LSTM × 257，不算小；步数未核
- 调研 013：FastEnhancer 完整训练约 12.8 万步

子代理建议（推断）：先跑 5k～20k 步，每 1k 步出一次检查，看降噪是否饱和、字头是否开始退。

## 5. 风噪

风噪能量集中在低频（大多 < 4 kHz），阵风 / 湍流式非平稳；风大时麦克风膜片非线性并削波，不能只当加性噪声。

- arXiv 2306.12867（表 1、§4.1）：一半模拟、一半实录；阵风数 U(1, 10)；SNR U(−6, 14) dB；以风噪为侧链的压缩器模拟膜片（比率 1–20、起音 5–100 ms、释放 5–500 ms）；75% 样本加削波（门限 0.85–1.0）；训练 25 小时；代码 `sp-uhh/storm`（MIT）
- arXiv 2507.01821（§4.2）：WindNetLite 249K 参数，双编码器、低频支路单独加强；真实风噪约 10 小时混成约 200 小时，SNR −20～20 dB
- 模拟器：`audiolabs/SC-Wind-Noise-Generator`（Python，MIT，按风速曲线生成，IWAENC 2022）· `ehabets/Wind-Generator`（MATLAB，GPL-3.0，多通道 Corcos 模型，arXiv 1805.09679）

子代理建议（推断）：用 MIT 的 Python 模拟器生成，套压缩器 + 削波；GTCRN 低频前 65 个频点（约 0–2 kHz）不做 ERB 压缩（`resources/gtcrn/gtcrn.py:280` `ERB(65, 64)`），低频分辨率够；风噪 SNR 下限放到 −10 dB。

## 来源

- https://ieeexplore.ieee.org/document/10448310
- https://sigport.org/sites/all/modules/pubdlcnt/pubdlcnt.php?fid=9363
- https://github.com/Xiaobin-Rong/SEtrain（`configs/cfg_train.yaml` · `train.py` · `scheduler.py` · `prepare_datasets/gen_DNS3_datasets.py`）
- https://arxiv.org/abs/2005.11760
- https://acoustics.ippt.gov.pl/index.php/aa/article/download/4315/3578/17018
- https://arxiv.org/abs/2603.07471
- https://arxiv.org/abs/2607.11630
- https://arxiv.org/abs/2009.04323
- https://arxiv.org/abs/2001.10601
- https://arxiv.org/abs/2009.12286
- https://arxiv.org/abs/2302.11768
- https://arxiv.org/abs/2606.21635
- https://arxiv.org/abs/2306.12867
- https://arxiv.org/abs/2507.01821
- https://arxiv.org/abs/1805.09679
- https://github.com/audiolabs/SC-Wind-Noise-Generator
- https://github.com/ehabets/Wind-Generator
- https://github.com/sp-uhh/storm
