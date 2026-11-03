# 评审 011 · 处理图描述 + Pipeline 交叉校验 + 会话记录记图（codex · gpt-6-astra · read-only）

> 对象：`git diff 0604143...HEAD`（分支 `feat/processing-graph`，#64）。**最新一轮：第 1 轮（2026-11-02）· 总评「修改」· 待第 2 轮**。
> 累计处理：P0×0、P1×0、P2×3、P3×0 逐条回源码核实，属实 3、驳回 0、无法核实 0。没有新的要人定的事。

## 第 1 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews-graph/processing-graph-codex-1.md`（不入库）。effort medium，耗时 62 秒。行号指评审时的版本。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮 · 留作 issue 的 P2：无（3 条都已修）

- **P2｜`src/types/src/graph.rs:298` 规范文本有结构歧义：名字不转义，带换行的名字能伪造出另一张图的文本与内容 ID**（第 1 轮 #1）
  - 核实：属实。`canonical_text` 原样写 `k.name`；`NodeKind` 是公开结构，名字可以是任意字符串，`test_kind` 也不查名字
  - 处理：名字里 `[a-z0-9_-]` 以外的字节写成 `%XX`（合法名字原样，现有内容 ID 不变）；新增 `valid_name`，`test_kind` 编译期查名字，`ProcessingGraph::check` 与 `match_chain` 对不合法的名字报 `ChainMismatch::BadName`；登记表测试改用 `valid_name`。测试 `graph::tests::names_cannot_forge_canonical_text` 用了 codex 给的反例
- **P2｜`src/types/src/session.rs:170` 「必填」声明与解析不一致：只有格式行的文件也能读成默认头**（第 1 轮 #2）
  - 核实：属实。原来解析结束只查格式号；`HEADER_KEYS` 标了 `optional: false` 的键缺了也能通过
  - 处理：解析时记每个键出现过没有。该格式里的必填键缺了、或任何头部键写了两次，都报错。格式 1 的必填键也是 rate · table · profile，与旧写入行为一致。`session.rs` 头注写明。新增测试 `session::tests::required_keys_must_be_present_once`：逐个删键，删必填键必须拒读、删可缺省键照常能读；每个键写两次都拒读。schema 那边的 `session_key_since_agrees_with_parser` 改成带上该格式全部键再比
- **P2｜`src/polish/src/params.rs:206` 参数清单比对只比两个挂点的并集，清单互换、共享参数只挂一边都查不出来**（第 1 轮 #3）
  - 核实：属实
  - 处理：改为逐个挂点比对。频域挂点 = `spectral.rs` 读的四项 + 过渡时长；AGC 挂点 = 本表其余各项 + 过渡时长（读源码：`spectral.rs` 只读 smooth · floor_db · comfort_db · demuffle_max_db，`agc.rs` 只读 agc · agc_target_dbfs · agc_range_db）

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：5 条
- 无法核实：1 条（CI 全绿与评测一致没有日志）。本机已重跑：`cargo run -p xtask -- ci` 全绿；`eval ci` 与 main 的对比见 `state/handoff/graph.md` 第 9 项

### 是否再审

本轮无 P0/P1，只是连续第 1 轮 → 再审一轮确认。
