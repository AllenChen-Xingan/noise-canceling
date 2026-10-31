# R4 · Rust 实时音频惯例与「用工具证明」

> 主题 4 调研笔记（research/008-rust-architecture）。写于 2026-10-30，本机 Apple M5 / macOS (Darwin 27) / rustc 1.99.1 / Apple clang 21。
> 证据等级：【官】官方明载 ·【源】源码可证 ·【测】本机实测 ·【传】二手 / 论坛 / 未能直接打开原文 ·【推】推断。
> 本机资源根目录简写 `RES=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources`。
> 本机实验探针（一次性、在 scratchpad，未进仓）：`…/scratchpad/rtprobe`（FTZ / assert_no_alloc / extern "C" panic / clippy）、`…/scratchpad/rtsanprobe`（RTSan）、`…/scratchpad/nopanicprobe`（no-panic）。

---

## 0. 一页结论

1. **实时线程要分级，而不是一刀切**【推】。本项目有三种「实时」：
   - **A 级·硬实时回调**：送音进程的 CoreAudio IO 回调（cpal 回调里）、以后 coreaudiod 里的 HAL 插件 IO。只做「拷贝 + 旁路判断 + 原子计数」，任何超时都直接爆音。
   - **B 级·准实时处理线程**：推理进程里的实时降噪线程（自建线程，每 10–16 ms 一帧，前后有环形缓冲吸收抖动）。要 time-constraint 优先级 + 自建 Audio Workgroup；要求不分配、不加锁，但 ONNX 推理这一步**先当作「被测对象」而不是「已证安全」**。
   - **C 级·非实时**：声纹线程（0.5 s）、控制、日志、回收（basedrop collector）、eval。
2. **「用工具证明」至少要三层叠起来，单一工具都有盲区**：
   - 编译期（AST/lint 层）：`#![forbid(unsafe_code)]` + 每个 RT crate 自己的 `clippy.toml`（`disallowed-methods/types/macros`）+ `clippy::unwrap_used/expect_used/panic/indexing_slicing` 等 deny。【官】【测】
   - 链接期：`#[no_panic]`（dtolnay/no-panic），只对 release/优化构建可信。【官】【测】
   - 运行期：`assert_no_alloc`（只挡 **Rust** 分配，默认 release 关闭）+ **RTSan（rtsan-standalone，macOS arm64 本机已跑通）**——后者在 libc 拦截层工作，能挡 `malloc/calloc`、`pthread_mutex_lock/unlock`、`nanosleep` 等，**也就能看到 ONNX Runtime 这种 C++ 库里的 malloc**。【测】
3. **最大的单点风险是 ONNX 推理**：ort `Session::run` 默认会分配输出张量；要用 `IoBinding` 预绑定输出、`with_intra_threads(1)`、关闭 spinning、`with_flush_to_zero`，然后**用 RTSan 实测**是否还有分配。tract 在 DeepFilterNet libDF 里每帧 `clone()` + `tvec!`，即每帧分配。【官】【源】
4. **FTZ 在 aarch64 上靠 inline asm 写 FPCR bit 24**；本机实测 M5 上 denormal **没有可测的性能惩罚**，但开 FTZ **会改变数值结果**——所以 eval 与实时必须用同一个「进入处理域」的 guard，否则「同一份代码」也会给出不同输出。【测】
5. **panic 策略**：`no_panic` 要求 `panic = "unwind"`，所以「CI 证明用 unwind 构建，发布可用 abort」；进程级兜底靠「推理进程挂了 → 送音/HAL 侧看心跳超时 → 原声旁路」。Rust ≥1.81 里 panic 越过 `extern "C"` 会直接 abort（本机实测 exit 134），但 coreaudio-rs 的输入回调声明为 `extern "C-unwind"`（render_callback.rs:766），panic 会尝试穿过 CoreAudio 的 C 帧——**所以回调最外层必须自己兜住**。【官】【源】【测】
6. **人的「AST / 模块化」建议在 Rust 里落点很具体**：lint（clippy/dylint 在 AST/HIR 上检查）+ proc-macro 属性（`#[nonblocking]`、`#[no_panic]` 本身就是 syn 改写语法树）+ **crate 边界当作规则边界**（clippy.toml 按 crate 目录查找，所以「实时代码独立成 crate」就是让机器规则能精确生效的前提）。【官】【推】

---

## 1. 实时规则清单（每条配机器检查）

