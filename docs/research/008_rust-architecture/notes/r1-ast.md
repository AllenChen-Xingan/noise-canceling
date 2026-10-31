# R1 · 「超级语法树 / AST 做模块化」在 Rust 里能指什么——逐种评估

> 调研线：research/008-rust-architecture · 主题 1 · 2026-10-30
> 证据等级：【官】官方明载 /【源】源码可证 /【测】本机实测 /【传】二手或论坛 /【推】本笔记推断
> 本机克隆根目录简写为 `R/` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`
> 范围：只取「规则、做法、取舍」层；单项目实现细节归 rusteco 的生态核查表。

---

## 0. 先把人的话拆开（不否掉，也不硬套）

人原话：「我建议用超级语法树 / ast 来把东西做成模块，或者大部分东西做成可拆分以及的和新包和模块」。

在 Rust 里，「用 AST 做模块化」能落到三件**不同**的事上，代价和收益差别很大：

| 含义 | 一句话 | 本笔记章节 |
|---|---|---|
| A. 用语言自身的结构单元拆分 | crate / module / trait / 可见性 / feature 本身就是编译器在 AST/HIR 上强制的边界 | §1 |
| B. 用 AST / 静态分析**守住**架构 | 写规则，让机器在语法树或类型信息上检查「谁不许依赖谁」「实时函数里不许出现什么」 | §2 |
| C. 用 AST / 宏**生成**代码 | derive / macro_rules / build.rs / bindgen 从一份描述生成重复代码 | §3 |
| D. 外语言思想 | Lisp/Elixir 宏、Unison 内容寻址、Zig comptime、ArchUnit 等 | §4 |

结论先行【推】：人的直觉（「大部分东西做成可拆分的核心包和模块」+「用语法树管住它」）在 Rust 里**最划算的落点是 A + B**——用 crate 图做物理拆分，用 AST 级检查（clippy 配置、cargo-deny、xtask tidy、ast-grep）把架构不变量变成 CI 里的红灯；**C（宏生成）要克制**，只在重复真正出现、且手写会出错时才上。这与 CLAUDE.md「能用规则判的，绝不留给人判」完全同向。

---

## 1. (a) Rust 自身的结构单元

### 1.1 crate：唯一的「物理」边界

- **解决什么**：编译单元 + 依赖方向的硬边界。crate 之间不能循环依赖（Cargo 拒绝循环），所以「engine 不许依赖 io-macos」只要 `Cargo.toml` 里不写就**结构上不可能**发生。【官】Cargo 依赖图为 DAG（Cargo Book：https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html ；循环依赖报错为 Cargo 行为，具体章节未找到单独条目，按【传】记）
- **社区做法（matklad，rust-analyzer 前负责人）**：
  - 大型 workspace 用**扁平** `crates/` 一层目录、根 `Cargo.toml` 用 virtual manifest、crate 名 = 目录名；理由是 Cargo 命名空间本来就扁平，树形目录会造出第二套层级并随时间腐化。【传】https://matklad.github.io/2021/08/22/large-rust-workspaces.html （作者本人博客，属一手观点，但非官方规范）
  - crate 图的**形状**决定编译并行度：线性链只能顺序编；推荐「一个共享词汇 crate + 若干互不依赖的功能 crate + 一个把它们接起来的叶子 crate」。【传】https://matklad.github.io/2021/09/04/fast-rust-builds.html
  - serde/序列化等重依赖放**叶子** crate，别放在地基 crate，否则每个上层 crate 都要等 syn/serde_derive 编完。【传】同上
- **代价**：crate 多了 → `Cargo.toml` 多、版本/feature 要统一（可用 `[workspace.dependencies]`、`[workspace.lints]` 收口【官】https://doc.rust-lang.org/cargo/reference/workspaces.html ）；跨 crate 内联依赖 `#[inline]` 或 LTO【推】，对每帧热路径要留意。
- **成熟项目**：rust-analyzer 架构文档把「`syntax` crate 不依赖 rust-analyzer 其余部分」写成 **Architecture Invariant**【官】https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md （镜像：https://rust.googlesource.com/rust/+show/f095de4bf105e92b035a7cd64c34bcce56b7cb78/src/tools/rust-analyzer/docs/dev/architecture.md ）。matklad 另文强调「很多不变量是以**缺席**表达的」（例：model 层不依赖 view 层），读代码看不出来，必须写下来。【传】https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html
- **本项目建议：用，作为主拆分手段。** 见 §5 的依赖方向图。

### 1.2 module + 可见性（`pub(crate)` / `pub(super)` / `pub(in path)`）

- **解决什么**：crate 内部的二级边界；`pub(crate)` 让 API 只在 crate 内可见，对外 API 面最小化。【官】Rust Reference「Visibility and privacy」https://doc.rust-lang.org/reference/visibility-and-privacy.html
- **代价**：几乎为零；但它是**软**边界——同 crate 内任何模块都能 `use crate::x`，挡不住「实时模块调用了非实时模块」。
- **关键用法**：把 lint 作用域绑在 module 上。本机实证：tympan-aspl（Rust 写的 macOS AudioServerPlugIn 框架）在 `src/realtime/mod.rs:45` 写 `#![deny(clippy::disallowed_methods, clippy::disallowed_types)]`，只让 `realtime` 子树受禁用清单约束，其余初始化代码可以自由分配。【源】`R/tympan-aspl/src/realtime/mod.rs:40-45`，`R/tympan-aspl/clippy.toml:1-24`
- **本项目建议：用。** 每个含实时路径的 crate 内设 `rt` 模块（或独立 crate），模块根 `#![deny(...)]`。

### 1.3 trait：可替换实现的接口边界

