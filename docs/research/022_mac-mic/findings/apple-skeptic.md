> workflow `wf_16dd3e65-ebc` · apple 路怀疑派原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

# 怀疑派复核 · 第①路「苹果官方」报告

报告引用的主干原文大多核得上：规格页原句、SDK 头文件、Asahi 文档、平台安全指南都对得上。有 4 处要改，其中 2 处影响后续怎么模拟：

1. **AirPods 那条「冲突」不成立。** 报告引的 iOS 选项，属于一个整套 macOS 都不提供的接口（`AVAudioSession`）。所以它在 macOS 头文件里标不可用，说明不了 Mac 上有没有这项功能。
2. **Asahi 的证据只覆盖 M1、M2 两代。** 本机是 Mac17,3（011 卡记为 MacBook Air，芯片代数这次没核）。三麦几何、48 kHz、无前置放大这几条，不能直接套到 M3 及以后的机器上。
3. **「SEP（安全隔区）在麦克风数据线上」不是每台都这样。** 原文前面有「On certain machines」（某些机型上）的限定，报告和处理阶段图都写成了所有机器。
4. **「HAL 只给 1 声道」依据不足。** 011 卡的本机查询只看到当前格式，没列出设备能提供的全部格式。

核查方式：
- 本地已下载的 64 份 support.apple.com 页面，逐份重新筛出原句，并核对页面标题对应的机型。
- 本机 CommandLineTools 自带的 macOS SDK 头文件，逐行核对。
- 主仓库里的源码：主仓 `resources/`。另外核了项目实际编译用的 crates.io 版 cpal 0.18.2 和 coreaudio-rs 0.14.2。
- 用 WebFetch 打开了 Asahi、Triforce、苹果帮助页、新闻稿、论坛原帖；用 `gh api` 查了 asahi-audio 的提交记录。
- 临时文件在 本会话临时目录 `scratchpad/skeptic/`（不入库）。

## 逐条判定

