# r4 · macOS「空闲 + 插电 + 不发热 + 未开会」低优先级任务机制

> 2026-10-30 · 只检索不运行 · 等级：【官方】【论文】【源码】【二手】【推算】
> Apple 文档页是 JS 渲染，文中引文取自同路径的 `developer.apple.com/tutorials/data/documentation/...json`（内容与网页相同）。

## 1. NSBackgroundActivityScheduler

- 可用 macOS 10.10+；原文「gives the system flexibility to determine the most efficient time to execute based on **energy usage, thermal conditions, and CPU use**」；「interfaces with the **XPC Activity API**」，app 不必是 XPC service。【官方】https://developer.apple.com/documentation/foundation/nsbackgroundactivityscheduler
- `qualityOfService` 默认 `NSQualityOfServiceBackground`，调高则「schedules the activity more aggressively」；`tolerance` 默认 interval 的一半，临近宽限期末尾调度更激进；`identifier` 用来累计运行次数、改进启发式；`repeats`；`interval` 适合「10 minutes or more」。【官方】https://developer.apple.com/library/archive/documentation/Performance/Conceptual/power_efficiency_guidelines_osx/SchedulingBackgroundActivity.html
- `shouldDefer`：长任务跑到一半条件变了，官方举例「**the user has unplugged the Mac**」→ 收尾并 `completion(.deferred)`。【官方】同上
- **官方 vs 二手分界**：官方只写了「能耗 · 热 · CPU」加拔电源一例，**没写「用户活跃」**。DAS（Duet Activity Scheduler，进程 `dasd`）按加权策略打分、超阈值才放行，日志里可见 `ThermalPolicy`（权重 5）、`DeviceActivityPolicy`（2 或 20）、`PowerNapPolicy`；`thermalLevel > 0` 判「Absolutely Must Not Proceed」；约每两分钟重算一次；作者原话「Scoring … isn't documented」。【二手】https://eclecticlight.co/2023/01/23/scheduled-activities-1-scheduling-by-das/ · https://eclecticlight.co/2023/01/21/how-macos-schedules-background-activities/
- 预判「DAS 一定会看用户活跃」：官方没找到；只有日志里 `deviceActivity == 1` 拦下过任务（二手），具体测什么没写清。**不能依赖它，自己判空闲**。

## 2. QoS 与 E 核；taskpolicy / setpriority / IO 节流

- 官方原文只说「the system is **more likely** to run background tasks on lower performance cores」，并让开发者用 `pthread_set_qos_class_self_np` 代替 `setpriority` / `pthread_setschedparam`。【官方】https://developer.apple.com/documentation/apple-silicon/tuning-your-code-s-performance-for-apple-silicon
- WWDC20 session 10686：「QoS is **a factor** in determining which core a task will be run on」。【官方】https://developer.apple.com/videos/play/wwdc2020/10686/
- **预判「官方说 background 只上 E 核」——证伪**：WWDC 讲稿与官方文档里都没找到「限定在 E 核」的原话。「QoS 9 只在 E 核跑、不能升到 P 核，E 核满载 P 核闲着也一样」来自实测。【二手】https://eclecticlight.co/2025/05/09/what-is-quality-of-service-and-how-does-it-matter/ · https://eclecticlight.co/2022/01/24/how-you-cant-promote-threads-on-an-m1/
- 题目里的「Tune your Core ML」场次：没找到。
- `taskpolicy`：`-b` 等同 `setpriority(PRIO_DARWIN_BG)`；`-c utility|background|maintenance` 是 QoS 钳位；`-d/-g` 走 `setiopolicy_np(IOPOL_TYPE_DISK)` 设成 throttle。【官方 man】https://keith.github.io/xcode-man-pages/taskpolicy.8.html
- `setpriority(PRIO_DARWIN_THREAD|PROCESS, 0, PRIO_DARWIN_BG)`：调度优先级降到最低，磁盘 IO 节流，进入后台后**新开**的 socket 网络 IO 也节流。【官方 man】https://keith.github.io/xcode-man-pages/getpriority.2.html

## 3. 热 · 电源 · 低电量 · 用户空闲

