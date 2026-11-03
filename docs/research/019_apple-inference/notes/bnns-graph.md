# 019 · BNNS Graph（Accelerate 的整图 CPU 推理）调研

> 2026-11-02 · 019 调研线的一个调研代理 · 只读调研：**没装包、没构建、没跑基准**。
> 等级：【官方】苹果文档、WWDC 讲稿、本机 SDK 头文件 ·【实测-他人】别人发表的测量，不是本机 ·【源码】读第三方源码或 SDK 符号表 ·【推断】没有直接出处。
> 路径缩写：
> - `H` = `/Library/Developer/CommandLineTools/SDKs/MacOSX27.0.sdk/System/Library/Frameworks/Accelerate.framework/Frameworks/vecLib.framework/Headers/BNNS/`
> - `SW` = `/Library/Developer/CommandLineTools/SDKs/MacOSX27.0.sdk/usr/lib/swift/Accelerate.swiftmodule/arm64e-apple-macos.swiftinterface`
> - `TBD` = `/Library/Developer/CommandLineTools/SDKs/MacOSX27.0.sdk/System/Library/Frameworks/Accelerate.framework/Versions/A/Accelerate.tbd`
> - `DAFx26` = Balasubramaniam 等，《Real-time neural audio on Apple Silicon》，DAFx 2026，<https://dafx.de/paper-archive/2026/papers/DAFx26_paper_16.pdf>；配套源码 `nab` = <https://github.com/dharanipathirk/neural-audio-bench>
> - 术语：「整图编译」指把整张网络一次编译成一个可执行对象，执行时一次调用跑完整张图。

## 0. 一句话结论

BNNS Graph 是目前苹果唯一**明文写了实时音频用法**的推理接口。它有纯 C 接口，单线程可选，给够工作区就不分配内存，可以从 Rust 直接调。它**只吃 Core ML 编译产物 `.mlmodelc`**（另有一个只能在 Swift 里用的建图接口），所以我们的 ONNX 不能直接喂，要从 PyTorch 源码经 `coremltools` 重新转换。

对 GTCRN 这种「时间主要花在调度」的小模型，它的收益空间最大【推断】。第三方在 M3 上测过同量级的音频小模型：比 ORT 快约 2–5 倍，尾部也稳【实测-他人，DAFx26】。

最大的不确定点有两个：
1. GRU（含双向）、LayerNorm、Unfold、STFT 这些算子能不能被它编译：苹果没有公开算子表。
2. fp32 能不能逐值对上：缓存走 Core ML「状态」时，`coremltools` 只允许 fp16。

## 1. 它吃什么：模型来源与算子

### 1.1 两条入口

| 入口 | 吃什么 | 语言 | 最低系统 | 出处 |
|---|---|---|---|---|
| `BNNSGraphCompileFromFile` | `.mlmodelc` 目录（Core ML 编译后的产物） | C | macOS 15 | 【官方】`H/bnns_graph.h:290-309`：「Compiles the given mlmodelc to a `bnns_graph_t` object」 |
| `BNNSGraph.makeContext { builder in … }`（建图接口，`Builder`） | Swift 闭包里逐个写算子，权重用 `constant` 传进去 | **只有 Swift** | macOS 26 | 【官方】`SW:14710-14748`；WWDC25 session 276 |

