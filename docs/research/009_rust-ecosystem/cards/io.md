# 009 · 卡组：实时音频 I/O 与线程

> 2026-10-30 · rusteco 子代理（AI 草稿，待人过目）。只读源码与 API，没跑 build / 推理。
> 路径前缀：`resources/` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`。
> 证据等级：【源码】亲眼读过的行 ·【官方】URL ·【API】GitHub / crates.io 返回 ·【推断】。

## 本组覆盖

**深读（读了热路径源码）**：voice-strip（★ 本组最大发现）· cpal · coreaudio-rs · audio_thread_priority · tympan-aspl
**中读（读了 IO 回调，主要当反例）**：prism · opennc-app · duomic · memmap2
**浅读（API / 官方）**：objc2-core-audio / objc2-audio-toolbox · coreaudio-sys · cubeb-coreaudio-rs · iceoryx2 · ipc-channel · shared_memory · pipewire-rs · thread-priority · HushMic · libASPL / BlackHole / roc-vad（C/C++ 对照）

本次新克隆进 `resources/`：tympan-aspl、prism、duomic、opennc-app、cubeb-coreaudio-rs、memmap2-rs、voice-strip（均 < 50 MB）。

## 推荐：首选 + 备选 + 理由（每个子需求一行）

| 子需求（落在哪个 crate） | 首选 | 备选 | 理由 |
|---|---|---|---|
| 麦克风采集（io-macos，送音进程） | **自写 HAL IOProc**（`AudioDeviceCreateIOProcID`），类型绑定用 `objc2-core-audio` 0.3 或像 voice-strip 那样手写几十行 FFI | `cpal` 0.18.2（原型、eval 工具、设备枚举） | IOProc 直接拿设备 sample_time / host_time，做时间轴对齐与漂移校正最省事（voice-strip 就这么干）；cpal 走 AUHAL + coreaudio-rs，帧数变化时回调里会读属性并 `Vec::resize`（见 coreaudio-rs 卡），不满足命门 2 的严格版 |
| 虚拟麦克风 HAL 驱动（hal-plugin） | **自写 Rust cdylib，照 voice-strip `vdev-macos-driver` 的结构**（只依赖 `libc`） | `tympan-aspl` 0.1.0（Rust ASPL 框架） | voice-strip 已证明纯 Rust 驱动可行且 IO 路径零分配零锁；tympan-aspl 设计好但 0 star、单版本、CI 不能端到端验证，做依赖太冒险，抄它的 `RealtimeContext` 思路即可 |
| 跨进程音频共享内存（送音↔HAL、送音↔推理） | **`libc::shm_open` + `mmap` + 固定 `#[repr(C)]` 全原子布局，自写**（协议照 voice-strip `shm.rs`） | `memmap2` 0.9.12 只负责映射 fd（它接受 `RawFd`） | 布局只有一份定义、两边共用；读端只读映射、永不等待、缺数据就当缺（我们：缺 → 原声）。iceoryx2 太重，shared_memory 已停更 |
| 进程内 SPSC（送音进程内部、推理进程内部） | `rtrb`（另一组详查） | `ringbuf` | voice-strip 两处热路径都用 rtrb 的 chunk API |
| 实时线程优先级（推理进程的降噪线程） | **自写 Mach time-constraint（~30 行）** 或 `audio_thread_priority` 0.38 | 同左互为备选 | audio_thread_priority 只做 time-constraint，**不支持 workgroup**；MPL-2.0 与 GPL-3.0 兼容 |
| os_workgroup / 工作区间（降噪线程按 10 ms 截止期提频） | **自写 FFI：`AudioWorkIntervalCreate` + `os_workgroup_join` + `interval_start/finish`**（照 voice-strip `rt.rs`） | 无现成 crate | objc2-audio-toolbox 明确跳过了 `AudioWorkIntervalCreate`；全 GitHub 只搜到零星应用内手写 |
| 声纹线程放能效核 | `libc` 的 `pthread_set_qos_class_self_np(QOS_CLASS_UTILITY/BACKGROUND)`【推断】 | `thread-priority` 3.1.1 | 不需要单独依赖；QoS 低档位由调度器优先放 E 核（Apple 文档语义，未在本组源码中验证） |
| 控制面（control：参数、状态、心跳之外的命令） | std `UnixStream`（无依赖） | `ipc-channel` 0.23.1 | 控制面不上实时线程，ipc-channel 走 serde 序列化，有分配，只能用于非实时；我们消息少，自己拼即可 |

