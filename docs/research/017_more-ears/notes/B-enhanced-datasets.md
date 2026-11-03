# 线 B · 已公开、带人评分的「降噪 / 增强后语音」数据集

> 2026-10-31 · research/017-more-ears · 线 B。只查、只下 label 文本（共 19 MB，无音频），未下载任何 > 500 MB 文件，未上传任何音频。
> 证据等级：【官方明载】【论文】【源码可证】【本机实测】【传言/未核】。查不到的写「未查到」。
> 不重复：014 卡片（`docs/research/014_real-datasets/cards/`：dns*、urgent*-blind-real、chime7-udase、nisqa-livetalk）· 012（`docs/research/012_eval-standards/sources.md` S38/S41/S42）· 004（`docs/research/004_eval-toolchain/gap.md:14` 已记 URGENT 2024 相关性）。

## 0. 一句话结论

**有现成的「众人耳朵」可接，而且比预想多**：URGENT 2024 / 2025 / 2026 三届的众包 P.808 评分都已公开到逐条，其中 **2024 与 2026 带逐听众分**；**2026 还有 12,600 个 CCR 成对比较（−3…+3）+ 117 条中文**，许可 CC BY 4.0。最关键的是这些评分**不下音频也能用**：`urgent-challenge/vmc2026-track1-meta` 把 MOS 与 DNSMOS / UTMOS / SCOREQ 等分数做成了纯文本 `.scp`，本机 19 MB 就能复算「机器分 vs 众人」的配对一致率（第 3 节）。
**缺口**：没有任何一个公开集的被评系统里明确包含 GTCRN / DeepFilterNet / RNNoise 这类小因果模型（未查到）；DNS Challenge 五届都只公开了队伍级结果。

## 1. 逐个数据集

### 1.1 URGENT 2026 · `urgent2026-sqa`（含 VMC 2026 Track 1 的 dev/test 切分）★
| 项 | 内容 | 出处 · 等级 |
|---|---|---|
| 内容 | ICASSP 2026 URGENT Track 1 盲测的主观集：**ACR 5,040 条**（MOS 1–5）+ **CCR 12,600 对**（CMOS −3…+3），**一共 141,386 个单人评分**，840 条源句，9 种语言，6 个系统 | HF 数据卡 huggingface.co/datasets/urgent-challenge/urgent2026-sqa【官方明载】 |
| 每条票数 | ACR 40,280 个评分，**每条 4–16 票**；CCR 101,106 个评分，**每对 4–20 票**；每条源句的 15 个无序系统对全都有 | 同上【官方明载】 |
| 逐听众 | 有 `listener_scores`（list[int8]），是**单人评分**，但**没有听众 ID**（字段表里没有）| 同上【官方明载】；没有 ID 是根据字段表推断 |
| 维度 | 只有整体质量（ACR）与成对偏好（CCR）；**没有 SIG / BAK** | 同上 |
| 系统 | 6 个参赛队：WR(1412) · GHW(1397) · subatomicseer(1401) · baird(1395) · RICK2000(1348) · Ali-Universal-SE(1399)。**未查到**其中有小因果模型；赛道是通用增强，前几名大概率是大模型【传言/未核】 | 同上【官方明载】 |
| 语言 | eng 163 · **zho 117** · ara/deu/fra/hin/ita/jpn/kor 各 80；有 `is_simulated` 字段区分模拟与真实录音 | 同上【官方明载】 |
| 许可 | **CC BY 4.0** | 同上【官方明载】 |
| 大小 | `data/acr` 574.83 MB · `data/ccr` 2,857.56 MB（音频内嵌在 parquet 里）| HF API tree【本机实测】|
| 能否直接下载 | 不用登录、没有 gate；**整份 > 500 MB → 按约束不下载**。但 VMC 2026 dev 子集的 **ACR 只有 111.96 MB**（`vmc2026-track1-dev`，1,008 条，CC BY 4.0），可以直接下载；dev 的 CCR 是 543.19 MB，也超线。test 的 ACR 462.55 MB / CCR 2,295.11 MB | HF API【本机实测】；vmc2026-track1-dev 卡片 `license: cc-by-4.0`【官方明载】|
| 已知结果 | VoiceMOS 2026 Track 1：句级 SRCC 最高 ACR 0.779、**CCR 0.487**；每条 / 每对「typically eight ratings」；没有报系统级 | arXiv 2609.13792 §2.1、§4.1【论文】（012 S42 已记）|

