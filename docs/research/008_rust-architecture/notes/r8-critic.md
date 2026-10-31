# R8 · 完整性批评：R1–R6 还缺什么、哪里互相打架、明天写代码会卡在哪

> 调研线：research/008-rust-architecture · 批评轮 · 2026-10-30
> 读过：`r1-ast.md` `r2-crates.md` `r3-traits.md` `r4-realtime.md` `r5-talks.md` `r6-quality.md`（同目录）
> 证据等级：【官】官方明载 /【源】源码可证 /【测】本机实测 /【传】二手 /【推】推断
> `R/` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`；`S/` = 本 scratchpad 目录
> 本轮自己补查了 8 项（§3），其中 3 项是本机实测。**有一项实测把 R3/R4 的「待测」直接定了性：ort 在稳态每帧约 1550 次 malloc 和 376 次加锁（§3.1）。**

---

## 0. 一页结论

1. **六篇在「方向」上一致**：扁平 `crates/`、核心不碰 IO、构建期和处理期分开、AST 用来「守」而不是用来「生成」、用三层工具证明实时安全。这些可以直接写进设计稿。
2. **但在「写代码要用的名字和边界」上没对齐**：trait 名有 4 套（`Denoiser` / `InferBackend + SpectralModel` / `Denoiser::process_frame` / `engine-ort`），引擎入口有 4 个名字（`process_hop` / `process` / `process_frame` / `process(…, ctx)`），共享内存布局放在 3 个不同的 crate，`dsp` 要不要 `no_std` 有 3 种说法，derive 宏有「推荐」也有「不采用」。要写「crate 列表与依赖图」和「核心 trait 草案」，必须先把这 10 处矛盾定掉（§1）。
3. **ort 不能当「零分配」用，这一条已经实测确认，不再是【推】**：GTCRN 流式模型、ort 2.0.0-rc.15、intra=1、IoBinding 预绑定、预热 200 帧之后，`OrtApis::RunWithBinding` 稳态**每帧约 1554 次 malloc、1555 次 free、376 对 pthread_mutex lock/unlock**（RTSan 逐次计数）【测】。来源主要是 ORT 的 `BFCArena`、`Tensor::InitOrtValue`、`MemPatternPlanner::TraceAllocation`、MLAS GEMM【测】。ort 自己的 Rust 封装 `run_binding` 每帧还多 28 次 Rust 分配加释放，`run` 每帧多 54 次【测】【源】。耗时没问题：p99.9 是 0.80–0.94 ms，max ≤ 1.07 ms，hop 是 16 ms【测】。**所以 R3 待拍板第 1 题（推理线程的承诺范围）事实上只剩选项 A**：IO 回调和搬运线程零分配；推理线程「有界、不阻塞别人、按 p99.9 和 max 验收」。
4. **R2「cargo-deny `wrappers` 能机器证明送音进程不链 ort」这一条是错的**：cargo-deny 只检查被禁 crate 的**直接**依赖者是不是都在白名单里【源】`R/cargo-deny/src/bans.rs:616-650`。`vdn-capture → vdn-infer-ort → ort` 是合法的。要证明「依赖树里没有」，得靠 xtask 读 `cargo metadata` 做传递闭包检查，或者给 `vdn-capture` 单独写一份 `deny.toml`。
5. **没有一篇算过端到端延迟账**：GTCRN 用 512/256 的 WOLA，算法延迟约 32 ms【源+推】；002 的口径是「管线直通 6–10 ms + 窗长 + 前瞻」【源 `docs/research/002_model-baseline/must-know.md:17`】，合起来 38–42 ms，离 45 ms 只剩 3–7 ms。还没算 48↔16 kHz 两次重采样（rubato `Async` sinc 的延迟是 `nbr_points × ratio / 2` 个输出帧【源】`R/rubato/src/asynchro.rs:747-749`）和推理耗时。**重采样器放在哪、选哪种，是延迟和「eval 与实时同一份代码」两条硬约束的交点**，六篇都没回答。
6. **明天写代码第一批会卡的地方**（§4）：cpal 本机克隆是**未发布的 0.19 master**，crates.io 最新是 0.18.2，回调类型名不一样；ort 只有 rc 版；tract 的 GTCRN 算子在 onnx 前端**全部有注册**【源】，但没能实际跑起来（构建外部克隆被权限拦下）；`os_workgroup_join` 没有现成 Rust 绑定；GTCRN 的 hop 是 16 ms，003 的环形缓冲按 10 ms 台阶设计。

---

## 1. 互相矛盾的结论（要先定，否则 trait 草案和依赖图写不出来）

| # | 话题 | 各篇说法 | 影响 | 建议的裁决【推】 |
|---|---|---|---|---|
| C1 | **核心 trait 的名字和层次** | R2：`vdn-types` 里放 `Denoiser`，engine 只拿 `Box<dyn Denoiser>`，后端 crate 实现它。R3：两层，`InferBackend`（Sized、故意不做成对象安全）加 `SpectralModel`（dyn），外加 `ModelBuilder`；`Gtcrn<B: InferBackend>`。R4 §5.1：`infer` crate 里放 `trait Denoiser { fn process_frame(&Frame,&mut Frame)->Status }`，GTCRN 和 FastEnhancer 的实现也在 infer 里。R6：`engine-ort` | 「模型知识」放在哪个 crate 有三种答案：models、后端、infer | 采用 R3 的两层：`InferBackend` 只懂张量，`SpectralModel` 懂 GTCRN 的语义。trait 放在 `vdn-types`（或者单独的 `vdn-model-api`），这样 `vdn-models` 里的 `Gtcrn<B>` 不依赖任何后端，后端 crate 也不依赖 models。bin 负责实例化 `Gtcrn<OrtBackend>` 并装箱成 `Box<dyn SpectralModel>`。**R2 的 `Denoiser` 改名为 `SpectralModel`，或者反过来，二选一，写进命名规约** |
| C2 | **引擎入口的签名** | R3 `process_hop(&mut self, input, output)`；R5 `process(&mut self, in, out, ctx: &FrameCtx)`；R1 和 R6 写 `process_frame`；R4 `process(...) -> Status` | eval、RTSan 回放、proptest 都要调它 | 统一成 `Engine::process(&mut self, input: &[f32], output: &mut [f32], ctx: &FrameCtx) -> Status`。`FrameCtx` 带样本计数和声纹快照，这样 eval 可以复现时序（R5 第 1 条）。返回 `#[must_use] Status`（R4 R11）。**输入长度任意，引擎内部攒 hop**，这样 R6 的「分块不变性」性质测试才有意义；如果引擎只收整 hop，分块不变性测试就退化成一句空话 |
| C3 | **共享内存布局放在哪** | R1 §5.1 放在 `vdn-types`，映射放在 `vdn-io-macos`；R2 和 R4 放在独立的 `vdn-shm`（`no_std`、cbindgen）；R6 的 `engine-sync` 又是另一个 crate | cbindgen 的输入、loom/miri 的检查对象、unsafe 白名单都跟着它走 | 采用 R2：独立的 `vdn-shm`。它同时是 R6 说的「自写同步原语集中地」，loom 和 Miri 只盯这一个 crate。`vdn-types` 保持零 unsafe |
| C4 | **`dsp` 要不要 `no_std`** | R5 第 6 条：「`dsp` crate `no_std` 无 alloc」；R4 §2.6：`no_std + alloc`，而且只适合纯内核；R2：核心不强求 `no_std`，RustFFT/rubato 都不是 `no_std`；R1：「no_std 友好」 | STFT 放在 dsp（R2），STFT 依赖 realfft/RustFFT，那 dsp 就**不可能** `no_std` | 定为：`vdn-dsp` 是 std 加 `forbid(unsafe_code)`，实时纪律交给 clippy.toml 和 RTSan。如果想要「编译器级禁令」，再拆一个很小的 `vdn-dsp-kernels`（biquad、压缩、门控平滑，`no_std`、不用 alloc）。R5 那句删掉 |
| C5 | **derive / 过程宏** | R3 第 9 条：推荐用 `#[derive(Params)]` 当「AST 用法 1」；R1 #18 不采用；R2 P1 不用；R5 v1 不写；R4 不建议 | 人原话里的「AST」到底落在哪，直接相关 | P1 不写过程宏（四比一，而且参数现在不到 10 个）。R3 第 9 条改成「触发条件：参数超过约 10 个，或者第三次手抄参数表出错」。这跟 CLAUDE.md「同一类错误犯到第三次才沉淀规则」同构 |
| C6 | **speaker 依赖谁** | R1 §5.1：`vdn-speaker` 依赖 `vdn-engine`；R2：只依赖 `vdn-types`，经发布槽接口和 engine 接触 | 声纹线程会不会把 engine 拖进依赖链 | 采用 R2。发布槽（triple_buffer 的类型别名）放在 `vdn-types` |
| C7 | **后端 crate 的名字** | `vdn-infer-ort`（R2）/ `vdn-backend-ort`（R1）/ `infer`（R4）/ `engine-ort`（R6）；另外 R2 的 bin 也叫 `vdn-infer` | 后端 crate 和推理进程 bin 在 R2 里名字撞车（`vdn-infer` 与 `vdn-infer-ort`） | 后端用 `vdn-backend-ort` / `vdn-backend-tract`；两个进程的 bin 用 `vdn-capture` / `vdn-denoised`（或 `vdn-proc`），避开 `infer` 这个词的两种意思 |
| C8 | **clippy.toml 的粒度** | R4：每个 RT crate 目录放自己的 clippy.toml（按 `CARGO_MANIFEST_DIR` 查找）；R1：workspace 一份，再在 `rt` 模块根写 `#![deny]` | 「禁用清单」本身只能按 crate 生效，「严重级别」才能按模块调 | 两者合用：根目录一份宽松清单，RT crate 放一份严格清单（clippy 不会把父目录的清单合并进来，【推】），在 `[lints]` 里提成 deny。只在 bin 里出现的 RT 回调，要放进 RT crate 的一个模块，否则管不到 |
| C9 | **原声旁路由谁做** | R3：`Engine` 内部有 `dry_delay`，按 `L_engine` 对齐；R4：推理进程挂掉时由送音进程/HAL 侧旁路；003 已定：「原声旁路由输出回调做，不由推理线程做」【源】`docs/research/003_mac-audio-pipeline/decision.md:30` | 两层旁路的分工没写清；如果 engine 和 IO 各做一次延迟对齐，可能对齐两次 | 分两层写进 ADR：**模型级**（NaN、单帧超时、换模型）由 engine 做，交叉淡化到 engine 的干声；**进程级**（推理进程崩溃、心跳停）由送音进程的输出回调做，读原声环 C。两层的干声延迟都等于同一个 `L_total`，由 engine 报告，再经 shm 头传给送音进程 |
| C10 | **RTSan 用哪条路** | R4 用 `rtsan-standalone` 0.3（stable 可用，本机已跑通）；R6 写的是 `-Zsanitizer=realtime`（nightly）或 standalone；R1 引了 rustc PR #147935【传】 | CI 要不要装 nightly | P1 只用 `rtsan-standalone`（stable，本机【测】，本轮又在 ort 上用了一次，见 §3.1）。nightly 留到 no_panic 和 public-api 那条线再考虑 |

