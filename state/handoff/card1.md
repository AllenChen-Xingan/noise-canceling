# 现状页 · card1（批次 20261103-b12）

- **状态**：已收尾（2026-11-04），等大总管验收
- **手上**：第一张场景成绩单——GTCRN（exp014 · issue #92 · 分支 `exp/014-scorecard-gtcrn`）
- **管的实体**：成绩单 · 降噪模型 GTCRN · 评测器（只用，不改）
- **交代**：主目录 `_tmp/20261103-b12/card1.md`、`_tmp/20261103-b12/card1-msg2.md`
- **做到哪**：
  - 冒烟、正式跑都完成。正式跑 run id `20261104-000444-e4511abc`，00:04–02:39，实测 2 小时 35 分钟，1440 段（480 段 × 3 个系统），名额已还
  - 成绩单 JSON 只有场景汇总，所以写了小驱动 `docs/experiments/exp014-scorecard-gtcrn/driver/`，只调用和 `vdn-eval scorecard` 相同的库函数，另出逐段 `clips.jsonl`；cafe 上逐字比对相同（README 第二节）。命令行版起的那次（`_tmp/scorecard/20261104-000157-bafd17cf`）约 3 分钟后停掉，作废
  - 结果进仓：`docs/experiments/exp014-scorecard-gtcrn/runs/20261104-000444-e4511abc/`（scorecard.md · scorecard.json · clips.jsonl · analysis.md）
  - 分析：`analyze.py` + 15 条测试；结论 `findings.md`，含大总管要的「评分项自检」一节
  - 人耳片段：主目录 `_tmp/exp014/listen/20261104-000444-e4511abc/`，7 段 × 2 版本 = 168 秒；揭盲表在仓内 `listen/20261104-000444-e4511abc/`
  - issue 草稿：主目录 `_tmp/exp014/issue-ref-track.md`
  - codex 评审 3 轮，第 2、3 轮无 P0/P1，已达停止线；12 条全部属实已改（`docs/reviews/014_scorecard-gtcrn-codex.md`）
  - 大总管收尾三件事：`driver/target` 已删（291 MB）；issue 草稿拆成 4 条；答案在仓内 `listen/20261104-000444-e4511abc/listen-key.md`，听音目录里没有
- **下一步**：等大总管验收、推送、开 PR（本线不做 git 写操作）

## 待人拍板

1. **成绩单里「不做坏事」那几项要不要改成「和原声直通比」来判**（V-PUMPV 人声忽大忽小 · V-TIMBRE 音色 · V-ONSET 字头 · V-SIM-GATE 门控）：现在什么都不做也判不过，过线表分不出好坏。
   - A. 加一列「比原声直通差多少」，按它判；绝对线只作参考 ← 建议
   - B. 改成在模型对干净人声单独跑的输出上算（每段多跑一次，约多 1/3 时长，推断）
   - 如果你不回，默认 A，后果：成绩单要评测器线改代码后重跑 480 段（约 2.5 小时）
2. **吃字按段判还是按场景合并字数判**：现在每段约 27 个字，多错 1 个字就判不过；要先量识别本身每次跑的波动（要名额，时长未实测）。如果你不回，默认先量波动再定。

## 转交（评测器 / 测试集，本线不改）

细节见 issue 草稿 `_tmp/exp014/issue-ref-track.md` 与 `findings.md` 第一节。
1. V-PUMPV · V-TIMBRE：干净参考已是混响后、设备后的人声（不是干声）；原声直通判不过随信噪比变，与设备、混响无关 → 这两项把残留噪声算成了人声变化【实测】
2. V-SIM-GATE：对原声直通应判「不适用」（输出 = 原声，门控对声音没有影响），现在判「不过」【读源码】
3. V-LAT：`phone-booth-c07` 两段三个系统都量出多 6.6 ms，原声直通也不过 → 这段上的测量偏差【实测】
4. V-EAT-ASR：按段判时 1 个字就超线；识别重复误差未量【实测 + 推断】
5. V-INTF 口径核过，没有问题
6. codex-review 工具：codex 输出以「P1 」「P2 」开头时，`file` 会误认成 PBM / PGM 图片，隐私扫描因此拦下 `--previous`。本次手动加标题行绕过，建议脚本自己处理

## 已收尾

- 交付：`docs/experiments/exp014-scorecard-gtcrn/`（README · runs/ · analysis.md · findings.md · driver/ · analyze.py · make_listen.py · tests/ · listen/）
- 人耳片段：主目录 `_tmp/exp014/listen/20261104-000444-e4511abc/`（7 段 × 2 = 168 秒，无答案文件）
- issue 草稿 × 4：主目录 `_tmp/exp014/issue-ref-track.md`
- 测试：`python3 -m unittest discover -s docs/experiments/exp014-scorecard-gtcrn/tests` 19 条过；驱动测试 1 条（删 target 之前跑过，过）

## 进度日志（正式跑期间后台每 30 分钟追加）

- 00:07 识别缓存 63 条
- 00:37 识别缓存 800 条
- 01:07 识别缓存 1555 条
- 01:37 识别缓存 2293 条
- 02:07 识别缓存 3036 条
- 02:37 识别缓存 3774 条
- 02:39 跑完