| # | 结论 | 判定 | 理由与核到的原文 | 修正建议 |
|---|---|---|---|---|
| 1 | Air 2018、2019 款写「Three microphones」；2017 款写「Dual microphones」 | 成立 | 111933、111948 是「Three microphones」；111924 是「Dual microphones」，页面标题都对得上机型 | 无 |
| 2 | Intel 2020 款 Air 和 M1 至 M5 款 Air 写「Three-mic array with directional beamforming」；M3 起多了模式句；M5 那句更长 | 成立 | 111991、111883、111867、111346、118551、118552、122209、126320 原句一致。现行 apple.com 规格页 13 寸和 15 寸都是 M5，同一句 | 15 寸 M4 Air 规格页没下载，表里缺这一行，应补上 |
| 3 | Air 规格页从没写过「studio-quality」或「high signal-to-noise ratio」 | 成立，但样本有缺口 | 已下载的 Air 页面和 apple.com 现行页都没有这两个词。但 15 寸 M4 Air 页没查 | 写成「已查的 10 份 Air 规格页里没有」 |
| 4 | 「studio-quality」最早出现在 MBP 16 寸 2019 款；MBP 13 寸 2020 款（Intel）没有 | 成立 | 111932 有「Studio‑quality three-mic array with high signal-to-noise ratio…」；111339、111981 只有「Three-mic array with directional b…」 | 「最早」只在已查样本内成立。13 寸 MBP 2017、2019 款没查 |
| 5 | MBP 16 寸 2019 新闻稿写「40 percent reduction in hiss」，信噪比「rivals… professional-grade digital microphones」 | 成立 | 原句核到。新闻稿里没有「studio-quality」这个词，这个词只在规格页上 | 无 |
| 6 | M2 Air 新闻稿写「advanced beamforming algorithms」 | 成立，但漏了一句 | 原文是「A three-mic array captures clean audio using advanced beamforming algorithms」。同一篇还写了「the speakers and mics are completely integrated between the keyboard and display」，这是麦克风位置的官方说法，对模拟很关键，报告漏了 | 补上「麦在键盘和屏幕之间（转轴区）」这条 |
| 7 | Asahi：三支 PDM 麦接到 AOP 里的 ADC 和抽取器，三路送到用户态，没有前置放大 | 成立，但机型要限定 | Asahi 文档原句核到。但文档列出的支持机型只有 M1、M2 两代 | 写明「仅 M1、M2 机型」；对 M3 至 M5 Air 只能算推断 |
| 8 | 不做波束成形就「too sensitive and omnidirectional」 | 存疑（出处写错） | Asahi 文档原文是「very sensitive and omnidirectional」；「far too sensitive and omnidirectional」出自 Triforce 的 README | 引文和出处对齐 |
| 9 | 每路 48 kHz，麦间距约 2 cm，延迟求和不够用 | 成立 | 原文是「mics are only about 2cm apart and only sample at 48 kHz」，Asahi 2025-03-21 进展报告。原文没说这个间距适用于哪些机型 | 注明「Asahi 作者的约数，没限定机型」 |
| 10 | SEP 在麦克风数据线路上，没有它的许可就拿不到麦 | 存疑（以偏概全） | 原文是「On certain machines, the Secure Enclave is in the path of the physical mic data lines」 | 处理阶段图里的「SEP 门控」要标「部分机型」 |
| 11 | macOS 给第三方的 1 路已在系统内部合成、波束成形过（推断） | 存疑，推断方向有证据，前提不严 | 支持的证据：Triforce README 原文「Apple… implement an adaptive beamformer in userspace」。但 011 卡的「1 声道」来自 `system_profiler`，它只显示当前格式（011 卡自己也这样注明），没排除有多声道格式可选 | 补一次只读查询：列出设备能提供的全部输入格式，再下结论 |
| 12 | 「环境降噪」在 Apple 芯片和 T2 机型上不出现；macOS 26 帮助页里已没有 | 成立 | 15.0 版原句：「…does not appear if a 4-channel built-in microphone format is selected…, on a Mac with Apple silicon or on Mac models with an Apple T2 chip」。26 版页面没有这一项 | 无。断句歧义报告已注明 |
| 13 | 三种麦克风模式**只**作用于开了苹果语音处理（VPIO）的 app | 存疑（话说过头） | WWDC23 原话只是「Choosing Apple's voice processing APIs also grants users full control over the mic mode settings」，没说用别的接口就不行。模式的读取接口在摄像采集那套接口的 `AVCaptureDevice.h:2711-2746`，只用采集接口的 app 受不受模式影响，没有原文。帮助页只说「options vary by app」 | 改成「已知开了语音处理的 app 支持；其他 app 未知」 |
| 14 | 「标准」是「standard voice DSP」；「宽频谱」是「processing minimized」；语音突显和宽频谱优先于 Bypass | 成立 | `AVCaptureDevice.h:2715-2720`；`AudioUnitProperties.h:2623-2629`（报告写 2624-2630，差 1 行） | 行号改一下 |
| 15 | 语音处理里的自动增益默认开 | 成立 | `AudioUnitProperties.h:2632-2634`「On by default」；`AVAudioIONode.h:230-235`「Enabled by default」 | 无 |
| 16 | 开语音处理要输入、输出两端一起开；离线渲染模式下不能开 | 成立 | `AVAudioIONode.h:154-156` 原句核到 | 无 |
| 17 | 模式由用户在控制中心选，app 只能读 | 成立 | `AVCaptureDevice.h:2731-2736`。另有 `activeMicrophoneMode`（2744 行），可能和用户选的不同 | 补上「当前生效的模式可能与用户选的不同」 |
| 18 | 麦克风模式要 2018 款及以后的 Mac | 成立，但版本旧 | 只有 macOS 12.0 版帮助页写了这句；现行版不写机型 | 标明是 macOS 12 时的口径 |
| 19 | HAL 有「有人在说话」检测，作用在回声消除后的输入上 | 成立，引文出处要拆开 | 头文件 `AudioHardware.h:995-1009` 原文是「can be used with input audio and has echo cancellation」，还有一个指定回声参考设备的属性（`kAudioDevicePropertySuggestedReferenceDevice`）。「detecting voice activity from echo-cancelled microphone input」出自 WWDC 讲稿，头文件里没有这句 | 两处引文分别标出处 |
| 20 | 别的进程开了 VPIO 时，不经 VPIO 读到的电平很低；VPIO 下混单声道；Studio Display 进黑名单 | 成立 | cubeb 源码 `mod.rs:66`、`:3561`、`:3625-3626`、`:3881`、`:3921` 逐行核到（提交 076ecf6）。来源是 Mozilla 工程经验，没写机型和样本量 | 无 |
| 21 | cpal 输入走 HalOutput，行号纠正为 `device.rs:84` 和 `macos_helpers.rs:110-113` | 成立，但行号对应的版本不对 | 84 行只在 `resources/` 的 cpal git 最新版（4bedd4e）成立。项目 Cargo.lock 编译的是 crates.io 的 cpal 0.18.2，那里是 `device.rs:237`；coreaudio-rs 0.14.2 是 `macos_helpers.rs:111-113`。另外 `resources/` 只在主仓库里有，本 worktree 没有 | 两个版本的行号都写上，并标明 Cargo.lock 的版本 |
| 22 | 内置麦有一个总音量，加一个不能单独调的 1 声道音量 | 成立，但标签过高、样本有限 | 回答者在论坛标为「Apple Staff, Apple Designer」，署名 Michael。提问机器是 MacBook Pro | 标签从【官方文档】改为「Apple 员工论坛回答，样本是 1 台 MBP」 |
| 23 | 「4 声道内置麦格式」：两位用户找不到 | 成立，但样本弱 | 发帖人的原句「with all 4 channels merged」是他自己的猜测，而 iMac 24 寸只有 3 支麦。两人都只是没找到选项 | 降级为「没人证实有，也没人证实没有」 |
| 24 | macOS 27.2 上两只蓝牙麦都是 16 kHz 单声道，走 HFP/mSBC | 存疑 | 帖子里写「macOS 27.2; sw_vers reports build 26B123」，版本号和构建号对不上。设备是 Beats Pill（音箱）和 SANWA 蓝牙麦，不是耳机，也不是 AirPods。0 条回复 | 注明版本号自相矛盾、设备不是耳机 |
| 25 | AirPods：iOS 26 的「高质量蓝牙录音」在 macOS 头文件里不可用，和新闻稿「work across iPhone, iPad, and Mac」冲突 | **推翻（冲突不成立）** | 这个选项属于 `AVAudioSession`，而这整套接口在 macOS 上都不提供（`AVAudioSession.h:31` 写着 `API_UNAVAILABLE(macos)`）。所以它不可用说明不了 Mac 上有没有这项功能。新闻稿写的是随 iOS 26、iPadOS 26、macOS Tahoe 26 的固件更新一起推出 | 删掉「冲突」；改成「Mac 上走哪条路、用什么格式：未查到，待实测」 |
| 26 | 合盖时硬件断开麦克风，延迟最多 500 ms，例如切断时钟线 | 成立 | 原文核到：「delay of up to 500 ms」「the clock line or a similar effective control」。13 寸 MBP 和所有带 T2 的 Air 都在列，所以 2018 款 Air 也包括在内 | 无 |
| 27 | Asahi 为 Air M1（J313）和 Air M2（J413）配的参数：三麦呈边长 2 cm 的等边三角形；水平角 120° 和 80°；增益 ×2、×7.25、×7.25；120 Hz 高通；默认音量 0.343 | 成立（数字全对），但含义要说清 | 三个配置文件下载核对，数字一致。MBP 13 寸（J293）的水平角也是 80°。×7.25 两级是为了绕开 PipeWire 内置混音器增益上不去的问题（提交 687d8cd0f6 的说明），是软件调参，不代表麦克风灵敏度。坐标是 Asahi 填的，不是苹果给的实测几何 | 注明「几何是 Asahi 估的，增益是绕开软件限制的调参」。提交日期：作者日期 2026-07-21，提交日期 2026-08-06 |
| 28 | 苹果官方没有「专门的麦克风 DSP」的说法；Intel T2 机型由 T2 集成音频控制器（个人说法） | 成立 | 安全指南原文核到，只讲断开机制，没讲在哪里处理。T2 那条报告自己已标为第三方说法 | 无 |
| 29 | M1 Air 发布新闻稿没提麦克风 | 成立 | 2020-11 那篇新闻稿里，「studio-quality mics」说的是 13 寸 MBP；Air 那段只讲摄像头的图像处理器（ISP） | 无 |
| 30 | iPhone 当 Mac 麦克风：帮助页写「a Mac that doesn't have a built-in or external microphone」 | 成立 | 原文核到。但这句只是应用场景举例，没说带内置麦的 Mac 不能选 | 无 |

