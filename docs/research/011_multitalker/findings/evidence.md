# 证据 · PSE / TSE 在干扰人数增加、低信噪比、远场混响下怎么掉

> 2026-10-30 · 调研子代理（只读：读文件 + 上网，没构建、没跑任何东西）。不重复 005_pse / 006_sota-landscape 已登记的条目。
> 标记：【官方】官方明载 · 【论文】 · 【源码】仓库可证 · 【推断】 · 【传言】

## 一句话结论

**每多一个干扰人、每加一层噪声或混响，PSE/TSE 都会明显掉分，而且「干扰人 + 噪声」叠在一起是最难的一格**（同一模型 SI-SDR 从约 13 dB 掉到约 7.5 dB）。2→3→4 人时输出 SI-SDR 依次约为 13.8 → 7.2 → 2.9 dB；真实会议里 3–4 人时，转写错误率比 2 人时高出 15–45 个百分点。**没查到任何论文把 6 人以上 babble 和单个干扰者放在同一张表里比**。现有证据只支持一个推断：babble 的人越多，越不容易「认错人」，但「能量被盖住」越严重。−5 dB、6 人 babble 这一格是能量遮蔽主导，声纹条件帮不上什么，要靠通用降噪那一层和保守增益。**已知名单之外，没找到同时满足「因果、≤35 ms、16 kHz、有代码+权重、GPL 兼容」的新候选。**

## 证据表

### A. 干扰人数 1→2→3→4（及更多）

| # | 结论 | 测试集 / 条件 | 数字 | 出处 | 标记 |
|---|---|---|---|---|---|
| A1 | 2 人 → 3 人时，所有 TSE/分离系统都掉 2–5 dB，ESTOI 掉 10–20 个百分点 | LibriMix 全重叠、min 模式、SIR ~N(0, 4.1 dB)、各自按人数单独训练、非因果 DPRNN / SpEx+ | SpEx+ ΔSI-SDR 14.4→11.7、ESTOI 85.4→70.3；SE-A 14.7→12.7；WSJ0Mix SpEx+ 16.1→12.2 | arXiv 2202.00733v2 表 II（第 6 页） | 【论文】 |
| A2 | 3 人时「先分离、再用声纹挑」更容易挑错人（均值被离群值拉低，中位数不变）；只有在 **3 人且 SIR 很低（−10 dB）** 时，带注册条件的 TSE 才比「分离 + 挑选」强 | 同上，SIR 从 −10 扫到 20 dB | 图 2、图 3；§IV-B「SV system sometimes selects an interferer」 | 2202.00733 §IV-B、§IV-C | 【论文】 |
| A3 | 只用 2 人数据训练时，测 3 人就崩了（4.2 dB）；加噪声直接成负数；混合条件训练能救回一半 | WSJ0-2mix / 3mix / WHAM 噪声混合 | TCN-Conformer：2mix 16.85 / 3mix 4.56 / noisy −0.24（表 1）→ 全条件训练 17.51 / 10.70 / 9.32（表 2） | arXiv 2205.13851 表 1–2 | 【论文】 |
| A4 | 只用 2 人训练、测 2/3/4 人：输出 SI-SDR 13.77 → 7.16 → 2.86；**提升量 SI-SDRi 掉得慢一些**（8.75 → 6.82 → 5.34），因为输入本身也更差了 | LibriSpeech test-clean、16 kHz、模拟房间（含距离线索）、非因果 TF-GridNet 系 NS-Extractor | 表 1；干扰人站在近处（「Intruded」）时，去掉声纹后 SI-SDRi 从 11.84 掉到 3.53（表 3） | arXiv 2306.16241 表 1、表 3 | 【论文】 |
| A5 | **真实会议**：人越多，转写错误率越高 | REAL-T PRIMARY（AMI / CHiME6 / DipCo / AISHELL-4 / AliMeeting），BSRNN（VoxCeleb1 训练，WeSep 实现，非因果） | TER%（英/中）：2 人 43.5 / 27.8 → 3 人 58.2 / 42.6 → 4 人 78.7 / 73.3（表 4） | Interspeech 2025 li25da（REAL-T）表 4 | 【论文】 |
| A6 | 模拟集上好看 ≠ 真实会议好用：Libri2Mix 上最强的 USEF-TFGridNet 到 REAL-T 上不如用 VoxCeleb 训练的 BSRNN | 同上 | Libri2Mix SI-SDR 18.05 dB，但 REAL-T TER 68.0 / 87.3%；BSRNN（VoxCeleb1）57.6 / 69.6%（表 5）；CHiME6 / DipCo（晚宴场景，多人 + 噪声 + 多房间）最差 | li25da 表 2、表 5 | 【论文】 |
| A7 | 注册片段的质量对结果影响很大：同一个人换不同注册片段，TER 在 44% 到 130% 之间 | REAL-T，样例 R003S04C01 | 图 4 + 正文 §3.3.3 | li25da §3.3.3 | 【论文】 |
| A8 | 助听用户听测：干扰从 1 人变成 2 人，未处理时更费力；**32 ms 算法延迟的因果掩码法（Algo-1）对听感几乎没有收益**，2.5 ms 的时域法（16.9 G MACs）有收益 | 德语 OLSA 句，1–2 个竞争说话人，**没有 babble、没有混响** | Algo-2 让 SRT 改善 1.2–3.2 dB；Algo-1 无改善（图 3–4、表 1） | PMC12340209（Sinha 等，CC BY-NC 文章） | 【论文】 |

