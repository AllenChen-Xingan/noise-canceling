# 两类发现 · 评测工具链用哪些现成的

> 2026-10-29 · evaltool。分级：P0 不补就不能开工 · P1 定标前要补 · P2 第一版评测跑起来后补 · P3 酌情。
> 出处标注：**明载** · **源码**（`resources/` 行号）· **二手** · **实测**（exp002）。

## 一、它们做到的

逐条对照三条命门（`must-know.md`），看整套现成工具做到了多少。

| # | 不可妥协的那条 | 状态 | 差距是什么 | 修起来要什么 | 分级 |
|---|---|---|---|---|---|
| 1 | 本机离线、Apple Silicon 能跑 | **部分**（缺 MPS 路径与实测速度） | DNSMOS + PESQ/STOI/SI-SDR/LUFS 已在 M5 上纯 CPU 跑通，10 秒音频 0.05 秒（实测，exp002）。其余都是「能切 CPU、没有 MPS」：URGENT 脚本把 GPU 判断写死成 `"cuda" in device`（`urgent2026_challenge_track1/evaluation_metrics/calculate_nonintrusive_dnsmos.py:100` 等三处，源码）；UTMOSv2 默认 `cuda:0`（`UTMOSv2/utmosv2/_core/model/_common.py:89`）；openai-whisper 不会自动选 MPS（`whisper/__init__.py:130-131`）。SCOREQ/UTMOS/NISQA/whisper.cpp 在 Mac 上的速度全部是推算。**`dnsmos_local.py:24-25` 不指定执行器，CoreML 没用上**（实测：实际是 `CPUExecutionProvider`） | 接一根线：打分器自己写薄封装，显式传 `cpu`/`mps`、onnxruntime 显式选执行器；下一批跑一次计时实验 | P1 |
| 2 | 许可允许开源评测工具；数据不妨碍发布权重 | **部分**（缺「许可干净」的清单与执行） | 许可上允许再分发或进 CI 的打分器只有：DNSMOS（CC BY 4.0，ONNX 归「其他内容」属解读）、SCOREQ（ONNX 权重 CC BY 4.0，Zenodo 15739280）、UTMOS22/SpeechMOS（MIT）、UTMOSv2（MIT）、whisper.cpp（MIT）、pystoi、pyloudnorm、`ebur128`。实际放 CI 层还是本机层，按下载量和 runner 资源另定，见 `decision.md`。**不能进仓**：NISQA 权重 CC BY-NC-SA（`NISQA/README.md:166`）；PESQ 的 ITU C 代码禁止修改、翻译、分发（`PESQ/pesq/dsp.c:34-75`）；LPS 依赖 GPL Levenshtein（`calculate_phoneme_similarity.py:7`）；URGENT-PK 与 Uni-VERSA-Ext 权重无许可、由 NC 数据训练。数据：URGENT 现成流水线**不按许可过滤 FSD50K**（`urgent2025_challenge/utils/filter_fsd50k_human_voice.py:10-14`），HF 预合成集整体 NC | 能力已有，只是没接上：许可白名单做成 `scripts/` 下的检查脚本（规则可判，CLAUDE.md 第三节），数据清单每条带许可字段，合成脚本按字段过滤 | P1 |
| 3 | 与人耳相关性有证据，尤其是降噪后语音 | **部分**（离线大模型、系统数少） | 第三方证据是 URGENT 2024 盲测：300 条 × 23 版本（含噪 + 22 个增强系统），P.808，8 人/条（HF 数据卡与 arXiv 2506.01611 明载）。各指标 KRCC/SRCC/LCC 见下表。图本身没写粒度，但和 `urgent2026_challenge_track2/BENCHMARK.md:2-10` 同一测试集的**系统级**数基本吻合（SCOREQ .93、UTMOS .90、DNSMOS .56、NISQA .70），所以图大概率是系统级（推断）。**同表已给出逐句级**：SRCC SCOREQ .79、UTMOS .74、NISQA .70、DNSMOS .64（`BENCHMARK.md:6-10`）。系统级只有 23 个点，SRCC .91 与 .80 的差可能不显著。没有任何指标在**实时低延迟小模型**输出上有证据；CCF AATC 2025 报告 DNSMOS 在 hybrid 系统上与 MOS ρ=−0.8（arXiv 2509.12974 摘要，样本数没写）。吃字类：WAcc（OWSM，不是 Whisper）对 MOS 的 KRCC .50 / SRCC .71 / LCC .30，线性关系弱，而且 MOS 量的是音质不是可懂度 | 能力本身没有：只能自己补，用作者的 AB 选择校准（逐句级数已有，不必再下 `urgent2024_mos` 自己算） | P1 |

