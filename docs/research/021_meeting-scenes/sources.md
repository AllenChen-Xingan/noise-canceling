# 021 · 出处表

> 用途：`catalog.md`、`coverage.md`、`decision.md` 里的每个 `[Sxx]` 都指这里的一行；`check_catalog.py` 会核对编号都存在。
> 「核对」列：**✓** = 怀疑派本轮打开原文逐项核过；**摘要** = 只读到摘要或检索摘录；**二手** = 转述，原文未核；**未核** = 调研员读过、怀疑派没复核。
> 原始调研报告与两位怀疑派的逐条判定在 `raw/`（`r1`–`r5`、`skeptic-A`、`skeptic-B`），本表已按怀疑派的判定改正。

## 标准与认证

| 编号 | 名称 | 链接 | 来源类型 | 核对 | 用在哪 / 注意 |
|---|---|---|---|---|---|
| S01 | ETSI ES 202 396-1 V1.10.2（背景噪声模拟与数据库） | https://www.etsi.org/deliver/etsi_es/202300_202399/20239601/01.10.02_60/es_20239601v011002p.pdf | 官方文档 | ✓ | 表 1–18 共 72 段噪声及 dB SPL(A) 声级。大多是人工头双耳录音，例外：表 16–18 用后视镜处 2 支心形麦，表 15 小飞机是真人戴耳机。`Cafeteria_Noise` 原文是「超市收银台」，`Mensa` 才是食堂。表 9 呼叫中心与表 10 客厅声级完全相同（56.6/57.8），疑为原文抄错。主表用到的值（L/R）：表 8 Mensa 63.4/61.9；表 4 Inside_Bus 75.8/77.6；表 9 路边施工风钻 76.2/78.4；客舱录音见表 6 |
| S02 | ETSI TS 103 224 V1.7.2（背景噪声场模拟） | https://www.etsi.org/deliver/etsi_ts/103200_103299/103224/01.07.02_60/ts_103224v010702p.pdf | 官方文档 | ✓ | 8 麦阵列，声级是录音位置每声道 dB SPL(A)，下面写 8 声道的范围。**只有表 8A（会议 · 家庭 · 办公）有桌面免提布置的声级**，最接近 MacBook 放桌上。主表用到的值——表 8（手持 / 手持免提）：车内副驾 MidSize 80/100/130 km/h 66.4–68.4 · 65.1–67.5 · 70.1–72.3，FullSize 60.3–64.0 · 63.0–67.1 · 68.3–71.9；Inside_Train 67.6–69.6；Inside_Bus 70.5–73.4；Roadnoise 69.9–73.8；Cafeteria 69.0–71.0；Airport 走廊 77.2–78.8；Callcenter1 69.6–72.6、Callcenter2 59.3–60.8。表 8A（桌面免提）：Conference1–3 48.2–55.0；Kitchen 65.8–67.3；Livingroom 63.1–64.4。表 9：StationBuilding 65.1–66.2；Roadnoise 74.5–75.7。表 10：Fridge 38.5–44.8；Talker 54.1–61.0。附录 A 表 A.1 两档混响 T60 0.3 s / 0.45–0.5 s |
| S03 | 3GPP TS 26.132 V18.2.0（终端声学测试方法） | https://www.etsi.org/deliver/etsi_TS/126100_126199/126132/18.02.00_60/ts_126132v180200p.pdf | 官方文档 | ✓ | §7.12 表 2d：手持 8 段噪声（酒吧 · 马路 · 十字路口 · 站台 · 车内 130 km/h · 收银台 · 食堂 · 呼叫中心）；表 2d2 手持免提 5 段。**没有桌面免提的环境噪声测试** |
| S04 | 3GPP TS 26.131 V18.1.0（终端声学性能要求） | https://www.etsi.org/deliver/etsi_TS/126100_126199/126131/18.01.00_60/ts_126131v180100p.pdf | 官方文档 | ✓ | 门槛是所有噪声条件的平均分：窄带 / 宽带手持 S-MOS ≥ 3.0、N-MOS ≥ 2.3；超宽带 / 全带（§7.10）手持 S-MOS ≥ 3.7、N-MOS ≥ 2.7 |
| S05 | ITU-T P.835 修订 1（2007-10）附录 III | https://www.itu.int/rec/dologin_pub.asp?lang=e&id=T-REC-P.835-200710-S!Amd1!PDF-E&type=items | 官方文档 | ✓ | 噪声类别：粉噪 · babble（≥ 4 人）· 街道 · 车内 · 单人说话 · 含鼓音乐；SNR 12 / 6 / 0 dB。无声级、无地点 |
| S06 | Müsch 等，DAGA 2017（转述 ITU-T P.1100 / P.1110 车内工况） | https://pub.dega-akustik.de/DAGA_2017/data/articles/000149.pdf | 论文 | 二手 | 车内 4 种工况与门槛，引的是 2015 版，2017 版已改；现行版未核 |
| S07 | Microsoft Teams 设备认证页「Special usage designations」 | https://learn.microsoft.com/en-us/microsoftteams/devices/usb-devices | 官方文档 | ✓ | 「Premium Microphone for Open Office」：不把邻座声音传出去。噪声清单与声级未公开 |
| S08 | HEAD acoustics 数据表：Microsoft Teams Audio Test Specification V5.0 测量 | https://cdn.head-acoustics.com/fileadmin/data/global/Datasheets/Analysis_Software/ACQUA_Standards/HEAD-acoustics-Data-Sheet-ACQUA-Standards-MS-Teams-Measurements-and-Analyses-According-to-Microsoft-Teams-Audio-Test-Specification-60026-EN.pdf | 厂商资料（转述未公开规范） | ✓ | 开放办公耳机模块有「旁人说话衰减」测项；规范正文未公开 |
| S09 | HEAD acoustics 数据表：Zoom Audio 规范测量 | https://cdn.head-acoustics.com/fileadmin/data/global/Datasheets/Analysis_Software/ACQUA_Standards/ACQUA-Standards-Zoom-Audio-60068-Data-Sheet.pdf | 厂商资料（转述未公开规范） | ✓ | 测试房间「有混响但安静的类办公室」；噪声只写「各环境典型背景噪声」 |

