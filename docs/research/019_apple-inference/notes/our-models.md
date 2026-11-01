# 019 · 我们要上用户电脑的模型盘点（仓内事实）

> 2026-10-31 · 019 调研一路 · 只读仓库，**没跑任何构建或推理**。
> 等级：【源码】亲读，给 文件:行 ·【实测】仅指仓内已记录的本机实测（Apple M5 / macOS 27），本次没有新测 ·【官方】上游仓库 README / 文档 ·【推断】。
> 路径约定：不带前缀的是本 worktree（`noise_canceling-engine`，分支 `research/019-apple-inference`，基于 main 03d93c8）；
> `主检出:` = `../noise_canceling/`（分支 `chore/coord-20261031-b6`，有 main 上还没有的 `docs/plan/008_product-chain.md` 和 D46–D53）；
> `015:` = `../noise_canceling-dfnhush/`；`018:` = `../noise_canceling-judge-loop/`；`017:` = `../noise_canceling-ears/`；`res:` = `../noise_canceling/resources/`。

## 0. 一句话

要上用户电脑的模型分五类、约 8 个。**真正有实时压力的只有降噪模型（每 10 / 16 ms 调一次）**。其余模型每 0.5 s 一次或更少：认人、场景、评分、调节。场景今天用的是纯 Rust 写的约 1 K 参数 MLP，不用推理库。评分模型和调节模型**还没选型**：018 线在调研，notes 目录是空的。

## 1. 总表

