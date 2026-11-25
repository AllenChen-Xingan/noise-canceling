# 第二批训练噪声 · 下载清单（调研稿，未下载）

> 2026-11-25 · traindata 写。交代：大总管（人定 D106「先补数据，再针对最弱场景加训」）。**只调研，本机和云上都没下载任何音频。**
> 用途：给 8 个「训练数据里没有噪声池」的场景找可训练的公开噪声，交人看后再由 cloudrun 下载。
> 场景名与弱项名次取自主仓 `_tmp/20261125-gap-v1/README.md` 第一张表（GTCRN 原版云上 480 段，不含吃字）。
> 许可规矩照 D64：CC0 · CC BY · BY-SA 可训练（BY-SA 记署名，权重按 BY-SA 共享）；NC · ND · 仅限学术 只评测、不训练。
> 来源标注：**实读** = 本次读了网页或元数据文件；**推断** = 由实读数字算出或按经验判断；**旧记录** = 抄自 027 调研、本次没重读。

## 1. 结论先说

> **人定（2026-11-25，经大总管转达）：第二批只下排序 1（CochlScene 3 类）；排序 2、3「人未选，暂不下」。盘放不下可扩到 200 GB。**

按「覆盖弱项前列 · 许可干净 · 体积小」排出 3 个来源。

| 排序 | 来源 | 补哪些场景（括号里是弱项名次） | 许可 | 下载量 | 能不能训练 |
|---|---|---|---|---|---|
| 1 | CochlScene 的 3 类：地铁站 · 地铁车厢 · 拥挤室内 | 车站（第 3）· 火车舱的「车厢」部分（第 1）· 联合办公（第 10）· 开放办公（第 6，只算部分） | CC BY-SA 3.0 | 53.9 GB（分卷包，只能整包下） | 能，要先筛人声 |
| 2（人未选，暂不下） | DEMAND 没被评测集用过的 2 个环境：OHALLWAY · DLIVING | 开放办公（第 6）· 安静的家（第 12）· 酒店房间（第 14，只算近似） | CC BY 4.0（Zenodo 元数据）/ BY-SA 3.0（说明文字），两种都能训练 | 0.16 GB | 能 |
| 3（人未选，暂不下） | FSD50K 按标签取：风扇 · 打字 · 键盘 · 打印机 · 门 · 敲门 | 电话亭（第 5）· 开放办公 · 安静的家 · 酒店房间（都只补突发声与风扇，不补底噪） | 每条单独许可；只留 CC0 和 CC BY，去掉 NC 和 Sampling+ | 24.7 GB（分卷包，只能整包下） | 能，要过滤许可和同源 |

有 3 个场景**没有合规的数据**，照实写：

- **飞机舱**（火车 / 飞机舱的「飞机」部分，第 1）：没找到可训练的机舱内录音。FSD50K 的 `Aircraft` 多是机外飞过的声音（推断，没逐条听）
- **会议室**（第 4）：DEMAND 的 OMEETING 是「讨论麦克风阵列时的会议室」（DEMAND.pdf，实读），**底下全是能听清的人声**，不能直接当噪声。AMI 会议语料是 CC BY 4.0，但同样以说话为主（实读许可页，没核有没有纯环境段）
- **电话亭**（第 5）：没有专门的数据；排序 3 只能补风扇和敲门

## 2. 逐条说明

### 2.1 CochlScene（排序 1）

- 出处：https://zenodo.org/records/7080122（实读）；元数据 https://github.com/cochlearai/cochlscene 的 `Data_info.tsv`（实读，4.2 MB，放在本机临时目录，不进仓）
- 许可：Zenodo 写 "Creative Commons Attribution Share Alike 3.0 Unported"（实读）。GitHub 仓库没有许可文件（实读）。仓库已在 2026-09-17 存档、只读（实读）
- 内容：76,115 条 × 10 秒，13 类，831 个录音人，手机录音（实读）。采样率 44.1 kHz、单声道是从官方代码读出来的（推断）
- 我们要的 3 类（条数与人数由 `Data_info.tsv` 实数）：

