# exp025 · 原版 GTCRN 的不回混基线成绩单与场景差距表

> 计划 014；大总管 noise-canceling-df 2026-11-25 交代。手下 evalcmp 写，管的实体：成绩单 · 评测集。
> **要回答**：原版 GTCRN 不回混时，15 个开会场景里哪几个最弱？临时选定的四个加训场景（街道 street-outdoor · 车站 transit-hub · 车内 car-passenger · 咖啡馆 cafe）对不对？
> 那四个来自 exp016 的旧表，旧表是**回混口径**：引擎在模型后面把原声混回去，说话时约 −6 dB、停顿时约 −12 dB，模型自己的差被压掉约一半（exp016 `findings.md` 第 337 行，推断）。
> **结论要等出数以后**由大总管下；本文只写跑法、分工、估时、验收和判据。

## 〇、口径与评测集

- **不回混** = 关掉引擎的保护处理（混回原声、超幅保护、坏值保护都关），打的是 GTCRN 自己的输出。命令行 `vdn-eval scorecard --no-safety`（本线新加，把全部任务的 `safety` 设为假；正本 `src/eval/src/suite.rs` 的 `Suite::without_safety`）。exp021 已核这条路：预混 + 关保护与原路径逐段相同（`docs/experiments/exp021-gtcrn-aux/README.md` 「核对」一节）
- **评测集**：exp013 build `20261124165427-6323`（480 段，15 个场景 × 常见 / 压力两档 × 两种设备），任务清单 `src/eval/suite/scenes.json`，不改。云上也读这一份音频（大总管定，见 `docs/experiments/exp013-scene-suite/crosscheck-20261125.md`）
- **规则**：成绩单新规则 v2（默认），规约 `specs/rubric/scorecard.md`

## 一、跑法

### 1. GTCRN 原版权重（两边相同）

- 来源：上游 `https://github.com/Xiaobin-Rong/gtcrn`，提交 `502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73`，文件 `stream/onnx_models/gtcrn_simple.onnx`（MIT）
- 身份：535 190 字节，sha256 `b4718df6228e7bdf1a8a435cf98f838636eb2fd331acabf86ba87c5192ebcb87`（本机实算，与调研 008 记录相同）
- 用法：`export VDN_GTCRN_ONNX=<路径>`，跑之前先 `shasum -a 256 "$VDN_GTCRN_ONNX"`，不等就停

```sh
git clone https://github.com/Xiaobin-Rong/gtcrn /workspace/gtcrn && git -C /workspace/gtcrn checkout 502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73
export VDN_GTCRN_ONNX=/workspace/gtcrn/stream/onnx_models/gtcrn_simple.onnx
echo "b4718df6228e7bdf1a8a435cf98f838636eb2fd331acabf86ba87c5192ebcb87  $VDN_GTCRN_ONNX" | shasum -a 256 -c
```

### 2. 评测集音频（云上）

`vdn-eval` 按「仓库根 + 清单里的相对路径 `_tmp/exp013/20261124165427-6323/…`」找音频，`scorecard` 没有指定音频根的参数。所以在云上**仓库根**下建链接，再按清单核一遍：

```sh
cd <云上仓库根>
mkdir -p _tmp/exp013 && ln -s /workspace/exp013-builds/20261124165427-6323 _tmp/exp013/20261124165427-6323
shasum -a 256 -c --quiet docs/experiments/exp013-scene-suite/builds/20261124165427-6323/hashes.txt \
  || { echo '音频核对不过：停下，不要打分'; exit 1; }   # 只看退出码：清单读不了、有文件不符都会非 0（Linux 用 sha256sum -c --quiet）
```

本机音频在主目录 `_tmp/exp013/20261124165427-6323/`（5020 个文件，2026-11-25 按清单全部核过）。

### 3. 两个评分模型（`--qa` 要用；两边相同）

