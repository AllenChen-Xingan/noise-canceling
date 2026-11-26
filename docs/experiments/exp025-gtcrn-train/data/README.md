# exp025 · data：降噪模型微调用的训练数据

训练数据只在云上下载、切分、混音；本机只跑代码和单元测试（训练数据不放本机，D88；重活上云，D97）。
负责的是会话 `traindata`（issue #156）。计划见 `docs/plan/014_training-kickoff.md` 第 0 步 ③ 与第 2 步。

## 一、这份文件有什么用

这里写数据怎么下、怎么切、怎么混，以及训练代码（会话 `trainer` 写的）怎么取数据。每样东西的正本在哪个文件，见第二节。
文中的 traindata · cloudrun · evalcmp · trainer 都是本项目里分工不同的 AI 会话：cloudrun 管云机器，evalcmp 管评测。
它们之间只经大总管（总协调的会话）传话，所以「发给 traindata」指发给大总管转交。
日期按北京时间记；云上产物的文件名用 UTC，本机报告的文件名用本机时间（美西），同一件事可能差一天。

## 二、文件与正本

| 文件 | 管什么 | 正本 / 第二处表示怎么核 |
|---|---|---|
| `sources.json` | 要下载的每个文件：地址 · 字节数 · 上游 md5 · 许可 | 正本。`fetch.py --head` 核服务器报的大小 |
| `fetched.json` | 第一次在云上下完记下的 sha256（上游没给的那些） | 云上生成后进仓 |
| `split_rules.json` | 切分比例与单位 | 是 027 `decision.md` 第六节表的第二处表示；`test_split_leak.Rules` 逐行比对 |
| `fetch.py` | 断点续传下载、核大小与哈希；限流（429 / 503）单独计次，按 `Retry-After` 等 | `test_fetch` |
| `split.py` | 按规则切分；OpenSLR 28 的房间从上游 `rir_list` 读 | `test_split_leak` · `test_openslr28_rooms` · `test_tum_units` · `test_products` |
| `filecache.py` | 云上逐文件读数缓存（sha256 · 帧数 · 采样率），放 `/workspace/cache/split-files.jsonl`，不进仓 | 按（相对路径 · 字节数 · 修改时间）认；变了就重算（`test_filecache`） |
| `splits/<来源>.jsonl` + `.json` | 每个文件进哪一份、sha256、段界。正文超 8 MB 时存成 `.jsonl.gz` | 云上生成后进仓；摘要里记解压后正文的哈希，读时核（`test_split_gzip`） |
| `recipe.json` | 混音旋钮、默认值、出处 | 正本。早期混响长度**不在这里**：正本是 exp013 `rooms.py` 的 `EARLY_REVERB_MS` 与 `early_rir` |
| `mix.py` | `plan`（读片段 + 抽随机数）与 `render`（CPU 参考版） | CPU 版是混音算法的正本 |
| `gpu_mix.py` | 同一套步骤的 torch 批量版（GPU） | `test_gpu_mix` 对拍 CPU 版：逐样本差 ≤ 1e-4；电平与有声区间逐个对拍 exp013 `dsp.py` |
| `leakcheck.py` | 训练原料和 exp013 评测素材求交集 | 正本是 exp013 build `20261103230756-17f5` 的 `exclude.json`；测试核它等于 exp013 `leak.exclude_table` 的输出 |
| `shard.py` | 转 16 kHz、打 tar 分片、写成员偏移索引 | — |
| `loader.py` | 给 trainer 的接口；登记训练 build | `builds/<id>/build.json` 进仓 |
| `exp013_inputs.py` · `exp013_inputs.json` | exp013 的 1,994 个原料逐文件 sha256 | 文件清单和 exp013 `pools.index_hash()` 比对，也和旧 build 的 `material_index` 比对 |
| `rebuild_exp013.py` | 核 exp013 评测素材能重现（现行判据见 6.5） | 报告与云上日志写 `rebuild/exp013-<时间>*`，进仓 |
| `cloud_steps.sh` · `requirements.txt` | 云上的操作步骤与钉死的版本 | 版本与 exp013 旧 build 一致（`test_rebuild`） |
| （读）`../cloud/config.py` | 网络盘配额 `VOLUME_SIZE_GB` 与挂载点 `VOLUME_MOUNT` | 正本归 cloudrun；`fetch.py` 按路径加载，不抄数字 |