小矛盾：R2 §4 说 panic 策略「待人定」，R4 已经给了有依据的建议（证明构建用 unwind 加 `#[no_panic]`，发布用 abort，进程隔离兜底）。建议设计稿直接用 R4 的方案，标【推】，交人确认。

---

## 2. 没回答的问题和证据缺口（按六件产出排）

### 2.1 crate 列表与依赖图
- **缺：重采样器放在哪个 crate**（见 §3.7）。003 写的是「推理线程入口先把采样率重采样成模型要的」【源】`003 decision.md:30`。放在 engine 里，eval 就能共用；放在 io-macos 里，eval 读 16 kHz WAV 时就绕过了它，「同一份处理代码」就少了一截。
- **缺：音色链跑在 16 kHz 还是 48 kHz**。EQ、去混响尾、限幅放在上采样前还是后，决定 `vdn-dsp` 的采样率参数和延迟账。
- **缺：`vdn-control` 和推理进程之间的 IPC 介质**（Unix socket？还是 shm 里另开一条命令环？），所以「serde 只在 control 和 bin」这条规则不知道 bin 侧要依赖什么。
- **错**：「wrappers 证明崩溃隔离」见 §3.4。

### 2.2 核心 trait 草案
- **FastEnhancer-M 的接口还是空白**：R3 已经说明 `R/fastenhancer` 不是它的源码，hop、cache 形状、是不是频域模型都没核到。`SpectralModel` 假设「单帧频谱进、单帧频谱出」，如果 FastEnhancer-M 是时域模型或者自带 STFT，trait 就不够用。要先核，或者在 trait 里留一个 `TimeDomainModel` 分支。
- **GTCRN 的 I/O 已经补核**（§3.2）：输入 `mix [1,257,1,2]`（实部、虚部分开放在最后一维），不是复数类型。R3 的 `infer(&[Cplx], &mut [Cplx])` 需要一层「复数 ↔ 交错 f32」的零拷贝视图，或者直接改成 `&[f32]`。
- **缓存的乒乓不能靠「交换绑定」实现**（§3.1）：ort 的 `bind_input`/`bind_output` 每次都会分配（`String` 名字、`MiniMap` 插入）【源】`ort-2.0.0-rc.15/src/session/io_binding.rs:136-200`。R3 §2.3 的「每帧 swap，零拷贝」在 ort 后端上做不到，现实做法是**绑定一次，每帧把 `*_out` 拷回输入**（三组 cache 合计约 1.7 万个 f32，约 68 KB，每帧一次 memcpy）【推】。

