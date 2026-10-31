# 主题 2 · workspace 与 crate 划分（vdn）

> 2026-10-30 · rustarch 子调研 r2。只取「规则、做法、取舍」层；单项目细节归 rusteco。
> 证据等级：【官】官方明载 /【源】源码可证 /【测】本机实测 /【传】二手或论坛 /【推】本文推断。
> 本机克隆根：`R = /Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources`（下文写 `R/…:行号`）。

---

## 0. 结论先行

1. **扁平 `crates/` 目录 + 虚拟 manifest + 目录名等于 crate 名 + 内部 crate 版本写 `0.0.0` / `publish = false` + 自动化放进 `xtask`**。这是 matklad 与 rust-analyzer 的做法，Ruff、Bevy、Zed、Firewheel 也在用。
2. **拆 crate 的头号理由是「它不依赖谁」**，不是代码多少。matklad 原话："The most important property of a crate is which crates it doesn't (transitively) depend on."【官】
3. **核心无平台依赖由机器守住**：用 cargo-deny 的 `wrappers`（只有指定 crate 能直接依赖某个被禁 crate）加上 xtask 的依赖方向测试（读 `cargo metadata`），不靠人看。
4. **核心 crate 不强求 `no_std`**：RustFFT、rubato 本机版本都不是 `no_std`。只有共享内存协议 crate 值得做成 `no_std`：它很小、`#[repr(C)]`，还要给 C 的 HAL 插件出头文件。
5. **两条正交的轴各拆成 crate**：「模型」（GTCRN / FastEnhancer-M）和「推理后端」（ort / tract）。后端之间用独立 crate、运行时选，**不做互斥 feature**（Cargo book 明确建议这样）。
6. 人说的「超级语法树 / AST 做模块」：在 Rust 里，**crate/module 树本身就是那棵结构树**。AST 最值钱的用法是**守架构**（依赖方向、实时禁用 API、公共 API 快照），**不是**用过程宏生成模块。见 §7。

---

## 1. 外部做法：逐条带出处

### 1.1 matklad「Large Rust Workspaces」（2021-08-22）
URL: https://matklad.github.io/2021/08/22/large-rust-workspaces.html

| 主张 | 要点 | 等级 |
|---|---|---|
| 扁平布局 | 适用于约 1 万到 100 万行的项目：所有 crate 平放在 `crates/` 下一层。"tree structure tends to deteriorate over time, while flat structure doesn't need maintenance." | 【官】 |
| 为什么扁平 | Cargo 的 crate 命名空间本来就是扁平的。目录树等于第二套层级，会和 crate 名对不上。树形布局还容易长出空文件夹和 `utils` 杂物堆 | 【官】 |
| 虚拟 manifest | 根 `Cargo.toml` 只放 `[workspace]` 和 `members = ["crates/*"]`，不放 package。否则根目录会混进 `src/`，每条命令都得加 `--workspace` | 【官】 |
| 目录名等于 crate 名 | 不要因为有公共前缀就把目录名缩短 | 【官】 |
| 内部 crate 版本 `0.0.0` | 不发布的 crate 一律写 `0.0.0` | 【官】 |
| 要发布的放 `libs/` | 要发布的 crate 移到顶层 `libs/`，方便检查「`libs/` 不依赖 `crates/`」 | 【官】 |
| 自动化用 Rust 写 | 不用 Makefile 和 `prepare.sh`，改用 `cargo xtask` | 【官】 |
| 单文件 crate 也保留 `src/` | 它以后可能长大 | 【官】 |
| 何时拆 crate | **原文没给判据**，只说扁平布局下拆分很便宜 | 【官】（原文缺这一项） |

### 1.2 rust-analyzer `architecture.md`
URL: https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md
写法论：https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html

可以搬到本项目的几条主张：
- **核心不做 IO**："never does any IO"。输入只有内存里的文件和 CrateGraph【官】。→ 对应本项目：engine 只吃 `&[f32]` 帧，不认识 WAV、设备和文件。
- **API boundary 写明哪些 crate 是对外边界**：`syntax`「completely independent from the rest… knows nothing about salsa or LSP」；`base-db`「knows nothing about cargo」；`hir-*` 内部 crate「not, and will never be, an API boundary」【官】。→ 本项目也应写明谁是边界：`vdn-types` 与 `vdn-engine` 是，`vdn-dsp` 的内部细节不是。
- **只有最外层知道协议**：「only crate that knows about LSP and JSON serialization」【官】。→ 只有 `vdn-control` 和 bin 认识 serde、socket 和配置文件格式。
- **核心层的类型故意不可序列化**，需要 IPC 的一方自己定格式【官】。→ 共享内存布局单独做成协议 crate，不让 engine 的类型直接上线。
- **不可信的东西放独立进程**：proc macro 放在独立进程，因为 "Bad proc macros may panic or segfault"【官】。→ 和本项目「送音进程与推理进程分开、崩溃也送原声」同构。
- **所有不变量都用 `#[test]` 检查，不另设 CI 专用检查**【官】。rust-analyzer 的 `xtask/src/tidy.rs` 就是作为 test 跑的：检查 `Cargo.toml` 格式、内部 path 依赖带不带 version、许可证白名单（读 `cargo metadata`）、模块必须有 `//!` 文档等【源】https://raw.githubusercontent.com/rust-lang/rust-analyzer/master/xtask/src/tidy.rs
- **生成的代码提交进仓库，不自举**（codegen 用 syn 和字符串）【官】。
- ARCHITECTURE.md 写法：内容要短；画 codemap（「国家地图，不是各州地图集」）；**明写不变量，尤其是「某样东西不存在」的那类**；标出边界；一年回看两次就够【官】。

