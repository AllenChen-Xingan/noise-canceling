# exp025 · data — training data for the GTCRN fine-tune

Download, split, and mix the training data on the cloud box; the Mac only runs code and unit tests (D88 · D97).
Owner: worker `traindata` (issue #156). Plan: `docs/plan/014_training-kickoff.md`, step 0 ③ and step 2.

## 一、这一节有什么用

这里写数据怎么下、怎么切、怎么混，以及 trainer 怎么取数据。每样东西的正本在哪个文件，见第二节。

## 二、文件与正本

| 文件 | 管什么 | 正本 / 第二处表示怎么核 |
|---|---|---|
| `sources.json` | 要下载的每个文件：地址 · 字节数 · 上游 md5 · 许可 | 正本。`fetch.py --head` 核服务器报的大小 |
| `fetched.json` | 第一次在云上下完记下的 sha256（上游没给的那些） | 云上生成后进仓 |
| `split_rules.json` | 切分比例与单位 | 是 027 `decision.md` 第六节表的第二处表示；`test_split_leak.Rules` 逐行比对 |
| `splits/<来源>.jsonl` + `.json` | 每个文件进哪一份、sha256、段界 | 云上生成后进仓；摘要里记 jsonl 的哈希，读时核 |
| `recipe.json` | 混音旋钮、默认值、出处 | 正本。早期混响长度**不在这里**：正本是 exp013 `rooms.py` 的 `EARLY_REVERB_MS` 与 `early_rir` |
| `mix.py` | `plan`（读片段 + 抽随机数）与 `render`（CPU 参考版） | CPU 版是混音算法的正本 |
| `gpu_mix.py` | 同一套步骤的 torch 批量版（GPU） | `test_gpu_mix` 对拍 CPU 版：逐样本差 ≤ 1e-4；电平与有声区间逐个对拍 exp013 `dsp.py` |
| `leakcheck.py` | 训练原料和 exp013 评测素材求交集 | 正本是 exp013 build `20261103230756-17f5` 的 `exclude.json`；测试核它等于 exp013 `leak.exclude_table` 的输出 |
| `shard.py` | 转 16 kHz、打 tar 分片、写成员偏移索引 | — |
| `loader.py` | 给 trainer 的接口；登记训练 build | `builds/<id>/build.json` 进仓 |
| `exp013_inputs.py` · `exp013_inputs.json` | exp013 的 1,994 个原料逐文件 sha256 | 文件清单和 exp013 `pools.index_hash()` 比对，也和旧 build 的 `material_index` 比对 |
| `rebuild_exp013.py` | 云上重建 exp013，逐文件核 4,540 个旧哈希 | 报告写 `rebuild/exp013-<时间>.json` |
| `cloud_steps.sh` · `requirements.txt` | 云上的操作步骤与钉死的版本 | 版本与 exp013 旧 build 一致（`test_rebuild`） |
| （读）`../cloud/config.py` | 网络盘配额 `VOLUME_SIZE_GB` 与挂载点 `VOLUME_MOUNT` | 正本归 cloudrun；`fetch.py` 按路径加载，不抄数字 |

## 三、切分与隔离

切分规则照 027 第六节，分四份：训练 · 开发 · 终测原料池 · D63 测试份。
测试份只用来测，不进任何合成原料。
切法是确定的：每个单位（说话人、房间或文件）按 `sha256(种子:来源:单位)` 排序，再按比例分。
分配用最大余数法，所以各份个数加起来正好等于总数。

### OpenSLR 28 实测 RIR 的房间怎么认（2026-11-25）

027 要求同一间房的不同收发位置只进一份。房间号取自上游包内的 Kaldi `rir_list`（`--room-id`）。
实测 RIR 有三个库，上游 README 只说「Overall there are 325 real RIRs」，**没有给 room-id 和真实房间的对照**。
所以只在 room-id 本身看得出是房间名时照用，看不出就整库归一个单位：

| 库 | 条数 | room-id 长什么样 | 怎么切 |
|---|---|---|---|
| REVERB 2014 | 36 | 9 个房间名（`RVB2014_largeroom1` …），每间 4 条 = 2 个距离 × 2 个角度 | 照用，每间一个单位 |
| AIR | 107 | 35 个 id：同一间房按位置、角度、有无假人头拆开（`aula_carolina_1_1_90`、`aula_carolina_1_3_0`；`lecture_0`、`lecture_1`），手机测的又是另一套（`air_phone_lecture`） | 整库一个单位 |
| RWCP | 182 | 阵列与设置代码（`circle_ane`、`cirline_e2a` …），看不出哪几个是同一间房；type4 的 38 条直接把文件名写成了 room-id（`RWCP_000l.wav`） | 整库一个单位 |

整库归一个单位的代价：这两个库各自只进一份（按种子算，AIR 进终测原料池，RWCP 进训练），开发份与测试份里没有它们。
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

所有步骤用法相同：`bash cloud_steps.sh <步骤>`。下载（`setup` · `head` · `fetch` · `fetch-e3`）已由 cloudrun 在 2026-11-24 跑完。
第一批 21 个包、E3 3 个文件都已核过，读数在 `fetched.json`。`rebuild` 是 exp013 重建，另行放行。

### 6.1 切分 · 分片 · 登记（2026-11-24 大总管放行，CPU 机即可）

前提：
- 机器挂的是同一块网络盘（在 US-MO-1；`/workspace/train-raw` 与 `/workspace/vdn-data` 都在）
- 代码是本分支 `exp/025-train-data` 的最新提交（大总管先推送）；放在 `/workspace/repo` 以外的新目录，别覆盖旧 checkout
- `register` 不依赖 evalcmp 的 `rooms.early_rir`（导入 loader 不再连带导入 mix，`test_products` 核）

按顺序跑，任何一步退出码非 0 就停下，把终端输出发给 traindata：

```bash
cd <新 checkout>/docs/experiments/exp025-gtcrn-train/data
bash cloud_steps.sh setup       # 建 .venv，装 requirements.txt
bash cloud_steps.sh inventory   # 写 inventory.txt：每个来源的文件数与前 20 个路径
bash cloud_steps.sh split       # 写 splits/<来源>.jsonl + .json，共 6 个来源 12 个文件
bash cloud_steps.sh shard       # 写 /workspace/shards/first-batch/*.tar 与 index.json（目录已存在就拒绝）
bash cloud_steps.sh register    # 交集检查，写 builds/<id>/build.json；检查不过退出码非 0
bash cloud_steps.sh pack        # 打包 inventory.txt + splits/ + builds/ 到 /workspace/out/，打印 sha256
```

`test` 一步可跑可不跑。在 evalcmp 与 cloudrun 的分支合进来之前，已知会红：混音测试（缺 `rooms.early_rir`）和 `test_real_cloud_config_quota`（缺 `cloud/config.py`）。

### 6.2 预期产物

| 步骤 | 产物 | 预期（出处） |
|---|---|---|
| inventory | `inventory.txt` | 每个来源一段：文件数 + 前 20 个路径 |
| split | `splits/` 12 个文件 | 文件数照 `split_rules.json` 的 `expect`：train-clean-100 28,539 句 251 人 → 训练 201 · 开发 25 · 测试份 25 人；dev-clean + test-other 5,642 句 73 人 → 终测 37 · 测试份 36 人；QUT 20 段；DEMAND 96 个（6 环境 × 16 声道）；OpenSLR 28 共 60,325 个 = 600 间模拟房 + 11 个实测单位（实测约 8 · 1 · 1 · 1）；TUM 478 个 = 100 个合成文件 + 32 次手机录音（上游数目见各条 `src`） |
| shard | 分片目录 | 只有训练份与开发份；估计十几 GB（推断：人声约 90 小时 × 16 kHz × 2 字节 ≈ 10 GB，另加噪声与 RIR），`/workspace` 已用 33 GB，配额 150 GB |
| register | `builds/<id>/build.json` | `valid: true`，`leak_check.ok: true`；`dropped_rir` 是被 `rirs.Measured.valid` 剔掉的 RIR 数 |
| pack | `/workspace/out/traindata-products-<时间>.tar.gz` | 几 MB；不含音频 |

耗时：2026-11-25 那次 split 跑了 1800 s 才在 OpenSLR 28 上停下。现在改成先只看路径认出每个文件的单位，认不出几秒内就停；逐文件读数（sha256 · 帧数 · 采样率）缓存在 `/workspace/cache/split-files.jsonl`，重跑只算变了的文件。第一次跑仍要全算，用时未量化，别给 split 设 30 分钟上限。下载阶段的哈希复用不了：那是整个压缩包的哈希，包里的文件没有单独的哈希。

### 6.3 出错时怎么办

- `split` 报「文件 N 个，应为 M」：通配符或上游数目不对，不写表。把 `inventory.txt` 发给 traindata，由 traindata 改 `split_rules.json` 后再跑
- `split` 报「不在任何 rir_list 里」「和规则核过的上游版本不同」「认不出路径与房间号」：OpenSLR 28 的房间表没对上，同样发清单
- `split` 中途被杀：直接重跑，已算的读数从缓存里取
- `shard` 报目录已存在：不删旧目录，设 `VDN_SHARDS=/workspace/shards/first-batch-<时间>` 换个新目录再跑
- `register` 报交集检查不过：`build.json` 仍会写出并记作废，照样 `pack` 带回

### 6.4 带回后的核对（traindata 做）

1. 核包的 sha256 与 cloudrun 报的一致，解到 `data/` 下（`inventory.txt` · `splits/` · `builds/`）
2. 人工读 `inventory.txt`：QUT 与 TUM 匹配到的确实是录音本身，没有说明文件或脉冲响应
3. 跑 `python -m unittest tests.test_products`。它核这几项：
   - 6 个来源都在
   - 规则哈希等于现行 `split_rules.json`，文件数符合 `expect`
   - 按单位切的重算一遍，分配逐个相同，同一单位不跨份
   - 按时段切的段界重算相同
   - build 有效、交集检查通过；它记的切分表哈希与仓里相同
   - 只有训练份与开发份进分片
4. 测试全过后提交（hook 自动做）。发现问题时，按 6.3 改规则重切，旧表不原地覆盖（`split.write` 会拒绝）

### 6.5 exp013 重建：直接核云上 main 出的那份 build（2026-11-25 大总管定，不另跑 rebuild）

云上用 main 复刻 `20261124165427-6323` 时，cloudrun 在**同一台机器**上多做两件事：

1. 出完 build 后，在仓库根下跑原料内容核对（build 自带的素材索引只记路径与字节数，不读内容）：
   `python3 docs/experiments/exp025-gtcrn-train/data/exp013_inputs.py --verify --data <数据根>/vdn-data | tee inputs-verify.log`
   要求输出「原料全对」、退出码 0。这个脚本在本分支上，main 上还没有
2. 带回新 build 目录里的 4 个小文件（不含音频）：`manifest.json` · `hashes.txt` · `exclude.json` · `clips.jsonl`，加上 `inputs-verify.log`

带回后 traindata 在本机跑：
`python3 rebuild_exp013.py --check-dir <带回的 build 目录> --inputs-log inputs-verify.log`
它核四项：旧 4540 个文件逐个相同、`exclude.json` 全等、版本相同、原料全对。报告写进 `rebuild/` 并提交。
有了这份报告，`test_rebuild.test_code_drift_covered_by_report` 才会变绿。

## 七、已知局限

- QUT-NOISE 和 TUM 风噪：包内结构已按云上清单与上游 zip 目录核过（2026-11-25）。TUM 手机录音的文件名前缀按一次录音算，上游没写前缀含义；按前缀并只会让单位更粗。QUT 的服务器没有声明支持断点续传，遇到时从头重下
- OpenSLR 28：包里的点声源噪声取自 MUSAN，里面各个来源的许可没有逐个核，所以第一批不用
- 测试状态（2026-11-25 产物带回后）：本机共 127 条，125 条过，2 条红：缺 cloud/config.py；exp013 代码变了、还没有云上核对报告（见 6.5）。evalcmp 已合进 main，混音测试用 main 上的 rooms.py（torch 用 CPU 后端）。云上 GPU 对拍还没跑
- 第一批产物（build `49349c158a4defdc`，云上提交 8b4194d3）：分片 15 GB 与 `index.json` 30 MB 只在网络盘 `/workspace/shards/first-batch`，没取回本机。分片时按 `rirs.Measured.valid` 丢掉 3307 条 RIR（训练加开发份共 48214 条，约 6.9%），逐条名单在那份 `index.json` 的 `dropped` 里
- `splits/openslr28-rir.jsonl` 原文 16.4 MB，auto-commit hook 不收超 10 MB 的文件，所以仓里存的是 `openslr28-rir.jsonl.gz`（2.6 MB）。`split.write` 遇到正文超 8 MB 会自动这样存；`split.load_split` 两种都能读，核的始终是解压后正文的 sha256，所以云上出的摘要不用改
- 网络盘查盘：RunPod 网络盘上 df 报的是整个集群的剩余（cloudrun 实测 158 TB），所以在挂载点上改用「配额 − du 已用」。配额的正本是 `../cloud/config.py`（cloudrun 的分支）。那个分支合进来之前，`test_real_cloud_config_quota` 会红，下载也会拒绝开始
- 依赖：测试要用 evalcmp 在分支 `feat/eval-paired-compare` 上加进 exp013 `rooms.py` 的 `early_rir`。那个分支合进 main 之前，在本分支上跑会红
