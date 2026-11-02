# C · 语音增强 benchmark 的批评与饱和（VoiceBank-DEMAND / EARS / 模拟 vs 真实）

> 016 子题 C。只回答「现有 benchmark 哪里坏、模拟测试集什么时候能、什么时候不能预测真实排名、好 benchmark 的清单」，并转成 016 能用的格子 / 检查项。
> 已有调研不重复：合成配方与 sim-to-real 落差数字见 `../../../../noise_canceling-dfnhush/docs/research/015_dfn-hush-synth/notes/B-synth-data.md` §3.3；verifier 清单见主仓 `docs/research/012_eval-standards/verifiers.md`。
> 证据等级：【官方】【论文】【源码】【实测】【二手】【推断】。本文无【实测】。

## 五行结论

1. **VoiceBank-DEMAND（VB-DMD）在说话人、SNR、语种、静音这几方面都太窄**：测试集只有 2 名说话人（23–24 岁），测试 SNR 只有 17.5/12.5/7.5/2.5 dB 四档，句首句尾超过 200 ms 的静音被裁掉【论文 SSW9 §4.1；Giraldo IS2025 §1】。榜单分数涨到 PESQ 3.55，加后处理到 3.69【论文 2405.06573 摘要】。只为刷 PESQ 训练的模型能拿到 3.82，人耳却评它最差【论文 2406.03460 表 1、§4.1】。**VB-DMD 上的排名搬不到更多样的数据上**【论文 Giraldo IS2025 §4】——但注意：Giraldo 的「更多样数据」是 CommonPhone 众包真实录音（无干净参考），PESQ/STOI/SI-SDR 由 SQUIM 模型估计，另加 UTMOS/SCOREQ/NISQA，**没有做人耳测试**（§2.3），只比了 4 个 VB-DMD 模型 + 1 个 spectral gate；所以这是「机器分排名不迁移」，不是「人耳排名不迁移」【复核改正】。
2. **EARS-WHAM 修了「录音质量与说话人多样性」，没修「测试说话人少、只有加法混合、只有英语」**：测试只用 6 名说话人（p102–p107），SNR 在 −2.5～17.5 dB 之间；噪声来自咖啡馆、餐厅、酒吧，正好对上我们的场景；人耳测试只有 20 人 × 10 句【论文 2406.06185 §3.1、§4.3；1907.01160 摘要】。v2 才改用 WHAM! 官方的噪声划分并加了高通【官方 ears_benchmark README】。
3. **模拟能不能预测真实，有正反两面的证据**：模拟和目标域对齐、又没满足某个方法偷偷依赖的假设时，排名大体一致。CHiME-3 上除 MVDR 外的方法基本一致；REVERB 的 ASR「very strong correlation」；LibriCHiME-5 的侵入式指标排名与 CHiME-5 真实录音的人耳排名一致【论文 CHiME-3 CSL 2016 §3.5；官方 REVERB 2014 slides p.28；论文 2402.01413 §6.2.5】。（注意：CHiME-3 / REVERB 的一致性证据是**多通道 ASR 的 WER**，UDASE 的人耳排名只有 4 个系统 + 带噪输入；都不是单通道因果降噪的感知排名，搬到我们场景是【推断】）**一旦模拟恰好满足某个方法的假设，排名就翻**：MVDR 在模拟 dev 上 WER 从 18.71 降到 10.78，在真实 dev 上只从 18.70 降到 18.20【论文 CHiME-3 表 4】。
4. **指标本身和人耳对不上，比「模拟 vs 真实」更致命**：REVERB 中客观指标与「整体质量」MUSHRA 分的相关绝对值只有 0.12–0.39（且是不受控的网上听测）；CHiME-7 UDASE 中 DNSMOS 排第一的系统人耳排最后，非侵入指标与主观的 PCC ≤ 0.50（BAK 除外）【官方 REVERB slides p.43；论文 2402.01413 §6.2.5】。和我们手上 DNSMOS OVRL 0.48 的一致率是同一个量级【推断】。
5. **能直接搬来的「好 benchmark」清单**：BetterBench 的 46 条里，可以直接变成检查项的有这些：地板 / 天花板 / 随机基线、统计显著性、防污染、可复现脚本、局限写明【论文 2411.12990 §4】。挑战赛总结出的做法有：最终盲测只用真实录音（DNS 第二届：650 条真实 + 50 条合成唱歌）、多指标、按困难标签而不是按 SNR 分难度、清洗参考干净轨【论文 2009.06122 §4.1.3；2506.01611 §3】。全部转成检查项见下文 §3。

---

## 1. 正文表格

### 1.1 VoiceBank-DEMAND 被批评的地方

