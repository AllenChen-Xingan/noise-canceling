# r6 · 实时音频 / 通话软件公开讲过的事故与可靠性做法

> 检索日 2026-10-30。只联网读，没跑任何东西。等级：【官方】【论文】【源码】【二手】【推算】。行号是 WebFetch 估的「约」值，引用前请再核对。

**先亮的预判和证伪结果**：原以为 Krisp、Zoom、Discord 会有工程复盘长文。**证伪了**：这三家都只有发版说明和社区帖，**没找到**事故复盘。真正能拿来抄作业的工程做法几乎都在 Chromium/WebRTC 源码，加上 Apple 的 WWDC 和开发者论坛。

## 1. 逐家核查

**1.1 Krisp**
- Mac 1.44.1（2023-02-24）修了：长时间不用后「卡死同时 CPU 很高」；悬浮窗卡住，「不再把音频流传给通话软件」。还写了「CPU 高负载时降噪质量更好」。https://krisp.launchnotes.io/announcements/ann_BeP8BhHunS4CW 【官方】
- Enterprise 23.Q1（2023-03-13）修了：通话中切换系统默认设备会重启音频流；拔掉正在用的物理设备会断音；唤醒后马上开会会卡住。https://krisp.launchnotes.io/announcements/ann_FOdwanTsBSQRv 【官方】
- 后台闲置时「CPU 占用低于 2–3%」（2022-08 更新）。https://krisp.ai/blog/?p=8685 【官方】
- 根因和之后加了什么防线：**没找到**。
- 2026-04 有论坛帖说 Discord 设置里没有 Krisp 选项了。https://support.discord.com/hc/nl/community/posts/4410828655383 【二手】

**1.2 Zoom**
- ZoomAudioDevice 是放在 `/Library/Audio/Plug-Ins/HAL/` 的 `.driver`。旧版的 `.kext` 已经弃用。https://devforum.zoom.us/t/how-to-install-the-zoomaudiodevice-driver/51460 【二手·论坛】
- 共享声音失灵时，社区的解法是去 Audio MIDI 设置里确认设备还在、重新授权、重装驱动。https://community.zoom.com/t5/Meetings/I-cannot-share-computer-audio/m-p/44546 【二手】
- 官方事故复盘：**没找到**。

**1.3 Discord**
- 桌面端用「一个建在 WebRTC native 上的 C++ 媒体引擎」，自己做音量控制，为的是不改系统音量。客户端会把通话质量指标上报（2018-09-10）。https://discord.com/blog/how-discord-handles-two-and-half-million-concurrent-voice-users-using-webrtc 【官方】
- Krisp 在本机跑，不往服务器发数据。https://discord.com/blog/tune-into-discord-with-krisp-noise-suppression-on-ios-and-android 【官方】
- 「Experimental」音频子系统已经并进 Standard，另外还留着一个 Legacy 子系统当兜底。https://support.discord.com/hc/ko/community/posts/360056487032 【二手·员工回帖】
- 语音引擎重写的博文：**没找到**。

**1.4 游戏语音**
- Steam：2018 年把语音改用 WebRTC 和 Opus。https://www.ghacks.net/?p=145131 【二手】。后来接入了 RNNoise，默认开启，可以关。https://steamdb.info/patchnotes/6075862 【二手·补丁镜像】
- Vivox / Riot：工程博文**没找到**，只有「VALORANT 用 Vivox」这类案例稿。https://unity.com/en/resources/riot-games-adds-vivox-voice-comms-to-its-latest-title 【二手】

