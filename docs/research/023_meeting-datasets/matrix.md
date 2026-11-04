# 覆盖表：开会地点 × 录音设备 → 有哪些真实录音

> 2026-11-03 · 手下 mtgdata · issue #82。用途：查某个「地点 × 设备」格子里，公开真实录音有没有、够不够、能不能用。
> 键：场景键来自 021 `catalog.md` v1，设备键来自 022 `devices.md` v1（`check_cards.py` 直接读这两份正本）。`other` 不是开会场景，不进本表。
> 每格写：数据集 · 量 · 参考 / 转写 · 许可结论。细节见 `cards/<名>.md`（023 新卡）或 `../014_real-datasets/cards/<名>.md` + `cards-014-addendum.md`（014 已有）。
> **措辞口径**：本文说「未找到 / 🟥」一律指「本轮检索范围内，没有确认到能筛出的匹配子集」，不等于世上没有；「自录 / 回放重录」是补法建议，不是结论。
> 🟥 = 公开集里没找到。「回放」= 用音箱放干净语音再录，不是真人在场说话。来源：全部出自卡片，卡上逐条带【官方文档 / 论文 / 实测 / 推断】标签；本表不另加结论。

## 零、一句话
**没有一个格子同时满足「Mac 内置麦 + 真人在场 + 有转写 + 许可可训练」**。本轮查到的 Mac 录音有三处：DR-VCTK（2013–2017 Intel 双麦 MacBook Air 回放重录，办公室）、DNS 2021 盲测里众包者自填 mac 的约 31 条（家里 / 办公混杂，没核实是不是内置麦）、MOBIO（2008 款 MacBook 真人说话，无地点标签，签 EULA）。真人在场的录音多在会议室、家里、车内、餐厅，设备多是研究麦阵或会议终端；公共地点另有手机通话（LIVETALK）与平板朗读（CHiME-3，付费）。

## 一、主表

列：**Mac** = `mba-builtin` / `mbp-builtin` / `mac-intel-builtin` / `mac-unknown` · **其他笔记本** = `laptop-unknown` · **台式** = `non-laptop-desktop` · **手机平板** = `non-laptop-phone` / `non-laptop-tablet` · **麦阵终端** = `non-laptop-mic-array` / `non-laptop-conf-device` · **近讲参考** = `non-laptop-close-talk`（可当干净参考的那一路）

