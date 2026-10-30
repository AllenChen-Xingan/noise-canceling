# 来源登记 · 006 降噪前沿全景（2026-10-30）

> 手下 sota。Workflow `wf_5b5f8eb3-fe1`（①–⑥ + 怀疑派 ×3 + 完整性批评）· `wf_b412f601-4d1`（⑦）· `wf_55ec3686-951`（⑧）。**本表已按怀疑派改正**：被推翻或改口的在「判定」列注明。
> 标注：【官】官方明载 ·【论】论文 ·【源】开源代码 / 仓库元数据（`gh api` 2026-10-30 查）·【本】本线亲自核过源码 ·【传】营销稿 / 测评 / 论坛 ·【推】推断。
> 002 已比过的（DPDFNet、GTCRN、UL-UNAS、DFN3、nnnoiseless……）与 005 已覆盖的 PSE 候选**不重登**，只登新证据。

## 一、自有积累

| 是什么 | 在哪 | 还算数吗 |
|---|---|---|
| 002 候选表、延迟 ①②③ 口径、exp001 每帧耗时（M5，ort CPU：dpdfnet2 最坏 0.89 ms、dpdfnet4 1.40 ms） | `docs/research/002_model-baseline/decision.md` | 算数；DPDFNet 60 ms 被本线三方印证（见下「DPDFNet 新证据」） |
| 005 PSE 取舍卡（VoxLock、门控三步、TSOS / IRL 评测） | `docs/research/005_pse/decision.md` | 算数；本线只补新东西（第④块），结论「VoxLock 仍是唯一能下手的」未被推翻 |
| 004 评测工具链（UTMOS / SCOREQ / DNSMOS、吃字率用 Qwen3-ASR） | `docs/research/004_eval-toolchain/decision.md` | 算数；第③块的外部证据**加强**了「不能只看 DNSMOS」 |
| 设计稿 §4 现有方案、§7 工程、§9 音色、§11 延迟与功耗 | `vault/raw/20261029-voice-denoise-design.md:129-222, 300-420, 474-532` | 部分算数：§11「ANE 主要用来省电」被第⑤块削弱；§11「每次处理 2 帧省电」被怀疑派指出混淆了 HAL 缓冲与模型批处理 |

## 二、GitHub 仓库

