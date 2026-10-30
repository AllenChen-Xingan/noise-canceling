# 两类来源 · 个性化降噪（PSE）：模型 · 自动注册 · 评测

> 2026-10-30 · 手下 pse。取材：workflow `wf_7240073b-611`（8 个核查代理用同一张核查表，见 `assign.md`；4 个怀疑派回源反驳；评测 1；完整性批评 1；全程只读，没下权重、没跑推理）。
> 只登记**经怀疑派复核后站得住**的结论；被推翻的已改，证据不足的标「未证实」。VoxLock 与两条声纹许可证由 pse 本人又回源核了一遍。
> 标记：**【官方】** 官方明载 · **【本机】** 本机读源码 · **【推断】** 推断 · **【第三方】** 非官方仓库或转述。

---

## 一、自有积累

| 是什么 | 在哪 | 还算数吗 | 判据 |
|---|---|---|---|
| 设计稿 §10「零录制个性化」【你定的】 | `vault/raw/20261029-voice-denoise-design.md:422-472` | **部分算数** | 「冷启动用通用降噪」「只在确信只有一人说话时注册」「保留初始声纹作锚点」「一键重置」都有外部证据支持（见下）；「三星方法模型减半」「WeSep 带因果预训练、可导出 ONNX」被推翻 |
| 设计稿 §9「声纹在另一个线程，每 0.5–1 s 更新」 | 同上 `:250` | **部分算数** | 对「打分门控」成立；对端到端 PSE 不成立——那类模型用固定的注册条件，不需要周期更新（见 gap.md） |
| 002 延迟口径与 exp001 每帧耗时 | `docs/research/002_model-baseline/decision.md`「延迟」一节 | 算数 | 本线照搬 ①②③ 口径 |
| 004「清楚 · babble」一行与声纹相似度缺口 | `docs/research/004_eval-toolchain/decision.md:20` · `gap.md:128` | 算数 | 本线补上量法（decision.md「评测」） |

---

## 二、他人积累

### 登记总表

