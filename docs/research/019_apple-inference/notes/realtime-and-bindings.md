# 019 · 实时线程能不能直接调苹果引擎 · Rust 绑定现状 · 公开实测

> 2026-11-02 · 019 调研线的一个调研代理 · 只读仓库、读本机 SDK 头文件、上网查。**没装包、没构建、没跑基准。**
> 本机 SDK：`xcrun --show-sdk-path` = `/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk`，版本 27.0。下文 `SDK:` 指这个目录。
> 等级：【官方】苹果文档 / 头文件 / 上游仓库 ·【实测-他人】网上的测量，**不是本机实测** ·【源码】读了源码 ·【推断】没有直接出处。
> 名字约定：「送音线程」= 送音进程里 HAL 的 IO 回调线程（规约 A 档）；「推理线程」= 推理进程里的降噪线程（规约 B 档，`specs/rust/architecture.md:291-295`）。本文问的「实时线程能不能直接调」，在 vdn 里指**推理线程**。送音线程什么推理库都不碰。

## 0. 一句话结论

**四个苹果引擎里，只有 BNNSGraph（`Accelerate` 里的 CPU 图执行器）在官方承诺和第三方 RTSan 实测两头都站得住：预分配工作区、单线程编译后，执行这一步不分配、不加锁。** Core ML 每次预测都要新建 Objective-C 对象，只能当 B 档「登记豁免」用，和今天的 ORT 同一个待遇。Core ML 走神经引擎（ANE）时，每次调用要阻塞等硬件中断，有约 0.2 ms 的固定下限（M1 数据）。MLX 每次 `eval` 都要过 `std::function`、互斥锁、条件变量。Metal 每帧要提交命令缓冲再等 GPU。这三条路都不该直接放在推理线程的每帧路径上。
**最大的硬冲突**：BNNSGraph 和 Core ML 的 `MLState` 都要求 macOS 15+，而人定的最低系统是 macOS 14.2+（D07，`docs/plan/001_decisions-20261029-b1.md:55`）。

## 1. 每个引擎在推理线程上会做什么

下表说明每个引擎在每帧调用时，会不会分配内存、加锁、阻塞等待、碰 Objective-C 运行时、跨进程。

| 引擎 · 每帧入口 | 分配 | 锁 / 阻塞等待 | Objective-C 运行时 | 跨进程 | 判定 |
|---|---|---|---|---|---|
| **BNNSGraph** · `BNNSGraphContextExecute` | 满足四条件就**不分配**：传了工作区、图里所有形状可定界、不传新形状、输出由调用方提供【官方】`SDK:…/vecLib.framework/Headers/BNNS/bnns_graph.h:705-711` | 头文件没写锁。WWDC24 的说法是「执行阶段不分配、不开多线程」【官方】WWDC24 10211。默认多线程，要 `BNNSGraphCompileOptionsSetTargetSingleThread(true)`【官方】`bnns_graph.h:192-195` | 纯 C 接口，不碰 | 否（纯 CPU）【推断】 | **能直接调**（B 档，大概率也够 A 档的标准，见 §5） |
| **Core ML** · `-[MLModel predictionFromFeatures:options:error:]` | **每次都分配**：返回值是新的 `id<MLFeatureProvider>`（objc2 里是 `Retained<…>`）【官方】`SDK:…/CoreML.framework/Headers/MLModel.h:76-78`；docs.rs `objc2-core-ml` 0.3.2 `MLModel`。`outputBackings` 只是「建议」，框架可以不用、自己再分配一块【官方】`MLPredictionOptions.h:26-39` | 未公开。走 ANE 时要等中断，见下一行 | **碰**：消息发送、引用计数；无参数版本的 `error` 是 `NSError * __autoreleasing *`【官方】`MLModel.h:67`，出错时对象进自动释放池 | CPU / GPU 在本进程内【推断】；ANE 见下一行 | **只能登记豁免**（B 档），不能进 A 档 |
| **Core ML → ANE**（`MLComputeUnitsCPUAndNeuralEngine` / `All`） | 同上，另有输入输出 fp16 拷贝（M1 约 25 µs）【实测-他人】arXiv 2606.22283 §2.3 表 2.3 | **阻塞等硬件**：提交后完成走中断，每次推理约两次中断；驱动全系统**同时只放一条固件命令**，两个线程同时提交会排队（实测 1.04 倍）【实测-他人】同文 §2.2-2.4 | 同上 | **每帧不过守护进程**：`aned` 只管编译、加载、实例化；每次推理由客户端直接调内核驱动（`IOConnectCallAsyncMethod` 2 号选择子）【实测-他人，反编译】同文 §5.1、§5.2.2。本机有 `/usr/libexec/aned`、`aneuserd`【本机 ls】 | **不建议每帧直接调**：固定下限 + 全系统共享队列，尾延迟受别的进程左右【推断】 |
| **MLX** · `mlx::core::eval` | 惰性图：每帧要新建数组节点；任务以 `std::function<void()>` 入队，这一步会分配【源码】`mlx/scheduler.h`（`enqueue(Stream, std::function<void()>)`） | 有 `std::mutex`、`std::shared_mutex`、`std::condition_variable`，`wait_for_one` 里 `completion_cv.wait`【源码】`https://github.com/ml-explore/mlx/blob/main/mlx/scheduler.h` | Metal 后端会碰（命令缓冲是 Objective-C 对象）【推断】 | 否 | **不能直接调** |
| **Metal 手写** · 每帧 `commandBuffer` + `commit` + 等完成 | 每帧新建命令缓冲、编码器（Objective-C 对象）【推断】 | 等 GPU 完成要阻塞（`waitUntilCompleted` 或事件）【推断】。GPU 往返有内核启动开销，小缓冲下可能超过截止时间【实测-他人，综述】DAFx26 论文 16 §1，引 Skare | 碰 | 否（驱动在内核） | **不能直接调** |
| （对照）**ORT CPU**（今天） | 每帧 1554 次 `malloc`【实测】`docs/research/008_rust-architecture/appendix-probes/README.md:43` | 每帧 376 对 mutex【实测】同上 | 不碰 | 否 | 已按 B 档登记豁免（`architecture.md:368`） |

