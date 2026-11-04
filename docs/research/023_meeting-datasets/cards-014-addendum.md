# 014 卡补充字段（023 汇总）

> 2026-11-03 · 手下 mtgdata 合并。014 原卡（`../014_real-datasets/cards/`）一字不改；这里只补 023 数据卡模板（`card-template.md`）要求而 014 卡缺的字段：场景键 · 设备键 · 笔记本段 · 转写（单列）· 语体 · 与 D62 的关系，以及查出的 014 疑误。
> 来源：4 路调研员 + 各路怀疑派复核（workflow wf_98de0537-23d）。同一张卡被几路都补过的，按路分小节，不合并改写；几路说法冲突时以带【实测】或【论文】且经怀疑派复核的为准，裁定见末节。
> 场景键 / 设备键只认 021 `catalog.md` / 022 `devices.md` 正本。
> ⚠️ **许可口径**：014 卡的「结论」按已作废的 D26 把 SA 判为「只评测」。按 D64（2026-11-01），只带 SA、不带 NC 的可以训练（权重按相同条款共享并署名），涉及 014 卡：AliMeeting · AISHELL-4 · AISHELL-5 · CHiME-5 / 6 · CHiME-7 UDASE（音频部分）· ASCEND 类同理。NC、自定义协议、LDC 不变。

共补 33 张 014 卡。

## 一、逐卡补充

### ace-challenge-corpus.md

#### ①消费设备

- **场景键**：open-office, meeting-room, other（报告厅、带咖啡厅的大堂）
- **设备键**：laptop-unknown（Chromebook Pixel 内置 2 麦）, non-laptop-mic-array, non-laptop-studio
- **笔记本段**：有（Chromebook 档 1.1 GB，单独打包，可筛）；无 Mac
- **转写**：没有（目标语音为消声室录音，babble 无转写）
- **语体**：朗读；babble 为 4–7 人同时朗读
- **与 D62 的关系**：真实 RIR + 真实噪声 + 需自己卷积相加的语音（真实噪声 + 人工混合）。开发阶段若用于评测：Chromebook 档 1.1 GB
- **014 卡疑误**：无

### aishell-4.md

#### ②按地点

- **场景键**：meeting-room
- **设备键**：non-laptop-mic-array（8 通道圆阵）
- **笔记本段**：没有
- **转写**：有（TextGrid，带时间戳）
- **语体**：中文会议自然讨论；笑声、附和是否标注未查到
- **与 D62 的关系**：真实录音。开发阶段若用于评测：eval 20 场 12.72 h 单通道 + 转写（吃字检查）；test 包 5.2 GB

#### ④中文

- **场景键**：meeting-room
- **设备键**：non-laptop-mic-array（录制时有 non-laptop-close-talk，但公开包只有阵列）
- **笔记本段**：没有。OpenSLR SLR111 只列 8 通道圆阵包（train_L 7.0 GB / train_M 25 GB / train_S 14 GB / test 5.2 GB）【官方文档，2026-11-03 复读】
- **转写**：有（TextGrid，带时间戳与说话人）
- **语体**：会议讨论（医疗、教育、商务、组织管理、工业生产及日常会议，每场约 30 min、4–8 人）【论文 arXiv:2104.03603 §3】；笑声 / 附和 / 情绪标注未查到
- **与 D62 的关系**：真实录音（真人会议，阵列）。若开发阶段拿来评测，只下 test 5.2 GB（20 场 12.72 h），无近讲参考，只能做无参考指标与吃字检查

### aishell-5.md

#### ②按地点

- **场景键**：car-passenger
- **设备键**：non-laptop-mic-array（4 个门把上方远场麦）, non-laptop-close-talk（近讲，仅 train 发音频）
- **笔记本段**：没有
- **转写**：有（TextGrid，带时间戳）
- **语体**：车内 2–4 人无脚本自然对话；笑声、附和是否标注未查到
- **与 D62 的关系**：真实录音（行驶中车内）。开发阶段若用于评测：Dev 3.3 h / Eval1 3.3 h 远场单通道 + 转写；另有约 40 h 车内噪声可当混合噪声

#### ④中文

- **场景键**：car-passenger
- **设备键**：non-laptop-mic-array（四个门把手上方各一个远场麦）, non-laptop-close-talk（仅 train）
- **笔记本段**：没有
- **转写**：有（TextGrid）
- **语体**：车内 2–4 人无脚本对话，各约 10 min；笑声 / 附和标注未查到
- **与 D62 的关系**：真实录音（真人在混动车内）。若开发阶段拿来评测，可用 Dev 1.9 GB（3.3 h）或 Eval1 1.7 GB / Eval2 1.8 GB；另有约 40 h 纯车内噪声 noise.tar.gz 13 GB【官方文档 openslr.org/159，2026-11-03 复读】

### alimeeting.md

#### ②按地点

- **场景键**：meeting-room
- **设备键**：non-laptop-mic-array（8 通道定向麦阵）, non-laptop-close-talk
- **笔记本段**：没有；远场 / 近讲分包
- **转写**：有（TextGrid，带时间戳）
- **语体**：中文会议自然讨论，重叠率高（train 42%）；笑声、附和是否标注未查到
- **与 D62 的关系**：真实录音。开发阶段若用于评测：Eval 4 h（8 场）或 Test 10 h，远场单通道 + 头戴参考；Train far 73.24 GB，不需下

#### ④中文

- **场景键**：meeting-room
- **设备键**：non-laptop-mic-array, non-laptop-close-talk
- **笔记本段**：没有。远场是桌上 8 通道定向麦阵，近场是每人头戴麦；按目录分 far / near，无笔记本【论文 arXiv:2110.07393 §2】
- **转写**：有（TextGrid，每段带起止时间戳、说话人；另给「未处理原始文本」，其中停顿、噪声、重叠等非语音事件也有转写）【论文 §2】
- **语体**：会议自由讨论（医疗、教育、商务等话题，每场 2–4 人，重叠率高）；笑声 / 附和是否单独标未查到（原始文本只写了停顿、噪声、重叠几类）【论文 §2】
- **与 D62 的关系**：真实录音（真人会议，阵列 + 头戴同步）。若开发阶段拿来评测，用 Eval 4 h（8 场）或 Test 10 h，取阵列单通道 + 头戴近讲做有参考评测；包大小见 014 卡

### ami.md

#### ②按地点

