# E-distill · 调研代理原文（以 E-distill-skeptic.md 核对为准）

> 2026-10-31 · Workflow wf_113bb80a-4ec · 只读调研，未运行程序。

## 结论

1. **【推翻一半】只有大老师需要云 GPU 这个预判不成立。** 大老师**训练**才需要云 GPU；**用老师给真实录音做伪标签**本机就能做。VoxLock 是 27M 参数的因果 TSE 模型（目标说话人提取），在一个 CPU 核上用 80 ms 块、int8 时占约 61% 核【源码 resources/voxlock/README.md:111,145】，L4 上比实时快 40 倍【同:145】。拿它当老师，几百小时录音在本机多核上一两天能跑完【推算】。本机最硬的瓶颈是**硬盘**：剩 177 GiB（`df`），DNS 数据解包要约 1 TB【源码 DNS-Challenge/README.md:92】，URGENT25 Track1 要 1.3 TB【源码 urgent2025_challenge/README.md:61】。
2. **【论文】蒸馏的收益集中在低信噪比和极小学生上。** 这正是咖啡馆 babble 场景。两步蒸馏（先只对齐老师的激活，再加监督）在 −5 dB 输入、学生压缩 63 倍时 SDR +1.1 / +0.9 dB。学生 62k 参数，是老师的 3.3%【论文 arXiv 2309.08144 摘要】，和 GTCRN 的 48k 同一档。
3. **【源码】同延迟档里，比 GTCRN 好的学生架构有现成的、MIT、同一套训练配置。** FastEnhancer 的 T / B 两档、LiSenNet（流式改版）、UL-UNAS 都在。其中 FastEnhancer 仓里自带一个**非因果 Huge 配置**（`configs/fastenhancer_dns/huge_noncausal.yaml:1`），可以直接当老师的结构。
4. **【二手+推算】训练成本整体很低。** 单张 L4 每小时 0.44–0.80 美元，单张 A100 80G 每小时 1.19–2.79 美元。三个阶段加起来约 **150–600 美元**，大头在训练老师。
5. **【官方】D26 下能用于训练并发布权重的数据。** 可以：LibriSpeech、LibriTTS、DNS5 的 LibriVox 和 AudioSet / Freesound 噪声、MUSAN、AISHELL-1、MLS / CommonVoice（CC0）。不行：WHAM!、EARS、WSJ，以及 FSD50K 里 NC 的那部分。DEMAND 是 CC BY-SA 3.0，**传染性对权重是否生效要人拍板**。

## 逐条

### 1. 蒸馏与 sim-to-real（模拟到真实录音的差距）

- **E3Net**（Thakker 2022，Interspeech）。学生比老师快 2–4 倍，质量「reasonable」。学生可以在**没有干净参考的真实噪声数据**上训练，目标用老师的输出；再加 ASR 损失，能改善 WER 和目标说话人过度抑制，质量不降【论文 arXiv 2204.00771 摘要】。老师是否非因果：未能核实。
- **CNNC-Distill**（Park 2024，IEEE SPL 31:1129）。明确是**非因果老师 → 实时学生**的跨结构蒸馏。强的非因果老师（MANNER-B）比实时老师蒸馏效果好，也比只蒸馏输出好【论文 DOI 10.1109/LSP.2024.3388956，数字未能核实】。
- **Nathoo 2023**：见结论第 2 条。
- **自训练 / RemixIT**。老师给域内混合语音出伪目标，学生在重混后的数据上训练，老师周期性用学生更新【论文 arXiv 2202.08862】。CHiME-7 UDASE（真实 CHiME-5 录音，单路）里，RemixIT 的 DNSMOS 数据如下【论文 arXiv 2312.08979 表】：

  | 指标 | 原始输入 | RemixIT |
  |---|---|---|
  | BAK（背景噪声分） | 2.92 | 3.62 |
  | SIG（语音质量分） | 3.48 | **3.28** |
  | OVR（总分） | 2.84 | 2.84（不变） |

  也就是说，**伪标签自训练压噪了，但伤了人声**，和我们「从不吃字」的方向直接冲突。用 DNSMOS 当判别器的 GAN 做到 OVR 3.51【论文 ISCA CHiME 2023 close23】。这说明「伪标签 + 不可微质量奖励」比只用伪标签强。
