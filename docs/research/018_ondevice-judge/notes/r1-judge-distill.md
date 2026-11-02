# r1 · 无参考评分模型的小型化 / 蒸馏（D60：SCOREQ 378 MB → 几 MB）

> 2026-11-02 · 018 判断回路线 r1。只做文献 / 源码调研：没下载权重，没跑推理。
> 证据等级：【官方】【论文】【源码】【二手】【推算】；只读到摘要的标「只读到摘要」。
> 已有结论只引用、不重复：010 `decision.md:20`（无参考分不当奖励）· 013 `decision.md:32`（DNSMOS 不进梯度）· r4（讨好与护栏）· ears exp009 `crowd_ears.md`（众人耳朵实测）。

## 一句话结论

**有现成先例：DistillMOS 把 478M 参数的 XLS-R 老师蒸成 4.3M 的卷积 Transformer，参数量约降到 1/111【已核改：原写「体积」，论文给的是参数量】，PCC 加权均值从 0.81 降到 0.76（掉 0.05）。本项目自己的 exp009 也量过：在 URGENT24 上，DistillMOS 语句 SRCC 0.757，SCOREQ 0.789，只差 0.03。**【已核改：补限定】但**系统级差得多**：URGENT24 系统 SRCC 0.772 对 0.926（差 0.15），CHiME-7 配对一致率 0.691 对 0.767；且 URGENT24 上 DistillMOS 只有 64,267 对 / 22 个系统，SCOREQ 是 70,381 对 / 23 个系统，两者不是同一批样本（`crowd_ears.md:16-17`）。 所以 D60 走得通，验收线可以照这个量级定：学生与人的 SRCC 比 SCOREQ 低不超过约 0.05，而且**必须不输 DistillMOS**，否则直接用 DistillMOS 更省事。体积目标 1–5 MB，算力 ≤50 MMAC/s，可以做到。最难的不是压体积，是**短窗**：老师在 4 s 片段上训练（推荐 10–15 s），我们每 0.5 s 要出一次分，所以学生只能是「滑动 2–4 s 窗、每 0.5 s 出一次」，没法用 0.5 s 窗直接判。

## 一、证据表

### A. 老师：SCOREQ

| 项 | 事实 | 出处 · 等级 |
|---|---|---|
| 骨干 | wav2vec 2.0 BASE；CNN 冻结，只微调 12 层 Transformer（768 维）；时间平均后接 ReLU + 全连接，投到 256 维嵌入 | arXiv 2410.06675 §3「Architecture」、App. G【论文】 |
| 参数量 | 论文没给。按 ONNX 378.0 MB、fp32 每参数 4 字节反推，约 9.4–9.5 千万（与 HuBERT/wav2vec2 Base 的 94.68M 同档，见 D 节） | Zenodo 15739280 文件表【官方】+【推算】 |
| 无参考模式 | NR 分两步：先用对比回归三元组损失（SCOREQ loss）训编码器 → 冻住，再用 L2 训线性 MOS 头。另有 NMR 模式：拿任意干净语音当「非匹配参考」，算嵌入距离 | 同上 §3「SCOREQ Training」【论文】 |
| 两个域 | natural（电话、VoIP、语音增强，训练集 NISQA TRAIN SIM，10,000 条）／synthetic（TTS、VC，训练集 VoiceMOS22） | github.com/alessandroragano/scoreq README【官方】；论文 Table 1、App. D【论文】 |
| 输入 | 16 kHz，其他采样率自动重采样；训练片段 ≤4 s，评测 ≤15 s，**建议截成 10–15 s** | 官方 README【官方】；论文 App. H【论文】 |
| 人评相关（PCC，语句级） | NR Adapt：NOIZEUS 0.91、TENCENT 0.86、LIVETALK 0.86、TCD-VoIP 0.85、TENCENT-Rev 0.79 | 论文 Table 3【论文】 |
| 在降噪输出上的表现（本项目实测） | 语句 SRCC / 系统 SRCC：URGENT24 0.789 / 0.926 · URGENT25 0.744 / 0.87 · CHiME-7 0.744 / 0.9；**TMHINT 只有 0.291，系统级 −0.714** | ears `exp009-more-ears/results/crowd_ears.md` A 表【源码 · 本项目实测】 |
| 权重与许可 | 4 个 ONNX 加 4 个 .pt，每个约 378 MB，**CC BY 4.0**；代码 MIT（本机副本 `resources/fastenhancer/utils/scoreq_onnx.py:2`） | Zenodo 15739280【官方】；`scoreq_onnx.py:71-76`【源码】 |
| 推理耗时 | 论文只给了训练步耗时（V100，4 条一批，0.75 s/步），**没给推理耗时、FLOPs 和参数量** | 论文 App. B【论文】 |

