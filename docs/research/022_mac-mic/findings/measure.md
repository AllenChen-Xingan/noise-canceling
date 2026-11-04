> workflow `wf_16dd3e65-ebc` · measure 路调研员原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

# Mac 内置麦录音：他人实测与说法（第②路）

## 要点

1. **MacBook Air 能找到的实测数字只有 DXOMARK 一家，每个机型只测了 1 台。** 测法：半消声室，声源距 1 m，用相机 app 录音，按 1/12 倍频程出频响。M2 MacBook Air 13″ 的结果：
   - 低频在约 163 Hz 降 3 dB，约 102 Hz 降 10 dB。
   - 6–11 kHz 上抬 +6 到 +9 dB。
   - 约 12.6 kHz 后开始衰减，16 kHz 处约 −13 dB。
   - 评语是「a bit thin-sounding」，背景噪声「not well attenuated」。
   - 这组数字是我从网页里嵌的图表数据算出来的。
2. **内置麦输出的那 1 路是有方向性的，侧面来声的高频被压得很重。** DXOMARK 指向性数据，M2 Air 13″ 在 90° 处：低频段 −3.4 dB、中频段 −13.6 dB、高频段 −31 dB。旁边说话的人会被录得又小又闷。这一条可以直接用在合成数据里。
3. **常见抱怨多数出在「另一层处理」上，不在麦本身。** 有三层会叠加：
   - 系统的语音突显（Voice Isolation）
   - 会议 app 自己的降噪
   - 老 Intel 机上系统级的「环境噪声降低」开关（Use ambient noise reduction）

   典型症状：音乐和乐器被压掉、钟声听不见、稳定的声音（哼唱、呼吸）约 1–5 秒后被压低、开头和结尾的字被吃掉、音量滑块自己上下动。
4. **Zoom 在 Mac 上确实会触发语音突显。** 多位用户说控制中心的麦克风模式里会列出「zoom.us」；Zoom 自己还会弹窗提示「turn off Voice Isolation」。有 2 例 M1 MacBook Air 是因为系统处在语音突显模式，导致 Zoom 的「Original Sound」不生效。
5. **Chrome 默认不走苹果语音处理（推断，依据源码）。** 只有功能开关 `kEnforceSystemEchoCancellation` 打开时，Chrome 才用 VPIO，而这个开关默认关闭。但 Chrome 会去开关内置麦的硬件降噪属性 `'nzca'`。Safari 和 Firefox 在要求回声消除时会走 VPIO。
6. **已知结论要补一条：「cpal 走 HAL 就完全不受系统处理」不成立。**
   - `'nzca'`（环境噪声降低）是设备级开关。Chrome 2017 年官方博客原话：「this setting is system-wide this will apply to all audio input streams from the same device」。
   - 别的 app 开了带自动增益的语音处理时，会改动设备的输入音量。2009 年 iChat / Skype 用户就报告过滑块随说话声自己移动。
7. **语音突显的伤害只有零散个人报告，没有找到任何量化测试。** 吃字首字（1 人）、压掉第二个人（无 Mac 实证，只有 Teams 官方承认它自家的同类功能会这样）、音乐被压（≥ 6 人）、延迟（未查到）。
8. **Reddit 和 YouTube 这次都没取到。** Reddit 拒绝抓取，搜索也没回出帖子。YouTube 只间接查到 1 个未具名的 M3 Air 对 M3 Pro 麦克风对比视频。已标为「未查到」。

---

## 表 1　嘈杂环境里录出来什么样（按机型分）

### MacBook Air · Apple 芯片

