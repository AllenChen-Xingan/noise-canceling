# 原始报告 · skeptic-A（workflow wf_c790c606-7d6 · 2026-11-03，未经合成，供追溯）

# 怀疑派「甲」审稿报告：第 1–3 路（标准 / 挑战赛 / 产品）

## 要点（5 行）
1. 第 1 路（标准）的声级数字我逐表核过，基本准确。错误集中在 5 处：段数「约 45 段」对不上；「都用 HATS（人工头）录」不成立；TS 103 224 表 8 的列名写错；Talker 一行的下限 54.6 应为 54.1；P.1100 引的是 2015 年旧版工况。另外漏掉了超宽带门槛（N-MOS ≥ 2.7）。
2. 第 1 路最要紧的漏项：3GPP 根本没有「桌面免提」的环境噪声测试，只测手持、手持免提和电气接口。MacBook 放在桌上开会，最接近的正是桌面免提，标准门槛直接套不上。
3. 第 2 路（挑战赛）把 DNS 2023 噪声名单分成 5 组，其中「交通和户外」「公共场所」两组名是报告自己加的。原文只分 3 组：Office scenarios、Home scenarios、Appliances。AliMeeting 的噪声原文是「made naturally」（自然产生），报告写成了「人为制造」，意思反了。
4. 第 2 路说「6 个地点在所有公开挑战里都没覆盖」，说过头了。DCASE 2016 的 TUT 声景集（不是语音集）就有 library、train、cafe/restaurant 三类场景。这句应收窄为「语音增强 / 识别类挑战里没有」。
5. 第 3 路（产品）引文大多核得上。Zoom 个性化音频隔离不必标「二手」：官方帮助页原文是「recommended for crowded environments」。Krisp 的「cabin rumble」属于手机通话组（车、街、店、地铁），不能算给高铁/飞机客舱背书。

---

## 一、核对方法
- 【实测】标准和论文：我用 `pdftotext` 读本会话缓存的官方 PDF，逐行比对，没有写任何文件。缓存目录：`/Users/allenchen/.claude/projects/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-scenes/d30acd32-697d-4346-9811-adea3e1dbdee/tool-results/`。
- 【实测】网页：用 WebFetch 现场打开。下文「核对」标「✓」的是我亲眼看到原文的；标「未核」的是我也没打开原文的。

---

## 二、第 1 路（标准）逐条判定

