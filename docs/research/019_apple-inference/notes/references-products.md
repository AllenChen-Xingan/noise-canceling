# 019 · 参照：成熟产品与开源实时降噪「实际怎么跑、守什么线」

> 2026-11-02 · 019 调研线的一个调研代理 · 只上网查、读本机 SDK 头文件、用 `curl` 下几份上游源码到 scratchpad 读。**没装包、没构建、没跑推理。**
> 用途：给 `../standard.md` 的过线值找外部参照。本文所有数字都是**别人的**，不是本机实测。
> 等级：【官方】厂商文档 / 源码仓库 / 论文作者本人 ·【官方·转述】厂商人员原话，经媒体转述 ·【官方·未亲读】厂商原件打不开，只见搜索引擎摘要 ·【实测-他人】第三方测量 ·【二手】第三方文章的说法，无测量 ·【源码】我读了代码 ·【推断】。
> 不重复：Core ML / BNNS / MLX / 实时线程绑定见同目录 `coreml.md` · `bnns-graph.md` · `mlx.md` · `realtime-and-bindings.md`；DFN 一系的延迟、许可、社区实现已在 `docs/research/015_dfn-hush-synth/notes/A-dfn-realtime.md`，本文只补「实时线程怎么调」。
> 本机源码副本（scratchpad，不进仓）：DeepFilterNet `ladspa/src/lib.rs`、`libDF/src/tract.rs`，RNNoise `src/denoise.c`、`src/rnn.c`，SpeexDSP `libspeexdsp/preprocess.c`，WebRTC `audio_processing_impl.cc`、`ns/noise_suppressor.cc`，均取自各仓 `main`/`master`，2026-11-02。

## 0. 一句话

**能在普通 CPU 上常开的 ML 降噪，公开数字都落在「一个核的 2–5%」**：PercepNet 不到 5%【官方】，Krisp 小模型约 3–5%【官方·未亲读】，RNNoise 未优化时约 60 倍实时，约合 1.7%【官方 + 推断】。它们的做法也相同：手写 C 加 SIMD、权重 8 位量化、内存只在创建时分配，每帧在处理回调里同步算完。
**没有一家公开 p99.9 或 max。** 公开的尾部线只有 NVIDIA 的「p95 < 一帧」【官方】和 DNS 挑战赛的「每帧处理 < 步长」【官方】，都比 vdn 想定的线松得多。所以 vdn 的尾部线只能从自己的延迟账来定，外部参照只能证明「平均耗时那条线不算苛刻」。

## 1. 总表

下表把每个对象的推理方式、帧长、公开耗时和实时线程做法放在一起，方便横向比。「—」表示没查到。