- **场景键**：meeting-room
- **设备键**：non-laptop-mic-array（8 麦圆阵）, non-laptop-close-talk（ME 3-N 头戴）, non-laptop-close-talk（领夹，部分站点）
- **笔记本段**：没有；按流名（Array1-01 / Headset-0 等）可筛
- **转写**：有（NXT 词级时间戳）
- **语体**：约 2/3 为角色扮演设计会议、1/3 自然会议；对话行为层有「backchannel」类（如 um、uh-huh）【官方文档：annotation.shtml】；词转写含 `<vocalsound>` 类非言语标记（是否单分笑声未核实）【论坛 / 博客，个人说法：第三方转换项目示例】；情绪层 AmiEmotion 未公开【官方文档】
- **与 D62 的关系**：真实录音（安静会议室）。开发阶段若用于评测：取附和标注 + 头戴参考，检查「嗯、对」是否被吃；约 100 h 中按需取几场

### arte.md

#### ②按地点·地点补充

- **场景键**：cafe, home-family（晚宴）, transit-hub（火车站）, open-office（「open-plan office, people typing, chatting and talking on the phone」）【论坛 / 博客，个人说法：检索摘录 ARTE overview PDF】
- **设备键**：non-laptop-mic-array（62 通道球阵）
- **笔记本段**：没有
- **转写**：没有
- **语体**：背景人声（交谈、打电话），无目标语体
- **与 D62 的关系**：真实录音（环境声）。是本路「开放办公」唯一真实带人声背景的公开集；只能当噪声 / 回放场景

### aspire-stirling.md

#### ②按地点

- **场景键**：canteen（大学食堂 / 餐厅午餐高峰）, other（隔音棚对照）
- **设备键**：non-laptop-close-talk（领夹麦，接在 iPad mini 2 上录，iPad 距说话人 90 cm 只管录像）, non-laptop-studio（Zoom H4n Pro + 双耳麦，由对面听者佩戴，约 140 cm；模板无双耳麦键）
- **笔记本段**：没有；iPad（领夹）与 H4n（双耳）分文件，可按设备筛【论文：arXiv 1910.00424 §2】
- **转写**：有（Grid 固定句，脚本即转写；时间戳未查到）
- **语体**：朗读 Grid 句，无对话、无笑声、无附和、无情绪
- **与 D62 的关系**：真实录音（真人在真实食堂朗读）。开发阶段若用于评测：真实嘈杂段 5,000 句的 H4n 双耳通道（140 cm，较接近远讲）或 iPad 文件里的领夹通道（近讲但在食堂里）；全包 13.0 GB

### chime3-chime4-real.md

#### ①消费设备·设备补充

- **设备键**：non-laptop-tablet（平板四周 6 麦）, non-laptop-close-talk
- **笔记本段**：没有；平板通道与近讲通道分文件，可筛
- **与 D62 的关系**：真实录音（真人朗读 WSJ 句子，公交 / 咖啡馆 / 步行街 / 路口）；LDC 付费（LDC2017S24）、不可再分发。开发阶段若拿来评测：真实 dev 1640 句约 2.74 h 的平板通道，前提是先付费取得【沿用 014】

#### ②按地点·地点补充

- **场景键**：cafe, transit-hub, bus-metro-cabin（公交车内）, street-outdoor（街口、步行区）
- **设备键**：non-laptop-tablet（手持平板装 6 麦，ch1–6）, non-laptop-close-talk（ch0 近讲）
- **笔记本段**：没有
- **转写**：有（WSJ0 提示文本，句级，无词级时间戳）
- **语体**：朗读，无笑声、附和、情绪
- **与 D62 的关系**：真实录音（真人在真实地点对平板朗读）。是本路「公交 / 街边 / 咖啡馆真人说话」的唯一按地点标注的真实集，但 LDC 付费不可再分发

### chime5-chime6.md

#### ②按地点

- **场景键**：home-family（厨房 / 餐厅 / 客厅晚餐聚会；禁止背景电视与商业音乐）
- **设备键**：non-laptop-mic-array（Kinect 4 麦线阵）, non-laptop-close-talk（双耳入耳麦）
- **笔记本段**：没有；按设备 ID（U01–U06 / 参与者 P..）可筛，但无笔记本
- **转写**：有（每场 JSON，逐句起止时间）
- **语体**：家中自然对话；转写含 `[laughs]`（参与者笑）、`[noise]`（咳嗽、咀嚼等）标签【官方文档：chimechallenge.github.io/chime6/track1_data】；附和随词转写保留【推断】；无情绪标注
- **与 D62 的关系**：真实录音。开发阶段若用于评测：dev 4:27 h 或 eval 5:12 h 的单个 Kinect 通道 + 双耳参考；CHiME-6 dev 包 11 GB

### chime9-echi.md

#### ②按地点

- **场景键**：canteen（模拟食堂：回放干扰对话 + WHAM! 背景）
- **设备键**：non-laptop-mic-array（Project Aria 眼镜 7 通道；助听器每耳前后 2 麦共 4 通道，键表无助听器类，也记 non-laptop-mic-array）, non-laptop-close-talk（近讲，仅训练可用）
- **笔记本段**：没有；按设备分文件（Aria / 助听器）
- **转写**：未查到（官网与数据卡未提）
- **语体**：桌边 4 人真实自然对话；笑声、附和未标注（未查到）
- **与 D62 的关系**：部分真实（真人对话 + 扬声器回放朗读语料与噪声）。开发阶段若用于评测：dev 6.0 h 的 Aria 单通道；全集约 105 GB；需签协议

### clarity-cec3-task2.md

#### ①消费设备

- **场景键**：other（录音室，回放布局）
- **设备键**：non-laptop-unknown（实为助听器壳 3 麦；暂定键里没有助听器）
- **笔记本段**：没有
- **转写**：未查到
- **语体**：未查到
- **与 D62 的关系**：回放重录（真人佩戴、音箱回放）。设备是助听器壳麦，无笔记本段；全包 74 GB【沿用 014】
- **014 卡疑误**：无

### common-voice.md

#### ④中文·中文子集，怀疑派补

- **场景键**：other（众包，各人所在环境，无环境标签）
- **设备键**：non-laptop-unknown（用户自己的设备，经 Common Voice 网站或 iPhone 应用录制【论文 arXiv:1912.06670 §2】）
- **笔记本段**：可能有，但筛不出来。网页端录音会用到笔记本内置麦（014 卡已写「含笔记本内置麦」），但数据里没有设备字段，不能按设备筛【论文】【推断：网页端必然含笔记本用户，比例未查到】
- **转写**：有（只有朗读提示文本，无时间戳）
- **语体**：朗读（读屏幕上的句子）；无笑声、附和、情绪
- **与 D62 的关系**：真实录音（真人朗读，各自设备；非回放、非合成，音频为 MP3 有损）。中文子集有 zh-CN（普通话，大陆）、zh-TW、zh-HK、yue。Scripted Speech 25.0 zh-CN：851,641 条、录音 1,073.77 h（已验证 239.14 h）、7,546 人、约 21.38 GB；23.0 zh-CN 约 21.26 GB、CC-0【官方文档，经搜索摘录；MDC 页面为 JS 渲染，本次未能亲读】。若开发阶段拿来评测，可取 zh-CN test 分集做「消费设备麦 + 中文」吃字检查，但设备混杂、无法单挑笔记本段

