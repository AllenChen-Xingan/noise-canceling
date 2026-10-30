# 两类发现 · Mac 上的实时音频管线与虚拟麦克风怎么接

> 2026-10-29，手下 pipeline 填。三轮核查：第一轮 5 个子代理，补检索 4 个，对抗核实 5 个（怀疑派 · 实现派 · 产品派 · macOS 版本 · 插件宿主进程），共 14 个代理，都用同一张核查表。来源登记见 `sources.md`。
> 证据等级：【官】官方明载 ·【源】源码可证 ·【测】本机只读实测（macOS 27.0.1）·【传】传言 ·【推】推断。
> 分级：P0 最高、P3 最低；**P0 / P1 / P2 清零，P3 酌情**。

## 〇、对抗核实推翻或修正的前提（先读这一节）

| 原来以为 | 核实后 | 出处 |
|---|---|---|
| 插件跑在 `coreaudiod` 里，崩了整台电脑没声音（设计稿 `vault/raw/20261029-voice-denoise-design.md:248`） | **过时。** 今天的 macOS 上，`/Library/Audio/Plug-Ins/HAL` 下的原生插件由独立的 `com.apple.audio.Core-Audio-Driver-Service.helper` 进程加载，`coreaudiod` 明确跳过进程内加载。插件崩溃的后果大概率是**本设备掉线**，不是整机失声。没在第三方插件上实测 | 【官】SDK `AudioServerPlugIn.h:34-41`（26.5 与 27 两版措辞相同）；【测】本机开机日志 `coreaudiod … RegisterPlugIns: skipping in-process plug-ins`、`HALC_ShellDevice.cpp:3821 Not loading the driver plug-in into coreaudiod`，以及工厂函数在 `Core-Audio-Driver-Service.helper[584]` 里执行；【源】`resources/roc-vad/README.md:679-681`（2023 年第三方插件日志，来源也是 Driver-Service） |
| 本机装着一个第三方插件 `ParrotAudioPlugin` | **不对**，它是 Apple 自家的插件（`com.apple.audio.ParrotAudioPlugin`，Platform identifier 26）。本机没有任何第三方 HAL 插件，也**没装 BlackHole** | 【测】`plutil -p`、`codesign -dv` |
| P1：插件只从**共享内存**环形缓冲读数据 | **部分成立。** 推理放在插件外面：成立。用共享内存传音频：查过的开源项目（BlackHole · Background Music · roc-vad · libASPL · eqMac）**没有一个这么做**，都是插件内部回环。宿主沙箱规则允许 POSIX 共享内存，但 Apple 没明说插件能用 | 【源】各仓库 grep `shm_open`/`mmap` 零命中；【测】`/System/Library/Sandbox/Profiles/com.apple.audio.coreaudiod.sb:141` `(allow ipc-posix-shm)`；第三方插件宿主是否沿用这份沙箱：【推】 |
| P4：推理线程加入设备的 Audio Workgroup，Rust 有现成办法 | **不成立。** 推理线程隔着环形缓冲按自己的节拍跑，属于异步线程；官方做法是用 `AudioWorkIntervalCreate` **自建** work interval，而不是加入设备的 workgroup。Rust 里没有现成封装，要写约 60 行 FFI | 【官】developer.apple.com/documentation/audiotoolbox/adding-asynchronous-real-time-threads-to-audio-workgroups；【官】SDK `AudioToolbox.framework/Headers/AudioWorkInterval.h:30-50,88`；【源】`resources/audio_thread_priority` 全仓无 workgroup |
| P5：最终产品用 HAL 插件，Developer ID 就够 | **方向成立，细节未定。** DriverKit 拿不到虚拟设备的 entitlement，排除。宿主进程只检查签名是否有效，不检查签发团队；但 macOS 15 以后 ad-hoc 签名能否加载，第三方说法互相矛盾，**只能实测**。给普通用户分发，Developer ID 加公证仍是 Gatekeeper 那一关的门槛 | 【官】developer.apple.com/forums/thread/775341（AudioDriverKit 只支持实体设备）；【测】宿主 helper 字符串 "not signed but is being loaded anyway"，带 `disable-library-validation`；【传】tympan-aspl / MacAudio README 说会被拒（-423），boss-gt-10 `docs/VERIFICATION.md:4` 说 macOS 27 上 ad-hoc 能加载 |
| 处理进程挂了就输出静音，算满足命门 1 | **违背收敛方向。** 静音时对方一个字都听不到，而且双方都不知道，这就是「吃字」。兜底应该是送原声 | 【源】`resources/BlackHole/BlackHole/BlackHole.c:4561-4571`（写端停写，读端整块填零）；【官】`.42cog/intent.md` 收敛方向「从不吃字」 |

---

## 一、它们做到的（首选方案逐项核对）

首选方案：**开发期 = cpal 两条流 + BlackHole；最终产品 = 自写的极简 HAL 回环插件 + Rust 处理进程**（详见 `decision.md`）。