**URGENT 2024 增强后语音上，各指标与 P.808 MOS 的相关**（`resources/urgent2024_analysis/mos/mos_correlations.png`，明载；两位核实代理逐柱对过）：

| 无参考 | KRCC / SRCC / LCC | 有参考 | KRCC / SRCC / LCC |
|---|---|---|---|
| UTMOS（SpeechMOS utmos22_strong） | .75 / .91 / .94 | POLQA（闭源） | .64 / .82 / .75 |
| SCOREQ | .75 / .91 / .91 | ESTOI | .61 / .80 / .63 |
| WV-MOS | .54 / .74 / .87 | PESQ | .57 / .77 / .72 |
| NISQA | .64 / .71 / .87 | SDR | .59 / .77 / .54 |
| DNSMOS Pro | .57 / .71 / .88 | LPS | .57 / .76 / .69 |
| VQScore | .49 / .65 / .87 | MCD | .52 / .71 / .72 |
| DNSMOS | .46 / .57 / .83 | SpeechBERTScore · SpkSim · WAcc | SRCC 都是 .71；LCC .75 · .42 · .30 |
| WADA-SNR | .24 / .35 / .11 | LSD | .34 / .49 / .42 |

**跨测试集会翻车**（`resources/urgent2026_challenge_track2/BENCHMARK.md`，明载）：CHiME-7 UDASE（真实录音的增强输出）上，系统级 SRCC：SCOREQ 0.90、Uni-VERSA-Ext 0.90、UTMOS 0.20、DNSMOS_OVRL −0.10；逐句 SRCC：SCOREQ 0.75、UTMOS 0.33、DNSMOS 0.28（`BENCHMARK.md:13-21`）。系统数没核，系统级数字仅供参考，逐句数更站得住。Uni-VERSA-Ext 拿 CHiME-7 当验证集选过模型（同仓 `README.md` 训练命令 `--cv-data`），它的 0.90 有泄漏，SCOREQ 没有泄漏。NISQA 在 CHiME-7 那一行和 SOMOS 那一行逐位相同（`BENCHMARK.md:19` 对 `:52`），疑似复制错误，**不采信**。

## 候选之间

同一批核查项逐格对照。「命门」三列：✅ 原生 · ◐ 部分 · ❌ 没有。速度只写实测或明载，其余写「待测」。

### 自然尺子 · 无参考 MOS

