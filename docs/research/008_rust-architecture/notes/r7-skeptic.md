# R7 · 怀疑派复核：r1–r6 关键结论逐条证伪

> 2026-10-30 · research/008-rust-architecture · 复核对象：同目录 `r1-ast.md` … `r6-quality.md`
> 证据等级：【官】官方明载 /【源】源码可证 /【测】本机实测 /【传】二手 /【推】推断
> 判词：**成立** / **部分成立**（附改法）/ **不成立** / **无法核实**
> `R=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources`；本轮实验探针放在 scratchpad：`clipprobe/`（clippy 宏/方法禁用）、`clipws/`（按 crate 放 clippy.toml）、`lintws/`（workspace lints 覆盖与 forbid）。都是一次性的，没有进仓。
> 本机 rustc/clippy 1.99.1 (aarch64-apple-darwin)，没有 nightly、ast-grep、dylint 或 cargo-deny 二进制。

---

## 0. 一页结论：要改的地方（按严重度）

| # | 问题 | 涉及笔记 | 判词 |
|---|---|---|---|
| P0-1 | **cargo-deny `wrappers` 只查「直接依赖边」**：`engine → io-macos → cpal` 能通过检查。所以「`cpal` wrappers=io-macos ⇒ eval/engine 不可能依赖设备层」是错的 | r1 §2.3、r6 §1.7/§3.2、r2 §2.4（最后一行「送音进程不链推理库由 wrappers 证明」） | 不成立（r2 §5 自己写了「传递依赖仍要靠 check-deps」，这句是对的） |
| P0-2 | **crate 自带的 clippy.toml 会整份替换上层配置，不会合并**：在 rt crate 里放「严格版」，根目录的禁用清单在这个 crate 里就失效了 | r4 §2.4、r6 §1.10 | 部分成立（能分 crate 配，但必须写全） |
| P0-3 | **`unsafe_code = "forbid"` 加「FFI crate 局部覆盖」写不通**：`lints.workspace = true` 之后不能再覆盖某一条；forbid 之下 `#[allow]` 直接报 E0453 | r1 §2.2、r4 §5.1 | 不成立（改用 `deny`，或者让 FFI crate 不继承 workspace lints） |
| P1-1 | rustc RTSan PR #147935 **已于 2025-11-08 合入，里程碑 1.93**，不是「还在评审」 | r1 §2.7 | 不成立（过期） |
| P1-2 | ADC 2024 RTSan 演讲的讲者是 **David Trevelyan 与 Chris Apple**，**不是** Josip Ćavar | r4 §2.2、r6 §1.12 | 不成立（讲者张冠李戴） |
| P1-3 | `R/fastenhancer` **就是** FastEnhancer 官方源码（aask1357/fastenhancer），不是项目骨架 | r3 §2.4 / §8 | 不成立 |
| P1-4 | tympan-aspl 说「列 `std::io::stdout` 就能顺带拦住 `println!`」——**拦不住**；`disallowed-macros = std::println` 可以直接用 | r1 §2.1 引用的 tympan 注释 | 不成立（tympan 注释有误，r1 的怀疑是对的） |
| P1-5 | rtsan-standalone 0.3.0 的 README 写自编库的环境变量叫 `RTSAN_LIBRARY_PATH`，build.rs 实际读的是 **`RTSAN_LIB_PATH`** | r4 §2.2 | 部分成立（照 README 写会被静默忽略） |
| P2 | 其余 15 条：成立，或只需小改措辞（见 §2） | — | — |

---

## 1. 方法

- 本机能看的先看本机：源码、行号、`git remote`；有歧义的就写最小探针直接跑。
- 网页回到原 URL 重新抓。YouTube 用 oEmbed（`youtube.com/oembed?url=…`）核对每个视频 ID 的标题和频道，这样能挡住编造的 ID。
- 没有跑 cargo-deny、ast-grep、dylint（本机没装）。这几项只核了文档和源码，没有【测】。

---

## 2. 逐条复核

### 2.1 工具能力类

