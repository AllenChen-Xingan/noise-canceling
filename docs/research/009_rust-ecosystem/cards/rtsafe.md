# 009 · 卡组：实时安全零件（rtsafe）

> 2026-10-30 · 调研子代理 · AI 产出，待大总管/人复核。路径前缀 `resources/` 均相对 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。
> 证据等级：【源码】亲读行 ·【官方】URL ·【API】GitHub/crates.io API（2026-10-30 取）·【推断】。

## 本组覆盖

深读（读了热路径源码）：**rtrb · ringbuf · triple_buffer · basedrop · assert_no_alloc · rtsan-standalone-rs**
中读（读了关键几行）：arc-swap · crossbeam ArrayQueue · seqlock · no_denormals · nih-plug（ScopedFtz / AtomicF32）· atomic_float
浅读（只看 API/官方）：nice-assert-no-alloc · portable-atomic · rustc `-Zsanitizer=realtime`

本次新浅克隆进 `resources/`：ringbuf、basedrop、triple-buffer、arc-swap、rust-assert-no-alloc、rtsan-standalone-rs、no_denormals、nice-assert-no-alloc、atomic_float、crossbeam、seqlock（rtrb、nih-plug 已存在）。全部 < 10 MB，无过大。

## 一句话总判

**跨进程那一段（送音进程 ↔ 推理进程的共享内存）没有任何一个现成 crate 能直接用**：rtrb / ringbuf / triple_buffer 都把索引和数据指针放在堆上的 `Box`/`Arc` 里，指针在另一个进程地址空间里无效【源码】resources/rtrb/src/lib.rs:155-160；resources/ringbuf/src/storage.rs:57-60；resources/triple-buffer/src/lib.rs:238（`shared: Arc<SharedState<T>>`）。（复核补：ringbuf 的 `StaticRb<T, N>` = `SharedRb<Array<T, N>>` 是内联数组、本身不含指针【源码】resources/ringbuf/src/alias.rs:17、resources/ringbuf/src/storage.rs:102，但结构不是 `#[repr(C)]`、构造只按值返回、无跨进程语义保证，结论「自写」不变【推断】）所以：**进程内用现成 crate，跨进程自写一个 `#[repr(C)]`、只用偏移不用指针的 SPSC 环 + 三缓冲，照抄 rtrb 的「缓存对端索引」和 triple_buffer 的「一个原子字节换槽位」两招**。

## 推荐（每个子需求一行）

| 子需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| 进程内 SPSC 音频环（如采集回调 → 送音主循环、推理进程内 IO 线程 → 降噪线程） | **rtrb 0.4.1** | ringbuf 0.6.0 | rtrb push/pop 热路径只有 1 次 Acquire load（仅在看似满/空时）+ 1 次 Release store，无分配无锁；10 月仍在发版；API 稳。ringbuf 功能更多但 2026-10 刚爆出两个 soundness 问题（#65/#66，已于 2026-10-30 随 0.6.0 的破坏性变更关闭【API】，修复未经时间检验）且 0.4→0.6 API 在剧烈变动 |
| 跨进程共享内存 SPSC 环（送音 ↔ 推理） | **自写**（抄 rtrb 思路） | 自写 voice-strip 式「绝对时间轴 + 分槽序号」环（io 卡首选） | 现成的都不支持放进 shm（见上）；要 `#[repr(C)]` 头 + `CachePadded<AtomicU64>` head/tail + 偏移寻址。⚠️ 复核异议：与 io 卡对同一条链给了不同协议，见文末复核记录 |
| 「最新值」发布（声纹线程 → 降噪线程的门控判定；推理 → 送音的心跳/状态） | **triple_buffer 9.0.0**（进程内） | 自写 shm 版三缓冲；arc-swap（仅非 RT 端） | publish/update 各一次 `swap(AcqRel)`，零分配零锁，读写都不等待；MPL-2.0 与 GPL-3.0 兼容 |
| 单个参数（衰减下限 dB、旁路开关、增益） | **自写 `AtomicU32` + `f32::to_bits`**（十来行） | portable-atomic `float` 特性；atomic_float | atomic_float 两年没发版；做法本身极简，没必要背依赖 |
| 大对象换装（切模型 GTCRN↔FastEnhancer、换声纹质心）后旧对象的释放 | **rtrb 回收队列**（复核改首选：RT 线程把旧 `Box` push 回非 RT 线程 drop，满了就暂存下一块再试；voice-strip 已这样落地【源码】resources/voice-strip/crates/engine/src/processor.rs:327-331、350-355） | basedrop（或抄其 collector 思路自写） | 少一个依赖、我们反正要用 rtrb；basedrop 最后发版 2025-10-29，到今天已超 12 个月（命门 3 不满足），SharedCell 有未回复的竞态报告 |
| 调试期抓「RT 线程分配」 | **assert_no_alloc（用 nice-assert-no-alloc 分叉）** | — | 稳定版 Rust、零外部依赖、只看 Rust 全局分配器 |
| 调试期抓「分配 + 锁 + 系统调用 + C/C++ 里的 malloc（ORT！）」 | **rtsan-standalone 0.3.0** | rustc nightly `-Zsanitizer=realtime` | 拦截 libc 层，能看到 ONNX Runtime 内部的 malloc/mutex，这是 assert_no_alloc 看不到的；构建时不设 `RTSAN_ENABLE` 就是空操作 |
| 去非正规数（FTZ） | **自写 RAII guard（抄 nih-plug ScopedFtz，`msr fpcr` 置 bit 24）** | no_denormals 0.3.0 | 6 行内联汇编；aarch64 只有 FZ 位，没有单独 DAZ |
| MPMC 有界队列（若真需要） | crossbeam `ArrayQueue` | — | 无分配，但争用时 `snooze()` 会 `yield_now()` 系统调用；SPSC 场景用 rtrb 更合适 |
| seqlock | **不要**（seqlock crate） | 三缓冲替代 | 写端用 parking_lot Mutex，读端争用时 `yield_now()`；2023 起无提交。（复核注：只否定这个 crate。io 卡首推的 voice-strip shm 环本身就是「每槽 seqlock」，但数据格是 `AtomicU32`、读端撕裂即判缺、从不重试，不在此列【源码】resources/voice-strip/crates/vdev-macos-driver/src/shm.rs:143-199） |