## 挑战赛与公开语料

| 编号 | 名称 | 链接 | 来源类型 | 核对 | 用在哪 / 注意 |
|---|---|---|---|---|---|
| S10 | DNS Challenge Interspeech 2020 | https://arxiv.org/pdf/2005.13981 | 论文 | ✓ | 开发集实录 300 条含「开放办公区和会议室」，未拆比例；合成集 RT60 300–1300 ms |
| S11 | DNS Challenge ICASSP 2021 | https://arxiv.org/pdf/2009.06122 | 论文 | ✓ | 盲测集 700 条按语言 / 情绪分组，不按地点 |
| S12 | DNS Challenge Interspeech 2021 | https://arxiv.org/pdf/2101.01902 | 论文 | 未核 | 不戴耳机时说话人离麦 1–5 m；分组条数未给 |
| S13 | DNS Challenge ICASSP 2022 | https://arxiv.org/pdf/2202.13288 | 论文 | ✓ | 盲测集 859 条 × 10 s，70% 手机录；图 1 给 19 类噪声占比（柱上印数值）：打字 10.2 · 鼠标 9.5 · 水流 9.3 · 风扇 8.8 · 关门 7.4 · 撕包装 7.4 · 餐具 5.7 · 婴儿哭 5.5 · 椅子 5.4 · 无噪声 5.3 · 狗叫 4.4 · 街道 4.2 · 粗重呼吸 3.7 · 碰撞声 3.5 · 空调 3.0 · 洗碗机 2.1 · 咀嚼 2.1 · 复印机 1.5 · 车内 0.9（%）。「噪声 / 邻座 / 两者」各几条未给 |
| S14 | DNS Challenge ICASSP 2023（论文 + 官方页） | https://arxiv.org/pdf/2303.11510 ；https://www.microsoft.com/en-us/research/academic-program/deep-noise-suppression-challenge-icassp-2023 | 论文 / 官方文档 | ✓ | 第 2 赛道明写含笔记本内置麦，开发 / 盲测各 600 条；盲测分组（副语言 · 情绪 · 强混响 · 强混响 + 噪声 · 噪声 + 邻座）各组条数未给；噪声名单原文只分「办公 · 家庭 · 家电」三组，其余未分组（含 Café、机场广播） |
| S15 | ConferencingSpeech 2021 | https://arxiv.org/pdf/2104.00960 | 论文 | ✓ | 共 12 间安静会议室，评测用其中 9 间；专用阵列录音；T60 与尺寸未给 |
| S16 | URGENT 2024 / 2025 / 2026 | https://arxiv.org/html/2406.04660v1 ；https://arxiv.org/html/2601.13531v1 | 论文 | 未核 | 按失真类型分组，不按地点 |
| S17 | NOTSOFAR-1（CHiME-8） | https://arxiv.org/html/2401.08887 | 论文 | ✓ | 摘要写约 30 间会议室，315 场 × 约 6 min，每场 4–8 人；专用会议设备；T60 未给，只给扫频录音可自算 |
| S18 | CHiME-7/8 DASR（CHiME-6 · DiPCo · Mixer 6） | https://arxiv.org/html/2407.16447 | 论文 | 未核 | 住户家里聚餐（厨房 / 客厅）；Kinect / 圆阵 |
| S19 | AEC Challenge ICASSP 2021 / 2023 | https://arxiv.org/pdf/2009.04972v3 ；https://arxiv.org/html/2309.12553v1 | 论文 | 未核 | 只有 Windows 设备；按干净 / 有噪、单讲 / 双讲分组；真实训练集 RT60 峰值约 0.2–0.25 s（图中估读） |
| S20 | AliMeeting（M2MeT） | https://arxiv.org/pdf/2110.07393 | 论文 | ✓ | 13 间会议室 8–55 m²，8 麦阵列离人 0.3–5.0 m；噪声是「自然产生」；会场参数说随训练数据发布，可能含 RT60，待查 |
| S21 | AMI · ICSI 会议语料 | https://www1.icsi.berkeley.edu/ftp/pub/speech/papers/icassp03-janin.pdf | 论文 | 二手 | AMI 3 间、ICSI 1 间（约 4.0 × 7.6 m） |
| S22 | DCASE 2016 TUT Acoustic Scenes（声景分类，非语音集） | https://dcase-repo.github.io/dcase_datalist/datasets/scenes/tut_asc_2016_dev.html | 官方文档 | 二手 | 15 类含 library · office · home · cafe/restaurant · train · tram · metro station；双耳实录 |
| S23 | DEMAND 多通道噪声库 | https://zenodo.org/records/1227121 | 官方文档 | 未核 | 6 大类 × 3 环境，16 声道；环境代号以 023 数据卡为准 |