| # | 类 | 模型 | 用途 | 当前选型与文件 | 参数 / 计算量 | 输入 → 输出 · 调用频率 | 现在怎么跑 | 关键算子 | 对后端的要求 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 降噪 | **GTCRN simple**（主模型，D33） | 逐帧去噪 | `res:gtcrn/stream/onnx_models/gtcrn_simple.onnx`，535 190 字节，MIT；查找顺序见 `src/models/src/lib.rs:101-124`；随安装包（`主检出:docs/plan/008_product-chain.md:42`） | 48.2 K 参数、33.0 MMAC/s【官方】`res:gtcrn/README.md:14,19`。按每秒 62.5 帧算，每帧约 0.53 MMAC【推断】 | 输入 `mix [1,257,1,2]` 加三个缓存 `conv [2,1,16,16,33]` · `tra [2,3,1,1,16]` · `inter [2,1,33,16]`。输出是 `enh` 和三个新缓存【源码】`src/models/src/gtcrn.rs:4,19-20,35`。**实时每帧**：hop 256 @16 kHz，每 16 ms 一次 | ort 2.0.0-rc.15（内含 ORT 1.31）CPU EP，intra = inter = 1，顺序执行，不自旋，`PreferCPU`，`GraphOptimizationLevel::All`【源码】`src/models/src/session.rs:30-45`。STFT（sqrt-Hann 512/256）在 Rust 侧做（`gtcrn.rs:51,117,103`）。输出预分配，Rust 侧每帧 0 次分配【实测】exp004 README:92 | Conv2d / ConvTranspose2d、BN、PReLU、Tanh、Unfold、Linear（ERB）、GRU（帧内是双向 GRU，跨帧是单向 GRU）、LayerNorm、Sigmoid【源码】`res:gtcrn/stream/gtcrn_stream.py:70,82,103-132,171-172,202-206`。缓存按切片原地写回，见 `gtcrn_stream.py:261-263,285-287,338-343`。导出用 opset 11 加 onnxsim（`:413-426`），ScatterND 大概就是这些写回生成的【推断】 | 硬截止：每帧远小于 16 ms。最坏帧 p99.9 / max 是验收口径（规约 §4 B 档）。与原模型逐值一致（exp004 对拍口径）。本机微调：BN/LN 仿射约 4.8 K 参数，或小适配器（010 decision:36-38）。**BN/LN 要在导出时做成图输入**，因为 ORT 加载时会把 BN 融进卷积（`specs/rust/architecture.md:252`） |
| 2 | 降噪 | **FastEnhancer-M**（同接口备选，D33；路由给 babble / 音乐，exp006 README:140） | 同上 | `~/vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx`，2 033 752 字节，官方 wav2wav 版（`src/models/src/lib.rs:109-113`）。权重许可待核（`fastenhancer.rs:5`） | 492 K 参数、2.9 G MACs（VBD 表，单位按同表 GTCRN 的「40M」推为每秒）【官方】`res:fastenhancer/README.md:217` 那一行。按此每帧约 29 MMAC，是 GTCRN 的 50 多倍【推断】 | `wav_in [.., 160]` 加 6 个缓存进，`wav_out` 加 6 个缓存出（7 进 7 出）【源码】`fastenhancer.rs:31,43,49-66`。**实时每帧**：hop 160，每 10 ms 一次。算法延迟 352 样本（`lib.rs:72-76`） | 同一个 ort 会话配置。**STFT / iSTFT / OLA 都在图里**（`fastenhancer.rs:1-4`）。因为超出 ort 栈上数组容量，Rust 侧每帧固定分配 7 次【实测】exp004 README:93 | StridedConv1d、ConvTranspose1d、BatchNorm1d、**GRU**、**自注意力**（qkv Linear）、Linear【源码】`res:fastenhancer/models/fastenhancer/default/model.py:15,62,113,140,187-194,316-317,449-517`。wav2wav 图里还有 STFT 类算子（官方说只有 FE 导得出 wav2wav，`res:fastenhancer/docs/docs/onnx.md:8-10`） | 同 #1。另：要和 GTCRN 共用 Rust STFT、或者挂修音链频谱节点，得换官方的 spec2spec 版 `fastenhancer_m.spec.onnx`（009 decision:73；exp006 README:117,191） |
| 3 | 降噪 | **DFN3 `_ll`**（015 线候选，D49 人定可用）（`015:docs/research/015_dfn-hush-synth/decision.md:21,26`） | 第三个同接口主模型候选。先测每帧耗时与耗电，再进盲听 | 官方 DFN 三件套 ONNX：enc / erb_dec / df_dec 加 `config.ini`（`015:…/notes/A-dfn-realtime.md:22-23`）。还没下进 vdn-data，也没有适配器 | 9.74 M 参数（enc 1.73 / erb_dec 3.20 / df_dec 4.81，数 ONNX 得到）。MAC 约 1.1–1.6 G/s【推断】`A-dfn-realtime.md:23,36` | 48 kHz，窗 20 ms，hop 10 ms，零前瞻。**实时每帧**：每 10 ms 一次。好处是省掉两次重采样（`A-dfn-realtime.md:134`） | 没跑。官方 Rust 运行时是 libDF `DfTract`（tract 0.21.4），STFT / ERB / DF 在 Rust 里做（`A-dfn-realtime.md:44`） | GRU（宽 512，DF 解码 3 层）、卷积、ERB、deep filtering。**状态不显式**：三张子图带时间轴 `S`，要靠 tract-pulse 才能逐帧（`A-dfn-realtime.md:22-23`）。用 ort 逐帧必须重新导出，把状态暴露成输入输出（`015:…/notes/C-hush-competing.md:85`） | 同 #1。另：FTZ 下长时间数字静音会出 NaN（issue #713），适配器得给归一化状态设下限（`A-dfn-realtime.md:55`）。参数是 GTCRN 的约 200 倍，耗电要按 D31 实测 |
| 4 | 降噪 | **Hush**（015：只做客观小加测，不随包，`decision.md:94`） | 竞争说话人场景的对照 | `res:Hush/deployment/models/*.tar.gz`，DF 三件套，16 kHz | 2.28 M（读 ckpt 得到；README 写约 1.8 M）（`A-dfn-realtime.md:25`） | 16 kHz，320/160，hop 10 ms | 官方 `weya_nc` cdylib 用 tract 跑（`C-hush-competing.md:79-85`） | 同 #3 | 只评测。015-1 胜出才重开「随包」这一题 |
| 5 | 认人 | **WeSpeaker ECAPA-TDNN512**（声纹，默认） | 自动注册主人；每个窗判是不是主人 | `~/vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx`，24 861 931 字节，HF 钉提交号，模型 Apache-2.0、数据 CC BY 4.0（`src/speaker/README.md:28`） | 512 通道版，输出 192 维。参数按 fp32 文件大小估约 6 M【推断】。3D-Speaker 的 1024 通道 ECAPA 是 20.8 M【官方】`res:3D-Speaker/README.md:28` | `feats [1,T,80]`（T 可变，1.5 s 窗约 150 帧）→ `embs [1,192]`【源码】`src/speaker/src/embed.rs:1,110-113`。**准实时**：判断回路每 0.5 s 一次，窗长 1.5 s（`src/speaker/src/judge.rs:55-56`） | ort CPU 单线程，配置同 #1（`embed.rs:46-60`）。fbank 是自写的 Kaldi 兼容版（`fbank.rs`）。每次调 `run` 都会分配（`embed.rs:110,115`）。整个判断回路 p50 5.9–6.1 ms、最慢 8.4 ms，约占 1.2% 个核【实测】`docs/experiments/exp005-p2-speaker/README.md:155`。Rust 与 Python 对拍：余弦 1.0000【实测】exp005 README:68-72 | Conv1d（Res2Net 分组）、BN1d、ReLU、SE（全局池化 + Linear）、注意力统计池化（ASTP）、Linear【源码】`res:wespeaker/wespeaker/models/ecapa_tdnn.py:29-118,160-204`、`pooling_layers.py:94-148` | 软截止，0.5 s 内出结果就行。**输入长度可变**，后端要支持动态形状，或者把窗长固定。余弦要和原模型一致。不在本机微调（声纹向量另存，`architecture.md:248`） |
| 5b | 认人 | WeSpeaker **CAM++**（同接口可替换） | 同上 | `voxceleb_CAM++.onnx`（`embed.rs:4`）；3D-Speaker 版的前端不同（`src/speaker/README.md:33`） | 7.2 M（3D-Speaker 版）【官方】`res:3D-Speaker/README.md:30`。WeSpeaker 版的维度注释写的是 512（`embed.rs:4`） | 同上 | 同上，还没跑过 | 未读源码 | 同上 |
| 6 | 认人 | **earshot VAD** | 每帧判有没有人声，驱动快开慢关和门控 | crates.io `earshot =1.2.2`，纯 Rust，权重编进 crate。**发布阻断：权重来源未说明**（`src/speaker/README.md:25`） | 未核实（013 F-pvad:67） | 每 256 样本（16 ms）出一个人声概率。**实时每帧**（`src/speaker/README.md:9`） | 纯 Rust，零分配（009 decision:82）。VAD 加包络加门控，每 hop p50 7.7 µs + 1.0 µs【实测】exp005 README:14,153 | 小网络，结构未查 | 已经满足实时规则，**不需要换后端**。这里列出来只是为了盘点完整 |
| 7 | 认人 | **PVAD 辅助小模型**（D47，计划中，还没有权重） | 每帧给出「是不是这台电脑的使用者在说话」的概率，驱动门控和修音配比 | 无。设计见 `docs/research/013_rl-environment/notes/F-pvad.md:6,68,117` | 约 8.5–13 万参数。2 层 LSTM / GRU ×64，FiLM 接 192 维声纹。约 6.4 万次乘加每帧，约 4 MMAC/s【推断】`013 decision.md:129` | 40 维 log-mel 加声纹，三类输出。**帧对齐 16 ms**，013 写的是「在判断回路里跑、不进声音回路」（`decision.md:129`）。但它是逐帧输出，跟不跟得上实时，看接法【推断】 | 无。训练会用 PyTorch / MPS（`F-pvad.md:80-88`） | LSTM 或 GRU、Linear、FiLM（逐元素仿射） | 每帧调用但计算极小，跟 #1 一样受框架调度开销支配【推断】。要监督训练、教师蒸馏、RL 调门控（D47）。在本机微调**未定** |
| 8 | 场景 | **场景 MLP**（vdn-scene） | 7 个噪音大类的后验；加上设备线索得到细场景，再出路由参数 | 权重写死在 `src/scene/src/weights.rs:7-15`，由 `docs/experiments/exp006-scene-sim/train.py` 生成。训练集是 build `20261030214530-44df`（`weights.rs:1-2,9`） | 35→24→7：840 + 24 + 168 + 7 = **1 039 参数**。每次判断约 800 次乘加（`src/scene/src/classify.rs:1-2`） | 35 维手工特征 → softmax 后验。逐帧特征每 16 ms 一次，判断约每 0.5 s 一次（`src/scene/src/lib.rs:4-10`） | **纯 Rust 手写**（`classify.rs:46-76`），不分配。特征平均每帧 4.8 µs，判断 177–197 µs，约占 0.07% 个核【实测】exp006 README:99 | Linear、tanh、softmax | **不需要推理后端**。D52 之后它降级成调节模型的一个输入或兜底（`主检出:docs/plan/001_decisions-20261029-b1.md:310`） |
| 9 | 本机评分 | **无参考评分模型**（D52，018 线，还没定） | 判当下输出好不好，作为本机微调的奖励 / 门禁 | 018 notes 目录是空的。现状页列出的起点倾向是 **SCOREQ**，本机现成的有 DNSMOS P.835 和 NISQA v2（`018:state/handoff/judgeloop.md:21`）。CHiME-7 上 SCOREQ 语句 SRCC 0.744，DNSMOS 0.232【实测，众人数据】`017:docs/experiments/exp009-more-ears/results/crowd_ears.md:10-11` | SCOREQ natural NR 的 ONNX **378 MB**（`docs/research/004_eval-toolchain/gap.md:39`）。DNSMOS 常用两个 ONNX 共 1.4 MB（`gap.md:42`）。D52 要求「小型化」，是 018 的 R1 题（judgeloop.md:19） | 一段音频 → 一个分数。**离线或准实时、低占空比**，跑在判断 / 学习回路（D52「AI 理解」，`主检出:001_decisions…md:311`）。窗长未定，SCOREQ 建议切 10–15 s（gap.md:39） | 开发机上用 Python onnxruntime。DNSMOS 在 M5 CPU 上 10 s 音频 0.05 s【实测】gap.md:12。不在用户电脑上 | 未查源码。SCOREQ 大概是 wav2vec2 类大骨干（Transformer）【推断】，DNSMOS 是小 CNN【推断】 | 精度：分数要和原模型一致，排序不能变。**要不要可微**：012 写了「我们的 ONNX 路线不可微」（`docs/research/012_eval-standards/verifiers.md:95`）。如果评分要当梯度奖励在本机微调，后端得能反传。体积 378 MB 进不了安装包，必须小型化或蒸馏【推断】 |
| 10 | 调节 | **调节模型**（D52，018 线，还没设计） | 输入当下声音的特征，直接输出怎么调，替代「场景分类 → 查表」 | 无。D52 原文 `主检出:001_decisions…md:309`。018 handoff 里的 exp011 初步思路是「逐窗 bandit，用评分模型当奖励」（judgeloop.md:20） | 未定。最接近的已有接口：输入可以用 `FrameFeatures`（`architecture.md:221`），输出可以是 `ParamSet`（`architecture.md:240`） | 每约 0.5 s 一次，经 `ParamBlock` 进实时线程（`architecture.md:220`）。**不在实时回路**（D52 引 D32，`001_decisions…md:310`） | 无 | 未定（多半是小 MLP 或 GRU 加 bandit 策略【推断】） | 「在本机实际微调（参数 / 小适配器）」（D52）。**本机训练这一条最可能需要反传**，不只是推理 |
| 11 | 修音（附带） | Opus **BBWENet** 带宽扩展（0.6 版，还只是可行性） | 补 8 kHz 以上 | libopus 1.6 内部 API，要写 C 薄壳（`docs/experiments/exp006-scene-sim/README.md:197`）；`exp006-scene-sim/notes/bwe-feasibility.md` | 未查 | 10 ms 帧。GTCRN hop 要重分块，最坏多 8 ms。**实时每帧** | 没跑 | 未查 | 在实时链上，耗时要单独计。当前不在 019 范围内，只登记 |