- **解决什么**：GTCRN / FastEnhancer-M 同接口可切换；ort / tract 后端可切换；engine 对「实时输入/输出」与「离线 WAV」无感（eval 与实时同一份代码的关键）。
- **取舍**【推】：
  - 实时热路径上优先**静态分发**（泛型 / enum 分发），避免 `dyn Trait` 的间接调用与 `Box<dyn>` 的分配诱惑；切模型是非实时事件，可以在控制线程构造好新实例再原子交换。
  - trait 签名本身就是约束：`fn process(&mut self, input: &[f32], output: &mut [f32])` 不给 `Vec`、不返回 `Result<_, Box<dyn Error>>`，类型层就挡掉一半分配。
  - trait 不能表达「这个实现不分配/不阻塞」——Rust 目前没有 effect 系统，相关 rustc RTSan 集成还在 PR 阶段（见 §2.6），所以这一层要靠 §2 的检查补。
- **本项目建议：用。** `Denoiser`（模型）、`InferBackend`（ort/tract）、`Stage`（DSP 级）三个 trait；具体在 engine 设计稿里定。

### 1.4 Cargo feature：编译期开关

- **解决什么**：ort vs tract 后端、开发期 BlackHole vs 自写 HAL、检查用的 `assert_no_alloc` / `rtsan` 只在测试/CI 打开。nih-plug 就用 `assert_process_allocs = ["dep:assert_no_alloc"]` 这种可选 feature 把分配断言做成开关。【源】`R/nih-plug/Cargo.toml:56,100-101`
- **代价**：feature 是**加性**的（unification），两个互斥后端放成 feature 会在 workspace 里被同时打开【官】Cargo Book「Features · Mutually exclusive features」https://doc.rust-lang.org/cargo/reference/features.html#mutually-exclusive-features 。组合爆炸会让 CI 矩阵变大。
- **本项目建议：克制使用。** 互斥后端用**不同 crate**（`vdn-backend-ort` / `vdn-backend-tract`）+ trait，而不是一个 crate 里两个互斥 feature；feature 只留给「加性」的检查开关（`rt-check`）。

---

## 2. (b) 用 AST / 静态分析**守住**架构

> 核心认识【推】：Rust 社区守架构的主流做法**不是**写一个大而全的 AST 框架，而是**叠几层便宜的检查**，每层只管自己看得见的部分：
> 依赖图层（cargo-deny）→ 名字/路径层（clippy disallowed_*）→ 语法模式层（ast-grep / xtask tidy）→ 类型/HIR 层（dylint）→ 运行时层（assert_no_alloc / RTSan）→ 链接层（no_panic）。

### 2.1 clippy `disallowed-methods` / `disallowed-types` / `disallowed-macros`（clippy.toml）

- **是什么**：在 `clippy.toml` 列出完全限定路径，每项可带 `path` / `reason` / `replacement` / `allow-invalid`；分别驱动 `disallowed_methods`、`disallowed_types`、`disallowed_macros` 三条 lint。默认空列表。【官】https://doc.rust-lang.org/clippy/lint_configuration.html
- **解决什么**：「实时路径里不许出现 `Mutex` / `thread::sleep` / `File::open` / `mpsc::channel` / `println!`」这类**名字可枚举**的禁令。
- **成熟项目**：
  - tympan-aspl 的 `clippy.toml` 正是为实时回调写的禁用清单：`std::sync::Mutex/RwLock/Condvar/Barrier`、`parking_lot::*`、`std::thread::sleep/park`、`std::fs::*`、`std::io::stdout/stderr`、`std::sync::mpsc::channel/sync_channel`，并在 `realtime` 模块根 `#![deny]`；CI 跑 `cargo clippy --workspace --all-targets --all-features -- -D warnings`。【源】`R/tympan-aspl/clippy.toml:26-66`，`R/tympan-aspl/.github/workflows/tier1.yml:54-55`
  - Zed 用 disallowed-methods 带 `reason` + `replacement`（例：禁 `std::process::Command::spawn`）。【传】https://git.secluded.site/zed/blob/linux-ci/clippy.toml （镜像）
  - 注意：tympan 注释说 `println!` 不能按路径列（`R/tympan-aspl/clippy.toml:58-60`），但官方配置有 `disallowed-macros`，应可列 `std::println` 等。【推】需本机实测确认
- **代价 / 局限**：
  - 只看**本 crate 源码里的调用点**：你调了一个第三方函数，它内部加锁，clippy 看不到。【推】（lint 在本 crate 的 HIR 上跑，依赖已编译成元数据，这是 clippy 工作方式的直接推论）
  - 构造函数类（`Vec::new`、`Box::new`）不能一刀切禁，因为实时模块自己也要在初始化阶段预分配——tympan 也因此没禁它们，改由运行时 `assert_no_alloc` 兜底。【源】`R/tympan-aspl/clippy.toml:16-24`
  - 极便宜：零依赖，CI 已经在跑 clippy。
- **配套的 clippy restriction lints**【官】https://rust-lang.github.io/rust-clippy/master/ （restriction 组，需逐条打开）：`unwrap_used`、`expect_used`、`panic`、`indexing_slicing`、`todo`/`unimplemented`——都可以只在 `rt` 模块 `#![deny]`，正好对应「不 panic」。
- **本项目建议：用，第一优先。** workspace 根放 `clippy.toml`（禁用清单）；每个 `rt` 模块根 `#![deny(clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros, clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]`。直接借 tympan 的「模块级作用域」做法（借思路，自己写清单，不 copy）。

### 2.2 rustc 自带 lint 组合与 `[workspace.lints]`

- **是什么**：`#![deny(missing_docs, unsafe_op_in_unsafe_fn, unreachable_pub, ...)]`；Cargo 1.74 起可在 `[workspace.lints]` 统一设，子 crate `lints.workspace = true` 继承。【官】https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table ，rustc lint 列表 https://doc.rust-lang.org/rustc/lints/listing/index.html
- **解决什么**：全仓统一基线（`unsafe` 必须带注释与块、公共项必须有文档、`unreachable_pub` 逼你用 `pub(crate)`）。
- **代价**：几乎零。
- **本项目建议：用。** `[workspace.lints]` 设基线，`#![forbid(unsafe_code)]` 放在不需要 unsafe 的 crate（dsp、engine 核心、eval、control），unsafe 集中在 io-macos / 共享内存 / hal-plugin 胶水层。【推】

