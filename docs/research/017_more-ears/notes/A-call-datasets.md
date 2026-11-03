# 线 A · 公开的、逐条带人评分的通话 / 语音质量数据集

> 调研日 2026-10-31 · 本笔记只调研、不下载大于 500 MB 的东西、不签协议、没有上传任何音频。
> 证据等级：【官方明载】【论文】【源码可证】【本机实测】【传言/未核】。「未查到」就是没查到。
> 本机实测用到的中间文件都放在 scratchpad（git 忽略，见文末「复现」）；要留存的话由大总管决定是否把小文件提交进 `docs/experiments/`。

## 0. 一句话结论

**URGENT 2024 MOS 是唯一同时满足「含多个降噪 / 增强系统的输出 + 每条带原始逐票」这两条的公开集**：22 个系统 + 带噪原声，每条 8 票，无需登录，1.34 GB。音频超过 500 MB 线，没下；标签列已用 HTTP 范围读取单独拿到。本机实测：**随机一位众包听众的一票，和其余 7 人平均分比谁更好，同向率只有 0.618**（语句级，去掉平票后），落在作者「机器分 vs 作者人耳」的 0.48–0.64 区间里。所以作者一人与机器分不一致，本身不能说明机器不准：单个听众相对众人平均，大致就是这么吵。注意这只是旁证，不是同一个量（理由见 §3.1）。
NISQA / PSTN / Tencent 只发布了**每条的平均分**（NISQA 另有每条标准差，PSTN 另有标准差、95% CI 和票数），**都没有逐听众的原始票**。

## 1. 总表

| 数据集 | 条数 | 每条票数 | 维度 | 逐听众原始票 | 含降噪 / 增强输出 | 语言 | 许可 | 大小 · 能否直接下载 |
|---|---|---|---|---|---|---|---|---|
| NISQA Corpus（全部子集）| 14,432 行（CSV 实测，不含已撤下的 NSC）| SIM / LIVE 约 5；TEST 18–34；LIVETALK 24 | MOS + NOI / COL / DIS / LOUD（每项带 std）| **否**（只有每文件 / 每条件的平均 + std）| 没有明确的降噪条件；P501 / FOR 有「Zoom/Skype/Discord + BGN」条件，平台自带的降噪可能已经介入【推断】| 英 / 德 | 按子集不同；整体「可用于非商业研究」| **15.9 GB**，zenodo 直链，无需登录 |
| PSTN Corpus（训练部分）| 58,709 | 1–10，平均 4.55（实测）| 只有 MOS（ACR）| **否**（只有 MOS、std、95%CI、票数）| 未查到（经 Skype/Teams PSTN 承载网，是否经过降噪没写）| 英 | 未查到 | **7.94 GB**，Azure blob 直链，无需登录 |
| PSTN 真实通话测试集（1,040 条 × 30 票）| 1,040 | 30 | MOS | 未查到 | 未查到 | 未查到 | 未查到 | 论文写「link: TBD」，**未查到发布地址** |
| Tencent Corpus（ConferencingSpeech 2022）| CSV 实测 8,366 + 3,197 = 11,563（训练 / 开发部分）| >24 人打分，清洗后 >20；盲测 28 | 只有 MOS | **否**（CSV 只有 `deg_wav,mos`）| **是**：10% 经「噪声抑制」、1.25% 经「噪声抑制 + PLC」；具体哪个降噪器**没写**，只引了 DNS 2021 论文；CSV 没有条件列，**分不出哪几条是降噪过的** | 中文 | 未查到 | 大小未查到（Dropbox / 微云，页面要浏览器）|
| ConferencingSpeech 2022 盲测集（Tencent 20% + PSTN 5% + TUB）| 4,372 | Tencent 28 / TUB 18 / PSTN 24 | MOS | 未查到 | 含「speech enhancement system」条件 | 中 / 英 / 德 | 未查到 | 「下载链接已发邮件给参赛队」，**不公开** |
| IU Bloomington（COSINE + VOiCES）| 36,000 | 未查到 | MUSHRA 0–100 | 未查到 | 否（参考 / 锚点 / 混响 / 带噪）| 英 | 未查到 | Google Drive，大小未查到 |
| **URGENT 2024 MOS** | **6,900**（300 句 × 23 版：带噪 + 22 个系统）| **8**（实测全部 8）| MOS（P.808 ACR）| **有原始票列表**，但**没有听众 ID** | **是：22 个参赛系统的增强输出** | 英 | **CC BY-NC-SA 4.0** | **1.34 GB**（3 个 parquet），HF 不设门槛，无需登录 |
| TMHINT-QI | 训练 12,937 + 测试 1,978 | 约 3 | 质量 1–5 + 可懂度 | **可能有**：`raw_data.csv` 带「subject index」| 是：FCN / Trans / DDAE 等增强模型 | 中文（台湾）| 未查到（代码 MIT）| Google Drive，大小未查到 |
| ITU-T P.Sup23（1998）| NISQA 论文用到 EXP1o 176 条、EXP3d 200 条、EXP3o 200 条 | 未核 | ACR 等 | 未核 | 否（编解码 / 噪声 / 丢包，窄带）| 英 / 日 等 | ITU 免费下载 | 大小未查到 |

