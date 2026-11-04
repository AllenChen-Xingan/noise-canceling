# 转交 macmic

> 2026-11-03 · 手下 r1-device 在查「消费设备真实录音」时顺手找到的 Mac 内置麦相关材料。每条带出处；未下载任何音频。

## 一、带 Mac 内置麦的公开录音（按可用度排）
1. **DR-VCTK**：MacBook Air 2 通道，与 iPhone 5S、iPad、两只 USB 麦同步重录 VCTK，回放距离 2 m，办公室开窗 / 关窗；有逐样本干净参考；1.67 GB；README 写 CC BY 4.0，但 datashare 另附 End-user Licence 未读到。机型年份未写。见 014 卡 `docs/research/014_real-datasets/cards/dr-vctk.md`【官方文档】
2. **DNS Challenge 2021 两届 blind 合集**（`V2_V3_Challenge_Combined_Blindset.zip`，778 MB）：文件名带 `macbook` / `mac` / `Macbookpro` 的真实录音共 31 条，来自 4 位众包者【实测：用 HTTP Range 读 zip 目录】
   - `A1RK5KQQ320OH4`：12 条，英语男声，`macbook`，噪声为薯片袋、风扇、狗叫、打字、空调、流水、关门、车、婴儿哭、洗碗机、厨房、杂物，near / far 都有（V3 目录）
   - `XMQ6Y5VEXRUU`：12 条，英语女声，`mac`，钢琴背景，far（V3 目录 musical）
   - `AD5IA651K0JG2`：6 条，男声，`macbook`，情绪段（哭 2、笑 2、喊 2）（V2 目录）
   - `ALBJ191ZJL9IS`：1 条，`Macbookpro`，打字 + 生气（情绪集目录）
   - 文件名是众包者自填，未核机型、未核是否用了外接麦；无干净参考；许可未查到（README：数据按 Microsoft 取得时的原条款）
3. **MOBIO**：「a standard 2008 MacBook」只录了第一个 session 的一部分，真人问答 + 朗读；需签 EULA、限非商业研究。见 `../cards/mobio.md`【官方文档】
4. **3D-Speaker**：PC 笔记本 96,262 条且逐条带设备标签，但品牌型号未写，是否含 Mac 未查到。见 `../cards/3d-speaker.md`【论文】

## 二、确认**没有** Mac 内置麦的（免得再查）
- DNS 2022 blind：设备只分 `Mobile` / `Desk`【实测：文件名】
- DNS 2023 blind：`ap` 段写了约 40 种设备型号，无 Apple 产品【实测：文件名】
- AEC Challenge 真实集：README「captured on Windows PCs」【官方文档】
- ACE：笔记本是 Chromebook Pixel【沿用 014】
- Libri-Adapt：MacBook Pro 只作外接 Shure MV5 的主机，没用内置麦【官方文档】

## 三、Apple 官方关于 Mac 麦与系统级处理的材料
- MacBook Air（M2, 2022）技术规格：「Three-mic array with directional beamforming」——https://support.apple.com/en-us/111867【官方文档】
- Mac 用户手册「Mic Mode」：在支持的 app（如 FaceTime）里从菜单栏选 Voice Isolation（滤掉背景声）/ Wide Spectrum（保留周围声）/ Standard；原句「options vary by app」——https://support.apple.com/guide/mac-help/mchle82b42f0/mac【官方文档】
- 开发者文档 `AVCaptureDevice.MicrophoneMode`：macOS 12.0 起；`voiceIsolation` 原句「processes microphone audio to isolate the voice and attenuate other signals」，`wideSpectrum` 原句「minimizes microphone audio processing to capture all sounds in the room」——https://developer.apple.com/documentation/avfoundation/avcapturedevice/microphonemode【官方文档】
- `AVCaptureDevice.preferredMicrophoneMode`：「The microphone mode that the user selects in Control Center」，app 只能用 KVO 观察；`activeMicrophoneMode` 可能与之不同（当前音频路由不支持时）；`showSystemUserInterface(_:)` 只能弹出系统界面让用户自己改——即 **app 不能直接替用户切换麦克风模式**【官方文档】【推断：文档未列任何 setter】
- `AVAudioIONode.setVoiceProcessingEnabled(_:)`：macOS 10.15 起，开启系统语音处理（回声消除等）；`voiceProcessingOtherAudioDuckingConfiguration` macOS 14.0 起——https://developer.apple.com/documentation/avfaudio/avaudioionode/setvoiceprocessingenabled(_:)【官方文档】
- 第三方说法：Wide Spectrum 需 2018 款及以后的 Mac、Voice Isolation 需 macOS 12+——https://www.macrumors.com/how-to/block-background-noise-facetime-mac 等【论坛 / 博客，个人说法】

## 四、对 macmic 的提示（推断，待你那路核）
- 若虚拟麦从内置麦取音，用户在控制中心选的 Voice Isolation 可能已在上游处理过一遍；需要实测「开 / 关 Voice Isolation」时内置麦原始信号的差别【推断】
- DNS 2021 那 31 条 Mac 录音是目前唯一找到的「Mac + 真实家庭噪声 + 真人」公开片段，可作 Mac 频响的粗旁证，但要先人耳确认是不是内置麦【推断】