| 项 | 环境变量 | 默认位置（家目录下） | sha256（正本 `src/eval/src/qa/nrmos.rs`） |
|---|---|---|---|
| SCOREQ（V-NRMOS-SCOREQ） | `VDN_SCOREQ_ONNX` | `.cache/scoreq/onnx-models/adapt_nr_telephone.onnx`（378 MB） | `d493ba8a…00b4` |
| Distill-MOS（V-NRMOS-DISTILL） | `VDN_DISTILLMOS_ONNX` | `vdn-data/models/distillmos/distill_mos_v7-seg122880.onnx`（17.6 MB） | `b8e2dc18…2928` |

`vdn-eval` 载入前自己核哈希，对不上就不用（该项判「不知道」）。云上从本机传这两个文件最省事（共约 396 MB），不另下载、不另导出。

### 4. 命令

```sh
cargo build --release -p vdn-eval          # ort 会在编译时下载对应平台的预编译库，云上编译要能联网
RUN=$(date +%Y%m%d-%H%M%S)-<机器名>
caffeinate -i target/release/vdn-eval scorecard --system gtcrn --no-safety --qa \
  --out docs/experiments/exp025-gtcrn-train/baseline/runs/$RUN       # 云上没有 caffeinate 就去掉
```

- 识别默认走 OpenRouter（D103）：密钥放仓库根 `.env.local` 的 `OPENROUTER_API_KEY`（或同名环境变量），云上同样放一份，权限 600、不进仓
- 成绩单 `meta` 里应看到 `"safety": "off"`、`"qa": true`；`"asr"` 是一个对象：`default` = `openrouter`、`backends.openrouter.model` = `qwen/qwen3-asr-1.7b-20260813`、各后端实际用到的段数、`checked` / `unstable`（多次转写的条数与不一致的条数）。装不起来时是 `{"error": 原因}`，吃字整列判「不知道」
- 输出：`scorecard.md` · `scorecard.json` · `clips.jsonl`；音频类中间产物（`qa/asr-wav/`）不进仓
- 识别花费累计在主目录 `_tmp/asr-openrouter-ledger.json`（全部真调用共用一本），到 15 美元就停发（第三节）
- 按段换后端（可选）：先把路由写成一个 JSON **文件**，例如 `route.json` 内容为 `{"default": "openrouter", "clips": {"<片段 id>": "local42"}}`，再 `export VDN_ASR_ROUTE=<route.json 的路径>`（变量里放路径，不放 JSON 本身；入口 `Route::load`，测试 `route_loads_from_file_path`）。`VDN_ASR_BACKEND=local42` 整套改回本机 42model
- 停发：遇到永久错误（数据政策 403/404、余额 402、密钥 401、到上限、账本坏或写不了、费用未知）整套停发 OpenRouter 请求，原因写在 `meta.asr.halted`。「费用未知」= 连不上、超时、返回读不出、或返回有文本但没带 `usage.cost`：可能已经计费。这时账本里的预留不退，并写上 `review`（待核）；**只要 `review` 在，任何进程、重启以后都拒发**。处理：到 OpenRouter 后台核实际花费，改正账本的 `spent_usd` 与 `reserved_usd`，删掉 `review` 一项，再重跑（缓存里已有的转写不会重发）

## 二、识别与外发（D103 · D104，人定 2026-11-25）

