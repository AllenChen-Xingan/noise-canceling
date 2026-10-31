# 009 · 完整性审查：七组卡片漏了什么（gaps）

> 2026-10-30 · rusteco 完整性审查员（AI 草稿，待人过目）。只读源码、GitHub API、crates.io API，没跑 build / 推理。
> 路径前缀：`resources/` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`。
> 证据等级：【源码】亲眼读过的行 ·【官方】URL ·【API】GitHub / crates.io 返回（含用 contents API 读到的远程文件行）·【推断】。
> 本次新克隆：`resources/m-audio-fast-track-ultra-macos-driver`（922 KB）。

## 审查范围与结论

读了 `cards/` 下 arch · dsp · infer · io · rtsafe · speaker · tooling 七份，对照七个模块和 crate 划分（dsp · engine · speaker · io-macos · hal-plugin · control · eval）逐项查。
**已经覆盖、不算缺口的**：HAL 虚拟设备的 Rust 先例（io：voice-strip、tympan-aspl）· 共享内存原声旁路协议（io：voice-strip shm.rs）· 参数平滑（arch：nih-plug Smoother）· 去非正规数（rtsafe：no_denormals、ScopedFtz）· 签名公证（tooling：自写脚本结论）· 时钟漂移（dsp：rubato Slip）· 限幅（dsp：sonora AGC2 思路）· 在线聚类（speaker）。

### 缺口清单（按重要性排序）

| # | 缺口 | 影响 crate | 为什么重要 | 本次 |
|---|---|---|---|---|
| G1 | **HAL 驱动 ↔ 用户进程的「认证通道」**：怎么把共享内存安全交给 coreaudiod 里的驱动（io 存疑 3「发布前必须定」） | hal-plugin · io-macos · 打包 | voice-strip 用 0644 的 POSIX shm，自认只够 spike；没有现成答案就发不了版 | **已补**（找到 GPL-3.0 的 Rust 先例） |
| G2 | **跨进程唤醒**：送音进程 → 推理进程「新帧到了」（io 存疑 1） | engine · io-macos | 决定推理线程是轮询还是被叫醒，直接影响 D31 能耗和 D32 延迟 | **已补** |
| G3 | **实时线程里的日志与遥测**：七份卡只有反例（prism 在 IO 回调里 `format!`+syslog），没有正面方案 | 全部实时 crate · control | 没日志就查不了爆音；用错日志库就违反命门 2 | **已补** |
| G4 | **control 的界面与进程托管**：菜单栏 UI 用什么 · 推理进程谁拉起、崩了谁重启 · 开机自启 | control · 打包 | io 卡只定了控制面走 UnixStream；UI 框架、launchd / SMAppService 全没查 | **已补** |
| G5 | **eval 的客观指标在 Rust 侧怎么算**（PESQ / STOI / DNSMOS） | eval | 「离线与实时同一份代码」只管处理；打分是留 Python 还是进 Rust 没定 | **已补**（短卡） |
| G6 | 默认输入设备切换 · 热插拔 · 睡眠唤醒 · 采样率变更后的重建流程 | io-macos | io 卡只提到 cubeb 有经验（io.md:196），没给方案 | 未补：先读 `resources/m-audio-fast-track-ultra-macos-driver/daemon/src/supervisor.rs:1-6`（它的服务循环就是为插拔、变采样率、睡眠写的），再读 cubeb-coreaudio-rs |
| G7 | 麦克风权限（TCC）：送音进程若是 LaunchAgent / 无界面进程，权限弹窗归谁、`NSMicrophoneUsageDescription` 放哪个 bundle | io-macos · 打包 | 授权不成就采不到声音，而且出错时不报错（只给静音）【推断】 | 未补：候选 `objc2-av-foundation` 0.3.2（Zlib/Apache/MIT）【API】做 `requestAccess`，需小实验 |
| G8 | 音色链里的「可选带宽扩展（Opus 1.6 BBWENet）」接 Rust | dsp | 003 设计第三节列了它，七份卡没碰 | 未补：要先查 libopus 绑定是否暴露这一步；优先级低（可选项） |
| G9 | 配置持久化（serde + toml + 用户目录） | control | 不难，但要定格式与位置 | 未补：标准做法，P4 前定即可 |
| G10 | 同类竞品：`silentmic-rt`（MIT，2026-10-25 建，0 star，README 宣称「低于 3 ms + 三平台虚拟麦 + 零分配」）【API】 | — | 名字与卖点和我们最像 | 未补：README 通篇是宣传口径，像 AI 生成，未克隆未核；只记一笔，别当证据 |

---

## G1 · HAL 驱动 ↔ 用户进程的认证通道

### m-audio-fast-track-ultra-macos-driver（https://github.com/vjustov/m-audio-fast-track-ultra-macos-driver）【深读 · 本次最大发现】

| 项 | 内容 |
|---|---|
| 做什么 | Apple Silicon 上的 M-Audio 声卡用户态驱动：**Rust 守护进程**负责 USB 收发和设备时钟，**C++ HAL 插件**（libASPL）跑在 coreaudiod 里，只在 Core Audio 和守护进程的环形缓冲之间拷样本；**共享内存经 XPC 交给插件**；另有 SwiftUI 控制面板。和我们「送音进程 + 驱动」那一半结构几乎一样【源码】`resources/m-audio-fast-track-ultra-macos-driver/README.md:35-36` |
| 许可证 | **GPL-3.0**【API】；与我们相同，可读可借鉴。守护进程只依赖 libc、libusb1-sys、serde、serde_json，构建期用 cc 编 C 垫片【源码】`daemon/Cargo.toml` |
| 维护状态 | 0.2.0；最近提交 2026-10-26；仓库 2026-10-23 才建，open issues 0，0 star，无 release【API】【源码 git log】。单人新项目，没经过外部检验 |
| 实时安全做法 | ① 共享区是匿名 `mmap(MAP_ANON|MAP_SHARED)`，**不落文件系统、没有名字**，别的进程无法 `shm_open` 来偷看【源码】`daemon/src/shm.rs:113`、`:128-132`；② 布局是 `#[repr(C)]` 全原子字段 + 定长 f32 环，Rust 端和 C 头文件各一份，靠 MAGIC/VERSION 对上【源码】`shm.rs:1-2`、`:6-7`、`:12-58`；③ 插件持有的映射**永不解除**，因为实时线程可能还拿着指针；守护进程重启时插件重连、拿新映射，旧映射直接泄漏（每次重启一份）【源码】`plugin/src/DaemonLink.hpp:41-43`。README 称「播放中重启守护进程也能挺过去」【源码】`README.md:16` |
| 值得学的一处 | **认证通道的完整做法**：守护进程装成 **LaunchDaemon（系统域）但以用户身份运行**，在 plist 里登记 `MachServices`【源码】`packaging/local.ftu.daemon.plist.in`（注释原话 "System domain so coreaudiod can look up the service; runs as the user, not root."）；驱动的 Info.plist 在 `AudioServerPlugIn_MachServices` 里登记同一个名字【源码】`plugin/Info.plist.in:5-8`；插件发 `{"op":"hello"}`，守护进程回一个 `xpc_shmem_create` 包好的共享区【源码】`daemon/src/xpc_shim.c:101-108`，插件用 `xpc_shmem_map` 映射【源码】`plugin/src/DaemonLink.hpp:292`；守护进程只接受 `_coreaudiod`（uid 202）和自己这个用户的连接【源码】`xpc_shim.c:20`、`:141-145`。BackgroundMusic 用的是同一套路（LaunchDaemon + MachServices，驱动端用 `NSXPCConnectionPrivileged`，注释说明 coreaudiod 在全局 bootstrap 域，只能查到全局域的服务）【源码】`resources/BackgroundMusic/BGMDriver/BGMDriver/BGM_XPCHelper.m:45-50`、`resources/BackgroundMusic/BGMApp/BGMXPCHelper/com.bearisdriving.BGM.XPCHelper.plist.template:32-36` |
| 我们怎么用 | **抄思路自己写**：hal-plugin 照这个「hello → 拿 xpc_shmem → 映射后永不解除」写；送音进程（或单独一个小守护进程）注册 Mach 服务。XPC API 要用 block，它的做法是**写一个 C 垫片**由 cc 编进来【源码】`xpc_shim.c:1`；纯 Rust 的备选见下表 |
| 风险 | ① uid 检查比代码签名检查弱：同一用户下的任意进程都能连上拿共享区；发布版应该再加 `xpc_connection_set_peer_code_signing_requirement`（macOS 12+）【推断】；② 它的 HAL 插件是 C++，Rust 写插件 + XPC 客户端仍无先例；③ 「每次重启泄漏一份映射」对我们的推理进程不适用（推理进程不直连驱动），但对送音进程重启有效【推断】 |
| 证据 | 【源码】×12 ·【API】GitHub repo / releases |