**1.5 Chrome**
- 音频服务在 Mac 上是独立进程，带沙箱；在 Linux 上是独立进程，不带沙箱。https://chromium.googlesource.com/chromium/src/+/main/services/audio/README.md 【源码】
- **看门狗**：`media/audio/audio_thread_hang_monitor.cc` 约 L34 写 `kMaxFailedPingsCount = 3`，约 L37 写 `kDefaultHangDeadline = 3 分钟`，所以每分钟 ping 一次。连续三次 ping 不通就判定挂死，处理方式可选 `kDump` / `kDumpAndTerminateCurrentProcess` / `kTerminateCurrentProcess`，结果记进直方图 `Media.AudioThreadStatus`。https://chromium.googlesource.com/chromium/src/+/main/media/audio/audio_thread_hang_monitor.cc 【源码】
- 起因：2015 年「现场报告音频线程会挂很久，甚至无限期」。https://codereview.chromium.org/1036703002 【源码·评审】
- 之后加了开关 `AudioServiceOutOfProcessKillAtHang`：挂死就杀掉音频服务，需要时再重启。默认在 Win/Mac 打开（M78 时），后来扩到 Linux/ChromeOS。https://chromium.googlesource.com/chromium/src/+/refs/tags/78.0.3896.1/media/audio/audio_features.cc 【源码】
- **超时补静音**：`services/audio/sync_reader.cc` 规定最长等待时间是缓冲时长的 0.95 倍，**Mac 和 ChromeOS 上是 0.5 倍**。等超时就 `dest->Zero()`，记一次 glitch，日志最多打 100 条。数据没就绪的时间记进 `Media.AudioOutputControllerDataNotReady`。https://chromium.googlesource.com/chromium/src/+/main/services/audio/sync_reader.cc 【源码】
- 旧版本另有 `Media.AudioRendererAudioGlitches2.*` 和 `MissedDeadline2.Short/Long`。https://chromium.googlesource.com/chromium/src/+/0df5b7204c88f7823f8319f3af41ff59889e5aec/services/audio/sync_reader.cc 【源码】
- 至于外部流传的「audio glitch」UMA 统计，公开数字**没找到**。
- **睡眠与唤醒**：`audio_manager_mac.cc` 里有 `AudioPowerObserver::ShouldDeferStreamStart()`，注释是「正在睡眠或刚开始唤醒时，Start 要推迟」。https://chromium.googlesource.com/chromium/src/+/main/media/audio/mac/audio_manager_mac.cc 【源码】
- **热插拔与采样率**：`HandleDeviceChanges()` 只在「默认输出设备或它的采样率真的变了」时才通知各条流重建，其余情况不动。https://chromium.googlesource.com/chromium/src/+/97785eaf0f3b8f017a38fe738ade818754492483/media/audio/mac/audio_manager_mac.h 【源码】

**1.6 WebRTC**
- **denormal（非规格化数）事故**：输入「从非零突然变成全零」时 APM 内部会产生 denormal，相关指令「最慢可到 100 倍」。2021-08-10 的修法是在 APM 每个入口上开硬件层的 FTZ/DAZ（x86 MXCSR、ARM FPSCR），并配了一个 `WebRTC-ApmDenormalDisablerKillSwitch` 开关。问题单是 crbug 1227566。https://webrtc.googlesource.com/src/+/0441bb625fb5c53db26d144cee07291510c742dd%5E%21/ 【源码】
- 稳定以后删掉了这个开关，改成无条件开启。https://webrtc.googlesource.com/src/+/54cf754dac12776455a7aa850f4735ab6e98c5b6 【源码】
- **NaN 事故**：旧 AEC 的相干值理论上在 [0,1]，实际会越界，变负的误差幅度取幂后得到 NaN，一路传到输出。这个 bug 是 AudioProcessing fuzzer 发现的（chromium:804634，2018-01）。修法是把误差幅度截在 0，原因是 MIPS / NEON / SSE 三条路径算出来的值不一致。https://webrtc.googlesource.com/src/+/e994058eb1397ad18523067408ebeb9117e25a25 【源码】
- **灰度与回滚**：AEC3 的新行为都带 `…KillSwitch` 字段试验。例如「减少啸叫抑制」先合入，又回退、重新合入、再回退。https://webrtc.googlesource.com/src/+/dba4db5668d6dd95475dc4e8aa42de3011b4a15e%5E%21 【源码】
- 新功能稳定后统一清理开关，并且做比特级一致（bit-exact）测试。https://webrtc.googlesource.com/src.git/+/68d6d441976db43d5bf44a8f1bdaf218edd78380 【源码】
- 还专门 fuzz 这些开关。https://webrtc.googlesource.com/src/+/790da37b72f96feb8d5309b4485ef3df54ffa2d1%5E%21/ 【源码】
- 规范：字段试验「必须有结束日期」，「超过 12 个月应当罕见」。https://webrtc.googlesource.com/src/+/main/g3doc/field-trials.md 【官方】

**1.7 Apple / macOS**
- 虚拟设备的正路是 AudioServerPlugIn，运行在 coreaudiod 里，有沙箱，只能读自己的 bundle。https://developer.apple.com/library/mac/qa/qa1811/_index.html 【官方】
- 官方示例要求 macOS 12+，装进 `/Library/Audio/Plug-Ins/HAL`。https://developer.apple.com/documentation/coreaudio/creating-an-audio-server-driver-plug-in 【官方】
- 文件带 quarantine 属性时 coreaudiod 会静默拒绝加载。https://docs.rs/tympan-aspl 【二手】
- WWDC20「Meet Audio Workgroups」：错过截止就会 overload，用户听到的是断续；「所有音频线程都应当加入 workgroup」；性能控制器会用 workgroup 的信息给音频线程分配核和频率。https://developer.apple.com/videos/play/wwdc2020/10224/ 【官方】
- **大版本升级后失效的案例**
  - macOS 14.4 开着 SIP 时 `launchctl kickstart coreaudiod` 报 `Operation not permitted`，装驱动后只能要求用户重启；有用户报告 `killall` 还能用。Apple 工程师没有回帖。https://developer.apple.com/forums/thread/748228 【二手·开发者论坛】
  - macOS 15 上把虚拟设备当输入会报「Audio Output Device Stopped」，BlackHole 也一样。https://developer.apple.com/forums/thread/772098 【二手】
  - macOS 11 测试版弄坏了 Rogue Amoeba 的 ACE。他们改用 ARK，到 macOS 14.4/14.5+ 做到不装扩展、不用重启；同时 14.4 的 System Audio 有断音问题，只能按系统版本拆成两个产品版本。https://weblog.rogueamoeba.com/2024/12/ 【官方·厂商博客】
  - Teams 通过 MAU 静默更新替换了 HAL 插件，但没重启 coreaudiod，结果 mediaremoted 单核 100%。只重启 coreaudiod 就好了（macOS 15.7.7，2026-08）。https://techcommunity.microsoft.com/discussions/microsoftteams/teams-hal-driver-update-skips-coreaudiod-restart-mediaremoted-spins-at-100-cpu/4545300 【二手·用户分析，微软未确认】

