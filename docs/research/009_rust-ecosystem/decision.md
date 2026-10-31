# 009 · 取舍卡：每个模块用哪个依赖 · 学谁的设计

> 2026-10-30 rusteco（AI 结论，待人过目）。依据：`cards/` 下 8 份项目卡（7 组检索 + 完整性补漏，约 110 个项目；怀疑派 3 人回源码复核约 450 条，改正约 40 处）、`answers-p1.md`、p1 本机实测。
> 路径 `resources/…` 相对主目录 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。证据等级：【源码】亲读行号 ·【官方】文档 ·【API】GitHub / crates.io 当日返回 ·【实测】本机跑过 ·【推断】。
> 命门（`must-know.md`）：① 与 GPL-3.0 兼容 ② 实时线程零分配零锁零系统调用 ③ macOS 可构建、近 12 个月有维护。

## 一句话

**Rust 生态里没有一个能整个拿来用的「降噪框架」，但每个零件都有现成的 crate 或成熟写法。** 直接依赖只需要十来个小 crate（ort · realfft · rubato · rtrb · triple_buffer · biquad · earshot · hound · cargo-deny 一类）；**跨进程的部分（共享内存环、HAL 驱动、唤醒、工作区间）没有能用的 crate，要自己写**，照 voice-strip、Firewheel、rtrb 的做法写。

## 一、每个模块的推荐依赖

「怎么用」：**依赖** = 直接写进 Cargo.toml ·**抄思路** = 读懂后自己写（铁律 3，不复制代码）·**dev** = 只在测试 / 评测里用。

### 1. 音频 I/O 与线程（io-macos · hal-plugin）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| 麦克风采集 | **抄思路**：自写 HAL IOProc（`AudioDeviceCreateIOProcID`），手写 FFI | cpal 0.18.2（只用于原型、设备枚举、eval） | cpal 底层 coreaudio-rs 在帧数变化时在实时线程读属性并 `Vec::resize`（`resources/coreaudio-rs/src/audio_unit/render_callback.rs:622,649`）；cpal 每次回调调 `mach_timebase_info`（`resources/cpal/src/host/coreaudio.rs:76-78`，读的是 main 分支，0.18.2 行为以 tag 为准）。IOProc 直接给设备 sample/host time，便于对齐时间轴 |
| 虚拟麦克风 HAL 驱动 | **抄思路**：照 voice-strip `vdev-macos-driver` 自写 Rust cdylib，只依赖 libc | tympan-aspl 0.1.0（0 star、只发过一版） | voice-strip 证明纯 Rust 驱动可行，IO 路径只有原子加减 + 一次读环（`resources/voice-strip/crates/vdev-macos-driver/src/plugin.rs:771-810`）。objc2-core-audio 最后发版 2025-10-04，已超 12 个月，复核建议手写 FFI |
| 跨进程音频共享内存 | **抄思路**：`shm_open + mmap` + 固定 `#[repr(C)]` 全原子布局，用 voice-strip 的「时间轴 + 分槽序号」环 | memmap2 0.9.12 只管映射 | 见下文「设计 1」。iceoryx2 太重（macOS 只是 tier 2），shared_memory 2022 后不发版【API】 |
| 共享内存安全交接（发布必需） | **抄思路**：LaunchDaemon 登记 Mach 服务，用 `xpc_shmem_create` 把匿名共享区交给驱动；只放行 `_coreaudiod` 和本用户 | — | GPL-3.0 先例 `resources/m-audio-fast-track-ultra-macos-driver`（gaps.md G1）；voice-strip 用 0644 文件，自注「只够 spike」（`shm.rs:7-10`） |
| 跨进程唤醒推理线程 | **抄思路**：`libc` 的 `os_sync_wait_on_address` / `wake`（SHARED 标志） | 推理线程定时醒来轮询 | 要 macOS 14.4+（gaps.md G2）；唤醒是实时线程里的一次系统调用，要和轮询做小实验比 |
| 实时线程优先级 + 工作区间 | **抄思路**：Mach time-constraint + `AudioWorkIntervalCreate` / `os_workgroup_join`，每帧 `interval_start` 带截止期 | audio_thread_priority 0.38（MPL-2.0，兼容；不支持 workgroup） | 见「设计 5」的补充；objc2-audio-toolbox 跳过了 AudioWorkIntervalCreate【源码，io.md】 |
| 声纹线程放能效核 | `libc::pthread_set_qos_class_self_np`（UTILITY） | thread-priority 3.1.1 | libc 有绑定（复核核过）；「低 QoS 就上 E 核」是推断，P3 用 powermetrics 量 |
| 控制面 IPC（control） | std `UnixStream` | ipc-channel 0.23.1 | 不上实时线程；ipc-channel 序列化会分配，有 UB 类 issue #457 无人回【API】 |