### 1.3 matklad「Fast Rust Builds」（2021-09-04）与编译时间
URL: https://matklad.github.io/2021/09/04/fast-rust-builds.html
- crate 依赖图的形状决定并行度：一条链只能串行编译；图越宽，越能并行，增量编译时兄弟 crate 也不受牵连【官】。
- 推荐的形状是「共享词汇 crate → 多个互不依赖的功能 crate → 一个叶子 crate 拼起来」【官】。
- **"do not put generic code at the boundaries between the crates"**：单态化在每个 crate 里各做一次，泛型跨 crate 可能让编译时间按平方增长。做法是泛型只包一层薄壳、内部转给非泛型实现，闭包参数用 `&dyn Fn` 而不是 `impl Fn`【官】。
- proc macro 会阻塞流水线：用到它的 crate 要等宏编译完才能开始。`syn` 编译很贵。**serde 之类的派生序列化放在系统边界的叶子 crate**【官】。
- 产物越少越好：每个二进制都要各自链接一遍，原文举了 BusyBox 式多调用二进制的例子【官】。
- CI 只缓存依赖，不缓存本项目自己的 crate【官】。
- 测试：`tests/*.rs` 每个文件都是一个单独的二进制，各要链接一次。matklad 建议合成一个 `tests/it/main.rs`，单元测试写成 `#[cfg(test)] mod tests;` 放到单独文件，这样只改测试时不重编库【官】https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html
- Rust 官方 2025 编译性能调查：受访者认为最有效的手段之一是 "splitting larger crates into smaller crates"；痛点是改一个 crate 后所有依赖它的 crate 都要重编，链接每次从零做，derive 宏展开不缓存。建议先试更快的链接器，用 `cargo build --timings` 找瓶颈【官】https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results
- 本机样本：DeepFilterNet 的 workspace 设 `[profile.dev] opt-level = 1`【源】`R/DeepFilterNet/Cargo.toml:12-13`。Zed 用 `[profile.dev.package]` 给个别重 crate 单独开 `opt-level = 3`【官】https://raw.githubusercontent.com/zed-industries/zed/main/Cargo.toml。→ 音频 DSP 用 debug 构建跑实时会太慢，本项目应给依赖和 dsp 开 dev 优化【推】。

### 1.4 Cargo book：workspace 继承与 feature
URL: https://doc.rust-lang.org/cargo/reference/workspaces.html ，https://doc.rust-lang.org/cargo/reference/features.html
- 虚拟 manifest 必须显式写 `resolver`，因为没有 `package.edition` 可以推断；在根目录跑命令默认作用于全部成员（`default-members` 可以改）【官】。
- `[workspace.package]` 可以继承 edition、license、rust-version、publish、version 等字段，子 crate 写 `xxx.workspace = true`【官】。
- `[workspace.dependencies]`：子 crate 写 `dep.workspace = true`。**这里声明的依赖不能 optional**；子 crate 另加的 features 和 workspace 的取并集【官】。
- `[workspace.lints]` 加子 crate 的 `[lints] workspace = true`，从 Rust 1.74 起生效【官】。**子 crate 不能在继承之后逐条覆盖**：Bevy 根 Cargo.toml 的注释写明了这一点，根包只好复制一份 lints【官】https://raw.githubusercontent.com/bevyengine/bevy/main/Cargo.toml。→ 实时 crate 需要更严的规则时，在 `lib.rs` 顶部用 `#![deny(...)]` 叠加【推】。
- **feature 必须可叠加**；互斥 feature "should be avoided if at all possible"。官方给的替代办法有三：拆成单独的包；冲突时定优先级；让多个实现能同时编进去、**运行时**再选【官】。→ ort 和 tract 做成两个 crate，运行时选。
- **想支持 no_std 时不要做 `no_std` feature，要做 `std` feature**（`#![no_std]` 加 `#[cfg(feature="std")] extern crate std`）【官】。
- feature 在依赖图里取并集；resolver 2 不把 dev-dependency、build-dependency 和不在当前编译目标上的依赖的 feature 并进来【官】。

