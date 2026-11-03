# 019 · Core ML 能不能当降噪模型的推理后端

> 2026-11-02 · 019 调研线 · Core ML 一路 · 只读：读了本机 SDK 头文件、`coremltools` 上游源码（`main` 分支，curl 拉到临时目录 grep，没装包）、苹果文档和 WWDC 讲稿、一篇第三方神经引擎论文。**没跑任何构建、转换或基准。**
> 等级：【官方】苹果文档、头文件、`coremltools` 上游仓库 ·【实测-他人】网上的测量，不是本机 ·【源码】·【推断】。
> 头文件路径缩写：`H/` = `/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/System/Library/Frameworks/CoreML.framework/Headers/`（SDK 版本 macOS 27.0）。
> 上游源码缩写：`ct/` = `github.com/apple/coremltools/blob/main/coremltools/converters/`。
> 名词：神经引擎 = ANE（`Neural Engine`）；计算单元 = `MLComputeUnits`；有状态模型 = 用 `MLState` 把缓存留在模型里的做法。

## 0. 一句话结论

**Core ML 能转也能跑 GTCRN 和 FE-M，但神经引擎和 GPU 对这两个模型基本没用【推断】。** 理由有三条。第一，`coremltools` 把 PyTorch 的 GRU 翻成逐时间步的 `while_loop`，不用 MIL 的 `gru` 算子【源码】；控制流和逐步小矩阵乘大概率留在 CPU，图会被切成几段【推断】。第二，神经引擎和 GPU 每调一次都有约 0.1–0.2 ms 的固定开销【实测-他人，M1 私有接口 / M2 Max 单帖，不是 M5、不是 Core ML 公开接口】，GTCRN 整帧在 ORT 里 0.4–0.7 ms（Python exp001 / Rust exp004 两个口径）。第三，神经引擎只算 fp16，有状态模型的缓存也只许 fp16【源码】，与 ORT 的差会远大于 exp004 里 Rust-ORT 对 Python-ORT 的 4.5e-7【推断】。
**值得测的只有一种配置：`cpuOnly` + `compute_precision=FLOAT32` + 缓存显式进出 + 输入输出全预分配。** 但它对 GTCRN 也可能比 ORT 慢：ORT 的 GRU 是一个融合算子，Core ML 这边是 8 个 33 步的循环，每帧约 264 次循环体、几千次 MIL 算子调度【推断，按 `gtcrn_stream.py` 结构数】。现在没有任何数字，要 applebe 实测。（怀疑派改，见文末）

## 1. 转换路线