| # | 规则 | 依据 | 机器检查（至少一种） | 适用 crate |
|---|---|---|---|---|
| R1 | RT 路径不分配 / 不释放 | Bencina 2011【传】；basedrop 文【官】；rtrb 文档「构造后不再分配」`RES/rtrb/src/lib.rs:7-9`【源】 | ① `assert_no_alloc` 包住 `process()`（debug/test）【测】② RTSan `#[nonblocking]`（含 C/C++ 库的 malloc）【测】③ clippy `disallowed-methods`: `Vec::push/with_capacity/reserve`、`disallowed-macros`: `vec!`/`format!`【测】④ 发布版计数分配器 + 指标上报 | dsp, engine, io-macos 的回调模块 |
| R2 | RT 路径不加阻塞锁 | Doumler 2020「try_lock 也不行，unlock 可能要系统调用」【官】 | ① clippy `disallowed-types: std::sync::Mutex, RwLock, Condvar`【测】② RTSan 捕获 `pthread_mutex_lock/unlock`（本机实测）③ 本机实测：**std::sync::Mutex 在 macOS 首次 lock 时还会 malloc**（RTSan 报 `malloc` → `pthread_mutex_lock` → `pthread_mutex_unlock` 三条） | 同上 |
| R3 | RT 路径不做系统调用 / IO / 日志 / sleep | Bencina【传】；RTSan 文档「anything else known to have a non-deterministic execution time」【官】 | ① RTSan（本机实测 `nanosleep` 被拦）② clippy `disallowed-macros: println!/eprintln!/log::*`，`disallowed-methods: std::thread::sleep, std::fs::*` ③ 日志只走 rtrb → 日志线程 | 全 RT crate |
| R4 | RT 路径不 panic | no-panic README【官】；Rust 1.81 extern "C" abort【官/传】 | ① `#[no_panic]` 标在 `Engine::process`、DSP kernel 上（release 构建链接期报错，本机实测）② clippy deny `unwrap_used/expect_used/panic/indexing_slicing/unreachable/todo`【官（lint 名）/推（组合）】③ 回调最外层 `catch_unwind` 兜底（仅 unwind 构建有意义） | dsp, engine |
| R5 | 无界时间算法禁止（摊还 O(1) ≠ 最坏 O(1)） | Bencina【传】；basedrop 文「there are no second chances」【官】 | ① 代码评审判据（specs 负例）② 基准里看 **p99.9/max** 而非均值（criterion/自写 histogram）③ RTSan 能挡到其中调系统库的部分 | dsp, engine |
| R6 | 跨线程只用 wait-free SPSC / 原子 / 不可变发布 | Doumler 2020【官】；rtrb `lib.rs:9` wait-free，head/tail `CachePadded` `lib.rs:104,109`，push 返回 `Result` 不 panic `lib.rs:346`【源】 | ① clippy 禁 `std::sync::mpsc`、`crossbeam_channel` 的阻塞接口 ② 自写 lock-free 结构用 **loom**（`https://github.com/tokio-rs/loom`）做交错穷举、**miri** 查 UB（本机已装 cargo-miri）【推】 | engine, shm |
| R7 | RT 线程不释放大对象（drop 推迟到非 RT 线程） | basedrop：`Owned/Shared` drop 时进 MPSC 队列，`Collector` 在别处回收【官】；rtrb：最后一个 Producer/Consumer drop 会释放缓冲 `lib.rs:241-258`【源】 | ① assert_no_alloc 也挡 `dealloc`（`assert_no_alloc-1.1.2/src/lib.rs:170,175` 同时实现 alloc/dealloc；realloc/alloc_zeroed 走 GlobalAlloc 默认实现回落到这两者）【源】② RTSan 挡 `free` | engine（模型热切换、参数块替换） |
| R8 | RT 线程要有 time-constraint 优先级 + 加入 Audio Workgroup | WWDC20「All audio threads should be joined to workgroups」【官】；audio_thread_priority 的 Mach 实现 `RES/audio_thread_priority/src/rt_mach.rs:113-141`【源】 | ① 启动自检：`thread_policy_get` 读回 `THREAD_TIME_CONSTRAINT_POLICY`（同文件 :91-106 的做法）② `os_workgroup_join` 返回码检查（0/EALREADY/EINVAL）【官】③ 运行期统计 deadline miss | io-macos（只有它碰 Mach/CoreAudio） |
| R9 | FTZ/DAZ 统一由「处理域 guard」设置 | Firewheel / truce-core 做法【源（docs.rs 源码）】；ARM FPCR.FZ = bit 24【官】 | ① 单测：guard 内 `1e-45*0.5` 为 0、guard 外不为 0（本机实测）② debug_assert 在 `process()` 入口读 FPCR | dsp（guard 定义）、engine（调用） |
| R10 | unsafe 只在少数 crate；其余 `#![forbid(unsafe_code)]` | rtrb 自己用 `deny(unsafe_op_in_unsafe_fn)` + `warn(clippy::undocumented_unsafe_blocks)` `RES/rtrb/src/lib.rs:65-66`【源】 | ① `[workspace.lints] rust.unsafe_code = "forbid"`，FFI crate 局部覆盖 ② `clippy::undocumented_unsafe_blocks = deny` ③ CI 跑 cargo-geiger 或 grep 统计 unsafe 只出现在白名单 crate【推】 | 全 workspace |
| R11 | 实时路径错误 = 错误码 + 原声旁路，绝不传播异常/阻塞上报 | Doumler 2020「try_lock 失败就用 last known value / bypass / fade-out」【官】；cpal 用 `try_lock` 发错误回调 `RES/cpal/src/host/error_emit.rs:25-37`【源】 | ① `process()` 返回 `#[must_use] enum Status`（clippy `must_use_candidate`）② 故障注入测试：模拟推理超时/NaN → 断言输出 == 原声（带交叉淡化）③ NaN/Inf 检测在 RT 内用 `is_finite` 原子计数 | engine, io-macos |

---

## 2. 工具逐个拆

### 2.1 assert_no_alloc

- **怎么接**：在 **binary / test 的根**放 `#[global_allocator] static A: AllocDisabler = AllocDisabler;`，然后 `assert_no_alloc(|| engine.process(..))`。文档建议配 `#[cfg(debug_assertions)]`。【官】https://docs.rs/assert_no_alloc/latest/assert_no_alloc/
- **挡什么**：`alloc` 与 `dealloc`（源码 `~/.cargo/registry/src/*/assert_no_alloc-1.1.2/src/lib.rs:170,175`），用 thread_local 计数器 `ALLOC_FORBID_COUNT`（同文件 :35-40）区分线程——**只影响调用它的线程**。【源】
- **不挡什么**：锁、系统调用、IO；以及 **C/C++ 库直接调 libc malloc**——`#[global_allocator]` 只接管经 `std::alloc` 的分配（GlobalAlloc 文档：通过 `alloc/alloc_zeroed/dealloc/realloc` 调用）【官】https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html ；FFI 的 malloc 不经过它【推，机制明确】。**→ 对 ort/onnxruntime 无效**，这正是要 RTSan 的原因。
- **发布版**：默认 feature `disable_release` → release 下什么都不做；要在 release 报警需 `default-features = false` + `warn_release`。【官】
- **本机实测**【测】：`rtprobe alloc`（RT 闭包里 `Vec::push` 到容量 0 的 Vec）——debug 构建 `memory allocation of 16 bytes failed` + abort（exit 134）；release 构建正常通过（`survived, len=1`）。即：**release 下它不是证明**。
- `permit_alloc(|| ..)` / `PermitDrop`：测试桩里显式放行。【官】
- 小坑【推】：abort 模式下报错信息自身可能触发再分配（实测输出出现「skipping backtrace printing to avoid potential recursion」）。