| # | 批评点 | 原文数字 / 原话 | 出处 | 等级 |
|---|---|---|---|---|
| V1 | 训练说话人少、窄 | 28 人（14 男 14 女），同一口音区（England）；另有 56 人版本（Scotland / US）| Valentini-Botinhao et al., SSW9 2016 §4.1（https://www.cstr.inf.ed.ac.uk/downloads/publications/2016/SSW9_Cassia_1.pdf）| 【论文】|
| V2 | 年龄窄 | 「ages predominantly between 20-24 years, except for one 38-year-old male speaker」| Giraldo et al., Interspeech 2025 §1（DOI 10.21437/Interspeech.2025-1760）| 【论文】|
| V3 | **测试只有 2 名说话人** | 「two other speakers from England…a male and a female」；Giraldo：「just two speakers aged 23-24 years」| SSW9 §4.1；Giraldo §1 | 【论文】|
| V4 | 测试集小 | 5 种噪声 × 4 档 SNR = 20 个条件，每人每条件约 20 句；常见说法是共 824 句 | SSW9 §4.1（条件数）；「824」只在二手页面查到（codesota.com/tasks/speech-enhancement），**SSW9 原文没写总句数** | 【论文】/【二手】|
| V5 | SNR 偏高，离散四档 | 训练 15/10/5/0 dB；测试「four slightly higher SNR values: 17.5 dB, 12.5 dB, 7.5 dB and 2.5 dB」| SSW9 §4.1；PESQetarian 2406.03460 §3.3 复述 | 【论文】|
| V6 | 噪声种类少 | 训练 10 种（DEMAND 8 种 + 语音形噪声 + babble），测试 5 种 DEMAND（living room、office、bus、open area cafeteria、public square）| SSW9 §4.1 | 【论文】|
| V7 | **裁掉静音** | 「silence segments longer than 200 ms had been trimmed off from the beginning and end of each sentence」→ 测试里几乎没有「只有噪声」的段 | SSW9 §4.1 | 【论文】（复核：这句写在**训练集**构造段里，测试集段没有重述，测试集也裁静音是【推断】，可信度高但未见原文）；对背景稳定 / 门控的影响是【推断】|
| V8 | 只有加法混合、模拟噪声 | 「limited diversity and simulated noise conditions」；另一篇称它「relatively simple…only two speakers and additive noise」| Giraldo 摘要；2402.16321（自监督 SE）正文 | 【论文】|
| V9 | 只有英语 | VB-DMD、DNS、CHiME「restricted exclusively to English」| Giraldo §1 | 【论文】|
| V10 | **排名搬不过去** | 「the model ranking on VB-DMD does not directly transfer to a more diverse and realistic dataset」；年龄越大分越低，最好的年龄段 20–40 正好是训练集年龄（Kruskal-Wallis χ²(6)=22.46, p<.001；原文说按模型 × 语种分别检验却只报了这一个统计量）| Giraldo §3.1.1、§4 | 【论文】；复核：评测集是真实众包录音，指标全是 SQUIM 估计或非侵入 MOS 预测器，无人耳 |
| V11 | 对 PESQ 过拟合 | Giraldo 认为 MP-SENet、SEMamba 显式优化 PESQ，存在「potential overfitting to PESQ」，并且「models achieving excellent quality scores often perform poorly in content retention」| Giraldo §4 | 【论文】|
| V12 | **指标可被刷** | 只为 PESQ 训练的「PESQetarian」：PESQ 3.82，但 POLQA 1.46、SI-SDR −19.8 dB、DNSMOS 2.06（都低于带噪输入）；10 名专家听，它最差。只改第一个采样点的「click trick」就让 PESQ 达到 3.46±0.71，「would rank fourth on the leaderboard」| PESQetarian（arXiv 2406.03460）表 1、§4.1–4.2 | 【论文】|
| V13 | 榜单分数接近饱和 | SEMamba：PESQ 3.55，加 PCS 后处理 3.69 | arXiv 2405.06573 摘要 | 【论文】；「0.95–0.96 STOI 自 2022 不动」只在 codesota 查到 | 【二手】|
| V14 | 小数据集上的过拟合 | URGENT：「SE models are often evaluated on fixed, small (e.g., ~10 h) or medium datasets (e.g., ~100 h). It is very likely that recent SOTA models heavily overfit these datasets」，测试集与训练「matched in terms of speech quality, linguistic content, noise family」| URGENT 2024（arXiv 2406.04660）§1 | 【论文】|

### 1.2 EARS-WHAM / EARS-Reverb 的设计与已知局限

