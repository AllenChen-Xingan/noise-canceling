# 取舍卡 · 评测工具链用哪些现成的

> 2026-10-29 · evaltool 起草。逐格证据见 `gap.md`，来源登记见 `sources.md`，命门原文见 `must-know.md`（AI 起草，待人改）。
> ⚠️ 最后一栏「需要你拍板的地方」只列题，**拍了吗一律 ☐**。

## 这次不可妥协的几条

1. 本机离线跑，不上传音频，Apple Silicon 上能跑
2. 许可证允许开源评测工具；数据许可不妨碍发布微调后的权重
3. 与真人听感的相关性有证据，尤其是对降噪后的语音

## 我选择什么

> 每把尺子都用「一个主指标 + 一个交叉指标」，分两层跑：**CI 层**只放许可干净、跑得快的；**本机层**再加需要 PyTorch 的模型。任何一个无参考 MOS 都只用来筛选，不当裁判。

| 尺子 | 主指标（实现） | 交叉 / 诊断 | 跑在哪 | 出处 |
|---|---|---|---|---|
| **清楚 · 吃字** | 删除率：whisper.cpp large-v3-turbo + Metal，贪心、`-tp 0 -nf -mc 0`；jiwer `.deletions`，插入另记为「脑补率」 | 中文加 SenseVoiceSmall 做第二意见：两家**都**看到删除增加，才判定为吃字 | 本机层 | `gap.md`「吃字率配方」；`whisper.cpp/src/whisper.cpp:3823-3826,7082-7087,7296`；`jiwer/process.py:126` |
| **清楚 · 压噪与人声** | 语音帧能量损失 + 非语音段残留噪音电平（Rust，`ebur128` crate + 参考信号上的 VAD） | ESTOI（pystoi，SRCC .80）· SI-SDR（手写） | CI 层 | crates.io `ebur128`（MIT）；`urgent2024_analysis/mos/mos_correlations.png` |
| **自然 · 整体** | SCOREQ natural NR（代码 MIT，权重 CC BY 4.0） | UTMOS22 strong（经 SpeechMOS，MIT） | 本机层 | URGENT 2024 SRCC 都是 .91；CHiME-7 系统级 SRCC SCOREQ .90、UTMOS .20（`urgent2026_challenge_track2/BENCHMARK.md`） |
| **自然 · 分项与音色** | NISQA 分项 NOI/COL/DIS/LOUD（权重 NC-SA，用户自行下载，不进仓）+ 有参考 LTASS 差 + LUFS | DNSMOS P.835 SIG/BAK（CC BY 4.0，CI 冒烟与业界对照） | NISQA 本机层；LTASS、LUFS、DNSMOS 进 CI 层 | `NISQA/README.md:166`；exp002 实测 DNSMOS 10 秒音频 0.05 秒 |
| **自然 · 人耳** | 本机静态 AB 页（借 P.808 的 CCR 与陷阱题设计，不上传，不用 MTurk） | URGENT-PK 的「成对比较」思路，用于新旧版本回归 | 本机，人听 | `P.808/src/master_script.py:868-873`（本地预览会关掉提交，要自己写收分） |
| **轻快** | 附加延迟 ①：离线脉冲过同一套处理代码，算互相关峰值 | ② cpal 两端回调时间戳（pipeline 线 M0）；每帧耗时 p99.9 / 最大值，Rust 打点 | ① 进 CI 层；② 只在本机 | 现成工具全部不覆盖（`gap.md`「轻快尺子」） |

**代码放哪**：信号处理类指标（能量损失、残留噪音、LUFS、SI-SDR、ESTOI、LTASS、C50、延迟、每帧耗时）放 **Rust 侧**，和实时运行用同一套处理代码；Python 版（pystoi、pyloudnorm、torchmetrics）只用来对拍。模型类打分器（SCOREQ、UTMOS、NISQA、DNSMOS）放 **Python 侧**，读 Rust 写出的 wav，输出 JSON。whisper.cpp 走命令行，两边都能调。汇总按场景格子出分，学 URGENT 的 `get_breakdown.py`（按采样率、SNR 每 5 dB 一档、失真类型分格，`urgent2026_challenge_track1/evaluation_metrics/get_breakdown.py:10-117`）。

**合成考题**：自己写生成器，学 URGENT 2025「先出参数表，再按表生成」两段式。参数范围见 `urgent2025_challenge/conf/simulation_train.yaml:58-110`：SNR −5~20 dB，丢包每包 20 ms，丢包率 5–25%。削波照 URGENT 做，因为 DNS 合成脚本其实不做削波（`noisyspeech_synthesizer_singleprocess.py:20` 只 import 不调用）。不引入 ESPnet。