### daps.md

#### ①消费设备

- **场景键**：open-office（office1/2）, meeting-room（confroom1/2）, home-quiet（livingroom1、bedroom1）, street-outdoor（balcony1 车流）
- **设备键**：non-laptop-tablet（iPad Air）, non-laptop-phone（iPhone 5S）
- **笔记本段**：没有；按版本目录（如 `ipad_confroom1`）可筛设备【沿用 014】
- **转写**：有（朗读脚本，篇章级，无时间戳）
- **语体**：朗读；无笑声 / 附和 / 情绪
- **与 D62 的关系**：回放重录（录音棚版经音箱放、设备重录）。开发阶段若用于评测：iPad confroom 两个版本，大小未查到（全包 16.1 GB）
- **014 卡疑误**：无

### dipco.md

#### ②按地点

- **场景键**：home-family（同一房间模拟晚餐，部分时段放音乐）
- **设备键**：non-laptop-mic-array（5 台 7 麦圆阵）, non-laptop-close-talk
- **笔记本段**：没有
- **转写**：有（JSON，≤10–15 s 分段，有起止时间）
- **语体**：4 人晚餐自然聊天；笑声标注未查到
- **与 D62 的关系**：真实录音。开发阶段若用于评测：dev 2:43 h 整份，单通道 + 头戴参考；包 13.4 GB

### dns-challenge-blind-real.md

#### ①消费设备

- **场景键**：home-family, open-office, other（按年份见上面各届）
- **设备键**：non-laptop-phone, non-laptop-desktop, laptop-unknown, non-laptop-close-talk, non-laptop-unknown
- **笔记本段**：按年份不同：2020 blind 无设备字段；2021 两届文件名带自由填写的设备字（含 macbook 约 31 条）；2022 只有 Mobile / Desk；2023 `ap` 段写型号、`mt` 段部分写 Windows / 安卓（见上面各节）【实测：文件名】
- **转写**：没有公开（blind 均未公开转写）
- **语体**：2021 合集有情绪 600 条、文件名带 laugh 约 100 条；2023 有附和 1 条（hmm-mhm）
- **与 D62 的关系**：真实录音；同上各届
- **014 卡疑误**：原句「约 70% 用手机录（arXiv 2202.13288 §3.3）」只适用于 2022 届；原句「语言：未查到」→ 2021 两届文件名有 nonenglish 与 tonal（普通话、粤语、旁遮普、越南语等）字段【实测：文件名】【沿用 014 dns3 卡】

### dns1-interspeech2020-real-testset.md

#### ①消费设备

- **场景键**：open-office, meeting-room, cafe（dev 内部真实录音文件名含 cafe / cafeteria / cafee 共 27 条）, other（AudioSet 片段）
- **设备键**：non-laptop-close-talk, non-laptop-conf-device（文件名 speakerphone）, laptop-unknown（文件名 `SurfaceBook` 44 条，Surface Book 是可拆键盘的笔记本【推断：据产品名】）, non-laptop-unknown（AudioSet 与 blind 段）
- **笔记本段**：dev：MS 内部 150 条文件名带 `headset` 57 / `speakerphone` 93；speakerphone 里 55 条带 surface 字样，其中 `SurfaceBook` 44 条、只写 `surface` 11 条（是 Surface Book 还是 Surface Pro 平板未写）；无 Mac 字样。「SurfaceBook」是否指用了机身内置麦，文件名没写【实测：文件名】【推断】。blind：300 条真实录音文件名只有 `realrec_fileid_N`，不能按设备筛【实测：文件名，分支 interspeech2020/master】
- **转写**：没有
- **语体**：朗读与自由说未区分；无笑声 / 附和 / 情绪标注
- **与 D62 的关系**：真实录音（dev MS 内部段 + blind MTurk 段）+ 手挑 AudioSet 片段。开发阶段若用于评测：blind 300 条真实段（每条约 10 s），或 dev 150 条 MS 内部段；直接在 GitHub 仓库（git LFS），大小未查到
- **014 卡疑误**：原句「dev 4 类各 300 条（其中真实 600 条：MS 内部 300 + AudioSet 300）」。仓库 `datasets/test_set/real_recordings/` 实有 300 条 = `ms_` 150 + `audioset_` 150；blind 为 realrec 300 + noreverb 150 + reverb 150【实测：GitHub contents API，分支 interspeech2020/master】

### dns2-icassp2021-real-testset-pdns.md

#### ①消费设备

- **场景键**：home-family, open-office, other（文件名噪声字段：狗叫、打字、婴儿哭、风扇等；无地点字段）
- **设备键**：mac-unknown（6 条自填 `macbook`，型号与是否内置麦未核）, laptop-unknown, non-laptop-close-talk, non-laptop-desktop（webcam）, non-laptop-phone, non-laptop-unknown
- **笔记本段**：有，可粗筛。T1 blind（合集 `noisy_blind_testset_v2_challenge_withSNR_16k/`，700 条 = 真实 650 + 合成唱歌 50（女 40、男 5、合唱 5））文件名里含 `laptop` 192 条、`macbook` 6 条、`headset` 97、`webcam` 41、`surface` 24、手机类 25。设备字段是众包者自由填写，写法不统一（如 `Win_laptop_mic`、`laptopmic`、`builtinmicrophone`，甚至有未替换的占位符 `your_device_type`），只能用正则粗筛，不可靠【实测：文件名】
- **转写**：没有
- **语体**：文件名分 english / nonenglish / emotional（emotional 102 条；`laughter` 34 条）；附和无标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包 + MS 内部）。开发阶段若用于评测：T1 blind 里文件名含 laptop / macbook 的约 200 条；合集 zip 778 MB（V2+V3+情绪+鼠标点击一起）【实测：HTTP 头 Content-Length】

### dns3-interspeech2021-real-testset.md

#### ①消费设备