| 对象 | 推理方式 | 帧长 / 算法延迟 | 公开的耗时 / CPU | 实时线程上怎么调 | 体积 | 来源等级 |
|---|---|---|---|---|---|---|
| 苹果 语音突显（Voice Isolation）/ `VoiceProcessingIO` | 系统内部，**实现未公开** | 未公开 | 未公开 | 第三方 App 只能**读**用户选的模式，改不了；`VoiceProcessingIO` 是系统 IO 单元 | 系统自带 | 【官方】头文件；其余未查到 |
| 苹果 BNNSGraph 示例（WWDC24 / 25） | `Accelerate` CPU 图执行器 | 示例是 bit crusher，不是降噪 | 「平均至少比旧 BNNS 原语快 2 倍」，无绝对数 | **在 AU 的 C++ 内核里直接调**；工作区初始化时预分配；单线程选项；每线程一个 context | — | 【官方】 |
| DeepFilterNet3 · libDF（Rust + `tract`） | `tract` 逐帧（pulse=1）跑三张子图；STFT / ERB / DF 在 Rust | 48 kHz，窗 20 ms，hop 10 ms；默认版前瞻 2 帧，约 40 ms | 论文：单线程笔记本 CPU，RTF 0.19 | `process(&mut self)` 无锁，**每帧有 `Tensor::clone`**（分配） | ONNX 包 8.0 MB | 【官方】【源码】 |
| DeepFilterNet · LADSPA 插件 | 同上，默认 `_ll` 零前瞻模型 | 最少 20 ms + 宿主延迟 | — | **另开工作线程**；音频与工作线程之间用 `Mutex<VecDeque>`；回调里 `sleep` 轮询等结果；**跟不上就自动加 1 帧延迟** | 同上 | 【官方】README ·【源码】 |
| RNNoise 0.2（Xiph） | **手写 C**，GRU + Conv1d；SSE4.1 / AVX2，运行时选路径 | 48 kHz，帧 10 ms（480 样本），窗 20 ms；合成时用上一帧频谱 | 2017 版：x86 约 60 倍实时，树莓派 3 约 7 倍 | 状态在 `rnnoise_create` 一次 `malloc`；`process_frame` 只用栈数组，同步调用 | 权重 8 位，2017 版 85 kB | 【官方】【源码】 |
| PercepNet（Amazon Chime 语音聚焦） | 手写，8 位权重 + SIMD，约 800 万权重 | 窗 20 ms、步 10 ms；前瞻 2 窗，算法延迟 30 ms | 笔记本 CPU 不到 5% 个核 | 未公开 | 约 8 M 权重 × 8 位 ≈ 8 MB【推断】 | 【官方】 |
| WebRTC 噪声抑制（NS，传统 DSP） | 手写 C++，无网络 | 10 ms 帧（16 kHz 时 160 样本），FFT 256；输出延迟 96 样本 ≈ 6 ms | — | 在采集路径同步调；APM 每次 `ProcessStream` 拿一把 `mutex_capture_` | 无模型 | 【源码】 |
| WebRTC RNN VAD（AGC2 里） | 手写 C++ GRU + 全连接，有 AVX2 文件 | 24 kHz，10 ms 帧（240 样本） | — | 同上，按 10 ms 块同步调 | 编进库 | 【源码】 |
| Google Meet 降噪（2020） | **云端**，Google 自己的 TensorFlow 处理器 | — | Google 产品总监说约 20 ms 延迟 | 不在本机 | — | 【官方·转述】 |
| Microsoft Teams 降噪 | **本机**，深度模型 | 必须按 10–20 ms 帧处理 | 「高」档要 AVX2，资源紧时会回退「低」 | 未公开 | — | 【官方·转述】【二手】 |
| Krisp（Discord 等内嵌） | 本机 CPU；SDK 有多种模型 | 10 / 20 / 30 ms 帧；增加延迟约 15 ms | 小模型约 3–5% 个核；内存 8–55 MB | Discord：CPU 太高时**自动关掉降噪** | 内存 8–55 MB | 【官方·未亲读】 |
| NVIDIA Broadcast / Maxine AFX（GPU 反例） | RTX Tensor Core GPU，不提供 CPU 回退 | 10 ms 帧 | 参考机：每帧 p50 0.374 ms、p95 0.410 ms；**冷启动 8.8 s**；显存 215 MiB | 发版线只要「p95 < 10 ms 一帧」 | 显存 181–215 MiB | 【官方】 |
| RTNeural（DAFx / arXiv 2021） | C++ 推理库，后端可选 Eigen / xsimd / STL | 库，不定帧长 | 小层比 PyTorch C++ 快，无单一数字 | **加载时分配全部内存**，前向不分配；编译期 API 更快 | 库 | 【官方】论文 |
| Silero VAD | ONNX / TorchScript，单线程 | 16 kHz 每块 512 样本（32 ms） | 「一块不到 1 ms，单线程」 | 未公开 | 约 2 MB | 【官方】README；块长【二手】 |
| SpeexDSP 预处理（传统 DSP） | 手写 C | 帧长由调用方给 | — | **全部 `speex_alloc` 都在 `_state_init` 里**；`_run` 不分配 | 无模型 | 【源码】 |

## 2. 各对象细节

### 2.1 苹果：语音突显、`VoiceProcessingIO`、WWDC 的建议

