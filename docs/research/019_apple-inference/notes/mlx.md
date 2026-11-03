# 019 · MLX 能不能当 vdn 的推理后端（含 mlx-c · mlx-swift · mlx-rs）

> 2026-11-02 · 019 调研代理（MLX 题）· 只读：读上游源码与文档、查网，**没装包、没构建、没跑基准**。
> 等级：【官方】苹果 / ml-explore 文档、头文件、上游仓库，附 URL 或 文件:行 ·【实测-他人】网上别人的测量，附出处，**不是本机实测** ·【源码】我读的上游源码 ·【推断】。
> 源码行号均指 `ml-explore/mlx` 的 **v0.32.4** 标签（2026-10-14 发布，PyPI 最新也是 0.32.4），写作 `mlx@v0.32.4:路径:行`。
> 怀疑派抽核：`utils.h:225-228`、`device.cpp:331,537-540,978-995`、`precision.rst` 已对 GitHub 上 v0.32.4 原文逐行核过，对得上。另有一份本地源码 `../noise_canceling/resources/mlx` 是 main 分支（`bd63d38`，版本号 0.32.5），行号与 v0.32.4 不同，引用时别混。
> 模型事实（算子、形状、调用频率、ORT 现状）全部来自 `docs/research/019_apple-inference/notes/our-models.md`，本文不重复。

## 0. 一句话结论

MLX **能表达** GTCRN 和 FE-M（GTCRN 已有第三方逐帧移植），也**能在本机做反传微调**；但它的执行模型是「惰性建图 → `eval` 跨线程派发 → 等完成」，每帧一次 `eval` 就要分配、加锁、跨线程等待，GRU 还要按时间步拆成很多小算子。所以**它不适合直接当 GTCRN 的逐帧实时后端**，很可能比现在的 ORT CPU 单线程更慢、尾部更差【推断，依据见 §2–§3】。MLX 最有价值的位置是**本机微调后端**（非实时线程、低占空比），推理仍走别的后端【推断】。

## 1. 能不能表达这两个模型

| 需要的东西 | MLX 有没有 | 等级与出处 |
|---|---|---|
| GRU | 有 `mlx.nn.GRU(input_size, hidden_size, bias)`，输入 `NLD`，**只有单向**，没有 `bidirectional` 参数 | 【官方】https://ml-explore.github.io/mlx/build/html/python/nn/_autosummary/mlx.nn.GRU.html |
| GRU 怎么算 | **Python 层逐时间步循环**：输入投影一次算完，然后 `for idx in range(x.shape[-2])` 每步做 matmul + sigmoid + tanh + 混合，最后 `mx.stack`。数下来每步约 12–15 个图节点（matmul、3 次切片、2 次加、sigmoid、split、乘、加、tanh、`1-z`、2 次乘、加）。没有融合的 GRU 内核 | 【源码】`mlx@main:python/mlx/nn/layers/recurrent.py`（`GRU.__call__`）|
| GRU 已知坑 | `hidden=None` 时第一步漏加 `bhn`，与「显式零初值」不等价（issue #3249，复现里最大差约 0.106）。**Python 版已修**：PR #3252，2026-03-16 合入；v0.32.4 的 `recurrent.py` 已有 `elif self.bhn is not None: n = n + r * self.bhn`。仍建议对拍时显式传零隐状态 | 【官方】https://github.com/ml-explore/mlx/issues/3249 ；【源码】`mlx@v0.32.4:python/mlx/nn/layers/recurrent.py`（GRU.__call__）|
| **mlx-rs 的 Gru 有更严重的错** | `mlx-rs/src/nn/recurrent.rs:312-351`（main，最后改动 2026-10-31）：循环体里 `let hidden = match hidden {…}` 是**新绑定**，遮住了参数，下一步仍拿初始隐状态。结果是多步序列**没有递推**，每一步都从初值算。`hidden=None` 时也还漏 `bhn`（#3249 没同步过来）。`Rnn::step`（`:135-146`）同样写法。GTCRN 帧内 GRU 一次 33 步，直接用会算错 | 【源码】https://github.com/oxiglade/mlx-rs/blob/main/mlx-rs/src/nn/recurrent.rs ；「会算错」是读码推断，未运行验证 |
| GRU 提速诉求 | issue #1500 请求融合 GRUCell，维护者承认 RNN 需要提速；2025-12 以「PyTorch 那边也只是拼现有算子」为由关闭，**上游没有融合 GRU** | 【官方】https://github.com/ml-explore/mlx/issues/1500 |
| Conv1d / Conv2d | 有 | 【源码】`mlx@v0.32.4:python/mlx/nn/layers/__init__.py:59` |
| ConvTranspose1d / 2d | 有 | 【源码】`mlx@v0.32.4:python/mlx/nn/layers/convolution_transpose.py:10,86` |
| LayerNorm / BatchNorm | 有，LayerNorm 走 `mx.fast.layer_norm` 融合内核 | 【源码】`mlx@v0.32.4:python/mlx/nn/layers/normalization.py:86,130,279` |
| PReLU、Sigmoid、Tanh、Linear | 有 | 【源码】`…/layers/__init__.py:16,42` |
| Unfold | 没有同名层。可用切片 / `as_strided` 拼出来【推断】 | — |
| ScatterND（缓存写回） | 不需要。MLX 是函数式写法，缓存直接作为新数组返回即可【推断】 | — |
| STFT 类（FE-M wav2wav 图里有） | 有 `mx.fft`；ORT-MLX 插件也列了 DFT/STFT | 【官方】`onnxruntime-ep-mlx` docs/OP_ARCHITECTURE.md:367 |
| 自注意力 | 有 `mx.fast.scaled_dot_product_attention` 和 `nn.MultiHeadAttention`【推断，未逐页核】 | — |
| 神经引擎（ANE） | **用不上**。MLX 只有 CPU 和 GPU 两种设备，不调 ANE。所以 Core ML 走 ANE 可能带来的省电，MLX 拿不到 | 【官方】https://github.com/ml-explore/mlx README（「currently the CPU and the GPU」）|
| 张量布局 | 卷积是**通道在后**（NHWC / OHWI），从 PyTorch 搬权重要转置 | 【实测-他人】下方 GTCRN 移植说明「ONNX Conv 权重转成 MLX OHWI，分组 ConvTranspose 另做变换」 |