### 1.5 Rust API Guidelines（只取跟结构有关的）
URL: https://rust-lang.github.io/api-guidelines/checklist.html
- C-FEATURE：feature 名不带占位词（不叫 `use-xxx` / `with-xxx`）【官】。
- C-SEND-SYNC：类型尽量 `Send + Sync`。实时侧的 `Engine` 至少要 `Send`，因为要移进音频线程【官】+【推】。
- C-STRUCT-PRIVATE：结构体字段私有。C-SEALED：用 sealed trait 挡住下游实现。→ 内部 crate 可以放宽；`vdn-types` 的 trait 计划让外部模型实现，所以**不** seal【推】。
- C-STABLE：稳定 crate 的公共依赖也必须稳定。→ 核心 crate 的公共 API 里不要露出 `ort::Session` 或 `cpal::Stream`【推】。
- 「C-RE-EXPORT」不在清单上，未找到出处。

### 1.6 大项目怎么切「核心」和「平台适配层」

| 项目 | 切法 | 出处 | 等级 |
|---|---|---|---|
| **wgpu** | `wgpu`（安全的公共 API）→ `wgpu-core`（校验、资源跟踪）→ `wgpu-hal`（trait 抽象各图形 API，"contracts are _unsafe_… minimal validation"；后端 metal/vulkan/dx12/gles；**静态分发**）；`wgpu-types` 是共享类型 | https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/wgpu-hal/README.md ，https://github.com/gfx-rs/wgpu | 【官】（types 无依赖这一点 README 没写，【推】） |
| **Zed / GPUI** | `Platform` trait 在 `crates/gpui/src/platform.rs`，是「GPUI 应用跟操作系统对话的唯一接口」。现在主仓 workspace 里有单独的 `crates/gpui_macos` / `gpui_linux` / `gpui_windows` / `gpui_platform` | https://zed.dev/blog/zed-decoded-linux-when ，https://raw.githubusercontent.com/zed-industries/zed/main/Cargo.toml | 【官】 |
| **Firewheel**（音频图引擎） | `firewheel-core` / `-graph` / `-nodes` 都是 `#![cfg_attr(not(feature="std"), no_std)]`；后端 `firewheel-cpal` 和 `firewheel-rtaudio` 是独立 crate，依赖 core 和 graph；伞 crate `firewheel` 用 feature 选后端 | `R/Firewheel/crates/firewheel-core/src/lib.rs:1`，`R/Firewheel/crates/firewheel-cpal/Cargo.toml:28-29`，`R/Firewheel/Cargo.toml:17,159-193` | 【源】 |
| **cpal** | 单个 crate，内部用 `HostTrait`/`DeviceTrait`/`StreamTrait` 抽象，各平台是 `src/host/<平台>` **模块**，用 `cfg(target_os)` 选 | `R/cpal/src/traits.rs:42,129,673`，`R/cpal/src/host.rs:48` | 【源】 |
| **nih-plug** | 核心 `Plugin` trait；插件格式的适配是同一个 crate 里的 `wrapper::{clap, vst3, standalone}` **模块**；GUI 适配（egui/iced/vizia）是独立 crate；有 `nih_plug_xtask` 和 cargo alias | `R/nih-plug/src/plugin.rs:59`，`R/nih-plug/src/wrapper.rs:4-11`，`R/nih-plug/Cargo.toml:15-20`，`R/nih-plug/.cargo/config.toml:7` | 【源】 |
| **Symphonia** | `symphonia-core` 加上每个 codec 一个 crate（`symphonia-codec-*`）、每个容器格式一个 crate（`symphonia-format-*`），伞 crate `symphonia` 用 feature 开关；"100% safe Rust" | https://github.com/pdeljanov/Symphonia | 【官】（伞 crate 具体 re-export 什么没写） |
| **Bevy** | `crates/*` 扁平平放（glob）；伞 crate `bevy` 依赖 `bevy_internal` 并转发 feature；`dynamic_linking` feature 用来缩短迭代编译时间；workspace lints 设 `unsafe_code = "deny"`、`alloc_instead_of_core = "warn"`；0.16 起引擎和大量子 crate 不再依赖 std | Bevy Cargo.toml（同上 URL），https://bevy.org/news/bevy-0-16/ | 【官】 |
| **Ruff** | `crates/` 扁平平放，`ruff_` 前缀；测试、benchmark、开发用 crate 一律 `publish = false`；`ruff_dev` 是开发工具 bin（相当于 xtask）；`ruff_wasm` 是一个适配器 crate | https://github.com/astral-sh/ruff/blob/main/CONTRIBUTING.md | 【官】 |
| **tokio** | 成员有 `tokio`、`tokio-macros`、`tokio-test`、`tokio-stream`、`tokio-util`，另有 internal 的 benches/examples/stress-test/tests-build/tests-integration；workspace lints 用 `check-cfg` 声明自定义 cfg | https://raw.githubusercontent.com/tokio-rs/tokio/master/Cargo.toml | 【官】（「平台层放在仓外的 mio」：【传】，未核实） |
| **DeepFilterNet libDF**（本机同领域） | **一个 crate 塞满所有东西**：dataset、tract、capi、wasm 全靠 feature 拼（`#[cfg(feature=…)] mod`）；`crate-type = ["cdylib","rlib","staticlib"]`；用 cbindgen 出 C 头文件 | `R/DeepFilterNet/libDF/src/lib.rs:17-37`，`R/DeepFilterNet/libDF/Cargo.toml:15`，`R/DeepFilterNet/cbindgen.toml:8` | 【源】 |
| **Servo** | 未核实（没取到出处） | 未找到出处 | — |

