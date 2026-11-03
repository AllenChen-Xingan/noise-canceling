# ADR 004 · 「AST 做模块化」：用 AST 守规矩，不用 AST 造代码

- 状态：提议（2026-10-30，rustarch）
- 规约落点：`specs/rust/architecture.md` §5、§6

## 背景
人的原话：「我建议用超级语法树 / ast 来把东西做成模块，或者大部分东西做成可拆分以及的和新包和模块」。要求认真评估，不一句否掉，也不硬套。

## 决定
1. **模块化**由 crate / module / trait 承担（编译器在语法树上强制的边界），核心包 = `vdn-types · vdn-dsp · vdn-models · vdn-engine · vdn-speaker`
2. **守规矩**用 AST / 依赖图工具，进 CI：
   - 根目录**一份** `clippy.toml` 禁用清单 + 实时 impl 块 / 模块上的 deny（构造函数放另一个 impl 块）
   - 各 crate `Cargo.toml` 的 `[package.metadata.vdn]`（档位、unsafe、禁依赖）作为依赖规则的机读正本
   - cargo-deny `[bans]` 管直接依赖边（含内部 crate）
   - `cargo xtask check-deps` 管传递闭包
   - xtask tidy（`#[test]`）：要懂 Rust 结构的规则用 `syn` 语法树判（模块树、字面量、路径、类型结构）；纯文本就够的（`.gitignore`、Cargo 元数据、行数）仍按文本
   - `rtsan-standalone` 的 `#[nonblocking]` 标实时入口
   - 第二阶段已启动（#71，2026-11-03）：选 `syn`，不选 ast-grep，理由见文末「修订（#71）」
3. **造代码**克制：只用 cbindgen（共享内存头）；不写过程宏、不用 build.rs 生成；参数表 derive 宏的触发条件 = 参数超过约 10 个，或手抄第三次出错
4. dylint 仍暂缓：现有规则都不需要类型信息，要类型的那部分交 clippy `disallowed-methods`。触发条件改为「某条规则必须靠类型信息才能判，且 clippy 配不出来」

## 理由
- 社区里三种「借 AST 做模块化」的做法都有代价：过程宏（nih-plug `#[derive(Params)]`、firewheel-macros）拖编译和 IDE；类型级 DSP 图（fundsp）类型报错难读；外部 DSL 生成（Faust）不适合我们【源+官】`notes/r5-talks.md` §4
- 「用规则判」比「用宏生成」更贴本项目的硬约束：实时安全要**机器证明**（命门 2），而生成代码本身证明不了任何东西；与 CLAUDE.md 第三节「能用规则判的，绝不留给人判」同向
- 先例：rust-analyzer 的 tidy 就是随 `cargo test` 跑的文本检查【源】；tympan-aspl 在实时模块根 `#![deny]` + clippy.toml 禁用清单【源】`resources/tympan-aspl/src/realtime/mod.rs:45`、`clippy.toml:26-66`
- 只写一份 clippy.toml：crate 目录下的 clippy.toml 会整份替换上层、不合并【测】；分散写违反铁律 7
- cargo-deny 的 `wrappers` 只查直接依赖者【源】`resources/cargo-deny/src/bans.rs:616-650`，所以必须配 check-deps
- 过程宏的触发条件与 CLAUDE.md「同一类错误犯到第三次，必须沉淀成一条规则」同构

## 代价
- xtask 要自己维护（tidy、check-deps 两个子命令）
- 禁用清单是打地鼠，漏列的 API（`clone` 等）静态层拦不住；每条禁令配一个必须报红的哨兵负例，防止写错路径后悄悄失效
- clippy 只看本 crate 的调用点，看不到依赖内部与宏展开内部——靠 RTSan 补，而 RTSan 只覆盖回放跑到的路径
- xtask 多依赖 `syn` / `proc-macro2` / `quote`（锁文件已有的版本），全依赖冷编译 1.59 s → 4.48 s（含给 syn 开优化的代价），`tidy` 0.563 s → 0.632 s（#71 实测）；tidy 规则要懂 Rust 模块规则，模块树代码本身要有负例测试
- 静态层证明的是「写法与依赖」，运行期层证明的是「回放覆盖范围内的行为」；这就是「机器证明」的诚实口径（⟨待拍板 1⟩ 一并确认）

## 一句话
**AST 这个想法我们这样用：crate 图本身就是那棵「语法树」里的模块边界；再用 AST 和依赖图工具把架构规则写成 CI 红灯；生成代码只在第三次手抄出错时才上。**

## 修订（2026-11-02，D72 · issue #57）
「要生成就用 xtask 生成、提交、加一致性测试」落成 schema 机制：Rust 类型是正本，工具 crate `vdn-schema` 导出到 `specs/schema/` 并进仓，测试重新导出逐字比对；仍不用 build.rs（`specs/schema/README.md`）。导出放独立的 tool 档 crate 而非 xtask，因为 xtask 约定不依赖任何 vdn crate。

## 修订（2026-11-03，#71）
触发：「同一类规则漏网第三次」已满足——#62 里用文本猜测试模块，codex 第 2–7 轮共挑出 9 个绕过点（注释里的声明、另一个入口的生产引用、生产引用的传递、内联模块路径、`r#tests`、带属性的声明、同一行写多项……），最后只能改用文件头标注；R-PARAM（rustfmt 拆行查不到）、R-MIRROR（改名副本查不到）的文件头也写着同类漏洞。人原话「可以真正用ast工具」。

做法：xtask 引 `syn`，从每个 crate 的编译根建模块树判测试代码（`src/xtask/src/ast/`），R-RATE · R3 · R-PARAM · R-MIRROR · R-MODEL 改在语法树上判；迁移前后对真仓逐条比对（`state/handoff/tidysyn-compare.md`），差异逐条解释；012 的负例全部做成测试（`docs/plan/012_tidy-syn.md`）。

选 `syn`、不选 ast-grep / dylint：
1. **Rust 生态内**：`cargo test` 就能跑，CI 不装外部二进制；ast-grep 要另装 CLI、规则写 YAML，规则自己的单测要另搭
2. **能建模块树**：测试识别的核心是跨文件的模块路径与 `#[path]`，要写程序，模式匹配做不了
3. **锁文件已有 syn**：不引新依赖树；cargo-deny 通过
4. **不选 dylint**：要 nightly 与 rustc 内部接口，升级工具链易坏；现有规则不需要类型信息

仍做不到（写进各规则文件头）：宏展开出的代码、跨文件 `pub use` 再导出、`cfg(any(test, …))` 求值（一律按生产算）、范围先赋给别的变量再用（R-PARAM P2）。
