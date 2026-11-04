# 笔记 04 · 为什么难 · 训练数据 · 评测集（子代理原始交回，pse 未逐条核；核过的在 sources.md 标出）

> 2026-11-04 · 只读联网；未下载权重或数据集、未跑模型。【官方】作者或发布方原文 ·【第三方】二手 ·【推断】。

## 1. 小模型在同时说话时分不干净
- 机理：同性别、声音相近时音高与频谱线索接近（USEF-TSE 2409.02615 同性别 SDR / PESQ 更低）；负 SIR 时条件模型跟住响的人（见 3）；因果小模型只能逐帧调制特征【推断】
- **TSE 专门的「参数量 vs SI-SDRi」曲线：没找到**
- 最接近：SE 规模化研究 2406.04269（Interspeech 2024）：BSRNN 0.5M–83.6M，效果随复杂度上升、高处变平；9 h 数据出现 double descent；数据 8.8→156.8 h 单调变好；多加约 900 h DNS-2020 反而变差（比例失衡，多样性比总量重要）；真实录音（CHiME-4、REVERB）上大模型识别仍低于不处理
- 分离旧证据：参数翻倍 SI-SNRi 只多 0.6–1 dB；动态混合让 SepFormer 26M 不变 20.4→22.3 dB【第三方】
- SOI 2410.03813：每 1 MMAC/s 约 +0.001 dB SI-SNRi
- TargetVoice（Interspeech 2025）：作者明说开源 TSE 在同性别干扰下吃力；LiMuSE 0.48 MB GRID 17.25 dB（2111.04063）
- 未解决：无公开「参数量 × SIR 分桶 × 同异性别」曲线

## 2. 个性化后误删用户（TSOS）
- 机理：声纹统计波动，因果模型「一时认不出」整段删（2110.09625）；状态与注册不同（情绪、麦、房间）时多发（PVQE Interspeech 2024）；多删与漏进旁人此消彼长（2211.02944）
- TSOS：逐帧被删目标幅度和 > γ × 该帧目标幅度和记 1；个性化模型 TSOS 高于不带声纹的 DCCRN，长音频更重；非对称损失（VoiceFilter-Lite）显著改善；接 ASR 多任务更好，两者结合最好
- PVQE：过度抑制内容减少 2.7 倍，近端过度抑制有所增加（检索摘要）
- **AV-PVQE（2609.30631）**：6000 条只有目标的 LRS3；>1 s 长 TSOS 事件 PVQE 142 次 → 加视觉 7 次；12.79M 参数，算法 + 缓冲 20 ms
- DNS 2023（2303.11510）：多数模型 SIG 下降，原因之一是「个性化模型压掉了主说话人」
- 2025–2026 纯音频抗过度抑制新损失：没找到带 TSOS 数字的
- 未解决：γ 各家不统一；纯音频因果模型长时误删无公认解法

## 3. 目标混淆（提成了别人）
- 机理：嵌入不够可分时整段跟错人，效果长尾，SI-SDRi 明显为负的基本就是混淆（2204.01355）
- 指标 NSR = SI-SDRi < 0 的样本占比
- SpkAugTSE（APSIPA 2025）：SpEx+ WSJ0-2Mix 2.35→1.10%；Libri2Mix clean 4.26→3.98%；noisy 4.85→3.81%；DPRNN WSJ0-2Mix 3.78→1.42%（重采样伪说话人）
- EvoTSE（2604.06810v2）：WSJ 训、ESD 测，USEF NSR 23.9% → 6.4%（v2 数字；笔记 03 引 v1 为 8.1%）；不混淆的样本 SI-SDRi 基本不变
- DCF-Net（2502.08191）：错误提取率 0.4%，未给基线
- **PVQE 在合成双人 LRS3 上 46% 输出更像干扰者**；按响度：目标更响 98.63% 选对 · 目标更小 **6.25%** · 等响 61.08%；SIR −10~+10 dB（2609.30631v1）→ **负 SIR 是混淆主因**
- 目标不在场：只用在场样本训练会输出噪声；加不在场样本改善但伤在场效果（Interspeech 2023 zhang23k）；TSEV（Springer 2025）加验证分支
- 检测：有参考用 NSR；无参考用输出 vs 注册的说话人相似度（REAL-TSE 官方指标 SpkSim）
- 未解决：没找到实时混淆检测并回退的论文；负 SIR 混淆纯音频无解