### 2.3 实时安全规则（每条配机器检查）
- R4 的 R1–R11 表已经很完整。**缺的是「推理线程」的单独条款**：因为 §3.1 证明它会分配，需要一条 R12「B 级推理调用：允许已知的、在 suppressions 里登记过的分配；验收看 p99.9 和 max，而不是零违规」，配套机器检查是「RTSan 带 suppressions 文件跑，suppressions 之外有新违规就红」，外加一个耗时直方图闸门。
- **缺：RTSan 在 CI 里的成本**。`suppress_equal_stacks=false` 时每帧约 3800 条报告，100 帧就写出 400 MB 日志、跑了 10 分钟以上，我手动停掉了【测】。CI 必须用默认的去重模式（只报唯一栈），加 `symbolize=false`，或者只跑短回放。
- **缺：mutex 的口径**。ORT 每帧 376 对 lock/unlock 来自它自己的 arena，intra=1 时没有竞争者【推】。R2「不加锁」这条是否对推理线程豁免，要写明。

### 2.4 AST 工具采用决定
- R1 §6 总表可以直接用。本轮补了两个待测项：`disallowed-macros` 能按路径禁 `std::println`/`std::format`/`std::vec`；`disallowed-methods` 写 `std::vec::Vec::reserve` 和 `alloc::vec::Vec::push` 两种路径都生效（§3.5）。R1 §7 的待测可以划掉；tympan 注释那个说法在本机 clippy（rustc 1.99.1）上不成立。