- **语音突显的实现完全没公开**。用了什么模型、跑在 CPU 还是神经引擎、多少延迟，都没查到。
- **第三方 App 改不了麦克风模式**：`preferredMicrophoneMode`、`activeMicrophoneMode` 都是 `readonly`；App 只能弹系统面板让用户自己选【官方】`SDK:…/AVFoundation.framework/Headers/AVCaptureDevice.h:2736,2746`。枚举里有 `AVCaptureMicrophoneModeVoiceIsolation = 2`，要 macOS 12+【官方】同文件 `:2725`。开发者论坛有人问「能否用代码打开语音突显」，没有权威回答【二手】https://developer.apple.com/forums/thread/844448 。
- **`VoiceProcessingIO`（`'vpio'`）**：头文件只说它「对输入做信号处理，去掉设备同时播放的声音」，即回声消除【官方】`SDK:…/AudioToolbox.framework/Headers/AUComponent.h:290-300`。可调的只有旁路、AGC、静音、闪避（ducking）、质量档等属性【官方】`AudioUnitProperties.h:2624-2759`。高级闪避配置 macOS 14 才有【官方】WWDC23 10235 https://developer.apple.com/videos/play/wwdc2023/10235/ 。它的延迟、CPU 占用都**未查到**。
- **对 vdn 的意义**【推断】：苹果自己的方案是「系统里做、App 只能读状态」。它不是能借的推理后端，只是用户可能同时开着的另一层处理。如果用户同时开了语音突显和 vdn，会叠两层降噪。这一点归音频链路线，不归 019。
- **WWDC24 10211 的原话**【官方】https://developer.apple.com/videos/play/wwdc2024/10211/ ：
  - 「实时处理的主要要求，是执行阶段不分配内存、不开多线程，因为两者都可能切进内核，导致错过实时截止时间。」
  - 示例在初始化时用 `aligned_alloc` 建页对齐工作区；`BNNSGraphCompileOptionsSetTargetSingleThread(options, true)` 把默认的多线程改成单线程。
  - 「一个 context 同一时间只能在一个线程上执行」，所以界面调参另建一个 context。
  - 性能只说「平均至少比旧 BNNS 原语快 2 倍」，没有绝对数。
- **WWDC25 276（What's new in BNNS Graph）**【官方】https://developer.apple.com/videos/play/wwdc2025/276/ ：推荐 BNNSGraph 用于音频，理由是「它让你控制内存分配和多线程」；新增 Swift 的 `BNNSGraphBuilder`；同一个 bit crusher 示例里 fp16 比 fp32 快很多。页面没写最低系统（头文件写 macOS 15，见 `realtime-and-bindings.md` §0）。
- **两场讲座都只示范了 bit crusher**，没有一个带 GRU 的降噪网络。苹果没给「ML 降噪在实时线程上多少耗时」的数字【官方，按讲稿核对】。

### 2.2 DeepFilterNet：libDF 运行时与 LADSPA 插件

DFN 的延迟、许可、各变体参数，015 已经写全（`docs/research/015_dfn-hush-synth/notes/A-dfn-realtime.md` §1）。这里只补耗时和线程做法。

- **耗时**：DFN3 论文（Interspeech 2023 演示稿）写「单线程笔记本 CPU 上 RTF 0.19」，没写 CPU 型号【官方】https://arxiv.org/abs/2305.08227 ；015 记为 i5-8250U，出处是 015 自己，本文没复核。DFN2 论文写「笔记本 Core-i5 上 RTF 0.04」【官方】https://arxiv.org/abs/2205.05474 。RTF 0.19 即每 10 ms 帧约 1.9 ms，占一帧的 19%【推断】。
- **体积**：`models/DeepFilterNet3_onnx.tar.gz` 7 983 136 字节，`_ll` 版 36 359 660 字节【官方】GitHub API 列目录，2026-11-02。
- **libDF `DfTract::process`**【源码】`libDF/src/tract.rs`：
  - 三张子图各包成 `SimpleState`（`:260-262`），用 `PulsedModel::new(&m, s, &1.to_dim())` 转成一次一帧（`:797,864,965`）。
  - 入口 `pub fn process(&mut self, noisy, enh)`（`:509`），靠 `&mut self` 独占，**没有锁**。
  - **每帧有分配**：`rolling_spec_buf_y/x.push_back(self.spec_buf.clone())`（`:541-542`）复制整块频谱张量；`cplx_buf.clone().into_tensor().permute_axes(..)`（`:462`）；静音时 `Tensor::zero`（`:492`）；合成前 `spec_ch.to_owned()`（`:637`）。`Tensor::clone` 会复制数据，所以算分配【推断，依据 `tract` 的 `Tensor` 自带数据】。rustbe 实测 `tract` 跑 GTCRN 每帧 2247 次 Rust 分配（`rustbe-numbers.md`），量级一致。