## 2. 逐个数据集

### 2.1 NISQA Corpus（TU Berlin，Mittag 等，Interspeech 2021）

- **子集与票数**：【官方明载】`NISQA_Corpus/*/readme.txt`（已用范围读取取到）。【本机实测】`NISQA_corpus_mos.csv` 的 `votes` 列：

  | 子集 | 条数 | 每条票数 最少 / 平均 / 最多 | 条件数 | 语言 | 来源 |
  |---|---|---|---|---|---|
  | TRAIN_SIM | 10,000 | 3 / 5.2 / 10 | 1 条 1 个条件 | 英 | LibriVox (DNS) / TSP / UK-Ireland / AusTalk |
  | VAL_SIM | 2,500 | 3 / 5.2 / 10 | 同上 | 英 | 同上 |
  | TRAIN_LIVE | 1,020 | 3 / 5.3 / 10 | 同上 | 英 | LibriVox 放进真实 Skype / 手机通话 |
  | VAL_LIVE | 200 | 4 / 5.3 / 10 | 同上 | 英 | 同上 |
  | TEST_P501 | 240 | 18 / 28.3 / 34 | 60 × 4 | 英式英语 | ITU-T P.501 附录 C |
  | TEST_FOR | 240 | 21 / 29.3 / 34 | 60 × 4 | 澳式英语 | Forensic Voice Comparison |
  | TEST_LIVETALK | 232 | 24 / 24 / 24 | 58 × 4 | 德语 | 真实通话，8 位说话人 |
  | TEST_NSC | 已从压缩包撤下 | — | — | 德语 | readme：「removed for license reasons」，非商业机构可发邮件索取 |

- **分项**：有。【源码可证】`NISQA_corpus_file.csv` 的表头（实测）：`db,con,file,con_description,…,votes,mos,noi,col,dis,loud,noi_std,col_std,dis_std,loud_std,mos_std,…`；训练配置也要求这几列，见 `resources/NISQA/config/train_nisqa_cnn_sa_ap.yaml:10`。
- **逐听众原始票**：**没有**。【官方明载】顶层 readme 原句：「NISQA_corpus_file.csv contains the ratings averaged per file … NISQA_corpus_con.csv contains the ratings averaged per condition」；各子集 readme 的写法相同。整个压缩包有 27,830 个成员，其中 csv 17 个、xlsx 3 个【本机实测，只列了目录】，没有发现逐票文件。那 3 个 xlsx 没打开，**是否藏着逐票，未核**。
- **采集方式**：【论文 arXiv 2104.09494 §3】训练 / 验证集按 P.808 众包，「5 ratings per file」；P501 / FOR / NSC 按 P.808 众包，「30 ratings per file」；LIVETALK「annotated in the lab according to P.800 with 24 ratings per file」。听众人数：**未查到**（只给了每条票数）。
- **降噪条件**：TRAIN_SIM 的失真清单（readme）只有白噪声、MNRU、DNS 噪声、各种滤波、削波、电平变化、编解码（AMR / G.711 / G.722 / EVS / Opus）、串接、丢包，**没有降噪器**【官方明载】。P501 / FOR 里有「Zoom Bad + BGN 20dB」「Skype Medium + BGN 25dB」这类条件【本机实测 `con_description`】，噪声是送进 Zoom / Skype 再录下来的，平台自带的降噪大概率处理过，但**没写明，属推断**。整个 CSV 里 `suppress|denois|enhanc` 一个都没出现【本机实测】。
- **许可**：LIVETALK「made available for research purposes under … CC BY 4.0」；P501「shall not be included in any commercial product/service」；FOR「only be used for non-commerical research and forensic casework」；SIM / LIVE 按源数据条款（TSP 是 BSD，UK-Ireland 是 CC BY-SA 4.0，AusTalk 是自有条款 PDF）【官方明载，readme 与 wiki https://github.com/gabrielmittag/NISQA/wiki/NISQA-Corpus 】。
- **大小与下载**：`NISQA_Corpus.zip` **15.9 GB**，md5 c4997a873e9397dd6d1fa787d70c3106【官方明载 https://zenodo.org/record/4728081 】，无需登录。本机**只读了目录和小文件**（readme / csv / license），没下载音频。
- **NISQA 自己的数**【论文 2104.09494 表 4，**条件级** PCC，一阶映射后】：TEST_P501 0.95、FOR 0.95、NSC 0.97、LIVETALK 0.90；P.563 在这几个集上分别是 0.72 / 0.52 / 0.69 / 0.70。表 5 分项（条件级）LIVETALK：NOI 0.76、COL 0.87、DIS 0.83、LOUD 0.71。**语句级的数，论文没报**。
- 已有卡片：`docs/research/014_real-datasets/cards/nisqa-livetalk.md`（这里不重复）。