### 2.3 cargo-deny `[bans]`：限制依赖方向（含 `wrappers` 白名单）

- **是什么**：`deny = [{ crate = "x", wrappers = ["只有它能直接用"] , reason = "...", use-instead = "..." }]`——只允许列出的父 crate **直接**依赖被禁 crate，其他任何直接或传递依赖都报错；另有 `allow`（严格白名单）、`[[bans.features]]`（逐 crate 禁/准 feature）。【官】https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html ；本机文档 `R/cargo-deny/docs/src/checks/bans/cfg.md`
- **解决什么**：这是 Rust 里**最接近 ArchUnit / import-linter 的 crate 级架构测试**。例：
  - `ort`、`tract-onnx` 只许 `vdn-backend-*` 直接依赖 → engine / dsp 永远碰不到推理库细节。
  - `cpal`、`coreaudio-rs` 只许 `vdn-io-macos` 直接依赖 → eval 不可能偷偷依赖真设备。
  - `serde` 不许进 `vdn-dsp`（保持地基 crate 轻，呼应 matklad 的建议）。
- **成熟项目**：gungraun 的 `deny.toml` 保留了 `wrappers` 用法示例注释【源】`R/gungraun/deny.toml:183`；tympan-aspl、clack、tract/cli、speakrs 都带 `deny.toml`【源】见本机各仓根目录。
- **局限**：`wrappers` 与 `deny-multiple-versions` 不能同用【官】同上页；粒度是 crate，管不到模块内。另外 workspace 内部 crate 也能被 deny（按名字匹配），【推】需实测确认对 path 依赖生效。
- **本项目建议：用，第一优先。** 把「依赖方向图」写成 `deny.toml` 的 `wrappers`，顺带开 licenses（GPL-3.0 兼容性——这是 GPL 项目必做）与 advisories。

### 2.4 cargo-modules / cargo-depgraph：**画**依赖

- **是什么**：cargo-modules 画 crate 内模块树与模块间依赖；cargo-depgraph 输出 Graphviz 的 crate 依赖图。【传】https://github.com/regexident/cargo-modules ，https://github.com/jplatte/cargo-depgraph （未逐页核对 README）
- **解决什么**：给人看、给评审看；能发现 crate 内模块循环（cargo-modules 有 orphans/cycles 类功能【传】）。
- **代价**：只是可视化，不是门禁（除非自己解析输出）。
- **本项目建议：偶尔用，不进 CI 门禁。** 每次里程碑生成一张图放 `docs/`，配合 ARCHITECTURE.md。

### 2.5 rust-analyzer 的 xtask tidy：用**文本/语法**守规则

- **是什么**：rust-analyzer 把 `xtask` 当作仓库自带的构建/工具系统【官】architecture.md；其中 tidy 是一个普通 `#[test]`，`cargo test` 就跑。它检查：`#[should_panic]` 必须带 FIXME、`.rs` 无行尾空白、大多数文件必须有 `//!` 模块文档、测试标记 `hit`/`check` 成对、内部 `Cargo.toml` 依赖写法规范、依赖许可证集合与白名单一致、LSP 扩展文档哈希与源文件同步。【源】https://github.com/rust-lang/rust-analyzer/blob/master/xtask/src/tidy.rs （WebFetch 读取，函数名 `check_test_attrs` / `check_trailing_ws` / `TidyDocs` / `TidyMarks` / `check_cargo_toml` / `check_licenses` / `check_lsp_extensions_docs`）
- **要点**【推】：它大多是**字符串匹配**，不是真 AST——matklad 一系的取舍是「够用就行，便宜、无依赖、随 `cargo test` 跑」。「文档与源码哈希同步」那招很适合本项目（例：`specs/` 里的帧格式文档 ↔ 共享内存布局结构体）。
- **本机同类**：nih-plug、speakrs 都有 `xtask/`（nih-plug 主要用于打包，`R/nih-plug/.cargo/config.toml` 用 alias `xtask = "run --package xtask --release --"`）。【源】
- **本项目建议：用。** 建 `xtask`（或 `crates/vdn-tidy` 测试），放这些项目特有规则：
  1. `rt` 目录下每个 `pub fn` 必须带 `#[nonblocking]` 标记（或在白名单里）——见 §2.7；
  2. `eval` 与实时入口都只调用 `engine::Engine::process_frame`（禁止 eval 自己复刻处理链）——grep 级检查；
  3. 共享内存布局文档哈希同步；
  4. 文件头许可证声明（GPL-3.0）。
  需要真语法树时再在 xtask 里引 `syn`（只在 xtask 用，不污染产品 crate 编译时间）。

### 2.6 syn + 自写检查脚本

- **是什么**：在 xtask/测试里用 `syn::parse_file` 遍历源码 AST（`syn::visit::Visit`），检查「某属性下的函数体里出现了 `Box::new` / `.to_vec()` / `format!`」等。【官】syn 文档 https://docs.rs/syn/latest/syn/visit/index.html
- **局限**：syn 只有**语法**，没有名字解析和类型——`use std::sync::Mutex as M;` 之后的 `M::new()` 它不认识；宏展开后的代码看不到。【推】（syn 是纯 token→AST 解析器，这是其定义）
- **代价**：syn 编译慢（matklad：「解析 Rust 很难，所以 syn 编得久」【传】fast-rust-builds），所以只放 xtask。
- **本项目建议：按需用**，作为 §2.5 tidy 的升级版；先用 ast-grep 能写的就别写 syn 代码。