### 2.5 命名与目录约定
- 前缀（`vdn-`）、bin 名、后端名还没统一（C7）。`hal-plugin/` 是新顶层目录，按铁律 10 要人拍板（R2 已列）。
- **缺：模型文件的位置和 ID**。R1 借了 Unison 的思路（用 sha256 当 ID），R2 的 `xtask models` 做下载和校验，但没定目录（`models/`？还是仓外？）。本轮给出 GTCRN 三个 onnx 的 sha256（§3.2），可以直接写进清单。

### 2.6 ADR 候选（3–6 条，证据够不够）

| ADR | 证据现状 | 还差 |
|---|---|---|
| A1 workspace 形态：扁平 `crates/`、虚拟 manifest、`vdn-` 前缀、xtask | 够（matklad、rust-analyzer、Zed，R2 和 R5） | 前缀要人拍板 |
| A2 核心与适配分离、两层模型 trait（C1） | 够，但要先定 C1/C2 | FastEnhancer-M 的接口 |
| A3 推理线程的实时承诺分级（A/B/C 三级），ort 允许有界分配 | **本轮补齐**（§3.1 实测） | tract 的对照数据（E2） |
| A4 跨进程旁路两层（C9）加 panic 策略 | R4 证据够 | 心跳超时 N 的取值 |
| A5 机器检查栈：clippy.toml 加 cargo-deny 加 xtask 传递依赖检查加 RTSan 加 assert_no_alloc，不写过程宏 | 够，§3.4 修正了 wrappers 的口径 | — |
| A6 采样率与重采样器的位置（新增） | **缺**，见 §3.7 和 §3.8 | 重采样延迟实测，人定 16 k 和 48 k 的分界 |

---

## 3. 自己补查的缺口（带出处）

### 3.1 【最重要】ort 在稳态每帧是否分配、是否加锁：实测

- **探针**：`S/ortprobe/`（`src/main.rs`、`Cargo.toml`）。ort `=2.0.0-rc.15`，用本机缓存的 ONNX Runtime 静态库，`--offline` 构建。会话配置为 `intra=1, inter=1, intra_op_spinning=false, memory_pattern=true`。模型是 `R/biglinux-gtcrn/stream/onnx_models/gtcrn_simple.onnx`（另跑了 `gtcrn.onnx`），预热 200 帧，再逐帧跑随机输入，每帧把 `*_out` 拷回 cache 输入。三种调法：
  - `plain`：`session.run(inputs![TensorRef…])`，输出用 `try_extract_tensor` 拷出
  - `binding`：`IoBinding` 绑定一次，每帧 `run_binding`
  - `raw`：同一个 binding，每帧直接调 `ort::api().RunWithBinding`，绕过 ort 的 Rust 输出封装
- **Rust 侧分配**（计数型 `GlobalAlloc`，alloc 和 dealloc 都计入）【测】：

  | 调法 | 每帧 alloc+dealloc | p50 | p99 | p99.9 | max（3000 帧） |
  |---|---|---|---|---|---|
  | plain | 54（其中 8 次是探针自己的 shape `Vec`） | 0.766 ms | 0.803 | 0.986 | 1.009 |
  | binding | 28 | 0.748 | 0.776 | 0.804 | 0.918 |
  | raw | ≈0（26 次全是一次性的 eprintln） | 0.748 | 0.864 | 0.942 | 1.009 |
  | raw，`gtcrn.onnx` 未简化版 | ≈0 | 0.760 | 0.790 | 0.834 | 0.902 |

  binding 那 28 次的来源【源】：`run_binding_inner` 在 `RunWithBinding` 之后调 `GetBoundOutputValues`，由 ORT 分配一个指针数组，再 `.collect()` 出新的 `DynValue` 和 `SessionOutputs`（`ort-2.0.0-rc.15/src/session/run.rs:288-320`）。
- **C++ 侧（RTSan，`rtsan-standalone` 0.3，`RTSAN_ENABLE=1`，release）**【测】：只把稳态帧放进 `#[nonblocking]`，`suppress_equal_stacks=false` 逐次计数：
  - **1 个稳态帧（raw）**：malloc 1554、free 1555、`pthread_mutex_lock` 376、`pthread_mutex_unlock` 376
  - **2 个稳态帧**：3109 / 3110 / 752 / 752，与帧数严格成正比，说明不是预热残留
  - 按唯一栈去重后，最多的上层函数依次是 `BFCArena::AllocateRawInternal`、`InsertFreeChunkIntoBin`、`Tensor::InitOrtValue`、`BFCArena::FindChunkPtr/Free/Coalesce`、`MemPatternPlanner::TraceAllocation`、`MlasGemmBatch`（含 KleidiAI 路径）、`InferenceSession::RunImpl`、`ExecutionFrame::~ExecutionFrame`（日志 `S/ortprobe/rtsan-raw.log`）
