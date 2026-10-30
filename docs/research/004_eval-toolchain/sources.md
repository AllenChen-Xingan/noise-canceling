# 两类来源 · 评测工具链用哪些现成的

> 2026-10-29 · evaltool。取材方式：19 个仓库浅克隆进 `resources/`（只读），加 Workflow `wf_14ba77e5-34d`（5 路按同一张核查表检索 + 5 路怀疑派对抗核实，共 10 个代理），再由 evaltool 补查 SCOREQ、AMI。
> 出处标注三类：**明载**（官方页面 / README / LICENSE）· **源码**（本机读 `resources/` 所得，带行号）· **二手**（检索摘要、转述）。
> 每个候选逐格的核查结果（三条命门 · 尺子 · 依赖 · Mac · 许可 · 相关性 · 被讨好的方式）见 `gap.md`「候选之间」。

---

## 一、自有积累

`scan-own.sh "DNSMOS"`、`scan-own.sh "whisper"` 扫了 `B-MyCreate/` 下全部项目：**本机旧项目里没有做过语音评测**，只命中本项目自己的文件。

| 是什么 | 在哪 | 还算数吗 | 判据 |
|---|---|---|---|
| 设计与参考汇总 §5/§9/§12/§13（评测五层、场景矩阵、专项指标表） | `vault/raw/20261029-voice-denoise-design.md:213-222,391-421,532-627` | 算，作方向；指标选择被本次推翻一部分 | 人整理的材料，标【你定的】的只有音色目标（§9）；「整体质量用 DNSMOS/NISQA/UTMOS」被 URGENT 2024 数据削弱（见 gap §一 #3） |
| exp002 打分冒烟（DNSMOS + PESQ/STOI/SI-SDR + LUFS） | `docs/experiments/exp002-eval-smoke/README.md` | 算 | 本机实测，大总管已抽查验收；每类只有 1 段样例，只证明「能跑、多快」，不证明「准」 |

## 二、他人积累

角色：**要比的** = 逐条核查；**要学的** = 只读架构；**零件** = 一两行说清能接在哪。还活着吗 = GitHub 最近一次 push。

### 无参考 MOS（自然尺子）

| 是什么 | 在哪 | 协议（代码 / 权重） | 角色 | 读多深 | 还活着吗 |
|---|---|---|---|---|---|
| DNSMOS P.835 + P.808 | `resources/DNS-Challenge/DNSMOS/` | MIT / CC BY 4.0（`README.md:247-250` 明载；ONNX 在「其他内容」里，属解读） | 要比的 | 逐条核查 + 本机实测 | 2024-07 |
| NISQA v2.0 | `resources/NISQA` | MIT / **CC BY-NC-SA 4.0**（`README.md:164-166` 明载） | 要比的 | 逐条核查 | 2024-12 |
| UTMOS22 strong（原仓） | `resources/UTMOS22` | MIT / MIT（HF Space `sarulab-speech/UTMOS-demo` 元数据 `license: mit`，明载） | 要比的 | 逐条核查 | 2024-04 |
| SpeechMOS（UTMOS22 打包，URGENT 2024 分析实际用的就是它） | `resources/SpeechMOS` | MIT / MIT（`hubconf.py:14` 明载权重派生自 UTMOS22、MIT） | 要比的 | 逐条核查 | 2023-10 |
| UTMOSv2 | `resources/UTMOSv2` | MIT / MIT（HF 模型卡明载） | 要比的 | 逐条核查 | 2026-04 |
| **SCOREQ**（核实时补进，原检索漏了） | github.com/alessandroragano/scoreq（未克隆） | MIT（README:151 明载；**仓库无 LICENSE 文件**）/ CC BY 4.0（默认用的 ONNX 在 Zenodo 15739280，每个 378 MB；PyTorch 版在 13860326；都明载） | 要比的 | 读 README 与 `src/scoreq/scoreq.py` | 2025-08 |
| URGENT-PK（成对排序） | `resources/URGENT-PK` | Apache-2.0 / **无模型卡、无许可**（HF `Twinkzzzzz/URGENT-PK`，明载为空），训练数据 NC | 要学的 | 只读思路 | 2026-08 |
| URGENT 2026 Track2 基线 Uni-VERSA-Ext | `resources/urgent2026_challenge_track2` | pyproject 写 MIT、**无 LICENSE 文件** / 无许可，训练数据含 NC 与 Unknown（`README.md:139-166`） | 要比的 | 逐条核查 | 2026-06 |
| torchaudio SQUIM | docs.pytorch.org/audio/2.9.0 | BSD-2 / OBJECTIVE CC BY 4.0、SUBJECTIVE CC BY-NC 4.0（明载） | 零件 | 一两行 | torchaudio 2.9 进入维护期（issue #3902） |
| Distill-MOS · DNSMOS Pro · WV-MOS · VQScore | 未克隆 | Distill-MOS 模型卡 MIT（二手经 gh api）；其余没核 | 零件 | 只看 URGENT 2024 图上的数字 | — |