### 1.2 URGENT 2024 · `urgent2024_mos` / `urgent2024-sqa`
| 项 | 内容 | 出处 · 等级 |
|---|---|---|
| 内容 | 300 条盲测句 × 23 个版本（含噪原句 + 22 个增强版本）= **6,900 条**，约 13.8 h | `urgent2024_mos` 卡片【官方明载】|
| 票数 · 方法 | P.808，MTurk 众包，**每条 8 个不同的人**；`raw_ratings` 是 8 个整数 | 同上【官方明载】；本机 meta.jsonl 每条 8 个分【本机实测】|
| 逐听众 | 有单人分，**没有听众 ID** | 同上 |
| 维度 | 只有整体质量 ACR | 同上 |
| 系统 | 匿名的 submission ID；基线是 TF-GridNet（`resources/urgent2024_analysis/README.md:20`）。**未查到**有小因果模型 | 【源码可证】/ 未查到 |
| 语言 | 英语，8–48 kHz | 卡片【官方明载】|
| 许可 | CC BY-NC-SA 4.0 | 卡片【官方明载】|
| 大小 | `urgent2024_mos` 三个 parquet，共 1,342.8 MB；`urgent2024-sqa/blind_test_mos` 1,380.1 MB（同一批，多了 20 多个客观分列）| HF API【本机实测】|
| 能否下载 | > 500 MB → 不下音频。**label 已经在本机**（见第 3 节）| — |
| 旁证 | `urgent2024_analysis/README.md:192` 说逐条 MOS 已经公开；`URGENT-PK/README.md:46` 写「训练用数据集 will be released soon」，而 `README.md:132`（Utilizing the open source Urgent2024_mos dataset）又给了一个脚本，能把 urgent2024_mos 转成 PKDataset → 实际可用的就是这一份 | 【源码可证】|

### 1.3 URGENT 2025 · `urgent2025-sqa`
| 项 | 内容 | 出处 · 等级 |
|---|---|---|
| 内容 | `blind_test_mos` **23,400 条**（本机 label 有 26 个系统 × 900 条），P.808 ACR，**每条 8 票** | HF viewer【官方明载】；arXiv 2507.11306「eight ratings per utterance」、每种语言 150 句【论文】；条数【本机实测】|
| 逐听众 | `raw_ratings` 在预览中**全是 null**；vmc2026-meta 里也没有给 urgent2025 的 meta.jsonl → **按「未公开逐听众」处理** | 【本机实测】|
| 语言 | 有主观分的是 EN / DE / **ZH** / JP（论文投稿时 FR、ES 还没出）；ZH 众包的初筛通过率只有 41.1% | arXiv 2507.11306【论文】|
| 许可 · 大小 | CC BY-NC-SA 4.0；blind_test_mos 3,347.1 MB | HF【官方明载】/【本机实测】|
| 注意 | track2 README:416–418 写「MOS split 赛后公开」，现在已经公开（HF 最后更新 2025-12-10）| `resources/urgent2026_challenge_track2/README.md:416-418`【源码可证】|

### 1.4 CHiME-7 UDASE 主观集（Zenodo 10418311）
| 项 | 内容 | 出处 · 等级 |
|---|---|---|
| 内容 | 真实 CHiME-5 晚餐聚会录音（C0 带噪）+ 4 个系统：C1 CMGAN-FT · C2 ISDS1 · C3 N&B · C4 RemixIT-VAD；另有 P.501 参考；**ITU-T P.835**（SIG / BAK / OVRL）| Zenodo 描述【官方明载】|
| 逐听众 | **有**：`raw_results_listening_test.csv`「individual ratings of each participant」| Zenodo【官方明载】|
| 听众 | 32 人，自报听力正常，分成 4 组、每组 8 人 | arXiv 2402.01413【论文】|
| 条数 | vmc2026-meta 里有 640 行 = 128 段 × 5 个条件；meta 里每条 24 个分，**可能把 SIG/BAK/OVRL 混在一起了，未核** | 【本机实测】|
| 许可 · 大小 | CC BY-SA 4.0；zip **3,491.4 MB**，服务器不支持 Range，没法只取 CSV | Zenodo API【本机实测】|
| 价值 | 唯一一份「真实多人嘈杂 + P.835 三量表 + 逐听众」的数据；014 卡片 `chime7-udase.md` 已收录音频本身 | — |

