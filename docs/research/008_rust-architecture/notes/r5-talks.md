# R5 · 社区与大会的技术决策（Rust 音频 / 实时 / 大型项目架构）

> 调研日期 2026-10-30。分工：本篇只取「规则 · 做法 · 取舍」层；单项目实现细节归 rusteco 的生态核查表。
> 证据等级：【官】官方明载 ·【源】源码可证 ·【测】本机实测 ·【传】二手/论坛/摘要转述 ·【推】我的推断。
> 说明：YouTube 页面抓不到简介，凡只拿到「标题 + URL」而没读到内容的演讲，结论栏写「仅标题可证」，不替它编内容。
> 本机克隆根目录简写为 `R=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources`。

---

## 0. 先答人的三问（一页结论）

| 人问 | 社区给的答案 | 对本项目的落点 |
|---|---|---|
| 「具体 Rust 结构、语法结构用什么」 | 中型项目（1 万–100 万行）用**扁平 `crates/*` + 根目录虚拟 manifest**；文件夹名 = crate 名；内部 crate `version = "0.0.0"`；自动化放 `xtask` crate（matklad，§3.1）。依赖、包元数据、lint 用 `[workspace.dependencies]` / `[workspace.package]` / `[workspace.lints]` 统一（Cargo book，§3.3） | 照搬。设计稿七个 crate 全放 `crates/`，加一个 `xtask`。 |
| 「用超级语法树 / AST 把东西做成模块」 | 社区里**确实有三种「借 AST 做模块化」的做法**，但都是有代价的工具，不是默认：① proc-macro（`syn` 解析 AST）生成参数声明代码——nih-plug `Params` derive、firewheel-macros；② 类型级 DSP 图（fundsp 用运算符 + `typenum` 在编译期检查通道数）；③ 外部 DSL → Rust 源码（Faust `-lang rust` + `build.rs`）。同时 matklad 明确警告 `syn` 类宏是编译关键路径瓶颈，生成代码要放在依赖图的叶子（§4） | **真正的「模块」由 crate 图承担**（编译器强制边界，是最硬的 AST 级隔离）；AST 工具用在「**检查**」而不是「生成」：clippy `disallowed-methods`、rustdoc-JSON 系的 `cargo-semver-checks`/`cargo-public-api`、RTSan `#[nonblocking]`。v1 不写 proc-macro；如需，只允许一个小 derive crate 且在叶子。见 §4 |
| 「Rust 社区和大会有什么好的技术决策」 | 七条被反复验证的决策（§5）：核心不做 IO（rust-analyzer / sans-IO）；IO 只在一个 crate；音频线程不加锁不分配、失败用旁路兜底（Timur Doumler）；参数走消息/原子（Hobson、Eckes、Firewheel）；图在非实时侧编译好再整体送进音频线程（Firewheel、fundsp）；用机器证明实时安全（assert_no_alloc + RTSan）；features 只能做加法 | 直接映射到设计稿 crate：`engine` = sans-IO 核心，eval 与实时同一入口；`io-macos` 是唯一碰 CoreAudio 的 crate |

---

## 1. 演讲清单（只收有视频 / 幻灯片 / 页面 URL 的）

### 1.1 ADC（Audio Developer Conference）上的 Rust 演讲