| 类 | 条数 | 录音人数 | 单个人最多占 | 补哪个场景 |
|---|---|---|---|---|
| SubwayStation（地铁站） | 5,897 | 191 | 2 % | 车站 transit-hub |
| Subway（地铁车厢） | 5,897 | 166 | 2 % | 火车 / 飞机舱 rail-air-cabin（只像火车车厢） |
| CrowdedIndoor（拥挤室内） | 5,835 | 213 | 1 % | 联合办公 cowork · 开放办公 open-office |

  另有 Bus（5,821 条）可加强已有的公交 / 地铁车厢池，Cafe（5,867 条）可加强咖啡馆池；不在这次 8 个场景里，先不列。
- 大小：分卷压缩包 `.z01`–`.z05` 每卷 10.7 GB，加 `.zip` 173 MB，共 53.9 GB；每卷有 md5（实读）。**分卷包不能只下 3 类**，要整包下完再解出 3 类
- 和评测集同源吗：**不同源**。exp013 的素材（`materials.json` 与 `sounds.json`）只有 DEMAND、ESC-50、ACE、McVAMPIRE、MIT 冲激响应、自己合成的声音、两段 Wikimedia 朗读和音乐；仓里 exp013 与 exp006 的文件里查不到 CochlScene（实测 `grep`）
- 怎么切分防泄漏：
  - 官方的 Train / Val / Test 三份录音人**互不重叠**：423 · 118 · 290 人，两两交集 0（`Data_info.tsv` 实数）
  - 我们照旧用自己的切分器（`split.py`），**按录音人切**：单位 = 文件名里的 `userNNNN`，同一个人的所有录音进同一份（训练或开发）。不用官方的份，切分规则只留一个正本
  - 不进终测：评测集照旧只用 exp013
- 要注意的事：
  - **人声**：数据页没写有没有可听清的人声（实读）。027 调研记着论文说「有些提交可能含录音者本人的说话声」（旧记录）。下完要先过一遍人声检测，筛掉有人声的段；筛掉多少，下完才知道（未量化）
  - 地铁车厢只能代表「火车舱」，不能代表飞机舱
  - BY-SA：用了它训练的权重要按 BY-SA 发、记署名（D64）

### 2.2 DEMAND 的 OHALLWAY · DLIVING（排序 2 · 人未选，暂不下）

- 出处：https://zenodo.org/records/1227121（实读）；环境说明见同一记录里的 `DEMAND.pdf`（实读）
- 许可：页面两处不一致：说明文字写 CC BY-SA 3.0，元数据字段写 CC BY 4.0（实读）。两种都能训练，按较严的 BY-SA 记署名
- 内容：每个环境 16 路单声道 WAV，每路 300 秒（实读）
  - OHALLWAY：「办公楼走廊，偶尔有个人或几个人走过」（实读）→ 开放办公
  - DLIVING：「客厅」（实读）→ 安静的家；酒店房间只能算近似
- 大小：16 kHz 包 OHALLWAY 77.9 MB、DLIVING 80.2 MB，共 0.16 GB（实读）
- 和评测集同源吗：同一个数据集，**但不是同一个环境**。
  - exp013 用了 DEMAND 的 10 个环境：`sounds.json` 里有 OOFFICE · DKITCHEN · PCAFETER · SCAFE · PRESTO · STRAFFIC · TBUS · TMETRO · TCAR · PSTATION（实读）。OHALLWAY 和 DLIVING 不在里面；exp013 与 exp006 的文件里都查不到这两个名字（实测 `grep`）
  - **OOFFICE 不能用**：exp013 用它当办公室底噪，前一半留作测试份，后一半是评测原料（`pools.py` 的 `demand_window`）
  - OMEETING 不列，原因见第 1 节（全是人声）
