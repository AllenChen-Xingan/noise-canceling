# 评审 011 · 删镜像副本 + 参数描述表 + tidy 两条新规则（codex · gpt-6-astra · read-only）

> 对象：`git diff bfe6266`（分支 `feat/types-single-source`，issue #56；本地 `main` 落后，用分支起点作基准）。**最新一轮：第 3 轮（2026-11-02）· 总评「修改」· 已达停止线（第 2、3 轮连续无 P0/P1）**。
> 累计处理：P1×1、P2×13 逐条回源码核实，属实 14、驳回 0、无法核实 0（第 1 轮 P2 #5 部分采纳）。剩余 P2：0（第 3 轮 3 条都改小、直接修了，不开 issue）。没有新的要人定的事。

## 第 1 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews/diff-bfe6266-codex-1.md`（不入库）。effort medium，耗时 76 秒。行号指评审时的版本。评审时计时基准占机，xtask 新文件与穷举解构改动尚未编译。
本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：见 #5 的「挡不住」部分（已写进现状页，第 2 轮后定是否开 issue）

- **P1｜`src/xtask/src/single_source.rs` 新规则误报自身测试**（第 1 轮 #1）
  - 核实：属实（规则只去掉 `//` 注释，测试里写在字符串中的负例 `"(-30.0..=0.0).contains(&p.floor_db)"` 会被当成代码，`repo_passes` 与 `tidy` 都会红）
  - 处理：新增 `code_lines()` 逐行剥掉字符串（含跨行、`r#"…"#`）、字符字面量、块注释（可嵌套、可跨行），R-MIRROR 与 R-PARAM 都改用它；加测试 `code_lines_strip_strings_and_comments_across_lines`，并在正例里加字符串 / 块注释 / 字符字面量 / 生命周期各一条
- **P2｜`src/speaker/src/judge.rs:55` 判断回路参数仍手写**（第 1 轮 #2）
  - 核实：属实（`JudgeParams` 8 个、`EnrollParams` 11 个可调参数没进描述表）
  - 处理：描述表加 `judge.*`（601–608）、`enroll.*`（701–711）共 19 条，新增单位 `Similarity`（余弦）、`Factor`（倍数）；两个结构的 `Default` 从表取，加 `validate()`（穷举解构）/ `clamped()`，`Judge::new` / `Enroller::new` 先夹再用；各加一条同源单测
- **P2｜`src/speaker/src/envelope.rs:208` 能量包络手抄 `detect_db: 9.0`**（第 1 轮 #3）
  - 核实：属实
  - 处理：改用 `DETECT_DB.default`（与 engine 同一条）
- **P2｜`desc.rs` 的 `ENGINE_MAX_ABS` 下限 0 与 engine 断言 `max_abs > 0` 冲突**（第 1 轮 #4）
  - 核实：属实（`src/engine/src/engine.rs:179` `debug_assert!(… cfg.max_abs > 0.0)`）
  - 处理：下限改 1.0（满刻度；更低会把正常语音判为异常）。最小合法值进 engine 的测试待编译后补
- **P2｜R-PARAM 可被跨行、指数写法绕过**（第 1 轮 #5）
  - 核实：属实
  - 处理：部分采纳——`num_at` / `num_ends_at` 认指数（`-6e1`、`6E+1`），加负例测试；跨行不改，按行检查是有意的边界，写进 `single_source.rs` 头注释与现状页「挡不住」清单（常量 / 别名 / 跨行 / 单边界都在里面）
- **P2｜`src/speaker-lab/src/main.rs:165` `--confirm 20.9` 先截断成 20 再过检查**（第 1 轮 #6）
  - 核实：属实
  - 处理：`--confirm` 直接按 u32 解析；测试加 `20.9`、`21` 两个拒收用例

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：4 条
- 无法核实：`FrameCtx::default`、`Stage::set_params` 的正本实现（不在它获准读的范围）——我已读源码核对，见现状页分歧表 #7、#8

### 是否再审

有 P1 → 改完、编译测试全绿后开第 2 轮，提示词附本记录。

## 第 2 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews/diff-bfe6266-codex-2.md`（不入库）。effort medium。第 1 轮输出因开头「P1」被 `file` 误判为位图、隐私扫描拒发，给它加了一行标题后重跑（内容未改）。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮 · 留作 issue 的 P2：无（本轮 5 条都已改）