## 本组最值得学的 2 个设计

1. **rtrb 的「缓存对端索引」**：生产者本地缓存 `cached_head`，只有在「按缓存看已满」时才去 Acquire 读一次真实 head【源码】resources/rtrb/src/lib.rs:490-503（消费者对称：lib.rs:778-791）。大多数 push/pop 只碰自己那条缓存行，跨核流量最小。**自写 shm 环时一字不差地沿用这个结构**（本地 Cell 缓存不进 shm，只有 head/tail 两个 CachePadded 原子进 shm）。
2. **triple_buffer 的「一个原子字节 = 槽位索引 + 脏位」**：写端 `swap(input_idx | DIRTY, AcqRel)` 换出旧后备槽【源码】resources/triple-buffer/src/lib.rs:339-345；读端先 Relaxed 看脏位、有新值才 `swap(output_idx, AcqRel)`【源码】lib.rs:516-547。读写两端都**永不等待**、永不失败，读端拿到的始终是一份完整快照。这正是「声纹线程原子发布结果」和「推理进程 → 送音进程发布状态/心跳」需要的形状，且三个槽是固定大小 `#[repr(C)]` 时**可以原样搬进共享内存**（索引是 0..3 的小整数，不是指针）【推断】。

---

## 项目卡

### rtrb（https://github.com/mgeier/rtrb）
| 项 | 内容 |
|---|---|
| 做什么 | 实时安全的无等待 SPSC 环形缓冲（crossbeam 作者的 spsc 分支演化而来），支持逐个与按块（chunk）读写 |
| 许可证 | MIT OR Apache-2.0；与 GPL-3.0 **兼容**（两者都是宽松许可，GPL-3.0 可吸收）；运行时依赖为零（只有 dev-dependencies rand/criterion）【源码】resources/rtrb/Cargo.toml:25-31 |
| 维护状态 | 0.4.1 发布于 2026-10-25【API crates.io】；最近提交 2026-10-25「Release 0.4.1」【源码 git log】；open issues 12【API】；最近 5 个 issue 全是维护者本人开的 PR/发布任务（#199–#203，2026-10-25），说明维护者活跃，但看不出对外部 issue 的响应速度【API】 |
| 实时安全做法 | `push`：`next_tail()` 先用本地缓存判满，只有疑似满才 `head.load(Acquire)`，然后写槽 + `tail.store(Release)`，**无分配无锁无系统调用**【源码】resources/rtrb/src/lib.rs:346-357、490-503。`pop` 对称【源码】lib.rs:592-600、778-791。按块写 `push_entire_slice` = `write_chunk_uninit` + 两段 `copy_to_uninit` + `commit_all`，同样无分配【源码】resources/rtrb/src/chunks.rs:307-317。**唯一分配点**在 `RingBuffer::new`（`Box::new` + `Vec::with_capacity`）【源码】lib.rs:150-160；**唯一释放点**：生产者与消费者都 drop 后，后 drop 的那一侧 `Box::from_raw` 释放整块内存【源码】resources/rtrb/src/arc_ring_buffer.rs:47-58、80-85 —— 若后 drop 的是 RT 线程，会在 RT 线程上 free |
| 值得学的一处 | 缓存对端索引（见文件头「最值得学」#1），lib.rs:490-503；head/tail 各占一条 `CachePadded` 缓存行避免伪共享【源码】lib.rs:104-109 |
| 我们怎么用 | **直接依赖**：`io-macos`（CoreAudio 输入回调 → 送音主线程）、`engine` 进程内的 IO ↔ 降噪线程。**抄思路自写**：跨进程 shm 环（建议放在一个小的 `shm` 模块，属 `io-macos` 或单列，由架构线定） |
| 风险 | ① 不能放进共享内存（堆指针）；② 关停时注意让非 RT 线程最后 drop，否则 RT 线程上 free；③ 元素类型若 `needs_drop`，按块读的 drop 有额外循环——我们只放 `f32`，无影响【推断】 |
| 证据 | 【源码】×8 【API】×3 |

