# 给 p1 的选型答复（2026-10-30 · rusteco）

证据等级：【源码】= 本机 `~/.cargo/registry/src/index.crates.io-*/ort-2.0.0-rc.15/`（下称 `ort/`）亲读 ·【API】= crates.io / GitHub API 当日返回 ·【推断】。

## 1. ort 2.0.0-rc.15：Rust 侧每帧会分配吗

**会，`run` 和 `run_binding` 都会。** IoBinding 只省掉了输出缓冲的分配，省不掉 Rust 侧包装：
- `run_binding_inner` 每次调 `GetBoundOutputValues`，用 session 的 ORT 分配器分配一个指针数组再释放【源码】`ort/src/session/run.rs:301-305`
- 每个输出都包成 `DynValue`，里面 `Arc::new(AllocatorHandle{..})`——**每个输出每帧一次 Rust 堆分配**【源码】`ort/src/value/mod.rs:304-310`（`from_session_output_ptr`，`run.rs:309` 调用）
- 普通 `run`：输入/输出名先转 C 字符串，总长不超过栈缓冲时走栈、超过就 `CString::new` 上堆【源码】`ort/src/util/stack.rs:53-80`；输入 ≤ 6、输出 ≤ 4 个时用 SmallVec 放栈上【源码】`ort/src/util/stack.rs:13-15`。GTCRN 流式模型有 1 个输出 + 3 个缓存输出 = 4 个，刚好卡在线上【推断，需看导出的 ONNX】

**ORT 的 C++ 内部分配绕过 Rust 全局分配器**——assert_no_alloc 抓不到（对）。业内常见做法，按推荐顺序：
1. **不把推理放进硬实时回调**：003 技术方案里降噪线程本来就在推理进程，送音进程的 CoreAudio 回调只读写环形缓冲（`docs/plan/003_tech-design.md` §一）。推理线程是「软实时」：允许有界的小分配，靠 p99.9 / 最大耗时与掉帧数守住。这是 DeepFilterNet、biglinux-gtcrn 一类实时降噪的共同形态【推断，Workflow 正在回源码核】
2. 固定形状 + `with_memory_pattern(true)`（`ort/src/session/builder/impl_options.rs:128`）+ CPU arena（`CPUExecutionProvider.use_arena`，同文件 :394 注释）→ 预热几帧后 ORT 内部复用 arena，基本不再向系统要内存【推断】
3. `with_intra_threads(1)` 加关掉 spinning（`with_intra_op_spinning(false)`，`ort/src/session/builder/impl_config_keys.rs:135`）：不起线程池、不空转，也省电（D31）【源码 + 推断】
4. **用 RTSan 量**：它在 libc 层拦截 `malloc` / `pthread_mutex_lock` 等，**连 ORT 的 C++ 分配也能抓到**，正好补 assert_no_alloc 的盲区（见第 2 条）

**建议（2026-10-30 更正，以此为准）**：**不用 `run_binding`，用 `run_with_options` + `OutputSelector::preallocate` 把全部输出都预分配**。这条路 Rust 侧可以零分配：
- 预分配的输出走 `DynValue::clone_of`，只是 `Arc::clone`，不 `Arc::new`【源码】`ort/src/session/run_options.rs:138`、`ort/src/value/mod.rs:393-397`；只有没预分配的输出才走 `from_session_output_ptr` 的 `Arc::new`【源码】`ort/src/session/run.rs:270-281`
- 输入输出名合计 ≤ 12 个、总长 ≤ 768 字节时走栈【源码】`ort/src/util/stack.rs:21-23`
- 输入用初始化时建好的自有 `Tensor`，每帧用 `extract_tensor_mut` 写数，**不要每帧 `TensorRef::from_array_view`**（它也会 `Arc::new`，`ort/src/value/mod.rs:103`，怀疑派复核）
- GTCRN 的缓存状态：建两套状态张量 + 两个 `RunOptions`（A 的输出是 B 的输入，奇偶帧轮换），参考 `resources/biglinux-gtcrn/ladspa/src/model.rs:282-302`（它自己每帧新建 TensorRef，是反例的一半）
- 每帧拿到的 `SessionOutputs` 里持有预分配值的 Arc 克隆，**下一帧 run 前必须 drop 掉**；输入输出是否允许别名没查到文档，乒乓方案正好避开
- ORT C++ 内部（arena、线程池）稳态是否还 malloc：未实测，用 RTSan 量；`with_flush_to_zero` 先别默认开（DeepFilterNet #713：FTZ + 单位范数 0/0 出 NaN），先加 NaN/Inf 守卫

