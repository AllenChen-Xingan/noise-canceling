# 009 · 取舍卡：每个模块用哪个依赖 · 学谁的设计

> 2026-10-30 rusteco（AI 结论，待人过目）。依据：`cards/` 下 8 份项目卡（7 组检索 + 完整性补漏，约 120 个项目；怀疑派 3 人回源码复核约 450 条，改正约 40 处）、`answers-p1.md`、p1 会话的本机实测；交付前又过了三视角自审（事实出处 · 实时架构 · 许可与可读性），改了 2 条 P0、10 条 P1。
> 路径 `resources/…` 相对主目录 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。证据等级：【源码】亲读行号 ·【官方】文档 ·【API】GitHub / crates.io 当日返回 ·【实测】本机跑过 ·【推断】。
> 命门（`must-know.md`）：① 与 GPL-3.0 兼容 ② 实时线程零分配、零锁、零系统调用 ③ macOS 可构建，近 12 个月有维护。D 编号见 `docs/plan/001_decisions-20261029-b1.md`；P1–P4 是原型阶段（`docs/plan/003_tech-design.md` §四），M0–M3 是路线图里程碑；「p1」小写指正在写第一个 crate 的那个会话。术语表在文末。

## 一句话

**Rust 生态里没有能整个拿来用的降噪框架，但每个零件都有现成的 crate 或成熟写法。** 进程内的零件直接依赖小 crate：ort（推理）· realfft（FFT）· rubato（重采样）· rtrb / triple_buffer（线程间传数据）· biquad（滤波）· earshot（VAD，权重来源待核）· hound（离线读写 WAV）。**跨进程的部分（共享内存环、HAL 驱动、跨进程唤醒、工作区间）没有能用的 crate，要自己写**，读懂 voice-strip、Firewheel、rtrb 的做法后自己写（铁律 3：不复制代码）。

进程和线程的关系（003 §一）：**送音进程**里有麦克风回调（硬实时，只碰环形缓冲）；**推理进程**里有**降噪线程**（实时，跑 GTCRN）和**声纹线程**（非实时，跑在能效核上）。下文的「推理线程」都指降噪线程。

## 一、每个模块的推荐依赖

「怎么用」：**依赖** = 写进 Cargo.toml ·**学思路** = 读懂后自己写，不复制代码 ·**dev** = 只在测试 / 评测里用。「发布阻断」= 发布前必须先核清许可。

