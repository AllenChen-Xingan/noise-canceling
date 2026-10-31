# 问题 6 · 商业产品怎么处理「一大堆人同时说话」（commercial）

> 调研日 2026-10-30 · 只读网页，未运行任何程序 · 已有 005/006 结论不重复（Krisp viva-tel-lite 15 ms、Teams 云端主档案、Apple 不能程序化打开 等只引用不展开）

## 一句话结论

**没有一家把「一大堆人同时说话」当默认要解决的事**：默认档一律是「去噪、保留人声」（Teams Auto / Zoom 降噪 + Auto / Webex Noise removal / Meet 降噪，Meet、Discord、Teams 的 High 都**明说不去掉人声**）；「只留我」都是**另开的一档**，靠两条路之一认出你——**① 注册声纹**（Teams、Zoom）或 **② 近讲假设**（Krisp：必须戴耳麦且在封闭空间；Webex：离麦 ≤ 1 m、再叠一个 2 s 窗口的说话人识别）。多人共用一台设备时，所有家的说法都一样：**关掉「只留我」**（Teams 检测到身边有人会弹提示；Webex 专门另设「Optimize for all voices」给会议室）。

## 一、模式划分总表（厂商给用户的档位）

| 产品 | 只留我 | 降噪·保留他人 | 增强所有人声 | 原声 | 默认 | 出处 |
|---|---|---|---|---|---|---|
| **Microsoft Teams** | Voice isolation（需声纹） | Noise suppression：Auto / Low / High（High「suppresses all background sound that isn't speech」）；会中菜单「Background noise only」 | — | 关掉 Noise suppression 开关（建议用于安静录音室的高保真麦） | Noise suppression 开、Auto；**有声纹后 Voice isolation 自动生效**（「Voice isolation will be active in your calls and meetings after you add your voice profile」） | 【官方】support.microsoft.com 1a9c6819…；a9756ea9… |
| **Zoom** | Personalized audio isolation（声纹，「Recommended for crowded environments」）；另：**High 档「can also remove background speech from others」** | Noise removal + Auto / Low / Medium | — | Original sound for musicians（关滤波，可再关回声消除、开 48 kHz） | Noise removal、背景降噪 **Auto** | 【官方】support.zoom.com KB0059985 |
| **Cisco Webex** | **Optimize for my voice**：「Removes all noise and background voices」，适用 open office | **Noise removal**：「Removes all noise detected on your side」 | **Optimize for all voices**：「Removes all noise and enhances background voices」，适用 **group conversations in a conference room** | Music mode；PSTN 来电另有「Original」 | **Noise removal (Default)**；选过一次后成为以后会议默认 | 【官方】help.webex.com yrsw3l |
| **Krisp（桌面）** | Background Voice Cancellation（BVC；新版帮助中心改叫 Voice Isolation / Agent Voice Isolation） | Noise Cancellation（BVC 设 Always OFF 后「噪声仍去、人声保留」） | — | 关 Krisp | **认证耳麦上 Auto 开**（会议开始自动开）；内置麦 / 3.5 mm 由管理员设「User's choice」默认关 | 【官方】help-jp.krisp.ai/background-voice-cancellation；英文版 help.krisp.ai 403（见 gap） |
| **Google Meet** | **无**（官方：「Voices from TV or people talking won't be canceled」） | Noise cancellation（设备端 / 云端可切换） | — | 关掉 | 多数 Workspace 版本默认开，教育版默认关；Gemini 用户由 Studio sound 取代 | 【官方】support.google.com/meet/answer/9919960 |
| **Apple（macOS/iOS Mic Mode）** | 无明确「只留我」；Voice Isolation =「Prioritizes your voice and blocks ambient noises」，**是否压他人人声官方未说** | Voice Isolation（同左） | — | Standard / Wide Spectrum（「Leaves ambient noises unfiltered」） | 官方页不写默认；iOS 18+ 有 Automatic（听筒用 Voice Isolation、免提用 Standard） | 【官方】support.apple.com/101993；mac-help mchle82b42f0 |
| **AirPods 语音隔离** | 同上，只说「remove background noise (such as wind) for the listener」 | — | — | — | 未写 | 【官方】support.apple.com/guide/airpods/devac96438de |
| **NVIDIA Broadcast（App）** | **无** | Noise and Echo Removal（例子只有键盘、风扇、混响） | — | 关 | — | 【官方】nvidia.com broadcast-app |
| **NVIDIA Maxine AFX SDK** | **Speaker Focus（Early Access）**：「identifies and isolates the prominent speaker… removes the speech of all other speakers」，**最多 4 人**、单声道、16/48 kHz | Noise removal（BNR） | — | — | — | 【官方】docs.nvidia.com/maxine/afx …AboutSpeakerFocusEffect |
| **Discord（内置 Krisp）** | **无** | Krisp：去掉「background noises that aren't human voices」 | — | 关 | — | 【官方】support.discord.com/360040843952（原页 403，取自搜索摘要） |