| # | 演讲 · 讲者 · 年份 | URL | 和我们相关的结论 | 对本项目意味着什么 |
|---|---|---|---|---|
| A1 | **Building Audio Apps with Rust – An Overview of Tools and Techniques** · Stephan Eckes（ai-coustics）· ADC 2024 | 会议页 https://conference.audio.dev/session/2024/building-audio-apps-with-rust-an-overview-of-tools-and-techniques/ ；视频 https://www.youtube.com/watch?v=G3WWr4uhP0I ；幻灯片 https://data.audio.dev/talks/2024/building-audio-apps-with-rust/slides.pdf ；配套代码 https://github.com/neodsp/adc-talk-24 | 会议摘要：覆盖系统音频接入、「在 audio callback 里做 DSP，重点是 real-time safe 代码和 thread-safe 参数管理」、GUI 选型【官·会议页】。幻灯片（本机 pdftotext 抽取）：平台 API 列表含 coreaudio-rs / cpal，并标注 cpal「no duplex streams」；有「Thread Synchronization → one-directional → shared memory」一节；专门一节讲 atomics `Ordering`（Relaxed/Release/Acquire/AcqRel/SeqCst）与「wrong ordering can lead to bugs」【测·本机抽取幻灯片文字，部分页为图片未能抽出】。讲者也是 `rtsan-standalone` Rust 封装作者（`$R/rtsan-standalone-rs/README.md:110`）【源】 | ① cpal 无 duplex → 我们本就是「采集进程 + 共享内存」单向流，方向一致；② 原子 ordering 要写进规约（发布声纹结果用 Release/Acquire，不要默认 SeqCst 糊过去也不要乱用 Relaxed）；③ 讲者作为 ai-coustics（做神经网络实时语音增强的公司）工程师，选的就是 Rust + rtsan，同类产品的佐证【推】 |
| A2 | **Cross-Platform Music Software with Rust** · Ian Hobson · ADC 2025 | 会议页 https://conference.audio.dev/session/2025/cross-platform-music-software-with-rust/ ；视频 https://www.youtube.com/watch?v=YLPglu2enaE ；幻灯片 https://hobson.dev/talks/adc_25 | 摘要：介绍「可复用的项目模板，同一套 Rust 音乐应用可发布为插件、移动端、桌面或浏览器」【官·会议页】。幻灯片：`AudioStream<M: AudioModule>` 让音频引擎对具体 DSP 泛型化；GUI↔处理器走**类型化消息** `ToProcessor { BeginEdit, SetParameter, EndEdit }`；DSP（freeverb-rs）与 app（audio-app）**分仓**；作者自述走「app → plugin」方向导致后补插件更难【传·WebFetch 摘要幻灯片】 | ① 「引擎对 DSP 模块泛型 / trait 化」正对应我们「GTCRN 与 FastEnhancer-M 同接口可切换」；② 控制面用**封闭 enum 消息**，不要回调；③ 他的教训：一开始就把「宿主形态」（我们是 BlackHole → 自写 HAL 插件 → eval 离线）当作多个前端，而不是从一个前端改造 |
| A3 | **Writing Elegant DSP Code in Rust** · Chase Kanipe · ADC23 | https://www.youtube.com/watch?v=RcBnGiDSn6g | 仅标题可证；未拿到摘要或幻灯片【官·仅标题】。WolfTalk 播客页确认该演讲存在 https://redcircle.com/shows/wolftalk-podcast-about-audio-programming-people-careers-learning/ep/9d5a7a5a-f133-4184-91bd-886ef2ea21d9 【传】 | 写 `dsp` crate 时值得看一遍再定 trait 形状；本篇不引用其具体主张 |
| A4 | **An Introduction to Rust for Audio Developers** · Ian Hobson · 年份未在页面确认（JUCE 论坛 ADC Spotlight 帖称其为 ADC 演讲） | 视频 https://www.youtube.com/watch?v=Yom9E-67bdI ；论坛 https://forum.juce.com/t/adc-spotlight-ableton-developer-ian-hobson/29390 | 视频简介：「Rust … is quickly developing into an alternative to C++ for realtime audio applications」【传·搜索摘要】 | 入门材料，无架构结论 |
| A5 | **Rusty Grains (Granular Synthesis in Rust)** · Ian Hobson | https://www.youtube.com/watch?v=Z4P5f6ZJ_nE | 仅标题可证 | — |
| A6 | **Bare Metal Audio Programming With Rust** · Antoine van Gelder · ADC20 | https://www.youtube.com/watch?v=udlK1LQ3f3g | 仅标题可证（搜索摘要称简介链到讲者幻灯片，未取到） | — |
| A7 | **Creative Coding in Rust: Building a Generative Music App With Nannou** · Zsolt Török · ADC21 | https://www.youtube.com/watch?v=JPFv3adyLB4 | 仅标题可证 | — |
| A8 | **Safe and Sound: Using C++ Audio Libraries from Rust** · James Hallowell（Focusrite）· ADC 2025 | https://conference.audio.dev/session/2025/safe-and-sound | 摘要：用 Rust FFI 给 C++ 库（如 JUCE）写底层绑定，`cxx` 等工具简化【传·搜索摘要】 | 我们将来自写 C 的 HAL 插件 + Rust 引擎，FFI 边界设计可参考；`webrtc-audio-processing` 绑定同理 |
| A9 | **Can Audio Programming be Safe?** · Dave Rowland（Tracktion，WG21 英国代表）· ADC 2024 | https://conference.audio.dev/session/2024/can-audio-programming-be-safe/ | 摘要：从 NSA/NIST 的内存安全倡议出发，比较 Rust / Hylo / Swift / Cmajor 是否适合音频，并回顾静态分析器与 sanitizer【官·会议页（转述）】 | 支持我们「用工具证明」而不是「靠自觉」的路线 |
| A10 | **LLVM's Real-Time Safety Revolution – Tools for Modern Audio Development** · David Trevelyan & Chris Apple · ADC 2024 | https://conference.audio.dev/session/2024/llvms-real-time-safety-revolution-tools-for-modern-audio-development/ | 摘要：RealtimeSanitizer（RTSan）已进入主线 Clang/LLVM，运行时检测 malloc、系统调用与用户声明的「unsafe」函数；Performance Constraints（`-Wfunction-effects`）在编译期阻止 nonblocking 函数调用阻塞函数；讲者称 ADC 2023 首次发布 RTSan【官·会议页】。视频 URL 未在会议页找到，**未找到出处**，不列 | 「音频线程不分配、不加锁、不阻塞」的**机器证明**主力工具；Rust 侧用 `rtsan-standalone`（§5.6） |

### 1.2 实时安全「经典」演讲 / 文章（C++ 圈，但规则语言无关）

| # | 演讲 · 讲者 · 年份 | URL | 结论 | 对本项目 |
|---|---|---|---|---|
| B1 | **Using Locks in Real-Time Audio Processing, Safely**（文章 + 演讲）· Timur Doumler | 文章 https://timur.audio/using-locks-in-real-time-audio-processing-safely ；视频 https://www.youtube.com/watch?v=DHx9kv74Mlc | 音频线程不得分配、系统调用、IO、等锁；「waiting on a mutex leads to priority inversion」；`std::mutex::try_lock` 也不行，因为 unlock 可能进系统调用；可接受的只有 try_lock 与 unlock 都实时安全的 spinlock，**等待循环绝不在音频线程上**，音频线程 try_lock 失败就用「上一次的值或 bypass」兜底；理想方案是 atomics、lock-free FIFO、不可变数据结构，「keep processing code separate from the data model」【官·作者原文】 | ① 「原声旁路」不只是功能，也是**失败策略**：推理来不及/拿不到新状态时就旁路或沿用上一帧增益；② 声纹线程→实时线程的结果发布用原子/不可变快照，不用锁 |
| B2 | **Real-time Programming with the C++ Standard Library** · Timur Doumler · CppCon 2021（同主题 ADC21 版 *C++ Standard Library for Real-time Audio*） | https://www.youtube.com/watch?v=Tof5pRedskI ；ADC21 https://www.youtube.com/watch?v=vn7563IAQ_E | 讲哪些标准库设施在实时上下文可用、哪些不可用【传·简介】 | 写 Rust 规约时类比：`Vec::push`、`format!`、`println!`、`Arc` drop 最后一个引用 都是「可能分配」 |
| B3 | **Thread Synchronisation in Real-Time Audio Processing with RCU** · Timur Doumler · ADC | https://www.youtube.com/watch?v=7fKxIZOyBCE | 仅标题可证 | 声纹结果「原子发布」的理论背景（RCU = 发布新快照 + 延后回收旧快照）；Rust 对应物 arc-swap / basedrop 归 rusteco |
| B4 | **Real time 101** · David Rowland & Fabian Renn-Giles · Meeting C++ 2019（社区 gist 分两部分列出） | https://www.youtube.com/watch?v=ndeN983j_GQ ；Part I https://www.youtube.com/watch?v=Q0vrQFyAdWI ；Part II https://www.youtube.com/watch?v=PoZAo2Vikbo | 仅标题可证（「Investigating the real-time problem space」「The real-time audio developer's toolbox」）；被播客称为音频圈引用最多的演讲之一【传】 | 必看入门；规约里的「禁止清单」可对照它校对 |
| B5 | **Real-time Confessions in C++** · Fabian Renn-Giles · ADC23 | https://www.youtube.com/watch?v=JG7lqt7V1uY | 仅标题可证 | — |
| B6 | **Real-time audio programming 101: time waits for nothing** · Ross Bencina · 2011 | http://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing | 本次抓取连接被拒，**原文未读到**；LAD 邮件列表 Paul Davis 称其「MUST read」https://lists.linuxaudio.org/archives/linux-audio-dev/2011-July/030643.html 【传】；LWN 讨论 https://lwn.net/Articles/453005/ | 经典，人工补读 |
| B7 | 社区汇总 gist「Must Read/Watch Basics About Real-Time Audio Programming」· WeirdConstructor（HexoDSP 作者） | https://gist.github.com/WeirdConstructor/276f7e0555b2dbe83614268b59a7a998 | 列了 B4、B6、nih-plug、basedrop、rubato、cpal、fundsp 等【官·gist 原文】 | RustAudio 圈的「入门书单」，可直接挂进 `docs/` 索引 |