补充事实：

- **Objective-C 运行时本身不是实时安全的**。消息发送在方法缓存命中时不加锁；缓存没命中，查找路径会拿运行时全局锁【源码】苹果 `objc4` 的 `lookUpImpOrForward`，未逐行核对，记为【推断】。自动释放池 `objc_autoreleasePoolPush` 在当前页用完或第一次压栈时，要分配新页【推断，依据 objc4 的 `AutoreleasePoolPage` 设计】。
- **`objc2` 能绕开大部分自动释放**：返回值走 `objc_retainAutoreleasedReturnValue`，命中快路径时对象不进池【源码】本机 `~/.cargo/registry/src/*/objc2-0.6.3/src/rc/retained.rs:455-557`。但快路径不保证命中，所以每帧调用外面仍要包一层 `autoreleasepool`【推断】。
- **苹果对「实时线程上不用 Objective-C / Swift」的官方说法**：
  - AU 的渲染接口「都用 block 实现，避免 ObjC 方法派发和阻塞的可能」【官方】`SDK:…/AudioToolbox.framework/Headers/AUAudioUnit.h:115-116`。
  - 工作区间 API「Swift 不可用，因为这些 API 只用于实时线程，而 Swift 运行时在实时线程上不安全」【官方】`SDK:…/AudioToolbox.framework/Headers/AudioWorkInterval.h:51-53`。
  - 渲染 block 标了 `CA_REALTIME_API`，即 `[[clang::nonblocking]]`【官方】`SDK:…/CoreAudioTypes.framework/Headers/CoreAudioBaseTypes.h:35-49`；`AUAudioUnitImplementation.h:196-203`。
- **SDK 里没有哪个推理接口标了 `nonblocking`**：在 `Accelerate`、`CoreML`、`Metal`、`os/` 头文件里 grep `nonblocking` / `CA_REALTIME_API`，都是空的【本机 grep】。所以 BNNSGraph 的「不分配」只是文档上的承诺，编译器的函数效应检查替我们查不了。
- **苹果没有官方说「Core ML 可以在渲染线程调」**。开发者论坛上专门问这件事的帖子（2023-02），0 条回复【实测-他人】https://developer.apple.com/forums/thread/724451 。WWDC24 讲 BNNSGraph 时，苹果给的音频插件示例是：在 AU 的 C++ DSP 内核里直接调 `BNNSGraphContextExecute`，工作区在初始化时用 `aligned_alloc` 页对齐预分配；还要给滑块调参另开一个 context，因为「一个 context 同时只能在一个线程上执行」【官方】https://developer.apple.com/videos/play/wwdc2024/10211/ 。

## 2. 隔离做法：如果不能直接调

### 2.1 vdn 已经是隔离结构

vdn 的推理本来就不在送音线程上。推理在另一个进程，中间隔着共享内存环（`architecture.md` §3.6；`009 decision.md:22-25`）。所以「单独推理线程 + 无锁环 + 超时送原声」**已经是现成结构**，不用再加一层。要问的只是：推理线程里调的那个库，能不能达到 B 档的要求。

- **超时送原声要靠送音那一侧来兜**。同步调用的 Core ML 或 ANE 不能从中间打断。推理线程卡住时，送音线程读环发现这一帧没到，就送对齐好的原声（R11，`architecture.md:333`）【推断】。
- **如果一个库连 B 档都过不了**（比如 MLX），就得在推理进程里再隔一层：B 档线程用 `rtrb`（本机 registry 有 0.4.1）把帧交给一个 C 档工作线程，取结果时不等待。代价是**多一帧延迟**：GTCRN 多 16 ms，FE-M 多 10 ms【推断】。第三方测试框架 anira 就是这样做的，论文把它记为「+1 个缓冲」【实测-他人】DAFx26 论文 16 表 2。这会把延迟账从 43.67 / 47.00 ms 推到约 60 ms，超出 45 ms 的预算。**所以这条路实际上走不通**【推断】。