### B. 噪声 + 干扰人叠加、低信噪比

| # | 结论 | 测试集 / 条件 | 数字 | 出处 | 标记 |
|---|---|---|---|---|---|
| B1 | **「2 人 + 噪声」远比「1 人 + 噪声」和「纯 2 人」难**：SI-SDR 几乎减半 | Libri2Mix 8 kHz、min 模式（WHAM 噪声） | SEF-PNet：1 人 + 噪声 14.50 / 纯 2 人 13.00 / 2 人 + 噪声 7.54（混合输入 3.27 / −0.03 / −2.03） | `resources/SEF-PNet/README.md:38-120`；TripleC arXiv 2512.04945 表 1（LGTSE 14.50 / 13.18 / 7.88） | 【源码】【论文】 |
| B2 | 前面先接 GTCRN 降噪，再做 TSE，「2 人 + 噪声」那格能涨一点 | WHAM! 8 kHz | SEF-PNet 8.28 → D-LGTSE（离线）8.78 dB SI-SDR | `resources/D-LGTSE/readme.md` 结果表；arXiv 2508.19583 | 【源码】 |
| B3 | 有干扰人时，**通用降噪反而让转写变差**（和 exp003 的「吃字」一致）；个性化模型能救回来，但「误杀主人」的比例上升 | 微软 ICR 会议录音，TS1 = 目标 + 1 个干扰人 + 噪声，SIR 0–10 dB | WER：未处理 36.36 → 通用 DCCRN 40.12 → pDCATTUNET 24.94–25.78；TSOS 通用 9.01% vs pDCCRN 14.51%（加非对称损失 + 多任务训练后降到 1.20–3.13%）（表 1） | arXiv 2110.09625 表 1 | 【论文】 |
| B4 | 「有语音干扰 + 混响」时，个性化的收益缩水一半以上 | LibriSpeech、RNN-T 识别，「speech noise」= 另外一批说话人，SNR 1–10 dB | 无处理 → VoiceFilter-Lite：加性语音干扰 77.9 → 23.5%；**混响下的语音干扰 79.3 → 53.7%**（表 1）；真实语音查询集：非语音噪声下 WER 反而上升 21.1 → 27.0%（表 2，L2 损失） | arXiv 2009.04323 表 1–2 | 【论文】 |
| B5 | 低 SIR 下所有系统都掉分；只有 SIR ≥ 10 dB 时，「分离 + 挑选」和 oracle 才相当 | 2202.00733 的 SIR 扫描 | 图 3 | 同上 §IV-C | 【论文】 |
| B6 | DNS 2023 主办方明说：有同时说话的干扰人 + 噪声时，SIG 很难提升；冠军相对噪声输入的总分只提升了 0.145 / 0.141 | DNS 2023 盲测集（头戴 / 免提） | 摘要 | arXiv 2303.11510 摘要 | 【官方】 |