> 「Fabian Giesen」：未找到他与 Rust 音频 / 实时相关的演讲或文章，**未找到出处**；任务里很可能指 Fabian **Renn-Giles**（B4、B5）。

### 1.3 Rust 专题大会（RustConf · EuroRust · RustNation UK · RustWeek）

- **RustConf**：搜索 2019–2025 未发现音频 / DSP 专题演讲。2025 视频合集 https://www.youtube.com/playlist?list=PL2b0df3jKKiRFEuVNk76ufXagOgEJ9sBZ （未逐条筛）。**未找到出处**。
- **EuroRust**：未找到音频相关演讲。**未找到出处**。
- **RustNation UK**：未找到音频相关演讲。**未找到出处**。
- **RustWeek（2025 Utrecht）**：搜到的演讲题目覆盖编译器、类型系统、rust-analyzer、Bevy，无音频。**未找到出处**。
- 结论【推】：Rust 音频的「大会知识」主要沉淀在 **ADC**（§1.1），Rust 专题会贡献的是**通用架构**知识（§3）。这本身是个信号——本项目的实时音频规则抄 ADC，工程组织规则抄 Rust 官方与 matklad。

### 1.4 FOSDEM

| # | 演讲 | URL | 结论 | 对本项目 |
|---|---|---|---|---|
| C1 | **Rusty instruments** · Benedict R. Gaster（UWE）· FOSDEM 2020 Rust devroom | https://archive.fosdem.org/2020/schedule/event/rust_muses/ ；视频 https://video.fosdem.org/2020/K.3.401/rust_muses.mp4 | 「Rust is a key ingredient in the Muses project」；架构是 STM32 嵌入式 + 树莓派 Pure Data 合成 + Rust 驱动与 OSC 服务器【官·日程页】 | 把「合成/DSP 引擎」与「控制/协议」分进程，和我们「送音进程 / 推理进程」同构 |
| C2 | **PipeWire State of the Union** · FOSDEM 2024 Open Media | https://archive.fosdem.org/2024/schedule/event/fosdem-2024-1988-pipewire-state-of-the-union/ | 非 Rust 专题 | 仅背景 |
| C3 | **Making VirtIO sing – virtio-sound in rust-vmm** · FOSDEM 2024 Virtualization | https://archive.fosdem.org/2024/schedule/event/fosdem-2024-1910-making-virtio-sing-implementing-virtio-sound-in-rust-vmm-project/ | Rust 实现虚拟声卡设备，后端接 PipeWire/ALSA【传·搜索摘要】 | 「虚拟音频设备」在 Rust 里有先例，但它是 VMM 设备，不是 macOS HAL |
| — | FOSDEM 2026 有 Rust 与 Audio/Video/Graphics Creation devroom，但**未找到**具体 Rust 音频演讲 | https://archive.fosdem.org/2026/news/2025-10-31-devrooms-announced/ | — | — |

### 1.5 Rust 在 Linux 音频栈 / 其他社区实践

| # | 项目 / 文档 | URL | 结论 | 对本项目 |
|---|---|---|---|---|
| D1 | Asahi Linux 音频用户态：speakersafetyd（Rust 写的开源「smart amp」扬声器保护守护进程）、Triforce（MVDR 自适应波束成形麦克风）、Bankstown（虚拟低音） | https://asahilinux.org/docs/sw/audio-userspace/ ；https://github.com/AsahiLinux/speakersafetyd | 文档称 speakersafetyd 是「the world's first (as far as we know) open source 'smart amp' implementation」，并有**内核侧互锁**：守护进程不在或无响应时内核限制音量【官】；speakersafetyd 为 Rust 由 Debian 包名 `rust-speakersafetyd` 佐证 https://tracker.debian.org/media/packages/r/rust-speakersafetyd/control-2.0.1-1 【传】；Triforce/Bankstown 的实现语言本次**未核实** | 「守护进程挂了要有安全兜底」= 我们推理进程崩了，送音进程必须自动走原声旁路（看门狗 / 心跳），这是同类产品的现成做法 |
| D2 | Bevy × Firewheel：Firewheel 计划上游进 Bevy 成为默认音频引擎 | README https://github.com/BillyDM/Firewheel ；Bevy 审计笔记 https://hackmd.io/@bevy/rkwFscXObx | 目标是「avoiding circular dependencies」，核心引擎只依赖轻量 `bevy_platform`【官·README】；审计指出 firewheel 依赖 bevy 会让版本发布「a publishing nightmare」【传·hackmd】 | 依赖方向教训：底层 `dsp`/`engine` 绝不能反向依赖上层 `control`/GUI 框架 |
| D3 | nice-plug：nih-plug 的社区分叉，自称「now the recommended toolkit for Rust audio plugin developers」，并 fork 了 assert_no_alloc（`nice-assert-no-alloc`） | https://docs.rs/crate/nice-plug/0.1.5/source/README.md ；本机 `$R/nice-assert-no-alloc/README.md:1-8` | 【传】（自述）／【源】（本机 fork 说明） | 生态细节归 rusteco；规则层只记：实时检测工具在社区里是「框架默认带」的 |
| D4 | RustAudio 社区站 | https://rust.audio/ | 列 Discord、GitHub RustAudio 组织、BillyDM 的 Awesome-Audio-DSP https://github.com/BillyDM/Awesome-Audio-DSP 【官】；**未找到**「Discord 演讲汇总」 | 资料入口 |
| D5 | Faust + Rust 经验 · obsoleszenz · IFC 2024 | https://faust.grame.fr/community/ifc/2024/Experiences_with_Rust_Faust.pdf | 幻灯片：项目结构 `crates/{dsp,backend,gui}`；`dsp/build.rs` 在 `faust-rebuild` feature 下用 `faust-build` 把 `.dsp` 生成 `src/faust_dsp.rs`；「generated rust code is not idiomatic … we wrap it」【测·本机抽取幻灯片文字】 | 见 §4.3「外部 DSL → Rust」 |

