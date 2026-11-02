# 线 D · 会听的大模型当评委（audio LLM as judge）

> 调研日期 2026-10-31 · 017 调研线 · 只写证据，不下「能用」结论（CLAUDE.md 五-3：没有 verifier 覆盖的一律「尚无判据，待测」）。
> 证据等级：【官方明载】【论文】【源码可证】【本机实测】【传言/未核】。论文数字均为原文表格数值，经 WebFetch 读 arXiv HTML 摘出；**未逐字对照 PDF**，引用前建议再核一次。

## 0. 先说结论（一屏）

1. **免调（zero-shot）的开源音频大模型，在「噪声 / 失真」维度上基本听不出好坏**：QualiSpeech 基准里 Qwen2-Audio-7B-Instruct 的 Noise 维 PCC = −0.048、SALMONN-13B = 0.001（语句级，见 §2.1）；SpeechEval 上 Qwen2.5-Omni-7B 总体 AVG PCC = 0.064，成对比较 AVG ACC = 0.322（低于随机，见 §2.2）。【论文】
2. **闭源大模型（GPT-4o audio / Gemini 2.5）免调成对比较能到 65–74%**，但领域不是降噪：TMHINT-QI（含增强语音）上 GPT-4o-Audio 0-shot 成对准确率 70.5%，4-shot 74.5%，同条件两名人类 78.0%（§2.3）。**它们是云端模型，按硬约束不可用于真实会议音频**。【论文】
3. **微调后才接近专用小模型**：LoRA 微调的 SALMONN 在 NISQA 三个测试集平均 LCC 0.861（语句级），专用 SOTA 0.894；ALLD 在 NISQA 域外 LIVE 集 LCC 0.86，与 wav2vec2 回归相同（§2.4）。即「会说话的 MOS 预测器」，**没有比 DNSMOS/UTMOS 类专用模型更像人**的公开证据。【论文】
4. **评委有系统性偏差，正打在降噪的痛点上**：成对比较里偏好更响的那条（+6 dB 被选中率 0.54–0.62，人类 0.612），以及位置偏差（GPT-4o-Audio 在 SOMOS/TMHINTQ 上偏好第一条，p<0.01）（§3）。降噪输出常更安静 → **送评前必须响度归一、AB/BA 双序**。【论文】
5. **本机可跑**：llama.cpp（本机 0.8.0 build 11960，有 `llama-mtmd-cli`）官方列出 Qwen2.5-Omni-3B/7B、Qwen3-Omni-30B-A3B、Voxtral、Ultravox、Gemma 4 的音频 GGUF；但**每个都 > 500 MB**（最小 Qwen2.5-Omni-3B Q4_K_M 2.10 GB + mmproj 1.54 GB），本线**未下载、未实测**，单条耗时「未测」（§4）。
6. **纯文字 LLM「模拟听众」不是人耳**：它听不到声音，只能读文字描述或数值（例如 GatherMOS 用 GPT-5 读 RMS/DNSMOS 等数字），**不得记作人耳证据**；音频 LLM 也不是人耳，只能当「又一台机器评委」，其可信度要拿人耳去校准（§5）。

**对本项目的含义**：音频 LLM 不能解决「更多人的耳朵」这个问题本身，只能作为 **第 N 个机器判据** 进 QA 层，且须先过「与作者人耳一致率 > DNSMOS」这一关（§6 给出可验证的实验设计）。

---

## 1. 题目核对（真实标题 · arXiv 号）

