# 009 · 卡片组：插件 / 音频图 / 宿主架构

> 2026-10-30 · 调研线 `research/009-rust-ecosystem` · 本组只读代码与 API，未构建、未运行。
> 路径约定：【源码】里的 `resources/<仓>/…` 相对 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。
> 本次新克隆（浅克隆，均 < 1 GB）：nih-plug · Firewheel · clack · kira · dasp。

## 本组覆盖

**深读（读了热路径源码）**：nih-plug · Firewheel · clack · kira · dasp（dasp_graph）
**浅读（只看 API / 官方信息）**：Meadowlark · knyst · glicol · rodio · oddio · symphonia · assert_no_alloc · rtsan-standalone-rs · basedrop · triple_buffer

一句话结论：**我们的链路是固定的（重采样 → 降噪 → 输出），不需要通用音频图，也不做插件格式**。本组没有一个该当直接依赖的「框架」；要拿的是**设计**：生命周期分层、原子参数 + 平滑器、「新对象在非实时线程造好 → 无锁队列送进去 → 旧对象送回去再释放」、预计算淡化表的交叉淡化、FTZ + 调试期分配断言。

## 首选 + 备选（每个子需求一行）

| 子需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| engine 生命周期分层（非实时造 / 实时跑 / 非实时拆） | 抄 clack 的类型分离：`activate()` 在主线程构造处理器、`process()` 实时、`deactivate(self)` 吃掉处理器（clack/plugin/src/plugin.rs:198,225,249） | 抄 nih-plug 三段 `initialize / reset / process`（nih-plug/src/plugin.rs:204,216,239） | 用类型系统把「可以分配」和「不可以分配」切成两个类型，比靠注释约束更硬；nih-plug 的 `reset()` 正好对应我们「清 GRU/卷积缓存」 |
| 参数跨线程（control → 实时线程） | 自写：每个参数一个原子量（`AtomicF32` 模式，nih-plug/src/params/float.rs:17）；跨进程放进共享内存 | kira 的「只要最新值」通道（triple_buffer，kira/crates/kira/src/command.rs:28-59） | 我们参数少（开关、强度、模型选择），原子量最省；triple_buffer 不能跨进程，只适合进程内 |
| 参数平滑（降噪强度、旁路） | 抄 nih-plug `Smoother`（smoothing.rs:47,260,283）：原子 `steps_left`/`step_size`，逐样本或逐块取值 | kira `Tween`（kira/crates/kira/src/tween.rs:84） | Smoother 的「最后一步直接落到目标值」处理避免了累积误差（smoothing.rs:298-301） |
| GTCRN ↔ FastEnhancer-M 交叉淡化 | 抄 Firewheel `Declicker::process_crossfade` + 预计算淡化表 `DeclickValues`（firewheel-core/src/dsp/declick.rs:17,113,486） | 用 Smoother(Linear) 生成增益自己乘 | 预计算表 SIMD 友好、零分配；默认 3 ms（declick.rs:494）对我们偏短，模型切换建议 20–50 ms |
| 实时线程不释放内存（换模型、换调度） | 抄 Firewheel：新调度 Box 从 ringbuf 送进，旧调度原路送回主线程释放（firewheel-graph/src/processor/handle_messages.rs:69-116） | basedrop / Firewheel 自带 `collector`（ArcGc，firewheel-core/src/collector.rs:1-2） | 「送回去再 drop」实现最简单，用已在 resources 里的 rtrb 即可 |
| 热路径零分配验证 | 自写 debug-only 全局分配器守卫（思路同 nih-plug 用 assert_no_alloc：nih-plug/src/wrapper/util.rs:35-37,189-199） | rtsan-standalone-rs（还能抓锁与系统调用） | assert_no_alloc 上次发布 2021、仓库 2023 后无推送，不合「仍在维护」；逻辑很小，自写更稳。rtsan 能管「零锁零系统调用」，但要构建 LLVM 运行时，放 CI |
| 非正规数（denormal）防护 | 抄 nih-plug `ScopedFtz`，aarch64 的 FPCR FZ 位（nih-plug/src/wrapper/util.rs:25,191） | no_denormals crate（MIT，0.3.0 · 2025-09） | GRU/卷积尾部衰减极易进 denormal，Apple Silicon 上要显式开 FZ |
| 延迟上报（latency_samples） | 抄 nih-plug：原子 swap，变了才通知非实时线程（nih-plug/src/wrapper/clap/wrapper.rs:1737-1745） | clack：延迟只在 activate 内允许变（clack/extensions/src/latency.rs:99） | 我们由 engine 上报，HAL 驱动读 `kAudioDevicePropertyLatency`；采 clack 的规矩更简单：**换模型 = 重激活**，延迟只在激活时定 |
| 后台任务（模型加载、声纹注册） | 抄 nih-plug `BackgroundTask` 有界队列 + `try_send` 失败即丢（nih-plug/src/event_loop/background_thread.rs:61-69） | Firewheel 的事件组送回模式（handle_messages.rs:44） | 但队列换成 rtrb：crossbeam 首次在新线程调用会因线程本地存储分配（background_thread.rs:62-63 作者自注） |
| 音频图 | **不要图**，自写固定管线 | Firewheel（真要动态图时） | 链路固定；dasp_graph 每次 process 都 DFS 遍历（dasp/dasp_graph/src/lib.rs:322），且 2020 后没发版 |
| 离线评测读 WAV（eval） | symphonia（MPL-2.0，0.6.1 · 2026-08） | hound（本组未查，见存疑） | 只用在 eval，不进实时路径 |