### Rust 侧 XPC 绑定（浅读）

| 候选 | 许可证 | 维护 | 有没有 shmem | 判断 |
|---|---|---|---|---|
| C 垫片 + `cc`（照上面那个仓） | 自写 | — | 直接调 `xpc_shmem_create` | **首选**：几十行 C，最透明 |
| `cidre`（Apple 框架绑定合集） | MIT【API】 | 2026-10-30 推送，225 star【API】 | `cidre/src/xpc.rs` 含 `xpc_shmem_create`【API 代码搜索】 | 备选；整库很大（7 MB 仓），只取 xpc 部分要看 feature 切分【推断】 |
| `rama-net-apple-xpc` 0.4.0 | MIT OR Apache-2.0【API】 | rama 2026-10-30 推送【API】 | 绑定里有 `xpc_shmem_create`、对象层有 fd【API 代码搜索】 | 依附于 rama 异步服务框架，对我们太重【推断】 |
| `xpc-sys` 1.0.0（launchk） | MIT【API】 | 最后推送 2025-06-07【API】，**超过 12 个月，命门 3 不过** | 有 `xpc_shmem.rs`【API 代码搜索】 | 只看看 |
| `xpc-connection` 0.2.3 | MIT【API】 | 2020 年停更【API】 | — | 不要 |