**C1 · assert_no_alloc：只拦经过 Rust 全局分配器的分配；release 默认不生效** — 成立
- `R/../~/.cargo/registry/src/*/assert_no_alloc-1.1.2/Cargo.toml`：`default = ["disable_release"]`【源】；`src/lib.rs` 里 `GlobalAlloc` 只实现了 `alloc`/`dealloc`，`check()` 判的是 thread_local 的 `ALLOC_FORBID_COUNT`，违规时走 `handle_alloc_error`（abort）【源】。
- 推论「C/C++ 库（onnxruntime）直接调 libc malloc 拦不到」：机制上成立，`#[global_allocator]` 管不到 FFI 分配【推，机制明确】。
- 补一条：计数器是线程局部的，**只管调用线程**。ort 自己的线程池里发生的分配也看不见（r6 §1.6 对 divan 提过同样的问题，这里也适用）。

**C2 · RTSan（Clang）能拦什么** — 部分成立
- Clang 文档只举了 `malloc`、`free`、`pthread_mutex_lock` 三个例子，再加一句「anything else known to have a non-deterministic execution time」，**没有完整清单**；「runtime slowdown … is negligible」原文确有，但没给测量数据；文档**没有写支持平台**【官】https://clang.llvm.org/docs/RealtimeSanitizer.html
- 改法：r6 §0 写「能抓 malloc / free / mutex 等」可以保留；「能抓系统调用」一律加上「清单未公开，按项实测」（r4 §2.2 已经这样写了，r6 §1.12 没写）。
- 「拦 libc 符号，所以 onnxruntime 内部的 malloc 也能抓到」：目前仍是【推】，要等实验 E1 证实。r6 §1.12 写成了肯定句（「连 ort 内部…都能看到」），应降级为【推】。

**C3 · rustc 内置 RTSan（PR #147935）** — r6 成立，r1 不成立
- PR 页面：标题「Add LLVM realtime sanitizer」，2025-11-08 由 bors 合入，里程碑 1.93.0【官】https://github.com/rust-lang/rust/pull/147935
- Unstable Book：`-Zsanitizer=realtime`；`#[sanitize(realtime = "nonblocking" | "blocking" | "caller")]`，`caller` 是默认值；需要 `#![feature(sanitize)]`；被归入「intended for testing or fuzzing (but not production use)」；**没有列 supported targets**【官】https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html
- 改法：r1 §2.7「截至检索时仍在评审」改为「已合入 nightly（1.93 起），仍是 unstable」；r6 §1.12「macOS 支持」只对 rtsan-standalone 成立（README 原文），**rustc 内置版的目标平台未找到出处**。

**C4 · rtsan-standalone 0.3.0 的细节** — 大部分成立，有一处 README 与代码不一致
- README:7「only works on Linux, macOS and iOS」；README:110 作者 Stephan Eckes【源】`R/rtsan-standalone-rs/README.md`
- `RTSAN_ENABLE` 是**编译期**变量：build.rs 读它，并设 `rerun-if-env-changed`；不设就直接 `return`，不加 `cfg(rtsan_enabled)`【源】`crates/rtsan-standalone-sys/build.rs:9,32,40-47`。r4 的说法成立。
- `#[nonblocking]` 的实现：在函数体最前面插一个 RAII guard `ScopedSanitizeRealtime`【源】`crates/rtsan-standalone-macros/src/lib.rs:22-38`。r1 写的「改写函数体、进入/退出时通知运行时」成立。guard 最后才 drop，所以函数体里局部变量 drop 时的 `free` 也算在实时上下文里【推】。
- **新发现**：README:74,79 说自编库用 `RTSAN_LIBRARY_PATH`，build.rs:54 实际读 `env::var("RTSAN_LIB_PATH")`（本机 clone 和 registry 里的 0.3.0 都是这样）【源】。照 README 设变量会被静默忽略，然后去下载预编译库。r4 §2.2 照抄了 README，应改为「以 build.rs 为准，用 `RTSAN_LIB_PATH`」。
- r4 实测「macOS 二进制没有 LC_RPATH，要用 cargo run/test」：和源码一致。build.rs:217 用 `cargo:rustc-link-arg=-Wl,-rpath,…`，而这种 link-arg 只作用于该 -sys 包自己的目标，不会传给下游二进制【推，依据 Cargo build-script 文档对 `rustc-link-arg` 的作用域说明】。成立。

