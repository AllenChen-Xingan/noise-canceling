# 两类来源 · Mac 上的实时音频管线与虚拟麦克风怎么接

> 2026-10-29，手下 pipeline 填。**两类进来都问同一句：它今天还算数吗？**
> 证据等级：【官】官方明载 ·【源】源码可证 ·【测】本机只读实测 ·【传】论坛 / 博客 / 第三方自述 ·【推】由前几类推出、未验证。
> 路径 `resources/…` 是浅克隆的参考区（不进仓，可用 `clone.sh` 重建）；逐条核查底稿在会话 scratchpad，结论已全部搬进本文件与 `gap.md`。
> 本机：macOS 27.0.1（26A434），SDK 头文件取自本机 Xcode / CommandLineTools。

---

## 一、自有积累

`scan-own.sh "AudioServerPlugIn"` / `"BlackHole"`：本机 `B-MyCreate/` 下没有做过音频驱动或实时音频的旧项目。

| 是什么 | 在哪 | 还算数吗 | 判据 |
|---|---|---|---|
| 人整理的设计与参考汇总 §6 / §7 / §11 / §16 | `vault/raw/20261029-voice-denoise-design.md:223-299, 309-355, 474-531, 677-708` | **部分算**：标【你定的】的目标算；系统层的四条预判有两条被推翻或修正（见 `must-know.md` 预判表与 `gap.md`） | 人手原件，只读；它的预判是建议，不是标准 |
| 本机 macOS 27 的进程与插件现状 | 本机 `ps` · `ls /Library/Audio/Plug-Ins/HAL` · `codesign -dv` · `log show`（开机日志） | 算（今天测的） | 【测】HAL 目录里只有 Apple 自家的 `ParrotAudioPlugin.driver`（Platform identifier 26），**没有第三方插件，也没装 BlackHole**。开机日志显示 coreaudiod「skipping in-process plug-ins」，插件工厂在独立的 `Core-Audio-Driver-Service.helper` 进程里执行 |

---

## 二、他人积累

### 要比的（逐条核查，同一张核查表，见 `assign.md` 末节）

| 是什么 | 在哪 | 协议 | 读多深 | 还活着吗 | 一句话判断 |
|---|---|---|---|---|---|
| **BlackHole** v0.7.1（纯 C 的 HAL 插件，回环虚拟设备） | `resources/BlackHole` · github.com/ExistentialAudio/BlackHole | 源码 GPL-3.0；**官方二进制和名字保留所有权利**（`LICENSE:1-9`） | 逐条 | 活跃，最近提交 2026-10-28 | **M0–M2 直接用；最终产品只借鉴架构** |
| **libASPL**（C++17 的 AudioServerPlugIn 库） | `resources/libASPL` · github.com/gavv/libASPL | MIT（另含 Apple 示例代码许可） | 逐条 | 最近提交 2025-04-14；CI 覆盖 macOS 13–15 | **在它基础上改造**（备选骨架，首选卡住时才启用） |
| **roc-vad**（基于 libASPL 的网络虚拟设备） | `resources/roc-vad` · github.com/roc-streaming/roc-vad | MPL-2.0 | 逐条 | 最近提交 2025-04-13 | **只借鉴架构**：重活全放在插件里，是我们的反例 |
| **MetalVoice**（Swift，DFN3 + BlackHole，跟我们同一件事） | `resources/MetalVoice` · github.com/Ghostkwebb/MetalVoice | MIT | 逐条 | 最近提交 2026-08-31，20 个提交，单人 | **只借鉴架构，主要当负例** |
| **Apple 官方示例** NullAudio / SimpleAudio / AudioDriverKit | developer.apple.com/documentation/coreaudio/creating-an-audio-server-driver-plug-in · …/building-an-audio-server-plug-in-and-driver-extension · developer.apple.com/documentation/audiodriverkit/creating-an-audio-device-driver | Apple 示例代码许可：可用、可改、可再分发，保留声明（原文见 `resources/BackgroundMusic/LICENSE-Apple-Sample-Code`）。示例原件已下架，libASPL（`README.md:643`）与 Background Music（`BGM_Device.cpp:26-27`）自述基于它 | 读官方说明，没下载 zip | 官方在维护 | **NullAudio：在它基础上改造；AudioDriverKit：不适合**（虚拟设备拿不到 entitlement） |
| **AudioServerPlugIn.h**（SDK 头文件） | 本机 `…/CoreAudio.framework/Headers/AudioServerPlugIn.h` | — | 逐段 | 随 SDK 27.0 | 官方对沙箱、IPC、实时线程的唯一成文说法 |

