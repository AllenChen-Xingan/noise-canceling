# 原始报告 · r1-standards（workflow wf_c790c606-7d6 · 2026-11-03，未经合成，供追溯）

## 要点（5 行）
1. 【官方文档】ETSI ES 202 396-1 V1.10.2 和 TS 103 224 V1.7.2（均为 2026-07 版）列出约 45 段噪声，每段都有声级，单位是 dB SPL(A)。都用 P.58 人工头（HATS）在真实地点录制，ES 版是双耳 2 声道，TS 版是 8 麦阵列。录制城市和具体场所，两份标准都没写，未查到。
2. 【官方文档】3GPP TS 26.132 V18.2.0 §7.12/§8.12 只取这批噪声的一个子集。手持（handset）用 8 段：酒吧、马路、十字路口、站台、轿车 130 km/h、超市收银台、学生食堂、呼叫中心。手持免提用 5 段。TS 26.131 §5.11/§6.10 只对所有条件的平均分设门槛，例如手持时 N-MOS ≥ 2.3。
3. 【推断】14 个场景键里，有对应噪声的是 car-passenger、street-outdoor、transit-hub、rail-air-cabin、meeting-room、open-office、home-family、cafe、canteen。phone-booth、cowork、hotel-room、library-classroom 没有任何标准录音对应，home-quiet 只有冰箱噪声和 P.340 房间噪声谱可以凑合。
4. 【官方文档】ITU-T 只有两处给出噪声类别：P.835（2007 修订 1）附录 III.5 要求粉红噪声、babble（≥ 4 人）、街道、车内、单人说话、含鼓音乐，信噪比 12、6、0 dB 加干净语音；P.1100/P.1110 是车内驾驶工况。P.808、P.804 未查到场景列表。
5. 【官方文档 / 厂商资料】微软 Teams 音频测试规范（V5.0 + Addendum A）和 Zoom 音频规范（耳机 v2.3 等）都没有公开。公开信息只到「开放办公加旁人说话（distractor）的衰减测试」「按会议室大小和噪声环境测试」「混响但安静的类办公室」这一层，噪声清单和声级都未查到。

---

## 1. ETSI ES 202 396-1 V1.10.2（2026-07）背景噪声数据库

来源：【官方文档】<https://www.etsi.org/deliver/etsi_es/202300_202399/20239601/01.10.02_60/es_20239601v011002p.pdf>，见 §8.0–8.3、表 1–18。

**录制方式**
- 用人工头（HATS）双耳录音，人工头须符合 ITU-T P.58（§5.1）。
- 首选 ID 均衡，其次 DF 均衡（§5.2）。
- 文件是 2 声道 WAV，48 kHz，多数截到 30 s（§8.0）。
- 声级是原始信号的 dB SPL(A)，L 和 R 分开给。§8.0 写 dBSPL(A)，表头只写 [dBSPL]。
- 回放房间的要求：RT 在 0.2–0.7 s 之间，底噪 < 30 dB SPL(A)（§6.1）。
- 录制城市和具体场所：未查到，原文只有「在酒吧录」「在人行道录」这类描述。

**§8.1 双耳录音（表 1–14）→ 场景键**