版本跟 Python 1.31 对齐利于对拍，同意。

## 2. 不分配断言：assert_no_alloc vs rtsan-standalone

| | assert_no_alloc 1.1.2 | rtsan-standalone 0.3.0 |
|---|---|---|
| 许可 | BSD-1-Clause【API】，兼容 GPL-3.0 | Apache-2.0【API】，兼容 |
| 维护 | 最后发布 2021-08，仓库最后推送 2023-08【API】——基本停更 | 发布 2025-09-27【API】 |
| 抓什么 | 只抓经 Rust 全局分配器的分配（要把它设成 `#[global_allocator]`） | 在 libc 层拦截：malloc/free、锁、系统调用、sleep 等；**C/C++ 依赖（ORT、CoreAudio）也抓** |
| 平台 | 都行 | Linux / macOS / iOS（README 明载）|
| 代价 | 几乎零，可常开 | 构建脚本默认用 curl 下载预编译库（供应链与离线构建要注意，可改为本地编译 LLVM，首编几分钟）；`RTSAN_ENABLE=1` 才生效 |

更正补充（复核后）：rtsan-standalone 默认分支最后提交 2025-09-27，命门 3 只算有条件满足；默认特性 `prebuilt-libs` 在 `RTSAN_ENABLE` 设了时才联网下载；环境变量是 `RTSAN_LIB_PATH`（README 写错成 RTSAN_LIBRARY_PATH，以 `sys/build.rs:54` 为准）。另有 rustc nightly 原生 `-Zsanitizer=realtime`，已支持 aarch64-apple-darwin，可做第二条路。assert_no_alloc 的替身可选 `nice-assert-no-alloc` 1.0.0（stable、零依赖），或照它自写约 60 行计数分配器。

**建议**：两个都用，分工——**`assert_no_alloc`（或 nice-assert-no-alloc）常开在 debug/test**（便宜，管纯 Rust 的 dsp crate）；**rtsan-standalone 作为可选 feature**（`--features rtsan`），在 CI 的一个 job 和 P3 实测时开，专抓 ORT 与系统调用。dsp crate 做成纯 Rust、零分配，用 assert_no_alloc 就够。

## 3. FFT：realfft 3.x
同意。realfft 3.5.1 最新发布 2026-10-26，MIT【API】；rustfft 6.4.1，MIT OR Apache-2.0【API】，aarch64 有 NEON 实现【推断，Workflow 在核文件:行号】。用 `process_with_scratch`，planner 与 scratch 在初始化时分配。

## 4. WAV：hound 3.5
同意，3.5.1，2023-09，Apache-2.0【API】。功能已定型，停更风险低；只在离线 / eval 路径用，不进实时线程。

## 5. cargo-deny 许可白名单（GPL-3.0 项目）
cargo-deny 0.20.2（2026-07），MIT OR Apache-2.0【API】。起步模板（`deny.toml` 里的 `[licenses]`）：

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause",
         "BSD-1-Clause", "ISC", "Zlib", "0BSD", "Unicode-3.0", "MPL-2.0", "GPL-3.0-or-later", "GPL-3.0-only",
         "LGPL-3.0-or-later", "LGPL-2.1-or-later", "BSL-1.0", "CC0-1.0"]
confidence-threshold = 0.9
```
不放 `GPL-2.0-only`（与 GPL-3.0 不兼容）。`BSD-1-Clause` 是给 assert_no_alloc 用的。ort 链接的 onnxruntime 二进制是 MIT，cargo-deny 看不到，要在第三方许可清单里手写一条【推断】。（rustarch 线在写「规则用哪个工具」，最终模板以它为准。）

## 6. FTZ / 非正规数：no_denormals 0.3.0
现成可用，但 0.3.0 发于 2025-09-11，已超 12 个月（命门 3 边缘）；替代是照 nih-plug `ScopedFtz` 自写两条指令（`resources/nih-plug/src/wrapper/util.rs:25,191`）。MIT，2025-09-11【API】；aarch64 上读写 FPCR 置 FZ 位（含 fp16）【源码，GitHub 上 `src/lib.rs:5,72-78`】，用的是 RAII guard（drop 时恢复原值，guard 不能跨线程）。在推理线程入口持有一个 guard 即可，不用自己写内联汇编。注意：FPCR 是线程级的，ORT 自己的线程池线程不受影响——又一个设 `intra_threads(1)` 的理由【推断】。