- **不吃 ONNX，也不吃 `.mlpackage`**。`.mlpackage` 要先编译成 `.mlmodelc`。Xcode 在构建时自动做这一步【官方，WWDC24 10211】。命令行可用 `coremlcompiler`。本机只有 Xcode.app 里有这个工具（`/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/coremlcompiler`），CommandLineTools 里没有【实测，只查了路径，没运行】。
- `coremltools` 的 ONNX 转换器从 6.0 起冻结，统一转换接口只收 PyTorch 和 TensorFlow【官方】<https://apple.github.io/coremltools/docs-guides/source/faqs.html>。所以要从 PyTorch 源码重新转换：GTCRN 用 `res:gtcrn/stream/gtcrn_stream.py`，FE-M 用 `res:fastenhancer/models/fastenhancer/default/model.py`【推断】。
- 编译产物可以写成文件，以后 mmap 进来直接用：`BNNSGraphCompileOptionsSetOutputPath`（`H/bnns_graph.h:254-264`），对象说明见 `H/bnns_graph.h:48-56`【官方】。所以可以首次启动时编译一次，然后缓存到磁盘【推断】。
- 建图接口建出的 Swift `Context` 不暴露底层 C 句柄，也没有序列化方法：`SW:3373-3402` 和 `SW:14705-14716` 里都没有【官方，读接口】。所以用建图接口就必须在运行时经 Swift 执行，见 §5.3。
- macOS 27 没有新增 BNNS Graph 接口。三份 SDK 互相比过：CLT 26.5、CLT 27.0、Xcode 27.2。比的是头文件、`tbd` 导出符号和 Swift 接口的函数名。27.0 和 27.2 完全相同。26.5 → 27.0 只删掉了一批旧的无后缀符号，另加了几个没写进头文件的符号【官方，本机 SDK 比对】。

### 1.2 支持哪些算子

- **走 `.mlmodelc` 这条路，苹果没有公开算子清单**。文档页、头文件、两场 WWDC 都没有列出来【官方，查过：<https://developer.apple.com/documentation/accelerate/bnns-library>、WWDC24 10211、WWDC25 276】。网上也没搜到第三方整理的清单。
- 不支持时怎么报：编译失败时返回 `{.data = NULL, .size = 0}`（`H/bnns_graph.h:302-303`）。日志有一档专门的 `BNNSGraphMessageLevelUnsupported`（`H/bnns_graph.h:134-143`）。默认日志掩码包含「不支持 / 警告 / 错误」三档，写到 `os_log`。也可以用 `BNNSGraphCompileOptionsSetMessageLogCallback` 自己接住（`H/bnns_graph.h:233-252`）【官方】。所以算子行不行，**只能拿真模型编译一次看日志**。
- Swift 建图接口有一份能读的算子表。这里假定它是 `.mlmodelc` 路径支持范围的**下限**【推断】。下面对照我们的算子（`SW` 行号）：

| 我们的算子 | 建图接口里有没有 | 出处 |
|---|---|---|
| Conv（1d/2d，可设步长、空洞、分组） | 有 `conv` | `SW:5222-5223` |
| ConvTranspose | 有 `convTranspose`（含 `outputPaddingValues`） | `SW:5224-5225` |
| BatchNorm / LayerNorm / InstanceNorm / RMSNorm | 有 | `SW:14-23` |
| PReLU | 有 `prelu(alpha: [Float])` | `SW:5245` |
| Sigmoid / Tanh / Softmax | 有 | `SW:5226,5232,5254` |
| Linear / MatMul（自注意力的 qkv 和打分） | 有 `linear`、`matmul` | `SW:5217-5220` |
| GRU | 有 `gru`，带初始隐状态，输出隐状态 | `SW:14178` |
| **双向 GRU** | **没有**。`Direction` 只有 `forward` 和 `reverse`（`SW:5142-5145`）。只有 LSTM 有现成的双向版 `bidirectionalLSTM`。GTCRN 的帧内双向 GRU 要拆成正、反两个 GRU 再拼接 | `SW:14175-14181` |
| Unfold | **没有同名算子**。要用切片加拼接，或用固定权重的卷积来模拟 | 函数名全表见 `SW`（`grep "public func"`） |
| ScatterND / 切片写回 | 有 `scatterND`、`scatter`，还有切片下标 | `SW:15079-15081`、`SW:1420-1424` |
| STFT / FFT | **没有** | 同上 |
| pad / reshape / transpose / concatenate / gather | 有 | `SW:13740-13749`、`SW:15076-15078` |

- 走 `.mlmodelc` 路径时，`coremltools` 会把 `torch.nn.GRU` 转成 MIL 的 `gru`，双向的大概会拆开。`nn.Unfold` 和 `torch.stft` 也有对应的转换。但 BNNS 编译器认不认这些 MIL 算子，**没有出处**【推断】。

## 2. 实时性：苹果原话与调用约定

### 2.1 苹果原话