### 2.2 RTSan（LLVM RealtimeSanitizer）与 rtsan-standalone

- **来历**：Clang 20 发布说明「Introduced Realtime Sanitizer, activated by using the -fsanitize=realtime flag … detects unsafe system library calls, such as memory allocations and mutex locks」。【官】https://releases.llvm.org/20.1.0/tools/clang/docs/ReleaseNotes.html
- **语义**：被 `[[clang::nonblocking]]` 标记的函数**及其调用的一切**都算实时上下文；`[[clang::blocking]]` 标自定义阻塞函数；`RTSAN_OPTIONS=halt_on_error=false`、`suppressions=`（`function-name-matches`/`call-stack-contains`）；ScopedDisabler。「The runtime slowdown … is negligible」。【官】https://clang.llvm.org/docs/RealtimeSanitizer.html
- ADC24 有 Josip Ćavar 与 Christopher Apple 讲 LLVM 实时安全工具链（RTSan + Performance Constraints），之后社区做出 Rust/Swift 工具。【传】（来源为搜索摘要，未打开演讲页）
- **Apple clang 不带**：本机 `clang -fsanitize=realtime` → `unsupported option '-fsanitize=realtime' for target 'arm64-apple-darwin27.0.0'`（Apple clang 21.0.0）。【测】→ C 写的 HAL 插件要用 Homebrew/上游 LLVM ≥20 编译才能上 RTSan。
- **Rust：rtsan-standalone 0.3.0**（`https://github.com/realtime-sanitizer/rtsan-standalone-rs`，作者 Stephan Eckes）。宏：`#[nonblocking]`、`#[blocking]`、`no_sanitize_realtime`、`scoped_disabler`；默认 feature `prebuilt-libs` 用 curl 下载 `rtsan-libs`；`RTSAN_LIBRARY_PATH` 用自编库。README：「Currently, this sanitizer only works on Linux, macOS and iOS.」【官】
  - **关键细节**：`RTSAN_ENABLE` 是**编译期**环境变量（build.rs 读，`rtsan-standalone-sys-0.3.0/build.rs:9`），不设就是空操作——本机先不设编译，再设运行，结果全部「ok」不报错。【源】【测】
  - macOS 上链接的是 `@rpath/libclang_rt.rtsan_osx_dynamic.dylib`（build.rs:21-24,200），二进制**无 LC_RPATH**，直接跑 `./target/debug/x` 会 dyld 报错 abort；用 `cargo run`/`cargo test`（cargo 会设置库搜索路径）即可。【测】→ CI 一律走 `RTSAN_ENABLE=1 cargo test`。
  - **本机实测（Apple M5，aarch64-apple-darwin）**【测】：

    | 用例（在 `#[nonblocking]` 内） | 结果 |
    |---|---|
    | 纯乘法循环 | ok |
    | `AtomicU32::fetch_add` | ok（原子不算违规） |
    | `vec![0.0; 256]` | `Intercepted call to real-time unsafe function 'calloc'` |
    | `std::sync::Mutex::lock` | 依次 `malloc`（macOS std Mutex 首次 lock 懒分配 pthread mutex）→ `pthread_mutex_lock` → `pthread_mutex_unlock` |
    | `std::thread::sleep` | `nanosleep` |
    | release 构建 + 加锁 | 同样报 `malloc`——**release 下仍然有效**（与 assert_no_alloc 相反） |

  - 它工作在 libc 拦截层，所以 ort（C++ onnxruntime dylib）里的 `malloc`/`pthread_*` 在 `#[nonblocking]` 动态作用域内**理论上同样会被拦**【推，依据：拦截发生在 libc 符号层，见上表栈帧 `libclang_rt.rtsan_osx_dynamic.dylib`】。待实验 E1 证实。
  - 「系统调用」覆盖范围：官方未给完整清单【官：未列】；本机至少证实 `nanosleep`。`mmap`、文件 IO、`semaphore_wait` 等是否被拦需逐项实测【未找到出处】。
- **局限**【推】：动态检测，只覆盖跑到的路径——需要把「模型切换、参数更新、旁路触发、xrun 恢复」等分支都在测试里跑一遍；挡不住「没调系统库但很慢」的代码（O(n²)、denormal、cache miss），这部分靠基准 p99.9。

### 2.3 dtolnay/no-panic