**权重怎么搬**：MLX 用 `safetensors` / `npz` 载权重（`mx.load`）。路线是 PyTorch `state_dict` → 改名 + 转置 → `safetensors`【推断】。GRU 的偏置布局要对齐 PyTorch 的 `b_ih / b_hh`，MLX 把 `bhn` 单列【源码】`recurrent.py` GRU 定义。

**ONNX 导入**：**官方没有**。issue #16「ONNX/Torch 转 MLX」以重复关闭【官方】https://github.com/ml-explore/mlx/issues/16。第三方有三条：
- `onnx2mlx`（muna-ai）：PyPI 0.0.5，2026-03-12 之后没有新版，算子覆盖未查【官方-第三方】https://pypi.org/project/onnx2mlx/
- **`agentable/gtcrn-simple-mlx`**：从 sherpa-onnx 的 `gtcrn_simple.onnx`（SHA-256 `e77603ac…b534`）做「只换布局」的确定性转换，179 个张量、208 427 字节 fp32、MIT；自带**逐帧流式 MLX 运行器**，管三类缓存；在 Metal fp32 上对 ONNX 逐帧 oracle 的最大绝对误差 < 5e-5；**没给延迟数字**；整段波形和语料质量没测【实测-他人】https://huggingface.co/agentable/gtcrn-simple-mlx 。⚠️ **它的源文件不是我们那份**：vdn 用的 `res:gtcrn/stream/onnx_models/gtcrn_simple.onnx` 本机 SHA-256 是 `b4718df6…cb87`（怀疑派用 `shasum -a 256` 实算），和它的 `e77603ac…b534` 不同。权重是否相同未核，所以「< 5e-5」是对 sherpa 那份 ONNX 的误差，不是对我们 ORT 输出的误差。页面写运行器是「重放钉死的单帧图」，可能是按 ONNX 图逐节点执行，而不是 `nn.GRU` 写法；TF32 开没开没写。它链接的代码仓 `github.com/agentable/model-gtcrn` 我访问是 404（可能私有），实现语言和调用方式未知
- **`onnxruntime-ep-mlx`**：ONNX Runtime 官方组织下的**插件执行提供者**（Rust 写、直接绑 mlx-c、不经 mlx-rs），要求 ORT ≥ 1.31，MIT。GRU 用「按形状展开」实现（仅 layout=0、默认激活、无 sequence_lens），Conv / ConvTranspose / PRelu / BatchNormalization / LayerNormalization 都在表里，不认的节点留给 ORT CPU。README 原话是「CPU 回退和小负载可能抵消 GPU 收益」「结果与 CPU 不逐位一致」。仓库只有 4 星，非常新【官方】https://github.com/onnxruntime/onnxruntime-ep-mlx README:49,53；GRU 限制见 `docs/OP_ARCHITECTURE.md:94`；ScatterND 归在 `shape.rs` 的 Gather/scatter 一类（`:594`），具体覆盖未核。怀疑派更正：原文引的 `:592-599` 是模块表，**里面没有「派发受限，CPU 占优」这句话**，已改成 README 原话。另：它在每个融合子图的边界做一次 `mlx_eval`（`OP_ARCHITECTURE.md:52`），被认领的浮点节点默认走 Metal（`:89`）。如果 GTCRN 被不认的节点切成几段，每帧就有几次 GPU 同步加几次 CPU↔GPU 交接【推断】