- **LADSPA 插件**【源码】`ladspa/src/lib.rs`：
  - 推理在 `thread::spawn` 起的工作线程里（`:226`）。音频和工作线程之间是 `type SampleQueue = Arc<Mutex<Vec<VecDeque<f32>>>>`（`:18`）。
  - 宿主的 `run()`（`:380`）先锁输入队列写入（`:411`），再进 `'outer: loop`（`:419`）：锁输出队列，不够就 `sleep(self.sleep_duration)`（`:431`）。睡眠间隔是 hop ÷ 5，即 2 ms（`:221`）。**也就是说，音频回调里既加锁又睡眠。**
  - 初始化时输出队列先垫 1 个 hop 的零（`:214-217`）。
  - **自适应延迟**：`run()` 量自己用时 ÷ 块时长，记作 RTF（`:436`）。RTF ≥ 1 就把延迟加一帧并补零（`:437-449`）；延迟到 1 秒直接 `panic`（`:443`）。持续一段时间 RTF < 0.5，再减回一帧（`:462-479`）。
  - README：「最小延迟 20 ms（STFT），另加宿主延迟」；插件用「没有前瞻的新模型」【官方】https://raw.githubusercontent.com/Rikorose/DeepFilterNet/main/ladspa/README.md 。
- **对 vdn 的意义**【推断】：这是「推理慢就加延迟」的做法。vdn 的延迟账是定死的 45 ms（D32），不能这样做；vdn 的做法是超时送原声（规约 R11）。这条反例说明：**尾部超线时的处理方式，本身就该是一条 verifier**（超时帧计数），不能只看平均。

### 2.3 RNNoise（Xiph）

- **结构**【源码】`src/rnn.c:44-57`：两层 Conv1d、三层 GRU、两个全连接（增益、VAD）。v0.2 的结构比 2017 版大（2017 版是三层 GRU + 全连接，无卷积）【推断，按 2017 演示页的描述对比】。
- **帧长**【源码】`src/denoise.h:31-34`：`FRAME_SIZE 480`（48 kHz 下 10 ms）、`WINDOW_SIZE 960`（20 ms）、32 个频带。
- **延迟**：`rnnoise_process_frame` 合成的是上一帧存下的 `delayed_X`（`denoise.c:478,496`），本帧频谱在末尾才存进去（`:498`）【源码】。演示页说「前瞻不超过几毫秒，这里是 10 ms」【官方】https://jmvalin.ca/demo/rnnoise/ 。总算法延迟我估 20–30 ms（窗重叠 10 ms + 帧缓冲 10 ms + 上一帧合成 10 ms 是否重复计，没核）【推断】。
- **分配**【源码】：状态在 `rnnoise_create` 里 `malloc(rnnoise_get_size())` 一次（`denoise.c:314`）；外部模型文件在加载时 `malloc`（`:237,254,262`）。`rnnoise_process_frame`（`:457`）和 `compute_rnn`（`rnn.c:44`）只用栈数组和状态结构，没看到 `malloc`。GRU / 卷积内核在别的文件里，没逐个读。没有锁。
- **耗时**：2017 演示页「x86 上约 60 倍实时，树莓派 3 上约 7 倍实时」，当时还没做 SIMD 优化，作者估计向量化后还能快约 4 倍【官方】同上。60 倍实时 ≈ 每 10 ms 帧 0.17 ms ≈ 1.7% 个核【推断】。**v0.2 的耗时没查到**。
- **v0.2（2024-04）**：加了 SSE4.1 / AVX2 和运行时 CPU 检测，模型只用公开数据集重训【二手】Phoronix https://www.phoronix.com/news/RNNoise-0.2-Released （页面 403，只见搜索摘要）。README 写可用 `--enable-x86-rtcd` 运行时选 AVX2 / SSE4.1，另有一个稀疏度更高的「little」模型【官方】https://github.com/xiph/rnnoise 。**README 没提 NEON**，Apple Silicon 上走哪条路径未查。
- **体积**：2017 版权重 8 位量化后 85 kB（fp32 要 340 kB）【官方】演示页。v0.2 的模型大小未查。

### 2.4 PercepNet（Amazon Chime「语音聚焦」）

- 同一作者（Valin）的下一代，也是「让网络做尽量少的事」的路线【官方】https://amazon.science/blog/how-amazon-chimes-challenge-winning-noise-cancellation-works 。
- **数字**【官方】同上 + 论文摘要 https://arxiv.org/abs/2008.04259 ：
  - 「现代笔记本 CPU 上，实时跑 PercepNet 不到一个核的 5%」；DNS 2020 实时赛道第二名时「只用 4% 个核」。CPU 型号没写。
  - 窗 20 ms、50% 重叠；网络前瞻 2 个窗；「算法总共多 30 ms 延迟」。
  - 34 个频带代替 400 个频点；约 800 万权重；**全部权重量化到 8 位**，加 SIMD，才压到这个 CPU 占用。
- 实时线程怎么调：未公开。

### 2.5 WebRTC 噪声抑制与 RNN VAD