## 商业产品

| 编号 | 名称 | 链接 | 来源类型 | 核对 | 用在哪 / 注意 |
|---|---|---|---|---|---|
| S24 | Krisp 语音隔离评测与数据集 | https://krisp.ai/blog/voice-isolation-benchmark/ ；https://krisp.ai/blog/speech-recognition-dataset-background-voices/ | 官方文档（厂商自测） | ✓ | 本次检索找到的唯一按使用场景分组的公开测试集：工作 96 条 / 呼叫中心 106 条 / 手机通话 63 条，共 265 条真实录音；「cabin rumble」属手机通话组（车、街、店、地铁），不是客舱 |
| S25 | NVIDIA Maxine 降噪支持的噪声类型（30 类） | https://docs.nvidia.com/maxine/afx/2.0.0/Appendix/DenoiserSupportedNoises.html | 官方文档 | 未核 | 按噪声分，不按地点 |
| S26 | Microsoft Teams 降噪与语音隔离（帮助页 + 2020 / 2024 博客） | https://support.microsoft.com/en-us/office/reduce-background-noise-in-microsoft-teams-meetings-1a9c6819-137d-4b3b-a1c8-4ab20b234c0d ；https://techcommunity.microsoft.com/blog/microsoftteamsblog/voice-isolation-in-microsoft-teams-enables-personalized-noise-suppression-for-ca/4096077 | 官方文档 | ✓ | 语音隔离点名「开放办公 · 咖啡馆 · 候机」；与同事共用设备时提示关掉 |
| S27 | Zoom 降噪设置 KB0059985 | https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0059985 | 官方文档 | ✓ | High 档去「旁人背景说话」；个性化音频隔离「适合拥挤环境」 |
| S28 | Google Meet 降噪帮助页 | https://support.google.com/meet/answer/9919960 | 官方文档 | ✓ | **原文：电视人声和旁人说话不会被去掉** |
| S29 | Apple macOS 麦克风模式 | https://support.apple.com/guide/mac-help/use-mic-modes-mchle82b42f0/mac | 官方文档 | 未核 | 语音突显 / 宽谱；不点名任何地点 |
| S30 | Cisco Webex 降噪模式 | https://help.webex.com/en-us/article/3jqfrs | 官方文档 | ✓ | 「只优化我的声音」对应开放办公；「优化所有人声」对应会议室 |