### 1.6 BillyDM（Firewheel / Meadowlark）

- **未找到** BillyDM 的大会演讲视频。可引用的是他的**设计文档**：Firewheel `DESIGN_DOC.md` https://github.com/BillyDM/Firewheel （README 链接）。
- 设计文档要点【官·DESIGN_DOC 转述】：
  - 目标「Properly respect realtime constraints (no mutexes!)」。
  - 激活时创建一个 realtime-safe 消息通道和 processor（executor）送进音频流；**图在非实时侧编译成 schedule 再经通道送给 executor**；编译失败则停用并报错。
  - 参数更新「purely event-driven」，每次 `update()` 把排队事件**成批**冲刷，保证同一处理周期的事件在一起。
  - 非目标：不做 DAW 引擎、不做多线程图处理、不做插件宿主——**明确写下「不做什么」**。
  - crate 拆分：`firewheel-core`（共享类型 + 节点 API）/ `-graph` / `-nodes` / `-cpal`（后端）/ `-macros` / `-pool`。本机实测目录：`$R/Firewheel/crates/` 下有 core、cpal、graph、macros、nodes、rtaudio、symphonium【测】。
  - `firewheel-core` 可 `no_std`：`$R/Firewheel/crates/firewheel-core/src/lib.rs:1` `#![cfg_attr(not(feature = "std"), no_std)]`【源】；根 `Cargo.toml:17-24` 用 `[workspace.package]` 统一元数据，`:26-31` 用 `std` feature 级联到子 crate【源】。
- 对本项目：`engine` 的「构建期 vs 运行期」分离照抄——模型加载、会话构建、DSP 链参数校验全部在非实时侧完成，产物（预分配好的 processor）一次性交给实时线程。

---

## 2. Rust 实时安全的「机器证明」工具链（大会 + 文档里的共识）

| 工具 | 出处 | 能证明什么 | 不能证明什么 | 本项目用法 |
|---|---|---|---|---|
| `assert_no_alloc` | https://github.com/Windfisch/rust-assert-no-alloc ；本机 `$R/rust-assert-no-alloc` | 自定义 `#[global_allocator]`，在 `assert_no_alloc(|| …)` 闭包内任何 Rust 分配/释放即 abort 或告警；**默认 `disable_release`：release 构建什么都不做**【官·README】 | 只拦经过 Rust 全局分配器的分配；锁、系统调用、C/C++ 库（ONNX Runtime）内部的 malloc 拦不到【推，README 只说 allocator 硬编码为 `System`】 | 测试与 debug 构建里包住 `engine.process()`；nih-plug 用法：`assert_process_allocs = ["dep:assert_no_alloc"]`，注释提醒「panics may also allocate if they use string formatting」（nih-plug `Cargo.toml` [features]）【源·WebFetch 原文】 |
| RTSan / `rtsan-standalone` | Clang 文档 https://clang.llvm.org/docs/RealtimeSanitizer.html ；Rust 封装 https://github.com/realtime-sanitizer/rtsan-standalone-rs ，本机 `$R/rtsan-standalone-rs/README.md:7,11-16,50,84,101` | 在 nonblocking 上下文中拦截「`malloc`, `free`, `pthread_mutex_lock`, or anything else known to have a non-deterministic execution time」【官·Clang 文档】；**拦的是 libc 拦截点，所以 ONNX Runtime 等 C++ 库内部的分配也能抓到**【推】；Rust 用 `#[nonblocking]` 宏、`RTSAN_ENABLE=1 cargo run` 运行；支持 Linux、macOS、iOS【源·README:7】；默认 `prebuilt-libs` feature 会用 curl 下载预编译库【源·README:84】；rustc 正在加 `sanitize(realtime = "nonblocking")` 属性 https://github.com/rust-lang/rust/pull/147935 【传·搜索摘要】 | 只测到**跑到的路径**（运行时检测） | 这是「要用机器证明」的主证据：实时线程入口函数标 `#[nonblocking]`，在 CI 跑 eval 语料全程开 RTSan；ort 与 tract 两条推理后端各跑一遍，**决定哪个后端能进实时线程** |
| `no_std` + 无 `alloc` crate | Cargo book features 章 https://doc.rust-lang.org/cargo/reference/features.html （推荐用 `std` feature 打开 std，而非 `no_std` feature）；Firewheel core 先例（见 §1.6） | 一个 crate 若 `#![no_std]` 且不 `extern crate alloc`，**编译器层面就不可能分配**（没有 `Vec`/`Box` 可用）【推，基于语言规则】 | 不能用于依赖 std 的推理 runtime | `dsp` crate（biquad、压缩限幅、去混响尾）做成 `no_std` 无 alloc：这是全项目最便宜、最硬的一条证明 |
| clippy `disallowed-methods` / `disallowed-types` | https://doc.rust-lang.org/clippy/lint_configuration.html | 在 `clippy.toml` 里按全路径禁用方法/类型，可附 `reason`、`replacement`【官】 | 只是静态 lint，跨 crate 调用链里间接用到的发现不了；clippy.toml 的查找规则（`CLIPPY_CONF_DIR` / 自 crate 目录向上）本次**未读到官方原文**【推】 | 在实时相关 crate 禁 `std::sync::Mutex::lock`、`Vec::push`、`println!` 背后的函数等，给人看得懂的 reason |
| clippy restriction 组：`unwrap_used`、`expect_used`、`panic`、`indexing_slicing` | 名称来自 clippy lint 列表，本次**未逐条打开文档核对**【推】 | 静态排除显式 panic 来源 | 算术溢出（debug 下 panic）、slice 越界 | `engine`/`dsp` 的 `lib.rs` 顶部 `#![deny(...)]` |