### 吃字率（清楚尺子）

| 是什么 | 在哪 | 协议（代码 / 权重） | 角色 | 读多深 | 还活着吗 |
|---|---|---|---|---|---|
| openai-whisper | `resources/whisper` | MIT / MIT（`README.md:160` 明载） | 要比的 | 逐条核查 | 2026-08 |
| whisper.cpp | `resources/whisper.cpp` | MIT / MIT（HF `ggerganov/whisper.cpp` `license:mit` 明载） | 要比的 | 逐条核查 | 2026-10-29 |
| mlx-whisper | `resources/mlx-examples/whisper` | MIT / **模型卡没写**（HF `mlx-community/whisper-large-v3-turbo` 明载为空） | 要比的 | 逐条核查 | 2026-04 |
| WhisperKit（argmax-oss-swift） | github.com/argmaxinc/argmax-oss-swift（未克隆） | MIT / MIT（HF `argmaxinc/whisperkit-coreml` 明载） | 要比的 | 读 README | 2026-10-27 |
| faster-whisper | github.com/SYSTRAN/faster-whisper（未克隆） | MIT / MIT（二手） | 要比的 → 排除 | 读 README | — |
| SenseVoiceSmall（中文第二意见） | github.com/FunAudioLLM/SenseVoice（现跳转 QwenAudio/SenseVoice） | MIT / FunASR Model License v1.1（允许商用、需署名；**澄清只覆盖官方权重**，README 约 :503-506 明载） | 要比的 | 读 README | 2026-09 |
| jiwer | github.com/jitsi/jiwer（未克隆，经 gh api 读源码） | Apache-2.0 | 零件 | 读 `alignment.py`、`process.py`、`transformations.py` | 2026-09 |
| OpenCC（中文繁简规范化） | github.com/BYVoid/OpenCC | Apache-2.0（gh api 明载） | 零件 | 一行 | — |
| libopus（输出端编码模拟，候选） | github.com/xiph/opus | 官方称 BSD；GitHub API 显示 NOASSERTION，待核 | 零件 | 一行 | — |

### 有参考指标与音色指标

| 是什么 | 在哪 | 协议 | 角色 | 读多深 | 还活着吗 |
|---|---|---|---|---|---|
| python-pesq（ITU-T P.862） | `resources/PESQ` | 包装 MIT，**内嵌 ITU C 源码禁止修改、翻译、分发、商用**（`pesq/dsp.c:34-75` 源码） | 零件 → 只作外部可选 | 逐条核查许可 | 2026-03 |
| pystoi（STOI/ESTOI） | `resources/pystoi` | MIT | 零件 | 读 `stoi.py` | 2023-12 |
| SI-SDR | 公式（Le Roux 2019）；torchmetrics Apache-2.0、asteroid MIT、fast_bss_eval MIT（PyPI classifier） | — | 零件 | 一两行 | — |
| pyloudnorm / Rust `ebur128` crate（BS.1770） | `resources/pyloudnorm`；crates.io `ebur128` 0.1.11 | MIT / MIT（crates.io 明载，2026-10 仍更新） | 零件 | 一两行 | 2026-01 / 2026-10 |
| LTASS（Byrne 1994，JASA 96(4):2108-2120） | 论文表格；**没找到机器可读数据** | 数值再发布许可没核 | 零件 | 二手索引 | — |
| C50 | 定义式从 RIR 算；python-acoustics `room.py:221,251`（BSD-3，**已归档**） | — | 零件 | 一两行 | 归档 |
| VERSA | `resources/versa` | **LICENSE 写 Apache-2.0，pyproject/README 写 MIT，自相矛盾**（`LICENSE:1`、`pyproject.toml:11`） | 要学的 | 只读指标清单 | 2026-10-29 |
| LPS / SpeechBERTScore（URGENT 内容保留指标） | `resources/urgent2026_challenge_track1/evaluation_metrics/` | 脚本 Apache；LPS 依赖 **GPL-2.0+ 的 Levenshtein**；wav2vec2-espeak 模型 Apache（1.26 GB）；mHuBERT-147 **CC BY-NC-SA** | 零件 | 逐条核查许可 | 2025-11 |
| ViSQOL v3（核实时提出的 PESQ 开源替代） | github.com/google/visqol（未克隆） | Apache-2.0 | 零件 | 只核了许可，相关性没核 | 2025-05 |
| POLQA（P.863） | 闭源商用（`urgent2024_analysis/README.md:156` 明载「proprietary GUI program」） | 商用授权 | 排除 | 一行 | — |

### 评测管线（要学的）