| 结论 | 原文或数字 | 出处 | 来源标记 | 样本量 |
|---|---|---|---|---|
| M2 Air 13″ 录音偏薄，背景噪声压得不够，指向性对通话不理想 | 「satisfactory tonal balance and intelligibility for voices, albeit a bit thin-sounding in capture. Background noises were not well attenuated and directivity was not optimal」 | https://www.dxomark.com/apple-macbook-air-13-2022-laptop-test/ | 评测网站 | 1 台 |
| M2 Air 13″ 双方同时说话时有门控（gating），伤可懂度 | 「some gating occurred that impaired voice intelligibility」 | 同上 | 评测网站 | 1 台 |
| M2 Air 15″ 只录单声道，电脑周围有多人时可懂度受损；扩散噪声处理得不错 | 「because of its mono recordings, intelligibility could be impaired when multiple people were placed around the computer. However, it handled diffused sounds well」 | https://www.dxomark.com/apple-macbook-air-15-m2-2023/ | 评测网站 | 1 台 |
| M2 Air 在纽约时代广场实地录音：清楚，但不如 16″ MacBook Pro | 「The recording was great but not as good as the MacBook Pro, so it's a good case for how much hardware matters」。没写是否开了降噪或语音突显 | The Verge / Vergecast，2023-04（读的是转载：https://suaralifestyle.blogspot.com/2023/04/your-laptops-microphone-might-sound.html ，原文未打开） | 评测网站（转载） | 6 台笔记本，1 位评测者 |
| M3 Air 15″ 安静房间里像电容麦，但很容易拾到环境噪声。**页面上有原始录音 MP3 可听** | 「quite prone to environmental noise」；录音：https://cdn.pokde.net/wp-content/uploads/2024/04/msg192994264-197673.mp3 | https://pokde.net/gadget/laptop/15-inch-apple-macbook-air-m3-review | 评测网站 | 1 台 |
| M3 Air 13″：Notebookcheck 对麦克风没有意见 | 「we have no complaints about its microphones」 | https://www.notebookcheck.net/Apple-MacBook-Air-13-M3-review-A-lot-faster-and-with-Wi-Fi-6E.811129.0.html | 评测网站 | 1 台 |
| M3 Air 在嘈杂环境下比 M1 Max MBP 好 | 只见搜索摘要，原文 403 未打开 | AppleInsider，2024-07-08 | 评测网站（未核实） | 1 人 |
| M1 Air 比 M1 Pro「更尖细」（tinny） | 「the Air sounds tinnier」，依据是 1 个 YouTube 对比视频（标题未查到） | https://forums.macrumors.com/threads/m1-air-vs-pro-audio-comparison.2270762/ | 论坛 / 博客，个人说法 | 1 人听 1 个视频 |
| 有人觉得 M3 Air 的麦比 M3 Pro 还好 | 「does the microphone on the MacBook Air sound better than the "Studio quality" microphone on the MacBook Pro?」，附 1 个视频（标题未取到） | https://forums.macrumors.com/threads/voice-isolation-feature-on-m3-macbook-air.2423732/ | 论坛 / 博客，个人说法 | 1 人 |

### MacBook Air · Intel

| 结论 | 原文或数字 | 出处 | 来源标记 | 样本量 |
|---|---|---|---|---|
| 老 Air 的「环境噪声降低」会让音乐录音变薄、发闷、颤动 | 「The resulting audio will sound thin, boxy, and a little wavery」（2014 年，Use Ambient Noise Reduction 开着） | https://www.macworld.com/article/223642/understanding-the-limitations-of-a-macs-microphone.html | 论坛 / 博客，专栏作者说法 | 1 人 |
| 2019 Intel Air 上 Zoom 降噪设为 High 也挡不住邻近办公室的人声 | 「my zoom room members can hear people talking in the offices, hallways, etc. that are next to me」 | https://community.zoom.com/rooms-workspaces-11/suppress-background-noise-not-working-on-macbook-air-52759 | 论坛 / 博客，个人说法 | 2 人（另 1 人用 M1 MBP） |
| 2011 Air 录到背景噪声而不是人声，桌面磕碰也录进去；回复者判断是硬件坏了 | 「picking up the background noise, and not my voice」 | https://discussions.apple.com/thread/5949916 | 论坛 / 博客，个人说法 | 1 人 |

### 其他 Mac（MacBook Pro 等）