### 1. 音频 I/O 与线程（io-macos · hal-plugin）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| 麦克风采集 | M0 直通沿用路线图的 cpal 0.18.2；**M1 起换成自写 HAL IOProc**（`AudioDeviceCreateIOProcID`），绑定用 **coreaudio-sys**（bindgen 生成，含 HAL 驱动接口）+ **mach2**（Mach 时间、端口），不整套手写 FFI | cpal 继续用于设备枚举、eval | cpal 底层 coreaudio-rs 在帧数变化时在实时线程读属性并 `Vec::resize`（`resources/coreaudio-rs/src/audio_unit/render_callback.rs:622,649`）；cpal 每次回调调 `mach_timebase_info`（`resources/cpal/src/host/coreaudio.rs:76-78`，读的是 main 分支）。IOProc 直接给设备的 sample/host time。coreaudio-sys 含 `AudioServerPlugInDriverInterface`（`resources/prism/src/driver.rs:3,2052`，io 卡）；cubeb-coreaudio-rs（Firefox 生产代码）就依赖 mach2（`resources/cubeb-coreaudio-rs/Cargo.toml:17`） |
| 虚拟麦克风 HAL 驱动 | **学思路**：照 voice-strip `vdev-macos-driver` 的结构自写 Rust cdylib | tympan-aspl 0.1.0（0 star、只发过一版） | voice-strip 证明纯 Rust 驱动可行，IO 路径只有原子加减 + 一次读环（`resources/voice-strip/crates/vdev-macos-driver/src/plugin.rs:771-810`）。objc2-core-audio 最后发版 2025-10-04，已过 12 个月线 |
| 跨进程音频共享内存 | **学思路**：`shm_open + mmap` + 固定 `#[repr(C)]` 全原子布局，用 voice-strip 的「时间轴 + 分槽序号」环 | memmap2 0.9.12 只管映射 | 见「设计 1」及其 D03 补丁。iceoryx2 太重（macOS 只是 tier 2），shared_memory 2022 后不发版【API】 |
| 共享内存安全交接 | **学思路**：LaunchDaemon 登记 Mach 服务，用 `xpc_shmem_create` 把匿名共享区交给驱动；驱动 Info.plist 要声明 `AudioServerPlugIn_MachServices` 才能在 coreaudiod 沙箱里连服务 | — | GPL-3.0 先例 `resources/m-audio-fast-track-ultra-macos-driver`（gaps.md G1）；voice-strip 用 0644 文件，自注只够 spike（`shm.rs:7-10`）。见待人拍板 3 |
| 跨进程唤醒推理线程 | **学思路**：`libc` 的 `os_sync_wait_on_address` / `wake`（SHARED 标志，macOS 14.4+） | 推理线程按周期自己醒来看（无需唤醒） | gaps.md G2。**唤醒是麦克风回调里的一次非阻塞系统调用**，列为命门 ② 的唯一例外（voice-strip 作者也这么取舍，`resources/voice-strip/crates/vad/src/lib.rs:255-256`）；和周期轮询做小实验比延迟与耗电 |
| 实时优先级 + 工作区间 | **学思路**：送音回调线程和推理线程都设 Mach time-constraint（computation 按实测推理耗时定），并 `AudioWorkIntervalCreate` / `os_workgroup_join`，每帧 `interval_start` 带截止期 | audio_thread_priority 0.38（MPL-2.0；不支持 workgroup） | 只有实时优先级不会提核心频率（voice-strip `crates/denoise-neural/src/rt.rs:243-275`）；推理线程没有 time-constraint 会被抢占，旁路就会来回抖。**跨进程加入麦克风 IO 线程的 workgroup，系统已有 API**：`os_workgroup_copy_port` / `os_workgroup_create_with_port`，头文件注释写明可在另一个进程用【官方】SDK `os/workgroup_object.h:79-118`；要实验的是周期不一致（IO 约 10.7 ms vs hop 16 ms）时有没有用 |
| 声纹线程放能效核 | `libc::pthread_set_qos_class_self_np`（UTILITY） | thread-priority 3.1.1 | libc 有绑定（复核核过）；「低 QoS 就上 E 核」是推断，P3 用 powermetrics 量 |
| 设备切换 / 热插拔 / 睡眠唤醒 / 采样率变更 | **学思路**：cubeb-coreaudio-rs 的设备重建流程（只浅读，未逐行核）；daemon 侧参考 fast-track 驱动的 `daemon/src/supervisor.rs`（gaps.md G6，未读） | — | M0 验收已含「蓝牙耳机、热插拔不用重启」（002 路线图）；AirPods 切到通话模式会变 16 kHz，coreaudiod 重启后驱动要重新映射 |
| 控制面 IPC（control） | std `UnixStream` | ipc-channel 0.23.1 | 不上实时线程；ipc-channel 序列化会分配，UB 类 issue #457 无人回【API】 |

### 2. 实时安全零件
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| 进程内单进单出环 | **依赖** rtrb 0.4.1 | ringbuf 0.6.0 | push/pop 只在疑似满/空时读对端索引，零分配零锁（`resources/rtrb/src/lib.rs:490-503,778-791`）；ringbuf API 常有破坏性变更，10-30 刚修完 soundness #65/#66【API】 |
| 最新值发布（声纹→门控、状态） | **依赖** triple_buffer 9.0.0（进程内） | arc-swap（只在非实时端） | 两端各一次 `swap(AcqRel)`，不等待（`resources/triple-buffer/src/lib.rs:320-348,516-547`）；MPL-2.0，rtsafe 卡 grep 证实无 Exhibit B。跨进程版按同样原理自写 |
| 单值参数 | 自写 `AtomicU32` + `f32::to_bits` | portable-atomic float | 十来行；atomic_float 2024-08 后不发版 |
| 实时线程不释放内存 | rtrb 回收队列：旧对象送回非实时线程 drop；**回收队列满了就先留在本地槽位、下一块再送** | basedrop（2025-10-29 后不发版；#10 竞态报告无人回） | voice-strip、Firewheel 都这么做；「满了怎么办」是 Firewheel 的漏洞（见设计 2） |
| debug/test 里断言不分配 | **dev** nice-assert-no-alloc 1.0.0，或照 assert_no_alloc 的原理自写约 60 行计数分配器 | assert_no_alloc 1.1.2（BSD-1-Clause，2021 后停更） | 只抓经 Rust 全局分配器的分配。**p1 已用 stats_alloc + assert_no_alloc 实测 GTCRN 每帧 0 次**【实测】 |
| 抓锁、系统调用、C++ 内部分配 | **dev，可选 feature** rtsan-standalone 0.3.0（`#[nonblocking]`，CI 里一个 `RTSAN_ENABLE=1` 的 job） | rustc nightly `-Zsanitizer=realtime`（已支持 aarch64-apple-darwin） | 在 libc 层拦截，看得到 ORT / CoreAudio 内部；最后提交 2025-09-27，命门 ③ 有条件满足。只有设了 `RTSAN_ENABLE` 才会联网下载预编译库（`sys/build.rs:40-44`），平时构建不联网；CI 里给 `RTSAN_LIB_PATH` 指本地库 |
| 去非正规数 | **学思路**：照 nih-plug `ScopedFtz` 自写作用域守卫（aarch64 FPCR 置 FZ 位），**只包住 DSP 段（STFT、门控、音色链），不包 ORT 调用** | no_denormals 0.3.0（MIT，2025-09，已过线） | `resources/nih-plug/src/wrapper/util.rs:25,191`。`with_intra_threads(1)` 时 ORT 就在推理线程上算，FZ 开在整个线程上等于对 ORT 开 FTZ，正撞 DeepFilterNet #713（FTZ + 单位范数 0/0 出 NaN） |
| NaN / Inf 防线 | 自写：每帧检查模型输出与状态是否有限；不是就把状态复位到预热参考状态（设计 3）、本帧走原声、计数上报 | — | GRU 状态是循环的，一次 NaN 永久污染；三视角自审补的 |
| MPMC 队列 / seqlock crate | **不要** | — | crossbeam ArrayQueue 争用时 `yield_now`（`crossbeam-utils/src/backoff.rs:207-219`）；seqlock crate 写端用 Mutex |