| # | 项 | 原文数字 | 出处 | 等级 |
|---|---|---|---|---|
| E1 | 语音 | EARS：100 h、107 人、48 kHz、消声室、7 种朗读风格、自由说话、22 种情绪；年龄 18–75 | 2406.06185 表 1、§2；年龄引自 Giraldo §1 | 【论文】|
| E2 | 噪声 | WHAM! 真实噪声，「coffee shops, restaurants, and bars in the San Francisco Bay Area」；CC BY-NC 4.0 | 2406.06185 §3.1；WHAM! arXiv 1907.01160 摘要 | 【论文】|
| E3 | SNR | 在 [−2.5, 17.5] dB 均匀抽取；按 ITU-R BS.1770（LKFS）计算，去掉静音 | 2406.06185 §3.1 | 【论文】|
| E4 | 划分 | 训练 p001–p099（32,485 条，86.8 h）、验证 p100–p101、**测试 p102–p107（6 人，886 条，3.7 h）**；测试排除超过 29 s 的条目；感叹、旋律、非言语、生理类声音被剔除 | 2406.06185 §3 开头（复核：划分与条数在 §3 引言段，不在 §3.1）；「29 s」与剔除类别未核 | 【论文】|
| E5 | 盲测 | 6 名 EARS 之外的说话人（3 男 3 女），743 条（2 h），**噪声是专门新录的**，只发布带噪音频，有在线评测服务器 | 2406.06185 §3.1、表 5 | 【论文】|
| E6 | EARS-Reverb | 7 个公开库的真实 RIR，共 2,313 条；去掉直达声之前的部分；RT60 超过 2 s 的剔除；混响语音按 LKFS 归一到干净语音 | 2406.06185 §3.2、表 2 | 【论文】|
| E7 | 人耳测试规模小 | MUSHRA 式，20 人 × 10 句（5 男 5 女），问「overall quality considering artifacts and residual noise」| 2406.06185 §4.3 | 【论文】|
| E8 | v1 → v2 的修补 | v2（2025-04-25）「Uses the recommended train, valid, test split from WHAM for the noise files」、高通滤掉干净轨里的低频噪声、用能量门限避免空语音文件；Reverb v2 按数据集划分 RIR | https://github.com/sp-uhh/ears_benchmark README | 【官方】|
| E9 | 由 E8 推出的 v1 问题 | v1 的噪声和 RIR 很可能没按官方划分 → 训练与测试之间可能有噪声 / 房间泄漏；「干净」参考里带低频噪声 | 由 E8 倒推 | 【推断】|
| E10 | 作者没写局限节 | 只有英语；噪声与混响**分成两个任务**，没有「噪声 + 混响」联合格；仍是加法混合；测试说话人 6 人 | 2406.06185 全文（无 limitations 节）；Giraldo §1 也指出只有英语 | 【论文】+【推断】|
| E11 | 第三方针对 EARS-WHAM 的系统性批评 | **没找到** | — | — |

### 1.3 「模拟测试集排名能不能预测真实」：系统性证据