| 是什么 | 在哪 | 协议 | 角色 | 读多深 | 还活着吗 |
|---|---|---|---|---|---|
| URGENT 2025 数据模拟（七种失真） | `resources/urgent2025_challenge/simulation/`、`conf/simulation_train.yaml` | Apache-2.0 | 要学的 | 只读架构 + 参数范围 | 2025-05 |
| URGENT 2026 Track1 评测脚本 | `resources/urgent2026_challenge_track1/evaluation_metrics/` | Apache-2.0 | 要学的 | 只读架构（`eval_all.sh`、`get_breakdown.py`） | 2025-11 |
| URGENT 2024 指标与 MOS 相关性分析 | `resources/urgent2024_analysis/`（`mos/mos_correlations.png`、`README.md:41-205`）；论文 arXiv 2506.01611 | MIT；MOS 数据 `urgent2024_mos` **CC BY-NC-SA 4.0**（HF 明载，6900 条） | 要比的（命门 3 证据） | 逐条核查 | 2025-08 |
| DNS Challenge 合成脚本 | `resources/DNS-Challenge/noisyspeech_synthesizer_singleprocess.py`、`.cfg` | MIT | 要学的 | 只读架构 | 2024-07 |
| ITU-T P.808 众包工具 | `resources/P.808`（1.1 GB，大总管：先留着） | MIT；附带片段多为 CC BY 4.0 | 要学的 | 只读架构 | 2026-10 |

### 数据（零件，只查许可、大小、获取方式，没下载）

| 是什么 | 在哪 | 协议 | 大小 | 进训练？ |
|---|---|---|---|---|
| DNS5 噪音 + RIR / dev testset | `resources/DNS-Challenge/download-*.sh` | 按子集：Freesound 只取 CC0、AudioSet CC BY 4.0、RIR Apache（`README.md:264-290`） | 噪音+IR 约 39 GB（`noise-ir.sh:7`）；dev testset 2.9 GB（`dev-testset.sh:8`） | 可（AudioSet 音频本身的版权没说清） |
| FSD50K | zenodo.org/records/4060432 | 逐片段：CC0 / CC BY / **CC BY-NC 11.8%** / Sampling+ | dev 18.3 + eval 6.2 GB | 过滤 NC 后可 |
| MUSAN | openslr.org/17 | CC BY 4.0，**逐文件署名**（不是「整包一个许可」） | 11 GB | 可（要逐条署名） |
| SLR26 / SLR28 RIR | openslr.org/26、/28 | Apache 2.0（SLR28 里的真实 RIR 来自 RWCP/REVERB/AIR，原始许可没核） | 178 MB / 1.3 GB | SLR26 可；SLR28 部分 |
| BUT ReverbDB | speech.fit.vut.cz | CC BY 4.0 | 仅 RIR 8.7 GB | 可 |
| LibriSpeech test-clean（英文，带转写） | openslr.org/12 | CC BY 4.0 | 346 MB | 可 |
| VCTK 0.92 | datashare.ed.ac.uk 10283/3443 | CC BY 4.0（`license_text.txt` 明载） | 10.9 GB | 可 |
| EARS · DAPS | GitHub / Zenodo | **CC-NC 4.0 · CC BY-NC 4.0** | 61 GB · 16.1 GB | 只能评测 |
| THCHS-30（中文，带 0 dB 含噪测试集） | openslr.org/18 | 标 Apache 2.0，页面又写「对学术用户免费」，**矛盾** | 6.4 GB 整包 + test-noise 1.9 GB | 不建议 |
| AISHELL-1 | openslr.org/33 | 同上，矛盾 | 15 GB | 不建议 |
| MAGICDATA · KeSpeech | openslr.org/68 · NeurIPS 2021 | CC BY-NC-ND · 学术用（二手） | — | 只能评测 |
| CommonVoice（含 zh-CN） | URGENT 2026 数据页 | CC0 | zh-CN 单语种大小没找到 | 可 |
| MLS | openslr.org/94 | **CC BY 4.0**（URGENT 页写 CC0，不一致） | — | 可 |
| AliMeeting Eval（真实中文会议） | openslr.org/119 | CC BY-SA 4.0 | 3.42 GB | 评测可；进训练有 SA 争议 |
| AMI Meeting Corpus（真实英文会议，evaltool 补查） | groups.inf.ed.ac.uk/ami/corpus/license.shtml | CC BY 4.0（明载） | 没核 | 可 |
| URGENT 2026 HF 预合成集 | huggingface.co/datasets/lichenda/urgent26_track1_universal_se | **整体 CC BY-NC 4.0** | 154 GB | 不可 |
| urgent2024_mos（增强后语音 + 人评 MOS） | huggingface.co/datasets/urgent-challenge/urgent2024_mos | CC BY-NC-SA 4.0 | 没核 | 只作校准 |

⚠️ 读 → 自己写。GPL（Levenshtein）只参考不链接；PESQ 的 ITU C 代码不许修改、翻译、分发，**不能进仓、不能 Rust 化**。
⚠️ DNS 的几个测试集下载脚本嘴上说默认只做 dry run，实际有没注释掉的 `wget`/`curl`（`download-dns5-dev-testset.sh:26`、`download-dns5-blind-testset.sh:25`、`download_dns_v2_v3_blindset.sh:27`、`download-dns-challenge-5-paralinguistic-train.sh:18`）。**照着跑就会真下载。**
