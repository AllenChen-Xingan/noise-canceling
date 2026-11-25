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
shasum -a 256 -c docs/experiments/exp013-scene-suite/builds/20261124165427-6323/hashes.txt | grep -vc ': OK$'   # 必须是 0
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

- 成绩单 `meta` 里应看到 `"safety": "off"`、`"qa": true`；`"asr"` 本机为 true、云上为 false
- 输出：`scorecard.md` · `scorecard.json` · `clips.jsonl`；音频类中间产物（`qa/asr-wav/`）不进仓

## 二、识别服务与评分模型能不能上云（实测 + 读源码）

- **评分模型能上云**：两个都是 ONNX，用 ort 在 CPU 上跑（`src/eval/src/qa/nrmos.rs`），和平台无关，装法见上一节
- **识别服务不能上云**：吃字（V-EAT-ASR）用的是本机 42model 应用里的 Qwen3-ASR（`/Applications/42model.app`，`127.0.0.1:11520`，`src/eval/src/qa/asr.rs`），只有 macOS 版。云上换别的识别实现，转写会不同，和本机已有的吃字读数不能比，所以不换
- **ort 只编了 CPU 版**（`Cargo.toml` 的 ort 特性里没有 CUDA），评分模型在 H100 上也只用 CPU，GPU 不起作用
- 云上没有识别服务时，`--qa` 照样算评分模型，吃字整列判「不知道」（读源码 `qa/mod.rs`：`Asr::alive()` 为假）

**分工**（走第三节 B 路时）：
1. 云上：第一节的命令，出 DSP 各项 + 两个评分模型，吃字为「不知道」
2. 本机：同一条命令（有识别服务），出全部各项
3. 合并：`python docs/experiments/exp025-gtcrn-train/baseline/merge.py --cloud <云上>/clips.jsonl --local <本机>/clips.jsonl --out <新目录>`：每段吃字取本机、其余取云上，并把其余每项两边逐段比一遍（跨平台对拍，判据见第四节）
4. 出成绩单：`target/release/vdn-eval scorecard --from-clips <新目录>/clips.jsonl --out <另一个新目录>`

## 三、估时与两条路（建议，待大总管定）

**依据**：本机 5 段冒烟，release 版，2026-11-25（`runs/smoke-20261124-191531/`，每步日志在同目录）。当时插电，负载约 6.9（别的项目在跑、cloudrun 在上传），**只作估时依据，不是测速基准**。

| 部分 | 实测 | 每段 | 480 段 |
|---|---|---|---|
| GTCRN + DSP 各项（不带 `--qa`） | 5 段 7.07 s | 1.4 s | 约 11 分钟 |
| 两个评分模型（带 `--qa`、识别已缓存：43.45 s − 7.07 s） | 5 段 36.4 s，每段 4 次评分 | 7.3 s（约每次 1.8 s，含载入） | 约 58 分钟 |
| 识别（带 `--qa`、没缓存：96.0 s − 43.45 s） | 30 次 | 每次 1.75 s | 2880 次 × 1.75 s ≈ 84 分钟 |

- **识别次数**：每段识别干净人声、原声、GTCRN 输出 3 条，每条 2 次（`asr.rs` 的 `REPEATS`）→ 480 × 6 = 2880 次（5 段实测 30 次，对得上）。干净人声那一路不送识别
- **以后每个模型**：干净人声和原声的转写可以预先放进 `<--out>/qa/asr_cache.json` 复用（按音频 sha256 去重），只剩输出 480 × 2 = 960 次 ≈ 28 分钟；评分模型对原声的分不落盘，每次都重算（读源码 `qa/mod.rs` 的 `mix_mos`）

| | A. 全在本机 | B. 云上 DSP + 评分模型，本机识别 |
|---|---|---|
| 每个模型耗时 | 第一个约 2.6 小时（11 + 58 + 84 分钟），之后约 1.6 小时（11 + 58 + 28） | 云上约 1.2 小时（11 + 58 分钟，按本机速度推，云上 CPU 快慢未量化）**加** 本机同样 1.6–2.6 小时 |
| 6 个模型合计 | 约 10.6 小时（2.6 + 5 × 1.6） | 本机同样约 10.6 小时，另加云上约 7 小时机时 |
| 费用 | 0 | 云上 CPU 机每小时 0.12 美元（cloudrun 那台）：装环境约 0.5 小时（编译、传 396 MB 评分模型，推断）+ 6 × 1.2 小时 ≈ 1 美元；回传每次约 3 MB（`clips.jsonl` 约 1.8 MB + `scorecard.json` 约 0.9 MB，按 5 段实测每段 3.8 KB 推算） |
| 风险 | 本机要一直开着：会被休眠打断（11-24 真出过）→ 用 `caffeinate -i`；和别的线抢 CPU（这次负载 6.9）→ 读数不受影响，只是变慢；计时类读数（最坏一帧耗时 V-FRAME 不在成绩单里）不受影响 | 本机那一半的风险和 A 一样；多一份云上开机、装环境、链接音频出错的风险；好处是多一份跨平台对拍 |