**一次完整评测多久**：只有 DNSMOS 有实测，实时倍率约 0.005（exp002），1 小时音频约 20 秒，这是推算。SCOREQ、UTMOS、NISQA、whisper.cpp 在 Mac 上都**待实测**；官方也没给「一次完整评测要多久」（URGENT 三届 README 与论文都没写，`gap.md` §二）。CI 层的指标全是 DSP 加 3 MB 的 ONNX，量级是分钟；本机层要等实测。

**最小评测集**（许可都已核，没下载）：LibriSpeech test-clean 346 MB（CC BY 4.0，带转写）· 中文二选一，待拍板 · FSD50K eval 6.2 GB（评测时 NC 片段也能用）· SLR26 178 MB + SLR28 1.3 GB（Apache）· 真实录音：DNS5 dev testset 2.9 GB（无参考）、AliMeeting Eval 3.42 GB（CC BY-SA，中文会议）、AMI（CC BY 4.0，英文会议，大小没核）· 加上作者自己录的咖啡馆真实测试集。

## 我放弃什么

> 放弃把 DNSMOS 当总分（第三方增强语音上 SRCC .57，真实录音上 −0.10），放弃 PESQ 进仓，放弃整包搬 URGENT/VERSA 评测脚本（ESPnet、默认 cuda、权重许可不明），也放弃所有「用 NC 权重或 NC 数据的输出当训练目标」的做法。

## 依据来自哪一类

- [x] 自有积累：设计稿 §13 的五层评测框架仍算数，作骨架；exp002 实测确认 CI 层能跑
- [x] 他人积累：19 个仓库浅克隆，逐条核查到文件行号；URGENT 2024 相关图、Track2 BENCHMARK、各 HF / Zenodo / OpenSLR 许可页。Workflow 里每路都配了怀疑派，推翻了 11 条原结论，包括漏掉的 SCOREQ、DTW 必须加 `-nfa`、jiwer 计数方法、MUSAN 逐文件许可、MLS 不是 CC0、DNSMOS「唯一干净」不成立
- [x] 自己跑的：exp002 确认 DNSMOS 不需要 GPU 或 PyTorch，10 秒音频 0.05 秒；顺带发现 `dnsmos_local.py:53` 不兼容新版 librosa，并改了一条实现约束：评测代码自己负责重采样

---

## 拿两把尺子量一遍

### 一、复用分级：能用到什么程度

| 来源 | 权限 | 这次实际用到哪一步 |
|---|---|---|
| **自家的** | 可照搬 | 设计稿 §13 作骨架；exp002 的 `smoke.py` 可直接长成 CI 冒烟脚本 |
| **外部的** | 只读借鉴 | 当依赖调用：DNSMOS ONNX、SCOREQ、SpeechMOS、NISQA（用户自装）、whisper.cpp、jiwer、pystoi、`ebur128`。只学思路、自己写：URGENT 生成器、`get_breakdown`、P.808 CCR 页面。只当参照：VERSA、URGENT-PK、Uni-VERSA-Ext。不碰：PESQ 的 C 代码、GPL 的 Levenshtein |

⚠️ 交付前已回 `resources/` 比对：本目录五份文件里没有代码片段，只有行号引用；`smoke.py` 只调用 `ComputeScore`，没有复制。

### 二、满意与最优：什么时候够

**这次的停止线**：每把尺子都有主指标和交叉指标，每个指标三条命门都有「有 / 部分 / 没有」加出处，剩下的空缺都能归到「待实测」或「待人拍板」。达到了。没有继续去核的：ViSQOL、WV-MOS、DNSMOS Pro、Distill-MOS 的相关性和 Mac 路径，因为它们在 URGENT 2024 上都不比 SCOREQ/UTMOS 强（SRCC .65–.74），也不改变首选。

## 支持它的最强依据

> URGENT 2024 盲测（300 条 × 23 个版本，P.808，每条 8 人）上，SCOREQ 和 UTMOS 与人耳 MOS 的 SRCC 都是 .91，高于所有有参考指标（POLQA .82、ESTOI .80、PESQ .77）和 DNSMOS（.57）（`resources/urgent2024_analysis/mos/mos_correlations.png`，两位代理逐柱核对）；换到真实录音的 CHiME-7 UDASE，SCOREQ 系统级仍有 .90，而且它没有训练或验证集泄漏（`urgent2026_challenge_track2/BENCHMARK.md`；Uni-VERSA-Ext 的 .90 有泄漏）。另外，WAcc 对 MOS 的 LCC 只有 .30，说明「清楚」和「自然」确实要分两把尺子量，吃字率不能拿 MOS 代替。

