# 评审 111 · 成绩单评分方法修正：新汇总规则 v2 · #109 延迟测量器 · #110 低采样率（codex · gpt-6-astra · read-only）

> 对象：`git diff 4493f0ce..HEAD`（分支 `feat/scorecard-v2`，issue #111）。依据：调研 025 `decision.md` · 人定 D85（2026-11-04，七题答案）· `specs/rubric/scorecard.md`。
> 本线按大总管安排插两次中途评审（各 1–2 轮，修掉 P0/P1 即继续），收尾那次按连续两轮无 P0/P1 停。
> **最新一轮：中途 2 第 1 轮（2026-11-05）· 总评「修改」· P1 已修，开第 2 轮确认**。
> 累计处理：P1×6、P2×5 逐条回源码核实，属实 11、驳回 0。新冒出要人定的事并入 `state/handoff/evalfix.md`「待人拍板」。

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