## 2. 每帧固定开销

**执行模型**【源码】：
- 每个算子调用先建一个惰性图节点（`array` 内含 `shared_ptr<Primitive>` 等）`mlx@v0.32.4:mlx/array.h:210-216`
- `eval` 进 `eval_impl`，每次新建若干 `unordered_map`（fence、event、cache）`mlx/transforms.cpp:80,99-118`，最后 `.wait()` 等完成 `:381-395`
- **CPU 流**：每个算子打包成 `std::function`，经互斥锁 + 条件变量队列丢给该流**专属的后台 `std::thread`** 执行 `mlx/backend/cpu/encoder.h:43-56`、`mlx/scheduler.cpp:42-92`；每个算子还新建一个 `unordered_set` 持有缓冲区引用 `mlx/backend/cpu/eval.cpp:38-51`。调用线程靠 `promise/future` 等后台线程 `scheduler.cpp:14-24`
- **GPU 流**：每个算子编码进 Metal 命令缓冲，每满 20–50 个算子提交一次（按芯片档位，环境变量可改）`mlx/backend/metal/device.cpp:537-540,631-652`；每个算子开一个 autorelease 池、加一个完成回调 `mlx/backend/metal/eval.cpp:29-69`；同步走 `waitUntilCompleted` `device.cpp:590-595`

**公开量级**（都不是本机）：
- `mx.eval` 内放 1/2/4/8/16 个空算子，每次 eval 都约 **0.11 ms**：同步本身是主成本，算子个数影响小。M4 Pro · MLX 0.32.2 · macOS 26.6.2【实测-他人】https://github.com/ml-explore/mlx/discussions/4512 。帖子**没写是 CPU 流还是 GPU 流**；因为同帖用了 `mx.fast.metal_kernel`，推断是 GPU【推断】。所以这 0.11 ms 不能套到 CPU 流上
- Metal 空内核 `waitUntilCompleted` 约 **0.12 ms**（M2 Max），同一帖 CUDA 约 0.001 ms【实测-他人】https://developer.apple.com/forums/thread/767853 （无苹果回复，方法未公开）
- 同帖 4512：16 个逐元素算子链 0.133 ms/步；手写一个 `mx.fast.metal_kernel` 后 0.0225 ms/步【实测-他人】

**`mx.compile` 能融合到什么程度**【源码 + 官方】：
- 只融合**逐元素一元 / 二元 / 三元和广播**，深度 ≤ 11、输入 ≤ 24 个数组 `mlx/compile.cpp:25-26,78-79,857-858`。**matmul、卷积、归一化、GRU 循环本身都不融**，只把它们之间的逐元素尾巴并掉
- 官方文档原话：小数组时函数是「受开销支配」的，融合能缓解【官方】`docs/src/usage/compile.rst:102-105`
- **CPU 上的 compile 要在运行时调系统 C++ 编译器**：先 `std::system("g++ --version")` 探测 `mlx/backend/cpu/jit_compiler.cpp:196-215`，再把生成的 C++ 写到临时目录、编成 `.so`、`dlopen` `mlx/backend/cpu/compiled.cpp:101-140`。探测失败就**静默关掉 compile** `compile.cpp:1110-1113`、`compiled.cpp:52-57`。用户电脑多半没装命令行工具，`/usr/bin/g++` 这个存根还可能弹出「安装开发者工具」对话框【推断，未在无 CLT 的机器上验证】
- GPU 上的 compile 生成 Metal 源码运行时编译，首次调用慢【官方】`compile.rst:43`

