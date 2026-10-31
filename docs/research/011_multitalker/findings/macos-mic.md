# 问题 2 · macOS 上能不能拿到多路麦克风原始信号（KEY=macos-mic）

> 只读调研，2026-10-30。SDK = `/Applications/Xcode.app/.../SDKs/MacOSX27.2.sdk/System/Library/Frameworks/`（下文简写 `SDK/`）；开源仓 = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`（简写 `res/`）。

## 一句话结论

**拿不到多路原始信号。** Apple Silicon MacBook 的「三麦阵列 + 波束成形」在 Core Audio 层只给 **1 声道**，且已经处理过（本机实查 1ch/48 kHz）。AirPods 也只给单声道窄带。真要多路原始信号，只有 USB 阵列麦这一条路（例如 ReSpeaker 6ch 固件 = 4 路原始 + 1 路处理后）。

系统的「语音突显」（Voice Isolation）**按 app 生效，只对用了 Apple 语音处理（VPIO / AVAudioEngine voice processing）的 app 生效**。我们现在用 cpal 走 HAL 采集，拿不到它。要拿到，我们的处理进程得自己开 VPIO，再由用户在控制中心为**我们这个进程**选「语音突显」。这样能做到「系统先突显、我们后处理」，会议软件那边再选标准模式，就不会重复处理。代价是：VPIO 只出单声道、会压低其他声音、延迟没有公开数字，而且 app 无法用程序打开语音突显。

## 证据表

| # | 结论 | 出处 | 标记 |
|---|---|---|---|
| 1 | MacBook Air 规格页原文：「Three-mic array with directional beamforming」，并写「Voice Isolation and Wide Spectrum microphone modes for enhanced voice clarity in audio and video calls」 | https://www.apple.com/macbook-air/specs/ | 【官方】 |
| 2 | 本机（Mac17,3，MacBook Air，macOS 27）只读查询：`MacBook Air Microphone` 显示 **Input Channels: 1**，Current SampleRate 48000 | `system_profiler SPAudioDataType` 只读输出（只显示当前格式） | 【本机】 |
| 3 | macOS 上波束成形由 Apple 在**用户态软件**里做（自适应波束成形）。硬件裸阵列在 Linux 上能看到多路，需要自写 MVDR 才能用：「without beamforming, the array is far too sensitive and omnidirectional」 | triforce README（Asahi，GPL-2.0）https://github.com/chadmed/triforce ；Debian triforce-lv2 包说明 | 【官方】（对 Asahi 而言）／对 macOS 内部实现是【推断】 |
| 4 | Apple 帮助页说，「Use ambient noise reduction」这个选项「does not appear if a 4-channel built-in microphone format is selected in Audio MIDI Setup … on a Mac with Apple silicon or on Mac models with an Apple T2 chip」。说明内置麦**至少在部分机型上有 4 声道格式可选**，但没说内容是否原始；原句断句也有歧义 | https://support.apple.com/en-gb/guide/mac-help/mchlp2567/14.0/mac/14.4.1 | 【官方】（内容不详） |
| 5 | 有用户报告：新款 Mac 的内置麦呈现为 1 声道，旧款（2015）呈现为 2 声道 | https://forum.audacityteam.org/t/built-in-mic-on-old-mac-is-stereo-on-the-new-one-it-is-mono/117951 | 【传言】 |
| 6 | **VPIO 把设备输入下混成单声道**：「a VPIO unit downmixes the device's input channels to MONO」；VPIO 输入侧的 input scope 里还包含 AEC 参考声道 | `res/cubeb-coreaudio-rs/src/backend/mod.rs:3881,3921` | 【源码】 |
| 7 | VPIO 定义：bus 1 是输入，会「taking out any of the audio that is played from the device」（回声消除） | `SDK/AudioToolbox.framework/Headers/AUComponent.h:290-300` | 【官方】头文件 |
| 8 | VPIO 属性只有 4 个：`BypassVoiceProcessing`(2100)、`VoiceProcessingEnableAGC`(2101，默认开)、`MuteOutput`(2104)、`MutedSpeechActivityEventListener`(2106，macOS 14+)。**关键一句：「Voice Isolation and Wide Spectrum take priority over Bypass」** | `SDK/AudioToolbox.framework/Headers/AudioUnitProperties.h:2624-2649,2669-2678` | 【官方】头文件 |
| 9 | macOS 14+ 压低其他声音的设置：`kAUVoiceIOProperty_OtherAudioDuckingConfiguration`(2108)，档位 Default=0 / Min=10 / Mid=20 / Max=30，外加 advanced ducking 开关。默认是「不开 advanced，档位 Default」。**没有「不压低」这一档** | `AudioUnitProperties.h:2692-2724`；AVFAudio 对应 `SDK/AVFAudio.framework/Headers/AVAudioIONode.h:67-95,259-267` | 【官方】头文件 |
| 10 | Firefox 为了撤销 VPIO 带来的压低，调用了私有 API `AudioDeviceDuck(id, 1.0, …)`（开了 `no-private-apis` 时不调） | `res/cubeb-coreaudio-rs/src/backend/mod.rs:1514-1537` | 【源码】 |
| 11 | 内置麦有个坑：别的进程在这只麦上挂了 VPIO 时，不经 VPIO 读到的音量「known to be very low」 | `res/cubeb-coreaudio-rs/src/backend/mod.rs:3553-3569` | 【源码】（Mozilla 的经验） |
| 12 | 首个 VPIO 单元创建要 > 3 s，后续 < 1 s；全部释放后再建又要等很久 | `res/cubeb-coreaudio-rs/src/backend/mod.rs:2317-2336` | 【源码】（注释里的实测观察） |
| 13 | Apple Studio Display 输入输出配对时，Firefox 把 VPIO 拉黑 | `res/cubeb-coreaudio-rs/src/backend/mod.rs:3618-3623` | 【源码】 |
| 14 | 麦克风模式枚举：Standard（标准语音 DSP）/ WideSpectrum（「processing minimized」）/ VoiceIsolation | `SDK/AVFoundation.framework/Headers/AVCaptureDevice.h:2720-2738` | 【官方】头文件 |
| 15 | `preferredMicrophoneMode` **只读**：「selected by the user in Control Center」。`activeMicrophoneMode` 只读，在「application's active audio route」不支持时可能和 preferred 不同。说明**模式按 app 区分** | `AVCaptureDevice.h:2741-2758` | 【官方】头文件（「按 app」那部分是【推断】，但有 #16 佐证） |
| 16 | app 只能调 `showSystemUserInterface(.microphoneModes)` 弹出系统面板，让用户自己改 | `AVCaptureDevice.h:2764-2792` | 【官方】头文件 |
| 17 | WWDC23「What's new in voice processing」原话：「Choosing Apple's voice processing APIs also grants users full control over the mic mode settings **for your app**, including Standard, Voice Isolation, and Wide Spectrum.」 | https://developer.apple.com/videos/play/wwdc2023/10235/ （约 0:00–3:19） | 【官方】 |
| 18 | Apple 帮助页：要「Go to an app that supports Mic Modes, such as FaceTime」，并说「options vary by app」 | https://support.apple.com/en-au/guide/mac-help/mchle82b42f0/mac | 【官方】 |
| 19 | 麦克风面板是否出现，和 app 怎么开麦有关：Firefox 先建 VPIO 再换成 HAL 单元时，面板会消失 | https://connect.mozilla.org/t5/ideas/voice-isolation-on-macos/idc-p/71913 等 | 【传言】（工程师论坛帖） |
| 20 | **我们现在的采集路径不会触发麦克风模式**：cpal 固定用 HalOutput 单元（「We explicitly pin HalOutput unit here」），没有 VPIO | `res/cpal/src/host/coreaudio/macos/device.rs:67,82-84`（勘误：原写 89-91 是输出分支） | 【源码】 |
| 21 | AVAudioEngine 的语音处理：必须输入、输出两个节点都开；只能在引擎渲染到真实设备时开，**manual rendering 模式不支持** | `AVAudioIONode.h:141-164` | 【官方】头文件 |
| 22 | HAL 层自带 VAD（不必开 VPIO）：`kAudioDevicePropertyVoiceActivityDetectionEnable`('vAd+') / `…State`('vAdS')，作用在回声消除后的输入上，process mute 时也能检测。macOS 27 新增 `kAudioDevicePropertySuggestedReferenceDevice`('eord')，用来指定回声参考的输出设备 | `SDK/CoreAudio.framework/Headers/AudioHardware.h:995-1031`；WWDC23 10235 约 11:37 | 【官方】头文件 |
| 23 | macOS 26/27 新 API 里**没有新的语音处理模式**：AVFAudio 新增的是 `AVAudioSessionCapability`（26）和实时安全的 manual rendering 输入块（27，`AVAudioIONode.h:50,218`）。AirPods「高质量录音」`AVAudioSessionCategoryOptionBluetoothHighQualityRecording` 和 `FarFieldInput` 标的是 **API_UNAVAILABLE(macos)**，只在 iOS 26 可用。`AVAudioSessionPortExtensionBluetoothMicrophone` 同样标 macOS 不可用 | `SDK/AVFAudio.framework/Headers/AVAudioSessionTypes.h:600-635`；`AVAudioSessionRoute.h:130-150` | 【官方】头文件 |
| 24 | AirPods 麦克风：HFP 是 8/16 kHz 单声道；2021 年后 AirPods Pro/3 改成 AAC-ELD，**24 kHz（+SBR）单声道**；2024 年固件传言通话可到 48 kHz | https://www.ithinkdiff.com/apple-upgraded-airpods-pro-speech-codec-aac-eld/ ；https://www.headphonesty.com/2024/07/apple-solves-bad-airpods-call-quality/ ；https://www.jeffgeerling.com/blog/2018/airpods-get-stuck-low-quality-16-khz-audio-mode-when-starting-vm/ | 【传言】 |
| 25 | 蓝牙 HFP 端口常带硬件语音处理；VPIO 会自动处理这种情况（iOS 文档） | `AVAudioSessionRoute.h:170-180`（`hasHardwareVoiceCallProcessing`，macOS 不可用） | 【官方】（仅 iOS） |
| 26 | ReSpeaker USB Mic Array（XMOS XVF-3000）有两种固件：1ch 固件 = 处理后音频；**6ch 固件 = ch0 处理后、ch1–4 四路原始、ch5 回放**。最高 16 kHz，macOS 免驱，产品已 EOL | https://wiki.seeedstudio.com/ReSpeaker-USB-Mic-Array/ | 【官方】（厂商） |
| 27 | 拆分多声道 USB 麦的开源虚拟设备（duomic）：把多声道 USB 麦逐声道拆成虚拟麦，说明 UAC 多声道在 macOS 上原样暴露 | `res/duomic/README.md:1-10` | 【源码】 |

## 对本项目的含义

**1. 多麦波束成形 / 空间 TSE 这条路线在内置麦上走不通（M1/M2 都不行）。**
- 内置麦第三方只拿到 1 路已处理信号（#2、#3、#6）。Asahi 那样拿裸阵列，只有 Linux 驱动做得到，macOS 下没有公开接口。
- 「4 声道格式」（#4）是唯一的悬念，要实测（见 gap）。
- 只能单通道：多人说话还得靠 PSE/TSE（声纹）路线。
- USB 阵列麦是小众选配（#26）：16 kHz 正好对上 GTCRN。可以在**场景层**加一个「检测到多声道 USB 阵列 → 启用空间分支」的开关，但这不进主线。ReSpeaker 已停产，不能当默认设备。

**2. 「让系统先做语音突显，我们再处理」技术上可行，前提是我们的采集改用 VPIO。**
- 现状：cpal 用的是 HalOutput（#20），我们的进程不会出现在控制中心的麦克风模式里。
- 做法：处理进程用 coreaudio-rs 的 `IOType::VoiceProcessingIO`（`res/coreaudio-rs/src/audio_unit/types.rs:464`）开内置麦，并设 `BypassVoiceProcessing=1`、`EnableAGC=0`。用户选「标准」时信号基本保持原样；用户选「语音突显」时系统照样突显，因为头文件写明 Voice Isolation 优先于 Bypass（#8）。
- 会议软件读的是我们的虚拟麦。它们的模式只作用在它们自己的 VPIO 上（#15、#17），所以会议软件里选「标准」，就不会重复突显。不过它们的 AEC/AGC/降噪仍会再跑一遍。这一点我们控制不了，只能在用户说明里写清楚。
- **代价**：
  - VPIO 只出单声道（#6，反正我们也只要单声道）。
  - 会压低其他声音，公开 API 最低只能压到 Min 档，无法关掉（#9、#10）。
  - 首次创建要 > 3 s（#12），但只在启动时付一次。
  - VPIO 的附加延迟**没有官方数字**，会不会吃掉 45 ms 预算要实测。
  - app 不能用程序打开语音突显，只能弹面板（#16）。所以它只能做成场景层的一个「建议用户手动开」的提示，不能做成自动切换。
  - 有时候语音突显本身就会吃字。exp003 那类 babble 场景下，它和我们的模型叠在一起是增益还是双重伤害，要盲听才知道。
- **不重复做**：我们开 VPIO + Bypass 时，用户在控制中心为「vdn」选「标准」，就是我们独自处理；选「语音突显」，就是系统先做、我们后做（这时 GTCRN 可以降一档强度，由场景层根据 `activeMicrophoneMode` 的 KVO 读到的值切换参数，符合「只改参数不碰声音」的规则）。

**3. 场景层可以白拿的信号（不计算力）。**
- HAL VAD（#22）：自带回声消除，作用在设备上，不用开 VPIO。可以作为「本人在说话」的一个廉价输入，供 0.5 s 一次的判断回路使用。
- `activeMicrophoneMode` 可读、可 KVO（#15）：用来知道系统当前是否已经在突显。

**4. AirPods。** 单声道，16 kHz（HFP）或 24 kHz（AAC-ELD）（#24，传言级）。iOS 26 的高质量录音 API 在 macOS 上不可用（#23）。重采样到 16 kHz 后可以直接喂 GTCRN。AirPods 本身已在耳机端做了波束成形和降噪，所以场景层应把它当作「已处理输入」，降低我们的强度。

**5. 许可。** 只用到系统 API 和厂商固件，没有许可问题。triforce 是 GPL-2.0（README 标的是否带 "or later" 未核实）：如果是 GPL-2.0-only，就和 GPL-3.0 不兼容，不能抄进来。反正 macOS 下也拿不到裸阵列，用不上它。

**6. 内置麦的音量坑（#11）。** 会议软件对内置麦开了 VPIO、我们同时用 HAL 读同一只麦时，我们读到的电平可能很低。这是改用 VPIO 采集的又一个理由；即使不改，M0/M1 的电平监控里也要有这一条处理。

## gap（需另开实验或查不到）

1. **4 声道内置麦格式是否存在、内容是什么**：在 Audio MIDI Setup 的格式下拉里看（只读），或者读 `kAudioStreamPropertyAvailablePhysicalFormats`（要跑代码，按禁令本次没做）。M1/M2/M3+ 各机型可能不同。如果真有 4 路原始信号，结论 1 要改。
2. **VPIO 附加延迟**：VPIO 和 HAL 各采一遍，注入脉冲做相关（可沿用 003 方案里的第三条流测法）。官方没给数字，【未查到】。
3. **VPIO + Bypass=1 + 用户选「标准」时信号是否真的接近原样**（有没有残留的高通、限幅），以及 Bypass=1 时选「语音突显」是否真的生效（头文件这么写，没实测）。
4. **我们的进程用 VPIO 时，控制中心是否把「vdn」列为可选模式的 app**：后台 / 菜单栏 app（LSUIElement）也算吗？【未查到】。
5. **会议软件对我们的虚拟麦开 VPIO 时，系统是否还提供语音突显**：模式是只作用在内置麦 / 苹果设备上，还是虚拟设备也行？【未查到】。如果虚拟设备也行，就要提醒用户在会议软件里选「标准」。
6. **语音突显在 exp003 那类 babble −5 dB、旁人说话片段上的表现**：系统突显、我们的模型、二者叠加，三组盲听比较。需要人手动在面板里开（006 gap 已登记过「Voice Isolation 当商业对照」，这次可以并进去）。
7. **压低其他声音能否用公开 API 关掉**：Min 档实际压多少【未查到】。私有 `AudioDeviceDuck` 不能用于上架，也不稳。
8. **AirPods 在 macOS 27 上的实际麦克风采样率**：可以用 `system_profiler` 只读查（要连上 AirPods 并激活麦克风）。AAC-ELD 在 Mac 端的官方说明【未查到】。
9. triforce 许可是否为 GPL-2.0-or-later：未核实（影响很小）。

## 来源清单

- Apple SDK 头文件（MacOSX27.2.sdk）：`AudioToolbox/AUComponent.h:290-300`；`AudioToolbox/AudioUnitProperties.h:2624-2759`；`AVFoundation/AVCaptureDevice.h:2720-2792`；`AVFAudio/AVAudioIONode.h:50,67-95,135-267`；`AVFAudio/AVAudioSessionTypes.h:600-635`；`AVFAudio/AVAudioSessionRoute.h:130-180`；`CoreAudio/AudioHardware.h:995-1031`
- WWDC23 10235「What's new in voice processing」https://developer.apple.com/videos/play/wwdc2023/10235/
- Apple MacBook Air 规格 https://www.apple.com/macbook-air/specs/
- Apple 帮助：麦克风模式 https://support.apple.com/en-au/guide/mac-help/mchle82b42f0/mac ；声音输入设置 https://support.apple.com/en-gb/guide/mac-help/mchlp2567/14.0/mac/14.4.1
- 开源：`res/cubeb-coreaudio-rs/src/backend/mod.rs:1514-1537,2317-2336,3553-3623,3881,3921`；`res/cpal/src/host/coreaudio/macos/device.rs:89-91`；`res/coreaudio-rs/src/audio_unit/types.rs:443,464`；`res/duomic/README.md`
- triforce（Asahi）https://github.com/chadmed/triforce
- Seeed ReSpeaker USB Mic Array https://wiki.seeedstudio.com/ReSpeaker-USB-Mic-Array/
- 传言级：https://www.ithinkdiff.com/apple-upgraded-airpods-pro-speech-codec-aac-eld/ ；https://www.headphonesty.com/2024/07/apple-solves-bad-airpods-call-quality/ ；https://www.jeffgeerling.com/blog/2018/airpods-get-stuck-low-quality-16-khz-audio-mode-when-starting-vm/ ；https://forum.audacityteam.org/t/built-in-mic-on-old-mac-is-stereo-on-the-new-one-it-is-mono/117951 ；https://connect.mozilla.org/t5/ideas/voice-isolation-on-macos/idc-p/71913
- 本机只读查询：`system_profiler SPAudioDataType`（Mac17,3）
