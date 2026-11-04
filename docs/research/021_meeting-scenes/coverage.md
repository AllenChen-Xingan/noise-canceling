# 021 · 地点 × 条件因素覆盖矩阵（2026-11-03）

> 用途：看清「现在的评测集在哪些开会地点、哪些因素上是空的」，并排出补的先后。
> 地点键的正本是 `catalog.md` 主表；因素 F1–F16 的正本是 `docs/research/016_bench-synth/benchmark.md` 第二节。本页只引用，不另立定义。
> 本页只是提议，不改 `src/`、不改任务集。出处编号 `[Sxx]` 查 `sources.md`。

## 一、「覆盖」怎么数【推断，本线自定的规则】

一条评测任务算覆盖某个地点的某个因素，要同时满足：

1. 任务的噪声或干扰是这个地点的**代表声音**（例如咖啡馆噪声、6 人 babble、邻座说话）；粉噪、白噪这类抽象稳态噪声**不算任何地点**；
2. 这个因素的取值落在该地点的典型范围里（范围见 `catalog.md` 主表）。
3. 例外：F6 混响、F7 距离只看取值，不看噪声素材（混响属于房间，和噪声是什么无关）。exp008 S-RT60 的四档是 0.08 · 0.35 · 0.7 · 1.3 s（`docs/experiments/exp008-bench-validity/cells.py` 第 44、56 行）【仓内】；地点典型值落在某档上记 ●；典型值未查到记 ○；只有 016 的假设值记 ◐。
4. 同一条任务可以同时算进几个地点（例如六人 babble 的 C04 / C05 同时算咖啡馆和食堂），`catalog.md` 主表的计数按同一映射。

只靠粉噪 / 白噪任务撑着的格子记 ◐，不记 ●。

数据来源：`src/eval/suite/ci.json` 32 条、`src/eval/suite/qa.json` 11 条、exp008 的 4 条单类轴（S-SNR、S-NOISE、S-RT60、S-SIR）【仓内】。
真实锚为零：qa 的 11 条全是 exp003 / exp005 的合成或挑战赛片段，没有一条是笔记本在开会地点录的【仓内】。

## 二、哪些因素跟地点有关

16 个因素里，9 个随地点变，7 个与地点无关：

- **随地点变**（进矩阵）：F1 噪声类型 · F2 SNR · F3 背景声级 × Lombard · F4 邻座 SIR · F5 重叠 · F6 混响 T60 · F7 距离 · F11 发声力度 · F15 突发
- **与地点无关**（不进矩阵）：F8 设备链（归 022 macmic）· F9 下游编解码 · F10 音高 · F12 语速 · F13 语种 · F14 停顿 · F16 输入电平
- **因素表里缺的一项**：风噪（街边户外最主要的难点，URGENT 2025 单列为失真 [S16]）。提议加为 F17，待 016 正本的主人定。

## 三、矩阵

记号：**●** 已覆盖 · **◐** 有任务但取值或素材不对路 · **○** 关键但评测集空 · **·** 这个地点不关键。

