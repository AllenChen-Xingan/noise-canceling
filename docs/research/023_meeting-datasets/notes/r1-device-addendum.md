# 014 卡补充字段（r1 消费设备路）

> 2026-11-03 · 手下 r1-device。014 原卡不改，只在这里补 card-template 的 6 个字段：场景键 · 设备键 · 笔记本段 · 转写（单列）· 语体 · 与 D62 的关系；每节末尾「014 卡疑误」写发现的问题。
> 键为暂定键（见 `../card-template.md`）。
> 本路新方法【实测】：用 HTTP Range 只读远端 zip 末尾的文件目录（每次读取 < 8 MB，未下载音频），或用 GitHub / Hugging Face 的文件树接口，拿到逐条文件名后统计设备字段。脚本留在本机临时目录，未入库。下文凡标【实测：文件名】的数字都来自这一步。

### dns1-interspeech2020-real-testset.md
- **场景键**：open-office, meeting-room, cafe（dev 内部真实录音文件名含 cafeteria 27 条）, other（AudioSet 片段）
- **设备键**：headset, conf-device（文件名 speakerphone）, unknown（AudioSet 与 blind 段）
- **笔记本段**：dev：MS 内部 150 条文件名带 `headset` 57 / `speakerphone` 93，部分 speakerphone 文件名带 `surface`（Surface 设备【推断】）；无笔记本 / Mac 字样。blind：300 条真实录音文件名只有 `realrec_fileid_N`，不能按设备筛【实测：文件名，分支 interspeech2020/master】
- **转写**：没有
- **语体**：朗读与自由说未区分；无笑声 / 附和 / 情绪标注
- **与 D62 的关系**：真实录音（dev MS 内部段 + blind MTurk 段）+ 手挑 AudioSet 片段。开发阶段若用于评测：blind 300 条真实段（每条约 10 s），或 dev 150 条 MS 内部段；直接在 GitHub 仓库（git LFS），大小未查到
- **014 卡疑误**：原句「dev 4 类各 300 条（其中真实 600 条：MS 内部 300 + AudioSet 300）」。仓库 `datasets/test_set/real_recordings/` 实有 300 条 = `ms_` 150 + `audioset_` 150；blind 为 realrec 300 + noreverb 150 + reverb 150【实测：GitHub contents API，分支 interspeech2020/master】

### dns2-icassp2021-real-testset-pdns.md
- **场景键**：home, open-office, other（文件名噪声字段：狗叫、打字、婴儿哭、风扇等；无地点字段）
- **设备键**：laptop-other, mbp-builtin（极少，见下）, headset, desktop（webcam）, phone, unknown
- **笔记本段**：有，可粗筛。T1 blind（合集 `noisy_blind_testset_v2_challenge_withSNR_16k/`，700 条 = 真实 650 + 合成唱歌 50）文件名里含 `laptop` 192 条、`macbook` 6 条、`headset` 97、`webcam` 41、`surface` 24、手机类 25。设备字段是众包者自由填写，写法不统一（如 `Win_laptop_mic`、`laptopmic`、`builtinmicrophone`，甚至有未替换的占位符 `your_device_type`），只能用正则粗筛，不可靠【实测：文件名】
- **转写**：没有
- **语体**：文件名分 english / nonenglish / emotional（emotional 102 条；`laughter` 34 条）；附和无标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包 + MS 内部）。开发阶段若用于评测：T1 blind 里文件名含 laptop / macbook 的约 200 条；合集 zip 778 MB（V2+V3+情绪+鼠标点击一起）【实测：HTTP 头 Content-Length】