### C. 远场 / 混响

| # | 结论 | 数字 | 出处 | 标记 |
|---|---|---|---|---|
| C1 | WHAM!（噪声）→ WHAMR!（噪声 + 混响），各家 TSE 掉 0.7–3.9 dB；时域模型掉得比时频域模型多；论文把原因归于认错人，以及干净注册语音和带混响的混合语音对不上 | SI-SDRi：SpEx+ 13.1 → 10.9；SpEx++ 14.3 → 11.7；Res-TFGridNet-v2 15.9 → 12.0；USEF-TFGridNet 17.6 → 16.1 | arXiv 2409.02615 表 X | 【论文】 |
| C2 | 因果、流式 TSE 在实测 RIR 下掉得很狠 | AFW（250 ms 块）：Libri2Mix 7.93 dB SI-SNRi；325 条实测 RIR（RWCP / AIR / REVERB）下只剩 3.3 dB；有 oracle 证据时 7.9 | arXiv 2609.20463 §4 | 【论文】 |
| C3 | 晚宴类真实场景（CHiME6 / DipCo，多房间、噪声大）在 REAL-T 里最差 | TER（PRIMARY）CHiME6 92.5%、DipCo 62.0%，AMI 50.3% | li25da 表 2 | 【论文】 |

### D. babble（很多人）比单个干扰者更容易还是更难？

| # | 结论 | 出处 | 标记 |
|---|---|---|---|
| D1 | **没查到** PSE/TSE 论文把「N = 6+ babble」和「1 个干扰者」放在同一张表里比。查过的都只到 2–4 人（A1–A5），或只有 1 个干扰者 + 非语音噪声（B3、B4、Personalized PercepNet） | 本次检索 | — |
| D2 | 人耳：辅音识别率随 babble 人数 N 是**非单调**的，N = 6 到 128 是一段宽阔的最低谷；也就是说 6 人左右的 babble 对人耳反而是最难的一档。换成按 babble 包络调制的噪声时，识别率随 N 平缓下降 | Simpson & Cooke 2005, JASA 118(5):2775–2778, doi 10.1121/1.2062650（摘要） | 【论文】 |
| D3 | 少数几个干扰者时，主要的失败方式是**认错人**（离群值；3 人时更多见），而声纹条件正好对付这一点（A2、A4「Intruded」、C1） | 2202.00733 §IV-B；2306.16241 表 3 | 【论文】 |
| D4 | 推断：6 人以上的 babble 里，单个人的音色已经认不出来，「认错人」的风险变小，但能量遮蔽变大（时频点被均匀占满）。−5 dB 时本地 SNR 低，掩码法能做的上限低，声纹条件的边际收益小于单个干扰者的情况。**实际表现取决于训练数据有没有覆盖 babble**：A3 的 2 人训练模型一遇到新条件就崩 | 由 A3、A4、D2、D3 推出 | 【推断】 |
| D5 | 推断：exp003 里「邻桌 6 人 babble −5 dB」和「旁边有人说话」是**两种不同的病**。前者是能量遮蔽，偏通用降噪那一侧：要靠训练分布里有 babble、增益保守。后者是认错人 / 漏掉干扰人，偏 PSE/TSE 那一侧：近处干扰人去掉声纹后 SI-SDRi 只剩 3.5 dB（A4） | 同上 | 【推断】 |

### E. 已知名单之外的新候选（门槛：因果 · 算法延迟 ≤ 35 ms · 16 kHz · 代码 + 权重 · GPL 兼容）