| 结论 | 原文或数字 | 出处 | 来源标记 | 样本量 |
|---|---|---|---|---|
| MBP 14″（M2 Pro）默认不降噪，所以噪声高；降噪交给会议 app | 「Noise reduction is disabled by default, so noise levels were quite high. (The device relies more on videoconferencing apps…)」 | https://www.dxomark.com/apple-macbook-pro-14-2023-laptop-test/ | 评测网站 | 1 台 |
| MBP 14″（M3 Pro）没有降噪；双方同时说话时彼此听不见 | 「its microphones lack noise reduction」；「two people talking at the same time could not hear one another」 | https://www.dxomark.com/apple-macbook-pro-14-m3-pro-2023-laptop-test/ | 评测网站 | 1 台 |
| 16″ M1 Max MBP 头一转开就被对方说闷；Apple 社区专员说这是预期现象 | 「some muffled audio if you're not facing or speaking directly towards the internal microphone」 | https://discussions.apple.com/thread/254491270 | 论坛 / 博客，个人说法 | 1 位用户 + 1 位社区专员 |
| 开放办公室里 MBP 麦会拾到远处大声说话的人 | 「the macbook mic picks up louder people from across the room」 | https://news.ycombinator.com/item?id=43463867 | 论坛 / 博客，个人说法 | 1 人 |
| M1 MBP 对入门枪麦（Boya MM1）：风扇声照样进来，Mac 麦意外地好。**有 m4a 可听** | 「The fan comes through just fine」，用 QuickTime 录、自动电平 | https://forum.audacityteam.org/t/m1-macbook-pro-mic-vs-boya-by-mm1-mic/63696 | 论坛 / 博客，个人说法 | 1 台，2 人评 |
| 键盘声：从 Zoom 录下的 MacBook 键盘声，模型识别准确率仍有 93% | 「recorded using the video-conferencing software Zoom, an accuracy of 93%」。摘要没写明用的是不是内置麦（推断：是 MacBook Pro 内置麦） | arXiv 2308.01074 | 论文 | 1 台 MacBook Pro |

---

## 表 2　数字（频响、指向性、失真、增益）

