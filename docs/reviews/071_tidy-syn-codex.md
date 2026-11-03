# 评审 071 · tidy 规则迁到 syn 语法树（codex · gpt-6-astra · read-only）

> 对象：`feat/tidy-syn` 相对 main 的 diff（issue #71）。**最新一轮：第 1 轮（2026-11-03）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×4、P2×4，逐条回源码核实，属实 8、驳回 0、无法核实 0。编号用 issue 号。

## 第 1 轮（2026-11-03）· 总评「修改」

- 原文：`_tmp/reviews/tidysyn-codex-1.md`（不入库）
- 对象：`origin/main...HEAD`（b87fe774）
- effort medium
- 本轮 P0/P1：4 条。停止线：未满足
- 留作 issue 的 P2：无
- 修复提交：6aa76575

- **P1｜ast/modtree.rs:168 测试范围只记行号**（#1）
  - 核实：属实。`#[cfg(test)] fn t() {} fn p(){std::fs::read("x");}` 整行算测试。
  - 处理：区间记（行，列），按位置判（`in_ranges`）。R-MODEL、R-PARAM、R3 都改按位置判。补负例：模块树与 R3 各一条同行用例。
- **P1｜ast/modtree.rs:252 内联模块忽略自身的 `#[path]`**（#2）
  - 核实：属实。
  - 处理：内联模块带 `#[path]` 时，子模块目录 = 当前模块目录 + 该路径。补负例 `inline_module_path_attr`。
- **P1｜param_range.rs:49 建树错误没接进闸门**（#3）
  - 核实：属实。`Scope.problems` 与 `orphans` 没人输出。
  - 处理：新增 `modtree::report`，tidy 调它：建树错误报红，孤儿报注释。补完整入口测试 `report_surfaces_problems_and_orphans`。
- **P1｜realtime_calls.rs:129 别名表跨作用域污染**（#4）
  - 核实：属实。测试模块里 `use crate::fake as std;` 会改写生产代码的 `std::fs::read`。
  - 处理：别名表只用非测试代码里的 `use`；路径不管还原前还是还原后，只要有一个以禁用前缀开头就报。补负例两条。
- **P2｜tidy.rs:167 R3 没跳过整份测试文件**（#5）
  - 核实：属实。
  - 处理：新增 `realtime_calls::check_file`，先查模块树。补负例 `check_file_skips_whole_test_files`。
- **P2｜rate_literal.rs:81 整数除法按浮点算**（#6）
  - 核实：属实。`96001u32 / 2` 被算成 48000.5。
  - 处理：整数与浮点分开折叠，整数除法截断。补正反例。
- **P2｜no_mirror.rs:103 trait 指纹只有方法名**（#7）
  - 核实：属实，也与 012 设计不符（codex 抽查列为不符）。
  - 处理：指纹带上规范化的参数类型与返回类型。补正反例。
- **P2｜ast/mod.rs:44 缓存把哈希相等当成源码相等**（#8）
  - 核实：属实。
  - 处理：改用原文当键。

### 引用抽查

- 不符：1 条，即 #7，已修
- 核实属实：2 条
- 无法核实：1 条（polish 默认值不在 codex 可读范围）。这条由 polish 测试改前改后逐条一致覆盖，见 `state/handoff/tidysyn-compare.md` 第 5 节