- **场景键**：home-family, open-office, other
- **设备键**：mac-unknown（自填 `macbook` 12、`mac` 12、情绪集 `Macbookpro` 1，型号与是否内置麦未核）, laptop-unknown, non-laptop-close-talk, non-laptop-desktop, non-laptop-phone, non-laptop-unknown
- **笔记本段**：有，可粗筛。V3 blind（600 条 = 真实 440 + 非真实 160）文件名含 `laptop` 137 条、`macbook` 12 条（同一众包者 `A1RK5KQQ320OH4`，英语男声，打字 / 风扇 / 厨房 / 洗碗机等，near / far 都有）、`mac` 12 条（同一人 `XMQ6Y5VEXRUU`，钢琴背景 far）、`headset` 124、`surface` 46。同包的情绪集 600 条（怀疑派重数，见文末）：`laptop` 252（不分大小写）、手机类 142、`headset` 114、`Macbookpro` 1；鼠标点击集 99 条：手机类 30、`headset` 29、`laptop` 10【实测：文件名】
- **转写**：没有
- **语体**：情绪集 600 条（文件名标 sad / angry / crying / yelling / happy / laughter；`laugh` 不分大小写 100 条）；V3 另有 musical（乐器背景）78 条；附和无标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包，1–5 m 内非头戴）+ 合成段。开发阶段若用于评测：V3 + 情绪集里文件名带 laptop / mac 的约 400 条；合集 zip 778 MB
- **014 卡疑误**：原句「V3 blind 600 条」，未说明其中 160 条不是 `realrec`（`nonenglish_synthetic` 59、`nonenglish_tonal_synthetic` 51、`stationary_english_tug`（Graz 干净语音 + 稳态噪声【推断】）50），真实只有 440 条；原句（014 `README.md` 索引行，非 dns3 卡正文）「合集另含 600 条情感片段、100 条鼠标点击片段」：情感片段**确为 600 条**（怀疑派重数 zip 目录：`handpicked_emotion_testclips_16k_600_withSNR/` 下 600 个 .wav、名字不重复），本补充原写的 584 是错的；鼠标点击实为 99 条，014 的 100 差 1【实测：文件名】

#### ③无参考

- **场景键**：未查到（论文只写多种背景噪声；无逐条场景标签）
- **设备键**：non-laptop-unknown
- **笔记本段**：未查到；众包设备未标型号（014 原卡）
- **转写**：没有
- **语体**：含情感语音（笑、哭、喊、惊讶）（014 原卡）——本路补：这是公开集里少有的带「笑声 / 情绪」的真实带噪段，但没有人评分随集发布
- **与 D62 的关系**：真实众包录音（与合成段混在同一测试集里）。DNSMOS P.835 的训练标签就来自「DNS Challenge V3 测试集 600 条 × 约 40 个降噪模型」的 P.835 众包评分（约 30,000 条、75 h）【论文 arXiv:2110.01763 §3】；**这些人评分本身未见公开发布**（检索未找到下载处）。若开发阶段拿来评测：用 V3 blind 600 条跑无参考分作回归；但用 DNSMOS 测它属于「训练分布内」，分数会偏乐观【推断】
- **014 卡需更正 / 补充**：补上一条「DNSMOS P.835 见过本集」——用它在本集上评 DNSMOS 不算独立验证【论文】

> 怀疑派复核（2026-11-03）：改了 nisqa-livetalk「笔记本段」（「有」→「未查到确证」，补论文同段两句互相矛盾的原文）；urgent2024 补「一半真实」只出自论文、HF 卡未写，T17 / T18 一句出处节号更正为 §3.1；dns3 补「真实与合成混在同一测试集」。urgent2024_mos 的 300 条、6,900 条、13.8 h、1.34 GB、8 人评分、CC BY-NC-SA 4.0 回 HF 数据卡核对一致；DNSMOS P.835 的 600 条 × 约 40 模型、约 30,000 条、75 h 回论文 §3 核对一致。

### dns4-icassp2022-real-testset-pdns.md

#### ①消费设备

- **场景键**：home-family, open-office, street-outdoor, car-passenger, other（文件名噪声字段：Typing 83、Water 68、Fan 62、Door 62、MouseClicks 61、Chips 56、Cutlery 53、Street 29、InsideCar 7、CopyRoom 10 等；无 babble / 咖啡馆字段）
- **设备键**：non-laptop-phone, non-laptop-desktop（文件名 `non-laptop-desktop` / `Desk`，不分台式与笔记本）
- **笔记本段**：blind 859 条每条文件名都有设备字段：`Mobile` 638、`desktop` / `Desk` 221；**笔记本与台式机分不开**，无 Mac 字样。另有距离字段 Near / Far、说话人字段 Primary / Neighbor（Neighbor 121 条）【实测：文件名】
- **转写**：部分（dev 有 `DNSChallenge4_devtest.tsv`；blind 标 TBD 未公开）
- **语体**：blind 情绪段 98 条（sad、angry、yelling、laughter 14、crying、happy）；无附和标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包）。开发阶段若用于评测：blind 的 Desk 段 221 条 × 10 s（约 37 min【推断】）；zip 1.66 GB
- **014 卡疑误**：无新发现；「638 条手机」与 README 一致，且文件名可逐条复核（怀疑派重数 `blind_testset_bothtracks.zip` 目录：testclips 859 = Mobile 638 + Desk 221，enrollment 178，zip 1,658,455,878 字节，一致）。另：DNS5 dev Track2 的 600 条里 599 条与本集 blind 同名，两集不要重复计数【实测：zip 目录比对】

### dns5-icassp2023-real-testset.md

#### ①消费设备