- 机制：属性宏把函数体包起来，若优化后仍有 panic 路径则引用一个不存在的符号 → **链接错误** `ERROR[no-panic]: detected panic in function ...`。【官】https://github.com/dtolnay/no-panic
- 限制（README）：要优化才可信（debug 可能误报，建议 dev profile `opt-level=1` 或只在 release 启用）；`cargo check` / 只编 lib 不链接时**不触发**；**`panic = "abort"` 下无效**；不支持 `const fn`；跨 crate 调用可能要 thin/fat LTO + `codegen-units=1`。【官】
- **本机实测**【测】（`nopanicprobe`）：
  - `for (x,y) in buf.iter_mut().zip(g)` 版：release 通过；**debug 误报** `detected panic in function gain_ok`。
  - `for i in 0..buf.len() { buf[i] *= g[i] }` 版：当调用方传入**编译期已知等长**的 Vec 时，release **居然通过**（LLVM 跨过程常量传播把边界检查消掉了，即使 `#[inline(never)]`）；改成运行期长度后才报 `detected panic in function gain_bad`。
  - 教训：**no_panic 证明的是「这次构建、这些调用方」下无 panic**，不是函数本身的性质 → 要在**真实调用图**（engine crate 的最终产物 / 专门的证明 bin，用运行期参数）上做，而不是在 dsp 单测里做。
- 与 clippy `indexing_slicing` 互补：clippy 在源码层禁索引（保守、稳定），no_panic 在产物层兜底（精确、易碎）。【推】

### 2.4 clippy：disallowed_* 与 crate 级配置

- 字段：`path`（必填）、`reason`、`replacement`、`allow-invalid`。【官】https://doc.rust-lang.org/stable/clippy/lint_configuration.html
- **查找规则**：`CLIPPY_CONF_DIR` → `CARGO_MANIFEST_DIR` → 当前目录，找不到就往上逐级找。【官】https://doc.rust-lang.org/stable/clippy/configuration.html
  → **每个 crate 目录可以放自己的 clippy.toml**：RT crate（dsp、engine-rt、io-macos 的回调模块所在 crate）放严格版；control/eval 用根目录宽松版。这就是「模块边界 = 规则边界」。【推】
- 本机实测：`disallowed-methods = [{ path = "alloc::vec::Vec::push", reason = ... }]` → `warning: use of a disallowed method alloc::vec::Vec::push … note: RT path: no growth`（默认 warn，需在 `[lints]` 提到 deny）。【测】注意要写**定义路径** `alloc::vec::Vec::push`（不是 `std::vec::Vec::push`）。【测：用前者生效；后者未测】
- 局限：只按路径匹配，挡不住 trait 方法间接调用、宏展开内部、依赖 crate 内部的分配（如 ndarray 内部 `to_owned`）——所以必须配 RTSan。【推】
- 进阶【推】：若要「函数级」而不是「crate 级」的规则（例如只禁 `#[rt]` 标注函数里的调用），可写 dylint（`https://github.com/trailofbits/dylint`）自定义 lint 在 HIR 上做调用图检查——这是对人提议「用语法树做模块」最贴切的 Rust 落点，但成本高，**建议先用 crate 级 clippy.toml，dylint 作为后续**。

### 2.5 panic = "abort" 还是 "unwind"

- 事实：
  - no_panic 需要 unwind 构建【官】。
  - Rust 1.81 起，panic 越过非 `-unwind` ABI（如 `extern "C"`）会 abort（此前是 UB）；需要跨边界 unwind 用 `"C-unwind"`。【传：release 公告摘要经搜索获得】本机实测：`extern "C" fn cb(){ panic!() }` → `thread caused non-unwinding panic. aborting.`，exit 134。【测】
  - coreaudio-rs 的输入回调声明为 `extern "C-unwind" fn input_proc`（`RES/coreaudio-rs/src/audio_unit/render_callback.rs:766`），闭包里 panic 会尝试穿过 CoreAudio 的 C/C++ 帧继续展开。【源】后果取决于 CoreAudio 帧有无展开表——**不可依赖**。【推】
  - cpal 的 macOS 输入回调在 `unsafe` 闭包里直接调用户 `data_callback`（`RES/cpal/src/host/coreaudio/macos/device.rs:621-657`），没有 `catch_unwind`。【源】
- 建议【推】：
  1. **证明构建**（CI）：`panic = "unwind"` + `#[no_panic]` + clippy deny 一组 panic 类 lint。
  2. **发布构建**：送音进程与推理进程 `panic = "abort"`（体积小、没有半毁状态），**进程级隔离当兜底**：推理进程挂 → 共享内存心跳停 → 送音进程/HAL 侧自动原声旁路；守护进程重启推理进程。这正是「多进程」设计的红利。
  3. A 级回调的最外层不管哪种 profile 都写成「不可能 panic 的薄壳」：只做 shm/rtrb 拷贝 + 旁路，核心逻辑不放这里。

### 2.6 预分配 / no_std 核心

- rtrb：`#![cfg_attr(not(feature = "std"), no_std)]` + `extern crate alloc`（`RES/rtrb/src/lib.rs:62,68`），构造时分配、之后不分配（:7-8）。【源】
- 「no_std + alloc」核心 crate 的价值【推】：dsp/engine-core 若 `#![no_std]`（只 `alloc` 用在构造期），就**无法**在处理函数里误用 `std::sync::Mutex`、`std::thread::sleep`、`println!`、文件 IO——编译器直接拒绝，比 lint 还硬。代价：ndarray/ort 等依赖是 std 的，所以只适合**纯 DSP 内核 crate**（biquad、压缩限幅、门控、STFT 帧缓存），不适合 engine 外壳。
- 更进一步：把「构造期」与「处理期」分到两个类型上——`Builder`（可分配，返回 `Result`）→ `Processor`（字段全是定长切片/`Box<[f32]>`，`process(&mut self, in:&[f32], out:&mut [f32]) -> Status`）。tympan-aspl 用零大小 `RealtimeContext` 令牌类型标记 RT 调用（只能显式化要求，不能单独防分配）。【传：docs.rs 源码摘要】

### 2.7 rtrb 与跨进程共享内存