| # | 报告原结论 | 判定 | 我看到的原文 / 理由 |
|---|---|---|---|
| 1-1 | ES 202 396-1 和 TS 103 224「列出约 45 段噪声」 | **删除，改数字** | 我按表数文件条目：ES 版表 1–14 共 38 条，表 15 共 25 条，表 16–18 共 9 条，合计 72 条（其中表 15 与 TS 版同源）。TS 版表 8 共 22 条，表 8A 共 10 条，表 9 共 9 条，表 10 共 24 条，合计 65 条。「约 45」对不上任何一种口径 |
| 1-2 | 「都用 P.58 人工头（HATS）在真实地点录制」 | **降级，改写** | 有 3 处例外。ES §8.3 表 16–18 原文是「Recording at the mirror position … 2 cardiod microphones」，没有人工头。ES 表 15 的 Inside_Light_Aircraft 原文是「binaural headset was worn by the test operator」，是真人戴耳机。TS §8.4 表 10 只写「recorded with the microphone array according to clause 5.3」，没提 HATS |
| 1-3 | ES 表 1–18 的各项声级 | **保留** ✓ | 逐行一致（ES V1.10.2 第 47–52 页）。两处可疑重复确实是原文如此：表 9 呼叫中心和表 10 客厅都是 56,6/57,8；表 8 两段体育馆都是 68,0/68,7。报告建议引用前实测核对，这条成立 |
| 1-4 | 表 8 Cafeteria_Noise（收银台）→ cafe（近似）；TS 版 Cafeteria → cafe / canteen | **降级（张冠李戴风险）** | 原文描述有两点要注意。第一，ES 表 8 的 `Cafeteria_Noise_binaural` 原文是「Recording at sales counter」，即收银台，不是咖啡馆。第二，`Mensa_binaural` 的原文才是「Recording in a cafeteria」；3GPP 表 2d 的写法相同。另外，英文 cafeteria 指自助食堂，TS 版的「inside a cafeteria」更该归到 canteen。把它们映射到 cafe 证据不足，应标「推断，弱」 |
| 1-5 | TS 103 224 表 8，列名「手持 / 手持免提 / 桌面免提」 | **删除，改列名** | 表 8 标题原文是「for handset and handheld hands-free」，**没有桌面免提列**。报告车内那一行的 3 个数，其实是 80、100、130 km/h 三档车速的范围，不是 3 种布置。另外，MidSize 100 km/h 的下限 65.1 取自免提列，手持列下限是 65.2 |
| 1-6 | TS 表 8、8A、9 的其余范围 | **保留** ✓ | 逐项核对一致。包括 Airport 77.2–78.8（50 s，原文「Airport hallway with overhead public address announcement」）、Conference 桌面 48.2–55.0、StationBuilding 65.1–66.2 |
| 1-7 | TS 表 10 Talker 54.6–61.0 dB SPL(A) | **改数字** | 原文最小值是 Talker_DUT3_AC1 第 8 声道的 54,1，DUT2_AC1 也有 54,3。应改成 54.1–61.0。其余各行（Sink 58.1–66.4、Fridge 38.5–44.8、All 61.1–66.7）一致；All 一行报告写的 61.3 也应改为 61.1 |
| 1-8 | 表 10 文件名抄错（括号里写成 Sink_DUT3_AC2、Talker_…） | **保留** ✓ | 原文第 49 页确实如此 |
| 1-9 | 附录 A 表 A.1 的混响参数 | **保留，补细节** ✓ | 低混响 T60：250 Hz 为 0,4 s，500 Hz–8 kHz 为 0,3 s，C50 为 10,5 dB。中混响 T60 为 0,45–0,5 s，C50 为 6,0 dB |
| 1-10 | 26.132 §5.1.5 方法 2 更好，参考声取 TrainStation_bin 15.0–22.5 s | **保留** ✓ | 原文「Method 2 is preferred」，文中第 1537、1551 行 |
| 1-11 | 26.132 表 2d / 2d2 的 8 段、5 段噪声，语音 −1.7 / +1.3 dBPa，16 句取平均 | **保留** ✓ | §7.12.1、§7.12.2 逐字一致。注意表 2d 原文第 6 段写的是「Recording at sales counter」 |
| 1-12 | 26.131 门槛「例如手持时 N-MOS ≥ 2.3」 | **降级（不完整）** | 窄带 §5.11 和宽带 §6.10 核对一致 ✓。但报告漏了超宽带 / 全带 §7.10：手持强制 S-MOS ≥ 3.7、N-MOS ≥ 2.7，性能目标 3.9 / 3.3；手持免提建议 S ≥ 3.4、N ≥ 2.3。26.132 对应 §9.12 和表 2i。对 48 kHz 降噪来说，2.3 不是该看的门槛 |
| 1-13 | 26.132 参考文献版本是 V1.4.1（2012）和 V1.2.1（2015） | **保留** ✓ | 参考文献 [35]、[43] |
| 1-14 | P.835 附录 III.5 / III.6 / III.2 | **保留** ✓ | 与 Amd.1（10/2007）原文一致。小修正：babble 用 4 支音箱各放一路，原文只要求「in at least some of the tests」，不是每次都要 |
| 1-15 | P.1100 / P.1110 的 4 种工况（风扇比行驶噪声高约 6 dB(A)）和 3.0 / 2.5 门槛 | **降级为「二手，且已过时」** | DAGA 2017 论文的表 1、表 2 引的是 01/2015 版。同一篇的表 6 显示，03/2017 版已把风扇改成「Medium setting; AIRFLOW=Directed to windows」。表 5 还给了 P.1110 宽带新门槛：安静时 N ≥ 3.9、S ≥ 4.6、G ≥ 3.9；≤ 80 km/h 时 3.0；> 80 km/h 时 N ≥ 2.6、S ≥ 3.0、G ≥ 2.9。现行 10/2025 版未核。「67–79 dB(A)」只是表 3 里 4 个对比案例的声级，不是 44 台实测的分布 |
| 1-16 | HEAD 两份数据表标成【论坛/博客，个人说法】 | **改来源类型** | 这是测试仪器厂商的产品数据表，应标「厂商资料（转述未公开规范，二手）」。内容核对一致 ✓：Teams 数据表 Rev.10（05/26）第 3–5 页写先消声室、后混响室，「3QUEST S-/N-MOS (various background noise types)」，「Distractor attenuation」属于可选模块 Open Office Headset，还有 EMTR 模块。Zoom 数据表第 1 页的规范版本、第 5–6 页「reverberant, yet quiet office-like room」、3PASS lab 8 支音箱 / flex 4 支加低音炮，都一致 |
| 1-17 | Teams 认证页的引文 | **保留** ✓ | learn.microsoft.com「Special usage designations」逐字一致 |
| 1-18 | §6 覆盖表：transit-hub「好」 | **降级为「中」** | 真正对应候车厅的只有 2 段：Airport 50 s、StationBuilding 35 s，各 1 条录音。其余都是站台。1–2 段录音不足以称「好」 |