### ringbuf（https://github.com/agerasev/ringbuf）
| 项 | 内容 |
|---|---|
| 做什么 | 功能更全的无锁 SPSC FIFO：本地/共享两种 rb、可选外部存储（数组/切片/引用/堆）、可选缓存包装、async/blocking 子 crate |
| 许可证 | MIT OR Apache-2.0，**兼容**；依赖 crossbeam-utils（MIT/Apache）、可选 portable-atomic（Apache/MIT）【源码】resources/ringbuf/Cargo.toml:7、34-37 |
| 维护状态 | 0.6.0 发布于 **2026-10-30（今天）**【API crates.io】；最近提交 2026-10-30「Support owned and borrowed exclusive buffer handles」【源码 git log】；open issues 7；最近 5 个 issue 均有回复（1–2 条），维护者 agerasev 对 #65（10-26 报）在 10-30 回复确认并当天关闭【API】。但 GitHub Releases 页最后一条仍是 v0.4.0（2024-04）【API】 |
| 实时安全做法 | 共享版索引 `read_index`/`write_index` 为 `CachePadded<AtomicUsize>`，Acquire 读 / Release 写【源码】resources/ringbuf/src/rb/shared.rs:51-56、98-104、209-215。缓存包装 `CachingProd::try_push`：先按本地缓存判满，满了才 `fetch()` 刷新对端索引，与 rtrb 同一招【源码】resources/ringbuf/src/wrap/caching.rs:124-138、58-65。热路径无分配无锁 |
| 值得学的一处 | `Storage` trait 把「存储从哪来」与「环的逻辑」分开，`Ref` 存储只是 `ptr + len`【源码】resources/ringbuf/src/storage.rs:57-60 —— 思路可借给 shm 环：存储由 mmap 区域提供。但注意 `Ref` 存的是**裸指针**，跨进程仍不可用 |
| 我们怎么用 | **只看看**（rtrb 的备选）；不作首选 |
| 风险 | ① **soundness**：#65「Public API is unsound」（2026-10-26，ASan 报 use-after-free，维护者确认并称修复需破坏性变更）、#66「safe traits 但提供 unsafe 能力」——两者均已于 2026-10-30 关闭，对应 0.6.0 当天发布【API】；修复刚出、未经时间检验；cubeb-coreaudio-rs 还钉在 0.5.2【源码】resources/cubeb-coreaudio-rs/Cargo.toml:20；② 0.4→0.5→0.6 一年内多次破坏性升级，跟版成本高【推断】；③ 同样不能直接放进 shm |
| 证据 | 【源码】×6 【API】×4 【推断】×1 |

### triple_buffer（https://github.com/HadrienG2/triple-buffer）
| 项 | 内容 |
|---|---|
| 做什么 | 单写单读的三缓冲：写端随时发布新值，读端随时取最新完整快照，两端互不等待 |
| 许可证 | **MPL-2.0**；与 GPL-3.0 **兼容**（MPL-2.0 §3.3 允许与 GPL 组合，除非文件附 Exhibit B "Incompatible With Secondary Licenses" 声明；复核 grep 全仓只命中 `LICENSE` 条款正文与 Exhibit B 模板（LICENSE:24、:369-372），无源文件附此声明【源码】）；依赖仅 crossbeam-utils（MIT/Apache）【源码】resources/triple-buffer/Cargo.toml:22 及 `[dependencies]` |
| 维护状态 | 9.0.0 发布于 2026-02-22【API crates.io / GitHub release】；最近提交 2026-07-29【源码 git log】；open issues 2；最近 5 个均为维护者自己的 PR，已合并【API】。MSRV 1.86【源码】Cargo.toml:24；crates 徽章自标 `passively-maintained`【源码】Cargo.toml:27 |
| 实时安全做法 | 三个 `CachePadded<UnsafeCell<T>>` 槽 + 一个 `CachePadded` 原子字节 `back_info`【源码】resources/triple-buffer/src/lib.rs:566-569。写端 `publish()` = 一次 `swap(AcqRel)`【源码】lib.rs:320-348；读端 `update()` = 先 Relaxed 看脏位，有新值才一次 `swap(AcqRel)`【源码】lib.rs:516-547。**零分配零锁零系统调用，且是 wait-free**。`write(value)` 是 move 进槽，T 若带堆内存则旧值 drop 时会 free——我们放定长 `#[repr(C)]` 结构（声纹得分、门控状态、时间戳）就没有这个问题【推断】 |
| 值得学的一处 | 见文件头「最值得学」#2；注释里把 AcqRel 为什么不能降成 Release 讲得很透（lib.rs:321-337），可直接作为我们 shm 版的正确性论证 |
| 我们怎么用 | **直接依赖**：`speaker`（声纹线程 → 降噪线程发布「是不是你 / 置信度」）。**抄思路自写**：shm 版三缓冲（推理 → 送音发布心跳/状态；送音 → 推理发布控制参数），放在 shm 模块 |
| 风险 | ① 读端每次只拿最新值，中间值会丢（对门控是想要的行为）；② 进程内版本不能放 shm（`Arc<SharedState>`，lib.rs:238）；④ 维护者自标被动维护（Cargo.toml:27）；③ MPL-2.0 是文件级 copyleft：若我们改了它的文件，改动须以 MPL 公开——我们只依赖不改，无影响【推断】 |
| 证据 | 【源码】×8 【API】×3 【推断】×3 |