## 报告漏查的重要方向

1. **苹果自带的离线人声分离组件（`AUSoundIsolation`）。** 头文件里有（`AUComponent.h:491,508`，`AudioUnitParameters.h:706-727`，macOS 13 起；「高质量」模型 macOS 15 起），有干湿比参数。它很可能就是「语音突显」背后的模型，可以离线跑，用来模拟、对照「语音突显」开着时用户听到的声音（这是推断）。报告完全没提。
2. **语音处理的「压低其他声音」参数。** 有默认、最小、中、最大四档，还有一个随说话自动调节的开关，默认关（`AudioUnitProperties.h:2683-2689`，WWDC23 原话）。会影响会议 app 里的听感。
3. **麦克风的物理位置。** M2 Air 新闻稿写麦在「键盘和屏幕之间」。苹果维修手册里应有麦克风部件和位置：已下载的 100586、100603、104029、119579、122003 几页只看到「Microphone issues」这一条目，正文没展开。腔体、开孔、离嘴距离是模拟的关键参数。
4. **把 M3 至 M5 和 M1、M2 分开写。** Asahi 只支持 M1、M2，这几代的麦克风、AOP、SEP 可能已经变了，报告里所有 Asahi 证据都要标机型边界。
5. **本机只读查询设备能提供的全部输入格式。** 这是在 011 卡「1 声道」那次查询之外补一次（不动设置，不开麦），用来判断第 11 条推断和「4 声道格式」这一条。
6. **HAL 路径上「随麦克风模式变化」的反例。** 报告只写了「没有原文」，没搜社区或开源项目的实测，例如 Chromium、WebRTC、OBS 在 macOS 上的相关讨论和 issue。
7. **系统设置里的输入音量是模拟增益还是数字缩放。** 可以从 HAL 的音量属性判断：看有没有 dB 范围、是不是软件音量（`kAudioDevicePropertyVolumeDecibelsToScalar` 等），比找苹果原文更可行。
8. **专利。** 苹果关于笔记本麦克风阵列、波束成形、风噪的专利可能给出几何和处理链的旁证，报告没查。