- **结论**：
  1. ort 的 IoBinding **不能**让推理零分配。ORT 内核在 arena、执行帧、形状对象上都会分配，并在 arena 上加锁【测】。R3 §3.4 和 R4 §5.3 的【推】、R6 §5 的「未找到出处」都可以改成【测】。
  2. 要把 Rust 侧做到零分配，得**绑定一次，然后用 raw `RunWithBinding`**，自己读预绑定的输出张量（实测输入张量原地改写后不重新绑定也会生效：`[check] enh changes w/o rebind: true`【测】，但这只是弱证据，只比较了一帧的输出和）。这需要 `unsafe`，正好落在后端 crate 的 unsafe 白名单里。
  3. 耗时余量很大：最坏一帧约 1 ms，hop 16 ms，RTF ≈ 0.06【测】，满足 002 的「最坏一帧 ≤ 2 ms」【源】`docs/research/002_model-baseline/must-know.md:17`。R3 的「热切换双跑需要 RTF ≤ 0.4」在 M5 上没问题。注意：这是空载、未提权、单次运行的结果，E 核和负载下要重测（R4 E5）。
  4. **还没试的调参**：关掉 CPU arena（`with_memory_pattern(false)` 或 arena 配置项）、`session.use_env_allocators`、自定义分配器（`with_allocator`）。它们可能把 malloc 换成别的形式，但从栈上看，ExecutionFrame 和 Tensor 对象的分配跟 arena 无关【推】，不太可能归零。
- 复现：`cd S/ortprobe && cargo build --release --offline && ./target/release/ortprobe <onnx> raw 3000`；RTSan 版本加 `RTSAN_ENABLE=1 CARGO_TARGET_DIR=target-rtsan` 构建，经 `cargo run` 运行（原因见 R4 §2.2，没有 LC_RPATH）。

### 3.2 GTCRN 用到哪些 ONNX 算子，tract 是否支持

- **算子清单**【测】：我用只依赖标准库的 protobuf 解析脚本 `S/tools/onnxops.py` 读 `gtcrn.onnx` 和 `gtcrn_simple.onnx`。opset 都是 ai.onnx 11，没有自定义域。`gtcrn_simple` 用了 24 种算子：Add、BatchNormalization、Concat、Conv、ConvTranspose、Div、**GRU**（14 个）、Gather、MatMul、Mul、PRelu、Pad、Pow、ReduceMean、Reshape、**ScatterND**（18 个）、Sigmoid、Slice、Sqrt、Squeeze、Sub、Tanh、Transpose、Unsqueeze。未简化版另外还有 Cast、Constant、ConstantOfShape、Equal、Expand、Range、Shape、Where（动态形状的残留）。**应当用 `_simple` 版**。
- I/O 名【测】：输入 `mix, conv_cache, tra_cache, inter_cache`，输出 `enh, conv_cache_out, tra_cache_out, inter_cache_out`，与 R3 引的 `gtcrn_stream.py:414-415` 一致。`mix` 的形状是 `[1,257,1,2]`【源】`R/biglinux-gtcrn/stream/gtcrn_stream.py:405`。
- **tract-onnx 的注册情况**【源】（本机 tract 克隆是 0.23.10-pre，`R/tract/onnx/Cargo.toml:3`）：上面 32 种算子在 onnx 前端**全部有** `reg.insert`，例如 `GRU` 在 `R/tract/onnx/src/ops/rec.rs:10`，`ScatterND` 在 `R/tract/onnx/src/ops/array/mod.rs:42`，`PRelu` 在 `R/tract/onnx/src/ops/nn/mod.rs:81`，`ConvTranspose` 在 `R/tract/onnx/src/ops/nn/mod.rs:61`。
- **没做到的**：「有注册」不等于「能跑对」，GRU 的 `linear_before_reset`、ScatterND 的语义、ConvTranspose 的 padding 都可能出问题。我写好了探针 `S/tractprobe/`（加载、`into_optimized`、逐帧跑、计数分配和耗时），但**构建本机外部克隆的 tract 被权限分类器拦下**（理由是 Code from External），没有绕开。**tract 能否跑 GTCRN 仍是【推】**，是 R4 E2 的头号任务。也可以改用 crates.io 上已发布的 `tract-onnx 0.23.9`（2026-10-15，见下文 §3.3 的版本表），这需要人放行联网下载 crate。
- sha256（写进模型清单用）【测】：`gtcrn_simple.onnx` b4718df6…ebcb87；`gtcrn.onnx` f648b02f…8cad17；`gtcrn_vctk.onnx` 9dce5e90…90d68b（全长见 `shasum -a 256`）。