**从这些项目归纳出的规律**【推】：
- 平台适配**拆成单独 crate** 还是**只做模块加 cfg**，取决于下游会不会只想要其中一部分。cpal 和 nih-plug 选模块，因为用户总是整包拿。wgpu、Zed、Firewheel 选 crate，因为要让核心在没有该平台时也能单独编译，还要隔离 unsafe 和重依赖。
- 本项目的要求是「eval 不能间接依赖 CoreAudio」，所以**适配层必须是 crate**，模块做不到。
- DeepFilterNet 的「一个 crate 加一堆 feature」是反面例子：tract、hdf5、clap 全靠 feature 门控，依赖边界要靠人记住哪个 feature 开了，机器没法守。

### 1.7 「核心 crate 无平台依赖、可 no_std」
- 官方模式：`std` feature，见 1.4【官】。Firewheel 和 rtrb 用的就是它【源】`R/Firewheel/crates/firewheel-core/src/lib.rs:1`、`R/rtrb/src/lib.rs:62`；biquad 直接 `#![no_std]`【源】`R/biquad-rs/src/lib.rs:51`。
- **本项目依赖的 no_std 情况**【源】（在本机克隆里 grep `no_std`）：biquad ✅；rtrb ✅（`std` feature）；dasp 部分支持；**RustFFT ✗、rubato ✗**（lib.rs 里查不到 `no_std`）；ort / tract 天然需要 std【推】。
- 推论【推】：`vdn-dsp` 要用 realfft/RustFFT 做 STFT，强行 `no_std` 就得换 FFT 库，还要自己补 `libm`，代价大于收益。**「无平台依赖」不等于「no_std」**。前者用 cargo-deny 和 xtask 守住，就已经满足命门 1（核心依赖树里没有 cpal/objc2/CoreAudio）。no_std 只在两个地方采用：`vdn-shm`（协议本体只用 `core::sync::atomic`），以及可以选做的 `vdn-types`。
- no_std 的真正好处是**编译器直接挡掉 `std::sync::Mutex`、`std::thread`、`std::fs`、`println!`**，这是一种免费的「实时禁用 API」检查。但它挡不住分配（`alloc::vec::Vec` 照样能用）。分配归 assert_no_alloc / RTSan 那条线管（另一主题）【推】。

### 1.8 xtask
URL: https://github.com/matklad/cargo-xtask
- `.cargo/config.toml` 里写 `[alias] xtask = "run --package xtask --"`，`xtask/` 是 workspace 里的一个 bin crate，连同 config 一起提交【官】。
- 好处：除了 cargo 和 rustc 不需要装别的；比 bash 容易跨平台【官】。
- 「advisable to minimize the compile time of xtasks」；放进 workspace 可以共享依赖，放在 workspace 外可以单独缓存、单独选 feature【官】。
- 先例：rust-analyzer 的 `xtask/src/{codegen,dist,tidy,metrics,pgo,…}.rs`【源】；nih-plug 的 `nih_plug_xtask` 用来打插件 bundle【源】`R/nih-plug/.cargo/config.toml:7`；Ruff 用 `ruff_dev`（`cargo dev generate-all`）【官】。

### 1.9 Rust 大会演讲
- 按「workspace / crate 划分」搜 RustConf、Rust Nation、EuroRust，**没找到专讲这个题目的演讲**（未找到出处）。社区的权威材料就是上面 matklad 的系列文章和官方的编译性能调查。
- 音频方向：ADC Japan 2026 有一场 Rust 插件架构的演讲（NovoNotes，CLAP First 加 Rust DSP），只看到日程页，没有幻灯片【传】https://adcjapan26.sched.com/event/2KCep/c++ 。对 crate 划分没有可用的具体主张。

---

## 2. 本项目的 crate 列表初稿

### 2.1 命名与布局
- 根目录是虚拟 manifest，`members = ["crates/*"]`，`resolver = "3"`（edition 2024）或 `"2"`。版本、edition、license(GPL-3.0)、`publish = false` 走 `[workspace.package]`【官 1.4】。
- **crate 名加 `vdn-` 前缀，目录名和 crate 名一样**（`crates/vdn-dsp/`）。理由：matklad 要求目录名等于 crate 名【官】；Ruff 和 Bevy 都加项目前缀【官】；不加前缀时 `dsp`、`engine` 这类名字在 `use` 和报错里难以区分【推】。
- `version = "0.0.0"`（matklad），`publish = false`（Ruff 规约）【官】。
- C 写的 HAL 插件**不在 Rust workspace 里**（`must-know.md` 已定）。它放在仓库顶层的 `hal-plugin/`（这是新顶层目录，按项目铁律 10 要人拍板），只吃 `vdn-shm` 用 cbindgen 生成的头文件。