- **识别改走云端**（D103）：OpenRouter 的 `qwen/qwen3-asr-1.7b-20260813`（Qwen3-ASR 1.7B，和本机 42model 的 `qwen3-asr-plus:1.7b` 同一个 1.7B 模型）。D103 原写的 `qwen3-asr-flash-2026-02-10` 在 OpenRouter 上没有（2026-11-25 03:36 UTC 读公开模型列表：472 个模型，转写类 30 个，没有 Flash），大总管已更正。型号正本 `src/eval/src/qa/asr_or.rs` 的 `OR_MODEL`
- **接口**：`POST /api/v1/audio/transcriptions`，JSON 体带 16 位 WAV 的 base64；返回 `text` 与 `usage.cost`（读 OpenRouter 文档 STT 一节）
- **每个请求都带 `provider.data_collection = "deny"`**（大总管硬要求）：路由不到不留存的提供方就当失败、停下，不退回允许留存的提供方。测试 `request_carries_model_audio_language_and_deny`（没带 deny 就红）、`credit_and_data_policy_failures_do_not_retry_or_fall_back`
- **密钥**：只从环境变量或 `.env.local` 读；经 `curl -K -` 从标准输入传，不进命令行参数、不打印；出错信息里若含密钥先抹掉（测试 `errors_never_contain_the_key`）
- **并发**（大总管定）：一段要识别的几条（干净人声、原声、输出 × 2 次）先一起并发发出，最多 `ASR_CONCURRENCY` = 8 路；429 与 5xx 各自按 `Retry-After` 或指数退避，最多 5 次。结果按音频 sha256 写缓存，之后照旧逐条取，所以成绩与完成先后无关（测试：`concurrent_batch_equals_serial_and_really_overlaps`、`prefetch_then_transcribe_equals_plain_serial` 核缓存文件逐字节相同）
- **外发许可**（D104，人定）：480 段都可以送去识别，含 ACE 卷积的 148 段、MIT 卷积的 300 段（MIT 原页没写许可，2026-11-25 核过）；存 RunPod 私有网络盘也认可。依据：识别服务替我们处理、不算分享，且不留存；这个解读没有人从法律上核过。范围只到评测识别和自家云盘，公开发布或给别人听仍要先问人。ESC-50（CC BY-NC）只评测
- 评分模型照旧在 CPU 上跑 ONNX（ort 没编 CUDA，H100 的 GPU 用不上）

### 「每条 2 次」是干什么的、怎么定（大总管 2026-11-25 定）

- **用途**（读源码 `asr.rs` · `qa/mod.rs`）：第 2 次是稳定性复核。两次转写不一致，这条音频剔除，该段吃字判「不知道」（rubric V7）
- **实测**（2026-11-25，`_tmp/exp025-asr-real-20261124-194229.log`）：同一条音频（car-passenger-c00-ext 的干净人声，LibriSpeech 英文 12 秒）连调 3 次，**逐字相同**；每次 0.00009 美元（`usage.cost`，等于每秒 0.0000075 美元 × 12 秒）；耗时 1.44 / 20.69 / 0.66 秒。只有 1 条音频，不能说明转写总是确定的
- **定了**：基线保留每条 2 次（`REPEATS` = 2），同时量出 1440 条音频的不一致率，记进 `meta.asr.unstable_rate`。不一致率 ≤ `MAX_UNSTABLE_RATE`（0）时，以后的模型改成只转 1 次（`VDN_ASR_REPEATS=1`），按音频 sha256 固定抽 `SPOT_FRACTION`（10%）照旧转 2 次复核。三个常量的正本在 `asr.rs`，测试 `spot_check_sampling_is_deterministic_and_about_ten_percent`

## 三、估时与费用（D103 后：整套上云）

**依据**：GTCRN 与 DSP、评分模型两项来自本机 5 段冒烟（`runs/smoke-20261124-191531/`，负载约 6.9，只作估时）；识别来自上面 3 次真调用。云上 CPU 的快慢未量化，先按本机速度推。

| 部分 | 每段 | 480 段 |
|---|---|---|
| GTCRN + DSP 各项 | 1.4 s | 约 11 分钟 |
| 两个评分模型（每段 4 次评分） | 7.3 s | 约 58 分钟 |
| 识别：每段 3 条 × 2 次 = 6 次，8 路并发成一批 | 一批约 1–2 s（偶有 20 s） | 480 批 ≈ 8–16 分钟（推断） |
| **合计** | | **约 1.3–1.5 小时 / 每个模型** |