- rtrb 是**进程内** SPSC：`RingBuffer::new` 用 `Vec` 分配、`Producer/Consumer` 持 `Arc`（`lib.rs:150` 起，drop 释放见 :241-258）。【源】
- **不能直接放进 mmap 共享内存**（指针、Arc 计数都是进程内地址）。【推】→ 送音进程 ↔ 推理进程需要自写 `#[repr(C)]` 定长头（`AtomicU64 head/tail` + cache-line padding + 版本号 + 心跳计数）+ 样本区的 SPSC，这块 unsafe 应集中在一个小 crate（建议 `shm`/`ipc`），按 rtrb 的做法：`CachePadded` head/tail（:104,109）、缓存对端索引（`cached_head/cached_tail` :312,319）、`Acquire/Release` 配对（:351,378）。【源（做法）/推（落点）】
- 验证：loom 穷举交错（需把原子类型抽象成可替换），miri 查越界与数据竞争；跨进程部分再做长跑压测（漂移 + 杀进程恢复）。【推】

---

## 3. Denormal / FTZ（aarch64）

- 寄存器：AArch64 FPCR.FZ 位于 bit 24；有 FEAT_AFP 的核上，FZ 只在 FPCR.AH=0 时生效，另有 FIZ（只冲输入）。【官】ARM FPCR 说明（镜像 https://df.lth.se/~getz/ARM/SysReg/AArch64-fpcr.html）；AFP 细节【传】（QEMU 提交说明）。
- Rust 做法：没有稳定 intrinsic，用 `core::arch::asm!("mrs {}, fpcr")` / `asm!("msr fpcr, {}")`，RAII guard 保存旧值、Drop 恢复；Firewheel `firewheel-graph/src/ftz.rs`、truce-core `denormal.rs`（`saved | (1u64 << 24)`，`options(nomem, nostack, preserves_flags)`，Miri 下跳过）。【源】https://docs.rs/crate/firewheel-graph/latest/source/src/ftz.rs ·https://docs.rs/truce-core/3.1.2/src/truce_core/denormal.rs.html
- **本机实测（Apple M5）**【测】：
  - 进程初始 `fpcr = 0x0`；置位后 `0x1000000`。
  - 4096×2000 次 `v*0.5 + 1e-45`：无 FTZ 5.562 ms（结果保持 `4e-45` 次正规数），FTZ 5.565 ms（结果 `0`）。→ **M5 上该微基准 denormal 无性能惩罚**，但数值不同。
- 含义【推】：
  1. 在 Apple Silicon 上开 FTZ 的主要理由不是速度，而是**与模型/x86 行为一致、避免 IIR 尾巴长期滞留在次正规区**；是否开要由 eval 定，不能凭 x86 经验。
  2. **「同一份处理代码」还要求「同一份浮点环境」**：FPCR 是线程状态，eval 线程默认 FZ=0，实时线程若开了 FZ=1，两者输出不逐位一致。→ `Engine::process` 入口统一套 `DenormalGuard`（dsp 定义、engine 调用），eval 走同一入口；回归测试比较 eval 与「模拟实时线程」逐位一致。
  3. ort 有 `with_flush_to_zero`（「for all threads」——指 ORT 自己的线程池）【官】；`intra_threads=1` 时计算在调用线程上（见 §5），调用线程的 FPCR 要由我们设。【推】
- 机器检查：单测（guard 内外 `1e-45*0.5` 是否为 0）；`debug_assert!(fpcr() & (1<<24) != 0)` 放 `process()` 入口。

---

## 4. 线程优先级、优先级反转、Audio Workgroup

- **Mach time-constraint**：audio_thread_priority（0.38.0）在 macOS 上用 `thread_policy_set(THREAD_TIME_CONSTRAINT_POLICY)`，`period = constraint = 回调周期`、`computation = 周期/2` 并封顶 50 ms、`preemptible = 1`（`RES/audio_thread_priority/src/rt_mach.rs:113-141`），并先 `thread_policy_get` 保存旧策略以便降级（:91-106，:40-61）。【源】
- cpal 在 Apple 上**不**依赖 audio_thread_priority（只在 Linux/Windows 的 `realtime` feature 下引入，`RES/cpal/Cargo.toml:23,132,141`）——CoreAudio 的 IO 线程本来就是实时线程。【源】→ 送音进程的回调不必再提权；**推理进程的自建降噪线程必须自己提权**。
- **Audio Workgroup**（WWDC20 *Meet Audio Workgroups*，Doug Wyatt）【官】https://developer.apple.com/videos/play/wwdc2020/10224/
  - 性能控制器（Apple Silicon 上决定大小核与频率）需要知道线程的 deadline；「All audio threads should be joined to workgroups.」
  - 与设备 IO **并行**的线程 → 加入设备 workgroup（macOS 可经 HAL 直接取）；**异步**、有自己 deadline 的线程 → `AudioWorkIntervalCreate` 自建 workgroup，主线程每周期 `os_workgroup_interval_start(start, deadline)` / `_finish`；`os_workgroup_join` 一次、退出前 `_leave`；`mach_absolute_time` 单位要用 `mach_timebase_info` 换算。【官】Apple 文档 *Adding Asynchronous Real-Time Threads to Audio Workgroups*（`os_workgroup_join` 返回 0 / EALREADY / EINVAL）【官】
  - **本项目落点**【推】：推理进程的降噪线程与送音进程的 IO 线程**异步**（中间隔着共享内存缓冲）→ 属于「自建 workgroup」一类；声纹线程是 C 级，不加入。
  - 本机 cpal / coreaudio-rs / audio_thread_priority **都没有** workgroup 支持（全文 grep `workgroup` 无结果）。【源】→ 需在 io-macos 里用 objc2 系列 / 手写 FFI 绑 `os_workgroup_*` 与 `AudioWorkIntervalCreate`（是否已有现成 Rust 绑定：**未找到出处**，交给 rusteco 核查）。