**套到 GTCRN 上**【推断】：
- GTCRN 的帧内 GRU 沿 33 个频点双向扫，共 2 个 DPGRNN 块，组 GRU 再拆 2 组。按 `nn.GRU` 写法，一帧约 2 × 2 × 2 × 33 = 264 个时间步，每步约 12–15 个图节点（见 §1），**每帧约 3000–4000 个小算子**。每步的矩阵很小：帧内 GRNN 隐层 8，分 2 组后每组每方向只有 4（`res:gtcrn/stream/gtcrn_stream.py:171-172,202`）。另有跨帧 GRU 2 块 × 2 组各 1 步，以及 TRA 注意力里的 GRU（`gtcrn_stream.py:82`，缓存 `tra [2,3,1,1,16]` 提示共 6 个）各 1 步，再加约 10 步。ORT 里这些是 `GRU` 节点，时间步循环在 ORT 的 C++ 内核里跑，不经图调度【推断：没打开 ONNX 数节点】
- 不写内核也能少一半以上：两组、两个方向的权重不同，但可以叠成一次批量 matmul（`mx.matmul` 带批维）或块对角矩阵。这样串行步数从 264 降到 2 块 × 33 = 66 步。数学等价，但累加顺序变了，逐值不一定一致【推断】
- GPU 路线：每帧至少一次约 0.1 ms 量级的同步，加上千次编码、二三十次命令缓冲提交。今天 ORT CPU p50 约 0.4 ms，**GPU 路线大概率更慢**
- CPU 路线：每个算子在调用线程上先分配输出缓冲（如 `binary_op_cpu` 里的 `set_binary_op_output_data`），再至少入队 2 次：一次是计算，一次是释放输入引用的空任务（`mlx/backend/cpu/eval.cpp:38-51`）；每 10 次还多入队一次完成通知（`encoder.h` 的 `DISPATCHES_PER_TASK = 10`）。每次入队都加锁、`std::function` 堆分配、`notify_one`，可能唤醒后台线程【源码】v0.32.4 `encoder.h:13,45-56` 与本地 main 版一致。一帧几千个算子就是上万次入队。量级上比 ORT 的「每帧 1554 次 malloc」只多不少。没有证据说它更快【推断】
- 唯一可能翻盘的写法：用 `mx.fast.metal_kernel` 手写**一个融合整段 GRU 循环的内核**（社区有 mlx-recurrence 这种做法，见 https://origin-devforums.apple.com/forums/profile/paulderr 相关帖，未读到数字），再把其余部分 compile。那时一帧可能只剩几十个派发 + 一次同步，下限仍是那 ~0.1 ms 的同步
- **逐帧 vs 整段批处理**：批处理时 GRU 跨帧那一维仍要逐帧，但帧内的卷积、频点 GRU 可以按帧批起来，固定开销被摊薄。差距多大**未量化**，要测（见「基准要测什么」）

**套到 FE-M 上**【推断】：每帧约 29 MMAC，比 GTCRN 更接近算力受限，GPU 的矩阵单元有发挥空间；但每 10 ms 一次的同步成本仍在。FE-M 的 GRU 是否只沿时间走 1 步（还是也沿频率扫），我没读到对应行，影响算子数，待核。

## 3. 实时安全

**结论：MLX 的 `eval` 不能在我们的推理线程上当作「只有已登记的库内分配」来用**【源码 + 推断】。原因逐条：
1. **分配**：每个算子建图节点（`shared_ptr`）、每次 eval 建哈希表、每个算子一个 `std::function` 和一个 `unordered_set`（见 §2 的行号）。这些是普通堆分配，和 ORT arena 不同，**没法预热消掉**【源码】
2. **缓冲区缓存**：Metal 分配器先查缓存，命中就复用；不命中才 `newBuffer` `mlx/backend/metal/allocator.cpp:129-152`。稳态下大多能命中【推断】。v0.32.0 起无 GPU 的 CPU 分配器也有缓存【官方】https://github.com/ml-explore/mlx/releases
3. **锁**：分配器全局互斥锁 `allocator.cpp:129`；调度器互斥锁 `mlx/scheduler.h:36-50`；每个流的队列锁 `scheduler.cpp:78-91`；取流线程时的读写锁 `scheduler.cpp:148-161`【源码】
4. **后台线程 + 跨线程等待**：CPU 流的**算术**不在调用线程上跑，而在 MLX 自己起的 `std::thread` 上跑 `scheduler.cpp:42-76`。怀疑派更正：原文写「活不在调用线程上跑」太绝对。实际是各算子的 `eval_cpu`（形状推导、输出缓冲分配）在调用线程上跑，只有计算闭包经 `encoder.dispatch` 交给后台线程【源码】`mlx@v0.32.4:mlx/backend/cpu/eval.cpp:35`；`binary.h` 的 `binary_op_cpu`（本地 main 版）。所以分配同时发生在两条线程上。这个线程用默认优先级，MLX 不暴露设置接口【源码；「不暴露」是推断】。我们的推理线程要阻塞等它，形成**优先级反转**：系统一忙，尾部就由这个普通线程的调度决定【推断】。补充：等待用的是 `promise/future` 和条件变量，这类等待没有「持有者」，内核无从做优先级继承；把我们自己的线程设成实时，也带不动 MLX 的后台线程【推断】。GPU 流则阻塞在 `waitUntilCompleted` 或事件上
5. **GPU 抖动**：GPU 由窗口服务器、视频会议的渲染、浏览器共用；Metal 计算队列创建时没设优先级 `device.cpp:331`【源码】。更准确地说是**公开接口就没有**：本机 SDK 27.0 的 `MTLCommandQueue.h` 只有 `label`、`device`、`maxCommandBufferCount`、`logState` 等属性，没有优先级；只有 `MTLIOCommandQueue`（文件读写队列）有 `MTLIOPriority`（`MTLIOCommandQueue.h:20-23,144`）【官方，本机头文件】。所以换任何走 Metal 计算的引擎，都没法让降噪的 GPU 活插队【推断】。GPU 空闲降频后再唤醒、别的应用占 GPU，都会拉长 p99.9 / max【推断，未找到公开测量】
6. **异常与错误**：错误以 C++ 异常在流线程里存起来、下次同步时抛 `scheduler.cpp:101-114`；mlx-c 把异常转成错误回调 `mlx-c:mlx/c/error.h:15-20`【源码】