| 表 | 文件名（描述） | 时长 s | 声级 L/R dB SPL(A) | 场景键（【推断】） |
|---|---|---|---|---|
| 1 | Fullsize_Car1_80/100/130Kmh（驾驶位） | 30 | 63.4/61.8 · 65.4/63.2 · 69.1/68.1 | car-passenger（近似，录的是驾驶位） |
| 2 | Midsize_Car1_80/100/130Kmh（驾驶位） | 30 | 62.3/61.4 · 63.4/62.6 · 67.0/65.9 | car-passenger（近似） |
| 3 | Midsize_Car2_80/100/130Kmh（驾驶位） | 30 | 68.3/67.8 · 70.6/70.4 · 74.4/74.5 | car-passenger（近似） |
| 4 | Inside_Bus_Noise（客舱） | 30 | 75.8/77.6 | 14 键无对应；v1 目录有 `bus-metro-cabin` |
| 5 | Inside_Train_Noise1/2/3（客舱） | 30 | 73.0/72.9 · 62.1/64.8 · 79.17/80.52 | rail-air-cabin（未写明是否高铁） |
| 6 | Inside_Aircraft_Speech / Aircraft1 / Aircraft2（客舱） | 30 | 80.4/79.9 · 74.6/75.1 · 81.4/81.1 | rail-air-cabin |
| 7 | Outside_Traffic_Crossroads（人行道） | 20 | 69.1/69.6 | street-outdoor |
| 7 | Outside_Traffic_Road（人行道） | 30 | 74.9/73.9 | street-outdoor |
| 8 | Pub_Noise_V2（酒吧） | 30 | 75.0/73.0 | canteen（近似） |
| 8 | Mensa（学生食堂） | 22 | 63.4/61.9 | canteen |
| 8 | Shopping_Center（边走边录） | 30 | 68.3/67.8 | 无对应 |
| 8 | Sports_Noise1 室内足球 / Sports_Noise2 羽毛球（体育馆边上） | 30 | 均为 68.0/68.7 ⚠️ | 无对应 |
| 8 | Train_Station（发车站台） | 30 | 68.2/69.8 | transit-hub（部分，站台不是候车厅） |
| 8 | Cafeteria_Noise（收银台） | 30 | 68.4/67.3 | cafe（近似） |
| 9 | Work_Noise_Office_Callcener（商务办公室） | 30 | 56.6/57.8 | open-office（近似） |
| 9 | Work_Noise_Jackhammer（路边施工） | 30 | 76.2/78.4 | street-outdoor（作突发声） |
| 9 | Work_Noise_Machine_Shop（机加工车间） | 30 | 78.1/78.8 | 无对应 |
| 10 | Livingroom（私人客厅） | 30 | 56.6/57.8 ⚠️ | home-family |
| 11 | Kindergarten_Noise1 / Noise2（游戏室） | 30 | 73.20/71.94 · 80.59/80.38 | home-family（小孩声素材，声级偏高） |
| 11 | Schoolyard（操场边） | 30 | 67.89/68.07 | 无对应（窗外操场可借给 library-classroom） |
| 12 | Nature1_Forest / Nature2_Creek / Nature3_Open_Field | 30 | 41.85/41.24 · 65.93/56.51 · 42.63/42.90 | street-outdoor（公园、户外，近似） |
| 13 | Male_Single_Voice_Distractor（安静房间里的单个男声） | 36 | 68.4/68.2 | 不对应地点。可作 open-office、cowork、meeting-room 的旁人说话素材 |
| 14 | RockMusic（听音室） / Orchestra_noise（乐团调音） | 198 / 149 | 73.9/73.8 · 74.1/73.5 | 摇滚可作 home-family、cafe 背景音乐（近似）；乐团调音无对应 |

⚠️【推断】有两处数字可疑，可能是标准里的抄写错误：
- 表 10 客厅的 56.6/57.8，和表 9 呼叫中心完全相同。
- 表 8 两段体育馆噪声的声级也完全相同。
- 引用前应和录音文件实测核对。

**§8.2 与 TS 103 224 同源的双耳版（表 15）**
- 车内录音都在副驾位置：
  - Mid-size 80/100/130 km/h：66.8/67.7 · 66.0/66.7 · 70.5/71.5
  - Full-size 80/100/130 km/h：60.9/62.9 · 63.8/66.0 · 68.7/70.7
  - 副驾 = 乘客位置，直接对应 **car-passenger**。
- 交通：Inside_Train 68.2/68.4 → rail-air-cabin；Inside_Bus 70.9/73.4 → bus-metro-cabin（14 键无对应）。
- 街道：Roadnoise 74.0/74.1、Crossroad 70.8/71.6 → street-outdoor。
- 公共场所：
  - Cafeteria（室内）69.8/70.3 → cafe 或 canteen
  - TrainStation（站台）78.1/78.8 → transit-hub（部分）
  - Pub 77.7/76.2 → canteen（近似）
  - SalesCounter（超市收银台）66.7/66.6 → 无对应
- 办公：Callcenter1 72.7/70.2、Callcenter2 60.2/60.0、Officeroom 55.4/58.0（投影仪、写字、键盘、电话铃、打电话、窗外声；85 s）→ open-office。
- **Conference 1/2/3**（会议桌，稳态声，或稳态加突发声；70 s）：44.08/45.87 · 45.40/47.00 · 49.50/50.80 → **meeting-room**。
- 家庭（85 s）：
  - Bathroom 72.9/73.1 → 无对应
  - Bathroom_withMusic 73.8/74.2 → 无对应
  - Kitchen 67.1/65.8 → home-family
  - Livingroom（吸尘器、碰杯、咳嗽、电视、收拾）64.1/63.4 → home-family