### 2. 实时安全零件
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| 进程内 SPSC 环 | **依赖** rtrb 0.4.1 | ringbuf 0.6.0 | push/pop 只在疑似满/空时读对端索引，零分配零锁（`resources/rtrb/src/lib.rs:490-503,778-791`）；ringbuf API 常破坏性变更，10-30 刚修完 soundness #65/#66【API】 |
| 最新值发布（声纹→门控、状态） | **依赖** triple_buffer 9.0.0（进程内） | arc-swap（只在非实时端） | 两端各一次 `swap(AcqRel)`，wait-free（`resources/triple-buffer/src/lib.rs:320-348,516-547`）；MPL-2.0 无 Exhibit B（复核升为源码）。跨进程版按同样原理自写 |
| 单值参数 | 自写 `AtomicU32` + `f32::to_bits` | portable-atomic float | 十来行；atomic_float 2024-08 后不发版 |
| 实时线程不释放内存 | rtrb 回收队列：旧对象送回非实时线程 drop | basedrop（最后发版 2025-10-29，已超 12 个月；#10 竞态报告无人回） | 复核把首选从 basedrop 改为回收队列（voice-strip、Firewheel 都这么做） |
| debug/test 里断言不分配 | **dev** nice-assert-no-alloc 1.0.0，或照 assert_no_alloc 自写约 60 行计数分配器 | assert_no_alloc 1.1.2（BSD-1-Clause，2021 后停更） | 只抓经 Rust 全局分配器的分配；**p1 已用 stats_alloc + assert_no_alloc 实测 GTCRN 每帧 0 次**【实测，p1】 |
| 抓锁、系统调用、C++ 内部分配 | **dev，可选 feature** rtsan-standalone 0.3.0（`#[nonblocking]`，`RTSAN_ENABLE=1` 的 CI job） | rustc nightly `-Zsanitizer=realtime`（已支持 aarch64-apple-darwin） | libc 层拦截，能看到 ORT / CoreAudio 内部；最后提交 2025-09-27，命门 3 有条件满足；默认 `prebuilt-libs` 会联网下载，CI 里关掉或给 `RTSAN_LIB_PATH` |
| 去非正规数 | **抄思路**：照 nih-plug `ScopedFtz` 自写（aarch64 FPCR 置 FZ 位） | no_denormals 0.3.0（MIT，2025-09，已超 12 个月） | `resources/nih-plug/src/wrapper/util.rs:25,191`；FPCR 每线程一份，每个实时/推理线程入口都要设。**先别对 ORT 开 `with_flush_to_zero`**：DeepFilterNet #713 是 FTZ + 单位范数 0/0 出 NaN |
| MPMC 队列 / seqlock crate | **不要** | — | crossbeam ArrayQueue 争用时 `yield_now`（`crossbeam-utils/src/backoff.rs:207-219`）；seqlock crate 写端用 Mutex |