### 2.2 Audio Workgroup（音频工作组）

- 推理线程和送音线程**不同进程、不同周期**（IO 约 5.33 / 2 ms 一块，hop 是 16 / 10 ms），属于头文件里的第 3 种情况：「异步实时线程，自建工作区间」。做法是用 `AudioWorkIntervalCreate` 建一个工作区间，推理线程 `os_workgroup_join` 加入，每帧 `os_workgroup_interval_start(start, deadline)` / `_finish`【官方】`SDK:…/AudioWorkInterval.h:43-49`；`SDK:usr/include/os/workgroup_interval.h:95-96`。
- `os_workgroup_join` / `_leave` / `interval_update` 头文件写明「实时安全」【官方】`SDK:usr/include/os/workgroup_object.h:155-178,186-203`；`workgroup_interval.h:103`。
- 另一条路是跨进程加入送音线程所在设备的工作组：`os_workgroup_copy_port` / `os_workgroup_create_with_port`。009 已经登记过，要做实验【官方】`009 decision.md:25,176`。
- **工作组只管 CPU 的调频和调度**。它告诉性能控制器截止时间，所以对 BNNSGraph 和 Core ML CPU 有用。ANE 或 GPU 上的时间不在 CPU 线程上，工作组管不到；推理线程等中断那段时间也不算 CPU 工作【推断】。
- Rust 侧没有现成绑定。cpal、coreaudio-rs、audio_thread_priority 里都 grep 不到 `workgroup`【源码】`docs/research/008_rust-architecture/notes/r7-skeptic.md:166`。要在 `vdn-rt-thread` 里手写约 60 行 FFI（`008 gap.md:23`）。

### 2.3 对延迟账的影响

- 账里「跨进程环余量 3.00 ms」要盖住「最坏推理耗时 + 唤醒抖动」（`009 decision.md:62`）。今天最坏一帧是 0.84–0.93 ms【实测】exp004 README:85-86。
- **BNNSGraph / Core ML CPU**：只要 max 还在 3 ms 内，账不变【推断】。
- **ANE**：
  - 稳态下，每次调用约 0.19–0.23 ms 的下限（M1），其中约 98% 是软件和固件的派发开销。p99 只比 p50 高 36%（p50 约 2.1 ms 的那个测例）【实测-他人】arXiv 2606.22283 §2.3、§9.3、§10.5 表 10.3。
  - **空闲 5 秒后的第一次调用约 260 ms，是稳态 p50 的 123 倍**；预热时第 1 次调用约 7.6 ms【实测-他人】同文 §10.5。vdn 正常每 16 ms 调一次，ANE 会一直保持唤醒。但只要有暂停（旁路、静音门控跳过推理、换模型），恢复时的第一帧就会远远超过 3 ms 余量【推断】。
- **GPU / MLX**：没找到小模型每帧的公开尾延迟数字。未量化。

## 3. Rust 绑定现状

下表列出每条路在 Rust 里能用什么，以及会不会在每帧路径上引入分配或自动释放。