| # | 不可妥协的那条 | 状态 | 差距是什么 | 修起来要什么 | 分级 |
|---|---|---|---|---|---|
| 1 | 崩溃隔离 | **部分** | 隔离边界已经有了：插件在独立宿主进程，推理在我们自己的进程。缺的是**兜底送什么**：所有先例在断供时都送静音，没有一个送原声 | **接一根线**：在 Rust 进程里做旁路，推理超时就把重采样后的原声写出去（挡得住模型卡顿）。**另立项**：拆成「不带模型的送音进程 + 推理进程」两层，才挡得住整个推理进程崩溃。原声兜底无先例【推】 | **P0** |
| 2a | 延迟可量、稳定 | **部分** | 能量：cpal 在回调时间戳里已经算进了设备延迟、安全偏移和 IO 缓冲（`resources/cpal/src/host/coreaudio/macos/device.rs:609-649,937-956`）。缺实测数字：所有参考仓库都**没有**附加延迟的实测。推算直通约 6–10 ms，加 10 ms 帧后约 15–20 ms，再加模型前瞻（归 002） | **接一根线**：M0 用两端回调时间戳实测，跑一小时 | **P1** |
| 2b | 时钟漂移 | **部分** | 零件齐全：rtrb 读水位（`resources/rtrb/src/lib.rs:666`），rubato 运行中调比率（`resources/rubato/src/asynchro.rs:806-830`）。但**没有一个参考项目把这两样接成水位控制**：MetalVoice 只靠丢帧兜底（`resources/MetalVoice/Sources/Core/AudioModel.swift:196-218`），Background Music 只是跳帧重对齐 | **接一根线**：在输出回调里用 PI 控制器调 `set_resample_ratio_relative`，约几十行 | **P1** |
| 2c | 采样率变化 / 热插拔 | **部分** | cpal 会报 `StreamInvalidated`、`DeviceNotAvailable`、`DeviceChanged`（`resources/cpal/src/host/coreaudio/macos.rs:251-268,481-483`），但**不会自动续流**。打开 AirPods 麦克风本身就会触发切到 16 kHz（`resources/MetalVoice/Sources/Core/AudioModel.swift:252-255` 注释） | **接一根线**：错误回调只发一个事件，主线程防抖后重建流、环形缓冲和重采样器。另一个坑：cpal 建流会改设备的全局采样率（`resources/cpal/src/host/coreaudio/macos/device.rs:560-571`），所以写 BlackHole 必须固定 48 kHz | **P1** |
| 3 | 普通用户装得上 | **部分** | 不用关 SIP，DriverKit 也排除了。剩下三件：**要管理员密码**（省不掉）；**要重启 coreaudiod 或重启电脑**（14.4 起 `launchctl kickstart` 被拒，只能 `killall`【传】developer.apple.com/forums/thread/748228）；**给别人装要 Developer ID 加公证**，99 美元/年【官】developer.apple.com/programs/whats-included/。Homebrew 从 2026-09-01 起禁用过不了 Gatekeeper 的 cask【传】github.com/Homebrew/brew/issues/20755 | **要人定**：付不付费（见 `decision.md` 待拍板）。开发期 ad-hoc 签名能否加载要做一次实验，实验要碰系统目录，需要人批准 | **P1** |
| — | 回声消除参考信号（P3） | **部分** | process tap 14.2 起能抓全系统输出，还能排除我们自己的进程（SDK `CATapDescription.h:58-83`）。但要多一道「系统录音」权限，而且没有公开 API 预检。和麦克风对时钟要靠把两者放进同一个聚合设备，这一步**没人实测过**。macOS 27 新增 `kAudioDevicePropertySuggestedReferenceDevice`（SDK `AudioHardware.h:1006`），可能是更省事的路，没核实 | **另立项**，归后续里程碑；不挡 M0–M2 | **P3** |
| — | 实时线程优先级（P4） | **部分** | `audio_thread_priority` 能设 Mach 时间约束策略（`resources/audio_thread_priority/src/rt_mach.rs:63-140`）；自建 work interval 要约 60 行 FFI，join token 在 arm64 上 36 字节、在别的架构上 28 字节，布局写错会静默踩内存（SDK `usr/include/os/workgroup_base.h:59-65`） | **接一根线**，M1 时做 | **P2** |

## 候选之间（同一批核查项，逐格对照）