### B. 已有的小模型 / 蒸馏先例

| 模型 | 参数 · 体积 | 算力 | 输入 | 与人一致（标明系统级 / 段级） | 在 DNN 降噪输出上验证过吗 | 许可 | 出处 |
|---|---|---|---|---|---|---|---|
| **DistillMOS**（Stahl & Gamper，ICASSP 2025） | 学生 v7 **4.3M**（老师 478M，约 1/111）；pip 包约 15.6 MB【二手】 | 论文没给 | 16 kHz；实/虚部压缩谱（窗 320、跳 160）→ 卷积 Transformer | PCC 加权均值：老师 0.81 → 蒸馏 0.76 → 只用有标签数据训的同尺寸模型 0.72，**蒸馏补回约一半差距**；DNS3 0.82→0.79，SIG2024 0.70→0.60，Blizzard FR 0.58→0.22（跨域掉得最狠）。项目实测语句 SRCC：URGENT24 0.757、URGENT25 0.698、CHiME-7 0.70 | 是（DNS3、SIG 2024 测试集；PLC 2024 是丢包补偿不是降噪【已核改】；本项目 exp009）。官方 README：输入 7.68 s 片段，更长按段切【已核改：补】 | 代码 MIT；README 说主要在英语和增强场景上训练 | arXiv 2502.05356 Table I、§IV-B、§VI【论文】；github.com/microsoft/Distill-MOS【官方】；exp009【源码】 |
| DistillMOS 的配方 | 无标签数据：DNS4 约 2,600 h 干净语音 + 180 h 噪声，其中 15% 是 17 个 TTS 生成的；老师给软标签。**有标签数据混入 20%**（在 0/10/20/40/80% 里网格搜出来的）。250k 步，batch 20 | — | — | 剪枝对照：PruneMOS 剪到原来的 29%（139M）时加权 PCC 0.80；**小尺寸下蒸馏优于剪枝** | — | — | 同上 §III、§IV、§V-B【论文】 |
| DNSMOS P.835 | ONNX `sig_bak_ovr.onnx` 1,157,965 B；DistillMOS 表里记 0.17M（按文件大小反推约 0.29M，两者对不上） | 论文没给 | 9.01 s 窗、1 s 跳；对数功率谱（20 ms 窗、10 ms 跳）→ 纯 CNN | **模型级** PCC SIG 0.94 / BAK 0.98 / OVRL 0.98；**片段级** PCC 0.71 / 0.83 / 0.82，SRCC 0.72 / 0.82 / 0.81。项目实测 CHiME-7 语句 SRCC **0.232** | 是（17 个降噪器） | 文档 CC BY 4.0，代码 MIT | arXiv 2110.01763 Table 2–3【论文】；`resources/DNS-Challenge/DNSMOS/dnsmos_local.py:19-20,27`、`LICENSE`【源码】 |
| DNSMOS P.808 | `model_v8.onnx` 224,860 B；DistillMOS 表记 0.05M | — | 120 维 mel | — | 是 | 同上 | 同上；arXiv 2502.05356 Table I【论文】 |
| NISQA v2 | 0.25M；本机权重 tar 包 0.9–1.2 MB | — | mel 谱 → CNN + 自注意力 | 项目实测语句 SRCC：URGENT24 0.702、CHiME-7 0.424 | 是 | 代码 MIT，**权重 CC BY-NC-SA 4.0（不能商用）** | arXiv 2502.05356 Table I【论文】；`resources/NISQA/weights/LICENSE_model_weights:1`【源码】 |
| UTMOS22（strong） | 103M（wav2vec2 骨干） | — | 16 kHz 波形 | 项目实测：URGENT24 0.741 / 系统 0.902；CHiME-7 0.327 | 是（URGENT 官方指标） | MIT | arXiv 2502.05356 Table I【论文】；`resources/SpeechMOS/LICENSE`【源码】 |
| UTMOSv2 | 论文没给参数量；wav2vec2-base 加 EfficientNetV2-S（谱图当图像） | — | 波形 + 多分辨率谱 | VMC22-main 语句 PCC 0.916、系统 SRCC 0.932，**但带数据泄漏**（训练用到了 BVCC 测试集） | 否（合成语音） | MIT | `resources/UTMOSv2/utmosv2/config/fusion_stage3.py:114,123`【源码】；arXiv 2502.05139 表（Audiobox 论文里的对比）【论文】；2409.09305【论文，只读到摘要】 |
| TorchAudio-Squim（客观版） | 7.39M | **40.27 GMAC / 5 s ≈ 8 GMAC/s** | 波形 → 1-D 卷积 + 4 个 DPRNN | 只对 PESQ / STOI / SI-SDR 回归，PESQ SRCC 0.963；**主观 MOS 版没报参数和相关性** | 是（DNS2020 + 3 个 GCRN 输出） | BSD（torchaudio）【二手】 | arXiv 2304.01448 Table 1–2【论文】 |
| Audiobox Aesthetics | 论文没给参数量；WavLM 结构，12 层 768 维 | — | 16 kHz | PQ 轴：VMC22-main 语句 PCC 0.689、系统 SRCC 0.752（DNSMOS 是 0.612 / 0.773） | 否 | 未核 | arXiv 2502.05139 §3 和对比表【论文】 |
| Uni-VERSA | 论文没给参数量；WavLM-large 冻结，加 3 个 4 层 Transformer | — | 参考音频、文本都可选 | 语句级 MOS：无参考 LCC .77 / SRCC .78；**Fbank 前端比 WavLM 低约 0.05**（.72 vs .77，**仅限多任务联合训练**；只训 MOS 单任务时 Fbank .53/.55 对 WavLM .80/.80，差约 0.27，Table 4【已核改：补】）；跨域 TTS 降到 .60 | 部分 | 未核 | arXiv 2505.20741 Table 2–5【论文】 |
| AttentiveMOS | **约 86K**（D=16） | 论文没给；目标是内存 <1 MB、延迟 <10 ms | 2 ms 帧（1 ms 重叠）直接嵌入，固定 20.48 s | 【已核改】原写「NISQA 集 PCC：P501 0.666、FOR 0.558、LIVETALK 0.359，明显不如大模型」**在 v1、v2 全文都查不到**（0.666 是 Table 1 里 NISQA 数据集的评分者标准差 σavg）。实际只在 SOMOS（TTS）上评：SOMOS-clean 验证集 PCC 0.449 / SRCC 0.442，SSL-MOS 0.453 / 0.444，**两者相当**（Table 3）；绝对值都低 | 否（只测了 TTS） | 未核 | arXiv 2410.12675v2 §2–3、Table 1、Table 3【论文】 |
| DistilMOS（层级自蒸馏，2601.13700） | 目标是提高准确率，不是压体积 | — | — | — | — | — | 搜索结果摘要【二手】 |