- 「…share how it provides real-time guarantees such as no runtime memory allocation and single-threaded running for audio or signal processing models.」【官方】WWDC24 10211 简介，<https://developer.apple.com/videos/play/wwdc2024/10211/>
- 「The main requirements for real-time processing are to avoid any memory allocations or multithreading during the execute phase…」（同上，6:47 一节）【官方】
- 讲稿里的演示：把 BNNS Graph 放进 Audio Unit 的 C++ DSP 内核里调用，「We want to make sure BNNS doesn't allocate any memory while it's processing the audio data.」【官方】WWDC24 10211，8:00 一节
- 「We recommend BNNSGraph for audio and other latency-sensitive use cases because it gives you control over allocating memory and multithreading.」【官方】WWDC25 276，3:12，<https://developer.apple.com/videos/play/wwdc2025/276/>
- 讲稿里说 Core ML 自己也用 BNNS Graph 在 CPU 上跑【官方，WWDC24 10211，据网页摘录，没逐字核对】。

### 2.2 头文件里的调用约定（`BNNSGraphContextExecute`，`H/bnns_graph.h:697-740`）

1. **线程**：「The same context must only be used by a single thread at a time.」（`:699`）。每个上下文同一时刻只能有一个线程用。头文件没写内部有没有加锁【官方】。
2. **零分配的四个条件**（`:705-711`），原文是「performs no memory allocation if」下面四条同时成立：
   - 传了工作区（`workspace` 不为空）；
   - 图里没有形状无法预先定界的张量；
   - 参数里不带新的形状信息；
   - 所有输出内存都由调用方提供。
3. **工作区**：必须按页对齐（「MUST be page-aligned」，`:725`）。大小用 `BNNSGraphContextGetWorkspaceSize` 查（`:743-759`）。改了动态形状要重新查。WWDC24 演示的做法是 `GetWorkspaceSize + NSPageSize()`，再用 `aligned_alloc` 分配【官方】。
4. **单线程**：默认多线程，要在编译选项里设 `BNNSGraphCompileOptionsSetTargetSingleThread(opts, true)`（`:192-196`）【官方】。
5. **优化偏好**：默认是 `Performance`，可以多做编译期工作（`:212-228`）【官方】。
6. **动态形状**：`BNNSGraphContextSetDynamicShapes` 返回 0 表示形状全确定，1 表示有上界，**返回 2 表示执行时要分配**（`:617-622`）。不能和执行同时调用（`:599`）【官方】。我们的两个模型形状都固定，不需要它【推断】。
7. **兜底**：如果内部还要分配，可以用 `SetWorkspaceAllocationCallback` 和 `SetOutputAllocationCallback` 换成自己的分配器（`:811-841`、`:843-868`）。可以接到预分配的内存池上【官方】。
8. **调试开关**：`BNNSGraphContextEnableNanAndInfChecks` 会检查中间张量里的 NaN 和 Inf，头文件注明「Not for use in production code」（`:674-678`）【官方】。

### 2.3 能不能在音频渲染线程直接调

- 苹果在演示里直接从 Audio Unit 的渲染内核里调 `BNNSGraphContextExecute`【官方，WWDC24 10211】。头文件里**没有「无锁」或「不做系统调用」的原话**。
- 第三方检查过（DAFx26 §3.3，第 130 页）：用 LLVM 22 的 `-fsanitize=realtime`（RTSan，实时消毒器）跑 100 次 `BNNSGraphContextExecute`，没报违规，也就是没有 malloc、`pthread_mutex_lock` 这类调用。作者自己写了这「不是形式证明」【实测-他人】。探针源码在 `nab:tools/rtsan-bnns-test/test_bnns_rt.cpp`【源码】。
- 对 vdn 的意思：推理线程是 B 档软实时（`specs/rust/architecture.md:295`）。BNNS Graph 有希望做到**内部也零分配、零锁**，比 ORT「内部分配要登记」更严。不能只信第三方，要本机用 RTSan 和 malloc 计数重测【推断】。

## 3. 有状态 / 流式：缓存怎么传

我们的缓存张量（GTCRN 3 个，FE-M 6 个）有三种接法：