- **优先级反转**：RT 线程等低优先级线程持有的锁 → 被中优先级线程间接拖住。Doumler 2020：等 mutex「not only blocks the audio thread but also leads to priority inversion」；`try_lock` 本身安全但 `unlock` 可能要系统调用唤醒等待者 → 音频线程连 `std::mutex` 的 try_lock 也别用；必须共享可变状态时用自旋锁，音频侧只 `try_lock` 失败即用旧值/旁路，等待与退避（spin → pause/WFE → 偶尔 yield）放在非音频线程。【官】https://timur.audio/using-locks-in-real-time-audio-processing-safely
  - `os_unfair_lock` 的优先级继承语义：Apple 页面未写明【官：未载】；常见说法是它记录持有者、可做优先级提升【传】。无论如何 RT 线程不调用阻塞版。
- 机器检查：启动时读回 thread policy 并上报；每周期记录 `mach_absolute_time` 用时与 deadline，超时计数进原子指标；RTSan 挡 `pthread_mutex_*`。

---

## 5. FFI / unsafe 边界与 crate 放置

### 5.1 原则
- unsafe **只出现在三类 crate**【推】：
  1. `io-macos`：CoreAudio（cpal / coreaudio-rs / objc2-core-audio）、Mach 线程策略、os_workgroup、FPCR asm 可以放这里或 dsp 的 `arch` 模块。
  2. `shm`（或 `ipc`）：跨进程共享内存 SPSC。
  3. `infer`（从 engine 里拆出）：ort/tract 的薄封装，暴露 safe 的 `trait Denoiser { fn process_frame(&mut self, in:&Frame, out:&mut Frame) -> Status }`；GTCRN / FastEnhancer-M 两个实现。
  其余（dsp、engine、speaker、control、eval）`#![forbid(unsafe_code)]`。
- 沿用 rtrb 的 unsafe 纪律：`#![deny(unsafe_op_in_unsafe_fn)]`、`#![warn(clippy::undocumented_unsafe_blocks)]`（`RES/rtrb/src/lib.rs:65-66`），每个 unsafe 块写 `// SAFETY:`（cpal 的回调也这样写，`device.rs:622-624`）。【源】
- 工具：`[workspace.lints.rust] unsafe_code = "forbid"`，三个 FFI crate 在自己的 `[lints]` 里改成 `deny` + `#[allow]` 局部放行；CI grep 白名单。【推】

### 5.2 CoreAudio
- cpal（本机版本）在 Apple 上同时依赖 `coreaudio-rs 0.14.2` 与 `objc2-core-audio 0.3` 系列（`RES/cpal/Cargo.toml:155-175`）。【源】
- cpal 回调路径本身：`draining_input` 原子判断、`pending_xrun.swap`、`try_emit_error`（`try_lock`，`error_emit.rs:29`）——RT 侧没有阻塞锁；但 `StreamInner` 外层是 `Arc<Mutex<..>>`（`device.rs:665`），只在控制路径使用。【源】→ 可作为 A 级回调的参考实现，但 `try_emit_error` 拿到锁后在 RT 线程上**执行用户错误回调**（error_emit.rs:31），我们的错误回调必须只做原子写。【推】
- 自写 HAL 插件（C，跑在 coreaudiod）：不在 Rust 规则体系内；用上游 LLVM ≥20 编译后可用 `-fsanitize=realtime` + `[[clang::nonblocking]]` 做同样的证明（Apple clang 不支持，见 §2.2）。【测】【推】

### 5.3 ONNX Runtime（ort）/ tract：能否放 RT 线程
- ort 2.0.0-rc.15（包 ONNX Runtime 1.31）`SessionBuilder` 有：`with_intra_threads`、`with_inter_threads`、`with_intra_op_spinning`、`with_inter_op_spinning`、`with_parallel_execution`（默认关）、`with_memory_pattern`、`with_allocator`、`with_independent_thread_pool`、`with_deterministic_compute`、`with_flush_to_zero`。【官】https://docs.rs/ort/latest/ort/session/builder/struct.SessionBuilder.html
- ORT 线程文档：intra 线程数是「total」并**包含调用线程**（例：设 3 = 调用线程 + 2 个）；inter 池仅 parallel 模式创建；spinning 默认开。【官】https://onnxruntime.ai/docs/performance/tune-performance/threading.html → `intra=1` 时不建额外线程，计算在调用线程（我们的 RT 线程）上。【推，由「total 含调用线程」推得】
- IoBinding：`Session::create_binding` / `bind_input` / `bind_output(Tensor::new(&allocator, shape))` / `run_binding`；绑定的输出「placed into the same memory allocation provided in the bind_output call」，每次 run 覆盖。页面**未声称** run_binding 完全不分配。【官】https://ort.pyke.io/perf/io-binding
- 社区曾有「在 run 外预分配输入输出」的需求贴（以音频处理为动机）。【传】https://github.com/pykeio/ort/discussions/37
- tract：DeepFilterNet libDF 的 `process_raw` 每帧 `self.erb_buf.clone()`、`tvec!(...)`、`.into_tensor()`、`.pop().unwrap()`（`RES/DeepFilterNet/libDF/src/tract.rs:441-498`），`process` 里 `rolling_spec_buf.push_back(self.spec_buf.clone())`（:541-542）→ **每帧分配 + unwrap**。【源】
- DeepFilterNet 的 LADSPA 插件把模型放在 `thread::spawn` 的 worker（`RES/DeepFilterNet/ladspa/src/lib.rs:106-149,226`），但音频 `run()` 里用 `Mutex<VecDeque>` 交换数据（:411,421,456,468 `lock().unwrap()`）——**「模型离开回调线程」是对的，「用 Mutex 交接」是反例**。【源】
- **建议**【推】：
  1. A 级回调永远不跑推理；推理放 B 级线程（本设计已如此）。
  2. B 级线程里 ort 配置：`intra=1`、`inter=1`、spinning 关、sequential、`with_memory_pattern(true)`（形状固定）、`with_flush_to_zero`、IoBinding 预绑定输入输出；预热若干帧后再进入「受检区」。
  3. 用 RTSan 实测（实验 E1）：`#[nonblocking]` 包 `run_binding`，看稳态是否还有 `malloc/free/pthread_*`。若有：(a) 记入 suppressions 并量化最坏耗时（p99.9 / max）当作「已知、有界的违规」；(b) 或评估 tract 的 `SimpleState` 复用 / 自写 GTCRN 推理（GTCRN 很小）作为长期方案。**在 E1 出结果前，「ort 可在实时线程跑」只能写【推】**。
  4. eval 与实时共用 `infer` crate 的同一个 `Denoiser` 实现——一致性的边界在 trait，不在线程。