| 任务里的叫法 | 真实标题 | arXiv / 出处 | 会场 |
|---|---|---|---|
| ALLD | Audio Large Language Models Can Be Descriptive Speech Quality Evaluators | 2501.17202 | ICLR 2025 |
| QualiSpeech | QualiSpeech: A Speech Quality Assessment Dataset with Natural Language Reasoning and Descriptions | 2503.20290 | ACL 2025 Long |
| SpeechLLM-as-Judges | SpeechLLM-as-Judges: Towards General and Interpretable Speech Quality Evaluation | 2510.14664 | ACL 2026 Long（anthology 2026.acl-long.349） |
| SpeechQualityLLM | SpeechQualityLLM: LLM-Based Multimodal Assessment of Speech Quality | 2512.08238 | — |
| MOS-Bench | MOS-Bench: Benchmarking Generalization Abilities of Subjective Speech Quality Assessment Models | 2411.03715 | **不是 LLM 评委**，是 SSQA 泛化基准（只看了摘要层面）|
| Speaking style judges | Audio-Aware Large Language Models as Judges for Speaking Styles | 2506.05984 | Findings of EMNLP 2025 |
| AudioJudge | AudioJudge: Understanding What Works in Large Audio Model Based Speech Evaluation | 2507.12705 | EACL 2026 |
| 「Can LLMs evaluate speech quality」 | **未查到同名论文**。最接近：A Study on Zero-shot Non-intrusive Speech Assessment using Large Language Models | 2409.09914 | ICASSP 2025（据搜索摘要）|
| 微调音频 LLM 做 MOS | Enabling Auditory Large Language Models for Automatic Speech Quality Evaluation | 2409.16644 | ICASSP 2025（据 ICLR 幻灯片引用）|
| 其他 | SpeechJudge: Towards Human-Level Judgment for Speech Naturalness | 2511.07931 | ICLR 2026 |
| 其他 | Few-Shot and Pseudo-Label Guided Speech Quality Evaluation with Large Language Models（GatherMOS）| 2604.13528 | — |
| 其他 | Leveraging LLMs for Scalable Non-intrusive Speech Quality Assessment | 2508.06284 | — |
| 偏差审计 | Louder, Longer, Livelier: Acoustic Shortcuts and Underspecified Rationales in Speech LLM Judges | 2609.36979 | — |
| 偏差审计 | Auditing Protocol-Level Shortcuts in Large Audio Language Model Judges for Speech Evaluation | 2607.13477 | — |
| 未核 | ParaPairAudioBench: Paralinguistic Pairwise Audio Benchmark for LALM-as-a-Judge | 2606.24648 | 只见搜索结果标题，正文未读【传言/未核】|

---

## 2. 与人的一致率：逐篇证据

### 2.1 QualiSpeech（2503.20290）—— 免调开源模型在噪声维几乎为零
- 表 1（zero-shot，PCC，**逐条样本**，论文未写明系统级）【论文】：
  - Noise 维：SALMONN-7B 0.003 · SALMONN-13B 0.001 · Qwen-Audio-Chat 0.014 · **Qwen2-Audio-7B-Instruct −0.048** · WavLLM −0.021
  - Overall：0.084 / 0.100 / 0.250 / 0.112 / 0.071
  - 部分为 nan：原文解释为「all predicted scores are the same for that aspect」
- 数据：14,577 条（表 5），真实部分含 NISQA live（Skype 实录）、FOR、P501 与 GigaSpeech；合成部分 20% 混入 DNS Challenge 噪声，SNR 0–15 dB。**25 名标注者，每条只有 1 人评**（Limitations）——本身也是「少数人耳」。【论文】
- 微调后 SALMONN-7B 能描述噪声（表 2b：GPT 判相关 0.55–0.60），但这是「描述对不对」，不是 MOS 一致率。【论文】
- GPT-4o / Gemini 不在该基准。

### 2.2 SpeechLLM-as-Judges / SQ-LLM（2510.14664）
- 数据 SpeechEval：32,207 条、128,754 个标注、中英日法四语；来源 Common Voice、VCC2018、BVCC、LibriSpeech、AISHELL-3、JSUT、JVS + 多家 TTS。**未提增强语音**；背景噪声只作为一种失真类别。【论文 §3】
- 表 4 质量评估 AVG PCC（论文未写明语句/系统级）：SQ-LLM 0.476 · Qwen2.5-Omni-7B **0.064** · Qwen2-Audio-7B **0.065** · MiDashengLM-7B 0.128。【论文】
- 表 5 成对比较 AVG ACC：SQ-LLM 0.672 · Qwen2-Audio-7B 0.347 · Qwen2.5-Omni-7B 0.322 · MiDashengLM-7B 0.293。【论文】
- SQ-LLM = Qwen2.5-Omni-7B + LoRA + GRPO（§5.1）。摘要说「Relevant resources will be open-sourced」，**权重发布未查到**；代码仓 github.com/NKU-HLT/SpeechLLM-as-Judges（据搜索结果，未打开核对）。