| # | 场景 | 结果（原文数字） | 结论方向 | 出处 | 等级 |
|---|---|---|---|---|---|
| S1 | CHiME-3 模拟 vs 真实（ASR + 增强）| 摘要：「with the exception of MVDR beamforming, most algorithms perform consistently on real and simulated data」；作者主张看**相对提升**而不是绝对 WER | 大体一致 | Vincent et al., CSL 2016 摘要、§3.1（https://eprints.whiterose.ac.uk/111196/）| 【论文】|
| S2 | CHiME-3 绝对 WER 差 | 基线 DNN：dev 真实 16.13 / 模拟 14.30；**test 真实 33.43 / 模拟 21.51**（测试说话人在噪声环境下说得更不清楚）| 绝对值不可比 | 同上，表 3 | 【论文】|
| S3 | **MVDR 翻车** | dev WER：无处理 真实 18.70 / 模拟 18.71；MVDR 真实 **18.20** / 模拟 **10.78**；DS 真实 12.43 / 模拟 14.52；MCA 真实 10.72 / 模拟 12.50。原因：模拟用纯延时滤波器，正好满足 MVDR 的约束，真实里有早期反射，这个约束不再成立 | **模拟偏袒依赖其假设的方法，排名翻转** | 同上，表 4、§3.2.1、§3.5 | 【论文】|
| S4 | CHiME-3 训练型增强 | DNN 多通道维纳滤波用真实数据训练更好，主要原因是模拟数据的 ground truth 定义方式不同 | 真值定义本身会造成落差 | 同上，§3.5 | 【论文】|
| S5 | REVERB 2014 ASR | 「Very strong correlation between SimData and RealData scores (Even stronger between SimData Room3 Far and RealData)」；Room3 的 T60 0.7 s 本来就是照 RealData 设的 | RT60 对齐后排名更一致 | REVERB summary slides p.6、p.28（https://reverb2014.audiolabs-erlangen.de/workshop/slides/reverb_summary.pdf）；期刊版 EURASIP JASP 2016 doi:10.1186/s13634-016-0306-6（全文被重定向，没读到相关系数）| 【官方】|
| S6 | REVERB SE：客观 vs 主观 | 与「混响量」MUSHRA 的平均相关：CD −0.70、FWSegSNR 0.71、LLR −0.43、SRMR 0.62、PESQ 0.77；与「整体质量」：CD −0.35、FWSegSNR 0.39、LLR −0.21、**SRMR 0.12**、PESQ 0.28。「The overall quality is not well captured with the objective measures used」（复核：CD、LLR 是越小越好，负号是正常方向，按绝对值是 0.12–0.39；该听测是「Web-based listening test (not well controlled)」，slides p.42）| 指标能粗测「压了多少」，测不了「好不好听」| 同上 slides p.43 | 【官方】|
| S7 | DNS 2020 | 「While the performance is good on the synthetic test set, often the model performance degrades significantly on real recordings」；常规客观指标「do not correlate well with subjective tests」| 合成 → 真实 掉分 | Reddy et al., arXiv 2005.13981 摘要、§2.4.1（表 2 分项数字是图片，没读到）| 【论文】|
| S8 | DNS 第二届（ICASSP 2021）的应对 | Track 1 盲测：700 条，**650 条真实录音 + 50 条合成唱歌**；合成「mainly composed of the scenarios that we were not able to collect in realistic conditions」；最好的模型只比带噪输入高 0.53 DMOS | 最终裁判换成真实录音，合成只补采不到的格 | arXiv 2009.06122 §4.1、§4.1.3、§5 | 【论文】|
| S9 | CHiME-7 UDASE（真实 CHiME-5 + 匹配模拟 LibriCHiME-5）| 侵入式指标在 LibriCHiME-5 上的排名与 CHiME-5 人耳 P.835 排名「consistent」；**CMGAN 在 DNSMOS 上排第一，人耳测试里最差**（复核：进人耳测试的只有 CMGAN-FT 一个变体，§6.2.2）；非侵入指标与主观的 PCC ≤ 0.50（DNSMOS BAK 0.73 除外）；4 个系统里只有 1 个（N&B）整体质量显著好于原始带噪（§6.2.4）；32 名听者分 4 组、每组 8 人，共 128 条、每条每个条件 8 票（§4）；**排名一致的结论只建立在 4 个系统上** | **域匹配的模拟 + 侵入式指标 > 真实数据 + 非侵入 MOS 预测器** | Leglaive et al., arXiv 2402.01413 §4、§6.1.3、§6.2.4–6.2.5 | 【论文】|
| S10 | URGENT 2024 盲测 | 500 条模拟 + 500 条真实；MOS 子集 300 条（各半）；与 MOS 相关最高的是 UTMOS、SCOREQ，最低的是 WADA-SNR；整体排名与 MOS 排名的 KRCC 为 0.73；T17/T18 在 DNSMOS/NISQA 上排前，MOS 上排后；**很多难样本 SNR > 10 dB**；困难来源是重叠语音、强宽带 / 瞬态噪声、强混响 | SNR 不是难度；单个指标会被骗 | Zhang et al., arXiv 2506.01611 §2.3、§3.1–3.2；本地 `resources/urgent2024_analysis/README.md`（MOS correlation 节）| 【论文】【源码】|
| S11 | URGENT 2024 标签噪声 | 约 2000 条训练「干净」语音 WADA-SNR 为负；「most SE studies are implicitly conducting a sort of noisy-target training」；LibriTTS 约 25%、DNS5 LibriVox 和 CommonVoice 约 100% 的有效带宽与标称采样率不符 | 「干净参考」本身不干净 | 2506.01611 §2.1 | 【论文】|
| S12 | 跨语料泛化（SE）| 低 SNR 下对新语料泛化失败，主因是**通道失配**（录音条件不同）；对新噪声、新说话人泛化尚可 | 设备 / 通道才是要命的维度 | Pandey & Wang, arXiv 2002.04027 摘要（TASLP）| 【论文】|
| S13 | 泛化落差量化 | ΔPESQ 落差：三重失配（语音 + 噪声 + 房间）、单库训练时，FFNN −46%，Conv-TasNet −79%，DCCRN −71%，MANNER −71%；**语音失配的落差最大**；新模型在匹配条件下更好，在失配条件下可能输给 FFNN | 匹配格上的「新 SOTA」不可信 | Gonzalez et al., arXiv 2309.06183 表 V、§V-B/C（TASLP 2023）| 【论文】|
| S14 | ASR：基准 vs 真实 | 商用 ASR 在真实自发会话上的 WER「significantly higher than the best reported results」| 基准低估真实难度 | Szymański et al., arXiv 2010.03432 摘要（EMNLP Findings）| 【论文】|
| S15 | ASR：多基准平均 | 用的基准够多时，跨基准平均 WER 是真实带噪数据表现的好代理 | 多格平均优于单格 | Likhomanenko et al., arXiv 2010.11745 摘要 | 【论文】|
| S16 | 静态基准饱和（NLP）| 模型很快在基准上拿到高分，却仍在简单挑战样例和真实场景上失败 → 人与模型一起在环里、动态造题 | 基准要会「长」| Kiela et al., Dynabench, arXiv 2104.14337 摘要 | 【论文】|
| S17 | 直接给出「VB-DMD 排名 vs 真实录音人耳排名」τ 的论文 | **没找到**。Giraldo 只给了定性的「不直接迁移」，排名对比做在**真实众包录音**（CommonPhone）上，但用的是 SQUIM 估计指标与非侵入 MOS 预测器，没有人耳（复核改正：原写「模拟 / 众包集」）| — | — | — |

### 1.4 「好的 benchmark 应满足什么」的清单来源