### C. URGENT 用哪些无参考指标

URGENT 2025：DNSMOS、NISQA、UTMOS（`resources/urgent2025_challenge/evaluation_metrics/README.md:18-33`）【源码】。URGENT 2026 Track 1 又加了 SCOREQ（`evaluation_metrics/calculate_nonintrusive_scoreq.py`）和 DistillMOS（`inference.sh:47`）【源码】。

### D. SSL 骨干的通用蒸馏（如果学生要保留 SSL 前端，能压到哪）

| 模型 | 参数 | 速度 | 下游损失 | 出处 |
|---|---|---|---|---|
| DistilHuBERT | 23.49M（老师 94.68M 的 25%） | 4 颗 CPU 上 574 s，老师 992 s，**快 1.73 倍** | PR 5.41 → 16.27，ASR WER 6.42 → 13.37 | arXiv 2110.01900 Table 1–2【论文】 |
| FitHuBERT | 老师的 23.8% | 推理时间是老师的 35.9% | WER 12.1%、PER 13.3% | arXiv 2207.00555【论文，只读到摘要】 |
| LightHuBERT a_Small | 27M（老师 95M） | 没报 | PR 6.60，ASR 8.33（老师 5.41 / 6.42） | arXiv 2203.15610v2 Table 4【论文】 |

这条路最多只能压到约 23–27M（约 94–108 MB fp32【已核改：原写 90–100 MB】），**不到「几 MB」**。所以 D60 的学生不能走 SSL 路线，得像 DistillMOS 那样从谱特征直接训小网络。

## 二、对本项目的含义（全部是假设）

**D60 方案草案**

1. **学生结构，两个候选并行试：**
   - **S1「DNSMOS 尺寸」**：log-mel 64–120 维（10 ms 跳）→ 6 层深度可分离 CNN → 注意力池化 → 头。参数 0.2–0.5M，int8 后 <1 MB。
   - **S2「DistillMOS 尺寸」**：小卷积 Transformer / conformer（2–4 层，d=128），参数 1–4M，fp16 体积 2–8 MB。**直接拿 DistillMOS 的公开权重当 S2 的起点做微调**（MIT），这样最省事。
   - 多头输出：OVRL 加上 SIG/BAK 风格的分项，好与红线 verifier 对照。
   - **流式**：编码器做成因果卷积，带状态缓存；池化覆盖最近 2–4 s，每 0.5 s 出一次分。