| 候选 | 有参考? | 依赖 · 模型 | Mac | 命门1 | 命门2 | 命门3 | 被讨好的方式 | 判断 |
|---|---|---|---|---|---|---|---|---|
| **SCOREQ natural NR** | 无 | `pip install scoreq`；**默认走 onnxruntime**，不需要 PyTorch（README:36；`src/scoreq/scoreq.py:66` `use_onnx=True`）；natural NR 权重 `adapt_nr_telephone.onnx` 378 MB（Zenodo 15739280） | CPU，待测；训练用 ≤4 s 段，建议切 10–15 s（README:128） | ✅ | ✅ 代码 MIT（README:151，仓库无 LICENSE 文件）/ 权重 CC BY 4.0 | ◐ URGENT 2024 系统级 SRCC .91、逐句 .79；CHiME-7 系统级 .90、逐句 .75，无泄漏 | 训练集 NISQA TRAIN SIM（README:9）是通信失真，不是降噪伪影 | **直接用（整体分首选）** |
| UTMOS22 strong（经 SpeechMOS） | 无 | torch + torchaudio，`torch.hub` 加载，`map_location=cpu`（`SpeechMOS/hubconf.py:31`）；wav2vec2-base 量级 | CPU，待测 | ✅ | ✅ MIT / MIT | ◐ URGENT 2024 系统级 SRCC .91、逐句 .74；CHiME-7 系统级 .20、逐句 .33 | 训练集是 TTS/VC 合成语音（VoiceMOS 2022）；`torch.hub` 要 `trust_repo=True`，会执行远端代码，必须锁 tag 并本地缓存 | **直接用（第二意见）** |
| NISQA v2.0 | 无 | Python 3.9 + PyTorch 1.10；`env.yml` 钉了 `cudatoolkit=10.2`，Mac 上要删掉；权重约 1 MB | CPU 回落（`NISQA_model.py:1036-1039`）；**超过 52 s 报错**（`NISQA_lib.py:2277`） | ✅ | ◐ 代码 MIT / 权重 **NC-SA** | ◐ 分项 NOI/COL/DIS/LOUD 有论文 PCC（arXiv 2104.09494 表 4/5），但测试集是通信失真；URGENT 2024 SRCC .71 | 响度分项能靠响度归一化直接拉高 | **直接用，只作分项诊断**，不进仓，用户自行下载 |
| DNSMOS P.835 | 无 | onnxruntime + librosa；4 个 ONNX 共 2.9 MB，常用的两个（`sig_bak_ovr` + `model_v8`）1.4 MB | **实测**：M5 CPU，10 s 音频 0.05 s；`dnsmos_local.py:53` 不兼容 librosa ≥0.10 | ✅ | ✅ MIT / CC BY 4.0（属解读） | ◐ 自家论文模型级 PCC .94–.98（17 个微软内部模型，arXiv 2110.01763 表 3）；第三方 URGENT 2024 SRCC 只有 .57；CHiME-7 −0.10 | 压得越狠 BAK 越高；exp002 里含噪样例 SIG 反而高于干净样例（n=1） | **直接用，只作 CI 冒烟与业界对照**，不当裁判 |
| UTMOSv2 | 无 | torch ≥2.1、timm、transformers；5 个 fold | 默认 `cuda:0`；**每次随机裁 3 s**（`dataset/ssl.py:46-48`），同一文件多次跑分数不同 | ◐ | ✅ MIT / MIT | ❌ 增强后语音上的数字没找到 | 随机裁剪，多跑几次取最大就能刷分 | 不适合 |
| Uni-VERSA-Ext（URGENT 2026 Track2 基线） | 无 | Python ≥3.10、wavlm-base-plus、HyperPyYAML（加载配置会执行代码，`README.md:33`） | 只认 cuda/cpu（`urgent2026_sqa/infer.py:79`） | ◐ | ❌ 权重无许可，训练数据含 NC 与 Unknown | ◐ 数字好，但有泄漏 | 多头里含 DNSMOS/UTMOS/NISQA 蒸馏头，继承它们的偏好 | 只借鉴 |
| URGENT-PK | 无，但要成对输入 | torch 2.1 + lightning；两两比较 O(K²)（`urgentpk/rank.py:52-99`） | 待测 | ◐ | ❌ 权重无许可 | ◐ 只有系统级，数字没核 | 两个都差时也会选出一个「赢家」 | 只借鉴「成对比较」的思路，用于新旧版本回归 |
| torchaudio SQUIM | OBJECTIVE 无 | torchaudio 2.9（维护期） | CPU | ✅ | ◐ OBJECTIVE CC BY / SUBJECTIVE NC | ❌ | 继承 PESQ/SI-SDR 的毛病 | 只借鉴：真实录音上无参考估 STOI |

### 清楚尺子 · 吃字率