### basedrop（https://github.com/micahrj/basedrop)
| 项 | 内容 |
|---|---|
| 做什么 | 实时音频用的内存管理：`Owned<T>`/`Shared<T>` 在 RT 线程被 drop 时不 free，而是挂到 collector 的无锁队列，由非 RT 线程 `collect()` 真正释放；另有 `SharedCell` 原子换指针 |
| 许可证 | MIT/Apache-2.0，**兼容**；零依赖【源码】resources/basedrop/Cargo.toml（`[dependencies]` 为空）|
| 维护状态 | 0.1.3 发布于 2025-10-29【API】；最近提交 2025-10-29「prepare for release of 0.1.3」【源码 git log】——到 2026-10-30 已**超出 12 个月一天，命门 3 严格说不满足**；open issues 4；#10「SharedCell::get 与 replace 之间无序，可能二次 queue_drop」（2026-09-22）**0 回复**【API】。整体：低频维护，响应慢 |
| 实时安全做法 | `Owned::drop` / `Shared::drop`（引用计数归零时）只调 `Node::queue_drop`【源码】resources/basedrop/src/owned.rs:64-66；resources/basedrop/src/shared.rs:102-108。`queue_drop` = 一次 `tail.swap(AcqRel)` + 一次 `next.store(Release)`，**节点是侵入式的（分配时就带链表头），入队不再分配**【源码】resources/basedrop/src/collector.rs:97-102。释放发生在 `Collector::collect()`（非 RT 线程调用）【源码】collector.rs:237、268-284。**注意** `SharedCell::replace` 会自旋等待读者计数归零（`while readers != 0 {}`）【源码】resources/basedrop/src/shared_cell.rs:112-118，所以 replace 只能在非 RT 线程调；RT 线程只调 `get()`（复核更正：是 4 次原子——readers 加、node load、`clone` 引用计数加、readers 减）【源码】shared_cell.rs:61-71。另：`Owned::new`/`Shared::new` 要分配节点，只能在非 RT 线程建【推断】 |
| 值得学的一处 | 「侵入式节点 + 单原子 swap 入队」的延迟释放（collector.rs:97-102）：RT 线程丢弃旧模型状态 / 旧声纹质心时 O(1)、无分配、无锁 |
| 我们怎么用 | **只抄思路**；复核后首选改为 rtrb 回收队列（见推荐表，voice-strip 已实证）。basedrop 适合将来出现「RT 线程上 `Shared` 引用计数归零」这类回收队列表达不了的场景再引入 |
| 风险 | ① #10 竞态报告未回复（报告者自称用 agent 写的，需自己复核）；② 2024 年 Miri 报过 collector 测试的数据竞争（#7，后由 #9 修订内存序）【API】；③ 维护边缘 |
| 证据 | 【源码】×7 【API】×4 【推断】×1 |

### arc-swap（https://github.com/vorner/arc-swap）
| 项 | 内容 |
|---|---|
| 做什么 | `ArcSwap<T>`：原子可替换的 `Arc<T>`，读多写少的配置/快照发布 |
| 许可证 | MIT OR Apache-2.0，**兼容**【API crates.io】 |
| 维护状态 | 1.9.3 发布于 2026-10-18；最近提交 2026-10-18；open issues 23；最近 5 个 issue 2 个有维护者/他人回复，3 个未回复【API】 |
| 实时安全做法（中读） | `load()` 走 debt 列表，常态无锁；但**每个线程第一次 load 时会分配一个 debt 节点并 `Box::leak`**【源码】resources/arc-swap/src/debt/list.rs:151、169、223-229。另外读端 `Guard` drop 时若是最后一个引用会在 RT 线程 free 旧 `T`；`store/swap` 写端会 `wait_for_readers`【源码】resources/arc-swap/src/lib.rs:480-493 |
| 值得学的一处 | 「首次使用才分配线程本地节点」这一坑本身值得记：任何带 thread_local 懒初始化的库，在 RT 线程开始工作前都要先「预热」调一次【推断】 |
| 我们怎么用 | **只看看 / 只用于非 RT 端**（`control` 进程内配置热更新可用）。RT 线程不用 |
| 风险 | #210「1.9.1 上 Guard 观察到被释放并复用的内存」open（2026-09-15，3 条讨论）【API】；RT 端分配/释放点隐蔽 |
| 证据 | 【源码】×4 【API】×3 【推断】×1 |

### crossbeam · ArrayQueue（https://github.com/crossbeam-rs/crossbeam）
| 项 | 内容 |
|---|---|
| 做什么 | 有界 MPMC 无锁队列（预分配槽 + stamp） |
| 许可证 | MIT OR Apache-2.0，**兼容**【API】 |
| 维护状态 | crossbeam-queue 0.3.14 发布于 2026-09-05；仓最近推送 2026-10-01；open issues 175（大仓，积压多）【API】 |
| 实时安全做法（中读） | push/pop 不分配；CAS 失败时 `backoff.spin()`，但在槽位被他人占用的路径会 `backoff.snooze()`【源码】resources/crossbeam/crossbeam-queue/src/array_queue.rs:189-199、376-392，而 `snooze` 超过阈值后调 `thread::yield_now()` ——**系统调用**【源码】resources/crossbeam/crossbeam-utils/src/backoff.rs:207-219 |
| 值得学的一处 | 每槽带 stamp（lap + index）判断槽位状态，无需全局锁——但对我们的 SPSC 用不上 |
| 我们怎么用 | **不要**（SPSC 用 rtrb）。`crossbeam-utils::CachePadded` 会作为 ringbuf/triple_buffer 的传递依赖进来，自写 shm 环可直接用它 |
| 风险 | MPMC 的 CAS 争用在 RT 线程上不 wait-free |
| 证据 | 【源码】×2 【API】×2 |