- Inside_Light_Aircraft（小型飞机）89.7/90.8 → 无对应（不是民航客舱）。
- §8.3 立体声版（表 16–18）：用 2 支心形麦放在后视镜位置录车内，声级与表 1–3 接近，例如 Fullsize 130 km/h 为 68.4/67.4 → car-passenger（近似）。

## 2. ETSI TS 103 224 V1.7.2（2026-07）声场模拟与数据库

来源：【官方文档】<https://www.etsi.org/deliver/etsi_ts/103200_103299/103224/01.07.02_60/ts_103224v010702p.pdf>，见 §5、§8、表 7A–10、附录 A。

**录制方式**
- 8 麦阵列放在 HATS 周围，多点声场重建（§5.0）。
- 有 3 种布置：手持（§5.2）、手持免提（§5.3.2）、**桌面免提**（§5.3.3，麦 5 在主麦正前方 25 mm、高出桌面 25 mm）。
- 文件是 8 声道，48 kHz，24 bit 或 32 bit 浮点（§8.0）。
- 声级是每声道的 dB SPL(A)。下表把 8 个声道的数值写成范围。

| 表 | 名称 | 时长 s | 手持 / 手持免提 / 桌面免提 dB SPL(A)，8 声道范围 | 场景键（【推断】） |
|---|---|---|---|---|
| 8 | MidSizeCar 80/100/130（副驾） | 30 | 66.4–68.4 · 65.1–67.5 · 70.1–72.3 | car-passenger |
| 8 | FullSizeCar 80/100/130（副驾） | 30 | 60.3–64.0 · 63.0–67.1 · 68.3–71.9 | car-passenger |
| 8 | Inside_Train | 30 | 67.6–69.6 | rail-air-cabin |
| 8 | Inside_Bus | 30 | 70.5–73.4 | 14 键无对应 |
| 8 | Roadnoise / Crossroadnoise | 30 | 69.9–73.8 / 69.5–71.6 | street-outdoor |
| 8 | Cafeteria | 30 | 69.0–71.0 | cafe / canteen |
| 8 | TrainStation（站台） | 30 | 78.4–80.1 | transit-hub（部分） |
| 8 | Pub | 30 | 74.8–77.2 | canteen（近似） |
| 8 | SalesCounter（超市） | 30 | 65.2–67.1 | 无对应 |
| 8 | **Airport**（机场走廊，有顶部广播） | 50 | 77.2–78.8 | **transit-hub** |
| 8 | Callcenter1 / Callcenter2 | 30 | 69.6–72.6 / 59.3–60.8 | open-office |
| 8 | Orchestra_Noise / RockMusic | 30 / 198 | 74.5–78.1 / 73.6–76.5 | 无对应 / home-family（近似） |
| 8 | SingleTalker 0° / −22° / +22°（单人在 1 m 处说话） | 90 | 约 58.9–63.5 | 旁人说话素材（open-office、cowork、meeting-room） |
| 8A | Conference1/2/3 | 70 | 手持 45.3–51.1；**桌面 48.2–55.0** | **meeting-room** |
| 8A | Pinknoise_HATS / noHATS | 10 | 80.0 | 无对应（测试信号） |
| 8A | Bathroom / Bathroom_withMusic（只有桌面） | 85 | 69.0–72.7 / 70.5–74.1 | 无对应 |
| 8A | Kitchen / Livingroom（只有桌面） | 85 | 65.8–67.3 / 63.1–64.4 | home-family |
| 8A | Officeroom（只有桌面） | 85 | 53.9–56.6 | open-office（近似，小办公室） |
| 9（双耳阵列） | StationBuilding（车站大楼内） | 35 | 65.1–66.2 | **transit-hub** |
| 9 | RailwayPlatform | 35 | 76.2–76.8 | transit-hub（部分） |
| 9 | Inside_Train_1 电动车 / Inside_Train_2 柴油车 | 35 | 65.7–68.3 / 70.4–70.9 | rail-air-cabin |
| 9 | Inside_Bus / Roadnoise / Crossroadnoise | 35 | 66.2–66.6 / 74.5–75.7 / 66.4–68.6 | 14 键无对应 / street-outdoor / street-outdoor |
| 9 | Inside_Light_Aircraft | 30 | 87.1–88.5 | 无对应 |
| 10（类家庭环境，给语音助手测试用） | Sink（水槽），DUT 位置 1–3 × 两种混响 | 30 | 58.1–66.4 | home-family |
| 10 | Fridge（冰箱） | 30 | 38.5–44.8 | home-quiet（近似，唯一的低声级家庭底噪） |
| 10 | Talker（旁人说话） | 59 | 54.6–61.0 | home-family |
| 10 | All（水槽 + 冰箱 + 说话） | 59 | 61.3–66.7 | home-family |

