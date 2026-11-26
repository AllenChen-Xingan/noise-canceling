# 场景差距表 · 第一版（2026-11-25）

> **第一版：不含吃字，识别补跑后出第二版。** 吃字（V-EAT-ASR）在 480 段里只有 82 段有值（识别中途停发、两次转写不一致），这一版两张表都不用它；其余评分项都用上。
> 出表人：evalcmp（只读，没发识别调用）。**结论（加训哪几个场景）留空，交人定。**

## 怎么算的（判据照 exp025 baseline README 第五节，正本 `gap.py`）

- 弱项表：`gap.py weak`，输入 = 云上 480 段基线成绩单（`_tmp/20261125-cloudrun-baseline/run480/scorecard.json`，sha256 前 12 位 `6ffc7eb06cd5`，GTCRN 原版、不回混、main 27d2808a）去掉每张卡里的吃字项（60 张卡各 1 项，`weak/scorecard-no-eat.json`），其余原样
- 提升空间表：`gap.py uplift`，输入 = exp021 的 `dfn3-nomix` 对同一次 exp021 里的 `gtcrn-h-nomix`（`docs/experiments/exp021-gtcrn-aux/runs/20261105-sweep-{gtcrn,dfn3}/clips.jsonl.gz`，各 480 段）。两份输入的 sha256 与之前的预览完全相同（`5804f1d5…` · `22babff8…`），重算结果与预览 `runs/uplift-exp021-dfn3-nomix/rerun-20261125b/` 逐行相同。这张表本来就没有吃字（exp021 当时没带 `--qa`），也没有两个评分模型
- 阈值：前 `TOP_K` = 4；与临时选的四个重合 ≥ `MIN_OVERLAP` = 3 算「一致」（`gap.py` 常量）
- 「训练数据有没有噪声池」：`docs/experiments/exp025-gtcrn-train/data/recipe.json` 的 `noise_pools`（最后改动 5ba6e0b7）；只列第一批能覆盖的场景

## 1. 弱项表：GTCRN 自己最弱的场景

来源：`weak/gap-weak.json`（`gap.py weak … --system gtcrn`）。排序：先按不过的（项, 切片）数，再按弱点分。数据完整：15 个场景、每个 4 个切片、没有缺段。

| 名次 | 场景 | 不过的（项, 切片）数 | 弱点分（0 最好 · 1 最差） | 有值格数 | 临时选定 | 训练数据噪声池 |
|---|---|---|---|---|---|---|
| 1 | rail-air-cabin | 11 | 0.585 | 56 |  | **没有** |
| 2 | canteen | 9 | 0.657 | 52 |  | 有（demand-train） |
| 3 | transit-hub | 9 | 0.468 | 56 | 是（车站） | **没有**（等第二批 CochlScene） |
| 4 | meeting-room | 8 | 0.657 | 48 |  | **没有** |
| 5 | phone-booth | 7 | 0.523 | 48 |  | **没有** |
| 6 | open-office | 7 | 0.486 | 52 |  | **没有** |
| 7 | home-family | 7 | 0.460 | 52 |  | 有（demand-train · qut-noise） |
| 8 | car-passenger | 7 | 0.428 | 56 | 是（车内） | 有（qut-noise） |
| 9 | bus-metro-cabin | 6 | 0.494 | 56 |  | 有（demand-train） |
| 10 | cowork | 5 | 0.570 | 52 |  | **没有** |
| 11 | library-classroom | 5 | 0.518 | 48 |  | 有（qut-noise） |
| 12 | home-quiet | 5 | 0.403 | 48 |  | **没有** |
| 13 | street-outdoor | 5 | 0.397 | 56 | 是（街道） | 有（demand-train · qut-noise · tum-wind） |
| 14 | hotel-room | 5 | 0.396 | 48 |  | **没有** |
| 15 | cafe | 4 | 0.469 | 52 | 是（咖啡馆） | 有（demand-train · qut-noise） |

- 前 4：rail-air-cabin · canteen · transit-hub · meeting-room
- 判据读数（不是结论）：与临时选的四个重合 1 个（transit-hub；边界并列范围 1–1），线 3 → 「不一致」

临时选的四个在这张表里的位置：

| 场景 | 名次 | 不过数 | 弱点分 | 训练数据噪声池 |
|---|---|---|---|---|
| 街道（street-outdoor） | 13 / 15 | 5 | 0.397 | 有（demand-train · qut-noise · tum-wind） |
| 车站（transit-hub） | 3 / 15 | 9 | 0.468 | **没有**（等第二批 CochlScene） |
| 车内（car-passenger） | 8 / 15 | 7 | 0.428 | 有（qut-noise） |
| 咖啡馆（cafe） | 15 / 15 | 4 | 0.469 | 有（demand-train · qut-noise） |