| 结论 | 数字 | 测法 / 出处 | 来源标记 | 样本量 |
|---|---|---|---|---|
| **M2 Air 13″ 频响**（以 0.5–2 kHz 平均为 0 dB） | 100 Hz −10.1；200 Hz −1.7；低端 −3 dB 点约 163 Hz、−10 dB 点约 102 Hz；4k +3.1、6k +6.2、10k +7.6，峰值 10.7 kHz +9.4；高端 −3 dB 点约 12.6 kHz、−10 dB 点约 15.7 kHz；16k −13.4 | DXOMARK 网页内嵌 echarts 数据，我解析后计算。测量只从 80 Hz 起 | 评测网站（数据）+ 推断（计算） | 1 台 |
| **M2 Air 15″ 频响** | 100 Hz −3.8；低端 −3 dB 点约 102 Hz；峰值 8.8 kHz +9.5；10k +8.2；高端 −3 dB 点约 13.7 kHz、−10 dB 点约 15.7 kHz；16k −10.0 | 同上，https://www.dxomark.com/apple-macbook-air-15-m2-2023/ | 同上 | 1 台 |
| 作对照：MBP 14″（M2 Pro / M3 Pro） | 较平：100 Hz −4.4 / −2.2；高端 −3 dB 点 14.7k / 14.3k；16k −13.8 / −12.0 | 同上 | 同上 | 各 1 台 |
| 所有被测 Mac 在约 15–16 kHz 都陡降；同表的 Windows 机延伸到 17.9 kHz | 几台 Mac 的 −10 dB 点都在 15.3–15.7 kHz | 同上 | 推断（原因未知，也许是内部处理带宽） | 4 台 Mac |
| **M2 Air 13″ 指向性**（相对 0°，正前方） | 90°：低频段 −3.4、中频段 −13.6、高频段 −31.0 dB；180°：−4.0 / −13.4 / −33.8；270°：−3.8 / −8.6 / −22.5（左右不对称） | DXOMARK 内嵌极坐标数据；各频段的频率范围未给 | 评测网站 | 1 台 |
| **M2 Air 15″ 指向性** | 90°：−3.9 / −8.3 / −20.5；180°：−4.0 / −8.5 / −33.7 | 同上 | 评测网站 | 1 台 |
| 录音总谐波失真加噪声（THD+N），M2 Air 13″ | 200 Hz–8 kHz 的中位数 0.17%，最大 0.98% | 同上，13″ 页面 | 评测网站 | 1 台 |
| 同条件下 Air 13″ 录音电平比 MBP 14″ 低约 9.5 dB | 0.5–2 kHz 平均：−23.8 对 −14.3（单位只标「Amplitude (dB)」） | 同上 | 推断 | 各 1 台 |
| 裸麦硬件：3 个 PDM 全向麦，送到用户态前没有前置放大 | 「three Pulse Density Modulation (PDM) mics … with no preamplification. They are very sensitive and omnidirectional」 | https://asahilinux.org/docs/sw/audio-userspace/ | 官方文档（Asahi） | 覆盖 M1/M2 Air 13″、M2 Air 15″ 等 |
| **Asahi 给 Air 配的阵列几何与增益**（Asahi 自己的选择，不是 Apple 的） | 等边三角形，边长 2 cm（mic2 (2, 0) cm、mic3 (1, 1.73) cm）；增益 2 × 7.25 × 7.25 ≈ +40.4 dB；末端高通 120 Hz、Q 0.707；导向角 J313 120°、J413/J415 80°；最优频率 1100 Hz；48 kHz | `AsahiLinux/asahi-audio` `firs/j313|j413|j415/mic.json`，提交 ab1b46928d（2026-10-07），GPL-2.0 | 源码 | 3 款 Air 配置文件 |
| 灵敏度、最大声压、自噪声 | 未查到。Apple 不公开；拆机里也没查到 MEMS 型号 | 搜索 Apple 社区、iFixit、MacRumors | — | — |
| 自动增益时间常数 | 未查到数字。间接线索：WebKit 默认处理下哼稳定音高约 5 秒后音量降一半以上；Zoom「Original Sound」开后约 3 秒降噪又回来；Zoom 只保留呼吸声的前约 1 秒 | https://bugs.webkit.org/show_bug.cgi?id=262759 ；Zoom 社区 48059、16415 | 论坛 / 博客，个人说法 | 各 1 人 |
| 老款 iMac 内置麦增益 −16 到 +30 dB | 只见搜索摘要（Apple 社区 2156070），未打开 | — | 未核实 | — |

---

## 表 3　会议 app 在 Mac 上叠加的处理