- **上限能保证到什么程度**（codex 第 9、10 轮）：程序在发之前按估算预留，预留放不下就不发；任何一笔实际金额（成功或失败带回的）高过预留，就照记、标待核、停发，之后任何进程都拒发。**这只是软上限**：已经在途的请求（本进程最多 8 路，几个进程同时跑时更多）照样会计费，单次实际计费没有程序能保证的上界，所以**超出多少没有保证**。**已定（人定，D103 补充，2026-11-25）：15 美元按软上限执行，不另设硬上限；OpenRouter 侧这把密钥的额度是 100 美元**。所以最坏情况下的硬性兜底是 OpenRouter 那边的 100 美元，不是 15 美元
- **识别费**：每个模型 480 × 6 = 2880 次 × 0.00009 美元 ≈ **0.26 美元**；以后改成 1 次 + 10% 抽查约 0.14 美元。6 个模型（基线 + 5 个训练出的）约 0.26 + 5 × 0.14 ≈ **1.0 美元**，远低于上限 15 美元（D103：按 `usage.cost` 实际金额累计，到上限就停）
- **机器费**：CPU 机每小时约 0.12 美元（cloudrun 那台）× 每个模型约 1.5 小时 × 6 ≈ 1.1 美元，另加装环境约 0.5 小时（推断）。RunPod 另有 60 美元上限（D103），不在本表
- **回传**：每次约 3 MB（`clips.jsonl` 约 1.8 MB + `scorecard.json` 约 0.9 MB，按冒烟每段 3.8 KB 推）
- 原来的 A / B / B′ 三条路（D103 之前）作废：识别能上云以后，整套在云上一台机器跑完即可。跨平台对拍用 `merge.py --parity-only`（两边都有吃字值，只比不合并，判据同第四节）；不带这个开关的合并模式只适用于「云上没有识别」的旧分工
- 正式跑前：申请名额；云上要有 `.env.local` 的密钥、两个评分模型文件、6323 音频链接（第一节）

## 四、验收

1. **没跑成的段为 0**：`scorecard.json` 的 `missing` 为空，`clips_scored` = 480（`merge.py` 遇到任一边有缺段就拒绝合并）
2. **进仓**：`runs/<run>/` 下的 `clips.jsonl` · `scorecard.json` · `scorecard.md`，以及 `gap.md` · `gap.json`；不进仓的音频写路径与哈希
3. **和本机已有成绩对拍**（`vdn-eval compare`）：
   - 对 exp021 的不回混 GTCRN（`docs/experiments/exp021-gtcrn-aux/runs/20261105-sweep-gtcrn/clips.jsonl.gz` 里系统 `gtcrn-h-nomix`，同一批音频，不带 `--qa`）：`vdn-eval compare --a <解压后只留 gtcrn-h-nomix 的文件> --b <本次>/clips.jsonl --system-a gtcrn-h-nomix`。
     - **比哪些项**：两边都有配对值的项（`compare.json` 里该项 `pairs` ≥ 1）。exp021 没带 `--qa`，吃字和两个评分模型没有配对，不比；只在有旁人说话的段才有值的项（如 V-SIM-FA）在那份里没有配对时也不比，报告里照实列「不可比」
     - **判据**：本机跑的那份，可比项里除音色 V-TIMBRE 外差值必须全为 0（同一台机器、同一份代码路径）；V-TIMBRE 在 exp021 之后由 #119 改了量法（30 个 ERB 频带的离散程度），不比
     - 5 段冒烟实测：23 项里 19 项有配对，其中 18 项差值为 0，只有 V-TIMBRE 不同；另 4 项（吃字、两个评分模型、V-SIM-FA）没有配对，不可比（`runs/smoke-20261124-191531/compare-vs-exp021/`）
   - **云上跑**的那份（走 B 时）：和本机那份用 `merge.py` 逐段比，三条都满足才过：每项逐段值差 ≤ `PARITY_ABS`（1e-3，该项自己的单位）；没有任何段一边有值一边没值；判定不同的段占全部段数 ≤ `PARITY_FLIP`（1%）。正本在 `merge.py`，测试 `test_merge.py`。用 `compare` 判不了这件事：两边差很小但方向一致时，区间会整个落在 0 的一边，被判成「涨」或「退」
   - 对 exp014 的回混 GTCRN（`docs/experiments/exp014-scorecard-gtcrn/runs/20261105-005916-4ccf0638/`）：只报不判，用来说明回混口径和不回混口径差多少