1. **ONNX 直转已经没有了。** 官方文档说 ONNX 转换器「will be deprecated in coremltools 6」【官方】<https://apple.github.io/coremltools/docs-guides/source/new-features.html>。统一转换接口只收 TensorFlow 和 PyTorch【官方】<https://apple.github.io/coremltools/docs-guides/source/unified-conversion-api.html>。今天的 `ct/_converters_entry.py` 全文没有 `onnx` 字样【源码】。所以**必须从 PyTorch 源码重新导**，不能用 `gtcrn_simple.onnx`。
2. **版本**：最新正式版 9.0（2025-11-10），另有 9.1.dev1 预发布（2026-08-03）【官方】<https://pypi.org/project/coremltools/#history>。9.0 支持 PyTorch 2.7、加了 macOS 26 部署目标和「读写模型状态」【官方】<https://github.com/apple/coremltools/releases>。
3. **`torch.jit.trace` 还是 `torch.export`**：`torch.export` 到 8.2 版时覆盖率才是 `jit.trace` 的 83%【官方】releases 页。**建议用 `jit.trace`**，它最成熟【推断】。exp010 的 `convert_gtcrn.py` 已经按 trace 写了【源码】`docs/experiments/exp010-inference-bench/apple/convert_gtcrn.py:1-40`。
4. **GRU 能转**：
   - MIL 的 `gru` 算子只有 `forward` / `reverse` 两个方向，**没有 `bidirectional`**；LSTM 才有【源码】`ct/mil/mil/ops/defs/iOS15/recurrent.py`。
   - PyTorch 前端把双向 GRU 拆成「正向 `gru` + `reverse` → 反向 `gru` → `reverse` → `concat`」【源码】`ct/mil/frontend/torch/ops.py:3640,3732-3770`。多层、`batch_first` 也处理了（`:3729,3778`）。
   - iOS 18 版 `gru` 多了 `reset_after` 和 `input_bias` 两个参数【源码】`ct/mil/mil/ops/defs/iOS18/recurrent.py`。**但 PyTorch 前端根本不用 `mb.gru`。** `_add_gru_layer` 的注释写明「Core ML GRU has different definition from Torch, so we cannot use mb.gru, and need to implement it with while loop」【源码】`ct/mil/frontend/torch/ops.py:3507-3512`（2026-11-02 拉的 `main`）。循环体每步是 `gather` · `squeeze` · 6 个 `linear` · `sigmoid` ×2 · `tanh` · 若干 `add/mul/sub`（`:3560-3593`）。reset 门写法因此和 PyTorch 一致，不靠 `reset_after`【源码】。
   - **这对 GTCRN 的代价**：GTCRN 每个 DPGRNN 的帧内 GRU 是 2 组 × 双向、序列长 33；跨帧 GRU 是 2 组、序列长 1【源码】`res:gtcrn/stream/gtcrn_stream.py:163-203`。两个 DPGRNN 合计每帧 8 个 33 步循环 + 4 个 1 步循环，约 268 次循环体，每次约 15–20 个 MIL 算子【推断】。转换器有没有把定长 `while_loop` 展开的图优化，没查到【存疑】。如果不展开，Core ML 的逐算子调度可能比 ORT 的 1554 次 malloc 更贵【推断】。
   - **GRU 落在哪个计算单元**：一篇第三方论文说 `gru` / `lstm` / `rnn` 在神经引擎上「No path on any family」，建议改成「unrolled fixed-trip-count graph」【实测-他人 + 逆向，arXiv 2606.22283 §4.2 表 4.1，直接测量只在 M1 和 M5 上，表里的「any family」含静态分析】<https://arxiv.org/html/2606.22283v1>。注意它说的是 MIL `gru` 算子；我们转出来的是 `while_loop` + `linear`，要看的是控制流能不能上神经引擎【推断：大概率不能】。原文引的苹果论坛帖（thread 718140）**不支持**「GRU 上不了神经引擎」：那帖说的是 TF 模型里 `get_shape` 类算子回落，苹果员工建议给全静态形状【官方论坛，非文档】<https://developer.apple.com/forums/thread/718140>。GPU 能不能跑这些循环，没查到，要用 `MLComputePlan` 看【存疑】。
5. **`ScatterND` 的问题在 Core ML 路线上不存在。** `ScatterND` 是 ONNX 导出时把「缓存按切片原地写回」（`gtcrn_stream.py:261-263,285-287`）翻成的。走 PyTorch 直转时：
   - 部署目标 ≥ iOS 18 / macOS 15，切片赋值翻成 `slice_update`；更低就翻成 `torch_tensor_assign`【源码】`ct/mil/frontend/torch/ops.py:5415-5470`。
   - 对图输入原地写，转换器不收。exp010 的做法是先 `clone()` 再交给流式模型【源码】`convert_gtcrn.py:33-40`。
   - 更干净的做法是把写回改成「切片 + `concat`」的纯函数写法，只改导出包装，不改权重【推断】。
6. **`Unfold`**：`Tensor.unfold` 翻成 `sliding_windows`，`F.unfold` 翻成 `im2col`（只支持 4 维）【源码】`ct/mil/frontend/torch/ops.py:9178,9206`。
7. **FE-M 的 STFT**：前端有 `stft` 和 `fft_*` 转换器，没找到 `istft`【源码】`ops.py:9434-9503`。FE-M 官方 wav2wav 图里的 STFT 是不是用卷积写的，没读源码【存疑】。改用 spec2spec 版（Rust 做 STFT）就绕开了。
8. **fp16 还是 fp32**：
   - `mlprogram` 的 `compute_precision` **默认 FLOAT16**：转换器在每个算子前后插 `cast`，把中间张量全转成 fp16【源码】`ct/_converters_entry.py:331-378`。
   - 设 `ct.precision.FLOAT32` 就不插 `cast`，保持原 dtype（同上）。还可以用 `FP16ComputePrecision(op_selector=...)` 挑算子保留 fp32（同上）。
   - fp32 的 `mlprogram` 能跑 CPU 和 GPU，「Only the NE is barred」【官方】<https://apple.github.io/coremltools/docs-guides/source/typed-execution.html>。
   - fp16 模型在 CPU 上也不一定按 fp32 算：「a few selected ops, such as convolution, are also available with float 16 precision on the CPU」（同上）。所以 **fp16 模型选 `cpuOnly` 也不能保证逐值一致**。
   - GPU 有个开关 `allowLowPrecisionAccumulationOnGPU`，默认 NO【官方】`H/MLModelConfiguration.h:57`。

