# 数据集卡片索引

> 2026-10-31 · 014。每集一张卡，按接近度降序。列定义见 `../must-know.md`：真实录音 / 参考 用「有 · 部分 · 没有」回答；许可结论按 D26（SA / NC → 只评测）；D64（2026-11-01）起只带 SA 的改为可训练，受影响的 5 行已于 2026-11-03 改（023 `cards-014-addendum.md` 页首）。
> 跨路重复的卡已合并进同一文件（`<details>` 里是另一路的版本），冲突裁定见 `../decision.md` 附录。

| 接近度 | 数据集 | 真实录音 | 参考 | 许可 | 结论 | 获取 | 语言 | 大小 |
|---|---|---|---|---|---|---|---|---|
| 3.5 | [CHiME-5 / CHiME-6 晚餐聚会](chime5-chime6.md) | 有 | 有 | CC BY-SA 4.0（2024-01-01 重新发布） | 可训练（D64：SA） | 直接下载 | 英语 | 约 50 h（train 40:33 / dev 4:27 / eval 5:12）；CHiME-6 包 97G+11G |
| 3.5 | [CHiME-9 ECHI（含 CHiME-10 ECHI-2 预览）](chime9-echi.md) | 部分 | 有 | CHiME-9 ECHI Data Use Agreement（自定义，2025-06 版）；ECHI-2 预览为 CHiME-10 ECHI-2 Previe | 只评测 | 签协议 | 英语 | 49 场×36 min ≈ 29.4 h（train 18.0 / dev 6.0 / eval 5.4，eval 尚未 |
| 3.5 | [EasyCom](easycom.md) | 部分 | 有 | CC BY-NC 4.0 | 只评测 | 直接下载 | 英语 | 12 场 ≈ 5 h 18 min（含错误场共约 6.6 h） |
| 3.5 | [CHiME-7 / CHiME-8 DASR（合集）](chime7-chime8-dasr.md) | 有 | 有 | 按子集各自许可（CHiME-6: CC BY-SA 4.0；DiPCo: CDLA-Permissive-1.0；Mixer 6: LDC CHiME-8 Ev | 只评测 | 签协议 | 英语 | CHiME-8 DASR：CHiME-6 约 50 h、DiPCo 5.3 h、Mixer 6 约 21 h（+ tra |
| 3.5 | [DNS Challenge 4（ICASSP 2022）dev/blind 真实测试集（含 PDNS）](dns4-icassp2022-real-testset-pdns.md) | 有 | 部分 | README：code MIT，documentation and other content CC BY 4.0；datasets 'provided und | 未查到 | 直接下载 | 英语 | 非个性化 dev 930 条；个性化 dev 1443 条（10–20 s/条）；dev_testset 包约 1.7  |
| 3.5 | [DNS Challenge 5（ICASSP 2023）Headset/Speakerphone 真实测试集](dns5-icassp2023-real-testset.md) | 有 | 部分 | README：documentation and other content CC BY 4.0，code MIT；datasets 按各源原条款、AS IS； | 未查到 | 直接下载 | 英语 | dev：非个性化 600 条、个性化 1443 条（10 s 测试 + 30 s enrollment）；blind：T |
| 3.5 | [CHiME-3 / CHiME-4 真实录音部分](chime3-chime4-real.md) | 有 | 有 | LDC User Agreement for Non-Members（LDC2017S24）；基线代码 Apache-2.0 | 不可再分发 | LDC付费 | 英语（朗读 WSJ0 句子） | 真实 train 1600 句≈2.90 h、dev 1640 句≈2.74 h、test 1320 句（官方页未给 t |
| 3 | [AliMeeting (M2MeT)](alimeeting.md) | 有 | 有 | CC BY-SA 4.0 | 可训练（D64：SA） | 直接下载 | 普通话 | 118.75 h（Train 104.75 / Eval 4 / Test 10）；Train far 73.24 GB |
| 3 | [DiPCo（Dinner Party Corpus）](dipco.md) | 有 | 有 | CDLA-Permissive-1.0 | 可训练 | 直接下载 | 英语（含非母语者） | 10 场，dev 2:43 h + eval 2:36 h ≈ 5.3 h；DipCo.tgz 13.4 GB |
| 3 | [MISP2021 AVSR / AVWWS（及 MISP2022/2023 复用）](misp2021.md) | 有 | 有 | 未查到（官方页仅写 available under license） | 未查到 | 未查到 | 中文普通话 | 约 141 h 音视频（各文献 106–141 h 不等） |
| 3 | [MISP-Meeting（MISP 2025）](misp-meeting.md) | 有 | 有 | Non-commercial Data License Agreement for MISP-Meeting（自定义） | 只评测 | 直接下载 | 中文普通话 | 163 场 125.15 h（train 118.80 / dev 3.24 / eval 3.11）；train F8 |
| 3 | [DNS Challenge 2（ICASSP 2021）真实测试集 + pDNS 赛道](dns2-icassp2021-real-testset-pdns.md) | 有 | 部分 | 仓库 LICENSE = CC BY 4.0（content）；README：数据集按各源原条款、AS IS；测试集未单列许可 | 未查到 | 直接下载 | 英语 + 非英语（葡、俄、西；声调语言：普通话、粤语、旁遮普 | Track1 dev 400 条真实（300 英语 = DNS1 blind 真实段复用 + 100 非英语）+ 200 |
| 3 | [CHiME-7 UDASE（基于 CHiME-5 真实晚餐录音）](chime7-udase.md) | 有 | 部分 | CHiME-5/6 音频：CC BY-SA 4.0（openslr.org/150 LICENSE.txt，无 NC/仅研究条款，但有 ShareAlike）； | 可训练（D64：SA） | 直接下载 | 英语 | train 27,517 段 83:22 h；dev 0/1/2/3 档共约 17.4 h；eval 0/1/2/3 档 |
| 3 | [MCoRec（CHiME-9 Task 1，多对话鸡尾酒会）](mcorec.md) | 有 | 部分 | MCoRec Data Agreement（自定义；数据所有方 Datoid LLC；仅非商业学术与教育、算法开发与评测；商用需 Datoid 书面许可；严禁再 | 只评测 | 签协议 | 英语 | train 56 段 5.6 h、dev 25 段 2.5 h、eval 69 段 6.9 h（约 15 h，每段约 6 |
| 3 | [ASPIRE（Stirling/Napier 真实食堂 AV 语料）](aspire-stirling.md) | 有 | 部分 | Zenodo 许可字段「Non-Commercial Government Licence」（但该字段链接指向 Apache-2.0 文本，自相矛盾；按严者算非 | 只评测 | 直接下载 | 英语（Grid 句式） | ASPIRE.zip 13.0 GB；论文：10,000 句（真实嘈杂 5,000 + 隔音棚 5,000），5 位说话 |
| 3 | [ACE Challenge corpus](ace-challenge-corpus.md) | 部分 | 部分 | Zenodo 6257551 写 CC BY 4.0；Imperial 官方项目页写 CC BY-ND 4.0（两处冲突，均为【官方】） | 只评测 | 直接下载 | 英语（消声室语音 + 现场 babble：真人朗读 TIMI | Zenodo 全集 24.5 GB；Chromebook 1.1 GB，Mobile 1.6 GB，Lin8Ch 4.2 |
| 3 | [Device Recorded VCTK (DR-VCTK)](dr-vctk.md) | 部分 | 有 | CC BY 4.0（包内 README.txt 原文：“You are free to use this database under Creative Com | 可训练 | 直接下载 | 英语（英式 RP 口音） | DR-VCTK.zip 1.67 GB（16 kHz；训练 28 人、测试 2 人） |
| 2.5 | [NOTSOFAR-1 Recorded Meeting Dataset](notsofar-1.md) | 有 | 部分 | CC BY 4.0（DATA_LICENSE 为标准文本，无附加条款）；Dev-set-2 另有「仅限 NOTSOFAR-1 挑战赛内使用」限制，且已下架 | 可训练 | 注册 | 英语 | 315 场 × 平均约 6 min（会议时长约 31.5 h【推算】）；按设备累加 SC 174.5 h / MC 12 |
| 2.5 | [DNS Challenge 3（Interspeech 2021）真实测试集](dns3-interspeech2021-real-testset.md) | 有 | 没有 | 仓库 LICENSE = CC BY 4.0（content）；README：数据集按各源原条款、AS IS；测试集未单列许可 | 未查到 | 直接下载 | 英语 + 普通话、粤语、旁遮普语、越南语、西/德/葡/法语 | V3 blind 600 条（并入 V2_V3 合集；合集另含 600 条情感片段、100 条鼠标点击片段、V2 bli |
| 2.5 | [REAL-M（真实双人混合分离评测集）](real-m.md) | 有 | 部分 | 未查到（论文与模型卡均未写数据许可；SpeechBrain 模型卡 apache-2.0 只管模型） | 未查到 | 直接下载 | 英语（母语与非母语口音） | 1436 条混合，569 组句对，约 3 h，50 人 |
| 2.5 | [VOiCES（Voices Obscured in Complex Environmental Settings）](voices-corpus.md) | 部分 | 有 | CC BY 4.0 | 可训练 | 直接下载 | 英语 | 1440 h 分段音频（347,688 文件）；VOiCES_release 417.5 GiB，devkit 27.5 |
| 2.5 | [DNS Challenge 真实盲测集（ICASSP 2022 等）](dns-challenge-blind-real.md) | 有 | 部分 | 未查到（README：代码 MIT、文档 CC BY 4.0、各数据集按 Microsoft 取得时的原始条款；盲测集未单列许可） | 未查到 | 直接下载 | 未查到（多为英语【推算】） | ICASSP 2022 盲测集 859 条×10 s（≈2.4 h）；其他年份另计 |
| 2.5 | [BUT ReverbDB (BUT Speech@FIT Reverb Database)](but-reverbdb.md) | 部分 | 有 | CC BY 4.0（NIST SRE10 / HUB5 重录部分仅限持有 LDC 原数据授权的单位） | 可训练 | 直接下载 | 英语（LibriSpeech test-clean 重放） | RIR-Only 8.7 GB；LibriSpeech-Only 117 GB（rel_19_06） |
| 2 | [AISHELL-4](aishell-4.md) | 有 | 部分 | CC BY-SA 4.0 | 可训练（D64：SA） | 直接下载 | 普通话 | 120 h，211 场（train 191 场 107.5 h / eval 20 场 12.72 h）；train_L |
| 2 | [AMI Meeting Corpus](ami.md) | 有 | 有 | CC BY 4.0（Creative Commons Attribution 4.0 International Public License） | 可训练 | 直接下载 | 英语（含大量非母语者） | 约 100 h |
| 2 | [ICSI Meeting Corpus](icsi.md) | 有 | 有 | 爱丁堡托管版：CC BY 4.0（「The ICSI corpus and its annotations are released under the Cre | 可训练 | 直接下载 | 英语（含非母语者） | 75 场，约 72 h（爱丁堡页写「about 70 hours」）；每场最多 16 通道 |
| 2 | [DNS Challenge 1（Interspeech 2020）真实测试集](dns1-interspeech2020-real-testset.md) | 有 | 没有 | 仓库 LICENSE = Creative Commons Attribution 4.0 International（README 写明 documentat | 未查到 | 直接下载 | 英语为主 | dev：4 类各 300 条（合成无混响 / 合成有混响 / MS 内部真实录音 / 手挑 AudioSet 片段）；b |
| 2 | [ICASSP/Interspeech AEC Challenge 真实录音集（近端单讲/双讲）](ms-aec-challenge-real.md) | 有 | 没有 | 代码 MIT；数据 'provided under the original terms that Microsoft received such datase | 未查到 | 直接下载 | 英语 | README 只说来自 'more than 10,000 real audio devices and human s |
| 2 | [URGENT 2024 Challenge blind 测试集（真实录音部分）](urgent2024-blind-real.md) | 有 | 没有 | CC BY-NC-SA 4.0（HF 数据卡 urgent2024_official；另注源数据各有许可） | 只评测 | 直接下载 | 英语 | blind 1000 条 = 500 模拟 + 500 真实，约 2.5 h；全数据卡 3 个 split 共 3000 |
| 2 | [URGENT 2025（Interspeech 2025）blind 测试集（真实录音部分）](urgent2025-blind-real.md) | 有 | 没有 | 测试集本体许可未查到（官网只列训练语料许可）；衍生 urgent2025-sqa 为 CC BY-NC-SA 4.0 | 未查到 | 直接下载 | 5 种训练语言（英、德、西、法、中）+ 1 种未见语言（日语 | blind 900 条（每语言 150 条，真实约 50%） |
| 2 | [MMCSG（Multi-Modal Conversations in Smart Glasses）](mmcsg.md) | 有 | 部分 | Meta「Data License」（官方页称 for research purposes，正文未读到） | 只评测 | 签协议 | 英语【推算】 | 约 26 h：train 8.5 h、dev 8.4 h、eval 9.4 h（CHiME-2024 论文表 1，经检索 |
| 2 | [ARTE（Ambisonic Recordings of Typical Environments）](arte.md) | 有 | 没有 | Zenodo 许可字段「Creative Commons Attribution 4.0 International」；但记录说明另写「仅限非商业个人、教育或研 | 只评测 | 直接下载 | 无目标语音 | 13 个环境；约 9.2 GB |
| 2 | [WHAM! / WHAM!48kHz 噪声](wham-noise.md) | 有 | 没有 | 未查到（wham.whisper.ai 证书错误无法读取；常见说法 CC BY-NC 4.0 二次检索仍未找到原文） | 未查到 | 直接下载 | 无目标语音 | 约 80 h（16 kHz 原版）；48 kHz 版另计 |
| 2 | [RealMAN](realman.md) | 部分 | 部分 | CC BY 4.0 | 可训练 | 直接下载 | 普通话 | 83.7 h 语音 + 144.5 h 噪声；test 39.3 GB |
| 1.5 | [LibriCSS](libricss.md) | 部分 | 有 | 仓库 MIT License（Copyright 2020 Zhuo Chen, Takuya Yoshioka）；源语音 LibriSpeech CC BY  | 可训练 | 直接下载 | 英语（朗读） | 10 h（10 session × 6 个 10 min mini-session） |
| 1.5 | [Mixer 6 Speech（LDC2013S03）](mixer6.md) | 有 | 有 | LDC 用户协议（目录页列为 members-only corpus，许可对象 Subscription & Standard Members 与 Non-Me | 不可再分发 | LDC付费 | 英语（美式母语） | 15,863 h（含 4,410 通电话）；14 路办公室麦 1,425 场 |
| 1.5 | [Clarity CEC3 Task 2（真实助听器佩戴录音）](clarity-cec3-task2.md) | 部分 | 有 | 未查到（数据页与下载页均无许可声明；同届 Task 3 的 Zenodo 记录为 CC BY 4.0，不能套用） | 未查到 | 直接下载 | 英语 | 6000 train + 2500 dev + 1500 eval 场景；clarity_CEC3_data.v1_0. |
| 1.5 | [DAPS（Device and Produced Speech）](daps.md) | 部分 | 有 | CC BY-NC 4.0 | 只评测 | 直接下载 | 英语 | daps.tar.gz 16.1 GB；15 个版本（3 个专业录音版 + 12 个设备×环境组合） |
| 1.5 | [Mozilla Common Voice（Scripted Speech 23.0）](common-voice.md) | 有 | 部分 | CC0-1.0；Mozilla Data Collective 下载条款另加：不得识别说话人身份、不得再托管/再分享（本轮页面为 JS 渲染未能复核） | 可训练 | 注册 | 100+ 语种，含中文与英语 | 英语 23.0：3746 h / 86.83 GB（未复核） |
| 1.5 | [Aachen Impulse Response Database (AIR)](aachen-air.md) | 有 | 部分 | MIT License（官方页称见包内 MIT license 文件；OpenSLR 镜像标“未声明”） | 可训练 | 直接下载 | 不涉及（RIR） | air_database_release_1_4.zip 175 MB |
| 1.5 | [MIRaGe](mirage.md) | 有 | 部分 | 未查到（官方下载页未写任何许可；论文称免费供研究社区使用） | 未查到 | 直接下载 | 不涉及（RIR；babble 回放语种未查到） | 每阵列 zip 约 12 GB，6 阵列 |
| 1.5 | [MIT IR Survey (Traer & McDermott)](mit-ir-survey.md) | 有 | 部分 | 原站未写许可（未查到）；HF 第三方镜像自标 CC BY 4.0（未经原作者确认） | 未查到 | 直接下载 | 不涉及 | 271 条 IR（zip 大小未查到） |
| 1.5 | [VoiceHome-2](voicehome-2.md) | 有 | 部分 | CC BY-NC-SA 4.0（Zenodo：仅非商业科研用途，须引用） | 只评测 | 直接下载 | 法语 | voiceHome-2_corpus_v1.0.zip 2.6 GB；约 5 h |
| 1.5 | [DIRHA English WSJ Audio](dirha-english-wsj.md) | 部分 | 部分 | DIRHA English WSJ Audio Agreement（LDC 自定义协议，非商业/研究用途） | 只评测 | LDC付费 | 美式英语 | 约 85 h（真实 + 仿真） |
| 1.5 | [NISQA_TEST_LIVETALK](nisqa-livetalk.md) | 有 | 没有 | CC BY 4.0（for research purposes） | 只评测 | 直接下载 | 德语 | 232 条 |
| 1 | [URGENT 2026（ICASSP 2026）Track1 blind 测试集（真实录音部分）](urgent2026-track1-blind-real.md) | 有 | 没有 | 未查到（官网、论文未写；HF urgent2026_leaderboard 无数据卡 'No dataset card yet'） | 未查到 | 直接下载 | 未见语言 HI/KR/AR/JP/IT（另含训练语种） | blind 840 条 = 360 模拟 + 480 真实（据论文）；HF 仓库总 3.79 GB |
| 1 | [AISHELL-5（车载）](aishell-5.md) | 有 | 部分 | CC BY-SA 4.0 | 可训练（D64：SA） | 直接下载 | 中文普通话 | 100+ h：train 94 h、Dev 3.3 h、Eval1 3.3 h、Eval2 3.58 h；另约 40 h |
| 1 | [Libri-Adhoc40](libri-adhoc40.md) | 部分 | 有 | GPL-3.0（GitHub 仓库 LICENSE） | 未查到 | 直接下载 | 英语（LibriSpeech） | 每通道 110 h×40 通道≈4510 h；dev/test 各 5 h/通道已放出，train 仍「即将发布」 |
| 1 | [dEchorate](dechorate.md) | 有 | 部分 | CC BY 4.0 | 可训练 | 直接下载 | 不涉及 | dechorate.hdf5 83.9 GB（v1.0，2021-03-22；Zenodo 显示有更新版本） |
| 1 | [EARS-Reverb](ears-reverb.md) | 没有 | 有 | EARS 论文：CC BY-NC 4.0；ears_benchmark README 写 “CC-NC 4.0 International”；上游 RIR 各自 | 只评测 | 直接下载 | 英语 | EARS 原始约 100 h、107 说话人；EARS-Reverb 由脚本生成，大小未查到 |
| 0.5 | [OpenAIR](openair.md) | 有 | 部分 | 逐条 IR 由上传者选 Creative Commons 某一版本或保留所有权利（无统一许可；来自搜索摘录） | 未查到 | 直接下载 | 不涉及 | 未查到 |