### 1.5 TMHINT-QI（VoiceMOS 2023 Track 3 训练集）与 TMHINT-QI(S)（VMC 2023 Track 3 测试集）
| 项 | 内容 | 出处 · 等级 |
|---|---|---|
| 内容 | 台湾普通话 TMHINT 句子，babble / street / pink / white × −2/0/2/5 dB；增强系统 KLT、MMSE、FCN、DDAE、Transformer；(S) 版换成 MMSE、FCN、Trans、DEMUCS、CMGAN | arXiv 2310.02640 §2.1.3【论文】|
| 听众 | 训练集 **226 人**（20–50 岁）；(S) 版另找 **110 人**（51 男 59 女），每人听 100 条 | 同上【论文】|
| 维度 | **质量 1–5 + 可懂度 0–10**（十词句里听对的词数）| arXiv 2307.04517【论文】|
| 票数 | 训练 1.3 ± 0.5 票/条（1–6）；测试 3.5 ± 1.1；(S) 5.3 ± 1.0 | MOS-Bench arXiv 2411.03715 表 I/II【论文】；测试集均值 3.52【本机实测】|
| 逐听众 | **有**：`raw_data.csv` 记「subject index, wave file name, quality score, and intelligibility score」| github.com/yuwchen/InQSS README【官方明载】|
| 系统数 | VMC 2023 把「方法 × SNR × 噪声」算一个系统 → 97 个 | arXiv 2310.02640 表 1【论文】；97【本机实测】|
| 许可 | URGENT 表写 **MIT**（`resources/urgent2026_challenge_track2/README.md:179-188`）；InQSS 页面只说「InQSS work ... MIT」，数据本身未单独声明 | 【源码可证】/【官方明载】|
| 大小 · 下载 | Google Drive 原件大小**未查到**；HF 转存 `TMHINTQI.zip` **2,278.2 MB** | HF API【本机实测】|
| 结果 | VMC 2023 Track 3：系统级 SRCC 最好 0.95（T02），各队平均 0.63 | arXiv 2310.02640 §4.2【论文】|
| 小模型 | 没有，都是 2015–2022 年的老方法或非因果模型 | — |
| (S) 版下载 | **未查到**独立下载地址 | — |

### 1.6 VoiceMOS 2024 Track 3（VMC24 track3）
- 训练 / 验证用 CHiME-7 UDASE（60 + 40 条，各 8 人）；**测试集基于 VoiceBank-DEMAND**：40 条干净句 × 4 种噪声 × 0 dB，经 BSSE、MPSENet、CMGAN、DEMUCS、Wiener 五个系统处理，共 280 条；10 位英语母语听者，每条 5 票；P.835 SIG/BAK/OVRL —— arXiv 2409.07001 §2.1.3【论文】。
- 句级 LCC 最好：SIG 0.297 · BAK 0.867 · OVRL 0.713（§4.1）【论文】。**SIG 几乎预测不了**，这一点对我们很要紧：机器分最弱的恰好是「语音被吃掉」这一维。
- 逐听众：未查到；下载地址：未查到（MOS-Bench 称「all publicly available」，但 SHEET 仓库页没列具体链接）【传言/未核】。
- **VoiceMOS 2025 / AudioMOS 2025**：三个赛道是 TTM 音乐、Audiobox Aesthetics、多采样率 TTS，**没有增强赛道**（arXiv 2509.01336）【论文】。

### 1.7 DNS Challenge 五届（INTERSPEECH 2020 → ICASSP 2023）
- **公开的只有队伍级**：本机 `resources/DNS-Challenge/V5_DNS_Challenge_FinalResults.pdf`（1 页）只有每队的 SIG/BAK/OVRL 均值、std、CI、n（每队约 1,800 票）、WAcc【本机实测】。
- 方法：DNS 1、2 用 P.808，DNS 3 起改用 P.835（`resources/DNS-Challenge/README.md:9`【源码可证】；arXiv 2101.01902【论文】）；ICASSP 2021 每段 5 票（012 S38）。
- **逐条评分：未查到公开**。DNSMOS 论文说内部有「about 120,000 audio clips with associated MOS」作训练标签（arXiv 2010.15258），但**未查到下载**。
- ICASSP 2023 SSI 挑战：训练数据里「near end single talk clips have been rated using P.835 and are provided」（signalprocessingsociety.org SSI 2023 页）【传言/未核：只看到搜索摘要】；那是通话信号改善，不是降噪后输出。