**能不能直接调**：音频 IO 回调里绝对不行。B 档推理线程里调，也违反「不阻塞等低优先级线程」这条精神，规约现在没写这条，要大总管 / 人定是否算违规【推断】。若要用，只能让推理跑在「判断回路」那种准实时线程上，或者整个推理线程允许偶发超时送原声（规约 §4 已有超时送原声）。

## 4. 从 Rust 调

- **mlx-c**：ml-explore 官方 C 绑定，最新标签 v0.8.0；头文件覆盖 array / ops / fast / compile / transforms / export / io / stream / metal / memory【官方】https://github.com/ml-explore/mlx-c （`mlx/c/` 目录列表）。含 `mlx_imported_function_new/apply`，能载 Python 导出的 `.mlxfn` 图 `mlx-c:mlx/c/export.h:58-64`【源码】
- **mlx-rs**：仓库已迁到 `oxiglade/mlx-rs`（旧名 oxideai），386 星，2026-11-02 当天还有提交，**很活跃**。crates.io 最新 `mlx-rs 0.33.0`（2026-10-28）、`mlx-sys 0.7.0`。README 写「积极开发中」，并警告 grad 闭包捕获外部数组「可能段错误」【官方】https://github.com/oxiglade/mlx-rs ；crates.io API
  - 版本跟随：`mlx-sys` 把 mlx-c 作为子模块（提交 `a341b49`），mlx-c 的 CMake 钉的是 **MLX v0.32.2** `mlx-c@a341b49:CMakeLists.txt:38`【源码】，比上游最新 0.32.4 落后两个补丁版
  - 构建方式：`mlx-sys/build.rs` 用 cmake 从源码编 mlx-c 和 MLX【源码】`mlx-sys/build.rs:1-3,111`。构建机要有 Xcode 的 Metal 工具链；用户机要能找到 `mlx.metallib`【推断】
  - 已有模块：`nn/` 下有 `recurrent.rs`（Rnn / Gru / Lstm，Gru 同样是 `for index in 0..x.dim(-2)` 逐步循环，`recurrent.rs:231,312`）。⚠️ **Gru / Rnn 的循环不递推隐状态**（见 §1 表「mlx-rs 的 Gru」），不能直接用，要自己写或先给上游提修复、`convolution_transpose.rs`（1d/2d/3d）、`normalization.rs`（LayerNorm / BatchNorm 等）、`activation.rs`（Prelu `:632`）；`transforms/` 下有 `compile`、`grad`、`value_and_grad`；还有 `optimizers`【源码】
  - `.mlxfn` 导入有没有安全封装：未查到；`mlx-sys` 是 bindgen 全量绑定，裸调可行【推断】
- **mlx-swift**：官方 Swift 绑定，最新 0.33.0（2026-10-27）【官方】https://github.com/ml-explore/mlx-swift/releases 。走 Swift 薄壳再 C ABI 给 Rust，比直接 mlx-c 多一层，没看到好处【推断】
- **建议**：若要测，Rust 侧用 `mlx-sys`（即 mlx-c）裸调最直接；模型结构在 Python MLX 里写、对拍、`export_function` 成 `.mlxfn`，Rust 侧 `mlx_imported_function_apply`。或者直接测 `onnxruntime-ep-mlx`，因为它保留 `ort` crate 和现有 ONNX 文件（我们的 ort 2.0.0-rc.15 内含 ORT 1.31，正好满足 ≥ 1.31；`ort` crate 能否注册插件 EP **未查**）【推断】

## 5. 自动求导与本机微调（D52）

- MLX 是可组合函数变换框架：`grad`、`value_and_grad`、`vjp`、`jvp` `mlx/transforms.cpp:398-621`【源码】；`nn.Module` 有 `freeze / unfreeze / trainable_parameters`，配 `mlx.optimizers`【官方，MLX 文档 nn 章节；未逐行核】
- 只训 BN/LN 仿射（约 4.8 K 参数）或小适配器：冻结其余参数，`nn.value_and_grad(model, loss)` 即可。mlx-rs 也有 `value_and_grad.rs`、`optimizers/`【源码】
- **这是 MLX 相对 Core ML 推理和手写 Rust 前向的真实优势**：后两者都不自带反传【推断，Core ML 的设备端更新另题查】
- 但要注意：
  - 跨帧 GRU 训练要沿时间反传（截断 BPTT），GTCRN 逐帧 + 上千小算子的图，反传的图更大，适合**整段批处理**，不适合逐帧【推断】
  - 训完的参数要回灌到推理后端（ORT 图输入的 BN/LN 覆盖层，`specs/rust/architecture.md:252`），所以 MLX 前向必须和 ORT 前向逐值对拍【推断】
  - 评分模型如果要当可微奖励，也得在 MLX 里重写一遍（012 已写 ONNX 路线不可微）【推断】