## 2. 事故表（发生了什么 → 根因 → 修法 → 新防线）

| 事故 | 根因 | 修法 | 新防线 |
|---|---|---|---|
| Krisp 闲置后卡死、CPU 很高、不出声 | 没公开 | 发版修 | 没公开 |
| Krisp 切换或拔掉设备断音 | 设备变化时流重启 | 不再重启 | 没公开 |
| APM 遇到 denormal 慢到 100 倍 | 输入从非零变全零 | 开硬件 FTZ/DAZ | 先带开关，稳定后转常开 |
| 旧 AEC 输出 NaN | 数值越界后取幂 | 截到 0 | fuzzer 发现，此后长期 fuzz |
| Chrome 音频线程挂死 | 现场原因不明 | 看门狗加转储 | 杀进程后重启（独立进程才做得到） |
| Teams 插件热替换后 CPU 打满 | 没重启 coreaudiod | 重启 coreaudiod | 没公开 |

## 3. 共性做法
- **送原声 / 降级**：Chrome 超时就补零，宁可静音一帧也不阻塞。Discord 留着 Legacy 子系统；Steam 的 RNNoise 可以关。【源码】【二手】
- **看门狗**：3 次 × 1 分钟，然后杀掉进程等待重启。【源码】
- **热插拔与采样率**：只在默认设备或采样率真的变了才重建，避免没必要的重启（Krisp 吃过这个亏）。【源码】【官方】
- **睡眠与唤醒**：唤醒初期推迟 Start。【源码】
- **灰度与回滚**：用 KillSwitch 字段试验，必须有结束日期，配 fuzz 和比特级一致测试。【源码】【官方】
- **denormal**：每个处理入口开硬件 FTZ/DAZ。【源码】
- **NaN 自愈**：公开材料只看到「截断加 fuzz」，「检测到 NaN 就重置状态」的做法**没找到**公开实现。

## 对我们的含义

**放哪条回路**
- 反射回路：每个入口开 FTZ/DAZ；每帧检查 NaN/Inf 和能量是否越界，越界就送原声、清零 GRU 状态；输出等不到就送原声，不送静音，因为我们是麦克风。
- 判断回路：设备、采样率、睡眠事件的去抖与重建；唤醒后推迟 Start；看门狗。
- 学习回路：用开关和影子比对灰度；新参数带结束日期。

**预算量级**【推算】
- NaN 检查：每帧 160 个样本，比较约 160 次，相对 33 MMACs/s 约 33 万次/帧，不到 0.1%。
- 超时阈值仿照 Chrome 在 Mac 上用 0.5 倍缓冲：16 ms 帧对应 8 ms，我们实测最坏 0.62 ms，余量约 13 倍。
- 看门狗每 1 秒 ping 一次，3 次不通就送原声并重建；Chrome 那 3 分钟是对整个浏览器而言的，对我们太长。

**护栏**
- 装驱动和升级只在没在开会时做，并且负责重启 coreaudiod，避免重演 Teams 的事故。
- 每个大版本 macOS beta 都要跑一遍虚拟设备回归。
- 去掉 quarantine 属性，检查 HAL 目录的权限。

**首选**：音频线程照 Chrome 的写法：超时送原声、FTZ、NaN 就重置，外加 KillSwitch 和本地看门狗。
**备选**：把降噪放进 AudioServerPlugIn 外的独立 helper 进程，崩了插件照样直通原声，代价是多一次进程间通信（IPC）。

**三个风险**
1. macOS 大版本改了 coreaudiod 的权限或行为，例如 14.4 的 kickstart、15 的虚拟输入报错。
2. 静默更新替换插件后，旧引用一直不收敛。
3. 设备抖动时误判成「变了」，反复重建，造成断音。