| 场景键 | Mac | 其他笔记本 | 台式 | 手机平板 | 麦阵终端 | 近讲参考 |
|---|---|---|---|---|---|---|
| `meeting-room` | 🟥 | DNS1 dev SurfaceBook 44 条（这是 DNS1 内部真实集的全集数，其中会议室多少条未知；许可未查到）。ACE Chromebook 档只是**合成原料**（实测 RIR + 现场噪声，语音要自己卷积相加），不算真实录音锚 | 🟥 | DAPS confroom（iPad / iPhone 回放，有参考，CC BY-NC → 只评测）· DDS（iPad 回放，有参考；DAPS 部分 CC BY-NC → 只评测，VCTK 部分 ODC-By → 训练可【推断：署名型许可】） | NOTSOFAR-1（会议约 31.5 h，转写，CC BY → 训练可）· AMI（约 100 h，训练可）· ICSI（约 72 h，训练可）· AliMeeting（118.75 h，CC BY-SA → 训练可（D64））· AISHELL-4（120 h，CC BY-SA → 训练可（D64））· MISP-Meeting（125 h，讯飞办公本内置 8 麦，非商用 → 只评测）· ConferencingSpeech 2021（>200 条，需注册） | AMI · ICSI · AliMeeting · NOTSOFAR（仅 train / dev）· MISP-Meeting |
| `open-office` | DR-VCTK Intel 双麦 MacBook Air（`mac-intel-builtin`，2013–2017；回放，有参考；CC BY 4.0，训练可）（2026-11-03 大总管据 macmic 转交改：论文原文为 MacBook Air 双麦，即 2013–2017 Intel 楔形 Air；许可按 datashare README 为 CC BY 4.0，训练可，见 022 `sources.md` D1） | DNS 2021 盲测文件名带 laptop 约 330 条（地点与家混，许可未查到）· MS AEC 真实录音（Windows PC，无逐条标签） | DNS4 盲测 `desktop` 221 条（台式与笔记本分不开，地点只按噪声字段） | DAPS office · DR-VCTK iPhone / iPad · DDS 办公室（iPad，均为回放） | 🟥 | 🟥 |
| `home-family` | DNS 2021 两届盲测自填 mac 约 31 条（与 open-office 混，未核实） | DNS 2021 盲测 laptop 段（同上） | DNS5 dev：Track1 641 条全为 `Desk` / `HS`，Track2 `Desk` 187 条（台式与笔记本分不开）【实测：文件名】 | DNS5 dev Track2 `Mobile` 413 条（文件名标手机；其中 599 / 600 个文件名与 DNS4 盲测相同，是复用）· AISHELL-DMASH（手机 / 平板 + 近讲同步，签协议）· DDS 客厅（iPad 回放，有参考） | CHiME-5/6（约 50 h，晚餐聚会，禁背景电视，CC BY-SA → 训练可（D64））· DiPCo（5.3 h，CDLA → 训练可）· MISP2021（中文电视房，许可未查到）· VoiceHome-2（法语，只评测） | CHiME-6 双耳 · DiPCo · MISP2021 · DMASH |
| `home-quiet` | 🟥 | Speech Commands v2（网页众包，笔记本 / 台式 / 手机混，无设备字段；单词 1 s，CC BY → 训练可） | 🟥 | MagicData-RAMC（手机两人对话 180 h，家里小房间、底噪 < 40 dBA，标笑声；CC BY-NC-ND → 只评测）· AISHELL-2（iOS 全集 1000 h 朗读，其中 1347 人在录音棚、客厅部分时长未知；只评测）· DAPS livingroom（回放） | 🟥（VOiCES 是回放重录，用录音棚麦与摆在房间里的领夹，不归此列；CC BY → 训练可） | 🟥 |
| `phone-booth` | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 |
| `cowork` | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 |
| `cafe` | 🟥 | 🟥（DNS5 盲测标咖啡馆 4 条，无设备） | 🟥 | CHiME-3/4 CAF（平板，真人朗读，LDC 付费 → 不可）· NISQA-LIVETALK（手机，德语通话，有人评分，只评测） | 🟥 | CHiME-3/4 ch0（LDC） |
| `canteen` | 🟥 | 🟥 | 🟥 | 🟥（ASPIRE 的 iPad 只录像，声音来自领夹） | EasyCom（AR 眼镜，真人对话 + 回放餐厅噪声，CC BY-NC → 只评测）· CHiME-9 ECHI（签协议） | ASPIRE 领夹（在食堂里，不干净，非商用）· EasyCom 头戴 · ECHI（近讲只在 train 提供，dev / eval 没有；用作评测参考要另按 D63 切测试份） |
| `transit-hub` | 🟥 | 🟥 | △ DNS5 盲测机场 25 条（`ap_desktop` / `ap_mobile` 混，未逐条核） | NISQA-LIVETALK | 🟥 | 🟥 |
| `bus-metro-cabin` | 🟥 | 🟥 | 🟥 | CHiME-3/4 BUS（平板，真人朗读，LDC → 不可） | 🟥 | CHiME-3/4 ch0（LDC） |
| `car-passenger` | 🟥 | 🟥 | 🟥 | NISQA-LIVETALK 车内（手机） | AISHELL-5（中文车内 2–4 人对话 100+ h，门把麦，CC BY-SA → 训练可（D64））· AVICAR（英语朗读，需申请） | AISHELL-5（仅 train） |
| `rail-air-cabin` | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 |
| `street-outdoor` | 🟥 | 🟥 | 🟥 | CHiME-3/4 PED / STR（平板，LDC → 不可）· NISQA-LIVETALK · DAPS balcony（iPad / iPhone 回放，车流背景，有参考） | 🟥 | CHiME-3/4 ch0（LDC） |
| `hotel-room` | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 | 🟥 |
| `library-classroom` | 🟥 | 🟥 | 🟥 | 🟥 | 🟥（ACE 报告厅只是合成原料） | 🟥（ASCEND 只有领夹单路、无可配对的远场通道，不能当干净参考；可作中英夹杂对话底料或吃字检查，CC BY-SA → 训练可（D64）） |

**有地点（或推断地点）、但设备未知的**（主表没有「设备未知」列，列在这里；设备未知不等于该地点没有数据）：
- `meeting-room`：WenetSpeech TEST_MEETING（中文真实会议 15 h，有转写，无干净参考，要填表申请）· ConferencingSpeech 2021 已在麦阵列
- `home-family`【地点是推断，卡上写各自在家 / 所在地、未逐条标】：CANDOR（Zoom 对话；设备也是推断，多为笔记本或台式摄像头麦；签协议）
- `street-outdoor`：VoxCeleb 户外采访（无逐条地点标签，要人工挑，申请表拿密码）
- 多地点：Ego4D（头戴相机，签协议）

**只有背景声、没有主说话人的**（不进主表，是合成原料；设备按各卡设备键）：
- `open-office`：ARTE 开放办公（`non-laptop-mic-array`）· DEMAND OOFFICE（`non-laptop-mic-array`）· TUT 2017 office（`non-laptop-studio`）
- `cafe` / `canteen`：ARTE · DEMAND SCAFE / PCAFETER / PRESTO · QUT CAFE（`non-laptop-studio`，Zoom H2 录音笔）
- `transit-hub` · `bus-metro-cabin`：TAU 2020 Mobile 机场 / 地铁站 / 公交 / 电车（`non-laptop-phone` · `non-laptop-studio`，非商用）· ARTE 火车站 · DEMAND PSTATION / TBUS / TMETRO
- `car-passenger`：QUT CAR · DEMAND TCAR
- `street-outdoor`：TAU 街道 · QUT STREET · DEMAND STRAFFIC
- 许可：DEMAND、QUT 为 CC BY-SA → 训练可（D64）；ARTE、TAU、TUT 为非商用 → 只评测