## 6. 精度

- fp32 全面支持【官方】
- **坑 1：M5 上 fp32 矩阵乘默认走降精度**。MLX 在 macOS ≥ 26.2、GPU 代数 ≥ 17 时启用 NAX（GPU 神经加速单元）`mlx/backend/metal/device.cpp:978-995`；只要 `MLX_ENABLE_TF32` 为真（**默认 1**，`mlx/utils.h:225-228`），fp32 的 matmul 就走 NAX `mlx/backend/metal/matmul.cpp:964-966`。官方文档说 matmul、卷积、注意力都可能降精度，误差「比 fp32 舍入大几个数量级」【官方】`docs/src/usage/precision.rst:6-18`、`environment_variables.rst:23-27`。**对拍必须设 `MLX_ENABLE_TF32=0`**，本机 M5 · macOS 27 正好命中这条。怀疑派补核：本机 `ioreg -l` 里 GPU 驱动类名是 `AGXAcceleratorG17G`，即第 17 代、后缀 `g`，满足 `gen >= 17`（`device.cpp:991`，非 `p` 后缀要求 ≥ 17）【本机读取 + 源码；类名与 `get_architecture_gen()` 返回值是否同源，是推断】。这条只管 GPU；CPU 流的 matmul 走 Accelerate 的 fp32 BLAS，不受 TF32 影响，但和 ORT 的 MLAS 累加顺序不同，也不会逐位一致【推断】
- **坑 2：Winograd**。Metal 上 3×3、步长 1 的 conv2d 可能走 Winograd，fp32 误差约大 10 倍，且是否启用取决于总输入大小，**批 1 和批 3 结果不同**。用 `MLX_CONV_WINOGRAD=0` 关掉【官方】`precision.rst:23-33`
- Metal 内核本身用 `-fno-fast-math` 编译【源码】`mlx/backend/metal/kernels/CMakeLists.txt:18`；v0.32.3 修了 compile 里浮点常量精度、让 sigmoid 在编译和即时两种模式下用同一个精确 exp【官方】releases 页
- 社区帖 4512：compile 后出现 1 ulp 差异，原因是标量常量只打印 7 位有效数字（#4503，已修）【实测-他人】
- 第三方 GTCRN 移植在 Metal fp32 上最大绝对误差 < 5e-5（那时 TF32 开没开不知道）【实测-他人】
- 结论：**和 ORT 逐位一致不可能**；能争取的是「最大绝对误差 ≤ 某阈值、逐帧带状态跑 10 s 不发散」。阈值要由 verifier 定，我不拍【推断】

## 对 vdn 的含义

**GTCRN simple**
- 能不能跑：能。已有第三方逐帧移植，对拍误差 < 5e-5【实测-他人】。但它对的是 sherpa 那份 ONNX，和我们那份哈希不同（§1），对我们的 ORT 输出是多少**未知**
- 预计瓶颈：**调度与同步**。帧内双向 GRU 被拆成上千个小算子；GPU 每帧还要付约 0.1 ms 量级的同步【推断】
- 实时线程能不能直接调：**不能**。eval 内有普通堆分配、多把锁、阻塞等 MLX 自己的默认优先级线程或 GPU（§3）【源码 + 推断】
- 要不要改模型结构：用 `nn.GRU` 不用改，但会慢。想快就要手写融合 GRU 的 Metal 内核，或把帧内 GRU 改写成别的形式（会改变数值）【推断】
- 逐值一致风险：中。GPU 上要关 TF32 和 Winograd；Python 版 GRU 的 #3249 已修，但仍建议显式零初值；双向要自己拼。**Rust 侧不能用 mlx-rs 的 `Gru`**（不递推隐状态，§1）【官方 + 源码 + 推断】
- 一句话：**不推荐当 GTCRN 的实时后端；可以当 GTCRN 的本机微调后端**【推断】