## 声学实测

| 编号 | 名称 | 链接 | 来源类型 | 核对 | 用在哪 / 注意 |
|---|---|---|---|---|---|
| S31 | Pearsons 等 1977，EPA-600/1-77-025（各场景说话与噪声声级） | https://www.nonoise.org/epa/Roll2/roll2doc16.pdf | 论文 | ✓ | 全文 A 计权 Leq。安静时对话 55 dB @1 m；背景 48–70 dB 时说话每 dB 升约 0.6 dB；电视在观看者耳边平均 61 dB、离电视平均 3 m（观看者自选音量）；捷运与客机里环境 77 / 说话 75 dB、相距 0.4 m；百货商店背景 54 / 说话 61 dB；印刷页 23–25：在家里说话保持 9–14 dB 的语音噪声比，在家门外只保持 5–8 dB |
| S32 | Wu 等 2018，Ear Hear 39(2) | https://pmc.ncbi.nlm.nih.gov/articles/PMC5824438 | 论文 | ✓ | 拐点 59.3 dBA、斜率 0.34 / 0.54；在家噪声中位 53.6 dBA、SNR 9.9 dB；交通 SNR 5.6 dB。**被试是 20 名 65–80 岁老人** |
| S33 | Brascamp 2021，J Vision 21(8):19 | https://pmc.ncbi.nlm.nih.gov/articles/PMC8383908/ | 论文 | ✓ | 自然坐姿眼到笔记本屏幕平均 51.7 cm（自测卷尺）；不等于嘴到麦 |
| S34 | Hongisto & Keränen，Euronoise 2021（ISO 3382-3 开放办公量） | https://www.sea-acustica.es/archivo/congresos/Madeira21/ID107.pdf | 论文 | ✓ | 文献极值：4 m 处语音 40–60 dB、背景 26–54 dB、干扰距离 2.1–21 m；图尔库一间：4 m 处语音 52.4 dB、背景 37.6 dB。标准声源、空场测量 |
| S35 | Leonard & Chilton 综述表（开放办公背景声级） | — | 论文 | 二手 | 20 间 41–53 dBA、均 47；呼叫中心 21 处均 58 dBA；统计量未知 |
| S36 | Hongisto 等，11 款电话亭 ISO 23351-1 实测 | https://www.turkuamk.fi/julkaisu/acoustic-performance-of-eleven-commercial-phone-booths-according-to-iso-23351-1/ | 论文 | 摘要 | 语音声级降低量约 27–30 dB |
| S37 | ASHRAE 手册 第 49 章（暖通噪声目标） | https://handbook.ashrae.org/Handbooks/A23/IP/A23_Ch49/a23_ch49_ip.aspx | 官方文档 | 摘要 | 会议室 NC/RC 25–30 |
| S38 | Lebo 等 1994，West J Med 161:45–49（湾区 27 家餐厅） | https://pmc.ncbi.nlm.nih.gov/articles/PMC1011367/ | 论文 | 二手 | 59–80 dBA 只见于转述 |
| S39 | Int J Audiol 2016;55(3):184–8（咖啡师剂量计） | https://informahealthcare.com/doi/abs/10.3109/14992027.2015.1124295 | 论文 | 摘要 | 6 家咖啡馆 Leq 71–83 dBA；在咖啡师身上测，不是座位 |
| S40 | Wang 等，天津两所高校食堂人群噪声 | https://discovery.ucl.ac.uk/10132069/1/Crowd%20Noise%20and%20Vocal%20power%20level%20in%20Large%20College%20Canteens%20in%20China.pdf | 论文 | ✓ | 用餐时 61–73 dBA；T60 1.04–1.08 s（500–2k Hz 均值，人少时测） |
| S41 | To & Chung 2014，Noise & Health 16(73)（香港餐厅） | https://research.mpu.edu.mo/en/publications/noise-in-restaurants-levels-and-mathematical-model/ | 论文 | 摘要 | Leq,1h 66.7–82.6 dBA |
| S42 | To & Chung 2015，Noise & Vibration Worldwide 46(8):11–17 | https://research.mpu.edu.mo/en/publications/restaurant-noise-levels-and-temporal-characteristics/ | 论文 | 摘要 | 均值 73.9 dBA（SD 3.81）——016 F3 的「香港 73.9」出处在此 |
| S43 | Odeon，BNAM 2012（丹麦大学食堂） | https://odeon.dk/pdf/BNAM%202012%20-%20Noise%20in%20eating%20establishments.pdf | 论文 | 二手 | 晚餐 LAeq 82–87 dB(A)，T60 0.8–2.5 s |
| S44 | Wu, Kang 等 2020，Appl Acoust 160（哈尔滨火车站） | https://discovery.ucl.ac.uk/10090746/1/Wu%2C%20Kang%20Acoustic_comfort_in_large_railway_station_final.pdf | 论文 | ✓ | 候车座位区 60.8–66.4 dBA、T60 8.6–10.0 s；测时 3 m 内不许有人说话（低估身边人声）；只有 1 座车站 |
| S45 | 美国机场合作研究计划报告 175 | https://www.nap.edu/read/24839/chapter/24 | 官方文档 | 未核 | 广播区建议 T60 ≤ 1.5 s（是建议值，不是实测） |
| S46 | Kendra 等 2015（满载轿车车内噪声） | https://zenodo.org/records/1109657/files/10002764.pdf | 论文 | 未核 | 50 km/h 65–67 dB；**未写计权**，不能与 dB(A) 混用 |
| S47 | Ozcan & Nemlioglu 2006，Canadian Acoustics 34(4)（A321 客舱） | https://jcaa.caa-aca.ca/index.php/jcaa/article/view/1854 | 论文 | ✓ | 巡航均值 78.1–79.4 dB(A)（2 类声级计读数算术平均，非 Leq）；广播突发 82–88；停机时乘客交谈 62–68 dB(A) |
| S48 | 汉阳大学 Euronoise 2015（高铁车厢） | — | 论文 | 二手 | 100 / 200 / 300 km/h 时 60 / 63 / 66 dBA |
| S49 | 德黑兰市中心街道噪声 2006 | https://ijehse.tums.ac.ir/index.php/jehse/article/view/99 | 论文 | 摘要 | 人行道 73.3、路边 78.5 dBA |
| S50 | Construction Canada：酒店客房声学 | https://www.constructioncanada.net/the-challenge-of-guest-room-acoustics-in-hotels/print | 论坛 / 博客 | 未核 | 客房环境声约 28–32 dBA（行业杂志说法） |
| S51 | Shield & Dockrell 2004（英国教室） | — | 论文 | 二手 | 空教室均 47 dBA；上课 56–77 dB |
| S52 | ANSI/ASA S12.60（教室声学标准） | — | 官方文档 | 二手 | 空教室 ≤ 35 dBA、T60 ≤ 0.6 s（目标值） |
| S53 | 葡萄牙大学阅览室（ICSV17） | https://web.fe.up.pt/%7Ecarvalho/icsv17.pdf | 论文 | 摘要 | 中频 T60 1.3 s；空场 32 dB(A) |
| S54 | 狗叫声级（Central Coast 噪声咨询报告） | https://cdn.centralcoast.nsw.gov.au/sites/default/files/2023-07/addendum_to_noise_report_-_renl417-03f02_technical_memo_r0_1.pdf | 实测（咨询报告） | 未核 | 1 m 处单声 LA5 75 dB(A)；另一份 LAeq 79 dB(A)（统计量不同，不可合成一个范围） |
| S55 | 美国宠物用品协会 2024 | — | 官方文档（行业协会） | 未核 | 44% 家庭养狗、30% 养猫 |
| S56 | 美国时间使用调查 2024（BLS ATUS） | https://www.bls.gov/news.release/atus.nr0.htm | 官方文档 | 未核 | 平均每天看电视 2.6 h |