### 3. DSP（dsp）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| FFT（512 点实 FFT，STFT 与 fbank 共用） | **依赖** realfft 3.5.1（底层 RustFFT 6.4.1） | 手写几行 FFI 调系统 vDSP_DFT | `process_with_scratch` 零分配（`resources/realfft/src/lib.rs:138-143`）；RustFFT 在 aarch64 运行时检测 NEON（`resources/RustFFT/src/neon/neon_planner.rs:156-157`）。RustFFT 最后发版 2025-09-18，见待人拍板 1 |
| STFT / iSTFT（sqrt-Hann 512/256，对齐 torch.stft） | **学思路**：在 dsp 自写，骨架参考 libDF | — | libDF 结构清楚但每帧分配两次（`resources/DeepFilterNet/libDF/src/lib.rs:360,397`），我们改成预分配；torch.stft 默认 center=True 反射填充，对拍时首尾帧单独处理 |
| 重采样 48k↔16k | **依赖** rubato 5.0.1；**目标每一跳 ≤ 2 ms**：Fft 型用 `new_custom` 调小子块，或改低延迟多相 sinc，P3 实测选 | rubato Async（sinc） | `process_into_buffer` 预分配；Fft 默认延迟上采约 8 ms、下采约 2.7 ms（复核手算），**按默认参数整条链会超 45 ms**，见下方延迟账 |
| 两个设备的时钟漂移 | rubato `Slip`（插/丢一帧 + 交叉淡化）+ **自写漂移估计**（用 IOProc 时间戳的 `mRateScalar` 或 sample/host time 做一个小的锁相环估比率） | rubato Async + 相对比率 | `resources/rubato/src/slip.rs:9-30` 只管执行，不估漂移 |
| 限幅 / 轻压缩 | **学思路**：照 sonora AGC2 Limiter 的原理自写 | — | 子帧增益插值、零前瞻（`resources/sonora/crates/sonora-agc2/src/limiter.rs:12-40`）；sonora 整条 AGC2 每帧分配（`limiter.rs:98`、`resources/sonora/crates/sonora/src/gain_controller2.rs:238`），**不能直接进实时线程** |
| 双二阶滤波 | **依赖** biquad 0.6 | 自写 DF2T | `run()` 无分配无锁（`resources/biquad-rs/src/lib.rs:170-181`）；依赖 num-traits(libm) |
| 离线读写 WAV（eval） | **依赖（只 eval）** hound 3.5.1 | symphonia 0.6.1（MPL-2.0） | 3.5.1 发于 2023-09，但默认分支 2026-08 仍有提交【API】；只用在离线路径 |
| 传统降噪 / VAD 对照基线 | **dev** sonora（纯 Rust WebRTC NS + AGC2） | nnnoiseless（BSD-3） | 只做对照；tonarino webrtc 绑定在 macOS 27 有构建 panic（issue #103 只有报告人回）且每帧分配 |