| 候选 | 依赖 | Mac | 命门1 | 命门2 | 命门3 | 被讨好 / 陷阱 | 判断 |
|---|---|---|---|---|---|---|---|
| **whisper.cpp** | C/C++，无 Python；ggml 模型 | Metal 原生；Core ML 编码器比纯 CPU 快 3 倍以上（`README.md:180-181` 明载）；**macOS 14 以下用 Core ML 可能出幻觉**（README 约 :195）；同一台机器的速度待测 | ✅ | ✅ MIT / MIT | ❌ 没有 Whisper-WER 在增强语音上和人耳的相关数据 | 见下方「配方」 | **直接用（主引擎）** |
| mlx-whisper | Python + MLX | Apple GPU；**没有 beam search**（`mlx_whisper/decoding.py:436-437`） | ✅ | ◐ 转换版权重模型卡没写许可 | ❌ | 只能贪心解码 | 备选（Python 侧） |
| WhisperKit | Swift + Core ML，macOS 14+ | ANE；论文延迟 0.46 s（arXiv 2507.10860，流式场景） | ✅ | ✅ MIT / MIT | ❌ | 权重经过压缩，对伪影的反应会变 | 只借鉴 |
| openai-whisper | PyTorch | 只选 cuda/cpu，CPU 上 FP16 退回 FP32（`transcribe.py:132`） | ◐ | ✅ | ❌ | `condition_on_previous_text` 默认开（`transcribe.py:47`），会把被削的词头补回来 | 只作解码语义和归一化器的基准 |
| faster-whisper | CTranslate2 | 只有 CPU，没有 Metal | ◐ | ✅ | ❌ | int8 与 fp16 结果有出入 | 不适合 |
| SenseVoiceSmall（中文） | FunASR / ONNX / GGUF | CPU 或 ONNX，没有 Metal | ◐ | ◐ 需署名；第三方转换版另核 | ❌ | 非自回归，理论上少脑补（没找到文献） | **中文第二意见** |
| jiwer | RapidFuzz | 纯 CPU | ✅ | ✅ Apache-2.0 | 不涉及 | `cer_default` 只做 Strip 和按字切，空格、标点、繁简都算错（`transformations.py:106-111`） | **直接用，加前处理** |

**whisper.cpp 吃字率配方**（三位代理来回核过，改过三处）：
- 全线贪心解码（mlx-whisper 只能贪心，两家要比就得一致）· `-tp 0 -nf`（关掉温度回退；`src/whisper.cpp:7082-7087` 修过回退路径不确定的 bug）· `-mc 0`（关上下文续写，`src/whisper.cpp:7296`）
- 要词级时间戳查词头词尾时用 `-dtw large.v3.turbo`，**必须同时加 `-nfa`**：否则 DTW 会被静默关掉，只在 stderr 留一条警告（`src/whisper.cpp:3823-3826`）。开不开 DTW 的两组数分开记
- 打分：中文先 OpenCC 转简体 → 去标点、空格 → 全角转半角 → 按字切；英文按词切。`process_words`/`process_characters` 返回的删除、替换、插入**三个数分开记**（`jiwer/process.py:126`）。**吃字率 = 删除数 ÷ 参考词数**；替换也要看，因为被削掉词头的词常被转成近音词，jiwer 会记成替换，只数删除会低估吃字；插入记为「脑补率」。不要用 `collect_error_counts`：它按对齐块计数，连续吃掉 3 个词只记 1 次（`alignment.py:229-264`）。中文另要规范化数字（「2024」对「二零二四」会被记成删除加插入），英文用 whisper 的 `EnglishTextNormalizer`（MIT）统一大小写和标点。OpenCC 是 Apache-2.0（gh api 明载）
- 锁模型文件：`ggml-large-v3-turbo.bin` 1.5 GiB，sha1 `4af2b29d7ec73d781377bfd1758ca957a807e941`（`whisper.cpp/models/README.md:60`），走 Metal，不用 Core ML，也不用 q5_0 量化版（547 MiB，结果和全精度不同）
- 打分时**不开 whisper.cpp 的 `--vad`**（`cli.cpp:216-223`）：它会跳段，虚增删除
- 参考文本用数据集自带的人工转写（LibriSpeech、THCHS-30、AISHELL 都有）；没有人工转写时才用「干净原声的转写」，而且要先量一次它和人工转写的差

### 清楚 / 自然尺子 · 有参考指标与音色指标

