# 评审 111 · 成绩单评分方法修正：新汇总规则 v2 · #109 延迟测量器 · #110 低采样率（codex · gpt-6-astra · read-only）

> 对象：`git diff 4493f0ce..HEAD`（分支 `feat/scorecard-v2`，issue #111）。依据：调研 025 `decision.md` · 人定 D85（2026-11-04，七题答案）· `specs/rubric/scorecard.md`。
> 本线按大总管安排插两次中途评审（各 1–2 轮，修掉 P0/P1 即继续），收尾那次按连续两轮无 P0/P1 停。
> **最新一轮：中途 2 第 2 轮（2026-11-05）· 总评「修改」· 中途评审按约定 2 轮即止，P1 已全部修掉；收尾评审另跑**。
> 累计处理：P1×7、P2×7 逐条回源码核实，属实 14、驳回 0。新冒出要人定的事并入 `state/handoff/evalfix.md`「待人拍板」。

## 中途 1 · 第 1 轮（2026-11-04）· 总评「修改」

原文：主目录 `_tmp/evalfix/reviews/scorecard-v2-codex-1.md`（不入库）。effort medium，耗时 90 秒。行号指评审时的版本。
本轮 P0/P1：3 条 · 停止线：未满足（中途评审只要求修掉 P0/P1）· 留作 issue 的 P2：无

- **P1｜`scorecard/load.rs:181` 重新汇总丢了缺段记录**（第 1 轮 #1）
  - 核实：属实。`resummarize` 固定传空的 `missing`；原表里因读不了音频而待测的组合会变成过，整组没跑成的组合会从表里消失。exp014 的原表 `missing` 为空，所以本次对照不受影响，但入口不通用
  - 处理：新增 `load::missing_beside`，从 `clips.jsonl` 同目录的 `scorecard.json` 读 `missing`，找不到就报错；`Missing` 加 `Deserialize`。测试 `resummarize_keeps_missing_from_scorecard`：没有成绩单报错；有缺段的组合判待测；整组没跑成的组合照样出一行
- **P1｜`scorecard/rules_v2.rs:103` 没有线时提前返回，漏标只报不判**（第 1 轮 #2）
  - 核实：属实。门控项或旧口径三项全组判不了时，线为空，提前返回让它成了挡住场景的待测
  - 处理：先定判法（只报 / 旧口径 / 区间），再看线。测试 `all_unknown_report_only_item_does_not_block_scene`，先红后绿
- **P1｜`analysis.rs:274` 相关峰 0.1 门槛擅自改了延迟判法**（第 1 轮 #3）
  - 核实：属实。025 ①-1 把「量不准改判不知道」列为要人批准的改动，本线只该修量法
  - 处理：删去 `XCORR_MIN_R`，恢复旧的「峰 > 0 才给滞后」。测试 `low_correlation_still_measures`（与输入无关的噪声也给出一个滞后，不判不知道）
- **P2｜`scorecard/load.rs:155` 真空感一列在不在取决于首段**（第 1 轮 #4）
  - 核实：属实。旧规则从首段取项目清单；首段 V-NR 无值时整列被漏掉
  - 处理：有 V-NR 的段都补一列，没值补「不知道」。测试 `vacuum_column_present_whatever_row_order`。exp014 重新汇总后真空感从每个系统 56 个组合变 60 个，其余计数不变
- **P2｜`scorecard/rules_v2.rs:193` 送人耳只按听感挑最差段**（第 1 轮 #5）
  - 核实：属实。X1 要靠「最差段送人耳」兜住均值掩盖的局部损伤
  - 处理：按区间判的项，最差一段单看就越线的，也送人耳（去重）。测试 `worst_clip_of_judged_item_goes_to_human`，先红后绿

### 引用抽查

- codex 列的「不符」：`load.rs:204` 的往返测试只核截短的一行，不覆盖汇总与缺段 → 已由上面 #1 的新测试补；另有 v1 重新汇总与原 exp014 成绩单 180 个组合逐一相同的实测（大总管验收可重跑）
- codex 列的「核实属实」：4 条（Holm 逐步下降、百分位区间与种子、吃字字数还原与 ≥ 5 字、音色频带截断）

### 是否再审

有 P1 → 改完开第 2 轮，提示词附本记录。

自审另修（codex 之前，同一提交链）：Holm 族把只报不判和没有区间的组合算进族大小，校正偏严 → 只数真按区间判的组合（测试 `holm_family_excludes_report_only_and_no_ci`）。

