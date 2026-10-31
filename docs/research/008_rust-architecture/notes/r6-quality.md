# R6 · 工具链与质量闸门（vdn Rust 架构调研 · 主题 6）

> 调研日期 2026-10-30。证据等级：【官】官方明载 /【源】源码可证 /【测】本机实测 /【传】二手或论坛 /【推】我的推断。
> 本机路径前缀 `RES=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources`。
> 本文只讲「规则、做法、取舍」。单项目细节（nih-plug、basedrop 等）归 rusteco 的核查表。

---

## 0. 结论先行（给决策者）

1. **「音频线程不分配、不加锁、不阻塞、不 panic」没有一个工具能单独证明，要四层叠起来**：
   - 编译期 / AST 层：clippy `disallowed-methods/types/macros` + `unwrap_used/expect_used/panic/indexing_slicing` 只对 rt 模块开到 deny；`#[no_panic]` 做链接期证明（仅限小函数）
   - 运行期 · 分配：`assert_no_alloc` 或自写计数分配器（测试和 debug 里用）
   - 运行期 · 阻塞调用：**RealtimeSanitizer**（`-Zsanitizer=realtime`，nightly，已合入 rustc）或 `rtsan-standalone` crate，能抓 malloc / free / mutex 等，**macOS 支持**
   - 并发正确性：自写无锁结构用 loom（穷举）、Miri（UB + 数据竞争）、TSAN（真线程）
2. **最重要的那条性质测试是「分块不变性」**：同一段 WAV 用 1 帧、7 帧、整段三种切法喂 engine，输出必须逐值一致（或在 1e-6 以内）。它就是「eval 和实时用同一份代码」这条约束的机器版本。工具用 proptest。
3. **Python ↔ Rust 对拍分两种做法**：DeepFilterNet 的做法是 Python 训练直接 `from libdf import DF` 调 Rust 写的特征（同一份代码，所以不用对拍），端到端只卡指标阈值；nnnoiseless 的做法是对 C 参考实现逐字节 `diff`，再加相对误差 1e-4。本项目要两者合用：特征 / 前后处理按逐值 golden 对拍，模型输出按容差对拍，整链按指标卡阈值。
4. **CI 成本是真约束**：仓库是私有的，macOS runner 每分钟价格约为 Linux 的 10 倍（$0.062 对 $0.006），macOS runner 也**没有可靠的音频设备**。所以 dsp / engine / speaker / eval 必须能在 Linux 上编译和测试（io-macos、hal-plugin 隔离在单独的 crate 里），macOS runner 只跑一个精简 job，带设备的测试和 soak 放到本机（自托管 runner 或者手动跑）。
5. **iai-callgrind（现名 Gungraun）在 macOS 上用不了**：它依赖 Valgrind（或 Linux perf），Homebrew 的 valgrind formula 写着 `depends_on :linux`。指令计数基准只能放在 Linux CI 上跑；墙钟基准（criterion / divan）只在本机 Mac 上看趋势，不进 CI 闸门。
6. **许可证**：GPL-3.0 项目可以接 MIT / Apache-2.0 / BSD / ISC / Zlib / BSL-1.0 / Unicode / MPL-2.0（FSF 明载这些与 GPLv3 兼容），要拒的是 GPL-2.0-only 和不明许可。ort 是 `MIT OR Apache-2.0`，ONNX Runtime 是 MIT，tract 是 `MIT OR Apache-2.0`，都没问题。**cargo-deny 看不到 ONNX 权重和 BlackHole**，模型和二进制的许可要单独开一张清单。
7. **用 AST 层工具回应「做成模块」那句话**：模块边界本身也可以由机器来守。cargo-deny 的 `bans.deny + wrappers` 能规定「只有 `engine-ort` 可以依赖 `ort`」；clippy 的 `disallowed-*` 能规定「rt 模块不准出现 `Vec::push`、`Mutex::lock`」；规则再细就用 dylint 自写 `LateLintPass`，它跑在编译器的 HIR（高层中间表示，是语法树解析之后的一层）上。这比肉眼看代码可靠。

---

## 1. 逐个工具：能干什么、不能干什么、对本项目的用法

### 1.1 cargo-nextest（测试运行器）

- 每个测试一个进程，这是默认并且「将一直是」默认的执行模型；好处是全局状态隔离、超时和取消安全；代价是进程创建开销，在 macOS 上可能因为反恶意软件扫描而变慢 【官】https://nexte.st/docs/design/why-process-per-test/
- **不跑 doctest**，要另跑 `cargo test --doc` 【官】https://nexte.st/book/usage.html （经搜索结果转引，原句 “Doctests are currently not supported because of limitations in stable Rust”）
- 配置文件放在 `.config/nextest.toml`，可以按 profile 分（`[profile.ci] fail-fast=false`、`retries`、`slow-timeout`）【官】https://nexte.st/docs/configuration/
- 能和 Miri 配合（`cargo miri nextest run`），每个测试一个 Miri 上下文，并行后可快 3–4 倍；但测试之间的数据竞争检测不到 【官】https://nexte.st/docs/integrations/miri/
- **本项目用法**【推】：快速层的默认测试命令；声纹或 ONNX 这类加载模型的「重测试」用 test-group 限制并发；给实时 engine 的回放测试设 `slow-timeout`，用来抓死循环。

### 1.2 Miri（UB 解释器）