**要签协议、未逐段核的**：Ego4D（头戴相机麦，卡上场景键含 `street-outdoor` · `transit-hub` · `cafe` · `home-family` · `car-passenger`，部分有转写；哪些片段有主说话人未逐段核）。

**无地点标签、但设备有用的**（进不了上表的行）：3D-Speaker（训练集笔记本 96,262 条，按设备可筛；全集是普通话 + 13 种方言，笔记本段的语种分布未知；型号未写，音频许可未查到）· MOBIO（2008 款 MacBook 真人说话，`mac-intel-builtin`，签 EULA）· SIG / SSI 挑战 2023–2024 真实集（2023 test 500 条 + 500 条转写、2023 blind 500 条、2024 blind 500 条；每条一台设备，PC 与手机混，无逐条设备标签；许可未查到）· NISQA TRAIN_LIVE / VAL_LIVE（有声书经音箱回放进手机 / 笔记本，1,220 条，设备不能逐条筛，地点未核；是回放，不是真人会议）· Common Voice（网页众包，推断混有笔记本麦，但无设备字段、筛不出来）· URGENT 2024–2026 盲测真实段（无设备、无地点）。

## 二、空格子怎么补

| 空格 | 公开集里有什么可借 | 怎么补 |
|---|---|---|
| 所有地点 × Mac | DR-VCTK（回放）· 约 31 条自填 mac · MOBIO（无地点） | 回放重录（见第三节）或自录 |
| `cafe` / `canteen` × 笔记本 | 背景噪声齐（QUT、DEMAND、ARTE）；本轮找到的真人来源是平板、眼镜、领夹与手机通话（LIVETALK，说话端有分歧），尚未确认可筛的笔记本子集 | 建议：合成（干净语音过 Mac 设备链 + 真实背景）；要真实锚可自录 |
| `home-family` × 「一人开会、家人在旁」 | MISP2021 是多人闲聊，不是一人开会 | 合成（QUT HOME / CHiME-Home 背景 + 干净语音）；或自录 |
| `open-office` · `transit-hub` · `street-outdoor` × 笔记本主说话人 | 开放办公只有背景与众包段；机场车站、街边找到的真人来源：手机通话（LIVETALK，德语，说话端是不是笔记本有分歧）、平板朗读（CHiME-3，付费）、Ego4D（签协议，头戴相机）、VoxCeleb 户外采访（无地点 / 设备标签） | 建议：核 LIVETALK 说话端与 Ego4D / VoxCeleb 子集；仍不够再自录，见第三节 |
| `phone-booth` · `cowork` · `rail-air-cabin` · `hotel-room` | 本轮未专门检索 | 先合成；下一轮补查后再定要不要自录 |
| 中文 × 笔记本 | 3D-Speaker 有设备标签但无地点、许可未查到 | 核清 3D-Speaker 音频许可后可做中文笔记本锚 |

## 三、自录方案（建议，用于开放办公 · 机场车站 · 街边 × 笔记本；本轮未确认到公开匹配子集）

> 这一节是**步骤草案**，不是已定的事。按 D62，开发阶段不做作者实录；要不要录、何时录，写在 `decision.md` 待人拍板。我（mtgdata）不录音、不放声。

**每个格子要录什么**【推断：按 016 第三节真实锚层的用途倒推】：
1. 说话人坐在 MacBook Air 前，正常开会距离（约 40–60 cm），念一段固定稿子（约 3 min）+ 自由说话（约 3 min）
2. **同时**录一路近讲参考：领夹麦接手机，或头戴耳机麦。开头拍一下手，用来对齐两路
3. 原地不说话，录 2 min 纯背景
4. 记下：时间、位置、Mac 型号、系统麦克风模式（标准 / 语音突显 / 宽频谱）、近讲设备型号

**要多久**【推断：依据上面每格约 8 min 录音 × 3 遍取稳】：
- 每个地点录 3 次（不同时段），每次约 10 min，现场约 30 min；加路程，每个地点约 1–1.5 h
- 三个地点合计约 3–4.5 h（人的时间）；产出约 24 min 有效语音 / 地点

**难点**：
- 公共场所会录到旁人声音：只存本机、不入库、不外发（铁律 13）
- 机场、车站可能不允许录音：先确认场所规定
- 近讲那一路在嘈杂处也不是完全干净，只能当「较干净参考」，算全参考指标时要注明

**替代：回放重录**（不出门，约 2 h）：安静房间里，一只音箱放干净语音（离 Mac 约 50 cm），另一只放 TAU / DEMAND / QUT 的真实背景，用 Mac 内置麦录。得到「真实设备链 + 已知干净参考」，但声场是回放的，不是真人在场。⚠️ 这一步要外放声音，需要人在场安排时间。