**C5 · dtolnay/no-panic** — 成立
- README（registry `no-panic-0.1.37/README.md:70-106`）：`cargo check` 或只构建 lib 时不触发；「useless in code built with `panic = "abort"`」；建议 dev 用 `opt-level = 1`；跨 crate 可能要 thin LTO，再不行就 fat LTO 加 `codegen-units = 1`【源】。
- r4 实测「编译期已知等长的调用方会让 release 通过」是一个重要警告（证明的是这次构建的这个调用图，不是函数本身的性质），没有反证，保留。

**C6 · clippy `disallowed-macros` 能否禁 `println!`；tympan 注释对不对** — tympan 注释不成立，r1 的怀疑成立【测】
- 探针 `clipprobe/`：`disallowed-macros = [{path="std::println"}, {path="alloc::vec"}]` 都命中；`disallowed-methods = [{path="std::io::stdout"}]` **没有**在 `println!` 上命中。原因：`println!` 展开后调用的是 `std::io::_print`，不是 `stdout()`【推，命中结果是【测】】。
- 所以 tympan-aspl 的 `R/tympan-aspl/clippy.toml:58-60`「`std::io::stdout` 能拦住它们的展开目标」**不对**。它的禁用清单在 `println!` 上有个洞。我们写 clippy.toml 时要把 `std::println`、`std::eprintln`、`std::print`、`std::eprint`、`std::dbg`、`std::format`、`alloc::vec` 都列进 `disallowed-macros`。
- 同一探针：`std::vec::Vec::push` 这个路径**能**命中。r4 §2.4「必须写定义路径 `alloc::vec::Vec::push`」偏保守，两种写法都能用【测】。

**C7 · clippy.toml 按 crate 目录查找 → 可以给 RT crate 单独配严格版** — 部分成立（有坑）【官】+【测】
- 官方 configuration 页：查找顺序是 `CLIPPY_CONF_DIR` → `CARGO_MANIFEST_DIR` → 当前目录，找不到就往上逐级找，「until it finds one」【官】https://doc.rust-lang.org/clippy/configuration.html 。r5 §2 说「未读到官方原文」，现在已核到。
- 探针 `clipws/`：根 clippy.toml 禁 `thread::sleep`，`crates/rt/clippy.toml` 禁 `Vec::push`。结果 `rt` 里的 `thread::sleep` **没有报**，只有 `nrt` 报了。也就是说**取最近的一份，不合并**【测】。
- 改法：「每个 RT crate 放自己的 clippy.toml」可以做，但这份必须**包含**全局禁用清单。推荐用 xtask 从一份真相源生成各 crate 的 clippy.toml，再加测试检查一致（符合铁律 7 唯一编码）。或者干脆只用根目录一份，再靠模块级 `#![deny(clippy::disallowed_*)]` 控制作用域（tympan 的做法）。

**C8 · `[workspace.lints]` 不能在成员里逐条覆盖** — 成立【官】+【测】
- Bevy 根 Cargo.toml:93-102 的注释原文写了这一点，并链到 cargo#13157【官】。
- 探针 `lintws/`：成员同时写 `workspace = true` 和 `rust.unsafe_code = "deny"`，cargo 报错 `cannot override workspace.lints in lints`【测】。

**C9 · 「workspace `unsafe_code = forbid` + FFI crate 局部放行」** — 不成立【测】
- 同一探针：继承 forbid 后在函数上写 `#[allow(unsafe_code)]`，报 `error[E0453]: allow(unsafe_code) incompatible with previous forbid`【测】。
- 改法（二选一）：① workspace 设 `unsafe_code = "deny"`（Bevy 就是这样，Bevy Cargo.toml:89），FFI crate 用 `#[allow]` 局部放行，非 FFI crate 在 `lib.rs` 加 `#![forbid(unsafe_code)]` 收紧（crate 内可以从 deny 升到 forbid）；② FFI crate 不继承 workspace lints，自己抄一份（Bevy 根包就是这么做的）。r2 §4 的 `deny` 写法本来就对；r1 §2.2 和 r4 §5.1 要改。

