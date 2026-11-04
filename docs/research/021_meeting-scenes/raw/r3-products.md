# 原始报告 · r3-products（workflow wf_c790c606-7d6 · 2026-11-03，未经合成，供追溯）

## 要点（5 行）

1. 本报告查了 8 家产品。没有一家公开过"按开会地点分场景"的降噪评测。只有 Krisp 公开了一套按使用场景分组的测试集：3 个使用场景、265 条真实录音，并报了每组的字错率（WER）。其余各家只列"能去掉哪些声音"，按场景的结果都未查到。
2. 各家声称最多的是"家里客厅"类噪声：狗叫、小孩、吸尘器、厨房声、门铃、割草机。其次是开放工位里的旁人说话和键盘声。
3. 有 3 家把"只留自己的声音"当成卖点：Teams 的语音隔离（Voice Isolation）、Zoom 的个性化音频隔离（Personalized Audio Isolation）、Webex 的"只优化我的声音"（Optimize for my voice）。点名的地点是开放工位、咖啡馆和候机。Webex 和 Teams 都写明：多人共用一台设备或在会议室时，要换模式或关掉它。
4. Google Meet 官方明说：电视里的人声和旁人说话不会被去掉，音乐会被当成噪声。所以"家人看电视"这类场景，Meet 的标准降噪是公开声明不管的。
5. 14 个场景键里，下面 6 个没有任何厂商点名：电话亭（phone-booth）、共享办公（cowork）、餐厅（canteen）、酒店（hotel-room）、教室/图书馆（library-classroom）、高铁/飞机客舱（rail-air-cabin，只有 Krisp 在噪声类型里提了一句"客舱轰鸣" cabin rumble）。这几格只能靠我们自己定义。

> 说明：引文由网页抓取工具摘录，逐字以原页为准。标【官方文档】的是我这次直接打开过的原页。标"二手"的是搜索摘要或媒体转述，原文未核。

---

## 一、产品 × 声称的噪声 / 地点

### 1. Krisp
- **地点**【官方文档】https://krisp.ai/noise-cancellation/ ：原页写 "whether it be your office, a noisy coffee house, or a crowded call center"，还写 "home office, cafe, or busy workspace"。
- **噪声**【官方文档】同页："typing, street noise, or chatter"，以及去除 "background voices"（背景人声）。
- **其他功能**【官方文档】同页：
  - 语音隔离（Voice Isolation）："eliminating all other nearby voices"
  - 回声消除（Echo Cancellation）："room echo and acoustic reflections"
  - 口音转换（accent conversion）：说话方、收听方两侧都有。和地点无关。
- **旧首页**【二手，原文未核】2021 年存档首页的演示选项里有 Barking Dog、Street and Traffic Noise、Crying Baby。来源是搜索摘要：https://webcf.waybackmachine.org/web/20210108101726/https://krisp.ai ，我直接打开时 SSL 握手失败。
- **按场景分组的测试集**【官方文档·厂商自测】https://krisp.ai/blog/speech-recognition-dataset-background-voices/ （"Location Types" 一节和样本表），数据集在 https://huggingface.co/datasets/Krisp-AI/VoiceIsolation-Benchmark-Dataset 。

| 使用场景 | 说话人 | 录音条数 | 设备 | 地点 |
|---|---|---|---|---|
| Work（工作） | 29 | 96 | 24 台 | 小房间、中房间、大会议室、开放工位层（open-plan floors） |
| Call center（呼叫中心） | 7 | 106 | 18 个耳麦 | 4 个真实呼叫中心 |
| Phone calls（手机通话） | 26 | 63 | 18 款手机 | 嘈杂楼宇、商店、街道、地铁站 |

  - 噪声类型：工作和呼叫中心两组是 "office chatter, keyboards, music, babble"；手机通话组是 "Street noise, music, cabin rumble, crowd chatter and telephony codecs"。
  - 总时长：未查到。
- **结果**【官方文档·厂商自测，无独立复现】https://krisp.ai/blog/voice-isolation-benchmark/ ，表 "Word error rate by scenario"。数字是"原始音频 → 语音隔离 VI 2.5 Default"的字错率（WER），测量条件是 11 套语音识别配置。