| 目标 | crate · 版本 | 覆盖 | 每帧路径上的隐患 | 等级 |
|---|---|---|---|---|
| Core ML | **`objc2-core-ml` 0.3.2**（objc2 0.6 家族） | 有 `MLModel`（`predictionFromFeatures_options_error`、`predictionFromFeatures_usingState_error`、`newState`、`modelWithContentsOfURL_configuration_error`）、`MLMultiArray`、`MLState`、`MLModel+MLState`，都是默认 feature；所有方法都是 `unsafe fn` | 每次预测返回 `Retained<ProtocolObject<dyn MLFeatureProvider>>`，**这个对象每帧都会新建和释放**；读输出要过 `featureValueForName` → `MLMultiArray` 再拷贝 | 【官方】https://docs.rs/objc2-core-ml ；https://docs.rs/crate/objc2-core-ml/latest/features |
| Core ML 封装 | `coreml-native` 0.2.0（建在 objc2-core-ml 上，Apache-2.0 OR MIT） | `BorrowedTensor` 用 `initWithDataPointer` 包 Rust 切片，输入零拷贝；`get_f32_into` 写进调用方的缓冲区；`predict_stateful`（macOS 15+） | README 没提自动释放池，也没说 `predict` 本身分不分配 | 【官方】https://docs.rs/crate/coreml-native/0.2.0/source/README.md |
| Core ML 封装 | `candle-coreml` 0.3.1、`coremlit` 0.1.2 | 有 `MLState`、`MLMultiArray` 的包装。`coremlit` 用 `initWithShape_dataType_error` **分配新数组** | 没细读 | 【官方】docs.rs 源码页（见文末） |
| BNNSGraph | **没有可用的 crate**。`apple-accelerate` 0.3.2 的 `src/ffi/bnns.rs` 可见部分只声明了 8 个 `BNNSGraphCompileOptions*` 函数，没看到 `Execute` / `ContextMake` | — | 要手写约 15 个函数的 C FFI。**坑**：头文件用汇编标签换了符号名，如 `BNNSGraphContextExecute` → `_BNNSGraphContextExecute_v2`、`BNNSGraphCompileFromFile` → `_v2`、`GetWorkspaceSize` → `_v2`。Rust 声明必须写 `#[link_name = "BNNSGraphContextExecute_v2"]`，否则链接到旧符号或者链接失败 | 【官方】https://docs.rs/crate/apple-accelerate/0.3.2/source/src/ffi/bnns.rs ；`bnns_graph.h:293,742,759,841,869`；link_name 写法为【推断】 |
| MLX | `mlx-rs` 0.25.3（oxideai，非官方，版本号跟着 MLX 走）；`mlx-sys` 0.2.0 | 前向 + 自动求导 | 继承 MLX 调度器的锁和 `std::function`（见 §1） | 【官方】https://docs.rs/crate/mlx-rs/latest/source |
| Metal | `objc2-metal` 0.3.2（本机 registry 已有）、`metal` 0.33.0（旧，本机已有） | 全接口 | 每帧要创建 Objective-C 对象 | 【本机 ls `~/.cargo/registry/src`】 |
| 线程间传数据 | `rtrb` 0.4.1（本机已有） | 单生产单消费环 | 无锁、不分配（已在 009 选用） | 【本机 ls】；`009 decision.md` |
| RTSan | `rtsan-standalone` 0.3.0（本机已有） | `#[nonblocking]` | — | 【本机 ls】 |
| 本机**没有** | `objc2-core-ml`、`mlx-rs`、`apple-accelerate` 都不在本机 registry | — | 测之前要发名额去拉 | 【本机 ls】 |

## 4. 苹果侧的公开实测

下面的数字都是别人在别的机器上测的，**都不是本机 M5 的实测**。

### 4.1 BNNSGraph 对 ORT 等（最接近 vdn 的问题）

出处：Balasubramaniam、Ramachandran、Timoney，《Real-time neural audio on Apple Silicon: benchmarking inference frameworks under realistic DAW contention》，DAFx26 论文 16。机器是 MacBook Air M3，macOS 26.4。https://www.dafx.de/paper-archive/2026/papers/DAFx26_paper_16.pdf 。代码在 `github.com/dharanipathirk/neural-audio-bench`。

- **模型**：时域有状态小模型，参数 841 到 93 745 个（表 1），和 GTCRN 的 48.2 K 同一量级。但它们是「每次处理一整块缓冲」，不是 GTCRN 这种每帧一张带缓存的图。
- **单跑时，buffer 128 @48 kHz（截止 2.67 ms）的 RTF**（表 4）：

  | 模型 | BNNSGraph | ORT |
  |---|---|---|
  | TCN-L | 0.022 | 0.115 |
  | LSTM-L | 0.053 | 0.100 |
  | WaveNet-L | 0.021 | 0.063 |

  【实测-他人】
- **单跑吞吐**（表 3，单位是实时的倍数）：TCN-L 上 BNNSGraph 117×，ORT 9×；LSTM-S 上 ORT 108× 对 BNNSGraph 149×。LSTM 的中小模型反而是 RTNeural 领先【实测-他人】。
- **在混音负载下**（36 条竞争轨，buffer 128）：BNNSGraph 各大模型的 p99 占用都不超过 20%，0 次爆音。RTNeural TCN-L 单跑时 RTF 0.211，放进 Core Audio 回调后 p99 到 158.5%，爆音 355 次（表 5，§4.2）【实测-他人】。
- **AMX 矩阵协处理器**：用 lldb 采指令流看到，BNNSGraph 和 LibTorch 的热路径用了 AMX，ORT 和 RTNeural 只用 NEON【实测-他人】§4.4。
- 论文把 ORT 标为「不是实时安全」，没放进负载测试【实测-他人】表 2。

### 4.2 流式降噪模型上 Core ML