## 本组最值得学的两个设计

1. **Firewheel 的「造新 → 送进 → 换下 → 送回 → 主线程释放」**：`new_schedule()` 在音频线程只做 `mem::swap` 与指针搬运，旧的 `ScheduleHeapData` 通过 `try_push(DropSchedule)` 送回主线程（resources/Firewheel/crates/firewheel-graph/src/processor/handle_messages.rs:69-116；消息枚举 resources/Firewheel/crates/firewheel-graph/src/processor.rs:242-256）。连事件用的 `Vec` 也送回去释放（handle_messages.rs:44）。**对我们**：GTCRN ↔ FastEnhancer-M 切换时，新模型的会话与缓存状态在非实时线程建好并预热，Box 指针经 rtrb 送进实时线程，交叉淡化结束后旧模型原路送回释放——实时线程全程不分配、不释放。
2. **nih-plug 的 `process_wrapper`**：每次进入处理函数先开 FTZ，调试构建下再套 `assert_no_alloc`（resources/nih-plug/src/wrapper/util.rs:189-199，全局分配器在 :35-37）。**对我们**：送音进程回调和推理进程降噪线程的入口统一套一层「FTZ + 调试期禁分配」守卫；离线 eval 走同一份处理代码时同样套上，这样单元测试就能抓到热路径分配。

---

## 卡片