- **NS（传统 DSP，无网络）**【源码】`modules/audio_processing/ns/ns_common.h`：`kFftSize = 256`，`kNsFrameSize = 160`，`kOverlapSize = 96`。`noise_suppressor.cc:115-125` 的 `DelaySignal` 用一块 `kFftSize - kNsFrameSize` = 96 样本的延迟缓冲。16 kHz 下 96 样本 = 6 ms【推断，按缓冲长度算】。
- **RNN VAD（AGC2 里）**【源码】`agc2/rnn_vad/common.h`：24 kHz、10 ms 帧（240 样本）、42 维特征。目录里有 `vector_math_avx2.cc`、`rnn_gru.cc`、`rnn_fc.cc`，是手写 GRU 和全连接，按 CPU 特性选 SIMD 路径。
- **调用方式**【源码】`audio_processing_impl.cc:909,919`：`ProcessStream` 一进来就 `MutexLock lock_capture(&mutex_capture_)`。也就是说，WebRTC 的整套音频处理在采集路径上同步调，**每 10 ms 拿一把锁**。这把锁平时没人争，争的是配置线程【推断】。WebRTC 自己也不是「零锁」。
- **Chrome / Google Meet 的 ML 降噪**：Meet 2020 年走云端。Google 产品总监说用了自家 TensorFlow 处理器，「只有约 20 ms 延迟」【官方·转述】VentureBeat 2020-06-12 https://venturebeat.com/technology/probeat-the-question-of-cloud-ai-or-edge-ai-is-far-from-settled 。之后有没有改成本机、用什么推理库，**未查到**。Meet 网页版的背景虚化走 MediaPipe + TFLite + XNNPACK（WebAssembly SIMD）【官方】https://ai.googleblog.com/2020/10/background-features-in-google-meet.html ，但那是视频，不能推到音频。

### 2.6 Krisp（及 Discord）

- **原件都没打开**：Krisp 2020 SDK 规格书和 2023 开发者页只在 Wayback 存档，本机两种方式都拉不到（SSL 握手失败 / 返回存档首页）。下面的数都来自搜索引擎对这两份原件的摘要，所以标【官方·未亲读】：
  - 处理 10 / 20 / 30 ms 帧，16 位 PCM；降噪增加延迟约 15 ms；内存 8–55 MB，看模型大小。
  - 2023 开发者页：小模型「低至一个核的约 3–5%」，没写测试条件。
  - SDK 平台：Windows、Mac、Linux、Web（JS / WASM）、Android、iOS。
- **LiveKit 的 Krisp 插件**（第三方集成方的官方文档）：帧长可选 10、15、20、30、32 ms，默认 10 ms【官方】https://docs.livekit.io/reference/python/livekit/plugins/krisp/index.html 。
- **Apple Silicon**：2021-11 Krisp 博客说原生 M1 版比上一版「CPU 能效最高 5 倍、续航最高 2 倍」，只有相对数【官方·未亲读】https://krisp.ai/blog/?p=8745 。
- **Discord**：模型在本机跑；「设备 CPU 占用很高时，Krisp 会自动关掉，以保住通话」【官方·未亲读】https://support.discord.com/hc/en-us/articles/360040843952 。用户反映短暂的 CPU 尖峰就会触发关闭【二手】Discord 社区帖。
- **Intel 方案简介**（2019）只说 Krisp 在用 Ice Lake 的 GNA 低功耗加速器，没有数字【官方】https://intel.com/content/dam/develop/external/us/en/documents/intel-krisp-solutionbrief-final-v5.pdf （本机 `pdftotext` 读过全文）。
- 第三方站点说「3–7% CPU、150–200 MB 内存」，没有测试条件【二手】，不采用。

### 2.7 NVIDIA Broadcast / Maxine 音频特效 SDK（GPU 路线，反例）

- **硬件**：Broadcast 最低 RTX 2060 / Quadro RTX 3000【官方】https://www.nvidia.com/broadcast-app-faq 。SDK 文档写，如果「必须只用 CPU、非 NVIDIA GPU、移动端或浏览器」，就该换别的方案，即**不提供 CPU 回退**【官方】https://docs.nvidia.com/maxine/afx/latest/WindowsAFXSDK/ReleaseReadiness.html 。
- **发版参考数字**【官方】同上页，参考机 RTX 3500 Ada 笔记本 GPU，SDK 3.0.0.25：
  - 降噪 v2 每个 10 ms 帧 `NvAFX_Run`：p50 0.374 ms、p95 0.410 ms；约 26.7 倍实时。
  - **发版线是「p95 < 10 ms 帧预算」**，只管 p95，不管更高分位。
  - 系统 CPU p50 / p95 = 46% / 68%，GPU p50 / p95 = 11% / 68%，p95 显存 215 MiB。CPU 这么高的原因页面没解释，可能是整机占用【推断】，见 §4。
  - 第一次进程墙钟 8.844 s，「含创建特效和加载模型」。