| app | 结论 | 原文或数字 | 出处 | 来源标记 | 样本量 |
|---|---|---|---|---|---|
| Zoom | 控制中心麦克风模式列出「zoom.us」，可选标准 / 语音突显 / 宽频谱 | 「It will show the app currently using the microphone (e.g., "zoom.us")」 | https://community.zoom.com/meetings-2/original-sound-for-musicians-issues-mac-15516 ；…-18153 | 论坛 / 博客，个人说法 | ≥ 3 人 |
| Zoom | Zoom 自己弹窗让用户关语音突显；用户找不到入口 | 「Audio sharing works best when "Voice Isolation" is turned off…」 | https://community.zoom.com/meetings-2/turn-off-voice-isolation-so-i-can-share-computer-audio-41632 | 论坛 / 博客（引了 Zoom 界面原文） | ≥ 6 人 |
| Zoom | 由上两条推出：Zoom Mac 版用苹果语音处理（VPIO），所以会触发系统语音突显 | — | — | 推断（证据较强） | — |
| Zoom | Mac 版降噪关不掉；Auto/Low/Medium/High 效果一样；「Original Sound」不生效 | 「the noise suppression for low, medium, high are all the same」；「Windows zoom has more options and can completely disable the noise suppression, but the mac version doesn't」 | Zoom 社区 16415、18153、11754、133990 | 论坛 / 博客，个人说法 | ≥ 7 人（M1 Pro、M3、M4 MBP 等） |
| Zoom | **M1 Air 的例子**：一台系统默认就处在语音突显，Original Sound 无效；切到标准就好了 | 「We noticed this issue on one of our M1 MacBook Airs but not others, even though the settings looked identical」 | Zoom 社区 15516、18153 | 论坛 / 博客，个人说法 | 2 个组织，各 3 台左右 |
| Zoom | Zoom 网页版不压、Voice Memos 不压，只有 Mac 客户端压 | 「If I use the web version of zoom, it works fine…」 | Zoom 社区 48059、11754 | 论坛 / 博客，个人说法 | 1 人（M1 Pro） |
| Zoom | Zoom 的虚拟声卡设备让麦克风模式变灰 | 「Mic Modes is greyed out … due to lack of support for Core Audio AUVoiceIO in Apple's DriverKit」 | https://devforum.zoom.us/t/enable-mic-modes-macos-12/67915 | 论坛 / 博客，个人说法 | 1 人 |
| Teams | 2021 年起 Mac 版有 AI 降噪：Auto / High / Low / Off；当时 M1 不支持 | 「This feature is not yet available for Mac machines with an M1 processor」 | https://techcommunity.microsoft.com/…/2281754 | 官方文档 | — |
| Teams | 2024-10 起 Mac 版有基于声纹的「voice isolation」（要先录声纹，管理员要开策略），会压掉其他人声 | 「eliminates unwanted background noise, including other human voices」 | https://mc.merill.net/message/MC891240 | 官方文档 | — |
| Teams | 它自家的声纹隔离会压近旁同事，并会弹提示让用户本场关掉 | 「identifies when a speaker very close to the microphone is being suppressed」 | Microsoft Teams 博客 4096077（只见搜索摘要） | 官方文档（未打开原文） | — |
| Teams | Teams 是否开 VPIO、控制中心是否列出 Teams | 未查到 | — | — | — |
| Chrome / Google Meet | Chrome 只有功能开关 `kEnforceSystemEchoCancellation` 打开时才用 VPIO，该开关 `FEATURE_DISABLED_BY_DEFAULT` | `media_switches.cc:1583`；`audio_low_latency_input.cc:192-205, 955-993` | Chromium main，2026-11-03 读取（`media/base/media_switches.cc`、`media/audio/apple/audio_low_latency_input.cc`） | 源码 | — |
| Chrome | 不走 VPIO、且网页没要求降噪时，Chrome 会关掉设备的 `'nzca'`（环境降噪），用完再恢复 | `audio_low_latency_input.cc:772-777`；`audio_manager_mac.cc:109-110, 1214-1250` | 同上 | 源码 | — |
| Chrome | 2017 年：`'nzca'` 是整台设备共用的开关，影响同一麦的所有流；只在 Sound 面板里有该开关的设备上存在 | 「Since this setting is system-wide this will apply to all audio input streams from the same device」 | https://developer.chrome.com/blog/disabling-hardware-noise-suppression/ | 官方文档 | — |
| Chrome + Meet | 用户报告：开了语音突显，Chrome 里的 Meet 录像仍有噪声 | 标题「Voice isolation is enabled but does not work with Google Meets on Chrome」（正文 429 未取到） | https://discussions.apple.com/thread/255661299 | 论坛 / 博客，个人说法 | 1 人 |
| Safari + Meet | Safari 里的 Meet 会出现麦克风模式 | TWiT 节目演示 | https://twit.tv/posts/transcripts/hands-mac-144-transcript | 论坛 / 博客（播客） | 1 人演示 |
| Safari | Safari 的 getUserMedia 默认处理下，哼稳定音约 5 秒后音量降一半以上；把三项处理（AGC / AEC / NS）全关就好；WebKit 判为「预期行为」 | 「the volume … diminishes in volume (maybe by more than 50%) after about 5 seconds … only … when humming a fixed pitch」 | https://bugs.webkit.org/show_bug.cgi?id=262759 | 官方（WebKit 缺陷库） | 1 人，Apple 芯片 MBP |
| Firefox | 要求回声消除时开 VPIO；面板时有时无 | 已在 `macos-mic.md` #6/#19 | — | — | — |
| FaceTime | 语音突显、宽频谱需「Mac models 2018 and later」 | 原文同左 | https://support.apple.com/guide/facetime/change-audio-options-fctme7c07113/mac | 官方文档 | — |
| Discord | 是否支持 Mac 麦克风模式，说法冲突 | 帖子标题「Discord doesn't support MacOS mic modes」；搜索摘要里也有人说新 Mac 上能看到 | support.discord.com 社区 4431095662487、4413340958999（403，未打开） | 论坛 / 博客，个人说法（未核实） | 未知 |
| Discord（Krisp） | Mac 版 Krisp 要 Apple 芯片、macOS 12+ | 第三方指南，2026-09 | https://www.technobezz.com/how-to-fix-discord-noise-suppression-not-working | 论坛 / 博客（未核实） | — |
| Slack huddles | 有自带降噪开关；是否开 VPIO | 未查到 | https://slack.com/help/articles/1500002037922 | 官方文档（只证明有开关） | — |
| 老 Intel Mac | 没有麦克风模式的老 Mac，要在 Sound 面板取消「Use ambient noise reduction」 | 「Older Macs don't have a Voice Isolation setting … make sure "Use ambient noise reduction" is unchecked」 | https://blog.farplay.io/?p=1592 | 论坛 / 博客（厂商博客） | — |

