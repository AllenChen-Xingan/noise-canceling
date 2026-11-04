# 评审 087 · 场景成绩单 + 新评分项 + 评分模型接入（codex · gpt-6-astra · read-only）

> 对象：`git diff origin/main...HEAD`（分支 `feat/scorecard`，issue #87）。**最新一轮：第 2 轮（2026-11-03）· 总评「修改」· 待第 3 轮**。
> 累计处理：P1×5、P2×4 逐条回源码核实，属实 9、驳回 0。

## 第 1 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/scorecard/reviews/scorecard-codex-1.md`（不入库）。effort medium，耗时 87 秒。行号指评审时的版本。
本轮 P0/P1：3 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜`src/eval/src/scorecard/mod.rs:170` 读不了的段、展开不出片段的任务会从成绩单里消失**（第 1 轮 #1）
  - 核实：属实。失败只进 `skipped`，`summarize` 只按有分数的段分组，整个场景读不了时成绩单写「没有不过或待测的场景」
  - 处理：新增 `scorecard::Missing`（场景 · 档 · 设备 · 系统 · 段 id · 原因），`run` 返回它；`rules::summarize(rows, missing)` 让缺段的组合最多判「待测」，一段都没打成的组合也出一行（待测）；Markdown 写「缺 N 段」和文末「没跑成的」。测试：`rules` 的 `missing_clips_block_pass_and_empty_combo_still_listed`、e2e 的 `task_without_clips_still_appears`、`hash_mismatch_is_skipped_not_scored`（补断言：缺段场景不是「过」）
- **P1｜`src/eval/src/scorecard/mod.rs:55` 四条轨长度不符被静默截短**（第 1 轮 #2）
  - 核实：属实。原来取四轨最短长度裁齐，不看 `dur_samples`
  - 处理：四条轨都必须等于清单 `dur_samples`，不等就报错、该段记进 `Missing`。测试：e2e `track_length_mismatch_is_reported_not_truncated`
- **P1｜`src/eval/src/verifiers/speech.rs:278` 干净参考全零时 V-ESTOI 判「过」**（第 1 轮 #3）
  - 核实：属实。全零参考的各帧能量相等，去静音逻辑全保留，两次 ESTOI 都是 0、Δ = 0 过线
  - 处理：`v_estoi` 先查参考是否全零 → 判「不知道」；`estoi()` 在干净轨最响帧也是静音时返回 `None`。测试：`silent_reference_is_unknown` 加 V-ESTOI、`estoi::tests::silent_clean_is_none`
- **P2｜`src/eval/src/scorecard/rules.rs:281` 两个评分模型的数值在报告里看不到**（第 1 轮 #4）
  - 核实：属实（听感一行只有计数）
  - 处理：`ListenCell` 加 `mean_scoreq` · `mean_distill`（各自 Δ 均值），Markdown 听感一行显示。听感不过时按声音分组比较未做：P2，记进现状页「转交」。测试：`listen_means_are_reported`
- **P2｜`src/eval/tests/scorecard_standard.rs:138` 文档比对只取过关线格里的第一个数，不查不等号方向**（第 1 轮 #5）
  - 核实：属实（V-SIM-GATE 的「词首 ≤ 0.2」没核对）
  - 处理：改为格里的全部数按顺序等于任务参数（没给参数的等于代码实际用的那个），并查 ≤ / ≥ 与登记表的好坏方向一致。变异核对：把「词首 ≤ 0.2」改成 0.3，测试红（「V-SIM-GATE：scorecard.md 写 0.3，代码用 0.2」），改回

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：3 条
- 无法核实：上游 `sqa.py` · pystoi 0.4.1 · 012 · 021 / 022 原表不在它的补读范围。对拍已实测：ESTOI 与 pystoi 差 0.00000（5 段），SCOREQ 差 0.00000、Distill-MOS 差 0.00225（6 段）

### 是否再审

有 P1 → 改完开第 2 轮。

## 第 2 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/scorecard/reviews/scorecard-codex-2.md`（不入库）。effort medium。
上一轮处理的验证：codex 抽查确认第 1 轮 #1、#2、#3、#5 已修；#4 只修了一半（见本轮 #3）。
本轮 P0/P1：2 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜`src/eval/src/qa/nrmos_v.rs:147` 带噪原声分按「片段 id + 模型」缓存，两个 build 的同名片段会串**（第 2 轮 #1）
  - 核实：属实。`SceneClip.id` 只在一个 build 内唯一；吃字的参考转写缓存（按任务 id）有同样的问题
  - 处理：原声分改按（混合音频内容哈希, 模型）缓存；成绩单展开出的任务 id 改为「片段 id + 混合轨文件哈希前 8 位」，吃字缓存随之分开
- **P1｜`src/eval/src/scorecard/rules.rs:204` 有段听感判不了时，只要别的段过，听感就判「过」**（第 2 轮 #2）
  - 核实：属实，与 scorecard.md 第四节「有待测 → 待测」冲突
  - 处理：听感：任一段不过 → 不过；没有不过但有判不了的 → 待测；全过才过。scorecard.md 第四节第 2 条写明（与别的项不同的理由：听感每段都该判得了）。测试：`listen_with_unknown_clip_is_not_pass`
- **P2｜`src/eval/src/scorecard/rules.rs:308` 第 1 轮 #4 没修完：听感不过不做声音分组，看不到逐模型的汇总**（第 2 轮 #3）
  - 核实：属实
  - 处理：`ListenCell.models` 给两个模型各一份逐项汇总（过 / 不过 / 待测、均值、最差段）；听感不过时，两个模型也进声音分组比较。scorecard.md 第五节加第 0 条。测试：`listen_failure_gets_sound_split`
- **P2｜`src/eval/tests/nrmos_parity.rs:110` 对拍不核条数，少回几行也绿**（第 2 轮 #4）
  - 核实：属实（`zip` 按短的一边结束）
  - 处理：先断言条数等于段数，再逐行核对 `wav` 字段是哪一段。这条测试要本机模型（ignore），本轮只编译过、没重跑（跑评分模型要名额）

### 引用抽查

- codex 列的「核实属实」：5 条（含第 1 轮修复的 4 条验证）
- 无法核实：同第 1 轮

### 是否再审

有 P1 → 改完开第 3 轮。