### 2.2 PSTN Corpus（Microsoft，Mittag 等，Interspeech 2020，arXiv 2007.14598）

- **怎么做的**【论文 §2.1】：经「80 different PSTN provider networks in over 50 countries」自动拨打，再经 Skype / Teams 的 PSTN 承载网转进 Skype VoIP 端录下。源音频是 LibriVox（筛后 441 h、2,150 位说话人），加 DNS 2021 噪声，SNR 0–40 dB【ConferencingSpeech 论文 2203.16032 §4.4】。
- **票数**【论文 §2.3】：训练集每条 5 人、测试集每条 30 人，清洗后会更少。【本机实测】`pstn_train/pstn_train.csv` 共 58,709 行，表头 `filename,MOS,std,95%CI,votes`；票数分布 {1:261, 2:2437, 3:9679, 4:21072, 5:18773, 6:319, 7:958, 8:1768, 9:2259, 10:1183}，平均 4.55；MOS 范围 1.0–5.0，平均 3.124。
- **逐听众原始票**：**没有**（只有 std 和 CI）【本机实测】。
- **含降噪吗**：未查到。论文没写 Teams / Skype 承载链路上开没开降噪。
- **真实通话测试集**：「1040 clips extracted from live phone calls」「rated … with 30 ratings」，论文 §2.5 写「under following link: TBD」【论文】。**发布地址未查到**。
- **下载**：https://challenge.blob.core.windows.net/pstn/train.zip ，**7,939,820,986 B（约 7.94 GB）**，Last-Modified 2022-01-27【本机实测 HTTP HEAD】，无需登录。只用范围读取取出了其中唯一的 csv（3.8 MB）。ConferencingSpeech README 原句是「Parts of the PSTN training dataset can be downloaded」，即**只是部分训练集**。
- **许可**：未查到（ConferencingSpeech 仓库的 Apache 2.0 只管代码）。
- **论文里一个有用的数**【论文 §4.2】：用 1–8 票重算 MOS 再训练，「model accuracy increases most rapidly for up to 3 ratings … still not saturated for 8 ratings」。这说明每条票数少于 3 时，标签噪声主导。

### 2.3 Tencent Corpus（ConferencingSpeech 2022）