---

## G2 · 跨进程唤醒（送音 → 推理）

| 项 | 内容 |
|---|---|
| 做什么 | macOS 14.4 起的公开 API `os_sync_wait_on_address` / `os_sync_wake_by_address_any`，带 `*_SHARED` 标志时可以在**两个进程共享的内存地址**上睡/醒——相当于 Linux 的跨进程 futex |
| 许可证 | 走 `libc` crate（MIT OR Apache-2.0）即可，不需要新依赖 |
| 维护状态 | libc 0.2.156（2024-08-15）加入这组绑定【API】libc CHANGELOG 第 189、193 行（PR #3769）；当前 0.2.191 / 2026-10-16【API crates.io】。函数声明与「Available starting with macOS 14.4」注释【API】`https://github.com/rust-lang/libc/blob/main/src/unix/bsd/apple/mod.rs` 第 4600-4632 行；`*_SHARED = 0x1` 常量第 4247-4250 行；`libc-test/semver/apple.txt` 第 1028-1031、2021-2027 行登记为稳定 API |
| 实时安全做法 | 等的一方（推理线程）可以用 `os_sync_wait_on_address_with_deadline` 带截止期睡，正好配 work interval 的 10 ms 周期；**叫醒的一方每叫一次就是一次系统调用**——放在送音 IO 线程里就和 voice-strip 的 `unpark` 一样，是「零系统调用」的折中（io.md:44 已讨论）。另外两条路：① 送音线程只做原子写、推理线程按周期定时醒来看序号，热路径零系统调用，代价是最坏多等一个周期【推断】；② 私有 API `__ulock_wait(UL_COMPARE_AND_WAIT_SHARED)`，`ulock-sys` 有绑定【API】`thomcc/ulock-sys src/lib.rs` 第 71 行，但 2021 年后没更新，且是私有接口 |
| 值得学的一处 | Apache 的 `bifromq-tenon` 有一份现成的 Rust「门铃」：在 `memmap2` 映射的共享区里拿一个 u32 槽，等用 `os_sync_wait_on_address(…, SHARED)`，叫用 `os_sync_wake_by_address_any(…, SHARED)`，**没人在等时返回 ENOENT 当成功处理**【API】`https://github.com/apache/bifromq-tenon/blob/main/sdk/rust/ipc/src/bell/platform/macos.rs` 第 20、32-33、108-133 行（Apache-2.0，2026-10-19 推送【API】）。**反例**：`atomic-wait` 在 macOS 上走 libc++ 的 `__libcpp_atomic_wait`，那是进程内按地址查表，**不能跨进程**【API】`m-ou-se/atomic-wait src/macos.rs` 第 6-17、19-35 行 |
| 我们怎么用 | **自写**十几行（engine 等、io-macos 叫），照 tenon 的写法；不加依赖。最低系统定在 macOS 14.4 以上（需人确认是否接受）。P3 先做有界小实验：同一负载下比「定时醒」与「被叫醒」的 CPU 占用和尾延迟 |
| 风险 | ① macOS 14.4 以下不可用；② 叫醒在 RT 线程里是系统调用，要实测耗时；③ 推理进程崩了，送音端的 wake 只会返回 ENOENT，不会阻塞——这一点对 D03 有利，但要写测试锁住【推断】 |
| 证据 | 【API】×6（libc、tenon、atomic-wait、ulock-sys、crates.io） |

---

## G3 · 实时线程里的日志与遥测