- 怎么切分防泄漏：
  - 照已有的 `demand-train` 规则，按时段切：**训练 0–40 %，开发 40–50 %，后 50 % 不用**
  - 后 50 % 空着是有意的：如果以后 exp013 加了这两个环境，它的规则是取后一半（`pools.py` 的 `TEST_SPLIT_FRAC = 0.5`），这样两边不会重叠
  - **放的位置要和 exp013 分开**：放到 `train-raw/demand-extra/`，不放 `vdn-data/demand/`。原因：exp013 列素材时会把 `vdn-data/demand/*/ch*.wav` 全部算进去（`pools.py` 第 159 行），放进去会改掉 exp013 build 的素材清单
  - 建议加一条机器检查：exp013 的 `sounds.json` 不许出现 `demand-extra` 里的环境名，出现就红（待做）

### 2.3 FSD50K 按标签取（排序 3 · 人未选，暂不下）

- 出处：https://zenodo.org/records/4060432（实读）；标签表 `vocabulary.csv` 读的是 HuggingFace 上的第三方镜像（实读），下载后要用官方包里的那份再核一次
- 许可：数据集整体是 CC BY，**每条另有许可**，由 Freesound 上传者定（实读）
  - 开发份 40,966 条：CC0 14,959 · CC BY 20,017 · CC BY-NC 4,616 · Sampling+ 1,374
  - 评测份 10,231 条：CC0 4,914 · CC BY 3,489 · CC BY-NC 1,425 · Sampling+ 403
  - 只留 CC0 和 CC BY；NC 和 Sampling+ 一律不进训练
  - 作者在说明里写，商用要另外联系他们（实读）。这一条要人看：我们按 D64 只看每条的 CC 许可，这句话会不会限制我们，**建议，待你定**
- 内容：51,197 条，0.3–30 秒，共 108.3 小时，44.1 kHz 单声道 16 位（实读）
- 用到的标签（拼写按镜像里的 `vocabulary.csv`，实读）：

| 标签 | 补哪个场景 |
|---|---|
| `Mechanical_fan` | 电话亭 · 安静的家 · 酒店房间 |
| `Typing` · `Computer_keyboard` · `Printer` · `Writing` | 开放办公 · 安静的家 |
| `Door` · `Knock` · `Drawer_open_or_close` · `Cupboard_open_or_close` | 酒店房间 · 会议室（只补突发声）· 电话亭 |

  没有「空调」「冰箱」「咖啡机」这几个标签（实读），底噪补不了。各标签的条数要下载后才能数（未量化）。
- 大小：分卷包共 24.7 GB（开发份约 18.3 GB，评测份约 6.2 GB，实读），**同样只能整包下**
- 和评测集同源吗：**有同源风险**。ESC-50 的片段也是从 Freesound 剪的（实读 ESC-50 说明），exp013 用了 168 条 ESC-50（`materials.json`）。处理办法：
  - FSD50K 的文件名就是 Freesound 编号（实读说明）；ESC-50 每条在 `materials.json` 里记着 `src_file`，也就是 Freesound 编号
  - **排除所有和 ESC-50 同一个 Freesound 编号的片段**。范围取 ESC-50 全部 2,000 条，不只 exp013 用到的 168 条：以后 exp013 换片段也不会撞
  - 残留风险：同一个上传者同一次录的不同文件，编号不同，排除不掉（推断）。**大总管 2026-11-25 定：按上传者也排除**（防泄漏的技术题）。ESC-50 没给上传者名，要先查出 ESC-50 那 2,000 条的 Freesound 上传者，再排除 FSD50K 里这些上传者的全部片段；剩多少条下载后才知道（未量化）
- 怎么切分：按上传者切（`clips_info` 里有上传者名，实读说明；字段名下载后再核）。同一个上传者的片段进同一份
- 不用的：exp013 自己合成的声音（风扇、空调、机舱底噪等，`synth_sounds.py`）**不能拿来训练**，同一个生成器就算同源

### 2.4 查过但不能用的