- **对 vdn 的意义**【推断】：GPU 路线的每帧中位数（0.37 ms）和我们今天 ORT CPU 跑 GTCRN 差不多（0.37–0.38 ms，`rustbe-numbers.md`），却要独显、两百多 MiB 显存、秒级冷启动。小模型上 GPU 没有好处，这与 `realtime-and-bindings.md` 里「Metal 每帧提交再等完成」的结论一致。

### 2.8 其他：Teams、Zoom、Silero VAD、RTNeural、SpeexDSP、DNS 挑战赛规则

- **Microsoft Teams**：Teams 负责人说必须按 10–20 ms 帧处理，所以不能送云端再拿回来；只要 CPU 不过载，本机多跑一点「应该没问题」【官方·转述】VentureBeat 同上。微软说为了让大多数 Windows 设备跑得动，在质量和复杂度之间做了取舍；「高」档要求 CPU 支持 AVX2、占资源更多，资源紧时会回退到「低」【二手】Windows Central / 高校帮助页转述，微软支持页本身没打开。
- **Zoom**：实现与数字**未查到**。
- **Silero VAD**【官方】https://github.com/snakers4/silero-vad ：「一块音频（30 ms 以上）在单个 CPU 线程上处理不到 1 ms」；JIT 模型约 2 MB；示例里 `torch.set_num_threads(1)`。16 kHz 每块 512 样本 = 32 ms【二手】下游封装文档。按 1 ms / 32 ms 算，约占 3% 个核【推断】。
- **RTNeural**【官方】Chowdhury 2021，arXiv 2106.03037（`pdftotext` 读全文）：
  - 开宗明义：PyTorch / TensorFlow 的 C++ 接口「大多数常用操作都要分配内存」，所以不适合实时。
  - 运行期 API：「加载模型时分配所需的全部内存，实时处理中从不分配」。编译期 API：内存静态分配，编译器能展开循环，更快。
  - 后端可选 Eigen、xsimd、STL，另有实验性的 Apple Accelerate 后端，功能不全。
  - 论文提到的实时网络，层宽一般 8–32 通道，都不超过 100；库专门为小层优化。小层比 PyTorch C++ 快，大层 PyTorch 更稳。测试机是 2019 MacBook Pro i7-9750H。
  - 作者建议「在目标硬件上实测再选后端」。
- **SpeexDSP 预处理**【源码】`libspeexdsp/preprocess.c`：所有 `speex_alloc` 都在 `speex_preprocess_state_init`（`:391-501`）；`speex_preprocess_run`（`:722` 起）里没有分配。
- **DNS 挑战赛实时赛道规则**【官方】https://www.microsoft.com/en-us/research/academic-program/deep-noise-suppression-challenge-interspeech-2021/ ：
  - 每帧处理时间必须小于步长，测试机是 Intel Core i5 四核 2.4 GHz 或同级。
  - 算法延迟 = 帧长 + 步长 + 前瞻，必须 ≤ 40 ms。原文举例：32 ms 帧、16 ms 步长 = 48 ms，**不合格**。
  - 按这个算法，GTCRN（32 / 16 ms）就是 48 ms，过不了 DNS 的线。vdn 用的是自己的端到端口径 ③（`002_model-baseline/must-know.md:17` 已写明「不再用 DNS 的 40 ms」），这里只是提醒：**对外说延迟时要写清用的哪种算法**【推断】。

## 3. 对 vdn 评判标准的含义

下面逐条对照 `../standard.md` 的过线值，说明外部参照支持还是不支持。结论都是**建议**，过线值最终以 `standard.md` 为准。

1. **每帧平均耗时占一帧的比例（对应 `BE-FRAME` 的 p50，以及 `BE-CPU`）**
   - 建议：**p50 ≤ 一帧的 5%**（GTCRN 0.8 ms / 16 ms，FE-M 0.5 ms / 10 ms）。
   - 依据：常开 ML 降噪的公开数字集中在 2–5% 个核：PercepNet < 5%【官方】、Krisp 小模型 3–5%【官方·未亲读】、RNNoise 2017 版约 1.7%【推断，按 60 倍实时】、Silero VAD 约 3%【推断】。DFN3 单线程 19%【官方，RTF 0.19】是「能跑」，不是「能常开」。今天 ORT 跑 GTCRN 约 2.3%、FE-M 约 3.7%（`rustbe-numbers.md` p50 ÷ 帧长）【推断】，都已在这一档。
   - 和 `standard.md` 的关系：`BE-CPU` 0.05 核等于这条线，**外部参照支持 0.05 核**。