## 2. 有状态模型（`MLState`）

1. **API**：macOS 15 起。`-[MLModel newState]` 建状态，状态缓冲由 Core ML 分配并清零；`predictionFromFeatures:usingState:options:error:` 做有状态预测【官方】`H/MLModel+MLState.h:13-78`。读写状态要用 `getMultiArrayForStateNamed:handler:`，**缓冲地址每次调用可能不同**，不能在回调外用【官方】`H/MLState.h:47-56`。
2. **转换**：PyTorch 用 `register_buffer` 声明缓存，`ct.convert(states=[ct.StateType(...)])`，名字要和 buffer 一致，部署目标要 ≥ iOS 18 / macOS 15【官方】<https://apple.github.io/coremltools/docs-guides/source/stateful-models.html>。
3. **硬限制：状态只能是 fp16。** 后端写死了检查「Core ML only support fp16 for state」，不是 fp16 就报错【源码】`ct/mil/backend/mil/load.py:736-740`。`torch.export` 路线会自动把 buffer 转成 fp16 并只打警告【源码】`ct/mil/frontend/torch/converter.py:645-650`。状态形状不能可变（`load.py:731-734`）。
4. **对 GTCRN 的意思**：三个缓存（`conv` · `tra` · `inter`）一旦做成状态，就每帧被截成 fp16。GRU 隐状态和卷积缓存是跨帧递推的，舍入误差会一帧帧带下去【推断】。exp004 的 4.5e-7、对拍 SNR ≥ 124 dB（`docs/experiments/exp004-p1-rust-parity/README.md:7`）是 **Rust 调 ORT 对 Python 调 ORT**，两边是同一个推理库【源码】。它不能直接当 Core ML 的过线标准；Core ML 对 ORT 要另定容差（按感知指标，如 SI-SDR 差），待人定。fp16 肯定远离 4.5e-7【推断：fp16 有效位约 3.3 位十进制】。
5. **省下的拷贝不多**：GTCRN 三个缓存一共 2×16×16×33 + 2×3×16 + 2×33×16 = 18 048 个 fp32，约 70 KB【推断，按 `our-models.md` 的形状算】。显式进出时可以用两套预分配缓冲来回交换（见第 4 节），拷贝量为零。苹果说状态的好处是「overhead can become noticeable as the size of the state increases」，例子是 7B 大模型的 KV 缓存，测得 1.6 倍【官方】WWDC24 <https://developer.apple.com/videos/play/wwdc2024/10161/>。我们的缓存小三个数量级，**收益未量化，多半很小**【推断】。
6. **并发约束**：同一个 `MLState` 上的预测必须串行；预测进行中不许读写状态缓冲【官方】`H/MLState.h:38-40`、`H/MLModel+MLState.h:84`。
7. **结论**：有状态版只作为对照测一次（看 fp16 状态的误差和省下的时间），**不作为主方案**。

## 3. 计算单元

