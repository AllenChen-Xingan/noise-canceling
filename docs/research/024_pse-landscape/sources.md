# 出处 · 调研 024 个性化降噪（先认人再提取）

> 2026-11-04 · 手下 pse。取材：5 个只读子代理联网检索（原始交回在 `notes/01`–`05`），pse 本人回源核了下表标「**核**」的条目。全程没下权重、没跑模型。
> 证据等级：**【核】** pse 亲读原文 · **【官方】** 子代理读到的作者 / 发布方原文，pse 未二次核 · **【第三方】** 二手转述 · **【推断】**。
> 所有性能数字都是**作者自测**，除非写明「主办方统一打分」。

## 一、自有积累（本项目已有，不重复查）

| 是什么 | 在哪 | 本线怎么用 |
|---|---|---|
| 005 PSE 调研（2026-10-30） | `docs/research/005_pse/` | 候选口径、TSOS 定义、注册规则 1–7 照用；本线只补 2025-01 以后的 |
| VoxLock 在 M5 上计时 | `docs/experiments/exp003-offline-ab/README.md:137` | int8 单帧 p50 8.36 ms / 8 ms 周期，跟不上实时【本机实测】 |
| 自写注册器 | `docs/experiments/exp005-p2-speaker/README.md:45` | 合成 8 段里 7 段锁对、0 段锁错，锁定前主人累计 20–28 s【本机实测，合成素材】 |
| 竞争说话人数据配方 | `docs/research/015_dfn-hush-synth/decision.md` 第三节 | SIR −6~+20 dB、约 20% ≤ 0 dB、「只有干扰者」段 ≥ 20% 帧 |
| D83 | `docs/plan/001_decisions-20261029-b1.md:594`（在 coordfe 检出） | 放弃门控，改为先认人再提取 |

## 二、开源候选（权重可得）

| 候选 | 关键事实 | 证据等级 · 出处 |
|---|---|---|
| **Alkove Focus**（`alkove-focus-1-preview`） | 约 0.8M 参数、int8 1.1 MB；**从 GTCRN 的 DNS3 检查点改来**（GTCRN 部分 MIT，见 `NOTICE`）；代码与权重 Apache-2.0；256 点（16 ms）步长，输出晚 16 ms；单线程 M1 Max 每秒音频约 56 ms（≈ 每 10 ms 0.56 ms） | **【核】** huggingface.co/alkove-ai/alkove-focus-1-preview · github.com/alkove-ai/alkove |
| 同上 · 注册方式 | 模型本身**不会**自动学声纹：吃一个 192 维声纹（另一个模型 alkove-timbre 从 ≥ 1.5 s 用户语音算出），或给零 = 「留最近的那个人」；运行中可换声纹 | **【核】** 模型卡 · docs.pipecat.ai 的 alkove 页。子代理说的「约 6 s 自动学、下次静音生效」**未证实**，三处原文都没有 |
| 同上 · 效果 | 旁人与目标同响（Common Voice 7 语）：有声纹 SI-SDR 7.1 dB；VoxLock 11.7；无声纹 4.5。旁人残留 有声纹 −22.6 / VoxLock −28.6 / 无声纹 −7.8 dB——**测量条件待核**：子代理笔记写「旁人单独说话」，pse 读到的模型卡写法未明说是否重叠段。AMI 头戴麦 WER 16.1%（原声 20.8%、VoxLock 15.1%，作者称差距不显著）。**只有噪声时**（VoiceBank+DEMAND）STOI 0.895、WER 4.0%，**都比带噪输入（0.921、2.7%）差** | **【核】** 模型卡，作者自测 |
| 同上 · 局限 | 预览版，权重与接口 v1 前可能改；只适合近讲麦；有声纹时新人刚开口约 150 ms 漏过（低 25–35 dB）；训练数据未公开；训练代码未见 | **【核】** 模型卡 |
| **voice-isolator / -cpu**（Scicom，arXiv 2610.35272，2026-10-26） | 因果；CPU 版 2210 万参数；单核占用 10 ms 帧 54%（fp）/ 34%（int8），同机 GTCRN 5%、VoxLock 264%（机器未写）；**会中自动注册**：第 3 s 起取「到目前为止最响的 3 s」，卡上写旁人更响（电视、音乐）会被选中；许可「other」无条款 | **【核】** 论文摘要 · HF 模型卡 |
| 同上 · 效果 | CPU 卡：AMI 60 s 窗 WER 15.55%（VoxLock 23.66%）；NOTSOFAR 7.58%（8.84%）；只有来电者一人 5.32%（VoxLock 2.92%，更差）。论文大模型 AMI 60 s 窗 16.13% | **【核】** 作者自测 |
| **VoxLock** | 005 已列；新增：voice-isolator 作者同机测其占单核 65–264%；本机 M5 **int8 单线程**跟不上，fp32 加缓冲、多线程、Core ML 未测（exp003 README:140） | 【官方】+【本机】 |
| REAL-TSE 因果基线 | BSRNN，窗 32 / 跳 8 ms、算法延迟 24 ms；仓库无 LICENSE、权重许可未证实 | 【官方】arXiv 2607.15198 |
| WeSep 新版 | 因果 BSRNN；Libri2Mix-100 上「SI-SDRi > 1 dB」占比：说话人嵌入条件 79.9%，USEF 等约 95–96% | 【官方】arXiv 2607.27436 |
| AFW | 2.9M 因果 TCN；旁人响 10 / 15 dB 时 SI-SNRi 13.6 / 14.0；提错人 4%；每块 250 ms（延迟超线）；仿真 | 【官方】arXiv 2609.20463 |

