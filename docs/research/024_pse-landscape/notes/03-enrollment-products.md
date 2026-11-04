# 笔记 03 · 会中自动注册与商用产品（子代理原始交回，pse 未逐条核；核过的在 sources.md 标出）

> 2026-11-04 · 只读联网检索，未下载、未运行。部分数字只读了摘要或第三方摘要，条目内注明。

**结论**：「会中自动注册 + 声纹条件提取」**产品上已有人公开做成**：Zoom「个性化音频隔离」会中自动生成声纹、存本机；Teams「快速注册」也是会中注册，2026 年已覆盖 Mac。**研究上没找到完整的冷启动方案**（从零开始、不预录、在线锁定并防锁错）。最接近的两类：在线更新注册（前提已有初始注册）；不用声纹直接认主讲人。

## A. 研究

### A1. 无注册 / 自注册的 PSE、TSE
- **EvoTSE（2604.06810，2026-04）**【论文】https://arxiv.org/abs/2604.06810
  - 历史提取结果进记忆库扩充注册；入库门槛：ECAPA 声纹与库内最大余弦 > τ，τ=0.5 最好（τ=0 不过滤，τ=1 退回静态注册）
  - 库容 64，满了踢「与其他条目平均相似度最高」的；检索按说话人相似度与情绪相似度各取前 k=3
  - **初始注册永远保留作锚点**，论文明说为防说话人混淆
  - ESD 测试（WSJ 训练）：SI-SDRi 2.09→10.73 dB；提错人比例（NSR）23.9%→8.1%；增益主要来自少提错人
  - 局限：要有初始注册；无混响、无噪声、单干扰、8 kHz；ESD 只 4 人；无延迟数字；没测错误入库后能否纠正
- **USEF-PNet / DSEF-PNet（2505.12288）**：不用说话人编码器但仍需注册语音；前作 SEF-PNet 2501.11274
- **Unmixing the Crowd（2604.03219，投 SLT 2026）**：从混合音直接预测每人一个声纹，在线记忆做长时重识别，用于无注册 TSE；摘要没说怎么选目标人
- **Enroll-on-Wakeup（2602.15519，Interspeech 2026）**：唤醒词当注册；段短含噪时现有 TSE 明显退化；TTS 扩充注册改善听感，识别仍有差距
- **个性化 VAD 自扩充（2601.12769，ICASSP 2026）**：混合音取关键帧嵌入叠加到短注册，迭代 5 次追平完整注册（摘要，无数字）
- **SE-DiCoW**【第三方综述】：用分段结果在整段对话里找目标人「最活跃、最干净」的一段当注册（离线）
- **NVIDIA 自适应多说话人识别（2506.22646）**【第三方页】：流式识别**倾向锁定离麦最近或最先出现的人**——即「开头锁错人」的已知偏差
- **Sivaraman & Kim 2021（2104.02018）**：只用用户带噪录音自监督个性化，逐帧 SNR 预测给伪干净源加权（数据净化）

### A2. 无注册的主讲人提取（佩戴者 / 本人语音）
- **EEND-SAA（2509.11957）**：无注册、可流式的主讲人 VAD，靠连续性与音量；主讲人分段错误率 6.63%→3.61%（LibriSpeech 模拟）
- **眼镜佩戴者识别（2509.14430）**：多通道波束 + 选麦 + 轻量旁人检测，WER 相对降 18%
- **Jalal 等 Interspeech 2025**：无注册时声纹嵌入被噪声与重叠污染（PDF 未能解析）
- **Target confusion（Zhao 等 2204.01355）**：输出与注册比相似度标出提错人，减法恢复，SI-SDRi +1 dB 以上
- 2025 训练侧：说话人增广 2505.21805；说话人一致性损失（第三方综述页）

### A3. 麦克风物理线索
- 单通道距离 + 房间嵌入 TSE（2505.14433，EUSIPCO 2025）：可学习距离嵌入（对应 DRR）；摘要无数字
- Interspeech 2025 jiang25：DRR 当距离判别特征
- 助听器本人语音检测专利：耳内麦比耳后麦高约 2.5 dB 判本人（US10142745）；自适应滤波跟随佩戴者（US10171922）
- 【推断】Mac 单麦拿不到双麦声级差，只能用能量、DRR、频谱近讲特征

### A4. 在线分段与声纹更新防漂移
- FASTDIAR（2610.02941）：只有与若干帧前嵌入高度相似的稳定帧才更新锚点（无消融）
- EURASIP 2024 轻量实时分段：相似度过阈才线性插值更新质心
- 2402.00067：太短的片段不更新质心
- Streaming Sortformer（Interspeech 2025）：说话人缓存选帧保留