## 本组最值得学的 2 个设计

1. **绝对时间轴 + 分槽 seqlock 的共享内存环**：`resources/voice-strip/crates/vdev-macos-driver/src/shm.rs:143`（write）· `:180`（read）· `:138`（magic 最后 Release 发布）。帧 `p` 落在固定槽 `(p/32)%512`，写端把槽序号先置奇数再置偶数；读端前后两次看到同一个偶数才算有效，否则这段当「缺」并填零、**从不等待写端**。驱动只读映射、不回写。对我们：送音→HAL、推理→送音两条链都能用同一份协议；推理进程崩了 = 槽不再更新 = 读端判「缺」→ 直接送原声，D03 天然成立，不需要任何超时锁。配套的「换映射」做法见 `plugin.rs:334`（`publish`：原子换指针后，等 `READERS` 计数归零再释放旧映射）与 `plugin.rs:796-797`（IO 线程只做 `fetch_add/fetch_sub`）。
2. **降噪工作线程加入 Audio Work Interval**：`resources/voice-strip/crates/denoise-neural/src/rt.rs:243-262`（创建并 join）· `:271-275`（每帧 `interval_start` 带截止期）。文件头注释 `rt.rs:21-25` 点破关键：**光有实时优先级不会提核心频率**，一个每 10 ms 只忙 0.1 ms 的线程会被功耗控制器放在最低频核上、慢约十倍。我们推理进程的 GTCRN 线程正是这种负载；D31（1/5 核）和 D32（45 ms）之间的权衡要靠它显式告诉系统截止期。

---

## 深读卡

### voice-strip（https://github.com/paolomainardi/voice-strip）

| 项 | 内容 |
|---|---|
| 做什么 | Rust 写的 Linux + macOS 麦克风效果链 app，DeepFilterNet3 降噪，输出为虚拟麦克风；macOS 侧**自带纯 Rust 的 AudioServerPlugIn 驱动**，与 app 用 POSIX 共享内存连接——几乎就是我们架构的单进程版 |
| 许可证 | `MIT OR Apache-2.0`【源码】`resources/voice-strip/Cargo.toml:18`；与 GPL-3.0 兼容：**是**。驱动 crate 只依赖 `libc`【源码】`crates/vdev-macos-driver/Cargo.toml`；降噪 crate 依赖作者 fork 的 DeepFilterNet（git rev，MIT/Apache）、ndarray、sha2，无不兼容项【源码】`crates/denoise-neural/Cargo.toml` |
| 维护状态 | v0.7.3 / 2026-10-23；最近提交 2026-10-23；open issues 2；**仓库 2026-10-17 才建**，单人维护，README 自述大量由 AI 编码代理生成【API】【源码】`README.md` Warning 段。最近 5 个 issue/PR 全是作者自己发的，无外部 issue 可判响应速度【API】 |
| 实时安全做法 | ① 采集 IOProc 只做「拷通道 0 → 转换 → 处理 → 写环」，文件头声明无分配无锁无等待【源码】`crates/vdev-macos/src/backend.rs:5-9`，实现 `:106-213`；② 转换器有 `assert_no_alloc` 测试覆盖变块长与变比率【源码】`crates/vdev-macos/tests/convert_alloc.rs:1-30`；③ 驱动 `do_io_operation` 只有两次原子加减 + 一次环读【源码】`crates/vdev-macos-driver/src/plugin.rs:771-810`；④ shm 映射/重映射/心跳检测全放在驱动内一个 250 ms 轮询的看门线程里，不在 IO 线程【源码】`plugin.rs:352-415`；⑤ 推理（tract 会分配）不进音频线程，音频线程用 rtrb 交帧 + `Thread::unpark` 唤醒工作线程【源码】`crates/denoise-neural/src/lib.rs:4-9`、`:238-249`。⚠️ `unpark` 在 park 状态下是一次非阻塞内核唤醒，严格讲违背「零系统调用」，是实践中可接受的折中【推断】 |
| 值得学的一处 | 见上文「最值得学」1、2。另：设备时间轴直接钉在 host clock 上（`kAudioDeviceClockAlgorithmRaw` 语义），app 只凭 host time 就能算出写入位置，不用从驱动读回任何东西【源码】`crates/vdev-macos-driver/src/clock.rs:1-9`；驱动读位置 = 客户端 input sample_time − 固定 128 帧传输余量【源码】`plugin.rs:796`、`model.rs:19`。这让「送音进程写、HAL 读」不需要任何握手 |
| 我们怎么用 | **抄思路自己写**：hal-plugin（驱动骨架 + shm 读端）、io-macos（IOProc 采集 + shm 写端 + 时钟换算）、engine（work interval）。不直接依赖（未发布到 crates.io，`publish.workspace`） |
| 风险 | ① 项目只有两周，未经广泛实战；② 它自己注明 shm 模式 0644 对所有用户可读，「只够 spike，发布版要走认证通道」【源码】`shm.rs:7-10`——我们发布前必须解决；③ 单进程设计，推理在线程里；我们拆成两进程后，跨进程唤醒（替代 `unpark`）要自己定 |
| 证据 | 【源码】×12 ·【API】GitHub repo / releases / issues |