上一轮处理的验证：codex 抽查 5 条「核实属实」（`--confirm` 整数解析、`detect_db` 引用表、`ENGINE_MAX_ABS.min = 1`、`ReleaseS` 未错接、`SpectralStage` 位置）。

- **P2｜speaker-lab 先按原始窗长建声纹模型、Judge 再夹值，两边分叉**（第 2 轮 #1）
  - 核实：属实（`src/speaker-lab/src/main.rs` 原先先 `Embedder::open(… jp.window_s …)` 再 `Judge::new`）
  - 处理：抽出 `judge_params()`，覆盖默认值后先 `validate()`，越界报错，之后才建任何部件；加测试 `judge_params_parse_and_reject_by_desc_table`（`--win 20` 被拒）
- **P2｜engine 构造只查三个手写下界，`crossfade_s = 2` 时 engine 与 Pipeline 分叉**（第 2 轮 #2）
  - 核实：属实
  - 处理：加 `SafetyConfig::clamped()`（全部数值参数按描述表夹），`Engine::new` 先夹再用；加测试 `engine_clamps_config_by_desc_table`。只加构造期一行，engine 结构不动
- **P2｜放行标记从原行认，字符串里写标记也能放行**（第 2 轮 #3）
  - 核实：属实
  - 处理：词法器改为逐行输出「代码 + 注释」（`lex_lines`），两条规则的放行标记只从注释里认；往上找理由时只走纯注释行与属性行，遇空行停；加两条负例（字符串里的标记）、一条正例（块注释里的理由）
- **P2｜数字解析漏整数后缀与 `-60.`**（第 2 轮 #4）
  - 核实：属实
  - 处理：后缀表补齐 `u8…u128 / i8…i128 / usize / isize`；认无小数位的 `60.`（后面不是 `..` 或方法名）；加负例 `clamp(1u32, 20u32)`、`(1usize..=20usize)`、`clamp(-60., 0.)`，正例 `3.max(confirm_n)`
- **P2｜polish_offline 的 `--ramp` 默认值手抄 1.0**（第 2 轮 #5）
  - 核实：属实
  - 处理：改取 `POLISH_RAMP_S.default`。未加测试：example 里的单测不随 `cargo test` 跑；改后是直接引用正本、没有第二处表示，不需要比对

### 是否再审

本轮无 P0/P1，连续第 1 轮 → 编译测试全绿后开第 3 轮确认。

## 第 3 轮（2026-11-02）· 总评「修改」

原文：`_tmp/reviews/diff-bfe6266-codex-3.md`（不入库）。effort medium，耗时 116 秒。第 2 轮输出同样加了一行标题才过隐私扫描。
本轮 P0/P1：0 条 · 停止线：**满足**（脚本 `stop-line:` 行：本轮与上一轮都无 P0/P1）· 留作 issue 的 P2：无

上一轮处理的验证：codex 抽查 5 条「核实属实」（`judge_params()` 先校验后建部件、`Engine::new` 先夹值、放行标记只从注释读、`60.` 与整数后缀、`--ramp` 默认取表）。

- **P2｜`br#"…"#` 字节原始字符串被当成代码**（第 3 轮 #1）
  - 核实：属实（只认独立的 `r`）
  - 处理：加 `byte_prefix()`，`br"…"` / `br#"…"#` 按原始字符串处理；加正例（字符串里的范围不报）与负例（字节原始字符串里的放行标记不算）
- **P2｜带尾逗号的 `clamp(-60.0, 0.0,)` 漏报**（第 3 轮 #2）
  - 核实：属实
  - 处理：`two_nums_then_close` 接受可选尾逗号；加 `clamp` 与 `=> (…,)` 两条负例
- **P2｜描述条目里夹注释会让该键名解析不出、静默漏查**（第 3 轮 #3）
  - 核实：属实（测试只要求 ≥ 30 条）
  - 处理：`desc_keys` 先剥注释再解析；`check` 比对「解析出的键名数 = `ParamDesc` 常量数」，不等即报错；`real_desc_file_parses` 改成逐条相等；加夹注释与解析失败两条测试

### 是否再审

已达停止线，不再开第 4 轮。第 3 轮修改待编译空档跑 `xtask ci` 验证。