### A5. 声纹漂移数字（条件不同，不可横比）
- 噪声：ECAPA EER 0 dB 3.31%，−20 dB 31.86%（2508.18913）
- 加噪 VoxCeleb：1.19%→3.95%（2311.15627）
- 跨域远场音乐干扰：21.66%→34.62%（第三方摘要，未核）
- 采样率：4 kHz 首位命中 83.4→36.8（2504.18950）
- 跨设备：不匹配错误率 +162%（2007，小语料）；多设备 EER 18.39%→8.84%（领域泛化后，2021）；**无 2025 ECAPA 跨麦数字**
- 情绪（MSP-Podcast，中性注册）：中性 8.95%、愤怒 12.98%、开心 11.63%、悲伤 11.89%（2002.05039）
- 感冒：只有老研究；VoxFlu（2025，约 300 人）结果未公开

## B. 产品（截至 2026-11）

### 要声纹的
- **Zoom 个性化音频隔离**【官方】support.zoom.com KB0074698：「the next time you speak in a meeting, the app will attempt to create an automatic voiceprint」，存本机直到删除；可选朗读 ≥30 s（存 Zoom 侧）；桌面 6.0.0+；更新与防锁错策略未公开。**最直接的同类产品**
- **Teams voice isolation**【官方】learn.microsoft.com/microsoftteams/voice-isolation · MC1197146：要声纹档案、存本机；模型「个性化 DeepVQE」；2026-06 起「快速注册」会中说话即注册（`PassiveVoiceEnrollment`），覆盖 Desktop / Mac；排障建议「安静、好麦、删了重注册」→【推断】漂移靠用户手动兜底。技术来源 pDCCRN 2110.09625 · E3Net 2204.00771

### 不注册、只留主讲人的
- **Krisp BVC**【官方】developers-docs.krisp.ai：「does not require user voice enrollment」；前提嘴离麦 < 50 cm、戴耳麦；窄带不支持；设备谎报采样率会误伤用户
- **ai-coustics Quail Voice Focus / VAD Voice Focus 2.0**【官方】：无注册，认主导说话人，「不以离麦距离为决定因素」；主讲人太小声会被压掉；CPU 实时、延迟 30 ms
- **Webex Optimize for my voice**【官方】：「focuses on the speaker closest to the microphone」
- **Sanas**【官方】：保留约 1 m 内主讲人；桌面本机、SDK 云端

### 只去噪或不明
- Apple Voice Isolation：无注册入口、无个性化说明 →【推断】通用模型
- AirPods：波束 + H2，优先佩戴者，无注册【第三方】
- Google Meet：官方明说「电视里或同时说话的人声不会被滤掉」；VoiceFilter-Lite 用于识别前端，无证据用于通话
- NVIDIA Broadcast · Discord（Krisp）· 三星：无注册式「只留你的声音」

## 防锁错 / 防污染已知做法
1. 初始注册永久作锚点（EvoTSE）　2. 相似度门槛入库 τ=0.5（EvoTSE）　3. 库容上限 + 踢冗余（EvoTSE）　4. 只用稳定帧更新（FASTDIAR）　5. 阈值 + 插值更新质心（EURASIP 2024）　6. 短片段不更新（2402.00067）　7. 选最活跃最干净段注册（SE-DiCoW）　8. 逐帧 SNR 加权（Sivaraman 2021）　9. 输出 vs 注册相似度查提错人（Zhao 2022）　10. 连续性与音量认主讲人（EEND-SAA）　11. 近场线索（Webex / Krisp / Sanas / 2505.14433 / 专利）　12. 已知偏差：流式锁最近或最先出现的人（2506.22646）　13. 人工兜底：删除重注册（Teams / Zoom）　14. 反例：主讲人太小声被压（ai-coustics）

## 检索记录
截至 2026-11-04。关键词：enrollment-free / self-enrollment / evolving enrollment TSE · primary/main speaker extraction without enrollment · wearer speech / own voice detection · online diarization centroid update · DRR near-field single-channel TSE · speaker embedding noise/far-field/cross-device/emotion/cold EER · target confusion · 各产品名 + voice isolation / enrollment / voiceprint。站点：arxiv · isca-archive · Zoom / Microsoft / Krisp / ai-coustics / Webex / Sanas / Apple / Google / NVIDIA / Discord / Samsung 官方页 · USPTO / Google Patents。没查到：VoxFlu 结果、2025 ECAPA 跨麦 EER、Apple 技术细节、Zoom / Teams 锁定更新策略。未做对抗自审。
