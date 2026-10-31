# 问题 5 · 会议室模式：自动识别「多个近讲说话人轮流说话」

## 一句话结论

不需要也不该上完整的在线 diarization 模型。可行做法是在 0.5 s 判断回路里复用 PSE 线本来就要算的**声纹向量 + 在线聚类**，统计「近讲簇」的个数、轮换次数、重叠比例和电平差，用**不对称滞回**来切：进入「保留所有人」快（约 10–20 s），退回「只留主人」慢（≥ 2 min）。拿不准时就保留人声。业界先例里最可抄的是 Teams：它检测到「旁边有人」后**提示用户**，由用户自己关掉 voice isolation。没有一家是静默自动切换的。

## 证据表

| # | 结论 | 出处 | 标记 |
|---|---|---|---|
| 1 | diart（Coria 等）：用 5 s 滚动缓冲做局部分段，加增量聚类（Hungarian 匹配 + cannot-link），每 500 ms 更新一次；延迟可在 500 ms–5 s 之间调 | arXiv 2109.06483 §2.2–2.3，p.3 | 【论文】 |
| 2 | 同一论文：λ=500 ms 时 DER 分别为 DIHARD III 29.6%、AMI 34.1%、VoxConverse 22.2%；λ=5 s 时为 25.0 / 27.5 / 16.8。延迟缩短时主要涨的是**说话人混淆**，误报和漏检基本不变 | arXiv 2109.06483 Fig.5 与 Table 1，p.4–5 | 【论文】 |
| 3 | 同一论文：开头要先把 5 s 缓冲填满（首次输出前有约 5 s 预热）；聚类越跑越准，约 5 分钟后接近离线上限 | arXiv 2109.06483 §2.3、Fig.6，p.3–5 | 【论文】 |
| 4 | 同一论文：每更新一步，Xeon 6248（20 核）上耗时 165 ms，V100 上 50 ms。**不满足**「1/5 核」预算，不能全套照搬 | arXiv 2109.06483 §4「Real time」，p.5 | 【论文】 |
| 5 | 同一论文：分段模型的「重叠语音检测」仍是瓶颈，换成理想分段后漏检大幅下降 | arXiv 2109.06483 §4，p.5 | 【论文】 |
| 6 | diart 代码为 MIT 许可，作者 Université Paris-Saclay / CNRS | pypi.org/project/diart/0.3.0 | 【官方】 |
| 7 | pyannote/segmentation-3.0：MIT 许可，但下载要在 HF 上同意条款（gated）。输入 10 s、16 kHz 单声道，powerset 共 7 类，每块最多 3 人、每帧最多 2 人重叠 | huggingface.co/pyannote/segmentation-3.0 | 【官方】 |
| 8 | NVIDIA Streaming Sortformer 4spk v2.1：117M 参数，16 kHz，帧分辨率 80 ms，**最多 4 人**，5 人及以上性能下降。低延迟配置为 1.04 s，DER 分别为 DIHARD III ≤4spk 15.09、CALLHOME 2spk 6.65。RTF 0.093 是在 RTX 6000 Ada GPU 上测的。许可为 NVIDIA Open Model License | huggingface.co/nvidia/diar_streaming_sortformer_4spk-v2.1 | 【官方】 |
| 9 | 有第三方页面称 NVIDIA 另有 Nemotron 3 Diarization（最多 8 人，openmdw-1.1）；未核对官方卡 | hackernoon.com/nemotron-3-diarization… | 【传言】 |
| 10 | CountNet：单通道说话人**计数**，5 s 片段、16 kHz；期刊版称「最多可估计 5 人」 | arXiv 1712.04555；TASLP 2019（hal-lirmm-02010805） | 【论文】，误差表未查到 |
| 11 | Teams voice isolation：检测到「Someone nearby」（例如你和同事共用一台设备开会）时弹出提示，用户想让他们也被听到时，可选「Turn off voice isolation」。**是提示用户，不是自动切换** | support.microsoft.com/office/voice-isolation-in-microsoft-teams-calls-and-meetings-a9756ea9-4cec-44c4-aefb-6f5d17c89427（见搜索摘要；本次没能直接抓到原页正文） | 【官方】（转引） |
| 12 | Teams Rooms：会议室设备给每位参会者建声纹做发言归属（需用户注册档案）；超过注册容量的人仍会被区分开，只是标成 Speaker 1/2…；各语言版本的人数上限不一致（10 / 20 / 50） | learn.microsoft.com/microsoftteams/rooms/voice-recognition | 【官方】，数字矛盾 |
| 13 | Teams 的超声啸叫检测：同一房间里多台设备入会时，抑制音频并提示「附近已有设备入会」 | m365admin.handsontek.net（第三方转述 Microsoft 更新） | 【传言】 |
| 14 | Apple 麦克风模式：Voice Isolation「优先你的声音、屏蔽环境声」，Wide Spectrum「不过滤环境声」；Automatic 在听筒通话时用 Voice Isolation，免提时用 Standard。**用户手动选**，没有按人数自动切换 | support.apple.com/101993 | 【官方】 |
| 15 | Wide Spectrum 适合会议室多人共用一部手机 | iphonelife.com（第三方） | 【传言】 |
| 16 | Zoom Rooms 的 Smart Gallery / Intelligent Director 是**视觉**分人（多摄像头，最多 16 人），没查到它在音频侧有多人保留或降噪策略 | news.zoom.com/intelligent-director | 【官方】（音频部分未查到） |
| 17 | Krisp 的背景人声消除按主人音高学习、滤掉其他音高段，官方承认「同音高类的人声偶尔漏过」；未查到它针对多人同屋的官方说明 | krisp.ai/blog/voice-cancellation-windows（2021） | 【官方博客】，多人场景未查到 |
| 18 | 前序调研：Teams 主档案存在云端，「检测到旁边有人说话时提示用户自己关」；「会议开头别人先说话」这件事无人专门处理 | 本仓 `docs/research/005_pse/sources.md:84`、`gap.md:41` | 【本机】 |