- **能抓**：越界、释放后使用、未初始化读、未对齐、无效 bool 等类型不变量；数据竞争和部分弱内存效果；内存泄漏；Stacked / Tree Borrows 别名检查（实验性）【官】https://github.com/rust-lang/miri
- **不能**：“no access to most platform-specific APIs or FFI”；它只走一条执行路径，会漏掉只在别的执行顺序下出现的 bug；弱内存模拟不完整 【官】同上
- `-Zmiri-native-lib` 能调真 .so，但只支持整数和指针参数，标明 unsound 【官】同上
- `--target` 能跨平台解释，比如在 macOS 上当 Linux 程序跑，或者用 s390x 测大端 【官】同上
- 实例：rtrb 的 CI 跑三遍 Miri，依次是默认、`-Zmiri-preemption-rate=0`（只跑单个测试）、`-Zmiri-tree-borrows` 【源】`$RES/rtrb/.github/workflows/main.yml:105-130`；测试里用 `cfg!(miri)` 把迭代次数降下来（100_000 → 1_000）【源】`$RES/rtrb/tests/lib.rs:69,96,112`
- **对本项目**【推】：
  - 能测：dsp（纯 Rust 的 biquad、压缩器、门控）、自写无锁结构（声纹结果的原子发布、共享内存帧头的 seqlock 或三缓冲）、所有 `unsafe` 块
  - 不能测：cpal / coreaudio、`shm_open`/`mmap`、ort（ONNX Runtime 是 C++ FFI）、tract（纯 Rust，但慢到不实用）。**做法**：把共享内存协议写成「对 `&[AtomicU64]` 操作的纯算法」，Miri 和 loom 里用堆内存跑，真 shm 只在集成测试里用
  - 本机只有 stable 1.98.1 / 1.99.1，没有 nightly；`cargo-miri` 只是 rustup 代理，装 nightly 之前用不了 【测】（`rustup toolchain list`）

### 1.3 loom（并发模型检查）

- 在 C11 内存模型下穷举并发执行，靠状态归约控制规模 【官】https://github.com/tokio-rs/loom
- 用法：`[target.'cfg(loom)'.dependencies] loom`，测试体包在 `loom::model(|| …)` 里，用 `RUSTFLAGS="--cfg loom" cargo test --release` 跑；推荐在 crate 里写一个 `sync` 模块，按 `cfg(loom)` 切换 std 和 loom 的原子类型 【官】https://docs.rs/loom/latest/loom/
- 局限：SeqCst 被当成 AcqRel 处理（可能误报），`fence(SeqCst)` 支持；load buffering 有些执行顺序不探索（可能漏报）；Relaxed 不能完全建模 【官】GitHub README 与 docs.rs
- `LOOM_MAX_PREEMPTIONS` 设 2–3 「足以抓到大多数 bug」【官】docs.rs/loom
- 实例：rtrb **不用 loom**，用 Miri 加 ThreadSanitizer 【源】`$RES/rtrb/.github/workflows/main.yml:105-151`；TSAN 对独立 fence 有误报，所以跳过 `no_race_with_is_abandoned` 【源】同文件 `:147-151`，测试位于 `$RES/rtrb/tests/lib.rs:164`
- 替代品：AWS 的 **Shuttle** 走随机调度（PCT），“not sound”，但能扩展到比 loom 大得多的用例 【官】https://github.com/awslabs/shuttle
- **对本项目**【推】：rtrb 已经由上游测过，**我们不重复测**。loom 只用在**自写**的同步原语上：①声纹线程 → 降噪线程的结果发布（AtomicU64 打包或 ArcSwap 式双缓冲）②跨进程环的读写索引协议（用线程模拟进程）③控制面参数热更新（triple buffer）。每个原语写成一个小 crate（或 `engine::sync` 模块），`cfg(loom)` 测试放在同一处。

### 1.4 proptest / quickcheck（性质测试）

- proptest 用显式 `Strategy`、按值域生成和收缩、遵守约束，所以拒绝率低；复杂值的生成可能比 QuickCheck 慢一个数量级；QuickCheck 只按类型生成，收缩无状态 【官】https://proptest-rs.github.io/proptest/proptest/vs-quickcheck.html
- proptest 会把失败用例持久化到文件（`proptest-regressions/`），这个文件要提交进仓 【官】（Failure Persistence 章节，本次没有单独抓取正文）
- **本项目选 proptest**【推】，原因是 DSP 参数都有物理值域（采样率、Q 值、阈值 dB），用 Strategy 表达很自然。必写的性质：
  1. **分块不变性**：`process(x)` 等于 `x` 随机切块后逐块 `process` 再拼接（最关键，守住「eval 与实时同码」）
  2. 限幅器输出 ≤ 设定上限；门控增益 ∈ [0,1]；输出处处有限（输入含 NaN / inf / 次正规数时，输出也不出 NaN）
  3. biquad 系数在给定参数域内极点都在单位圆内（稳定）
  4. reset 之后的状态和新建的状态等价
  5. GTCRN 和 FastEnhancer 的切换不改变帧长、延迟报告等接口契约

### 1.5 cargo-fuzz（模糊测试）

- 只支持 libFuzzer（经 `libfuzzer-sys`）【官】https://rust-fuzz.github.io/book/cargo-fuzz.html
- “only works on x86-64 and Aarch64, and only on Unix-like operating systems (not Windows)”，并且需要 nightly 【官】https://github.com/rust-fuzz/cargo-fuzz → Apple Silicon 上的 macOS 可用【推】
- **本项目的 fuzz 目标**【推】：WAV / 配置文件解析；共享内存帧头解析（另一进程写进来的数据要当不可信输入看）；控制面的 IPC 消息；`dsp` 在任意 f32 输入下不 panic、输出有限。fuzz 不进 PR 闸门，夜间跑固定时长（例如每个目标 10 分钟），语料库提交进仓。

### 1.6 基准：criterion vs divan；iai-callgrind / Gungraun

- criterion FAQ：**不该在 CI 里跑**（至少别信它的结果），云端虚拟化噪声太大，推荐 Iai 【官】https://bheisler.github.io/criterion.rs/book/faq.html
- rtrb 用 criterion 0.8，`[profile.bench] lto=true, codegen-units=1`，CI 只跑 `cargo test --benches`（编得过、跑得通，不比较数值）【源】`$RES/rtrb/Cargo.toml:30,35-38`、`$RES/rtrb/.github/workflows/main.yml:34-36`
- divan 的 `AllocProfiler` 是 GlobalAlloc 包装，统计 alloc / dealloc / realloc 的次数和字节数，计数存在线程局部变量里；**divan 不管的线程里的分配不计入** 【官】https://docs.rs/divan/latest/divan/struct.AllocProfiler.html
  - 推论【推】：它能证明「`engine.process_frame()` 在 bench 线程上每次迭代分配 0 次」，但证明不了 ort 内部线程池里的分配。ort 要设 `intra_op_num_threads=1`，并且在实时线程上同步调用，这时它的分配才落在当前线程，能被统计到（ORT 的分配走它自己的 arena，是否经过 Rust 的 global allocator 要实测，**未找到出处**）