给三个缺口场景（飞机舱 · 会议室 · 电话亭）找候选噪声时，`screen.py` 把每条片段过一遍机器筛查，判 `keep` / `drop` 并写下理由（有人声 · 不像目标场景 · 混响时间不符 · 不平稳），没查的项记「未查：…」。
判据只有一份正本 `screen_rules.json`：阈值全在里面，代码里不写数字；改判据只改这份 json。
片段要对上 exp013 `scenes.json` 的场景名，所以 `screen_rules.json` 的键也用它（`tests/test_screen.py` 核对）。
两个神经模型（Silero VAD · PANNs CNN14）不在导入时加载、也没进 `requirements.txt`：云上 `pip install silero-vad panns-inference` 后才有，本机测试用假函数替；`--no-models` 空跑只算声学量。
新增的文件就是这三个：`screen_rules.json` · `screen.py` · `tests/test_screen.py`。

## 三、切分与隔离

切分规则照 027 第六节，分四份：
- 训练：拿来训练模型
- 开发：训练中途看效果、挑存档点
- 终测原料池：以后合成正式终测题目用，训练时不碰
- 测试份（D63）：只用来测，不进任何合成原料

按「单位」切，不按单个文件切。同一个说话人、同一间房的录音彼此很像；如果一部分进训练、一部分进测试，模型等于见过考题，分数会虚高（这叫泄漏）。
切法是确定的：每个单位（说话人、房间或文件）按 `sha256(种子:来源:单位)` 排序，再按比例分。
分配用最大余数法，所以各份个数加起来正好等于总数。

### OpenSLR 28 实测 RIR 的房间怎么认（2026-11-25）

RIR（房间脉冲响应）是在一间房里放一声短脉冲、在某个位置录下的回声。混音时把干净人声和它做卷积，就得到「在那间房里说话」的声音。
同一间房换个喇叭或麦克风位置（收发位置）再测一次，又是一条 RIR，但房间的混响特征相同，所以算同一个单位。
OpenSLR 28 是 Kaldi 工具包配的 RIR 库：有用程序模拟出来的，也有在真房间里实测的。
027 要求同一间房的不同收发位置只进一份。房间号取自上游包内的 Kaldi `rir_list`（`--room-id`）。
实测 RIR 有三个库，上游 README 只说「Overall there are 325 real RIRs」，**没有给 room-id 和真实房间的对照**。
所以只在 room-id 本身看得出是房间名时照用，看不出就整库归一个单位：

| 库 | 条数 | room-id 长什么样 | 怎么切 |
|---|---|---|---|
| REVERB 2014 | 36 | 9 个房间名（`RVB2014_largeroom1` …），每间 4 条 = 2 个距离 × 2 个角度 | 照用，每间一个单位 |
| AIR | 107 | 35 个 id：同一间房按位置、角度、有无假人头拆开（`aula_carolina_1_1_90`、`aula_carolina_1_3_0`；`lecture_0`、`lecture_1`），手机测的又是另一套（`air_phone_lecture`） | 整库一个单位 |
| RWCP | 182 | 阵列与设置代码（`circle_ane`、`cirline_e2a` …），看不出哪几个是同一间房；type4 的 38 条直接把文件名写成了 room-id（`RWCP_000l.wav`） | 整库一个单位 |

整库归一个单位的代价：这两个库各自只进一份（按 `split_rules.json` 里固定的种子 20261116 算，AIR 进终测原料池，RWCP 进训练），开发份与测试份里没有它们。
不这么做的风险更大：同一间房的 RIR 可能同时进训练和测试份。把几个单位并成一个，只会让单位更粗，不会跨份。
规则的正本是 `split_rules.json` 里 `openslr28-rir` 的 `room_merge`（每条带 `why`）。上游 4 份 `rir_list` 的 sha256 钉在同一处，上游一变就停（`test_openslr28_rooms`）。
这些都是 2026-11-25 按 HTTP Range 只读上游 zip 的目录与 `rir_list` 核的，没下音频。模拟 RIR 的 600 间房，每间一个 room-id，照用。

交集检查查的是训练、开发、终测原料池三份里能取到的**全部**原料，不只是某一次抽到的那些。
检查对照 exp013 的 `exclude.json`，查这几类：说话人 · DEMAND 时段 · ESC-50 · 实测 RIR · 房间噪声 · 中文与音乐 · 合成函数。
exclude.json 里只要出现不认识的键，检查就直接红。

