# 评审 062 · 代码卫生：单文件 600 行 · 采样率只写一处 · 评测阈值正本（codex · gpt-6-astra · read-only）

> 对象：`feat/code-hygiene` 相对 main 的 diff（issue #62）。**最新一轮：第 1 轮（2026-11-02）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×1、P2×2，逐条回源码核实，属实 3、驳回 0、无法核实 0。
> 编号用 issue 号（本线没有 plan 文档）。

## 第 1 轮（2026-11-02）· 总评「修改」

- 原文：`_tmp/reviews/hygiene-p1-codex-1.md`（不入库）
- 对象：`origin/main...HEAD`（当时只有第一段：R-SIZE 规则、cli 拆分与改名）
- effort medium，耗时 50 秒
- 本轮 P0/P1：1 条。停止线：未满足
- 留作 issue 的 P2：无

- **P1｜src/xtask/src/file_size.rs:123 基线被人手改大也能过棘轮**（第 1 轮 #1）
  - 核实：属实。原逻辑只拿现状比基线，基线本身由提交者写；把 700 改成 900，800 行的文件照样通过。
  - 处理：和目标分支的基线比对（CI 用 `GITHUB_BASE_REF`，浅检出时先 fetch；本机用 `origin/main`）。新增或调大的条目，行尾必须写 `# 放宽：<理由>`，否则红。补了负例 `widening_needs_a_reason`。
  - 后续：人定 2026-11-02「所有代码文件都不超过 600 行，不设基线、不留豁免」（CLAUDE.md 第五节第 7 条）。基线与放宽机制会在收尾时整体删掉，R-SIZE 改成硬上限。这条修复届时作废。
- **P2｜src/xtask/src/file_size.rs:59 重复路径按第一条取值**（第 1 轮 #2）
  - 核实：属实。
  - 处理：重复路径一律报错，两条都不采信，结果不依赖顺序。路径必须以 `src/` 开头，不许出现 `./`、`..`、`\`。补了测试 `duplicate_baseline_paths_are_flagged_in_any_order` 和 `non_canonical_paths_are_flagged`。
- **P2｜src/xtask/src/file_size.rs:116 扫描失败被当成文件不存在**（第 1 轮 #3）
  - 核实：属实。原来用 `.ok()?` 吞掉读错误，`rs_files` 也吞掉读目录错误。
  - 处理：R-SIZE 用自己的 `walk`，读错误照实报。`size-baseline` 遇到任何错误都不写回。补了测试 `scan_reports_missing_src`。

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：5 条
