# ADR 004 · 「AST 做模块化」：用 AST 守规矩，不用 AST 造代码

- 状态：提议（2026-10-30，rustarch）
- 规约落点：`specs/rust/architecture.md` §5、§6

## 背景
人的原话：「我建议用超级语法树 / ast 来把东西做成模块，或者大部分东西做成可拆分以及的和新包和模块」。要求认真评估，不一句否掉，也不硬套。

## 决定
1. **模块化**由 crate / module / trait 承担（编译器在语法树上强制的边界），核心包 = `vdn-types · vdn-dsp · vdn-models · vdn-engine · vdn-speaker`
2. **守规矩**用 AST / 依赖图工具，进 CI：
   - 根目录**一份** `clippy.toml` 禁用清单 + 实时 crate 根的 deny 头
   - cargo-deny `[bans]` 管直接依赖边（含内部 crate）
   - `cargo xtask check-deps` 管传递闭包
   - xtask tidy（`#[test]`，文本 / 轻语法检查）
   - `rtsan-standalone` 的 `#[nonblocking]` 标实时入口
   - ast-grep 规则留作第二阶段
3. **造代码**克制：只用 cbindgen（共享内存头）；不写过程宏、不用 build.rs 生成；参数表 derive 宏的触发条件 = 参数超过约 10 个，或手抄第三次出错
4. dylint 暂缓（同一类规则漏网第三次再上）

## 理由
- 社区里三种「借 AST 做模块化」的做法都有代价：过程宏（nih-plug `#[derive(Params)]`、firewheel-macros）拖编译和 IDE；类型级 DSP 图（fundsp）类型报错难读；外部 DSL 生成（Faust）不适合我们【源+官】`notes/r5-talks.md` §4
- 「用规则判」比「用宏生成」更贴本项目的硬约束：实时安全要**机器证明**（命门 2），而生成代码本身证明不了任何东西；与 CLAUDE.md 第三节「能用规则判的，绝不留给人判」同向
- 先例：rust-analyzer 的 tidy 就是随 `cargo test` 跑的文本检查【源】；tympan-aspl 在实时模块根 `#![deny]` + clippy.toml 禁用清单【源】`resources/tympan-aspl/src/realtime/mod.rs:45`、`clippy.toml:26-66`
- 只写一份 clippy.toml：crate 目录下的 clippy.toml 会整份替换上层、不合并【测】；分散写违反铁律 7
- cargo-deny 的 `wrappers` 只查直接依赖者【源】`resources/cargo-deny/src/bans.rs:616-650`，所以必须配 check-deps
- 过程宏的触发条件与 CLAUDE.md「同一类错误犯到第三次，必须沉淀成一条规则」同构

## 代价
- xtask 要自己维护（tidy、check-deps 两个子命令，预计各一两百行）
- clippy 只看本 crate 的调用点，看不到依赖内部与宏展开内部——靠 RTSan 补，而 RTSan 只覆盖回放跑到的路径
- 静态层证明的是「写法与依赖」，运行期层证明的是「回放覆盖范围内的行为」；这就是「机器证明」的诚实口径（⟨待拍板 1⟩ 一并确认）

## 一句话
**AST 这个想法我们这样用：crate 图本身就是那棵「语法树」里的模块边界；再用 AST 和依赖图工具把架构规则写成 CI 红灯；生成代码只在第三次手抄出错时才上。**
