# H · 补充来源：「吃字」听写、挑战赛逐条分、成对偏好

> 调研线 017 · 2026-10-31 · 只查不下大文件（本机只取了 < 1 MB 的标签包与 20 MB 的 demo，放在会话草稿目录，**未入仓**）。
> 不重复：URGENT 2024–2026、CHiME-7 UDASE、TMHINT-QI、VMC24 track3、DNS 队伍级结果已在 `B-enhanced-datasets.md`；预测器在 `C-predictors.md`。
> 证据等级：【官方明载】【论文】【源码可证】（含【本机实测】）【传言/未核】。

## 0. 结论先行

1. **能接进来测「吃字」的真人逐句听写，公开的只有 Clarity CPC1/2/3 一家**：逐句、逐听众、带原始听写文本（`response`/`transcript`）、正确词数（`hits`/`words_correct`）、系统 ID，**许可 CC BY-SA 4.0**（语音另有 Clarity 语音库许可：只限制「在设备 / 广播上回放」），Zenodo 免登录直下。标签包很小（46 KB–516 KB），本机已核过字段【本机实测】。
2. **但听众全是听障者，处理系统是助听器算法（带按听力图做的放大）**，能不能外推到正常听力的会议用户：文献指向**不能直接外推**（§1.4）。只能当「吃字 verifier 的管道冒烟 + 量级参考」，不能当正常听力用户的判据。
3. DNS 5 的 WAcc **不是众包听写**，是 ASR 的 1−WER；人工只参与转写参考文本【论文】。逐条未公开。
4. SSI 2023/2024、AEC 挑战：**逐条 P.804 / P.835 分未查到公开**；只有音频、转写、SigMOS / AECMOS 模型。
5. `vmc2026-track1-meta` 里的 `speecheval/test`、`speechjudge/test`：**全是 TTS / VC 合成语音，没有降噪或带噪配对**，而且这两个 test 集**没有人评分文件**（只有机器分）【本机实测】。
6. 其他（Hurricane 2.0、ESTOI 原始听测、ODAQ、SpeechJudge-Data、SpeechEval）：要么不是降噪，要么要注册 / 签条款，要么逐条未公开。

## 1. Clarity Prediction Challenge（CPC1 / CPC2 / CPC3）

### 1.1 总表

| 项 | CPC1（2022）| CPC2（2023–24）| CPC3（2025）| 出处 |
|---|---|---|---|---|
| 听众 | 27 名听障者（1 轻度 · 9 中度 · 17 重度，2–8 kHz 平均 35–76 dB HL）| 听障者；数量未查到（标签里 test 每子集 5 人）| 听障者；test 标签里 dev 8 人、eval 16 人；分级 Mild / Moderate / Moderately severe | CPC1：Barker 等 Interspeech 2022 §2.3【论文】；CPC2/3：标签文件【本机实测】|
| 条数 | 共 7,233 条：Track 1 训练 4,812 / 测试 2,421；Track 2 训练 3,545 / 测试 432（新听众）+ 249（新系统）| 公开 test 标签 305 + 294 + 298 = 897；训练集条数在大包里，二手说法 2,779 / 8,599 两种，**未核** | 公开 dev 926 + eval 7,674；训练集条数未查到 | CPC1 §2.3【论文】；其余【本机实测】|
| 维度 | 逐句「正确词百分比」：听众**复述**所听句子 → Google STT 转写 → 人工校对 → 与原文对齐数正确词 | 同 | 同（字段 `words_correct`、`correctness`，`#` = 没听清）| CPC1 §2.3【论文】；CPC3 数据页【官方明载】|
| 逐条 / 逐听众 | **是**：每行 = 一个听众 × 一句 × 一个系统，含 `listener`、`system`、`prompt`、`response`、`hits`、`n_words`、`volume` | 是（同 CPC1 字段，加 `pid`）| 是（加 `hearing_loss`、`transcript`）| 【本机实测】|
| 处理系统 | CEC1 的 10 个助听器算法：单通道分离、多通道波束、放大；**须因果、算法延迟 ≤ 5 ms**；E001 是基线，均值 36.5% · 最好 91.4% | CEC1 + CEC2 参赛系统；test 每子集 3 个 | 训练来自 CEC1/2；dev/eval 来自 CEC3（含真实背景与实测助听器信号）；eval 9 个系统，均值两群：E001–E004 ≈ 79–92%，E005–E009 ≈ 32–41%（为什么两群：**未核**，可能对应真实录音场景）| CPC1 §2.2【论文】；均值【本机实测】|
| 场景 | 仿真房间 + OlHead HRTF；单个干扰源；better-ear SNR −6 ~ +6 dB | CEC2：多个干扰源 | 动态场景、真实背景 | CPC1 §2.1【论文】；CPC2/3 数据页【官方明载】|
| 听测方式 | 自家安静房间，Chromebook + Sennheiser PC-8 耳机（Listen@Home），按系统分块 | 可在开头自调音量 | 未查到 | CPC1 §2.3【论文】；CPC2 数据页【官方明载】|
| 语言 | 英式英语句子 | 同 | 同 | 【官方明载】|
| 许可 | CC BY-SA 4.0；语音目标另走「Clarity speech database license 1.00」，例外条款 2(a)(2)：**不得在设备上或广播中回放**语音及其处理版，非商业研究里少量示例除外 | 同 | 同；Zenodo 记录标 cc-by-4.0，包内 README 写 CC BY-SA 4.0，**以包内为准（更严）** | 标签包内 `README.md`、`Clarity_speech_database_license_1.00.txt:101-106`【本机实测】|
| 下载 | Zenodo 17046604，免登录 | Zenodo 17045970，免登录 | Zenodo 17039000，免登录 | Zenodo API【本机实测】|