1. **四档**：`MLComputeUnitsCPUOnly = 0` · `CPUAndGPU = 1` · `All = 2` · `CPUAndNeuralEngine = 3`（macOS 13 起）【官方】`H/MLModelConfiguration.h:17-22`。选「含神经引擎」只是允许用，不保证全图上去【官方，ORT CoreML EP 文档，见 `docs/research/009_rust-ecosystem/cards/infer.md:184`】。
2. **神经引擎每调一次的固定开销**【实测-他人，arXiv 2606.22283，M1，私有直连接口，不是 Core ML 公开接口】：
   - 极小图（3×3 卷积 8→8 通道 + ReLU + mean）热循环约 2000 次，每次约 **190 µs**，「About 98 percent」是软件和固件调度，不是计算（§2.3）。
   - 拆开看：用户态绑定和 fp16 拷贝约 25 µs，组请求约 16 µs，固件往返约 130 µs，内核收尾约 10 µs（§2.3 表 2.3）。全文给的下限是每次约 0.23 ms（引言、§9.3）。
   - 驱动同时只放一个固件命令在途，两个线程同时提交会串行（§2.4，怀疑派没读到这一节原文，未核）。**别的进程（系统功能）也在用神经引擎时，我们的帧会排队**【推断】。
   - 空闲 5 s 后的第一次调用约 260 ms，约为稳态 p50 的 123 倍；空闲不到 1 s 也会慢 1.2–1.7 倍（§10.6 表 10.3，**M1**）。按 123 倍反推，那组测量的稳态 p50 约 2 ms，不是上面 190 µs 的极小图，两组数不能混用【推断】。空闲时功耗约 0 mW，有动态断电（§10.4）。
   - 另一份社区测量（M3 Max）拟合出约 119 µs 固定开销【实测-他人】<https://github.com/skyfallsin/ane.cpp>（只看了搜索摘要，没读原文）。
   - M5 上的数字：论文在 M5 上只给了压缩加速和形状限制（§7.3、§4.7），没给每次调用开销【实测-他人，怀疑派读全文核过】。走 Core ML 公开接口时，还要再加 Core ML 自己的开销，这部分没有公开数字【存疑】。
3. **GPU 每调一次的开销**：有人测空 kernel 的 `waitUntilCompleted` 在 M2 Max 上 **0.12 ms**【实测-他人，单一论坛帖，测法源码没贴出】<https://developer.apple.com/forums/thread/767853>。这是 Metal 直调，不是 Core ML；Core ML 走 GPU 时每帧至少付一次这个量级【推断】。
4. **实时音频领域的旁证**：一篇 2026 年的 iOS 吉他音箱建模论文说，GPU（Metal）和神经引擎（Core ML）公开接口的「per-block dispatch latency」让它们「impractical for the small, per-audio-block workloads of real-time inference at a 256-sample block size」，所以只用 CPU【他人定性判断，无数字，不是测量】<https://arxiv.org/html/2607.10086v1> §3.1。
5. **怎么看每个算子落在哪**：`MLComputePlan`（macOS 14.4 起）【官方】`H/MLComputePlan.h:57`。
   - `loadContentsOfURL:configuration:completionHandler:` 读 `.mlmodelc` 和配置（`:70-72`）。
   - `computeDeviceUsageForMLProgramOperation:` 给出这个算子「能在哪些单元跑」和「首选哪个」（`:99`；`H/MLComputePlanDeviceUsage.h:22-25`）。
   - `estimatedCostOfMLProgramOperation:` 只给一个 0–1 的 `weight`，是这个算子占整图工作量的比例，**不是时间**（`:87`；`H/MLComputePlanCost.h:24-25`）。
   - `MLModel.availableComputeDevices` 列出本机有哪些单元，神经引擎能读核数 `totalCoreCount`【官方】`H/MLModel+MLComputeDevice.h:26`、`H/MLNeuralEngineComputeDevice.h:24`。
6. **对 GTCRN 的推断**：GRU 回落 CPU，卷积可能上神经引擎。一帧里有编码器卷积 → 2 个 DPGRNN（每个含帧内双向 GRU 和跨帧 GRU）→ 解码器卷积，单元切换至少 2–4 次【推断，按 `gtcrn_stream.py:202-206,335-343` 的结构数】。每次切换按 0.1–0.2 ms 算，**光开销就超过 ORT 整帧 0.4 ms**【推断】。

## 4. 预测调用本身