| 核查项 | BlackHole | 自写 C 插件（照 NullAudio 的结构） | libASPL | roc-vad | Background Music | MetalVoice |
|---|---|---|---|---|---|---|
| 设备注册 | HAL 插件 ✅ `BlackHole.plist:25-35` | HAL 插件 ✅（官方示例） | HAL 插件 ✅ `README.md:31-45` | HAL 插件（基于 libASPL）✅ | HAL 插件 ✅ | ❌ 自己不注册，依赖 BlackHole |
| 和用户态怎么传音频 | 插件内部回环，无 IPC `BlackHole.c:339-340` | 内部回环（照 BlackHole 的形状自己写） | 库不提供 IPC `README.md:506-518` | 音频在插件里收发 RTP；控制走 gRPC/TCP `driver/driver.cpp:82-92` | 走 HAL 回环；XPC 只做控制 `BGM_XPCHelper.m:68-71` | 单进程 |
| 重活在哪 | 无重活 | 无重活 | — | **在插件里**（反例）`driver/request_handler.cpp:61-134` | 在用户态 app | **在实时回调里跑模型**（反例）`DeepFilterNetDSP.swift:262-272` |
| 漂移补偿 | ❌ 只有手动 ±1% 调速 `BlackHole.c:290-293` | 由 Rust 侧做 | ❌ `src/Device.cpp:1261-1298` | ✅ roc 自适应重采样（在插件里） | ❌ 跳帧重对齐 | ❌ 丢帧兜底 |
| 运行中改采样率 | ✅ `BlackHole.c:915-935` | 要自己写（照官方配置变更流程） | ✅ `src/Device.g.cpp:179-204` | ❌ | ❌ 只在启动时同步一次 | ✅ 重建转换器 |
| 断供时输出 | 静音 `BlackHole.c:4561-4571` | 由我们定（建议原声，见上） | — | 静音 `driver/ring_buffer.cpp:112-135` | — | 静音（欠载填零） |
| 延迟实测 | 无 | — | 无 | 无（默认缓冲 60 ms `device_defs.hpp:38`） | 无 | 无 |
| 签名公证 | Developer ID + 公证 ✅ `Installer/create_installer.sh:62-67,119-126` | 待定（待人拍板） | 可选 | 无（`d8c45d2` 删掉了签名） | ad-hoc + 去隔离属性 `pkg/postinstall:55-63` | ad-hoc，没公证 |
| 许可证 | GPL-3.0，二进制和商标保留 | 自有 | MIT | MPL-2.0 | GPL-2.0+ | MIT |
| 维护 | 活跃，2026-10-28 | — | 2025-04-14 | 2025-04-13 | 2026-06-10 | 2026-08-31 |
| **四选一** | **直接用**（只限 M0–M2，让开发者自己装） | **在它基础上改造**（最终产品首选） | **在它基础上改造**（备选骨架） | 只借鉴架构 | 只借鉴架构 | 只借鉴架构（负例） |

## 二、它们没做的

| 找过什么 | 结论 | 是哪一种 |
|---|---|---|
| 断供时**送原声**而不是静音 | **无人涉足。** BlackHole、roc-vad、Background Music、MetalVoice 全仓 grep watchdog / passthrough / fallback 零命中。插件自己拿不到真麦克风，SDK 禁止插件调用 HAL 客户端 API（`AudioServerPlugIn.h:34-36`），所以只能在用户态做 | **没顾上做**：没人提过，也没人试过。是机会，也是命门 1 的缺口 |
| 麦克风到虚拟设备的**水位控制漂移补偿** | **无人涉足**（在开源虚拟麦克风里）。roc-vad 有自适应重采样，但那是网络到设备方向，而且在插件里 | **没顾上做**：零件现成（rubato 就是为这个设计的，`README.md:83-116`），属于组装 |
| 附加延迟的**实测数字** | **无人涉足**。BlackHole 只说「零附加延迟」，指的是驱动本身不加；MetalVoice 写的是宣传语「Zero Latency Feel」 | **没顾上做**：M0 自己测 |
| 插件与用户态之间**用共享内存传音频** | 有设计稿没有实现（jack-router 设计稿；Background Music 作者在提交 `2171d9f` 里说「最好改」但没做）；第三方 PR 里有人这么用【传】 | **没顾上做**，不是做不到：沙箱允许，但 `_coreaudiod` 用户权限、命名、清理都要自己处理。留作二期，前提是实测证明回环那一跳超出延迟预算 |
| Rust 写的 HAL 插件 | 有一个先例 tympan-aspl 0.1.0，但没有真机数据通路验证，0 star | **没顾上做**：不是天堑，CFPlugIn 只要求 C ABI。第一版不值得冒这个险 |
| macOS 15 以后 ad-hoc 签名插件能否加载 | 第三方说法互相矛盾，没有官方说法 | **查不实**：读源码判不了，只能实测 |
| 插件宿主进程崩了，coreaudiod 是重启宿主还是让设备消失 | 官方文档和 Apple 工程师回复里都没有 | **查不实**：要做一次让插件故意崩溃的实验 |

### 该不该自己跑一个

| 问题 | 判据 | 结论 |
|---|---|---|
| M0 直通的附加延迟和一小时稳定性 | 输出清楚，没人测过 | **跑**：这就是 M0 本身，不另开实验 |
| ad-hoc 签名能否加载 + 插件崩溃时 coreaudiod 会不会跟着倒 | 读源码判不了，第三方说法矛盾，而且同时决定命门 1 和命门 3 | **跑**：开 `docs/experiments/exp001-hal-plugin-load/`。要往 `/Library/Audio/Plug-Ins/HAL` 装一个空插件、`killall coreaudiod`，**越过本批红线，等人批准** |
| process tap 与麦克风放进同一个聚合设备后的时钟对齐 | 输入清楚，没人测过 | **暂不跑**：不挡 M0–M2，回声消除是后续里程碑 |
| 共享内存直供 | 先例都没用，依据不足 | **暂不跑**：等回环那一跳的实测数字再说 |

---

**下一步**：压成一次取舍，见 `decision.md`。