## 三、第 2 路（挑战赛）逐条判定

| # | 报告原结论 | 判定 | 我看到的原文 / 理由 |
|---|---|---|---|
| 2-1 | DNS 2022 盲测集 859 条、每条 10 s、70% 用手机录 | **保留** ✓ | arXiv 2202.13288 §3.3 |
| 2-2 | DNS 2022 图 1 的 19 类百分比，合计 99.9% | **保留** ✓ | pdftotext 抽到柱上数值，与报告一致。条数是用 859 × 占比算的推断，报告已标明 |
| 2-3 | 「home-family 合计 29.1%」 | **降级（内部矛盾）** | 29.1% 把咀嚼 2.1% 算了进去，但报告自己的表里把咀嚼标成「通用」。去掉咀嚼是 27.0% |
| 2-4 | 「只有 2022 届公布了噪声分布数值」 | **保留，加限定** | 只对盲测集成立。DNS 2020 开发集原文给了「15 clips from 12 noise categories … 180」，也是数值分布 |
| 2-5 | DNS 2020 / 2021 的分组、条数和 RT60 300–1300 ms | **保留** ✓ | 2005.13981 §2.4 和 2009.06122 §4.1 一致。700 = 102 + 276 + 272 + 50，「声调语言」112 条是「非英语」272 条的子集 |
| 2-6 | DNS 2023 第 2 赛道包含笔记本内置麦 | **保留** ✓ | 官方页原文：「non-headset (speakerphone, built-in mic in laptop/desktop/mobile phone/other meeting devices etc.)」，每个集 600 条 |
| 2-7 | DNS 2023 噪声名单分成「办公 / 家庭 / 家电 / 交通和户外 / 公共场所」 | **降级：后两组是报告自加** | 原文 §3.3 只写「Office scenarios (…)」「Home scenarios (…)」「Appliances (…)」，后面是一串没分组的名词：Fire alarm、Car、…、Café、…、Airport Announcements。另外原文设备一处是占位符「[list of devices]」，论文没给设备清单 |
| 2-8 | ConferencingSpeech「12 间会议室」「评测集取自另外 9 间房」 | **降级（措辞误导）** | 原文是「other 9 rooms' recordings of ConferencingSpeech 2021 Challenge Database which were unseen in development」。数据库总共 12 间，评测用的应是其中 9 间，不是另加 9 间。原文还写半实录在「12 quiet meeting rooms」里录，即房间本身是安静的，报告漏了。其余数字一致 ✓（98 条、2500 间仿真房、0.5–5.0 m、135 / 165 条） |
| 2-9 | ConferencingSpeech 2022 标「论坛/博客式检索摘要」 | **改来源类型** | 它是 arXiv 论文 2203.16032，应标「论文，原文未核」 |
| 2-10 | AliMeeting「录音时人为制造了点击、键盘…噪声」 | **删除，改写** | 原文是「various kinds of indoor noise … are made naturally」，意思是自然产生。13 间房、8–55 m²、0.3–5.0 m、212 / 8 场、456 / 25 人都一致 ✓。评测集交叠率原文自相矛盾：正文 34.76%，表 1 是 34.20%。另外原文说「detailed parameters of each meeting venue will be released together with the Train data」，所以「RT60 未查到」应补一句：数据包里可能有，待查 |
| 2-11 | NOTSOFAR-1 房间数 30 / 38 未核清 | **保留，补信息** | 摘要原文「30 conference rooms」；表 1 是 20 / 5 / 13 间。正文还说评测集「entirely disjoint … no overlap in speakers or rooms」，所以训练和开发两部分大概率共用房间，38 是简单相加，不能当房间数【推断】。「另一 Interspeech 版本写 20 + 10」：我没找到，降级为「来源未知」。设备、15,000 条声学传递函数一致 ✓ |
| 2-12 | 「6 个地点在所有公开挑战里都没有覆盖」 | **降级，收窄** | DCASE 2016 Task 1（TUT Acoustic Scenes 2016，15 类）有 library、train、tram、cafe/restaurant、office、home 等类。它是声景分类挑战，不是语音集，但有真实录音。应改成「语音增强 / 识别类挑战中没有」 |
| 2-13 | URGENT 2025 用了 TUT 2018 和 DEMAND，条数未查到 | **保留**（我未核） | 报告已自标「推断」 |
| 2-14 | AEC 2021 / 2023 的条数和 RT60 图估读 | **二手，原文未核**（我本轮没打开） | 报告标的来源类型合理 |