### seqlock（https://github.com/Amanieu/seqlock）
| 项 | 内容 |
|---|---|
| 做什么 | 序列锁：写端递增序号写数据，读端读前后序号比对重试 |
| 许可证 | Apache-2.0/MIT，**兼容**【API】 |
| 维护状态 | 0.2.0 发布于 2023-03-30；最近推送 2023-10-16——**不满足近 12 个月维护**【API】 |
| 实时安全做法（中读） | 写端拿 `parking_lot::Mutex`【源码】resources/seqlock/src/lib.rs:57、70、186；读端遇到写进行中会 `thread::yield_now()`【源码】lib.rs:117-129；读用 `ptr::read_volatile` 读非原子数据【源码】lib.rs:136——在 Rust 内存模型下是否算数据竞争有争议（issue #10）【API】 |
| 值得学的一处 | 读端「读前序号 → 读数据 → Acquire fence → 读后序号」的骨架（lib.rs:121-145） |
| 我们怎么用 | **不要**。「最新值」用三缓冲，读写都不重试 |
| 风险 | 无人维护；读端可能无限重试（写端高频时） |
| 证据 | 【源码】×4 【API】×2 |

### assert_no_alloc（https://github.com/Windfisch/rust-assert-no-alloc）
| 项 | 内容 |
|---|---|
| 做什么 | 自定义全局分配器：在 `assert_no_alloc(|| …)` 闭包内若发生 Rust 堆分配/释放就中止（或计数告警）；`permit_alloc` 局部放行 |
| 许可证 | BSD-1-Clause（GitHub 显示 NOASSERTION，Cargo.toml 写 BSD-1-Clause）【源码】resources/rust-assert-no-alloc/Cargo.toml:6；与 GPL-3.0 **兼容**（极宽松许可）；可选依赖 backtrace/log 均 MIT/Apache【推断】 |
| 维护状态 | 1.1.2 发布于 **2021-08-03**；最近推送 2023-08-09；open issues 10，最近 5 个（2023–2024）中 4 个 0 回复、#17「New crates.io release?」仅 1 条外部用户回复【API】——**原仓已停维护**（#17 只有一条外部用户的 +1，无维护者回复）。活分叉：**nice-assert-no-alloc 1.0.0（2026-05-17，BillyDM）**，内容基本是原仓 + 合并「嵌套 assert_no_alloc / permit_alloc 修复」（原仓 PR #13 一直没合）【源码】resources/nice-assert-no-alloc 与原仓 diff 主要是格式化；【API】nice-assert-no-alloc PR #1。nih-plug 至今用的是 robbert-vdh 的 git 分支【源码】resources/nih-plug/Cargo.toml:101 |
| 实时安全做法 | 线程本地两个计数器 `ALLOC_FORBID_COUNT` / `ALLOC_PERMIT_COUNT`【源码】resources/rust-assert-no-alloc/src/lib.rs:35-37；`GlobalAlloc::alloc/dealloc` 先 `check()`：forbid>0 且 permit==0 就 `handle_alloc_error` 中止【源码】lib.rs:151-175、178-188。默认特性 `disable_release`：release 构建里整个变空操作【源码】lib.rs:43-50、Cargo.toml `[features]` |
| 值得学的一处 | RAII Guard 增减计数器，保证 `catch_unwind` 时计数也正确（lib.rs:60-74）——我们自己的「RT 区段」标记也该这样写 |
| 我们怎么用 | **直接依赖（dev/debug 特性）**，用 nice-assert-no-alloc；包在 `engine::process_block` 与送音回调外层，只在 debug 与离线 `eval` 对拍时开 |
| 风险 | ① 只看 Rust 全局分配器：**ONNX Runtime（C++）内部 malloc、CoreAudio 内部分配一律看不见**；② 看不到锁和系统调用；③ 原仓无人维护 |
| 证据 | 【源码】×6 【API】×3 【推断】×1 |