- **arXiv 2309.02432**：把真实录音放进训练集，在合成和真实测试集上都变好【二手，数字未核】。
- **VoxLock 本身就是 sim→real 的例子**：只在单张 L4 上微调。阶段是：模拟通话 4000 步 → 真实 AMI 串话 4000 步 → 长通话 800 步 → 房间与距离 400 步【源码 voxlock/README.md:166-174】。

### 2. 学生结构（≤45 ms）

| 结构 | 参数 | MAC/s | M5 单线程 RTF | VB PESQ | 算法延迟 / 许可 |
|---|---|---|---|---|---|
| GTCRN | 48.2K（论文 23.7K） | 33M | 0.026 | 2.87 | 32 ms / MIT |
| LiSenNet（流式改版） | 37K | 56M | 0.017 | 2.98 | 同档 / 官方无权重 |
| FastEnhancer-T | 22K | 60M | 0.0058 | 2.99 | 32 ms / MIT |
| FastEnhancer-B | 91K | 262M | 0.011 | 3.13 | 32 ms / MIT |
| FastEnhancer-M | 492K | **2.9G** | 0.039 | 3.24 | 32 ms，MAC 是 GTCRN 的 88 倍 |
| UL-UNAS | 171K | 35M | 未测 | — | 32 ms / MIT |
| DFN3 | 2.31M | 0.36G | — | — | 40 ms，端到端 46–50 ms ❌；权重许可不明 |

出处：
- GTCRN 参数与 MAC【源码 gtcrn/README.md:14,19】
- 表中 RTF 与 PESQ 全部取自 FastEnhancer 的同口径对照表【源码 fastenhancer/README.md:28-250】。该表只计网络、不含 STFT（项目 006 sources.md:20 的说明）
- UL-UNAS【论文 arXiv 2503.00340 表 VI】
- DFN3【002 sources.md:44】
- 延迟【002 decision.md 延迟表】

**DFN3 小配置的数字：未能核实。**

### 3. 训练算力

- **GTCRN 官方 GPU 小时数：未能核实。** 论文海报只给了每个 epoch 用 4 万对 8 秒音频（约 89 h）【二手】；SEtrain 的 README 写了 4 卡示例，没给时长。
- **FastEnhancer DNS 配置**：batch 64，500 epoch，每 epoch 16384 段 × 2 s（约 9.1 h），fp16【源码 configs/fastenhancer_dns/b.yaml:40-41,78-83】。合计**过约 4550 h 音频**【推算】。Huge 非因果用 4 卡【源码 docs/docs/train/fastenhancer.md:29】。GPU 型号与耗时未能核实。
- **算量**：GTCRN 训练每秒音频约 0.1 GMAC（前向 33M × 约 3）；4550 h 共约 3 PFLOP【推算】。小 RNN 的瓶颈在逐帧串行和内核启动开销，不在 FLOPs。另外 MPS 上 GRU / LSTM 有过反向正确性问题和退回逐步执行的问题【二手 pytorch issue #83144；PyTorch commit】。
- **M 系列 MPS / MLX 训练这种小模型的公开吞吐基准：未找到**（未能核实）。
- **价格**（2026-10-30 查）：

  | 来源 | GPU | 每小时（美元） | 级别 |
  |---|---|---|---|
  | RunPod 社区 / 安全云 | L4 | 0.44 / 0.59 | 【官方 runpod.io/pricing】 |
  | 同上 | A100 80G | 1.19–1.79 | 同上 |
  | 同上 | H100 SXM | 2.69 / 3.99 | 同上 |
  | AWS g6.xlarge | L4 | 0.805 | 【二手 instances.vantage.sh，2026-10-31 更新】 |
  | AWS g5.xlarge | A10G | 1.006 | 同上 |
  | Lambda | A100 80G | 2.79 | 【二手聚合站；官方页没打开】 |
  | Lambda | H100 SXM | 3.99–4.29 | 同上 |
  | Vast | — | 价格表没抓到 | 未能核实 |

- **三阶段估算**【推算】：

  | 阶段 | 内容 | 在哪跑 | 耗时 | 成本 |
  |---|---|---|---|---|
  | 策略搜索（bandit） | 7 场景 × 200 配置 × 100 段 × 10 s ≈ 390 h 音频；GTCRN 推理约 10 CPU·h，Whisper / DNSMOS 打分约 10–20 h | 本机 | 1–2 天 | 0 |
  | 单场景微调 | 每场景 50–200 h 音频，在预训练 GTCRN 上做 MetricGAN 类微调，瓶颈是每步调用奖励指标 | 本机 | 每场景数小时到 1 天 | 上云每场景 < 10 美元 |
  | 蒸馏 | 训非因果老师 4×A100 × 2–3 天 ≈ 200–300 GPU·h | 云 | — | 240–840 美元 |
  | | 伪标签 | 本机 | — | 0 |
  | | 学生按 DNS 规模（约 4500 h）训练 | L4 | 1–2 天 | 20–100 美元 |
  | | 同上 | 本机 | 3–10 天（未核） | 0 |