### 3.3 依赖版本现实：本机克隆和已发布版对不上

crates.io API 查询结果（2026-10-30）【官】`https://crates.io/api/v1/crates/<name>`：

| crate | 已发布的最新版 | 本机克隆 | 影响 |
|---|---|---|---|
| cpal | **0.18.2**（2026-08-16） | `R/cpal` 是 **0.19.0 未发布 master**（`Cargo.toml:3`，commit 4bedd4e） | R4/R2 引的 `device.rs:621-657` 等行号是 master 上的，在 0.18.2 里可能对不上。0.18.2 的回调是 `FnMut(&[T], &InputCallbackInfo)`、错误类型是 `cpal::Error`【官】https://docs.rs/cpal/0.18.2/cpal/traits/trait.DeviceTrait.html ；master 已经把它合并成 `CallbackInfo`，并把 `play` 改名为 `start`【源】`R/cpal/CHANGELOG.md`（Unreleased 段） |
| rubato | 5.0.1（2026-10-01） | 一致 | 5.x 有 `Slip`，以及 `Adjustable`/`Resizable` 拆分【源】`R/rubato/README.md:521-592` |
| ort | **没有稳定版**，最新 2.0.0-rc.15 | registry 里是 rc.15 | 要用 `=2.0.0-rc.15` 精确锁定。rc 之间 API 常变（例如 `Session::builder(&env)` 要传环境，`run` 要 `&mut self`）【源】`ort-2.0.0-rc.15/src/session/mod.rs:159`、`run.rs:48` |
| tract-onnx | 0.23.9 | 0.23.10-pre | 用已发布版 |
| rtrb | 0.4.1；triple_buffer 9.0.0；basedrop 0.1.3；audio_thread_priority 0.38.0；rtsan-standalone 0.3.0 | — | basedrop 很久没更新（rusteco 的范围） |

另外：ort 默认的 `download-binaries` 会从 pyke 的 CDN 下载 ONNX Runtime 预编译库，本机缓存在 `~/Library/Caches/ort.pyke.io/dfbin/aarch64-apple-darwin/<hash>/libonnxruntime.a`【测】。R2 §9 记为【传】的「ort 构建会下载预编译库」现在是【测】。CI 要缓存这个目录，否则每次都要联网。

### 3.4 cargo-deny `wrappers` 的真实语义

- 源码：只有当被禁 crate 的**每一个直接依赖者**都在 `wrappers` 里，才算放行（`ctx.krates.direct_dependents(nid)` 逐个比对 crate 名）【源】`R/cargo-deny/src/bans.rs:616-650`。按 crate 名匹配，所以 workspace 的 path 依赖也适用（R1 §7 那条待测到此有了答案）【源+推】。
- **推论**：
  1. 「只有 `vdn-backend-ort` 能直接用 `ort`」可以证明。
  2. 「`vdn-capture` 的依赖树里没有 ort 和 engine」**证明不了**（R2 §2.4 和 §5 的说法要改）。要用 xtask 的 `check-deps`：读 `cargo metadata`，对每个 bin 求传递闭包，再与禁表求交集。等价的人工命令是 `cargo tree -p vdn-capture -i ort`，输出为空才算通过。
  3. 给 `serde` 加 wrappers 风险很高：只要任何第三方依赖直接依赖 serde（例如 `toml`，或者某个带 serde feature 的库），就会触发违规。建议 serde 的规则也交给 check-deps 管，范围限定为「核心 crate 的闭包里不出现 serde」。

### 3.5 clippy 的路径写法（R1 和 R4 的待测项）

- 探针 `S/clippyprobe/`，本机 clippy（rustc 1.99.1），把 `[lints.clippy]` 的 `disallowed_*` 设为 deny【测】：
  - `disallowed-macros`：`std::println`、`std::format`、`std::vec` **都能命中**
  - `disallowed-methods`：`alloc::vec::Vec::push` 能命中，`std::vec::Vec::reserve` **也能命中**（R4 说只有定义路径生效，不准确）；`alloc::boxed::Box::new` 能命中
  - `disallowed-types`：`std::sync::Mutex` 能命中，类型注解和构造各报一次
- 局限不变：只检查本 crate 的调用点，跨 crate 和宏展开内部的调用看不到（R4 §2.4）。

### 3.6 Audio Workgroup 的 Rust 绑定

- `objc2-core-audio` 0.3.2 里有常量 `kAudioDevicePropertyIOThreadOSWorkgroup`，可以取设备的 workgroup【官】https://docs.rs/objc2-core-audio/0.3.2/objc2_core_audio/index.html （页面只读到前 10 万字符）。
- `objc2-audio-toolbox` 0.3.2 的 `AudioWorkIntervalCreate` 文档页返回 404【官：未载】。`dispatch2` 0.3.1 里**没有** os_workgroup 相关项【官】https://docs.rs/dispatch2/0.3.1/dispatch2/index.html 。搜索也没找到现成 crate（https://developer.apple.com/documentation/audiotoolbox/audioworkintervalcreate 只有 C API）。
- 结论：`os_workgroup_join/leave/interval_start/finish` 和 `AudioWorkIntervalCreate` 要在 `vdn-io-macos`（或推理进程的平台模块）里手写 `extern "C"`。003 估计「约 60 行 FFI」【源】`003 decision.md:30`。**注意**：推理线程在推理进程里，不在 io-macos 里，所以「只有 io-macos 碰 Mach/CoreAudio」这条规则（R2、R4 R8）要么放宽到「推理进程 bin 的 `rt_thread` 模块」，要么把提权和 workgroup 抽成一个小 crate `vdn-rt-thread`，两个进程共用。