1. **当普通输入、输出传（像现在的 ONNX）**：上一帧的输出缓冲交给下一帧当输入，两份缓冲轮流用。参数类型设为裸指针（`BNNSGraphArgumentTypePointer`，`H/bnns_graph.h:518-525`）。指针每帧可以换，调用方自己管内存。输出放在前、输入放在后，位置用 `BNNSGraphGetArgumentPosition` 查（`H/bnns_graph.h:472-486`、`:712-718`）。这条路**不限精度**，可以全 fp32【推断】。
2. **当 Core ML 的「状态」（`MLState` 对应物）**：Core ML 的状态张量在 BNNS 里显示成「输入兼输出」（`BNNSGraphArgumentIntentInOut`，原地更新，`H/bnns_graph.h:443-451`）。第三方后端就是这样用的：读意图，给 InOut 参数分配常驻缓冲，复位时清零（`nab:src/backends/BnnsGraphBackend.mm:60-61,117,330-333`）【源码】。**代价**：`coremltools` 转带状态的模型时只允许 fp16 状态。`nab` 的导出代码注释写着报错原文「State only support fp16 dtype」，结果整个模型只能 fp16 跑（`nab:python/neural_audio_bench/export/exporters.py:22-28`）【实测-他人】。
3. **流式环形缓冲**（`BNNSGraphContextMakeStreaming`，`H/bnns_graph.h:553-586`）：模型函数要带 `BNNSOptions` 属性 `{'StateMode': 'Streaming'}`。BNNS 认出 `output_state = concat(input_state, x)` 这种写法后，自己用环形缓冲管理状态，省掉每帧挪动缓存的拷贝【官方】<https://developer.apple.com/documentation/accelerate/bnnsgraphcontextmakestreaming(_:_:_:_:)>。限制有三条：
   - 开了流式，调用方给的 InOut 指针会被忽略；
   - 所有状态必须按同一步长推进（`H/bnns_graph.h:681-693`，步长接口要 macOS 15.4）；
   - `coremltools` 怎么写出 `BNNSOptions` 属性，**没查到出处**。

   GTCRN 的卷积缓存在时间轴上推进 1 帧，大概符合这种写法。可是 GTCRN 的跨帧 GRU 隐状态不是 concat 形式，各状态步长不一，所以大概用不了这一档【推断】。

建议先用第 1 种，保住 fp32 和逐值对拍。第 3 种留作提速备选【推断】。

## 4. 精度

- 张量类型支持 fp32、fp16、bf16（`H/bnns_constants.h:97-99`）。整图用什么精度，由 `.mlmodelc` 里的类型决定【官方】。
- `coremltools` 转 ML Program 时默认 fp16，要 fp32 得显式设 `compute_precision=ct.precision.FLOAT32`【官方】<https://apple.github.io/coremltools/docs-guides/source/typed-execution.html>。带状态的模型只能 fp16，见 §3【实测-他人】。
- 苹果说 fp16 「significantly faster」，但没给数字【官方，WWDC25 276，22:34】。WWDC24 提到 16 位累加器会溢出成 Inf【官方】。
- 符号表里有没写进头文件的 `_BNNSGraphCompileOptionsSetDownconvertMode` 和 `…SetUpconvertMode`（`TBD`）【源码】。fp32 图在内部会不会被悄悄降精度，**没有出处**，必须实测。
- **逐值一致的风险**：
  - BNNS 的卷积、矩阵乘内核换成了按芯片重排的权重，累加顺序和 ORT 的 MLAS 不同。fp32 下也几乎不可能逐位一致，只能按容差对拍【推断】。
  - GRU 的误差会随帧累积，要看长音频上的最大偏差，不能只看单帧【推断】。
  - fp16 路线对 GTCRN 的影响要单独评。它的缓存、LayerNorm 统计量和 ERB 压缩都可能对精度敏感【推断】。

## 5. 从 Rust 调

### 5.1 C 接口的绑定难度：低