### 2.2 清单

`实时级`的含义：RT = 代码在音频或实时降噪线程上跑，适用最严的 lint 和不分配检查；NRT = 非实时；Tool = 工具。

| crate | 层 | 实时级 | 职责 | 主要外部依赖 | 不许依赖 |
|---|---|---|---|---|---|
| `vdn-types` | 核心·词汇 | RT | 帧和格式类型（`SampleRate`、`FrameSpec`、`latency_samples`）；trait `Denoiser`、`SpeakerEmbedder`、`FeatureSink`；错误类型；无锁发布槽的**接口**；可选 `no_std` | 几乎没有 | 所有平台库、ort/tract、serde |
| `vdn-dsp` | 核心 | RT | STFT/iSTFT、biquad EQ、压缩和限幅、去混响尾、门控增益平滑、交叉淡化、原声旁路的淡入淡出；全部在构造时预分配，`process(&mut self, …)` | realfft、biquad | 同上 |
| `vdn-models` | 核心 | RT | 每个模型一个文件：GTCRN 和 FastEnhancer-M 的 `ModelSpec`（帧长、hop、状态张量名和形状、IO 名、延迟）和与后端无关的前后处理；`registry`。**加一个模型就是加一个文件、注册一行**（命门 3） | 只依赖 `vdn-types`、`vdn-dsp` | 后端、平台 |
| `vdn-engine` | 核心 | RT | 逐帧管线：输入帧 → `dyn Denoiser` → 门控 → 音色链 → 输出；接收声纹结果（读原子槽）；超时或出错时切原声的状态机；**eval 和实时进程的唯一处理入口** | `vdn-types`、`vdn-dsp`、`vdn-models` | 后端（只认 trait）、平台、serde |
| `vdn-speaker` | 核心 | NRT | 注册（enroll）、fbank、余弦打分、保守门控决策，每 0.5 s 一次；通过 `SpeakerEmbedder` trait 调模型；结果写进原子槽 | `vdn-types`（fbank 库待定） | 平台、后端 |
| `vdn-infer-ort` | 适配·推理 | RT* | 用 ort 实现 `Denoiser` 和 `SpeakerEmbedder`；会话和张量缓冲都在构造时备好 | ort | 平台、engine |
| `vdn-infer-tract` | 适配·推理 | RT* | 用 tract 做同样的实现 | tract-onnx | 平台、engine |
| `vdn-shm` | 协议 | RT | 共享内存布局：`#[repr(C)]` 头（magic、版本、采样率）、SPSC 环形缓冲、心跳和序号；`#![no_std]`；`posix` feature 提供 `shm_open`/`mmap` 映射；cbindgen 生成 `vdn_shm.h` 并提交进仓库 | 核心部分无依赖；`posix` 用 libc/rustix | 引擎、后端 |
| `vdn-io-macos` | 适配·平台 | RT | cpal 采集、coreaudio-rs 读设备属性、audio_thread_priority、rtrb、rubato 漂移补偿、原声旁路；**唯一允许出现 CoreAudio、objc 的 crate** | cpal、coreaudio-rs、rtrb、rubato、audio_thread_priority | engine、后端 |
| `vdn-control` | 边界·NRT | NRT | 配置 schema、预设、控制消息（CLI/UI ↔ 进程）、参数存储（原子量）；**serde 只允许出现在这里和 bin 里** | serde、toml | 平台、后端 |
| `vdn-eval` | 边界·离线 | NRT | 读 WAV，逐帧喂给 `vdn-engine`；结果、哈希、参数落盘；bin `vdn-eval` | hound、`vdn-engine`、两个后端、`vdn-control` | **`vdn-io-macos`、`vdn-shm` 的 posix 部分** |
| `vdn-capture`（bin，送音进程） | 应用 | — | 把 io-macos、shm、control 拼起来；推理进程死掉时送原声 | — | **ort、tract、`vdn-engine`**（送音进程不链推理库，崩溃隔离有机器保证） |
| `vdn-infer`（bin，推理进程） | 应用 | — | 把 shm、engine、后端、speaker、control 拼起来；起实时线程和声纹线程 | — | `vdn-io-macos` |
| `vdn-testkit` | 开发 | Tool | 测试夹具：合成信号、golden 比对、带分配计数的全局分配器包装；只出现在 dev-dependencies | — | 不能出现在任何 `[dependencies]` 里 |
| `xtask` | 工具 | Tool | 见 §5 | 尽量少（matklad） | — |

\* 后端在实时线程上被调用，但推理库内部会不会分配要靠实测（`ort` 的 `run` 很可能分配【推】），这归实时安全那条线。

### 2.3 依赖方向图