| # | 条目 | 原文 | 出处 | 等级 |
|---|---|---|---|---|
| B1 | 设计 14 条（分阶段条数未核：14+11+19+3=47，与摘要总数 46 对不上）| 包括「Metric floors and ceilings are included」「Human performance level is included」「Random performance level is included」「Has validated automatic evaluation」「Input sensitivity is addressed」「How benchmark score should or shouldn't be interpreted」| BetterBench, arXiv 2411.12990 §4.1 图 2 | 【论文】|
| B2 | 实现 11 条 | 「Script to replicate results」「Statistical significance or uncertainty quantification」「Globally unique identifier or encryption of evaluation instances」「Task to identify if model has been trained on benchmark data」| 同上 §4.2 图 3 | 【论文】|
| B3 | 文档 19 条 + 维护 3 条 | 「Limitations are documented」「Data representativeness is explained」「Applicable license is specified」；维护：一年内检查过可用性、有反馈渠道、有联系人；退役要声明 | 同上 §4.3–4.4 | 【论文】|
| B4 | 评估现状 | 评估了 24 个基准：「most benchmarks do not report statistical significance…nor allow for their results to be easily replicated」| 同上，摘要 | 【论文】|
| B5 | SE 挑战赛的经验 | 多指标（URGENT 2024 用了 13 个）；用 UTMOS/SCOREQ 补 DNSMOS；用困难度指标平衡语料；重估有效带宽；过滤标签噪声 | 2506.01611 §3–4 | 【论文】|
| B6 | 统计稳健 | DNS 2020：每条 10 名评分者，每个提交的 95% CI 为 0.02；前几名再补 10 人，用 ANOVA 定显著性（p<0.05）| 2005.13981 §6.2 | 【论文】|
| B7 | 听测必须有 | PESQetarian 结论：「the paramount importance of formal listening experiments」，并用多个指标交叉 | 2406.03460 §5 | 【论文】|

---

## 2. 归纳：模拟格什么时候能代替真实（给 016 的判据骨架）

把 S1–S17 合起来，**模拟格的排名能代表真实，要同时满足四个条件**；任何一条不满足，就会出现 S3 / S9 / V10 那样的翻转：

1. **域对齐**：混响、通道、说话方式等关键物理量和真实对齐（S5 Room3 ↔ RealData；S9 LibriCHiME-5 ↔ CHiME-5）【论文】
2. **不偷偷满足某方法的假设**：模拟比真实「更理想」的那一维会偏袒依赖它的方法（S3 MVDR 与纯延时；对我们来说是「纯加法 + 无房间 + 无设备通道」）【论文 + 推断】
3. **真值定义一致**：干净参考是怎么来的，模拟和真实要一致，而且参考本身要干净（S4、S11）【论文】
4. **指标对齐人耳**：指标本身的一致率不够，排名对不对就无从谈起（S6、S9、V12）。我们 DNSMOS OVRL 0.48 的情况属于这一条【推断】

**模拟格不能告诉你绝对分数**（S2 中真实与模拟的 test WER 是 33.43 对 21.51），**只能用来比较相对提升 / 排名**【论文 CHiME-3 §3.1】。

---

## 3. 对 016 的启示（可直接转成 benchmark 格子 / 合成数据检查项）

> 编号 C-xx，供 `benchmark.md` / `synth-standard.md` 引用。门限数字一律**待人定**，下表只给出处里的参照值。

### 3.1 benchmark 格子

| 编号 | 格子 / 维度 | 为什么 | 依据 |
|---|---|---|---|
| C-G1 | **SNR 至少下探到 −5 dB**（VB 最低 2.5 dB，EARS 最低 −2.5 dB），并且 SNR 只作为一个轴，不当作难度本身 | 很多难样本 SNR > 10 dB；咖啡馆邻座说话常常 ≤ 0 dB | V5、E3、S10 |
| C-G2 | **按噪声结构分格**：稳态 / babble / **邻座清晰人声（重叠）** / 瞬态（杯碟、键盘）/ 强混响 | URGENT 的困难标签正是重叠、强瞬态、强混响 | S10 |
| C-G3 | **「只有噪声」段与停顿格**：每条至少有 1 段 ≥ 2 s 不含目标语音（时长待定）| VB 裁掉了 > 200 ms 的静音，测不出背景稳定性和门控误开 | V7 |
| C-G4 | **设备通道格**：笔记本内置麦录的干净语音（或设备 IR）vs 录音棚语音 | 跨语料失败的主因是通道失配 | S12 |
| C-G5 | **说话人维度**：测试说话人 ≥ N（VB 2 人、EARS 6 人都太少），覆盖性别与年龄段；报告逐说话人的方差 | 年龄效应显著，最好的年龄段 = 训练集年龄段 | V3、V10、E4 |
| C-G6 | **语种格**：至少中文 + 英文 | 主流基准只有英语；纯生成式模型有语言依赖 | V9、2505.23212 摘要 |
| C-G7 | **「噪声 + 混响」联合格**，不要只分开测 | EARS 把两者拆成两个任务 | E10 |
| C-G8 | **真实锚格（必须有）**：同场景实录的笔记本麦会议片段，只用作排名的裁判 | DNS 第二届的最终盲测约 93% 是真实录音（650/700 = 92.9%；复核改正，原写 97%）；我们现有答卷全是加法混合 | S8、任务背景 |

### 3.2 合成数据 / benchmark 检查项（能用规则判的，交给 verifier）

