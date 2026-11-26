# 场景差距表 · 第二版（2026-11-26）

> **和第一版的差别：弱项表加进了吃字**（识别补跑后，吃字有值 452 / 480 段；第一版只有 82 段有值，没用）。识别有分歧时调第 3 次、取多数（正本 `src/eval/src/qa/asr_vote.rs`）。
> **提升空间表照旧不含吃字，表本身没变**，见第一版 `../v1-20261125/uplift/`（原因见第 3 节）。
> 出表人：evalcmp。**结论（加训哪几个场景）留空，交人定。**

## 0. 输入与核对（实测）

- 弱项表输入：识别补跑合并后的成绩单 `../../runs/rerun-20261126-013313/scorecard.json`（sha256 `7ac82d0b…`，全值见同目录 `SHA256SUMS.cloud`）
  - 补跑人：cloudrun-39，云上分 4 片，main ab0f4f0a，2026-11-26 01:33–02:44 UTC。GTCRN 原版、不回混，480 段
  - 识别花费：账本 +1953 次、+0.187488 美元（上限 0.25）。4 片 `meta.asr` 的 calls 之和也是 1953。各片没有错误，也没有停发。账本快照 `../../ledger/20261126-rerun-after.json`
  - 各片的第 3 次调用（`tie_breaks`）：52 · 51 · 51 · 56，合计 210 次。第 3 次没拿到（`tie_break_missing`）：0。三次都不同（`no_majority`）：8 · 4 · 7 · 10，合计 29 次
- 和第一版的输入（run480，`6ffc7eb0…`）逐段对比（实测）：
  - 480 段的混音 sha256 全同
  - 吃字以外的项：判定 0 处变化。两个听感评分模型的值差 ≤ 4.4e-16（浮点打印），延迟完全相同
  - 每个场景第二版的「不过数」减第一版，正好等于这个场景吃字卡的不过数。所以第二版的变化全部来自吃字
- `gap.py weak` 重跑第一版的输入，输出和第一版逐字节相同
- `vdn-eval scorecard --from-clips` 重新汇总补跑的 `clips.jsonl`，除 meta 外和补跑成绩单完全相同（敏感性用的就是这条路）

## 1. 弱项表：GTCRN 自己最弱的场景（含吃字）

来源：`weak/gap-weak.json`（`gap.py weak … --system gtcrn`）。排序：先按不过的（项, 切片）数，再按弱点分。数据完整：15 个场景，每个 4 个切片，没有缺段。

| 名次 | 场景 | 不过数 | 其中吃字 | 弱点分（0 最好 · 1 最差） | 第一版名次 | 吃字没值的段 | 临时选定 | 训练数据噪声池 |
|---|---|---|---|---|---|---|---|---|
| 1 | rail-air-cabin | 14 | 3 | 0.580 | 1 | 7 |  | **没有** |
| 2 | canteen | 13 | 4 | 0.676 | 2 | 2 |  | 有（demand-train） |
| 3 | meeting-room | 11 | 3 | 0.655 | 4 | 3 |  | **没有** |
| 4 | transit-hub | 11 | 2 | 0.482 | 3 | 0 | 是（车站） | **没有**（等第二批 CochlScene） |
| 5 | car-passenger | 11 | 4 | 0.449 | 8 | 3 | 是（车内） | 有（qut-noise） |
| 6 | bus-metro-cabin | 10 | 4 | 0.492 | 9 | 6 |  | 有（demand-train） |
| 7 | cowork | 8 | 3 | 0.590 | 10 | 2 |  | **没有** |
| 8 | cafe | 8 | 4 | 0.487 | 15 | 0 | 是（咖啡馆） | 有（demand-train · qut-noise） |
| 9 | open-office | 8 | 1 | 0.486 | 6 | 2 |  | **没有** |
| 10 | street-outdoor | 8 | 3 | 0.405 | 13 | 0 | 是（街道） | 有（demand-train · qut-noise · tum-wind） |
| 11 | phone-booth | 7 | 0 | 0.490 | 5 | 0 |  | **没有** |
| 12 | home-family | 7 | 0 | 0.452 | 7 | 3 |  | 有（demand-train · qut-noise） |
| 13 | library-classroom | 6 | 1 | 0.491 | 11 | 0 |  | 有（qut-noise） |
| 14 | home-quiet | 5 | 0 | 0.384 | 12 | 0 |  | **没有** |
| 15 | hotel-room | 5 | 0 | 0.382 | 14 | 0 |  | **没有** |

- 「不过数」：这个场景 4 个切片里判「不过」的（项, 切片）数。「其中吃字」：4 张吃字卡里判不过的张数。吃字判法：一段比原声多错 ≥ 5 个字，这段就否决
- 噪声池一列照第一版（`data/recipe.json` 的 `noise_pools`），没有重查
- 前 4：rail-air-cabin · canteen · meeting-room · transit-hub。和第一版是同一组，只是第 3、4 名对调
- 第 4、5 名的不过数都是 11，靠弱点分分开（0.482 对 0.449）。`gap.py` 算了边界并列：重合数范围是 1–1，结论不受影响
- 判据读数（不是结论）：和临时选的四个重合 1 个（transit-hub），线是 3，所以判「不一致」。第一版也是 1 个、「不一致」

临时选的四个场景在两版里的名次：

