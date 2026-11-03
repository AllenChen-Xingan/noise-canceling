# 线 Z · 完整性批评：七条线的缺口、矛盾，以及「本机立刻能做的那个实验」

> 2026-10-31 · 017 · 线 Z（批评者）。逐个读完 `notes/A`–`G` 七份。只写本文件，没有做任何 git 操作，没有上传任何音频，没有下载任何音频。
> 证据等级：【官方明载】【论文】【源码可证】【本机实测】【传言/未核】。「推导 / 假设」不算证据。
> `resources/` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`。
> 本线亲自补查的东西都只是**纯文本标签**，用 HTTP Range 从 zip 或 parquet 里单独抽出来，**一共不到 12 MB**（见 §5 复现）。

---

## 0. 结论先行

1. **核心问题的答案：CHiME-7 UDASE 主观集（P.835，32 名固定听众）**。它是唯一一份同时满足以下条件的公开「增强语音」人评集：每位听众的身份可以追溯，而且**同一个人把同一段话的 5 个条件全部听过**。所以它能直接算「单个听众 vs 其余 7 人平均」的配对一致率分布，而这正好就是作者在 exp003 里的处境。它的逐听众 CSV 只有 1.5 MB，可以从 HF 镜像用 Range 单独取出，**不用下 3.49 GB 的整包**（线 B 说「服务器不支持 Range、只能整包下」，那是 Zenodo；HF 镜像支持）【本机实测】。预测器分数也已经有现成的 `.scp`。所以本机**现在就能算完**，结果见 §3.1。
2. **补查后最重要的新数字**（CHiME-7，只用 OVRL，同一句话两两条件配对，去掉平票）【本机实测】：
   - **单个听众 vs 其余 7 人平均：均值 0.774，范围 0.603–0.861**（32 人）。
   - **DNSMOS OVRL vs 单个听众：中位数 0.454，范围 0.290–0.689**。作者测到的 0.48 正落在这个分布的中间。
   - **SCOREQ vs 单个听众：中位数 0.726，范围 0.588–0.799**。
   - 由此推出：「作者 0.48」与「DNSMOS 在真实录音上本来就这么差」完全相容，**不需要假设作者偏好特殊**。另外，线 A、B、F 给出的「单人上限约 0.62 / 0.64–0.70」是**低估**：那是不同人之间的票，或者是模拟出来的；同一个固定听众相对众人能到约 0.77。
3. **发现一个上游 bug，会污染线 B、C 的 CHiME-7 数字**：URGENT 2026 Track 2 的数据准备脚本没有按 `scale` 过滤，把 SIG、BAK、OVRL 三张量表的票混在一起，当成同一个「MOS」（`resources/urgent2026_challenge_track2/scripts/data/chime-7-udase-eval/data_prep.py:42-58`）【源码可证】。所以 `vmc2026-track1-meta` 里 CHiME-7 每条有 24 个分（8 人 × 3 量表），官方 BENCHMARK 的 CHiME-7 那张表也是按这个混合均值算的。用纯 OVRL 重算后，DNSMOS 的配对一致率从 0.513 变成 **0.485**，结论方向不变，但数字要改。
4. **线 B 有一处事实错误**：`vmc2026-track1-dev`（112 MB）**没有逐人分**，字段里只有 `label`（MOS），没有 `listener_scores`【官方明载 HF 卡片】。带 `listener_scores` 的是 `urgent2026-sqa`；它的 ACR 拆成 3 个分片，大小是 **112.0 / 344.4 / 118.4 MB**，单片都不到 500 MB，但合计 574.8 MB【本机实测 HF API + release_manifest.json】。
5. **TMHINT-QI 确实带听众编号**：`raw_data.csv` 的 `idx` 列，226 人，每人 105–108 条。但**每个人每句话只听了一个条件**，所以做不了「同一个人在同一句上比较两个系统」【本机实测】。它只能做跨句、系统级的个人偏差，不能直接对上作者的配对格式。线 A 给它打「潜在 3 分」，应下调。

---

## 1. 互相矛盾的说法与需要更正的地方

| # | 说法 A | 说法 B | 核查结论 | 等级 |
|---|---|---|---|---|
| 1 | A §0：URGENT 2024 MOS 是「**唯一**」同时有增强系统 + 原始逐票的公开集 | B：URGENT 2026 有 `listener_scores`；CHiME-7、TMHINT-QI 也有逐人分 | A 的「唯一」**不成立**。A 在 HF 上找 `urgent2025_mos` / `urgent2026_mos` 没找到，就判为不存在，但实际名称是 `urgent2025-sqa` / `urgent2026-sqa` | 【本机实测 HF API】 |
| 2 | B §1.4：CHiME-7 的 zip 不支持 Range，「没法只取 CSV」 | — | **可以取**：HF 镜像 `urgent-challenge/urgent26_track2_sqa/CHiME-7-UDASE-evaluation-data.zip`（3,491,399,540 B）支持 Range。本线已取出 `listening_test/raw_results_listening_test.csv` 和 `MOS_results_listening_test.csv`。镜像地址见 `resources/urgent2026_challenge_track2/scripts/data/chime-7-udase-eval/prepare.sh:13-14` | 【本机实测】【源码可证】 |
| 3 | B §1.4：meta 每条 24 个分，「可能把 SIG/BAK/OVRL 混在一起了，未核」 | — | **已核：确实混了**。`mos_overall.scp` 里 C0:S01_P01_11 的值是 3.625，等于 24 个分的均值；而原始 CSV 里 BAK 那 8 票正是 meta 的前 8 个数。原因见 §0 第 3 条 | 【本机实测】【源码可证】 |
| 4 | B §3 第 2 条：`vmc2026-track1-dev` 的 ACR（112 MB）「有音频 + 每条 4–16 个单人分」 | — | **错**。dev 的字段只有 `label`，没有 `listener_scores`。带单人分的是 `urgent2026-sqa` 的 `data/acr/test-0000{0,1,2}-of-00003.parquet` | 【官方明载】HF 卡片 |
| 5 | F §2.2：「48 对只来自 12 屏」（引 exp007 README:124） | E §0：「48 对只来自 9 屏」（引 calib json `metrics[0].screens = 9`） | **两边都有出处，源头自己不一致**：calib JSON 顶层是 `screens: 12`，每个指标下是 `screens: 9`；md 和 README 写的是 12。合理的读法是：共 12 屏，其中 3 屏全答「分不出」，有信息的只有 9 屏（E 的说法）。**exp007 的 md 应该同时写出 12 和 9**（建议，属于实现线的事） | 【本机实测】`docs/experiments/exp007-eval-suite/calib/20261031-exp003-round1.json` |
| 6 | G §2：URGENT-PK 权重「大小未查到」 | C §3：mel_resnet34 21.6 MB、utmos_ft 427 MB（HEAD 实查） | 以 C 为准。G 的待拍板第 2 条可以直接消掉：mel 版不到 500 MB | 【本机实测（线 C）】 |
| 7 | G §4：「能否只拉元数据列，没有核实」 | A、B 已经用 Range 读了 parquet 的非音频列 | 以 A、B 为准。本线也对 `urgent2026-sqa` 全部 12 个分片做了同样的读取，只拉了 8.3 MB | 【本机实测】 |
| 8 | A、B、F：单人上限约 0.62（A、B 实测）/ 0.64–0.70（F 模拟） | 本线：固定听众相对 LOO 均值 0.774（CHiME-7 OVRL） | 不矛盾，但口径不同。A、B 的「单人」是**随机抽的一票**，同一配对的两端多半是不同的人，所以会低估；F 的数是拍参数模拟出来的。**作者是同一个人比较两个系统，该对比的是 0.77 这类数字**。注意：0.77 是 ACR 打分后相减，没有强制二选一（平票率 0.39，已剔除）；作者做的是 2AFC，两者仍不完全同口径 | 【本机实测】 |
| 9 | F §2.1：「合成格 0.25–0.38 低于 0.5，个体噪声解释不了」 | — | 这条推理是对的，CHiME-7 的实测也支持。DNSMOS vs 单个听众最低到 0.290，所以**单个人 + 弱预测器**也可能低于 0.5。不过本线的 32 人里，低于 0.38 的只有少数几人（p10 = 0.342）。0.25 在这个分布里属于偏尾。「个体噪声完全解释不了」的说法要弱化为「**不太可能只靠个体噪声**」 | 【本机实测】 |
| 10 | D §4：HF 文件大小标【官方明载】 | — | 那是本线自己调 HF API 查到的，应该标【本机实测】。不影响数字本身 | 等级标错 |
| 11 | A §2.6：TMHINT-QI 的 README 原句标【官方明载】，后面又注「经 WebFetch 摘要，未逐字核对」 | — | 两个标注自相矛盾。现在已经实测：表头是 `idx,method,uttr,snr,noise,quality_score,intelligibility_score,file_name`（`resources/urgent2026_challenge_track2/scripts/data/tmhint-qi/data_prep.py:136-137` 的注释与此一致）；`idx` 按 InQSS README 是 subject index | 【本机实测】【源码可证】 |
| 12 | E §② 把 P.800「20 / 45 分钟」标【传言/未核】 | F §4 写「012 decision 第 79 行（P.800 建议 20 分钟）」，当成依据用 | 同一条依据，在两条线里的等级不一样。在没有核 P.800 原文之前，统一按【传言/未核】处理 | — |
| 13 | C §2.2 与 B §2：CHiME-7 上 NISQA 是 0.47，BENCHMARK 是 0.03 | — | **已核**：BENCHMARK 第 19 行与第 52 行逐格相同（1.38 / 0.02 / 0.03 / 0.02 / 1.10 / 0.12 / 0.13 / 0.08），是抄错了（`resources/urgent2026_challenge_track2/BENCHMARK.md:19,52`）。本线用纯 OVRL 重算 NISQA，uttSRCC = 0.424 | 【源码可证】【本机实测】 |

---

## 2. 遗漏（没覆盖的重要数据集或方法）

| 遗漏 | 为什么重要 | 补查状态 |
|---|---|---|
| **CHiME-7 的逐听众 CSV 可以单独取**（见 §1 第 2 条） | 这条直接决定核心问题的答案 | **已补**（§3.1） |
| **URGENT 2026 的 ACR 与 CCR 互相印证** | 作者用的是配对格式，CCR 是众人在配对格式下的真值；ACR 打分后相减，与 CCR 方向的一致率能说明「绝对打分换成配对」会损失多少 | **已补**（§3.2）：ACR 方向 vs CCR 方向只有 0.632 |
| **URGENT 2026 的 6 个系统都是前排队伍，彼此很接近** | 这和我们「3 个小因果模型、分差小」的处境最像；线 B 只当它是「又一个数据集」 | **已补**：单票 vs LOO 只有 0.576（ACR），比 URGENT 2024 的 0.618 低。**系统越接近，单人越吵**【本机实测】 |
| **Clarity Prediction Challenge（CPC1–3）**：助听器增强输出、有听众 ID 的逐人**可懂度**反应 | 有固定听众 ID 的增强语音数据，但测的是可懂度，不是质量 | **未查**（本线没有时间核大小与许可）。014 卡片 `clarity-cec3-task2.md` 只收了 CEC3 音频 |
| `vmc2026-track1-meta` 里的 `speecheval/test`、`speechjudge/test`、`tcd-voip`、`tencent` 的预测器分数 | 线 B 提到了，但没算 | 未补（不是增强语音，优先级低） |
| 作者自己的**重测信度**数据 | E 和 F 都指出这是天花板的关键，但现在没有 | 没有公开数据可以代替，只能新做（E §⑥） |
| URGENT 2024 的 `raw_ratings` 在同一句的各系统之间**是不是同一批人** | 决定 A 的 0.618 是在测「不同人」还是「同一人」 | **未查到**。P.808 按 HIT 随机分配，按「不是同一批人」处理【假设】 |

---

## 3. 亲自补查的结果

### 3.1 CHiME-7 UDASE：单个听众 vs 留一众人，以及预测器 vs 单个听众【本机实测】

**数据结构（实测）**：`raw_results_listening_test.csv`，19,968 行，表头 `,csv,condition,sample,scale,rating,config_ID,age_listener`。
- **听众 ID = `csv` 列**（每位听众一个答卷文件名），共 32 人，每人 624 行；`config_ID` 有 32 个；`age_listener` 是年龄。
- 条件：`ref`（P.501 参考，4,608 行）和 C0–C4（各 3,072 行）。量表 BAK / SIG / OVRL 各 6,656 行。
- 128 段 × 5 个条件 = 640 条，**每条 8 人**，而且**同一段话的 5 个条件由同一组 8 人听完**（128/128 段都是这样）。
- `MOS_results_listening_test.csv`：2,064 行，`condition,sample,scale,rating_list,MOS`。

**口径**：同一段话里两两条件配对（每段 10 对），比较方向，**两边任一为平票就剔除**。LOO = 去掉该听众后其余 7 人的均值。预测器分数取自 `vmc2026-track1-meta/chime-7-udase-eval/test/*.scp`。

| 量 | OVRL | SIG | BAK |
|---|---|---|---|
| 单个听众 vs LOO 7 人平均（32 人）：均值 / 最小 / p10 / 中位 / p90 / 最大 | **0.774** / 0.603 / 0.710 / 0.794 / 0.824 / 0.861 | 0.789 / 0.638 / 0.691 / 0.801 / 0.860 / 0.905 | 0.869 / 0.762 / 0.821 / 0.877 / 0.907 / 0.930 |
| 单个听众自己的平票率（均值） | 0.392 | 0.346 | 0.287 |
| 每人可用配对数（均值） | 180 | 197 | 220 |

| 预测器（对 OVRL）| 语句级 SRCC | 系统级 SRCC（5 个系统）| 配对一致率 vs 8 人 OVRL 均值（n = 1,191）| **vs 单个听众**：最小 / p10 / 中位 / p90 / 最大 |
|---|---|---|---|---|
| DNSMOS OVRL | 0.232 | −0.100 | **0.485** | 0.290 / 0.342 / **0.454** / 0.583 / 0.689 |
| SCOREQ | 0.744 | 0.900 | **0.767** | 0.588 / 0.654 / **0.726** / 0.767 / 0.799 |
| Distill-MOS | 0.700 | 0.700 | 0.691 | 0.546 / 0.576 / 0.657 / 0.716 / 0.756 |
| NISQA | 0.424 | 0.200 | 0.568 | 0.389 / 0.435 / 0.536 / 0.632 / 0.716 |
| UTMOS | 0.327 | 0.200 | 0.534 | 0.326 / 0.430 / 0.524 / 0.587 / 0.628 |
| （对照）meta 的混合「MOS」| 0.969 | 1.000 | 0.936 | — |

系统 OVRL 均值：C0 2.679 · C1 2.143 · C2 2.752 · C3 3.106 · C4 2.452。

**怎么读**（数字是实测，解释是【假设】）：
- 作者 exp003 的 DNSMOS 一致率 0.48 [0.26, 0.70]，在「DNSMOS vs CHiME-7 单个听众」的分布里位于中位数附近。**光凭这个数，区分不了「作者特殊」和「DNSMOS 在真实录音上不行」**；后者已经由 8 人均值上的 0.485 单独证实。
- 换成 SCOREQ，即便只对一个人，也有 0.59–0.80。所以**下一步最便宜的检验，是用 SCOREQ 给 exp003 打分**（权重约 378 MB，见线 C）。如果作者对 SCOREQ 的一致率远低于 0.59，「作者偏离大众」才开始有证据。这是一个可以写死的判据草案，阈值属【假设】。
- 固定听众相对众人约 0.77，这是作者一致率在「大众口味」意义下的参照线。要注意：作者是降噪伪影的「专家听众」（线 F §1.3），可能系统性偏离普通听众。

### 3.2 URGENT 2026 SQA：系统接近时，单人有多吵【本机实测】

只读了 12 个 parquet 的非音频列（合计 8.3 MB 标签，sha256 `166f26f6…cdb803d128`）。ACR 5,040 条，绝大多数每条 8 票（4,810 条）；CCR 12,600 对，绝大多数每对 8 票（11,699 对）。**没有听众 ID**。

| 子集 | ACR：单票 vs LOO 配对一致率 | CCR：单票 vs LOO 方向一致率（单票平票率）| ACR 均值方向 vs CCR 均值方向 |
|---|---|---|---|
| 全部 | 0.576（7,750 对）| 0.593（0.264）| **0.632**（n = 10,361）|
| 模拟 | 0.576 | 0.575（0.280）| 0.610 |
| 真实 | 0.573 | 0.608（0.253）| 0.648 |
| 中文 zho | 0.599 | 0.558（0.224）| 0.668 |
| 英文 eng | 0.556 | 0.571（0.282）| 0.590 |

- 6 个系统的 ACR 均值都在 3.23–3.55 之间，非常接近。CCR 算出的系统排名与 ACR 不同：ACR 第一是 1412，CCR 第一是 1395。**单票只有 0.58–0.59，比 URGENT 2024 的 0.618 还低**。
- **「ACR 打分后相减」与「众人直接配对」的方向一致率只有 0.632**，这对我们的 verifier 设计很要紧：用 ACR 训出的预测器去对作者的配对判断，先天就有这道折损。
- CCR 的符号约定是推断的（正 = A 更好）：如果反过来，ACR 与 CCR 的方向一致率会低于 0.5。卡片没有写明，属【假设】。

### 3.3 TMHINT-QI：有听众 ID，但做不了同句配对【本机实测】

`TMHINTQI/raw_data.csv`（从 HF 镜像 `TMHINTQI.zip` 用 Range 单独取出，1.7 MB，sha256 `832ac3fd…02732a5`）：
- 有效评分 24,397 条，**226 名听众**（`idx`），每人 105–108 条。
- 14,915 个文件，每文件票数：1 票 8,538 个、2 票 4,316 个、3 票 1,448 个……
- 方法：Noisy / MMSE / DDAE / FCN / Trans / KLT 各约 3,690 条，clean / None 各 1,130 条。
- **(听众, 句子) 组合全部只出现 1 次**，即每个人每句话只听一个版本，没有同一个人的同句配对。

结论：可以研究「个人系统性偏差」（比如某人整体偏严，或特别讨厌某个方法），**不能**直接算作者那种同屏配对一致率。系统是 2015–2022 年的老方法，中文（台湾）。

---

## 4. 核心问题的回答：本机立刻做实验，选哪个

**首选：CHiME-7 UDASE 主观集**（标签现在就能用；音频需要人拍板）

| 项 | 内容 | 等级 |
|---|---|---|
| 下载地址 | Zenodo https://zenodo.org/records/10418311 （不支持 Range，见线 B）；HF 镜像 `https://huggingface.co/datasets/urgent-challenge/urgent26_track2_sqa/resolve/main/CHiME-7-UDASE-evaluation-data.zip`（支持 Range）| 【官方明载】【源码可证】`prepare.sh:13-14` |
| 确切大小 | 整包 **3,491,399,540 B**；zip 内 30,593 个成员，其中 wav 30,539 个、csv 3 个。`raw_results_listening_test.csv` 1,494,380 B；`MOS_results_listening_test.csv` 184,832 B；`objective_evaluation/results_objective_evaluation.csv` 29,871,913 B | 【本机实测】 |
| 文件格式 | zip 包里是 wav（`listening_test/data/{c0..c4,ref}/`）加 CSV | 【本机实测】 |
| 评分文件字段 | raw：`(行号), csv(=听众答卷文件 → 听众 ID), condition(ref/C0–C4), sample(wav 名), scale(BAK/SIG/OVRL), rating(1–5), config_ID, age_listener`；MOS：`(行号), condition, sample, scale, rating_list, MOS` | 【本机实测】 |
| 规模 | 32 人；640 条（128 段 × 5 个条件），每条 8 人，同段 5 个条件由同一组人听；P.835 三量表 | 【本机实测】 |
| 许可 | CC BY-SA 4.0（线 B，Zenodo）| 【官方明载】（线 B 记录）|
| 为什么选它 | ①唯一能算「固定听众 vs LOO」分布的；②真实多人嘈杂录音，最像会议；③有 SIG/BAK/OVRL 三张量表，能对上我们的「吞字 / 残噪」诊断；④预测器分数已有现成 `.scp`，**不下音频**就能得到 §3.1 的结果 | — |
| 局限 | 系统只有 5 个，系统级相关很粗；不是小因果模型；英文；P.835 是 ACR 打分，不是 2AFC | — |
| 要用我们**自己的**预测器管线复算（核对 meta 里的 `.scp`），就要下音频 | 这需要 3.49 GB 整包，超过 500 MB，**待拍板**。按 zip 成员用 Range 只抽 `listening_test/data/c0–c4` 下的 640 个 wav 在技术上可行，大小未测，估计远小于整包【假设】 | — |

**次选（音频在 500 MB 内）：`urgent2026-sqa` 的 ACR 分片 0**
- 地址：`https://huggingface.co/datasets/urgent-challenge/urgent2026-sqa/resolve/main/data/acr/test-00000-of-00003.parquet`，**112,029,611 B**，1,008 行，sha256 `ba471691…a5da205`【官方明载 release_manifest.json】。CC BY 4.0。
- 格式：parquet，音频是内嵌的单声道 FLAC。
- 字段：`sample_id, audio, utterance_id, system_id, team_name, language, speaker_id, text, is_simulated, sample_rate, duration, mos, listener_scores(list[int8])`。
- 用途：本机用 DNSMOS / NISQA / UTMOS / SCOREQ 打分，再算对众人的配对一致率（这份数据没有现成的 `.scp`，正好用来检验我们自己的管线）。逐人分可以做单票 vs LOO，但**没有听众 ID**。
- 注意：分片 0 里各句、各系统分布是否完整，**未核**；按句做配对之前，要先检查每个 `utterance_id` 下 6 个系统是否都在。

**不选**：URGENT 2024 MOS（单片最小 430 MB，但数据集合计 1.34 GB，算不算超线待拍板，而且没有听众 ID）；TMHINT-QI（理由见 §3.3）；`vmc2026-track1-dev`（没有逐人分）。

---

## 5. 复现（scratchpad，git 忽略）

根目录 `/private/tmp/claude-501/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-ears/b8014977-0059-4e04-914f-bea9acf86685/scratchpad/`
- `scripts/rzip.py`（线 A 写的 Range 解 zip 工具）→ `z/chime/listening_test/*.csv`（raw sha256 `4b1aa36e…dae134d8`，MOS sha256 `60fd3889…0c9f5fa536`）、`z/tmhint/TMHINTQI/raw_data.csv`
- `zscripts/chime_listener.py`：§3.1，用法 `uv run --no-project --with scipy python -I chime_listener.py <listening_test 目录> <meta/chime-7-udase-eval/test>`
- `zscripts/u26_cols.py`（读 parquet 的非音频列）→ `z/u26/labels.json`；`zscripts/u26_stats.py`：§3.2（种子 0，单票随机抽 20 次取均值）
- `zscripts/tmhint_listener.py` 和内联检查：§3.3
- 建议大总管把 `chime_listener.py` 和两份 CSV 的 sha256 收进 `docs/experiments/`，做成 verifier `v-crowd-single-listener`（QA 层）：输入一个预测器的 `.scp`，输出它对 32 个单人的一致率分布，再拿作者的一致率去对照这个分布。

## 6. 待拍板

1. CHiME-7 音频：是整包下载（3.49 GB），还是用 Range 只抽 640 个评测 wav，或者不下、只用现成 `.scp`？默认：先不下，现成 `.scp` 的结论已经够用。
2. `urgent2026-sqa` ACR 分片 0（112 MB，在线内）：是否可以直接下来，跑我们自己的预测器管线？默认：可以，它没有超线。
3. 是否向上游报告 CHiME-7 数据准备脚本的混量表 bug 和 BENCHMARK 里 NISQA 那一行的抄错？这算对外发布，要人同意。