| 编号 | 检查项 | 判法（建议的 verifier 形态） | 依据 |
|---|---|---|---|
| C-K1 | **排名迁移检查**：同一组系统（≥ 5 个，含传统 DSP 基线）在合成格上的排名，与真实锚格上人耳 / 可信指标排名的 Kendall τ ≥ 阈值才算合格 | `V-RANK-TRANSFER`：τ 与 bootstrap CI；阈值待人定（参照：URGENT 整体排名 vs MOS 的 KRCC 为 0.73）| S1、S9、S10、S17 |
| C-K2 | **假设审计**：列出合成器比真实「更理想」的每一维（纯加法、无房间、无设备 IR、无削波、无 AGC、平稳），每一维至少有一个打破它的对照格；如果某方法只在「理想维」上领先，标红 | 清单式检查 + 对照格分差 | S3 |
| C-K3 | **干净参考的质量**：WADA-SNR（或同类指标）为负的参考剔除；有效带宽与标称采样率一致；低频噪声做高通 | 自动统计 + 门限 | S11、E8 |
| C-K4 | **划分泄漏**：噪声、RIR、说话人按**录音会话 / 来源库**划分（不按文件），训练、验证、测试之间零交集；记下哈希 | 规则检查，CI 层 | E8、E9 |
| C-K5 | **地板 / 天花板 / 平凡基线**：每格必须同时报带噪输入、干净参考（侵入式指标的天花板）、谱减或噪声门这类平凡基线 | 报表缺一项就判失败 | B1、Giraldo 用 spectral gate 做基线 |
| C-K6 | **统计显著性**：每格报 95% CI；两个系统 CI 重叠就判「不可区分」，不许排先后 | bootstrap 或 ANOVA | B2、B4、B6 |
| C-K7 | **防刷分探针**：verifier 套件里放对抗样本（例如 PESQ 的 click trick、只压电平、只截断首帧），任一指标对探针给高分，该指标就降权 | CI 层单测 | V12 |
| C-K8 | **单一指标不进裁判**：训练用的损失指标不得作为唯一验收指标；至少一个侵入式、一个非侵入式、一个内容保真（ASR / 音素）| 规则检查 | V11、V12、B5 |
| C-K9 | **非侵入 MOS 预测器要先在本域定标**：DNSMOS / UTMOS 等用之前，先在真实锚格上算它与人耳的相关，不过线就只当参考 | 与 012 的 L0–L3 定标对接 | S9、S10、任务背景（0.48）|
| C-K10 | **饱和监测**：某格里前几名与天花板的差落进 CI 时，该格标「饱和」，需要加难度或退役 | 自动报表 | V13、S16、B3 |
| C-K11 | **盲测分离**：裁判格的干净参考不进任何调参流程；限制查询次数；定期轮换 | 访问日志 | E5、S8、015 B §3.4 |
| C-K12 | **版本与变更日志**：benchmark 本身有 build id 和 changelog（EARS 也出了 v1→v2）| 规则检查 | E8、B3；CLAUDE.md §5.1 |
| C-K13 | **只比相对、不比绝对**：合成格只报相对带噪输入的提升与排名，不把绝对分数当成真实场景的预期 | 文档规则 | S2 |
| C-K14 | **多格平均作为总分**：总分取各格平均（或加权），不取单格；权重按真实会议分布（交给 F-distrib）| 报表规则 | S15、`notes/F-distrib.md` |
| C-K15 | **人耳测试的最低规模**：EARS 用了 20 人 × 10 句，UDASE 用了 32 人 × 128 条、每条 MOS 8 票；我们的人工层要写明人数、条数、CI | 文档 + CI 计算 | E7、S9、B6 |

### 3.3 对「整体层」合成数据的直接含义

- 合成数据合格的最终判据，不是「合成格上分数高」，而是 **C-K1 的排名迁移 τ 过线**；单类层、多类层是为了定位落差，整体层是为了过 C-K1【推断，来自 §2】
- 现有答卷全是加法混合，**等于整套人耳答卷都坐在 C-K2 的「理想维」上**。在补上真实锚格（C-G8）之前，任何「合成数据合格」的说法只能标「假设」（CLAUDE.md §5.3）【推断】

---

## 4. 没查到的空白

1. REVERB 期刊版（EURASIP JASP 2016）的 SimData vs RealData 相关系数**没读到**（Springer 重定向到登录）；只有 slides 上的「very strong」，SE 部分的模拟 / 真实排名对比也没读到
2. DNS 2020 表 2（合成 / 合成带混响 / 真实三类的 P.808 分项）是图片，**没拿到数字**，所以没法算三类之间的排名 τ
3. **没找到**直接测「VB-DMD 排名 vs 真实录音人耳排名」τ 的论文
4. **没找到**第三方对 EARS-WHAM 的系统批评论文；E9（v1 可能有泄漏）是从 v2 的修补说明倒推的
5. VB-DMD 测试集「824 句」只有二手来源，原始论文没写总数
6. 没查到任何以「笔记本内置麦 + 咖啡馆」为测试条件的公开基准

---

## 5. 来源清单