不上用户电脑、只在开发机用的（列出来免得混）：VoxLock（教师，27 M）、DPDFNet（已出局）、FRCRN / MossFormer2 / MossFormerGAN（`~/vdn-data/models/`，教师或对照）、UTMOS、Whisper / Qwen（吃字率）。

## 2. 架构规约里对后端的现有规定（`specs/rust/architecture.md`）

- **crate 拆分**：`vdn-backend-ort` / `vdn-backend-tract` 是「适配·推理」层的兄弟 crate，档位 B（`:53`）。依赖方向是 `vdn-models` 用后端，engine 不认识后端（`:69,73,76,87`）。
- **什么时候拆**：「第一次要接第二个推理后端（tract）」时，才拆出 `vdn-backend-ort` / `-tract`。拆完之后 models 只认后端 trait，**不用互斥 feature**；用 cargo-deny 守住「ort 的直接依赖者只有 `vdn-backend-ort`」（`:99`）。拆分判据之一是「同一接口的可替换实现 · 重依赖隔离」（`:103`）。
- **后端 trait 现在还不存在**：规约只说「models 只认后端 trait」（`:99`），没定签名。今天 ort 直接出现在 `vdn-models`（`src/models/Cargo.toml:3,13`，过渡期例外，`architecture.md:89`），也作为 `onnx` feature 出现在 `vdn-speaker`（`src/speaker/Cargo.toml:13-14,35`）。
- **模型接口 `Model`**（`:111-114`，`src/types/src/model.rs`）：`name / sample_rate / hop / latency_samples / reset / process`。一个 hop 进、一个 hop 出，都是波形。**构造在非实时线程**，实时线程只调 `process` 和 `reset`。频域模型在适配器里自己做 STFT。加载时用 `session.inputs()/outputs()` 校验名字和形状。`ModelError` 是 `Copy` 枚举。
- **分发**：`Engine<M: Model>` 静态分发，加 `AnyModel` 枚举，不用 `Box<dyn>`（`:118-119`；`src/models/src/lib.rs:138-177`）。新后端的适配器就是 `AnyModel` 里多一个变体【推断】。
- **换模型**：外层持有两套 Pipeline，补齐到较大的延迟，再交叉淡化。新 Pipeline 在非实时线程建好、预热后经 rtrb 送进实时线程（`:189-193`）。淡化期 CPU 翻倍，要计入最坏帧（`:222`）。
- **实时档**：推理线程是 **B 档软实时**。我们自己的 Rust 代码零分配、零锁；**ORT 内部的分配和 arena 锁允许**，按函数名登记；每帧有截止时间，超时就送原声；验收看 p99.9 和 max（`:295`）。实时 crate 清单里写明了 `vdn-models` / `vdn-backend-*`（`:302`）。
- **机器检查**：RTSan 的 B 档豁免按命名空间写，`call-stack-contains:onnxruntime::`、`Mlas`，**升级 ort 必须重跑**（`:368`）。换后端就要重写豁免表【推断】。R9：FTZ 守卫不包推理调用（`:331`）。R1 / R7：不分配、不在实时线程 drop（`:323,329`）。
- **其他约束**：ORT Environment 一个进程只许一个（`src/speaker/README.md:47`）。BN/LN 覆盖层要在导出时做成图输入（`architecture.md:252`）。小适配器换上去也走 §3.5 的预热和淡化。