### 3. DSP（dsp）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| FFT（512 点实 FFT，STFT 与 fbank 共用） | **依赖** realfft 3.5.1（底层 RustFFT 6.4.1） | 手写几行 FFI 调系统 vDSP_DFT | `process_with_scratch` 零分配（`resources/realfft/src/lib.rs:138-143`）；RustFFT 在 aarch64 运行时检测 NEON（`resources/RustFFT/src/neon/neon_planner.rs:156-157`）。RustFFT 最后发版 2025-09-18，算法库已定型，见「待人拍板 1」 |
| STFT / iSTFT（sqrt-Hann 512/256，对齐 torch.stft） | **抄思路**：在 dsp 自写，骨架参考 libDF | — | libDF 结构清楚但每帧分配两次（`resources/DeepFilterNet/libDF/src/lib.rs:360,397`），改成预分配；torch.stft 默认 center=True 反射填充，对拍时首尾帧要单独处理 |
| 重采样 48k↔16k | **依赖** rubato 5.0.1（Fft 同步型，`new_custom` 调小子块） | rubato Async（sinc） | `process_into_buffer` 预分配；Fft 默认延迟上采约 8 ms、下采约 2.7 ms（复核手算），**45 ms 预算里 STFT 已占约 32 ms**，必须调参并实测 |
| 两个设备的时钟漂移 | rubato `Slip`（插/丢一帧 + 交叉淡化） | rubato Async + 相对比率 | `resources/rubato/src/slip.rs:9-30`，几乎零延迟 |
| 限幅 / 轻压缩 | **抄思路**：照 sonora AGC2 Limiter 自写 | — | 子帧增益插值、零前瞻（`resources/sonora/crates/sonora-agc2/src/limiter.rs:12-40`）；sonora 整条 AGC2 每帧分配（`limiter.rs:98`、`gain_controller2.rs:238`），**不能直接进实时线程** |
| 双二阶滤波 | **依赖** biquad 0.6 | 自写 DF2T | `run()` 无分配无锁（`resources/biquad-rs/src/lib.rs:170-181`）；依赖 num-traits(libm) |
| 传统降噪 / VAD 对照基线 | **dev** sonora（纯 Rust WebRTC NS + AGC2） | nnnoiseless（BSD-3） | 只做对照；tonarino webrtc 绑定在 macOS 27 有构建 panic（issue #103 无人回）且每帧分配 |

### 4. 推理（engine）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| GTCRN 流式 ONNX | **依赖** ort 2.0.0-rc.15（CPU EP、`with_intra_threads(1)`、关 spinning、`with_auto_device(PreferCPU)`） | tract 0.23 | biglinux-gtcrn 已用 ort 逐帧跑通；`run_with_options` + `OutputSelector::default().preallocate(全部输出)` Rust 侧零分配（`~/.cargo/…/ort-2.0.0-rc.15/src/session/run_options.rs:138`、`src/value/mod.rs:393-397`），**p1 实测 GTCRN 每帧 0 次**【实测】。tract 每个节点每次 eval 都新建输出张量（tract `plan.rs:682-692`），分配更多 |
| 注意的坑（p1 实测） | `OutputSelector::no_default()` + preallocate 会让 ORT 报 "At least one output should be requested"，要用 `default()`；rc.15 在 macOS 默认会挂 CoreML，GTCRN 编译失败，必须 `PreferCPU`【实测，p1】 | — | — |
| FastEnhancer-M 同接口备选 | ort，用官方 **16 kHz spec2spec** 导出（`onnx-dns-v1.0.0` / `onnx-vd-v1.0.0` 里的 `fastenhancer_m.spec.onnx`）【官方，release 说明明载 16 kHz】 | tract | spec2spec 才能和 GTCRN 共用 dsp 的 STFT；FE-M 7 进 7 出，超出 ort 栈容量（≤ 12 个名字），**每帧 7 次小分配**【实测，p1】，要么接受（推理线程是软实时），要么合并状态张量再导出 |
| 声纹 ECAPA / CAM++ | ort，共用 Environment、单独 Session | tract | 不在实时路径；少带一份运行时 |
| CoreML / ANE | **不作默认** | CPU EP | CoreML EP 算子表没有 GRU、ScatterND，GTCRN 两样都用，图会被切碎【官方】；省不省电要实测，见「待人拍板 4」 |