### 4. 数据许可与存储

| 数据集 | 许可 | 出处 |
|---|---|---|
| LibriSpeech、MUSAN | CC BY 4.0 | 【官方 openslr 12/17】 |
| AISHELL-1 | Apache-2.0 | 【官方 openslr 33】 |
| DNS（LibriVox 公有领域；AudioSet CC BY 4.0；Freesound 只选 CC0） | 可用 | 【源码 DNS-Challenge/README.md:271-285】 |
| DNS 里的 VoxCeleb2 | 「版权仍属原作者」，风险 | 同上 |
| DNS 里的 PTDB / CREMA-D | ODbL / DbCL | 同上 |
| VCTK | ODC-By（DNS 与 URGENT 的说法）；CC BY 4.0（二手） | — |
| DEMAND | CC BY-SA 3.0 | 【源码 同上:285】 |
| URGENT25 的 CC0 / CC BY 子集 | 可用 | 【官方 urgent-challenge.github.io/urgent2025/data】 |
| URGENT25 的 WHAM!、EARS、FSD50K-NC 子集 | NC，不行 | 同上 |
| URGENT25 的 WSJ | LDC 协议，不行 | 同上 |
| URGENT26 Track1 | 去掉了 WSJ；ESD 子集因许可被排除 | 【源码 urgent2026_challenge_track1/readme.md:33,69】 |
| AMI | CC BY 4.0 | 【二手】 |

**本机 `~/vdn-data`**（只 ls，没读音频）：

| 目录 | 大小 | 内容 |
|---|---|---|
| demand | 2.2G | DKITCHEN、OOFFICE、PCAFETER、PRESTO、SCAFE（48k）、STRAFFIC、TBUS |
| librispeech | 687M | 只有 test-clean |
| models | 175M | fastenhancer-dns、voxlock、wespeaker |
| music | 3.5M | — |
| zh | 30M | — |

本机**没有可训练规模的干净语音**。DNS5 光语音就要 225 GB【源码 urgent2025_challenge/README.md:68】，超出剩余空间，需要外置盘或 NAS（铁律 8）。

## 对我们最值得借的 3 件事

1. **两步蒸馏**：先只做激活对齐，再上监督。专门对低 SNR 和约 60k 参数的学生有效【arXiv 2309.08144】。
2. **伪标签 + 不可微质量奖励联用**，并且必须守住 SIG（语音质量）。纯 RemixIT 在真实录音上 SIG 从 3.48 降到 3.28【arXiv 2312.08979】。奖励里要放吃字率 / SIG 的底线（对应 D43 的「评测当奖励」）。
3. **分阶段从模拟到真实的微调课程**（模拟 → 真实串话 → 长段 → 距离与房间），只用单张 L4。VoxLock 本身（Apache-2.0、因果、27M）可以当 babble 场景的老师【voxlock/README.md:166-174】。

## 推翻 / 证实了哪条预判

- **部分证实**：GTCRN 量级的学生、单场景微调、bandit 策略搜索都能在本机做【推算】。
- **推翻**：
  - 云 GPU 主要花在**训练非因果大老师**上，不在伪标签上：伪标签用 VoxLock 或已有老师在本机 CPU 就能跑。
  - 「全部在本机」实际先卡在 **177 GiB 剩余硬盘**和 **MPS 上的 RNN 支持**，不卡在算力。
  - MPS 未必比 CPU 快，要先做一次有界冒烟实验（人拍板后）。

## 未能核实

- GTCRN、UL-UNAS、FastEnhancer 官方训练用的 GPU 型号与小时数
- M 系列 MPS / MLX 训练小模型的公开吞吐量
- Vast 实时价格；Lambda 官方价格页（页面没打开）
- CNNC-Distill、E3Net 蒸馏的具体 PESQ 数字；Thakker 2022 的老师是否非因果
- DFN3 小配置的参数与 MAC
- DEMAND 的 BY-SA 传染性对权重是否生效（要人拍板）
- VCTK 的许可是 ODC-By 还是 CC BY 4.0