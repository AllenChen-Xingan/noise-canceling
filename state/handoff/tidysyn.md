# 现状 · tidysyn（issue #71 · 已合并 PR #77，2026-11-03）

大总管：noise-canceling-8d。实体：xtask（tidy 闸门）；为默认值字面量（P3）临时扩到 vdn-polish · vdn-scene · vdn-speaker-lab。

## 结果

- `xtask tidy` 从逐行文本匹配改成 syn 语法树判。做了什么、迁移前后差异、时间、测试，见 `state/handoff/tidysyn-pr.md`
- 迁移前后对真仓的逐条比对：`state/handoff/tidysyn-compare.md`
- codex 换谱系评审 8 轮：`docs/reviews/071_tidy-syn-codex.md`。没达到「连续两轮无 P0/P1」的停止线，人定 2026-11-03 提前合并
- 已知绕过写法 4 种，转 issue #76（草稿 `state/handoff/tidysyn-bypass-issue.md`）
- 设计与实现的差异：`docs/plan/012_tidy-syn.md` 第十节；ADR 004 已修订

## 留给后来的人

- 新加 tidy 规则：按语法树写，放 `src/xtask/src/<查什么>.rs`，测试放同名 `_tests.rs`
  - 共用 `ast::parse_src`（有缓存）与 `ast::modtree`（测试代码判定，精确到行和列）
  - 放行标记写在行尾注释里，用 `scan::lex_lines` 找
- 规则文件头都写了「边界」：哪些写法查不到。改规则时一并更新
- 教训（#62、#71 都踩过）：别「找到一种写法补一种」。同一类问题第三次出现，就换成走遍全部节点的通用访问器

## 转交

无。