- **Gungraun（原 iai-callgrind）**：依赖 Valgrind 或 Linux perf；“cannot be run on Windows or targets that support neither” 【官】https://github.com/iai-callgrind/iai-callgrind 、https://gungraun.github.io/gungraun/latest/html/installation/prerequisites.html（安装页只列了 Linux 发行版和 FreeBSD）
  - Homebrew 的 valgrind formula `depends_on :linux`，bottle 只有 `arm64_linux`、`x86_64_linux` 【官】https://github.com/Homebrew/homebrew-core/blob/HEAD/Formula/v/valgrind.rb ；本机 `brew info valgrind` 显示 Not installed 【测】
  - 社区 fork（LouisBrunner/valgrind-macos）对 arm64 只标「实验性」【传】
  - **结论：macOS 上不可用**【推，证据链：官+测】。在 Linux CI 上跑指令计数是可以的，因为 dsp / engine 能在 Linux 上编
- **本项目取舍**【推】：
  - divan 用来做「分配为零」的闸门（数值是确定的，不受噪声影响，可以进 CI）
  - Gungraun 在 Linux CI 上做指令数回归（确定性好，可以设阈值，比如 +5% 报警）
  - criterion 或 divan 的墙钟数只在本机 Mac 上看，保存 baseline 对比；真正的实时余量看 §4 的「每帧耗时 p99.9 / 截止时间」

### 1.7 cargo-deny（许可证 · 安全公告 · 禁用 crate · 来源）

**licenses** 【官】https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html
- `allow` 白名单，不在名单里的默认拒绝；`exceptions` 按 crate 单独放行；`clarify` 给没有机器可读许可的 crate 手动指定 SPDX（锁定 license 文件的哈希）；`private.ignore` 跳过不发布的工作区 crate；`include-dev` 默认 false；`confidence-threshold` 默认 0.8

**和 GPL-3.0 的兼容性** 【官】https://www.gnu.org/licenses/license-list.en.html
- Apache-2.0：与 GPLv3 兼容，**与 GPLv2 不兼容**
- Expat (MIT)、BSD-3-Clause、ISC、Zlib、BSL-1.0、Unicode（含 v3）：兼容
- MPL-2.0：§3.3 提供间接兼容（组合作品整体按 GPL）
- ⇒ 选 GPL-3.0 而不是 GPL-2.0，正是 Apache-2.0 依赖（cpal 是纯 Apache-2.0）能合法接入的前提 【推】

**本项目依赖的许可实况**
| 依赖 | 许可 | 出处 |
|---|---|---|
| cpal | Apache-2.0 | 【源】`$RES/cpal/Cargo.toml:7` |
| rtrb | MIT OR Apache-2.0 | 【源】`$RES/rtrb/Cargo.toml:13` |
| rubato | MIT OR Apache-2.0 | 【源】`$RES/rubato/Cargo.toml`（license 行） |
| coreaudio-rs | `MIT/Apache-2.0`（旧式斜杠写法，cargo-deny 会宽松解析成 OR 并给警告【推】） | 【源】`$RES/coreaudio-rs/Cargo.toml:11` |
| audio_thread_priority | MPL-2.0 | 【源】`$RES/audio_thread_priority/Cargo.toml` |
| nnnoiseless | BSD-3-Clause | 【源】`$RES/nnnoiseless/Cargo.toml` |
| webrtc-audio-processing | `license-file = "COPYING"`（BSD 风格）→ **需要 `clarify`** | 【源】`$RES/webrtc-audio-processing/Cargo.toml:25` |
| deep_filter (libDF) | `MIT/Apache-2.0` | 【源】`$RES/DeepFilterNet/libDF/Cargo.toml` |
| ort | MIT OR Apache-2.0；默认 feature 含 `download-binaries`、`copy-dylibs` | 【官】https://raw.githubusercontent.com/pykeio/ort/main/Cargo.toml |
| ONNX Runtime | MIT | 【官】https://github.com/microsoft/onnxruntime/blob/main/LICENSE |
| tract-core | MIT OR Apache-2.0 | 【官】https://raw.githubusercontent.com/sonos/tract/main/core/Cargo.toml |
| GTCRN 权重/代码 | MIT | 【源】`$RES/gtcrn/LICENSE` |
| FastEnhancer | MIT | 【源】`$RES/fastenhancer/LICENSE` |
| 3D-Speaker（CAM++）/ wespeaker（ECAPA） | Apache-2.0（代码；**预训练权重的许可要逐个模型卡核对**，未查） | 【源】`$RES/3D-Speaker/LICENSE`、`$RES/wespeaker/LICENSE` |
| BlackHole | 源码 GPL-3.0；官方二进制和品牌另有保留 | 【源】`$RES/BlackHole/LICENSE:1-7` |

**cargo-deny 的盲区**【推】：ort 的 `download-binaries` 会在 build 时下载预编译的 ONNX Runtime 动态库，ONNX Runtime 自带的 ThirdPartyNotices 不在 Cargo 图里；`.onnx` 权重、BlackHole 驱动也不在。⇒ 另设 `docs/licenses/assets.md`（或类似文件）列「二进制和模型资产 · 来源 URL · 哈希 · 许可」，用脚本检查「仓里每个 .onnx 都有一行」。

**advisories** 【官】https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html
- 默认用 RustSec DB；`unmaintained` 默认 `all`（命中就失败）；`unsound` 默认只对直接依赖失败；`yanked` 默认只警告；`ignore` 列表把条目降级为 note
- 本项目【推】：`yanked = "deny"`；`ignore` 每一条都要写理由和到期日