**延迟账（D32 端到端附加 ≤ 45 ms）**——数字来自卡片手算，P3 实测前都是【推断】：

| 项 | 默认参数 | 目标上限 |
|---|---|---|
| STFT 512/256 @16k（算法延迟） | 约 32 ms | 32 ms（模型定了，动不了） |
| 重采样 48→16k + 16→48k | 约 10.7 ms（rubato Fft 默认） | ≤ 4 ms（两跳各 ≤ 2） |
| 麦克风 IO 缓冲 | 512 帧 @48k ≈ 10.7 ms | ≤ 5.3 ms（256 帧） |
| 跨进程环余量（盖住最坏推理耗时 + 唤醒抖动） | 未定 | ≤ 3 ms |
| HAL 传输余量（voice-strip 用 128 帧） | 约 2.7 ms | ≤ 2.7 ms |
| **合计** | **≈ 56 ms + 环余量，超标** | **≤ 45 ms** |

所以重采样和 IO 缓冲**必须调参**。另一个省法（交 rustarch 008 评估）：推理进程直接写 HAL 读的那个环，驱动缺数据时改读原声环——少过一跳环，D03 的语义也更简单。

### 4. 推理（engine）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| GTCRN 流式 ONNX | **依赖** ort 2.0.0-rc.15：CPU EP、`with_intra_threads(1)`、关 spinning、`with_auto_device(PreferCPU)` | tract 0.23 | biglinux-gtcrn 已用 ort 逐帧跑通；`run_with_options` + `OutputSelector::default().preallocate(全部输出)` 后 Rust 侧零分配（`~/.cargo/registry/src/*/ort-2.0.0-rc.15/src/session/run_options.rs:138`、`src/value/mod.rs:393-397`），**p1 实测 GTCRN 每帧 0 次**【实测】。tract 每个卷积节点每次 eval 都新建输出张量（`resources/tract/core/src/ops/cnn/conv/blocked.rs:118`，infer 卡），分配更多 |
| p1 实测踩到的坑 | `OutputSelector::no_default()` + preallocate 会让 ORT 报 "At least one output should be requested"，要用 `default()`；rc.15 在 macOS 默认挂 CoreML，GTCRN 编译失败，必须 `PreferCPU`【实测，p1】 | — | — |
| FastEnhancer-M 同接口备选 | ort，用官方 16 kHz spec2spec 导出，**首选 `onnx-dns-v1.0.0` 的 `fastenhancer_m.spec.onnx`**（DNS-Challenge 3 训练；`onnx-vd-v1.0.0` 只用 VCTK-Demand 28 人训练）【官方，两组 release 说明都写明 16 kHz】 | tract | spec2spec 才能和 GTCRN 共用 dsp 的 STFT。FE-M 7 进 7 出，超出 ort 放在栈上的上限（输入 ≤ 6、输出 ≤ 4，`ort/src/util/stack.rs:13-15`），**每帧 7 次小分配**【实测，p1】；可以接受（推理线程按软实时管），或把状态张量合并后重新导出 |
| 声纹 ECAPA / CAM++ | ort，共用 Environment、单独 Session | tract | 不在实时路径；少带一份运行时 |
| CoreML / ANE | **不作默认，也不先做实验**：CPU 单线程到底，P3 实测耗电超出 D31 再做有界实验（ProfileComputePlan + powermetrics） | CPU EP | CoreML EP 算子表没有 GRU、ScatterND，GTCRN 两样都用，图会被切碎【官方】。这是技术判断，AI 按 D31 实测结果定，不占人的拍板 |
| 发布阻断 | ORT 预编译库（从 cdn.pyke.io 下载）的许可清单 ThirdPartyNotices 未逐项核；GTCRN、FE-M、ECAPA / CAM++ 权重许可（005 #2、D33 注意项） | — | 构建说明里记 ORT 来源 URL 与 sha256 |

