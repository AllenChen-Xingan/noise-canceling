# D-finetune · 调研代理原文（以 D-finetune-skeptic.md 核对为准）

> 2026-10-31 · Workflow wf_113bb80a-4ec · 只读调研，未运行程序。

## 结论

1. **证实，并且有直接对照**：在 MetricGAN 原文表 1（TIMIT）里，按 Koizumi 思路做的 policy gradient（PE policy grad）PESQ 是 2.098，MetricGAN 是 2.133，L1 监督基线是 2.033。两者都只比监督高约 0.1，判别器式略好【论文 arXiv 1905.04874 表 1】。Koizumi 小组 2020 年专门写了一篇论文，修补黑盒指标训练里「代理 DNN 近似误差导致不稳」的问题【论文 arXiv 2002.05879 摘要】。也就是说，两条路都不算很稳。
2. **2025–2026 年 RL / DPO / GRPO 的 SE 工作全部做在生成式大模型上**（LM 式或流匹配，几千万参数、非流式、带 vocoder）。这些论文都没报告 PESQ / WER。凡是只用单一奖励的都报告了讨好现象，多指标奖励才压得住【论文 2601.16483 §4.1；2606.21458 摘要；2507.09929 §4.1】。它们对 48K 参数的因果掩码模型没有直接证据，只能借「奖励怎么组」这一点【推算】。
3. **我们自己的上游已经有现成的 MetricGAN 式训练代码**：LiSenNet 用 PESQ 当判别器的回归目标（MIT），FastEnhancer 留着可微 PESQ 损失的开关（MIT）。GTCRN 仓库只有模型和损失，训练代码在另一个仓库 SEtrain（MIT）。
4. **在 Apple Silicon 上训练，最大的雷是 GRU**：PyTorch MPS 上的 `nn.GRU` 会输出 NaN，issue #94691 仍是 open【官方 GitHub issue】。GTCRN 一共有 GRU 5 处。MLX 有 `nn.GRU`、`ConvTranspose2d`、`LayerNorm`【官方 MLX 文档】，但需要把整个模型移植过去。
5. **分场景专家 vs 一个模型加条件**：没有找到「同参数、同延迟」的严格对照。专家混合 SE 只有「较少参数、性能超过通用模型」这一定性结论【论文 arXiv 2005.08128 摘要，未见同参数对照】。对 GTCRN 来说，**切换权重会让循环状态不连续**，所以要么共用骨干、只换小头或 FiLM，要么交叉淡变两路【推算】。

## 逐条

### 1. RL 与不可微指标优化

| 工作 | 效果数字 | 因果实时 | 代码与许可 | 讨好 / 伪影 |
|---|---|---|---|---|
| Koizumi 2017（ICASSP，pp.81–85） | 奖励用 PEASS，原文只说「随迭代变好，最终超过常规方法」；数字未能核实。2018 期刊版（arXiv 1810.09137）改用 PESQ 等指标做奖励，摘要只说「显著提高」 | 未见 | 未见 | 2020 年同组指出训练不稳【2002.05879】 |
| MetricGAN（Fu 2019） | VoiceBank-DEMAND：PESQ 2.86 / CSIG 3.99 / CBAK 3.18 / COVL 3.42【1905.04874 表 2】 | **否**，生成器是 2 层 BLSTM | — | 加大 CGAN 权重会出现「难听伪影」；掩码下限设 0.05 以防音乐噪声 |
| MetricGAN+（2021） | PESQ 3.15、STOI 93.0【2104.03538 摘要；HF 模型卡】 | 未核实（沿用 BLSTM 的可能性大【推算】） | SpeechBrain，**Apache-2.0**【官方 HF】 | 未见 |
| MetricGAN-U | 只用含噪语音，以 DNSMOS 为目标；有一处二手来源给出 VB-D 上 PESQ 2.45【二手】 | 未核实 | 未核实 | 以 DNSMOS 为目标，天然容易被讨好（见 007） |
| MetricGAN-OKD（ICML 2023） | PESQ 3.24 / CSIG 4.23 / CBAK 3.07 / COVL 3.73；**因果版 PESQ 3.12**【二手 PwC，原表未核】 | 有因果版 | 未核实 | 每个指标配一个生成器、互相蒸馏，用来缓解多指标冲突 |
| CMGAN | PESQ 3.41、SSNR 11.10【2209.11112】 | 否（两级 Conformer） | GitHub 开源，许可未核 | **CHiME-7 UDASE 里它按 DNSMOS 训练，结果 DNSMOS 最好、听测最差**【2402.01413 §6.2.5，转引自 007 must-know 第 12 行】 |
| GSEPF（DPO，2507.09929） | GenSE*，DNS 无混响集：UTMOS 3.91→4.17，OVRL 3.41→3.44；有混响集只用 DPO 时 UTMOS 2.03→3.18（+56%）【表 1】 | 自回归 LM，无流式数字 | 未见代码 | **只用 DPO 时 SECS 0.691→0.651（说话人像不像变差）**，加上 CE 后回到 0.667【§4.1】 |
| FlowSE-GRPO（2601.16483） | 46.18M 参数 DiT + LoRA。DNS 无混响集 OVRL 3.373→3.549，真实录音 3.115→3.356；GRPO 5k 步，DPO 对照 20k 步【表 1–2】 | 未说明，有 vocoder | 未见代码 | 单一奖励涨分快，但损伤保真；多指标奖励（DNSMOS + 说话人相似度 + SpeechBERTScore）才缓解【§4.1】 |
| Berdoz 等（GSPO，2606.21458，Interspeech 2026） | 奖励是 DNSMOS + WER + UTMOS，具体数字未能核实 | 自回归 LM | 未见 | **只用 DNSMOS 会把感知质量降到 SFT 基线以下**；复合奖励在人评中第一【摘要】 |