## 3. 延迟账里推理占多少

- 账（`architecture.md:271-286`；`主检出:docs/plan/008_product-chain.md:44-46`）：攒满一个 hop 16.00 / 10.00 ms，加模型算法延迟 16.00 / 22.00 ms（两个模型合计都是 32 ms，等于 STFT 窗长 512），加重采样 4.00，加 IO 块 5.33（256 帧）或 2.00（96 帧），加跨进程环余量 3.00，加 HAL 2.67。合计 **47.00 ms（超 2 ms）或 43.67 ms**，预算 45。
- **推理算力时间在账上没有单独一项**。它被「跨进程环余量 3.00 ms」盖住，009 对这一项的定义是「盖住最坏推理耗时 + 唤醒抖动」（`docs/research/009_rust-ecosystem/decision.md:62`）。今天最坏一帧 0.84–0.93 ms【实测】exp004 README:85-86，在 3 ms 之内。
- 所以：把推理从 0.7 ms 提速到 0.1 ms，**基本不减延迟账**。模型带来的 32 ms 是 STFT 窗长决定的算法延迟，后端快慢改不了它【推断】。后端的收益在三处：① 尾部更稳（p99.9 / max），② 耗电（D31：约 1/5 个核）【决定】`主检出:001_decisions…md:139-143`，③ 给更大的模型（DFN3 `_ll`、FE-M 双跑淡化）腾出余量。