**bans** 【官】https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html
- `multiple-versions`（重复版本）、`deny` + `wrappers`（只允许指定的 crate 直接依赖被禁的 crate）、`skip` / `skip-tree`、`wildcards`、`[[bans.features]]`（禁止或白名单 feature）、`[bans.build] allow-build-scripts`（构建脚本白名单）
- **本项目可以拿它当「模块边界守卫」**【推】：
  - `deny = [{ name = "ort", wrappers = ["vdn-infer-ort"] }, { name = "tract-onnx", wrappers = ["vdn-infer-tract"] }]`：引擎核心不能直接碰推理后端，GTCRN 和 FastEnhancer 才能在同一个接口下切换
  - `deny = [{ name = "cpal", wrappers = ["vdn-io-macos"] }, { name = "coreaudio-rs", wrappers = [...] }]`：dsp / engine 永远不依赖设备层，因此能在 Linux CI 上编
  - `[[bans.features]]` 禁掉 `ort` 的 `download-binaries`（在 release / 可复现构建 profile 下），改走固定版本的 dylib 加哈希
  - `allow-build-scripts` 白名单：限制供应链里的 build.rs

**sources** 【官】https://embarkstudios.github.io/cargo-deny/checks/sources/cfg.html
- `unknown-registry` / `unknown-git` 设成 deny；`allow-git` 要求 URL 精确匹配；`allow-org` 可以放行整个组织
- 本项目【推】：只允许 crates.io；如果要 fork 某个 crate 打补丁，放在 `AllenChen-Xingan` 组织下，用 `allow-org` 放行

### 1.8 cargo-udeps / cargo-machete（未用依赖）

- cargo-udeps：要 nightly 才能跑；查不到只在 doctest 里用的依赖；按名字判断，同名不同版本的依赖会出问题 【官】https://github.com/est31/cargo-udeps
- cargo-machete：“fast (yet imprecise)”，不需要编译；build.rs 生成的代码、包名和导入名不一致的 crate 会造成误报；`--with-metadata` 更准但可能改动 Cargo.lock 【官】https://github.com/bnjbvr/cargo-machete
- 本项目【推】：PR 层跑 machete（快，用 stable）；周层跑 udeps（nightly，更准）。两者都只给警告，不阻塞合并。

### 1.9 cargo-semver-checks

- 读 rustdoc JSON 检查 API 变动是否违反 semver；会漏检一部分违规（字段类型变化、泛型或生命周期变化、只在部分 feature 组合下才出现的破坏）；不需要发布，可以用 `--baseline-rev`；退出码 0 / 100 / 101 【官】https://github.com/obi1kenobi/cargo-semver-checks
- 本项目【推】：crate 都不发布到 crates.io，**唯一真正对外的 API** 是将来的插件接口和 C ABI（hal-plugin 和共享内存协议）。对这一类，semver-checks 帮不上（它查 Rust API，不查 C ABI 和内存布局）。做法：共享内存协议结构体用 `#[repr(C)]`，加 `const _: () = assert!(size_of::<Header>() == N)` 静态断言，再加协议版本号字段，这比 semver-checks 更贴题。semver-checks 放进「将来要发布 crate」的待办里。

### 1.10 rustfmt / clippy / `[workspace.lints]`

- `[workspace.lints]` 在根 `Cargo.toml` 统一配置，成员 crate 写 `[lints] workspace = true` 继承；Cargo 1.74 起生效 【官】https://doc.rust-lang.org/cargo/reference/workspaces.html
- clippy.toml 的 `disallowed-methods` / `disallowed-types` / `disallowed-macros` 支持 `path` / `reason` / `replacement` 字段；`allow-unwrap-in-tests` 等选项让测试代码可以放宽 【官】https://doc.rust-lang.org/clippy/lint_configuration.html
- clippy 没有「一条 lint 抓住所有 panic」，只能组合 `unwrap_used`、`expect_used`、`panic`、`indexing_slicing`、`arithmetic_side_effects` 等 【传】https://github.com/rust-lang/rust-clippy/issues/12754 、https://emschwartz.me/your-clippy-config-should-be-stricter/
- 实例：
  - DeepFilterNet：`rustfmt.toml` 设 `max_width=100`、`group_imports="StdExternalCrate"`（这个选项要 nightly rustfmt），CI 里 fmt 和 clippy 用 nightly 【源】`$RES/DeepFilterNet/rustfmt.toml:1-4`、`$RES/DeepFilterNet/.github/workflows/rust_lint.yml:20-22`；`clippy.toml` 只设了 `msrv` 【源】`$RES/DeepFilterNet/clippy.toml:1`
  - cpal：clippy 按 target 矩阵跑（包括 `aarch64-apple-darwin`，`--all-features` 和默认 feature 各跑一遍），`-D warnings` 【源】`$RES/cpal/.github/workflows/quality.yml:33-143`；`.githooks/pre-commit` 本地跑 fmt 和 clippy 【源】`$RES/cpal/.githooks/pre-commit`
- **本项目建议**【推】：
  - rustfmt 只用 stable 选项（避免为了 fmt 被迫装 nightly），这份配置全工作区共用
  - `[workspace.lints.rust]`：`unsafe_op_in_unsafe_fn = "deny"`、`missing_docs` 只对公开接口 crate 开
  - `[workspace.lints.clippy]`：`unwrap_used`、`expect_used`、`panic`、`todo`、`indexing_slicing` 全局设 warn；**rt 路径的 crate 或模块**（dsp 的 process 函数、engine 的实时线程函数、io-macos 的回调）在模块头用 `#![deny(...)]` 升成 deny
  - 每个 rt crate 各放一份 clippy.toml（clippy 从 crate 目录向上找配置，这是常识，**本次没有找到官方原句**）：`disallowed-methods` 列 `Vec::push`、`Vec::with_capacity`、`Box::new`、`String::from`、`std::sync::Mutex::lock`、`std::thread::sleep`、`std::fs::*`、`println!`/`dbg!`（放在 disallowed-macros 里）；`disallowed-types` 列 `std::sync::Mutex`、`std::collections::HashMap`
  - 规则再细（比如「被 `#[rt]` 标记的函数里不准调用没被 `#[rt]` 标记的函数」）：用 **dylint** 自写 `LateLintPass` 【官】https://github.com/trailofbits/dylint ——这就是「用语法树把规则做成模块」最直接的落点