| 简称 | 来源 | 读法 |
|---|---|---|
| SSW9 | Valentini-Botinhao et al., *Investigating RNN-based speech enhancement methods for noise-robust TTS*, SSW9 2016, https://www.cstr.inf.ed.ac.uk/downloads/publications/2016/SSW9_Cassia_1.pdf | 读了 PDF 全文 §4.1 |
| VB 数据页 | https://datashare.ed.ac.uk/handle/10283/2791 | 官方页 |
| Giraldo | Giraldo et al., *Evaluating Speech Enhancement Performance Across Demographics and Language*, Interspeech 2025, doi:10.21437/Interspeech.2025-1760，https://www.isca-archive.org/interspeech_2025/giraldo25_interspeech.pdf | 读了 PDF §1、§3、§4 |
| PESQetarian | arXiv 2406.03460 | 读了 HTML 版 |
| SEMamba | arXiv 2405.06573 | 摘要 |
| EARS | Richter et al., arXiv 2406.06185（Interspeech 2024）| 读了 HTML 版 |
| EARS repo | https://github.com/sp-uhh/ears_benchmark（README）| 官方 |
| WHAM! | Wichern et al., arXiv 1907.01160 | 摘要 |
| URGENT 2024 | Zhang et al., arXiv 2406.04660 | 读了 PDF §1 |
| URGENT 2024 lessons | Zhang et al., arXiv 2506.01611（Interspeech 2025）；`resources/urgent2024_analysis/README.md` | 读了 HTML 版 + 本地源码 README |
| URGENT 2025 | Saijo et al., arXiv 2505.23212 | 摘要 |
| CHiME-3 分析 | Vincent et al., *An analysis of environment, microphone and data simulation mismatches in robust speech recognition*, CSL 2016, doi:10.1016/j.csl.2016.11.005，https://eprints.whiterose.ac.uk/111196/ | 读了 PDF 全文 |
| REVERB | Kinoshita et al., REVERB summary slides，https://reverb2014.audiolabs-erlangen.de/workshop/slides/reverb_summary.pdf；期刊版 doi:10.1186/s13634-016-0306-6 | 读了 slides；期刊版没读到 |
| DNS 2020 | Reddy et al., arXiv 2005.13981 | 读了 PDF |
| DNS ICASSP 2021 | Reddy et al., arXiv 2009.06122 | 读了 PDF §4–5 |
| UDASE | Leglaive et al., arXiv 2402.01413（CSL 2024）| 读了 HTML 版 |
| Pandey | Pandey & Wang, arXiv 2002.04027（TASLP）| 摘要 |
| Gonzalez | Gonzalez et al., arXiv 2309.06183（TASLP 2023）| 读了 HTML 版 |
| WER we are | Szymański et al., arXiv 2010.03432（EMNLP Findings 2020）| 摘要 |
| Rethinking ASR eval | Likhomanenko et al., arXiv 2010.11745 | 摘要 |
| Dynabench | Kiela et al., arXiv 2104.14337（NAACL 2021）| 摘要 |
| BetterBench | Reuel et al., arXiv 2411.12990（NeurIPS 2024 D&B）| 读了 PDF §4 |
| 自监督 SE | arXiv 2402.16321 | 只用了搜索摘要里的一句（V8）|
| codesota | https://www.codesota.com/tasks/speech-enhancement | 二手，只用于 V4、V13 |

---

## 怀疑派复核（2026-10-31）

> 复核方法：逐条回原文。PDF 用 `pdftotext` 抽文本后 grep（SSW9、CHiME-3 CSL 2016 eprints PDF、REVERB summary slides、DNS ICASSP 2021 arXiv 2009.06122、Giraldo IS2025 ISCA PDF）；HTML 用 WebFetch（2406.03460、2405.06573、2406.06185、2402.01413、2506.01611、2309.06183、2411.12990、ears_benchmark README）。没有读 resources 源码。