### 1.2 文件大小（Zenodo API 实测）

| 记录 | 标签包（已取，本机核过）| 小音频包 | 大包 |
|---|---|---|---|
| CPC1 17046604 | `labels.v1_0.tar.gz` 129 KB（test 2,421 + indep 632 条）sha256 `63580ece…76f3` | — | test 6.49 GB · 全量 14.19 GB |
| CPC2 17045970 | `labels.v1_0.tar.gz` 46 KB（3 个 test 子集共 897 条）sha256 `3de29215…19c0` | **test 501.6 MB**（略超 500 MB 线）| 全量 12.10 GB |
| CPC3 17039000 | `labels.tar.gz` 516 KB（dev 926 + eval 7,674）sha256 `fe040336…ba7` | **demo 20.2 MB（已取：30 条带音频 + 听写 + 27 名听众分级）**；dev 752 MB | eval 6.12 GB · train 7.51 GB |

**标签单独没用**：要拿来校 verifier，必须有对应的助听器输出音频。最小可用组合：
- CPC3 demo 20 MB：30 条 → **只够管道冒烟**（跑通「Whisper WER / STOI / 我们的吃字 verifier vs 真人正确词%」），不够出相关数。
- CPC2 test 501.6 MB + 46 KB 标签：897 条、15 名听众、9 个系统 → 第一份能出数的；过线 1.6 MB，**按约束要人拍板**。
- CPC3 dev 752 MB + 516 KB 标签：926 条、8 名听众、4 个系统，含真实场景 → 次选。

### 1.3 对我们有用的结构（本机实测）

- **系统间差距大**：CPC1 test 系统均值 18.1%–91.4%，CPC3 eval 31.8%–91.6%。适合检验「吃字 verifier 能否排对系统」，但比我们三个降噪模型之间的差距大得多（假设：我们的差距在几个百分点内）。
- **同一句、不同系统的配对**：CPC1 test 12,949 对（全是不同听众）；CPC3 eval 24,314 对不同听众 + **849 对同一听众**。同一听众配对正好对应「单人耳朵」的情形，可量单人配对的噪声（与 `E-single-listener.md` 对照）。
- **天花板 / 地板**：CPC1 test 48% 的句子满分、12% 零分；CPC3 eval 33% 满分、24% 零分 → 分布两头堆，做相关要用配对或系统级，别直接用句级 PCC。

### 1.4 能不能外推到正常听力的会议用户 —— 不能直接外推

- **听众不同**：CPC 的 SNR 范围（−6 ~ +6 dB）是按「不戴助听器的听障者」预试定的（CPC1 §2.1【论文】）。正常听力的人在这个范围里大多接近满分，吃字的位置不一样。
- **处理系统不同**：CEC 系统包含按每个听众听力图做的放大（CPC1 §2.2：「signal amplification」【论文】）。正常听力的人听到的是带高频补偿的信号，与会议降噪输出不是同一类东西。
- **降噪的收益方向因听力而异**：
  - Healy 等的一系列 JASA 研究（2015–2021）：DNN 掩蔽对听障者的可懂度提升很大（最不利 SNR 下平均 +56~59 个百分点），对正常听力者的提升小，且依噪声类型而定（2015 年那篇：babble 有提升、cafeteria 没有）——【论文，只读到二手摘要 labs.sonicfield.org/library/…，未核原文】。
  - Hu & Loizou 2007（JASA 122:1777）与 Loizou & Kim 2011（IEEE TASLP 19:47）：对正常听力者，传统单通道降噪**基本不提高可懂度**，有的还降低——【论文，凭已有知识，本轮未核原文与页码】。
  - 因此「对听障者无吃字」不能推出「对正常听力者无吃字」，反过来也一样。