### 1.8 其他
| 名称 | 结论 | 出处 · 等级 |
|---|---|---|
| NOIZEUS 主观（Hu & Loizou 2007）| 13 个经典算法，Dynastat 按 P.835 测，论文里有均值表；**逐条 / 逐听众数据未查到公开** | PMC2098693【论文】|
| VCTK-DEMAND 的 MOS | 除 VMC24 track3 外**未查到** | — |
| QualiSpeech | TTS + 加噪，CC BY-NC-SA 4.0，1.58 GB；**未查到**含增强后语音 | HF tsinghua-ee/QualiSpeech【官方明载】|
| SpeechQualityLLM | 只是在 NISQA 上套模板做问答，没有新的人评 | arXiv 2512.08238【论文】|
| NISQA / PSTN / Tencent / TCD-VoIP | 失真与通话类，不含降噪输出（Tencent 无逐听众）| MOS-Bench 表 I【论文】|
| vmc2026-track1-meta 里另有 `speecheval/test`、`speechjudge/test` | 没核内容 | HF API【本机实测】|

## 2. 能否回答「作者一人 vs 大众」？—— 本机实测

**数据**：只下了 `urgent-challenge/vmc2026-track1-meta` 里 4 个集的 label 文本（`mos_overall.scp`、`dnsmos_ovrl.scp`、`utmos.scp`、`scoreq.scp`、`nisqa_mos.scp`、`distill_mos.scp`、`utt2system`、`meta.jsonl`），共 19 MB，**没有音频**。
**脚本**：scratchpad 里的 `scripts/corr.py` 与 `scripts/single.py`（一次性，没进仓；要复现就照下表的定义重写）。
**定义**：
- **uttSRCC** = 语句级 Spearman；**sysSRCC** = 按系统取均值后的 Spearman。
- **pairAgree** = 同一条源句里两两系统配对，机器分的大小方向与众包 MOS 方向一致的比例（剔除平局）。口径和我们 exp003 的「配对一致率」最接近。
- **|ΔMOS|≥1** = 只算众人差距 ≥ 1 分的配对。

| 数据集（全部众包或实验室 ACR）| 指标 | uttSRCC | sysSRCC | pairAgree | pairAgree（\|ΔMOS\|≥1）|
|---|---|---|---|---|---|
| URGENT 2024（6,900 条 · 23 个系统 · 8 票）| DNSMOS OVRL | 0.641 | 0.556 | **0.641** | 0.801 |
| | UTMOS | 0.741 | 0.902 | 0.677 | 0.850 |
| | SCOREQ | 0.789 | 0.926 | **0.695** | 0.890 |
| URGENT 2025（23,400 条 · 26 个系统 · 8 票）| DNSMOS OVRL | 0.654 | 0.670 | **0.657** | 0.799 |
| | SCOREQ | 0.744 | 0.870 | **0.708** | 0.877 |
| TMHINT-QI 测试（1,978 条 · 中文 · ~3.5 票）| DNSMOS OVRL | 0.369 | 0.239 | 0.627 | 0.700 |
| | UTMOS | 0.482 | 0.602 | 0.689 | 0.775 |
| CHiME-7 UDASE（640 条 · 真实 · 5 个条件）| DNSMOS OVRL | 0.276 | −0.100 | **0.513** | 0.521 |
| | SCOREQ | 0.754 | 0.900 | **0.762** | 0.959 |