## 中途 1 · 第 2 轮（2026-11-04）· 总评「修改」

原文：主目录 `_tmp/evalfix/reviews/scorecard-v2-codex-2.md`（不入库）。effort medium，耗时 99 秒。
上一轮处理的验证：codex 抽查确认第 1 轮 #1 #2 #4 #5 已修；#3 的代码已修，但注册表说明没同步（即本轮 #2）。
本轮 P0/P1：1 条 · 中途评审 2 轮即止（大总管 2026-11-04 定）

- **P1｜`analysis.rs:260` 选窗没给滞后搜索留尾部空间**（第 2 轮 #1）
  - 核实：属实。输出与输入等长（延后、尾部截掉）时，4 秒窗盖住整段或贴着末尾，`lag = 1` 就越界退出，搜不到真延迟。原有已知延迟测试把输出加长了 300 样本，漏了这个边界
  - 处理：窗长限到 `|y| − 最大滞后`，起点限到 `|y| − 最大滞后 − 窗长`（放不下先往前挪，再缩短）。测试 `equal_length_delayed_output_short_and_loud_tail`（3 秒等长 · 8 秒末尾最响两种），先红（量出 0）后绿（300）。`spectrum_golden` 对拍与 c07 回归仍过
- **P2｜`verifiers.rs:77` 注册表仍写已撤回的 0.1 门槛**（第 2 轮 #2）
  - 核实：属实
  - 处理：改为「窗口太靠后时往前挪、给滞后留出空间；输入窗全零才判不知道」

### 引用抽查

- 「不符」1 条即本轮 #2，已改；「核实属实」5 条

## 中途 2 · 第 1 轮（2026-11-05）· 总评「修改」

对象同上（`4493f0ce..HEAD`），重点换成第 3 组口径：G-2 B 干净人声那一路 · 量输出四项（`verifiers/pse.rs`）· 025 旁报 · B3 主值修正 · 频谱对拍基准的冻结法。原文：主目录 `_tmp/evalfix/reviews/scorecard-v2-mid2-codex-1.md`（不入库）。effort medium。

- **P1｜`verifiers/pse.rs:211` 全零输出被算成 SI-SDR 200 dB**（中途 2 第 1 轮 #1）
  - 核实：属实。投影与残差都为 0 时走了「完美重建」分支，把用户整段删掉的系统反而得高分、失败率 0
  - 处理：投影能量为 0 记下限 −200 dB（`SI_SDR_CAP`），完美重建仍记 +200。测试 `pse_ovl_zero_output_is_total_failure`
- **P1｜`scorecard/mod.rs:295` 干净人声那一路的音色仍用带噪场景的掩码与扣噪**（中途 2 第 1 轮 #2）
  - 核实：属实。V-TIMBRE 排除原场景旁人有声帧、又从停顿帧扣残留噪声；干净输出的停顿里是主讲人混响尾巴，会被当噪声扣掉，读数还随原场景旁人时间变
  - 处理：`runner::Prepared::clean_view` 给干净人声那一路专用上下文：旁人掩码与「只有噪音」掩码全空（不扣噪，参考与输出对称），主讲人掩码照用真值。测试 `clean_view_timbre_is_zero_and_ignores_interferer_timing` · `clean_view_does_not_subtract_reverb_tail`
- **P2｜`verifiers/pse.rs:144` 整秒网格漏掉跨秒的旁人独说**（中途 2 第 1 轮 #3）
  - 核实：属实
  - 处理：1 s 窗按 100 ms 步长滑动，读数是窗的占比。测试 `leak_finds_window_off_the_second_grid`（时间轴平移 0.3 s 仍检出）
- **P2｜`verifiers/pse.rs:95` 漏了字头 200 ms 内的误删比例**（中途 2 第 1 轮 #4）
  - 核实：属实（025 `notes/group-D.md:50` 要求单列）
  - 处理：V-TSOS 旁报 `onset_frames` · `onset200_pct`（字头 = 前面停了 ≥ 300 ms 的开口）。测试 `tsos_reports_onset_window_share`

### 引用抽查

- 「核实属实」4 条；「无法核实」1 条：引擎重置与延迟对齐在 `system.rs`（codex 未读）。补充：两路都调用 `system::run`，每次新建引擎；对齐由同一个 `run` 做

## 中途 2 · 第 2 轮（2026-11-05）· 总评「修改」

