> workflow `wf_16dd3e65-ebc` · measure 路怀疑派原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

| # | 结论（报告原句的大意） | 判定 | 理由与核到的原文 | 修正建议 |
|---|---|---|---|---|
| 1 | 能找到实测数字的 Air 只有 DXOMARK 的两台：M2 Air 13″ 和 M2 Air 15″，每款测 1 台 | 成立 | DXOMARK 笔记本评测列表里只有这两款 Air。另外 3 台新 Mac（MBP 14″ M4 / M5、MacBook Neo）也测过，但页面里只有摄像头图表，没有音频图表（我重新解析了已存的网页）。Apple 官方规格不给频响 | 补一句：DXOMARK 新版测法已不测音频，以后也不会有 M3/M4 Air 的曲线 |
| 2 | 测法：半消声室，声源距 1 m，用相机 app 录，按 1/12 倍频程出频响 | 成立（有小误） | 页面原文：「1/12 octave frequency response … at 1 meter in an anechoic environment」「recorded in our semi-anechoic room」。「using the camera app」这句只写在**指向性**图下面。频响用的是哪个 app，页面没写 | 写成「指向性用相机 app 录；频响用的 app 没写，推测相同」 |
| 3 | M2 Air 13″：100 Hz −10.1、200 Hz −1.7；低端 −3 dB 点约 163 Hz，−10 dB 点约 102 Hz | 成立（数字可复现，含义存疑） | 我重跑解析脚本，数字完全一致，取的是「Audio capture frequency response」那张图，不是放音图。但 200 Hz 是 −1.7，300 Hz 是 −2.8，曲线在 −3 dB 上下来回起伏，所以「163 Hz」这个点很不稳定，数据从 80 Hz 才开始 | 改写成「约 150–300 Hz 在 −2 到 −3 dB 间起伏，100 Hz 约 −10 dB」，别给单一截止点 |
| 4 | 6–11 kHz 上抬 +6 到 +9 dB | 存疑 | 解析值：6 kHz +6.2，**8 kHz +4.5**，10 kHz +7.6，峰值在 10.7 kHz +9.4，12 kHz +3.8。这一段中间有凹口，不是平的宽峰 | 改成「4–11 kHz 有起伏的抬升（+3 到 +9 dB），8 kHz 附近有个凹口」 |
| 5 | 约 12.6 kHz 后开始衰减，16 kHz 约 −13 dB | 成立 | 高端 −3 dB 点算出来是 12554 Hz，16 kHz 是 −13.4 dB | — |
| 6 | 评语「a bit thin-sounding」「not well attenuated」 | 成立 | 原文一字不差。但这些评语属于「视频通话」那一项的评分，评测时用的是哪个 app、哪个麦克风模式，页面没写 | 注明「评语出自视频通话评分，用的 app 和模式未知」 |
| 7 | M2 Air 13″ 指向性，90° 处：低频段 −3.4、中频段 −13.6、高频段 −31.0 dB；180° 和 270° 的数也对；「各频段的频率范围未给」 | 数字成立，后半句推翻 | 我重新解析极坐标数据，数字全部一致。同一页频响图里标了频段：**BASS 20–330 Hz、MIDRANGE 330–3000 Hz、TREBLE 3000–20000 Hz**（推断：指向性的三个频段也是这么分的） | 补上频段范围；也补上 Air 15″ 270° 的数：−4.2 / −7.8 / −17.5 |
| 8 | 「这一条可以直接用在合成数据里」 | 存疑 | (1) 样本只有 1 台，而且是在消声室里测的。(2) Asahi 的 Triforce 说明里写着：Apple「implement an adaptive beamformer in userspace」，也就是指向性是**自适应**的，会随声场变，不是固定滤波器。(3) MBP 14″ 的形状完全不同：90° 处中频段 −23.5、高频段只有 −10.8，可见这组数和机型、处理方式都有关 | 降级成「参考数值」。合成时角度衰减要加随机范围，并注明「自适应波束形成，静态近似」 |
| 9 | 「旁边说话的人会被录得又小又闷」 | 存疑（是推断） | 没有出处直接这么说。DXOMARK 只说「directivity was not optimal」。Air 15″ 页面说的是「mono … intelligibility could be impaired when multiple people…」 | 标成「推断」 |
| 10 | M2 Air 15″ 只录单声道 | 成立，但不是 Air 15″ 独有 | MBP 14″（M2 Pro 和 M3 Pro）两页也写「mono-only audio capture」 | 改成「DXOMARK 测过的 4 台 Mac 都只出单声道」 |
| 11 | 双方同时说话时有门控（gating）（Air 13″） | 成立，但不是 Air 独有 | 4 台 Mac 的页面都写了「Duplex handling issues … gating」。Air 15″ 原文是「Despite some gating, it handled duplex situations quite well」 | 改成「Apple 默认处理在双方同时说话时普遍会门控（推断：来自回声消除）」 |
| 12 | 录音总谐波失真加噪声（THD+N），M2 Air 13″：中位数 0.17%，最大 0.98% | 成立（有陷阱） | 13″ 页面「Audio capture THD+N」那张图算出来确实是这两个数。但 **Air 15″ 页面没有录音失真图**，脚本在那页抓到的是放音失真（2–3%）。别把那几个数误当成录音失真 | 注明「Air 15″ 没有录音失真数据」；脚本应该按图题选图，不要按顺序选第几张 |
| 13 | Air 13″ 录音电平比 MBP 14″ 低约 9.5 dB（−23.8 对 −14.3） | 存疑，接近推翻 | 同一个解析下，Air 15″ 页面上 Air 15″ 是 −0.5，Air 13″ 是 −23.8，两台 Air 差了 23 dB，这不像真实电平差。各页曲线的绝对位置多半不能拿来比 | 删掉，或写成「图上纵轴偏移，不能当电平比较」 |
| 14 | 所有被测 Mac 在约 15–16 kHz 都陡降，−10 dB 点在 15.3–15.7 kHz；Windows 机延伸到 17.9 kHz | 成立 | 解析值：Air 13″ 15671 Hz，Air 15″ 15671 Hz，MBP M2 Pro 15327 Hz，MBP M3 Pro 15671 Hz。Windows 机：Surface 17901 Hz，Asus 18302 Hz，Dell 18713 Hz | 补一句推断：3 台 Mac 的 −10 dB 点相同，像是处理链里有个固定低通（比如内部用 32 kHz 采样），值得在本机只读核实 |
| 15 | 3 个 PDM 全向麦，送到用户态前没有前置放大 | 成立 | Asahi 文档原文：「three Pulse Density Modulation (PDM) mics … with no preamplification. They are very sensitive and omnidirectional」。支持的机型是 M1/M2 Air 13″ 和 M2 Air 15″ | 补更权威的出处：Apple 规格页，M1–M4 Air 都写「Three-mic array with directional beamforming」（在 `kb/` 下的规格页存档里核到，例如 `kb/111883.html` 是 M1 Air）。老 Intel Air 的规格页写「Dual microphones / Two microphones」 |
| 16 | Asahi 给 Air 的增益 2 × 7.25 × 7.25 ≈ +40.4 dB；阵列边长 2 cm；导向角 J313 120°、J413 80°；高通 120 Hz | 成立（漏了一项） | 读了 j313 和 j413 的配置，数字一致。但输出节点还有 `"state.default-volume": 0.343`，所以默认的净增益小于 40 dB。Triforce 的说明还写了阵列「arranged either in a triangle or in a straight line」 | 补上默认音量 0.343。别把 +40 dB 当成 macOS 的增益 |
| 17 | Chrome 只在 `kEnforceSystemEchoCancellation` 打开时走 VPIO，这个开关默认关 | 成立 | `media_switches.cc:1583` 原文 `BASE_FEATURE(kEnforceSystemEchoCancellation, base::FEATURE_DISABLED_BY_DEFAULT)`。`audio_low_latency_input.cc:192-205` 和 `:955-993` 也对得上（我从 GitHub 镜像的 main 分支读的） | 补：Chrome 不走 VPIO，但默认会跑自己的 WebRTC 处理（AGC / NS），而且有 `SetVolume`（`:873`），能改设备的输入音量。「不走苹果处理」不等于「不处理」 |
| 18 | Chrome 不走 VPIO、网页也没要求降噪时，会关掉 `'nzca'`，用完再恢复 | 成立 | `audio_low_latency_input.cc:772-777`，`audio_manager_mac.cc:109-110` 和 `:1214-1250` 都核实了 | 注意文件路径是 `media/audio/mac/audio_manager_mac.cc`，不在 `apple/` 下 |
| 19 | 2017 年 Chrome 博客：`'nzca'` 是整台设备共用的开关 | 成立，但条件和现在相反 | 原文「Since this setting is system-wide this will apply to all audio input streams from the same device」。不过 2017 年的实验是在 **echoCancellation 开着时**关 `'nzca'`，现在的源码改成在**不要求降噪时**关 | 引用时只用它证明「整台设备共用」，触发条件以现行源码为准 |
| 20 | Zoom 在控制中心列为「zoom.us」，≥ 3 人这么说 | 推翻（人数） | 15516 和 18153 两帖里「It will show the app currently using the microphone (e.g., "zoom.us")」是**同一段文字**，作者都是 WalkingRiver，两帖互贴，读起来像写好的教程模板 | 改成「1 人（同一帖子贴了两处）」。Zoom 会触发麦克风模式，改用 41632 帖里 Zoom 自己的弹窗原文作证据（这条成立） |
| 21 | M1 Air 案例：2 个组织，各约 3 台 | 推翻 | 是**同一个组织**：WalkingRiver 的教会有 3 台 M1 Air，只有 1 台出问题。「We noticed this issue on one of our M1 MacBook Airs」是这个人自己的后续回复，被转贴到了 18153 | 改成「1 个组织，3 台里 1 台」。「macOS 可能在更新后默认开语音突显」是这个人的猜测 |
| 22 | Zoom Mac 版降噪关不掉，≥ 7 人（M1 Pro、M3、M4 MBP） | 存疑 | 「Windows zoom has more options…」在 18153 帖里，没在 133990 帖里。18153 帖里的「M4 MacBook Pros」出自回复者的推测（「Assuming…」），不是有人用 M4 机器报告。「low, medium, high are all the same」出自 16415 帖，M3 用户 | 逐条重点人数，把 M4 去掉 |
| 23 | Safari 默认处理下哼稳定音约 5 秒后音量降一半以上，三项处理全关就好，WebKit 判为「预期行为」；报告还拿它当自动增益时间常数的线索 | 事实成立，用途推翻 | 原文「diminishes … after about 5 seconds」，「Marking as WontFix since this is expected behaviour」，机器是 Mac（Apple Silicon）上的 MacBook Pro。但 Apple 工程师和报告人都说是**降噪**造成的：「it looks like this was an artifact of noise reduction」 | 从「自动增益时间常数」里移走，改成「降噪把稳定音当噪声，约 5 秒压下去」。合成数据那节「1–5 秒增益漂移」要拆成降噪自适应和自动增益两项 |
| 24 | 吃开头和结尾的字（Intel 2019 MBP 16″ + Scarlett） | 成立，但不该用来说内置麦 | 原文一致。但这台机器用的是 SM7B 外接麦加 Scarlett 声卡，不是内置麦，也不是 Air | 在「对内置麦的启示」里降权，或注明是外接麦 |
| 25 | macOS 26 Tahoe 后输入电平自己动（1 人 + 4 个「我也是」） | 存疑（张冠李戴） | 原文是「when I plug my **usb mic** into the mac」，机器是 Mac Pro 或 MacBook Pro，**不是内置麦** | 注明是 USB 麦。用它证明内置麦有问题不成立 |
| 26 | Air 13″（macOS 15.6）输入音量隔半天自己降 | 成立 | 标题「MacBook Air M1 mic volume decreases automatically」，原文「after a day or two or even a half day」 | 机型补上 M1。时间尺度是几天，和秒级自动增益不是一回事 |
| 27 | 2011 Air 录到背景噪声而不是人声，回复者判断是硬件坏了；1 人 | 成立（人数少报） | 帖子有「Me too (12)」。回复原文「Hardware fault」，另有人说「very rare」 | 人数改成「1 人发帖 + 12 个我也是」 |
| 28 | 16″ M1 Max MBP，社区专员说声音发闷是预期现象 | 存疑（语气说强了） | 原文是「It wouldn't necessarily be **unexpected**」 | 改成「专员说『不算意外』」 |
| 29 | 2009 年 iChat / Skype 用户报告音量滑块自己动；日志显示回声消除把音量拉到 0 | 成立 | 原文「the slider moves both to the left and to the right」，机器是 Late 2008 15″ MBP。版主说「iChat reports an Echo Cancelling issue … reducing the Volume to Zero」 | — |
| 30 | M1 Air 比 M1 Pro「更尖细」 | 存疑（型号写错） | 帖子发于 2020-11，那时 M1 Pro 芯片还没发布，比的是 **13″ MacBook Pro（M1）**。原帖还说「you have to turn up the volume on the Pro one to match」，两边音量没对齐，主观判断不可靠 | 改成「M1 Air 对 13″ M1 MBP，音量没对齐，1 人听感」 |
| 31 | 键盘声论文：从 Zoom 录下的键盘声识别准确率 93%，没核实是不是内置麦 | 成立（这项现在可以核掉） | arXiv HTML 原文：「using the MacBook's built-in microphone array. The noise-suppression parameter of Zoom was set to … 'low'」，机器是 MacBook Pro 16″（2021，M1 Pro） | 从「未核实」里移除。补一条：键盘瞬态声在 Zoom 低档降噪下仍然大量保留 |
| 32 | Macworld：老 Air 的环境降噪让音乐录音「thin, boxy, a little wavery」 | 成立 | 2014-06-25 的文章，作者 Christopher Breen，他在文中讲到「Use Ambient Noise Reduction option」 | — |
| 33 | Teams：Mac 版 AI 降噪当时不支持 M1；声纹语音隔离会压掉其他人声 | 成立 | 原文「not yet available for Mac machines with an M1 processor」；MC891240 原文「including other human voices」，发布于 2024-09-16（「2024-10 起」这个日期没核） | 日期写成「2024-09 公告」 |
| 34 | 时代广场测试；AppleInsider；Zoom 的 DriverKit「AUVoiceIO」帖；Teams 博客 4096077；Discord 帖子 | 未核（本次没取到原文） | 已存的网页里搜不到「AUVoiceIO」原文。其余几条报告自己也标了「未打开」 | 保持「未核实」，别进要点 |

