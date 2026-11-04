# 014 卡补充字段（r1 消费设备路）

> 2026-11-03 · 手下 r1-device。014 原卡不改，只在这里补 card-template 的 6 个字段：场景键 · 设备键 · 笔记本段 · 转写（单列）· 语体 · 与 D62 的关系；每节末尾「014 卡疑误」写发现的问题。
> 键为暂定键（见 `../card-template.md`）。
> 本路新方法【实测】：用 HTTP Range 只读远端 zip 末尾的文件目录（每次读取 < 8 MB，未下载音频），或用 GitHub / Hugging Face 的文件树接口，拿到逐条文件名后统计设备字段。脚本留在本机临时目录，未入库。下文凡标【实测：文件名】的数字都来自这一步。

### dns1-interspeech2020-real-testset.md
- **场景键**：open-office, meeting-room, cafe（dev 内部真实录音文件名含 cafe / cafeteria / cafee 共 27 条）, other（AudioSet 片段）
- **设备键**：headset, conf-device（文件名 speakerphone）, laptop-other（文件名 `SurfaceBook` 44 条，Surface Book 是可拆键盘的笔记本【推断：据产品名】）, unknown（AudioSet 与 blind 段）
- **笔记本段**：dev：MS 内部 150 条文件名带 `headset` 57 / `speakerphone` 93；speakerphone 里 55 条带 surface 字样，其中 `SurfaceBook` 44 条、只写 `surface` 11 条（是 Surface Book 还是 Surface Pro 平板未写）；无 Mac 字样。「SurfaceBook」是否指用了机身内置麦，文件名没写【实测：文件名】【推断】。blind：300 条真实录音文件名只有 `realrec_fileid_N`，不能按设备筛【实测：文件名，分支 interspeech2020/master】
- **转写**：没有
- **语体**：朗读与自由说未区分；无笑声 / 附和 / 情绪标注
- **与 D62 的关系**：真实录音（dev MS 内部段 + blind MTurk 段）+ 手挑 AudioSet 片段。开发阶段若用于评测：blind 300 条真实段（每条约 10 s），或 dev 150 条 MS 内部段；直接在 GitHub 仓库（git LFS），大小未查到
- **014 卡疑误**：原句「dev 4 类各 300 条（其中真实 600 条：MS 内部 300 + AudioSet 300）」。仓库 `datasets/test_set/real_recordings/` 实有 300 条 = `ms_` 150 + `audioset_` 150；blind 为 realrec 300 + noreverb 150 + reverb 150【实测：GitHub contents API，分支 interspeech2020/master】

### dns2-icassp2021-real-testset-pdns.md
- **场景键**：home, open-office, other（文件名噪声字段：狗叫、打字、婴儿哭、风扇等；无地点字段）
- **设备键**：laptop-other（含 6 条自填 `macbook`，型号与是否内置麦未核，不能归 `mba-builtin` / `mbp-builtin`）, headset, desktop（webcam）, phone, unknown
- **笔记本段**：有，可粗筛。T1 blind（合集 `noisy_blind_testset_v2_challenge_withSNR_16k/`，700 条 = 真实 650 + 合成唱歌 50（女 40、男 5、合唱 5））文件名里含 `laptop` 192 条、`macbook` 6 条、`headset` 97、`webcam` 41、`surface` 24、手机类 25。设备字段是众包者自由填写，写法不统一（如 `Win_laptop_mic`、`laptopmic`、`builtinmicrophone`，甚至有未替换的占位符 `your_device_type`），只能用正则粗筛，不可靠【实测：文件名】
- **转写**：没有
- **语体**：文件名分 english / nonenglish / emotional（emotional 102 条；`laughter` 34 条）；附和无标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包 + MS 内部）。开发阶段若用于评测：T1 blind 里文件名含 laptop / macbook 的约 200 条；合集 zip 778 MB（V2+V3+情绪+鼠标点击一起）【实测：HTTP 头 Content-Length】