## 对本项目的含义

**定位**：这是**场景层**的识别器，挂在 0.5 s 判断回路上，只决定 PSE 门控是开（只留主人）还是关（保留所有近讲人）。M1 主链路（GTCRN）不动，**不增加音频路径延迟**，45 ms 预算不受影响。

**算力**：
- 声纹（ECAPA / CAM++，PSE 线已选）每 0.5 s 算一次，取 1.5–2 s 窗口，本来就要算，增量约为零。
- 在线聚类只是 K ≤ 8 个质心的余弦距离加 Hungarian 匹配，微秒级。
- VAD 和电平统计可直接用 GTCRN 前后的能量。
- **不要**引入 Sortformer（117M，按 GPU 标定，NVIDIA 许可与 GPL 分发的关系需人判断），也**不要**引入 diart 全套（Xeon 上 165 ms/步，偏重；segmentation 是 MIT 但要过 HF 门槛）。如果后续要做重叠检测，pyannote segmentation-3.0（MIT）是唯一许可干净的候选，可以每 2–5 s 跑一次而不是每 0.5 s 跑一次。它的参数量在卡片上未查到，算力需实测。

**许可**：diart（MIT）和 segmentation-3.0（MIT）与 GPL-3.0 兼容【推断】。Sortformer 的 NVIDIA Open Model License 是否允许随 GPL 程序再分发，需要人看条款。

**延迟**：识别本身就慢，这是有意的：要累积若干轮发言（十几秒）才下判断。由于安全方向是「保留人声」，这种慢只会让开头十几秒里，邻座同事的声音按通用降噪处理被保留，而不是被吃掉。

## 给 scene 线的识别建议

**输入特征**（每 0.5 s 更新，滑窗统计 60 s）：
1. `vad`：语音帧占比（通用 VAD 或 GTCRN 掩码能量）。
2. `emb`：语音段的声纹向量（1.5–2 s 窗，仅在 VAD 判为单人、能量稳定时提取）。
3. `cluster_id`：在线聚类结果。距离 > δ_new 时开新簇，簇累计 ≥ 3 s 才算「真说话人」，避免一句咳嗽就成一簇。主人簇由 PSE 注册向量锚定。
4. `near_score`（每簇）：簇内段的平均电平相对主人簇的 dB 差，加上直达混响比的代理量（频谱倾斜 / 高频能量比，越近越亮）【推断，需实测有效性】。
5. `overlap_ratio`：窗口内同时有 ≥ 2 个簇活跃、或单段声纹纯度低（段内两半向量距离大）的比例。
6. `turns`：60 s 内说话人切换次数（A→B 且两段各 ≥ 1 s）。
7. `diffuse`：持续多人重叠的 babble 指纹，包括语音帧占比极高（> 90%）、帧间无静音间隙、4 Hz 调制深度低【推断】。

**判据**（会议室 = 下面全部成立）：
- 非主人的近讲簇 ≥ 1 个（加上主人共 ≥ 2 人），且每簇累计说话 ≥ 5 s；
- `near_score` 相对主人 ≥ −10 dB（电平相近，排除远处的邻桌）；
- `turns` ≥ 3 / 60 s，且 `overlap_ratio` < 20%（轮流说、清晰，而不是一直叠在一起）；
- `diffuse` 不成立（排除 babble：babble 是持续重叠、低电平、簇不纯、没有轮换结构）。