**报告漏查的重要方向**
1. **Apple 官方规格**：各代 Air 规格页的麦克风写法（M1–M4 是「Three-mic array with directional beamforming」，Intel Air 是双麦）比 Asahi 更权威，也能直接回答「麦有方向性」。
2. **HAL 拿到的不是原始阵列**：Asahi 的 Triforce 说明里写着「Apple … implement an adaptive beamformer in userspace」（推断：在 coreaudiod 里）。所以 cpal 走 HAL 拿到的那 1 路，已经过 Apple 的自适应波束形成，可能还有固定的约 15.7 kHz 低通。这比 `'nzca'` 更直接地推翻「走 HAL 就不受处理」。
3. **Apple 开发者文档**：麦克风模式只对用语音处理（VPIO / AVAudioEngine voice processing）的 app 生效，相关 API 是 `AVCaptureDevice.preferredMicrophoneMode` / `activeMicrophoneMode`。有了这条，「Zoom 用了 VPIO」就不只是推断。
4. **Chrome 自己的处理链**：默认的 WebRTC 自动增益可以通过 `SetVolume` 改硬件输入音量。这也是「别的 app 会改设备音量」的源码级证据，比 2009 年的帖子强。
5. **DXOMARK 曲线跨页不可比**：同一台机器出现在不同页面，纵轴偏移不一样，不能拿来比电平。频段划分其实在页面里有（20–330 Hz / 330–3000 Hz / 3–20 kHz）。
6. **把「降噪自适应」和「自动增益」分开**：报告用作「增益时间常数」线索的 5 秒（Safari）、3 秒（Zoom）、1 秒（呼吸声），出处都把原因归到降噪。合成数据要把这两样分开建模。
7. **论坛样本要去重**：Zoom 社区里同一作者的模板式回复贴在两帖里，导致人数和「2 个组织」都多算了。建议按作者去重，并标出「疑似模板 / AI 生成文本」。
8. **能听的录音没去取**：pokde（M3 Air 15″ 的 MP3）和 Audacity 论坛（M1 MBP 的 m4a）都有录音。拿它们做频谱，可以和 DXOMARK 曲线交叉校验，小于 50 MB，可以本地分析。

我核对时存的文件都在 本会话临时目录 `scratchpad/skeptic2/`（不入库）：3 份 Chromium 源码、6 帖 Apple 社区、键盘声论文的 arXiv HTML（`ks.html`）。仓库没改，没碰麦克风。