- **DeepFilterNet3 流式 Core ML**（iky1e，48 kHz，每次一个 480 样本的 hop，状态显式输入输出，F32）：不按时钟节拍连续跑，稳态每 hop **0.264 ms**；52.13 s 音频用 1.494 s。和官方 PyTorch 输出的相关系数 0.999993。**没写机型、没写跑在哪个计算单元、没给 p99 和 max**【实测-他人】https://huggingface.co/iky1e/DeepFilterNet3-Streaming-CoreML 。
- **DeepFilterNet3 Core ML**（aufklarer，FP16，ANE，M2 Max）：按整段算 RTF，5 s 段 0.13，10 s 段 0.12，20 s 段 0.24。**不是每帧数字**【实测-他人】https://huggingface.co/aufklarer/DeepFilterNet3-CoreML/blob/main/README.md 。
- **DPDFNet-8 Core ML**：p50 约 6.3 ms，机型未写。这个数字来自搜索摘要，**没打开原页核对**【实测-他人，未核】https://huggingface.co/iky1e/DPDFNet8-48kHz-HR-CoreML 。
- **GTCRN**：没找到 Core ML 版。有一个 MLX 版 `agentable/gtcrn-simple-mlx`，从 sherpa-onnx 的 `gtcrn_simple.onnx` 只换了权重布局。它和 ONNX 逐帧对拍，Metal FP32 下最大绝对误差 < 5e-5。**没给延迟数字**【实测-他人】https://huggingface.co/agentable/gtcrn-simple-mlx 。
- **Silero VAD Core ML**（FluidAudio）：ANE 第一次编译约 15 s，之后每次加载约 2 s。每块延迟没有公开数字【实测-他人，搜索摘要】https://docs.fluidinference.com/reference/benchmarks 。
- **DTLN、RNNoise 在苹果引擎上**：没找到公开的每帧数字。

### 4.3 ANE 的底层数字（绕开 Core ML 直接调私有接口）

出处：Bryngelson，《Apple Neural Engine: Architecture, Programming, and Performance》，arXiv 2606.22283 v2（2026-10-15）。只在 M1 和 M5 上实测了硅片。https://arxiv.org/abs/2606.22283

- 每次派发的下限：M1 约 0.19–0.23 ms，约 98% 是软件和固件开销（§2.3、§9.3）。
- 空闲功耗约 0 mW，因为供电轨直接关掉（§10.3）。
- 在派发下限附近，能效掉到约 22 pJ/FLOP；峰值时约 0.37 pJ/FLOP（§10.4，引言）。
- 以上【实测-他人】。走 Core ML 时还要叠上 Core ML 自己的开销，量没有公开【推断】。我抓到的部分里**没有 M5 的派发下限数字**。

### 4.4 ORT 的 Core ML 执行提供器（CoreML EP）

- 官方文档的 MLProgram 算子表里**没有 GRU、LSTM、ScatterND**；BatchNormalization 只在旧的 NeuralNetwork 格式下列出【官方】https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html 。GTCRN 和 FE-M 都有 GRU，所以图会被切成好几段，每帧在 Core ML 和 ORT CPU 之间来回好几次。**每帧的派发开销只会更多**【推断】。
- 其他人的测量：M2 上 RapidOCR 各模型用 CoreML EP 比 CPU 慢 3–14 倍【实测-他人】https://rapidai.github.io/RapidOCRDocs/main/blog/2026/02/28/coreml-vs-cpu-provider-onnxruntime-rapidocr 。
- 可用选项：`ModelFormat=MLProgram`、`MLComputeUnits`、`RequireStaticInputShapes=1`、`ModelCacheDirectory`、`SpecializationStrategy=FastPrediction`、`ProfileComputePlan=1`（打印每个算子跑在哪个单元）【官方】同上。

### 4.5 耗电

没找到小型流式语音模型在苹果引擎上的公开耗电数字。只有 ANE 大模型的单次能耗：M2 上 ResNet-18 一次前向 2.6 mJ（ANE），GPU 是 22.3 mJ【实测-他人】arXiv 2606.22283 §10.2。这个数字不能外推到每 16 ms 一次的小模型。

## 5. RTSan（实时消毒器）在 macOS 上对这些库的报告

- **BNNSGraph**：作者给一个最小推理循环开了 `-fsanitize=realtime`（LLVM 22），函数标 `[[clang::nonblocking]]`。**连续 100 次调用 0 次违规**；作者也说这不算对所有配置的证明【实测-他人】DAFx26 论文 16 §3.3。100 次样本偏少，也没写有没有用单线程选项、动态形状和流式状态。
- **Core ML、MLX、Metal、ORT CoreML EP**：没找到任何公开的 RTSan 报告。
- **RTSan 在 macOS 上拦不到什么**：它拦截 `malloc` 家族、`pthread_mutex_*`、`pthread_cond_*`、`os_unfair_lock_lock`、`OSSpinLockLock`、`mmap`、`kevent` 等。**不拦截** `objc_*`、`dispatch_*`、`mach_msg` / `mach_*`、`xpc_*`、`os_sync_*`、`sem_*`【源码】https://github.com/llvm/llvm-project/blob/main/compiler-rt/lib/rtsan/rtsan_interceptors_posix.cpp 。
  - 后果：ANE 这条路「提交后等中断」那一步，如果走的是 Mach 端口或 IOKit 异步回调，RTSan 很可能报不出来【推断】。所以 **Core ML 跑出「0 次违规」也不能证明它实时安全**，必须加测每帧的上下文切换次数。
  - Objective-C 对象的分配最终会走 `malloc`，这部分拦得到【推断】。