### 5. 声纹与 VAD（speaker）
| 需求 | 首选 | 备选 | 理由 · 出处 |
|---|---|---|---|
| 80 维 fbank | **抄思路**：自写 `dsp::fbank`（约 200 行，realfft + 调用方给工作区 + 窗型可配），以 torchaudio `kaldi.fbank` 为对拍基准 | **dev** kaldi-fbank（纯 Rust、Apache-2.0，单作者、建仓 2 个月）做第二个对照 | **两个候选模型前端不一样**：WeSpeaker 用 hamming 窗、输入 ×32768；3D-Speaker CAM++ 用 povey 窗、输入保持 [-1,1]（speaker.md 存疑 1）。knf-rs 把窗写死成 povey，还要 cmake 编 C++ |
| 实时 VAD（门控「快开」） | **依赖** earshot | 自写能量门限 + 滞回 | 纯 Rust、状态全是定长数组、零分配（`resources/earshot/src/lib.rs:238-293`）；16 ms/256 点帧和 GTCRN hop=256 对齐；**输入要先夹到 ±1.0**，否则 debug 构建 panic（`frame.rs:25`、`lib.rs:249`）；权重来源未说明，发布前要核 |
| 非实时 VAD（挑清楚的单人语音） | silero（findit-studio）的结构 + 自家 ort 跑 silero_vad.onnx | earshot | Session 和 StreamState 分离（`resources/silero-findit/src/session.rs:160-224`） |
| 在线注册 / 认人 | **抄思路**：sherpa-onnx manager 的「先求和再 L2 归一化、矩阵×向量算余弦」+ pyannote-rs 的「过阈值归入、否则新建」，改成固定容量 | speakrs（kodama AHC）做锁定前一次性聚类 | `resources/sherpa-onnx/sherpa-onnx/csrc/speaker-embedding-manager.cc:63-81,117-135`；Rust 生态没有零分配的在线注册 crate，核心不到 150 行 |
| 不要 | sherpa-onnx 官方 Rust crate 整包用 | — | 自带并静态链接一份 ORT，build.rs 从 GitHub 下载，可能和 ort crate 冲突；**TEN VAD 有「不得与 Agora 竞争」条款，与 GPL-3.0 不兼容** |

### 6. 宿主架构（engine · control）
| 需求 | 首选 | 理由 · 出处 |
|---|---|---|
| 音频图 | **不要图**，自写固定管线 | 链路固定；dasp_graph 每次处理都重做 DFS 且 2020 后不发版（`resources/dasp/dasp_graph/src/lib.rs:322`）。nih-plug、Firewheel、clack、kira **都不当依赖，只学设计** |
| engine 生命周期 | 照 clack 用类型分离：`activate()` 造处理器（可分配）、`process()` 实时、`deactivate(self)` 吃掉处理器 | `resources/clack/plugin/src/plugin.rs:198,225,249`；比 nih-plug 的注释约束更硬。nih-plug 的 `reset()`（`plugin.rs:216`）对应「清 GRU 缓存」 |
| 参数 | 每个参数一个原子量 + 稳定字符串 ID（nih-plug `Params` derive 生成「ID → 参数指针」表，`resources/nih-plug/nih_plug_derive/src/params.rs:415-418`）；平滑照 nih-plug `Smoother`（`src/params/smoothing.rs:47,260,283`） | 用稳定 ID 而不是下标，加参数不错位 |
| 延迟上报 | 照 clack 规矩：延迟只在激活时定，**换模型 = 重新激活** | `resources/clack/extensions/src/latency.rs:99`；HAL 驱动读 `kAudioDevicePropertyLatency` |
| 模型交叉淡化 | 照 Firewheel `Declicker` + 预计算淡化表 | `resources/Firewheel/crates/firewheel-core/src/dsp/declick.rs:17,113,486`；默认 3 ms 太短，调到 20–50 ms |
| 日志 / 遥测 | 自写两层：共享内存原子计数器 + 定长事件经 rtrb 送日志线程 | tracing-appender non_blocking 每行 `to_vec`，不能进实时线程（gaps.md G3） |
| 菜单栏 / 进程托管 | tray-icon + muda；launchd KeepAlive 拉起；SMAppService 用 objc2-service-management | gaps.md G4；Slint 有 GPL-3.0 许可分支可做设置窗口 |
| 别照抄 | nih-plug CLAP 包装的 process 里拿 `parking_lot::Mutex`（`src/wrapper/clap/wrapper.rs:2239`）；`schedule_gui` 在音频线程调 `CFRunLoopWakeUp`（`macos.rs:99-103`）；Firewheel 回程队列满时 `let _ = try_push(..)` 会在音频线程 drop（`handle_messages.rs:42-44`） | 复核找到的反例 |