- **校验**：URGENT 2024 这一行和 `resources/urgent2026_challenge_track2/BENCHMARK.md:6-10` 的同名数字逐个吻合（DNSMOS 句级 .64 / 系统级 .56，SCOREQ .79 / .93）【本机实测 = 源码可证】。CHiME-7 的 DNSMOS / UTMOS 也吻合（BENCHMARK.md:20-21）；NISQA 不吻合（我算的 0.47，表里是 0.03），**原因未查**。
- **人与人的上限（粗估）**：随机抽一个单人评分，和其余人的均值比配对方向，一致率 URGENT 2024 **0.617**（8 票）· CHiME-7 **0.689** · TMHINT-QI 0.71。注意这里配对的两边是**不同的人**、用的是 ACR，会低估同一人自己比较时的一致性，**只作量级参考**。
- **读法（假设，不是结论）**：作者一人与机器分的配对一致率 0.48–0.64，**落在「机器 vs 8 人众包」（DNSMOS 0.64–0.66）和「单个众包评分者 vs 其他人」（≈0.62）的同一量级**。所以现有证据**不支持**「作者偏好与大众格格不入」这一解释；更像是单人加小差距配对本来就噪。在真实录音（CHiME-7）上 DNSMOS 掉到 0.51，与「合成不像」这条线一致。要真正分开三种原因，见第 4 节的 verifier 设计。

## 3. 「对我们最快能用」前 3

1. **`vmc2026-track1-meta` label 文本（已在本机，19 MB，零音频）**
   - 立刻能做的 verifier：`crowd-pair-agree`，用上表的口径给每个机器分算「对众人的配对一致率 + 自助法 CI」，按数据集（合成 / 真实 / 中文）分层。这就是「机器分准不准」的外部标尺，不依赖作者一人。
   - 许可：标签随各源集。URGENT 2024/2025 是 NC，**只作评测**。
   - 缺口：评不了我们自己的模型，只能定标机器分。
2. **`vmc2026-track1-dev` ACR（111.96 MB，CC BY 4.0，可直接下载）→ 之后是 `urgent2026-sqa` 全量（待人拍板 > 500 MB）**
   - 有音频 + 每条 4–16 个单人分 + 中文 + 真实 / 模拟标记；全量还有 CCR 成对偏好，正对应我们的 AB 页。
   - 可以做：①在本机对这些音频重跑我们自己的 verifier，核对 meta 里的数（这会是我们自己管线的第一个外部回归基线）；②**挑一部分中文 CCR 对，夹进作者的盲听页当「锚点题」**：作者在已知众人 CMOS 的配对上答，就能直接测「作者 vs 众人」一致率。这是三种原因里唯一能直接回答「作者是否特殊」的办法，而且不上传、不众包。
3. **CHiME-7 UDASE 主观集（3.49 GB，CC BY-SA 4.0，待人拍板）**
   - 唯一一份真实多人嘈杂 + P.835 SIG/BAK/OVRL + 逐听众（32 人）的数据，最贴会议场景。服务器不支持 Range，只能整包下。

**次选**：TMHINT-QI（中文、有逐听众和可懂度，但系统老旧、训练集每条只有约 1.3 票，HF 转存 2.28 GB）。

## 4. 待拍板（人）

| # | 题 | 选项 | 建议 |
|---|---|---|---|
| 1 | 下载超过 500 MB 的主观集 | A 不下 · B 只下 `urgent2026-sqa` ACR（574.83 MB）· C 再加 CCR（2.86 GB）· D 再加 CHiME-7 UDASE（3.49 GB）| B（中文 + 单人分；先 dev 版 112 MB 冒烟）|
| 2 | 在作者盲听页里插入 URGENT 2026 的众人已评配对作锚点 | 是 / 否；每轮插几题 | 是，每轮 4–6 题（音频是公开的 CC BY 4.0，本机播放）|
| 3 | 能否在 NC 许可的数据（URGENT 2024/2025）上算出数字写进公开仓库 | 只写汇总数 / 不写 | 只写汇总数，不再分发 label 原文 |

## 5. 未查到清单
- 任何公开主观集里有 GTCRN / DeepFilterNet / RNNoise / FastEnhancer 的输出。
- DNS 1–5 与 DNSMOS 训练用的逐条 P.808 / P.835 评分。
- URGENT 2024/2025/2026 的听众 ID（只有匿名的单人分列表）。
- VMC24 track3 与 TMHINT-QI(S) 的独立下载地址；TMHINT-QI Google Drive 原件的大小。
- NOIZEUS 主观集的逐条数据。
- urgent2026-sqa 的听众总数（卡片只给了评分总数 141,386）。