4. **识别**：`meta.asr` 里没有 `error`；`backends.openrouter.model` = `qwen/qwen3-asr-1.7b-20260813`，`clips` = 480；`checked` = 1440（每段 3 条，基线每条 2 次）并写出 `unstable` 与 `unstable_rate`（这个数决定以后的模型能不能改成 1 次，见第二节）；账本 `_tmp/asr-openrouter-ledger.json` 的增量与本次调用数相符、累计 ≤ 15 美元
5. **差距表**（两种，见第五节）：`python docs/experiments/exp025-gtcrn-train/baseline/gap.py weak <run>/scorecard.json --system gtcrn --out <run>` 与 `gap.py uplift …`；算法和阈值的正本在 `gap.py` 文件头和常量，测试 `test_gap.py`

冒烟对拍（实测，5 段，不带 `--qa`；debug 版和 release 版各跑一次，结果相同）：对 exp021 `gtcrn-h-nomix`，19 项有配对，18 项差值为 0，只有 V-TIMBRE 均值差 +0.169 dB（0.188 → 0.357，原因见上）；4 项不可比。合并 → `--from-clips` → 差距表整条链跑通。

## 五、差距表怎么得出「加训哪几个场景」

大总管 2026-11-25 定：**场景依据先不定**，基线出数后把两张表并排交人定。两表都给前 4 和重合度，临时选定的四个照实列出来对照，**结论留空**。算法与阈值的正本都在 `gap.py`（文件头 + 常量），测试 `test_gap.py`。

### 1. 弱项表：GTCRN 自己最弱的场景（`gap.py weak`）

- 只看判定项（不看「只报不判」的项），外加两个评分模型；逐个 档 × 设备 切片，把场景按该项均值从好到差排名，换成 0–1 的名次分（最差 = 1）
- 场景的**弱点分** = 它所有（项, 切片）名次分的平均；**不过数** = 它判「不过」的（项, 切片）个数。先看不过数，再看弱点分
- 输入：本次基线 `runs/<run>/scorecard.json`（不回混，480 段）

### 2. 提升空间表：已有候选不回混时比 GTCRN 好多少（`gap.py uplift`）

- 输入：`vdn-eval compare --a <GTCRN 的 clips.jsonl> --b <候选的 clips.jsonl> --by-scene` 的 `compare.json`（成对 bootstrap 的「涨 / 退 / 不确定」由 Rust 判），外加基线 `scorecard.json`（只用来认判定项）
- 场景的**涨数** = 判定项里判「涨」的个数；**提升分** = 每项按「候选比 GTCRN 好多少」把场景排名成 0–1（提升最大 = 1）再平均。先看涨数，再看提升分
- **两边必须同一口径、同一代码版本**：不然改过量法的项会混进来。不得已时用 `--exclude` 去掉，表头会列出被去掉的项

**哪些候选已有不回混的成绩单**（查了各实验 `runs/*/scorecard.json` 的 `meta`，2026-11-25）：

| 候选 | 不回混成绩单 | 有的项 | 缺口 |
|---|---|---|---|
| DFN3 低延迟版（dfn3-ll） | exp021 `runs/20261105-sweep-dfn3/` 的 `dfn3-nomix`（480 段，代码 eb2296f7） | 只有 DSP 各项（当时没带 `--qa`） | 吃字、两个评分模型没有 |
| DPDFNet-2 | 没有：exp016 的 `dpdfnet2-20261105-115707-fcf7cfb0` 是回混口径（`suite` = 原清单，保护处理开着） | — | 整张不回混成绩单都没有 |