### 要学的（只读架构）

| 是什么 | 在哪 | 协议 | 还活着吗 | 学到什么 |
|---|---|---|---|---|
| **Background Music**（虚拟输出设备 + 用户态 app + XPC helper 三件套） | `resources/BackgroundMusic` · github.com/kyleneideck/BackgroundMusic | GPL-2.0+（只读懂后自己写，不复制、不链接） | v0.5.0，2026-06-10 | 驱动只当设备、处理放在 app；Mach 服务白名单加 LaunchDaemon 中转只做控制；app 崩溃后把默认设备改回去 |
| **AudioCap**（process tap 样例） | `resources/AudioCap` · github.com/insidegui/AudioCap | BSD-2-Clause | 2025-08-07，定位是 API 样例 | 怎么建 tap 和聚合设备、要什么权限 |

### 零件（一两行）

| 零件 | 在哪 | 协议 | 版本 / 最近提交 | 能接在哪 · 关键事实 |
|---|---|---|---|---|
| `cpal` | `resources/cpal` | Apache-2.0 | 0.19.0（master；最新正式版 0.18.2，方案用到的 `CallbackInfo` 在 [Unreleased]，`CHANGELOG.md:8-29,87`）/ 2026-10-28 | 采集麦克风、写 BlackHole 或自制设备。底层是 AUHAL；支持 `BufferSize::Fixed`（`src/host/coreaudio/macos/device.rs:904-930`）；采样率一变就报 `StreamInvalidated`，要上层重建（`macos.rs:262-268`）；`DeviceChanged` 只对按默认设备建的流触发（`src/error.rs:14-22`），换默认麦克风要自己监听；loopback 已经用上 process tap 加漂移补偿（`macos/loopback.rs:93-123`）；**没有** workgroup |
| `coreaudio-rs` | `resources/coreaudio-rs` | MIT/Apache | 0.14.2 / 2026-06-18 | 只封装了采样率监听（`src/audio_unit/macos_helpers.rs:707-787`）；设备列表监听、聚合设备要直接调 `objc2-core-audio` |
| `rtrb` | `resources/rtrb` | MIT/Apache | 0.4.1 / 2026-10-25 | 进程内无锁 SPSC 环形缓冲，实时安全；**不能放进跨进程共享内存**（数据区是堆指针，`src/lib.rs:114,159`） |
| `rubato` | `resources/rubato` | MIT/Apache | 5.0.1 / 2026-10-01 | 漂移补偿：`Async` 重采样器运行中可调比率（`src/asynchro.rs:806-830`）；sinc 128 点约 1.3 ms @48k；实时路径只用 `process_into_buffer` |
| `webrtc-audio-processing`（tonarino） | `resources/webrtc-audio-processing` | BSD 风格 | 2.1.0 / 2026-07-16 | AEC3（`webrtc-audio-processing-sys/src/wrapper.cpp:46-58`）；帧长必须 10 ms，否则 panic（`src/lib.rs:209,246`）；`bundled` 要 meson + ninja |
| `audio_thread_priority`（mozilla） | `resources/audio_thread_priority` | MPL-2.0 | 0.38.0 / 2026-09-17 | 用 Mach 时间约束策略提升实时优先级（`src/rt_mach.rs:63-140`）；**不碰 workgroup** |
| Audio Workgroup 的 Rust 绑定 | docs.rs/objc2-core-audio（常量 `kAudioDevicePropertyIOThreadOSWorkgroup`） | — | — | 只有常量；`os_workgroup_join`、`AudioWorkIntervalCreate` **没有 Rust 封装**，要自己写几行 `extern "C"` |

### 补检索查到的（多为网页，没克隆）