## 二、「怎么判主讲人」逐家证据

| 产品 | 判据 | 要不要注册 | 关键数字 / 条件 | 标记 · 出处 |
|---|---|---|---|---|
| Teams Voice isolation | 「personalized deep voice quality enhancement」模型，去掉「don't match the profile」的声音 | **要**。手动读一段约 30 s；或 Express 被动注册「typically takes a couple of meetings」 | 主档案存 Office 365 compliance store，本机加密副本 14 天过期；一年不用自动删；25 种语言（含 zh-cn） | 【官方】learn.microsoft.com/microsoftteams/voice-isolation；…/rooms/voice-and-face-recognition |
| Teams（调参取向） | 「trade-off… leakage vs. over-suppression」，**刻意选宽容**：声音相近的家人近距离说话**可能漏过** | — | 训练评测 >2000 h，无公开准确率 | 【官方博客】Teams Blog ba-p/4096077（读的是 txstate 转载） |
| Zoom Personalized audio isolation | 用 voiceprint 区分你的声音 | **自动**：同意后在会议中自动采集，**存本机**；可选再读一段脚本，**该录音由 Zoom 存储**直到删除 | 不支持公共区域设备与老设备 | 【官方】support.zoom.com KB0074698（搜索摘要）· KB0059985；Krisp 博客转述【传言】 |
| Krisp BVC | 「primary speaker detected by **speaker-to-microphone proximity cues**」（viva-tel-lite 博客）；桌面版要求「Enclosed space」+「Compatible headset」，USB 有线 / 长杆麦最好 | **不要**（2022 起；旧版 Voice Cancellation 要录音，已被替代） | 算法延迟 15 ms、16 kHz（服务端 BVC-tel/app 同为 15 ms）；Mac 1.44.1「less adaption time required」→ 说明在线自适应 | 【官方】krisp.ai/blog/small-voice-isolation-model；…/improving-turn-taking…；launchnotes ann_3Py46lEkWVlr9、ann_BeP8BhHunS4CW |
| Webex Optimize for my voice | 「focuses on the speaker **closest to the microphone**」；BabbleLabs 团队海报：主讲人须 **≤ 1 m**，干扰者是「reverberated background speech」；再叠说话人识别（SI）+ 干扰说话人检测（ITD） | **不要**（自适应主讲人检测与跟踪） | SI 网络：MobileNetV2 类、Common Voice 850 人 / 170 万条训练、ArcFace 损失、**感受野 2 s**、96 维嵌入、互相关阈值 0.8（0.9 时误检 196→8）；海报表：无 SI 时 1–2 m「word chopping」、2–3 m 被压掉，加 SI 后 1–2 m 保住、2–3 m 吃字；挑战原文「Speaker ID network introduces the decision latency」 | 【官方】cisco.com audio-intelligence-webex-calling-aag；【论文/海报】Cisco BabbleLabs「Optimize for my voice with speaker identification」，sigport fid=8965（会议名海报上未写） |
| NVIDIA Speaker Focus | 「prominent speaker」，判据未公开 | 未写 | ≤ 4 人；无延迟数字；EA | 【官方】docs.nvidia.com/maxine/afx |
| Apple Voice Isolation | 只说「机器学习」 | 无 | — | 【官方】见 006 |