## 四、给 trainer 的接口（2026-11-16 与 trainer 商定，2026-11-24 改成在 GPU 上混音）

```python
from loader import torch_iterable, render_on_gpu, DevSet
it = torch_iterable(shards, "recipe.json", split="train", seed=0, seg_s=4.0, batch_size=32, start_batch=0)
dl = DataLoader(it, batch_size=None, num_workers=N)      # worker 只读片段 + 抽随机数（mix.plan）
for b in dl:
    out = render_on_gpu(b, "cuda")                       # 在 GPU 上批量混音（gpu_mix.render_batch）
    out["noisy"], out["clean"]                           # [B, T] float32，在 GPU 上
    out["replay"], out["near_silent"]                    # bool [B]
    out["owner_ivs"], out["interf_ivs"]                  # 每段一个区间 list，样本下标，左闭右开
it.build_id                                              # 写进训练运行记录
```

- 第 k 批只由 (seed, k, batch_size) 决定，和 num_workers 无关。从第 k 批续训时传 `start_batch=k`；如果换了 batch_size，批的内容会变
- `clean` 只留直达声加前 100 ms 早期混响（`rooms.early_rir`）。均衡同时作用在 `clean` 上；削波只作用在 `noisy` 上
- `owner_ivs` 用 exp013 `dsp.intervals` 算（和评测真值同一个正本）。算的时机：混响、均衡、电平调整之后，削波之前。区间换成 10 ms 掩码的那一步由 trainer 做
- 第一批配方里没有能听清的旁人说话（D89），所以 `interf_ivs` 一律为空
- `render="cpu"` 走旧路，在 worker 里直接用 CPU 渲染（对拍和没有显卡时冒烟用）。`DevSet(shards, recipe, n=2000)[i]` 是 CPU 渲染，结果固定

## 五、混音旋钮（数据消融时一次只动一个）

旋钮的默认值和出处见 `recipe.json`。所有旋钮：目标场景 · 回放 · 信噪比 · 电平 · 随机均衡 · 削波 · 噪声段数 · 近静音段 · 混响。
**目标场景**默认为空，等缺口表（014 第 1 步）出来再定。
**近静音段**默认关：只在 trainer 配了防误压损失时才开（Taherian 2023 p.4）。
官方 SEtrain 混音代码里 `alpha = 10**(-snr*1.5/20)`，所以它的实际信噪比是名义值的 1.5 倍（读源码得知）。本配方按真实信噪比取值。

## 六、云上步骤（RunPod，网络盘 `/workspace`）

RunPod 是租云机器的服务；网络盘是挂在云机器上的共享硬盘，换机器数据还在。
所有步骤用法相同：`bash cloud_steps.sh <步骤>`。下载（`setup` · `head` · `fetch` · `fetch-e3`）已由 cloudrun 在 2026-11-24 跑完。
第一批 21 个包和 exp013 原料里的 3 个维基共享资源文件（E3）都已核过，读数在 `fetched.json`。
`cloud_steps.sh rebuild`（从官方源重下原料、整套重建 exp013）**现在不跑**，6.5 的核对代替了它。

### 6.1 切分 · 分片 · 登记（2026-11-24 大总管放行，CPU 机即可）

前提：
- 机器挂的是同一块网络盘（在 US-MO-1；`/workspace/train-raw` 与 `/workspace/vdn-data` 都在）
- 代码是本分支 `exp/025-train-data` 的最新提交（大总管先推送）；放在 `/workspace/repo` 以外的新目录，别覆盖旧 checkout

按顺序跑，任何一步退出码非 0 就停下，把终端输出发给 traindata：

```bash
cd <新 checkout>/docs/experiments/exp025-gtcrn-train/data
bash cloud_steps.sh setup       # 建 .venv，装 requirements.txt；机器上没有 uv 就由 setup_uv.sh 装钉死版本到 /workspace/bin（核官方 sha256，不用 pip）
bash cloud_steps.sh inventory   # 写 inventory.txt：每个来源的文件数与前 20 个路径
bash cloud_steps.sh split       # 写 splits/<来源>.jsonl + .json，共 6 个来源 12 个文件
bash cloud_steps.sh shard       # 写 /workspace/shards/first-batch/*.tar 与 index.json（目录已存在就拒绝）
bash cloud_steps.sh register    # 交集检查，写 builds/<id>/build.json；检查不过退出码非 0
bash cloud_steps.sh pack        # 打包 inventory.txt + splits/ + builds/ 到 /workspace/out/，打印 sha256
```

