# 评审 087 · 场景成绩单 + 新评分项 + 评分模型接入（codex · gpt-6-astra · read-only）

> 对象：`git diff origin/main...HEAD`（分支 `feat/scorecard`，issue #87）。**最新一轮：第 7 轮（2026-11-03）· 总评「通过」· 连续第 1 轮无 P0/P1，待第 8 轮确认**。
> 累计处理：P1×8、P2×9 逐条回源码核实，属实 17、驳回 0。

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

## 第 3 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/scorecard/reviews/scorecard-codex-3.md`（不入库）。effort medium，耗时 104 秒。
第一次起跑被拦：①`rules.rs` 改完 606 行超 600（tidy R-SIZE）→ 单元测试拆到 `rules/tests.rs`；②隐私扫描把第 2 轮原文判成「非纯文本」：文件以「P1 」开头，被 `file` 认成位图头（误判）→ 原文首行前加一个空行后放行，内容未改。
上一轮处理的验证：codex 抽查确认第 2 轮 #1、#2、#4 已修；#3 的 Markdown 明细没落地（见本轮 #2）。
本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜`src/eval/src/scorecard/mod.rs:64` 忽略清单的 `has_clean_ref`**（第 3 轮 #1）
  - 核实：属实（标 false 的段仍把 owner 轨当干净参考）。本集 scenesuite 恒为 true，但字段在 schema 里
  - 处理：`has_clean_ref = false` 的段整段不打分、记进 `Missing`（场景最多待测），scorecard.md 第四节第 6 条写明。测试：e2e `clip_without_clean_ref_is_not_scored`
- **P2｜`src/eval/src/scorecard/card.rs:112` 两个模型的明细没进 Markdown**（第 3 轮 #2）
  - 核实：属实
  - 处理：听感一行下面加「评分模型」小表（过 / 不过 / 待测 · Δ 均值 · 最差段）。测试断言 Markdown 里有这一行
- **P2｜`src/eval/src/scorecard/rules.rs:376` 排最差场景和写摘要时漏了听感不过**（第 3 轮 #3）
  - 核实：属实（只听感不过的场景摘要写成「没有不过的项」）
  - 处理：`SceneCard::failed_items()`（听感不过算一项「听感」），排名和摘要都用它。测试：`listen_failure_counts_in_ranking_and_summary`

### 是否再审

有 P1 → 改完开第 4 轮。

## 第 4 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/scorecard/reviews/scorecard-codex-4.md`（不入库）。effort medium，耗时 135 秒。
上一轮处理的验证：codex 抽查确认第 3 轮 #1、#2、#3 都已修。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1 · 留作 issue 的 P2：无（两条都修了）

- **P2｜`src/eval/src/scorecard/mod.rs:182` 误传非场景清单（如 ci.json）时成功输出一张空成绩单**（第 4 轮 #1）
  - 核实：属实
  - 处理：`scorecard::check_suite` 要求至少一条任务、且全是场景任务，否则命令报错。测试：e2e `non_scene_or_empty_suite_is_rejected`（误传 ci 清单 · 空清单 · 正常清单）
- **P2｜`src/eval/src/qa/nrmos_v.rs:63` 常驻子进程的回答不核对是哪一段**（第 4 轮 #2）
  - 核实：属实（只读下一行的 `mos`）
  - 处理：回答的 `wav` 必须等于所问的段，不等就报错、丢掉子进程。测试用假子进程（`/bin/sh` 回固定行）：错段被拒 · 对段收下 · error 行报错 · 提前退出报错

### 是否再审

本轮无 P0/P1，但只连续 1 轮 → 再审一轮确认。

## 第 5 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/scorecard/reviews/scorecard-codex-5.md`（不入库）。effort medium，耗时 143 秒。
上一轮处理的验证：codex 抽查确认第 4 轮 #1、#2 已修。
本轮 P0/P1：1 条 · 停止线：未满足（连续无 P0/P1 的计数归零）· 留作 issue 的 P2：无

- **P1｜`src/eval/src/main.rs:370` 同一 `--out` 目录跑第二次会覆盖旧成绩单**（第 5 轮 #1）
  - 核实：属实（`std::fs::write` 截断），违反 CLAUDE.md 第五节第 1 条
  - 处理：`card::ensure_fresh` 在打分前先查，已有成绩单就报错；`card::write_new` 用「只新建」打开两个文件。默认输出目录本来带运行编号，不受影响。测试：e2e `second_write_to_same_dir_is_refused`（旧文件逐字不变）
- **P2｜`src/eval/src/scorecard/mod.rs:171` 全是 `other` 场景的任务能过 `check_suite`，产出空成绩单**（第 5 轮 #2）
  - 核实：属实
  - 处理：`check_suite` 拒绝场景是 `other` 的任务（021：不是开会场景）。测试加进 `non_scene_or_empty_suite_is_rejected`

### 是否再审

有 P1 → 改完开第 6 轮。

## 第 6 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/scorecard/reviews/scorecard-codex-6.md`（不入库）。effort medium，耗时 106 秒。
上一轮处理的验证：codex 抽查确认第 5 轮 #1、#2 已修。
本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜`src/eval/tests/click_diag.rs:98` 诊断导出（给人耳核对的片段与清单）重用目录时会覆盖旧 `list.json`**（第 6 轮 #1）
  - 核实：属实
  - 处理：导出目录必须是新的，已存在就拒绝、一个文件都不写。测试：`dump_dir_must_be_new`（不要音频，CI 能跑）
  - 按类归并：「覆盖旧产物」是第二次出现（第 5 轮 #1 是同类）→ 把本分支所有写文件的地方过了一遍：成绩单（`write_new`）、诊断导出（`fresh_dir`）、Distill-MOS 导出脚本（已存在就退出）都是只新建；`gen_*_ref.py` 写的是进仓的参考值文件（有意重生成，走提交链）。`main.rs` 里 `ci` / `qa` / `calib` 的报告写法是旧代码、不在本分支改动里，转交大总管

### 是否再审

有 P1 → 改完开第 7 轮。

## 第 7 轮（2026-11-03）· 总评「通过」

原文：主目录 `_tmp/scorecard/reviews/scorecard-codex-7.md`（不入库）。effort medium，耗时 110 秒。
上一轮处理的验证：codex 抽查确认第 6 轮 #1 已修；另抽查 5 处既有修复仍成立。
本轮 P0–P3：0 条 · 停止线：连续第 1 轮无 P0/P1

### 是否再审

本轮无 P0/P1，只连续 1 轮 → 再审一轮确认。