**FastEnhancer-M**
- 能不能跑：能，算子都有（StridedConv1d、ConvTranspose1d、BN、GRU、注意力、FFT）。没有现成移植，需要从 PyTorch 源码重写并搬权重【推断】
- 预计瓶颈：算力占比比 GTCRN 高，GPU 矩阵单元有收益空间；但每 10 ms 的同步固定成本仍在，且 FE-M 每帧只有约 29 MMAC，GPU 难以吃满【推断】
- 实时线程能不能直接调：同 GTCRN，不能
- 省电：MLX 不用 ANE（§1）。FE-M 如果要省电，ANE 只能走 Core ML，MLX 这条路没有【官方 + 推断】
- 要不要改模型结构：wav2wav 图里的 STFT 要用 `mx.fft` 重写；或改用 spec2spec 版，STFT 留在 Rust【推断】
- 逐值一致风险：高于 GTCRN。注意力和大矩阵乘最容易受 TF32 影响【官方 + 推断】

**对后端 trait 的影响**【推断】：如果 MLX 只做微调，trait 里不用为它留实时入口；但要留「把一组参数（BN/LN 仿射或适配器）从微调后端交给推理后端」的接口，和规约 §3.5 的预热、交叉淡化接上。

## 基准要测什么（给 applebe）

> 目的：用本机数字确认或推翻 §2–§3 的推断。全部在 M5 · macOS 27.0.1 上测，接电源与电池各一遍。

**被测对象**（同一份权重）：
1. GTCRN：Python MLX 版（可借 `agentable/gtcrn-simple-mlx` 的运行器做起点，先核许可和代码；**权重必须从我们那份 `gtcrn_simple.onnx`（SHA-256 `b4718df6…cb87`）重新转**，不能用它那份 `e77603ac…`，否则不是同一份权重）+ Rust `mlx-sys` 调 `.mlxfn` 版。Rust 侧 GRU 自己写，别用 mlx-rs `nn::Gru`
2. FE-M：先只做 Python MLX 版，结论好再做 Rust 版
3. 对照：现有 ORT CPU 单线程；以及 `onnxruntime-ep-mlx` 插件（`ONNXRUNTIME_EP_MLX_CLAIM_DEBUG=1` 先看哪些节点被认领）

**配置矩阵**：
- 设备：`mx.cpu` / `mx.gpu`
- `mx.compile`：关 / 开（CPU 开 compile 要先确认本机 `g++` 存在，并记录首次编译耗时和临时目录产物）
- 精度：`MLX_ENABLE_TF32=0` / `1`；`MLX_CONV_WINOGRAD=0` / 默认
- GRU 写法：`nn.GRU` 逐步 / 手写 `mx.fast.metal_kernel` 融合整段 GRU（只 GPU）
- 调用粒度：逐帧一次 `eval` / 每 N 帧一次（N = 1, 8, 64, 整段 10 s），用来量出「固定开销 vs 算力」
- 环境：`MLX_MAX_OPS_PER_BUFFER` 默认 / 调大（看提交次数的影响）
- 线程：推理线程设成实时时间约束（`thread_policy_set` TIME_CONSTRAINT）/ 默认

**口径要求**（怀疑派补）：
- **尾部数字只认 Rust 版**。Python 版的每帧时间含解释器和 GIL，只能用来对拍和看 p50 的上限，p99.9 / max 不进结论
- Rust 版要用和 ORT 基准同一套计时框架（exp010 的那套），同一段输入、同一帧数、同一台机、同一电源状态
- 从**非主线程**调用：MLX 的流是线程局部的，`enqueue` 只在主线程查错（`scheduler.cpp` 的 `is_main_thread()`），我们的推理线程不是主线程，要测这种调法能不能跑、出错能不能报上来

**测哪些量**：
- 每帧墙钟时间：p50 / p99 / p99.9 / max，≥ 10 万帧（GTCRN 约 27 分钟音频）。前 100 帧单列（预热、编译）
- 有干扰时的尾部：同时放全屏视频或跑一个 GPU 负载，再测 p99.9 / max
- 每帧分配次数与字节（malloc 计数插桩）、每帧加锁次数、每帧线程切换次数，按函数名列前 10（给 RTSan 豁免表用）
- MLX 起了几个线程、各自优先级 / QoS
- 每帧 CPU 时间：调用线程与 MLX 后台线程**分开**统计（`thread_info` 或 `proc_pidinfo` 按线程取），加起来和 ORT 单线程比。只看墙钟会漏掉后台线程的开销
- GPU 侧：`MTLCommandBuffer` 的 `gpuStartTime / gpuEndTime`，算出 GPU 真算的时间和「提交到完成」的差
- 耗电：`powermetrics` 的 CPU / GPU 功耗（mW），每种配置跑 5 分钟取均值，和 ORT CPU 单线程对比（D31）
- 逐值一致：对 ORT 输出逐帧最大绝对误差、10 s 带状态连续跑后的误差、输出缓存的误差；TF32 开 / 关各一遍
- 冷启动：加载 + 首帧耗时；`mlx.metallib` 和库文件的体积（安装包用）
- 微调可行性（单独一组）：冻结其余参数只训 BN/LN 仿射，10 s 片段一次前向 + 反传的耗时、峰值内存、功耗

