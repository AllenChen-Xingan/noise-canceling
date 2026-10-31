# rustarch 对 p1 骨架的初审（2026-10-30，调研未完，终版以 specs/rust/architecture.md 为准）

读的：根 Cargo.toml、四个 crate 的 Cargo.toml、engine/src/{model,engine,offline,lib}.rs、models/src/{session,gtcrn}.rs。

## 先修（推送前）
0. target/ 被自动提交进分支（另发）。.gitignore 加 `target/`。

## 你的四问
1. **4 个 crate 够 P1**。eval 现在不拆，但 `offline::run` 留在 engine 是对的（它是「同一份处理代码」的驱动，不碰 IO）。**一旦出现 WAV 读写之外的东西（指标、场景矩阵、报告 JSON）就拆 `vdn-eval`**，规则：engine 永远不依赖 hound / serde。CLI 现在依赖 hound 没问题。
2. **波形接口对**：GTCRN（512/256 STFT）和 FastEnhancer 前端不同，外层统一「hop 进 hop 出」最稳。频域共享（设计稿 §11「共用一次 STFT」）留到音色链要频谱时再加一个**扩展 trait**，不要现在改 Model。
3. **静态分发 Engine<M> 留着**，但实时切换 GTCRN ↔ FE-M 需要一个运行期可选的类型：建议在 models 里加 `enum AnyModel { Gtcrn(Gtcrn), FastEnhancer(FastEnhancer) }` 并 impl Model（手写 match，2 个变体不必用 enum_dispatch 宏）。CLI 的 Box<dyn> 也可以，二选一，别两套都留。trait 上加 `Send` 约束（`pub trait Model: Send`），模型要搬到实时线程。
4. **命名 vdn-* 包名 + 目录不带前缀可以**（Cargo 惯例，rust-analyzer 也是包名有前缀、目录短名之一种；终版我会核实 matklad 原文再定）。

## 我看到的另外几处
- `Engine::process` 里两个 `assert_eq!`：实时路径会 panic。改 `debug_assert_eq!`，release 下长度不符直接整 hop 送原声并计数。
- **`Gtcrn::process` 会分配**：`ort::inputs![]` 建 Vec，`run_with_options` 返回的 SessionOutputs 也分配（待 rusteco/调研核实到行号）。现在的 no_alloc 测试只覆盖假模型，**它证明不了真模型路径**。P1 离线无所谓，但请在 gap 里写明，并在 bench 里加「每 hop 分配次数」计数（divan 的 AllocProfiler 或自写计数分配器），给 P3/P4 留基线。架构上我会把规则分两档：IO 回调线程=硬实时（绝不分配，机器挡）；推理线程=软实时（先量 p99.9 与分配次数，再逐步清零）。
- `ort` 用 `download-binaries`：构建时联网下载 ORT 二进制，CI/复现要钉版本与哈希；GPL-3.0 与 ORT(MIT) 兼容，没问题。
- `Instant::now()` 在 macOS 走 mach_absolute_time，实时线程里可以用。
- `SafetyConfig` 里有 `Option<Duration>` 等，参数以后要跨线程实时改（门控、下限），建议到 P2 拆成「构造期配置」和「运行期参数（原子/三缓冲）」两半，现在不用动。
