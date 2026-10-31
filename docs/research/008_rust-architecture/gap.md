# 两类发现 · 008

> 2026-10-30。四态：已实现 / 部分 / 没有 / 不适用。

## 一、它们做到的（可借）

| 做法 | 谁做到 | 态 | 我们怎么用 |
|---|---|---|---|
| 核心不做 IO，IO 集中在一个 crate | rust-analyzer【官】 | 已实现 | `vdn-engine` sans-IO；平台只在 `vdn-io-macos` |
| 实时 / 非实时分开（构造 vs process） | nih-plug、firewheel、fundsp【源】 | 已实现 | `Model` 构造在 NRT，RT 只调 process / reset |
| 实时模块根 `#![deny]` + clippy 禁用清单 | tympan-aspl【源】 | 已实现（其 println 注释有误，r7 C6） | §5.1 §5.2 |
| tidy 测试随 `cargo test` 跑 | rust-analyzer【源】 | 已实现 | `xtask` |
| RTSan 进 Rust | rtsan-standalone【源+测】 | 已实现，本机可用 | G8 |
| 封闭 enum 控制消息、引擎对模块泛型 | Hobson ADC 2025【官·幻灯片】 | 已实现 | `Cmd`、`Engine<M>` |
| 旧对象回收队列 | fundsp `NetReturn`【源】 | 已实现 | 换模型 |

## 二、它们没做的

| 缺口 | 机会还是天堑 | 我们的处理 |
|---|---|---|
| 推理库在实时线程零分配 | **天堑（就 ort 而言）**：ORT 内核每帧约 1554 次 malloc【测】 | ADR 002 分档；E-ort / E-tract 再看能缩多少 |
| cargo-deny 证明传递依赖不出现 | 没顾上做（工具只管直接边）【源】 | `xtask check-deps` |
| Rust 的 Audio Workgroup 绑定 | 没顾上做（cpal、coreaudio-rs、audio_thread_priority 都没有）【源】 | `vdn-rt-thread` 手写约 60 行 FFI |
| 共享内存上的 SPSC | 没顾上做（rtrb 放不进 shm）【源】 | `vdn-shm` 自写，loom + Miri |
| Rust 大会上的音频架构演讲 | 无人涉足（RustConf 等没找到） | 实时规则学 ADC，工程组织学 matklad / rust-analyzer |
| 端到端延迟账 | 没人算过整条链 | rusteco 估约 56 ms（默认参数）；P3 实测 |
| ast-grep 规则带属性函数的写法、cargo-deny ban 工作区成员 | 未实测 | `decision.md`「仍未补齐」E-deny |