## 三、提示文案与交互（只摘官方）

- **Teams**：检测到身边有人（「you and a colleague sharing a device」）时弹通知，告诉你**要让对方被听到就关掉 voice isolation**；博客说该开关**只关本场**。会中指示有 5 种状态：开且生效 / 静音入会 / 只显示你的名字（未检测到杂音）/ 正在过滤 / 检测到身边有人。错误码「ufd2」= 设备开了太多应用。【官方】support.microsoft.com a9756ea9…
- **Krisp**：图标显示 BVC 状态；不认识的设备显示警告图标，可勾「Force Activation」，并提示质量可能下降；官方建议通话前**让人在背后说话测一下**。【官方】help-jp.krisp.ai
- **Google Meet**：声音指示圈「圈越大，去掉的噪声越多」；音乐、电子喉用户建议关。【官方】
- **Webex**：未见检测提示；选过的档位记为以后会议默认。【官方】
- **Zoom**：未查到自动提示文案。

## 四、对本项目的含义

1. **模式划分可以直接抄 Webex 的四档结构**（只是结构，不是文字）：
   - 只留我（≈ Optimize for my voice）
   - 降噪（保留他人）（≈ Noise removal）——**默认**，与全行业一致
   - 多人同用（≈ Optimize for all voices，会议室 / 共用 MacBook）
   - 原声（≈ Music mode / Wide Spectrum）——同时也是 D03「出事送原声」的落点
   【推断】把「多人同用」单列出来，比 Teams「弹窗让你关掉」更不容易误压，也方便场景层自动切换。
2. **「只留我」的判据，行业两条路我们都能用，而且能叠加**：
   - 近讲（Krisp、Webex）：**不用注册、不加延迟**，但前提是戴耳麦 / ≤ 1 m。Krisp 对内置麦默认不开 → 【推断】MacBook 内置麦 + 邻桌 babble 正是近讲判据最弱的情况，exp003 失败片段很可能落在这里。
   - 声纹（Teams、Zoom）：都在 M1 的「自写注册器 + 不对称门控」路线上；Zoom「会中自动采、存本机」最贴合我们的隐私定位，Teams 的「云端主档案」是反例（005 已记）。
   - Webex 海报证明**近讲 + 2 s 说话人识别叠加**能把 1–2 m 吃字救回来，而且它自己承认 SI 带来「decision latency」→ 这正好对应我们 **0.5 s 一次的场景判断回路只改参数**：2 s 窗口的识别放判断回路，不进音频路径，**不占 45 ms 预算**。
3. **延迟**：唯一公开数字是 Krisp 15 ms 算法延迟（16 kHz）；GTCRN 约 32 ms 已比它大。【推断】若 M2 要叠一个「只留我」模型，留给它的只有约 45 − 32 − 管线 6–10 ≈ 3–7 ms，**只能并联做掩码 / 增益门控，不能串联第二个带前瞻的模型**。
4. **算力**：没有任何家公布「只留我」的 CPU 占比；Teams 有「设备开太多应用就关」的错误码，Zoom 说 High「may increase CPU utilization」→ 侧证这一档比普通降噪贵。
5. **许可**：全部闭源，**只借鉴交互与判据，不能借代码**；Krisp 基准 CC BY-NC（005 已记）。
6. **落点**：
   - **M1**：默认档=降噪保留他人；「只留我」作为可选档（门控只压旁人单独说话）；Teams 式「检测到身边有人」提示 + 一键本场关闭。
   - **M2**：近讲 + 声纹叠加（Webex 式），处理同时说话；评测加 Webex 海报那种「距离 0.5 / 1–2 / 2–3 m」格子。
   - **场景层**：耳麦 vs 内置麦是第一判据（Krisp 按设备白名单决定 Auto）；检测到多个近距离说话人→建议切「多人同用」。