```
                         ┌──────────────┐
                         │  vdn-types   │  词汇层：trait + 帧类型（可选 no_std）
                         └──────┬───────┘
          ┌──────────────┬──────┼───────────┬───────────────┬─────────────────┐
          ▼              ▼      │           ▼               ▼                 ▼
     ┌─────────┐   ┌──────────┐ │    ┌──────────────┐ ┌───────────────┐ ┌────────────┐
     │ vdn-dsp │   │vdn-speaker│ │    │vdn-infer-ort │ │vdn-infer-tract│ │vdn-control │
     └────┬────┘   └────┬─────┘ │    └──────┬───────┘ └──────┬────────┘ └─────┬──────┘
          ▼             │       │           │ (只认 trait，     │                │
     ┌──────────┐       │       │           │  不依赖 engine)   │                │
     │vdn-models│◄──────┼───────┘           │                  │                │
     └────┬─────┘       │                   │                  │                │
          ▼             │                   │                  │                │
     ┌──────────┐       │                   │                  │                │
     │vdn-engine│       │                   │                  │                │
     └────┬─────┘       │                   │                  │                │
          │             │                   │                  │                │
   ═══════╪═════════════╪═══════════════════╪══════════════════╪════════════════╪═══ 核心/适配分界
          │             │                   │                  │                │
   ┌──────┴──────┬──────┴───────────────────┴──────────────────┴─────┐          │
   ▼             ▼                                                  ▼          │
┌────────┐  ┌──────────┐ ◄──── vdn-shm (no_std 协议) ────► ┌────────────┐      │
│vdn-eval│  │vdn-infer │      ▲                       │    │vdn-capture │◄─────┘
│ (bin)  │  │  (bin)   │      │                       └───►│   (bin)    │
└────────┘  └──────────┘      │ cbindgen 生成 .h            └─────┬──────┘
                              ▼                                   ▼
                     hal-plugin/ (C, 仓内但不在 workspace)  ┌──────────────┐
                                                           │ vdn-io-macos │ 唯一碰 CoreAudio
                                                           └──────────────┘
```

图的意思：
- **核心 = `vdn-types · vdn-dsp · vdn-models · vdn-engine · vdn-speaker`**，eval 和实时共用；依赖树里不出现 cpal、coreaudio、objc2、ort、tract、serde。
- 两个后端是**兄弟**，互不依赖，也不依赖 engine，可以并行编译（Fast Rust Builds 的「宽图」）。engine 编译时完全不知道 ort 存在，只拿 `Box<dyn Denoiser>`。
- 只有 bin（`vdn-eval`、`vdn-infer`）知道「用哪个后端」，按配置在运行时选（Cargo book 的「运行时选」建议）。

### 2.4 设计稿七个 crate 逐个怎么处理

| 设计稿 | 处理 | 理由 |
|---|---|---|
| `dsp` | **保留**，改名 `vdn-dsp` | 依赖边界干净（realfft、biquad），引擎和 eval 都用，RT 级，单元测试最多 |
| `engine` | **保留，并拆出 `vdn-types` 和 `vdn-models`** | ① 后端 crate 需要 trait 但不该依赖整个 engine，否则 engine → 后端 → engine 形成环，或者编译成一条长链（Fast Rust Builds：词汇 crate 在最底层）；② 模型规格单独成 crate，「加模型不改 engine」就能由机器检查（没改 `vdn-engine` 目录） |
| `speaker` | **保留** | 实时级和 engine 不同：NRT，允许分配，lint 策略不同；它的运行节奏（0.5 s）和输入（fbank）也和 engine 不同。和 engine 只经过 `vdn-types` 里的发布槽接口接触 |
| `io-macos` | **保留** | 平台和 unsafe 集中在这一处（wgpu-hal 的做法）；命门 1 要求核心依赖树里没有它，只有拆成 crate 才能让 cargo-deny 守住 |
| `hal-plugin` | **移出 Rust workspace**（C），workspace 里换成 `vdn-shm` | 003 已定 C 实现；Rust 侧真正要共享的是**内存布局协议**，不是插件代码；布局用 `#[repr(C)]` 加 cbindgen（本机先例 `R/DeepFilterNet/cbindgen.toml`） |
| `control` | **保留，但限定为「边界层」** | serde 和 IO 集中在叶子（Fast Rust Builds："keep serialization at the boundary"）；核心 crate 不能依赖它；核心的参数用普通结构体，由 control 转换后传入 |
| `eval` | **保留**（lib + bin） | 用来证明「同一份处理代码」：它和 `vdn-infer` 调同一个 `vdn-engine::Engine::process`；不能依赖 `vdn-io-macos`，否则核心对平台的独立性就没有可执行的证据了 |
| （新）推理后端 | `vdn-infer-ort`、`vdn-infer-tract` 两个 crate | 互斥 feature 是 Cargo 官方不推荐的做法；两个 crate 可以同时编进去，速度和功耗对比能在同一个 eval 里跑；ort 的构建（可能要下载预编译库【传】）不会拖累核心 crate 的增量编译 |
| （新）`vdn-shm` | 协议 crate | 见上 |
| （新）两个进程各一个 bin crate | `vdn-capture`、`vdn-infer` | Fast Rust Builds 主张产物越少越好，倾向做成一个 BusyBox 式的 bin；**这里不采纳**，因为分成两个 bin 后，「送音进程依赖树里没有 ort/tract/engine」可以用 cargo-deny `wrappers` 由机器证明（D03 崩溃隔离）。代价是多链接一个二进制，可以接受【推】 |
| （新）`xtask`、`vdn-testkit` | 工具 | 见 §5 |