| 候选 | 许可证 · 维护 | 实时线程里能不能用 | 证据 |
|---|---|---|---|
| `tracing-appender` 的 `non_blocking` 0.2.5 | MIT；2026-04-17 发版，tracing 仓 2026-10-29 推送【API】 | **不能**：格式化在调用线程做，然后每行 `buf.to_vec()` 一次堆分配再 `try_send` 进 crossbeam 有界通道 | 【API】`tokio-rs/tracing tracing-appender/src/non_blocking.rs` 第 239-252 行 |
| `ring-log` 0.2.15 | MIT；2025-11-29 推送【API】 | **不能**：每条日志是 `Box<dyn FnOnce() -> String>`，装箱就分配 | 【API】`FreerGit/ring-log src/lib.rs` 第 15-18 行 |
| `tracing-oslog` 0.3.0 | Zlib；2025-05-29 推送，**刚好超过 12 个月边缘**【API】 | 只适合放在**日志线程**一侧，把事件转给系统 os_log（Console.app 能看） | 【API】 |
| 共享内存里的原子计数器 | 自写 | **能**：IO 回调只做 `fetch_add(Relaxed)` | fast-track 的共享区头部有 20 来个计数器（迟写、短读、下溢、回调卡顿、最大回调间隔…）【源码】`resources/m-audio-fast-track-ultra-macos-driver/daemon/src/shm.rs:35-54`；voice-strip 驱动的 `MISSING` 计数【源码】`resources/voice-strip/crates/vdev-macos-driver/src/plugin.rs:117` |

**推荐（自写，两层）**：
1. **计数器层（热路径唯一允许的东西）**：每个实时线程一组 `AtomicU64`，放进共享内存头（送音↔推理那块），记 xrun、原声旁路帧数、推理超时、最大回调间隔。control 每秒读一次显示。零分配零锁零系统调用。
2. **事件层**：定长 `#[repr(C)] enum Event { code: u16, a: f32, b: u64 }` 之类的 POD，经 `rtrb` 推给日志线程，满了就丢并计数；日志线程再格式化、交给 `tracing`（文件）或 `tracing-oslog`（Console.app）。字符串只在日志线程出现。
3. 调试期这两层之外的任何日志宏都要包在 `assert_no_alloc` 的豁免里（nih-plug 就这么做，见 arch.md:178）。

**风险**：os_log 本身在实时线程里调用是否安全没核到，按「不安全」处理【推断】；事件层的丢弃策略要和计数器对得上，否则排障时会误以为「没出事」。

---

## G4 · control：菜单栏界面与进程托管

### 界面框架（浅读）

| 候选 | 许可证 | 维护 | 判断 |
|---|---|---|---|
| `tray-icon` 0.26.1 + `muda` 0.21.2（Tauri 出品，只做状态栏图标 + 原生菜单） | MIT OR Apache-2.0【API】 | 2026-10-07 / 10-08 发版，仓 2026-10-26 推送，414 star【API】 | **首选**：我们的 control 首版只要开关、强度、状态灯，原生菜单够用；macOS 依赖全是 objc2 家族【API】`tray-icon Cargo.toml` macOS 段。注意文档要求「主线程上事件循环已在跑之后才能建图标」【API】`tray-icon src/lib.rs` 第 21 行 |
| `objc2-app-kit` 0.3.2 直接写 `NSStatusItem` | Zlib OR Apache-2.0 OR MIT【API】 | objc2 仓 2026-10-17 推送【API】 | 备选：少一层，但代码量大 |
| `slint` 1.18.1 | `GPL-3.0-only OR` 免版税 `OR` 商业【API crates.io】 | 2026-09-21 发版【API】 | 要做设置窗口时的首选：GPL-3.0 那一支和我们正好一致；opennc-app 已用它（io.md:123） |
| `eframe`（egui）0.37.0 | MIT OR Apache-2.0【API】 | 2026-10-15【API】 | 备选：即时模式，画电平表方便，但常驻时 GPU/重绘耗电要核【推断】 |
| `tauri` 3.0.0-alpha.5 | Apache-2.0 OR MIT【API】 | 活跃 | 不要：WebView 太重，D31 不划算【推断】 |
| SwiftUI（非 Rust） | — | — | 对照：fast-track 的控制面板就选了 SwiftUI【源码】`resources/m-audio-fast-track-ultra-macos-driver/README.md:17`。若 Rust UI 不顺手，这是退路 |

### 进程托管 · 开机自启 · 崩了重启