### cpal（https://github.com/RustAudio/cpal）

| 项 | 内容 |
|---|---|
| 做什么 | Rust 跨平台底层音频 I/O；macOS 后端基于 coreaudio-rs（AUHAL）+ objc2 家族 |
| 许可证 | `Apache-2.0`【API】；兼容 GPL-3.0：**是**（Apache-2.0 → GPL-3.0 单向兼容）。Apple 目标下的直接依赖：coreaudio-rs（MIT/Apache）、objc2-*（Zlib/Apache/MIT）、mach2（BSD-2/MIT/Apache）、dasp_sample（MIT/Apache）——无不兼容【API】crates.io 0.18.2 dependencies |
| 维护状态 | 0.18.2 / 2026-08-16；最近提交 2026-10-28；open issues 97；最近 5 个 issue/PR 中有 COLLABORATOR 回复（#1416、#1411 有 10 条讨论），活跃【API】 |
| 实时安全做法 | 输入回调里只读原子（`draining`、`pending_xrun.swap`）、包装数据指针后调用户回调，无分配【源码】`resources/cpal/src/host/coreaudio/macos/device.rs:621-657`；错误路径用 `try_lock`，拿不到就丢【源码】`resources/cpal/src/host/error_emit.rs:29`；overload 监听器在 RT 线程触发，只置标志【源码】`resources/cpal/src/host/coreaudio/macos.rs:195`。但底层 coreaudio-rs 输入回调在帧数变化时会分配（见下卡），cpal 层兜不住 |
| 值得学的一处 | xrun 用一个 `AtomicBool` 从 overload 监听线程传到数据回调，在回调里 `swap(false)` 后随 `CallbackInfo` 交给用户【源码】`device.rs:655-656`——把「出过事」这种低频事件塞进热路径的零成本做法，我们的送音进程报告「本周期用了原声旁路」可照做 |
| 我们怎么用 | **直接依赖，但只在非实时/原型处**：eval 的设备枚举、早期原型；正式送音进程首选自写 IOProc。做 HAL 虚拟设备 cpal 帮不上（它是客户端 API）——预判成立 |
| 风险 | 每个设备/默认设备变化要 cpal 重建流；AUHAL 额外一层缓冲带来的延迟未量化【推断】；跨平台抽象让 host_time / sample_time 细节被包装成 `StreamInstant` |
| 证据 | 【源码】×4 ·【API】×3 |

### coreaudio-rs（https://github.com/RustAudio/coreaudio-rs）