---

## 6. 实时路径错误处理

- 规则【推，依据 Doumler 2020 的 fallback 思路【官】】：
  - `process()` 签名：`fn process(&mut self, input:&[f32], output:&mut [f32]) -> Status`，`Status` 是 `#[repr(u8)] #[must_use]` 枚举（Ok / Bypassed{reason} / Underrun / ModelTimeout / NonFinite）。不返回 `Result<_, Box<dyn Error>>`（Box 分配）、不 `format!`。
  - 任何非 Ok：输出 = 原声（带几毫秒交叉淡化，避免咔哒声），原子计数器 +1，**错误详情经 rtrb 推给日志线程**（推失败就丢弃并计数，绝不等待）。cpal 的 `try_emit_error` 是同一思路（try_lock 失败就不报）`RES/cpal/src/host/error_emit.rs:25-37`。【源】
  - NaN/Inf：输出前 `is_finite` 检查，命中即该帧旁路并重置模型状态（重置所需缓冲预分配）。
  - 跨进程：推理进程每帧写心跳序号；送音进程/HAL 侧若 N 帧未见前进 → 自动原声旁路。
- 机器检查：故障注入测试（模型返回 NaN、推理人为超时、shm 生产者被杀）→ 断言输出等于原声（容差内）且无阻塞；这些测试同时在 RTSan 下跑，保证「出错路径」本身也是实时安全的（出错路径是最容易偷偷 `format!` 的地方）。

---

## 7. 经典演讲 / 文章对 Rust 的适用性

| 来源 | 主张 | 对 Rust 的翻译 | 证据 |
|---|---|---|---|
| Ross Bencina, *Real-time audio programming 101: time waits for nothing*（2011，`http://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing`） | 回调里不分配、不加锁、不做 IO/阻塞系统调用、不用最坏时间无界的算法；用无锁 FIFO 传消息 | 规则全部照搬；Rust 的所有权让「谁释放」显式化（→ basedrop / 不在 RT 线程 drop），但**类型系统不保证实时安全**——`Vec::push`、`Arc` 的最后一次 drop、`Mutex` 都是 safe Rust | 【传】：原文本机抓取被拒（ECONNREFUSED），内容依 2011-07 linux-audio-dev 列表转述（https://lists.linuxaudio.org/archives/linux-audio-dev/2011-July/030643.html）与记忆；LWN 评论指出部分规则针对通用 OS（https://lwn.net/Articles/453005/） |
| Timur Doumler, *Using locks in real-time audio processing, safely*（2020-04-14） | 见 §4 | Rust `std::sync::Mutex` 同样不可用（本机 RTSan 证明还会懒分配）；`try_lock` 方案可用 `AtomicBool` 自旋锁，但优先选原子 / SPSC / 不可变发布（`arc-swap` 类或 basedrop `SharedCell`） | 【官】https://timur.audio/using-locks-in-real-time-audio-processing-safely ；【测】 |
| Timur Doumler, *Real-time Programming with the C++ Standard Library*（CppCon 2021） | 标准库哪些设施实时安全 | 对应到 Rust std：`core`/`alloc` 中不分配的部分（切片迭代、原子）可用；`std::sync`、`std::thread::sleep`、`std::io` 不可用 | 【官】视频 https://www.youtube.com/watch?v=Tof5pRedskI （页面无描述，内容未逐条核对） |
| Timur Doumler, *Lock-Free Programming with Modern C++*（ACCU 2017） | atomic、lock-free vs wait-free、无锁队列 | Rust 内存序与 C++ 同一模型；rtrb 即 wait-free SPSC | 【传】https://accu.org/video/spring-2017-day-1/doumler （搜索摘要） |
| Timur Doumler ADC20 同名锁演讲、ADC24 *Wait-Free Thread Synchronisation With the SeqLock* | SeqLock 用于「一个写者、RT 读者」的大块参数 | Rust 中 SeqLock 读侧需要 `read_volatile`/原子拷贝避免 UB，需 miri/loom 验证 | 【传】标题来自搜索摘要，视频 URL **未找到出处** |
| Fabian Renn-Giles & Dave Rowland, *Real-time 101* Part I/II（ADC，年份待核） | 实时问题空间 + 工具箱 | 同上 | 【传】标题来自搜索摘要；常被说成 Doumler 的演讲，**实为 Renn-Giles & Rowland**；视频 URL **未找到出处** |
| Micah Johnston, *Basedrop: A garbage collector for real-time audio in Rust*（2021-04-26） | `Owned/Shared` drop 入 MPSC 队列、Collector 他处回收；`SharedCell` 读者计数、写者自旋（让 RT 读者便宜） | 模型热切换 / 参数块替换的现成模式；限制：不支持 DST、无 Weak | 【官】https://micahjohnston.com/posts/basedrop/ |
| WWDC20 *Meet Audio Workgroups* | 所有音频线程都要入 workgroup | 见 §4 | 【官】 |

