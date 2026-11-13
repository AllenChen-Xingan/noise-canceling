# 20261106-mixretest · 混回比例重测（D90）

> 2026-11-06 · 手下 cands · 成绩单 run `20261106-182047-1de5e57b`（提交 1de5e57b，接电源）· 带评分模型（`--qa`）· 规则 v2
> 结论在 `../../findings-mixretest-20261106.md`；检查表在本目录 `checklist.md`。

## 四档

都由 GTCRN 原始输出（`_tmp/030/gtcrn-h-raw`，带预热段 `.head`）经 `driver remix` 用真引擎重混。

| 系统名 | 说话时 | 停顿时 | 说明 |
|---|---|---|---|
| `gtcrn-h-om6-fm12` | −6 dB | −12 dB | 默认档（引擎现值） |
| `gtcrn-h-om9-fm12` | −9 dB | −12 dB | |
| `gtcrn-h-om12-fm12` | −12 dB | −12 dB | 人说「少混回效果最好」那一档，另有单独检查表 |
| `gtcrn-h-nomix` | — | — | 不混回（`--off`） |

测试集：`_tmp/030/scenes-safety-off.json`（由 `driver suite` 派生，全部任务关保命，免得混两次）。每档 480 段，15 个场景各 32 段。

## 怎么跑的

```sh
# mixretest worktree 根目录；音频在主目录 _tmp/030/，不进仓
caffeinate -i target/release/vdn-eval scorecard --suite <主目录>/_tmp/030/scenes-safety-off.json \
  --system external:<主目录>/_tmp/030/gtcrn-h/o6-f12,external:<主目录>/_tmp/030/gtcrn-h/o9-f12,external:<主目录>/_tmp/030/gtcrn-h/o12-f12,external:<主目录>/_tmp/030/gtcrn-h/nomix-h \
  --qa --rules v2 --out <主目录>/_tmp/030/mixretest-20261106-182300
python3 docs/experiments/exp021-gtcrn-aux/checklist.py <clips.jsonl> gtcrn-h-om6-fm12 \
  gtcrn-h-om9-fm12 gtcrn-h-om12-fm12 gtcrn-h-nomix --sheet12 gtcrn-h-om12-fm12 --out checklist.md
```

## 经过（实测）

1. 第一次跑（`_tmp/030/mixretest-20261106-112237`）：11:22 开跑。识别服务变慢到每次 17–18 s，11:25 干净停跑，缓存 3843 条完好。14:33 识别服务重启后测得每次 3.8 s / 0.9 s，14:35 在同一目录续跑，18:20 跑完。
2. 第一次跑有 1 段吃字判「不知道」：`gtcrn-h-om9-fm12` 的 `canteen-s04-ext`（识别失败不写缓存，见 #137）。
3. 补跑：新目录 `_tmp/030/mixretest-20261106-182300`，先放入第一次的缓存（6718 条），四档重跑，18:21 到 21:38。识别只新增 2 次，时间基本花在评分模型上。吃字「不知道」0 段。
4. 核对（`analyze.py check`，两次逐段逐项比）：默认档、−12、不混回三档 **差 0**；−9 档只有 `V-EAT-ASR` 一项、`canteen-s04-ext` 一段不同（就是补上的那段）。
5. 和 2026-11-05 的扫档（`../20261105-sweep-gtcrn/`，提交 eb2296f7）比：默认、−9、−12 三档逐段核对，除下面四项外**差 0**。
   - 吃字和两个听感分扫档没测（不带评分模型）。
   - 音色偏差（V-TIMBRE）值不同、判定全同：两次提交之间音色偏差的算法改过（`git log eb2296f7..1de5e57b -- src/eval/src/verifiers/`）。本次检查表不用这一项。

## 文件

| 文件 | 内容 |
|---|---|
| `clips.jsonl.gz` | 补跑的逐段分数（1920 行，gzip） |
| `scorecard.json` | 补跑的成绩单。**改过一处**：`meta.suite` 原是本机绝对路径，复制时去掉主目录前缀，改成 `_tmp/030/…`；原件哈希见 `SHA256SUMS` 第 2 行 |
| `checklist.md` | `checklist.py` 的输出：每档按调研 030 标准逐条判，外加 −12 档单独检查表 |
| `SHA256SUMS` | 主目录原件（逐段分数、成绩单、测试集、四档与原始输出的 `run.json`）和本目录文件的哈希 |