### 2.7 过程宏 / 属性宏打标记：`#[realtime]` / `#[nonblocking]`

- **两种用法要分清**【推】：
  1. **标记 + 外部检查**：宏本身什么也不做（或只做最小改写），检查交给 tidy / ast-grep / dylint 去找「带标记的函数」。
  2. **标记 + 运行时插桩**：rtsan-standalone 的 `#[nonblocking]` 属性宏会改写函数体，在进入/退出时通知 LLVM RealtimeSanitizer 运行时；在实时上下文里调用 `malloc`/`free` 等非确定性时长函数会报错；另有 `#[blocking]`、`#[no_sanitize_realtime]`、`scoped_disabler!`；运行时 `RTSAN_ENABLE=1` 开启；支持 Linux / macOS / iOS。【源】`R/rtsan-standalone-rs/crates/rtsan-standalone-macros/src/lib.rs:22-23,48-49,86-87`；【官】https://docs.rs/rtsan-standalone/latest/rtsan_standalone/
- **上游方向**：rust-lang/rust PR #147935 拟在 rustc 内置 RTSan，复用 `#[sanitize(realtime = "nonblocking" | "blocking" | "caller")]`，不再需要新属性；截至检索时仍在评审。【传】https://github.com/rust-lang/rust/pull/147935 （PR 页面与提交 https://github.com/rust-lang/rust/commit/93cd52dc2527ffe2662a2fd4f2d9262f04b8ca48 ，状态以页面为准）
- **Firewheel 的类似做法**：`#[derive(RealtimeClone)]` 作为「克隆不分配、不会在音频线程 drop 释放」的声明式标记。【源】`R/Firewheel/crates/firewheel-macros/src/lib.rs:25-28`。这是「用类型/derive 表达实时安全承诺」的样例（承诺本身仍靠人保证，宏不证明）。【推】
- **关键提醒**【推】：RTSan 是**运行时**检查——只能抓到测试/运行中**实际走到**的路径；它与 clippy（静态、只看本 crate）互补，能抓到第三方库内部的 malloc/锁。
- **本项目建议：用 rtsan-standalone 的 `#[nonblocking]` 作为唯一的实时标记**（不要自己再发明 `#[realtime]` 宏，一份标记两用：运行时由 RTSan 查，静态由 tidy/ast-grep 查「`rt` 模块的入口都带了它」）。开发期 dev-dependency + feature 开关；CI 加一个 `RTSAN_ENABLE=1` 跑 eval 回放的 job。等 rustc 内置版稳定后迁移到 `#[sanitize(realtime=...)]`。

### 2.8 assert_no_alloc（及 nih-plug / nice-assert-no-alloc 分支）

- **是什么**：自定义全局分配器，在 `assert_no_alloc(|| ...)` 闭包内任何分配/释放 → abort 或打印；可嵌套 `permit_alloc`。【官】https://docs.rs/assert_no_alloc ；nih-plug 用 feature `assert_process_allocs` 包住 process 回调。【源】`R/nih-plug/Cargo.toml:56,100-101`；tympan-aspl 在 `tests/realtime_safety.rs` 用全局 guard 做测试。【源】`R/tympan-aspl/Cargo.toml:38-46`，`R/tympan-aspl/tests/realtime_safety.rs`
- **与 RTSan 的取舍**【推】：assert_no_alloc 只管分配，纯 Rust、跨平台、无 LLVM 运行时；RTSan 还管锁/系统调用（上游 RTSan 设计目标，Rust 封装文档只演示了 calloc【官】docs.rs 页）。两者可叠：单元测试用 assert_no_alloc（快），集成回放用 RTSan（全）。
- **本项目建议：用**，engine 的 `process_frame` 单测与 eval 回放都包在 `assert_no_alloc` 里。

### 2.9 dtolnay 的 `no_panic`：链接期证明「不 panic」

- **是什么**：`#[no_panic]` 让编译器尝试证明函数无 panic 路径；若存在或无法证明，**链接失败**。前提：需开优化（dev 可设 `opt-level=1` 或仅 release 开）、`cargo check` 不触发、跨 crate 非内联调用可能需 LTO、`panic = "abort"` 时无效。【官】https://github.com/dtolnay/no-panic
- **代价**：误报多（编译器证不出来也报），需要把切片索引改成 `get` / 迭代器、算术改 wrapping；对 ONNX 推理调用（ort 内部）基本无望证明。【推】
- **本项目建议：有界试用**：只给 dsp 的叶子函数（biquad、限幅、门控增益）加；推理调用不加。与 clippy `indexing_slicing`/`unwrap_used` 互补——clippy 管「写法」，no_panic 管「结果」。

### 2.10 ast-grep 的 Rust 规则（YAML，CI）

- **是什么**：基于 tree-sitter 的结构化搜索/改写；规则是 YAML（`id` / `language: rust` / `rule` / `severity` / `message`，一文件多规则用 `---` 分隔），`ast-grep scan` 跑；可写正/负例 fixture 测规则；单行可用 `ast-grep-ignore` 豁免。【官】https://ast-grep.github.io/guide/project/lint-rule.html ，https://ast-grep.github.io/reference/yaml.html
- **成熟项目**：有 Rust crate 在 GitHub Actions 里固定版本 `cargo install ast-grep --locked` 后跑规则审计（含负例检查）。【传】https://docs.rs/crate/yog/0.0.1/source/.github/workflows/ci.yml ；CodeRabbit 把 ast-grep 作为内置审查工具。【传】https://docs.coderabbit.ai/tools/ast-grep
- **优点**：比 syn 脚本写得快（模式即代码），能表达**结构上下文**（「在带 `#[nonblocking]` 的函数体内 `inside` 出现 `$X.to_vec()`」）；不需要编译，秒级。
- **局限**：tree-sitter 语法树，无名字解析/类型（同 syn）；宏调用内部是 token 树，匹配受限。【推】
- **本项目建议：用，作为 clippy 之外的第二道语法门禁**。典型规则：`rt` 内禁 `.to_vec()` / `.collect::<Vec<_>>()` / `format!` / `String::from` / `.clone()`（对非 Copy 类型时人工复核）；`#[nonblocking]` 函数内禁 `Box::new`/`vec!`；engine 外禁出现 `GTCRN` 模型张量名字面量（接口只在 backend crate）。工具不在则降级为 xtask grep（写进 `scripts/check-tools.sh`）。