### 1.11 cargo-llvm-cov（覆盖率）

- `cargo llvm-cov nextest` 内部调 nextest；doctest 和分支覆盖要 nightly；可以输出 lcov / codecov / json / cobertura / html；`#[coverage(off)]` 不稳定，要配合 `cfg(coverage_nightly)` 用 【官】https://github.com/taiki-e/cargo-llvm-cov
- 本项目【推】：覆盖率**不设硬门槛**（音频项目的覆盖率数字意义有限），只看 dsp / engine 的增量覆盖，周层跑

### 1.12 运行期实时安全：assert_no_alloc · RTSan · no-panic · TSAN/ASAN

- **assert_no_alloc**：作为 global allocator，`assert_no_alloc(|| …)` 闭包里禁止分配，`permit_alloc` 局部放行；默认 `disable_release`（release 构建里什么都不做）；默认违规时 abort，`warn_debug` / `warn_release` 改成只打印 【官】https://docs.rs/assert_no_alloc/latest/assert_no_alloc/ 。只管经过 global allocator 的分配，锁和系统调用它看不到【推】
- **RealtimeSanitizer**：
  - rustc PR #147935「Add LLVM realtime sanitizer」2025-11-08 合入，里程碑 1.93 【官】https://github.com/rust-lang/rust/pull/147935
  - 语法：`-Zsanitizer=realtime`，函数上标 `#[sanitize(realtime = "nonblocking")]`（需要 `#![feature(sanitize)]`），还有 `"blocking"` 和 `"caller"` 两个值 【官】https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html
  - clang 文档：在 nonblocking 函数里调用 `malloc`、`free`、`pthread_mutex_lock` 会被拦截报错；运行期开销「negligible」【官】https://clang.llvm.org/docs/RealtimeSanitizer.html
  - 稳定版可用的替代：`rtsan-standalone` crate，提供 `#[nonblocking]` 宏，支持 Linux / macOS / iOS，默认下载预编译库，`RTSAN_ENABLE=1` 开启 【官】https://github.com/realtime-sanitizer/rtsan-standalone-rs 、https://docs.rs/rtsan-standalone
  - 演讲：CppCon 2024 “LLVM’s Realtime Safety Revolution: Tools for Modern Mission Critical Systems”（Chris Apple、David Trevelyan）【传】日程见 https://cppcon2024.sched.com ，视频链接 https://www.youtube.com/watch?v=KvhgNdxX6Uw 来自二手博客，本次**没有核实视频页**；ADC 2024 有同题的音频版（Chris Apple、Josip Ćavar）【传】，没找到可核实的视频 URL
  - **对本项目这是最关键的工具**【推】：它能直接回答「实时线程有没有调用会阻塞的函数」，连 ort 内部（C++）的 malloc 和锁都能看到，因为它拦截的是 libc 符号——这一层 assert_no_alloc 看不到。用法：engine 的帧处理函数标 nonblocking，用 eval 回放一段 WAV 跑完整链路；测试跑在 macOS arm64 上（本机或 CI 的 macOS job）
- **no-panic**（dtolnay）：编译器证明不了函数不 panic 就报链接错误；要开优化，必须是 `panic="unwind"`（在 `panic="abort"` 下无效），`cargo check` 触发不了 【官】https://github.com/dtolnay/no-panic 。本项目【推】：只用在小而纯的 DSP 内核上（biquad tick、增益平滑），放在 release 构建的检查 job 里；对 engine 整帧函数不现实（它调用 ort）
- **TSAN / ASAN**：rustc 支持 `aarch64-apple-darwin` 上的 ThreadSanitizer 和 AddressSanitizer 【官】unstable-book sanitizer 页；cpal 在 macOS 和 Linux 上各跑 ASAN、TSAN（`-Zbuild-std`）【源】`$RES/cpal/.github/workflows/sanitizers.yml:21-82`；rtrb 在 Linux 上跑 TSAN 【源】`$RES/rtrb/.github/workflows/main.yml:132-151`
- 社区做法：几个音频 crate 用「线程局部计数分配器 + 稳态 N 个周期分配数为 0」做测试闸门（比如 timestretch、audio-graph-bsd 的 `rt_alloc_free.rs`）【传】https://docs.rs/crate/audio-graph-bsd/latest/source/tests/rt_alloc_free.rs 。要点：只在测量窗口里计数，并且要先预热（第一帧允许分配）

### 1.13 补充：cargo-hack、测试组织

- cargo-hack 的 `--each-feature` / `--feature-powerset` 能保证 `ort` / `tract` 两个后端 feature 单开、合开都编得过 【传】https://github.com/taiki-e/cargo-hack （README 本次没有重新抓取）
- matklad 的测试建议【官·作者博客】：
  - 「test features, not code」；用 `check` 函数集中调用 API；「value in, value out」的数据驱动测试；把 I/O 推到边缘 https://matklad.github.io/2021/05/31/how-to-test.html
  - 每个 `tests/*.rs` 是一个单独的二进制，大项目应该只留一个集成测试 crate、下面分模块（Cargo 自身这样改后编译时间降到约 1/3）https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html
  - 本项目【推】：eval crate 本身就是「数据驱动测试驱动器」，回归测试写成「清单（WAV, 参数）→ 期望指标」的表；每个 crate 只有一个 `tests/it/main.rs`
- 快照测试：`expect-test`（rust-analyzer 出品，`UPDATE_EXPECT=1` 原地更新）或 `insta` 【官】https://github.com/rust-analyzer/expect-test 。适合 DSP 配置解析、延迟报告这类文本输出；音频数据不适合用文本快照，见 §3

---

## 2. CI 分层惯例（社区实例 → 本项目方案）

### 2.1 社区实例（本机可证）