| 仓库 | 是什么 | 许可（码 / 权重 / 数据） | 因果实时 | Apple Silicon | 判定 | 出处 |
|---|---|---|---|---|---|---|
| **aask1357/fastenhancer** → `resources/fastenhancer` | FastEnhancer T/B/S/M/L（ICASSP 2026）：RNNFormer，16 kHz 窗 512 / 跳 256 | **代码 MIT**【本 `LICENSE`】；权重在 release，无单独声明，随仓 MIT【推】；16k VB / DNS 版数据干净，**48k 版训练含 WHAM（NC）等**【源·refute-2 #3】 | 是：wav2wav 版输入输出固定延迟 n−h = 16 ms + 一跳缓冲 16 ms ≈ **算法延迟 32 ms**【本 `docs/docs/onnx.md:48-50`】；**状态显式**（`cache_in_*` 输入、逐帧回灌，`scripts/test_onnx_spec.py:44-61`）【本】 | M1 / M5 单线程 RTF 有表（B：M5 0.011），**只计网络、不含 STFT**【源·refute-2 #3】 | **改造**（同台候选）；M / L 档跳长不同（160 / 100）| arXiv 2509.21867 |
| 同仓 release `onnx-vd-v1.0.0` | **基线动物园**：同一训练设置下的 GTCRN、LiSenNet（流式改版）、FSPEN、BSRNN、DPRNN、DPT spec2spec ONNX | MIT（随仓）；**只训练于 VoiceBank** | 是 | 同上 | 只看看（评测对照用） | `gh api …/releases`【本】 |
| 同仓 release `ckpt-dereverb-v0.1.0` | FastEnhancer 去混响检查点（T/B/S/M/L） | 随仓 MIT【推】；训练数据未核 | 是 | 未测 | 只看看（M4 去混响候选） | 同上【本】 |
| Acceleration123/CoFi-Lite | 粗细双路径超轻模型（SPL 2026），83K 参数 / 12.9 M MAC/s | **无 LICENSE**（API 返回 null）→ 不能用；2026-10-19 起放出训练 + 推理代码与 DNS3 权重 | 是，零前瞻 32 ms | 自测 RTF 0.033 vs GTCRN 0.050（机型未写） | **不适合**（许可）；可去 issue 问许可 | refute-2 #1【源】 |
| dddaniel-z/HALO | 半帧率插件，GTCRN+HALO PESQ 2.101→2.198 | 无 LICENSE、无权重 | 是 | 无 | 值得自己实现（要训练） | arXiv 2606.12328【源】 |
| yskim3271/LaCo-SENet | 可配前瞻（12.5–200 ms）流式模型，1.37M | 代码**无 LICENSE**；HF 权重 **MIT**，仅 VB 训练 | 是，但 Xeon 逐帧 RTF 4.59（未核） | 无 | 只看看 | arXiv 2606.19688【源】 |
| narrietal/Fast-ULCNet | ULCNet 的 GRU 换 FastGRNN | MIT；**无权重、无训练脚本** | 称延迟降 34%（未核） | 无 | 只看看 | arXiv 2601.14925【源】 |
| xiph/rnnoise v0.2 | RNNoise 新训练 | BSD；**只用公开数据训练**【官】 | 是 | **有 NEON**（`src/vec.h:41-42`），无 ARM RTCD【源·refute-2 #10】 | 只看看（垫底基线，002 已有） | releases/tag/v0.2 |
| xiph/opus（1.5 / 1.6 的 OSCE：LACE / NoLACE / BBWENet、Deep PLC、DRED） | 编解码内的神经后处理与盲带宽扩展 | 代码 BSD-3；**权重靠 `dnn/download_model.sh` 下载，单独许可文本没找到** | BBWENet：10 ms 帧、前瞻 0.27 ms、约 70 MMAC/s【论·官】；1.6 中是实验功能、`--enable-osce` 编译、只对 SILK 宽带解码生效【源】 | 无 | **改造**（从 `dnn/bwe_demo.c` 抽出独立用，见第⑧块）| arXiv 2412.11392 · opus-codec.org/demo/opus-1.6 |
| kylehowells/DeepFilterNet-mlx · HF iky1e/DPDFNet4-CoreML、DPDFNet8-48kHz-HR-CoreML、DeepFilterNet3-Streaming-CoreML | DFN3 / DPDFNet 的 Swift + MLX + 有状态 Core ML 流式版 | Apache-2.0 / MIT | 是 | **DPDFNet4 Core ML 每帧 p50 / p95 = 3.69 / 4.17 ms**（比 ort CPU 0.89–1.40 ms 慢）；DFN3 Core ML 按 10 ms 真实节拍 p99 9 ms、漏拍 5%（另一次 0.6%）；机型未写 | **只看看**：反证「上 ANE 更快」 | refute-3 漏掉的反例 1 · `docs/live-device-benchmark.md`【源】 |
| pumfleet/altvane PR #6 | 用 DPDFNet 的产品：去掉 ort，改自写 C++ + Accelerate GEMM | 未核 | x86 单核每帧 1.03→0.74 ms，去掉一跳 10 ms worker 延迟；原话「剩下的 60 ms 来自帧长和前瞻」 | M5 未测 | 只借鉴（后端思路） | github.com/pumfleet/altvane/pull/6【源】 |
| dharanipathirk/neural-audio-bench | DAFx 2026「实时安全神经音频推理」基准 | 论文 CC BY 4.0；代码许可未核 | — | M3 MacBook Air 实测（见论文行） | **直接用**（作为后端选型的测法） | 见论文 DAFx26 #16 |
| dignifiedquire/sonora（`sonora-agc2` 0.2.0） | WebRTC AGC2 纯 Rust 移植（自适应增益 + 限幅 + 削波预测） | BSD-3 | 是 | 纯 Rust【推】 | **直接用**（M4 响度链候选）；数值是否对齐 C++ 未核 | docs.rs/sonora-agc2【源】 |
| `ebur128` crate | BS.1770 / R128 响度计量 | MIT（未核） | 短时 / 瞬时响度可实时 | — | 直接用（评测与 AGC 监测） | crates.io/crates/ebur128 |
| sarulab-speech/Sidon（+ 社区 Core ML / ONNX 移植） | 离线语音修复，自称与 Miipher 相当 | 代码 MIT、权重 MIT；**DialogueSidon 是 CC BY-NC** | **否**（w2v-BERT 2.0 + HiFi-GAN） | 有移植，无实测 | 只看看（离线教师 / 参照） | arXiv 2509.17052【源】 |
| Tencent-Hunyuan/AuK → jindratilk/voicy | 离线生成式修复；Voicy 是它的 Mac MLX 应用 | MIT（LICENSE 原文含权重） | **否**：8.7 s 音频在 M5 上约 60 s | 有 | 只看看 | arXiv 2609.08936【源】 |
| resemble-ai/resemble-enhance · NVIDIA/CleanUNet · Open-Miipher-2 | 离线修复 / 研究代码 | MIT | 否 / 未证实 | 无 | 只看看 | 第①块 |
| HF Scicom-intl/voice-isolator(-cpu) | 因果主讲人隔离器，2210 万参数，10 ms 延迟；Krisp 基准 WER 5.74% vs Krisp 5.63% | **`license: other`，卡片无许可正文** → 按命门 2 算「无许可」 | 是；单线程约 33% 核 | 无 | 不适合（许可）；**应转 005 线评估**，可问作者 | arXiv 2610.35272 · refute-1 漏掉的反例 1 |
| REAL-TSE/wesep-real-tse · xu-shitong/TSE-through-Positive-Negative-Enroll · IiuZiKai/Evo_TSE | 2025–26 TSE 代码 | **都没有 LICENSE** | — | — | 不适合 | 第④块 · refute-3【源】 |
| sp-uhh/streamfm | 流匹配流式 SE / BWE | AGPL-3.0 | 算法 32 ms，需 GPU | 无 | 不适合（Mac CPU 跑不动） | arXiv 2512.19442【源】 |