| 项 | 内容 |
|---|---|
| 做什么 | Core Audio / AudioUnit 的 Rust 友好封装，cpal 的 macOS 底座；0.14 起改用 objc2-* 绑定 |
| 许可证 | `MIT/Apache-2.0`（crates.io）；GitHub 标 Apache-2.0【API】；兼容：**是** |
| 维护状态 | 0.14.2 / 2026-04-29；最近提交 2026-06-18（仅 CI 依赖升级），最近实质 PR #152 2026-04；open issues 25；最近 5 个 issue 0 评论，外部 issue #155（2026-09-08）无人回复——**维护偏慢**【API】 |
| 实时安全做法 | `set_input_callback` 在注册时按当前 buffer 大小预分配 `AudioBufferList`【源码】`resources/coreaudio-rs/src/audio_unit/render_callback.rs:565-603`；**但**回调中一旦 `in_number_frames` 与预期不同，就在 RT 线程上 `get_property` 读流格式并 `vec.resize`【源码】`render_callback.rs:622`、`:627`、`:649`——这是分配 + 跨库调用，命门 2 **不满足**（变帧数场景：采样率转换、设备切换后） |
| 值得学的一处 | 反面：预分配思路对，但「尺寸变化就地扩容」不对；我们应在初始化时按 `kAudioDevicePropertyBufferFrameSizeRange` 的最大值一次分配，回调里只截取切片【推断】 |
| 我们怎么用 | **不要直接依赖**（间接经 cpal 用于原型可以） |
| 风险 | 维护慢；上述 RT 分配 |
| 证据 | 【源码】×4 ·【API】×3 |

### audio_thread_priority（https://github.com/mozilla/audio_thread_priority）

| 项 | 内容 |
|---|---|
| 做什么 | Firefox 用的「把线程提到实时音频优先级」小库；macOS 用 Mach time-constraint policy |
| 许可证 | `MPL-2.0`【API】；兼容 GPL-3.0：**是**（MPL-2.0 §3.3 允许与 GPL 组合，除非标注 Incompatible With Secondary Licenses，未见此标注）【推断】。macOS 依赖仅 `mach2`、`libc`、`log`【源码】`resources/audio_thread_priority/Cargo.toml:14-29` |
| 维护状态 | 0.38.0 / 2026-09-17；最近提交 2026-09-17；open issues 1；外部 issue #49（musl 编译）有 3 条回复并有修复 PR #50【API】 |
| 实时安全做法 | 只在线程启动时调用一次，不在热路径；computation 取回调周期一半、上限 50 ms【源码】`resources/audio_thread_priority/src/rt_mach.rs:113-134` |
| 值得学的一处 | 保存旧 policy 以便 demote 恢复【源码】`rt_mach.rs:91-106`。**不支持 os_workgroup**：全仓 grep `workgroup` 无结果【源码】（推翻「它可能支持 workgroup」的猜想） |
| 我们怎么用 | 推理进程降噪线程：**直接依赖或抄 30 行自写**，二选一；workgroup 另写 |
| 风险 | 参数「somewhat arbitrary」（源码自述 `rt_mach.rs:112`）；我们 10 ms 帧应自己给 period/computation/constraint（voice-strip 用 10/2/2.6 ms）【源码】`voice-strip/crates/denoise-neural/src/rt.rs:111-116` |
| 证据 | 【源码】×5 ·【API】×2 ·【推断】×1 |

### tympan-aspl（https://github.com/penta2himajin/tympan-aspl）

| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 的 AudioServerPlugIn 框架（相当于 Rust 版 libASPL）：FFI 层 + 实时原语层 + Driver/Device/Stream API + `plugin_entry!` 宏 + bundle/Info.plist 生成 |
| 许可证 | `MIT OR Apache-2.0`【API】；唯一运行时依赖 `crossbeam-utils`（MIT/Apache）【源码】`resources/tympan-aspl/Cargo.toml:32-33`；兼容：**是** |
| 维护状态 | 0.1.0 / 2026-05-17（crates.io 下载 52）；最近提交 2026-05-17，之后 5 个半月无提交；0 star，issues 全是作者自己的 PR【API】 |
| 实时安全做法 | `process_io` 先查原子状态再分派，注释写明无分配无内核调用【源码】`resources/tympan-aspl/src/driver.rs:343-357`；CI 用 `assert_no_alloc` 钉住每个示例的 `process_io`、TSan 跑环形缓冲（README CI 表）【源码】`README.md` |
| 值得学的一处 | `RealtimeContext`：一个 `PhantomData<*const ()>` 让它 `!Send`、只有框架能构造，凡要求在实时线程调用的 API 都收 `&RealtimeContext`，把「这段在 RT 线程」变成类型信息【源码】`src/realtime/context.rs:16-28`、`:52`。我们 engine 的 `process()` 可以照此给签名加标记 |
| 我们怎么用 | **抄思路**（类型标记、生命周期状态机、CI 分层）；不直接依赖 |
| 风险 | 没有跨进程 shm 设施（grep `shm_open/mmap` 无结果）；CI 自述 AMFI 拒绝 ad-hoc 签名，设备枚举与数据通路未自动验证；单人项目可能已停 |
| 证据 | 【源码】×4 ·【API】×2 |

