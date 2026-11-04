> workflow `wf_16dd3e65-ebc` · apple 路调研员原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

# 第①路 · 苹果官方：Mac 内置麦录出来是什么样

## 要点

1. **MacBook Air 有三支麦，各代写法不同。**
   - Intel 2018、2019 款规格页写「Three microphones」。
   - Intel 2020 款起改写「Three-mic array with directional beamforming」。M1 / M2 / M3 / M4 / M5 款都是这句。
   - Air 规格页从没写过「studio-quality」或「high signal-to-noise ratio」。这两个词只出现在 MacBook Pro（从 Intel 16 寸 2019 起）、iMac 24 寸和 Studio Display 上。【官方文档】
   - Air 规格页从 M3 款起多了一句「Voice Isolation and Wide Spectrum microphone modes」。
2. **苹果说波束成形是「算法」**：M2 Air 新闻稿原文是「advanced beamforming algorithms」。但官方没有一句话说明它在哪一层做（芯片、驱动还是 coreaudiod）。【官方文档】
3. **Linux 上能拿到三路原始信号，macOS 上只给一路。** Asahi 文档写明：三支 PDM 麦接到 AOP 里的 ADC 和抽取器，三路原样送到用户态，没有前置放大，每路 48 kHz。不做波束成形就「too sensitive and omnidirectional」。所以 macOS 给第三方的那 1 路，【推断】已在系统内部合成、波束成形过。
4. **苹果官方没写 HAL 路径上还做不做高通、降噪、自动增益。**
   - 唯一一条 HAL 层的处理开关是「Use ambient noise reduction」（环境降噪）。官方帮助页说它在 Apple 芯片和 T2 机型上不出现，macOS 26 起帮助页里已没有这一项。
   - 所以在 Apple 芯片 Air 上，用户没有设备级的降噪开关。【官方文档 + 推断】
5. **三种麦克风模式只作用于开了 Apple 语音处理（VPIO）的 app。**
   - 头文件写：「标准」是「standard voice DSP」，「宽频谱」是「processing minimized」。
   - 这说明「标准」模式本身就有处理，但这是 VPIO 路径上的事，不是 HAL。
   - 头文件还写：语音突显和宽频谱优先于 Bypass（VPIO 的「跳过全部处理」开关）。【官方文档】
6. **自动增益只在 VPIO 里有官方开关**：`VoiceProcessingEnableAGC`，默认开。HAL 路径没有自动增益开关。有用户报告输入音量滑块会自己动，【论坛 / 博客，个人说法】，原因官方未说明。
7. **AirPods 在 Mac 上当麦**
   - 官方只说「两种模式」，开麦后音质降低；没给采样率和编解码。
   - iOS 26 的「高质量蓝牙录音」接口在 macOS 头文件里标为不可用。
   - 但苹果新闻稿说「studio-quality audio recording … work across iPhone, iPad, and Mac」。两者表面冲突，见文末待查清单。
8. **Apple 芯片里有没有专门的音频 DSP 做波束成形：官方材料里未查到。** 能确认的只有两条旁证：
   - Asahi：ADC 和抽取器在 AOP（常开协处理器）里，SEP（安全隔区）在麦克风数据线路上做门控。
   - 苹果平台安全指南：合盖时用硬件逻辑断开麦克风。
9. **纠错**：011 卡第 #20 条把 cpal 的取音位置写成 `device.rs:67,82-84`。实际输入走 `device.rs:84`，再进 `coreaudio-rs/src/audio_unit/macos_helpers.rs:110-113`，那里选的是 `IOType::HalOutput`。结论不变：cpal 不经 VPIO。

---

## 一、机型 × 麦克风规格（support.apple.com 技术规格原句）

下表用来查每款机器官方怎么描述麦克风，以及措辞从哪一代开始变。均为【官方文档】，每行样本量 = 1 份官方页面。抓取方式：先用 `curl` 下载 `support.apple.com/en-us/<编号>`，再用 grep 筛出原句。