1. **同步预测不是线程安全的**：「Since the synchronous prediction API is not thread safe, the app has to ensure that predictions are run serially on the model.」异步接口是线程安全的【官方】WWDC23 <https://developer.apple.com/videos/play/wwdc2023/10049/>。异步接口 macOS 14 起（`H/MLModel.h:87-101`），走完成回调，**不适合实时线程**（回调在别的线程，要等）【推断】。实时线程用同步接口，一个模型实例只给推理线程用。
2. **输入零拷贝**：`initWithDataPointer:shape:dataType:strides:deallocator:error:`「reference the existing buffer as the storage without copy」【官方】`H/MLMultiArray.h:126-153`。还可以用 IOSurface 的 `CVPixelBuffer` 包成 `MLMultiArray`，只限 fp16 或 int8，说能省一次拷贝（`:155-179`）。
3. **输出预分配**：`MLPredictionOptions.outputBackings`，macOS 11 起【官方】`H/MLPredictionOptions.h:25-89`。要点：
   - 框架**可以不用**你给的缓冲：名字没给、模型不支持、批量模式时会自己分配。要「compare … by object identity」才知道用没用（`:35-40`）。
   - 建议页对齐地址（`:63-74`）。
   - 输出缓冲要通过 `isAllowedValue:`，不然报错（`:82-86`）。
4. **缓存来回交换**：两套缓冲 A、B。第 n 帧输入 A、`outputBackings` 指向 B；第 n+1 帧反过来。事先建好两个输入 `MLFeatureProvider` 和两个 `MLPredictionOptions`，每帧只往频谱缓冲里写数【推断】。exp010 的 `bench_coreml.swift` 已经这样写【源码】`docs/experiments/exp010-inference-bench/apple/bench_coreml.swift:10,154-193`。
5. **做不到 Rust 侧零分配**：每次 `prediction` 都返回一个新的 `id<MLFeatureProvider>`，是 ObjC 对象，必有分配和自动释放池【推断】。所以按规约，这算「第三方库内部分配」，要按函数名登记进 RTSan 豁免（`specs/rust/architecture.md:295,368`）。每帧调用要包一层 `@autoreleasepool`（Rust 里用 `objc2::rc::autoreleasepool`），不然对象攒到线程结束才释放【推断】。
6. **线程数没法设**：`MLModelConfiguration` 里没有 CPU 线程数选项，整个 CoreML 头文件里也没有任何线程相关的设置【官方，头文件里查无】。Core ML 的 CPU 路径会不会在内部开工作线程（GCD），不知道【存疑】。如果开了，推理就会依赖非实时优先级的线程，尾部风险大。**这是 `cpuOnly` 方案最大的未知数。**
7. **Rust 怎么调**：
   - `objc2-core-ml` 0.3.2（2025-10-04），Zlib / Apache-2.0 / MIT【官方】<https://docs.rs/crate/objc2-core-ml/latest>。有没有包全 `MLState`、`outputBackings`、`MLComputePlan`，没核【存疑】。
   - 另一条路：`ort` 的 CoreML EP，不用自己转模型。但它的算子表里没有 GRU 和 `ScatterND`，图会被切碎；`ort` rc.15 在 macOS 默认挂 CoreML 时 GTCRN 编译失败【实测】`docs/research/009_rust-ecosystem/decision.md:72,75`。这条路不推荐。

## 5. 编译与冷启动

1. **两次编译**：`.mlpackage` → `.mlmodelc` 是第一次，可以在开发机上做好随包发。工具是 `MLModel compileModelAtURL:completionHandler:`（macOS 13 起）【官方】`H/MLModel+MLModelCompilation.h:55-58`；本机 Xcode 里有 `coremlcompiler`（`xcrun --find coremlcompiler` 在 `DEVELOPER_DIR` 指向 Xcode 时能找到；当前 `xcode-select` 指向 CommandLineTools，找不到）【实测，本机只查了路径】。
2. **加载时按设备专门化**：第一次加载要「device-specialized compilation」，结果缓存在磁盘，和模型路径、配置绑定，重启后还在；磁盘紧张、系统更新、模型文件被改时缓存会被删【官方】WWDC23 <https://developer.apple.com/videos/play/wwdc2023/10049/>。官方说神经引擎的专门化编译「may take a few seconds or even minutes for large models」【官方】<https://apple.github.io/coremltools/docs-guides/source/model-prediction.html>（经搜索摘要转述）。
3. **两个提示**【官方】`H/MLOptimizationHints.h:31,39`：
   - `specializationStrategy = FastPrediction`（macOS 15 起）：用更长的加载时间和更多内存、磁盘换预测延迟（`H/MLSpecializationStrategy.h:15-16`）。
   - `reshapeFrequency`：我们形状固定，设 `Infrequent`，避免为换形状牺牲速度（`H/MLReshapeFrequencyHint.h:12-17`）。