---

## 中读卡（反例为主）

### prism（https://github.com/petitstrawberry/prism）

| 项 | 内容 |
|---|---|
| 做什么 | Rust 写的 macOS 64 通道虚拟音频分路驱动 + CLI/守护进程 |
| 许可证 | `MIT`【API】；依赖含 coreaudio-sys、cocoa/objc 0.2、serde 等，均宽松【源码】`resources/prism/Cargo.toml:17-27`；兼容：是 |
| 维护状态 | 最近提交 2025-12-06（> 10 个月前），13 star，0 open issues【API】 |
| 实时安全做法 | **反例**：`do_io_operation` 每次调用都 `format!` 拼日志并 `syslog`【源码】`resources/prism/src/driver.rs:1629-1633`、`:2000`；用 `static mut` 计数器【源码】`:1965`。混音用 vDSP 倒是对的【源码】`:12-43` |
| 值得学的一处 | 证明 coreaudio-sys 的 bindgen 输出含 `AudioServerPlugInDriverInterface`，可直接拿来填 vtable【源码】`driver.rs:3`、`:2052` |
| 我们怎么用 | **只看看**（当负例写进 specs：HAL IO 回调禁止格式化日志） |
| 风险 | — |
| 证据 | 【源码】×4 ·【API】 |

### opennc-app（https://github.com/MAkcanca/opennc-app）

| 项 | 内容 |
|---|---|
| 做什么 | Rust + cpal + ort + Slint 的桌面实时降噪 app，输出到虚拟线缆（Windows 用 VB-Cable，macOS 只能配 BlackHole，无打包） |
| 许可证 | `Apache-2.0`【API】；兼容：是 |
| 维护状态 | 2026-10-19 建仓，最近提交 2026-10-20，0 star【API】 |
| 实时安全做法 | **反例**：输入回调每满一个 hop 就 `Vec::with_capacity` 新分配并经 `mpsc::sync_channel<Vec<f32>>` 发送【源码】`resources/opennc-app/src/audio.rs:128`、`:138`；输出回调走 `Mutex<VecDeque>`【源码】`:152` |
| 值得学的一处 | ort 会话限 `intra_threads(1)` / `inter_threads(1)`【源码】`src/engine.rs:89-91`——单帧小模型时避免线程池抖动与多核唤醒，对 D31 有参考价值（推理组细查） |
| 我们怎么用 | **只看看** |
| 风险 | — |
| 证据 | 【源码】×4 ·【API】 |

### duomic（https://github.com/syy/duomic）

| 项 | 内容 |
|---|---|
| 做什么 | 把立体声 USB 麦拆成多个虚拟单声道麦；驱动用 libASPL（C++），CLI 用 Rust（cpal + memmap2）经共享内存喂驱动 |
| 许可证 | `MIT`【API】；兼容：是 |
| 维护状态 | 最近提交 2026-01-05，3 star【API】 |
| 实时安全做法 | **反例**：驱动 `OnReadClientInput` 每次先调 `connect()`，未连上时在 RT 线程里 `open/lseek/mmap`【源码】`resources/duomic/Driver/duomicDriver/Driver.cpp:150`、`:64-73`；驱动还持全局 `std::mutex`（非 IO 路径）【源码】`:55`。共享内存是 `/tmp` 下的普通文件 + memmap2【源码】`cli/src/ipc/shm.rs:6`、`:31-37` |
| 值得学的一处 | 仅一个 `writePos` + 固定目标延迟 1024 帧的极简读法【源码】`Driver.cpp:174-178`——对照 voice-strip 的槽协议，看清它缺什么（无撕裂检测、无代际、无心跳） |
| 我们怎么用 | **只看看**（负例：重映射必须在看门线程，不在 IO 回调） |
| 风险 | — |
| 证据 | 【源码】×5 ·【API】 |