| 来源 | 为什么不能用 |
|---|---|
| DEMAND OOFFICE · PSTATION · TMETRO · TCAR | exp013 评测集在用（第 2.2 节） |
| DEMAND OMEETING | 底下全是人声（第 1 节） |
| ACE 语料的房间噪声 | exp013 用它当会议室、办公室、大厅的底噪（`scenes.json` 的 `noise`） |
| TAU Urban 2020（机场 · 地铁站 · 电车） | 非商用（旧记录） |
| WHAM! noise | CC BY-NC（旧记录） |
| DNS Challenge 噪声 | 里面含 DEMAND 整段，和评测集撞源；AudioSet 部分的音频版权不清楚（旧记录） |
| MS-SNSD | 噪声来自 Freesound（只收 CC0）和 DEMAND 整段，和 ESC-50、DEMAND 都可能撞源，也没有场景标签（实读 README） |
| AMI 会议语料 | CC BY 4.0，但以说话为主，不是噪声（实读许可页） |

## 3. 网络盘

**这一项我量不了。** 网络盘挂在云上的 `/workspace`，本机碰不到；交代要求用 `du` 量，不用 `df`（cloudrun 记过：`df` 看到的是整个集群 158 TB，看不到 150 GB 配额）。下面是已有的记录和推算，**请 cloudrun 下次开机先跑**：

```bash
du -sh /workspace
du -sh /workspace/* | sort -h
```

已有的读数（cloudrun 的日志，实读）：

- 2026-11-25 00:21Z 第一批下完、解压后：`/workspace` 共 33 GB
- 之后加了分片：`/workspace/shards/first-batch` 15 GB（02:22Z）
- 之后还有 exp013 云上 build、基线 480 段、`.venv`，没有读数
- 所以现在至少约 48 GB（推断，没算后面几项）

第二批的峰值（推断）：

| 项 | 大小 | 依据 |
|---|---|---|
| CochlScene 分卷包 | 53.9 GB | 实读 |
| 解出 3 类的 44.1 kHz WAV | 约 15.4 GB | 17,629 条 × 10 秒 × 88.2 KB/秒 |
| 转成 16 kHz 后保留 | 约 5.6 GB | 17,629 条 × 10 秒 × 32 KB/秒 |
| FSD50K 分卷包 | 24.7 GB | 实读 |
| DEMAND 两个环境 | 0.16 GB | 实读 |

建议按顺序下，每个来源下完就删压缩包，不要两个大包同时在盘上：

- 先 CochlScene：峰值约 48 + 53.9 + 15.4 ≈ 117 GB。离 cloudrun 定的「超 120 GB 停」只差约 3 GB，太紧
- 删掉分卷包和 44.1 kHz 中间文件后，再下 FSD50K：峰值约 48 + 5.6 + 24.7 + 解出的标签片段（未量化，估几 GB）≈ 80 GB

**要不要扩盘**（人定：放不下可扩到 200 GB；这次只下 CochlScene，FSD50K 那一步不做）：

- 如果 cloudrun 量出来现在 ≤ 45 GB，又能确认第一批的压缩包已经删了：不用扩，按上面的顺序下
- 否则：扩 50 GB（150 → 200 GB），CochlScene 的峰值才有约 80 GB 余量。按 D96 的单价（150 GB 约 $10.5 / 月）推算，每月约多 $3.5（推断）
- 另一个办法是解压时直接从分卷包解出 3 类，不先合成一个大包（`7z x` 能直接读分卷，推断）。合成大包要再占 53.9 GB，一定不要这么做

## 4. 下载后要做的（给 cloudrun 和 traindata，不在这次）

1. `sources.json` 加三个来源，`dest` 分别是 `train-raw/cochlscene` · `train-raw/demand-extra` · `train-raw/fsd50k`，钉住 md5
2. `split_rules.json` 加三条规则：CochlScene 按录音人、DEMAND 按时段（0–40 % / 40–50 %）、FSD50K 按上传者，并排除 ESC-50 的 Freesound 编号与上传者。先写会红的测试
3. 查 ESC-50 全部 2,000 条的 Freesound 上传者（排除用，见 2.3）
4. 人声筛查：CochlScene 和 FSD50K 都要做；方法和阈值另定
5. `recipe.json` 的 `noise_pools` 加新场景；`target_scene.scene` 可以写成列表，比如 `["rail-air-cabin", "transit-hub"]`（loader 已支持，见本分支 `tests/test_target_scenes.py`）
6. 改了 loader 和配方，build 身份就变了，云上要重跑 `register`