原文：主目录 `_tmp/evalfix/reviews/scorecard-v2-mid2-codex-2.md`（不入库）。上一轮原文以 `mid2-round1-text.md` 传入：原文件开头是「P1」，被 `file` 认成 netpbm 位图、隐私扫描拒发，加一行标题后照发，内容不变。
上一轮处理的验证：codex 抽查确认第 1 轮 #1–#4 都已修。

- **P1｜`verifiers/pse.rs:278` 两人各自桥接停顿会造出不存在的重叠**（中途 2 第 2 轮 #1）
  - 核实：属实。两人每 200 ms 轮流说话时，各自桥接后交集是连续的 1.6 s
  - 处理：先取真实同时有声的帧，再桥接这些重叠帧之间 300 ms 内的停顿。测试 `alternating_turns_make_no_overlap`；注册表同步
- **P2｜`tests/scorecard_e2e.rs:397` 干净人声那条测试允许三项全无分也通过**（中途 2 第 2 轮 #2）
  - 核实：属实
  - 处理：逐项断言三项都在、都出分、读数有限且近 0
- **P2｜`verifiers/physical.rs:71` A2 的电源状态旁报未做**（中途 2 第 2 轮 #3）
  - 核实：属实
  - 处理：新文件 `src/eval/src/power.rs` 读 `pmset -g batt` 第一行，记 `ac` / `battery` / `unknown`（读不到、认不出都记不知道），写进成绩单元数据 `meta.power`。测试 `parses_pmset_first_line`

### 引用抽查

- 「核实属实」5 条；「无法核实」1 条（同第 1 轮，`system.rs` 未读）

## 收尾 · 第 1 轮（2026-11-05）· 总评「修改」· P0×0 P1×0 P2×3

对象：`4493f0ce..4ccf0638`（整条分支的代码）；依据另带结论文档 `docs/experiments/exp014-scorecard-gtcrn/findings.md` 第十一节、全量 run `20261105-005916-4ccf0638` 的 README 与 compare.md、人定 D85。模型 gpt-6-astra · read-only · effort medium，119 s。

- **P2｜`scorecard/load.rs:199` v2 的 clips.jsonl 能用 `--rules v1` 重新汇总，结果并非旧口径**（收尾第 1 轮 #1）
  - 核实：属实。v2 下三项（人声忽大忽小 · 音色偏差 · 弱音丢失）是干净人声那一路的读数，v1 要的是带噪读数
  - 处理：`resummarize` 在 v1 下遇到带 `clean_pass` 的读数即报错，提示用 v1 重跑音频。先写会红的测试 `v1_refuses_rows_scored_on_clean_pass`（红 → 绿）；`specs/rubric/scorecard.md` 第 7 行同步
- **P2｜findings 第十一节不过原因与吃字组合的数字合计对不上，且 compare 里查不到**（收尾第 1 轮 #2）
  - 核实：属实。「9 个带吃字」应为 11；「10 个压力档 + 3 个常见档」应为 8 + 3
  - 处理：`runs/20261105-005916-4ccf0638/compare.py` 加三节（场景总判不过原因分解 · 吃字与人声忽大忽小判不过的组合明细含区间 · 只重新汇总 → 重跑变了的组合），重生成 compare.md；正文按表改为 22 + 7 + 11 = 40、8 压力 + 3 常见、区间下沿 ≤ 0 的 10 个
- **P2｜「三个系统都接近完全不处理旁人」写成实测**（收尾第 1 轮 #3）
  - 核实：属实。漏旁人 ≈ 100% 只说明压低没到 10 dB；同时说话那项 GTCRN 只有 68/242 段全部重叠段不到 1 dB
  - 处理：两项分开写各自的量与实测数；「离把旁人去掉很远」改标【推断】；写明两项尚无判定线

### 引用抽查

- 「核实属实」6 条（场景总判排除只报项 · 重叠先交集后桥接 · e2e 逐项断言 · meta.power · `planned_platform_for` 16 kHz 仍计重采样且旧函数未改 · v2 标准与阈值机器比对）；「无法核实」1 条（测试先红后绿的先后，只能看提交链）

自检：`cargo test -p vdn-eval` 全绿 · `cargo clippy -p vdn-eval --all-targets` 无警告 · `xtask tidy` 通过。大总管已亲手跑全量测试（`--no-fail-fast`）0 红；其间发现 `specs/schema/eval-suite.schema.json` 没随 4 个新项重导出，已用 `cargo run -p vdn-schema` 重导出（只多 4 个项名）并随 `4dd81b48` 提交。