- **内容**【论文 2203.16032 §4.1，表 1–2】：无混响约 10,000 条中文（源自 MagicData / ST-CMDS / AISHELL-1），每条只施加一种失真（非平稳噪声 60%、白噪声 10% …）；然后第二步「noise suppression [20] and packet loss concealment [21]」。表 2：**「First step + noise suppression」10%、「+ noise suppression + PLC」1.25%**、干净 3.75%、干净 + PLC 1.25%。[20] 是 DNS ICASSP 2021 挑战赛论文，**具体用的哪个降噪器，原文没写**。另有约 4,000 条混响条（28% 仿真、72% 真实房间录音）。
- **票数**：「Each clip was rated by more than 24 listeners. After data cleaning, more than 20 subjective scores」【论文 §4.1】；盲测部分平均每条 28 票、95%CI 0.20【论文表 3】。
- **逐听众原始票**：**没有**。【本机实测】`withoutReverberationTrainDevMOS.csv`（8,366 行）与 `withReverberationTrainDevMOS.csv`（3,197 行）的表头都只有 `deg_wav,mos`，**也没有条件 / 失真类型列**，所以分不出哪些条是降噪过的。
- **语言**：中文。**许可**：未查到。**大小**：未查到（Dropbox 链接返回 HTML 页面，要浏览器；微云 https://share.weiyun.com/B4IS0l3z 没试）。下载页：https://github.com/ConferencingSpeech/ConferencingSpeech2022/tree/main/Training/Dev%20datasets 【官方明载】。

### 2.4 ConferencingSpeech 2022 整体

- 4 个训练语料（Tencent / NISQA / IUB / PSTN），约 86,000 条用于训练开发，4,372 条用于评测，「composed of Chinese, English, and German, and consider background noise, speech enhancement system, reverberation, codecs, packet-loss」【官方明载，仓库 README 与论文 §4.5】。
- 盲测集：Tencent 20% + PSTN 5% + 新建的 TUB（62 个合成条件 × 7 句，平均每条 18 票）。**「The download link of the validation test set has been emailed to the participating teams」**【官方明载 `Evaluation Test Dataset/README.md`】，即不公开。
- 有个可借用的判断【论文 §6.2】：Tencent 盲测上各队 PCC 在 [0.88, 0.97]，另两个集「strongly smaller」；「PCC is strongly dependent to how well the MOS values in the test set are distributed」。这里是**语句级**的数。言下之意：PCC 高低很大程度取决于测试集 MOS 分布得多开，不能跨集比。
- 18 支队伍参赛；第 1 名「consistently in the top-three for all datasets」，但在 Tencent / TUB / PSTN 上，前 2 / 8 / 9 名之间的映射后 RMSE 没有显著差别【论文 §6.2】。

### 2.5 URGENT 2024 MOS（最有用）

- **内容**【官方明载 https://huggingface.co/datasets/urgent-challenge/urgent2024_mos README】：「the noisy and 22 enhanced versions of a subset (300 samples) of the blind test dataset … In total, it 6900 speech samples with MOS labels (~13.8 hours)」；「collected from 8 distinct human subjects through Amazon Mechanical Turk … following the P.808 recommendation」；字段 `id`（`{participant_id}-{fileid}`）、`audio`、`sampling_rate`、`mos`、`raw_ratings`。英语，单声道，8–48 kHz。
- 【源码可证】`resources/URGENT-PK/urgentpk/get_urgent2024_mos.py:12-20`（样例 `'raw_ratings': [3, 3, 4, 2, 2, 2, 1, 2]`）；`resources/urgent2024_analysis/README.md:192`（「The MOS labels for each original/enhanced speech sample in the blind test set are released at …」）。
- 【本机实测】只按列读了 parquet 里的 `id / mos / raw_ratings` 三列，没下载音频：6,900 行，**每行恰好 8 票**，23 个「系统」（含带噪原声）× 300 句。每条 8 票的标准差，平均 0.891。
- **听众 ID**：没有。`raw_ratings` 只是一个列表，同一下标是否同一个人，**未查到，按不是处理**。所以只能估「单张票相对平均的噪声」，**估不了某个固定的人有什么系统性偏好**。
- **许可**：CC BY-NC-SA 4.0【官方明载】。**大小**：3 个 parquet 共 1,342,842,140 B（约 1.34 GB）【本机实测 HF API】，`gated: False`。**超过 500 MB，没下音频**。
- 是不是我们这类因果低延迟小模型：22 个参赛系统多数是非因果的大模型（URGENT 不限因果），**每个系统具体是什么，未逐一核**。
- 同系列：`urgent2025_mos` / `urgent2026_mos` 在 HF 上**不存在**【本机实测 API 返回空】；URGENT 2025 的 P.808 测试论文是 arXiv 2507.11306，摘要里没说放没放数据，**未查到**。