- 附录 A 表 A.1 给了类家庭环境的两种混响：
  - 低混响：T60 约 0.3 s，C50 为 10.5 dB
  - 中混响：T60 约 0.45–0.5 s，C50 为 6.0 dB
- 【推断】MacBook 内置麦最接近其中的**桌面免提**布置。只有会议、家庭、办公这几段有桌面免提的声级。
- 表 10 的文件名有抄写错误，例如 Talker_DUT3_AC2 写成了 (Sink_DUT3_AC2)，All_DUT2 写成了 (Talker_…)。这是原文问题，引用时要注意。

## 3. 3GPP 怎么引用这批噪声

来源：【官方文档】ETSI TS 126 132 V18.2.0（2024-05，即 3GPP Rel-18）<https://www.etsi.org/deliver/etsi_TS/126100_126199/126132/18.02.00_60/ts_126132v180200p.pdf>。

- §5.1.5：噪声场回放有两种做法。
  - 方法 1：按 ES 202 396-1 回放。
  - 方法 2：按 TS 103 224 第 7 章回放，用 `TrainStation_bin` 的 15.0–22.5 s 作参考声。标准说方法 2 更好。
- **§7.12.1 表 2d（窄带手持）**：8 段 ES 202 396-1 双耳噪声，必须按下面顺序测，声级 L/R 为 dB(A)。§8.12 宽带的表 2h 是同样 8 段。

  | 顺序 | 噪声 | 声级 L/R dB(A) | 场景键（【推断】） |
  |---|---|---|---|
  | 1 | Pub | 75.0/73.0 | canteen |
  | 2 | Road | 74.9/73.9 | street-outdoor |
  | 3 | Crossroads | 69.1/69.6 | street-outdoor |
  | 4 | Train_Station | 68.2/69.8 | transit-hub |
  | 5 | Fullsize_Car1_130Kmh | 69.1/68.1 | car-passenger |
  | 6 | Cafeteria（收银台） | 68.4/67.3 | cafe |
  | 7 | Mensa | 63.4/61.9 | canteen |
  | 8 | Office_Callcenter | 56.6/57.8 | open-office |

  - 语音声级为 −1.7 dBPa @MRP，按 P.56 测。
  - 每段噪声算 16 句的 S-MOS、N-MOS、G-MOS，再对所有噪声取平均。
- **§7.12.2 表 2d2（手持免提）**：5 段 TS 103 224 噪声，免提 8 声道 / 双耳 L/R 声级如下。语音声级为 +1.3 dBPa。

  | 噪声 | 免提 8 声道 | 双耳 L/R |
  |---|---|---|
  | FullSizeCar_130 | 68.6–69.7 | 68.7/70.7 |
  | Crossroadnoise | 69.5–69.9 | 70.8/71.6 |
  | Cafeteria | 69.0–70.0 | 69.8/70.3 |
  | SalesCounter | 65.2–65.6 | 66.7/66.6 |
  | Callcenter2 | 59.3–59.6 | 60.2/60.0 |

- 表 2（ITU-T P.340 房间噪声谱）：63 Hz 为 −45 dBPa，125 Hz 为 −60 dBPa，250 Hz–8 kHz 各倍频程为 −65 dBPa。这是安静房间的谱形，可作 home-quiet、hotel-room 的底噪参考（【推断】）。我没有把它换算成总 dB(A)，未量化。
- 门槛来自【官方文档】TS 126 131 V18.1.0 <https://www.etsi.org/deliver/etsi_TS/126100_126199/126131/18.01.00_60/ts_126131v180100p.pdf>：

  | 条款 | 带宽 | 手持（强制） | 手持（性能目标） | 手持免提（仅建议） |
  |---|---|---|---|---|
  | §5.11 | 窄带 | S-MOS ≥ 3.0，N-MOS ≥ 2.3 | S ≥ 3.5，N ≥ 3.0 | S ≥ 3.3，N ≥ 2.2 |
  | §6.10 | 宽带 | S-MOS ≥ 3.0，N-MOS ≥ 2.3 | S ≥ 3.5，N ≥ 3.0 | S ≥ 3.4，N ≥ 2.3 |

  - 门槛都是对所有测试条件的平均分，G-MOS 没有要求。