- **场景键**：home-family, open-office, cafe（Cafe 4 条）, transit-hub（Airport 25、Announcements 9）, street-outdoor（Traffic 12）, car-passenger, other
- **设备键**：non-laptop-close-talk, non-laptop-conf-device（回音消除免提）, non-laptop-desktop（Logitech C920、LifeCam 摄像头麦）, laptop-unknown（推断：`Microphone_Array_RealtekR_Audio` 36 条，Realtek 阵列麦常见于 Windows 笔记本内置）, non-laptop-phone（Pixel 4a、Moto G 共 4 条）, non-laptop-unknown
- **笔记本段**：blind 分 4 个来源前缀：`ap`（349 条，再分 `ap_desktop` 176 / `ap_mobile` 173，并写全设备型号，如 Plantronics、Jabra、EPOS、Realtek）· `mt`（众包；其中 `mt-win_` 81 条标 Windows 电脑、`mt-and_mobile` 44 条标安卓手机，其余约 45 条无设备字段）· `sa`（131 条，只有 GUID）· `lk`（实验室，距离 0.5 / 1 / 2 m）。`ap` 段能按型号筛，`mt` 段只能分 Windows / 安卓；Windows 电脑里笔记本与台式机分不开；无 Mac / MacBook 字样【实测：文件名，怀疑派重数】
- **转写**：部分（dev Track2 有 `v5_devset_transcripts_track2.tsv`；Track1 只有 README；blind 未公开）【实测：文件名】
- **语体**：blind Track1 有副语言片段 `pl_cat1-throat-clear`、`pl_cat2-hmm-mhm`（附和）、`pl_cat3-Huh-what` 等 4 条，`Laughing` 2 条；dev Track2 情绪 72 条（laughter 11）【实测：文件名】
- **与 D62 的关系**：真实录音（众包 + 采集商 + 实验室）。开发阶段若用于评测：blind Track2 的 `ap` 段 128 条（含 Realtek 阵列 36 条、摄像头 11 条）；blind zip 3.89 GB、dev zip 3.24 GB
- **014 卡疑误**：
  1. 原句「blind 条数未查到」→ blind `noisy/` 目录 Track1 389 条（怀疑派重数，原写 388 差 1）、Track2 364 条（另各有同数 enrol）【实测：zip 目录】
  2. 原句「dev 全部笔记本或台式机」（出自论文 §3）→ dev Track2 600 条文件名里 `Mobile` 413 条、`Desk` 187 条；Track1 641 条全为 `Desk` + `HS`【实测：zip 目录】。怀疑派复核结论：以文件名为准。依据：①论文自相矛盾，摘要后的贡献列表原句「Headset, and Speakerphone both has desktop and mobile recordings in testsets」，§3 又写 dev「recorded using a laptop or desktop computer」【论文 arXiv 2303.11510】；②dev Track2 的 600 个文件名里有 599 个与 DNS4（2022）blind 的 859 个文件名逐字相同，即 dev Track2 是复用 DNS4 blind，DNS4 论文本就写 70% 用手机录【实测：两个 zip 目录比对】
  3. 原句「dev：非个性化 600 条、个性化 1443 条」→ dev zip `noisy/` 为 Track1 641 + Track2 600 条；1443 条个性化 dev 不在此 zip 内【实测：zip 目录】

#### ②按地点·语体补充

- **场景键**：home-family, open-office（dev 写「办公」）, car-passenger, cafe, transit-hub（blind 含机场广播）
- **设备键**：laptop-unknown, non-laptop-desktop（dev 全部笔记本或台式机）, non-laptop-unknown（blind 设备清单未公开）
- **笔记本段**：dev 有，但无逐条设备标签，不能筛
- **转写**：部分（见 014 原卡的说法冲突）
- **语体**：193 条情绪语音，分高兴、悲伤、愤怒、喊叫、哭、笑 6 类，由众包者按提示表演，混有噪声【论文：arXiv 2202.13288，DNS4 非个性化 dev】；DNS5 是否原样复用这 193 条为 014 的推算；无附和标注
- **与 D62 的关系**：真实录音（真机录制）。开发阶段若用于评测：193 条情绪段 + 含 café / 机场广播的 blind 段

### dr-vctk.md

#### ①消费设备

- **场景键**：open-office（中等办公室，开窗 / 关窗）
- **设备键**：mba-builtin（MacBook Air 2 通道）, non-laptop-phone（iPhone 5S）, non-laptop-tablet（iPad）, non-laptop-studio（Apogee MiC、Blue Snowball）
- **笔记本段**：有，且能按设备筛（8 通道分设备）；机型年份未写
- **转写**：有（VCTK 文本）
- **语体**：朗读；无笑声 / 附和 / 情绪
- **与 D62 的关系**：回放重录（距麦 2 m）。开发阶段若用于评测：MacBook Air 两通道，16 kHz；全包 1.67 GB
- **014 卡疑误**：datashare 页标题为「Device Recorded VCTK (Small subset version)」，014 卡未写这是子集版【官方文档】；End-user Licence 本轮仍未读到正文

### easycom.md

#### ②按地点

- **场景键**：canteen（餐厅噪声为扬声器回放，对话为真人现场）
- **设备键**：non-laptop-mic-array（AR 眼镜 6 通道）, non-laptop-close-talk（非佩戴者的头戴参考麦）
- **笔记本段**：没有；按通道区分眼镜麦与头戴麦，无笔记本设备
- **转写**：有（按视频帧计时，20 fps，即有时间戳）
- **语体**：点餐、拼图、I spy 游戏中的自然对话，有重叠；笑声、附和未单列标注（未查到）
- **与 D62 的关系**：部分真实（真人对话 + 回放餐厅噪声，约 71 dB SPL）。开发阶段若用于评测：取眼镜单通道 + 头戴参考，约 5 h 18 min；GB 未查到

### icsi.md

#### ②按地点·怀疑派补

- **场景键**：meeting-room
- **设备键**：non-laptop-close-talk（头戴为主，早期部分领夹）, non-laptop-mic-array（桌面 4 支 PZM）, non-laptop-unknown（「假 PDA」上 2 支廉价麦，型号未写）【沿用 014】
- **笔记本段**：没有；按通道可筛，最接近消费级的是 PDA 廉价麦
- **转写**：有（带时间）
- **语体**：研究组例会自然对话；转写用 VocalSound 标记笑声（laugh 是最常见的非言语标记，另有 Comment 标长段笑），原始转写共 13,678 处笑声；MRDA 对话行为层覆盖全部 75 场，含 backchannel 类（5 类版 14,275 处）【论文：Laskowski & Burger 2007；MRDA 相关论文 arXiv 1807.08587，经检索摘录】；无情绪标注
- **与 D62 的关系**：真实录音（安静会议室）。爱丁堡托管版 CC BY 4.0（官方页写信号、转写和部分标注都按 CC BY 4.0 发布）。开发阶段若用于评测：取笑声与附和标注密的几场，用 PDA 廉价麦或桌面麦单通道 + 头戴参考，检查「笑声、嗯、对」是否被吃；约 72 h 中按需取几场

### misp-meeting.md

#### ④中文

- **场景键**：meeting-room
- **设备键**：non-laptop-conf-device（8 麦阵集成在讯飞智能办公本 X3 里；Insta360 X3 全景相机两侧 2 麦、48 kHz 也归此键，暂定键里没有「相机」）, non-laptop-close-talk（头戴麦，44.1 kHz，接 Zoom F8N）
- **笔记本段**：没有笔记本。但远场「阵列」实为消费级办公本 iFLYTEK Smart Office Book X3 的内置 8 麦（矩形 197 × 134 mm，16 kHz / 32-bit），设备高出桌面 30–40 cm；全景相机另录 2 通道 48 kHz 音频【论文 arXiv:2505.13971 §2.2】
- **转写**：有（转写包；时间戳据 AVDR 任务推断有，014 卡已注【推断】）
- **语体**：真实会议自然讨论（医疗、教育、商务、工业生产），参与者是与话题相关的专业人员或学生，重叠段占比 57.30%；有键盘、点击、开关门、风扇声【论文 §2.1–2.2】；笑声 / 附和标注未查到
- **与 D62 的关系**：真实录音（真人会议，消费级办公本麦阵 + 相机麦 + 头戴同步）。若开发阶段拿来评测，用 dev F8N 1.26 GB / eval F8N 1.27 GB（近讲）加对应远场包（远场包大小见 014 卡），dev / eval 各 9 场、各约 3 h