- `ProcessInfo.thermalState`（macOS 10.10.3+）分四档：nominal「within normal limits」/ fair「slightly elevated」/ serious「high」/ critical「significantly impacting performance … needs to cool down」。通知 `ProcessInfo.thermalStateDidChangeNotification`，**注册前须先读一次 thermalState**。【官方】https://developer.apple.com/documentation/foundation/processinfo/thermalstate-swift.enum · https://developer.apple.com/documentation/foundation/processinfo/thermalstatedidchangenotification
- `IOPSGetProvidingPowerSourceType(IOPSCopyPowerSourcesInfo())` 返回 `kIOPMACPowerKey / kIOPMBatteryPowerKey / kIOPMUPSPowerKey`。`IOPSNotificationCreateRunLoopSource` 在电量百分比或剩余时间变化时都会触发，频繁；头文件建议只关心插拔就用 `IOPSCreateLimitedPowerNotification`，或用 notify(3) 订阅 `kIOPSNotifyPowerSource`（仅在 AC 与电池之间切换时投递）。【源码】https://github.com/apple-oss-distributions/IOKitUser/blob/main/ps.subproj/IOPowerSources.h（约 130–144、306–340 行，行号为抓取时估计）
- `isLowPowerModeEnabled`：**macOS 12.0+**；通知 `NSProcessInfoPowerStateDidChange`；低电量模式会「Pausing discretionary and background activities」。【官方】https://developer.apple.com/documentation/foundation/processinfo/islowpowermodeenabled
- 用户空闲：`CGEventSource.secondsSinceLastEventType(.combinedSessionState, eventType: kCGAnyInputEventType)`（macOS 10.4+，涵盖键盘、鼠标、数位板）。【官方】https://developer.apple.com/documentation/coregraphics/cgeventsource/secondssincelasteventtype(_:eventtype:)
- IOKit `IOHIDSystem` 的 `HIDIdleTime` 属性单位是纳秒；**不是公开文档 API**，有在无用户会话时读到 0 的个例。【二手】https://dssw.co.uk/powermanager/guide/finding-and-fixing/inactivity/ · 【源码】https://docs.rs/crate/user-idle-time/latest/source/src/macos_impl.rs

## 4. 「未开会」怎么判

- `kAudioDevicePropertyDeviceIsRunningSomewhere`（'gone'）：设备在任一进程里运行。官方页只有声明，没有正文。【官方】https://developer.apple.com/documentation/coreaudio/kaudiodevicepropertydeviceisrunningsomewhere。**蓝牙麦恒报 0** 的反馈。【二手】https://developer.apple.com/forums/thread/741026
- 进程级：`kAudioHardwarePropertyProcessObjectList`（'prs#'）→ 每个进程对象读 `kAudioProcessPropertyPID / BundleID / IsRunningInput / Devices`。官方页**没标版本号**。该常量首次出现在 Xcode 15.1 b2 的头文件 diff，即 macOS **14.2** SDK，**不是 14.0**。【二手】https://github.com/dotnet/macios/wiki/CoreAudio-macOS-xcode15.1-b2（未能抓到正文，只见到标题）。另有反馈：macOS 15.0.1 上 `IsRunningInput` 的监听回调收不到，`IsRunning` 能收到。【二手】https://developer.apple.com/forums/profile/existentialaudio
- 自家虚拟麦：AudioServerPlugIn 的 `StartIO` / `StopIO`「Tells the device to start IO」，就是「有人在读我们的麦」最可靠的信号。【官方】https://developer.apple.com/documentation/coreaudio/audioserverplugindriverinterface/startio

## 5. App Nap · BGTaskScheduler · launchd · XPC Activity