### 5. 声纹与 VAD（speaker）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| 80 维 fbank | **学思路**：自写 `dsp::fbank`（约 200 行，realfft + 调用方给工作区 + 窗型可配），以 torchaudio `kaldi.fbank` 为对拍基准 | **dev** kaldi-fbank（纯 Rust、Apache-2.0，单作者、建仓 2 个月）做第二个对照 | **两个候选模型前端不一样**：WeSpeaker 用 hamming 窗、输入 ×32768；3D-Speaker CAM++ 用 povey 窗、输入保持 [-1,1]（speaker 卡存疑 1）。knf-rs 把窗写死成 povey，还要 cmake 编 C++ |
| 实时 VAD（门控「快开」） | **依赖** earshot（**发布阻断：权重来源未说明**） | 自写能量门限 + 滞回 | 纯 Rust、状态全是定长数组、零分配（`resources/earshot/src/lib.rs:238-293`）；16 ms / 256 点帧和 GTCRN hop=256 对齐；**输入要先夹到 ±1.0**，否则 debug 构建 panic（`frame.rs:25`、`lib.rs:249`） |
| 非实时 VAD（挑清楚的单人语音） | silero（findit-studio）的结构 + 自家 ort 跑 silero_vad.onnx | earshot | Session 和 StreamState 分离（`resources/silero-findit/src/session.rs:160-224`） |
| 在线注册 / 认人 | **学思路**：sherpa-onnx manager 的「先求和再 L2 归一化、矩阵×向量算余弦」+ pyannote-rs 的「过阈值归入、否则新建」，改成固定容量 | speakrs（kodama AHC）做锁定前一次性聚类 | `resources/sherpa-onnx/sherpa-onnx/csrc/speaker-embedding-manager.cc:63-81,117-135`；Rust 生态没有零分配的在线注册 crate，核心不到 150 行 |
| 不要 | sherpa-onnx 官方 Rust crate 整包用；**TEN VAD**（「不得与 Agora 竞争」条款与 GPL-3.0 不兼容） | — | sherpa-onnx 自带并静态链接一份 ORT，build.rs 从 GitHub 下载，可能和 ort crate 冲突 |

### 6. 宿主架构（engine · control）
| 需求 | 首选 | 理由 · 出处 |
|---|---|---|
| 音频图 | **不要图**，自写固定管线 | 链路固定；dasp_graph 每次处理都重做 DFS 且 2020 后不发版（`resources/dasp/dasp_graph/src/lib.rs:322`）。nih-plug、Firewheel、clack、kira **都不当依赖，只学设计** |
| engine 生命周期 | 照 clack 用类型分开：`activate()` 造处理器（可分配）、`process()` 实时、`deactivate(self)` 吃掉处理器 | `resources/clack/plugin/src/plugin.rs:198,225,249`；比 nih-plug 靠注释约束更硬。nih-plug 的 `reset()`（`plugin.rs:216`）对应「清 GRU 缓存」 |
| 参数 | 每个参数一个原子量 + 稳定字符串 ID（nih-plug 的 `Params` derive 生成「ID → 参数指针」表，`resources/nih-plug/nih_plug_derive/src/params.rs:415-418`）；平滑照 nih-plug `Smoother`（`src/params/smoothing.rs:47,260,283`） | 用稳定 ID 而不是下标，加参数不错位 |
| 延迟上报 | 照 clack 的规矩：延迟只在激活时定，**换模型 = 重新激活** | `resources/clack/extensions/src/latency.rs:99`；HAL 驱动读 `kAudioDevicePropertyLatency` |
| 模型交叉淡化 | 照 Firewheel `Declicker` + 预计算淡化表 | `resources/Firewheel/crates/firewheel-core/src/dsp/declick.rs:17,113,486`；默认 3 ms 太短，调到 20–50 ms |
| 日志 / 遥测 | 自写两层：共享内存原子计数器 + 定长事件经 rtrb 送日志线程 | tracing-appender non_blocking 每行 `to_vec`，不能进实时线程（gaps.md G3） |
| 菜单栏 / 进程托管 | tray-icon + muda；launchd KeepAlive 拉起；SMAppService 用 objc2-service-management | gaps.md G4；Slint 有 GPL-3.0 分支可做设置窗口（只有 GPL-3.0-only，见待人拍板 2） |
| 别照着学 | nih-plug CLAP 包装的 process 里拿 `parking_lot::Mutex`（`src/wrapper/clap/wrapper.rs:2239`）；`schedule_gui` 在音频线程调 `CFRunLoopWakeUp`（`macos.rs:99-103`）；Firewheel 回程队列满时 `let _ = try_push(..)` 在音频线程 drop（`handle_messages.rs:42-44`） | 复核找到的反例 |