| 候选 | 量哪把尺子 | 命门1 | 命门2 | 命门3 | 能 Rust 化？ | 判断 |
|---|---|---|---|---|---|---|
| 语音帧能量损失 · 残留噪音电平（`ebur128` crate + 参考信号上的 VAD） | 清楚 | ✅ | ✅ MIT | ❌ 电平计，本来就不对应听感 | ✅ 现成 crate | **直接用**。门限会排除静音块，积分响度量不到停顿里的残留噪音，要在 VAD 标出的非语音段上用短时响度单独算 |
| ESTOI（pystoi） | 清楚（可懂度旁证） | ✅ | ✅ MIT | ◐ URGENT 2024 SRCC .80（对 MOS，不是对可懂度） | ✅ 手写，再拿 pystoi 对拍；URGENT 设 `np.random.seed(0)` 是因为 `pystoi/utils.py:178,185` 会加 EPS 噪声 | **直接用（旁证）** |
| SI-SDR | 清楚 | ✅ | ✅ 公式 | ◐ 只有 bss SDR 的 .77，SI-SDR 本身没有 | ✅ 几行 | **直接用（辅助）**；要先补偿算法延迟、按样本对齐 |
| 有参考 LTASS 差（输出 vs 干净原声，只取语音帧，1/3 倍频程） | 自然（音色偏没偏） | ✅ | ✅ 自己写 | ❌ 没找到 | ✅ | **改造**：Byrne 1994 只当无参考粗参照（数值没有机器可读来源） |
| C50（从 RIR 按定义算） | 场景分层变量 | ✅ | ✅ | 不适用 | ✅ | **改造**：只用来给测试集按混响分档；盲估没有公开模型 |
| LPS（URGENT 音素相似度） | 清楚（内容保留） | ◐ CPU，模型 1.26 GB | ◐ 模型 Apache，但要把 GPL Levenshtein 换成 rapidfuzz（MIT） | ◐ SRCC .76 对 MOS | ❌ | 备选（吃字的第二种量法） |
| PESQ | 清楚 + 自然 | ✅ | ❌ ITU C 代码禁止修改、翻译、分发 | ◐ SRCC .77 | ❌ 法律上不行（crates.io 上的 `pesq` 0.2.0 也落在这个问题里） | 只作用户自装的外部可选项，**不进仓**；发不发布 PESQ 数字要人拍板 |
| ViSQOL v3 | 自然 | 没核 | ✅ Apache-2.0 | 没核 | 没核 | PESQ 的开源替代候选，下一批再核 |
| VERSA | 全部 | ◐ 有 CPU 跑法，没有 MPS（`scorer_shared.py:611`） | ◐ LICENSE 与 pyproject 矛盾 | 自身无 | ❌ | 只借鉴指标清单 |

### 轻快尺子 · 附加延迟（应大总管 / pipeline 线要求列入）

**现成工具全部没覆盖**：URGENT/DNS/VERSA 都不量延迟、每帧耗时和 CPU（`urgent2026_challenge_track1/evaluation_metrics/eval_all.sh:21-38`，源码）；pipeline 线报告参考仓库里也都没有附加延迟的实测。设计稿 §13 只写了「输入输出互相关；每帧耗时 p99.9」（`vault/raw/20261029-voice-denoise-design.md` §13 第二层表格）。下面是 evaltool 列的四种量法，**全部要自己写，没有一种在本批验证过**：

| 量法 | 量到哪一截 | 精度（推理所得，待实测） | 自动化 | 进 CI？ |
|---|---|---|---|---|
| ① 离线文件进、文件出：一段**干净语音**过同一套处理代码，算输入输出互相关峰值，再和「帧长 + lookahead」的理论值对一次（脉冲或 chirp 可能被降噪模型当噪音压掉，峰值会消失或漂移） | 算法延迟（帧长 + lookahead + 重采样） | 1 个采样点 | 全自动 | 是 |
| ② cpal 两端回调时间戳（pipeline 线 M0 打算用）：输入回调的捕获时刻 vs 输出回调的播放时刻 | 本进程内的缓冲 + 处理 | 取决于 CoreAudio 报的时间戳，回调周期量级 | 本机自动 | 否（要真设备） |
| ③ 回环：往输入里打一个 click，从虚拟麦克风的输出录回来，算互相关 | 端到端，含 HAL 和驱动 | 1 个采样点，但受设备时钟漂移影响 | 半自动；**本批规矩不碰音频设备** | 否 |
| ④ 设备上报值：CoreAudio 的设备延迟 + 安全偏移 + 缓冲帧数相加 | 驱动自报的那一截 | 取决于驱动是否如实上报 | 全自动 | 否 |