2. **尾部（`BE-FRAME` 的 p99.9 与 max）**
   - 建议：**维持 `standard.md` 的 p99.9 ≤ 1.0 ms、max ≤ 2.0 ms**，不因外部参照放宽。
   - 依据：没有一家公开 p99 以上的分位。公开的尾部线只有 NVIDIA「p95 < 一帧」【官方】和 DNS「每帧 < 步长」【官方】，都等于「占一帧 100%」，比我们的 12.5%（2 / 16 ms）和 20%（2 / 10 ms）松 5–8 倍。产品端对尾部的真实做法是**降级**：DFN LADSPA 加延迟【源码】、Discord 关掉降噪【官方·未亲读】、Teams 从「高」回退「低」【二手】。所以尾部线只能从 vdn 自己的 3 ms 余量来定。外部参照只支持一点：002 的「最坏一帧 > 周期一半 = 不合格」（`must-know.md:17`）这条底线，与 DNS 的「< 步长」同一方向，但更严。
   - 补一条：**超时帧计数**也该进 verifier（建议名 `BE-OVERRUN`：按实时节奏跑 10 分钟，超过 max 线的帧数 = 0）。依据：成熟产品都有「超时怎么办」的机制，而 vdn 的机制是送原声，送了原声就是用户能听到的一次降噪中断【推断】。
3. **冷启动（`BE-COLD`）**
   - 建议：**维持 500 ms**。
   - 依据：唯一公开的冷启动数是 NVIDIA 的 8.8 s，含加载模型【官方】，是 GPU 路线的反例，不该当线。其他产品都**未查到**。RNNoise、Speex 这类手写实现的「冷启动」只是一次 `malloc` 和权重拷贝【源码】，接近 0。今天 ORT 打开 GTCRN 约 18 ms（`rustbe-numbers.md`）。500 ms 比所有 CPU 方案宽一个数量级以上，不会误杀。
4. **体积（`BE-SIZE`）**
   - 建议：`standard.md` 的「不高于 ORT 方案」保留，**另加一条绝对上限**作参考：模型文件 + 后端库增量 ≤ 10 MB【推断，依据如下，人可改】。
   - 依据：RNNoise 2017 版权重 85 kB【官方】；GTCRN 535 KB（`our-models.md`）；Silero VAD 约 2 MB【官方】；DFN3 ONNX 包 8.0 MB【官方】；PercepNet 约 8 M 个 8 位权重 ≈ 8 MB【推断】；Krisp 运行内存 8–55 MB【官方·未亲读】。10 MB 刚好盖住 DFN3 一档。ORT 动态库自己有多大，本文没查，`standard.md` 已要求实测。
5. **实时线程做法（`BE-RT-ALLOC` / `BE-RT-SAN`）**
   - 建议：**「每帧零分配」作为「能在推理线程直接调」的线，外部参照完全支持**。
   - 依据：所有手写实现都是「创建时分配、处理时不分配」：RNNoise【源码】、SpeexDSP【源码】、RTNeural【官方】、苹果 BNNSGraph 示例【官方】。反过来，用通用推理库的实现每帧都分配：libDF + `tract`【源码】、ORT（仓内实测 1554 次）。
   - 「零锁」要放宽理解：WebRTC 自己每 10 ms 也拿一把没人争的锁【源码】。`BE-RT-SAN` 按豁免表判、不要求绝对零锁，这个设计有参照支持。
6. **路线选择的旁证（不是过线值）**
   - 能常开、耗电低的产品，路线全是**手写 C + SIMD + 8 位权重**：RNNoise、PercepNet【官方】。没有一家公开说在实时音频路径上用 Core ML、神经引擎或 GPU 跑降噪；唯一的 GPU 方案 NVIDIA 要独显【官方】。这支持 rustbe 的手写 Rust 路线（GTCRN p50 0.10 ms）【推断】，也提示：**8 位量化**是这些产品降 CPU 的主要手段之一，`BE-PARITY` 的 60 dB 线会挡住量化，以后要量化时得另立一条「听感等价」的线【推断】。