- App Nap 的候选条件：不在前台、可见窗口近期没有更新、**不发声**、没持有 IOKit / NSProcessInfo 断言、没在用 OpenGL；进入后会降优先级、节流定时器、节流 IO。【官方】https://developer.apple.com/library/archive/documentation/Performance/Conceptual/power_efficiency_guidelines_osx/AppNap.html。`ProcessInfo.ActivityOptions.latencyCritical`「highest amount of timer and I/O precision」；`.background`「not as the direct result of user request」。【官方】https://developer.apple.com/documentation/foundation/processinfo/activityoptions
- **BGTaskScheduler**：平台列表只有 iOS / iPadOS 13、Mac Catalyst 13.1、tvOS、visionOS，**原生 macOS 不在列**。`BGProcessingTaskRequest.requiresExternalPower` 同样如此。【官方】https://developer.apple.com/documentation/backgroundtasks/bgtaskscheduler
- launchd：`ProcessType=Background`「resource limits … intended to prevent them from disrupting the user experience」；`Adaptive` 按 XPC 活动在 Background 与 Interactive 之间切换；`LowPriorityIO`、`LowPriorityBackgroundIO`、`Nice`。【官方 man】https://keith.github.io/xcode-man-pages/launchd.plist.5.html
- XPC Activity（`xpc_activity_register`，macOS 10.9+）：
  - `XPC_ACTIVITY_ALLOW_BATTERY`：「whether the activity should be allowed to run while the computer is on battery power」，utility 级默认 true，**maintenance 级默认 false**
  - `XPC_ACTIVITY_PRIORITY_MAINTENANCE`：「user-invisible maintenance tasks such as garbage collection or optimization」；`_UTILITY` 用于用户可见的任务
  - `DELAY / GRACE_PERIOD / INTERVAL / REPEATING`
  - `XPC_ACTIVITY_STATE_DEFER / CONTINUE`
  - 出处：【官方】https://developer.apple.com/library/archive/documentation/Performance/Conceptual/power_efficiency_guidelines_osx/CentralizedTaskScheduling.html
  - `XPC_ACTIVITY_REQUIRE_SCREEN_SLEEP`：「performs only while the primary screen is in sleep mode」，默认 false；同组还有 `XPC_ACTIVITY_PREVENT_DEVICE_SLEEP`。【官方】https://developer.apple.com/documentation/xpc/xpc_activity_require_screen_sleep
  - `xpc_activity_should_defer`：判断条件是否仍满足；不满足就 set DEFER，任务回到 WAIT，条件再满足时重新调用 handler。【官方】https://developer.apple.com/documentation/xpc/xpc_activity_should_defer(_:)
  - `CPU_INTENSIVE / REQUIRE_BATTERY_LEVEL` 的官方原文：没找到。

## 6. 组合方案

- **门（全部满足才开跑）**：
  - AC 供电
  - `!isLowPowerModeEnabled`
  - `thermalState == .nominal`
  - 输入空闲 ≥ 10 min（CGEventSource）
  - 自家 IO 未运行（StartIO 计数 = 0）
  - 物理麦 `IsRunningSomewhere == 0`
  - 进程列表里无 `IsRunningInput` 的会议 app
- **调度**：外层用 `NSBackgroundActivityScheduler`（QoS `.background`，interval 6 h，tolerance 3 h）或 XPC Activity（MAINTENANCE + ALLOW_BATTERY=false）托底省电决策；进门后自检上面的门；训练线程设 `QOS_CLASS_BACKGROUND`，并设 `PRIO_DARWIN_BG`（IO 节流）。
- **立即让出**：订阅热状态通知、`kIOPSNotifyPowerSource`、`NSProcessInfoPowerStateDidChange`、StartIO 回调、`IsRunningSomewhere` 属性监听；任一变化就置原子 cancel 标志。训练循环每个小批（≤ 200 ms）查一次标志和 `shouldDefer`，命中后写检查点并 `completion(.deferred)`。

## 对我们的含义

- **回路**：全部放在**学习回路**，独立进程 / 线程。反射回路（音频线程）与它零共享锁，只经判断回路原子换指针；新参数要等影子比对通过才换上。
- **预算量级【推算】**：反射回路最坏 0.62/16 ≈ 3.9% 单核。学习按「前向 + 反向 ≈ 3× 前向」估：1 h 素材 ≈ 3600 × 33M × 3 ≈ 3.6×10¹¹ MAC。若 E 核有效吞吐取 1 GMAC/s（假设，未实测），约 6 分钟；每次只更新调节向量 / 归一化统计 / 小适配器，可以切成 ≤ 5 min 一段的块。
- **护栏**：门里任何一条为假就 0 工作；检查点可续跑；保留新旧两版、一键回滚；设置里可关；全程不联网（DAS 本身不上传东西，我们也不触网）。
- **首选**：`NSBackgroundActivityScheduler` + 自检门 + 事件订阅即时取消。
- **备选**：launchd LaunchAgent（ProcessType=Background、LowPriorityIO）+ XPC Activity（MAINTENANCE、ALLOW_BATTERY=false，可选 REQUIRE_SCREEN_SLEEP）。
- **三个风险**：
  1. DAS 打分不公开，可能连着几天不放行 → 用 tolerance 兜底，记录「上次成功」时间，超期在 UI 提示
  2. 「在开会」判定有漏洞（蓝牙麦恒为 0、进程级属性的监听回调不可靠）→ 以自家 StartIO 为主，轮询为辅
  3. background QoS 钉在 E 核属二手实测，未来芯片或系统可能改变 → 不依赖它，另加 `PRIO_DARWIN_BG` 与自限占空比（例如每跑 1 s 歇 1 s）