**取舍结论【推】**：社区共识是「静态 + 动态」两层——`no_std`/clippy 管能静态排除的，`assert_no_alloc`/RTSan 管跑出来的。没有任何一个工具单独够；而且 assert_no_alloc 默认 release 失效、RTSan 只覆盖跑到的路径，所以 **eval 语料要当「实时覆盖率」来设计**（覆盖模型切换、声纹结果更新、设备漂移补偿触发等分支）。

---

## 3. Rust 官方与社区的大型项目指南

### 3.1 matklad · Large Rust Workspaces（2021-08-22）
URL：https://matklad.github.io/2021/08/22/large-rust-workspaces.html 【官·作者原文】
- 1 万–100 万行「the flat layout makes the most sense」：所有 crate 一层放 `crates/`，根 `members = ["crates/*"]`；扁平「doesn't need maintenance」，树形会「deteriorate over time」。
- 「Make the root of the workspace a virtual manifest」。
- 文件夹名 = crate 名，不剥公共前缀，方便导航和改名。
- 内部不发布的 crate 用 `version = "0.0.0"`。
- 有 semver API、打算发布的放 `libs/`，便于检查 `libs/` 不依赖 `crates/`。
- 自动化写成 Rust（`cargo xtask`），不散落 Makefile / `prepare.sh`。
- 单文件 crate 也保留 `src/`。
- **对本项目**：`crates/{vdn-dsp,vdn-engine,vdn-speaker,vdn-io-macos,vdn-control,vdn-eval}` + `xtask`；`hal-plugin` 是 C 的话不进 Cargo members，放 `hal-plugin/` 顶层（新顶层目录 = 协议事件，需人拍板）。是否加 `vdn-` 前缀是命名规约问题，归 `specs/README.md`，matklad 只要求「文件夹名与 crate 名一致」。

### 3.2 matklad · ARCHITECTURE.md（2021-02-06）
URL：https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html 【官】
- 内容：鸟瞰问题 + codemap（「a map of a country, not an atlas of maps of its states」）+ **架构不变量**（「important invariants are expressed as an absence of something」）+ 层与系统边界 + 横切关注点单列。
- 不写易变细节，短，「revisit it a couple of times a year」，不追求与代码同步。
- **对本项目**：我们的硬约束天然是「缺席型不变量」——「`engine` 不依赖 `io-macos`」「实时线程上不存在分配/锁/阻塞/panic」「`dsp` 没有 `alloc`」。写进 `ARCHITECTURE.md`，并且每条都配一条机器检查（§2）。

### 3.3 Cargo Book
- Workspaces https://doc.rust-lang.org/cargo/reference/workspaces.html 【官】：虚拟 manifest 必须显式写 `resolver`（「as they have no `package.edition` to infer it from」）；共享 `Cargo.lock` 与 `target/`；`[workspace.package]`（`version.workspace = true` 继承）、`[workspace.dependencies]`（不能声明 `optional`；features 与成员处叠加）、`[workspace.lints]`（MSRV 1.74，成员写 `[lints] workspace = true`）；`default-members`。
- Features https://doc.rust-lang.org/cargo/reference/features.html 【官】：
  - `#feature-unification`：同一依赖取所有 feature 的并集，「features should be *additive*」，开启 feature 不应关闭功能。
  - `#mutually-exclusive-features`：互斥 feature「should be avoided if at all possible」；替代：拆成独立 package、规定优先级、或「select the behavior at runtime」。
  - **对本项目（重要）**：「GTCRN vs FastEnhancer-M」「ort vs tract」**不要做成互斥 feature**。正确做法：两个推理后端各自一个 crate 或各自一个**可同时开启**的 feature，具体用哪个在运行时由配置选（Cargo book 原文给的第三条替代）。这也恰好满足「同接口可切换」与 eval 同时对比两模型。

### 3.4 Rust API Guidelines
URL：https://rust-lang.github.io/api-guidelines/checklist.html 【官】
与我们 crate 间接口最相关的条目：C-SEND-SYNC（类型尽量 Send/Sync）、C-GOOD-ERR（错误类型有意义）、C-SEALED（sealed trait 防下游实现）、C-STRUCT-PRIVATE（字段私有）、C-NEWTYPE-HIDE、C-CALLER-CONTROL（「The caller decides where to copy and place data」）、C-OBJECT（可能当 trait object 用的 trait 要 object-safe）、C-VALIDATE、C-DTOR-FAIL / C-DTOR-BLOCK（析构不失败；可能阻塞的析构要有替代）。
- **对本项目**：
  - **C-CALLER-CONTROL** 是实时 API 的核心：`process(&mut self, input: &[f32], output: &mut [f32])` 由调用方给缓冲区，引擎不自己分配。
  - **C-DTOR-BLOCK**：实时线程上 drop 最后一个 `Arc`/`Box` 会释放内存——这就是 basedrop/延迟回收存在的理由；规约要写「实时线程不 drop 拥有堆内存的值」。
  - **C-OBJECT vs 泛型**：增强模型接口（`trait Enhancer`）若要运行时切换就必须 object-safe（`Box<dyn Enhancer>` 在构建期分配一次，运行期只做虚调用，可接受）【推】。
  - **C-SEALED**：内部 trait 不打算让外部实现，可 seal；本项目不发布库，优先级低【推】。

