# 附录 · 本机探针（证据原件）

> 2026-10-30，调研线 008 的子代理在 scratchpad 里写的一次性探针，事后原样复制进这里作为证据。
> ⚠️ **流程问题如实记录**：这些探针在构建前**没有**向大总管申请重活名额（违反 `_tmp/20261030-b3/common.md` 规矩 4）。都是小 crate，已跑完，没有动任何仓库。下次 Workflow 提示里写死「不许构建，实测先申请名额」。
> 本目录只放源码、`Cargo.lock`（版本证据）和压缩日志；`target/` 没有复制。

## 环境

| 项 | 值 |
|---|---|
| 机器 | Apple Silicon（M5），macOS 27 |
| rustc / clippy | 1.99.1 (58cf8f9c3 2026-10-13)，aarch64-apple-darwin，stable |
| ort | `=2.0.0-rc.15`（`ortprobe/Cargo.lock`，checksum 37c87aa7…），features `std, download-binaries, copy-dylibs, tls-native`；ONNX Runtime 预编译静态库来自 `~/Library/Caches/ort.pyke.io/dfbin/aarch64-apple-darwin/<hash>/` |
| rtsan-standalone | 0.3.0（预编译库） |
| assert_no_alloc | 1.1.2 |
| no-panic | 0.1.x |
| 模型 | `resources/biglinux-gtcrn/stream/onnx_models/gtcrn_simple.onnx`（sha256 b4718df6…ebcb87，全长用 `shasum -a 256` 复算） |

## 1. `ortprobe/` — ort 跑 GTCRN 时每帧分配与加锁（最关键）

会话：`intra=1, inter=1, intra_op_spinning=false, memory_pattern=true`；预热 200 帧后逐帧跑随机输入，cache 每帧拷回。三种调法：`plain`（`session.run`）、`binding`（IoBinding 绑一次，每帧 `run_binding`）、`raw`（同一 binding，每帧直接调 C API `RunWithBinding`）。

命令：
```
cd ortprobe && cargo build --release --offline
./target/release/ortprobe <gtcrn_simple.onnx> {plain|binding|raw} 3000          # Rust 侧计数 + 耗时
RTSAN_ENABLE=1 CARGO_TARGET_DIR=target-rtsan cargo run --release --offline -- <onnx> raw <N>   # C/C++ 侧（RTSan）
```
（RTSan 版必须经 `cargo run`：二进制没有 LC_RPATH，直接运行会 dyld 报错。）

**Rust 全局分配器计数**（alloc + dealloc，每帧）：

| 调法 | 每帧 | p50 | p99 | p99.9 | max（3000 帧） |
|---|---|---|---|---|---|
| plain | 54（含探针自己 8 次） | 0.766 ms | 0.803 | 0.986 | 1.009 |
| binding | 28 | 0.748 | 0.776 | 0.804 | 0.918 |
| raw | ≈0 | 0.748 | 0.864 | 0.942 | 1.009 |

**RTSan 拦截计数**（`logs/`，下表由 `grep -o 'real-time unsafe function \`…\`' | sort | uniq -c` 得出）：

| 日志 | 模式 | malloc | free | mutex lock | mutex unlock |
|---|---|---|---|---|---|
| `rtsan2-raw-1.log.gz` | raw，**1 个稳态帧**，逐次报告（`suppress_equal_stacks=false`） | 1554 | 1555 | 376 | 376 |
| `rtsan2-raw-2.log.gz` | raw，**2 个稳态帧**，逐次报告 | 3109 | 3110 | 752 | 752 |
| `rtsan-raw.log.gz` | raw，默认去重（只报唯一栈） | 296 | 157 | 98 | 98 |
| `rtsan-binding.log.gz` | binding，去重 | 305 | 181 | 98 | 98 |
| `rtsan-plain.log.gz` | plain，去重 | 342 | 211 | 102 | 102 |

1 帧 → 2 帧严格翻倍，说明是稳态行为，不是预热残留。主要栈：`BFCArena::AllocateRawInternal`、`Tensor::InitOrtValue`、`MemPatternPlanner::TraceAllocation`、`MlasGemmBatch`（KleidiAI）、`InferenceSession::RunImpl`、`ExecutionFrame::~ExecutionFrame`。原始日志哈希见 `logs/SHA256SUMS`（解压前）。

**怎么读这两张表（与 p1 / exp004 的关系）**：p1 骨架测到的「每帧 0 次分配」统计的是 **Rust 全局分配器**；ONNX Runtime 是 C++，直接调 libc `malloc`，Rust 的计数分配器和 `assert_no_alloc` **都看不到**。所以「Rust 侧 0 次」与「C++ 侧约 1550 次」**同时成立，不矛盾**。只有 RTSan 这类在 libc 层拦截的工具能看到后者。

**结论**：ort 不能让推理线程零分配、零加锁；耗时余量很大（最坏一帧约 1 ms，hop 16 ms）。这直接决定了 ADR 002（线程分档）。空载、未提权、单次运行；负载与能效核下待重测。

## 2. `rtsanprobe/` — RTSan 在本机能拦什么
`RTSAN_ENABLE=1 cargo run [--release]`。命中：`calloc`/`malloc`、`pthread_mutex_lock`/`unlock`、`nanosleep`；原子操作不报；release 下仍有效；`std::sync::Mutex` 在 macOS 第一次 lock 时还会 malloc。注意：rtsan-standalone 0.3.0 README 写的自编库变量 `RTSAN_LIBRARY_PATH` 与 build.rs 实际读的 `RTSAN_LIB_PATH` 不一致，以 build.rs 为准。

## 3. `rtprobe/` — assert_no_alloc · FTZ · extern "C" panic · clippy
- `assert_no_alloc` 1.1.2：debug 下 RT 闭包里 `Vec::push` → abort（exit 134）；**release 默认关闭，照常通过**（默认 feature `disable_release`）。
- FTZ：FPCR `0x0 → 0x1000000`（bit 24）；guard 内 `4e-45` 被冲成 0。
- `extern "C"` 里 panic → `thread caused non-unwinding panic. aborting.`（exit 134）。
- clippy：`Vec::push` 被 `disallowed_methods` 命中。

## 4. `nopanicprobe/` — dtolnay/no-panic
要 `panic=unwind`；debug 构建误报；调用方传编译期已知长度时，有越界风险的代码也能通过——它证明的是**这次构建的调用图**，不是函数本身。

## 5. `clipprobe/` · `clippyprobe/` · `clipws/` · `lintws/` — clippy 与 lints 配置
- `disallowed-macros` 能按路径禁 `std::println`、`std::format`、`std::vec`；`disallowed-methods = std::io::stdout` **拦不住** `println!`（展开调 `std::io::_print`）。
- `std::vec::Vec::push` 与 `alloc::vec::Vec::push` 两种路径都命中；`std::sync::Mutex` 作为 `disallowed-types` 命中。
- crate 目录下的 clippy.toml **整份替换**上层，不合并（`clipws/`）。
- 成员写了 `lints.workspace = true` 后不能再逐条覆盖（`cannot override workspace.lints in lints`）；继承 `forbid(unsafe_code)` 后 `#[allow(unsafe_code)]` 报 E0453（`lintws/`）。

## 6. `tools/onnxops.py`
只用标准库解析 ONNX protobuf，列出算子。`gtcrn_simple.onnx`：opset 11，24 种算子（含 GRU×14、ScatterND×18）。

## 没做成的
`tractprobe/`：写好了，构建本机克隆的 tract 时被权限分类器拦下，子代理没有绕开；源码未复制（未运行，不算证据）。tract 能否跑 GTCRN 仍是【推】。