- **macOS 上 RTSan 可能「静默不生效」**：JUCE 论坛报告，在宿主程序里跑插件时，即使故意放 `malloc` 也报不出来，要用 `DYLD_INSERT_LIBRARIES` 注入；新版会因为拦截器校验失败直接报错【实测-他人】https://forum.juce.com/t/using-realtimesanitizer-inside-daws/64557 。vdn 自己的仓里也记过，RTSan 版必须经 `cargo run` 启动（`appendix-probes/README.md:29`）。基准里**要放一个故意分配的负例**，确认 RTSan 真的在拦。

## 对 vdn 的含义

### GTCRN simple（每 16 ms 一帧，每帧约 0.53 MMAC，瓶颈是调度开销）

- **能不能跑**：
  - BNNSGraph：取决于 GRU、帧内双向 GRU、Unfold 能不能经 Core ML 格式编进 BNNSGraph。本文没查到算子覆盖表，属存疑。
  - Core ML CPU：能。
  - ANE：大概率能跑，但不划算。M1 上约 0.2 ms 的派发下限，已经是今天 ORT p50（0.4 ms）的一半。
  - GPU / MLX：能跑（MLX 版已经有人做了），但每帧要提交 GPU，不划算【推断】。
- **预计瓶颈**：派发和调度开销。BNNSGraph 是预先编译、整图融合、一次调用跑完，正对这个瓶颈。这类小模型上它对 ORT 有 2–13 倍的差距，见 §4.1 表 3、表 4 的 TCN【实测-他人，模型不同】。
- **推理线程能不能直接调**：
  - BNNSGraph：能。要单线程编译、预分配工作区、输出由我们提供、形状全固定。
  - Core ML CPU：只能在 B 档登记豁免后调。每帧至少新建和释放一个 Objective-C 输出对象，豁免表要按 `CoreML` / `Espresso` / `objc_` 命名空间重写（`architecture.md:368`）。
  - ANE：不建议，理由见 §1、§2.3。
- **要不要改模型结构**：
  - 缓存要做成显式输入输出，或者 Core ML 的 `state`。BNNSGraph 的流式模式能把 `state` 放进内部环形缓冲，省掉每帧拷贝缓存【官方】`bnns_graph.h:555-587`。但它「不支持不同状态推进量不同的模型」（`:689-690`）。GTCRN 的卷积缓存和 GRU 隐状态推进方式不同，所以多半只能用普通模式加显式缓存【推断】。
  - ScatterND 来自切片原地写回，转 Core ML 时要在 PyTorch 侧改成 concat / slice【推断】。
  - BN/LN 要做成图输入的约束（`architecture.md:252`）照样适用。Core ML 和 BNNS 编译时也会折叠常量【推断】。
- **逐值一致的风险**：
  - BNNSGraph / Core ML CPU 用 fp32：风险低，但融合会改变求和顺序，要量最大绝对误差，不能指望比特一致【推断】。
  - ANE 只有 fp16：GRU 隐状态逐帧累积误差，长时间跑会漂。必须做 10 分钟以上的长跑对拍【推断】。

### FastEnhancer-M（每 10 ms 一帧，每帧约 29 MMAC，更接近算力受限）

- **能不能跑**：
  - wav2wav 版的图里有 STFT 类算子，在 Core ML、BNNSGraph、ANE 上的支持都未知。建议改用官方的 spec2spec 版（`fastenhancer_m.spec.onnx`），STFT 放到 Rust 里做（`our-models.md` 表第 2 行）【推断】。
  - 自注意力、Conv1d、GRU 在 Core ML 的 MIL 格式里都有对应算子【推断，未核 coremltools 文档】。
- **预计瓶颈**：矩阵乘。BNNSGraph 用 AMX，ORT 只用 NEON（§4.1），这里的收益空间比 GTCRN 大。ANE 的约 0.2 ms 下限相对今天的 0.67 ms 也不算大，ANE 有机会赢，但尾延迟和空闲后第一帧的风险同 GTCRN【推断】。
- **推理线程能不能直接调**：同 GTCRN。另外，7 进 7 出的 Core ML 调用每帧要包 7 个 `MLMultiArray`。输入可以预先建好、指向固定缓冲；输出对象每帧新建【推断】。
- **要不要改模型结构**：换成 spec2spec 版；6 个缓存做显式输入输出。注意力的序列长度要固定，BNNSGraph 才不会在执行时分配（`bnns_graph.h:619-621`：返回 2 表示形状无界、执行时要分配）。
- **逐值一致的风险**：fp32 低。fp16（ANE，或 BNNS 的 fp16 选项）下 softmax 和 GRU 都有溢出或漂移风险；WWDC24 也提到 16 位累加器会溢出出 inf【官方】WWDC24 10211【推断：对 FE-M 的影响】。