### dns3-interspeech2021-real-testset.md
- **场景键**：home, open-office, other
- **设备键**：laptop-other（含自填 `macbook` 12、`mac` 12、情绪集 `Macbookpro` 1，型号与是否内置麦未核）, headset, desktop, phone, unknown
- **笔记本段**：有，可粗筛。V3 blind（600 条 = 真实 440 + 非真实 160）文件名含 `laptop` 137 条、`macbook` 12 条（同一众包者 `A1RK5KQQ320OH4`，英语男声，打字 / 风扇 / 厨房 / 洗碗机等，near / far 都有）、`mac` 12 条（同一人 `XMQ6Y5VEXRUU`，钢琴背景 far）、`headset` 124、`surface` 46。同包的情绪集 600 条（怀疑派重数，见文末）：`laptop` 252（不分大小写）、手机类 142、`headset` 114、`Macbookpro` 1；鼠标点击集 99 条：手机类 30、`headset` 29、`laptop` 10【实测：文件名】
- **转写**：没有
- **语体**：情绪集 600 条（文件名标 sad / angry / crying / yelling / happy / laughter；`laugh` 不分大小写 100 条）；V3 另有 musical（乐器背景）78 条；附和无标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包，1–5 m 内非头戴）+ 合成段。开发阶段若用于评测：V3 + 情绪集里文件名带 laptop / mac 的约 400 条；合集 zip 778 MB
- **014 卡疑误**：原句「V3 blind 600 条」，未说明其中 160 条不是 `realrec`（`nonenglish_synthetic` 59、`nonenglish_tonal_synthetic` 51、`stationary_english_tug`（Graz 干净语音 + 稳态噪声【推断】）50），真实只有 440 条；原句（014 `README.md` 索引行，非 dns3 卡正文）「合集另含 600 条情感片段、100 条鼠标点击片段」：情感片段**确为 600 条**（怀疑派重数 zip 目录：`handpicked_emotion_testclips_16k_600_withSNR/` 下 600 个 .wav、名字不重复），本补充原写的 584 是错的；鼠标点击实为 99 条，014 的 100 差 1【实测：文件名】

### dns4-icassp2022-real-testset-pdns.md
- **场景键**：home, open-office, street, car, other（文件名噪声字段：Typing 83、Water 68、Fan 62、Door 62、MouseClicks 61、Chips 56、Cutlery 53、Street 29、InsideCar 7、CopyRoom 10 等；无 babble / 咖啡馆字段）
- **设备键**：phone, desktop（文件名 `desktop` / `Desk`，不分台式与笔记本）
- **笔记本段**：blind 859 条每条文件名都有设备字段：`Mobile` 638、`desktop` / `Desk` 221；**笔记本与台式机分不开**，无 Mac 字样。另有距离字段 Near / Far、说话人字段 Primary / Neighbor（Neighbor 121 条）【实测：文件名】
- **转写**：部分（dev 有 `DNSChallenge4_devtest.tsv`；blind 标 TBD 未公开）
- **语体**：blind 情绪段 98 条（sad、angry、yelling、laughter 14、crying、happy）；无附和标注【实测：文件名】
- **与 D62 的关系**：真实录音（众包）。开发阶段若用于评测：blind 的 Desk 段 221 条 × 10 s（约 37 min【推断】）；zip 1.66 GB
- **014 卡疑误**：无新发现；「638 条手机」与 README 一致，且文件名可逐条复核（怀疑派重数 `blind_testset_bothtracks.zip` 目录：testclips 859 = Mobile 638 + Desk 221，enrollment 178，zip 1,658,455,878 字节，一致）。另：DNS5 dev Track2 的 600 条里 599 条与本集 blind 同名，两集不要重复计数【实测：zip 目录比对】