## 4. 训练数据
- LibriMix / WSJ0-2mix 几乎全程重叠、SIR 窄，跨集明显掉点（2005.11262）；真实会议多数少重叠、3–4 人为主（REAL-T，Interspeech 2025 li25da）；REAL-T PRIMARY 156 min，重叠 70 min，单条平均重叠比 0.49
- **说话人多样性比架构重要**：VoxCeleb1 训练的 BSRNN 在 Libri2Mix 上相当、REAL-T 上显著更好；USEF-TFGridNet 合成强、REAL-T 明显差（过拟合 Libri2Mix-100）
- Libri2Vox（2412.12512，APSIPA Trans. 2025）：目标 LibriTTS 1151 人、干扰 VoxCeleb2 5900 人、约 250 h；DNS 真实录音 DNSMOS 增益 Libri2Talker −0.666 → Libri2Vox −0.081 → 加噪增广 +0.139；合成说话人只在课程后期少量加入，合成:真实约 0.2–0.5 最好，纯合成 7.17 dB
- 负 SIR：没找到「训练 SIR 分布 vs 效果」消融；【推断】应覆盖 −10 dB 以下，并加入「目标不在场」「只有目标」样本（DNS5 个性化合成三种场景）
- PS4（2607.08111）：AISHELL-4 / AliMeeting / AMI / CHiME-6 训练集按分段切 71,771 条，4 个代理损失（Whisper CE、说话人相似度、逐帧 VAD、DNSMOS）弱监督微调；TER 0.83→0.64，SpkSim 0.42→0.57
- 会话时间结构合成：2025 后没找到专为 TSE/PSE 的；全双工对话可借（2512.21706、2602.11065 ConversationGoT-120h）；「ConvoTalk」未查到

| 语料 | 许可 | 标记 |
|---|---|---|
| LibriSpeech | CC BY 4.0 | 本轮未复核 |
| LibriTTS-R | CC BY 4.0 | 【官方】openslr 141 |
| MLS | CC BY 4.0 | 【官方】 |
| VCTK | ODC-By 1.0（DNS 仓所列；常见说法 CC BY 4.0） | 【官方】 |
| Common Voice | CC0 | 【第三方】 |
| AMI | CC BY 4.0 | 【官方】 |
| Emilia（约 101k h） | **CC BY-NC 4.0** | 【官方】 |
| Emilia-YODAS（约 114k h） | CC BY 4.0 | 【官方】 |
| VoxCeleb | CC BY 4.0，另有「再分发同条件」条款，部分目录标 BY-SA | 【第三方】 |
| DNS5 数据 | 各子集原许可（DEMAND CC BY-SA 3.0 等） | 【官方】 |
| WSJ0 / wsj0-2mix | LDC 非会员不得商用 | 【官方】 |

- 【推断】凡 wsj0-2mix 训练的公开权重，商用有风险
- 未解决：缺许可干净、带真实会话时间结构、带干净目标参考的大规模训练集

## 5. 评测
| 测试集 | 注册音频 | 干净参考 | 许可 |
|---|---|---|---|
| REAL-T / REAL-TSE（AISHELL-4、AliMeeting、AMI、CHiME-6、DipCo、EVAL-2） | 有，每目标约 5 段以上、平均约 9–11 s | 无 | CC BY-SA 4.0 |
| DNS5 个性化盲测 | 每条 30 s | 无 | 原条款 |
| AMI | 可切 | 无（近讲耳麦作伪参考，有串音） | CC BY 4.0 |
| AliMeeting | 可切 | 无（近讲） | CC BY-SA 4.0 |
| CHiME-6 | 可切 | 无 | **冲突**：OpenSLR CC BY-SA 4.0 vs Sheffield 分商用 / 非商用 |
| DipCo | 可切 | 无 | CDLA-Permissive（版本两说） |
| NOTSOFAR-1 | 可切 | 无（close_talk 标签未核） | CC BY 4.0 |
| MISP | — | — | 非商用 |