### 7. 工具链（CI · 发布，都是工具，不进 Cargo 依赖）
| 需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| 依赖树许可白名单 | cargo-deny 0.20.2（`[licenses]` 默认全拒 + 白名单）。**白名单正本以 rustarch 008 为准**；本线草稿见 `answers-p1.md` §5 | cargo-about | 对 OR 表达式逐叶求值（`resources/cargo-deny/src/licenses.rs:74-97`）；GPL-3.0-only 与 -or-later 要分开写；官方 Action 是 Docker 动作，放 ubuntu runner。**cargo-deny 查不出 MPL 的 Exhibit B**：加一条 CI 规则，凡新引入 MPL crate 就 grep「Incompatible With Secondary Licenses」 |
| 第三方许可清单（GPL 发布） | cargo-about | cargo-license（15 个月无提交） | ORT 原生库与模型权重不在 Cargo 图里，要手补 |
| 测试 | cargo-nextest · proptest · insta（只存摘要指标）· 自写对拍 Report（照 approx 的混合判据，报出哪帧哪维挂） | cargo test · approx | — |
| 并发正确性 | loom（`sync` 门面模块切换原子类型）+ Miri | — | `resources/loom/src/lib.rs:120-131`；两者都测不了跨进程 mmap，算法与映射要拆开测 |
| 基准 | criterion（本机） | divan（近 18 个月未发版） | 指令数回归 gungraun 只能放 Linux runner（Valgrind 不支持 arm64 macOS） |
| 打包签名公证 | 自写脚本：codesign → pkgbuild/productbuild → productsign → notarytool → stapler（开发者账号按 D06 M3 前办） | cargo-packager（不出 .pkg） | cargo-dist 不签 bundle、不公证，HAL 驱动 .pkg 用不了 |
| 其他 | cargo-auditable · cargo-machete | cargo-udeps（要 nightly） | cargo-semver-checks 不要（不对外发库） |

（「哪条规则用哪个工具、CI 怎么分层」归 rustarch 008，这里只给选型。）

## 二、最值得学的 5 个设计

**1. voice-strip 的「时间轴 + 分槽序号」共享内存环，加上我们的 D03 补丁**
写端改一个槽前把序号置成奇数，写完置成偶数；读端前后两次看到同一个偶数才认，否则这段当「缺」，**从不等待写端**。位置按绝对时间轴（host clock）算，所以送音进程写、HAL 读不需要握手。
出处：`resources/voice-strip/crates/vdev-macos-driver/src/shm.rs:143-173`（write）、`:180-`（read，本人复看过）；驱动 IO 路径 `plugin.rs:771-810`；时钟 `clock.rs:1-9`。
**注意，它判缺后是填零，不是送原声**（`shm.rs:178,211-215`；映射不在时整块填零 `plugin.rs:803`），出来的是静音。D03 要的是原声，而且原声要和降噪路径对齐到同一延迟，否则逐槽在「降噪声 / 静音 / 原声」之间跳，会有咔哒声。我们的补丁：
- 送音进程留一条**原声延迟线**，按同一时间轴取样；
- **连续缺 N 帧才切旁路**，恢复前先预热；
- 切换两头都用交叉淡化（设计 2 的 Declicker）；
- 卡顿、写一半被打断（撕裂）各写一条测试。

跨进程统一用这种环，rtrb 只在进程内用（head/tail 环在一端崩溃后要重置索引、另加代际号）。
voice-strip 10-17 才建仓，单人维护，README 自述大量由 AI 写：只学思路，不当依赖。

**2. Firewheel 的换装协议：造新 → 送进 → 换下 → 送回 → 主线程释放（用于 GTCRN ↔ FastEnhancer-M 热切换）**
新对象在非实时线程造好，经无锁队列送进实时线程；实时线程只做 `mem::swap`，换下的旧对象原路送回主线程释放。
出处：`resources/Firewheel/crates/firewheel-graph/src/processor/handle_messages.rs:69-116`、`processor.rs:242-256`。节点分成两个 trait：主线程用的 `AudioNode`，实时线程用的 `AudioNodeProcessor`（`firewheel-core/src/node.rs:266,486`）。
对我们：新模型的 Session 和状态在后台建好，先跟跑 N 帧预热，再把指针经 rtrb 送进去；交叉淡化结束后，旧模型原路送回释放。
**它有个漏洞要补**：回程队列满时，它的 `let _ = try_push(..)` 会在音频线程上直接 drop（`:42-44`、`:114-116`）。我们改成「满了先放进本地槽位，下一块再送」。