2. **体积和算力目标：** ≤5 MB，**≤50 MMAC/s**，与 GTCRN 的 33 MMAC/s 同一档【推算】。对照一下老师：wav2vec2-base 约 9.5 千万参数，每秒 50 帧，光 Transformer 就约 4–5 GMAC/s，再加 CNN 前端，总共约 8 GMAC/s，与 Squim 实测的 8 GMAC/s 同量级【推算】。学生要比老师省约 160 倍。
3. **蒸馏数据：**
   - 合成降噪输出：GTCRN 和 ParamSet 网格扫出的各种 strength / floor / tone 组合，正好覆盖调节模型会去探的区域。
   - D63 训练份里的带噪语音和干净语音当原料；SCOREQ（natural、NR）打软标签。标签窗口与老师训练时一致，取 4 s，每 0.5 s 滑一次。
   - **混入约 20% 人评锚点**（照 DistillMOS 网格搜出的比例），用 URGENT24/25 带人评的降噪输出，**但必须按系统 / 源句与验收用的份互斥**。这跟 D63「测试份只测不训」有冲突风险，见第三节。
   - 同时蒸馏 SCOREQ 的 256 维嵌入（特征对齐），不只蒸馏标量分。
   - 刻意补「讨好样本」：过抑制、吃词首、人声被压低、音乐噪声。这些样本的标签**不用老师给**，而用红线 verifier（Qwen3-ASR 吃字、人声电平）的判定压低，防止学生继承老师的盲区（r4 A4/A5）。
4. **验收线（与 ears exp009 同一口径）：**
   - 与老师：同分布留出集上语句 SRCC ≥0.90。
   - 与人：URGENT24 语句 SRCC ≥0.74（SCOREQ 0.789 − 0.05），且 ≥ DistillMOS 的 0.757 − 0.01；URGENT25 ≥0.69；CHiME-7 ≥0.69；系统 SRCC ≥0.85。同源句配对一致率在 CHiME-7 上 ≥0.72（SCOREQ 0.767，DistillMOS 0.691）。
   - **任一条不过，就退回用 DistillMOS 原版当本机评分**，D60 改成「只对 DistillMOS 做量化」。
   - 学生只是判断回路里的旁证，**不能放行任何越红线的候选**，这点和 r4 一致。
5. **M1 能效核占空比【推算，未实测】：**
   - 假设：Icestorm 能效核约 2 GHz【二手】，小卷积在 NEON 上实际约 1–3 GFLOP/s。
   - 流式 S1：每 0.5 s 只算新进来的 0.5 s，约 25 MMAC，即 50 MFLOP，耗时 17–50 ms，**占一颗能效核 3–10%**。
   - 不做流式、每次重算 4 s 窗：再乘 8，占 25–80%，不可取。
   - S2 取 4M 参数、约 200 MMAC/s：流式时每次约 100 MMAC，占一颗能效核 13–40%。不够的话可以降到每 2 s 判一次（D37）。
   - 老师 SCOREQ 每 0.5 s 判一次要约 4 GMAC 以上，在能效核上要几秒，**根本不可能上本机**。这再次说明 D60 是必须做的。

## 三、没查到 / 存疑

- **DistillMOS 学生的 MACs 和实测耗时**：论文没给，10 个学生尺寸只画在 Fig. 2 里，没读到具体数。**v7 以外的尺寸对应的 PCC 也没拿到**，所以「1M 以下还能保住多少」没有直接证据。
- **公开的 MOS 预测器量化 / 端侧论文：没找到。** int8 会损失多少一致率要我们自己量。
- **SCOREQ 在短窗（<4 s）上的可靠性**：官方只说 ≤15 s；TMHINT 上 SCOREQ 系统级为负（exp009），可能是中文或短句的问题，未查。
- **许可链（要人拍板）**：SCOREQ 权重是 CC BY 4.0，但 natural 版的训练集 NISQA 语料本身的许可没核（NISQA 权重是 NC）。拿这个老师的软标签训出要随包发的学生，许可算不算传染，要人定。
- **D63 冲突（要人拍板）**：URGENT 人评输出拿来当锚点，用哪一份？目前设想「URGENT24 当锚、URGENT25 加 CHiME-7 当测」，但 URGENT 人评本身是不是「测试份」要回原数据集核对。
- DNSMOS P.835 的参数量，DistillMOS 记的 0.17M 和按文件大小反推的约 0.29M 对不上，没装 onnx 包，没核。
- Audiobox Aesthetics、Uni-VERSA 的权重许可没核。Squim 的许可写的是「BSD」，来源是二手。