---

## 3. 何时拆 crate、何时只做 module（判据）

满足**任一条**「拆」的条件就拆 crate；一条都不满足就保持 module。

**拆（crate）：**
1. **要机器守的依赖边界**：它不能看见某个依赖，或者某个依赖不能看见它（matklad："which crates it doesn't depend on"【官】）。本项目最主要的拆分理由就是这一条。
2. **实时级不同**：RT 和 NRT 要用不同的 lint 和检查强度。Cargo 不能在一个 crate 里按模块分 lint 级别（最多用 `#[deny]` 属性叠加），以 crate 为单位最清楚【官 Bevy 注释】+【推】。
3. **同一接口的可替换实现**：后端、平台。互斥 feature 不推荐，那就拆成单独的包【官 Cargo features】。
4. **产物形态或使用方语言不同**：bin、cdylib/staticlib、要给 C 出头文件、要 `no_std`。
5. **重依赖隔离**：ort、tract、serde/syn、hound 这类会拖编译或传染 feature 的依赖，放到叶子 crate【官 Fast Rust Builds】。
6. **unsafe 集中**：其他 crate 都 `#![forbid(unsafe_code)]`，只有 `vdn-io-macos`、`vdn-shm`（可能还有后端）放开（wgpu-hal、Bevy `unsafe_code = "deny"` 的做法）【官】+【推】。

**保持 module：**
- 只有一个使用方，而且没有上面任何一条理由。
- 和兄弟代码共用私有不变量，需要 `pub(crate)`；一旦拆开，就得把内部细节公开。
- 主要是泛型代码，拆开后泛型会落在 crate 边界上（"do not put generic code at the boundaries"【官】）。
- 平台差异小、用户总是整包拿：用 `cfg` 模块就行（cpal、nih-plug 的做法【源】）。

**反面信号（别这么拆）**【官 matklad】+【推】：
- 建 `utils` / `common` 这种杂物 crate。
- 嵌套目录树。
- 一个文件一个 crate。
- 只因为文件长就拆。代码量不是判据：matklad 只给了扁平布局适用的规模（1 万到 100 万行），没有给单个 crate 的行数上限，这一点未找到出处。

---

## 4. Cargo.toml 骨架要点（只列决策，不贴全文）

- 根目录：`[workspace] members=["crates/*"]` 加上 `resolver`；`[workspace.package]` 设 edition、license="GPL-3.0-only"（具体写法待人定）、`publish=false`、`version="0.0.0"`。
- `[workspace.dependencies]`：**内部 crate 也在这里登记 path**（Zed 的做法【官】），子 crate 一律写 `x.workspace = true`。注意这里的依赖**不能 optional**【官】，要做可选依赖就在子 crate 里写 `optional = true, workspace = true`【推，需实测】。
- `[workspace.lints.rust]`：`unsafe_code = "deny"`，`unsafe_op_in_unsafe_fn = "warn"`，`missing_docs`（核心 crate 打开）；`unexpected_cfgs` 用 check-cfg 声明（tokio 的做法【官】）。
- `[workspace.lints.clippy]`：Zed 用 `dbg_macro`、`todo` 设 deny【官】；RT crate 在 `lib.rs` 里另外叠加 `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]`，配合 `clippy.toml` 的 `disallowed_types`/`disallowed_methods`（禁 `std::sync::Mutex`、`Vec::push` 等）。lint 名来自 clippy 官方文档，**具体组合归实时安全那条线定**【推】。
- `[profile.dev]`：给依赖和 `vdn-dsp`/`vdn-engine` 单独开 `opt-level`（Zed 用 `profile.dev.package`、DeepFilterNet 用 `opt-level = 1` 都是先例），否则 debug 构建跑实时会超时【推】。
- `[profile.release]`：`panic` 的策略（abort 还是 unwind）关系到「出事送原声」，**待人定**：abort 会直接让推理进程退出，由送音进程兜底；unwind 才能在线程里 catch。DeepFilterNet 的 `release-lto` 用 abort【源】`R/DeepFilterNet/Cargo.toml`。

---

## 5. xtask 职责（P1）