### rtsan-standalone-rs（https://github.com/realtime-sanitizer/rtsan-standalone-rs）
| 项 | 内容 |
|---|---|
| 做什么 | 把 LLVM RealtimeSanitizer（RTSan）运行时链进 Rust 程序：`#[nonblocking]` 标记的函数里若调到 malloc/free、pthread 锁、sleep、文件/网络 IO 等「RT 不安全」libc 函数，就打栈报错 |
| 许可证 | 绑定本身 Apache-2.0【源码】resources/rtsan-standalone-rs/Cargo.toml（`license = "Apache-2.0"`）；运行时来自 LLVM compiler-rt（Apache-2.0 WITH LLVM-exception）【官方】https://clang.llvm.org/docs/RealtimeSanitizer.html；均与 GPL-3.0 **兼容**，且只在测试构建里链接、不随产品分发【推断】 |
| 维护状态 | 0.3.0 发布于 2025-09-27；仓最近推送 2026-09-15；open issues 1（#29 2026-04，维护者自开）；最近 5 个 issue/PR 有回复【API】。预编译库仓 rtsan-libs 最新 v20.1.1.3（2026-02-24）【API】，而 build.rs 写死的是 `v20.1.1.1`【源码】crates/rtsan-standalone-sys/build.rs:7。⚠️ crates.io 最后发版 0.3.0 已过 12 个月，仅仓库推送在 12 个月内——命门 3 **有条件满足**（且只做 dev 依赖，不进产品） |
| 实时安全做法 | 不是零件而是**检测器**。只有构建时设了 `RTSAN_ENABLE` 且目标在列表里（含 `aarch64-apple-darwin` → `libclang_rt.rtsan_osx_dynamic.dylib`）才启用，否则所有 API 编译成空函数【源码】resources/rtsan-standalone-rs/crates/rtsan-standalone-sys/build.rs:12-28、40-47；resources/rtsan-standalone-rs/src/lib.rs:32-33。启用时默认**构建期用 curl 从 GitHub 下载预编译 dylib**（默认特性 `prebuilt-libs`【源码】resources/rtsan-standalone-rs/Cargo.toml:43）【源码】build.rs:78-98，或设 `RTSAN_LIB_PATH` 用本地库【源码】build.rs:54；若关掉默认特性又不给本地库，build.rs 会 `git clone` llvm-project 并用 cmake 现场编译【源码】build.rs:102-125（很重） |
| 值得学的一处 | 「构建期环境变量开关 + 不支持就静默 no-op」（build.rs:40-44）——让检测器能常驻代码里而不影响正式构建 |
| 我们怎么用 | **直接依赖（dev-dependency / 可选特性）**：`engine`、`io-macos`、`hal-plugin` 的 RT 函数加 `#[nonblocking]`；在 `eval` 离线跑整段 WAV 时开 `RTSAN_ENABLE=1`，一次把 ORT 推理路径里的 malloc/锁全扫出来 |
| 风险 | ① README 写的环境变量是 `RTSAN_LIBRARY_PATH`，build.rs 实际读 `RTSAN_LIB_PATH`——**文档与代码不一致**【源码】README.md「Using Custom-Built RTSan Libraries」节 vs build.rs:54；② 构建期联网下载二进制（供应链 / 离线构建问题，建议固定本地库）；③ 只在启用时有开销，开销来自每次被拦截函数调用时检查线程状态【官方】clang 文档；④ HAL 插件运行在 coreaudiod 里，dylib 注入不现实，只能在单元/离线层面测【推断】 |
| 证据 | 【源码】×7 【API】×3 【官方】×2 【推断】×2 |

### rustc 原生 RTSan（`-Zsanitizer=realtime`）（https://github.com/rust-lang/rust/pull/147935）
| 项 | 内容 |
|---|---|
| 做什么 | rustc nightly 内建的 RealtimeSanitizer 支持，配合 `#[sanitize(realtime = …)]` 属性 |
| 许可证 | 工具链本身，不进依赖树 |
| 维护状态 | PR #147935「Add LLVM realtime sanitizer」**2025-11-08 合并**；之后有 ICE（#148750）、增量编译 SIGSEGV（#149250）两个 issue，均已关闭【API】 |
| 实时安全做法 | 同 RTSan（检测器）；**需要 nightly**；PR 作者提醒对 async 函数/闭包行为不符合预期【API PR 正文】 |
| 值得学的一处 | — |
| 我们怎么用 | **只看看**；主线用 stable + rtsan-standalone，等它稳定化再换（浅读） |
| 风险 | nightly only。复核：`aarch64-apple-darwin` 的 `supported_sanitizers` 已含 `SanitizerSet::REALTIME`【官方】https://github.com/rust-lang/rust/blob/master/compiler/rustc_target/src/spec/targets/aarch64_apple_darwin.rs （第 23-26 行） |
| 证据 | 【API】×3 【官方】×1 |

### no_denormals（https://github.com/Sin-tel/no_denormals）
| 项 | 内容 |
|---|---|
| 做什么 | RAII guard：闭包内关掉非正规数（x86 设 MXCSR FTZ+DAZ，aarch64 设 FPCR.FZ） |
| 许可证 | MIT，**兼容**；零依赖【API】 |
| 维护状态 | 0.3.0 发布于 2025-09-11；最近推送 2025-09-11——**刚过 12 个月线**；open issues 1；最近 issue 维护者都有回复【API】 |
| 实时安全做法 | aarch64：`mrs fpcr` 读 → `msr fpcr` 写回 `| (1<<24)`（有 fp16 时再加 bit 19 FZ16），guard drop 时写回原值【源码】resources/no_denormals/src/lib.rs:36-41、72-78、96-98。两条指令，无系统调用。API 标 `unsafe` 并在文档里坦白「按 Rust 文档这是技术上的 UB」【源码】lib.rs:8-9、103-112 |
| 值得学的一处 | 把 fp16 的 FZ16 一起置位（lib.rs:39-41）——若以后 ORT/CoreML 走 fp16 路径有用 |
| 我们怎么用 | **抄思路自写**（和 nih-plug 二选一，代码 < 20 行，不值得背依赖）；用在 `engine`、`io-macos`、`dsp` 的 RT 线程入口 |
| 风险 | Rust 层面「改浮点环境 = UB」的官方立场；且 FPCR 是**每线程**的：ORT 若开了内部线程池，那些线程不会继承，需设 `intra_op_num_threads=1` 让推理在我们自己的线程上跑【推断】 |
| 证据 | 【源码】×4 【API】×2 【推断】×1 |