### 3.7 重采样：放哪、用哪种、延迟多少

- rubato 官方：`process_into_buffer` 不分配、不阻塞，日志 feature 不要在实时场景打开【官】`R/rubato/README.md:198-206`。
- 延迟公式【源】：`Fft`（同步）的延迟是 `fft_size_out / 2`（`R/rubato/src/synchro.rs:692-694`），默认把 sub-chunk 定在 256 帧左右以压低延迟（`synchro.rs:221-222`）；`Async` sinc 的延迟是 `nbr_points × ratio / 2`（`asynchro.rs:747-749`）；`Slip` 的延迟是 0（`slip.rs:481-483`），只适合几个 ppm 的漂移【官】`README.md:108-112`。
- 本项目的链路【推】：麦克风 48 kHz →（漂移补偿）→ 48→16 kHz → GTCRN → 16→48 kHz → BlackHole 48 kHz。**漂移补偿用 `Slip`（零延迟）**；固定比例转换用 `Fft`，每次延迟约几毫秒，要按 chunk 大小实测。**这两次固定比例重采样放在 engine 里**，eval 读 48 kHz WAV，才能覆盖「同一份代码」；`Slip` 放在 io-macos（它只在实时侧存在，eval 不需要）。这一条写成 ADR A6，交人确认。

### 3.8 延迟账（六篇都没算）

| 环节 | 量 | 依据 |
|---|---|---|
| GTCRN STFT 512/256 的 WOLA 算法延迟 | 512 样本 @16 kHz = **32 ms** | 窗 512、hop 256【源】`gtcrn_stream.py:371`；第 t 帧覆盖 [tH, tH+512)，样本 tH 要等到 tH+512 才完整输出【推】 |
| 管线直通（IO 缓冲、safety offset、环形缓冲） | 6–10 ms | 002 口径【源】`docs/research/002_model-baseline/must-know.md:17` |
| 48→16、16→48 两次 `Fft` 重采样 | 每次约 `fft_size_out/2`，合计估 2–6 ms | §3.7【源+推，待测】 |
| 推理耗时（输出要等算完） | ≤ 1 ms | §3.1【测】 |
| BlackHole 自身延迟 | 0（`kLatency_Frame_Size` 默认 0） | `R/BlackHole/BlackHole/BlackHole.c:235-236`【源】 |
| **合计** | **约 41–49 ms**，上限 45 ms | 【推】 |

→ **处在超限边缘**。R3 已经要求启动时把 `latency_samples` 逐级求和并做断言，这里再补两件事：① 第一个实验就做脉冲测试，测出真实的端到端延迟；② 如果超限，可选的手段有：降低 IO 缓冲、选更短的重采样器、把音色链的 lookahead 设为 0。这个取舍要人定（D01 已经把上限定为 45 ms）。

---

## 4. 实现障碍清单：明天照着写代码会卡在哪