## 反对它的最强依据

> 全部相关性证据都来自 **23 个离线增强系统**，粒度没写明，也没有显著性检验；.91 和 .80 的差距在 n=23 下可能不显著（arXiv 2506.01611 没给检验）。同一个 UTMOS 换到 CHiME-7 就掉到 .20，说明换个分布排序就会翻车，而我们的实时小模型正是另一个分布。CCF AATC 2025 还报告 DNSMOS 在 hybrid 系统上与 MOS 负相关 ρ=−0.8（arXiv 2509.12974）。吃字率这条更弱：Whisper 删除率和人耳可懂度之间**没有任何**相关性证据，它可能量的是 ASR 对伪影有多敏感，而不是人听不听得清。

**这条反对意见不改变选择，但改变用法**：所有自动指标只用来排序和挑片段，结论由作者 AB 来下（与意向书真相源权重第 2–4 位一致）。

## 仍未补齐的缺口

> **这些指标在我们自己的实时降噪输出上，和作者的耳朵是否一致。** 补法：下一批原型每做一次 AB，同时记下各指标打分，攒到 20–30 对后算一致率；不一致的指标降权。顺带用 `urgent2024_mos` 在本机算一次逐句相关作对照，下载前先报大总管。

## ⚠️ 需要你拍板的地方

| # | 需要拍板的 | 选项 | 代价 | 我的建议 | 默认（不拍时） | 拍了吗 |
|---|---|---|---|---|---|---|
| 1 | 真实测试集用哪些麦克风录（设计稿 §13 场景矩阵：内置 / AirPods / USB 耳机 / 专业麦克风） | A 只用手头已有的 · B 再买或借 1–2 个 · C 四类凑齐 | B 和 C 要花钱、花时间；A 覆盖不全 | 先 A：内置麦克风加 AirPods 录咖啡馆，跑通飞轮后再按评测矩阵暴露的弱项补 | A | ☐ |
| 2 | 真实录音里会有旁人的声音，怎么存 | A 只存本机、不进仓、不外发 · B 只把剪过、没有旁人声音的片段放进开源测试集 | A 别人复现不了；B 要人工剪 | A，开源测试集只用公开数据合成（铁律 13：个人隐私数据不入库） | A | ☐ |
| 3 | 许可不干净的打分器怎么处理：NISQA 权重 NC-SA；PESQ 的 C 代码禁止分发，评估结果也不许对外商用（`dsp.c` PERMITTED USE 1.iii） | A 不用 · B 作「用户自装的可选项」，只出内部数字，不进 CI、不进发布报告 · C 照常用 | A 少了分项诊断；C 有法律风险 | NISQA 选 B；PESQ 选 A，需要同类指标时下一批核 ViSQOL（Apache） | NISQA B · PESQ A | ☐ |
| 4 | 中文评测集用哪个 | A CommonVoice zh-CN（CC0，大小没核）· B THCHS-30（标 Apache，页面又写「对学术用户免费」，6.4 GB + 1.9 GB）· C 两个都要 | B 许可有矛盾，不能进要发布的东西 | A 进开源测试集，B 只在本机做对照（它自带 0 dB 含噪测试集） | A | ☐ |
| 5 | 评测数据下载与存放：最小评测集约 15–20 GB，超过本批 500 MB 的线；铁律 8 要求大文件放仓外 | A 放本机 `~/` 下的仓外目录 · B 放 NAS / 云盘，仓里只留路径和哈希 | A 没有远端副本；B 要先有 NAS | 先 A，仓里记路径、哈希、来源 URL 和许可，换机时照清单重下（都是公开数据，可重建） | A | ☐ |
| 6 | 以后的模型微调能不能用带 SA（相同方式共享）条款的数据：AliMeeting CC BY-SA、DEMAND CC BY-SA 3.0 | A 只用于评测 · B 也进训练 | B：SA 会不会传染到权重，目前没有定论 | A | A | ☐ |

---

## 收尾三件

1. **判断落回真相源**：这张卡由人拍板后，再整理进 `vault/truth/`。本批手下只写 004，不自己去建 `vault/truth/`
2. **交付前回一趟参考区**：已查，没有照抄，见上文「复用分级」
3. **放行**：选了什么 → 上面那张表；舍了什么 → DNSMOS 当总分、PESQ 进仓、整包搬 URGENT；哪里要拍板 → 上面 6 题