| 候选 | 论文 | 官方仓库 / 本地 | 代码许可 | 权重 | 权重许可 / 训练数据 | 角色 | 活着吗 |
|---|---|---|---|---|---|---|---|
| **VoxLock**（因果 BSRNN 目标说话人提取） | 无论文，作者博文 | github.com/arbazsiddiqui/voxlock · `resources/voxlock` | Apache-2.0 | HF `arbazsiddiqui/voxlock`（ONNX：enroll + chunk，int8/int8-full/fp16/fp32，47–94 MB） | Apache-2.0，**依据是 REAL-TSE 基线作者的私下确认**（`NOTICE:4-7`【本机】）；微调数据 LibriSpeech + AMI（CC BY 4.0）；基线训练数据未知 | 要比的（唯一现成的） | 2026-10-11 建、10-28 仍在推，单作者 |
| REAL-TSE 因果基线（WeSep 系） | — | github.com/REAL-TSE/wesep-real-tse | 源文件头 Apache-2.0 | Google Drive `spk_emb_causal_100`、`tfmap_context_causal_100` | **仓库无 LICENSE**（GitHub API license=null）【官方】 | 只借鉴 | 挑战赛基线 |
| **PVQE**（Pärnamaa & Saabas，个性化 DeepVQE，用模型内部表示当声纹） | arXiv 2406.09928（Interspeech 2024） | **没找到官方仓库** | — | 无 | 训练用 VoxCeleb2 | 要学的 | — |
| pDCCRN / pDCATTUNET / E3Net（微软，提出 TSOS） | arXiv 2110.09625 · 2204.00771 | **没找到官方仓库**；第三方 fnusid/pDCCRN 无权重 | — | 无 | — | 要学的（指标 + 非对称损失） | — |
| 三星 Cross-Seq+（cross-attention 动态挑帧） | Interspeech 2023（arXiv 2211.04346） | github.com/shucongzhang/CrossAttnPse **只有 README** | — | 无 | — | 只借鉴 | 停在 2023-06 |
| USEF-PNet / DSEF-PNet | arXiv 2505.12288 | 论文所列仓库 **404**；前身 SEF-PNet `resources/SEF-PNet`（MIT，无权重） | MIT | 无 | — | 不适合 | — |
| USEF-TSE | arXiv 2409.02615 | `resources/USEF-TSE` | **CC BY-NC 4.0** | HF ZBang/USEF-TSE | 无许可声明；WSJ0（LDC） | 不适合 | — |
| TEA-PSE 1/2/3 · NAPSE（DNS 个性化赛道前列） | arXiv 2303.07704 · 2303.06811 | **没找到官方仓库** | — | 无 | — | 要学的 | — |
| pDeepFilterNet2（Orosound） | arXiv 2404.08022 | **没找到官方代码**，只有演示页 | — | 无 | ECAPA（VoxCeleb2） | 要学的 | — |
| PSE-NoEnroll（pDFN2 训练脚手架） | 无 | github.com/HaoranWeiUTD/PSE-NoEnroll | MIT | 无 | — | 只借鉴训练脚手架 | 2025-04 |
| Hush（DFN3 改，「只留最响的人」，无注册） | 无 | github.com/pulp-vision/Hush · `resources/Hush` | Apache-2.0 | HF weya-ai/hush | Apache-2.0；训练数据含 **ESC-50（CC BY-NC）**、DNS | 只借鉴 | 2026-07 |
| VoiceFilter-Lite | arXiv 2009.04323 | **没找到官方仓库** | — | — | — | 要学的（非对称损失、重叠时加大抑制） | — |
| open-voicefilter-lite + livekit 插件 | 无 | github.com/linsanzhu/*（**第三方，不是 LiveKit 官方**） | Apache-2.0 | 仓内 ONNX | 声纹权重来自 maum-ai，**无单独许可**，VoxCeleb2 训练 | 只当接口样例 | 2026-09 建、10-10 后无提交 |
| Personal VAD 1.0 / 2.0 | arXiv 1908.04284 · 2204.03793 | 没找到 Google 官方仓库；论文合著者个人复现 github.com/wq2012/personal_vad（「not officially supported Google product」） | Apache-2.0 | HF wq2012（tflite + safetensors，可自转 ONNX） | MLS / LibriSpeech（CC BY） | 要学的（门控信号） | 2026-10 |
| WeSep | arXiv 2607.27436 | `resources/wesep` | Apache-2.0；**`usef.py` 是 CC BY-NC**（`THIRD_PARTY_NOTICES.md:31-39`） | v0.1 无官方权重 | — | 只借鉴 | 研究预览 |
| SpeakerBeam（BUT） | — | `resources/speakerbeam` | **仅评估许可，禁止修改与再分发**（`LICENSE.txt` 4(b)） | 8 kHz 非因果 | — | 不适合 | 停更 2021 |
| SpEx / SpEx+ · ClearerVoice TSE · Look Once to Hear · SpeakerBeam-SS · ID-PSE · D-LGTSE · G-MaP-SE | 见 gap.md | — | — | — | 非因果 / 8 kHz / NC / 无许可 / 无权重 | 不适合 | — |
| **WeSpeaker**（声纹零件） | arXiv 2210.17016 | `resources/wespeaker` | Apache-2.0 | 官方 ONNX（wenet.org.cn） | **「跟随数据集许可」，VoxCeleb → CC BY 4.0**（`docs/pretrained.md:19-23`【本机】）；VoxBlink2 版 **CC BY-NC-SA**，排除 | 零件：**用** | 2026-09 |
| **3D-Speaker**（CAM++ / ERes2Net） | arXiv 2303.00332 | `resources/3D-Speaker` | Apache-2.0 | ModelScope `iic/speech_*` | Apache-2.0，**维护者 2026-10-19 在 issue #268 书面确认**可转 ONNX、可商用分发（针对 `eres2net_base_sv_zh-cn_3dspeaker_16k`）【官方】；common 版训练数据「约 20 万说话人中文数据」**来源未披露** | 零件：并列备选 | 2025-12 |
| SpeechBrain ECAPA | arXiv 2005.07143 | HF speechbrain/spkrec-ecapa-voxceleb | Apache-2.0 | 无官方 ONNX | VoxCeleb1+2 | 只借鉴 | — |
| NeMo TitaNet-small · Resemblyzer | — | — | — | — | 训练含 LDC（Fisher/Switchboard）/ 未披露；small 版许可没找到 | 不适合 | — |
| diart（在线说话人分段） | JOSS | `resources/diart` | MIT | pyannote 分段模型（DIHARD、Ego4D 等训练） | 分段模型训练数据条款严 | 只借鉴（在线聚类做法） | 停滞 2025-02 |
| 家用说话人识别（被动注册 + 在线更新） | arXiv 2205.00288 | github.com/underdogliu/household-speaker-recognition（未克隆） | — | — | — | 要学的（更新规则） | — |
| Microsoft Teams 被动注册 · Apple 隐式注册 | 产品文档 | 闭源 | — | — | — | 要学的（流程） | — |
| DNS Challenge 个性化赛道 | ICASSP 2022/2023 | `resources/DNS-Challenge` | 代码 MIT，数据 CC BY 4.0（`LICENSE`、`LICENSE-CODE`） | pDNSMOS ONNX | — | 评测零件 | — |

### 关键事实（逐条带出处）

#### VoxLock（唯一同时有因果、开源权重、自动注册、真实录音证据的）
- **延迟**：16 kHz，NFFT 512（32 ms）、HOP 128（8 ms）（`voxlock/core.py:11`【本机】）。官方「24 ms 算法延迟 + 块长」（`README.md:147`）是「窗 − 步长 + 前瞻」口径；按本项目 ① 窗 + 前瞻 = **32 ms**，③ 约 **38–42 ms + 块长**【推断】
- **算力**：27M 参数；Xeon 2.2 GHz 单线程 int8：20 ms 块 RTF 1.10（跟不上）、40 ms 0.77、80 ms 0.61（`README.md:137-145`【官方】）。折成每 10 ms 音频约 6–11 ms，**是 002 预算 2 ms 的 3–5 倍**；Apple Silicon 没有实测【推断】
- 单线程只有块 ≥ 40 ms 才跟得上（`README.md:96`），块长计入延迟 → 单线程下 ③ 约 **78–82 ms，超过 75 ms 入围线**【推断】
- **ONNX**：enroll 跑一次、chunk 每块跑、LSTM 状态显式（`README.md:162`）
- **自动注册**：`live()` 先**原样直通**，听满 3 s 语音后取作样本，中位 3.6 s 切换（`README.md:86`）；**不判断是谁在说**——默认谁先说话 3 s 就锁谁【本机】；文件模式取「前 15 s 最响的 3 s」（`core.py:16-26`）。注册条件是 3 s 语音算出的 k/v 序列，不是单个向量
- **效果**：Krisp 基准 265 条真实录音 WER 4.95%（未处理 23.21%），背景人声泄漏 0.11 词/分钟；AMI 真实会议 WER 10.12%（33.03%），泄漏 2.12 词/分钟（`bench/results/TABLE_krisp.md:6`、`TABLE_ami.md:6-7`【本机】，作者自测）。⚠️ AMI 部分会议也在训练里（同领域测试，偏乐观，`README.md:42,171`）；Krisp 基准是 CC BY-NC，只能测不能训
- **误杀**：主人漏词率 Krisp 4.22%（未处理 4.28%）；1.2.1 修过一个冷启动 bug——切换时只拿 250 ms 冷启动，63 通电话里 3 通把主人整通静音（`CHANGELOG.md:3-5`）。这是流式预热问题，**不是**「别人先说话」的实例（怀疑派更正）

#### 端到端 PSE 论文（全部没有代码或权重）
- **PVQE**（2406.09928）：窗 20 / 步长 10 ms、无前瞻（§4.3）→ ① 20 ms。S 版 1.07M 参数、每帧 0.135 ms（i7 单线程）；L 版 8.38M、3.64 ms（§5.3）。DNS 2023 盲测主观：L 版 3.25 > 冠军 3.10，**S 版 2.96 < 冠军**（表 2）。AMI 上 TSOS：不给声纹 0.003、内部表示 0.010、独立声纹 0.027（表 1）——**个性化让误杀变多**。初始声纹是零向量 = 退化成通用模型（§4.3）【官方】
- **pDCCRN 一系**（2110.09625）提出 **TSOS**（式 3：逐帧过度压制量超过 γ=0.1 倍参考幅度（p=0.3 次幂）即记误杀帧）。不加非对称损失 + ASR 多任务时，pDCCRN 14.5%、pDCATTUNET 9.5%，**都高于通用 DCCRN 9.0%**；加上后 3.1% / 1.2%（ICR 集）【官方】
- **TEA-PSE 3.0**（2303.07704）：DNS5 两赛道第一（头戴与 NAPSE 并列），但 22M 参数、19.7 GMACs/s、RTF 0.46（Xeon）≈ 4.6 ms/10 ms 帧，超预算【官方】
- **DNS5 所有个性化系统的 WAcc 都低于未处理音频**（最高约 0.76–0.77 vs 0.84/0.86，`V5_DNS_Challenge_FinalResults.pdf`【本机】）
- DNS5 延迟规则：算法 + 缓冲延迟 ≤ 20 ms、不许前瞻，等于本项目 ① ≤ 20 ms【官方】赛事页。DPDFNet、GTCRN、VoxLock 都不满足——本项目用自己的 75 ms 入围线，不以 DNS 为准

#### 门控（不训练的做法）
- Personal VAD 论文：「声纹打分 + 普通 VAD」拼出的门控（SC 基线）在四种做法里**最差**（mAP 0.900 / 0.801，ET 0.946 / 0.883，表 1），且测试集**没有重叠语音**【官方】
- 「声纹相似度增益门 + 滞回」：**没找到**开源实现或评测
- d-vector 一次用约 0.8 s 窗算（maum README:133-134【第三方】）；EER 随片段变短明显上升（ERes2NetV2 数字未读原文，不引）

#### 声纹零件
- WeSpeaker 单线程 Xeon 8160：ECAPA512 6.19M、RTF 0.018；CAM++ 7.18M、0.023；ResNet34 6.63M、0.061（`runtime/onnxruntime/README.md:77-89`【本机】，按 5 s 段测，换成 2 s 是线性外推）；**无 Apple Silicon 数字**
- fbank 在 ONNX 图外，80 维；CAM++ / ERes2Net 用 povey 窗、其余 hamming，都要逐句减均值（`wespeaker/cli/speaker.py:50,97-99,342-348`【本机】）；3D-Speaker 默认不做 CMN（`speakerlab/process/processor.py:137,156`）——**两家前端不能混**
- Rust fbank：`knf-rs`、`kaldi-native-fbank`（MIT）【官方】crates.io
- **VoxCeleb 许可原文**：三个代理各连 2 次 kaist / ox 官网都失败，按规矩停；GPL 软件 Praat 随附 WeSpeaker VoxCeleb 权重并抄录了 CC BY 4.0 许可文本（fon.hum.uva.nl/praat/manual/VoxCeleb_CC-BY-4_0_license.html【官方·转引】），文中有「原视频版权归原作者」字样。FSF 列 CC BY 4.0 与 GPLv3 兼容【官方】gnu.org/licenses/license-list.html。**原文未亲见，标「未证实」**

#### 自动注册
- 家用说话人识别（2205.00288）：被动注册用 AHC 聚类；在线更新 c = αx + (1−α)c，**只在得分 > τ 时更新**，τ 太低会更新错人（§4 式 1、§7.1）；不像任何已知说话人的标「未知」不更新（§6.4）【官方】
- Apple Personalized Hey Siri：隐式注册设上限（最多 40 个向量、滚动替换），并警告「早期混进错误句子会污染档案」（machinelearning.apple.com/research/personalized-hey-siri【官方】，未二次核）
- Teams：用户 opt-in 后从会议发言生成档案，「typically takes a couple of meetings」；Express 方式持续更新、不留录音；**主档案存云端**（Office 365 compliance store），本机只有 14 天缓存（learn.microsoft.com/microsoftteams/rooms/voice-and-face-recognition【官方】）。检测到旁边有人说话时提示用户自己关
- diart：中心点在分段概率均值 ≥ rho（默认 0.3）时更新、距离 > delta_new 新建说话人（`clustering.py:138-140`）；中心点累加不遗忘，越早的样本惯性越大（`:99`）【本机】
- 物理线索（近场 / DRR / 回声参考判断主人）：**没找到**能实时用的开源实现；NS-Extractor（2306.16241，按距离自注册）无代码【官方】
- AFW（2609.20463，MIT，无权重）：注册后用快速权重跟踪漂移，**仍需一次静态注册锚点**【官方】