### dns3-interspeech2021-real-testset.md
- **场景键**：home, open-office, other
- **设备键**：laptop-other, mbp-builtin（文件名 `macbook` / `mac`，见下）, headset, desktop, phone, unknown
- **笔记本段**：有，可粗筛。V3 blind（600 条 = 真实 440 + 合成 160）文件名含 `laptop` 137 条、`macbook` 12 条（同一众包者 `A1RK5KQQ320OH4`，英语男声，打字 / 风扇 / 厨房 / 洗碗机等，near / far 都有）、`mac` 12 条（同一人 `XMQ6Y5VEXRUU`，钢琴背景 far）、`headset` 124、`surface` 46。同包的情绪集 584 条：`laptop` 241、手机类 142、`headset` 114、`Macbookpro` 1；鼠标点击集 99 条：手机类 30、`headset` 29、`laptop` 10【实测：文件名】
- **转写**：没有
- **语体**：情绪集 584 条（文件名标 sad / angry / crying / yelling / happy / laughter；`laugh` 99 条）；V3 另有 musical（乐器背景）78 条；附和无标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包，1–5 m 内非头戴）+ 合成段。开发阶段若用于评测：V3 + 情绪集里文件名带 laptop / mac 的约 400 条；合集 zip 778 MB
- **014 卡疑误**：原句「V3 blind 600 条」，未说明其中 160 条是合成（`nonenglish_synthetic` 58、`nonenglish_tonal_synthetic` 51、`stationary_english_tug` 30 等），真实只有 440 条；原句「合集另含 600 条情感片段、100 条鼠标点击片段」，实际文件数为 584 与 99（目录名写 600）【实测：文件名】

### dns4-icassp2022-real-testset-pdns.md
- **场景键**：home, open-office, street, car, other（文件名噪声字段：Typing 83、Water 68、Fan 62、Door 62、MouseClicks 61、Chips 56、Cutlery 53、Street 29、InsideCar 7、CopyRoom 10 等；无 babble / 咖啡馆字段）
- **设备键**：phone, desktop（文件名 `desktop` / `Desk`，不分台式与笔记本）
- **笔记本段**：blind 859 条每条文件名都有设备字段：`Mobile` 638、`desktop` / `Desk` 221；**笔记本与台式机分不开**，无 Mac 字样。另有距离字段 Near / Far、说话人字段 Primary / Neighbor（Neighbor 121 条）【实测：文件名】
- **转写**：部分（dev 有 `DNSChallenge4_devtest.tsv`；blind 标 TBD 未公开）
- **语体**：blind 情绪段 98 条（sad、angry、yelling、laughter 14、crying、happy）；无附和标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包）。开发阶段若用于评测：blind 的 Desk 段 221 条 × 10 s（约 37 min【推断】）；zip 1.66 GB
- **014 卡疑误**：无新发现；「638 条手机」与 README 一致，且文件名可逐条复核

### dns5-icassp2023-real-testset.md
- **场景键**：home, open-office, cafe（Cafe 4 条）, transit（Airport 25、Announcements 9）, street（Traffic 12）, car, other
- **设备键**：headset, conf-device（回音消除免提）, desktop（Logitech C920、LifeCam 摄像头麦）, laptop-other（推断：`Microphone_Array_RealtekR_Audio` 36 条，Realtek 阵列麦常见于 Windows 笔记本内置）, phone（Pixel 4a、Moto G 共 4 条）, unknown
- **笔记本段**：blind 分 4 个来源前缀：`ap`（文件名写全设备型号，如 Plantronics、Jabra、EPOS、Realtek）· `mt`（众包，有噪声与邻座字段，无设备）· `sa`（只有 GUID）· `lk`（实验室，距离 0.5 / 1 / 2 m）。只有 `ap` 段能按设备型号筛；无 Mac / MacBook 字样【实测：文件名】
- **转写**：部分（dev Track2 有 `v5_devset_transcripts_track2.tsv`；Track1 只有 README；blind 未公开）【实测：文件名】
- **语体**：blind Track1 有副语言片段 `pl_cat1-throat-clear`、`pl_cat2-hmm-mhm`（附和）、`pl_cat3-Huh-what` 等 4 条，`Laughing` 2 条；dev Track2 情绪 72 条（laughter 11）【实测：文件名】
- **与 D62 的关系**：真实录音（众包 + 采集商 + 实验室）。开发阶段若用于评测：blind Track2 的 `ap` 段 128 条（含 Realtek 阵列 36 条、摄像头 11 条）；blind zip 3.89 GB、dev zip 3.24 GB
- **014 卡疑误**：
  1. 原句「blind 条数未查到」→ blind `noisy/` 目录 Track1 388 条、Track2 364 条（另各有 enrol 目录）【实测：zip 目录】
  2. 原句「dev 全部笔记本或台式机」（出自论文 §3）→ dev Track2 600 条文件名里 `Mobile` 413 条、`Desk` 187 条，与论文说法冲突；Track1 641 条全为 `Desk` + `HS`【实测：zip 目录】。以文件名为准还是以论文为准，待怀疑派复核
  3. 原句「dev：非个性化 600 条、个性化 1443 条」→ dev zip `noisy/` 为 Track1 641 + Track2 600 条；1443 条个性化 dev 不在此 zip 内【实测：zip 目录】

