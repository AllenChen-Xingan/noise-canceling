# ADR 002 · 实时分三档；推理线程允许 ONNX Runtime 内部分配

- 状态：提议（2026-10-30，rustarch）· **涉及 `.42cog/intent.md` 作品区「音频线程不分配内存、不加锁」的口径，⟨待拍板 1⟩**
- 规约落点：`specs/rust/architecture.md` §4

## 背景
设计稿 §7「音频线程里不分配内存、不加锁、不做系统调用」。003 管线已定 IO 回调与推理线程分开，中间用环形缓冲，原声旁路由输出回调做。问题是推理线程能不能也做到零分配。

## 证据
本机探针（`appendix-probes/README.md` §1，命令、版本、原始日志都在）：ort `=2.0.0-rc.15`、GTCRN simple、intra=1、IoBinding 绑一次、预热 200 帧后，RTSan 逐次计数**每个稳态帧 malloc 1554、free 1555、pthread_mutex lock/unlock 各 376**；2 帧严格翻倍【测】。来源是 ORT 的 `BFCArena`、`Tensor::InitOrtValue`、`MemPatternPlanner`、MLAS GEMM。耗时：p99.9 0.80–0.94 ms，max ≤ 1.07 ms，hop 16 ms【测】。
同时 Rust 计数分配器在 raw `RunWithBinding` 调法下每帧 ≈ 0 次——Rust 全局分配器看不到 C++ 的 malloc【测】。

## 决定
| 档 | 线程 | 承诺 |
|---|---|---|
| A 硬实时 | IO 回调（以后 HAL 驱动 IO） | 零分配、零锁、零阻塞系统调用、不 panic，第三方代码也算；唯一例外是回调里一次非阻塞的跨进程唤醒（`os_sync_wake`，009 §一.1），按函数名放行。cpal / coreaudio-rs 在回调里有 `mach_timebase_info`、`Vec::resize`，过不了，所以 A 档零容忍从 M1 自写 IOProc 起生效 |
| B 软实时 | 推理进程降噪线程 | 我们的 Rust 代码零分配零锁；ORT 内部分配 / arena 锁允许，按**函数名或命名空间**登记在进仓的 RTSan 豁免文件里（ORT 静态链接，栈里没有库名；匹配要求符号化），B 档 `halt_on_error=false`，去重后出现新栈即红；适配器为 ort 编排输入输出的 Rust 侧分配按模型登记上限（GTCRN 0，FE-M 见下）；每帧截止时间，超时 / 出错走模型级送原声；验收看 p99.9 与 max |
| C 非实时 | 声纹、控制、日志、eval、构造期 | 不限 |

## 理由
- B 档就算偶尔慢一帧，也不会直接造成爆音：IO 回调读到环 B 欠载就切到原声环 C（003 decision M1–M2 行）。真正不能出错的是 A 档
- ort 的分配在 C++ 内核里，Rust 侧改不掉；替代方案（tract、自写推理）都还没验证（`decision.md`「仍未补齐」E-tract）
- 耗时余量约 15 倍，分配带来的抖动目前看不出来，但要在负载、能效核下重测（E-load）

## 代价
- intent.md 的「音频线程不分配、不加锁」要改成分档表述，需要人同意
- 豁免文件是一个新的维护面：升级 ort 时要重跑 RTSan、重审豁免栈
- 如果负载下 p99.9 变差，要回头做 E-ort（关 arena、自定义分配器）或换 tract

## 对 p1 / exp004 的影响
p1 的「每帧 0 次分配」统计的是 Rust 全局分配器，C++ 层看不到，与上面的结论不矛盾。exp004 README 要补一句注明口径（由大总管转给后续批次改）。Rust 侧的 0 依赖调法：p1 用 `OutputSelector` 预分配全部输出，实测 0 次；不预分配的 `run` 是 54 次。FastEnhancer-M 7 进 7 出超出 ort 栈上限，每帧 7 次小分配（rusteco 009 `decision.md` §一.4），要登记为 B 档已知分配或重新导出。