- 头文件只有 4 个，都是纯 C（`H/bnns.h`、`bnns_graph.h`、`bnns_structures.h`、`bnns_constants.h`）。句柄都是 `{void*, size_t}` 小结构体，按值传（`H/bnns_graph.h:48-77`）。`BNNSTensor` 是普通结构体（`H/bnns_structures.h:146-178`）。链接参数是 `-framework Accelerate`【官方】。
- **坑：符号带版本后缀**。好几个函数用 `__asm__` 改了链接名，比如：
  - `_BNNSGraphCompileFromFile_v2`（`H/bnns_graph.h:308`）
  - `_BNNSGraphContextExecute_v2`（`:740`）
  - `_BNNSGraphContextGetWorkspaceSize_v2`（`:759`）
  - `_BNNSGraphContextDestroy_v2`（`:593`）

  27.0 SDK 的 `TBD` 已经删掉了旧的无后缀名，比如 `_BNNSGraphContextExecute`【官方，本机 SDK 比对】。手写 `extern "C"` 时必须照抄 `#[link_name]`。bindgen 能自动带上，下面这个 crate 的生成结果就带着【源码】。
- **现成 crate**：`apple-accelerate`（docs.rs 上最新 0.4.2）。机器生成的原始绑定里有这些函数：
  - `BNNSGraphCompileFromFile`，链接名 `"\u{1}_BNNSGraphCompileFromFile_v2"`；
  - `BNNSGraphContextMake`、`BNNSGraphContextMakeStreaming`；
  - `BNNSGraphContextExecute`，链接名 `…_v2`；
  - `BNNSGraphContextGetWorkspaceSize`，链接名 `…_v2`。

  安全封装只覆盖了编译选项【源码】<https://docs.rs/crate/apple-accelerate/latest/source/src/ffi/generated/bnns_missing.rs>。许可证、维护状况、要不要引入，都没查。按项目惯例，自己用 bindgen 只生成 `bnns_graph.h` 也就几十个函数，工作量很小【推断】。

### 5.2 实时线程上的调用形态（草案）【推断】

- **非实时线程**：
  1. 编译图，或 mmap 缓存好的图；
  2. 调 `ContextMake`，设参数类型为裸指针；
  3. 查工作区大小，按页对齐分配；
  4. 按名字查每个参数的位置，备好缓存的两份缓冲；
  5. 预热。
- **实时线程**：每帧填一个栈上的 `bnns_graph_argument_t[8]`（GTCRN 是 4 进 4 出），交换缓存指针，调一次 `BNNSGraphContextExecute`。Rust 侧零分配。
- 每个模型持有自己的上下文，正好符合「每个模型各选各的后端」（`our-models.md` 第 2 条含义）。
- RTSan 豁免表要换掉 ORT 那套。如果实测 BNNS 内部也零分配，就不用登记豁免。

### 5.3 Swift 建图接口：实时线程上不推荐

- 只能经 Swift 调用。从 Rust 调要写 `@_cdecl` 薄壳，带上 Swift 运行时。
- 执行入口有三个：
  - `executeFunction(_:arguments: inout [BNNSTensor])`：macOS 15 上是 async 版（`SW:3385`），macOS 26 上有同步版（`SW:14707`）；
  - 泛型的 `executeFunction<T: PointerArgument>(_:arguments: [T])`（`SW:3386`）。

  三个都收 Swift 数组【官方】。苹果原话：「although Swift doesn't guarantee real-time safety…」（WWDC24 10211，14:52 一节）【官方】。
- 它的用处：`.mlmodelc` 路径编不过时，可以拿它手工搭图，确认 BNNS 内核本身能不能跑我们的结构，以及有多快。真上线仍应走 C 接口【推断】。

## 6. 公开实测

苹果没有公布任何每帧耗时数字。两场 WWDC 只说「平均至少比旧 BNNS 逐层接口快 2 倍」（WWDC24 10211，1:18 一节）【官方】。

第三方：DAFx26（Apple **M3** MacBook Air，macOS 26.4，48 kHz 单声道，随机权重）【实测-他人】：

- **测试设置**：
  - 模型：LSTM、TCN、WaveNet 三族，各有小、中、大三档，参数 841 到 9.4 万（表 1）；
  - 对照：RTNeural、LibTorch、ONNX Runtime 1.19.2；
  - BNNS Graph 设成单线程、动态形状，每个缓冲块调一次。
- **单独跑，缓冲 128 样本（截止时间 2.67 ms，表 4）**，下面的「单次」= 实时系数 × 2.67 ms，是我换算的：