### 2.3 AudioJudge（2507.12705）—— 唯一含「降噪相关」人评集的闭源成对证据
- 表 1，GPT-4o-Audio 成对准确率（%）【论文】：
  | 设置 | SOMOS | TMHINTQ | ThaiMOS |
  |---|---|---|---|
  | 0-shot | 70.5 | 70.5 | 65.5 |
  | 4-shot（示例与测试拼接）| 71.0 | 74.5 | 64.0 |
  | 人类（2 名非原标注者）| 71.0 | 78.0 | 77.5 |
- 表 2 专用模型对比：TMHINTQ 上 UTMOS 71.5 · MOSANET+ 77.5 · SALMONN-FT 61.0 · AudioJudge 74.5。【论文】
- 构对：每集抽 200 对；SOMOS 只留平均 MOS 差 > 1.0 的对（593 对里抽 200）；**TMHINTQ 构对规则论文未写**。【论文】
- 关于 TMHINT-QI 的内容：AudioJudge 文中只说「加噪」；原始数据集论文 InQSS（2111.02585）明载含 MMSE、KLT、DDAE、FCN、Transformer 增强输出，4 种噪声 × 4 个 SNR（−2/0/2/5 dB），**226 名听者**，24,408 条评分（据搜索摘要，未打开原文）。【论文·二手】→ 这是现有公开数据里**最接近「降噪输出 + 多人人耳」**的成对素材，可作为校准集候选（下载大小未查到）。
- 系统级：多维集成 + Gemini-2.5-Flash 在 13 个语音对话系统上 Spearman ρ = 0.91（表 8，**系统级**，领域是对话回复偏好，不是降噪）。【论文】
- 噪声鲁棒：给 ChatbotArena-Spoken 加 1 dB 白噪声，GPT-4o 判断不变率 Chosen 85% / Not-Chosen 93% / Tie 73%（§6.1）——说明它**对噪声不敏感**，这在「判内容」时是优点，在「判降噪」时恰恰是缺点（假设，未测）。【论文】

### 2.4 微调路线：2409.16644 与 ALLD（2501.17202）
- 2409.16644 表 I（**全部 LoRA 微调**，NISQA = LIVETALK/FOR/P501 三集平均，**语句级**，NISQA 无系统标签）：单任务 SOTA LCC 0.894 · WavLM-Large 0.850 · SALMONN(vic1.5) **0.861** · Qwen-Audio 0.771 · Qwen2-Audio 0.768。【论文】
- 同文表 IV，SOMOS A/B 准确率（微调后）：Qwen2-Audio 0.698（MOS 差 > 0.5 时 0.803）。【论文】
- 同文表 V（描述任务，GPT-4o-mini 打相关）：未调 Qwen2-Audio 0.20 → 调后 0.55；**Gemini 1.5 Pro 未调 0.43**。【论文】
- 同文**没有报告 zero-shot MOS 数字**。
- ALLD 表 1（NISQA_TRAIN_SIM 域内，语句级）：ALLD(2×) LCC 0.93 / MSE 0.17；wav2vec2 回归 0.93 / 0.27。表 2 域外：LIVE 集 ALLD 0.86 = wav2vec2 0.86；P501 ALLD 0.92 < wav2vec2 0.94。A/B 98.6%（表 3，语句级，Appendix A 明载），但成对训练数据要求「score difference more than 0.5」，**是易对**，不能和我们 exp003 的近似对比较。附录 C：Qwen2-Audio、SALMONN 等免调「all fail to reliably evaluate the input speech」。【论文】
- SpeechQualityLLM（2512.08238）：AST/Whisper 编码器 + Llama 3.1 8B（4-bit + LoRA），NISQA 无参考 r 0.78 / ρ 0.77（表 2，逐条）；声称代码与权重在 github.com/Monjur-Mahathir/Speech-Quality-LLM（未打开核对）。【论文】
- 2508.06284：用微调的 Vicuna-7B+Whisper 音频 LLM 给 101,129 条 LibriAugmented 打伪标签，再训 DNSMOS Pro；TMHINT-QI 测试集 PCC 0.56（两阶段）vs 0.49（只用 NISQA，表 3，逐条）。——**音频 LLM 当「标注员」再蒸馏成小模型**，这条路对 CI 层更实用。【论文】