## 用户在哪开会

| 编号 | 名称 | 链接 | 来源类型 | 核对 | 用在哪 / 注意 |
|---|---|---|---|---|---|
| S57 | Caros 等 2023（SWAA 月度调查），arXiv 2303.06186 | https://arxiv.org/pdf/2303.06186 | 论文 | ✓ | 美国 N=27,364（2021-11 至 2022-06），按**工作工时**：雇主场所 44.4% · 家 29.1% · 客户处 10.6% · 亲友家 5.3% · 共享办公 5.3% · 公共场所 5.3%（含咖啡馆、图书馆、社区中心）。数值只在图 1(a) 里。**是工作地点，不是开会地点** |
| S58 | Buffer State of Remote Work 2023 | https://buffer.com/state-of-remote-work/2023 | 官方文档（厂商调查） | ✓ | 全球 N=3,000 远程工作者：主要地点家 82% · 办公室 9% · 共享办公 5% · 咖啡馆 2%；在家：独立办公空间 37% · 卧室 21% · 客厅 20% · 换来换去 14% |
| S59 | Puglisi 等 2021，Frontiers in Built Environment | https://www.frontiersin.org/journals/built-environment/articles/10.3389/fbuil.2021.688484/full | 论文 | ✓ | 意大利 N=1,934（2020-05）：独立房间 54.6% · 共用房间 43.1%；25% 认为成年人发出的声音是主噪声源；受访者偏大学与研究机构 |
| S60 | FinanceBuzz 远程工作调查 | https://financebuzz.com/survey-remote-work-lifestyle | 媒体自办调查 | ✓ | N=500（2020-03 至 04）：视频会议被打断——宠物 47% · 快递员 39% · 伴侣 32% · 孩子 21%；「过去一年在哪工作过」口径会放大偶发地点，样本疑似偏 |
| S61 | Dimensional Research for Cisco Webex 2020 | https://www.cisco.com/c/dam/en/us/products/collateral/collaboration-endpoints/global-workforce-survey.pdf | 官方文档（厂商委托） | ✓ | 全球 N=1,569：在家开会最常用收音设备——笔记本 42%（麦与扬声器合计）；自己这边有背景噪声 32% |
| S62 | Plantronics《Managing Noise》2017 | https://m.softchoice.com/web/newsite/documents/amplify/Plantronics-Hardware-Managing-Noise.pdf | 官方文档（厂商） | ✓ | 7 国 N=2,184，均在 500 人以上开放办公室。第 12 页问「What are the most annoying noises you hear?」：通话时大声说话 29% · 工位前交谈 23% · 手机提示音 13% · 私人电话 11% · 吸鼻子或咳嗽 9% · 敲键盘 8% · 吃东西 5%；第 7 页：29% 嫌吵时离开工位 |
| S63 | Owl Labs State of Hybrid Work 2025 | https://owllabs.com/state-of-hybrid-work/2025 | 官方文档（厂商） | 未核 | 美国 N=2,000：67% 放弃会议室设备改用自己笔记本 |
| S64 | IWG 2019（经 FE News 转述） | https://www.fenews.co.uk/skills/school-holidays-signal-stress-for-home-workers/ | 官方文档（厂商） | 二手 | 英国：49% 工作电话被孩子、家人或宠物打断 |