`test` 一步可跑可不跑。cloudrun 的分支合进来之前，已知只有 `test_real_cloud_config_quota` 会红（缺 `cloud/config.py`）。

### 6.2 预期产物

| 步骤 | 产物 | 预期（出处） |
|---|---|---|
| inventory | `inventory.txt` | 每个来源一段：文件数 + 前 20 个路径 |
| split | `splits/` 12 个文件 | 文件数照 `split_rules.json` 的 `expect`：train-clean-100 28,539 句 251 人 → 训练 201 · 开发 25 · 测试份 25 人；dev-clean + test-other 5,642 句 73 人 → 终测 37 · 测试份 36 人；QUT 20 段；DEMAND 96 个（6 环境 × 16 声道）；OpenSLR 28 共 60,325 个 = 600 间模拟房 + 11 个实测单位（实测单位分到训练 · 开发 · 终测 · 测试份各 8 · 1 · 1 · 1 个，2026-11-25 实测）；TUM 478 个 = 100 个合成文件 + 32 次手机录音（上游数目见各条 `src`） |
| shard | 分片目录 | 只有训练份与开发份；估计十几 GB（推断：人声约 90 小时 × 16 kHz × 2 字节 ≈ 10 GB，另加噪声与 RIR），`/workspace` 已用 33 GB，配额 150 GB |
| register | `builds/<id>/build.json` | `valid: true`，`leak_check.ok: true`；`dropped_rir` 是被 exp013 `rirs.Measured.valid` 剔掉的 RIR 数：直达声之前有明显能量、混响时长估不准、或尾巴衰减不到 −30 dB 的不用 |
| pack | `/workspace/out/traindata-products-<时间>.tar.gz` | 几 MB；不含音频 |

耗时：2026-11-25 那次 split 跑了 1800 s 才在 OpenSLR 28 上停下。现在改成先只看路径认出每个文件的单位，认不出几秒内就停；逐文件读数（sha256 · 帧数 · 采样率）缓存在 `/workspace/cache/split-files.jsonl`，重跑只算变了的文件（缓存按数据根 · 相对路径 · 字节数 · 修改时间认；2026-11-25 加了数据根，此前写的缓存行不再认，下次全算一遍）。第一次跑仍要全算：2026-11-25 云上实测 1346 s（4 核 CPU 机）。别给它设超时（前一次 cloudrun 设了 30 分钟上限）。下载阶段的哈希复用不了：那是整个压缩包的哈希，包里的文件没有单独的哈希。

### 6.3 出错时怎么办

- `split` 报「文件 N 个，应为 M」：通配符或上游数目不对，不写表。把 `inventory.txt` 发给 traindata，由 traindata 改 `split_rules.json` 后再跑
- `split` 报「不在任何 rir_list 里」「和规则核过的上游版本不同」「认不出路径与房间号」：OpenSLR 28 的房间表没对上，同样发清单
- `split` 中途被杀：直接重跑，已算的读数从缓存里取
- `shard` 报目录已存在：不删旧目录，设 `VDN_SHARDS=/workspace/shards/first-batch-<时间>` 换个新目录再跑
- `register` 报交集检查不过：`build.json` 仍会写出并记作废，照样 `pack` 带回
- 改了规则要重切：`split.write` 拒绝覆盖已有的表，而 `split` · `shard` · `register` 三步都用同一个 `splits/` 目录，现在没有换目录的参数。这条路还没打通（待开 issue），遇到时先找大总管

### 6.4 带回后的核对（traindata 做）

1. 核包的 sha256 与 cloudrun 报的一致，解到 `data/` 下（`inventory.txt` · `splits/` · `builds/`）
2. 人工读 `inventory.txt`：QUT 与 TUM 匹配到的确实是录音本身，没有说明文件或脉冲响应
3. 跑 `python -m unittest tests.test_products`。它核这几项：
   - 6 个来源都在
   - 规则哈希等于现行 `split_rules.json`，文件数符合 `expect`
   - 单位列和规则对得上（说话人、按文件切的从路径整个重算；房间核 `room_merge`）
   - 按单位切的重算一遍分配，逐个相同，同一单位不跨份
   - 按时段切的段界重算相同
   - build 有效、交集检查通过；它记的切分表哈希与仓里相同
   - 只有训练份与开发份进分片