- 注意：26.132 的参考文献表引用的是 ES 202 396-1 **V1.4.1（2012）** 和 TS 103 224 **V1.2.1（2015）**，比本文核对的 2026 版旧。Rel-19 版本未核。

## 4. ITU-T P 系列

| 标准 | 有没有场景或噪声分类 | 内容 | 来源 |
|---|---|---|---|
| P.835 | 有类别，没有声级 | 2003 版修订 1（10/2007）附录 III.5 的噪声类型：粉红噪声；babble（≥ 4 人，男女各半，最好 4 支音箱各放一路）；繁忙街道；行驶中的汽车；单人说话（男、女各 ≥ 1）；含鼓的音乐。噪声用 4 支音箱加低音炮在 2 m 处回放，可以直接取 EG 202 396-1 的录音。要有声源移动、多源同时发声。III.6 规定信噪比至少 12、6、0 dB，再加干净语音。III.2 列了 5 种使用方式：手持、远讲、头戴、**桌面免提（speakerphone）**、车载。现行版 P.835（07/26）目录里仍有附录 III.5、III.6，正文未读到，内容是否有变未核 | 【官方文档】<https://www.itu.int/rec/dologin_pub.asp?lang=e&id=T-REC-P.835-200710-S!Amd1!PDF-E&type=items>；目录 <https://www.itu.int/dms_pubrec/itu-t/rec/p/T-REC-P.835-202607-I!!TOC-HTM-E.htm> |
| P.1100 / P.1110（车内免提） | 有，限于车内 | 01/2015 版附录 D 有 4 种使用工况：① 停车怠速 0 km/h，风扇最低；② 城市 60 km/h，风扇比行驶噪声高约 6 dB(A)；③ 高速 120 km/h，风扇最低；④ 高速 120 km/h，风扇高。§11.13.3.1 门槛：≤ 80 km/h 时 N/S/G-MOS ≥ 3.0，80–130 km/h 时 ≥ 2.5。人工头放在**驾驶位**，用 EG 202 396-1 的方式回放。论文实测 44 台车载免提，几段行驶噪声为 67–79 dB(A)（表 3）。→ car-passenger（驾驶位，近似） | 【论文】DAGA 2017，Müsch 等，表 1–3，<https://pub.dega-akustik.de/DAGA_2017/data/articles/000149.pdf>（二手转述，P.1100/P.1110 原文未核）；P.1110（10/2025）摘要 <https://www.itu.int/dms_pubrec/itu-t/rec/p/T-REC-P.1110-202510-I!!SUM-HTM-E.htm> |
| P.1120 | 未查到 | 只从厂商资料知道 P.1100、P.1110、P.1120、P.1140 都有附录 F 讲车内噪声模拟，噪声列表未核 | 【论坛/博客，个人说法】HEAD 3PASS flex 应用说明（二手，原文未核） |
| P.501 | 有噪声序列，类型未查到 | 2000 版修订 1（05/2004）附录 B 带「test speech files and noise sequences」，用 CD 发布。噪声类型和声级未查到 | 【官方文档】<https://www.itu.int/rec/T-REC-P.501> 版本页 |
| P.808 | 未查到 | 只读到版本页（06/21 版现行），没有场景或噪声分类信息 | <https://www.itu.int/rec/T-REC-P.808> |
| P.804 | 没有场景分类 | 是对话质量的诊断法，分 7 个感知维度，听的阶段里有「噪声感（noisiness）」一项。没有场景列表 | 摘要 <https://www.itu.int/dms_pubrec/itu-t/rec/p/T-REC-P.804-201710-I!!SUM-HTM-E.htm>；维度名来自二手资料，原文未核 |
| P.340 | 有一条房间噪声谱 | 倍频程声级见第 3 节 26.132 表 2 → home-quiet（近似） | 经 TS 26.132 表 2 引用（二手，P.340 原文未核） |

## 5. 厂商认证

**Microsoft Teams**
- 【官方文档】<https://learn.microsoft.com/en-us/microsoftteams/devices/usb-devices>，「Special usage designations」一节：
  - 「Premium Microphone for Open Office」认证的要求是：能拾取戴耳机的人的声音，不把邻座的声音传出去。
  - 共享设备「are tested at certification against well-defined rooms sizes and noise environments」。
  - 这里没有给房间尺寸、噪声种类和声级。