### nih-plug（ScopedFtz · AtomicF32 · process_wrapper）（https://github.com/robbert-vdh/nih-plug）
| 项 | 内容 |
|---|---|
| 做什么 | Rust 音频插件框架；本卡只看它的实时安全小件 |
| 许可证 | ISC（框架本体），**兼容**【源码】resources/nih-plug/Cargo.toml:7；（其 VST3 绑定另有 GPL 要求，与本卡无关）【推断】 |
| 维护状态 | 最近推送 2026-09-03；open issues 125【API】 |
| 实时安全做法 | `process_wrapper` 每次进 process 先 `ScopedFtz::enable()`，debug + 特性开启时再包 `assert_no_alloc`【源码】resources/nih-plug/src/wrapper/util.rs:187-199。aarch64 分支：`AARCH64_FTZ_BIT = 1 << 24`（FPCR.FZ）【源码】util.rs:21-26；只有原来没开才置位、drop 时才复原【源码】util.rs:236-249。参数平滑器的 `current/step_size` 用 `AtomicF32`（来自 atomic_float）【源码】resources/nih-plug/src/params/smoothing.rs:7、54-63 |
| 值得学的一处 | **「FTZ + 分配检查」合在一个 process_wrapper 里**（util.rs:187-199）：我们的 `engine::process_block` 与送音回调都该有同款入口包装，并在里面加 rtsan 的 realtime_enter/exit |
| 我们怎么用 | **抄思路自写**：`dsp` 或 `engine` 里一个 `rt_scope(|| …)`：FTZ guard + (debug) assert_no_alloc + (test) rtsan |
| 风险 | 无（只抄思路） |
| 证据 | 【源码】×6 【API】×1 【推断】×1 |

### atomic_float（https://github.com/thomcc/atomic_float）
| 项 | 内容 |
|---|---|
| 做什么 | `AtomicF32/AtomicF64`：底层就是 `AtomicU32/U64` + `to_bits/from_bits` |
| 许可证 | Apache-2.0 OR MIT OR Unlicense，**兼容**【源码】resources/atomic_float/Cargo.toml:7 |
| 维护状态 | 1.1.0 发布于 2024-08-31；最近推送 2024-08-31——**不满足近 12 个月**；#15（2025-12）PR 有 2 条讨论未合【API】 |
| 实时安全做法（中读） | `load` = `f32::from_bits(bits.load(ord))`【源码】resources/atomic_float/src/atomic_f32.rs:172-173；`store`/`swap` 同理（:204、:232）；在 aarch64 上是单条原子指令，无锁【推断】 |
| 值得学的一处 | 编译期断言 `AtomicU32` 与 `UnsafeCell<f32>` 尺寸/对齐一致（atomic_f32.rs:78-79） |
| 我们怎么用 | **不要依赖，自写**十来行（`control` → `engine` 的单值参数，如衰减下限、旁路标志） |
| 风险 | 维护停滞（但代码极简，风险低） |
| 证据 | 【源码】×3 【API】×2 【推断】×1 |

### portable-atomic（https://github.com/taiki-e/portable-atomic）— 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 跨平台原子类型补全（128 位原子、`AtomicF32/F64` 需 `float` 特性） |
| 许可证 | Apache-2.0 OR MIT，**兼容**【API】 |
| 维护状态 | 1.15.0 发布于 2026-08-09；最近推送 2026-10-01；open issues 17【API】；特性列表含 `float`【API crates.io】 |
| 实时安全做法 | 未读源码；aarch64-apple-darwin 上 32/64 位原子本就原生，`float` 特性预计是同样的位转换【推断】 |
| 值得学的一处 | — |
| 我们怎么用 | **只看看**；若以后需要 128 位原子（例如 shm 里「序号 + 时间戳」一次写）再考虑 |
| 风险 | — |
| 证据 | 【API】×3 【推断】×1 |

### nice-assert-no-alloc（https://github.com/BillyDM/nice-assert-no-alloc）— 浅读
| 项 | 内容 |
|---|---|
| 做什么 | assert_no_alloc 的活分叉（改名重发），合入嵌套修复 |
| 许可证 | BSD-1-Clause，**兼容**【API crates.io】 |
| 维护状态 | 1.0.0 发布于 2026-05-17；最近推送 2026-05-17；open issues 0【API】 |
| 实时安全做法 | 同 assert_no_alloc（与原仓 diff 主要是格式化 + 嵌套修复）【源码】resources/nice-assert-no-alloc/src/lib.rs 与原仓对比 |
| 值得学的一处 | 同上 |
| 我们怎么用 | **直接依赖（dev/debug）**，替代原仓 |
| 风险 | 单人维护；同名 `nih_assert_no_alloc` 也由同一人发布【API】 |
| 证据 | 【源码】×1 【API】×3 |

---

## 给 rustarch 线：assert_no_alloc vs RTSan 两种做法的差异