## 四、第 3 路（产品）逐条判定

| # | 报告原结论 | 判定 | 我看到的原文 / 理由 |
|---|---|---|---|
| 3-1 | Krisp 字错率表（31.84→6.30 等）、265 条、11 套识别配置 | **保留** ✓ | 原文是「eleven configurations across nine engines」。各组条数 96 + 106 + 63 = 265，与总数吻合（来自数据集博客的统计块） |
| 3-2 | Krisp 数据集博客的「Location Types」一节 | **改位置描述** | 页面没有叫这个名字的小节，只有统计块「4 room types / 4 locations / 4 location types」。Work 组原文是「small rooms, medium rooms, large conference rooms and open-plan floors」✓ |
| 3-3 | Krisp「cabin rumble」→ rail-air-cabin | **降级** | 这个词属于手机通话组（「Cars, streets, shops, metro」），更可能指车厢或地铁，不能证明覆盖高铁或飞机客舱 |
| 3-4 | Zoom 个性化音频隔离标「二手，原文未核」 | **升级为【官方文档】** | KB0059985 原文：「Personalized audio isolation is recommended for crowded environments」「uses your voiceprint」。High 档原文是「eliminate noise such as background speech from others」✓ |
| 3-5 | Meet「音乐会被当成噪声」 | **降级为推断** | 原文是「If you need other sounds in your meeting, like music, turn off noise cancellation」，只是建议需要音乐时关掉降噪。「Voices from TV or people talking won't be canceled」✓ |
| 3-6 | Teams 2020 AI 降噪博客的训练数据（760 h / 180 h / 150 类 / 3000+ 真实房间 / 11.5 万合成房间） | **保留** ✓ | 原文一致。「your child's conflicting school lesson」「厨房声」两处我没核到 |
| 3-7 | Teams 语音隔离博客的场景、2000+ h、25 种语言、没有评测数字 | **保留** ✓ | 原文一致 |
| 3-8 | Webex 四种模式 | **保留** ✓ | 原文一致 |
| 3-9 | DNS 2021「二手，经搜索摘要」 | **升级为【论文】** | 第 2 路已核过原文（2009.06122 §4.1.3） |
| 3-10 | NVIDIA、Apple、Dolby 的说法 | 我本轮没核，维持原标 | — |

---

## 五、漏掉的重要来源或事实（10 条）
1. **3GPP 没有桌面免提的环境噪声测试。** 【实测读原文】26.132 只有 §7.12 / §8.12 / §9.12 的手持、手持免提和电气接口。TS 103 224 只有表 8A（会议、家庭、办公）给了桌面免提的声级（§5.3.3）。这决定了 MacBook 场景能直接借用的标准录音只有表 8A 这几段。
2. **超宽带 / 全带门槛。** 【实测】26.131 §7.10：手持 S-MOS ≥ 3.7、N-MOS ≥ 2.7。26.132 §9.12 用表 2i，评价方法是 TS 103 281。
3. **ETSI TS 103 106 和 TS 103 281（3QUEST 打分法）。** 它们是 N-MOS、S-MOS 的计算方法，第 1 路没列。来源：26.132 参考文献 [34]、[50]，【官方文档】。
4. **P.1100 / P.1110 的 2017 年及以后修订。** 工况和门槛都改过，见 DAGA 2017 表 5、表 6，<https://pub.dega-akustik.de/DAGA_2017/data/articles/000149.pdf>。现行版本还要核。
5. **DEMAND 噪声库。** 共 6 大类（Domestic / Office / Public / Transportation / Street / Nature），每类 3 个环境，16 声道阵列，48 kHz 和 16 kHz 两版。【官方文档】<https://zenodo.org/records/1227121>。环境代号（OMEETING、OOFFICE、PCAFETER、PRESTO、PSTATION、TBUS、TMETRO、TCAR、DLIVING、DKITCHEN）是我凭记忆写的，二手未核。它可直接补 canteen、cafe、transit-hub、meeting-room 这几格。
6. **TUT Acoustic Scenes 2016（DCASE 2016）。** 15 类，含 library、office、home、cafe/restaurant、train、tram、metro station，双耳录音，约 30 s 一段。【二手检索摘要】<https://dcase-repo.github.io/dcase_datalist/datasets/scenes/tut_asc_2016_dev.html>。这是 library-classroom 唯一找到的公开真实录音。
7. **CHiME-3/4 实录噪声。** 有公交、咖啡馆、步行街、街口 4 种环境，用平板上的 6 麦阵列录（设备形态接近笔记本）。我凭记忆写的，二手未核。第 2 路只提了一句「CHiME-4 有咖啡馆」。
8. **WHAM! 噪声。** 在湾区的咖啡馆、餐厅、酒吧、办公楼、公园里实录的人声嘈杂声。我凭记忆写的，二手未核。可补 cafe、canteen。
9. **ANSI/ASA S12.60 教室声学标准。** 规定空教室背景噪声 ≤ 35 dB(A)，混响时间约 0.6 s（中小教室）。我凭记忆写的，二手未核。它能给 library-classroom 一个底噪基准。
10. **AliMeeting 数据包里的会场参数。** 原文说会场参数「will be released together with the Train data」，可能有 RT60，值得下载清单（不下载音频）核一下。