## 4. 「每帧时间主要花在框架调度上」这个假设，仓内已有的旁证

- GTCRN 每帧约 0.53 MMAC，Python ORT 单线程 p50 0.40 ms（exp001 README:49），折合约 **1.3 GMAC/s**。
- FE-M 每帧约 29 MMAC。官方 M5 单线程 spec2spec 的 RTF 是 0.0386（`res:fastenhancer/README.md:217`，RTF 的说明在 `docs/docs/onnx.md:10`），折合每 10 ms 帧约 0.39 ms、约 **75 GMAC/s**。我们的 wav2wav 版本测到 0.67–0.68 ms【实测】exp004:86，折合约 43 GMAC/s。
- 同一颗 CPU、同一个 ORT，GTCRN 的算术吞吐比 FE-M 低 **30–60 倍**。再加上 GTCRN 每个稳态帧里 ORT 内部有 **1554 次 malloc、376 对 mutex**【实测】`docs/research/008_rust-architecture/appendix-probes/README.md:43`。**这强烈支持「GTCRN 每帧时间主要是逐节点调度开销」**【推断，依据是官方 MAC 和仓内实测】。FE-M 的情况更接近算力受限。
- 推论：对 GTCRN 来说，融合算子、手写 Rust、整图编译（Core ML / MLX compile）的收益空间最大。对 FE-M 和 DFN3 `_ll` 来说，是矩阵后端（Accelerate / AMX / GPU / ANE）的收益空间更大【推断】。