### misp2021.md

#### ②按地点·怀疑派补

- **场景键**：home-family（34 个真实家庭电视房，边看电视边聊天）
- **设备键**：non-laptop-mic-array（远场 6 麦线阵 3–5 m；中场 2 麦阵 1–1.5 m）, non-laptop-close-talk（每人下巴处高保真麦）【沿用 014，二手】
- **笔记本段**：没有
- **转写**：有（MISP2022 起带时间戳）
- **语体**：中文多人家中闲聊，含重叠；笑声、附和是否标注未查到
- **与 D62 的关系**：真实录音（真人在开着电视的家里聊天）。这是本路「家里 + 电视 + 有人声 + 有近讲参考」唯一的真实集，r2 原稿漏了；许可原文名未查到、下载页现未列链接（014）。开发阶段若用于评测：中场 2 麦单通道 + 近场参考，量按需

#### ④中文

- **场景键**：home-family
- **设备键**：non-laptop-mic-array（远场 6 麦线阵、中场 2 麦阵）, non-laptop-close-talk
- **笔记本段**：没有
- **转写**：有（MISP2022 起带 RTTM / 时间戳，据 014 卡）
- **语体**：家中边看电视边闲聊，多人重叠；有电视声；笑声 / 附和标注未查到
- **与 D62 的关系**：真实录音（真实家庭）。获取方式仍不明：2026-11-03 复读 challenge.xfyun.cn/misp_dataset，MISP2021/2022/2023 各条仍只写「This dataset is available under license.」且无下载链接【官方文档】；开发阶段目前拿不到

### ms-aec-challenge-real.md

#### ①消费设备

- **场景键**：home-family, open-office
- **设备键**：laptop-unknown, non-laptop-desktop, non-laptop-phone（`fullband_mobile`）
- **笔记本段**：数据集 README 原句「The real files were captured on Windows PCs」；文件名只有 GUID + 场景（`nearend_singletalk` 等），**不能按设备筛**；全为 Windows，没有 Mac【官方文档：datasets/README.md】【实测：git tree 文件名】
- **转写**：没有
- **语体**：近端单讲为朗读或自由说未写
- **与 D62 的关系**：真实录音（众包 PC）。开发阶段若用于评测：各届 blind 的 `nearend-singletalk` 子目录（2022 届 400 个文件、2023 届 600 个文件，含 mic / lpb）【实测：git tree】
- **014 卡疑误**：README 原句「Some PCs may have audio DSP processing in the send and/or receive path, even though we used a raw mode」——014 卡已写「设备 DSP 残留」，与此一致

#### ②按地点·怀疑派补

- **场景键**：home-family, open-office（014 写「真实家庭和办公」，无逐条场景标签）
- **设备键**：laptop-unknown, non-laptop-desktop, non-laptop-phone（众包 PC 与移动设备；是否有逐条设备标签未查到）
- **笔记本段**：未查到（众包 PC 里应有笔记本，属推断；能否按设备筛未查到）
- **转写**：没有（无公开转写文件）
- **语体**：近端单讲，朗读或自由说未查到；无笑声、附和、情绪标注
- **与 D62 的关系**：真实录音（众包真机）。数据许可未查到（MIT 只管代码）。开发阶段若用于评测：nearend_singletalk 片段做「各种电脑麦的电平、频响」对照；量未查到

### nisqa-livetalk.md

#### ①消费设备

- **场景键**：cafe, car-passenger, transit-hub, street-outdoor, other（电梯、商场）
- **设备键**：non-laptop-phone, laptop-unknown, non-laptop-unknown（说话人「spoke directly into the terminal device (i.e. a smartphone or laptop)」；Google Pixel 3 与笔记本是**录音端**，不一定是说话人的设备）
- **笔记本段**：可能有，不能筛。官方原句「The calls were recorded on a Laptop for the VoIP calls and on a Google Pixel 3 for the mobile phone calls」——笔记本是录通话的那一端，录的是经网络传过来的信号，不是笔记本麦直录；说话人哪几条用的是笔记本未写；逐条设备标签未查到【官方文档：NISQA Corpus wiki】
- **转写**：未查到
- **语体**：真人对设备说话；无笑声 / 附和标注
- **与 D62 的关系**：真实通话录音。开发阶段若用于评测：232 条，作无参考音质分的定标旁证
- **014 卡疑误**：无；全集另见 023 新卡 `../cards/nisqa-corpus.md`（他路所写）

#### ③无参考

- **场景键**：cafe, car-passenger, transit-hub, street-outdoor, other（电梯、信号差的楼里、商场）
- **设备键**：non-laptop-phone, laptop-unknown
- **笔记本段**：未查到确证，不能按条筛。维基原句：VoIP 通话「recorded on a laptop」，移动通话在 Google Pixel 3 上录；论文 §3 同一段里先写说话人「spoke directly into the terminal device (i.e. a smartphone or laptop)」，后写「The talkers used their mobile phone to call either through the mobile network or with a VoIP service (Skype/Facebook)」——两句互相矛盾，说话端有没有用笔记本麦说不清；笔记本更可能是接收端录音机【推断】。逐条设备标签未查到；58 个条件名里可能写了网络类型，未下载核对【官方文档】【论文】
- **转写**：没有（未提）
- **语体**：真人对话式说话（德语），按条件要求大声、小声、免提或放背景音乐；笑声 / 附和未标注【论文 §3】
- **与 D62 的关系**：真实录音（真人对终端说话，经移动网或 VoIP 传输后录下）。人评分为实验室 P.800，每条 24 分，58 条件 × 4 条。若开发阶段拿来评测：232 条全用，作无参考音质分在「真实嘈杂地点 + 真实通话链路」上的旁证；大小未查到（每条 6–12 s，总时长约 0.6 h【推断】）
- **014 卡需更正 / 补充**：
  - 原句「**VoIP 通话在笔记本上录**」：维基确是此说；但论文 §3 原句是「The talkers used their mobile phone to call either through the mobile network or with a VoIP service (Skype/Facebook)」，同段前文又说对着「smartphone or laptop」说话。两处出处说法不同——「笔记本」更可能是接收端录音机，说话人手里多是手机。建议改成「说话人对手机或笔记本说话（论文）；VoIP 接收端在笔记本上录（维基）」【论文】【官方文档】
  - 原句「每条 24 票」：补「实验室 P.800（不是众包 P.808）」【论文 §3】
  - NISQA 模型在本集按条件平均、一阶映射后 PCC 0.90、RMSE 0.35【论文 Table 4】；SCOREQ（natural、无参考）README 报 0.86（按条件平均）【官方文档】

