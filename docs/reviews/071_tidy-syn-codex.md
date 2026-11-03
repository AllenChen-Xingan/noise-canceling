# 评审 071 · tidy 规则迁到 syn 语法树（codex · gpt-6-astra · read-only）

> 对象：`feat/tidy-syn` 相对 main 的 diff（issue #71）。**最新一轮：第 2 轮（2026-11-03）· 总评「修改」· 待第 3 轮**。
> 累计处理：P1×7、P2×7，逐条回源码核实，属实 14、驳回 0、无法核实 0。编号用 issue 号。

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

## 第 2 轮（2026-11-03）· 总评「修改」

- 原文：`_tmp/reviews/tidysyn-codex-2.md`（不入库）
- 对象：`origin/main...HEAD`（4a00d5eb）
- effort medium
- 本轮 P0/P1：3 条。停止线：未满足
- 留作 issue 的 P2：无
- 修复提交：见下一个提交

- **P1｜ast/modtree.rs:189 函数体里的模块没进模块树**（#1）
  - 核实：属实。`fn f() { #[path = "shared.rs"] mod p; }` 这条生产引用被漏掉，shared.rs 被整份当成测试代码。
  - 处理：函数体扫描（`BodyScan`）收集块里的模块。内联的照常展开；带 `#[path]` 的按「文件目录」和「模块目录」两种基准都当成引用，宁可多查。补负例 `block_module_with_path_is_production`。
- **P1｜realtime_calls.rs:144 第 1 轮 #4 没修完：同名别名只取第一个**（#2）
  - 核实：属实。
  - 处理：不解析作用域，同名别名的所有候选路径里有一个命中就报。补负例 `same_alias_in_different_scopes`。
- **P1｜ast/modtree.rs:254 漏了生产代码里的 `include!`**（#3）
  - 核实：属实，012 第四节写了要报红。
  - 处理：模块树在非测试位置（条目级与函数体里）遇到 `include!` 报红。真仓没有 `include!`（只有引入数据的 `include_str!`）。补负例 `production_include_is_an_error`。
- **P2｜param_range.rs:235 `else if` 沿用第一个条件的上下文**（#4）
  - 处理：`visit_expr_if` / `visit_expr_while` 让每个条件单独算一条。补负例。
- **P2｜ast/mod.rs:121 R-MODEL 漏了字节字符串**（#5）
  - 核实：属实，旧规则查得到，属于迁移退步。
  - 处理：`b"…"` 也收。补负例。
- **P2｜realtime_calls.rs:89 路径没去掉 `r#`**（#6）
  - 处理：路径与 use 各段一律 `unraw`。补负例两条。

### 引用抽查

- 核实属实：4 条
- 无法核实：1 条（同第 1 轮，polish 测试改前改后一致已覆盖）