### dns-challenge-blind-real.md
- **场景键**：home, open-office, other（按年份见上面各届）
- **设备键**：phone, desktop, laptop-other, headset, unknown
- **笔记本段**：按年份不同：2020 blind 无设备字段；2021 两届文件名带自由填写的设备字（含 macbook 约 31 条）；2022 只有 Mobile / Desk；2023 只有 `ap` 段写型号（见上面各节）【实测：文件名】
- **转写**：没有公开（blind 均未公开转写）
- **语体**：2021 合集有情绪 584 条、笑声约 99 条；2023 有附和 1 条（hmm-mhm）
- **与 D62 的关系**：真实录音；同上各届
- **014 卡疑误**：原句「约 70% 用手机录（arXiv 2202.13288 §3.3）」只适用于 2022 届；原句「语言：未查到」→ 2021 两届文件名有 nonenglish 与 tonal（普通话、粤语、旁遮普、越南语等）字段【实测：文件名】【沿用 014 dns3 卡】

### urgent2024-blind-real.md
- **场景键**：other（未标）
- **设备键**：unknown
- **笔记本段**：没有设备信息。HF 数据卡原句「no labels or meta information are provided for the blind test set」；数据卡列的源语音含 CHiME-6、VOiCES、EasyCom、YouTube 提取音频，真实段是否取自这些未写【官方文档】
- **转写**：没有
- **语体**：未查到
- **与 D62 的关系**：真实 500 条（来源未写）。开发阶段若用于评测：blind parquet 480 MB（含 500 模拟 + 500 真实）【实测：HF 文件树】

### urgent2026-track1-blind-real.md
- **场景键**：other
- **设备键**：unknown
- **笔记本段**：没有设备信息；`track1/blind_noisy.zip`（115 MB）内 840 条文件名只有 `fileid_N.flac`，另带 `__MACOSX/` 资源叉文件（打包者用 Mac 压缩，与录音设备无关）【实测：zip 目录】
- **转写**：未查到
- **语体**：未查到
- **与 D62 的关系**：真实约 480 条 + 模拟 360 条（据论文）；zip 115 MB
- **014 卡疑误**：原句「文件 track1/blind_noisy.zip 的路径未在页面核到」→ 已核到，路径存在，大小 114,939,046 字节【实测：HF 文件树】

### voices-corpus.md
- **场景键**：home（家具齐全的真实房间）, other
- **设备键**：studio（SHURE SM58 动圈）, close-talk（AKG 417L / SM11 领夹，摆在房间里）
- **笔记本段**：没有
- **转写**：有（LibriSpeech 文本）
- **语体**：朗读（有声书回放）；babble 为回放
- **与 D62 的关系**：回放重录。开发阶段若用于评测：取单麦单距离子集；全集 417.5 GiB，需挑子集
- **014 卡疑误**：无

### daps.md
- **场景键**：open-office（office1/2）, meeting-room（confroom1/2）, home（livingroom1、bedroom1）, street（balcony1 车流）
- **设备键**：tablet（iPad Air）, phone（iPhone 5S）
- **笔记本段**：没有；按版本目录（如 `ipad_confroom1`）可筛设备【沿用 014】
- **转写**：有（朗读脚本，篇章级，无时间戳）
- **语体**：朗读；无笑声 / 附和 / 情绪
- **与 D62 的关系**：回放重录（录音棚版经音箱放、设备重录）。开发阶段若用于评测：iPad confroom 两个版本，大小未查到（全包 16.1 GB）
- **014 卡疑误**：无