**C10 · cargo-deny `wrappers` 能守住依赖方向** — 不成立（被夸大）【官】+【源】
- 文档：wrappers「allows specific crates to have a direct dependency on the banned crate but denies all transitive dependencies on it」；不能和 `deny-multiple-versions` 同时用【官】https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html
- 源码才是关键：`R/cargo-deny/src/bans.rs:616-650` 对被禁 crate 只遍历 `direct_dependents`，检查每个直接父节点是否在 wrappers 名单里【源】。所以 `vdn-engine → vdn-io-macos → cpal` 时，cpal 唯一的直接父是 io-macos（在名单内），**不报错**。
- 后果：r1 §2.3「cpal wrappers=io-macos ⇒ eval 不可能偷偷依赖真设备」、r6 §1.7「dsp/engine 永远不依赖设备层」、r6 §3.2(1)「由 G7 的 bans.wrappers 机器保证」、r2 §2.4「送音进程依赖树里没有 ort/tract/engine 可以由 wrappers 证明」，**全都证不出来**。
- 正确做法：
  1. 把**内部 crate 本身**列进 deny，例如 `{ name = "vdn-io-macos", wrappers = ["vdn-capture"] }`、`{ name = "vdn-engine", wrappers = ["vdn-eval", "vdn-infer"] }`。这样 engine 若依赖 io-macos 就会报错。workspace 成员可以被 ban：`allow-workspace` 默认 `false`，源码在 `bans.rs:608-611`、`bans/cfg.rs:567`，文档在 `docs/src/checks/bans/cfg.md:131-145`【源】。本机没装 cargo-deny，**没有实测**。
  2. 「某个 bin 的整棵依赖树里不出现 X」这种传递性的不变量，用 r2 §5 的 `xtask check-deps`（读 `cargo metadata` 遍历依赖闭包）。**这一层不能省**。
- r1 §2.3 说「其他任何直接或传递依赖都报错」，是把文档里那句话读反了。

**C11 · ast-grep 能表达「在带 `#[nonblocking]` 的函数体内 `inside`…」** — 部分成立
- 规则 YAML 支持 `inside/has/precedes/follows`，带 `stopBy` / `field`；文档只讲语法树节点，**完全没提**类型和名字解析，也没提宏内部怎么匹配【官】https://ast-grep.github.io/reference/rule.html
- 坑【推】：在 tree-sitter-rust 里，外层属性 `attribute_item` 是 `function_item` 的**前一个兄弟节点**，不是它的子节点。所以「带属性的函数」要写成 `inside: { kind: function_item, follows: { kind: attribute_item, regex: nonblocking } }` 这样的组合，不能只写一个 `inside`。另外 `vec![]`、`format!` 的参数在 `token_tree` 里，模式匹配受限。r1 §2.10 的规则示例要先写好正例、负例 fixture 再上 CI。
- 「CodeRabbit 内置 ast-grep」「yog CI 先例」：没有复核，维持【传】。

**C12 · dylint：自定义 `LateLintPass`、要 nightly / rustc 内部 API** — 部分成立
- README 原文：「implement the LateLintPass trait」；`dylint_library!` 会加 `extern crate rustc_driver`；rust-analyzer 配置要 `rustc_private = true`；库在 `[workspace.metadata.dylint] libraries` 里声明（git/path，加 `pattern`）【官】https://github.com/trailofbits/dylint
- README **没有直接写**「需要 nightly」。用了 `rustc_private` 就意味着 nightly 工具链，这是【推】（理由很强）。r1 §2.11 标【官】偏高，改成【官（LateLintPass/声明方式）+推（nightly）】。
- 「暂缓、第三次漏网再上」这个取舍不受影响。

**C13 · iai-callgrind/Gungraun 在 macOS 上不可用** — 成立
- Homebrew valgrind formula：`depends_on :linux`，bottle 只有 `arm64_linux` / `x86_64_linux`【官】（本轮 curl 原文复核）。

**C14 · ort IoBinding「预分配输出、每次 run 覆盖」；没有声称零分配** — 成立
- 页面原文：输出「placed into the same memory allocation provided in the bind_output call」，后续 run「override the data」；**没说** `run_binding` 不分配；另有 `bind_output_to_device`，用于 shape 未知的情况（这时输出由 ORT 分配，实时路径要避开）【官】https://ort.pyke.io/perf/io-binding 。r4/r3「ort 能否进实时线程只能写【推】、等 E1」这个口径是对的。