| 项目 | 快 | 中 | 慢 / 定时 | 出处 |
|---|---|---|---|---|
| rtrb | test（mac / win / linux 矩阵），fmt + clippy `-D warnings` + rustdoc `-D warnings` + `cargo package` | MSRV check、no_std（cargo-nono）、**Miri ×3**、**TSAN** | 只做 bench 编译测试 | `$RES/rtrb/.github/workflows/main.yml:8-165` |
| cpal | fmt、按 target 矩阵跑 clippy（含 `aarch64-apple-darwin`）、docs | ASAN / TSAN（macOS + Linux），**会跑 `beep` 示例**（Linux 上走 `alsa:null` 设备） | — | `$RES/cpal/.github/workflows/quality.yml`、`sanitizers.yml:92-115` |
| DeepFilterNet | fmt / clippy（nightly）、test、按包 build | — | **每日 cron**（`rust_lint.yml:5-6`）；**每周 cron 跑端到端**：Python 和 Rust 二进制分别处理同一个 `noisy_snr0.wav`，用 DNSMOS 卡阈值 | `$RES/DeepFilterNet/.github/workflows/rust_lint.yml`、`test_df.yml:16-17,131-153` |
| nnnoiseless | build / test | **C 参考 vs Rust 输出逐字节 `diff`** | — | `$RES/nnnoiseless/.github/workflows/rust.yml:27-33` |
| coreaudio-rs | 只用 macOS-latest 跑 test / clippy / fmt | — | — | `$RES/coreaudio-rs/.github/workflows/coreaudio-rs.yml` |
| rubato | check / test / fmt / clippy，外加 `ubuntu-24.04-arm` | — | — | `$RES/rubato/.github/workflows/ci_test.yml:8-47` |

观察【推】：几个都没用 nextest / cargo-deny / loom / 覆盖率；**分层的核心是「把 nightly 和慢东西挪到单独的 job 或 cron」**。DeepFilterNet 的 macOS job 被注释掉了（“Fails to install hdf5 from git”，`test_df.yml:27`），说明 macOS CI 维护成本高。

### 2.2 macOS GitHub Actions runner 的限制

- arm64 runner（`macos-latest` / `macos-14` / `macos-15` / `macos-26`）规格是 3 vCPU、7 GB 内存、14 GB SSD，公开仓和私有仓规格相同；Intel 的只剩 `macos-15-intel` / `macos-26-intel`；arm64 上**不支持嵌套虚拟化** 【官】https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- 价格：macOS 3/4 核 $0.062/分钟，Linux 2 核 $0.006/分钟；GitHub Free 的私有仓每月含 $12 额度；公开仓的标准 runner 免费 【官】https://docs.github.com/en/billing/concepts/product-billing/github-actions
  - ⇒ 私有仓每月的 $12 只够约 190 分钟 macOS（或约 2000 分钟 Linux）【推，按官方单价计算】
- **音频设备**：有的 runner 没有音频设备（vim 在 macos-12 上遇到过，按需 `brew install blackhole-2ch`）；macos-15 有「Null Audio Device 偶发初始化失败」的报告；另有报告说 macos-15 的 coreaudiod 拒绝加载 ad-hoc 签名的 HAL 插件，要 Developer ID 签名 【传】https://github.com/vim/vim/issues/14715 、https://docs.rs/crate/tympan-aspl/latest/source/docs/ja/testing.md
  - ⇒ **在 CI 上不要测真设备，也不要测自写的 HAL 插件加载**；io-macos 在 CI 上只做编译和单元测试（设备枚举 mock 掉）【推】
- 自托管 runner：免费；官方建议只在私有仓上用（公开仓的 fork PR 可能在你的机器上执行代码）【传，GitHub security-hardening 文档，本次未重新抓取】→ 本项目现在是私有仓，可以把本机 Mac 注册成自托管 runner 跑 soak 和设备测试；**仓库改公开前必须撤掉或加审批**【推】

### 2.3 Python ↔ Rust 逐值对拍（parity test）

**两个参照实例**
- **DeepFilterNet = 「同一份代码」而不是「对拍」**：Python 侧的 `df/enhance.py:22` 写 `from libdf import DF, erb, erb_norm, unit_norm`，特征和 STFT 直接调用 Rust（pyDF 是 pyo3 cdylib）【源】`$RES/DeepFilterNet/DeepFilterNet/df/enhance.py:22`、`$RES/DeepFilterNet/pyDF/Cargo.toml:1-16`。因此训练和推理的前后处理天然一致。端到端靠指标卡阈值：`test_df.py` 把 composite / STOI / SI-SDR 和写死的目标值比较，`atol=rtol=1e-4` 【源】`$RES/DeepFilterNet/DeepFilterNet/df/scripts/test_df.py:18-19,38`；CI 里 Rust 二进制增强后的 wav 也用 DNSMOS 卡阈值 【源】`test_df.yml:149-153`
  - 顺带发现：libDF 的 `test_stft_istft_delay` 里 `yy` 用的是 `ich`，不是 `och`（`transforms.rs:629`），所以那个相关系数检验实际上弱于它的本意 【源】`$RES/DeepFilterNet/libDF/src/transforms.rs:627-633` ——**教训：对拍测试本身也要被审**
- **nnnoiseless = 对参考实现严格对拍**：CI 编出 C 的 `rnnoise_demo`，和 Rust 版处理同一份 `testing.raw`，然后 `diff out_rust.raw out_c.raw` 【源】`$RES/nnnoiseless/.github/workflows/rust.yml:27-33`；单元测试里用 `include_bytes!` 引入参考输出，相对平方误差 `< 1e-4`，并且跳过第一帧（延迟对齐）【源】`$RES/nnnoiseless/src/lib.rs:174-205`