- **可以借的**：① 字段格式与打分流程（复述 → ASR 初转 → 人工校对 → 数正确词），我们自己做小规模真人听写时可以照抄；② 用 CPC 校验「ASR 类吃字 verifier 能不能排对系统」，作为**管道正确性**的证据，而不是阈值的证据。
- **仍缺**：正常听力听众 × 现代因果降噪 × 逐句听写的公开数据。**未查到**。

## 2. DNS Challenge 5（ICASSP 2023）的 WAcc

- **不是众包听写**。arXiv 2303.11510 §4.3：「We computed word accuracy (WAcc) from a state-of-the-art speech recognition system」，WAcc = 1 − WER【论文】。ICASSP 2022 DNS 4 也一样：「Word Accuracy (Wacc) from a state-of-the-art speech recognition system」（`resources/DNS-Challenge/docs/ICASSP_2022_4th_Deep_Noise_Suppression_Challenge.pdf`，pdftotext 第 38 行附近）【源码可证】。
- 人工只用来做**盲测集参考转写**：五步——提示文本 → ASR → 专家人工转写（反复听）→ 算 WER → WER > 0.5 的再人工复核（§4.3）【论文】。
- 本机工具：`resources/DNS-Challenge/WAcc/WAcc.py:13` 把音频 POST 到一个 Azure 服务，读 `DNSChallenge4_devtest.tsv`【源码可证】。这个服务**现在能不能用：未核**；按项目约束也不该把音频外发。
- 逐条 WAcc：**未查到公开**。只有队伍级（`resources/DNS-Challenge/V5_DNS_Challenge_FinalResults.pdf`，B 笔记已记）。盲测集参考转写：README 第 66–68 行写「Blind testset: <TBD>」【源码可证】。
- 对我们的用途：**不是人耳证据**。它说明「降噪过度会吃字」被官方当作排名项（DNS 4 论文第 83–84 行：过去的模型「noticeable Wacc degradation resulting from over suppression」【源码可证】），支持我们单设吃字维度；ASR 版的做法我们本机用 Whisper 就能复刻。

## 3. ICASSP 2023 / 2024 SSI 挑战与 AEC 挑战

| 项 | 结论 | 出处 |
|---|---|---|
| SSI 2023 测试集 | 500 条真实发送端录音，`ICASSP2023/test_data` 1,065 个 wav（含 blind）直接在 GitHub；**没有任何评分文件** | GitHub API 列目录【本机实测】|
| SSI 2023 评分 | 众包 P.804（SIG、Noisiness、Coloration、Discontinuity、Loudness、OVRL）；论文只给模型级结果和相关（例如 P.804 OVRL vs DNSMOS OVRL 0.695 / 0.884）；还有一节 WER 用人工转写算 | arXiv 2303.06566（pdftotext 第 426–450、494–499 行）【论文】|
| SSI 逐条 P.804 | **未查到公开**。SigMOS 的 README 说它是用 P.804 标注数据训练的，但数据没放出 | github.com/microsoft/SIG-Challenge `ICASSP2024/sigmos`【官方明载 · 经 WebFetch 摘要】|
| SSI 2024 | 仓库多了 `transcripts_ICASSP2023_test_data`、`loudness`（只有 1 条示例）、`global_test_samples`；**仍无逐条评分** | GitHub API【本机实测】|
| SSI 训练数据里的 P.835 | 挑战页说 AEC 数据里「near end single talk」片段有 P.835 评分并提供 → 在 AEC 仓库 `datasets/` 下**没找到评分文件**（只有命名规则 README）；B 笔记同样标为传言 | github.com/microsoft/AEC-Challenge `datasets/README.md`【本机实测】；评分是否存在：【传言/未核】|
| AEC 挑战逐条分 | **未查到公开**；只有 AECMOS 模型（`AECMOS/AECMOS_local/*.onnx`，1–3 MB，MIT）| GitHub tree【本机实测】|
| 用途 | SSI test_data 是**真实会议设备录音**，免登录、单条 0.4–1.2 MB，可当我们的真实输入；但没有人评分，测不了「众人耳」| — |

## 4. VoiceMOS 2026：`vmc2026-track1-meta` 的 speecheval / speechjudge

> 注意：HF 上叫 `urgent-challenge/vmc2026-track1-meta`；`urgent-challenge/vmc2026-meta` 返回鉴权错误（不存在或私有）【本机实测】。