**为什么 B 省不了本机的时间**：吃字要识别 GTCRN 的输出，本机拿不到云上的输出音频，只能自己重跑 GTCRN；而现在的 `--qa` 不能只开识别，评分模型也会跟着重算。
**B′（要改代码，没做）**：给 `--qa` 加「只算识别」的开关，本机每个模型可省约 58 分钟（6 个模型约 5.8 小时）；或者云上把 GTCRN 输出音频传回本机（每段 1 条，480 段约 0.4 GB：冒烟时 `qa/asr-wav/` 5 段写了 20 条共 15 MB，每条约 0.75 MB），本机只做识别；这也要改代码，让识别能读外部输出。

**建议，待大总管定**：走 A。理由：识别只能在本机，B 不省本机时间，只多花钱、多一处出错的地方；A 的风险用 `caffeinate` 和错开别的重活就能压住。若要跨平台对拍的证据，基线这一次走 B，之后的模型走 A。

## 四、验收

1. **没跑成的段为 0**：`scorecard.json` 的 `missing` 为空，`clips_scored` = 480（`merge.py` 遇到任一边有缺段就拒绝合并）
2. **进仓**：`runs/<run>/` 下的 `clips.jsonl` · `scorecard.json` · `scorecard.md`，以及 `gap.md` · `gap.json`；不进仓的音频写路径与哈希
3. **和本机已有成绩对拍**（`vdn-eval compare`）：
   - 对 exp021 的不回混 GTCRN（`docs/experiments/exp021-gtcrn-aux/runs/20261105-sweep-gtcrn/clips.jsonl.gz` 里系统 `gtcrn-h-nomix`，同一批音频，不带 `--qa`）：`vdn-eval compare --a <解压后只留 gtcrn-h-nomix 的文件> --b <本次>/clips.jsonl --system-a gtcrn-h-nomix`。**本机跑**的那份，除音色 V-TIMBRE 外各项差值必须为 0；V-TIMBRE 在 exp021 之后由 #119 改了量法（30 个 ERB 频带的离散程度），不比。5 段冒烟实测：22 项差 0，只有 V-TIMBRE 不同（`runs/smoke-20261124-191531/compare-vs-exp021/`）
   - **云上跑**的那份（走 B 时）：和本机那份用 `merge.py` 逐段比，每项逐段值差 ≤ `PARITY_ABS`（1e-3，该项自己的单位），且判定不同的段 ≤ `PARITY_FLIP`（1%）；正本在 `merge.py`，测试 `test_merge.py`。用 `compare` 判不了这件事：两边差很小但方向一致时，区间会整个落在 0 的一边，被判成「涨」或「退」
   - 对 exp014 的回混 GTCRN（`docs/experiments/exp014-scorecard-gtcrn/runs/20261105-005916-4ccf0638/`）：只报不判，用来说明回混口径和不回混口径差多少
4. **差距表**：`python docs/experiments/exp025-gtcrn-train/baseline/gap.py <run>/scorecard.json --system gtcrn --out <run>`；算法和阈值的正本在 `gap.py` 文件头和常量，测试 `test_gap.py`

冒烟对拍（实测，5 段，不带 `--qa`；debug 版和 release 版各跑一次，结果相同）：对 exp021 `gtcrn-h-nomix`，23 项里 22 项差值为 0，只有 V-TIMBRE 均值差 +0.169 dB（0.188 → 0.357，原因见上）。合并 → `--from-clips` → 差距表整条链跑通。

## 五、差距表怎么得出「加训哪几个场景」

算法写在 `gap.py` 文件头，要点：
- 只看判定项（不看「只报不判」的项），外加两个评分模型；逐个 档 × 设备 切片，把 15 个场景按该项均值从好到差排名，换成 0–1 的名次分（最差 = 1）
- 场景的**弱点分** = 它所有（项, 切片）名次分的平均；**不过数** = 它判「不过」的（项, 切片）个数
- 排序：先看不过数，再看弱点分。有值的格子少于 `MIN_CELLS`（10）的场景不排
- **判据**：前 `TOP_K`（4）个场景与临时选定的四个重合 ≥ `MIN_OVERLAP`（3）个 → 「一致」，否则「不一致」。正本是 `gap.py` 的常量，测试 `test_gap.py`
- 结论由大总管下：表只说「数据和临时选择对不对得上」

**冒烟时注意到的一点（实测，推断原因）**：用同一算法读 exp014 的**回混** GTCRN 成绩单，前 4 是 canteen · bus-metro-cabin · transit-hub · rail-air-cabin，和临时选定的四个只重合 1 个（transit-hub）。可能的原因：exp016 挑那四个时看的是「两个候选模型比 GTCRN 好多少」（相对差），本算法看的是 GTCRN 自己离线多远（绝对弱点）。这两个问题不一样；用哪个当加训依据，待大总管定。我没有为了对上旧选择去改算法。

## 文件

| 文件 | 用途 |
|---|---|
| `gap.py` · `test_gap.py` | 场景差距表 |
| `merge.py` · `test_merge.py` | 云上 + 本机两份逐段分数合并，跨平台对拍 |
| `subset.py` · `test_subset.py` | 裁 ≤ 5 段的冒烟测试集 |
| `runs/smoke-20261124-191531/` | 5 段冒烟：三次运行的成绩单与日志（路径里的本机目录换成了占位符） |

测试：`python3 -m unittest discover -s docs/experiments/exp025-gtcrn-train/baseline -p "test_*.py"`（18 条）。