**本项目做法**【推】
1. **Golden 向量由 Python 侧生成**（GTCRN 官方推理脚本 + onnxruntime CPU 后端），逐级落盘：输入 PCM → STFT 帧 → 模型输入张量 → 模型输出（mask / 复谱）→ iSTFT 输出 → 门控后输出。存成 `.npy` 或原始 f32 小文件（一段 2–3 s 片段即可，单文件 < 1 MB，可以进仓；大文件按铁律 8 只留路径和哈希）
2. **生成脚本、ORT 版本、模型哈希写进 manifest**，提交进 `docs/experiments/...` 或 `tests/golden/`；Rust 测试先校验哈希，再比数值
3. **分级容差**：STFT / iSTFT 用 `atol 1e-6`（f32）；模型输出 ort 对 ort 用 `1e-5`、tract 对 ort 用 `1e-4`（不同 kernel 的累加顺序不同）；整链用 SI-SDR 对比 golden ≥ 60 dB，这比逐点比较更稳。**具体阈值要先实测再定**，不要先拍脑袋
4. **流式 vs 整段**：GTCRN 有状态（缓存的卷积和 GRU 状态），Python 的整段推理和 Rust 的逐帧推理必须先对齐延迟和首帧，再比较（nnnoiseless 跳过首帧就是这个原因）
5. **三方对拍**：Python ORT ↔ Rust ort ↔ Rust tract，任意两两都比较；任何一方升级版本都要重跑
6. 声纹线程：ECAPA / CAM++ 的 embedding 比余弦相似度（≥ 0.999），不比逐值

---

## 3. 本项目「闸门 → 工具 → 何时跑 → 失败意味着什么」表

> 层：**L0** 本地提交前（hook，几秒）· **L1** 每个 PR（Linux 为主，目标 < 10 分钟）· **L2** 每个 PR 中速层或合并到 main（含 nightly / 慢工具）· **L3** 夜间或每周 cron · **L4** 本机 Mac 手动或自托管（设备、soak、人耳）

| # | 闸门 | 工具 / 命令 | 何时跑 | 平台 | 失败意味着什么 |
|---|---|---|---|---|---|
| G1 | 格式 | `cargo fmt --all --check`（只用 stable 选项） | L0 + L1 | Linux | 风格不一致；自动修，不讨论 |
| G2 | 静态 lint | `cargo clippy --workspace --all-targets --all-features -D warnings`，配 `[workspace.lints]` | L0 + L1 | Linux + macOS 一个 job（`aarch64-apple-darwin`，覆盖 io-macos） | 有可疑写法；rt 模块里出现 `unwrap` / 索引 / panic 宏 ⇒ **实时线程可能 panic** |
| G3 | **实时禁用 API（AST 层）** | 各 rt crate 的 clippy.toml `disallowed-methods/types/macros`；后期加 dylint 自写 lint | L1 | Linux | rt 路径调用了会分配 / 加锁 / 做 I/O 的 API ⇒ **违反实时铁律，拒绝合并** |
| G4 | 编译矩阵 | `cargo hack check --each-feature`（ort / tract 后端各单开一次）+ `cargo check` MSRV | L1 | Linux | 某种后端组合编不过 ⇒ GTCRN / FastEnhancer 或 ort / tract 不能切换 |
| G5 | 单元 + 集成测试 | `cargo nextest run --profile ci` + `cargo test --doc` | L1 | Linux（dsp / engine / speaker / eval）；macOS job 只跑 io-macos | 功能回归 |
| G6 | **分块不变性 · 性质测试** | proptest：随机切块 vs 整段，外加 §1.4 列的性质 | L1（case 数少）/ L3（case 数多） | Linux | **eval 和实时走的路径不一致**，或者 DSP 有状态泄漏 ⇒ 离线评测分数不能代表实时效果 |
| G7 | 依赖治理 | `cargo deny check`（licenses / advisories / bans / sources）+ 资产许可清单脚本 | L1（PR 改了 Cargo.lock 时）+ L3 每日（新公告） | Linux | 许可冲突 ⇒ **不能按 GPL-3.0 分发**；有安全公告 ⇒ 要升级或写理由；bans 违规 ⇒ **模块边界被打破**（比如 engine 直接依赖了 cpal） |
| G8 | 零分配 | 计数分配器测试（`assert_no_alloc` 或自写），engine 预热后稳态跑 N 帧，分配数为 0；divan `AllocProfiler` 基准也断言 0 | L1 | Linux + macOS | **实时线程在分配内存** ⇒ 会有偶发爆音 / 掉帧 |
| G9 | UB / 未定义行为 | `cargo +nightly miri nextest run -p dsp -p engine-sync`（+ `-Zmiri-tree-borrows`、`-Zmiri-many-seeds`） | L2 | Linux | `unsafe` 写错了，或者自写的原子协议有数据竞争 ⇒ **潜在崩溃或静默数据损坏** |
| G10 | 并发模型检查 | `RUSTFLAGS="--cfg loom" cargo test -p engine-sync --release`，`LOOM_MAX_PREEMPTIONS=3` | L2 | Linux | 声纹结果发布或共享内存索引协议在某种交错下会读到撕裂值 ⇒ **偶发错误增益 / 杂音，现场极难复现** |
| G11 | 真线程竞争 + 内存错误 | TSAN / ASAN（nightly，`-Zbuild-std`） | L2 | Linux + macOS（参照 cpal 做法） | 真实线程（含 FFI）上的竞争或越界 |
| G12 | **实时阻塞调用** | RTSan：`-Zsanitizer=realtime` + `#[sanitize(realtime="nonblocking")]`（或 stable 上用 `rtsan-standalone`），用 eval 回放整链 | L2 | **macOS arm64**（也可在 Linux 上跑一份） | 实时帧函数里（包括 ort / C++ 内部）出现 malloc / free / mutex / 系统调用 ⇒ **证据直指「音频线程不阻塞」这条约束被违反** |
| G13 | 链接期不 panic | `#[no_panic]` 标在 DSP 内核上，release 构建 | L2 | Linux | 编译器证明不了内核不 panic ⇒ 加边界检查前置或改写 |
| G14 | Python 对拍 | golden 向量（§2.3），Rust ort / tract 逐级对比 | L1（小片段）+ 升级 ort / tract / 模型时强制跑 | Linux | Rust 推理和参考实现不一致 ⇒ **听到的不是论文里的那个模型** |
| G15 | 模糊测试 | `cargo +nightly fuzz run <target> -- -max_total_time=600` | L3 | Linux | 解析器或 DSP 在恶意 / 异常输入下 panic 或输出 NaN ⇒ 另一个进程写坏共享内存时，实时线程会被带崩 |
| G16 | 指令数回归 | Gungraun（Valgrind，**只能在 Linux 上**）测 `process_frame` | L3（或 L2，带阈值） | Linux | 每帧计算量涨了 X% ⇒ **实时余量被吃掉**，接近 45 ms 预算 |
| G17 | 未用依赖 / 覆盖率 | `cargo machete`（L1 警告）、`cargo +nightly udeps`（L3）、`cargo llvm-cov nextest`（L3） | L1 / L3 | Linux | 依赖腐烂；没测到的代码。只警告，不阻塞 |
| G18 | 评测集回归 | eval crate 跑固定评测集 → DNSMOS / PESQ / STOI / SI-SDR 和 baseline 比（参照 DeepFilterNet `test_df.yml:144-153` 写死阈值） | L3 每周 + 改模型 / DSP 参数时 | Linux（只用 CPU） | 客观指标退化 ⇒ 降噪变差或音色被破坏；**指标通过 ≠ 人耳通过**（D33 的教训，最终交人判） |
| G19 | 时延与截止时间 | 本机实测：每帧耗时 p50 / p99 / p99.9 / max 对比帧周期；端到端附加延迟（环回法）≤ 45 ms | L4（每次改 engine / 升级 ort 时） | 本机 Mac | 超出预算 ⇒ 掉帧或延迟违约。**CI 虚拟机的数值不可信**（criterion FAQ） |
| G20 | Soak / 设备 | BlackHole 或真设备连续跑 ≥ 1 h：计 xrun 数、漂移补偿状态、内存曲线；配合 Instruments / `log stream` | L4 夜间（自托管）或发版前 | 本机 Mac | 长时漂移、内存泄漏、设备热插拔问题 ⇒ 用户现场才会遇到的 bug |
| G21 | 人耳验收 | 盲听页（见 CLAUDE.md §五 揭盲表规则） | 里程碑 | 人 | **验证红线**，不能交给机器 |