## 未查 / 存疑

- **本机没有任何 MLX 数字**。§2 的 0.11 ms / 0.12 ms 是 M4 Pro / M2 Max 上别人的测量，方法不全公开
- `agentable/model-gtcrn` 代码仓访问 404，GTCRN 移植的语言、调用方式、是否用了融合 GRU、有没有延迟数字都不知道；也没核它的许可是否覆盖运行器代码
- FE-M 里 GRU 沿哪一维扫、每帧多少时间步，没读到 `model.py` 对应行
- MLX 流线程的优先级 / QoS 能否从外部改（比如 `pthread` 层面找到线程再设），没查
- `new_thread_unsafe_stream`（`mlx/stream.h:43`）是否能让 CPU 活在调用线程上同步执行、绕开后台线程：我读到它仍然经 `StreamThread`，但没把调用链读完，**存疑**
- M5 的 GPU 代数：怀疑派从 `ioreg` 读到 `AGXAcceleratorG17G`，按源码条件 NAX 会启用；没运行 MLX 确认
- mlx-rs `Gru` 不递推的问题：只读了代码，没跑；上游有没有人报过没查
- `agentable` 那份 GTCRN 权重和我们那份是否逐值相同：哈希不同，没拆开比
- `ort` crate 2.0.0-rc.15 能不能加载 ORT 插件 EP，没查
- `onnxruntime-ep-mlx` 对 GTCRN 里 ScatterND、Unfold 生成的算子、分组 ConvTranspose 是否都认领，没逐个核；它很新（4 星），稳定性未知
- `onnx2mlx` 的算子覆盖与维护状态没查（PyPI 页面没加载出来，只拿到元数据）
- MLX 在 GPU 空闲降频后再唤醒的延迟，没找到公开测量
- MLX 的 nn 文档关于 `freeze` / `value_and_grad` 的页面我没逐页打开，等级按源码目录推定
- 苹果 Metal 头文件里计算队列优先级相关接口，本次没查本机 SDK

## 怀疑派核查（2026-11-02）

> 做法：逐条打开 URL、对 GitHub 上 v0.32.4 原文和本机 SDK 头文件核行号。没装包、没构建、没跑基准。
- 核对：v0.32.4 的 `utils.h:225-228`（TF32 默认 1）、`device.cpp:331,537-540,978-995`、`precision.rst` 原文都对得上；讨论帖 4512、issue #1500 的转述对得上
- 驳倒：#3249 写成「现存坑」→ 已在 2026-03-16 修好（PR #3252），v0.32.4 Python 版没这个错
- 新增反例（最重要）：mlx-rs 的 `Gru::step` 循环里重新绑定 `hidden`，隐状态不递推，GTCRN 帧内 33 步会算错；`hidden=None` 也还漏 `bhn`
- 驳倒：`agentable` 移植被当成「同一份权重」→ 它的源 ONNX 哈希 `e77603ac…` ≠ 我们的 `b4718df6…`，误差 < 5e-5 不能套到我们身上
- 驳倒：`onnxruntime-ep-mlx` 的「派发受限，CPU 占优」引文在所引行号里不存在，改为 README:49,53 原话
- 降级：讨论帖 4512 的 0.11 ms 没写是 CPU 还是 GPU 流，补「推断是 GPU，不能套到 CPU 流」
- 更正：CPU 流不是全部在后台线程，`eval_cpu`（含输出分配）在调用线程，算术在后台线程；每算子至少 2 次入队
- 更正：每步 GRU 约 12–15 个节点（原写 6–10），每帧约 3000–4000 个算子；补上 TRA 里的 6 个单步 GRU
- 补充：两组、两个方向可以叠成批量 matmul，串行步数 264 → 66，不用写内核
- 补充：MLX 不用 ANE，省电上拿不到 Core ML 走 ANE 的那份
- 补充：Metal 计算队列的优先级不是 MLX 没设，是本机 SDK 27.0 公开接口就没有（只有 IO 队列有）
- 补核：本机 GPU 为 `AGXAcceleratorG17G`，NAX 条件满足，GPU 对拍必须关 TF32；CPU 流不受 TF32 影响但也不会和 MLAS 逐位一致
- 基准补口径：尾部只认 Rust 版、用 exp010 同一套计时、分线程统计 CPU 时间、从非主线程调用、权重从我们那份 ONNX 重转
- 仍存疑：`ort` crate 能否注册插件 EP；`onnxruntime-ep-mlx` 对 GTCRN 每帧切成几段；MLX 后台线程的 QoS 能否从外部改；GPU 降频唤醒延迟；`new_thread_unsafe_stream` 的完整调用链