---

## 表 4　语音突显和相关处理的伤害报告

| 伤害 | 原文或数字 | 出处 | 来源标记 | 样本量 / 有无录音 |
|---|---|---|---|---|
| **吃开头和结尾的字**，开口时有爆裂声（Intel 2019 MBP 16″ + Scarlett 外接声卡） | 「It absorbs word beginnings and endings and brings in terrible cracking noise when you start speaking」 | https://obsproject.com/forum/threads/macos-sonoma-voice-isolation-killing-mic-sound-and-freezes-obs.171263/ | 论坛 / 博客，个人说法 | 1 人，无录音 |
| 音乐、钢琴、吉他、钟声被压 | 「No one could hear the bell…」（MacBook Air 15″，macOS 14.1）；「voice would be recorded cleaning but guitar suppressed entirely」 | https://discussions.apple.com/thread/255243669 ；Zoom 社区 41632、15516、18153 | 论坛 / 博客，个人说法 | ≥ 6 人，其中 Air 3 例 |
| 声音变小，学生听不见（Air 上 Zoom 教声乐） | 「my students are complaining they can't hear me. I'm too quiet」 | Zoom 社区 15516 | 论坛 / 博客，个人说法 | 1 人（Air） |
| 呼吸声只保留前约 1 秒，其余被压 | 「zoom only picks up the first second of the breath and then suppresses the rest」 | Zoom 社区 16415 | 论坛 / 博客，个人说法 | 1 人（M3） |
| 选了「标准」后几秒自己跳回语音突显 | 只见搜索摘要（Zoom 社区） | — | 论坛 / 博客（未核实） | 1 人 |
| 多人在场时压掉第二个人 | Mac 上未查到直接实证。最接近的是：DXOMARK 说单声道录音让周围多人时可懂度受损；Teams 官方承认它自家声纹隔离会压近旁同事；Zoom 用户说学生笑声盖住主讲（与语音突显无关） | 见表 1、表 3 | 评测网站 / 官方文档（Teams） | — |
| 双方同时说话时被门控 | MBP M3 Pro：「two people talking at the same time could not hear one another」；Air 13″：「some gating occurred」 | DXOMARK | 评测网站 | 各 1 台 |
| 自动增益让音量滑块自己动（**与是否开语音突显无关，2009 年就有**） | 「the slider moves left when he speaks louder … only while Skype is running」；iChat 日志显示回声消除反复把音量拉到 0 | https://discussions.apple.com/thread/2129454 | 论坛 / 博客，个人说法 | 约 5 人，Intel 2008 MBP 等 |
| macOS 26 Tahoe 后输入电平自己动 | 「the input level is moving by itself up and down while I talk」；绕过方法是建聚合设备 | https://discussions.apple.com/thread/256152544 | 论坛 / 博客，个人说法 | 1 人 + 4 个「我也是」 |
| Air 13″ 输入音量隔半天自己下降（macOS 15.6） | 「after a while it decreases by itself」 | https://discussions.apple.com/thread/256125271 | 论坛 / 博客，个人说法 | 1 人，附视频 |
| 别的进程在同一只内置麦上开了 VPIO，不经 VPIO 读到的音量很低 | 已在 `macos-mic.md` #11 | — | 源码 | — |
| 机器人声、发闷 | 只有第三方指南泛泛而谈（Mackie、Focusrite、UA 的帮助页都 403，只见搜索摘要） | — | 论坛 / 博客（未核实） | — |
| 语音突显的延迟 | 未查到可核实的数字 | — | — | — |
| 开 / 关语音突显的录音对比（可听） | 未查到 | — | — | — |