| # | 说法 | 核实结果 | 依据 |
|---|---|---|---|
| R1 | V3/V5/V6：VB 测试 2 人、SNR 17.5/12.5/7.5/2.5、5 种 DEMAND 噪声、20 个条件；训练 28 人 14 男 14 女 | **成立** | SSW9 §4.1 原文逐字一致；「每人每条件约 20 句」→ 约 800 句，与二手的 824 量级相符（824 仍是【二手】）|
| R2 | V7：句首句尾 > 200 ms 静音被裁 → 测试里几乎没有纯噪声段 | **部分成立，已加注** | 原句在训练集构造段；测试集段没重述，测试集也裁是【推断】|
| R3 | V10 / 结论 1：「VB-DMD 排名搬不到更多样的数据上」 | **数字成立、推广过度，已改正** | Giraldo §4 原句在；但 §2.3 写明无干净参考，用 SQUIM 估计 PESQ/STOI/SI-SDR，且「subjective listening assessment is infeasible」，只有 5 个系统。是机器分排名不迁移，不是人耳 |
| R4 | V12：PESQetarian PESQ 3.82、POLQA 1.46、SI-SDR −19.8、DNSMOS 2.06；10 名专家评为最差；click trick 3.46±0.71，「第四」| **成立** | 2406.03460 表 1、§3.3、§4.1–4.2；原话是「fourth best on the leaderboard … as of June 2024」；带噪输入 PESQ 1.97 / POLQA 3.11 / SI-SDR 8.4 / DNSMOS 2.69 |
| R5 | V13：SEMamba PESQ 3.55，加 PCS 3.69 | **成立** | 2405.06573 摘要；3.55 是非因果配置 |
| R6 | E3/E4/E5/E7：EARS-WHAM SNR −2.5～17.5、训练 32,485 条 86.8 h、测试 6 人 886 条 3.7 h、盲测 743 条 2 h 新录噪声、听测 20 人 × 10 句 | **成立，节号小改** | 2406.06185；划分与条数在 §3 引言段不在 §3.1 |
| R7 | E8：v2（2025-04-25）用 WHAM 官方划分、高通、能量门限；Reverb v2 按数据集划 RIR | **成立** | ears_benchmark README 原文逐字一致；E9「v1 有泄漏」仍是【推断】|
| R8 | S2/S3：CHiME-3 表 3 DNN dev 16.13/14.30、test 33.43/21.51；表 4 MVDR 真实 18.20 / 模拟 10.78 等；原因是纯延时滤波 | **成立，加了适用范围注** | CHiME-3 CSL 2016 表 3、表 4（GMM 后端）、§2.3.2 与 p.13「pure delay filter」；但这是多通道 ASR WER，不是单通道感知质量 |
| R9 | S5/S6：REVERB「Very strong correlation」；整体质量相关 −0.35/0.39/−0.21/0.12/0.28 | **成立，加注** | slides p.28、p.43 逐字一致；p.42 写明听测「Web-based … not well controlled」；负号是 CD/LLR 的正常方向 |
| R10 | S8 / C-G8：DNS 第二届盲测 700 条 = 650 真实 + 50 合成唱歌；最好模型只比带噪高 0.53 DMOS；「97% 真实」| **数字成立；「97%」改正为约 93%** | 2009.06122 §4.1 原句「700 noisy speech clips out of which 650 are real recordings and 50 synthetic noisy singing clips」；§5「only 0.53 DMOS better than the noisy」；650/700 = 92.9% |
| R11 | S9：UDASE 侵入式排名一致、CMGAN DNSMOS 第一 / 人耳最差、PCC ≤ 0.50（BAK 0.73 除外）、32 人 × 128 条 | **成立，已加限定** | 2402.01413 §4、§6.2.2、§6.2.4–6.2.5；人耳里只有 CMGAN-FT；32 人分 4 组 × 8；只有 4 个系统 + 带噪 |
| R12 | S10/S11：URGENT 2024 500+500 盲测、300 条 MOS 子集、KRCC 0.73、高 SNR(>10 dB) 难样本多、约 2000 条负 WADA-SNR、带宽不符 LibriTTS 约 25% / DNS5 与 CommonVoice 约 100%、13 个指标 | **成立** | 2506.01611 §1、§2.1、§2.3、§3.1–3.2 原文一致；本地 README 未复核 |
| R13 | S13：Gonzalez 三重失配 ΔPESQ 落差 −46/−79/−71/−71%、新模型失配下可能输给 FFNN | **成立** | 2309.06183 表 V(a)、§V；「语音失配影响最大」是指单因素失配中 |
| R14 | B1–B3：BetterBench 设计 14 / 实现 11 / 文档 19 / 维护 3 条，共 46 条 | **总数成立，分阶段条数未核** | 摘要与 §3 写「46 criteria」；分项相加是 47，对不上，可能某条计数错，未读到附录 J |

**改动计数**：正文改了 11 处（改正 3 处：C-G8 的 97%、S17 的「模拟」、结论 1 / V10 的推广范围；加限定或降级 8 处）。没改的条目（S1、S4、S7、S12、S14–S16、B4–B7、E1、E2、E6、V1、V2、V8、V9、V11、V14）本轮**未核**，维持原等级。

**这份笔记漏了什么最要紧的**：所有「模拟排名 ≈ 真实排名」的正面证据，系统数都很小（UDASE 4 个、Giraldo 5 个），而且要么是多通道 ASR（CHiME-3、REVERB），要么是机器分（Giraldo）。**没有一条证据是「单通道、因果、低延迟降噪，在真实录音上用人耳排名」**。所以 C-K1 的 τ 阈值没法从文献借（URGENT 的 0.73 是「整体分排名 vs MOS 排名」，不是「模拟 vs 真实」）。另外 5 个系统的 Kendall τ 置信区间极宽（只有 10 对比较），C-K1 要求「≥ 5 个系统」几乎判不出东西，需要更多系统，或者改用逐条成对偏好的一致率。最直接、可补的数据源是 DNS 2020/2021 各队在合成集与真实集上分别的 P.808 分数（表 2 是图片，可以试着从挑战赛官网或 resources/DNS-Challenge 里找原始表），能直接算出「合成 vs 真实」的排名 τ——笔记 §4.2 已经提到拿不到，但没把它标成最高优先级。