### 2.5 闭源模型的其他证据
- 2409.09914（TMHINT-QI(S)，1,960 条含噪/干净/增强）：直接把音频给 GPT-4o 打分，论文**只给定性例子**，称其「determines the quality … based only on metrics such as SNR, clipping, amplitude, or loudness」，不可靠；GPT-Whisper（先 ASR 再文字打分）质量 SRCC 0.4360（表 I，推断为语句级）。【论文】
- 2506.05984（说话风格，非降噪）：Pearson r，人-人 0.596 · 人-Gemini-2.5-pro **0.640** · 人-GPT-4o-audio 0.355（表 2，voice style IF，80 个分数 = 4 模型 × 20 例）；4 名人评。【论文】
- SpeechJudge（2511.07931，TTS 自然度成对，表 2，Total 准确率 %）：Gemini-2.5-Flash 69.1 · GPT-4o Audio 67.4 · **Kimi-Audio-7B-Instruct 67.0** · Gemini-2.5-Pro 66.5 · MiDashengLM 61.6 · Qwen2.5-Omni-7B 60.6 · Voxtral-Mini-3B 56.0 · Gemma-3n-E4B 48.2；对照 DNSMOS 57.9、UTMOS 53.7。微调的 SpeechJudge-GRM（Qwen2.5-Omni-7B Thinker）77.2%。【论文】
  → 开源里 Kimi-Audio 免调成对最好，但**领域是 TTS 自然度**，不是降噪残噪/伪影。

### 2.6 按模型汇总（任务点名的每一个）
| 模型 | 降噪/噪声语音上的公开一致率 | 其他领域 | 等级 |
|---|---|---|---|
| Qwen2-Audio-7B | QualiSpeech Noise PCC −0.048；SpeechEval PCC 0.065 / 成对 0.347；微调后 NISQA LCC 0.768 | — | 论文 |
| Qwen2.5-Omni-7B | SpeechEval PCC 0.064 / 成对 0.322 | SpeechJudge 60.6% | 论文 |
| Qwen3-Omni-30B-A3B | **未查到**质量一致率；2607.13477 报告 Thinking 版在拼接 A/B 中「often picked the same slot regardless of order swaps」 | 技术报告 2509.17765 未见 MOS 评测（第三方评测提到「MOS-X」，无数字）| 论文 / 传言 |
| SALMONN | QualiSpeech Noise PCC 0.001–0.003；微调后 NISQA LCC 0.861 | AudioJudge 表 2 SALMONN-FT TMHINTQ 61.0% | 论文 |
| Gemini 1.5 | 描述任务 0.43（2409.16644 表 V）；位置偏差 18.3% | — | 论文 |
| Gemini 2.x | **降噪上未查到** | SpeechJudge 69.1%（2.5 Flash）；风格 r 0.640（2.5 Pro）；系统级 ρ 0.91（对话）| 论文 |
| GPT-4o audio | TMHINTQ 成对 70.5%（0-shot）/ 74.5%（4-shot）vs 人 78.0% | SpeechJudge 67.4% | 论文 |
| Kimi-Audio-7B | **降噪上未查到** | SpeechJudge 67.0% | 论文 |
| Step-Audio | **未查到**作为评委的数字（2506.05984 里它是被评对象）| — | — |
| MiDashengLM-7B | SpeechEval PCC 0.128 / 成对 0.293 | SpeechJudge 61.6% | 论文 |

---

## 3. 已知偏差（对降噪评测尤其致命）
- **响度偏好**（2609.36979 表 4）：成对比较中 +6 dB 那条被选中率 GRM 0.541 · UniSRM 0.575 · SQ-LLM 0.591 · Gemini-2.5 0.566 · Gemini-3.1 0.616；人类 0.612（表 14）。逐点打分偏差弱或不显著。论文建议「normalize loudness and trim leading or trailing silence」、AB/BA 双序。**该文未测噪声/SNR 操纵**。【论文】
- **位置偏差**（2507.12705 §6.3）：GPT-4o-Audio、Gemini-1.5-Flash、Gemini-2.5-Flash 偏好第一位（p<0.05）；非词汇任务上 GPT-4o 在 SOMOS 与 TMHINTQ 偏好第一位（p<0.01），交换顺序后稳定率 < 80%。【论文】
- **冗长偏好**：所有模型在平局样本上偏好更长回复（p<0.01）。【论文】
- **捷径**（2607.13477）：给错的专家标签可把 5 个评委的情绪准确率压到 ≤ 0.10——评委会「抄提示」而不是听。【论文】
- 与 exp003 已知的人耳位置偏好（D33，见 `docs/research/012_eval-standards/sources.md:96`）叠加，**双序是硬要求**。