## 对 vdn 的含义

1. **019 的同口径对比，必须做的只有降噪这一类**：GTCRN、FE-M，加上候选 DFN3 `_ll`。场景 MLP 和 earshot 已经是纯 Rust 零分配，不用比。声纹每 0.5 s 一次，单线程约 1.2% 个核，提速收益小，只要求「不比 ORT 差、能共用后端」【推断】。
2. **GTCRN 和 FE-M 的瓶颈类型不同**（调度开销 vs 算力），所以最优后端可能不是同一个。后端 trait 要能让每个模型各选各的后端，不能整个进程只绑一个【推断】。规约里「models 只认后端 trait、不用互斥 feature」（`architecture.md:99`）正好允许这样做。
3. **要先把后端 trait 的签名定下来**，规约现在没有（`architecture.md:99` 只有一句）。最小面是：非实时线程加载并预热；实时线程调 `run(inputs, outputs)`，要求零分配；状态张量由调用方持有或由后端持有二选一；`ModelError` 不带 String。还要覆盖准实时的可变长输入（声纹的 T）【推断】。
4. **本机训练是另一条要求**：D52 的本机微调，以及 010 的 BN/LN 仿射和小适配器，需要反传。Core ML 推理和手写 Rust 前向都不自带反传，MLX 有自动求导【推断，未查官方】。所以要把「推理后端」和「本机微调后端」分开评，或者一并评。
5. **延迟账不会因为换后端变好**。验收要盯的是 p99.9 / max、耗电（D31），以及与原模型逐值一致（D53 原文 `001_decisions…md:317`）。换后端后，RTSan 豁免表（`architecture.md:368`）和「Rust 侧 0 分配」测试（`src/models/tests/no_alloc.rs`）要按新后端重建。
6. **评分模型的体积是硬问题**：SCOREQ 有 378 MB，必须先由 018 小型化，019 才有东西可比。调节模型还没有结构。这两项在 019 里只能写成「接口要求」，没有实测对象。

## 未查 / 存疑

- **ScatterND 的具体来源**：只从 `gtcrn_stream.py` 的切片写回推断出来，没有打开 `gtcrn_simple.onnx` 数算子（本机没装 `onnx` 包，按规则没装）。exp010 的 `tools/dump_gtcrn.py` 也许能给出算子表，没读。
- **FE-M 的 MAC 单位与配置**：README 表 1 是 VoiceBank-Demand 口径，MACs 列没写单位，「每秒」是我比照同表 GTCRN 推的。DNS 版 M 的 hop 和 MAC 是否相同，没核。
- **ECAPA512 的确切参数量**：只按文件大小估了约 6 M，没数 initializer。WeSpeaker CAM++ 的维度（`embed.rs` 写 512，013 写 192）互相矛盾，没核。
- **earshot 的结构和参数**：未查。
- **018 的评分模型和调节模型**：018 notes 是空的，workflow 还在跑（`018:state/handoff/judgeloop.md:7,19`）。等 018 的 R1（小型化）和 R5（本机盘点）出来再补第 9、10 行。
- **D46–D53 和 `008_product-chain.md` 只在主检出的 coord 分支上**，本 worktree 的 main 里还没有。行号以主检出为准，合并后可能会变。
- **PVAD 跑在哪条回路**：013 说「判断回路、不进声音回路」，可它又是 16 ms 逐帧输出、要驱动实时门控。到底是实时线程每帧调，还是判断线程批量算再送回，仓内没定。这一点决定它要不要满足 B 档实时规则。
- **D31「约 1/5 个核」怎么换算成每帧预算**：仓内没有把它折成「推理最多占多少 ms 或多少 mW」，耗电基线一次都没测过（exp001 README:61 写了没测 CoreML 和耗电）。
