# 评审 011 · 删镜像副本 + 参数描述表 + tidy 两条新规则（codex · gpt-6-astra · read-only）

> 对象：`git diff bfe6266`（分支 `feat/types-single-source`，issue #56；本地 `main` 落后，用分支起点作基准）。**最新一轮：第 1 轮（2026-11-02）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×1、P2×5 逐条回源码核实，属实 6、驳回 0、无法核实 0（其中 P2 #5 部分采纳）。没有新的要人定的事。

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