4. **对 vdn 的影响**：加载和预热都在非实时线程做，规约本来就这样要求（`architecture.md` §3.5）【源码】。系统更新后第一次开会重新编译，要在界面上能等【推断】。神经引擎空闲后首调极慢（见 3.2），所以**静音门控时也不能停调**，否则恢复说话那一帧会超时【推断】。

## 对 vdn 的含义

**GTCRN**
- 能不能跑：能。PyTorch → `mlprogram` 全部算子有转换器（Conv / ConvTranspose / BN / PReLU / GRU / LayerNorm / Unfold / 切片写回）【源码】。
- 预计瓶颈：调度开销，不是算力。GPU 和神经引擎每调一次 0.1–0.2 ms【实测-他人】，GRU 又上不了神经引擎，`All` 和 `CPUAndNeuralEngine` 预计比 ORT 慢【推断】。`cpuOnly` 能不能比 ORT 的 1554 次 malloc 少、更快，**尚无判据，待测**。
- 实时线程能不能直接调：能调同步接口，但做不到零分配（ObjC 返回对象），要登记豁免；CPU 内部是否开线程不明【存疑】。
- 要改模型结构吗：`cpuOnly` 不用改。要上神经引擎，就得把 GRU 展开成固定步数的矩阵乘（帧内 GRU 33 步 × 双向 × 2 组 × 2 块），图会变大很多；开销账算下来仍不划算【推断】。不建议做。
- 逐值一致风险：fp32 + `cpuOnly` 风险低到中，核函数和融合不同，预计 1e-6 量级，要实测【推断】。fp16、神经引擎、有状态版风险高：状态被强制 fp16，跨帧递推带误差【源码 + 推断】。

**FE-M**
- 能不能跑：大概能。GRU、注意力（`scaled_dot_product_attention` 有转换器，`ops.py:10092`）、Conv1d 都有转换器。wav2wav 图里的 STFT 写法没核，建议用 spec2spec 版【存疑】。
- 预计瓶颈：每帧约 29 MMAC，更接近算力受限，是三家里最可能吃到神经引擎红利的【推断】。但 GRU 照样回落 CPU，图会被切开，每帧多付至少一次 0.1–0.2 ms，ORT 整帧才 0.67 ms【推断】。净收益要测。
- 实时线程能不能直接调：同 GTCRN。7 进 7 出用两套缓冲交换，可以做到每帧不新建 `MLMultiArray`【推断】。
- 要改模型结构吗：`cpuOnly` / `CPUAndGPU` 不用。要整图上神经引擎，得展开 GRU【推断】。
- 逐值一致风险：同 GTCRN。exp004 里 FE-M 的 Rust 和 Python 是逐样本相同（差 0），换 Core ML 后一定会有非零差，验收要改成按容差判【推断】。

**共通**
- 后端 trait 要允许每个模型各选各的后端（`our-models.md` 对 vdn 的含义第 2 条），Core ML 后端可能只对 FE-M 有意义【推断】。
- 微调：Core ML 有 `MLUpdateTask`，但只对「updatable」模型；BN/LN 仿射按规约做成图输入就不需要它（`architecture.md:252`）【源码】。

## 基准要测什么（给 applebe）

用途：判断 Core ML 哪种配置值得进后端候选。所有数在本机 Apple M5 · macOS 27.0.1 上测，模型和 exp004 用同一份权重。

1. **转出 6 个模型**（exp010 `convert_gtcrn.py` 已有前 3 个的 GTCRN 版）：
   - GTCRN：`fp32` 显式缓存 · `fp16` 显式缓存 · `fp16` 有状态版。
   - FE-M（优先 spec2spec 版）：`fp32` 显式缓存 · `fp16` 显式缓存 · 有状态版（如果能转）。
   - 每个都记 `coremltools` 版本、`torch` 版本、部署目标、`sha256`，按第五节第 1 条进 `docs/experiments/exp010-inference-bench/builds/<build-id>/`。