- 【论坛/博客，个人说法】测试仪器厂商 HEAD acoustics 的数据表，Code 60026 Rev.10（05/26），第 4–5 页：<https://cdn.head-acoustics.com/fileadmin/data/global/Datasheets/Analysis_Software/ACQUA_Standards/HEAD-acoustics-Data-Sheet-ACQUA-Standards-MS-Teams-Measurements-and-Analyses-According-to-Microsoft-Teams-Audio-Test-Specification-60026-EN.pdf>
  - 规范名是「Microsoft Teams Audio Test Specification V5.0 + Addendum A」。
  - 设备先在消声室测，再在混响室测。
  - 测项包括「3QUEST S-/N-MOS（various background noise types）」，以及开放办公耳机模块里的「Distractor attenuation（average and minimum）」。
  - 会议室音箱另有 EMTR（多用途会议室）模块。
- 场景映射（【推断】）：open-office（开放办公 + 旁人说话）、meeting-room（会议室尺寸）。
- **规范本身未公开**。噪声清单、声级、旁人说话的角度和距离、过线值都未查到。

**Zoom**
- 【论坛/博客，个人说法】HEAD 数据表，Zoom Audio 60068（09/23），第 1、6–7 页：<https://cdn.head-acoustics.com/fileadmin/data/global/Datasheets/Analysis_Software/ACQUA_Standards/ACQUA-Standards-Zoom-Audio-60068-Data-Sheet.pdf>
  - 规范版本：Zoom Meetings Headsets / Speakerphone v2.3、Pro Microphone v1.4、Zoom Rooms Speakerphone v2.3、Pro Audio v2.2。
  - 测试环境是「reverberant, yet quiet office-like room」。噪声用 3PASS lab（8 支音箱）或 3PASS flex（4 支音箱加低音炮）回放。
  - 原文是「background noise typical for the respective environment」，没有给清单。
- 场景映射（【推断】）：open-office、meeting-room。
- 规范未公开，噪声清单和声级都未查到。

## 6. 14 个场景键覆盖汇总（【推断】）

| 场景键 | 标准里最贴近的录音（声级 dB SPL(A)） | 覆盖 |
|---|---|---|
| home-quiet | 冰箱 Fridge 38.5–44.8；P.340 房间噪声谱 | 弱 |
| home-family | 客厅 Livingroom 63–64；厨房 Kitchen 66–67；旁人说话 Talker 55–61；幼儿园 72–81 | 中 |
| open-office | 呼叫中心 Callcenter 57–73；办公室 Officeroom 54–58；Teams 开放办公（声级未公开） | 中 |
| phone-booth | 无对应 | 无 |
| meeting-room | Conference1–3：桌面 48–55，手持 45–51 | 中 |
| cowork | 无对应（只能借 Callcenter 和单人说话素材） | 无 |
| cafe | Cafeteria 67–71 | 中 |
| canteen | Mensa 62–63；Pub 73–78 | 中 |
| transit-hub | Airport 77–79；StationBuilding 65–66；站台 68–80 | 好 |
| car-passenger | 副驾 Mid/Full-size 80–130 km/h 60–72；P.1100 工况 | 好 |
| rail-air-cabin | 火车 62–81；客机 75–81 | 中 |
| street-outdoor | Road 72–76；Crossroads 66–72；风钻 76–78；自然声 41–66 | 好 |
| hotel-room | 无对应 | 无 |
| library-classroom | 无对应（Schoolyard 68 只是室外声） | 无 |

- 不对应任何开会地点的录音：Bathroom、Shopping_Center、Sports、Machine_Shop、SalesCounter、Light_Aircraft、Orchestra、Pinknoise。
- Inside_Bus 在 14 键里没有对应，在 v1 目录（`docs/research/021_meeting-scenes/catalog.md`）里对应 `bus-metro-cabin`。

**怎么核对**：ETSI 和 3GPP 的数字是我用 `pdftotext` 从官方 PDF 逐表抽出来的，读到终端里，没有写任何文件。PDF 由 WebFetch 自动缓存在本会话的 `/Users/allenchen/.claude/projects/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-scenes/d30acd32-697d-4346-9811-adea3e1dbdee/tool-results/` 目录下。