## 三、论文

| 论文 | 一句话 | 关键数字 | 判定 |
|---|---|---|---|
| DPDFNet arXiv 2512.16420v2 + altvane PR + 002 源码 | 算法延迟 | 窗 20 / 跳 10 / 前瞻 ℓ=2；**60 ms 三方印证**；Core ML 卡的 50 ms 和 README「约 20 ms」是另一口径【critic #1】；训练数据含 FSD50K（有 CC BY-NC 片段）【推：数据许可有瑕疵，需登记】 | 002 结论不变 |
| StuPASE arXiv 2603.09234（Interspeech 2026） | 开源对 Adobe Enhance Speech V2 的唯一硬对比 | 有混响：UTMOS 4.01 vs 3.71，**dWER 7.89% vs 18.87%**；无混响 Adobe 音质略高（4.08 vs 3.99）、dWER 4.55% vs 2.84% | 成立（refute-1 #4）。**生成式离线修复会改字** |
| The Voice on the Line arXiv 2610.35272 | 开源因果主讲人隔离器对 Krisp | 见上表 | 推翻第①块「无开源与 Krisp 比过」 |
| DeepVQE arXiv 2306.03177 | AEC + 降噪 + 去混响合一 | 完整版（7.5M、i7 每帧 3.66 ms）DNS 2023 表 3 第二组 OVRL 2.83 vs 2.66；**S 版 0.59M、0.14 ms/帧没有单独的盲测分**；20 ms 延迟 | 部分成立（refute-2 #7）；无代码无权重 |
| FastEnhancer arXiv 2509.21867 | 速度优先的流式 SE | VB PESQ T 2.99 / B 3.13 / S 3.19（GTCRN 2.87）；DNS1 PESQ B 2.69（README，单种子） | 见仓库行 |
| CoFi-Lite 2607.10142 · AdaptCRN 2502.14224 · HALO 2606.12328 · µNet 2608.21155 · RT-SEMamba 2608.12099 · BASENet 2606.12662 · StreamSE 2610.37024 | 2025–26 新轻模型 | DNS3 PESQ：GTCRN 2.07 → CoFi-Lite 2.16 / AdaptCRN 2.30；µNet 是专利（论文 CC BY-NC-ND 只管文本）；StreamSE 无代码；BASENet 7.1 G MACs 超预算 | **全部无可用许可 + 权重组合**；均**无 babble / ≤0 dB 分档** |
| Diffusion Buffer 2510.18744 · Stream.FM 2512.19442 · SB-Mamba 2510.16834 | 生成式流式 SE | 32–176 ms、需 GPU | Mac CPU ≤2 ms/帧下不可行【推】 |
| URGENT 2024 分析 2506.01611 · URGENT 2025 2505.23212 · VoiceMOS 2026 2609.13792 | 指标与真人分的关系 | UTMOS / SCOREQ 与 MOS 相关最高；生成式 T13 MOS 最高但低 SNR 下 CAcc 67.9%（中文 20.1%），**DNSMOS 在幻听时仍高**；CCR 逐条 SRCC 只有 0.487 | 成立 |
| CHiME-7 UDASE 2402.01413 · CHiME-9 ECHI 2609.26306 | 真实多人交谈录音上的听测 | CHiME-7：4 个方法只有 1 个整体好于带噪；DNSMOS 第一的真人垫底；CHiME-9：**强制因果 ≤20 ms 前瞻**，客观与主观「几乎不一致」 | 成立 |
| DAFx 2026 #16（Maynooth） | Apple Silicon 实时安全推理 | M3 Air：**ort 走 MLAS NEON、不走 AMX，判「非实时安全」**；BNNSGraph 用 AMX、RTSan 无分配无锁；单测 RTF 0.21 的模型在 Core Audio 回调里 p99 158%、355 次 xrun | 成立（critic #2）；BNNSGraph 要 macOS 15+ 与 `.mlmodelc` |
| Bryngelson《Apple Neural Engine》arXiv 2606.22283 | ANE 逆向与实测 | 派发下限 **M1 0.23 ms / M5 0.11 ms**；GRU 无原生算子但可展开成矩阵乘上 ANE；单次调用循环约 755 mW（powermetrics） | 部分成立（refute-3 #8、#9）；单作者未评审 |
| NEC IWAENC 2022（IEEE 9914802） | 残噪 vs 失真偏好 | 听众「怕噪」「怕失真」两派**人数相当**，偏好设置置信区间不重叠 | 成立【论】→ 衰减下限应可调 |
| 2608.07781（MLSP 2026） | ASR 对齐找出被吃掉的片段，按比例混回原声 | 无训练；非因果 | 离线评测的吃字定位工具 |
| 非对称窗 2204.09911 · 2304.08707；Wu & Braun 2409.10358 | 低延迟前端 | 2 ms 算法延迟仍强；但强模型上非对称窗与预测未来帧**无增益**（Wu & Braun 摘要；表内数字未核） | 都要重训 |
| 2210.07692（MCU 量化） | RNN 降噪 int8 | 全 8 bit PESQ −0.3，FP16-INT8 混合 −0.06、提速至 4×（MCU） | 不适用 Apple CPU；ort 不量化 GRU（`registry.py:32-38`） |
| 部分去混响：2603.02641 · 2505.01338 · 2204.08765 · JASA 2024 | 保留多少早期反射 | **结论相反**：0 ms 最好 / 80 ms 最好 / 20 ms 折中 | 必须自己 A/B |
| REAL-TSE 2607.15198 + CARTSE / SA-Mamba 报告 | SLT 2026 真实会议 TSE | 在线赛道 ≤100 ms；CARTSE 因果 24 ms + 8 ms 缓冲，**训练里最大化 DNSMOS（刷分嫌疑）**；SA-Mamba 15.89M、48 ms；前三都用真实录音伪标签 | 无可用权重；只借数据配方 |
| FVAD 2609.19856 · EEND-SAA 2509.11957 · 主说话人分割 2603.13379 | **无注册**的前景 / 主说话人判别 | FVAD 0.6M、t2.micro 每帧 1–2 ms、BG-FAR babble 0.12 vs Silero 0.27 | 都没代码；作 005 第 1 步门控的思路 |
| 延迟可感：Kitawaki & Itoh 1991（JSAC 9(4)）经 Egger 2010 · Schoenenberg 2014（IJHCS 72(5)）· Stivers 2009（PNAS） | 对话延迟察觉 | 未受训者自由对话往返约 1120 ms 才察觉；**受训者快速轮读往返 90 ms 就察觉**；延迟会被误归因为「对方不积极」；话轮间隙众数 0–200 ms | 部分成立（refute-4 #1–3）：**没有实验测过在基线上再加 45 / 75 ms 能否察觉** |