| 候选 | 许可证 · 维护 | 判断 |
|---|---|---|
| launchd plist（`KeepAlive` + `RunAtLoad`） | 系统自带 | **首选**：推理进程和送音进程都交给 launchd 拉起、崩了自动重启，D03「推理崩了照样送原声」的另一半（谁把它拉回来）就有了；fast-track 的守护进程就是 `RunAtLoad` + `KeepAlive`【源码】`resources/m-audio-fast-track-ultra-macos-driver/packaging/local.ftu.daemon.plist.in` |
| `objc2-service-management` 0.3.2（`SMAppService`） | Zlib OR Apache-2.0 OR MIT【API】 | **首选**：从 .app 里注册包内的 LaunchAgent / 登录项（macOS 13+，系统设置里能看到能关），不用写 `~/Library/LaunchAgents` |
| `smappservice-rs` 0.1.3 | MIT；**2025-05-27 后没更新，命门 3 不过**【API】 | 只看看：它就是对上一行的薄封装，枚举了 MainApp / Agent / Daemon / LoginItem 四种【API】`gethopp/smappservice-rs src/lib.rs` 第 72-102 行 |
| `servicemanagement-rs` 0.5.2 | MIT OR Apache-2.0；2026-10-26 推送，1 star，56 次下载【API】 | 太新，不要 |
| `launchd` 0.3.0（生成/解析 plist） | MIT；2023-08 停更【API】 | 不要；plist 是静态文件，打包脚本里写死即可 |
| `auto-launch` 0.6.0 | MIT；2026-01 发版【API】 | 不要：跨平台登录项，macOS 上不如 SMAppService 正规【推断】 |

**风险**：HAL 驱动要找的 Mach 服务必须在**系统域**（见 G1），这要 LaunchDaemon（装到 `/Library/LaunchDaemons`，.pkg 安装时要管理员）而不是用户域的 LaunchAgent；SMAppService 也能注册包内的 Daemon，但需用户在系统设置里批准【推断，待查 Apple 文档】。谁持有 Mach 服务、谁持有麦克风权限（G7），要一起定。

---

## G5 · eval 的客观指标在 Rust 侧（短卡）

| 候选 | 许可证 · 维护 | 判断 |
|---|---|---|
| `pesq` 0.2.0（纯 Rust 的 ITU-T P.862 / P.862.2） | MIT OR Apache-2.0；2026-08-19 发版，0 star，413 次下载【API】 | **只看看**。README 自称通过 P.862 Annex A 窄带一致性测试、唯一依赖 rustfft【API】README；宽带模式是「对一个代用语料收敛」的（提交信息原话 "Converge P.862.2 wideband mode against a proxy corpus"）【API commits】。单人新项目，数字可信度要先拿 Python `pesq`（`resources/PESQ`）对拍。另：PESQ 参考实现历来有 ITU 版权与专利条款，一个「纯 Rust 重写」是否从 ITU 源码派生、能否随 GPL 发布，**存疑** |
| `voirs-evaluation` 0.1.0-rc.1 | Apache-2.0；rc 版【API】 | 不要：TTS 框架的一部分，太大 |
| DNSMOS / UTMOS | 无 Rust 现成 | 走 `ort` 跑官方 ONNX 可以做，但没必要 |

**推荐**：eval crate 只负责「同一份处理代码跑 WAV → 出 WAV + 摘要」（tooling 的 insta / approx 方案），**打分继续留在 Python**（resources 里已有 PESQ、pystoi、SpeechMOS、versa、UTMOSv2）。这样指标实现只有一份，也避开 PESQ 许可问题。Rust 侧最多加 `ebur128` 响度（dsp 卡已列）和自写的 SNR / 残留电平这类几行能写完的量。

---

## 没查到 / 存疑

1. **Rust 写的 HAL 插件 + XPC 客户端**：voice-strip（纯 Rust 驱动，但 shm 不走 XPC）和 fast-track（走 XPC，但插件是 C++）各占一半，两者合起来的先例没找到。
2. **`xpc_connection_set_peer_code_signing_requirement` 的实际用法**没查到开源例子；G1 的 uid 检查只够内测。
3. **SMAppService 注册 Daemon 是否满足 coreaudiod 的系统域查找**：没实测，只是推断。
4. **`os_sync_wake_by_address_any` 在 Apple Silicon 上的单次耗时**：没有公开测量，P3 实验要量。
5. `silentmic-rt` 未克隆未核（G10）；若之后要当竞品对照，再按命门三条过一遍。

## 自检

- 只写了本文件；只克隆了一个仓（922 KB，主目录 resources/）；没跑 build / test / 推理。
- 【源码】行号都是本次 `sed`/`grep -n` 亲眼看过的；远程文件（libc、tenon、atomic-wait、ulock-sys、tracing-appender、ring-log、tray-icon、smappservice-rs）通过 GitHub contents API 读取，标【API】并给出行号。