### 2. GTCRN 及其他模型的训练

- **GTCRN 损失**：`resources/gtcrn/loss.py:16-30`。做法是幂律压缩（复谱 ^0.7、幅度 ^0.3），再加 SI-SNR：`30*(real+imag) + 70*mag + sisnr`（:30）。STFT 是 512 / 256 / sqrt-Hann（:24）。仓库里**没有训练脚本、数据配置和超参**；README :53 指向 SEtrain（MIT），SEtrain 的 `cfg_train.yaml` 没能读到。
- **GTCRN 的循环结构**：`gtcrn.py:81` 的 TRA 里有 GRU。`:163-164` 的 GRNN 由两路 GRU 组成。`:193` 的 intra 是**沿频率的双向 GRU**，不破坏时间因果。`:197` 的 inter 是时间上的单向 GRU。流式状态有三类：`stream/gtcrn_stream.py:146-147` 的 `conv_cache` / `tra_cache`，以及 `:213` 的 `inter_cache`。官方权重有 DNS3 和 VCTK 两版（`checkpoints/`）。
- **LiSenNet**：训练代码开源（MIT）。它是 MetricGAN 式的：`model/model.py:145-166` 在 CPU 上算 PESQ，把它当判别器的回归目标；`:139-141` 是生成器的对抗项。权重在 `config.yaml:48-51`：complex 0.1、mag 0.9、adv 0.05；lr 5e-4，100 个 epoch。
- **UL-UNAS**：README :30–31 写着训练「参考 SEtrain」，即没有自带训练脚本。
- **FastEnhancer**：训练代码完整（`train.py`、`losses.py`，MIT）。`losses.py:150-162` 是基于 `torch_pesq` 的可微 PESQ，但 `configs/fastenhancer_dns/m.yaml:98-99` 把它注释掉了。实际使用的是 mag 0.3 + complex 0.2 + consistency 0.3 + wav_l1 0.2，AdamP，lr 2e-3，batch 64，500 个 epoch，fp16，在线混合，SNR −5 到 20 dB（:66）。
- **单场景微调要多少数据和步数**：没有文献直接给出 48K 模型的数字，未能核实。参照 GSEPF 的 DPO（400 步、batch 128、单张 A40）和 FlowSE-GRPO（5k 步），**我估计**从 DNS3 权重出发，用 exp006 在线生成几十小时、几千到一两万步就能看出效果【推算】。
- **MPS**：GRU 输出 NaN 的问题仍未关闭【issue #94691】，所以 MPS 训练 GTCRN 不可靠，可以把 GRU 留在 CPU 上（慢）。MLX 的层齐全，但要自己移植并核对数值一致。exp006 的 `sim.py` / `train.py` 用的是什么设备，我没有去核。

### 3. 分场景专家 vs 通用模型 + 条件

- Sivaraman & Kim 2020（2005.08128）：门控网络按退化程度或性别把输入分给专家；微调后的专家集成在更少参数下超过通用模型，但摘要里看不出是否在同参数下比较。Chazan 2017（WASPAA）的数字未能核实。
- 用噪声嵌入做 FiLM 或条件 LayerNorm 的工作，都在说「对没见过的环境泛化更好」（1810.12757、2409.01545、2602.04307），**都没有和专用专家对比**。
- **我的判断【推算】**：GTCRN 的算力预算是固定的，换权重不增加每帧算力，但会多占内存，而且切换瞬间 GRU 状态不匹配（专家 B 拿到的是专家 A 的 `inter_cache`）。可行做法有两种：(a) 共用骨干和状态，只按场景换 FiLM 的 γ/β 或掩码头，状态连续；(b) 两路并行跑约 0.5 s 后交叉淡变，算力短时翻倍。我更倾向 (a)，它和 010 的「小适配器」路线一致。

### 4. 「吃字感知」损失