## 三、论文（无开源权重，数字可参考）

| 是什么 | 关键事实 | 证据等级 · 出处 |
|---|---|---|
| **AV-PVQE**（微软 Cutler 组） | 起点的纯音频个性化模型（PVQE）「confuses the target in 46% of two-speaker mixtures despite clean enrollment」；加嘴部视频降到 1.6%，算法延迟 20 ms | **【核】** arXiv 2609.30631 摘要 |
| 同上 · 按响度拆 | 目标更响 98.63% 选对 · 同响 61.08% · **目标更小 6.25%**；SIR −10~+10 dB；>1 s 长误删事件 142 → 7 | 【官方】2609.30631v1 正文（子代理读） |
| **CARTSE**（REAL-TSE 在线赛道第一，CyberAgent） | 因果 BSRNN，实测有效延迟约 22.9 ms；仿真预训练 → 真实会议录音 + 非因果教师伪标签微调；**训练时 35% 概率给错的注册（distractor enrolment），合计 38.25% 样本目标不在场**；报告内 TER 0.699、SIM 0.504 | **【核】** real-tse.github.io/assets/pdf/CARTSE-Track1.pdf |
| REAL-TSE 挑战赛 | 只用合成数据训练的基线是「下界」；前三名都用了真实录音；评测集 REAL-T 有注册音频、**无干净参考**，CC BY-SA 4.0 | 【官方】arXiv 2607.15198 · HF REAL-TSE/REAL-T（主办方统一打分） |
| REAL-T 论文 | 同一说话人换注册片段，TER 从 44% 到 130%；VoxCeleb1 训练的 BSRNN 在真实集上显著好于 Libri2Mix 训练的 | 【官方】Interspeech 2025 li25da |
| TF-MLPNet | 50.9 万参数、因果、算法延迟 10 ms、一层 FiLM 注入 d-vector；LibriSpeech 两人混合 SI-SDR 12.37 dB（pDCCRN 321.8 万 10.71）；无代码 | 【官方】arXiv 2508.03047 |
| PVQE-S | 107 万参数；i7 单线程每帧 0.135 ms；模型内部表示当声纹 | 【官方】arXiv 2406.09928（005 已列） |
| Sivaraman & Kim | 同架构 13.9 万 / 22.4 万 / 43.8 万 / 103.7 万参数，个性化增益 +1.0 / +0.2 / +0.3 / −0.2 dB SDR：**越小越受益** | 【官方】arXiv 2104.02017 |
| Personal VAD 2.0 · HyWA | FiLM 不差于拼接（PVAD 2.0 WER 11.3 vs 15.3；HyWA 8.5 万参数 mAP 拼接 89.7 · FiLM 89.7 · 超网络 91.6） | 【官方】2204.03793 · 2510.12947 |
| 2601.16235（Serre 等） | 15 万参数 TinyECAPA 在线修正声纹，多 44 MMAC/s（比 GTCRN 全模型还大） | 【官方】 |
| EvoTSE | 记忆库更新注册，初始注册永远作锚点；跨域提错人 23.9% → 8.1%（v1）/ 6.4%（v2） | 【官方】arXiv 2604.06810 |
| SpkAugTSE | 重采样造伪说话人，提错人比例 SpEx+ WSJ0-2Mix 2.35% → 1.10% | 【官方】APSIPA 2025 P041 |
| Libri2Vox | 目标 LibriTTS 1151 人、干扰 VoxCeleb2 5900 人；DNS 真实录音 DNSMOS 增益 −0.666 → +0.139 | 【官方】arXiv 2412.12512 |
| PS4 | 真实会议训练集按分段切 71,771 条，代理损失弱监督，TER 0.83 → 0.64 | 【官方】arXiv 2607.08111 |
| 2406.04269 | SE 规模化：0.5M–83.6M 参数效果上升后变平；数据多样性比总量重要 | 【官方】 |
| BSRNN PSE 2212.00406 | 主观 OVRL 升、识别准确率大降 → pDNSMOS 抓不住误删 | 【官方】 |
| URGENT 2024 总结 | 只看无参考指标会误导 | 【官方】arXiv 2506.01611 |
| URGENT 2026 · DNS | URGENT 2026 无个性化赛道；DNS 2023 后无个性化续作 | 【官方】arXiv 2601.13531 |