### 2.11 dylint（Trail of Bits）：自定义 lint，拿到类型信息

- **是什么**：从动态库加载自定义 lint，**不用 fork clippy**；在 `[workspace.metadata.dylint] libraries = [{ git/path = ..., pattern = ... }]` 声明；lint 实现 `LateLintPass`，用 rustc 内部 API（`rustc_lint`、`rustc_hir`、`rustc_middle`、`TyCtxt`），需要 `rustc_private`（即 nightly 工具链，与 clippy_utils 一样无稳定性保证）。【官】https://github.com/trailofbits/dylint ；介绍文 https://blog.trailofbits.com/2021/11/09/write-rust-lints-without-forking-clippy/
- **成熟项目**：NordSecurity 有公开的 dylint lint 库。【传】https://github.com/NordSecurity/nagger （通过 dependabot 跳转搜到，未逐页核对）
- **解决什么**：能做 clippy 配置做不到的**语义级**检查：例如「`#[nonblocking]` 函数**直接调用**的函数也必须是 `#[nonblocking]`」（一层调用图传播）、「`Drop` 实现会释放堆内存的类型不许在 rt 模块被 drop」。
- **代价**：nightly + rustc 内部 API，每次升级工具链都可能要修 lint；学习成本高；单人项目维护负担明显。【推】
- **本项目建议：暂不用，列为「第二阶段候选」。** 触发条件：clippy + ast-grep + RTSan 三层出现第三次漏网（按 CLAUDE.md「第三次沉淀成规则」）。

### 2.12 cargo-semver-checks

- **是什么**：先生成 rustdoc JSON，再用 Trustfall 查询引擎跑声明式 lint，检测公开 API 破坏性变更；只能看本包 JSON，暂无完整类型信息。【官】https://github.com/obi1kenobi/cargo-semver-checks ，Rust 项目目标页 https://rust-lang.github.io/goals/2025h2/cargo-semver-checks.html
- **解决什么**：发布到 crates.io 的库防止误破 API。
- **本项目建议：现在不用。** 我们是 workspace 内部 crate、一起发版，没有外部消费者；等哪个 crate（例如 `vdn-dsp` 或 HAL 胶水）独立发布再加。【推】

### 2.13 小结：静态层能「证明」到哪里（诚实版）

| 要求 | 能被机器证明的部分 | 证明不了的部分 → 靠什么 |
|---|---|---|
| 不分配 | 本 crate 内调用点（clippy/ast-grep） | 第三方内部 → assert_no_alloc / RTSan（运行时，覆盖率取决于测试） |
| 不加锁/不阻塞 | 名字可枚举的类型/方法（clippy） | 第三方内部系统调用 → RTSan |
| 不 panic | 写法（clippy restriction）+ 叶子函数链接期证明（no_panic） | ort 内部 → `catch_unwind` 不适用于实时线程【推】，只能靠模型加载期校验 + 测试 |
| 依赖方向 | crate 级全证明（cargo-deny wrappers） | 模块级 → `pub(crate)` + tidy |
| eval 与实时同一份代码 | 结构上证明（两者都只依赖 `engine`，tidy 检查入口） | — |

→ 「用机器证明」的落地表述应是：**静态层证明写法与依赖，运行时层在回放测试覆盖范围内证明行为**。这条应进 `.42cog/real.md` 的约束说明（由大总管/人决定是否写入）。【推】

---

## 3. (c) 用 AST / 宏**生成**重复代码

### 3.1 derive 宏（例：nih-plug `#[derive(Params)]`）

- **是什么**：nih-plug 用 `#[proc_macro_derive(Params, attributes(id, persist, nested))]` 从带 `#[id = "..."]` 字段的结构体生成参数枚举、持久化、宿主映射代码；整个 derive crate 约 750 行（params.rs 585 行）。【源】`R/nih-plug/nih_plug_derive/src/lib.rs:13`，`R/nih-plug/nih_plug_derive/src/params.rs`
- **为什么它值得**：插件框架要对接 VST3/CLAP 宿主，每个参数都要稳定 ID、自动化、序列化——重复量大、手写易错。
- **本项目情况**【推】：我们的「参数」只是控制面（门控阈值、EQ 段、压缩比、声纹开关），十几个，且不对接插件宿主。
- **本项目建议：不写自己的 derive。** 用普通结构体 + 原子（或 triple-buffer/arc-swap 发布整份参数快照）；需要序列化就在 `control` 叶子 crate 用 `serde::Deserialize`（已有 derive，零自研）。

### 3.2 `macro_rules!`

- **是什么**：声明式宏，卫生性、无需单独 crate、编译开销远小于过程宏；matklad 也建议「宏能接受简化语法就别用 syn」。【传】fast-rust-builds
- **本项目建议：小范围用**：例如为一组 biquad 类型批量实现同一 trait、或生成单元测试矩阵（每个 DSP 级 × 采样率 × 帧长）。规则：宏 ≤ 30 行、只在一个 crate 内部用、不导出。【推】

### 3.3 build.rs 代码生成