| 候选 | 关键事实 | 过门槛吗 | 出处 | 标记 |
|---|---|---|---|---|
| **nomo-pvad**（tuya） | **只做检测、不输出音频**：每 160 ms 块输出一个目标说话人概率；16 kHz；18.16M 参数（ERes2NetV2 主干微调）；代码 Apache-2.0，权重声明为 Apache-2.0 衍生作品；注册图示写「≥10 s」，<1 s 时效果变差；README 自述远场、同性别音色相近时会变差；**训练数据没写**；没有效果数字（指向外部技术报告） | 不能当降噪模型；**可以当场景层 / 门控信号**（判断回路 0.5 s 一次，160 ms 粒度够用） | github.com/tuya/nomo-pvad README | 【官方】 |
| APSR-P | MIT（代码 + 权重）；神经网络部分因果，中心化 STFT 带来 16 ms 前瞻；**8 kHz**；用 WSJ0-2mix / WHAM! 训练（WSJ0 是 LDC 数据，许可有问题） | ✗（8 kHz，训练数据许可有问题） | github.com/Xiang-Lin75/APSR-P README | 【官方】 |
| pDFNet3（joaorura） | DFN3 + FiLM + ECAPA，48 kHz，有 ONNX 权重；**PolyForm Noncommercial 1.0.0** | ✗（非商用） | github.com/joaorura/pdfnet3 README | 【官方】 |
| AFW · Anchored Fast Weights | 因果流式，但按 **250 ms 块**处理；分离器 2.9M 参数；CPU RTF 0.235；代码 ym2976/anchor-fast-weight 没写许可，也没说有权重 | ✗（延迟太大、许可不明） | arXiv 2609.20463 | 【论文】 |
| Personalized PercepNet（Amazon） | 10 ms 帧 + 30 ms 前瞻；PPN-512 有 8.5M 参数，在 i7-8565U 单核上占 4.7%；只测了 1 个干扰人 + DEMAND 噪声、SIR/SNR 3–15 dB（表 1–2） | ✗（没找到代码和权重）；可当算力参照 | arXiv 2106.04129 | 【论文】 |
| Logitech 实时说话人提取 app | 因果的 VoiceFilter 改版，40 ms 帧 / 16 ms 跳，总延迟约 50 ms，SI-SDRi 8.5 dB | ✗（闭源，延迟超标） | Interspeech 2022 ronssin22 | 【论文】 |
| tse-conv-tasnet-48k（HF penta2himajin） | 搜索摘要称：因果、48 kHz、1.45M 参数、ECAPA 192 维、有 ONNX | **未核**（HF 页面返回 401） | huggingface.co/penta2himajin/tse-conv-tasnet-48k | 【传言】 |
| Zachshotamartin/target-speaker-extraction | 非因果离线；没选许可；权重不公开 | ✗ | GitHub README | 【官方】 |
| CHiME-9 ECHI 参赛系统（低延迟 TSE：hu26 / sharma26 / jhou26） | 多麦（眼镜 / 助听器）为主；没核代码 | 未核 | isca-archive.org/chime_2026 | — |

## 对本项目的含义

- **延迟**：查到的「2/3/4 人」证据几乎全来自非因果模型（DPRNN、TF-GridNet、BSRNN）。因果 + ≤35 ms 的模型只会比这些数字更差（B3、B4、C2 是因果或流式的）【推断】。A8 显示，32 ms 掩码法在 1–2 人干扰下对听感几乎没有收益，这是对「GTCRN 这一档延迟 + 掩码」路线的警告【论文】。
- **算力**：Personalized PercepNet-512 在 x86 单核上占 4.7%，是「PSE 能塞进 1/5 核」的唯一公开参照（无代码）；nomo-pvad 18M 参数每 160 ms 跑一次，Apple Silicon 上的占用**未知**。
- **许可**：本次新找到的候选里，只有 nomo-pvad（Apache-2.0，训练数据未披露）和 APSR-P（MIT，但 8 kHz 且用 LDC 数据训练）的许可能过；pDFNet3 是 NC，排除。
- **落点**：
  - **M1（通用降噪）**：「邻桌 6 人 babble −5 dB」更像能量遮蔽，先在通用模型这一层解决：训练 / 选型时要有 babble 覆盖，在低 SNR 时增益要保守，免得吃字（D4、D5、B3 中通用 DCCRN 让 WER 变差）。
  - **M2（个性化 / 门控）**：「旁边有人说话」（1–2 个近处干扰人）才是 PSE/TSE 的主场（A4「Intruded」、B3）；人一多（≥3）、加上噪声、混响，收益就递减（A1、A5、B1、C1）。所以 PSE 不该被当成「babble 解药」去评估。
  - **场景层**：建议增加一个「干扰人数 / babble 密度」内部细场景：1–2 个近处说话人 → 开 PSE / 门控；多人 babble → 用通用降噪 + 保守增益，PSE 只当门控。nomo-pvad 这类 160 ms 检测器可以给 0.5 s 判断回路提供「主人在不在说」的信号（只改参数，不碰声音）。
  - **评测**：005 的 IRL 网格（单个干扰 vs 3–5 人 babble）应扩到 **1 / 2 / 3 / 6+ 人 × SNR {−5, 0, 5} × 有无混响**，否则答不了本题。