---

## 对合成数据模拟的启示（全部是推断）

- **频响滤波：**高通的 −3 dB 点约 100–160 Hz（Air 13″ 偏高），4–11 kHz 有 +3 到 +9 dB 的宽峰，约 15 kHz 以上陡降。可以用 DXOMARK 的 250 点曲线直接做成 FIR 滤波器。
- **指向性：**旁边说话的人和侧后方的噪声，按角度加一个低通衰减（90° 时高频段约 −20 到 −31 dB、中频段约 −8 到 −14 dB），不要按全向来加。
- **三种处理状态都要覆盖：**（a）标准模式、不降噪（MBP 类）；（b）系统语音突显（会出现稳定声音几秒内被压低、吃首尾字）；（c）会议 app 降噪再叠一层。
- **增益起伏：**加入秒级的增益漂移（约 1–5 秒时间常数，依据只有个人说法）。

---

## 没查到 / 仍不确定

1. Reddit（r/macbookair、r/mac、r/podcasting、r/audioengineering）：抓取被拒，搜索也没回出帖子。**未查到。**
2. YouTube 麦克风对比视频的标题和频道：只间接知道 MacRumors 两帖各引了 1 个视频，标题没取到。**未查到。**
3. RTINGS、AnandTech、The Verge 评测正文里对 Air 麦克风的描述：只读到 The Verge 时代广场测试的转载。**其余未查到。**
4. 灵敏度、最大声压、自噪声、MEMS 型号、Apple 自动增益的时间常数、VPIO 和语音突显的附加延迟：**未查到。**
5. DXOMARK 用的「相机 app」当时处在哪种麦克风模式：页面没写；高、中、低三个频段的频率划分也没写。曲线**不能**当作「原始阵列」，只能当作「Apple 默认那 1 路」。
6. DXOMARK 只覆盖 M2 Air 13″/15″。**M1、M3、M4 Air 和所有 Intel Air 都没有实测曲线。**
7. Apple 芯片的 Air 有没有 `'nzca'` 这个属性（也就是 cpal 走 HAL 时会不会被硬件降噪处理过）：要本机只读查 `AudioObjectHasProperty`。按禁令本次没跑。
8. Teams、Discord、Slack 在 Mac 上是否开 VPIO、控制中心是否列出它们：**未查到**（Discord 只有相互冲突的帖子标题）。
9. Chrome 的 `kEnforceSystemEchoCancellation` 是否通过服务端实验（Finch）在部分用户那里被打开：源码默认关，线上实际状态**未查到**。
10. 吃首字、压第二个人这两项，Mac 上都只有个人说法或旁证，**没有任何一份可听的开 / 关对比录音**。
11. 键盘声论文是否用的 MacBook 内置麦：摘要没写明，**未核实**。

---

**DXOMARK 数据怎么来的：**我从页面内嵌的 echarts 数据里抽出数值再计算。解析脚本在 本会话临时目录 `scratchpad/dxofr.py`（不入库），原始网页在 本会话临时目录 `scratchpad/web/`（不入库）。

**本次范围：**只读网页和源码，没有下载超过 50 MB 的文件，没碰麦克风，没改仓库。