| 子命令 | 做什么 | 依据 |
|---|---|---|
| `xtask check-deps` | 读 `cargo metadata`，断言依赖方向表（§2.2「不许依赖」列）；同时以 `#[test]` 形式在 `cargo test` 里跑 | rust-analyzer tidy 读 cargo metadata，"All invariants are checked via `#[test]`"【官】【源】 |
| `deny.toml`（cargo-deny） | `bans.deny`：`coreaudio-sys`、`cpal`、`objc2` 加 `wrappers = ["vdn-io-macos"]`；`ort` 加 `wrappers = ["vdn-infer-ort"]`；`serde` 加 `wrappers = ["vdn-control", bin…]`；`multiple-versions` 设 warn；同时管许可证（GPL 兼容） | cargo-deny："This field allows specific crates to have a direct dependency on the banned crate"【官】https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html 。`wrappers` 只管**直接**依赖，传递依赖仍要靠 check-deps 补【推】 |
| `xtask codegen` | 跑 cbindgen 生成 `vdn_shm.h` 并提交进仓库；测试里检查已提交的头文件和重新生成的一致 | rust-analyzer「generated code is committed」【官】 |
| `xtask models` | 按清单下载或校验 ONNX 的 sha256；模型不进仓库 | 项目铁律 8 |
| `xtask hal` | 调 clang/xcodebuild 构建 C 的 HAL 插件，把 `.driver` 打包 | nih-plug 的 xtask 打 bundle【源】 |
| `xtask rt-check` | 跑实时安全测试套件（assert_no_alloc、RTSan） | 归另一主题 |
| `xtask api` | `cargo public-api` 给 `vdn-types`、`vdn-engine` 的公共 API 拍快照，API 一变测试就红（需要 nightly） | 【官】https://github.com/cargo-public-api/cargo-public-api |

xtask 本身尽量少依赖（matklad【官】）；`.cargo/config.toml` 加 alias【官】。

---

## 6. 测试与编译时间安排

- 每个 crate 的单元测试写成 `#[cfg(test)] mod tests;` 放到单独文件；整个 workspace 只开**一个**集成测试二进制，`crates/vdn-engine/tests/it/main.rs`（或单独的 `vdn-it` crate）【官 matklad】。
- 测试只测「系统边界」：`vdn-engine::process` 的帧进帧出（golden WAV），以及 `vdn-eval` 的端到端。和 rust-analyzer 一样，测试由数据驱动，不直接测内部 API【官】+【推】。
- CI 只缓存依赖；先用 `cargo build --timings` 看瓶颈；macOS 默认用的链接器已经不慢，Linux CI 用 lld【官 2025 调查】+【推】。
- 引擎和后端的边界用 `Box<dyn Denoiser>`：每帧一次虚调用，相比 GTCRN 推理开销可以忽略（预判还没实测），同时避免泛型落在 crate 边界上【官 Fast Rust Builds】+【推】。

---

## 7. 人说的「超级语法树 / AST 做模块化」：怎么认真接住

**怎么理解这句话**【推】：人想要的是「结构清楚、能拆能换、模块边界不糊」。在 Rust 里，这件事主要由**语言自带的两层树**完成：crate 依赖图（DAG）和每个 crate 内部的 module 树。**它们本身就是可以被机器读取的结构**，用不着另造一棵「超级语法树」来组织代码。

AST 和语法树在本项目里**值得用**的地方，按性价比排：
1. **守架构，判对错**：
   - crate 层：`cargo metadata` 读依赖图，配合 check-deps 和 cargo-deny `wrappers`（§5）。
   - module 层：`cargo modules structure` 打印模块树，`cargo modules dependencies --acyclic` 发现模块间的环就报错【官】https://github.com/regexident/cargo-modules 。
   - 语法层：clippy `disallowed_*`；需要时用 syn 或 ast-grep 写 xtask 规则，比如「RT crate 里不出现 `Box::new`/`format!`」。rust-analyzer 的 tidy 就是这么做的（逐文件扫描，作为 test 跑）【源】。
   - API 层：`cargo public-api` 读 rustdoc JSON，给公共 API 拍快照【官】。
2. **让结构可见**：`cargo modules structure` 的输出，加上 ARCHITECTURE.md 的 codemap（matklad 写法），给人和 AI 一张地图【官】。
3. **生成代码**（过程宏或 codegen）：**P1 不用**。用过程宏批量生成模型适配器得不偿失：模型只有 2–3 个，宏会阻塞编译流水线，`syn` 编译很贵，derive 展开不缓存【官 Fast Rust Builds、2025 调查】，还会让代码不好读。真有大段重复，就用 xtask codegen 生成普通 `.rs` 文件并提交进仓库（rust-analyzer 的做法【官】）。

这样落地既不否掉人的想法，也不硬套：**「拆成模块和核心包」由 §2 的 crate 列表实现，「AST」用来保证这些边界永远不被破坏。**

---

## 8. 待人拍板（只列，不替人定）

1. crate 前缀用 `vdn-`，还是不加前缀？
2. 新顶层目录 `hal-plugin/`（C），按铁律 10 需要人拍板。
3. release 构建的 `panic = "abort"` 还是 `"unwind"`，关系到出事时由哪一层送原声。
4. 两个进程分两个 bin（本文建议，换来机器可证的隔离），还是做成一个多调用 bin（编译和链接更省）？

## 9. 本文没有核实或存疑的地方
- Servo 的切法：没取到出处。
- 「tokio 的平台层在 mio」：【传】。
- 「ort 构建会下载预编译库」：【传】，归 rusteco 核查。
- `optional` 和 `workspace = true` 能否组合使用：【推】，需要实测。
- 专讲 workspace 划分的大会演讲：未找到出处。