### 3.5 The Rust Performance Book（Nicholas Nethercote）
- Build configuration https://nnethercote.github.io/perf-book/build-configuration.html 【官】：`codegen-units = 1`、`lto = "fat"`（「gains are not always present」）、替代分配器、`-C target-cpu=native`（产物可能在别的 CPU 上跑不了）、PGO（可达 10%+）；`panic = "abort"` 的告诫：「Only use it if you do not need to unwind on panic」。建议逐项基准验证。
- Heap allocations https://nnethercote.github.io/perf-book/heap-allocations.html 【官】：DHAT 定位热点分配；`Vec::with_capacity`；在循环外保留「workhorse」集合，用 `clear` 复用容量；`SmallVec`。
- 本机先例：DeepFilterNet 根 `Cargo.toml:22-27` 定义 `[profile.release-lto]`（`inherits = "release"`、`lto = "fat"`、`strip`、`panic = "abort"`），`:14-20` dev 用 `opt-level = 1`、release 用 `lto = "thin"`【源】。
- **对本项目**：
  - Mac 分发：`target-cpu=native` 不能用于发布包（Intel/Apple Silicon、M1/M3 指令差异）；可以用 `aarch64-apple-darwin` 默认 + 必要时 `target-feature` 精确开启【推】。
  - `panic = "abort"` 与「不 panic」：音频线程 panic 后栈展开跨 FFI（CoreAudio 回调是 C ABI）是 UB 风险区；abort 起码确定性。配合 §1.5 D1 的「进程挂了送音进程自动旁路」【推】。
  - eval 用同一 profile 跑基准，否则「离线测得快、实时卡」。

### 3.6 rust-analyzer `architecture.md`
URL：https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md 【官】
可直接迁移的五条：
1. **ground state vs derived state**、核心「never does any IO」，`base-db` 不知道文件系统。→ `engine` 不知道 CoreAudio、不知道 WAV、不知道共享内存，只吃 `&[f32]`。
2. **显式标注 API boundary**（`syntax`、`hir`、`ide`、`rust-analyzer`），并写明哪些 crate「are not, and will never be, an api boundary」。→ 我们标：`engine` 的 `Engine` 类型与 `control` 的消息 enum 是边界；`dsp` 内部 stage 不是。
3. **「rust-analyzer is the only crate that knows about LSP and JSON serialization」** → `io-macos` 是唯一知道 CoreAudio 的 crate；`control` 是唯一知道配置文件/IPC 序列化格式的 crate；`engine` 以下的类型**故意不可序列化**（原文：ide、base_db 及以下的类型「deliberately not serializable」）。
4. 错误处理：核心「don't interact with the outside world and thus can't fail」；返回 `(T, Vec<Error>)` 而非 `Result`；LSP 请求用 `catch_unwind` 包。→ 实时 `process()` 设计成**不可失败**（无 `Result`），异常状态用「旁路 + 计数器」上报；`catch_unwind` 只放在非实时边界（如模型加载）。
5. **进程隔离**：proc macro 放进程外，因为「Bad proc macros may panic or segfault accidentally」。→ 与我们「送音进程 / 推理进程」分离同理：ONNX Runtime 崩了不能把真麦克风一起带走。
6. 测试：三个边界测试、统一 `check` 辅助函数、数据驱动、不依赖外部资源、`expect` 快照、`cov_mark`。事件循环用**封闭 enum**列出全部事件，而非开放回调。

### 3.7 matklad 的其他「基本功」文章
- **Basic Things**（2024-03-22）https://matklad.github.io/2024/03/22/basic-things.html 【官】：Not Rocket Science Rule（主干一组自动检查永远绿，不可靠的检查删掉）；「a single, clearly documented command」+ O(1) 入口；CI 调用构建系统而非复制逻辑；「zero tolerance for flaky tests」，默认快测、慢测显式开启；**每个 benchmark 也是一个 test**（小输入跑在常规测试里防腐）；按提交跟踪构建时长/测试时长/产物大小；fuzz 固定种子作测试；尽早建立发布流程。
  → 本项目：`cargo xtask ci` 一条命令；端到端延迟 ≤45 ms 和 RTF 写成「也是测试的 benchmark」；按提交记录延迟与 CPU 曲线。
- **How to Test**（2021-05-31）https://matklad.github.io/2021/05/31/how-to-test.html 【官】：测功能不测代码（「neural network test」）；sans-io「Let the caller do input and output, and let the callee do compute」；慢测用环境变量开关；避免 mock；数据驱动 + expect 快照。
  → eval 本身就是这套思路：WAV 进、WAV/指标出、黄金文件比对。
- **Delete Cargo Integration Tests**（2021-02-27）https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html 【官】：`tests/` 里每个文件是一个独立 crate、各自链接；大项目只留一个 `tests/it/main.rs`；内部库直接用 `src/tests.rs` 单元测试；大型内部库可 `doctest = false`。
- **Fast Rust Builds**（2021-09-04）https://matklad.github.io/2021/09/04/fast-rust-builds.html 【官】：「the most important property of a crate is which crates it doesn't (transitively) depend on」；依赖图要「宽」不要「长链」；`syn` 类 proc macro 卡关键路径，序列化 derive 放叶子；**不要把泛型放在 crate 边界**，用薄泛型包装立即转发到非泛型内部函数；`-Z timings`、`cargo llvm-lines`；CI 里 `-D warnings` 走 `RUSTFLAGS`。
- **Inline In Rust**（2021-07-09）https://matklad.github.io/2021/07/09/inline-in-rust.html 【官】：非泛型函数无 `#[inline]` 不能跨 crate 内联；泛型隐式可内联。补充：Rust 1.75 起编译器会自动让**小函数**跨 crate 可内联（PR https://github.com/rust-lang/rust/pull/116505 ，阈值是内部参数）【传·搜索摘要，未核 release notes】。
  → `dsp` 的逐样本热函数（biquad tick）若被 `engine` 跨 crate 调用：要么按块处理（`process_block(&mut [f32])`，调用一次）消掉内联问题，要么显式 `#[inline]`；以基准为准【推】。