### dns5-icassp2023-real-testset.md
- **场景键**：home, open-office, cafe（Cafe 4 条）, transit（Airport 25、Announcements 9）, street（Traffic 12）, car, other
- **设备键**：headset, conf-device（回音消除免提）, desktop（Logitech C920、LifeCam 摄像头麦）, laptop-other（推断：`Microphone_Array_RealtekR_Audio` 36 条，Realtek 阵列麦常见于 Windows 笔记本内置）, phone（Pixel 4a、Moto G 共 4 条）, unknown
- **笔记本段**：blind 分 4 个来源前缀：`ap`（349 条，再分 `ap_desktop` 176 / `ap_mobile` 173，并写全设备型号，如 Plantronics、Jabra、EPOS、Realtek）· `mt`（众包；其中 `mt-win_` 81 条标 Windows 电脑、`mt-and_mobile` 44 条标安卓手机，其余约 45 条无设备字段）· `sa`（131 条，只有 GUID）· `lk`（实验室，距离 0.5 / 1 / 2 m）。`ap` 段能按型号筛，`mt` 段只能分 Windows / 安卓；Windows 电脑里笔记本与台式机分不开；无 Mac / MacBook 字样【实测：文件名，怀疑派重数】
- **转写**：部分（dev Track2 有 `v5_devset_transcripts_track2.tsv`；Track1 只有 README；blind 未公开）【实测：文件名】
- **语体**：blind Track1 有副语言片段 `pl_cat1-throat-clear`、`pl_cat2-hmm-mhm`（附和）、`pl_cat3-Huh-what` 等 4 条，`Laughing` 2 条；dev Track2 情绪 72 条（laughter 11）【实测：文件名】
- **与 D62 的关系**：真实录音（众包 + 采集商 + 实验室）。开发阶段若用于评测：blind Track2 的 `ap` 段 128 条（含 Realtek 阵列 36 条、摄像头 11 条）；blind zip 3.89 GB、dev zip 3.24 GB
- **014 卡疑误**：
  1. 原句「blind 条数未查到」→ blind `noisy/` 目录 Track1 389 条（怀疑派重数，原写 388 差 1）、Track2 364 条（另各有同数 enrol）【实测：zip 目录】
  2. 原句「dev 全部笔记本或台式机」（出自论文 §3）→ dev Track2 600 条文件名里 `Mobile` 413 条、`Desk` 187 条；Track1 641 条全为 `Desk` + `HS`【实测：zip 目录】。怀疑派复核结论：以文件名为准。依据：①论文自相矛盾，摘要后的贡献列表原句「Headset, and Speakerphone both has desktop and mobile recordings in testsets」，§3 又写 dev「recorded using a laptop or desktop computer」【论文 arXiv 2303.11510】；②dev Track2 的 600 个文件名里有 599 个与 DNS4（2022）blind 的 859 个文件名逐字相同，即 dev Track2 是复用 DNS4 blind，DNS4 论文本就写 70% 用手机录【实测：两个 zip 目录比对】
  3. 原句「dev：非个性化 600 条、个性化 1443 条」→ dev zip `noisy/` 为 Track1 641 + Track2 600 条；1443 条个性化 dev 不在此 zip 内【实测：zip 目录】

### dns-challenge-blind-real.md
- **场景键**：home, open-office, other（按年份见上面各届）
- **设备键**：phone, desktop, laptop-other, headset, unknown
- **笔记本段**：按年份不同：2020 blind 无设备字段；2021 两届文件名带自由填写的设备字（含 macbook 约 31 条）；2022 只有 Mobile / Desk；2023 `ap` 段写型号、`mt` 段部分写 Windows / 安卓（见上面各节）【实测：文件名】
- **转写**：没有公开（blind 均未公开转写）
- **语体**：2021 合集有情绪 600 条、文件名带 laugh 约 100 条；2023 有附和 1 条（hmm-mhm）
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
- **设备键**：phone, laptop-other, unknown（说话人「spoke directly into the terminal device (i.e. a smartphone or laptop)」；Google Pixel 3 与笔记本是**录音端**，不一定是说话人的设备）
- **笔记本段**：可能有，不能筛。官方原句「The calls were recorded on a Laptop for the VoIP calls and on a Google Pixel 3 for the mobile phone calls」——笔记本是录通话的那一端，录的是经网络传过来的信号，不是笔记本麦直录；说话人哪几条用的是笔记本未写；逐条设备标签未查到【官方文档：NISQA Corpus wiki】
- **转写**：未查到
- **语体**：真人对设备说话；无笑声 / 附和标注
- **与 D62 的关系**：真实通话录音。开发阶段若用于评测：232 条，作无参考音质分的定标旁证
- **014 卡疑误**：无；全集另见 023 新卡 `../cards/nisqa-corpus.md`（他路所写）