4. 测试全过后提交（hook 自动做）。发现问题时，按 6.3 改规则重切，旧表不原地覆盖（`split.write` 会拒绝）

### 6.5 exp013 评测素材能否重现：改核本机 6323 加云上记录（2026-11-25 大总管定）

为什么在这里核：exp013 是评测用的那套场景音频。训练数据要和它求交集（第三节），云上评测也要用它，所以要确认云上那份和本机正式版是同一份。
涉及三个 build（一次生成的一整套音频，编号是生成时刻加短哈希）：
- `20261103230756-17f5`：旧正式版，当初的 4540 个文件哈希记在仓里
- `20261124165427-6323`：本机用 main 现行代码重新生成的，多了 480 条早期混响轨，共 5020 个文件
- `20261125022300-aa9f`：云上用同样代码生成的

云上 x86 和本机 arm64 的浮点运算不同，`aa9f` 和 `6323` 的样本差约 4e-13，做不到逐位相同（见 exp013 的 `crosscheck-20261125.md`）。
所以不再要求「云上重建的 4540 个文件逐个同 17f5」。云上评测改用本机上传的 6323 音频，判据改成四项：

1. 本机 6323 的旧 4540 个文件逐个同 17f5
2. 云上的 6323 音频按 6323 的 `hashes.txt` 全部对上，5020 个。日志里的路径要带 6323 的编号；拿 aa9f 自己的 `hashes.txt` 核出来的不算
3. 云上 `aa9f` 的 `exclude.json`（训练要避开的评测素材清单）与依赖版本同 17f5，代码与参数哈希同 6323；`--cloud` 和 `--local` 不许指向同一个 build；两份的文件清单要相同（同代码独立生成，防本机清单漏行）
4. 云上原料 1,994 个的内容对上（build 自带的素材索引只记路径与字节数）

「官方源可复现」（从各数据集官网重新下载原料，也能生成同样的音频）这次**未验证**：云上原料是本机上传的，只证明与本机原料逐个相同。报告里照实写。

cloudrun 在同一台云机器上做两件事（`python3` 指装了钉死依赖的环境：Python 3.12.14 · numpy 2.5.4 · scipy 1.18.1 · soundfile 0.14.0，即 `cloud_steps.sh setup` 装的 `.venv` 或按 main 的 `requirements.lock` 装的环境；系统自带的 python3 没有 soundfile）：
1. 在云上仓库根下，让 `_tmp/exp013/20261124165427-6323` 指向上传的 6323 音频目录，然后跑
   `sha256sum -c docs/experiments/exp013-scene-suite/builds/20261124165427-6323/hashes.txt > upload-verify.log`（每行「路径: OK」）
2. 在云上仓库根下跑
   `python3 docs/experiments/exp025-gtcrn-train/data/exp013_inputs.py --verify --data <数据根>/vdn-data | tee inputs-verify.log`（要「原料全对」）

两份日志带回本机后，traindata 把它们放进 `rebuild/`，改名为 `exp013-<时间>-upload-verify.log` 与 `exp013-<时间>-inputs-verify.log`。
然后在本目录（`docs/experiments/exp025-gtcrn-train/data`）下跑：
`python3 rebuild_exp013.py --check-cloud --local ../../exp013-scene-suite/builds/20261124165427-6323 --cloud ../../exp013-scene-suite/builds/20261125022300-aa9f --upload-log rebuild/<上传日志> --inputs-log rebuild/<原料日志>`
`aa9f` 的小文件 evalcmp 已经放进仓里。报告写进 `rebuild/` 并提交；有了 ok 的报告，`test_rebuild.test_code_drift_covered_by_report` 才变绿。
现行报告：`rebuild/exp013-20261124-200416.json`（判据第 4 版，四项全过）。旧报告留作记录、不再算数：`195551` 是第 3 版（本机 hashes.txt 和日志同时漏行也能过，codex 第 4 轮发现；第 4 版加了「本机与云上的文件清单相同」）；`192451` 是第 1 版（认不出 aa9f 的日志，自审发现），`194001` 是第 2 版（`--cloud` 误填成本机那份也能过，codex 第 2 轮发现）。

## 七、已知局限