| 使用场景 | 原始 | VI 2.5 Default |
|---|---|---|
| Work | 31.84% | 6.30% |
| Call center | 23.83% | 6.86% |
| Phone calls | 3.48% | 3.91%（变差） |
| 全部 265 条合并 | 23.24% | 6.24% |

  - 原文把三组环境描述为：Work 是 "Offices, homes, meeting rooms"，Phone calls 是 "Cars, streets, shops, metro"。
  - 每条录音带 `environment` 字段，但**没有按地点细分的结果**。每组各多少条参与评分，原文也没写。
  - 另有一篇 Digg 转述给的呼叫中心数字是 23.93% → 6.91%，和表里不一致【二手，原文未核】。

### 2. NVIDIA Broadcast / Maxine
- **产品页**【官方文档】https://www.nvidia.com/en-us/geforce/broadcasting/broadcast-app/ ："Loud keyboard typing. Annoying microphone static. Bothersome PC fans. Nagging background reverb." 环境只有一句："without worrying about finding a quiet place"。
- **1.2 版更新**【官方文档】https://www.nvidia.com/en-us/geforce/news/may-2021-nvidia-broadcast-update/ ：
  - 降噪新增 "cats, dogs and even insects (cicadas be gone!)"
  - 去房间回声针对 "bedrooms, living rooms and kitchens"
- **Maxine 开发包的噪声清单**【官方文档】https://docs.nvidia.com/maxine/afx/2.0.0/Appendix/DenoiserSupportedNoises.html ，附录 "Types of Background Noise Removed"，共 30 类：
  - AC noise · Babble and crowd noise · Baby crying · Bird chirping · Body noises · Chatter from other people
  - Clapping · Construction site sounds · Cooking sounds · Door slamming · Drums · Fan noise
  - Furniture moving · Gaussian/white noise · Glass breaking · Keyboard · Metal sounds · Mouse clicks
  - PC noise · Pet sounds · Phone ringing · Rains · Sirens · Tapping
  - Traffic noise · Train passing · Vacuum cleaner · Washing machine · Water taps · Wrappers rustling
  - 最新版的同名附录直接打开返回 404。
- 这是唯一一份公开的、细到类别的噪声清单，但它按噪声分类，不按地点分类。

### 3. Microsoft Teams
- **2020 年 AI 降噪博客**【官方文档】https://techcommunity.microsoft.com/t5/microsoft-teams-blog/reduce-background-noise-in-microsoft-teams-meetings-with-ai/ba-p/1992318
  - 噪声：键盘、零食包装袋、吸尘、"your child's conflicting school lesson"、厨房声、冲马桶、打鼾
  - 场景："multiple meetings occurring in a small space"、"children playing loudly nearby"、"construction noise outside of your home office"
  - 训练数据：约 760 小时干净语音 + 180 小时噪声，150 种噪声，10 种以上语言；房间是 3000 多个真实房间 + 11.5 万个合成房间
  - 评测：众包 1–5 星主观打分（MOS）。按场景分组的结果未查到。
- **档位说明**【官方文档】https://support.microsoft.com/en-us/office/reduce-background-noise-in-microsoft-teams-meetings-1a9c6819-137d-4b3b-a1c8-4ab20b234c0d
  - Auto：适合多数情况
  - Low：持续的低噪声，"computer fan or air conditioner"
  - High："all background sound that isn't speech"
  - Off："quiet studio"（安静录音室）
- **语音隔离博客（2024）**【官方文档】https://techcommunity.microsoft.com/blog/microsoftteamsblog/voice-isolation-in-microsoft-teams-enables-personalized-noise-suppression-for-ca/4096077
  - 场景："offices with open floor plans, busy coffee shops, or even waiting for a flight"
  - 训练和评测数据合计 "over 2000 hours"，没有按类别分的时长
  - 声纹录入支持 25 种语言；2024 年 4 月先上 Windows 桌面版
  - 没有给出评测数字
- **语音隔离帮助页**【官方文档】https://support.microsoft.com/en-us/office/voice-isolation-in-microsoft-teams-calls-and-meetings-a9756ea9-4cec-44c4-aefb-6f5d17c89427 ：和同事共用一台设备时，Teams 会弹出提示，建议 "Turn off voice isolation"。原理（依赖本机声纹，去掉不匹配的声音）见管理员文档：https://learn.microsoft.com/en-us/microsoftteams/voice-isolation
- **研究线**【论文】Microsoft 的深度降噪挑战赛（DNS Challenge）：
  - ICASSP 2023 版按"戴耳麦 / 用外放"分两条赛道，每条都有个性化和非个性化两类模型：https://arxiv.org/abs/2303.11510 摘要
  - ICASSP 2021 版的测试集按真实录音 / 合成、非英语、情绪语音、唱歌分组：https://arxiv.org/pdf/2009.06122【二手，经搜索摘要，原文未核】
  - 都不按地点分组。