**C15 · arc-swap 读端会给旧值减引用计数 ⇒ 实时线程可能 dealloc** — 成立（后半句是推论）
- 原文：「readers still need to decrement the reference counts of the old instance」；读「always lock-free… Most of the time… wait-free」【官】https://docs.rs/arc-swap/latest/arc_swap/docs/performance/index.html 。「读者可能成为最后一个持有者，因而 free」是 Arc 语义推出来的，页面没写【推】。r3 标注得正确。

### 2.2 演讲与出处类（防编造）

**C16 · 14 个 YouTube 视频 ID 是否存在、标题和讲者对不对** — 全部成立
oEmbed 逐个核对（标题 | 频道）：
- `G3WWr4uhP0I` Building Audio Apps with Rust… – Stephan Eckes – ADC 2024 | ADC
- `YLPglu2enaE` Cross-Platform Music Software with Rust – Ian Hobson – ADC 2025 | ADC
- `RcBnGiDSn6g` Writing Elegant DSP Code **in** Rust – Chase Kanipe – ADC23（r4 写成「with Rust」，小笔误）
- `Tof5pRedskI` Real-time Programming with the C++ Standard Library – Doumler – CppCon 2021
- `vn7563IAQ_E` C++ Standard Library for Real-time Audio – Doumler – ADC21
- `DHx9kv74Mlc` Doumler – Using Locks in Real Time Audio Processing, Safely | **The Audio Programmer**（不是 ADC 频道）
- `7fKxIZOyBCE` Thread Synchronisation… With RCU – Doumler ADC
- `ndeN983j_GQ` Real time 101 – Rowland & Renn-Giles – **Meeting C++ 2019**；`Q0vrQFyAdWI` / `PoZAo2Vikbo` Part I / II | ADC 频道
- `JG7lqt7V1uY` Real-time Confessions in C++ – Renn-Giles – ADC23
- `udlK1LQ3f3g` Bare Metal Audio Programming With Rust – van Gelder – ADC20
- `Yom9E-67bdI` An introduction to Rust for audio developers – Ian Hobson | ADC（年份仍未知）
- `KvhgNdxX6Uw` LLVM's Realtime Safety Revolution: Tools for Modern Mission Critical Systems – **CppCon 2024**。r6 标【传·没核视频页】，现在可以升为已核。
- 结论：**没有发现编造的视频 ID**。r4 §7「Real-time 101 年份待核」：Meeting C++ 2019 那一版已确认；ADC 那两部分的年份标题里没写，仍待核。

**C17 · ADC 2024 RTSan 演讲的讲者** — r5 成立，r4 和 r6 不成立
- 会议页：「LLVM's Real-Time Safety Revolution – RealtimeSanitizer and Performance Constraints Attributes」，2024-11-13，讲者 **David Trevelyan、Chris Apple**；摘要说 RTSan「first presented at ADC 2023」「coming to mainline Clang」；Performance Constraints 在编译期抓「nonblocking 调用 blocking」【官】https://conference.audio.dev/session/2024/llvms-real-time-safety-revolution-tools-for-modern-audio-development/
- r4 §2.2「ADC24 有 Josip Ćavar 与 Christopher Apple 讲」、r6 §1.12「ADC 2024 同题音频版（Chris Apple、Josip Ćavar）」：搜不到任何 Josip Ćavar 与 ADC 或 RTSan 相关的记录（检索结果全是同名无关的人）。**判不成立**，讲者应改为 Trevelyan & Apple。
- CppCon 2024 同题版（Apple & Trevelyan）：sched 描述与视频都已核到【官】。

**C18 · Eckes ADC 2024 演讲内容** — 成立
- 会议页摘要：讲 ai-coustics；「real-time-safe DSP inside the audio callback and thread-safe parameter management」【官】。
- 幻灯片抽取文本（scratchpad `eckes.txt:60-73`）：CPAL 一栏确实写着「no duplex streams」；`:133` one-directional、`:177` shared memory、`:242-263` Orderings / SeqCst、`:600` assert_no_alloc【测·抽取】。r5 A1 成立。
- 提醒【推】：「cpal 无 duplex」是 2024 年幻灯片上的说法，cpal 现状要由 rusteco 复核，不要当成永久事实。

**C19 · Hobson ADC 2025 幻灯片的具体主张** — 成立
- 幻灯片原文里有 `enum ToProcessor { BeginEdit(usize), SetParameter(usize, f32), EndEdit(usize) }`、`AudioStream<M: AudioModule>`、链接到 `github.com/irh/freeverb-rs`，还有「'app -> plugin' … made things harder」【官·幻灯片】https://hobson.dev/talks/adc_25 。r5 A2 标的【传】可以升为【官】。