## 4. 本机（Apple M5，32 GB 统一内存，盘剩约 172 GB）可跑性
本机实测项：`sysctl hw.memsize` = 34359738368（32 GB）、CPU = Apple M5；`llama-server --version` = 0.8.0 (build 11960)，`/opt/homebrew/bin/llama-mtmd-cli` 存在。【本机实测】下面文件大小来自 Hugging Face API（`/api/models/<repo>/tree/main`，2026-10-31 查询）【官方明载】。**均超 500 MB，本线一律未下载、未实测，单条耗时未测。**

| 模型 | 路线 | 文件 | 大小 | 32 GB 能否装下（假设）|
|---|---|---|---|---|
| Qwen2.5-Omni-3B | llama.cpp GGUF | Q4_K_M + mmproj Q8_0 | 2.10 + 1.54 GB | 能 |
| Qwen2.5-Omni-7B | llama.cpp GGUF | Q4_K_M + mmproj Q8_0 | 4.68 + 1.55 GB | 能 |
| Qwen3-Omni-30B-A3B-Instruct | llama.cpp GGUF | Q4_K_M + mmproj Q8_0 | 18.56 + 1.33 GB | 勉强（MoE 激活 3B，快但占内存）|
| Qwen3-Omni-30B-A3B-Instruct | MLX（mlx-vlm 0.3.10 转换）| 4bit | 21.8 GB | 勉强 |
| Qwen2-Audio-7B-Instruct | MLX（mlx-audio）| 4bit | 6.6 GB | 能；**llama.cpp 官方不给 GGUF**，原话「no pre-quantized GGUF this model, as they have very poor result」|
| Voxtral-Mini-3B | llama.cpp GGUF | Q4_K_M + mmproj | 2.47 + 0.72 GB | 能 |
| Ultravox 0.5 8B | llama.cpp GGUF | Q4_K_M + mmproj | 4.92 + 1.38 GB | 能 |
| Gemma 4 E4B | llama.cpp GGUF | Q4_0 + mmproj Q8_0 | 4.59 + 0.56 GB | 能 |
| Kimi-Audio-7B-Instruct | 仅 PyTorch 原版 | 全仓 | 42.6 GB | 未查到 MLX/GGUF；原版 Mac 上可跑性未查到 |
| Step-Audio-2-mini | 仅 PyTorch 原版 | 全仓 | 17.9 GB | 未查到 MLX/GGUF |
| Qwen2.5-Omni-7B 原版 | PyTorch/transformers | 全仓 | 22.4 GB | VERSA 用此路（见下）|

- llama.cpp 多模态文档：github.com/ggml-org/llama.cpp/blob/master/docs/multimodal.md 列出上述 `-hf ggml-org/...` 仓库。旧版文档曾写「audio is highly experimental and may have reduced quality」，当前版本未见该警告（据搜索摘要与 WebFetch 对比）。【官方明载】
- **VERSA 已集成**（本机只读源码）：`resources/versa/versa/utterance_metrics/qwen2_audio.py:287`（默认 `Qwen/Qwen2-Audio-7B-Instruct`）、`resources/versa/versa/utterance_metrics/qwen_omni.py:106`（`Qwen/Qwen2.5-Omni-7B`）；其中「录音质量」是 5 档分类提示（Professional/Good/Fair/Poor/Very poor，`qwen2_audio.py:249`），**不是 MOS、也不是 A/B**；配置示例 `resources/versa/egs/separate_metrics/qwen2_audio.yaml`。走 transformers + `device_map="auto"`（`qwen2_audio.py:307`），Mac 上为 MPS/CPU，速度未测。【源码可证】
- 单条耗时：**未查到可信公开数字，本机未测**。只能写成待测项（§6 实验 E1 会记）。