### 4. Zoom
- **档位说明**【官方文档】https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0059985 ，"Configure noise removal and background noise suppression" 一节：
  - Auto：需要时做中等程度降噪
  - Low："low levels of persistent background noise"
  - Medium："standard environments, including fans, pen tapping"
  - High："background speech from others"，并说会加大 CPU 占用
  - 原声模式（Original sound for musicians）会关掉所有处理
  - 注意：二手转述里的旧版文字把 High 写成 "crunching of paper or wrappers, keyboard typing"（来源如 https://www.technipages.com/zoom-enable-background-noise-suppression ）。两版说法不同。
- **网页开发包**【官方文档】https://developers.zoom.us/docs/video-sdk/web/audio-sound-options/ ："dogs barking, lawn mowers, clapping, fans, pen tapping"。
- **个性化音频隔离**【二手，原文未核】用用户的声纹做区分，"even in open environments"。出处是 Zoom AI 白皮书的搜索摘要；落地页我打开了，里面没有这段：https://library.zoom.com/zoom-workplace/artificial-intelligence/artificial-intelligence-bluepaper/algorithms/algorithm-features 。说它适合"共享办公、常有背景人声的家"的，是第三方博客的说法。

### 5. Google Meet
- **帮助页**【官方文档】https://support.google.com/meet/answer/9919960
  - 能去掉："typing, closing a door, room echo, or construction sound"
  - **"Voices from TV or people talking won't be canceled"**（电视人声和旁人说话不处理）
  - 音乐会被当成噪声
  - 订阅 Gemini 的用户由 Studio sound 取代标准降噪
  - 处理可能在本机和云端之间切换
- **2020 年发布帖**【官方文档】https://workspaceupdates.googleblog.com/2020/06/remove-background-noise-google-meet.html ："keyboard typing, doors opening and closing, and construction outside your window"。
- **媒体转述**【二手，原文未核】负责人 Serge Lachapelle 提到狗叫、小孩哭、鼓掌，来源如 https://venturebeat.com/ai/google-meet-noise-cancellation-ai-cloud-denoiser-g-suite 。

### 6. Apple macOS 麦克风模式
- **官方说法**【官方文档】https://support.apple.com/guide/mac-help/use-mic-modes-mchle82b42f0/mac
  - 语音突显（Voice Isolation）："To filter out background sounds"
  - 宽谱（Wide Spectrum）："To include the sounds around you"
  - 还有标准模式（Standard），以及 "options vary by app"
- **发布稿**【官方文档】Apple Newsroom iOS 15 发布稿只有一句："separate the user's voice from background noise"：https://www.apple.com/newsroom/2021/06/ios-15-brings-powerful-new-features-to-stay-connected-focus-explore-and-more/
- 不点名任何噪声或地点。
- 硬件要求"2018 年及以后的 Mac、macOS 12 起"【二手，原文未核】。当前官方页面上没看到这条。

### 7. Cisco Webex（收购的 BabbleLabs 技术）
- **四个模式**【官方文档】https://help.webex.com/en-us/article/3jqfrs

| 模式 | 做什么 | 官方点名的场景 |
|---|---|---|
| Noise removal（默认） | 去掉所有噪声 | "noisy environment" |
| Optimize for my voice | 去掉噪声 + 背景人声 | "open office space" |
| Optimize for all voices | 去掉噪声，增强背景人声 | "conference room" |
| Music mode | 保留音乐 | 演奏、播放音乐 |

  - 例子："doorbell rings"、"noisy traffic when you answer calls on the go"、"loud keyboard typing in an open office"
- **官方博客**【官方文档】https://blog.webex.com/collaboration/video-conferencing/16-billion-minutes-of-noisy-distractions-disappeared/
  - 噪声："lawn mowers, traffic, dogs and kids"
  - 场景："make-shift home offices"、在家上课、"highly reverberant rooms"
  - 数字：自 2020 年 9 月起累计处理 "over 16 billion minutes" 用户语音，没有按地点拆分
- **收购报道**【二手，原文未核】狗叫、割草机、汽车警报、警笛、"vacuum cleaner two rooms away"。

### 8. Dolby Voice
- 官方产品页 Dolby Voice for PC 和 Dolby Voice Room Pro 打开都是 404。
- 二手材料【二手，原文未核】：
  - 维基百科引 The Verge 2021-01-11 的报道：有 "noise reduction to limit unwanted background sounds in noisy environments" 和回声抑制：https://en.wikipedia.org/wiki/Dolby_Voice
  - Dolby.io 实时通信的 High 档位"抑制打字和狗叫"，来自搜索摘要。原文档页已重定向，未核。