按交代，**不为此补跑候选**。所以现在能出的只有「DFN3 对 GTCRN、只看 DSP 项」这一张；对照的 GTCRN 用同一次 exp021 扫描里的 `gtcrn-h-nomix`（同一代码版本、同一批音频）。

**预览（实测数据，结论留空）**：`runs/uplift-exp021-dfn3-nomix/rerun-20261125b/gap-uplift.md`（数据完整性已核：15 个场景都有，每个场景有差值的判定项 ≥ 5，没有只在一边的片段，每个场景的配对段数 32 = 基线成绩单里 gtcrn-h-nomix 的段数）。只含 DSP 判定项，前 4：cowork · car-passenger · cafe · home-family；与临时选定的四个重合 2 个（cafe · car-passenger），按线判「不一致」。基线出数后，这张表的 GTCRN 一侧不用换（候选只有 exp021 那一份，必须配同版本的 GTCRN）。

### 3. 两张表共用的判据（正本 `gap.py`）

- 前 `TOP_K`（4）个场景与临时选定的四个重合 ≥ `MIN_OVERLAP`（3）个 → 「一致」，否则「不一致」
- **第 4 名边界并列**：重合数算成范围，整个范围 ≥ 线才「一致」，整个 < 线才「不一致」，跨线 → 「不确定（第 4 名边界并列）」（codex 第 1 轮 P1 #2）
- **数据不全不下结论**，两张表各自的条件（codex 第 1 轮 P1 #1、第 2 轮 P1 #1）：
  - 弱项表：场景数 < `MIN_SCENES`（15，与 `src/eval/suite/scenes.json` 的场景数由测试比对）、有场景缺切片（≠ `SLICES` = 4）、有场景因格子太少（< `MIN_CELLS` = 10）被排除、成绩单有没跑成的段
  - 提升空间表：逐场景结果 < `MIN_SCENES`、有场景的有差值判定项 < `MIN_ITEMS`（5）、有片段只在一边、某场景的配对段数 ≠ 基线成绩单里**基线那个系统**在该场景的段数（两边共同缺段靠这条抓；codex 第 3 轮 P1 #1）、基线卡里没有段数、基线成绩单自己有基线系统没跑成的段（codex 第 4 轮 P1 #1）（compare 不带切片信息，这张表不查切片）
  - 任一成立 → 「数据不足」，只展示排名。5 段冒烟出的表因此判「数据不足」（`runs/smoke-20261124-191531/gap-rerun-20261125/`；日志里旧版判的「一致」已作废，见同目录 `SUPERSEDED.md`）
- 结论（加训哪几个场景）由大总管交人定；表只说「数据和临时选择对不对得上」

**冒烟时注意到的一点（实测，推断原因）**：用弱项表的算法读 exp014 的**回混** GTCRN 成绩单，前 4 是 canteen · bus-metro-cabin · transit-hub · rail-air-cabin，和临时选定的四个只重合 1 个（transit-hub）。exp016 挑那四个时看的是「候选比 GTCRN 好多少」，接近提升空间表的问题，不是弱项表的问题。我没有为了对上旧选择去改算法。

## 文件

| 文件 | 用途 |
|---|---|
| `gap.py` · `test_gap.py` | 场景差距表两种：弱项表 · 提升空间表 |
| `merge.py` · `test_merge.py` | 云上 + 本机两份逐段分数合并，跨平台对拍 |
| `subset.py` · `test_subset.py` | 裁 ≤ 5 段的冒烟测试集 |
| `runs/smoke-20261124-191531/` | 5 段冒烟：三次运行的成绩单与日志、对 exp021 的对拍（路径里的本机目录换成了占位符） |
| `runs/uplift-exp021-dfn3-nomix/` | 提升空间表预览：exp021 的 DFN3 对 GTCRN，都不回混，只有 DSP 项 |

测试：`python3 -m unittest discover -s docs/experiments/exp025-gtcrn-train/baseline -p "test_*.py"`（31 条）；每个测试文件也能单独直接运行。