## 四、产品

| 产品 | 关键事实 | 证据等级 |
|---|---|---|
| **Zoom 个性化音频隔离** | 「Zoom will collect an automatic voiceprint from meetings that the user joins」，存本机直到删除；可选录 ≥ 30 s，存 Zoom 侧；锁定与防锁错策略未公开 | **【核】** support.zoom.com KB0074698 |
| **Teams voice isolation** | 要声纹档案；2026-06 起「快速注册」会中说话即注册，覆盖 Mac；底层是个性化 DeepVQE | 【官方】learn.microsoft.com · MC1197146 |
| Krisp BVC · ai-coustics · Webex · Sanas | 不注册，留最近 / 最响的主讲人；Krisp 要求嘴离麦 < 50 cm；ai-coustics 承认主讲人太小声会被压 | 【官方】各家文档 |
| Apple Voice Isolation · Meet · NVIDIA · Discord · 三星 | 无注册式「只留你的声音」；Meet 明说同时说话的人声不滤 | 【官方】，Apple 内部做法【推断】为通用模型 |

## 五、训练与评测语料许可（子代理查，pse 未二次核）

| 语料 | 许可 | 训练可用？【推断】 |
|---|---|---|
| LibriSpeech · LibriTTS-R · MLS · AMI · NOTSOFAR-1 · Emilia-YODAS | CC BY 4.0 | 可（署名） |
| Common Voice | CC0 | 可 |
| VoxCeleb1/2 | CC BY 4.0（另有同条件再分发条款，部分目录标 BY-SA，**冲突待核**） | 待核 |
| AliMeeting · REAL-T | CC BY-SA 4.0 | 可（D64：权重按 CC BY-SA 4.0 发布并署名）；REAL-T 留作评测 |
| CHiME-6 | OpenSLR 标 BY-SA，Sheffield 分商用 / 非商用，**冲突** | 先只评测 |
| Emilia（主体）· MISP · WSJ0 | NC / LDC | 不可 |

## 六、「有没有人做过」的检索范围

- **截至**：2026-11-04
- **关键词（英）**：personalized speech enhancement 2025/2026 · speaker-conditioned SE · personalized noise suppression · pDNS · target speaker extraction causal / streaming / real-time / low-latency / on-device / lightweight · enrollment-free / zero-enrollment / self-enrollment / passive / implicit enrollment · primary / main speaker extraction · own voice / wearer speech · personal VAD · voice isolation · target / speaker confusion · TSOS over-suppression · TSE scaling · REAL-TSE · URGENT · DNS personalized · CHiME-8/9 · GTCRN personalized · FiLM vs concat
- **关键词（中）**：个性化语音增强 · 目标说话人增强 · 目标说话人提取 实时 / 流式 / 因果 · 轻量 个性化语音增强 · GTCRN 个性化 声纹（中文几乎无直接结果）
- **站点**：arXiv（abs / html）· ar5iv · ISCA Archive（Interspeech 2024/2025）· ICASSP 2026 程序页 · APSIPA 2025 · SLT 2026 / real-tse.github.io · Hugging Face · GitHub · PyPI · 各产品官方支持页 · USPTO / Google Patents
- **没覆盖**：没逐篇翻 Interspeech 2025 与 ICASSP 2026 全部录用列表；SSRN、PubMed 被拦；部分 PDF 无法解析