## 四、榜单与挑战

| 榜单 | 还能信吗 | 对我们 | 出处 |
|---|---|---|---|
| VoiceBank+DEMAND | **饱和**：只为 PESQ 优化可刷到 3.82 但听测很差 | 不看 | arXiv 2406.03460 |
| DNS 1–5 | 2023 后停办；DNS5 规则 ≤20 ms、不许前瞻；冠军 Score 只比带噪 +0.145；多数系统 SIG / WAcc 不如不处理 | **测试集还能用**（DNS5 盲测集真实录音 + WAcc），榜单不更新 | arXiv 2303.11510 |
| URGENT 2024 / 2025 / 2026 | 方法论可信（多语种 P.808 + CAcc）；名次对因果小模型**无参考价值**（前列 1 亿参数级、非因果）；2026 主观与客观只是前两名互换（refute-1 #10）；冠军 baird 系统没找到 | 借方法，不借名次 | FinalRankingDetails_URGENT2026.pdf |
| CHiME-9 ECHI | **最贴我们**：真实四人桌边对话 + babble、强制因果 ≤20 ms、真人听测；但多麦（眼镜 / 助听器），数据许可未核 | 对齐口径；看能否取单通道 | arXiv 2609.26306 |
| CHiME-7 UDASE | 真实晚宴录音，证明 DNSMOS 在真实录音上会选错 | 当反面教材 | arXiv 2402.01413 |
| LRAC 2025 / 2.0（ICASSP 2027） | 编解码 + 降噪，限 FLOPS 与延迟（≤50 ms）的真人 ACR；前三代码与分数没读到 | 唯一「低复杂度」带约束的比赛，跟踪 | crowdsourcing.cisco.com/lrac-challenge |
| Paperswithcode / codesota / hyper.ai | 无复杂度、延迟、因果列 | 不看 | 【传】 |
| 常设低复杂度排行 | **没有**；最接近的是 FastEnhancer 论文的同机对比表 | 用它的口径（单线程 ONNX、M 系 RTF） | arXiv 2509.21867 |