每帧耗时 p99.9 / 最大值、掉帧、soak、CPU 占用同样没有现成评测工具，要在 Rust 侧自己打点（model 线 exp001 正在做每帧计时，可复用它的方法）。

### 旁路交叉淡化时长（应大总管 / pipeline 线收尾转交列入）

背景：管线在推理卡顿时切回原声（旁路），切换时要做交叉淡化（pipeline 线转交）。**现成评测工具都不覆盖「切换点」这种局部事件**，URGENT/DNS 都是整段打分。以下是 evaltool 列的评法，**没有一条在本批验证过，时长也不在这里定值**。

**先要知道的一条（推理所得）**：处理路径有算法延迟（帧长 + lookahead），原声路径没有。如果切换时不先把原声按同样的延迟对齐，交叉淡化前后会出现等于「延迟差」那么长的**重复**（切到原声时）或**跳过**（切回处理时）。会议里听到的就是重字或跳字。所以先评「对齐了没有」，再评「淡化多长」。

| 要判的毛病 | 用哪把尺子 | 怎么量 | 进 CI？ |
|---|---|---|---|
| 时间跳变（跳字 / 重字的根因） | 清楚 | 离线测试里在已知帧号强制触发旁路，把输出在切换点前后各一小段和参考做互相关，量跳了多少个采样点；判据是 0 个采样点 | **是**：确定性，规则可判 |
| 咔嗒声 / 不连续 | 自然 | 切换点附近的窗口里量高频能量突增或频谱通量，和不切换的同段输出比；NISQA 的 DIS（不连续）分项作旁证（权重 NC，只在本机） | DSP 判据**进**；NISQA 不进 |
| 跳字 / 重字（听感层面） | 清楚 | 用 whisper.cpp 加 DTW 词级时间戳（`-dtw` 须加 `-nfa`），只统计切换点 ±0.5 s 内的删除和插入 | 否（本机层） |
| 淡化太长带来的「噪音漏回来」 | 清楚 | 切换窗口内的残留噪音电平（同上方 `ebur128` 短时响度） | 是 |
| 整体听感 | 人耳 | 机器挑出切换点最多的几段，放进 AB 页 | 否 |

**时长候选**：5 / 10 / 20 / 50 ms，各配等功率和线性两种曲线，在同一组强制旁路的测试段上扫一遍，按上表各项出一张小矩阵。这些候选是 evaltool 拟的扫描范围，没有出处，由下一批原型定值。

**缺口**：P2。方法清楚、只差代码，跟原型的「强制旁路」测试钩子一起写。

## 二、它们没做的