- **是什么**：编译前跑的 Rust 程序，可读文件生成 `.rs` 放进 `OUT_DIR`。【官】https://doc.rust-lang.org/cargo/reference/build-scripts.html
- **代价**：每次相关输入变化都重跑；生成代码不在仓库里可见，评审困难；跨编译、IDE 体验变差。Rust 官方编译性能调查也提到「稳定某些语言特性可减少 build script 与过程宏的需要，从而加速整个生态」。【官】https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results
- **替代做法（社区常见）**：rust-analyzer 风格——生成代码**提交进仓库**，由一个测试检查「重新生成后与仓库一致」（codegen + 一致性测试），而不是每次构建都生成。【推】（rust-analyzer 的 `cargo test -p xtask` 会收集语法测试到 `test_data/parser/inline`【官】architecture.md，属同一思路）
- **本项目建议：避免 build.rs 生成**；如需生成（如 EQ 系数表、mel 滤波器组），用 xtask 生成并提交，测试守一致性。

### 3.4 bindgen / cxx / cbindgen：FFI 生成

- **bindgen**：从 C 头生成 Rust 声明，coreaudio-sys 等已经这么做（由 coreaudio-rs 依赖链引入，非本项目自写）。【推】需 rusteco 核查表确认 coreaudio-sys 当前是否仍在 build 时跑 bindgen。
- **cxx**：Rust↔C++ 安全桥；本项目没有 C++ 边界，不需要。【推】
- **cbindgen**：从 Rust 生成 C 头。【传】https://github.com/mozilla/cbindgen
- **本项目关键点**【推】：自写 C 的 HAL 插件 ↔ Rust 推理进程之间**只有共享内存**，没有函数调用 FFI。真正要「单一真相源」的是**共享内存布局**（环形缓冲头、帧格式、版本号）。两个选择：
  1. Rust 侧 `#[repr(C)]` 结构体为真相源 → cbindgen 生成 C 头 → CI 检查头文件与仓库一致；
  2. 手写 C 头 + Rust 侧 `const _: () = assert!(size_of::<Header>() == N && offset_of!(Header, x) == M);` 编译期断言（`offset_of!` 自 Rust 1.77 稳定【官】https://doc.rust-lang.org/std/mem/macro.offset_of.html ）。
  - 建议 **1 + 2 同时**：cbindgen 生成、编译期断言兜底。若 HAL 改走 Rust（tympan-aspl 路线，归 rusteco 评估），此条自动简化。
- **本项目建议：用 cbindgen（仅限共享内存头），不用 cxx。**

### 3.5 ort 的模型接口能否从 ONNX 元数据生成

- **事实**：ort 2.0.0-rc.15 的 `Session` 有 `inputs()` / `outputs()`（返回 `&[Outlet]`）和 `metadata()`（返回 `ModelMetadata`）。【官】https://docs.rs/ort/latest/ort/session/struct.Session.html ；`Outlet` 字段细节本次未核到——**未找到出处**，交 rusteco 核查。
- **两条路**【推】：
  - 编译期：build.rs 读 `.onnx` 生成强类型输入输出结构 → 模型二进制进构建依赖、GTCRN 的有状态缓存（conv/GRU cache）张量名各版本不同，生成器要处理流式状态，复杂度高；且模型大文件不进仓（铁律 8），CI 拿不到模型就编不过。
  - 运行期：每个 backend 写一份手写 `ModelSpec`（输入/输出名、形状、dtype、状态张量列表、帧长、采样率），**加载时**对照 `session.inputs()/outputs()` 校验，不符就拒绝加载（非实时线程，可以分配、可以报错）；外加一条测试用真实模型跑校验。
- **本项目建议：运行期校验，不生成。** 只有两三个模型，`ModelSpec` 手写几十行，比生成器可读；加载期校验把「模型换了张量名」变成明确错误而不是实时线程里的 panic。

### 3.6 过程宏的代价（社区论述汇总）

| 代价 | 出处 |
|---|---|
| 使用过程宏的 crate 要等宏 crate 及其依赖（syn/quote）编完才能开始，造成 CPU 利用率里的「syn/serde 形状的空洞」 | 【传】matklad https://matklad.github.io/2021/09/04/fast-rust-builds.html |
| derive 宏展开当前**不缓存**，官方在做缓存 | 【官】https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results |
| dtolnay 做 watt（把过程宏预编译为 Wasm 运行）就是为了省下游 20+ 秒的宏编译；但展开可能比原生慢 | 【官】https://docs.rs/watt |
| 预编译二进制的信任问题：serde_derive 曾发布预编译二进制，Fedora 等打包方无法再分发、并有安全顾虑，引发 #2538 讨论 | 【官】https://github.com/serde-rs/serde/issues/2538 （后来撤回预编译的具体版本号本次**未核到出处**，记【传】） |
| IDE：rust-analyzer 要在 proc-macro-srv 里执行宏；Zed 曾把一个 GPUI derive 从 rust-analyzer 展开中排除，因为展开可达 10 秒 | 【传】https://git.secluded.site/zed/commit/a66a539a0984ad02908b79952e90d9978a603053 ；nightly 下 proc macro 支持问题见 https://fasterthanli.me/articles/proc-macro-support-in-rust-analyzer-for-nightly-rustc-versions |
| 可读性：生成代码不可见，需 `cargo expand` 才能看；错误信息落在宏调用点 | 【推】 |

matklad 关于「过程宏与 IDE」的专门博文：**未找到出处**（检索未命中，不臆造）。

**本项目规则**【推】：产品 crate 只**使用**成熟宏（serde derive 限 control/eval 叶子、rtsan `#[nonblocking]`、`thiserror` 视需要），**不自写过程宏**；确需自写时先证明「macro_rules 或普通泛型做不到」。

---

## 4. (d) 其他语言的借鉴：哪些能在 Rust 落地