### dr-vctk.md
- **场景键**：open-office（中等办公室，开窗 / 关窗）
- **设备键**：mba-builtin（MacBook Air 2 通道）, phone（iPhone 5S）, tablet（iPad）, studio（Apogee MiC、Blue Snowball）
- **笔记本段**：有，且能按设备筛（8 通道分设备）；机型年份未写
- **转写**：有（VCTK 文本）
- **语体**：朗读；无笑声 / 附和 / 情绪
- **与 D62 的关系**：回放重录（距麦 2 m）。开发阶段若用于评测：MacBook Air 两通道，16 kHz；全包 1.67 GB
- **014 卡疑误**：datashare 页标题为「Device Recorded VCTK (Small subset version)」，014 卡未写这是子集版【官方文档】；End-user Licence 本轮仍未读到正文

### ace-challenge-corpus.md
- **场景键**：open-office, meeting-room, other（报告厅、带咖啡厅的大堂）
- **设备键**：laptop-other（Chromebook Pixel 内置 2 麦）, array, studio
- **笔记本段**：有（Chromebook 档 1.1 GB，单独打包，可筛）；无 Mac
- **转写**：没有（目标语音为消声室录音，babble 无转写）
- **语体**：朗读；babble 为 4–7 人同时朗读
- **与 D62 的关系**：真实 RIR + 真实噪声 + 需自己卷积相加的语音（真实噪声 + 人工混合）。开发阶段若用于评测：Chromebook 档 1.1 GB
- **014 卡疑误**：无

### ms-aec-challenge-real.md
- **场景键**：home, open-office
- **设备键**：laptop-other, desktop, phone（`fullband_mobile`）
- **笔记本段**：数据集 README 原句「The real files were captured on Windows PCs」；文件名只有 GUID + 场景（`nearend_singletalk` 等），**不能按设备筛**；全为 Windows，没有 Mac【官方文档：datasets/README.md】【实测：git tree 文件名】
- **转写**：没有
- **语体**：近端单讲为朗读或自由说未写
- **与 D62 的关系**：真实录音（众包 PC）。开发阶段若用于评测：各届 blind 的 `nearend-singletalk` 子目录（2022 届 400 个文件、2023 届 600 个文件，含 mic / lpb）【实测：git tree】
- **014 卡疑误**：README 原句「Some PCs may have audio DSP processing in the send and/or receive path, even though we used a raw mode」——014 卡已写「设备 DSP 残留」，与此一致

### nisqa-livetalk.md
- **场景键**：cafe, car, transit, street, other（电梯、商场）
- **设备键**：phone（Google Pixel 3）, laptop-other（VoIP 段，型号未写）
- **笔记本段**：有（VoIP 通话在笔记本上录），但逐条设备标签未查到
- **转写**：未查到
- **语体**：真人对设备说话；无笑声 / 附和标注
- **与 D62 的关系**：真实通话录音。开发阶段若用于评测：232 条，作无参考音质分的定标旁证
- **014 卡疑误**：无；全集另见 023 新卡 `../cards/nisqa-corpus.md`（他路所写）

### clarity-cec3-task2.md
- **场景键**：other（录音室，回放布局）
- **设备键**：other（助听器壳 3 麦）——暂定键里没有助听器，写 unknown
- **笔记本段**：没有
- **转写**：未查到
- **语体**：未查到
- **与 D62 的关系**：回放重录（真人佩戴、音箱回放）。与笔记本无关，开发阶段不建议用
- **014 卡疑误**：无

### chime3-chime4-real.md（设备补充）
- **设备键**：tablet（平板四周 6 麦）, close-talk
- **笔记本段**：没有；平板通道与近讲通道分文件，可筛
- **与 D62 的关系**：真实录音（真人朗读 WSJ 句子，公交 / 咖啡馆 / 步行街 / 路口）；LDC 付费，开发阶段不可用