**C20 · Timur Doumler 锁文章引文** — 成立
- 2020-04-14；「you should not use `std::mutex` on the audio thread, not even with `try_lock`」；「waiting on a mutex… leads to priority inversion」；「only ever call try_lock() and fall back to an alternative strategy on failure」【官】https://timur.audio/using-locks-in-real-time-audio-processing-safely
- 细节：「用上一次的值或 bypass」是对「alternative strategy」的转述，本轮抓取没看到这几个原词。r5 B1 不要把它放进引号。

**C21 · rust-analyzer architecture.md 引文** — 部分成立（一处引号不准）
- 原文核到：「never does any IO」「the only crate that knows about LSP and JSON serialization」「Bad proc macros may panic or segfault accidentally」「Generated code is generally committed」「There's no additional checks in CI, formatting and tidy tests are run with `cargo test`」【官】
- r5 §3.6 引的「deliberately not serializable」**不是原文**，原文是「not serializable **by design**」。意思相同，但打了引号就该照原文改。
- tidy.rs 里的函数名（`check_lsp_extensions_docs` / `check_cargo_toml` / `check_licenses` / `check_test_attrs` / `check_trailing_ws` / `TidyDocs` / `TidyMarks`）都复核到【源】。

**C22 · matklad「Fast Rust Builds」引文** — 成立
- 「The most important property of a crate is which crates it doesn't (transitively) depend on」「do not put generic code at the boundaries between the crates」「keep serialization at the boundary of the system, in the leaf crates」「BusyBox-style」都是原文【官·作者博客】。注意等级：r1 标【传】，r2/r5 标【官】，**口径不一**。建议统一为「【官·作者一手】，但不是 Rust 官方规范」。

### 2.3 许可与项目主张类

**C23 · 依赖许可** — 成立，补一项
- 本机 Cargo.toml 复核：cpal `Apache-2.0`、rtrb / rubato / ort / arc-swap / tract-core / tract-onnx `MIT OR Apache-2.0`、coreaudio-rs / libDF / basedrop `MIT/Apache-2.0`（旧斜杠写法）、audio_thread_priority `MPL-2.0`、nnnoiseless `BSD-3-Clause`、webrtc-audio-processing `license-file = "COPYING"`（Google 2011 BSD 风格，确实需要 `clarify`）【源】。GTCRN LICENSE 是 MIT（Rong Xiaobin），BlackHole：源码 GPL-3.0，二进制和品牌保留权利【源】。
- **补**：r3 推荐的 `triple_buffer` 是 **MPL-2.0**（`R/triple-buffer/Cargo.toml`）【源】。和 GPL-3.0 兼容，但 cargo-deny 的 allow 里要列上 MPL-2.0（r6 的表漏了这一项）。

**C24 · Firewheel / Asahi 的架构主张** — 成立，并补全
- Firewheel README：「Properly respects realtime constraints (no mutexes!)」；「planned to be upstreamed into the Bevy game engine where it will become the default audio engine」（**计划**，不是已完成）；「avoiding circular dependencies」【官】。
- Asahi 文档：「the world's first (as far as we know) open source "smart amp"」；有内核侧互锁，「if speakersafetyd is not running or non-responsive」时限音量【官】。文档**没写**实现语言。GitHub languages API：speakersafetyd 以 Rust 为主，**Bankstown 也是 Rust**（chadmed/bankstown）；Triforce 的仓库路径没找到【官·API】。r5 D1「Triforce/Bankstown 语言未核实」可以补上 Bankstown 一项。