## 2. 提升空间表：已有候选（DFN3 低延迟版，不回混）比 GTCRN 好多少

来源：`uplift/gap-uplift.json`（`gap.py uplift uplift/compare/compare.json --card uplift/card-gtcrn/scorecard.json`）。排序：先按判定项里「涨」的个数，再按提升分。数据完整：15 个场景、每个场景有差值的判定项 ≥ 5、配对段数 = 基线段数。只含 DSP 判定项。

| 名次 | 场景 | 判定项里「涨」 | 「退」 | 提升分（0 最小 · 1 最大） | 有值项数 | 临时选定 | 训练数据噪声池 |
|---|---|---|---|---|---|---|---|
| 1 | cowork | 6 | 1 | 0.715 | 11 |  | **没有** |
| 2 | car-passenger | 6 | 1 | 0.621 | 12 | 是（车内） | 有（qut-noise） |
| 3 | cafe | 6 | 1 | 0.618 | 11 | 是（咖啡馆） | 有（demand-train · qut-noise） |
| 4 | home-family | 6 | 2 | 0.601 | 11 |  | 有（demand-train · qut-noise） |
| 5 | street-outdoor | 6 | 1 | 0.528 | 12 | 是（街道） | 有（demand-train · qut-noise · tum-wind） |
| 6 | canteen | 6 | 2 | 0.491 | 11 |  | 有（demand-train） |
| 7 | home-quiet | 5 | 1 | 0.504 | 10 |  | **没有** |
| 8 | transit-hub | 5 | 3 | 0.467 | 12 | 是（车站） | **没有**（等第二批 CochlScene） |
| 9 | library-classroom | 4 | 0 | 0.566 | 10 |  | 有（qut-noise） |
| 10 | open-office | 4 | 1 | 0.519 | 11 |  | **没有** |
| 11 | hotel-room | 4 | 2 | 0.427 | 10 |  | **没有** |
| 12 | rail-air-cabin | 4 | 3 | 0.351 | 12 |  | **没有** |
| 13 | bus-metro-cabin | 4 | 3 | 0.334 | 12 |  | 有（demand-train） |
| 14 | phone-booth | 3 | 1 | 0.404 | 10 |  | **没有** |
| 15 | meeting-room | 2 | 2 | 0.351 | 10 |  | **没有** |

- 前 4：cowork · car-passenger · cafe · home-family
- 判据读数（不是结论）：与临时选的四个重合 2 个（cafe · car-passenger），线 3 → 「不一致」

临时选的四个在这张表里的位置：

| 场景 | 名次 | 涨 | 退 | 提升分 | 训练数据噪声池 |
|---|---|---|---|---|---|
| 街道（street-outdoor） | 5 / 15 | 6 | 1 | 0.528 | 有（demand-train · qut-noise · tum-wind） |
| 车站（transit-hub） | 8 / 15 | 5 | 3 | 0.467 | **没有**（等第二批 CochlScene） |
| 车内（car-passenger） | 2 / 15 | 6 | 1 | 0.621 | 有（qut-noise） |
| 咖啡馆（cafe） | 3 / 15 | 6 | 1 | 0.618 | 有（demand-train · qut-noise） |

## 3. 两张表放在一起看（只是读数，不是结论）

- 两张表的前 4 有没有交集：没有
- 弱项表前 4 里没有训练噪声池的：rail-air-cabin · transit-hub · meeting-room
- 提升空间表前 4 里没有训练噪声池的：cowork
- 两张表的口径不同：弱项表是 GTCRN 原版在云上 480 段（含两个评分模型）；提升空间表是 exp021 那次本机的 GTCRN 对 DFN3、只有 DSP 项

## 4. 结论（交人定）

（留空）

## 文件

| 文件 | 是什么 |
|---|---|
| `weak/scorecard-no-eat.json` | 弱项表的输入：480 段成绩单去掉吃字项 |
| `weak/gap-weak.md` · `.json` | `gap.py weak` 原样输出 |
| `uplift/in-a/clips.jsonl` · `uplift/in-b/clips.jsonl` | 从 exp021 两份 `clips.jsonl.gz` 里只取 `gtcrn-h-nomix` / `dfn3-nomix` 的行 |
| `uplift/compare/` | `vdn-eval compare --by-scene` 输出 |
| `uplift/card-gtcrn/` | `vdn-eval scorecard --from-clips` 给 GTCRN 一侧出的成绩单（`gap.py uplift` 用它认判定项） |
| `uplift/gap-uplift.md` · `.json` | `gap.py uplift` 原样输出 |