## 4. 未查 / 存疑

- **苹果语音突显的实现、延迟、CPU、跑在哪个硬件**：全部未查到，苹果没公开。
- **Krisp 的全部数字**：原件（2020 SDK 规格书、2023 开发者页、2021 博客、Discord FAQ）本机都没打开，只见搜索摘要。要用这些数，先让人在浏览器里打开存档核一遍：`https://web.archive.org/web/20200808123722/https://krisp.ai/KrispSDK.pdf`。
- **RNNoise 0.2 的耗时、模型大小、Apple Silicon 上的 SIMD 路径**：未查到。Phoronix 页面 403。要数，得本机编译实测（本轮按规则不跑）。
- **RNNoise 的总算法延迟**：只从源码看出「合成上一帧」，没把窗重叠、帧缓冲、`delayed_X` 三项严格算清，20–30 ms 是估计。
- **NVIDIA 参考页「系统 CPU p50 46%」**：一个 GPU 方案 CPU 占这么高，不合常理。可能是整机占用、可能含示例程序的文件读写，页面没解释。只当「GPU 路线也不省 CPU」的弱旁证，不当数用。
- **NVIDIA ReleaseReadiness 页的版本**：页面写 SDK 3.0.0.25，没核这是不是当前发行版。
- **Teams 的「高档要 AVX2」「资源紧回退」**：来自第三方指南，微软支持页本身没打开。Teams 在 Apple Silicon 上用什么实现，未查到。
- **Google Meet 现在是不是还走云端**：只有 2020 年的说法。
- **DFN3 论文 RTF 0.19 的 CPU 型号**：论文摘要没写；015 记为 i5-8250U，本文没复核。
- **WebRTC NS 的 6 ms**：按延迟缓冲长度推算，没对照 WebRTC 文档。
- **Zoom**：没查到任何实现资料。
- 其他「把小网络放进音频回调」的开源实现（如 Neural Amp Modeler、anira）：本轮没细查；anira 已在 `realtime-and-bindings.md` 用过。

## 5. 自审

我当一遍怀疑派，逐条回看出处，改了下面这些：

1. **「RNNoise 约 1.7% 个核」**：原稿标【官方】。演示页只说「60 倍实时」，1.7% 是我换算的，而且是 2017 年未优化版、不是 0.2。→ 改成【官方 + 推断】，并在 §0 注明是 2017 版。
2. **「Krisp 3–5% 个核」**：原稿标【官方】。原件没打开，只见搜索摘要。→ 全部降为【官方·未亲读】，并在 §4 写了怎么核。
3. **「RNNoise v0.2 加了 SSE4.1 / AVX2、用公开数据重训」**：Phoronix 页面 403，只见摘要。→ 降为【二手】；README 里能直接看到的 AVX2 / SSE4.1 / little 模型，单独标【官方】。
4. **「libDF 每帧分配」**：WebFetch 摘要把 `erb_buf.clone()` 也算成分配。`TValue` 的 clone 可能只是引用计数，不一定分配。→ 只保留 `Tensor` 本体的 clone（`:541-542`）和 `to_owned`（`:637`）作主证据，`:462` 留着但不单独下结论；分配一事标【推断】，并用 rustbe 的 2247 次实测作旁证。
5. **「WebRTC NS 延迟 6 ms」**：原稿标【源码】。源码只给缓冲长度，6 ms 是换算。→ 改为【推断】。
6. **「WebRTC 每 10 ms 拿一把锁、平时没人争」**：拿锁是【源码】；「没人争」是我推的。→ 分开标。
7. **「PercepNet 约 8 MB」**：博客只说约 800 万权重、8 位。→ 体积标【推断】。
8. **「Silero 每块 32 ms」**：README 只写「30 ms 以上」。→ 512 样本这一说法标【二手】。
9. **「Teams 高档要 AVX2」**：原稿标【官方】。我没打开微软支持页。→ 降为【二手】。
10. **§3 第 4 条「10 MB 上限」**：外部参照只能给量级，不能给出一条必然的线。→ 明确标【推断，人可改】，不写成已定。
11. **§3 第 2 条「比我们松 5–8 倍」**：12.5% 和 20% 对 100%，是 5–8 倍，算法核过，保留。
12. **NVIDIA 数字**：都来自同一页官方发版文档，保留【官方】；但「系统 CPU 46%」的解读降为存疑，见 §4。

内部流程：本文由调研代理单独写成，自审一轮；hook 会自动提交。