### notsofar-1.md

#### ②按地点

- **场景键**：meeting-room
- **设备键**：non-laptop-conf-device（Yealink、EPOS、Sennheiser、Logitech、Poly、Jabra）, non-laptop-close-talk（Shure WH20 近讲）
- **笔记本段**：没有；有逐场逐设备标签，可按设备筛
- **转写**：有（专业人工转写，带时间戳；eval 无近讲但有转写）
- **语体**：会议自然对话，有打断、重叠；014 卡写环境含笑声咳嗽、元数据标了声学事件，但笑声是否作为单独事件标签未核实；附和随转写保留【推断】；无情绪标注
- **与 D62 的关系**：真实录音。开发阶段若用于评测：MC 原始流单麦 + 转写（吃字检查）；dev 35 场约 14 h（MC）；HF 全库 19.5 TB 含模拟集，只需下 dev / eval 子集

### realman.md

#### ④中文

- **场景键**：other（室内、室外、半室外、交通工具，具体场所 014 卡注未查到）
- **设备键**：non-laptop-mic-array（32 通道）
- **笔记本段**：没有
- **转写**：有（`transcriptions.trn`）
- **语体**：朗读语音经音箱回放，无笑声、附和、情绪
- **与 D62 的关系**：回放重录（语音由音箱播放，非真人现场说话），噪声为真实场景单独录制。若开发阶段拿来评测，可只下 test 39.3 GB；真实噪声部分可作合成用噪声库

### urgent2024-blind-real.md

#### ①消费设备

- **场景键**：other（未标）
- **设备键**：non-laptop-unknown
- **笔记本段**：没有设备信息。HF 数据卡原句「no labels or meta information are provided for the blind test set」；数据卡列的源语音含 CHiME-6、VOiCES、EasyCom、YouTube 提取音频，真实段是否取自这些未写【官方文档】
- **转写**：没有
- **语体**：未查到
- **与 D62 的关系**：真实 500 条（来源未写）。开发阶段若用于评测：blind parquet 480 MB（含 500 模拟 + 500 真实）【实测：HF 文件树】

#### ③无参考

- **场景键**：未查到（真实段「collected from unseen sources」，未点名）
- **设备键**：non-laptop-unknown
- **笔记本段**：未查到；无逐条设备标签
- **转写**：没有（HF 数据卡：blind「no labels or meta information」，014 原卡已写）
- **语体**：未查到
- **与 D62 的关系**：盲测 1,000 条 = 500 模拟 + 500 真实【论文 2506.01611 §2.3】。人评分子集 `urgent2024_mos`：从盲测抽 300 条（一半模拟一半真实），连同 22 个提交系统的输出共 6,900 条、约 13.8 h、1.34 GB，P.808 众包，每条 8 分【官方文档：HF urgent2024_mos 数据卡】【论文 §3.1】。若开发阶段拿来评测：用 `urgent2024_mos` 中真实那 150 条 × 23 版本（约 3,450 条【推断】）校准无参考 verifier 的系统级排序；约 0.7 GB【推断，按一半算】
- **014 卡需更正 / 补充**：
  - 原句「另有 MOS 子集（urgent2024_mos）」：补规模与协议（见上）；该子集许可为 HF 标签 cc-by-nc-sa-4.0 → 只评测【官方文档】
  - 补：组委会用该子集做的相关分析结论——UTMOS、SCOREQ 与人评分相关最高；WADA-SNR 最低；DNSMOS 排序相关（KRCC）落后但线性相关（LCC）较高，即数值接近、排序可能错；NISQA、DNSMOS Pro 在排序与数值上都更一致；说话人相似度与识别正确率 KRCC 较高但 LCC 低（非线性）。具体数字只在图 4 里，文本未给【论文 2506.01611 §3.2】
  - 补：T17、T18 两队 DNSMOS / NISQA 很高，但人评分与其他指标都差——**只看无参考分会被骗**【论文 §3.1 末段】
  - 补：HF `urgent2024_mos` 数据卡本身没写真实 / 模拟比例；「一半模拟一半真实」出自论文 §3.1 原句「half simulated and half real-recorded」【论文】

### urgent2025-blind-real.md

#### ③无参考

- **场景键**：未查到
- **设备键**：non-laptop-unknown
- **笔记本段**：未查到
- **转写**：未查到（官网说 CER 只在「if transcription is available」时算，014 原卡已写）
- **语体**：未查到
- **与 D62 的关系**：盲测 900 条，每语种 150 条，真实约一半（014 原卡）。人评分在衍生集 `urgent2025-sqa`（本路新卡 `cards/urgent2025-sqa.md`）：26 个系统、每条 8 分，含中文。若开发阶段拿来评测：不必下本体，直接用 sqa 集里 `zh` 的行与已算好的各 verifier 分
- **014 卡需更正 / 补充**：原句「衍生 urgent2025-sqa 为 CC BY-NC-SA 4.0」核对一致（HF 标签）；补 sqa 集规模：约 168k 行，其中 `blind_test_mos` 23.4k 行【官方文档】

### urgent2026-track1-blind-real.md

#### ①消费设备

- **场景键**：other
- **设备键**：non-laptop-unknown
- **笔记本段**：没有设备信息；`track1/blind_noisy.zip`（115 MB）内 840 条文件名只有 `fileid_N.flac`，另带 `__MACOSX/` 资源叉文件（打包者用 Mac 压缩，与录音设备无关）【实测：zip 目录】
- **转写**：未查到
- **语体**：未查到
- **与 D62 的关系**：真实约 480 条 + 模拟 360 条（据论文）；zip 115 MB
- **014 卡疑误**：原句「文件 track1/blind_noisy.zip 的路径未在页面核到」→ 已核到，路径存在，大小 114,939,046 字节【实测：HF 文件树】

#### ③无参考

- **场景键**：未查到（「collected from public sources」）
- **设备键**：non-laptop-unknown
- **笔记本段**：未查到
- **转写**：未查到
- **语体**：未查到
- **与 D62 的关系**：真实 480 条 + 模拟 360 条（014 原卡）。Track2 是「增强后语音质量评估」，即 verifier 本身的比赛；Track2 数据与人评分是否公开本路未核。若开发阶段拿来评测：只作泛化抽检，盲测语种不含中英
- **014 卡需更正 / 补充**：无新发现

### voices-corpus.md

#### ①消费设备