| 模型 | BNNS Graph 实时系数 | BNNS Graph 单次 | ORT 实时系数 | ORT 单次 |
|---|---|---|---|---|
| TCN-L（9.4 万参数） | 0.022 | 约 0.059 ms | 0.115 | 约 0.31 ms |
| LSTM-L（3.8 万参数） | 0.053 | 约 0.14 ms | 0.100 | 约 0.27 ms |
| WaveNet-L | 0.021 | — | 0.063 | — |

  论文另有一处直接写了 ORT 跑 TCN-L 单次 0.31 ms（§4.3.1）。
- **混音争用，36 条轨同时跑系统 Audio Unit（表 5）**：BNNS Graph 三个大模型的 p99 占截止时间 11.4–17.1 %，没有爆音（xrun = 0）。RTNeural 跑 TCN-L，没加争用时 p99 就到了 158.5 %，爆音 355 次。
- **多实例（表 6）**：16 个 TCN-L 实例同时跑，BNNS Graph 的 p99 是 27.1 %，爆音 1 次。
- **注意三点**：
  1. 他们的 BNNS Graph 模型是 **fp16**，其他后端是 fp32。原因是状态只能 fp16（见 §3），所以速度对比偏向 BNNS；
  2. 芯片是 M3，不是本机的 M5；
  3. 作者用 lldb 抽样看到 BNNS 用了 AMX 指令，ORT 只用 NEON（§5）。
- 和我们的对照：GTCRN 有 4.8 万参数，每帧约 0.53 MMAC，和 TCN-L、LSTM-L 同一量级。但 GTCRN 的结构碎得多，有 ERB、双向 GRU 和多处缓存写回。不能直接套用倍数【推断】。

## 对 vdn 的含义

**GTCRN simple**
- 能不能跑：除双向 GRU、Unfold 外，主干算子在建图接口里都有。走 `.mlmodelc` 能不能编过，**待测**【推断】。STFT 在 Rust 侧做，图里没有 STFT，这是优势。
- 预计瓶颈：现在每帧有约 1554 次 malloc、376 对 mutex，加上逐节点调度。整图编译、算子融合、零分配正好针对这些。快到 0.1 ms 量级是合理期望【推断，依据 DAFx26 同量级模型约 0.06–0.14 ms】。
- 实时线程能不能直接调：能，C 接口加预分配工作区、单线程【官方】，第三方 RTSan 也没报违规【实测-他人】。
- 要改模型结构吗：数学上不用改，但要改导出：
  - 从 PyTorch 重新转换；
  - 三个缓存显式做成输入、输出；
  - 双向 GRU 可能要手工拆开；
  - Unfold 可能要改成切片加拼接；
  - BN/LN 覆盖层要做成图输入（`architecture.md:252`）。BNNS 也会融合或重排常量，图输入能不能保住，待测【推断】。
- 逐值一致：fp32 下也只能按容差对拍。走状态或流式就被迫 fp16，风险高【推断】。

**FastEnhancer-M**
- 能不能跑：wav2wav 版图里有 STFT（`torch.stft`），建图接口里没有 STFT 或 FFT。应改用官方 spec2spec 版（`fastenhancer_m.spec.onnx` 对应的 PyTorch 导出），STFT 放回 Rust 侧做【推断】。自注意力用 `matmul` 加 `softmax` 能表达【官方，算子表】。
- 预计瓶颈：每帧约 29 MMAC，偏算力受限。BNNS 能用上 AMX，收益可能大于 GTCRN 的倍数，fp16 再快一档【推断】。
- 实时线程能不能直接调：同 GTCRN。6 个缓存都要显式做成输入、输出，参数一共 14 个左右【推断】。
- 要改模型结构吗：要换成 spec2spec 版，`StridedConv1d` 里的 reshape 技巧要确认能转【推断】。
- 逐值一致：同上。官方说明 FE 只有 wav2wav 版能导 ONNX（`res:fastenhancer/docs/docs/onnx.md:8-10`），所以 spec2spec 路线的对拍基准要重新建【推断】。

**共性**
- 首次启动要编译（耗时未量化），可以缓存到磁盘后 mmap。
- 模型更新时要重新编译。
- 没写进头文件的 `_BNNSGraphContextSetWeights`（`TBD`）暗示可以运行时换权重，但它是私有接口，产品里不能用【源码】。