### 2.6 其他（只做了初查）

- **TMHINT-QI**（InQSS，https://github.com/yuwchen/InQSS ）：中文，带噪 + 增强（文件名前缀 FCN / Trans / DDAE），质量 + 可懂度；README 写 `raw_data.csv`「record the corresponding subject index, wave file name, quality score, and intelligibility score」【官方明载，经 WebFetch 摘要，**未逐字核对**】。如果属实，这是**唯一带听众编号**的增强语音集，可以用来量**固定个人的偏差**。约 3 票 / 条【传言/未核，来自检索摘要】。大小、许可：未查到。**值得下一步优先核**。
- **IU Bloomington**：MUSHRA 0–100，36,000 条，MTurk 700 个 HIT【论文 2203.16032 §4.3】，不含降噪输出，逐票未查到。
- **ITU-T P.Sup23**：ITU 免费下载（https://www.itu.int/rec/T-REC-P.Sup23 ），是 1998 年的窄带编解码库，没有 DNN 降噪。NISQA 用到其中 EXP1o / EXP3d / EXP3o 三个集，条件级 PCC 0.92 / 0.92 / 0.91【论文 2104.09494 表 4】。是否附逐票、大小：未核。
- **NOIZEUS**：PSTN 论文 §2.5 说它「designed to evaluated noisy speech that is processed with speech enhancement algorithms」【论文】；主观分能否公开下载：未查到。
- **DNS Challenge 历届 P.808 / P.835 逐条分**：本机 `resources/DNS-Challenge` 下只找到 DNSMOS 模型，**没找到逐条人评分的发布**。未查到公开版本。

## 3. 对三种用途的打分（0–3，3 = 直接能用）

| 数据集 | ① 给 verifier 定标 | ② 检验合成格效度（系统排名）| ③ 量单个听众 vs 众人平均的偏差 |
|---|---|---|---|
| URGENT 2024 MOS | **3**：22 个增强系统 + 带噪，正是降噪伪影的分布；8 票平均，可当语句级和系统级的标尺 | **3**：23 个系统 × 300 句，有众人系统排名，可直接拿来对照我们「合成格 → 排名」的预测 | **2**：有原始票，能量「单张票 vs 众人」（已做，见 3.1）；没有听众 ID，量不了「固定的人」|
| NISQA TEST（P501 / FOR / LIVETALK）| 2：28–30 票，分项 NOI / COL / DIS 能对上我们的「怪声」诊断位；但失真是通话链路，不是 DNN 降噪 | 1：条件是编解码 / 网络，不是降噪系统 | 0：没有逐票；只有 std，只能粗估 |
| NISQA SIM / LIVE | 1：约 5 票，噪声大；量大，适合做大样本粗校 | 0 | 0 |
| Tencent | 1–2：有 10% 降噪处理过的条，但不知道是哪几条；中文 | 0：没有系统维度 | 0 |
| PSTN 训练 | 1：约 4.5 票，链路失真 + 加噪，没有降噪 | 0 | 0（只有 std）|
| TMHINT-QI | 1–2：增强输出，但是老模型（FCN / DDAE），中文，票少 | 1：有几个系统 | **潜在 3**：若真有 subject index，是唯一能量「固定个人偏差」的公开集，**待核** |
| P.Sup23 / IUB | 0–1 | 0 | 未核 |

### 3.1 本机实测：单个听众对众人平均有多吵（URGENT 2024 MOS）

脚本 `scratchpad/scripts/u24_stats.py`，种子 0，数据是本机读到的标签 JSON（sha256 a7b500ce…a399e3）：

| 量 | 结果 | 层级 |
|---|---|---|
| 同一句、两个系统：随机抽一张票 vs 其余 7 张票的平均，判谁更好的**同向率**（去掉平票）| **0.618**（n = 254,068 对）；单张票的平票率 0.273 | 语句级配对 |
| 每条随机抽 4 票 vs 另外 4 票平均的 PCC（分半信度）| 0.645（50 次的中位数）| 语句级 |
| 每条只用 1 张随机票算系统均值，与 8 票的系统排名比 SRCC | 中位数 0.960，第 5 百分位 0.925（200 次）| 系统级，23 个系统 |
| 同上，每条 2 票 / 4 票 | 0.978 / 0.992 | 系统级 |