- REAL-T 指标：TER、SpkSim（WeSpeaker）、DNSMOS P.808、目标活动 F1；v1.0.1（2026-10-30）勘误，与挑战期结果不可直比
- **没有任何公开集同时具备「真实会议 + 同时说话 + 干净目标参考」**——录音物理限制，REAL-T 原话真实录音里干净参考「不存在」
- REAL-T：同一说话人，最好一段注册 TER 44%、最差 130%；晚宴场景最难
- 无参考指标：DNSMOS P.835 与人评相关 SIG 0.94、BAK / OVRL 0.98（一般降噪，2110.01763）；pDNSMOS 无专门验证；反例 BSRNN PSE（2212.00406）主观 OVRL 显著升、识别准确率大降 → pDNSMOS 抓不住过度抑制；URGENT 2024（2506.01611）UTMOS、SCOREQ 相关最高，DNSMOS 排序相关偏低，只用无参考指标会误导；URGENT 2025（2507.11306）生成式要加音素保真
- WER 坑（DNS 2023）：WER 高常因旁人漏进被识别成插入；相似片段人工转写听 10 遍以上

## 6. sim-to-real
- REAL-TSE EVAL-2：只用 Libri2Mix-100 训的四个 BSRNN 基线 TER 0.81–0.84、SpkSim 0.36–0.39（「下界」）；18 队 24 系统，最优靠伪标签与真实数据自训练（2607.15198）
- REAL-T PRIMARY：BSRNN 平均 TER 中 57.6%、英 69.6%；BASE 子集 110% / 124%
- PS4 真实数据弱监督挽回约 0.2 TER
- 多麦阵列：真实比仿真低 2.74–2.96 dB SI-SDRi，仅 8.40 dB（2611.00894）
- Libri2Vox 训的 Conformer 到 Libri2Talker 只有 −0.16 dB iSDR
- MERL 挑战报告（2607.09043）：主要差距来自仿真训练与真实评测的域差

## 7. 为何常不放权重
- 找不到「因隐私或声音克隆不放权重」的明确声明
- 事实：企业论文（微软 PSE 系列、PVQE、AV-PVQE）正文不说明是否发布；挑战赛不要求开源（REAL-TSE、DNS 2023）；数据许可限制（WSJ0、Emilia、MISP、CHiME-5 非商用档）；REAL-T 作者明说从会议语料构建 TSE 数据集「受数据许可限制」
- 反例：ClearerVoice-Studio（SpEx+ 等）、WeSep（ModelScope）、REAL-T 基线（许可未核）
- 【推断】①训练数据含 NC / LDC，再分发说不清 ②企业产品模型是商业资产 ③声纹条件模型可被用于定向监听 ④挑战赛不强制开源

## 检索记录
截至 2026-11-04。关键词：TSE scaling / model size SI-SDRi · PSE over-suppression TSOS 2025 · target confusion NSR · target speaker absent · REAL-T / REAL-TSE · sim-to-real TSE · pDNSMOS reliability · URGENT non-intrusive metrics · Libri2Vox · 各语料 license · PSE weights release。站点：arxiv · isca-archive · huggingface · github（DNS-Challenge、ClearerVoice-Studio）· openslr · apsipa · emerald · groups.inf.ed.ac.uk · mm.kaist.ac.kr · mispchallenge。待人核实冲突：CHiME-6 许可、VoxCeleb BY 还是 BY-SA、DipCo CDLA 版本。