### memmap2（https://github.com/RazrFalcon/memmap2-rs）

| 项 | 内容 |
|---|---|
| 做什么 | 跨平台内存映射 |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | 0.9.12 / 2026-10-21；最近提交 2026-10-21；open issues 20【API】 |
| 实时安全做法 | 不在热路径；映射建好后就是裸指针。支持对任意 `RawFd` 映射（可接 `shm_open` 的 fd）【源码】`resources/memmap2-rs/src/lib.rs:102`；有 `lock()`（mlock，防换页）【源码】`lib.rs:1060` |
| 值得学的一处 | `lock()`：实时读写的 shm 页应在初始化时 mlock，避免第一次触页缺页中断落在 IO 线程【推断】 |
| 我们怎么用 | **备选直接依赖**（若不想写 `mmap/munmap` 的 unsafe 封装）；首选 voice-strip 式 30 行 libc 封装 |
| 风险 | 无 |
| 证据 | 【源码】×2 ·【API】 |

---

## 浅读卡

### objc2-core-audio / objc2-audio-toolbox / objc2-core-audio-types（https://github.com/madsmtm/objc2）

| 项 | 内容 |
|---|---|
| 做什么 | 从 Apple SDK 自动生成的 Core Audio / AudioToolbox 绑定；cpal、coreaudio-rs 的底座 |
| 许可证 | `Zlib OR Apache-2.0 OR MIT`【API】crates.io；兼容：是。仓库 LICENSE.md 自述：派生自 Apple SDK，「是否允许分发派生物不清楚」，但实际需本机 Xcode SDK 链接【官方】https://github.com/madsmtm/objc2/blob/main/LICENSE.md |
| 维护状态 | 三个 crate 最新 0.3.2 / 2025-10-04（**已超 12 个月未发版**）；仓库最近推送 2026-10-17，open issues 111，作者常回复【API】——有条件满足命门 3 |
| 实时安全做法 | 纯 FFI 绑定，零开销（浅读） |
| 值得学的一处 | `objc2-core-audio` 有 `AudioServerPlugIn` feature，可拿驱动接口类型【官方】https://github.com/madsmtm/objc2/blob/main/framework-crates/objc2-core-audio/Cargo.toml （第 51、80 行）；**`objc2-audio-toolbox` 明确跳过 `AudioWorkIntervalCreate` 与 `AUAudioUnit.osWorkgroup`（缺 `os_workgroup_t`）**【官方】https://github.com/madsmtm/objc2/blob/main/framework-crates/objc2-audio-toolbox/translation-config.toml （第 56-62 行） |
| 我们怎么用 | io-macos / hal-plugin 的类型绑定**首选**（比 coreaudio-sys 免 bindgen/libclang）；workgroup 手写 FFI |
| 风险 | 0.3 → 0.4 升级可能破坏 API；发版节奏慢 |
| 证据 | 【API】×2 ·【官方】×3 |

### coreaudio-sys（https://github.com/RustAudio/coreaudio-sys）

| 项 | 内容 |
|---|---|
| 做什么 | bindgen 生成的 CoreAudio 原始绑定 |
| 许可证 | `MIT`【API】；构建依赖 bindgen（BSD-3）；兼容：是 |
| 维护状态 | 0.2.18 / 2026-06-04；最近推送 2026-09-09；open issues 19【API】 |
| 实时安全做法 | 纯绑定（浅读） |
| 值得学的一处 | 含 `AudioServerPlugInDriverInterface`（prism 实证）【源码】`resources/prism/src/driver.rs:2052` |
| 我们怎么用 | **备选**绑定（objc2-core-audio 不够用时） |
| 风险 | 构建时跑 bindgen，需要 libclang + Xcode SDK，CI 慢 |
| 证据 | 【API】·【源码】×1 |

### cubeb-coreaudio-rs（https://github.com/mozilla/cubeb-coreaudio-rs）

