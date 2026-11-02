# 线 G · 把「更多人耳」接进评测的其他办法

> 调研日期 2026-10-31。只写本文件，没有任何 git 操作。
> 证据等级标签：【官方明载】【论文】【源码可证】【本机实测】【传言/未核】。查不到的写「未查到」。
> 本地路径前缀 `resources/` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`。
> 数据集卡片见 `docs/research/014_real-datasets/cards/`，评测标准见 `docs/research/012_eval-standards/`，这里不重复。

## 0. 一句话结论

**「众人耳朵」里最便宜、最近、又合 D39 的是两条**：
(1) 本机下载 **URGENT 2024 MOS 原始答卷**，在上面训或校成对偏好模型（URGENT-PK 路线），再用作者自己的 48 个配对做小样本校正；
(2) **本机现场请人听**（P.808/P.835 流程，在本机放页面），把答卷当成新的 verifier 校准数据。
听觉模型（HASQI/ViSQOL）和 ASR 可懂度当「人耳代理」都偏弱，只能做旁证。会议软件星级反馈根本拿不到音频级数据，只能借它的**方法**。

## 1. 总表（多接近真人耳 · 成本 · D39 可行性）

| # | 办法 | 多接近真人耳 | 成本 | D39 是否可行（不上云、不在网上众包） | 主要出处 |
|---|---|---|---|---|---|
| G1 | 在公开人评数据上做的成对排序模型（URGENT-PK）+ 作者配对校正 | 高（系统级）。直接学众人的 MOS 差值 | 中。权重在 HF 上，不需要 GPU 训练也能推理 | ✅ 本机推理；数据在仓外下载 | §2 |
| G2 | 听众条件化 MOS（LDNet / UTMOS22 的 listener embedding） | 中。能学出「某一类听众」，但公开权重都是 TTS 域 | 中高。要有带听众 ID 的增强语音数据 | ✅ 本机训练；⚠️ 增强域里缺带听众 ID 的数据 | §3 |
| G3 | 直接用别人发布的听测原始答卷当判据或校准集 | 高（这些就是真人答卷） | 低到中。下载 1.34 GB，超过 500 MB 上限，只记大小 | ✅ 下载不上传；⚠️ 超过 500 MB 要人拍板 | §4 |
| G4 | 本机现场请人听（P.808/P.835 流程放在本机） | 最高（真人、真设备、可控） | 人时成本高 | ✅ 明确允许 | §5 |
| G5 | 听觉模型（HASQI/HASPI/PEMO-Q/ViSQOL/响度掩蔽） | 低到中。要参考信号，在增强语音上与 PESQ 差不多 | 低 | ✅ 全本地 | §6 |
| G6 | ASR 可懂度（WAcc/WER）当代理 | 低（质量维）。只管可懂度 | 低（本机已有 whisper.cpp） | ✅ 全本地 | §7 |
| G7 | 会议软件的通话质量星级（Teams CQF 等） | 真人，但是整通电话层级，混着网络问题 | 我们拿不到数据 | ❌ 拿不到数据；产品自己收集要云端，与 D39 冲突 | §8 |
| G8 | 开源项目和厂商的用户 A/B 反馈与听测规模 | 低（几乎没有公开答卷） | — | — | §9 |

## 2. G1 · 成对偏好模型：先在众人数据上训，再用作者配对校正

**证据**
- URGENT-PK 输入一对「同源增强语音」，预测谁更好，用来做**系统级排序**【源码可证】`resources/URGENT-PK/README.md:1-7`。
- 训练时 `--delta` 只保留 MOS 差大于阈值的配对【源码可证】`resources/URGENT-PK/README.md:39-41`。这个做法和我们的「配对盲听」同构。
- 官方给了 3 个 checkpoint（mel+ResNet34 / UTMOS 冻结 / UTMOS 微调），都在 URGENT 2024 上训练，放在 HF `Twinkzzzzz/URGENT-PK`【源码可证】`resources/URGENT-PK/README.md:77-83`（文件行号 77–83 是表格位置）。大小未查到。
- 系统级相关【论文】arXiv 2506.23874 Table III（urgent25en，22 个系统 × 150 句，与 oracle MOS 比，**系统级**）：
  - DNSMOS：KRCC 0.602 / SRCC 0.713 / LCC 0.847
  - UTMOS：KRCC 0.835 / SRCC 0.951 / LCC 0.894
  - URGENT-PK_UTMOS(ft) NBS：KRCC 0.879 / SRCC 0.972 / LCC 0.959
- 域外泛化【论文】同文 Table II（**系统级**）：
  - urgent25zh：DNSMOS KRCC 0.472，PK_UTMOS(ft) NBS 0.723
  - CHiME-7 UDASE（真实录音，5 个系统 × 128 句）：DNSMOS KRCC 0.000 / LCC −0.140；最好的 PK_UTMOS(ft) 只有 KRCC 0.400 / LCC 0.489
  - **启示**：一到真实会议录音，所有人评代理都会塌。这与我们「合成格预测作者人耳只有 0.25–0.38」的现象一致（这是假设，没有 verifier 证明二者同源）。
- 训练集：论文 Table I 写的是 urgent24en，23 个系统 × 300 句（WebFetch 摘要里还给了「292 训练 / 8 验证」，单位存疑）【论文·待核原表】。

**做法（可验证）**
1. 本机推理官方 mel 版 checkpoint，在 exp003 的 48 个配对上算与作者选择的一致率。这样得到一个 verifier `pk_pair_agree`，带 bootstrap 95% 区间。
2. 用作者的配对做小样本校正：冻结编码器，只调最后的打分层，或做 Platt / 温度校准。必须**留一交叉验证**，否则 48 对会过拟合。
3. 判据：校正后的一致率要显著高于 DNSMOS 的 0.48–0.64；而且留出集上的区间下界要高于 0.5，才算「学到了作者」。

**成本 / 隐私**：只在本机推理，没有上传 ✅。checkpoint 大小未查到，下载前要先查，超过 500 MB 就只记大小。
**多接近真人耳**：系统级很高，句级没有报告（未查到）；真实会议域上很弱（UDASE）。

## 3. G2 · 听众条件化 MOS（学出「作者型听众」）

**证据**
- LDNet：输入语音和**听众 ID**，预测该听众的分数；另有一个 learned「mean listener」推理模式。作者是 Huang, Cooper, Yamagishi, Toda，ICASSP 2022【论文】arXiv 2110.09103（摘要），代码在 github.com/unilight/LDNet【官方明载】。
- 他引称 LDNet 在 VCC2018 上**系统级** SRCC > 0.98【论文·他引转述，未核原文】（来源：arXiv 2206.13817）。这是 TTS/VC 域，不是增强域。
- UTMOS22 本机源码就有这套机制：`LDConditioner` 用 `nn.Embedding(num_judges, judge_dim)` 给 SSL 特征加上听众嵌入【源码可证】`resources/UTMOS22/strong/model.py:117-135`。数据管线按 `listener_info` 拆出 `listener_name`，并合成一个 `MEAN_LISTENER_{domain}`【源码可证】`resources/UTMOS22/strong/dataset.py:59-83`。
- 个性化 MOS 的少样本适配：SpeechQoE（UT Arlington，SenSys'22）做逐用户 few-shot 微调，报告在 5 级 QoE 上准确率 91.4%【论文·未读原文，摘自检索摘要】ranger.uta.edu/~mingli/publications/Sensys22.pdf。场景是通话 QoE，不是降噪伪影。

**卡点**：增强语音的公开数据**带听众 ID 的未查到**。URGENT 2024 MOS 每句有 8 个原始评分，但页面**不给听众 ID**（见 §4），所以不能直接训 LDNet 式模型。
**可行变体**（假设，待测）：
- 把作者当成第 N+1 个「听众」，用「众人 mean listener + 作者偏置头」的结构。
- 作者的 48 对 + 未来 exp 的配对就是这个头的训练数据。
- verifier：在留出配对上的作者一致率，要比无条件模型高（有区间、过线）。
**D39**：全本地 ✅。**成本**：中高，要自己搭训练。

## 4. G3 · 利用别人发布的听测原始答卷

| 数据 | 内容 | 有没有个人答卷 | 大小 | 许可 | 证据 |
|---|---|---|---|---|---|
| URGENT 2024 MOS | 300 句盲测 × (noisy + 22 个增强) = 6,900 行，约 13.8 h | 每句 8 个 `raw_ratings`（1–5），**无听众 ID**；8 名 AMT 受试，P.808 | **1.34 GB → 超过 500 MB，只记大小，不下载** | CC BY-NC-SA 4.0 | 【官方明载】huggingface.co/datasets/urgent-challenge/urgent2024_mos；README 也指向它：`resources/urgent2024_analysis/README.md:192` |
| DNS Challenge P.835 结果 | 各队 SIG/BAK/OVRL 汇总 | 公开的只有汇总 PDF，原始答卷未查到 | — | — | 【源码可证】`resources/DNS-Challenge/V5_DNS_Challenge_FinalResults.pdf`；README 写明用 P.835 + WAcc：`resources/DNS-Challenge/README.md:9` |
| Blizzard / BVCC 等 | TTS 域 | 有听众 ID | — | — | 按任务要求，不相关的排除 |

**URGENT 2024 盲测上的「客观分 vs 众人 MOS」**【源码可证·图】`resources/urgent2024_analysis/mos/mos_correlations.png`（README 第 193–200 行引用）。README **没有明说是系统级还是句级**；图里画了「Overall ranking score」参照线，推断为系统级（推断，待核 arXiv 2506.01611）。

| 指标 | KRCC | SRCC | LCC |
|---|---|---|---|
| DNSMOS | 0.46 | 0.57 | 0.83 |
| NISQA | 0.64 | 0.71 | 0.87 |
| UTMOS | 0.75 | 0.91 | 0.94 |
| SCOREQ | 0.75 | 0.91 | 0.91 |
| PESQ | 0.57 | 0.77 | 0.72 |
| ESTOI | 0.61 | 0.80 | 0.63 |
| POLQA | 0.64 | 0.82 | 0.75 |
| WAcc | 0.50 | 0.71 | **0.30** |

**用法**（不下载全量也能做的部分）：
- 只取 parquet 的元数据列（mos、raw_ratings）。这样可以算**众人评分者之间的一致上限**（split-half），作为「一个人 vs 众人」的天花板参照：作者与众人的一致率达到这个上限，就说明作者不算离群。
- 能否只拉元数据列，没有核实（HF parquet 一般音频和元数据在同一个文件里）。
- 全量 1.34 GB 列为**待拍板**。

## 5. G4 · 本机现场请人听（最接近真人耳，D39 明确允许）

- P.808 众包流程（DNS/URGENT 都用它）【官方明载】`resources/DNS-Challenge/README.md:9`（P.835）、HF 页面（P.808）。
- 微软开源了 P.808 工具包（github.com/microsoft/P.808），设计给 AMT 用。它的 HIT 页面能不能离线在本机运行，**未核**【传言/未核】。
- Krisp 自述用 ITU-R BS.1284-2 主观测试，S/N/G-MOS，强调「diverse listeners」，**听众人数未写**【官方明载·无数字】krisp.ai/blog/noise-cancellation-quality-evaluation/。
- 有文章泛称「筛后约 15 人以上」才可靠，是行业经验，不是标准原文【传言/未核】forasoft.com 文章。

**做法**：
- 沿用 exp003 的盲听页（按 CLAUDE.md §五.1，build id、揭盲表进仓，给人的目录不带答案）。
- 改成「每位访客 ≤ 10 分钟、含 2 道陷阱题（同一音频配对）」，答卷导出 JSON 就放进 `docs/experiments/<exp>/results/`。
- verifier：听众间一致率（Fleiss κ 或 split-half）、陷阱题通过率、作者与众人多数票的一致率。
- 隐私：只放本机，不上传 ✅；答卷不写真名，用匿名 ID。

## 6. G5 · 听觉模型（HASQI / HASPI / PEMO-Q / ViSQOL / 响度掩蔽）

- HASQI 在助听器降噪处理过的语音上，「prediction performance comparable to PESQ」，摘要里**没有给相关数字**【论文】Kressner, Anderson, Rozell 2013, IEEE TASLP 21(2):407–415, doi 10.1109/tasl.2012.2217132。
- 相位失真研究里，NH/HI 两组听众中 PESQ 的 Pearson 都最高，HASQI 只在 noisy/reverb 条件下有竞争力【论文·检索摘要，未读全文】arXiv 2007.14986。
- HASPI w2 与 noise suppression 条件下的听众**关键词正确率**（可懂度，不是质量）Pearson 0.9773（NH+HI，IBM 降噪）【论文·检索摘要转述，未核原表】。
- ViSQOL：原论文在 NOIZEUS（含增强方法）上「comparable with PESQ」【论文·检索摘要】research.google/pubs/pub43990；VoIP 降噪研究称 ViSQOL/POLQA 在部分条件下低估质量【论文·检索摘要，未核】。
- 神经编解码研究：ViSQOL-Speech 在混合数据上 ρ = 0.593，在纯语音上 0.85【论文·检索摘要，未核数据集与层级】arXiv 2511.19734。
- 本机：versa 有 ViSQOL 封装【源码可证】`resources/versa/versa/utterance_metrics/visqol_score.py:6-16`；versa 里**没有 HASQI/HASPI**（grep 无结果）【本机实测】。
- HASQI/HASPI 在 pyclarity 有开源实现【传言/未核】。PEMO-Q 公开实现与增强语音上的相关数字：**未查到**。
- 增强语音上，HASQI/HASPI/ViSQOL 与人评的系统级或句级公开数字：**未查到可直接对照的**。

**判断**：都需要干净参考信号；真实会议录音没有参考，用不上。相关性大体与 PESQ 同档，而 URGENT 图上 PESQ 是 KRCC 0.57。**不能**当成「更多人耳」，只能放进 CI 层当伪影诊断旁证。

## 7. G6 · ASR 可懂度代理

- DNS 挑战用 WAcc = 1 − WER，与 P.835 一起排名；2022 年总分 = 0.5[WAcc + 0.25(OVRL − 1)]【论文】arXiv 2202.13288（ICASSP 2022 DNS）；脚本在 `resources/DNS-Challenge/WAcc/WAcc.py`【源码可证】。
- 在 URGENT 2024 盲测上，WAcc 与众人 MOS 的相关是 KRCC 0.50 / SRCC 0.71 / **LCC 0.30**，在图中 18 个指标里 LCC 最低【源码可证·图】`resources/urgent2024_analysis/mos/mos_correlations.png`。
- **判断**：管可懂度，不管好听程度；只能当「降噪别把字吃掉」的护栏 verifier，不能代表人耳偏好。本机有 whisper.cpp，零成本 ✅。

## 8. G7 · 会议软件端的通话质量反馈（间接人耳）

- Microsoft 的 CQF（Call Quality Feedback）在「a random subset of calls」结束后弹出，是对整通电话的单一 1–5 星评分。
- 用 40 项遥测（echo、noise、speech distortions 等）训 lightGBM 预测「差评（1–2 星）」，验证集 **AUC 74%**，原文称「good performance」，并承认聚合遥测抓不到通话中的变化【论文】arXiv 2304.00652 §3.3.2（Hosseinkashi, Cutler 等，Microsoft）。
- 同文的会议有效性问卷：过滤后 **15K ratings**（外部组织约 7,330 条有效）【论文】同文 §3.2。
- Teams「Rate My Call」：每 10 通或 10% 的通话弹一次；好通话常被跳过，数据偏负面【传言/未核】厂商文档（docs.uxmapp.com、martellotech.com）。
- Zoom / Google Meet 公开的「星级 vs 客观音频指标」研究：**未查到**。
- 微软专利描述了「用户主观 MOS + 遥测」训回归模型，以及按相似用户预测个人评分【传言/未核·专利】USPTO 10225313 等。

**判断**：
- 真人，但混着网络和设备问题；数据是厂商私有，我们拿不到。
- 在 vdn 里做同类机制，需要回传评分 → 违反 D39 ❌。
- 退一步的方案：**本地 opt-in 星级**，只存本机，用户手动导出 JSON 交回。这算「本机现场请人」的延伸，列为**待拍板**。
- 借鉴点：「随机抽样弹窗 + 只问一题 + 低分才追问」的问卷设计，以及「≥ 4 星为好」的二值化（arXiv 2304.00652 §3.1、§3.3.1）。

## 9. G8 · 开源项目和厂商的 A/B 反馈与听测规模

- RNNoise 演示页**没有**评分或 A/B 投票，只有「Donate Your Noise to Science」录一分钟噪声（累计 6.4 GB，可下载）和博客留言【官方明载】jmvalin.ca/demo/rnnoise/。
- Krisp：方法有，人数没有（见 §5）。
- Gaudio Lab 对 Krisp 做 MUSHRA，16 个条目中 7 个在 95% 水平上显著更好，听众人数未在摘要中给出【传言/未核·厂商博客】gaudiolab.com/blog/172。
- NVIDIA Broadcast / RTX Voice 白皮书的听测规模：**未查到**。
- **判断**：开源和厂商几乎不发布原始答卷，这条路基本是空的。

## 10. 待拍板（人定）

1. 下载 URGENT 2024 MOS 全量（1.34 GB，超过 500 MB 上限），还是只尝试拉元数据？默认：先只拉元数据。
2. URGENT-PK checkpoint 下载（大小未查到）。默认：先查大小，≤ 500 MB 就下。
3. 产品内加「本地 opt-in 星级 + 手动导出」？默认：不做，先用 G4 现场听。

## 11. 建议的 verifier 雏形（全部待测，现在都是假设）

| verifier | 层 | 输入 | 过线判据（草案） |
|---|---|---|---|
| `pk_pair_agree` | QA | exp003 的 48 对 + URGENT-PK | 与作者一致率的 95% 区间下界 > 0.5，并高于 DNSMOS |
| `pk_author_calibrated` | QA | 同上 + 留一校正 | 留出一致率比未校正的高，bootstrap p < 0.05 |
| `crowd_ceiling` | QA | URGENT 2024 raw_ratings | 给出 split-half 一致上限，作为作者一致率的参照 |
| `local_panel_kappa` | 人工 | 本机现场听测答卷 | 陷阱题通过率 ≥ 80%，κ 报告 |
| `wacc_guard` | CI | whisper.cpp | 增强后的 WAcc 不低于 noisy 原文 − x（x 待定） |