另外提醒：仓库里的目录 `docs/research/021_meeting-scenes/catalog.md` 第 3 行已经是「15 个地点键 + 1 个兜底键」（加了 `bus-metro-cabin`），本线任务写的是 14 个键，两边口径不一致。

---
来源（本轮打开过的原文）：
- [ETSI ES 202 396-1 V1.10.2](https://www.etsi.org/deliver/etsi_es/202300_202399/20239601/01.10.02_60/es_20239601v011002p.pdf) · [ETSI TS 103 224 V1.7.2](https://www.etsi.org/deliver/etsi_ts/103200_103299/103224/01.07.02_60/ts_103224v010702p.pdf) · [TS 126 132 V18.2.0](https://www.etsi.org/deliver/etsi_TS/126100_126199/126132/18.02.00_60/ts_126132v180200p.pdf) · [TS 126 131 V18.1.0](https://www.etsi.org/deliver/etsi_TS/126100_126199/126131/18.01.00_60/ts_126131v180100p.pdf) · [P.835 Amd.1](https://www.itu.int/rec/dologin_pub.asp?lang=e&id=T-REC-P.835-200710-S!Amd1!PDF-E&type=items) · [DAGA 2017](https://pub.dega-akustik.de/DAGA_2017/data/articles/000149.pdf)
- [HEAD Teams 数据表](https://cdn.head-acoustics.com/fileadmin/data/global/Datasheets/Analysis_Software/ACQUA_Standards/HEAD-acoustics-Data-Sheet-ACQUA-Standards-MS-Teams-Measurements-and-Analyses-According-to-Microsoft-Teams-Audio-Test-Specification-60026-EN.pdf) · [HEAD Zoom 数据表](https://cdn.head-acoustics.com/fileadmin/data/global/Datasheets/Analysis_Software/ACQUA_Standards/ACQUA-Standards-Zoom-Audio-60068-Data-Sheet.pdf) · [Teams 认证页](https://learn.microsoft.com/en-us/microsoftteams/devices/usb-devices)
- [DNS 2020](https://arxiv.org/pdf/2005.13981) · [DNS ICASSP 2021](https://arxiv.org/pdf/2009.06122) · [DNS Interspeech 2021](https://arxiv.org/pdf/2101.01902) · [DNS 2022](https://arxiv.org/pdf/2202.13288) · [DNS 2023](https://arxiv.org/pdf/2303.11510) · [DNS 2023 官方页](https://www.microsoft.com/en-us/research/academic-program/deep-noise-suppression-challenge-icassp-2023) · [ConferencingSpeech 2021](https://arxiv.org/pdf/2104.00960) · [M2MeT / AliMeeting](https://arxiv.org/pdf/2110.07393) · [NOTSOFAR-1](https://arxiv.org/html/2401.08887)
- [Krisp 评测](https://krisp.ai/blog/voice-isolation-benchmark/) · [Krisp 数据集](https://krisp.ai/blog/speech-recognition-dataset-background-voices/) · [Zoom KB0059985](https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0059985) · [Google Meet](https://support.google.com/meet/answer/9919960) · [Teams 语音隔离博客](https://techcommunity.microsoft.com/blog/microsoftteamsblog/voice-isolation-in-microsoft-teams-enables-personalized-noise-suppression-for-ca/4096077) · [Teams 2020 降噪博客](https://techcommunity.microsoft.com/t5/microsoft-teams-blog/reduce-background-noise-in-microsoft-teams-meetings-with-ai/ba-p/1992318) · [Webex](https://help.webex.com/en-us/article/3jqfrs) · [DEMAND](https://zenodo.org/records/1227121) · [TUT ASC 2016](https://dcase-repo.github.io/dcase_datalist/datasets/scenes/tut_asc_2016_dev.html)