- QUT-NOISE 和 TUM 风噪：包内结构已按云上清单与上游 zip 目录核过（2026-11-25）。TUM 手机录音的文件名前缀按一次录音算，上游没写前缀含义；按前缀并只会让单位更粗。QUT 的服务器没有声明支持断点续传，遇到时从头重下
- OpenSLR 28：包里的点声源噪声取自 MUSAN，里面各个来源的许可没有逐个核，所以第一批不用
- 测试状态（2026-11-25 产物带回后）：本机共 128 条，127 条过，1 条红：缺 cloud/config.py。exp013 核对报告见 `rebuild/exp013-20261124-200416.json`（6.5 四项全过，官方源可复现未验证）。evalcmp 已合进 main，混音测试用 main 上的 rooms.py（torch 用 CPU 后端）。云上 GPU 对拍还没跑
- 2026-11-25 codex 评审后，build_id 多算了分片索引本身和 gpu_mix · leakcheck · split 三个文件，登记时交集检查也改成查规则里全部 6 个来源（以前漏了不进分片的终测人声来源；本机补查结果仍是 0 处交集）。codex 第 2 轮后 build_id 又加了 mix.py 直接调的 exp013 `dsp.py` · `rooms.py` · `rirs.py`。所以下面这个 build 和现行代码算出的 id 对不上。2026-11-25 cloudrun 用 main 86698924 重跑了 `register` 与 `pack`（没重切、没重分片，register 89 s）：**现行 build 是 `f032a75eedad80d6`**，交集检查查了 6 个来源 83,460 行，0 处交集；分片、切分表、各份条数都和旧 build 相同。旧 build `49349c158a4defdc` 的记录留着，loader 不再认它
- 第一批产物（build `49349c158a4defdc`，云上提交 8b4194d3）：分片 15 GB 与 `index.json` 30 MB 只在网络盘 `/workspace/shards/first-batch`，没取回本机。分片时按 `rirs.Measured.valid` 丢掉 3307 条 RIR（训练加开发份共 48214 条，约 6.9%），逐条名单在那份 `index.json` 的 `dropped` 里
- **旧分片只能用当时的提交复现**（读源码，2026-11-26）：loader 启动时按现行代码与配方重算 build id，对不上就拒绝。所以 `first-batch`（build `49349c158a4defdc` · `f032a75eedad80d6`）只能在当时的提交上读；第三批合进 main、配方一变，`batch2`（build `6173bf9ca6a6e143`）也一样。要重跑旧的冒烟或评测，先切到记录里的提交。删不删这些旧分片由人定
- 第二批（D106，2026-11-26）：加 CochlScene 3 类（地铁站 · 地铁车厢 · 拥挤室内），按录音人切，385 人 → 训练 347 人 / 16,193 个文件、开发 38 人 / 1,436 个文件。**现行 build 是 `6173bf9ca6a6e143`**（云上 fac82500，分片 `/workspace/shards/batch2`，6 个来源，噪声场景 11 个；cowork 与 open-office 成员相同，抽样并成一组）。第一轮 build `889a389708c8a1e9`（c0c9bda3，没有去重抽样）作废，不进仓，记录在主仓 `_tmp/20261126-cloudrun-batch2/`。CochlScene 未筛人声，方法待定；飞机舱暂由地铁车厢代替
- `splits/openslr28-rir.jsonl` 原文 16.4 MB，auto-commit hook 不收超 10 MB 的文件，所以仓里存的是 `openslr28-rir.jsonl.gz`（2.6 MB）。`split.write` 遇到正文超 8 MB 会自动这样存；`split.load_split` 两种都能读，核的始终是解压后正文的 sha256，所以云上出的摘要不用改
- 网络盘查盘：RunPod 网络盘上 df 报的是整个集群的剩余（cloudrun 实测 158 TB），所以在挂载点上改用「配额 − du 已用」。配额的正本是 `../cloud/config.py`（cloudrun 的分支）。那个分支合进来之前，`test_real_cloud_config_quota` 会红，下载也会拒绝开始
- `split_rules.json` 里有几条说明已过时：QUT 的 `_未核`、LibriSpeech 两条 `expect.src` 里的「待云上核」。云上已核过（见上面第一条与 6.2）。没改是因为说明字段也算进规则哈希，改一个字 6 张切分表就要重切；待把说明移出哈希（另开 issue）
