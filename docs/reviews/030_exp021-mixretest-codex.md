# 评审 030 · exp021 混回重测结论 + 检查表脚本（codex · gpt-6-astra · read-only）

> 对象：`docs/experiments/exp021-gtcrn-aux/findings-mixretest-20261106.md` · `docs/experiments/exp021-gtcrn-aux/checklist.py`。**最新一轮：第 2 轮（2026-11-06）· 总评「修改」· 本轮无 P0/P1，按大总管定的中途评审 1–2 轮停**。
> 累计处理：P1×2、P2×6 逐条回源码核实，属实 8、驳回 0、无法核实 0。中途评审，大总管定 1–2 轮；第 2 轮的 2 条 P2 当场改掉，没留 issue。

## 第 1 轮（2026-11-06）· 总评「修改」

原文：`_tmp/reviews-exp021-mixretest/findings-mixretest-20261106-codex-1.md`（worktree 下，不入库）。类型 research，effort medium，耗时 37 秒。行号指评审时的版本。
本轮 P0/P1：2 条 · 停止线：未满足 · 留作 issue 的 P2：无（4 条都改了）

- **P1｜checklist.py:76 把「没证明变差」判成「不变差」**（第 1 轮 #1）
  - 核实：属实。旧判法：听感分只看上界 ≥ 0，背景稳和旁人只看下界 ≤ 0；区间越宽越容易过。
  - 处理：改成三态 `ci_state`：区间整体在好的一侧 = 过，整体在坏的一侧 = 不过，跨 0 = 待测。文件头和 findings 第一节同步写明口径。加测试 `test_interval_across_zero_is_tbd_not_pass`。重出 `checklist.md`，判定没有变化（本次区间都不跨 0）。
- **P1｜findings 待人拍板第 1 题跳过了 030 原定的「−9 / 停顿 −9」组合**（第 1 轮 #2）
  - 核实：属实。`decision.md:21` 列了这一组；本轮被大总管砍掉，findings 没说。
  - 处理：第六节第 1 题写明这一档没测、被谁砍掉，给出 030 单独测停顿 −9 的字头读数（−0.12 dB，实测），叠加效果标「没测，推断」，建议放宽线之前先测这一档。
- **P2｜缺测可能被静默放行；「新增 / 消失」会混进「不知道」**（第 1 轮 #3）
  - 核实：属实（逻辑上）。旧 `sheet12` 用否决集合做差。本次「不知道」为 0，旧计数没错。
  - 处理：加 `eat_state` / `eat_moves`，按「通过 / 否决 / 不知道」列转移；字头缺分的场景、弱场景缺可懂度判「待测」；检查表加「各项有分的段数」表。加测试 `test_unknown_not_counted_as_new_or_gone`、`test_missing_onset_scene_is_tbd`、`test_missing_weak_scene_estoi_is_tbd`。
- **P2｜checklist.py:38 舍入和 Rust 不同**（第 1 轮 #4）
  - 核实：属实。Python `round` 是银行家舍入，Rust `f64::round` 是半整数远离 0。
  - 处理：改用 `round_half_away`，加测试（4.5 → 5，−4.5 → −5，3.5 → 4）。对本批 1920 段扫了一遍：落在半整数、两种舍入结果不同的段为 0（实测），计数不变。
- **P2｜「只差一点」强于证据**（第 1 轮 #5）
  - 核实：属实。0.07–0.19 dB 是场景均值超线的距离，没有区间，也没有人耳证据。
  - 处理：删掉「只差一点」，改成越线数值 + 「对人耳影响尚无判据，待测」。待人拍板里的听感后果改标「假设，待盲听」。
- **P2｜「大部分来自混回」归因过强**（第 1 轮 #6）
  - 核实：属实。V-NR 只说明客观降噪量变了，分不出主观原因的占比。
  - 处理：改为「混回可能是原因之一」，写明分不出占比、人声电平也在变、要用同一批片段配对盲听验证。

### 引用抽查

- codex 列的「不符」2 条：
  - 「与 rules_v2.rs 同算法」：即 #4，已改。
  - 「逐段相同，新增只有两类数字」：属实，漏了音色偏差（V-TIMBRE）。改为逐项写明：音色偏差算法两次之间改过；核对只覆盖默认、−9、−12 三档。
- codex 列的「核实属实」：4 条。
- 无法核实 1 条（V-NR 场景均值，codex 没打开逐段分数）。数字来自 `checklist.py` 的 `scene_means` 对 `clips.jsonl` 的实测输出，第 2 轮把读数附进依据。

测试（第 1 轮改后）：`test_checklist.py` 14 条，连 `test_analyze` · `test_plot` 共 27 条全过。

## 第 2 轮（2026-11-06）· 总评「修改」

原文：`_tmp/reviews-exp021-mixretest/findings-mixretest-20261106-codex-2.md`（不入库）。带 `--previous` 第 1 轮原文与本记录；另附 `nr-scene-means.md`（降噪量场景均值读数）。effort medium。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1（脚本 `stop-line:`）；大总管定中途评审 1–2 轮，到此停 · 留作 issue 的 P2：无（2 条都改了）

- **P2｜checklist.py:113 缺测只查了整场景缺失**（第 2 轮 #1）
  - 核实：属实。场景里只缺一段时，`pair()` 丢掉那段、照样判过。
  - 处理：加 `one_sided`（只有一边有分的段）和 `guard`：有单边缺测的条目不许判过，最多判待测；两边都没分 = 不考这项。加测试 `test_one_segment_missing_one_side_blocks_pass`（听感分 · 背景稳 · 字头各缺一段）、`test_missing_on_both_sides_is_not_missing`。本批重出检查表，单边缺分 0 段，判定不变。
- **P2｜checklist.py:45 第二处表示没有和 Rust 正本机器比对**（第 2 轮 #2）
  - 核实：属实，违反 `CLAUDE.md` 第五节第 5 条。
  - 处理：加 `RustSource` 两条测试，读 `src/eval/src/thresholds/table.rs`（V-EAT-ASR 的 `fail_chars`）和 `src/eval/src/scorecard/rules_v2.rs`（`eat_chars` 公式、「≥ 线才否决」），和 Python 侧比对；把 `EAT_VETO` 临时改成 6，这条测试确实变红（实测）。顺带：字头 3 dB 线、人声电平 −3 dB 线原来也是手抄常量，改为从每段分数里记的 `threshold` 读（`line_of`，各段不一致就报错）。

### 引用抽查

- codex 列的「不符」：0 条。「核实属实」：8 条。
- 无法核实 1 条：「字头超过 3 dB 线的场景从 1 个变 7 个」。已在 −12 检查表里加这一行（从数据里的线算），读数 1 → 7，和 findings 一致。

测试（第 2 轮改后）：`test_checklist.py` 20 条，连 `test_analyze` · `test_plot` 共 33 条全过。
