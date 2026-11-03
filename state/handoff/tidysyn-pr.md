# PR 草稿 · #71 tidy 规则迁到 syn 语法树（tidysyn 写，大总管推送前过目）

## 做了什么

把 `xtask tidy` 的检查从「逐行文本匹配」改成「用 `syn` 解析 Rust 语法树」。人原话：「可以真正用ast工具」。设计见 `docs/plan/012_tidy-syn.md`，第十节写了实现与设计的差异。

1. **测试代码识别改用模块树**（`src/xtask/src/ast/modtree.rs`）
   - 从每个 crate 的编译根出发，跟 `mod` 声明走，按 Rust 的模块路径规则找文件：内联模块、`#[path]`、`cfg_attr` 里的 path（含嵌套）、`r#name`、函数体或常量块里的模块
   - 一个文件只要有一条非测试路径能到达，就算生产代码
   - 测试范围精确到（行，列）
   - 替换了 #65 的文本版 `test_scope.rs`（已删），也恢复了 #62 放弃的「测试模块自动识别」
   - 建树错误（找不到模块文件、`#[path]` 跳出 crate、生产代码里的 `include!`、语法错）报红；孤儿 `.rs` 报注释（已定）
2. **五条规则迁到语法树**：
   - **R-RATE**（`rate_literal.rs`）：按数值认 16000 / 48000，写法不限（含十六进制、宏参数），并对两边都是字面量的算式做常量折叠（整数除法截断）
   - **R3**（`realtime_calls.rs`）：路径还原同文件的 `use` 别名后判（别名链递归、`r#`、宏参数里的、函数当值引用都认）；测试代码、整份测试文件跳过
   - **R-PARAM**（`param_range.rs`）：在「同一条语句」里找参数名（rustfmt 拆行也查得到；带块的只看头部，遇到块边界就停）。新增 P3：非测试代码里等于描述表默认值的字面量报红，放行写 `// tidy: 预设值，不随默认变：<理由>`
   - **R-MIRROR**（`no_mirror.rs`）：同名之外，新查「改名但结构相同」的副本（结构指纹；字段、变体或方法 ≥ 3 个）
   - **R-MODEL**（`model_name.rs`）：按语法树查字符串、字节字符串、C 字符串字面量（含宏与属性里的；跳过文档注释）
3. **P3 在真仓查出的 6 处已逐条处理**（大总管定在本 PR 修，实体临时扩到 polish / scene / speaker-lab）：
   - 3 处改成引用默认值（polish 预设的 AGC 三项）
   - 3 处是预设、碰巧等于默认，保留并写理由（polish「只开 AGC」的开关值、scene 训练产出的滞回参数、speaker-lab 激进版慢收）
   - 依据见 `state/handoff/tidysyn-compare.md` 第 5 节
   - 三个 crate 的测试改前改后逐条一致
4. **性能**：调试构建里给 `syn` / `proc-macro2` / `quote` 开 `opt-level = 3`（根 `Cargo.toml`）；同一次 tidy 里每份源码只解析一次、模块树只建一次
5. **ADR 004 修订**：记触发、选 syn、不选 ast-grep / dylint 的理由（`docs/research/008_rust-architecture/adr/004-ast-guard-not-generate.md`）
6. `scan.rs` 只剩找行尾放行注释的部分。注释不在语法树里，仍逐行找

## 新规则比旧规则准的地方（单列）

- `#[cfg(all(test, debug_assertions))]` 认作测试代码（`denoised/src/lib.rs:171–173`）。旧规则只认字面写的 `#[cfg(test)]`，属于漏报
- 外置测试文件（`#[cfg(test)] mod tests;` 引入的）认作测试代码。旧 R3 认不出
- `use std::fs::File` 之后的 `File` 用法认作 `std::fs`（`shm/src/owned.rs`，NRT 模块，真跑照旧跳过）。旧 R3 只认出 use 行
- 端点是算式的范围（`0..62 * 8`）不再被误当成数字范围。旧 R-PARAM 文本截出中间一段，误报 7 处
- R-PARAM 能查 rustfmt 拆成多行的范围；R-RATE 能查算出来的值与各种进制