### 7. 工具链（CI · 发布）
| 需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| 依赖树许可白名单 | cargo-deny 0.20.2（`[licenses]` 默认全拒 + 白名单） | cargo-about | 对 OR 表达式逐叶求值（`resources/cargo-deny/src/licenses.rs:74-97`）；GPL-3.0-only 与 -or-later 要分开写；官方 Action 是 Docker 动作，放 ubuntu runner |
| 第三方许可清单（GPL 发布） | cargo-about | cargo-license（15 个月无提交） | ONNX Runtime 原生库与模型权重不在 Cargo 图里，要手补 |
| 测试 | cargo-nextest · proptest · insta（只存摘要指标）· 自写对拍 Report（照 approx 的混合判据，报出哪帧哪维挂） | cargo test · approx | — |
| 并发正确性 | loom（`sync` 门面模块切换原子类型）+ Miri | — | `resources/loom/src/lib.rs:120-131`；两者都测不了跨进程 mmap，算法与映射要拆开测 |
| 基准 | criterion（本机） | divan（近 18 个月未发版） | 指令数回归 gungraun 只能放 Linux runner（Valgrind 不支持 arm64 macOS） |
| 打包签名公证 | 自写脚本：codesign → pkgbuild/productbuild → productsign → notarytool → stapler | cargo-packager（不出 .pkg） | cargo-dist 不签 bundle、不公证，HAL 驱动 .pkg 用不了 |
| 其他 | cargo-auditable · cargo-machete | cargo-udeps（要 nightly） | cargo-semver-checks 不要（不对外发库） |

（「哪条规则用哪个工具、CI 怎么分层」归 rustarch 008，这里只给项目选型。）

## 二、最值得学的 5 个设计

**1. voice-strip 的「时间轴 + 分槽序号」共享内存环 —— 推理进程崩了，原声自动顶上（D03）**
写端改一个槽前把序号置成奇数、写完置成偶数；读端前后两次看到同一个偶数才认，否则这段当「缺」、填零，**从不等待写端**。位置按绝对时间轴（host clock）算，送音进程写、HAL 读不需要任何握手。
出处：`resources/voice-strip/crates/vdev-macos-driver/src/shm.rs:143-173`（write）、`:180-`（read，本人复看过）；驱动 IO 路径 `plugin.rs:771-810`；时钟 `clock.rs:1-9`。
对我们：送音→HAL、推理→送音两条链共用一份协议；推理进程一停，槽不再更新 → 读端判缺 → 送原声，不用锁、不用超时。复核结论：**跨进程统一用这种环，rtrb 只在进程内用**（head/tail 环在一端崩溃后要重置索引、另加代际号）。
注意：voice-strip 10-17 建仓、单人、README 自述大量由 AI 写；只抄思路，不当依赖。