## 基准要测什么（给 applebe）

清单用途：判断 BNNS Graph 能不能替换 ORT，以及该走 fp32 还是 fp16。

1. **转换可行性**（先做，不过就停）：
   - GTCRN、FE-M（spec2spec）各从 PyTorch 用 `coremltools` 转出 `.mlpackage`，参数为 `convert_to="mlprogram"`、`compute_units=CPU_ONLY`、`minimum_deployment_target=macOS15`，缓存作显式输入、输出；
   - 再用 `coremlcompiler compile` 编成 `.mlmodelc`；
   - 用 `BNNSGraphCompileFromFile` 编译，用 `BNNSGraphCompileOptionsSetMessageLogCallback` 加 `SetMessageLogMask`（全档）收集日志；
   - 记录：成功或失败、每条 `Unsupported` 消息原文、编译耗时（ms）、编译产物大小。
2. **配置矩阵**：
   - 精度：fp32 和 fp16 两档；
   - 线程：`SetTargetSingleThread(true)` 和默认多线程两档；
   - 优化偏好：`Performance`；
   - 参数类型：`BNNSGraphArgumentTypePointer`；
   - 工作区：用 `GetWorkspaceSize`，按页对齐预分配。
3. **耗时**：
   - 每帧调一次 `BNNSGraphContextExecute`，用 `mach_absolute_time` 计时；
   - 跑 ≥ 10 万帧，报 p50、p99、p99.9、max（µs）；
   - 和 ORT 同口径（`exp004` 的做法）；
   - 另跑一组在推理线程上加 CPU 争用的。
4. **实时安全**：
   - 稳态每帧的 malloc、free 次数（malloc 钩子或 `no_alloc` 测试）；
   - `pthread_mutex` 次数；
   - RTSan（`-fsanitize=realtime`）包住执行调用跑 ≥ 1 万帧；
   - 关掉预分配工作区再跑一组对照。
5. **逐值一致**：
   - 用同一段长音频（≥ 60 s，含静音段和突发噪声），和 PyTorch fp32 参考、ORT 输出各对拍；
   - 报逐样本最大绝对误差、SNR（dB），以及缓存张量在第 1、100、1 万帧的最大偏差；
   - fp32 和 fp16 分开报。
6. **耗电**：
   - 用 `powermetrics` 记 CPU 功耗（mW），条件是单模型实时跑 5 分钟，和 ORT 对比；
   - 顺带看 AMX 有没有用上。看不到 AMX 计数就写「未量化」。
7. **可选**：
   - 流式上下文（`ContextMakeStreaming`）在 GTCRN 上能不能用、快多少；
   - `.mlmodelc` 编不过时，用 Swift 建图接口手搭 GTCRN 一层，只看内核耗时。

## 未查 / 存疑

- **`.mlmodelc` 路径的算子支持表**：苹果没公开。双向 GRU、Unfold、`torch.stft` 转出的 MIL 算子能不能编译，只能实测。
- **`coremltools` 状态只能 fp16**：只有 `nab` 源码注释这一处出处，没在 `coremltools` 官方文档里找到原文。最新版本是否放开了，没查。
- **fp32 图会不会被内部降精度**：只看到私有符号 `…SetDownconvertMode`，没有文档。
- **执行时有没有锁、有没有系统调用**：头文件没写，只有第三方 100 次的 RTSan 检查。BNNS 闭源。
- **`BNNSOptions {'StateMode':'Streaming'}` 怎么从 `coremltools` 写进去**：没查到。
- **`apple-accelerate` crate** 的许可证、维护状况、和 `ort` 并存时的链接问题：没查。
- **编译耗时与磁盘缓存**：mmap 缓存的产物跨系统升级还能不能用（`_BNNSGraphGetPackageVersion` 暗示有版本号），没查。
- **WWDC24 里「Core ML 用 BNNS Graph」这句**：只来自网页摘录，没逐字核对。Core ML 的 CPU 路径会不会多一层 `MLMultiArray` 开销，归 Core ML 那份笔记。
- DAFx26 的数字全部来自 M3 和 fp16，**不是本机实测**。
