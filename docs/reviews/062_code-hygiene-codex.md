# 评审 062 · 代码卫生：单文件 600 行 · 采样率只写一处 · 评测阈值正本（codex · gpt-6-astra · read-only）

> 对象：`feat/code-hygiene` 相对 main 的 diff（issue #62）。**最新一轮：第 5 轮（2026-11-02）· 总评「修改」· 待第 6 轮**。
> 累计处理：P1×8、P2×6，逐条回源码核实，属实 14、驳回 0、无法核实 0。
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

## 第 2 轮（2026-11-02）· 总评「修改」

- 原文：`_tmp/reviews/hygiene-p1-codex-2.md`（不入库）
- 对象：`origin/main...HEAD`（含第二段：采样率、R-RATE、test_modules、阈值正本、拆 verifiers / runner）
- effort medium，耗时 71 秒
- 本轮 P0/P1：3 条。停止线：未满足
- 留作 issue 的 P2：无

- **P1｜src/eval/src/thresholds.rs:421 待定项吞掉同一参数上的新错误**（第 2 轮 #1）
  - 核实：属实。原来按 `(id, key)` 比，清单里给 V-NR 加一个 `max = 999` 也会被已有的 V-NR max 待定项放过。
  - 处理：待定项精确到「那组分档」或「那个数」（`Pend::Edges(&[…])` / `Pend::Value(x)`）。不一致按种类区分：分档数值、分档顺序、取值、清单格式。待定从 7 条拆成 8 条（V-NR 的 −8、−10 各一条）。补负例 `pending_does_not_swallow_new_value_on_same_param`。
- **P1｜src/xtask/src/test_modules.rs:24 注释里的模块声明也能放过生产文件；内联模块作用域没认**（第 2 轮 #2）
  - 核实：属实。原来逐行看纯文本。
  - 处理：先用 `lex_lines` 剥掉注释与字符串，只认花括号深度 0 的 `#[cfg(test)]` + `mod x;`；内联模块里的声明不认（宁可多查）。补负例：块注释、字符串、内联模块各一种。
- **P1｜src/eval/src/thresholds.rs:315 校验丢了正负号和档位顺序**（第 2 轮 #3）
  - 核实：属实。`same` 比绝对值，分档只比集合。
  - 处理：数字带符号提取、带符号比较。文档写损失量、代码记负 dB 的那处，用新出处类型 `Src::RubricLoss` 显式登记取负。分档按 verifiers 登记表的好坏方向检查严格单调。补负例 `sign_and_order_are_checked`。
- **P2｜src/xtask/src/rate_literal.rs:103 测试放行一直延伸到文件末尾**（第 2 轮 #4）
  - 核实：属实。`mod tests;` 声明也会开放行区；内联测试模块闭合后也继续放行。
  - 处理：按花括号深度认内联测试模块的边界；`mod x;` 引入的文件改用 `test_modules` 整份认（只限底层 crate）。补负例 `exemption_ends_with_test_module`。
- **P2｜src/xtask/src/rate_literal.rs:112 空理由也能放行**（第 2 轮 #5）
  - 核实：属实。
  - 处理：标记后必须有非空理由。补负例 `empty_allow_reason_is_flagged`。
- **P2｜src/eval/src/thresholds.rs:102 V-SPLVL 的量法说明写错**（第 2 轮 #6）
  - 核实：属实。代码按主讲人有声帧做投影能量损失，`verifiers.rs` 登记写明「不用 P.56」。
  - 处理：改正注释。这样 V-SPLVL 和 rubric C3 的口径其实很接近（差在投影去噪），「量法不同」的保留理由对 V-SPLVL 不成立。已同步改 `state/handoff/hygiene-thresholds.md`，并报告大总管。

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：5 条

## 第 3 轮（2026-11-02）· 总评「修改」

- 原文：`_tmp/reviews/hygiene-p1-codex-3.md`（不入库）
- 对象：`origin/main...HEAD`（第 2 轮修复与 V-SPLVL 分档改为 −1 / −2 / −4 之后；不含之后合进来的 #66）
- effort medium
- 本轮 P0/P1：2 条。停止线：未满足
- 留作 issue 的 P2：无

- **P1｜src/eval/src/thresholds.rs:428 待定匹配没比出错原因**（第 3 轮 #1）
  - 核实：属实。把 V-TIMBRE 的摘录改成不存在的文本，仍得到同样的 `What::Edges([2,4,6])`，会被待定项放过。
  - 处理：新增 `Cause`（摘录缺失 · 无出处 · 数对不上 · 顺序 · 未登记 · 默认值缺 · 清单坏），记在每条不一致上；待定项 `Pend` 同时写原因，匹配时原因也要相同。补负例 `same_value_different_cause_is_red`。