**2. Firewheel 的「造新 → 送进 → 换下 → 送回 → 主线程释放」—— GTCRN ↔ FastEnhancer-M 热切换的骨架**
新对象在非实时线程造好，经无锁队列送进实时线程，实时线程只做 `mem::swap`，换下的旧对象原路送回主线程释放。
出处：`resources/Firewheel/crates/firewheel-graph/src/processor/handle_messages.rs:69-116`、`processor.rs:242-256`；节点拆成主线程 `AudioNode` 与实时 `AudioNodeProcessor` 两个 trait（`firewheel-core/src/node.rs:266,486`）。
对我们：新模型的 Session 与状态在后台建好并跟跑 N 帧预热，指针经 rtrb 送进去，交叉淡化后旧模型送回释放。**要补它的漏洞**：回程队列满时它 `let _ = try_push(..)` 会在音频线程上 drop（`:42-44`、`:114-116`）——我们要「满了就留在本地槽位、下一块再送」。

**3. biglinux-gtcrn + ort 预分配 —— GTCRN 状态显式进出，每帧零分配**
状态张量初始化时建好两套，奇偶帧轮换（A 的输出是 B 的输入），全部输出用 `OutputSelector::preallocate`；开机先预热 20 帧，把 GRU 状态从全零推开，记下预热后的参考状态用于复位。
出处：`resources/biglinux-gtcrn/ladspa/src/model.rs:282-302`（状态进出）、`:30-33,130-149,226-239`（预热与复位）；`ort-2.0.0-rc.15/src/session/run_options.rs:95-140`。
对我们：engine 热路径的直接样板；p1 已按此实现，实测 GTCRN 每帧 0 次分配【实测】。预热 + 参考状态也用于推理进程崩溃重启、换模型，避免冷启动过度抑制。

**4. rubato 的「分配版只是包一层预分配版」—— 离线评测与实时回调同一份代码**
`process()`（会分配，给离线用）内部只是建好缓冲后调 `process_into_buffer()`（预分配，给实时用），核心算法只有一份。
出处：`resources/rubato/src/lib.rs:213-230`、`:263-268`；同类做法：libDF STFT 状态机让离线和实时共用一个 process（`resources/DeepFilterNet/libDF/src/lib.rs:119-122,356-393`，但它漏了预分配）；speakrs fbank 的「共享不可变 Recipe + 每个句柄私有工作区 + `compute_into`」（`resources/speakrs/src/inference/cpu/fbank.rs:15-34,58-75`，去直流和预加重用 f64 防安静频带误差）。
对我们：dsp / engine 每个处理器都出这对双接口，满足 intent「离线评测与实时用同一份处理代码」，单元测试直接覆盖实时路径。

**5. nih-plug `process_wrapper` + rtsan `#[nonblocking]` —— 一层守卫，平时零成本、测试时抓违规**
每次进处理函数先开 FTZ，debug 构建再套「禁止分配」，全局分配器换成守卫版；rtsan 的宏只插一个 RAII 守卫，build.rs 按 `RTSAN_ENABLE` 决定编不编进去，发布版零开销。
出处：`resources/nih-plug/src/wrapper/util.rs:25,35-37,189-199`；`resources/rtsan-standalone-rs/build.rs:10-17`、`src/lib.rs:32-37`、`crates/rtsan-standalone-macros/src/lib.rs:33`。
对我们：送音回调、降噪线程、离线 eval 入口统一套这一层；CI 用 `RTSAN_ENABLE=1` 跑离线 WAV，每次提交都验命门 2，顺带量出 ORT C++ 内部到底有没有 malloc / 锁。

（候补：rtrb 的「缓存对端索引 + CachePadded」`resources/rtrb/src/lib.rs:104-109,490-503`，自写跨进程环时照搬；triple_buffer 的「一个原子字节 = 槽索引 + 脏位」`resources/triple-buffer/src/lib.rs:320-348`，正是声纹线程原子发布结果的形状。）