## 迁移前后对真仓的比对

逐条记录在 `state/handoff/tidysyn-compare.md`。摘要：

| 规则 | 旧 | 新 | 差异 |
|---|---|---|---|
| 测试范围 | 123 个文件 | xtask 以外 113 个里 111 个一致 | 2 处已解释（上一节第 1 条；一行文档注释） |
| R-MODEL（临时扩大命中面） | 2 | 2 | 一致 |
| R-RATE（不看放行） | 4 | 4 | 一致 |
| R3（不看 NRT） | 7 行 | 12 行 | 新多 6 行（owned.rs 的 File 用法，旧漏报）、少 1 行（外置测试文件，新更准） |
| R-PARAM 键名 / 范围候选 | 58 / 159 | 58 / 152 | 少的 7 处是旧规则误报（算式端点） |
| R-MIRROR | 名单 + 1 处同名 | 同 | 一致；改名副本 0 处 |

## 时间

Mac17,3 · macOS 27.0.1 · 插电 · 中位数：

| 项 | 改前 | 改后 | 预算（已定，超了只报不挡） |
|---|---|---|---|
| xtask 全依赖冷编译 | 1.59 s | 4.48 s | 增加 ≤ 30 s：过。多出的主要是 syn 带来的，包括给它开优化 |
| xtask 本身冷编译 | 0.52 s | 0.92 s | — |
| xtask 热编译 | 0.21 s | 0.26 s | — |
| `xtask tidy` 运行 | 0.563 s | 0.724 s | ≤ 3 s：过 |

## 关联 issue

Closes #71

## 测试（必填）

- **新增的测试**：
  - 012 的负例全部做成测试：测试识别 T1–T14、R-RATE R1–R8、R-PARAM P1–P3（P2 是已知限制，写成 `#[ignore]` 记录）、R-MIRROR M1–M3、R3 S1–S4
  - codex 8 轮评审里修了的每条问题各补负例
  - 禁用清单与 `clippy.toml` 的比对测试
- **跑了哪些**（本机实测）：
  - `cargo test -p xtask`：83 过 / 0 败 / 1 忽略（P2 已知限制）
  - `cargo test -p xtask -p vdn-polish -p vdn-scene -p vdn-speaker-lab`：全过
  - fmt、clippy `-D warnings`、`cargo deny check`、`xtask check-deps`、`xtask tidy`：通过
  - 全量 `xtask ci`：由大总管跑
- **测不了的**：无

## 唯一真相源

- R3 的禁用清单 `BANNED` 与根 `clippy.toml`：比对测试 `banned_list_matches_clippy_toml`
- 默认值：P3 让「等于描述表默认值的字面量」不能再作为第二处表示悄悄存在
- scene 训练脚本 `train.py` 生成 `weights.rs` 时自带放行注释，与默认值的一致性由 `scene/tests/behavior.rs` 比对

## 未做 / 已知限制

- **已知绕过写法 4 种，见 #76**（草稿 `state/handoff/tidysyn-bypass-issue.md`）：
  - `include!` 经 `use` 改名
  - 跨文件 `pub use` 再导出
  - 范围先赋给变量再在别的语句里用
  - 宏展开出的代码
- `cfg(any(test, …))` 一律按生产代码算，宁可多查
- R-RATE 不扫 `docs/experiments/` 下的独立实验程序（它们不依赖 vdn-types）

## 评审

codex 换谱系评审 8 轮，记录在 `docs/reviews/071_tidy-syn-codex.md`：
- 共提出 37 条（P1×14 · P2×23），全部属实；修了 36 条，1 条按人定记入后续 issue
- **没满足「连续两轮无 P0/P1」的停止线**：P1 依次为 4、3、3、1、1、0、1、1，第 3 轮起多是「又一种等价写法能绕过」
- **人定（2026-11-03，大总管 8d 转）有底线地先合**：
  - 等价写法的绕过记入新 issue
  - 比旧规则退步、真实代码上的误报必须修完（第 8 轮的 P1 两样都不是）
- 评审期间模块树的展开改成通用访问器（第 3 轮），不再按位置逐个补

🤖 Generated with [Claude Code](https://claude.com/claude-code)