## gap（需另开实验或查不到）

1. **6 人以上 babble vs 单个干扰者，在同一个因果 PSE 上的对照**：文献里没有，需在本项目评测网格里实测（VoxLock / GTCRN + 门控）。
2. **babble 人数 N 的非单调曲线**对**模型**是否存在（D2 只有人耳数据）：需实测 N = 1, 2, 3, 6, 12。
3. nomo-pvad 在 babble、远场下的检测准确率，训练数据许可，Apple Silicon 上的耗时：README 没写，外部技术报告（tuya.com/model/doc/pvad）未读到。
4. tse-conv-tasnet-48k 的许可与真实性：HF 返回 401，未核。
5. CHiME-9 ECHI 低延迟 TSE 系统论文（hu26、sharma26、jhou26）有没有单通道结果、代码、许可：未读正文。
6. DNS4 / DNS5 个性化赛道按「有无干扰人」拆开的分项分数：未查到公开分项表。
7. arXiv 2609.25948（真实对话多说话人提取）正文里真实 vs 模拟的掉分数字：只读到摘要（STOI 0.55 → 0.60）。

## 来源清单

- arXiv 2202.00733v2 Elminshawi 等, New Insights on Target Speaker Extraction（表 II、§IV-B/C、图 2–3）
- arXiv 2205.13851 Speaker-conditioning single-channel TSE using Conformer（表 1–2）
- arXiv 2306.16241 NS-Extractor, Focus on the Sound around You（表 1–3）
- REAL-T, Interspeech 2025: https://www.isca-archive.org/interspeech_2025/li25da_interspeech.pdf（表 2、4、5，§3.3.3）
- PMC12340209 Evaluation of Speaker-Conditioned TSE for Hearing-Impaired Listeners: https://pmc.ncbi.nlm.nih.gov/articles/PMC12340209/
- arXiv 2110.09625 Eskimez 等, Personalized speech enhancement: new models and comprehensive evaluation（表 1）
- arXiv 2009.04323 VoiceFilter-Lite（表 1–2）
- arXiv 2303.11510 ICASSP 2023 DNS Challenge（摘要）
- arXiv 2409.02615 USEF-TSE（表 X、XI）
- arXiv 2609.20463 Anchored Fast Weights 流式 TSE（§4、表 1）
- arXiv 2512.04945 TripleC（表 1）· arXiv 2508.19583 D-LGTSE
- `resources/SEF-PNet/README.md:38-120` · `resources/D-LGTSE/readme.md`（结果表）
- Simpson & Cooke 2005, JASA 118(5):2775, doi 10.1121/1.2062650
- arXiv 2106.04129 Personalized PercepNet（表 1–2）
- Ronssin & Cernak, Interspeech 2022: https://www.isca-archive.org/interspeech_2022/ronssin22_interspeech.pdf
- https://github.com/tuya/nomo-pvad · https://github.com/Xiang-Lin75/APSR-P · https://github.com/joaorura/pdfnet3 · https://github.com/Zachshotamartin/target-speaker-extraction · https://huggingface.co/penta2himajin/tse-conv-tasnet-48k（401）
- https://www.isca-archive.org/chime_2026/index.html（ECHI 论文清单）