| 维度 | assert_no_alloc（nice- 分叉） | RTSan（rtsan-standalone / `-Zsanitizer=realtime`） |
|---|---|---|
| 原理 | 替换 Rust `#[global_allocator]`，线程本地计数器判是否在禁区【源码】resources/rust-assert-no-alloc/src/lib.rs:35-37、151-188 | 运行时在 libc 层拦截函数（malloc/calloc/free、pthread_mutex_*、sleep、文件/socket IO 等），线程在 `#[nonblocking]` 区段内调用即报错【官方】https://clang.llvm.org/docs/RealtimeSanitizer.html；报错样例见 resources/rtsan-standalone-rs/README.md「Usage」节 |
| 检测：Rust 分配 | ✅ | ✅（最终都落到 libc malloc） |
| 检测：C/C++ 库内分配（**ONNX Runtime、CoreAudio**） | ❌ 看不见 | ✅ 只要经 libc malloc |
| 检测：锁 | ❌ | ✅ pthread 锁族（Rust `std::sync::Mutex` 在 macOS 上走 pthread 或 futex 类原语，是否全部被拦需实测【推断】） |
| 检测：系统调用 / 阻塞 IO | ❌ | ✅ 拦截的 libc 包装函数；直接 `svc` 的裸系统调用拦不到【推断】 |
| 自定义「这也算阻塞」 | 无 | `#[blocking]` / `notify_blocking_call()` 自己标【源码】resources/rtsan-standalone-rs/src/lib.rs:159-176 |
| 开销 | 开启时每次分配多两次 TLS 读；默认 `disable_release` 在 release 下整个是空操作【源码】lib.rs:43-50 | 未设 `RTSAN_ENABLE` 时全是空函数、零开销【源码】build.rs:40-44、src/lib.rs:32-33；开启时每次被拦函数调用多一次线程状态检查，另有进入/退出 RT 区段的成本【官方】clang 文档 |
| 是否需 nightly | 否（stable） | rtsan-standalone：否，MSRV 1.77【源码】resources/rtsan-standalone-rs/Cargo.toml `rust-version`；rustc 原生版：**是**（nightly，2025-11-08 合并）【API】 |
| 平台 | 任意 | Linux/macOS/iOS；aarch64-apple-darwin 在支持列表【源码】build.rs:22-25 |
| 构建依赖 | 无 | 构建期联网下载 dylib（或本地 LLVM 构建 / `RTSAN_LIB_PATH`）【源码】build.rs:54、78-98 |
| 发现问题时 | abort（或计数），可选打印 backtrace | 打完整栈 + SUMMARY；`RTSAN_OPTIONS=halt_on_error=false` 可只报不停【源码】README.md:101 |
| 建议用法 | 常驻 debug 构建、单元测试；成本最低的第一道网 | CI/手动的「离线整段 WAV 走一遍实时路径」专项；专门用来扫 ORT 推理与 CoreAudio 调用路径 |

**结论**：两者互补而不是二选一。assert_no_alloc 便宜常开；RTSan 才回答得了「ORT `Run()` 里有没有 malloc/锁」这个对我们最要命的问题。【推断】

---

## 没查到 / 存疑

1. **跨进程 shm 无锁环的现成 Rust crate**：本组没找到维护中、可直接用的（候选如 `shared_memory`、`ipc-channel`、`raw_sync` 未在本组范围内调研）。复核补：crates.io 上有几个 2026 年新建的小 crate——`shmring` 0.4.1（MIT，下载 269）、`shm-rings` 1.0.0（MIT/Apache，2673）、`ringfire` 0.5.2（MIT/Apache，88）【API】，都是单人新仓、下载极少、未读源码，不建议依赖；当前判断仍是自写（io 卡 `shared_memory` 已判停更）。
2. **ONNX Runtime 推理路径是否 RT 安全**：不属于本组，但决定 RTSan 的价值——ORT 默认每次 `Run` 可能分配中间张量（arena 有时能复用），需在 engine 线用 RTSan 实测（本组按规矩没跑）。
3. **ORT 的 `session.set_denormal_as_zero` 选项在 aarch64 是否生效**：未核实【推断：可能只对 x86 SSE 有效】，因此 FTZ 仍建议在我们自己线程入口设，并让 ORT 单线程。
4. **aarch64 FPCR.FZ 是否同时涵盖「输入非正规数当零」**（即 x86 DAZ 的等价）：ARMv8 文档对 FZ=1 的描述是输入与输出非正规数均按零处理（不同于 x86 FTZ/DAZ 两位），本组未亲读 ARM 文档原文，标【推断】；nih-plug 与 no_denormals 都只置 bit 24，侧面印证 aarch64 不需要第二个位。
5. **共享内存里放 `AtomicU64` 跨进程是否可靠**：aarch64 上 64 位原子是无锁且地址无关的，跨进程映射同一物理页应可用【推断】；需架构线确认 macOS `shm_open`/`mmap` 的对齐与 `MAP_SHARED` 语义。
6. ~~triple_buffer 是否带 "Incompatible With Secondary Licenses"~~：复核 grep 已确认无源文件附此声明（见该卡）。
7. **basedrop #10 竞态是否成立**：报告未获维护者回复，本组未做独立验证。
8. ~~rustc `-Zsanitizer=realtime` 的 macOS aarch64 支持~~：复核已确认支持（见该卡风险栏）。
9. **audio_thread_priority（mozilla，MPL-2.0，0.38.0 · 2026-09-17）**：已在 resources/，实时线程优先级/workgroup 属于 io-macos 组，本组未写卡。