| 地点键 | F1 噪声 | F2 SNR | F3 声级×Lombard | F4 SIR | F5 重叠 | F6 T60 | F7 距离 | F11 力度 | F15 突发 | 空格数 ○ |
|---|---|---|---|---|---|---|---|---|---|---|
| home-quiet | ◐ 只有合成风扇嗡声 R06 | ◐ 只有粉噪任务 R01 | · | · | · | ● exp008 0.35 s | ○ | ◐ R03 轻声，粉噪 | ○ 键盘、门声 | 2 |
| home-family | ○ 电视人声、小孩、宠物都没有 | ◐ | · | ○ 电视人声，麦处 SIR 未知 | ○ 电视连续说话 | ● | · | · | ○ 狗叫、门铃 | 4 |
| open-office | ◐ 没有键盘 | ◐ 邻座任务的底噪是粉噪 | · | ● C01–C06、S-SIR | ● C02 重叠 | ○ 开放办公 T60 无数据 | ○ 邻座 1.5–4 m | · | ○ 电话铃、提示音 | 3 |
| phone-booth | · | ◐ 只有粉噪任务 | · | · | · | ○ 小空间强早期反射 | · | · | · | 1 |
| meeting-room | · | ◐ | · | ○ 近处几人都要留 | ○ 多人交叠 | ● exp008 0.7 s | ○ 0.5–2 m 多人 | · | ○ 椅子、笑声 | 4 |
| cowork | ○ 无专属素材 | ◐ | · | ○ 陌生人说话 | · | · | · | · | ○ 咖啡机、走动 | 3 |
| cafe | ● exp008 DEMAND 真实咖啡馆 | ● R08 5 dB、C07 0 dB | ○ 声级与 SNR 未联动 | ◐ 只有 C07 一条 | ◐ | ○ 未查到典型值，任务固定 | · | · | ◐ 合成杯碟未过统计对齐 | 2 |
| canteen | ◐ 合成 babble、赌场片段 | ● −5 dB C05 | ○ 61–73 dBA 未联动 | ○ 同桌多人 | · | ○ 约 1 s 混响加 babble | · | · | ◐ | 3 |
| transit-hub | ○ 广播 | ○ | · | · | · | ○ 8.6–10 s 超出网格 | · | · | ○ 广播、行李箱 | 4 |
| car-passenger | ◐ exp008 traffic 是车外 | ○ 约 5.6 dB | · | ○ 司机、导航 | · | ◐ exp008 有 0.08 s 档 | · | · | ○ 转向灯 | 3 |
| bus-metro-cabin | ◐ exp008 TBUS 1 段 | ○ | · | ○ 乘客 | · | · | · | · | ○ 报站广播 | 3 |
| rail-air-cabin | ○ 舱噪 | ○ 约 −2 dB [S31] | ○ 77–79 dB 环境下的 Lombard | ○ 邻座 0.4 m | · | · | · | · | ○ 广播 82–88 dB(A) | 5 |
| street-outdoor | ◐ exp008 traffic | ○ | · | · | · | · | · | · | ○ 喇叭、风噪（缺 F17） | 2 |
| hotel-room | ◐ 只有抽象稳态 | ◐ 只有粉噪任务 | · | · | · | ○ | · | · | ○ 走廊、空调启停 | 2 |
| library-classroom | · | ◐ 只有粉噪任务 | · | ○ 旁人轻声交谈 | · | ● exp008 1.3 s | · | ◐ R03 轻声，粉噪 | ○ 翻书、键盘 | 2 |

空格总数 43 个【仓内，按上表数】。全表一个 ● 都没有的因素有 4 个：F3 声级与 SNR 联动、F7 距离、F11 发声力度（只有粉噪任务 R03）、F15 突发（ci、qa、exp008 里都没有按地点的突发声音）【仓内】。

## 四、补的先后（按「用户占比 × 难度」）

用户占比只有工作地点的工时比例 [S57]：雇主场所 44.4%（工位、会议室、电话亭未拆）、家 29.1%、共享办公 5.3%、公共场所 5.3%；出行类没有工时数据，只知道「过去一年去过」的比例偏高且口径存疑 [S60]。
难度按三条看：空格数（上表）；现有模型在类似片段上的表现（exp003 里食堂 / 多人 babble 片段三家全「不能用」【仓内】）；是不是人声对人声。

| 批次 | 地点 | 理由 | 先补哪几格 |
|---|---|---|---|
| 第 1 批 | home-family | 家占 29.1% 工时 [S57]，在家时客厅 20% [S58]；电视人声可能和用户差不多响（未量化，依据弱），人声对人声；评测 0 条 | F1 电视 / 视频外放人声、狗叫、小孩；F4 SIR 档位待定（测试假设 0 ± 6 dB，未验证）【推断】；F15 突发 |
| 第 1 批 | open-office | 雇主场所 44.4% 工时 [S57]；邻座已有 11 条，但缺键盘、电话铃、距离 | F15 键盘与电话铃；F7 邻座 1.5 / 2 / 4 m；F6 开放办公混响 |
| 第 2 批 | meeting-room | 也在 44.4% 里；方向和「只留我」相反，要先定产品支不支持（见 `decision.md` 待拍板第 1 题） | F4 / F5 / F7 多人都要留 |
| 第 2 批 | canteen · cafe | 各约 5.3% 以内；exp003 同类片段全「不能用」【仓内】；缺声级与 SNR 联动 | F3 联动；F6 约 1 s 加 babble；F4 同桌多人 |
| 第 3 批 | rail-air-cabin · transit-hub | 占比未查到；最难（SNR 约 −2 dB、T60 8–10 s）；空格最多 | F2 负 SNR；F6 长混响（网格要加到 8 s 以上）；F15 广播 |
| 第 3 批 | car-passenger · bus-metro-cabin · street-outdoor | 占比未查到；中等难度；street 要先有 F17 风噪 | F1 真实车内 / 车厢噪声；F15 |
| 第 4 批 | home-quiet · hotel-room · phone-booth · library-classroom · cowork | 安静类，难在不做坏事，现有抽象稳态任务大体管住；cowork 缺素材但占比小 | F15 轻突发；F6 小空间早期反射 |