| 是什么 | 在哪 | 协议 | 还活着吗 | 判断 |
|---|---|---|---|---|
| **tympan-aspl**（纯 Rust 的 AudioServerPlugIn 框架） | github.com/penta2himajin/tympan-aspl · docs.rs/tympan-aspl | MIT 或 Apache-2.0（两处说法不一，都不传染） | 0.1.0，2026-05 建仓，只发过一版，CI 只验证到「尝试加载」 | 观察项；它关于 macOS 15 签名的说法是线索，要交叉核实 |
| **eqMac** | github.com/bitgapp/eqMac | Apache-2.0 | 公开代码停在 v1.3.2 | 不适合：据网页阅读本质也是设备内部回环，和 BlackHole 一类【传】 |
| Soundflower | github.com/mattingalls/Soundflower | MIT | 已废弃，不支持 Apple Silicon | 不适合 |
| NoiseTorch-ng / werman noise-suppression-for-voice | GitHub | GPL-3.0 | 一个只支持 Linux；一个只做 AU/VST、不注册设备 | 不适合 |
| Krisp / Loopback / NVIDIA Broadcast | 商业 | — | — | 没找到官方技术文档，不登记做法 |

### 官方文档与论坛（逐条结论里引用的）

| 是什么 | URL | 等级 | 还算数吗 |
|---|---|---|---|
| Notarizing macOS software before distribution | developer.apple.com/documentation/security/notarizing-macos-software-before-distribution | 【官】 | 算 |
| macOS 15 起不能按住 Control 绕过 Gatekeeper | developer.apple.com/news/?id=saqachfa（2024-08-06） | 【官】 | 算 |
| DTS 回帖：coreaudiod 不要求插件公证 | developer.apple.com/forums/thread/116601（2019，10.14.5） | 【官·旧】 | **算旧证据**：之后没有更新的官方说法，也没被推翻。旁证：本机宿主 helper 只检查签名是否有效（字符串 "not signed but is being loaded anyway"），带 `disable-library-validation` |
| AudioDriverKit 只支持实体设备，虚拟设备用 HAL 插件 | developer.apple.com/forums/thread/775341 | 【官】（Apple 工程师引用官方文档） | 算 |
| Requesting entitlements for DriverKit development | developer.apple.com/documentation/driverkit/requesting-entitlements-for-driverkit-development | 【官】 | 算 |
| Adding parallel / asynchronous real-time threads to audio workgroups | developer.apple.com/documentation/audiotoolbox/adding-parallel-real-time-threads-to-audio-workgroups · …/adding-asynchronous-real-time-threads-to-audio-workgroups | 【官】 | 算 |
| Capturing system audio with Core Audio taps | developer.apple.com/documentation/coreaudio/capturing-system-audio-with-core-audio-taps | 【官】 | 算 |
| Apple Developer Program 年费 99 美元 | developer.apple.com/programs/whats-included/ | 【官】 | 算 |
| 插件宿主进程叫 `Core-Audio-Driver-Service.helper` | gist.github.com/cyounkins/a4c9ca82684f861f14d5ba055eff10dc（macOS 12.5.1 崩溃报告）· `resources/roc-vad/README.md:679-681`（2023 年日志） | 【传】+【源】 | **算**：本机开机日志也证实了（Apple 插件样本）；平时不在进程表里，有插件要加载时才拉起 |

| macOS 27 SDK 新增：插件 IO 回调标 `CA_REALTIME_API`（编译期不阻塞检查）、`kAudioDevicePropertySuggestedReferenceDevice` | 本机 `MacOSX27.0.sdk` `AudioServerPlugIn.h:1014,1080,1123,1153` · `AudioHardware.h:1006,1031` | 【源】 | 算；26.5 SDK 里没有 |
| macOS 27 release notes：Intel 版 HAL 插件到 28 不再兼容；不加载带隔离属性的 launchd plist | developer.apple.com/documentation/macos-release-notes/macos-27-release-notes | 【官】 | 算 |
| 「macOS 15+ ad-hoc 签名插件被拒」 | tympan-aspl README · github.com/cortexuvula/MacAudio | 【传】 | **有矛盾**：boss-gt-10（github.com/lukasmalkmus/boss-gt-10-driver-macos `docs/VERIFICATION.md:4`）称 macOS 27 上 ad-hoc 能加载，关键是复制到位后再就地签名。只能实测 |
| Homebrew 从 2026-09-01 起禁用过不了 Gatekeeper 的 cask | github.com/Homebrew/brew/issues/20755 | 【传】 | 待用前再核 |

---

⚠️ **读懂 → 自己写，绝不复制粘贴。** GPL（BlackHole、Background Music）只参考、不链接、不复制；MPL（roc-vad、audio_thread_priority）改了它的文件就要开源那个文件；MIT / Apache / BSD（libASPL、AudioCap、Rust 零件）照协议保留声明即可。