2. **算子落点**：每个模型 × 四档计算单元，用 `MLComputePlan` 导出每个算子的 `preferredComputeDevice` 和 `weight`，数单元切换次数。重点看 GRU 在 `CPUAndGPU` 下落在哪。
3. **每帧耗时**：组合 = 模型 × 计算单元（`cpuOnly` · `CPUAndGPU` · `CPUAndNeuralEngine` · `All`）× 是否 `FastPrediction`。
   - 接口：同步 `predictionFromFeatures:options:error:`（有状态版用 `usingState:`），输入 `initWithDataPointer` 页对齐，输出 `outputBackings`，两套缓存交换，每帧包 `@autoreleasepool`。
   - 节奏：按真实节拍调（GTCRN 每 16 ms、FE-M 每 10 ms 一次，用 `mach_wait_until`），不要死循环连调——死循环测不到神经引擎和 CPU 降频后的唤醒代价。另测一组死循环作对照。
   - 量：预热 500 帧后测 ≥ 30 000 帧（GTCRN 约 8 分钟），报 p50 / p99 / p99.9 / max。
   - 每帧都核对 `outputBackings` 有没有被用上（对象身份比较），统计没用上的帧数。
4. **空闲后首调**：停调 0.1 / 0.5 / 1 / 5 s 后第一帧的耗时，每档重复 50 次。决定门控时能不能停调。
5. **分配和锁**：用 exp010 的 `alloc_hook` 数每帧 malloc 次数和锁次数，和 ORT 的 1554 / 376 比。记下栈顶函数名（给 RTSan 豁免表用）。
6. **线程**：推理期间用 `ps -M` 或 Instruments 的 System Trace 看 Core ML 有没有开工作线程，记线程数和优先级。`cpuOnly` 下尤其要看。
7. **耗电**：`sudo powermetrics --samplers cpu_power,gpu_power,ane_power -i 1000` 跑 5 分钟，记 CPU / GPU / ANE 三项平均 mW，和 ORT CPU 单线程同条件对比（D31）。
8. **逐值一致**：12 段 exp004 测试音频，整段流式跑完，和 PyTorch fp32 原模型比最大绝对误差、对拍 SNR、SI-SDR 差，用 exp004 的同一个脚本口径。fp16 和有状态版要看误差是否随帧数增长（画每 100 帧的最大误差）。
9. **冷启动**：删缓存后第一次加载耗时 · 第二次加载耗时 · 首帧耗时，四档计算单元各测。
10. **抗干扰**：`All` / `CPUAndNeuralEngine` 档下，同时开一个占用神经引擎的系统功能（如实时字幕），看 p99.9 / max 变多少。

## 未查 / 存疑

- **Core ML 公开接口自己的每次开销**：只有神经引擎私有接口的数（M1 约 190 µs）和 GPU 空 kernel 的数（M2 Max 约 0.12 ms），没有「Core ML + `cpuOnly` + 极小模型」的公开测量。M5 上的任何数字都没有。
- **`cpuOnly` 是否开内部线程、内部有没有锁**：头文件没有线程设置，文档没提。
- **GRU 在 GPU 上能不能跑**：没查到，要 `MLComputePlan` 实测。
- **PyTorch GRU 的 reset 门写法和 MIL `gru` 是否一致**：iOS 18 加了 `reset_after`，没读 `_add_gru_layer` 用的是哪种，要靠逐值对拍兜底。
- **FE-M wav2wav 图的 STFT 实现**：没读 FastEnhancer 源码，不知道能不能直转；没找到 `istft` 转换器。
- **arXiv 2606.22283 的身份**：第三方逆向论文，数字在 M1 上测，走私有接口；附录 A 的完整算子表没读。ane.cpp 的 119 µs 只看了搜索摘要。
- **`objc2-core-ml` 是否包全**了 `MLState`、`outputBackings`、`MLComputePlan`，没核。
- **fp16 模型在 CPU 上哪些算子真按 fp16 算**：官方只说「a few selected ops, such as convolution」。
- **苹果官方没有给出神经引擎支持的算子清单**；落点只能靠 `MLComputePlan` 和实测。