| 找过什么 | 结论 | 是哪一种 |
|---|---|---|
| 任何指标在**实时低延迟小模型**输出上与人耳的相关性 | **无人涉足**：URGENT 2024 的 22 个系统多是离线大模型（`urgent2024_analysis` 与 arXiv 2506.01611） | 没顾上做：挑战赛关心的是离线通用增强 |
| URGENT 2024 相关性的逐句级数字 | **有**：`urgent2026_challenge_track2/BENCHMARK.md:2-10`（初稿误判为没有，怀疑派自审时指出） | — |
| Whisper 系吃字率与人耳可懂度的相关性 | **无人涉足**：只有 OWSM WAcc 对 MOS（LCC .30）。降噪前处理对 Whisper WER 的影响：arXiv 2512.17562 只测了 MetricGAN+ 一种降噪器，在 4 个 ASR 上 40 组配置全部变差；不能推广到所有降噪器 | 做不到的成分大：ASR 有语言模型先验，和人耳是两套机制 |
| NISQA 分项在**降噪后**语音上的相关性 | 有但不适用：论文测试集是通信失真和真实通话 | 没顾上做 |
| 附加延迟、每帧耗时、soak、旁路切换点的评测工具 | **无人涉足**（见上方两节） | 没顾上做：挑战赛都是离线打分 |
| Apple Silicon 上各打分器的同机速度对比 | 只有 DNSMOS 有实测（exp002）；whisper.cpp 只有「Core ML 比 CPU 快 3 倍以上」这类相对数 | 没顾上做 |
| onnxruntime CoreML 执行器跑 DNSMOS 能快多少 | 没试（exp002 实际用的是 CPU 执行器） | 没顾上做 |
| Byrne 1994 LTASS 的机器可读数据 | 没找到 | 没顾上做 |
| C50 盲估的公开模型（Gamper MMSP 2020） | 只有论文，没有公开代码和权重 | 没顾上做 |
| PESQ 专利是否已全部过期 | 没找到权威说法 | — |
| 带人评 MOS 的「降噪后会议语音」数据集（中文） | 没找到；英文只有 urgent2024_mos（NC） | 没顾上做 |
| **babble / 单个干扰说话人**（真难题第一条，咖啡馆的头号噪音） | 现成噪音源（FSD50K、MUSAN、DNS5）里没有单独成格；设计稿 §13 的「残留部分与干扰者声纹的相似度」没有现成实现，声纹模型的许可也没核 | 没顾上做：合成参数表单设 babble 格子，干扰语音取 LibriSpeech 其他说话人（CC BY 4.0） |
| **笑声、附和声**（设计稿 §3、§13） | 没有公开测试集；Whisper 删除率量不到，ASR 常把「嗯」和笑声丢掉或归一化掉 | 没顾上做：在作者自录集里标出这类片段，量这些片段上的能量损失 |
| **背景忽大忽小**（意向书「自然」尺子） | 没有现成指标；设计稿 §13 提了「残留噪音电平随时间的波动」 | 能力已有没接上：非语音段短时响度（`ebur128`）的方差和跳变次数，成本很低，已列进 `decision.md` |
| **经会议软件编码后的声音**（对方实际听到的） | 没有。URGENT 的编码和丢包失真模拟的是输入端，不是输出后再编码 | 没顾上做：输出端加一道 Opus 编解码后再打一遍分。libopus 官方称 BSD，GitHub API 显示 NOASSERTION，待核 |
| **16 kHz 以上的伪影** | DNSMOS、SCOREQ、Whisper 都按 16 kHz 打分，8 kHz 以上的频段看不见 | 盲区：用 48 kHz 的有参考 LTASS 差补，其余写明是盲区 |
| **真实录音上的有参考指标与吃字率** | DNS5 dev 没有干净参考也没有转写；AliMeeting 有近场耳麦录音和转写，但多人重叠；作者自录集既没参考也没转写 | 见 `decision.md`「真实录音怎么评」 |

### 该不该自己跑一个

| 缺口 | 判据 | 结论 |
|---|---|---|
| 指标在我们自己的模型输出上和人耳是否一致 | 多年没解决，且分布特殊（实时小模型） | **跑它**：下一批原型里，每次 AB 时同时记下各指标的打分，攒够 20–30 对后算一致率 |
| 逐句级相关 | 搜一下就有：`BENCHMARK.md:2-21` 已给出 | **不用跑** |
| 各指标的抖动（同一构建跑两次的差）与门禁阈值 | 输出清楚，不知道该设多少 | **跑它**：下一批第一件事，回归门禁的阈值要由它定（`decision.md`「回归门禁」） |
| 各打分器在 Mac 上的速度 | 搜不到 | **跑它**：exp002 的第二轮，加 SCOREQ、SpeechMOS、NISQA、whisper.cpp，量 10 分钟音频 |
| 附加延迟 ① | 方法清楚，只差代码 | 跟原型一起写，不单独立项 |
| 旁路交叉淡化时长 | 方法清楚，只差代码 | 跟原型的强制旁路测试钩子一起写，扫 5–50 ms |

---

**下一步**：压成 `decision.md`。