### 两个模型共同的

- **最低系统冲突**：BNNSGraph 和 `MLState` 都标了 `macos(15.0)`（`bnns_graph.h` 各函数；`SDK:…/CoreML.framework/Headers/MLState.h:42`），vdn 定的是 macOS 14.2+。要么 14.x 上回落到 ORT，要么把最低系统抬到 15。**这是要人拍板的事**，交大总管登记，本文不定。
- **后端 trait 要允许每个模型各选各的后端**，和 `our-models.md` 的含义第 2 条一致。

## 基准要测什么（给 applebe）

这份清单说明每种配置怎么接、在什么条件下测、记哪些量，供发名额后照着做。

### A. 配置矩阵（每个模型都测；GTCRN 先测）

1. **ORT CPU 基线**：今天的配置（intra = inter = 1，`src/models/src/session.rs:30-45`）。
2. **BNNSGraph C 接口**：
   - 编译：`BNNSGraphCompileFromFile(mlmodelc)`，加 `BNNSGraphCompileOptionsSetTargetSingleThread(true)`；优化偏好分别试性能优先和体积优先。
   - 执行：`BNNSGraphContextMake`；`SetArgumentType` 设为指针；`GetWorkspaceSize` 拿到大小后，用 `aligned_alloc(页大小)` 预分配工作区；输出缓冲由我们提供。
   - 精度：fp32 和 fp16 各一版。
   - 如果模型能用流式模式，加测 `BNNSGraphContextMakeStreaming`。
   - 每个用到的函数都按 `_v2` 符号绑定。
3. **Core ML**（经 `objc2-core-ml`），做 4 种计算单元：`CPUOnly`、`CPUAndNeuralEngine`、`CPUAndGPU`、`All`。每种再分 2 个变体：
   - 显式缓存的无状态版；
   - `MLState` 有状态版（macOS 15+）。

   通用设置：
   - 输入 `MLMultiArray` 用 `initWithDataPointer` 指向固定缓冲，`MLDictionaryFeatureProvider` 只建一次；
   - 设 `outputBackings`，并按对象身份检查框架有没有真用它（`MLPredictionOptions.h:38-49`）；
   - `MLOptimizationHints.specializationStrategy` 设 `FastPrediction`；
   - 每帧外面包一层 `autoreleasepool`；
   - 用 `MLComputePlan` 记录每个算子实际跑在哪个单元。
4. **MLX**：编译过的函数，每帧 `eval`，CPU 流和 GPU 流各一次。只作对照。
5. **ORT CoreML EP**：`ModelFormat=MLProgram`、`RequireStaticInputShapes=1`、`ProfileComputePlan=1`，记录图被切成几段。

### B. 运行条件

- **按节拍跑，不要连续猛跑**：每 16 ms（FE-M 每 10 ms）一帧。连续跑测不出 ANE / GPU 降频和唤醒的问题。
- 推理线程设 Mach time-constraint，再加入 `AudioWorkIntervalCreate` 建的工作区间。加入和不加入各测一次。
- 每个配置至少跑 10 分钟（GTCRN 37 500 帧，FE-M 60 000 帧）。
- 两种负载各测一次：
  - 空载；
  - 有负载：多核 CPU 压力，同时开一个用 ANE 的系统功能，如实时字幕；能不能开、开哪个由 applebe 定。
- 双跑：两个模型同时跑（换模型淡化期，`architecture.md:222`）。

### C. 测哪些量

1. **每帧墙钟时间**：p50、p99、p99.9、max。另记超过 3 ms（环余量）的帧数。
2. **启动和恢复**：
   - 冷加载时间（含编译）和热加载时间（有缓存）；
   - 第 1 帧和前 100 帧的耗时；
   - **空闲 1 s / 5 s / 30 s 后的第一帧**（arXiv 那篇在 M1 上测到 5 s 空闲后约 260 ms）。
3. **实时违规**：
   - RTSan 不停模式，去重后每个唯一栈记函数名；
   - 加一个故意 `malloc` 的负例，证明拦截在起作用；
   - 另外数每帧的 `malloc` / `free` 次数、`pthread_mutex_lock` 次数、`objc_autoreleasePoolPush` 次数。
4. **RTSan 看不到的东西**：
   - 每帧的自愿和非自愿上下文切换次数（`thread_info` / `getrusage` 前后差）；
   - Instruments System Trace 里推理线程有没有阻塞在 Mach 消息或 IOKit 上。
5. **耗电**：
   - 用 `sudo powermetrics --samplers cpu_power,gpu_power,ane_power`，按节拍跑 5 分钟取平均 mW，减去同条件下不推理的基线；
   - 另记推理线程的 CPU 占用（核的比例），对照 D31 的「约 1/5 个核」。