**3. 显式状态进出 + 预热（biglinux-gtcrn），加上我们自己的双缓冲预分配（p1 已实现）**
- **biglinux 提供**：GTCRN 的缓存状态每帧显式送进、取出（`resources/biglinux-gtcrn/ladspa/src/model.rs:282-302`）；开机先预热 20 帧，把 GRU 状态从全零推开，记下预热后的参考状态，复位时用（`:30-33,130-149,226-239`）。注意它每帧都新建 `TensorRef`、让 ORT 新分配输出，**不是零分配**。
- **我们加的**：两套状态张量奇偶帧轮换（A 的输出就是 B 的输入），全部输出用 `OutputSelector::preallocate` 预先分配好（`ort/src/session/run_options.rs:95-140`）。p1 按这个做，实测 GTCRN 每帧 0 次分配【实测】。

预热和参考状态还有两个用处：推理进程崩溃重启、换模型时，避免冷启动压得过狠；NaN 防线的复位点（§一.2）。

**4. rubato 的双接口：分配版只是包一层预分配版，离线评测和实时回调用同一份代码**
`process()` 会分配，给离线用；它内部只是先建好缓冲，再调 `process_into_buffer()`（预分配，给实时用）。核心算法只有一份。
出处：`resources/rubato/src/lib.rs:213-230`、`:263-268`。
同类做法还有两处：
- libDF 的 STFT 状态机，离线和实时共用一个 process（`resources/DeepFilterNet/libDF/src/lib.rs:119-122,356-393`），但它漏了预分配；
- speakrs 的 fbank：共享的不可变 Recipe + 每个句柄私有的工作区 + `compute_into`（`resources/speakrs/src/inference/cpu/fbank.rs:15-34,58-75`）。去直流和预加重用 f64 算，防止安静频带出现数值误差。

对我们：dsp 和 engine 的每个处理器都提供这样一对接口，就满足了 intent 的「离线评测与实时用同一份处理代码」，单元测试也能直接测到实时路径。

**5. 守卫层：nih-plug `process_wrapper` + rtsan `#[nonblocking]`，平时零成本，测试时抓违规**
- nih-plug：每次进处理函数先开 FTZ，debug 构建再套一层「禁止分配」，全局分配器换成守卫版（`resources/nih-plug/src/wrapper/util.rs:25,35-37,189-199`）。
- rtsan：宏只插一个 RAII 守卫，build.rs 看有没有设 `RTSAN_ENABLE` 决定编不编进去（`resources/rtsan-standalone-rs/build.rs:27-34`；支持哪些平台在 `:10-17`；`src/lib.rs:32-37`；`crates/rtsan-standalone-macros/src/lib.rs:33`），发布版零开销。

对我们：送音回调、降噪线程、离线 eval 的入口统一套这一层（FTZ 只包 DSP 段，见 §一.2）。CI 用 `RTSAN_ENABLE=1` 跑离线 WAV，每次提交都验命门 ②，顺带量出 ORT 的 C++ 内部到底有没有 malloc、有没有锁。

候补设计：
- rtrb 的「缓存对端索引 + CachePadded」（`resources/rtrb/src/lib.rs:104-109,490-503`）：自写跨进程环时，按同样原理自写。
- triple_buffer 的「一个原子字节 = 槽索引 + 脏位」（`resources/triple-buffer/src/lib.rs:320-348`）：正是声纹线程原子发布结果要的形状。
- cubeb-coreaudio-rs 的设备重建流程：未逐行读。

## 三、最值得学的一个项目

**voice-strip**（Rust，MIT OR Apache-2.0）几乎就是我们架构的单进程版：Rust 自写的 macOS HAL 虚拟麦克风驱动，加共享内存、神经网络降噪和 Audio Work Interval。

可以学的：
- 共享内存环协议（设计 1，要补 D03）；
- IOProc 采集只做「拷贝 → 转换 → 写环」；
- 推理不进音频线程；
- 降噪线程加入工作区间（`crates/denoise-neural/src/rt.rs:243-275`）。

风险：才两周、单人、AI 写的，**只学思路**。

## 四、还没答上的（转实验或转别的线）

| 问题 | 去向 |
|---|---|
| ORT C++ 内部稳态是否 malloc / 加锁 | P1 / P3 跑一次 RTSan（要名额） |
| 跨进程唤醒：`os_sync_wait_on_address` 和周期轮询比延迟与耗电 | 有界小实验，P3 前；结果决定待人拍板 1 |
| 推理线程跨进程加入 IO workgroup，周期不一致时有没有用 | P3 实验（API 已有，见 §一.1） |
| 延迟账逐项实测（重采样型号、IO 缓冲、环余量） | P3 |
| 推理进程直接写 HAL 环、少一跳 | rustarch 008 |
| tract 能否加载 GTCRN、数值能否对上 | 只在 ort 出问题时再做 |
| aarch64 FPCR.FZ 是否也把输入的非正规数当零 | 读 ARM 手册，不阻塞 |
| 发布阻断三项：earshot 权重、FastEnhancer / 声纹权重、ORT ThirdPartyNotices | 发布前许可核查（并入 D33 注意项） |
| panic 策略（abort 还是 catch_unwind） | rustarch 008 |