### nih-plug（https://github.com/robbert-vdh/nih-plug）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | Rust 写 VST3/CLAP 插件的框架，含参数系统、平滑器、STFT 辅助、独立运行（standalone）包装 |
| 许可证 | 框架本体 ISC（resources/nih-plug/LICENSE:1）；**与 GPL-3.0 兼容：是**（ISC 是宽松许可）。VST3 绑定 `vst3-sys` 是 GPLv3（README.md:231-234），对我们无妨；依赖多为 git 依赖（clap-sys、assert_no_alloc 分支、baseview：Cargo.toml:89,101,106） |
| 维护状态 | **不在 crates.io**（API 查 `nih_plug` 无结果），只有滚动 tag `latest-builds` 2026-09-03；最近提交 2026-09-03；open issues 125；最近 5 个 issue（#281、#279、#275、#274、#271）**维护者都没回复**——响应慢，基本是一人维护 |
| 实时安全做法 | ①生命周期分层：`initialize()` 可分配，`reset()` 可能在音频线程调用、禁止分配，`process()` 实时（src/plugin.rs:204,214-216,239）。②参数是原子量：`FloatParam` 的 `value: AtomicF32`、`smoothed: Smoother`（src/params/float.rs:17,34）；Smoother 内全是原子（src/params/smoothing.rs:54-63），作者自注是为了与 GUI 共享（:44-45）。③`Buffer` 里的切片 `Vec` 预分配，每次只改指针（src/buffer.rs:24-31,121-133）。④可选 `assert_process_allocs`：调试期把全局分配器换成 `AllocDisabler`，在处理函数外套断言（src/wrapper/util.rs:35-37,189-199）。⑤后台任务用 crossbeam 有界队列 `try_send`，满了就丢（src/event_loop/background_thread.rs:61-69,81）。**反例**：CLAP 包装的 process 里对插件实例拿 `parking_lot::Mutex`（src/wrapper/clap/wrapper.rs:108,2239）——无竞争时便宜，但严格讲违反「零锁」；macOS 上 `schedule_gui` 从音频线程调 Obj-C `isMainThread`（src/event_loop/macos.rs:89-115）；`schedule` 外面包 `permit_alloc`，作者承认首次在新线程会分配（background_thread.rs:62-63） |
| 值得学的一处 | `Params` derive 宏：`#[proc_macro_derive(Params, attributes(id, persist, nested))]`（nih_plug_derive/src/lib.rs:13）为结构体生成 `unsafe impl Params`，`param_map()` 返回「稳定字符串 ID → 参数指针 → 分组」（nih_plug_derive/src/params.rs:415-418；trait 在 src/params.rs:282,298）。**稳定 ID** 是关键：我们的 control 面板与送音/推理两进程之间传参数，也该用稳定 ID（或哈希，src/wrapper/util.rs:40 `hash_param_id`）而不是下标，免得加参数后错位。另：延迟上报 `set_latency_samples` 原子 swap、变了才通知（src/wrapper/clap/wrapper.rs:1737-1745），可在 initialize 或 process 里调（plugins/crossover/src/lib.rs:210,231,236）；`StftHelper::latency_samples`（src/util/stft.rs:36,256）把 STFT 帧引入的延迟算成样本数 |
| 我们怎么用 | **抄思路自己写**：engine（生命周期 + latency_samples 上报）、dsp（Smoother、FTZ 守卫）、control（稳定参数 ID）。不做插件格式，不直接依赖 |
| 风险 | 不在 crates.io、依赖 git 分支，直接依赖会让构建可复现性变差；一人维护、issue 不回；它的线程模型面向 DAW 宿主，与我们「两进程 + 共享内存」不同，参数原子量不能原样跨进程 |
| 证据 | 【源码】plugin.rs、params/*、buffer.rs、wrapper/util.rs、wrapper/clap/wrapper.rs、event_loop/*、nih_plug_derive/*；【API】GitHub repo/releases/issues、crates.io（无此 crate） |

### Firewheel（https://github.com/BillyDM/Firewheel）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | 面向游戏与应用的中层音频引擎：节点 + 音频图编译成调度 + 实时处理器（Meadowlark 作者 BillyDM 的后续工作） |
| 许可证 | `MIT OR Apache-2.0`（crates.io；仓库带 LICENSE-MIT 与 LICENSE-APACHE；GitHub 识别为 Apache-2.0）；**兼容：是**。依赖 ringbuf / thunderdome / smallvec / arrayvec / bevy_platform（firewheel-core/Cargo.toml:107-113）均宽松许可；triple_buffer 是 MPL-2.0（兼容 GPL-3.0）【推断：MPL-2.0 §3.3 允许与 GPL 组合，除非标了「不兼容次级许可」】 |
| 维护状态 | 0.14.0 · 2026-09-05（crates.io）；最近提交 2026-09-05；open issues 26；最近 5 个 issue 中 3 个有维护者回复，响应中等 |
| 实时安全做法 | 设计文档明说「不用互斥锁」（DESIGN_DOC.md:31）。主线程 ↔ 处理器用 `ringbuf` 单生产单消费（firewheel-graph/src/processor.rs:125-126，容量在 context.rs:322 预分配），时钟用 triple_buffer（processor.rs:136）。节点拆成两个 trait：`AudioNode`（主线程，`construct_processor` 可分配，firewheel-core/src/node.rs:266,285）与 `AudioNodeProcessor: 'static + Send`（实时，`events / bypassed / process`，node.rs:486,502,515,534）；`new_stream` 明确在主线程调、可分配（node.rs:559-560）。旧调度和事件 Vec 都送回主线程释放（handle_messages.rs:44,116），主线程定期 `GlobalRtGc::collect()`（graph.rs:795） |
| 值得学的一处 | ①上面「首选设计 1」。②旁路去咔嗒由图处理器自动做（process.rs:359-396），节点不用管；`Declicker` 状态机四态 `SettledAt0/SettledAt1/FadingTo0/FadingTo1`（dsp/declick.rs:17-27），`process_crossfade(a, b, …)` 在两路缓冲之间淡化（declick.rs:113），淡化曲线预计算成表（`DeclickValues`，declick.rs:486-494）——**直接对应我们 GTCRN ↔ FastEnhancer-M 交叉淡化与原声旁路淡入淡出** |
| 我们怎么用 | **抄思路自己写**（engine：模型热切换协议；dsp：Declicker）。不直接依赖：我们不需要图编译 |
| 风险 | 节点可声明 `latency_frames`（node.rs:92,164），但我在 firewheel-graph 里没搜到用它做自动延迟补偿【推断：grep 无命中】——两模型延迟不同时，补齐要我们自己做；默认淡化 3 ms 对模型切换太短；0.x 版本 API 变动频繁 |
| 证据 | 【源码】node.rs、processor.rs、handle_messages.rs、process.rs、declick.rs、collector.rs、graph.rs、context.rs、DESIGN_DOC.md；【API】GitHub、crates.io；【推断】延迟补偿缺失、MPL 兼容性 |

### clack（https://github.com/prokopyl/clack）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | CLAP 插件标准的安全 Rust 绑定，插件侧（clack-plugin）与宿主侧（clack-host）分开 |
| 许可证 | `MIT OR Apache-2.0`（crates.io；仓库 LICENSE-MIT/LICENSE-APACHE）；**兼容：是** |
| 维护状态 | clack-plugin / clack-host 0.2.0 · 2026-09-12（GitHub release v0.2 同日）；最近提交 2026-09-12；open issues 4；最近 5 个 issue 3 个有维护者回复，全部已关闭——响应好 |
| 实时安全做法 | 用**类型**分三份线程归属：`PluginShared: Send + Sync`（任意线程，plugin/src/plugin.rs:71）、`PluginMainThread`（非 Send，钉在主线程，:84）、`PluginAudioProcessor: Send`（不 Sync，:170）。处理器由 `activate()` 在主线程构造、可分配（:198 起，文档写明非实时安全），`process()` 必须实时安全（:225），`deactivate(self, …)` 按值吃掉处理器（:249）。FFI 入口包 `catch_unwind`，panic 变成 `CLAP_PROCESS_ERROR` 而不是把宿主带崩（plugin/src/extensions/wrapper.rs:23,164；plugin/src/plugin/instance.rs:307-320） |
| 值得学的一处 | 「处理器只在激活期存在」的类型状态：没激活就根本没有 AudioProcessor 对象，`process` 无从调用。**对我们**：engine 的模型适配器做成 `ModelConfig →(非实时 activate)→ ActiveModel →(实时 process)→ (非实时 deactivate)`，换模型 = 造一个新的 ActiveModel；`latency_samples` 也照 clack 的规矩「只在 activate 时定」（extensions/src/latency.rs:99-102,113） |
| 我们怎么用 | **抄思路自己写**（engine trait 形状）。只有将来要出 CLAP 插件版才直接依赖 clack-plugin |
| 风险 | 0.x；CLAP 生态在 macOS 上主要是 DAW，与虚拟麦克风无关；panic 捕获我们用两进程隔离已覆盖，进程内仍需 `catch_unwind` 或 panic=abort 策略二选一（见存疑） |
| 证据 | 【源码】plugin/src/plugin.rs、plugin/src/plugin/instance.rs、plugin/src/extensions/wrapper.rs、extensions/src/latency.rs；【API】GitHub、crates.io |

### kira（https://github.com/tesselode/kira）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | 游戏音频库：声音播放、轨道混音、效果、调制器、带缓动的参数补间（Tween） |
| 许可证 | `MIT OR Apache-2.0`（crates.io）；**兼容：是**；依赖 rtrb（宽松）、triple_buffer（MPL-2.0，兼容） |
| 维护状态 | 0.12.6 · 2026-10-28（release 同日）；最近提交 2026-10-27/28；open issues 33；最近 5 个 issue 4 个有维护者回复——活跃 |
| 实时安全做法 | 两种跨线程通道分工明确（kira/crates/kira/src/command.rs:1-20 文档）：**只要最新值** → `CommandWriter/Reader` 包 triple_buffer（:28-59），写方覆盖、读方无新值返回 None（:47-53）；**要 FIFO** → rtrb。资源（声音、轨道）新增走 rtrb 送进（backend/resources.rs:132），移除时实时侧把对象推回 `unused_resource_producer` 让主线程释放（:159-170），key 列表 `Vec::with_capacity` 预分配（:118）。`Mutex` 只出现在主线程一侧的控制器上（:190-191），实时侧不碰 |
| 值得学的一处 | 「最新值」与「FIFO」两类消息分开选原语。**对我们**：降噪强度、旁路开关这类用「最新值」（原子量或 triple_buffer）；「切换到模型 X」「这是新的声纹嵌入」这类必须一个不丢的用 rtrb FIFO。推理进程把声纹结果原子发布给降噪线程，正是「最新值」语义 |
| 我们怎么用 | **抄思路**（control/engine 的消息分类）。不直接依赖（它是播放库，不是处理管线） |
| 风险 | `remove_unused` 在送回队列满时 panic（resources.rs:167-169），抄的时候要改成「满了留到下一块再试」 |
| 证据 | 【源码】command.rs、backend/resources.rs、tween.rs；【API】GitHub、crates.io |

### dasp / dasp_graph（https://github.com/RustAudio/dasp）【深读 dasp_graph】
| 项 | 内容 |
|---|---|
| 做什么 | 数字信号基础件集合（采样类型、帧、插值、环形缓冲、RMS、包络），dasp_graph 是基于 petgraph 的极简音频图 |
| 许可证 | `MIT OR Apache-2.0`（dasp/dasp_graph/Cargo.toml:8；GitHub 识别为 NOASSERTION 是因为双许可证文件）；**兼容：是** |
| 维护状态 | dasp / dasp_graph 0.11.0 · 2020（crates.io）；默认分支最近提交 2025-09-09；open issues 68；最近 5 个 issue 2 个有维护者回复。**近 12 个月无发版，不合命门 3** |
| 实时安全做法 | 节点缓冲是定长数组 `[f32; 64]`（dasp_graph/src/buffer.rs:6-12），不分配；但 `process()` 每次调用都重置并重跑 DFS 后序遍历（lib.rs:322），`inputs` 是 `Vec`，`clear()` 后 `push()`（:326,334）——容量够就不分配，入边数一涨就会分配；图本身不能跨线程改 |
| 值得学的一处 | 定长 64 帧块作为图内统一单位（buffer.rs:12）。我们 GTCRN 帧移固定，可同样在管线内部用定长块，回调块长不定时由环形缓冲吸收 |
| 我们怎么用 | **不要**（图）；dasp 其余子件只看看，dsp crate 自己写 |
| 风险 | 停更；petgraph 0.5 老版本（Cargo.toml:28） |
| 证据 | 【源码】dasp_graph/src/buffer.rs、lib.rs、Cargo.toml；【API】GitHub、crates.io |

### Meadowlark（https://github.com/MeadowlarkDAW/Meadowlark）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 未完成的开源 DAW |
| 许可证 | AGPL-3.0（GitHub API）；与 GPL-3.0 **有条件兼容**（GPLv3 §13 允许与 AGPLv3 组合，但 AGPL 部分仍按 AGPL，网络交互条款随之而来）【推断】 |
| 维护状态 | GitHub 仓 README 写明已迁往 Codeberg；GitHub 最近推送 2025-09-22；最近 5 个 issue 维护者 0 回复（都在 2024）【API】 |
| 实时安全做法 | 未读；其音频引擎思路已由同作者沉淀进 Firewheel【推断】 |
| 值得学的一处 | 无（看 Firewheel） |
| 我们怎么用 | 不要 |
| 风险 | AGPL；GitHub 版已停 |
| 证据 | 【API】GitHub repo/readme/issues；【推断】 |

### knyst（https://github.com/ErikNatanael/knyst）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 实时音频合成框架，运行时可改的节点图 |
| 许可证 | crates.io `MIT OR Apache-2.0`（GitHub 识别 Apache-2.0）；兼容：是 |
| 维护状态 | 0.5.1 · 2024-11-25；最近推送 2024-11-25；open issues 5；当时维护者回复积极，但**近 12 个月无提交，不合命门 3**【API】 |
| 实时安全做法 | 未读 |
| 值得学的一处 | — |
| 我们怎么用 | 不要 |
| 风险 | 停更 |
| 证据 | 【API】 |

### glicol（https://github.com/chaosprint/glicol）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 图导向的现场编码音乐语言与 DSP 引擎（含 wasm） |
| 许可证 | GitHub 识别 MIT；crates.io 字段为非标准写法；兼容：是（以 MIT 计）【API】 |
| 维护状态 | crates 0.13.5 · 2024-04；GitHub release 停在 2022；默认分支最近提交 2025-01-23；open issues 51；**不合命门 3**【API】 |
| 实时安全做法 | 未读 |
| 值得学的一处 | — |
| 我们怎么用 | 不要 |
| 风险 | 停更；仓库 ~114 MB |
| 证据 | 【API】 |

### rodio（https://github.com/RustAudio/rodio）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 基于 cpal 的音频播放库（拉式 `Source` 迭代器） |
| 许可证 | crates.io `MIT OR Apache-2.0`（GitHub 识别 Apache-2.0）；兼容：是 |
| 维护状态 | 0.23.0 · 2026-10-11；最近提交 2026-09-07；open issues 173；最近 5 个 issue 只 1 个有维护者回复【API】 |
| 实时安全做法 | 未读源码 |
| 值得学的一处 | — |
| 我们怎么用 | 不要（播放库，与采集 → 处理 → 虚拟麦克风无关） |
| 风险 | — |
| 证据 | 【API】 |

### oddio（https://github.com/Ralith/oddio）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 轻量游戏音频库（空间化、混音） |
| 许可证 | crates.io `MIT OR Apache-2.0`；兼容：是 |
| 维护状态 | 0.7.4 · 2023-10-15；最近推送 2023-10；**不合命门 3**【API】 |
| 实时安全做法 | 未读 |
| 值得学的一处 | — |
| 我们怎么用 | 不要 |
| 风险 | 停更 |
| 证据 | 【API】 |

### symphonia（https://github.com/pdeljanov/Symphonia）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 音频解码与容器解析（WAV、FLAC、MP3、AAC 等） |
| 许可证 | MPL-2.0；与 GPL-3.0 **兼容：是**【推断：MPL-2.0 默认允许作为 GPL 作品的一部分分发，除非声明不兼容次级许可；未逐文件核查该声明】 |
| 维护状态 | 0.6.1 · 2026-08-13（release 同日）；open issues 121；最近 5 个 issue（2026-10-25~29）维护者暂无回复，但都是一周内新开的【API】 |
| 实时安全做法 | 不进实时路径，不评 |
| 值得学的一处 | — |
| 我们怎么用 | eval crate 读评测音频的候选（直接依赖）；若评测只有 WAV，可换更小的 hound |
| 风险 | 体量大，只为读 WAV 不划算 |
| 证据 | 【API】；【推断】许可证 |

### assert_no_alloc（https://github.com/Windfisch/rust-assert-no-alloc）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 调试期全局分配器：在 `assert_no_alloc(|| …)` 范围内一分配就报错或中止 |
| 许可证 | crates.io `BSD-1-Clause`；兼容：是 |
| 维护状态 | 1.1.2 · 2021-08-03；最近推送 2023-08；open issues 10。nih-plug 用的是作者分支 `feature/nested-permit-forbid`（resources/nih-plug/Cargo.toml:101）。**不合命门 3** |
| 实时安全做法 | 本身就是实时安全检查工具；只抓分配，不抓锁和系统调用 |
| 值得学的一处 | `permit_alloc` 局部豁免（日志、断言），nih-plug 的调试宏都包在里面（resources/nih-plug/src/debug.rs:57,97） |
| 我们怎么用 | **抄思路自己写**一个几十行的 debug-only `GlobalAlloc` 守卫（线程本地计数 + 豁免），放 dsp 或 engine 的 dev 工具里 |
| 风险 | 只在 debug 构建生效；release 下的分配要靠 rtsan 或 Instruments |
| 证据 | 【源码】nih-plug 的使用处；【API】GitHub、crates.io |

### rtsan-standalone-rs（https://github.com/realtime-sanitizer/rtsan-standalone-rs）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | LLVM RealtimeSanitizer 的 Rust 绑定：给函数标「非阻塞」，运行时抓分配、锁、系统调用 |
| 许可证 | Apache-2.0；兼容：是 |
| 维护状态 | crates 0.3.0 · 2025-09-27；最近推送 2026-09-15；open issues 1【API】 |
| 实时安全做法 | 正是三条命门中「零分配零锁零系统调用」的运行时检查器 |
| 值得学的一处 | — |
| 我们怎么用 | 备选：CI 里对 eval 跑一遍离线处理（与实时回调同一份代码）当回归检查 |
| 风险 | macOS/aarch64 支持程度未核实；需要构建 C++ 运行时，属于重活（本次未试）【推断】 |
| 证据 | 【API】；【推断】 |

### basedrop（https://github.com/micahrj/basedrop）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 实时安全的延迟回收智能指针（`Shared`/`Owned` 在实时线程 drop，交给收集器线程释放） |
| 许可证 | crates.io `MIT/Apache-2.0`；兼容：是 |
| 维护状态 | 0.1.3 · 2025-10-29；最近推送 2025-10-29；open issues 4【API】 |
| 实时安全做法 | 未读源码；Firewheel 自带同类 `collector`（resources/Firewheel/crates/firewheel-core/src/collector.rs:1-2） |
| 值得学的一处 | — |
| 我们怎么用 | 备选；首选是更简单的「rtrb 送回再 drop」 |
| 风险 | 0.1；近 12 个月只此一次发版 |
| 证据 | 【API】【源码】Firewheel 同类实现 |

### triple_buffer（https://github.com/HadrienG2/triple-buffer）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 单写单读的「最新值」无锁三缓冲 |
| 许可证 | MPL-2.0；兼容：是【推断，同 symphonia】 |
| 维护状态 | 9.0.0 · 2026-02-22；最近推送 2026-07-29；open issues 2【API】 |
| 实时安全做法 | kira（command.rs:22,59）、Firewheel（processor.rs:136）都用它在实时线程读最新值 |
| 值得学的一处 | — |
| 我们怎么用 | 推理进程内：声纹线程 → 降噪线程发布最新嵌入/门控结果的候选（直接依赖）；跨进程不能用，要在共享内存里自写同语义（seqlock 或三槽 + 原子索引） |
| 风险 | 只适合进程内 |
| 证据 | 【API】【源码】kira、Firewheel 的使用处 |

---

## 没查到 / 存疑

1. **自动延迟补偿**：本组所有框架都只有「上报延迟」，没有现成的「两个延迟不同的处理器之间先补齐再交叉淡化」实现（Firewheel 的 `latency_frames` 在图里没搜到使用）。GTCRN 与 FastEnhancer-M 若前瞻不同，补齐逻辑要我们自己写进 engine。【推断】
2. **神经模型热切换的「预热」**：所有参考都是无状态或状态很短的 DSP 节点。带 GRU/卷积缓存的模型切过去如果从零状态开始，前几十毫秒会有伪迹；我建议新模型先在后台跟跑 N 帧再开始淡化——没有找到现成先例。【推断】
3. **ONNX 推理本身是否零分配**：本组框架只保证「框架层」不分配；推理库调用（ort 等）在热路径是否分配、加锁，归 engine/推理组核查，这里不下结论。
4. **跨进程参数通道**：本组原语（triple_buffer、ringbuf、crossbeam）都只在进程内有效；共享内存里的 SPSC 环与原子发布要另查（rtrb 是否能放进 `mmap` 区域需核实）。
5. **panic 策略**：clack 用 `catch_unwind` 把 panic 变成错误码；我们是两进程，推理进程可直接 `panic = "abort"` 让送音进程走原声旁路，但送音进程该选哪种，待 rustarch 线定。
6. **rtsan 在 macOS/aarch64 上的可用性**：只查了仓库活跃度，未验证能否构建。
7. **hound**（WAV 读写）未查许可证与维护状态。
8. nih-plug 的 `editor()` 在 plugin.rs:165，只看了签名（编辑器在宿主线程创建、通过 `Arc<Params>` 读参数），没深读 GUI 包装。