### MacBook Air（单列在前）

| 机型 | 芯片 | 麦数 | 原文 | URL |
|---|---|---|---|---|
| Air 13" 2017（对照） | Intel，无 T2 | 2 | 「Dual microphones」 | https://support.apple.com/en-us/111924 |
| Air (Retina, 13", 2018) | Intel + T2 | 3 | 「Three microphones」 | https://support.apple.com/en-us/111933 |
| Air (Retina, 13", 2019) | Intel + T2 | 3 | 「Three microphones」 | https://support.apple.com/en-us/111948 |
| Air (Retina, 13", 2020) | Intel + T2 | 3 | 「Three-mic array with directional beamforming」（列在 Audio 节下） | https://support.apple.com/en-us/111991 |
| Air (M1, 2020) | Apple M1 | 3 | 「Three-mic array with directional beamforming」 | https://support.apple.com/en-us/111883 |
| Air (M2, 2022) | M2 | 3 | 同上 | https://support.apple.com/en-us/111867 |
| Air 15" (M2, 2023) | M2 | 3 | 同上 | https://support.apple.com/en-us/111346 |
| Air 13" / 15" (M3, 2024) | M3 | 3 | 同上，另加「Voice Isolation and Wide Spectrum microphone modes」 | https://support.apple.com/en-us/118551 ；/118552 |
| Air 13" (M4, 2025) | M4 | 3 | 同上 + 模式句 | https://support.apple.com/en-us/122209 |
| Air 13" (M5) | M5 | 3 | 同上，模式句加长为「…modes for enhanced voice clarity in audio and video calls」 | https://support.apple.com/en-us/126320 ；现行页 https://www.apple.com/macbook-air/specs/ （13" 与 15" 均为 M5） |
| M2 Air 新闻稿 | M2 | 3 | 「A three-mic array captures clean audio using advanced beamforming algorithms」 | https://www.apple.com/newsroom/2022/06/apple-unveils-all-new-macbook-air-supercharged-by-the-new-m2-chip/ |

### MacBook Pro · iMac · Studio Display

| 机型 | 芯片 | 麦数 | 原文 | URL |
|---|---|---|---|---|
| MBP 13" 2016 两口 | Intel | 2 | 「Two microphones」 | https://support.apple.com/en-us/111999 |
| MBP 13" 2016 四口 / 15" 2016–2019 / 13" 2018 | Intel（2018 起有 T2） | 3 | 「Three microphones」 | /112003 ；/111975 ；/111947 ；/111925 ；/111949 ；/111941 |
| **MBP 16" 2019** | Intel + T2 | 3 | **最早出现「Studio‑quality three-mic array with high signal-to-noise ratio and directional beamforming」** | https://support.apple.com/en-us/111932 |
| MBP 16" 2019 新闻稿 | Intel | — | 「40 percent reduction in hiss」；「signal-to-noise ratio that rivals that of popular professional-grade digital microphones」 | https://www.apple.com/newsroom/2019/11/apple-introduces-16-inch-macbook-pro-the-worlds-best-pro-notebook/ |
| MBP 13" 2020（两口与四口） | Intel + T2 | 3 | 「Three-mic array with directional beamforming」（**没有** studio-quality） | /111339 ；/111981 |
| MBP 13" M1 2020 / M2 2022 | Apple 芯片 | 3 | 「Studio-quality three-mic array with high signal-to-noise ratio and directional beamforming」 | /111893 ；/111869 |
| MBP 14" / 16"（2021、2023、2023 年 11 月、2024） | M1 Pro 至 M4 Max | 3 | 同上 | /111902 ；/111901 ；/111340 ；/111838 ；/117735–117737 ；/121552–121554 |
| MBP 14" M5、M5 Pro / Max | M5 | 3 | 同上 + 「Voice Isolation and Wide Spectrum microphone modes for enhanced voice clarity…」 | /125405 ；/126318 ；https://www.apple.com/macbook-pro/specs/ |
| iMac 24"（M1 2021、M3 2023、2024） | Apple 芯片 | 3 | 「Studio-quality three-mic array with high signal-to-noise ratio and directional beamforming」 | /111895 ；/117733 ；/121557 ；https://www.apple.com/imac/specs/ |
| Studio Display | 外接 USB 设备 | 3 | 同上 | https://www.apple.com/studio-display/specs/ |

**措辞规律**【推断】：「studio-quality」和「高信噪比」只给 Pro、iMac 和 Studio Display，Air 从没用过。可能是 Air 的麦克风元件或腔体规格更低，也可能只是营销分级。官方没有给出灵敏度或信噪比数字。

---

## 二、系统处理链：每一步谁能控制

| # | 结论 | 原文或数字 | 出处 | 标记 | 样本量 |
|---|---|---|---|---|---|
| 1 | VPIO 的定义：bus 1 是输入，会去掉本机正在播放的声音（回声消除） | 「does signal processing on the incoming audio (taking out any of the audio that is played from the device…)」 | SDK `AudioToolbox/AUComponent.h:290-300`（macOS 27.0 CommandLineTools SDK） | 【官方文档】 | — |
| 2 | VPIO 的内容（WWDC 原话） | 「echo cancellation, noise suppression, automatic gain control, among others」 | https://developer.apple.com/videos/play/wwdc2023/10235/ | 【官方文档】 | — |
| 3 | Bypass 关掉 VPIO 的全部处理，但语音突显和宽频谱优先于它 | 「Bypass all processing for microphone uplink… Voice Isolation and Wide Spectrum take priority over Bypass.」 | `AudioUnitProperties.h:2624-2630` | 【官方文档】 | — |
| 4 | 自动增益只在 VPIO 上有开关，默认开 | 「Enable automatic gain control on the processed microphone uplink signal. On by default.」 | `AudioUnitProperties.h:2632-2637`；AVAudioEngine 对应 `voiceProcessingAGCEnabled`（`AVFAudio/AVAudioIONode.h:235`） | 【官方文档】 | — |
| 5 | AVAudioEngine 开语音处理，必须输入、输出两端一起开；离线渲染模式（manual rendering）下不能开 | 「Voice processing requires both input and output nodes…」「not in the manual rendering mode」 | `AVAudioIONode.h:141-164` | 【官方文档】 | — |
| 6 | 三种模式的定义 | Standard「processed with standard voice DSP」；Wide Spectrum「processing is minimized to capture all sounds in the room」；Voice Isolation「isolate the voice and attenuate other signals」；macOS 12 起 | `AVFoundation/AVCaptureDevice.h:2715-2724` | 【官方文档】 | — |
| 7 | **「宽频谱」只是「处理减到最少」，并不是不处理；「标准」本身就有语音 DSP。** 但这些说的都是 VPIO 路径 | 同上 + WWDC23：「grants users full control over the mic mode settings for your app」 | 同上；WWDC23 10235 | 【官方文档】，「只在 VPIO 路径」是【推断】（有 WWDC 原话支撑） | — |
| 8 | 模式由用户在控制中心选，app 只能读，不能设 | `preferredMicrophoneMode` 只读，「selected by the user in Control Center」；只能调 `showSystemUserInterface(.microphoneModes)` 弹出面板 | `AVCaptureDevice.h:2727-2792` | 【官方文档】 | — |
| 9 | 机型门槛：2018 款及以后的 Mac；只在支持的 app 里出现 | 「Mic Modes are only available on Mac models (2018 and later).」「options vary by app」 | https://support.apple.com/en-is/guide/mac-help/mchle82b42f0/12.0/mac/12.0 ；现行版 https://support.apple.com/guide/mac-help/mchle82b42f0/mac | 【官方文档】 | — |
| 10 | **HAL 路径做了什么：苹果没有文档。** 官方只说「阵列有定向波束成形」（规格页）和「beamforming algorithms」（新闻稿）。第三方用 HAL 拿到的是 1 声道 48 kHz（011 卡本机实查，Mac17,3）。Linux 上同一硬件是 3 路原始信号 | 见第一节；Asahi 见第五节 | — | 「HAL 拿到的已是波束成形后的信号」是【推断】，证据：官方阵列描述 + 1 声道 + Asahi 三路原始信号 | 本机 1 台（011 卡） |
| 11 | HAL 上有没有高通、降噪、自动增益：**未查到**任何苹果原句 | — | — | 未查到 | — |
| 12 | HAL 层唯一有文档的「处理」开关是「环境降噪」，Apple 芯片和 T2 机型上不出现。macOS 14、15 帮助页有这句，macOS 26、27 帮助页已删 | 「Select "Use ambient noise reduction"… This option does not appear if a 4-channel built-in microphone format is selected in Audio MIDI Setup, on a Mac with Apple silicon or on Mac models with an Apple T2 chip.」 | https://support.apple.com/en-gb/guide/mac-help/mchlp2567/15.0/mac/15.0 ；26 版 https://support.apple.com/en-gb/guide/mac-help/mchlp2567/26/mac/26 （无此项） | 【官方文档】。读作「三种情况下都不出现」是【推断】，原句断句有歧义 | — |
| 13 | HAL 自带「有人在说话」检测，作用在回声消除后的输入上，不用开 VPIO | 「detecting voice activity from echo-cancelled microphone input」 | `CoreAudio/AudioHardware.h:995-1031`；WWDC23 10235 | 【官方文档】 | — |
| 14 | 别的进程在内置麦上开了 VPIO 时，不经 VPIO 读到的电平会很低 | 「its volume is known to be very low without VPIO whenever VPIO is hooked up to it elsewhere」 | `resources/cubeb-coreaudio-rs/src/backend/mod.rs:3561` | 【源码】（Mozilla 的经验） | 未写 |
| 15 | VPIO 会把输入下混成单声道；它的 input scope 里还带回声消除用的参考声道 | 注释原文 | `cubeb…/mod.rs:3881,3921` | 【源码】 | — |
| 16 | Firefox 把 Studio Display 的「输入 + 输出」配对拉进 VPIO 黑名单 | `APPLE_STUDIO_DISPLAY_USB_ID = "05AC:1114"` | `cubeb…/mod.rs:66,3625-3626` | 【源码】 | — |
| 17 | cpal 的输入走 HalOutput，不经 VPIO（纠正 011 卡 #20 的行号） | `device.rs:84` 调 `audio_unit_from_device_id_uninitialized`，后者在 `!input` 不成立时选 `IOType::HalOutput` | `resources/cpal/src/host/coreaudio/macos/device.rs:84`；`resources/coreaudio-rs/src/audio_unit/macos_helpers.rs:110-113` | 【源码】 | — |

## 三、「音频 MIDI 设置」：格式、输入音量、自动增益

| # | 结论 | 原文或数字 | 出处 | 标记 | 样本量 |
|---|---|---|---|---|---|
| 1 | 「格式」菜单只选采样率和位深；音量滑块是蓝色时才可调 | 「choose the sample rate and bit depth」；「If software volume control is available… sliders appear blue」 | https://support.apple.com/guide/audio-midi-setup/ams1010/mac | 【官方文档】 | — |
| 2 | 内置麦有一个总音量，加一个不能单独调的 1 声道音量 | 「a primary volume control… and a volume control for channel 1 which cannot be set」 | https://developer.apple.com/forums/thread/763325 （Apple 员工 Michael 回答） | 【官方文档】（Apple 员工在论坛的回答） | 1 台 MacBook Pro |
| 3 | 系统设置里能拖输入音量。**这个滑块是模拟增益还是数字缩放：未查到** | 「Adjust the input volume: Drag the volume slider.」 | https://support.apple.com/guide/mac-help/change-the-sound-input-settings-mchlp2567/mac （macOS 27） | 【官方文档】 | — |
| 4 | 「4 声道内置麦格式」只在帮助页里提过一次。两位用户找不到这个选项：一台 iMac 24" M1，一台 16" MBP 2019 T2 | 「showing 1channel input (with all 4 channels merged...)」 | https://discussions.apple.com/thread/254239272 | 【论坛 / 博客，个人说法】 | 2 台机、2 个人 |
| 5 | **HAL 层没有官方的自动增益开关**。有人报告 M1 Air 输入音量会自己往下掉，也有人报告升级 macOS 26 后滑块随说话移动。多数归因于 Zoom 这类 app 的「自动调整麦克风音量」 | — | https://discussions-jp-prz.apple.com/thread/256125271 ；https://community.zoom.com/meetings-2/how-do-i-prevent-zoom-from-resetting-my-input-volume-to-1-4-on-launch-18356 ；https://sixcolors.com/post/2016/05/the-mystery-of-os-xs-haunted-input-volume-slider/ | 【论坛 / 博客，个人说法】 | 约 3–4 人，机型多数未写 |

## 四、蓝牙耳机、AirPods、iPhone 当 Mac 麦克风

| # | 结论 | 原文或数字 | 出处 | 标记 | 样本量 |
|---|---|---|---|---|---|
| 1 | 蓝牙有两种模式，开麦后音质降低 | 「The second mode is for both speaking through the microphone and listening… audio quality is reduced until the microphone is no longer in use.」 | https://support.apple.com/102217 | 【官方文档】 | — |
| 2 | 苹果官方**没有**公开 Mac 上蓝牙麦的采样率和编解码 | — | — | 未查到 | — |
| 3 | 一份 macOS 27.2 的报告：两只 HFP 麦都是 16 kHz 单声道，日志显示 HFP/mSBC | 「Both input devices expose 16,000 Hz, mono. Bluetooth logs show HFP/mSBC.」 | https://developer.apple.com/forums/thread/854419 | 【论坛 / 博客，个人说法】（没有 Apple 员工回复） | 1 人 |
| 4 | AirPods Pro / AirPods 3 用 AAC-ELD，24 kHz 单声道；也有「48 kHz」的说法 | — | 011 卡 #24 已列；https://www.idropnews.com/?p=178979 | 【论坛 / 博客，个人说法】 | 各 1 篇 |
| 5 | iOS 26 的「高质量蓝牙录音」选项在 macOS 头文件里标为不可用；头文件还提示它会增加延迟，不建议用于实时通话 | `AVAudioSessionCategoryOptionBluetoothHighQualityRecording API_AVAILABLE(ios(26.0)) API_UNAVAILABLE(… macos)`；「may increase input latency… not recommended for real-time communication」 | `AVFAudio/AVAudioSessionTypes.h:615-635`；`AVCaptureSession.h:569` | 【官方文档】 | — |
| 6 | 但苹果新闻稿说 AirPods 4、AirPods 4 降噪版、AirPods Pro 2 的「studio-quality audio recording … work across iPhone, iPad, and Mac」，靠 H2 芯片、波束成形麦和计算音频；没给采样率和编解码 | 原句如左 | https://www.apple.com/au/newsroom/2025/06/airpods-now-more-versatile-with-studio-quality-audio-recording-and-camera-remote/ | 【官方文档】。和 #5 冲突，Mac 上是否已经上线未核实 | — |
| 7 | iPhone 当 Mac 麦克风：系统设置 → 声音 → 输入里选 iPhone；要求 iPhone 横放、静止、熄屏。帮助页首句写的是「on a Mac that doesn't have a built-in or external microphone」，所以 Air 上能不能用、做了什么处理：**未查到** | 原句如左 | https://support.apple.com/guide/mac-help/mchl77879b8a/mac | 【官方文档】 | — |

## 五、Apple 芯片：有没有专门的麦克风 DSP、Linux 上看到的原始信号

| # | 结论 | 原文或数字 | 出处 | 标记 | 样本量 |
|---|---|---|---|---|---|
| 1 | 三支 PDM 麦接到 AOP 里的 ADC 和抽取器，三路分别送到用户态，没有前置放大 | 「three Pulse Density Modulation (PDM) mics wired up to an ADC and decimator in the AOP」「plumbed directly to userspace on separate channels, with no preamplification」 | https://asahilinux.org/docs/sw/audio-userspace/ | 【官方文档】（Asahi 项目文档；对 macOS 内部只是旁证） | — |
| 2 | 每路 48 kHz，麦间距约 2 cm；作者说延迟求和不够用 | 摘自 2025-03 进展报告 | https://asahilinux.org/2025/03/progress-report-6-14/ | 【官方文档】（Asahi） | — |
| 3 | SEP（安全隔区）在麦克风数据线路上：「If SEP is unhappy, you don't get mic access.」 | 同上 | 同上 | 【官方文档】（Asahi） | — |
| 4 | 支持麦克风的机型：MBP 13" M1/M2、**Air 13" M1/M2**、MBP 14"/16" M1/M2 Pro/Max、**Air 15" M2**；MBP 14" M2（J414）AOP 起不来；要求固件 ≥ 13.5 | 原文 | 同 #1；https://github.com/chadmed/triforce | 【官方文档】（Asahi） | — |
| 5 | **Air M1（J313）和 Air M2（J413）的阵列几何与调参**：三麦呈边长 2 cm 的等边三角形。Asahi 的处理链是：每路 ×2 → MVDR 波束成形（目标水平角 J313 为 120°、J413 为 80°，`opt_freq` 1100 Hz，`t_win` 500 ms）→ ×7.25 → ×7.25 → 120 Hz 高通（Q 0.707）；采集 3 声道 48 kHz，输出单声道，默认音量 0.343 | J313 / J413：`mic2_x: 0.02`，`mic3_x: 0.01`，`mic3_y: 0.0173`；MBP13（J293）是线阵：`mic3_x: 0.04, mic3_y: 0` | https://github.com/AsahiLinux/asahi-audio `firs/j313/mic.json`、`firs/j413/mic.json`、`firs/j293/mic.json`（main 分支，最新提交 ab1b46928d，2026-10-07；「Make the mics much louder」那次提交是 687d8cd0f6，2026-07-21） | 【源码】（这是 Asahi 的调参，不是苹果的） | 每种机型 1 份配置 |
| 6 | Triforce 作者称 macOS 在用户态做自适应波束成形 | 「implement an adaptive beamformer in userspace」 | https://github.com/chadmed/triforce | 【论坛 / 博客，个人说法】（第三方判断，苹果未证实） | 1 人 |
| 7 | 苹果官方材料里**没有**「专门的麦克风 DSP」或「AOP 做波束成形」的说法 | — | WWDC、新闻稿、平台安全指南都查过 | 未查到 | — |
| 8 | 合盖时硬件断开麦克风：Apple 芯片笔记本，以及 2019 年及以后带 T2 的笔记本（13" MBP 和 Air 的 T2 机型也在列）；用不可重编程的硬件逻辑，最多延迟 500 ms，例如切断时钟线 | 「turning off one of the input lines to the microphone module」 | https://support.apple.com/guide/security/hardware-microphone-disconnect-secbbd20b00b/web | 【官方文档】 | — |
| 9 | Intel T2 机型：T2 集成了音频控制器 | 引述苹果的说法 | https://cdm.link/?p=91277 ；https://en.wikipedia.org/wiki/Apple_T2 | 【论坛 / 博客，个人说法】（苹果原页未找到） | — |
| 10 | speakersafetyd 只管扬声器：估算音圈温度、限制音量，当前硬上限 −7 dBFS。和麦克风无关 | 原文 | https://asahilinux.org/docs/sw/audio-userspace/ | 【官方文档】（Asahi） | — |

---

## 处理阶段图（文字版，Apple 芯片 MacBook Air）

```
[3 支全向 PDM 麦膜片]        Air M1/M2：边长 2 cm 的等边三角形（Asahi 配置）
   │  合盖 → 硬件逻辑切断麦克风输入线（≤500 ms）           【官方 · 平台安全指南】
   ▼
[SEP 门控 → AOP 内 ADC + 抽取器 → 3 路 × 48 kHz，无前置放大] 【Asahi】
   │
   ├─ Linux：3 路原样进 ALSA → Triforce MVDR → 增益 → 120 Hz 高通 【Asahi 源码】
   │
   ▼ macOS：
[苹果未公开的环节：三路合成 / 「directional beamforming」/ 有无高通、降噪、增益未知]
   │    苹果只说「beamforming algorithms」；位置不明      【官方 + 推断】
   │    Intel T2 机型：经 T2 音频控制器                    【个人说法】
   │    T2 之前的 Intel 机型：设备级「环境降噪」开关          【官方】
   ▼
[HAL 设备「MacBook Air Microphone」：1 声道 48 kHz，输入音量滑块]  【本机 1 台 + 官方】
   │    可选：HAL「有人在说话」检测（基于回声消除后的输入）     【官方】
   ●══ 第三方 HAL 取音点（cpal HalOutput；我们现在在这里取）   【源码】
   │    ⚠ 若别的进程在这只麦上开了 VPIO，这里电平会很低      【源码 · Mozilla】
   ▼
[VPIO / AVAudioEngine 语音处理：回声消除 + 降噪 + 自动增益（默认开）+ 下混单声道 + 压低其他声音]
   │    麦克风模式由用户在控制中心按 app 选：标准（standard voice DSP）/
   │    语音突显 / 宽频谱（processing minimized），后两者优先于 Bypass  【官方】
   ▼
[会议 app（FaceTime / Zoom / Webex…）自己的回声消除、自动增益、降噪]
```

## 没查到 / 仍不确定

1. HAL 输出的 1 路信号，是在 AOP、内核驱动还是 coreaudiod 里合成的：苹果没有原文。
2. 那 1 路信号在「标准」以外是否随麦克风模式变化：没有原文。HAL 路径上有没有固定的高通、降噪、自动增益：没有原文。只能 A/B 实测。
3. Air 内置麦的灵敏度、信噪比、频响：苹果不公开，Air 规格页连「high SNR」都没写。
4. Air 和 Pro 的麦克风硬件到底差在哪：只有措辞差别，没有数字。
5. 「4 声道内置麦格式」在哪些机型上存在、内容是什么：只有帮助页一句话。2 位用户找不到。
6. 系统设置里的输入音量是模拟增益还是数字缩放：未查到。
7. Mac 上蓝牙麦的官方采样率和编解码：未查到。AirPods「studio-quality 录音」在 macOS 26 上是否真的上线、走什么链路：新闻稿和 SDK 头文件冲突，未核实。
8. iPhone 当麦克风在带内置麦的 Air 上能不能选、有没有经过语音突显：未查到。
9. Intel T2 机型上，波束成形是否在 T2 里做：只有「T2 集成音频控制器」这条第三方转述。
10. apple.com 上 M1 Air 的发布新闻稿没提麦克风。M1 Air 有没有官方「高信噪比」一类说法：未查到。

本次没有改仓库文件。临时下载的网页放在 本会话临时目录 `scratchpad/kb/`（不入库）。