## 5. 不算人耳的东西（明确划线）
- **纯文字 LLM 当「模拟听众」听不到声音**。例：GatherMOS（2604.13528）用 GPT-5 读 RMS、ZCR、削波率、DNSMOS/VQScore 伪标签等文本化特征，VoiceBank-DEMAND 200 条上 LCC 0.6439 / SRCC 0.6014（表 2，推断为语句级），本质是「对已有机器分做加权」；2409.09914 的 GPT-Whisper 只读 ASR 文本。**这些只能算机器分的再组合，不得记作人耳证据，也不得用来回答「大众偏好是否不同于作者」。**【论文】
- **音频 LLM 也不是人耳**：它是用人评数据（或根本没用）训出来的又一个预测器，上面的「与人一致率」本身就是要拿人耳去量的东西。它只能进 QA 层当机器评委，可信度靠 §6 的人耳校准。
- **云端模型**（GPT-4o audio、Gemini）：按硬约束**任何真实会议音频不得上传**；若要用，只能用于无隐私的合成开发音频（如 DNS 噪声 + 公开朗读语料混合），且**需人另行批准**——列入待拍板，本线不做。

## 6. 可验证的下一步（建议，未执行）

**E1 · 本机音频 LLM 评委 vs 作者人耳（QA 层 verifier 候选 `V-LLMJ-AB`）**
- 素材：exp003 的 48 个配对（已有作者答卷与揭盲表，见 `docs/experiments/exp003*/`），全程本机。
- 预处理：两条都做响度归一（如 −23 LUFS，`pyloudnorm` 已在 resources）、裁首尾静音；每对 AB 与 BA 各问一次。
- 候选：Qwen2.5-Omni-7B Q4_K_M（6.2 GB）、Qwen3-Omni-30B-A3B Q4_K_M（19.9 GB）——**均 > 500 MB，需人批准下载**。
- 判据（写死再跑）：① 双序一致率 ≥ 80%（不过 = 位置偏差主导，直接判不可用）；② 与作者一致率的 95% bootstrap 区间下界 > DNSMOS OVRL 同口径一致率（现 0.48–0.64）才算「比现有机器分更像作者」；③ 记录每条耗时与峰值内存。
- 注意：48 对、1 名听者，样本太小，**过线也只能说「与作者一人更一致」，不能说「更像大众」**。

**E2 · 多人人耳校准集（回答「作者是否特殊」的唯一公开素材）**
- TMHINT-QI：含增强输出、226 名听者（据 InQSS 2111.02585）。可把作者也放进去听其中一小批，看作者与 226 人平均的偏离；同时给 E1 的评委、DNSMOS 打分对照。**下载大小未查到**，需先查再决定（> 500 MB 只记不下）。语种为中文普通话，与会议场景部分匹配。

**E3 · 蒸馏路线（CI 层）**：若 E1 过线，参照 2508.06284 用评委给合成测试格打伪标签再蒸馏到小模型——属后续，不在本线。

## 7. 待人拍板
1. 是否批准下载 > 500 MB 的本地音频 LLM（最小可用 Qwen2.5-Omni-3B 约 3.6 GB；推荐 7B 约 6.2 GB）做 E1。
2. 是否允许把**无隐私的合成开发音频**送 Gemini / GPT-4o 做对照（默认：不允许）。
3. 是否获取 TMHINT-QI（E2），需先查授权与大小。

## 来源
- arXiv：2501.17202 · 2503.20290 · 2510.14664 · 2512.08238 · 2411.03715 · 2506.05984 · 2507.12705 · 2409.09914 · 2409.16644 · 2511.07931 · 2604.13528 · 2508.06284 · 2609.36979 · 2607.13477 · 2111.02585 · 2509.17765（均 https://arxiv.org/abs/<号>）
- llama.cpp 多模态文档：https://github.com/ggml-org/llama.cpp/blob/master/docs/multimodal.md
- HF API 文件大小：https://huggingface.co/api/models/ggml-org/Qwen2.5-Omni-7B-GGUF/tree/main 等（§4 表中各仓库）
- 本地：`resources/versa/versa/utterance_metrics/qwen2_audio.py:249,287,307` · `resources/versa/versa/utterance_metrics/qwen_omni.py:106` · `resources/versa/README.md:21` · `resources/versa/egs/separate_metrics/qwen2_audio.yaml`
- 已有卡片（不重复）：`docs/research/014_real-datasets/cards/nisqa-livetalk.md` · `docs/research/012_eval-standards/verifiers.md`