- 系列索引 One Hundred Thousand Lines of Rust https://matklad.github.io/2021/09/05/Rust100k.html 【官】。

### 3.8 sans-IO（网络圈，但与「eval 与实时同一份代码」完全同构）
- Thomas Eizinger，*sans-IO: The secret to effective Rust for network services*，Firezone 博客 2024-07-02 https://www.firezone.dev/blog/sans-io 【官】
  - 协议逻辑是纯状态机，不碰 socket 也不读时钟；**时间作为参数传入**（不在内部调 `Instant::now`）；测试可「跳到未来」、可把一端输出直接喂另一端输入；用 `&mut self` 避免 `Arc<Mutex<T>>`、channel、spawn。
  - 缺点：自己写事件循环容易出细 bug（如不推进 timeout 导致忙等）；多步流程要手写状态机；依赖若内部调 `Instant::now` 就破功。
- 未找到 Eizinger 的大会演讲视频，**未找到出处**。
- **对本项目**：`engine` 就是一个音频 sans-IO 状态机：`process(input, output, frame_index)`；声纹结果、参数变化都作为**显式输入**进来；时间用「样本计数」而非墙钟。于是 eval（读 WAV 循环调用）与实时（CoreAudio 回调调用）只是两个不同的「事件循环」。依赖审查加一条：推理 runtime / DSP 库内部不得读墙钟或起线程（否则离线不可复现）【推】。

### 3.9 Embark Studios 的生态做法
- `cargo-deny` 出处：Embark 开源 https://github.com/EmbarkStudios/cargo-deny 【官】：四类检查 licenses / bans（含同一 crate 多版本检测）/ advisories / sources。
- License 配置 https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html 【官】：不在 `allow` 里的一律拒；`copyleft` 字段已移除，copyleft 需显式允许；GNU 许可证「pedantically」匹配（`GPL-3.0` vs `GPL-3.0-only` vs `GPL-3.0-or-later` 是不同条目）；`[[licenses.exceptions]]` 针对单 crate 放行；`[licenses.private] ignore = true` 跳过不发布的工作区成员。
- `rust-ecosystem` 仓 https://github.com/EmbarkStudios/rust-ecosystem ：根目录有 `lints.rs`、`lints.toml`、`guidelines.md`、`audits.toml`；**该仓已于 2026-05-21 归档只读**【传·WebFetch 页面摘要】。`guidelines.md`：每个 crate 顶部 `#![warn(clippy::all)]`、`#![warn(rust_2018_idioms)]`，CI 查 clippy；rustfmt 上 CI；偏好 `parking_lot` 而非 `std::sync`；当时抱怨 lint 只能逐 crate 配置「more error prone」【官·原文】——此问题现已由 Cargo `[workspace.lints]`（1.74）解决【官·§3.3】。
- **对本项目**：
  - 我们是 GPL-3.0：cargo-deny `allow` 里要**精确写** `GPL-3.0-or-later`（或 `-only`，取决于 LICENSE 选择）以及 MIT / Apache-2.0 / ISC / BSD 等；注意 nih-plug README 写明其 VST3 绑定是 GPLv3、框架本身 ISC【官·nih-plug README】——GPL 项目吃 GPL 依赖没问题，但反过来提醒我们**所有依赖许可证都要和 GPL-3.0 兼容**（例如部分模型权重的非商用许可不是代码许可，cargo-deny 管不到，要人判）【推】。
  - `bans` 多版本检测：防止 `ndarray`/`ort` 被两个版本拉进来。
  - Embark 的 `parking_lot` 偏好**只适用于非实时线程**；实时线程不该有任何锁（§1.2 B1）。

---

## 4. 认真对待「超级语法树 / AST 做模块」

### 4.1 Rust 里「AST 级模块化」已经内建：crate 图
- Rust 的 crate 是编译单元，crate 之间的可见性与依赖方向由编译器强制（`pub` / `pub(crate)`、`Cargo.toml` 不声明就无法 `use`）。matklad：「the most important property of a crate is which crates it doesn't (transitively) depend on」（§3.7）。rust-analyzer 用 crate 边界表达「syntax 完全独立于其余部分」（§3.6）。
- 【推】人说的「用 AST 把东西做成模块、可拆分的核心包」在 Rust 里最直接的实现就是：**细粒度 crate + 明确的依赖方向**。这比任何自建 AST 工具都硬，因为违反它根本编译不过。

### 4.2 proc-macro（`syn` 解析 Rust AST 生成代码）
- 社区用法：nih-plug 的 `nih_plug_derive` 为 `Params` 生成参数注册、稳定 `#[id]`、`#[persist]` 序列化【官·nih-plug README】；Firewheel 的 `firewheel-macros` 生成参数 diff/patch【官·DESIGN_DOC】。
- 代价：matklad「Fast Rust Builds」指出 `syn` 系宏卡编译关键路径，且 feature 统一可能把 `syn` 拖进本不需要的 crate；建议把 derive 重的代码放依赖图叶子（§3.7）。
- 【推】本项目 v1 参数很少（门控阈值、EQ 若干段、压缩参数、模型选择），手写一个参数 struct + 一个消息 enum 足够；proc-macro 收益小、代价确定。若将来参数多到要自动生成 UI/持久化，再加**一个**位于叶子的 `vdn-macros`。

### 4.3 外部 DSL → Rust 源码（`build.rs` 代码生成）
- Faust 编译器支持 `-lang rust`，并有若干 Rust 专用选项（不生成 `FaustDsp` trait 实现、不生成 libm FFI、是否强制 C ABI 布局）【传·Faust man page 搜索摘要 https://faustdoc.grame.fr/manual/options 】。
- IFC 2024 经验（§1.5 D5）：`crates/dsp/build.rs` 在 `faust-rebuild` feature 下才重新生成，生成物提交进仓；生成代码不地道，外面包一层手写 Rust【测·幻灯片文字】。
- 【推】适合「音色链 DSP」（EQ/压缩/限幅）想快速原型时用；但它引入 Faust 编译器这个构建依赖，且是一个 GPL 例外条款需核对的工具链。v1 不建议，手写 biquad/压缩器量很小；把它记为「音色链想大幅扩展时的选项」。