| 项 | 内容 |
|---|---|
| 做什么 | Firefox 在 Mac 上的音频后端（Rust） |
| 许可证 | `ISC`【API】；依赖 ringbuf、triple_buffer、mach2、cubeb-backend 等，宽松【源码】`resources/cubeb-coreaudio-rs/Cargo.toml:10-22`；兼容：是 |
| 维护状态 | 最近提交 2026-10-21；open issues 34；不单独发 crates.io【API】 |
| 实时安全做法 | 输入回调把数据推进预分配的 buffer manager【源码】`src/backend/mod.rs:637-638`；但全双工日志路径每回调 `push` 一条记录【源码】`mod.rs:645-656`（容量是否预留未查）。未见 workgroup 用法（grep 无） |
| 值得学的一处 | 工业级处理设备切换/聚合设备的经验在这里（5484 行 `mod.rs`），遇到设备热插拔问题时回来查 |
| 我们怎么用 | **只看看** |
| 风险 | 体量大、与 cubeb C API 耦合 |
| 证据 | 【源码】×3 ·【API】 |

### iceoryx2（https://github.com/eclipse-iceoryx/iceoryx2）

| 项 | 内容 |
|---|---|
| 做什么 | 零拷贝无锁进程间通信框架（Rust 核心） |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | 0.10.0 / 2026-09-18；天天有提交；open issues 246；成员回复积极【API】 |
| 实时安全做法 | 自称 lock-free、零拷贝；macOS 为 **tier 2**【官方】https://github.com/eclipse-iceoryx/iceoryx2#supported-platforms （浅读，未读源码） |
| 值得学的一处 | 未深读 |
| 我们怎么用 | **不要**：normal 依赖 19 个（含 serde、toml、flatbuffers）【API】crates.io 0.10.0 dependencies；我们只需两块固定布局 shm，且 HAL 驱动跑在 `coreaudiod` 沙箱里，引入它的服务发现/配置文件机制风险大【推断】 |
| 风险 | 体量与沙箱兼容性 |
| 证据 | 【API】×2 ·【官方】·【推断】 |

### ipc-channel（https://github.com/servo/ipc-channel）

| 项 | 内容 |
|---|---|
| 做什么 | Servo 的多进程 channel，macOS 下走 Mach port |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | 0.23.1 / 2026-10-24；open issues 66；#457「remote-peer can trigger UB/panic」2026-09-30 未见回复【API】 |
| 实时安全做法 | 基于 serde 序列化 → 有分配（浅读、【推断】），不能上 RT 线程 |
| 我们怎么用 | control 的**备选**；首选 std Unix socket |
| 风险 | #457 安全问题未决 |
| 证据 | 【API】×2 ·【推断】 |

### shared_memory（https://github.com/elast0ny/shared_memory）

| 项 | 内容 |
|---|---|
| 做什么 | 原生共享内存的 Rust 封装（README 标题只写 Linux/Windows） |
| 许可证 | crates.io `MIT OR Apache-2.0`，GitHub 无 SPDX【API】 |
| 维护状态 | 0.12.4 / **2022-03-01**；最近推送 2023-09；2026-06 有人发 issue 问「还维护吗」无回复【API】 |
| 我们怎么用 | **不要**（命门 3 不满足） |
| 证据 | 【API】×3 |

### pipewire-rs（https://gitlab.freedesktop.org/pipewire/pipewire-rs）

| 项 | 内容 |
|---|---|
| 做什么 | PipeWire 的 Rust 绑定（Linux），只看思路 |
| 许可证 | `MIT`【API】crates.io `pipewire` 0.10.1 / 2026-08-19 |
| 值得学的一处 | 思路上，Linux 侧「虚拟麦克风 = 一个 filter 节点」的模型与 macOS HAL 驱动完全不同；voice-strip 的 `crates/vdev-linux/` 是同一项目两种落法的对照（未深读）【源码】目录存在 |
| 我们怎么用 | **只看看**（只做 Mac） |
| 证据 | 【API】 |

### thread-priority（https://github.com/iddm/thread-priority）

| 项 | 内容 |
|---|---|
| 做什么 | 跨平台线程优先级/调度策略 |
| 许可证 | `MIT`【API】；兼容：是 |
| 维护状态 | 3.1.1 / 2026-06-20；推送 2026-10-19；open issues 6【API】 |
| 我们怎么用 | 声纹线程降 QoS 的**备选**（是否覆盖 macOS QoS class 未核，浅读） |
| 证据 | 【API】 |