| 子集 | 条数 | 内容 | 有人评分吗 | 有降噪 / 带噪配对吗 |
|---|---|---|---|---|
| `speecheval/test` | 20,592 | 系统前缀：zh_Aishell3 4,906 · ja_JSUT 3,948 · fra_SIWIS 3,565 · en_VCC2018 2,800 · zh_CSMSC 1,004 · en_GT 497 · CosyVoice / ChatTTS / F5TTS / Bark / AnyGPT / SpeechGPT / ElevenLabs / 火山 / 微软 / 阿里 …（TTS 与 VC）| **没有**：目录里只有 `distill_mos / dnsmos_ovrl / nisqa_mos / scoreq / utmos.scp` + `utt2dur / utt2reference / utt2system`，没有 `mos_overall.scp` 和 `meta.jsonl` | **没有**：`denois|enhanc|noise` 匹配 0 条 |
| `speechjudge/test` | 1,976 = 988 对（`<id>_a` / `<id>_b`）| SpeechJudge-Data 的成对 TTS 样本 | **没有**（同上，只有机器分；人的偏好标签不在这里）| **没有** |

- 原始来源：
  - **SpeechEval**（HF `Hui519/SpeechEval`，arXiv 2510.14664）：32,207 条、128,754 条人工核过的标注，含成对比较任务 SQC，四种语言，**CC BY-NC-SA 4.0，无 gate**；是 TTS / VC / 深伪评测，不含降噪【官方明载：数据卡】。
  - **SpeechJudge-Data**（HF `RMSnow/SpeechJudge-Data`，arXiv 2511.07931）：中英 TTS 的人类偏好（可懂度 + 自然度成对），**CC BY-NC 4.0，gated：要填姓名、单位、导师并勾选同意条款 → 按约束不签**【官方明载：数据卡】。
- 用途：对降噪**没有直接用**。唯一可借的是 SpeechJudge 的「成对 + 可懂度」标注格式。

## 5. 其他候选

| 来源 | 内容 | 逐条 / 逐听众 | 降噪？ | 能否直接下 | 结论 | 出处 |
|---|---|---|---|---|---|---|
| Hurricane Challenge 2.0（2020）| 近端听感增强（在加噪**之前**改语音），英 / 德 / 西三种语言，187 名听众，9 个算法，矩阵句 | 听众应答数据**未查到公开**；数据页只列转写、自然语音、TTS 数据 | 否（反方向的问题）| 要注册，非商业许可 | 不适用 | cstr.ed.ac.uk/projects/hurricane/2【官方明载 · 经搜索摘要】；Rennies 等 Interspeech 2020【论文】|
| ESTOI 原始听测（Jensen & Taal 2016）| 用来验证 ESTOI 的几组听测数据 | **未查到公开** | 部分是理想二值掩蔽 | — | 未查到 | — |
| ODAQ（Fraunhofer + Netflix，2023 / 2025）| 240 条 MUSHRA，26 名专家；扩展后 42 人、10,080 个分；**逐听众 XML + 汇总 CSV** | 是 | 部分：对白增强（分离）系统的输出，但素材是影视 / 音乐 | Zenodo 10405774，免登录；大小与许可未查到 | 格式可借，内容不对口 | arXiv 2401.00197、2504.00742【论文】|
| TMHINT-QI 可懂度 | 已在 B §1.5：是**主观可懂度打分**，不是听写 | 有听众编号 | 是（老模型）| HF 转存 2.28 GB | 见 B | B-enhanced-datasets.md:61-75 |
| NOIZEUS 可懂度（Hu & Loizou 2007）| 正常听力 × 13 个经典降噪算法的听写 | 只有论文里的均值表，逐条**未查到** | 是 | — | 只能当文献依据 | 【论文，未核原文】|
| 正常听力 × DNN 降噪 × 逐句听写的公开集 | 搜了 Zenodo / DataShare 方向 | **未查到** | — | — | 缺口 | WebSearch 无结果 |

## 6. 对我们的用途与下一步（待人拍板的标 ⚑）

1. **今天就能做（无需拍板）**：用 CPC3 demo（20 MB，30 条）跑通「吃字」verifier 管道：Whisper 转写 → 正确词% → 对照真人 `words_correct`。只验证管道和字段对齐，**不出结论**。
2. ⚑ **下 CPC2 test（501.6 MB，CC BY-SA 4.0 + 语音库许可）**：897 条、15 人、9 个系统 → 第一份能算「ASR 吃字 verifier 与真人听写的系统级 / 配对一致率」的数据。代价：过线 1.6 MB；结论只适用于「听障 + 助听器」的外部效度。
3. ⚑ 次选 CPC3 dev（752 MB）：含真实场景，并有同一听众的配对。
4. 正常听力会议用户的吃字判据：**公开数据里没有**。要么自己组织小规模真人听写（照抄 CPC 的复述 → ASR → 人工校对流程），要么接受「ASR 吃字 = 代理」并标为假设。

---
本机临时文件（不入仓）：标签包与 demo 放在会话草稿目录 `/private/tmp/claude-501/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-ears/b8014977-0059-4e04-914f-bea9acf86685/scratchpad/{cpc1lab,cpc2lab,cpc3lab,cpc3demo,vmcmeta}/`。