| 场景 | 第二版名次 | 第一版名次 | 不过数（其中吃字） |
|---|---|---|---|
| 街道（street-outdoor） | 10 / 15 | 13 / 15 | 8（3） |
| 车站（transit-hub） | 4 / 15 | 3 / 15 | 11（2） |
| 车内（car-passenger） | 5 / 15 | 8 / 15 | 11（4） |
| 咖啡馆（cafe） | 8 / 15 | 15 / 15 | 8（4） |

## 2. 吃字还没值的 28 段：落在哪、会不会改排名

这 28 段的判定是「不知道」：三次转写各不相同，没有多数（rubric V7）。

分布（实测，每个场景 32 段）：

| 场景 | 没值段数 | 其中常见档 · 压力档 |
|---|---|---|
| rail-air-cabin | 7 | 1 · 6 |
| bus-metro-cabin | 6 | 3 · 3 |
| car-passenger | 3 | 1 · 2 |
| home-family | 3 | 0 · 3 |
| meeting-room | 3 | 2 · 1 |
| canteen | 2 | 1 · 1 |
| cowork | 2 | 0 · 2 |
| open-office | 2 | 1 · 1 |
| 其余 7 个场景 | 0 | — |

敏感性：把 28 段全当「过」、全当「不过」，各重排一次。做法：改 `clips.jsonl` 里这 28 段的吃字分，再用 `vdn-eval scorecard --from-clips --rules v2` 重新汇总，最后跑 `gap.py weak`。

- 全当过：多错 0 字，参考字数取全部有值段的中位数 26
- 全当不过：取实测按上面判法不过的段里、多错字数居中的那一段，多错 7 字、参考 25 字（错字率 28%）
- 改分的脚本和中间文件在主目录 `_tmp/20261126-gap-v2/sens/`（不进仓）。`sens.py` 的 sha256 是 `512ef6c5…`，两份改过的 `clips.jsonl` 是 `cf922d94…`（过）· `fd4b2ed8…`（不过）

| 名次 | 照实（第 1 节） | 全当过 | 全当不过 |
|---|---|---|---|
| 1 | rail-air-cabin 14 | rail-air-cabin 14 | rail-air-cabin 15 |
| 2 | canteen 13 | canteen 13 | canteen 13 |
| 3 | meeting-room 11 | meeting-room 11 | meeting-room 12 |
| 4 | transit-hub 11 | transit-hub 11 | transit-hub 11 |
| 5 | car-passenger 11 | car-passenger 11 | car-passenger 11 |
| 6 | bus-metro-cabin 10 | bus-metro-cabin 10 | bus-metro-cabin 10 |
| 7 | cowork 8 | cowork 8 | open-office 10 |
| 8 | cafe 8 | open-office 8 | cowork 9 |
| 9 | open-office 8 | cafe 8 | home-family 9 |
| 10 | street-outdoor 8 | street-outdoor 8 | cafe 8 |
| 11 | phone-booth 7 | phone-booth 7 | street-outdoor 8 |
| 12 | home-family 7 | home-family 7 | phone-booth 7 |
| 13 | library-classroom 6 | library-classroom 6 | library-classroom 6 |
| 14 | home-quiet 5 | home-quiet 5 | home-quiet 5 |
| 15 | hotel-room 5 | hotel-room 5 | hotel-room 5 |

（场景后的数字是不过数。全表在 `sens-pass/` · `sens-fail/`。）

- **前 4 在三种情况下都一样**：rail-air-cabin · canteen · meeting-room · transit-hub。和临时选定的重合都是 1 个，都判「不一致」
- 全当过：吃字卡没有一张变（不过的仍是 32 / 60 张），只有第 8、9 名换位，是弱点分差 0.001 以内的变动
- 全当不过：吃字卡不过的从 32 张变成 39 张。open-office 从第 9 名升到第 7，home-family 从第 12 名升到第 9；cafe 从第 8 名降到第 10，street-outdoor 从第 10 名降到第 11
- 读数（不是结论）：这 28 段不会改变前 4 和重合判定。第 7–12 名的先后会受影响

## 3. 提升空间表：没变，照旧不含吃字

用第一版的表：`../v1-20261125/uplift/gap-uplift.md`。前 4 是 cowork · car-passenger · cafe · home-family；和临时选定的重合 2 个，判「不一致」。

为什么还不含吃字：这张表比的是 DFN3 低延迟版和 GTCRN 的成对差值，要 DFN3 一侧也有吃字。但 exp021 跑的时候没带识别，DFN3 的输出音频也已不在本机。本机盘只剩约 7 GB，重新生成放不下。如果你要这一侧的吃字，要在云上 CPU 机器重跑 DFN3 并识别。识别次数和花费未量化，跑之前先估。

## 4. 结论（交人定）

（留空）

## 文件

| 文件 | 是什么 |
|---|---|
| `weak/gap-weak.md` · `.json` | 第二版弱项表，`gap.py weak` 原样输出（来源行里的本机路径换成了 `<主目录>`） |
| `sens-pass/` · `sens-fail/` | 28 段全当过 / 全当不过时的 `gap.py weak` 输出 |
| `../v1-20261125/` | 第一版：README、弱项表、提升空间表（不含中间输入） |
| `../../runs/rerun-20261126-013313/` | 补跑结果：合并成绩单 `scorecard.json` · `.md`、`clips.jsonl.gz`、各片 meta 和日志（`shards/`）、云上 `SHA256SUMS.cloud` |
| `../../ledger/20261126-rerun-after.json` | 补跑后的识别账本快照（sha256 `d78d4a00…`）。逐次明细 1953 行留在主目录 `_tmp/20261126-asr-rerun/`，不进仓 |