### HushMic（https://github.com/Fovty/HushMic）

| 项 | 内容 |
|---|---|
| 做什么 | Linux 系统级虚拟麦实时降噪（DPDFNet + PipeWire），Rust 为主 |
| 许可证 | `Apache-2.0`【API】 |
| 维护状态 | 2026-10-23 推送，238 star【API】 |
| 我们怎么用 | **只看看**：Linux only；但它用 DPDFNet（resources/ 里已有），推理组可能对其流式状态管理有兴趣【推断】。未克隆（10.8 MB，非 Mac） |
| 证据 | 【API】 |

### libASPL / BlackHole / roc-vad（C/C++ 对照，已在 resources/）

| 项 | 内容 |
|---|---|
| 做什么 | libASPL：C++17 写 ASPL 的库；BlackHole：C 写的回环驱动；roc-vad：基于 libASPL 的网络虚拟声卡 |
| 许可证 | libASPL `MIT` · BlackHole 源码 `GPL-3.0`（【源码】`resources/BlackHole/LICENSE:3`；GitHub 显示 NOASSERTION 是因为附带二进制版权声明）· roc-vad `MPL-2.0`【API】；三者与 GPL-3.0 都兼容 |
| 维护状态 | libASPL 推送 2025-08-17；BlackHole 2026-10-28；roc-vad 2025-04-13【API】 |
| 我们怎么用 | **只看看**：hal-plugin 写属性表时对照 libASPL 的对象/属性覆盖面；BlackHole 对照时钟与零时间戳；duomic 是「libASPL 驱动 + Rust 客户端」的现成拼法 |
| 证据 | 【源码】×1 ·【API】 |

---

## 没查到 / 存疑

1. **跨进程唤醒用什么最好**：voice-strip 是单进程，用 `Thread::unpark`。我们送音进程→推理进程要跨进程通知「新帧到了」。候选：推理线程按 10 ms 周期自定时轮询（work interval 本身就给周期，最简单）、POSIX 命名信号量 `sem_post`、Mach `semaphore_signal`（要传端口）、`os_sync_wake_by_address_any`（macOS 14.4+ 支持跨进程共享内存地址？未核实）。本组没有找到 Rust 现成实现，**存疑，建议 008/p1 做一次有界小实验**。
2. **workgroup 能否跨进程共享**：理想是推理线程加入**麦克风设备 IO 线程的 workgroup**（`kAudioDevicePropertyIOThreadOSWorkgroup`），这样调度器把两进程当一个截止期链。os_workgroup 有 `os_workgroup_copy_port` / `create_with_port` 可跨进程传，但本组没找到任何 Rust 或开源实例；voice-strip 是新建独立 work interval【源码】`rt.rs:243`。**存疑**。
3. **HAL 驱动沙箱里 POSIX shm 的权限**：voice-strip 用 0644 让 `_coreaudiod` 可读并自认不适合发布【源码】`shm.rs:7-10`；它说的「认证通道」具体怎么做（Info.plist `AudioServerPlugIn_MachServices` + XPC 传 fd？）没查到实现。**存疑，hal-plugin 发布前必须定**。
4. **E 核放置**：`pthread_set_qos_class_self_np` 低 QoS 会优先上 E 核是 Apple 文档的一般语义，本组未在任何 Rust 源码里见到声纹类负载的实测；`libc` crate 是否导出该函数与 `qos_class_t` 常量未逐行核对。【推断】
5. **未克隆**：iceoryx2（26 MB，判为不用）、HushMic（非 Mac）、objc2 全仓（13 MB，只用 GitHub API 读了两个配置文件）。
6. **GitHub 搜「AudioServerPlugIn rust」只有 tympan-aspl 一个框架**；带驱动的 Rust 应用只找到 voice-strip、prism、duomic（C++ 驱动）。可能有私有/未标注语言的仓漏网。
7. 本组未覆盖：rtrb / ringbuf / triple-buffer / seqlock / assert_no_alloc / rtsan（已在 resources/，属「实时零件」组）。