**C25 · 本机源码引证抽检** — 全部成立
- `coreaudio-rs/src/audio_unit/render_callback.rs:766` `extern "C-unwind" fn input_proc` ✔；Rust 1.81 公告「non-unwind ABIs… will now abort on uncaught unwinds」✔【官】https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/
- cpal 只在 Windows / Linux 系 / Android 的 target 段里引入 audio_thread_priority（`cpal/Cargo.toml:115,137,221` 段），Apple 段没有 ✔。r4 漏说了 Android，无关紧要。cpal、coreaudio-rs、audio_thread_priority 里 grep `workgroup` 都是空 ✔。
- `DeepFilterNet/libDF/src/tract.rs:460-498` 的 `tvec!` / `clone()` / `pop().unwrap()`，`:527` 的 `log::warn!`，`:541-542` 的 `push_back(clone())` ✔；`ladspa/src/lib.rs` 的 `lock().unwrap()` 出现在 411/421/456/468 附近 ✔。
- `biglinux-gtcrn/stream/gtcrn_stream.py:371` `torch.stft(x, 512, 256, 512, …)` ✔；`Firewheel/crates/firewheel-core/src/lib.rs:1` 的 `cfg_attr(no_std)` ✔；`nih-plug/Cargo.toml:56,100-101` ✔；`firewheel-macros/src/lib.rs:25-28` 的 RealtimeClone ✔；`gungraun/deny.toml:183` 的 wrappers 示例 ✔；`tympan-aspl/src/realtime/mod.rs:45` ✔。
- **不成立的一处**：r3 §2.4「`R/fastenhancer` 是项目骨架（README 是『Mac 开源实时语音降噪』）」。实际 README 第一行是「Official repository of "FastEnhancer: Speed-Optimized Streaming Neural Speech Enhancement" (ICASSP 2026)」，`origin = github.com/aask1357/fastenhancer`，LICENSE MIT（AHN Sung Hwan）【源】。所以 FastEnhancer-M 的 hop 和 cache 形状**可以从本机源码查**（`configs/`、`onnx/`、`models/`），r3 §8 那条待办应改成「去本机查」。r6 的「FastEnhancer MIT」是对的。

**C26 · GitHub Actions 价格与 Embark 归档** — 成立
- Linux 2 核 $0.006/分钟，macOS 3/4 核 $0.062/分钟，Free 账户私有仓每月含 $12，自托管 runner 不计费【官】https://docs.github.com/en/billing/concepts/product-billing/github-actions
- EmbarkStudios/rust-ecosystem 横幅：「archived by the owner on May 21, 2026」【官】。r5 标的【传】可以升为【官】。

---

## 3. 对综合稿的具体改写建议

1. **依赖方向守卫分两层，写清楚各管什么**：cargo-deny `wrappers` 管「谁能**直接**依赖 X」，并且要把内部 crate（`vdn-io-macos`、`vdn-engine`、`vdn-infer-*`）本身列进 deny；`xtask check-deps`（读 cargo metadata 做传递闭包）管「某个 bin 或 crate 的依赖树里**不出现** X」。后者不能省。
2. **clippy 配置唯一编码**：一份真相源（例如 `specs/rt-disallowed.toml`）→ xtask 生成根目录和各 RT crate 的 clippy.toml → 测试检查一致。`disallowed-macros` 必须列 `std::println/eprintln/print/eprint/dbg/format`、`alloc::vec`。
3. **unsafe 策略**：workspace 用 `unsafe_code = "deny"`；非 FFI crate 在 `lib.rs` 写 `#![forbid(unsafe_code)]`；FFI crate 用 `#[allow]` 局部放行，并配 `clippy::undocumented_unsafe_blocks = deny`。
4. **RTSan 口径**：「rtsan-standalone（stable，Linux/macOS/iOS，实测可用）为主；rustc `-Zsanitizer=realtime` 自 1.93 起在 nightly 可用（unstable，目标平台未公开）；能拦什么 = malloc/free/pthread_mutex + 未公开的其他函数，按项实测；ort 内部能否拦到 = 待 E1」。
5. 演讲表：RTSan 演讲改为「ADC 2024 Trevelyan & Apple（会议页，视频未找到）· CppCon 2024 Apple & Trevelyan（`KvhgNdxX6Uw`）」，删掉 Josip Ćavar。

## 4. 仍然无法核实

- cargo-deny 对 workspace path 依赖的 ban 实际效果（源码支持，没实跑）。
- ast-grep 带属性函数的规则实际写法（本机没装 ast-grep）。
- rustc 内置 RTSan 在 `aarch64-apple-darwin` 上能不能用（没有 nightly）。
- ADC 2024 RTSan 演讲视频 URL；ADC 版 Real-time 101 的年份；Ian Hobson 入门演讲的年份。
- RTSan 拦截函数的完整清单（官方没给）。