## 五、厂商公开资料（只信官方页与论文；其余标【传】）

| 厂商 / 产品 | 公开到什么程度 | 延迟 | 出处 |
|---|---|---|---|
| iZotope RX 11 / 12 Dialogue Isolate | 只说「神经网络」，实时与离线两套网 | 实时版 2048 采样【传，采样率未注明】 | izotope.com · production-expert.com；**RX 单列见第⑧块与 `gap.md`** |
| CEDAR Voxis / VoicEX 2 / DNS | Voxis「神经网络、<10 ms」【官】；价格官网 £199 + 税（$269 与发售日是【传】） | <10 ms【官】 | cedaraudio.com/plugins/voxis |
| Krisp | 早期专利 US11100941B2（dense DNN + 比值掩码，8 kHz）【官】；2025-11 博客：`viva-tel-lite-v1` BVC 小模型算法延迟 15 ms、16 kHz、按「说话人到麦克风距离线索」区分主讲人，比**大号** viva-tel 小 3.5×、与上代 lite **同大小**【官，refute-1 #1】；SDK 文档 NC 约 25 ms【官】；M1 版「2× 续航」【传，2021】 | 15–25 ms | krisp.ai/blog/small-voice-isolation-model · sdk-docs.krisp.ai |
| NVIDIA Broadcast / Maxine AFX | 只给采样率与局限（SNR<5 dB 失真、头 1–2 s 漏噪）【官】；无 macOS【推，文档只给 Win/Linux】 | 无数字 | docs.nvidia.com/maxine/afx |
| Apple Voice Isolation | 只说「机器学习」；**App 不能程序化打开**：`preferredMicrophoneMode` 只读，只能 `showSystemUserInterface(.microphoneModes)` 弹面板【官】；无功耗数字 | 无 | developer.apple.com/documentation/avfoundation/avcapturedevice/preferredmicrophonemode |
| Waves Clarity Vx | 「神经网络」 | **2048 采样 @44.1 / 48k ≈ 43 ms**【官】 | waves.com/plugins/clarity-vx |
| Acon Extract:Dialogue 2 | 「AI」「低延迟」 | 约 96 ms【传，论坛】 | acondigital.com |
| Supertone Clear（已改名 Antinode Clear） | 三旋钮：人声 / 环境 / 混响【官】 | 无 | supertone.ai |
| Adobe Enhance Speech · ElevenLabs Voice Isolator · Descript Studio Sound | 离线 / 云端；无技术说明 | 不适用 | 第①块；Adobe 被 StuPASE 量出混响下 dWER 18.87% |
| Zoom / Teams 自带降噪 | 无延迟数字；「Zoom 高档会再吃人声，建议调低」只有厂商经验【传】；Teams Rooms 官方建议有外部 DSP 时关内置降噪（只限会议室设备）【官】 | 无 | critic #5 |
| 商业盲听（Production Expert 2024 / 2025） | 媒体自办，票数未公开 | dxRevive Pro › Hush Pro › RX DI › Extract:Dialogue › Supertone › Clarity Vx（2024）| 【传】 |

**降噪后补偿 / 修复（第⑧块）**：见 `gap.md` 第二节「补偿与修复」与 `decision.md`。