**与 babble 的分界**：babble 的特点是 `overlap_ratio` 高、各簇都弱（< 主人 −10 dB）、簇不稳定（不断开新簇、累计都不到 3 s）、没有静音间隙。会议室则是簇少（2–6 个）而稳定、电平相近、有明显的轮换和停顿。exp003 的「邻桌 6 人 babble」应当落在 babble 一侧，「旁边有人说话」则要看对方是否参与轮换：参与就是同事，不参与、只在背景持续说话就是干扰。

**滞回（不对称）**：
- 进入「保留所有人」：条件连续成立 ≥ 10 s（20 个判断周期中 ≥ 16 个成立）即进入。只要出现一次「非主人近讲、电平 ≥ 主人 −6 dB、且主人在此前后 5 s 内说过话」（即一问一答），就立即临时放宽门控（只改参数），不必等满 10 s。
- 退出（回到只留主人）：非主人近讲簇连续 ≥ 120 s 无活动，并且用户当前不在「会议室」场景时才退出。
- 切换时只改门控强度参数，用 0.5–1 s 的斜坡过渡，不跳变。

**默认**：
- 首次运行、注册未完成时：保留所有人声（通用降噪），不做主人门控。
- 用户层场景选了「会议室 / 多人」：强制保留，识别器只负责提示。
- 用户层选了「只留我」：识别器仍然运行，触发时**提示但不自动改**（尊重显式选择）。

**何时问用户、怎么提示**（照 Teams 先例）：
- 自动进入「保留所有人」时：菜单栏图标变色，加一条非模态通知「检测到身边有人一起说话，已保留所有人声 · [只留我]」。一次会话最多提示 1 次。
- 置信度低（判据成立 50–80%）时不切换，也不提示，保持当前状态；但如果当前是「只留我」，就把门控放松一档（这样判错的代价只是多漏一点噪声）。
- 用户点了「只留我」后，本次会话不再自动切，直到下次启动或场景改变。

**失败时的安全方向**：任何不确定（声纹模型未就绪、聚类 < 2 簇但 VAD 显示有多人、CPU 超预算被降级、主人注册向量不可信）一律回到**通用降噪、保留所有人声**。宁可把一点邻桌人声放进来，也不吃掉在场同事；判据只单向地「放宽」门控，而不会单向收紧。

## gap（需另开实验或查不到）

1. **ECAPA / CAM++ 声纹在 0.5 s 回路里的聚类纯度**：Mac 内置麦、1–3 m 距离、混响会议室，2–6 人时簇数的准确率。需要录真实会议片段来测（建议 AMI 单通道远讲 + 自录）。
2. **`near_score` 区分近讲同事与邻桌**是否可靠（电平差 / 频谱倾斜在 1 m 与 4 m 处的分布），需实测。
3. **exp003 两段素材**在上述判据下会不会误判为会议室：这是 babble 分界的关键回归用例。
4. **DOA**：macOS 是否把 MacBook 内置三麦的原始多通道暴露给第三方（AVAudioEngine / CoreAudio 通常拿到的是波束后的单声道）。未查到官方说明，需在本机用只读枚举确认。如果拿不到，DOA 就不可用，本方案不依赖它。
5. pyannote segmentation-3.0 的参数量与 M 系列单线程耗时：卡片上未查到，需实测。
6. Sortformer 的 NVIDIA Open Model License 与 GPL-3.0 分发是否兼容，需人看条款。
7. Teams「Someone nearby」检测靠什么信号（声学还是超声、本机还是云端）：未查到。
8. Krisp 和 Apple 在多人时是否有自动行为：未查到官方说明。Apple 文档只给了手动模式。
9. 「一问一答立即放宽」规则在主人与对方交替很快（< 1 s 抢话）时的误触发率，需实测。

## 来源清单

- arXiv 2109.06483（Coria, Bredin 等，ASRU 2021）§2.2–2.3、§4、Table 1、Fig.5–6
- https://pypi.org/project/diart/0.3.0/
- https://huggingface.co/pyannote/segmentation-3.0
- https://huggingface.co/nvidia/diar_streaming_sortformer_4spk-v2.1
- https://hackernoon.com/nemotron-3-diarization-heres-what-you-need-to-know （传言）
- arXiv 1712.04555；https://hal-lirmm.ccsd.cnrs.fr/lirmm-02010805
- https://support.microsoft.com/office/voice-isolation-in-microsoft-teams-calls-and-meetings-a9756ea9-4cec-44c4-aefb-6f5d17c89427
- https://learn.microsoft.com/MicrosoftTeams/rooms/voice-recognition
- https://m365admin.handsontek.net/microsoft-teams-ultrasound-howling-detection-in-teams-meetings/ （第三方）
- https://support.apple.com/101993
- https://news.zoom.com/intelligent-director/
- https://krisp.ai/blog/voice-cancellation-windows/
- 本仓 `docs/research/005_pse/sources.md:84`、`gap.md:41`