- **场景键**：home-quiet（家具齐全的真实房间）, other
- **设备键**：non-laptop-studio（SHURE SM58 动圈）, non-laptop-close-talk（AKG 417L / SM11 领夹，摆在房间里）
- **笔记本段**：没有
- **转写**：有（LibriSpeech 文本）
- **语体**：朗读（有声书回放）；babble 为回放
- **与 D62 的关系**：回放重录。开发阶段若用于评测：取单麦单距离子集；全集 417.5 GiB，需挑子集
- **014 卡疑误**：无

## 二、各路的疑误与复核记录

### ①消费设备 · 怀疑派复核（2026-11-03）

> 方法同上：只用 HTTP Range 读 zip 末尾目录（每次 < 8 MB，未下载音频），GitHub contents 接口读文件名，回论文原文核句子。
- 重数并核实无误：DNS1 dev 真实 300 = `ms_` 150 + `audioset_` 150；DNS2 V2 blind laptop 192、macbook 6、headset 97、webcam 41、emotional 102、laughter 34；DNS3 V3 600 = realrec 440 + 非真实 160、laptop 137；全合集自填 mac 类 31 条（V2 6 + V3 24 + 情绪 1）；DNS4 blind 859 = Mobile 638 + Desk 221；DNS5 dev Track2 Mobile 413 / Desk 187、blind Track2 364、`ap` 段 128（Realtek 36）、cafe 4；URGENT 2026 `blind_noisy.zip` 114,939,046 字节；URGENT 2024 数据卡「no labels or meta information」原句；AEC README「captured on Windows PCs」原句；DR-VCTK datashare 页标题「Small subset version」；DNS 2022 论文「70% of the test clips」原句。
- 推翻 / 改正 9 处：①DNS3「情绪集 584」错，实为 600，014 索引行的 600 是对的；②DNS5 blind Track1 388 → 389；③DNS5 dev「Mobile 413 与论文冲突」已解：论文自相矛盾，且 dev Track2 是 DNS4 blind 的复用；④DNS5 `mt` 段并非无设备，81 条标 Windows、44 条标安卓；⑤DNS1 dev 漏了 `SurfaceBook` 44 条（笔记本）；⑥DNS2 合成唱歌是 50 条不是 45；⑦DNS2 / DNS3 设备键 `mbp-builtin` 不成立（只有 1 条写 Macbookpro，其余写 macbook / mac），改为 `laptop-other` 并注明；⑧NISQA-LIVETALK 的笔记本是录音端，不是说话人的麦；⑨Clarity / CHiME 两节的「与 D62 的关系」原写了结论（不建议用 / 不可用），改为只列事实，Clarity 设备键 `other` 不在暂定键里，改 `unknown`。
- 未复核（保留原文）：VOiCES、DAPS、ACE、AEC 2022 / 2023 文件数、DNS5 副语言片段与 dev 情绪 72 条。

### ②按地点 · 014 卡疑误（未改原卡）

1. `aspire-stirling.md`：原句「iPad mini 2 距说话人 90 cm，接领夹麦」——**怀疑派已回论文核实，014 原句正确**：arXiv 1910.00424 §2 原文「A collar microphone was also connected to the iPad」，iPad 录像、音频走领夹麦。错的是本补充原稿把 iPad 当平板麦（tablet）录音，已改为 close-talk。
2. `aishell-5.md`：原卡未提 AISHELL-5 即 ICASSP 2024 ICMC-ASR 挑战数据修正后的开源版（修了截断与转写错位），评测集沿用 ICMC-ASR 设置【论文：arXiv:2505.23036，经检索摘录】。建议补一句，避免把 ICMC-ASR 当成另一份数据集重复登记。
3. `chime5-chime6.md`：原卡「参考」只写转写有时间戳，未写 `[laughs]` 等非言语标签；补充见上（不是错误，是漏项）。
4. `ami.md`：原卡未提对话行为层的 backchannel 标签与未公开的情绪层；补充见上（漏项）。
5. 其余各卡本轮未发现与原文冲突之处。
6. 怀疑派复核（2026-11-03）：已改 aspire（设备键与评测通道）、notsofar-1（笑声事件标签降为未核实）、dns5（情绪 6 类改按 DNS4 论文原文，原稿「笑、哭、喊、惊讶」并非 014 dns4 卡写法，014 卡里没有类别）；aishell-5 即 ICMC-ASR 修正版已由论文正文核实（修了截断与转写错位、沿用其评测集）；chime6 `[laughs]` `[noise]` 定义与 AMI backchannel、情绪层不公开已在官方页核实。补了 icsi、misp2021、ms-aec-challenge-real 三节。

### ④中文 · 014 卡疑误（不改原卡，供汇总时裁定）

1. **misp-meeting.md 设备描述不全**：原句「桌面 8 麦阵列 + 360° 全景相机」。补充：8 麦阵集成在讯飞智能办公本 X3（消费级设备）里，全景相机另有 2 麦 48 kHz 音频。出处：arXiv:2505.13971 §2.2【论文】
2. **misp-meeting 场数与人数口径不一**：014 索引写「163 场」、卡写「共 286 人」；MISP 2025 挑战论文写 train / dev / eval = 72 / 9 / 9 场（共 90 场），参与者 233 + 15 + 15 = 263 人，train 每场 2 h、dev / eval 每场 20 min。可能是 ACL 论文另有口径（014 卡出处含 aclanthology 2025.acl-long.753，本路未读），未裁定。出处：arXiv:2505.13971 §2.1【论文】
3. **misp-meeting「每场 4–8 人」**：挑战论文 PDF 抽出文字为「48 participants」，应是「4–8」排版丢了连字符，与 014 一致，无误。
4. **怀疑派复核（2026-11-03）**：MISP-Meeting 办公本 X3、197 × 134 mm、16 kHz / 32-bit、相机 2 麦 48 kHz、头戴 44.1 kHz 接 Zoom F8N、72 / 9 / 9 场、233 + 15 + 15 人、重叠 57.30%，逐条回 arXiv:2505.13971 §2 核对一致；AISHELL-4（SLR111 四个包大小、只有 8 通道阵列）与 AISHELL-5（SLR159 各分集时长与包大小、仅 train 有近讲、噪声 13 GB 约 40 h）回 OpenSLR 页核对一致。改动：misp-meeting 设备键原写「other」不在暂定设备键里，改并入 conf-device 并括注；新增 common-voice.md 一节（中文子集，原调研漏补）。
5. 其余 5 张卡本路抽查的数字（AISHELL-4 包大小、AISHELL-5 分集时长与「仅 train 有近讲」、AliMeeting 近讲同步、MISP2021 无下载链接）与官方页一致，未发现错误。