### 4.4 类型级「图」（fundsp）
- fundsp：`>>`（串联）`|`（并行）`&`（总线）`^`（分支）组成图表达式；通道数用 `typenum` 编码在类型里，「Connectivity is checked during compilation」；动态的 `Net` 分前端（编辑）/后端（实时渲染），后端「real-time safe」，`allocate()` 要在送进实时上下文前调用【官·README】。
- 【推】这是「把信号链当 AST」最纯粹的 Rust 方案。但我们的链是**固定线性**的（GTCRN → 门控 → 去混响尾 → 压缩限幅 → EQ），类型级图的编译期复杂度和报错可读性代价大于收益。可取的是它的两个**思想**：① 构建期/运行期前后端分离（与 Firewheel 一致）；② 每个 stage 声明输入输出形状并在构建期校验。

### 4.5 把 AST 工具用在「检查」而非「生成」——更贴合本项目
| 工具 | 基于什么 | 用途 | 出处 |
|---|---|---|---|
| clippy `disallowed-methods/types` | 编译器 HIR | 禁实时 crate 里的锁/分配/IO API | §2 |
| `cargo-semver-checks` | rustdoc JSON + Trustfall 查询 | 检查公共 API 变化（「sources its data from rustdoc's json output」） | https://github.com/obi1kenobi/cargo-semver-checks 【官】 |
| RTSan `#[nonblocking]` | 编译期插桩 + 运行时拦截 | 实时上下文违规 | §2 |
| `cargo-deny` bans | 依赖图 | 禁止 `engine` 依赖 `io-macos` 之类的反向依赖可用 `bans.deny` + `wrappers` 表达【推，wrappers 字段未在本次读到的页面里核对】 | §3.9 |
- 【推】这条路线回应了人的直觉（「模块化要能被机器看懂」），又不增加生成代码的维护面。

---

## 5. 汇总：七条社区技术决策 → 本项目规则候选

> 这些是**候选**，最终落到 `specs/` 或 `ARCHITECTURE.md` 前需人拍板（本篇不改仓库文件）。

1. **核心不做 IO、不可失败**（rust-analyzer §3.6；sans-IO §3.8；matklad How to Test §3.7）。`engine::Engine::process(&mut self, in: &[f32], out: &mut [f32], ctx: &FrameCtx)` 是 eval 与实时**唯一**入口；时间用样本计数传入。【官】出处 + 【推】落点
2. **IO 只在一个 crate**（rust-analyzer「only crate that knows about LSP」）。`io-macos` 独占 CoreAudio/cpal；`eval` 独占 WAV；`control` 独占配置/IPC 序列化；`engine` 以下类型不实现 `Serialize`。
3. **音频线程：不锁、不分配、不阻塞、不 panic；失败 = 旁路或沿用上一值**（Timur Doumler B1；Asahi 内核互锁 D1）。原声旁路兼作故障策略；推理进程看门狗失效 → 送音进程自动旁路。
4. **构建期 / 运行期分离，整体移交**（Firewheel DESIGN_DOC §1.6；fundsp `Net` 前后端 §4.4）。模型会话、DSP 链、缓冲区全部在非实时侧建好，一次性交给实时线程；旧对象回收也在非实时侧（C-DTOR-BLOCK §3.4）。
5. **控制面 = 封闭 enum 消息 + 原子快照**（Hobson ADC25 `ToProcessor` A2；rust-analyzer 封闭事件 enum；Eckes 原子 ordering A1）。参数走单向无锁队列，声纹结果走原子发布；ordering 写进规约。
6. **用机器证明实时安全：静态 + 动态两层**（§2）。`dsp` crate `no_std` 无 alloc；实时 crate 开 clippy restriction + `disallowed-methods`；debug/测试开 `assert_no_alloc`；CI 用 eval 语料跑 RTSan `#[nonblocking]`。RTSan 结果决定 ort/tract 谁能进实时线程。
7. **features 只做加法；可切换的东西在运行时选**（Cargo book §3.3）。GTCRN/FastEnhancer-M、ort/tract 不做互斥 feature。

工程组织附带规则：扁平 `crates/` + 虚拟 manifest + `xtask` + `version = "0.0.0"`（§3.1）；`[workspace.lints]` 统一（§3.3）；单一 `tests/it` 或 `src/tests.rs`（§3.7）；benchmark 也是 test、主干检查永远绿（Basic Things）；cargo-deny 精确列 GPL-3.0 兼容许可（§3.9）；`ARCHITECTURE.md` 写缺席型不变量并逐条配机器检查（§3.2）。

---

## 6. 未找到 / 未核实清单（诚实列出）

- RustConf / EuroRust / RustNation UK / RustWeek 的 Rust 音频演讲：未找到。
- BillyDM 的大会演讲视频：未找到（只有设计文档）。
- Robbert van der Helm（nih-plug 作者）的演讲或设计博文：未找到；只引用 nih-plug README 与 Cargo.toml。
- Fabian Giesen 与 Rust 音频：未找到（疑为 Fabian Renn-Giles）。
- 「RustAudio Discord 演讲汇总」：未找到；只有 rust.audio 社区站与 WeirdConstructor 的 gist。
- Ross Bencina 原文：URL 已知，本次抓取连接被拒，未读。
- ADC 2024 RTSan 演讲视频 URL：会议页没有嵌入，未列。
- A3–A7、B3–B5 只验证了标题与 URL，内容未读。
- Rust 1.75 自动跨 crate 内联：只有 PR 与搜索摘要，未核 release notes。
- clippy.toml 查找规则、clippy restriction lint 名、cargo-deny `wrappers` 字段：凭既有知识，未打开对应文档核对【推】。
- Asahi 的 Triforce / Bankstown 实现语言：未核实。