| # | 卡点 | 现状 / 证据 | 绕法【推】 |
|---|---|---|---|
| B1 | **ort 在推理线程里每帧分配约 1550 次、加锁 376 次** | §3.1【测】 | 承诺分级（A3）；raw `RunWithBinding` 让 Rust 侧零分配；RTSan 带 suppressions；看 p99.9 和 max。长期方案：tract 对照（E2），或者自写 GTCRN 推理（模型只有约 0.35 MB） |
| B2 | ort 的 `run_binding` 返回 `SessionOutputs`，每帧都会分配；`bind_*` 每次也分配 | `run.rs:288-320`、`io_binding.rs:136-200`【源】 | 只在构建期绑定；每帧调 `ort::api().RunWithBinding`（需要 unsafe）；cache 用拷回而不是交换绑定 |
| B3 | 只能 `bind_output` 用 `Tensor::new` 建的张量，不能绑定借来的缓冲 | 「only tensors created with Tensor::new can be bound to outputs」`io_binding.rs:183-188`【源】 | 输出缓冲由 ORT 分配器在构建期分配；每帧通过 `extract_tensor` 读取（是否零分配要再测一次，本轮在计时窗口外调用） |
| B4 | tract 跑 GTCRN 没有验证 | 算子全部有注册【源】；构建被拦【—】 | 用已发布的 `tract-onnx 0.23.9` 做 E2；`set_inputs/exec/outputs` 这组 API（`R/tract/core/src/plan.rs:611,794,904`）也许能减少每帧分配，【推】 |
| B5 | cpal 的版本和 API | 0.18.2 和 master 0.19 的回调类型名不同（§3.3） | 锁定 `cpal = "=0.18.2"`；io-macos 里的代码按 docs.rs 0.18.2 写；R4 引的行号要按 0.18.2 重核（rusteco） |
| B6 | cpal 的 `BufferSize::Fixed` 设的是**设备级**属性，还会校验硬件范围 | `R/cpal/src/host/coreaudio/macos/device.rs:903-930`【源】（master） | 启动时读回实际的 buffer 大小写进延迟账；回调帧数可能和请求的不同，A 级回调要按任意长度写 |
| B7 | GTCRN 的 hop 是 16 ms，003 的环形缓冲按 10 ms 台阶设计 | `gtcrn_stream.py:371`【源】；`003 decision.md:30`【源】 | 环 A/B 按样本计，不按「帧」计；engine 内部攒 hop（C2） |
| B8 | 没有 workgroup 绑定；推理线程又不在 io-macos 里 | §3.6 | 新增 `vdn-rt-thread` 小 crate（提权、workgroup、FTZ guard），两个进程共用；unsafe 白名单加入它 |
| B9 | 共享内存里用不了 rtrb | R4 §2.7【源】 | `vdn-shm` 自写 `#[repr(C)]` SPSC，loom 和 Miri 只盯它 |
| B10 | cargo-deny 证明不了 bin 级的依赖隔离 | §3.4【源】 | xtask check-deps 按 bin 求传递闭包 |
| B11 | RTSan 的逐次模式日志爆量（100 帧 400 MB 以上） | §2.3【测】 | CI 用默认去重模式加 `symbolize=false`，回放要短 |
| B12 | FastEnhancer-M 接口未知 | R3【源：缺】 | trait 留时域分支；先只接 GTCRN |
| B13 | 数值一致性：FTZ 会改变结果；ORT 的 KleidiAI/MLAS 路径可能跟随 CPU 特性变化 | R4 §3【测】；§3.1 栈里出现 `ArmKleidiAI::MlasGemmBatch`【测】 | eval 和实时共用 guard；golden 对拍对模型输出用容差，不要求逐位一致（R6 第 3 条） |
| B14 | 两个 bin 加 workspace 级构建，feature 会统一 | Cargo resolver 的行为【官，Cargo Book「Feature unification」】 | 测试用的全局分配器只放在 test/bin 里；`assert_no_alloc` 不进任何 lib 的默认 feature |

---

## 5. 结论：六件产出现在能不能写

| 产出 | 能写吗 | 前提 |
|---|---|---|
| crate 列表与依赖图 | 能写 v0 | 先定 C1、C3、C6、C7，并新增 `vdn-rt-thread`（B8）；重采样器放在 engine（A6，待人定） |
| 核心 trait 草案 | 能写 v0 | 采用 C1 的两层方案、C2 的统一入口；`mix` 用交错 f32；cache 用拷回 |
| 实时安全规则加机器检查 | 能写 | R4 的表，加上 R12（推理线程有界分配）；wrappers 的口径按 §3.4 修正 |
| AST 工具采用决定 | 能写 | R1 §6 的表，划掉 §3.5 已经测过的待测项；C5 定为「不写过程宏」 |
| 命名与目录约定 | 只差人拍板 | 前缀、bin 名、`hal-plugin/`、模型目录 |
| ADR | A1、A3、A4、A5 能写；A2 要先定 C1；A6 缺实测 | — |

## 6. 建议下一个有界实验（按价值排）

1. **E2'**：用已发布的 `tract-onnx 0.23.9` 跑 `gtcrn_simple.onnx`，记录每帧分配、p99.9，并和 ort 的 `enh` 输出对拍（最大绝对误差）。需要放行联网下载 crate。
2. **E-lat**：脉冲测试测端到端延迟（STFT、两次 `Fft` 重采样、BlackHole），对照 §3.8 的账。
3. **E1'**：ort 关掉 arena 或换自定义分配器之后的 RTSan 计数（§3.1 第 4 点）。
4. 在负载和 E 核条件下重测 §3.1 的 p99.9（R4 E5）。

---

### 本轮出处补充
- 本机探针：`S/ortprobe/`（含 `rtsan-*.log`）、`S/clippyprobe/`、`S/tools/onnxops.py`、`S/tractprobe/`（写好了，没有构建）
- crates.io API：https://crates.io/api/v1/crates/cpal （其余同理）
- cpal 0.18.2 文档：https://docs.rs/cpal/0.18.2/cpal/traits/trait.DeviceTrait.html
- objc2-core-audio 0.3.2：https://docs.rs/objc2-core-audio/0.3.2/objc2_core_audio/index.html
- dispatch2 0.3.1：https://docs.rs/dispatch2/0.3.1/dispatch2/index.html
- Apple AudioWorkIntervalCreate：https://developer.apple.com/documentation/audiotoolbox/audioworkintervalcreate