### 3.1 macOS CI job 的最小集（控制费用）【推】
只在一个 `macos-latest` job 里依次跑：G2（clippy `aarch64-apple-darwin`）→ G5（只跑 io-macos 和 hal-plugin 的单元测试，设备 mock 掉）→ G8 → G12（RTSan 回放）。其余全部放在 Linux 上。按官方单价，每次约 6–8 分钟 ≈ $0.4–0.5；如果每月 PR 多，就改成「仅在动了 io-macos / hal-plugin / engine 时触发」（`paths:` 过滤），或者全部转到自托管的本机。

### 3.2 结构上的前提（不满足的话，以上闸门大半跑不起来）【推】
1. **dsp / engine / speaker / eval 不得依赖任何 macOS 专有 crate**（由 G7 的 bans.wrappers 机器保证）
2. **自写的同步原语集中放在一个小 crate 或模块里**（比如 `engine-sync`），用 `cfg(loom)` 切换原子类型；Miri、loom、TSAN 都盯这一个地方
3. **engine 对外只暴露 `process_frame(&mut self, in, out)` 这类纯计算接口**，I/O 在边缘（matklad「value in, value out」），这样 eval、proptest、RTSan 回放、golden 对拍都能直接调用
4. 推理后端（ort / tract）放在 trait 后面的独立 crate 里，测试可以注入 `MockModel`（固定增益），使 dsp / engine 的大多数测试不需要加载 ONNX
5. 共享内存协议用 `#[repr(C)]` 加 `size_of` 静态断言、带版本字段，hal-plugin（C 写的）那侧也照同一份头文件断言

---

## 4. 和「AST / 模块化」那句话的对应（不硬套，也不否掉）

人原话：「用超级语法树 / ast 来把东西做成模块……可拆分的核心包和模块」。在质量闸门这一侧，它能这样落地【推】：
- **AST 级规则就是 lint**：clippy 的 `disallowed-*` 是现成的；dylint 让我们写自己的 `LateLintPass`（跑在 HIR 上，是比 AST 更晚一层、带类型信息的表示）【官】https://github.com/trailofbits/dylint 。适合写死本项目特有的规则：「标了 `#[rt]` 的函数只能调用标了 `#[rt]` 的函数」，这正是 RTSan 运行期检查在编译期的补集
- **crate 图就是模块边界**：cargo-deny 的 `bans.deny + wrappers` 能把「谁可以依赖谁」写成可以检查的规则；切分 crate 不只是代码组织问题，而是让每条闸门都有一个**刚好够小的检查对象**（Miri 只跑 `engine-sync`，loom 只跑 `engine-sync`，RTSan 只盯 `engine::process_frame`）
- 不建议的方向：自建「超级语法树」代码生成框架，或者用宏重写整个 DSP——维护成本高，现成工具已经覆盖了需求【推】

---

## 5. 没找到出处 / 留待实测

- ONNX Runtime 在 `intra_op_num_threads=1`、同步调用时，稳态 `Run()` 是否零 malloc：**未找到出处**，要用 G12（RTSan）实测。这是整个「音频线程不分配」主张里风险最高的一环；如果不满足，就要考虑 tract（纯 Rust，可以用计数分配器验证）或者预分配的 IoBinding
- RTSan 在 Rust 稳定版上的进展（nightly 之外）：只确认了 PR 合入和 unstable 标记，**稳定化时间未找到出处**
- Rust 大会（RustConf / RustLab / Rust Nation）上专讲「实时音频 + 质量闸门」的演讲：**未找到出处**；最接近的是 CppCon / ADC 2024 的 RTSan 演讲（【传】见 §1.12）
- ECAPA / CAM++ 预训练权重的许可：没有逐个模型卡核对
- 本机工具实况【测】：stable 1.98.1 / 1.99.1（aarch64-apple-darwin）；没有 nightly、nextest、cargo-deny、llvm-cov、fuzz、machete、valgrind。L2 层开工前要先装 nightly 和这几样工具（按 CLAUDE.md 先过 `scripts/check-tools.sh`）