Rust 社区会议：在 RustConf / EuroRust / RustNL 未找到专门讲实时音频的演讲【未找到出处】；Rust 音频演讲主要出现在 ADC（Ian Hobson、Chase Kanipe ADC23 *Writing Elegant DSP Code with Rust*）【传：播客页面】。**社区共识的「决策」主要沉淀在库与工具里**（rtrb、basedrop、assert_no_alloc、rtsan-standalone、Firewheel 的 FTZ），而不是会议演讲。

---

## 8. 回应人原话：「超级语法树 / AST 做模块、可拆分的核心包」

人原话：「我建议用超级语法树 / ast 来把东西做成模块，或者大部分东西做成可拆分以及的和新包和模块」。认真对待后的落点【推】：

1. **Rust 里「AST 级」的机器规则有现成三件套**，都能直接服务实时证明：
   - proc-macro 属性（syn 改写语法树）：`#[nonblocking]`（RTSan）、`#[no_panic]`——在函数上「贴标签」即接入证明。
   - lint：clippy（`disallowed-*`、`unwrap_used` 等）在 AST/HIR 上检查；需要函数级调用图规则时上 dylint。
   - 编译器本身：`#![no_std]`、`#![forbid(unsafe_code)]` 是最硬的「语法层禁令」。
2. **crate 是规则生效的最小可靠单位**（clippy.toml 按 crate 目录查找、`[lints]` 按 package、`no_std`/`forbid` 按 crate 根）。所以「拆成核心包」不仅是工程整洁，而是**让实时规则能被机器精确地只施加在实时代码上**。
3. 建议在设计稿 7 个 crate 基础上再拆两个 unsafe 小核【推】：
   - `dsp`（`no_std + alloc`，forbid unsafe 但 FPCR guard 例外——可把 guard 移到 `io-macos` 或单独 `rt-arch` 小 crate 保持 dsp 纯 safe）
   - `infer`（ort/tract 封装，unsafe 允许，严格 clippy.toml，RTSan 测试）
   - `shm`（跨进程 SPSC，unsafe 允许，loom/miri）
   - `engine`（forbid unsafe，严格 clippy.toml，`#[no_panic]` + `#[nonblocking]` 标在 `process`）
   - `io-macos`（CoreAudio/Mach/workgroup，unsafe 允许）
   - `speaker` / `control` / `eval`（C 级，宽松规则；eval 只通过 `engine::process` 调用）
4. 不建议的方向：自建一套 AST 代码生成框架来「生成模块」——Rust 的 crate + trait + feature 已覆盖；额外的元编程层会让 RTSan/no_panic 的栈与符号更难读。

---

## 9. 待做的有界小实验（建议进 exp 线）

| # | 实验 | 判据 | 预计 |
|---|---|---|---|
| E1 | ort + GTCRN：`#[nonblocking]` 包 `run_binding`（intra=1、IoBinding、预热 100 帧），`RTSAN_OPTIONS=halt_on_error=false` 统计稳态违规函数与次数 | 稳态 0 违规 → ort 可进 B 级；否则列出违规函数 + 最坏耗时 | 半天 |
| E2 | 同上换 tract（复用 `SimpleState`），对比违规与 p99.9 | 同上 | 半天 |
| E3 | engine 证明 bin：release + unwind + `#[no_panic]`（运行期长度参数）+ fat LTO | 链接通过 | 2 h |
| E4 | FTZ 一致性：eval 与模拟实时线程逐位比较（guard 开/关两组） | 同 guard 下逐位一致 | 1 h |
| E5 | 推理线程 time-constraint + 自建 workgroup vs 不加入，在 E 核负载下统计 deadline miss | miss 率下降 | 半天 |
| E6 | 故障注入（NaN、超时、杀推理进程）在 RTSan 下跑 | 输出 = 原声且无违规 | 半天 |

---

## 10. 本机实测记录（可复现）

- 环境：Apple M5；rustc 1.99.1；Apple clang 21.0.0（clang-2100.3.34.2）。
- `rtprobe`（assert_no_alloc 1.1.2）：debug `alloc` → abort exit 134；release → 通过。FTZ：`fpcr 0x0 → 0x1000000`；无 FTZ 5.562 ms / FTZ 5.565 ms；样本 `4e-45` vs `0`。`extern "C"` 内 panic → `thread caused non-unwinding panic. aborting.` exit 134。clippy：`Vec::push` 被 `disallowed_methods` 命中。
- `rtsanprobe`（rtsan-standalone 0.3.0，prebuilt libs）：须 `RTSAN_ENABLE=1` **编译**且经 `cargo run` 运行；命中 `calloc` / `malloc` / `pthread_mutex_lock` / `pthread_mutex_unlock` / `nanosleep`；原子不报；release 仍有效。
- `nopanicprobe`（no-panic 0.1）：见 §2.3。

## 11. 未找到出处 / 待 rusteco 补

- RTSan 拦截的系统调用完整清单（mmap、semaphore、文件 IO 等）——官方文档未列。
- `os_workgroup` 的现成 Rust 绑定（objc2 系列是否覆盖）。
- ort `run_binding` 稳态是否零分配——官方未声称，需 E1。
- Bencina 原文逐条（本机抓取被拒）；Real-time 101 / SeqLock 演讲视频 URL。
- `os_unfair_lock` 优先级提升语义的官方明文。