## 五、与现有两套分类的对应（只提议，不改代码）

### 5.1 用户层 5 类（`src/types/src/scene.rs`：quiet · cafe · commute · music · meeting）

| 用户层 | 本目录地点 | 问题 |
|---|---|---|
| quiet | home-quiet · phone-booth · hotel-room · library-classroom ·（home-family 勉强） | home-family 的电视、小孩、宠物和「安静」完全相反，塞不进来 |
| cafe | cafe · canteen ·（cowork 勉强） | canteen 声级与混响都高一档 |
| commute | transit-hub · car-passenger · bus-metro-cabin · street-outdoor ·（rail-air-cabin） | 室内长混响（transit-hub）和车内短混响放在一类 |
| meeting | meeting-room ·（open-office、cowork 勉强） | open-office 要「只留我」，meeting-room 要「留所有人」，方向相反 |
| music | 无对应地点 | music 是一种噪声，不是地点 |

提议（建议，待人定）：用户层至少要能分出「家里有人」和「近处多人都要留」两种，即在 quiet 之外加一个家里有人的类，并把 meeting 拆成「开放工位只留我」和「会议室留所有人」。具体是否改、改成几类，见 `decision.md` 待拍板第 2 题。

### 5.2 噪音 7 类（`src/scene/src/classify.rs`：Quiet · Hvac · Babble · Neighbor · Clatter · Traffic · Music）

| 噪音类 | 主要出现的地点 |
|---|---|
| Quiet | home-quiet · phone-booth · hotel-room · library-classroom |
| Hvac | home-quiet · hotel-room · open-office · meeting-room · rail-air-cabin（舱噪近似） |
| Babble | cafe · canteen · transit-hub · cowork |
| Neighbor | open-office · cowork · cafe · rail-air-cabin · meeting-room（但这里要留） |
| Clatter | cafe · canteen · home-family |
| Traffic | street-outdoor · car-passenger · bus-metro-cabin |
| Music | cafe · home-family |

7 类里**缺 4 种声音**，它们各自是某个地点的主要难点【推断】：

- **外放的人声**（电视、视频、广播）：home-family、transit-hub、rail-air-cabin、bus-metro-cabin。与 Neighbor 不同，它常常连续不停、可能比用户还响
- **非语言的人声和动物声**（小孩哭闹、狗叫）：home-family
- **键盘与提示音**：open-office、home-quiet、cowork（现在大概只能算进 Clatter）
- **风噪**：street-outdoor（对应提议的 F17）

## 六、真实锚

- `open-office`、`transit-hub`、`street-outdoor`：mtgdata（023）的卡片里没有带主说话人的公开真实集，但**还有两条候选没核**，不能下「只能自录」的结论：
  - DNS 2020 开发集的微软内部实录含「开放办公区和会议室」，未拆比例 [S10]
  - Krisp 测试集工作组含开放工位层，手机通话组含街道、地铁站 [S24]
  - 要核：设备是不是笔记本、有没有主说话人、许可是不是只评测——这是 023 的活，已转交
- 开发阶段不用真实录音（D62）；要不要为这三格破例自录，见 `decision.md` 待拍板第 3 题。
- 其余地点有没有公开真实集，以 023 数据卡的场景键为准，本页不重复列。