## 五、待人拍板

（自审后调整：「CoreML 实验」改由 AI 按 D31 实测结果定，不占人的拍板；开发者账号已由 D06 定，蓝牙耳机已写进 M0 验收，都不再问。）

| # | 题面 | 选项 | 代价 | 建议 | 默认（不回就按它） |
|---|---|---|---|---|---|
| 1 | **要不要把最低系统版本从 D07 的 macOS 14.2 提到 14.4？** 14.4 起才有 `os_sync_wait_on_address`，能让送音进程跨进程「一下叫醒」推理进程 | A 提到 14.4 · B 留在 14.2，推理线程按周期自己醒来看 | A：放弃 14.2–14.3 这一小段用户；B：多一点延迟或耗电（多少要实验） | **先跑小实验再问**：两种方式差不到 1 ms、耗电差不明显就选 B | 先实验；实验前按 B 写，不阻塞 |
| 2 | 本项目许可写 GPL-3.0-only 还是 GPL-3.0-or-later | A or-later · B only | 白名单、LICENSE 头都据此写。**注意**：只要有依赖是 GPL-3.0-only（例如 Slint 的 GPL 分支），整个发行包实际上只能按 GPL-3.0-only 发；or-later 只对我们自己的代码有意义 | **A**（FSF 推荐，将来能跟到 GPL-4；有 -only 依赖时整体按 -only 发） | A |
| 3 | HAL 驱动怎样拿到共享内存 | A：系统域 LaunchDaemon + Mach 服务 + `xpc_shmem_create`，只放行 coreaudiod 和**签名为本项目的进程** · B：普通文件共享内存（voice-strip 现在的做法） | A：安装要管理员权限、多一个常驻进程，驱动 Info.plist 要声明 Mach 服务；B：同一用户下任何程序都能读到麦克风音频，**不能发布** | M0–M1 用 B，发版前必须换 A | M0–M1 用 B，发版前换 A |
| 4 | 命门 ③「近 12 个月有维护」：**RustFFT** 最后发版 2025-09-18（仓库仍有提交），要不要破例当依赖？（hound 发版停在 2023-09，但 2026-08 仍有提交，按「仓库有维护」算过线） | A 破例：算法库已定型，仓库有人维护、无未修 soundness 问题即可 · B 不破例：改用系统 vDSP 自写 FFI | A：将来可能要 fork 自修；B：多写一层 FFI 和对拍，FFT 也是 fbank 对拍的基础 | **A** | A |

## 术语表

| 词 | 意思 |
|---|---|
| HAL / HAL 驱动 | macOS 音频系统（coreaudiod）加载的插件；我们用它做「虚拟麦克风」，会议软件选它当输入 |
| IOProc | 向 macOS 音频设备注册的回调函数，系统每几毫秒调一次，交给你一块麦克风采样 |
| cdylib | 编译成动态库（`.dylib`），HAL 驱动必须是这种形态 |
| 单进单出环（SPSC） | 一个线程写、一个线程读的环形缓冲，可以不加锁 |
| 不等待（wait-free） | 读写都不会因为对方在忙而停下来等 |
| 非正规数 / FTZ | 极小的浮点数会让 CPU 变慢几十倍；FTZ 是 CPU 的一个开关，把它们直接当 0 |
| RAII 守卫 | 进作用域时打开、出作用域自动恢复的小对象 |
| workgroup / 工作区间 | macOS 的接口：告诉系统「这几个线程一起赶同一个截止时间」，系统据此提核心频率 |
| time-constraint | Mach 内核的实时调度策略，给线程保证的 CPU 时间片 |
| EP / CoreML / ANE | ORT 的「执行后端」；CoreML 后端可以用苹果神经引擎（ANE） |
| arena | ORT 内部预先申请的一大块内存，之后在里面切小块用，少向系统要内存 |
| soundness 问题 | 安全代码也可能触发未定义行为的库缺陷 |
| Exhibit B | MPL-2.0 附件；作者加了它，该文件就不能和 GPL 组合 |
| tier 2 | 某平台「能编译但不保证测试过」的支持等级 |
| 发布阻断 | 发布前必须核清的许可问题，不影响原型开发 |