- **P1｜src/xtask/src/test_modules.rs:99 文件被测试引用一次就整份放行**（第 3 轮 #2）
  - 核实：属实。lib.rs 里 `#[cfg(test)] mod worker;`、main.rs 里普通 `mod worker;` 时，worker.rs 会被当成测试代码。
  - 处理：再收集「不在测试清单里的文件」的普通 `mod x;` 声明，它们指到的模块文件（`x.rs` / `x/mod.rs`）从测试清单里去掉；内联模块里的 `mod x;` 路径没解析，按同名保守算生产引用。补负例 `file_also_used_by_production_entry_is_not_exempt`。
  - 自查发现并修掉一个回归：生产引用原来按整棵 `x/` 子目录去排除，会把 `x/` 下面真正的测试子模块也排除掉。改成只指到模块文件本身；正例 `cfg_test_module_files_are_exempt_from_r3` 覆盖。
- **P2｜src/xtask/src/rate_literal.rs:126 测试模块闭合行整行放行**（第 3 轮 #3）
  - 核实：属实。`mod tests {} const SR: u32 = 48000;` 会整行放过。
  - 处理：逐字符划分测试区，测试部分换成空格后再查；同一行闭合之后的代码照查。补负例：同一行闭合后声明常量、同一行闭合后打开生产模块。

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：6 条
- codex 列的「无法核实」：1 条（rubric C3 原文不在它可读的范围内）。已由 `src/eval/tests/thresholds.rs` 的摘录逐字比对覆盖

## 第 4 轮（2026-11-02）· 总评「修改」

- 原文：`_tmp/reviews/hygiene-p1-codex-4.md`（不入库）
- 对象：`origin/main...HEAD`（第 3 轮修复之后，含 docs/experiments 扫描）
- effort medium
- 本轮 P0/P1：1 条。停止线：未满足
- 留作 issue 的 P2：无

- **P1｜src/xtask/src/test_modules.rs:124 生产引用的排除没有往下传递**（第 4 轮 #1）
  - 核实：属实。worker.rs 被移出测试清单后，它普通引用的 `worker/child.rs` 还留在清单里。
  - 处理：生产引用的排除反复做，直到清单不再变化；真正带 `#[cfg(test)]` 的子模块照样保留。补负例 `production_exclusion_is_transitive`（三级引用，加一层孙模块）。

### 引用抽查

- codex 列的「不符」：1 条（本记录第 3 轮 #2 的处理说明没写到传递引用），即本轮 #1，已修
- codex 列的「核实属实」：4 条

## 第 5 轮（2026-11-02）· 总评「修改」

- 原文：`_tmp/reviews/hygiene-p1-codex-5.md`（不入库）
- 对象：`origin/main...HEAD`（第 4 轮修复之后）
- effort medium
- 本轮 P0/P1：1 条。停止线：未满足
- 留作 issue 的 P2：无

- **P1｜src/xtask/src/test_modules.rs:130 内联模块里的生产引用路径拼错**（第 5 轮 #1）
  - 核实：属实。worker.rs 里 `mod inner { mod child; }` 指到的是 `worker/inner/child.rs`，原来只按名字拼成 `worker/child.rs`，真文件漏掉了。第 3 轮记录里「按同名保守算」的说法不成立（codex 抽查列为不符）。
  - 处理：`mod_names` 跟踪外层内联模块的路径，生产引用按 Rust 的模块路径规则解析成 `inner/child.rs` 或 `inner/child/mod.rs`。补负例 `production_reference_inside_inline_module_is_excluded`，并验证 worker.rs 的测试子模块仍算测试代码。

### 引用抽查

- codex 列的「不符」：1 条（第 3 轮处理说明「按同名保守算」），即本轮 #1，已修
- codex 列的「核实属实」：5 条

## 第 5 轮之后：test_modules 改成窄规则（大总管 8d 定，2026-11-02）

第 3–5 轮的 P1 都出在 `test_modules.rs`（tidy 认哪些文件是测试代码）。宽规则认「任何 `#[cfg(test)] mod x;` 引入的文件」，每补一个绕过点，代码就更复杂。大总管决定改成窄规则，原来跑到一半的第 6 轮（审旧逻辑）停掉作废：

- 新规则：只有父模块顶层写了 `#[cfg(test)] mod tests;`、文件名是 `tests.rs` / `tests/mod.rs` / `tests/` 下的文件，才整份按测试代码处理；同目录有普通 `mod tests;` 引用就不放行。其余一律照实时代码查，需要例外的文件用文件头整模块 NRT 标注
- 删掉：生产引用排除、反复排除到不再变化、内联模块路径跟踪
- `test_modules.rs` 287 → 195 行
- 负例：名字不是 tests 的 `#[cfg(test)] mod helper;` 照查；没有 cfg(test) 的 `mod tests;` 照查；注释 / 字符串 / 内联模块里的写法、另一个入口的普通引用，都不放行