## 三、最值得学的一个项目

**voice-strip**（Rust，MIT OR Apache-2.0）：它几乎就是我们架构的单进程版——Rust 自写 macOS HAL 虚拟麦克风驱动 + 共享内存 + 神经网络降噪 + Audio Work Interval。学它的共享内存环协议（设计 1）、IOProc 采集只做「拷贝 → 转换 → 写环」、推理不进音频线程、降噪线程加入工作区间（`crates/denoise-neural/src/rt.rs:243-275`）。风险：才两周、单人、AI 写，**只抄思路**。

## 四、还没答上的（转实验或转别的线）

| 问题 | 去向 |
|---|---|
| ORT C++ 内部稳态是否 malloc / 加锁 | P1/P3 跑一次 RTSan（要名额）|
| 跨进程唤醒：`os_sync_wait_on_address` vs 定时轮询，哪个延迟、耗电更好 | 有界小实验，P3 前 |
| 推理线程能否跨进程加入麦克风 IO 线程的 workgroup | 没找到开源先例，P3 实验 |
| tract 能否加载 GTCRN、数值能否对上 | 只在 ort 出问题时再做 |
| rubato Fft 与 Async 哪个在 45 ms 预算里更划算 | P3 实测 |
| aarch64 FPCR.FZ 是否也把输入非正规数当零 | 读 ARM 手册，非阻塞 |
| earshot 权重来源、FastEnhancer 权重许可、ORT ThirdPartyNotices | 发布前许可核查（与 D33 注意项合并）|
| panic 策略（abort 还是 catch_unwind） | rustarch 008 |

## 五、待人拍板

| # | 题面 | 选项 | 代价 | 建议 | 默认（不回就按它） |
|---|---|---|---|---|---|
| 1 | 命门 3「近 12 个月有维护」要不要放宽给「已定型的小库」？RustFFT（2025-09）、hound（2023-09）、no_denormals、rtsan-standalone、basedrop 都刚过线 | A 放宽：算法 / 格式库只要无未修 soundness issue、仓库仍有人回就算满足 · B 严格：过线一律不当依赖，自写或找替代 | A：将来可能要自己 fork 修；B：FFT、WAV 读写要自写，工作量大且更容易出错 | **A** | A |
| 2 | 最低支持的 macOS 版本 | A macOS 14.4+（能用 `os_sync_wait_on_address` 跨进程唤醒）· B 更老版本（只能用定时轮询或 Mach 信号量） | A：放弃 2024 年 3 月前的系统；B：唤醒更慢或更费电 | **A**（用户基本在新系统上；做完小实验再最终定） | A |
| 3 | 本项目许可写 GPL-3.0-only 还是 GPL-3.0-or-later | A or-later · B only | cargo-deny 白名单、LICENSE 头都要据此写；or-later 允许将来 GPL-4 | **A**（FSF 推荐、生态常见）| A |
| 4 | 要不要做一次 CoreML / ANE 耗电实验 | A 做（有界：ProfileComputePlan + powermetrics，约半天）· B 不做，CPU 单线程到底 | A：花半天，可能发现更省电；B：GTCRN 的 GRU、ScatterND 不受 CoreML 支持，大概率白做 | **B**，等 P3 实测 CPU 耗电超出 D31 再做 | B |
| 5 | HAL 驱动拿共享内存走「LaunchDaemon + XPC」 | A 照 GPL 先例：系统域 LaunchDaemon + Mach 服务 + `xpc_shmem_create`，发版前改成按代码签名放行 · B 先用文件共享内存（只够内测）| A：安装要管理员权限、多一个常驻进程；B：同一用户的任何进程都能读麦克风音频，不能发布 | **M0–M1 用 B，发版前必须换 A** | 同建议 |