怎么读（【本机实测】数字 + 【假设】解释）：
- 单张众包票和众人平均在配对上的同向率约 0.62，**与作者一人 vs 机器分的 0.48–0.64 相当**。所以「机器分和作者不一致」，有很大可能只是单人判断本身的噪声，不一定是机器不准。【假设】
- 但这两个量不完全可比：①这里每张票来自不同的人（不是同一个固定的人）；②这里是 ACR 绝对打分再相减，作者用的是直接配对盲听（配对通常更敏感，平票更少）；③这里的「真值」只是 7 人平均，本身也有噪声，会把同向率往下拉。
- 系统级就稳得多：每条只要 1 票、300 句，排名 SRCC 已到 0.96。**想判断系统好坏，增加句子数比增加每句票数更划算**。我们 exp003 的 48 对，量级远不到这里的 300 句 × 23 个系统。
- 可落地成 verifier：`v-single-rater-noise-floor`。用 URGENT 2024 的原始票，自助法估出「单人配对同向率」的分布，算出 95% 区间。作者与机器分的一致率**落在这个区间里**，就判为「与单人噪声不可区分」，不再把它当成机器不准的证据。

## 4. 待拍板（只能人定）

1. **要不要下 URGENT 2024 MOS 音频（1.34 GB，CC BY-NC-SA 4.0）**：超过 500 MB 线。下了之后可以本机跑 DNSMOS / NISQA / UTMOS，拿到「机器分 vs 众人 MOS」在降噪输出上的语句级和系统级相关（URGENT 官方图 `resources/urgent2024_analysis/mos/mos_correlations.png` 已有一版，但我们自己的 verifier 组合要自己跑）。不下的话，只能用标签做 §3.1 这类纯统计。
2. **要不要下 NISQA Corpus（15.9 GB）**：只有 TEST 子集（约 700 条）对定标有用。可以用范围读取只抽 TEST 子集的音频（单条几百 KB，三个 TEST 子集合计估计 < 500 MB，**未实测**），这样不用整包下。
3. **Tencent / PSTN 的许可未写明**：只拿来本机评测，风险【假设】较低；不入库、不再分发。
4. 以上都不涉及上传。众包仍然不做（硬约束）。

## 5. 下一步建议（大总管定）

- 优先核 **TMHINT-QI 的 `raw_data.csv`**（小文件，应能单独取）：如果真带 subject index，就能做 ③ 的「固定个人偏差」，正好对上作者的情况。
- 把 §3.1 的统计做成 `v-single-rater-noise-floor`（CI 层，纯标签、几秒钟），标签 JSON（约 0.5 MB）连同 sha256 一起进 `docs/experiments/` 留存。
- 打开 NISQA 包里那 3 个 xlsx（小文件，可范围读取），确认里面有没有逐票。

## 复现（本机 scratchpad，git 忽略）

根目录 `/private/tmp/claude-501/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-ears/b8014977-0059-4e04-914f-bea9acf86685/scratchpad/`
- `scripts/rzip.py`：远程 zip 列目录 + 按正则抽取小文件（HTTP Range）
- `scripts/u24.py`：只读 URGENT 2024 parquet 的非音频列；`scripts/u24_stats.py`：§3.1 的统计
- `dl/pstn/pstn_train/pstn_train.csv` sha256 9fc746b7c2350ad7d270b8b8d5a0e96d745b28d72df91b4fd86d6951cb1d973c
- `dl/cs22/NISQA_corpus_mos.csv` d5bbc813…fa70b09 · `withoutReverberationTrainDevMOS.csv` 9e366eba…d7678 · `withReverberationTrainDevMOS.csv` baa13374…34bc
- `dl/nisqa_meta/NISQA_Corpus/`：顶层与各子集 readme、`NISQA_corpus_file.csv`、三个 TEST 的 `_file.csv`、许可文本
- `dl/u24_labels.json` sha256 a7b500cebf4f9c2b2245ed262bcdbd8d00755c63dbec8e79946e129005a399e3
- 论文全文：`dl/nisqa.txt`（arXiv 2104.09494）、`dl/cs22.txt`（2203.16032）、`dl/pstn.txt`（2007.14598）