- 按场景的声明或评测：未查到。

---

## 二、映射到 14 个场景键

每格写"谁点名了什么"。"—"表示未查到。【推断】表示厂商只点名了噪声，是我把噪声对到这个地点上。

| 场景键 | 有厂商点名该地点 | 只能从噪声类型推到这里【推断】 |
|---|---|---|
| home-quiet 书房/卧室 | NVIDIA：卧室回声；Teams：Off 档适合 "quiet studio"；Webex："highly reverberant rooms" | Teams Low：风扇、空调；NVIDIA：PC 风扇、电流声 |
| home-family 家里客厅 | Teams：小孩上网课、小孩吵闹、吸尘、厨房、冲马桶；NVIDIA：客厅、厨房、猫狗；Webex：在家办公、狗、小孩、门铃；Krisp 旧首页：狗叫、婴儿哭（二手） | Maxine：婴儿哭、做饭、洗衣机、吸尘器；Zoom 开发包：狗叫、割草机。⚠️ Meet 官方声明不处理电视人声 |
| open-office 开放工位 | Webex Optimize for my voice；Teams 语音隔离："open floor plans"；Krisp："office / busy workspace"，测试集有开放工位层；Zoom 个性化隔离："open environments"（二手） | Zoom High：旁人说话；键盘声（多家） |
| phone-booth 电话亭/专注舱 | — | Krisp 测试集"小房间"（推断近似）；小空间回声（NVIDIA） |
| meeting-room 会议室（多人共用一台 Mac） | Webex Optimize for all voices；Krisp 测试集"大会议室 / meeting rooms"；Teams：共用设备时提示关掉语音隔离 | 个性化隔离会把同屋的人当噪声去掉，与这个场景冲突（推断，依据是 Teams 帮助页的提示） |
| cowork 共享办公 | —（"适合共享办公"只见于第三方博客） | 同开放工位 |
| cafe 咖啡馆 | Krisp："noisy coffee house / cafe"；Teams 语音隔离："busy coffee shops" | Maxine：嘈杂人群、餐具类金属声、玻璃碎 |
| canteen 餐厅/食堂 | — | Maxine：嘈杂人群、做饭声 |
| transit-hub 机场/车站候车厅 | Teams 语音隔离："waiting for a flight"；Krisp 测试集：地铁站 | Maxine：火车经过、警笛；Krisp：人群嘈杂声 |
| car-passenger 车内乘客 | Krisp 测试集"Cars"（手机通话组）；Webex："traffic … on the go" | Maxine：交通噪声 |
| rail-air-cabin 高铁/飞机客舱 | — | Krisp 噪声类型里的 "cabin rumble"；Maxine：火车经过 |
| street-outdoor 街边/户外 | Krisp："street noise"，测试集有街道、商店；Webex：交通 | Meet：窗外施工；Zoom：割草机；Maxine：警笛、鸟叫、雨、施工 |
| hotel-room 酒店房间 | — | 空调声（Teams Low）、房间回声（NVIDIA） |
| library-classroom 教室/图书馆 | — | Teams 的"小孩上网课"是家里，不算；旁人说话（Zoom High、Webex） |

---

## 三、"有没有按场景分组的评测"

| 产品 | 按场景分组的评测 | 来源类型 |
|---|---|---|
| Krisp | 有，3 个使用场景（Work / Call center / Phone calls），公开数据集，带 `environment` 字段，但不按单个地点出结果 | 官方文档·厂商自测 |
| Microsoft | 产品侧：未查到。研究侧：DNS 挑战赛按"耳麦 / 外放"、真实 / 合成、语种、情绪分组，不按地点 | 论文 |
| NVIDIA | 只有 30 类噪声清单，没有分组评测数字 | 官方文档 |
| Zoom、Meet、Apple、Webex、Dolby | 未查到 | — |

**对本线的一点推论【推断】**：14 个场景键里，只有开放工位、咖啡馆、候车厅、车内、街边、会议室能找到厂商的"地点级"背书。其余 8 格要么只能靠噪声类型推过去，要么完全空白。建场景目录时，这 8 格的定义和典型噪声只能由本项目自己写判据，没有业界现成的说法可以对齐。

**本报告本身的局限**：只做了网络检索和阅读，没有实测。WebFetch 摘录的引文没有逐字二次核对。