6. **逐值一致**：
   - 和 ORT CPU 同一段输入逐帧比：最大绝对误差、均方误差；
   - 长跑 10 分钟看误差有没有随时间变大（GRU 状态漂移）；
   - fp16 和 ANE 配置必须测。
7. **实际跑在哪**：`MLComputePlan` 或 ORT 的 `ProfileComputePlan` 的输出。配置写 ANE 不等于真跑在 ANE。

## 未查 / 存疑

- **BNNSGraph 支持哪些算子**：GRU（尤其是帧内双向 GRU）、Unfold、ConvTranspose2d 能不能编进 BNNSGraph，没查到官方算子表。WWDC25 276 只点名了卷积、矩阵乘、归约、gather / scatter、pad、reshape、transpose，**没提 GRU / LSTM**【官方】https://developer.apple.com/videos/play/wwdc2025/276/ 。DAFx 论文跑通了 LSTM，所以循环层至少部分能用【实测-他人】。
- **`BNNSGraphContextExecute` 内部有没有锁**：头文件和 WWDC 都只说「不分配」，没说不加锁。DAFx 的 RTSan 0 违规间接支持「不加锁」，但样本只有 100 次。
- **Core ML CPU 路径每帧到底分配多少次、加几把锁**：没有公开数据，只能实测。
- **ANE 等待完成用的是哪种机制**（Mach 端口、IOKit 异步回调还是信号量）：arXiv 那篇说完成靠中断，我没读到用户态怎么等。这决定 RTSan 能不能看到它。
- **M5 的 ANE 派发下限**：arXiv 那篇说测过 M5 的硅片，但我读到的部分没有 M5 的派发下限数字。§9.3 没有完整读到。
- **ANE 在多进程之间怎么排队**：「同时只放一条固件命令」测的是同一进程里的两个线程。系统其他进程（实时字幕、语音隔离等）占用 ANE 时，vdn 的帧会被延后多久，未量化。
- **MLX 的 `mx.compile` 能不能消掉每帧的分配**：只读了 `scheduler.h`，没读编译路径。
- **`objc4` 运行时锁和自动释放池分页的行号**：凭对源码的记忆，没逐行核对，所以标【推断】。
- **DPDFNet-8 的 6.3 ms、FluidAudio 的 15 s / 2 s**：来自搜索摘要，没打开原页核对。
- **coremltools 把 GTCRN（ScatterND、opset 11）转成 Core ML 的可行性**：不归本题，应由负责转换的那一路查。

## 出处清单（本文用到的网址）

- WWDC24 10211：https://developer.apple.com/videos/play/wwdc2024/10211/
- WWDC25 276：https://developer.apple.com/videos/play/wwdc2025/276/
- 苹果开发者论坛帖子 724451：https://developer.apple.com/forums/thread/724451
- DAFx26 论文 16：https://www.dafx.de/paper-archive/2026/papers/DAFx26_paper_16.pdf
- arXiv 2606.22283：https://arxiv.org/abs/2606.22283 （全文 https://arxiv.org/html/2606.22283 ）
- RTSan 拦截器源码：https://github.com/llvm/llvm-project/blob/main/compiler-rt/lib/rtsan/rtsan_interceptors_posix.cpp
- JUCE 论坛 RTSan 帖：https://forum.juce.com/t/using-realtimesanitizer-inside-daws/64557
- MLX 调度器源码：https://github.com/ml-explore/mlx/blob/main/mlx/scheduler.h
- ORT CoreML EP 文档：https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html
- RapidOCR CoreML EP 对 CPU：https://rapidai.github.io/RapidOCRDocs/main/blog/2026/02/28/coreml-vs-cpu-provider-onnxruntime-rapidocr
- `objc2-core-ml`：https://docs.rs/objc2-core-ml ；`coreml-native`：https://docs.rs/crate/coreml-native/0.2.0/source/README.md ；`candle-coreml`：https://docs.rs/crate/candle-coreml/latest/source/src/state.rs ；`coremlit`：https://docs.rs/crate/coremlit/latest/source/src/multi_array/mod.rs
- `apple-accelerate` 0.3.2：https://docs.rs/crate/apple-accelerate/0.3.2/source/src/ffi/bnns.rs ；`mlx-rs`：https://docs.rs/crate/mlx-rs/latest/source
- DFN3 流式 Core ML：https://huggingface.co/iky1e/DeepFilterNet3-Streaming-CoreML ；DFN3 Core ML：https://huggingface.co/aufklarer/DeepFilterNet3-CoreML ；DPDFNet-8：https://huggingface.co/iky1e/DPDFNet8-48kHz-HR-CoreML ；GTCRN MLX：https://huggingface.co/agentable/gtcrn-simple-mlx ；FluidAudio：https://docs.fluidinference.com/reference/benchmarks