| 思想 | 原意 | Rust 里的对应物 | 本项目用不用 |
|---|---|---|---|
| **Lisp / Elixir 宏**（代码即数据，宏在 AST 上编程） | 用宏造领域语言；Elixir 社区信条是「能用函数就别用宏」【传】Elixir 官方指南 Macros 章节 https://hexdocs.pm/elixir/macros.html （页面内容本次未逐字核对） | `macro_rules!` + 过程宏；同样的告诫适用 | 不造 DSL；处理链用普通 Rust 结构体组合（`Chain { gate, dereverb, comp, eq }`），类型就是「语法树」 |
| **Unison 内容寻址代码**（定义由其 AST 哈希标识，重命名免费、无依赖冲突） | 代码按 AST 存数据库而非文本 | 无直接对应。可借的是**「按内容哈希标识产物」**：模型文件、参数快照、eval build 都用 sha256 作 ID【官】Unison 文档 https://www.unison-lang.org/docs/the-big-idea/ | **借思想**：`ModelSpec` 里写模型 sha256，加载时校验；eval 结果记录模型哈希 + 代码 commit——与 CLAUDE.md 揭盲表/build id 规则同构 |
| **Zig comptime**（编译期执行普通代码，泛型即 comptime） | 无宏也能编译期计算与特化 | `const fn`、const generics（`Biquad<const N: usize>`）、`const _: () = assert!(...)` 编译期断言【传】https://scattered-thoughts.net/writing/assorted-thoughts-on-zig-and-rust/ ；社区 crabtime 尝试模仿【传】 | **用 const generics 固定帧长/通道数**，让缓冲区尺寸成为类型参数 → 实时路径全是定长数组，天然不分配；共享内存布局用编译期断言 |
| **ArchUnit**（Java：在字节码上写架构单元测试，`layeredArchitecture()`） | 架构规则写成测试 | crate 级 = cargo-deny `wrappers`；模块级 = xtask tidy/ast-grep；语义级 = dylint【推】 | 用前两层 |
| **import-linter**（Python：`layers` 合约，高层可导入低层，反之不可，含间接导入；另有 `forbidden` 合约） | 声明分层，CI 验证 | 「分层」在 Rust 里由 crate DAG 天然保证；「forbidden」= cargo-deny `deny`+`wrappers`【官】https://import-linter.readthedocs.io/en/stable/contract_types/layers/ | 已由 §2.3 覆盖；借它的**表述方式**：在 ARCHITECTURE.md 里把层次写成一张有序表 |
| **dependency-cruiser**（JS：`forbidden` / `allowed` / `required` 规则，`allowed` 之外即违规） | 白名单式依赖规则 | cargo-deny `[bans] allow`（非空即严格白名单）【官】cargo-deny cfg 页；规则来源 https://github.com/sverweij/dependency-cruiser/blob/main/doc/rules-tutorial.md 【传】 | 可选：对 `vdn-dsp`、`vdn-engine` 这类地基 crate 用 `allow` 白名单思路（列出允许的外部依赖），新增依赖必须改 deny.toml——留痕且需评审 |

---

## 5. 落到本项目：结构草案 + 守护规则（供设计稿用）

### 5.1 crate 依赖方向（【推】，在初拟 7 个 crate 上加了「词汇 crate」与后端拆分）

```
vdn-types   (帧格式、ModelSpec、共享内存布局 #[repr(C)]、错误码；零重依赖)
   ↑
vdn-dsp     (biquad / 压缩限幅 / 去混响尾 / 门控；no_std 友好，forbid(unsafe_code))
   ↑
vdn-engine  (Engine::process_frame；Denoiser / InferBackend trait；rt 模块；不依赖 ort/tract/cpal)
   ↑                       ↑                    ↑
vdn-backend-ort   vdn-backend-tract     vdn-speaker (ECAPA/CAM++，非实时线程，原子发布)
   ↑                       ↑                    ↑
   └──────────── 叶子：vdn-io-macos（cpal/coreaudio、共享内存）· vdn-eval（读 WAV）· vdn-control（serde 配置）
hal-plugin (C，独立构建；只认 cbindgen 生成的共享内存头)
```

- eval 与实时进程都只通过 `vdn-engine` 处理 → 「同一份处理代码」由依赖图保证。
- matklad 的「宽图」：dsp / backend / speaker 互不依赖，可并行编。

### 5.2 守护规则清单（第一阶段，全部便宜）

1. `deny.toml`：`ort`/`tract-*` wrappers = backend crate；`cpal`/`coreaudio-rs`/`coreaudio-sys` wrappers = io-macos；`serde` 不进 types/dsp/engine；licenses 白名单 GPL-3.0 兼容；advisories。
2. `clippy.toml` 禁用清单 + 各 `rt` 模块根 `#![deny(...)]`（借 tympan-aspl 的模块作用域做法）。
3. `[workspace.lints]` 基线 + 非胶水 crate `#![forbid(unsafe_code)]`。
4. `#[nonblocking]`（rtsan-standalone）标所有实时入口；CI job：`RTSAN_ENABLE=1` 跑 eval 回放。
5. engine 单测与 eval 回放包 `assert_no_alloc`。
6. `xtask tidy`（cargo test 跑）：`rt` 入口都带 `#[nonblocking]`、eval 只经 engine、共享内存头一致、许可证头。
7. ast-grep 规则目录 `rules/`（带正负例 fixture），CI 固定版本跑；无工具时降级 grep。
8. `no_panic` 有界试用于 dsp 叶子函数（release profile）。

### 5.3 第二阶段候选（触发再上）

dylint 自定义 lint（调用图一层传播）；cargo-semver-checks（某 crate 独立发布时）；cargo-modules 画图进里程碑文档。

---

## 6. 总表：AST 用法 → 本项目采用决定