### clarity-cec3-task2.md
- **场景键**：other（录音室，回放布局）
- **设备键**：unknown（实为助听器壳 3 麦；暂定键里没有助听器）
- **笔记本段**：没有
- **转写**：未查到
- **语体**：未查到
- **与 D62 的关系**：回放重录（真人佩戴、音箱回放）。设备是助听器壳麦，无笔记本段；全包 74 GB【沿用 014】
- **014 卡疑误**：无

### chime3-chime4-real.md（设备补充）
- **设备键**：tablet（平板四周 6 麦）, close-talk
- **笔记本段**：没有；平板通道与近讲通道分文件，可筛
- **与 D62 的关系**：真实录音（真人朗读 WSJ 句子，公交 / 咖啡馆 / 步行街 / 路口）；LDC 付费（LDC2017S24）、不可再分发。开发阶段若拿来评测：真实 dev 1640 句约 2.74 h 的平板通道，前提是先付费取得【沿用 014】

## 怀疑派复核（2026-11-03）
> 方法同上：只用 HTTP Range 读 zip 末尾目录（每次 < 8 MB，未下载音频），GitHub contents 接口读文件名，回论文原文核句子。
- 重数并核实无误：DNS1 dev 真实 300 = `ms_` 150 + `audioset_` 150；DNS2 V2 blind laptop 192、macbook 6、headset 97、webcam 41、emotional 102、laughter 34；DNS3 V3 600 = realrec 440 + 非真实 160、laptop 137；全合集自填 mac 类 31 条（V2 6 + V3 24 + 情绪 1）；DNS4 blind 859 = Mobile 638 + Desk 221；DNS5 dev Track2 Mobile 413 / Desk 187、blind Track2 364、`ap` 段 128（Realtek 36）、cafe 4；URGENT 2026 `blind_noisy.zip` 114,939,046 字节；URGENT 2024 数据卡「no labels or meta information」原句；AEC README「captured on Windows PCs」原句；DR-VCTK datashare 页标题「Small subset version」；DNS 2022 论文「70% of the test clips」原句。
- 推翻 / 改正 9 处：①DNS3「情绪集 584」错，实为 600，014 索引行的 600 是对的；②DNS5 blind Track1 388 → 389；③DNS5 dev「Mobile 413 与论文冲突」已解：论文自相矛盾，且 dev Track2 是 DNS4 blind 的复用；④DNS5 `mt` 段并非无设备，81 条标 Windows、44 条标安卓；⑤DNS1 dev 漏了 `SurfaceBook` 44 条（笔记本）；⑥DNS2 合成唱歌是 50 条不是 45；⑦DNS2 / DNS3 设备键 `mbp-builtin` 不成立（只有 1 条写 Macbookpro，其余写 macbook / mac），改为 `laptop-other` 并注明；⑧NISQA-LIVETALK 的笔记本是录音端，不是说话人的麦；⑨Clarity / CHiME 两节的「与 D62 的关系」原写了结论（不建议用 / 不可用），改为只列事实，Clarity 设备键 `other` 不在暂定键里，改 `unknown`。
- 未复核（保留原文）：VOiCES、DAPS、ACE、AEC 2022 / 2023 文件数、DNS5 副语言片段与 dev 情绪 72 条。