- **不对称损失**：VoiceFilter-Lite（2009.04323）用不对称损失让模型「不伤其他条件下的识别」。公式和 α 我没能从 PDF 中提取到，未能核实。据我记得，它对过抑制（增强谱低于干净谱）乘 α>1【二手 · 记忆】。
- **ASR 特征损失**：在 AVSR 场景里，有工作用冻结的 Whisper-medium 编码器当感知损失，但训练明显变慢（2601.12436）。另有工作用自监督编码器损失做无转录微调，WER 迁移到 Whisper 上仍有收益（2406.08914）。增强会让 Whisper 的 WER 变差，且模型越大越明显（2603.04710）。
- **能量 / 过抑制惩罚**：007 已经把「BAK 高 + 能量损失大 = 过抑制」定为制衡规则（must-know 第 33 行），可以直接写成损失项：只惩罚语音活动段里增强能量低于干净能量的部分【推算】。

## 对我们最值得借的 3 件事

1. **LiSenNet 的 MetricGAN 训练循环**（`model/model.py:130-166`，MIT）：把 GTCRN 换进去当生成器，判别器的回归目标从 PESQ 换成「SCOREQ + 吃字代理（语音段能量比 / ASR 删除）」的组合分。
2. **复合奖励 + 保留监督锚点**：GSEPF 的 CE+DPO 修回了说话人相似度（§4.1），FlowSE-GRPO 和 Berdoz 都证明单一指标会被讨好。微调时始终保留 `gtcrn/loss.py:30` 的监督项作锚点，奖励里至少放一个侵入式指标。
3. **FastEnhancer 的在线混合 + 可微 PESQ 开关**（`m.yaml:31-75`，`losses.py:150`）：这是从 exp006 接训练循环的现成模板，它也是我们备选的 FastEnhancer-M 自己的配方。

## 推翻 / 证实了哪条预判

- **「判别器奖励比 policy gradient 稳得多」：部分证实，但没有「得多」**。同一实验里 MetricGAN 只领先 PG 0.035 PESQ【1905.04874 表 1】，判别器自己也有近似误差导致的不稳【2002.05879】。
- **「监督 + 不对称损失 + 判别器学吃字代理」最现实：证实，需要补一条**。判别器的目标必须是多指标组合，并留一个不参与训练的裁判指标（D44）；否则就会重演 CMGAN 那种「按 DNSMOS 训、DNSMOS 最好、听测最差」。
- **013 must-know 里「MPS 上可行」那条：推翻**（GRU NaN）。替代方案是 MLX 移植、CPU 训练，或云 GPU（待人拍板）。

## 未能核实

- Koizumi 2017 / 2018 的具体 PESQ 数字。
- MetricGAN+ / MetricGAN-U 是否因果；MetricGAN-U 的 DNSMOS 数字；MetricGAN-OKD 原表；CMGAN 的代码许可。
- Berdoz 2026 的具体数字。
- VoiceFilter-Lite 不对称损失的公式、α 和 WER 表。
- SEtrain 的 GTCRN 超参；GTCRN 单场景微调的数据量和步数（只有推算）。
- Chazan 2017 的数字；同参数下专家与 FiLM 的对照。

来源：
- [MetricGAN 1905.04874](https://arxiv.org/html/1905.04874v1)
- [MetricGAN+ 2104.03538](https://arxiv.org/abs/2104.03538)
- [SpeechBrain MetricGAN+ 模型卡](https://huggingface.co/speechbrain/metricgan-plus-voicebank)
- [MetricGAN-OKD（PwC）](https://paperswithcode.com/paper/metricgan-okd-multi-metric-optimization-of)
- [MetricGAN-U 2110.05866](https://arxiv.org/pdf/2110.05866)
- [Koizumi 2018 1810.09137](https://arxiv.org/abs/1810.09137)
- [黑盒训练稳定性 2002.05879](https://arxiv.org/abs/2002.05879)
- [CMGAN 2209.11112](https://arxiv.org/abs/2209.11112v3)
- [GSEPF 2507.09929](https://arxiv.org/html/2507.09929v2)
- [FlowSE-GRPO 2601.16483](https://arxiv.org/html/2601.16483v1)
- [Berdoz 2606.21458](https://arxiv.org/abs/2606.21458)
- [专家混合 SE 2005.08128](https://arxiv.org/abs/2005.08128)
- [VoiceFilter-Lite 2009.04323](https://arxiv.org/abs/2009.04323)
- [2406.08914](https://arxiv.org/pdf/2406.08914)
- [2603.04710](https://arxiv.org/html/2603.04710v1)
- [2601.12436](https://arxiv.org/pdf/2601.12436)
- [PyTorch issue #94691](https://github.com/pytorch/pytorch/issues/94691)
- [MLX 层列表](https://ml-explore.github.io/mlx/build/html/python/nn/layers.html)
- [SEtrain](https://github.com/Xiaobin-Rong/SEtrain)
- [1810.12757](https://ar5iv.arxiv.org/html/1810.12757)
- [2409.01545](https://arxiv.org/pdf/2409.01545)
- [2602.04307](https://arxiv.org/pdf/2602.04307)