## 五、gap

| 缺什么 | 状态 |
|---|---|
| Krisp 英文帮助中心 BVC 正文、兼容设备列表 | help.krisp.ai 返回 403，只读到日文镜像；需人用浏览器打开核对 |
| Krisp BVC 在 MacBook 内置麦 + babble −5 dB 的真实表现 | **需另开实验**（人手动装 Krisp 免费版做 AB；Krisp 不能进训练） |
| Apple Voice Isolation 是否压旁人人声 | 官方未说；**需另开实验**（006 已列「Apple 当商业对照」，加一段「旁边有人说话」片段即可） |
| macOS 26/27 Mic Mode 新功能 | 未查到官方新档位；macOS 帮助页仍只有 Standard / Voice Isolation / Wide Spectrum；iOS 18+ 的 Automatic 是否到 Mac 未查到 |
| Teams / Zoom「只留我」延迟、CPU | 未查到 |
| Webex 海报的会议名、年份 | 海报上未写，sigport 链接 fid=8965 |
| Zoom 自动 voiceprint 需要几场会议 | 未查到 |
| Google Meet Studio sound 是否去人声 | 未查到 |
| Discord 原页 | 403，只有搜索摘要 |
| NVIDIA Speaker Focus 判据与延迟 | 文档未写 |

## 六、来源清单

- Teams：https://learn.microsoft.com/en-us/microsoftteams/voice-isolation 【官方】· https://learn.microsoft.com/en-us/microsoftteams/rooms/voice-and-face-recognition 【官方】· https://support.microsoft.com/en-gb/teams/meetings/voice-isolation-in-microsoft-teams-calls-and-meetings 【官方】· https://support.microsoft.com/en-us/office/reduce-background-noise-in-teams-meetings-1a9c6819-137d-4b3b-a1c8-4ab20b234c0d 【官方】· Teams Blog ba-p/4096077（经 https://digitaltrends.wp.txstate.edu/?p=2951 转载）【官方博客·转载】
- Zoom：https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0059985 【官方】· KB0074698（搜索摘要）【官方】· https://krisp.ai/blog/zoom-voice-isolation/ 【传言·竞品转述】
- Webex：https://help.webex.com/en-us/article/yrsw3l 【官方】· https://help.webex.com/en-us/article/b3kqr9 【官方】· https://www.cisco.com/c/en/us/products/collateral/unified-communications/webex-calling/audio-intelligence-webex-calling-aag.html 【官方】· https://sigport.org/sites/all/modules/pubdlcnt/pubdlcnt.php?fid=8965 （Cisco BabbleLabs 海报）【论文】
- Krisp：https://help-jp.krisp.ai/background-voice-cancellation 【官方】· https://krisp.ai/blog/small-voice-isolation-model/ 【官方博客】· https://krisp.ai/blog/improving-turn-taking-of-ai-voice-agents-with-background-voice-cancellation/ 【官方博客，自测】· https://krisp.launchnotes.io/announcements/ann_3Py46lEkWVlr9 · ann_BeP8BhHunS4CW 【官方】
- Google Meet：https://support.google.com/meet/answer/9919960 【官方】
- Apple：https://support.apple.com/en-asia/101993 【官方】· https://support.apple.com/guide/mac-help/mchle82b42f0/mac 【官方】· https://support.apple.com/guide/airpods/devac96438de/web 【官方】· https://www.apple.com/newsroom/2025/06/airpods-now-more-versatile-with-studio-quality-audio-recording-and-camera-remote/ 【官方】
- NVIDIA：https://www.nvidia.com/en-us/geforce/broadcasting/broadcast-app/ 【官方】· https://docs.nvidia.com/maxine/afx/latest/AboutTheEffects/AboutSpeakerFocusEffect.html 【官方】· …/AboutNoiseRemovalBackgroundNoiseSuppression.html 【官方】
- Discord：https://support.discord.com/hc/en-us/articles/360040843952 【官方，403，取自搜索摘要】