| # | 用法 | 解决什么 | 代价 | 成熟先例（出处） | 决定 | 用在哪 |
|---|---|---|---|---|---|---|
| 1 | crate 拆分（扁平 workspace、virtual manifest、宽图） | 物理边界、依赖方向、编译并行 | Cargo.toml 增多 | rust-analyzer 架构不变量【官】；matklad 两文【传】 | **采用** | 全仓，见 §5.1 |
| 2 | module + `pub(crate)` | crate 内边界、最小 API | 软边界 | Rust Reference【官】；tympan `realtime` 模块【源】 | **采用** | 每 crate 的 `rt` 模块 |
| 3 | trait（静态分发） | 模型/后端可切换、eval 与实时共用 | 不能表达实时安全 | — 【推】 | **采用** | Denoiser / InferBackend / Stage |
| 4 | Cargo feature | 加性开关 | 互斥 feature 会被统一打开 | Cargo Book【官】；nih-plug `assert_process_allocs`【源】 | **限用**（只做检查开关） | `rt-check` |
| 5 | clippy disallowed-methods/types/macros | 实时路径禁锁/禁阻塞/禁 IO | 只看本 crate 调用点 | Clippy 配置文档【官】；tympan-aspl【源】 | **采用·第一优先** | workspace `clippy.toml` + `rt` 模块 `#![deny]` |
| 6 | clippy restriction（unwrap_used/panic/indexing_slicing） | 不 panic 的写法 | 改写成本 | Clippy lint 列表【官】 | **采用** | `rt` 模块 |
| 7 | rustc lint + `[workspace.lints]` | 统一基线、forbid unsafe | ≈0 | Cargo Book【官】 | **采用** | 全仓 |
| 8 | cargo-deny bans + wrappers | crate 级架构测试（≈ArchUnit/import-linter） | 粒度只到 crate | cargo-deny 文档【官】；gungraun 示例【源】 | **采用·第一优先** | `deny.toml` |
| 9 | cargo-modules / cargo-depgraph | 可视化 | 非门禁 | 项目 README【传】 | 偶尔用 | 里程碑文档 |
| 10 | xtask tidy（文本/轻语法检查进 cargo test） | 项目特有不变量 | 自己维护 | rust-analyzer tidy.rs【源】 | **采用** | `xtask/` |
| 11 | syn 检查脚本 | 需要真语法树的规则 | syn 编译慢、无类型 | syn 文档【官】 | 按需（只在 xtask） | tidy 升级版 |
| 12 | 属性宏标记 `#[nonblocking]`（rtsan-standalone） | 实时入口标记 + 运行时查 malloc/阻塞 | 运行时、覆盖率依赖测试 | rtsan-standalone 源码【源】/docs【官】；rustc PR #147935【传】 | **采用**（不自造 `#[realtime]`） | 所有实时入口 |
| 13 | assert_no_alloc | 分配断言（含第三方） | 全局分配器、仅测试 | docs【官】；nih-plug / tympan【源】 | **采用** | engine 单测、eval 回放 |
| 14 | no_panic | 链接期证明不 panic | 误报多、需优化/LTO | dtolnay README【官】 | **有界试用** | dsp 叶子函数 |
| 15 | ast-grep YAML 规则 | 结构化语法门禁，秒级 | 无类型、宏内受限 | ast-grep 文档【官】；CI 先例【传】 | **采用** | `rules/` + CI |
| 16 | dylint | 语义级自定义 lint | nightly + rustc 内部 API | Trail of Bits README/博客【官】 | **暂缓**（第三次漏网再上） | — |
| 17 | cargo-semver-checks | 公开 API 破坏检测 | 只对外发布有意义 | 项目 README / Rust goals【官】 | **暂不用** | 某 crate 独立发布时 |
| 18 | 自写 derive 宏 | 大量重复声明 | 编译时间、IDE、可读性 | nih-plug Params【源】 | **不采用** | 参数用普通结构体 |
| 19 | macro_rules! | 小范围重复 | 可读性 | matklad【传】 | **小范围用**（≤30 行、不导出） | trait 批量实现、测试矩阵 |
| 20 | build.rs 生成 | 编译前生成 | 不可见、慢、IDE 差 | 官方性能调查【官】 | **不采用**；改 xtask 生成+提交+一致性测试 | 系数表等 |
| 21 | cbindgen | Rust→C 头，单一真相源 | 多一个工具 | 项目 README【传】 | **采用**（仅共享内存头） | types ↔ hal-plugin |
| 22 | bindgen / cxx | C/C++→Rust | — | — | 不自写（依赖链已有）/ 不用 cxx | — |
| 23 | 从 ONNX 元数据生成模型接口 | 强类型张量接口 | 模型进构建、流式状态复杂 | ort `inputs()/outputs()/metadata()`【官】 | **不生成**；手写 `ModelSpec` + 加载期校验 | backend crate |
| 24 | Zig comptime 思想 | 编译期特化 | — | 社区比较文【传】 | **借**：const generics 定长帧、编译期断言 | dsp / types |
| 25 | Unison 内容寻址思想 | 按内容哈希标识 | — | Unison 文档【官】 | **借**：模型/参数/eval build 用 sha256 作 ID | ModelSpec、eval |
| 26 | Lisp/Elixir 宏 DSL | 领域语言 | 可读性 | Elixir 指南【传】 | **不采用** | — |
| 27 | ArchUnit / import-linter / dependency-cruiser | 架构规则即测试 | — | 各自文档【官/传】 | **借思想**，由 #8 #10 #15 落地 | deny.toml + tidy + rules/ |

---

## 7. 待人拍板 / 待核

- 【待人定】是否接受「静态层证明写法与依赖、运行时层在回放覆盖范围内证明行为」作为「用机器证明」的正式口径（影响 `.42cog/real.md` 措辞）。
- 【待核·rusteco】ort `Outlet` 的字段（名称/形状/dtype）；coreaudio-sys 是否构建期跑 bindgen；tympan-aspl 作为 Rust HAL 路线的可行性。
- 【待测】clippy `disallowed-macros` 能否禁 `std::println`（tympan 注释称不能按路径列，与官方配置项矛盾）；cargo-deny `wrappers` 对 workspace path 依赖是否生效。
- 未找到出处：matklad 专论「过程宏与 IDE 代价」的博文；serde_derive 撤回预编译二进制的确切版本号。
