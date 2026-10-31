# ADR 001 · workspace 形态与 crate 划分：从 p1 骨架分阶段长出来

- 状态：提议（2026-10-30，rustarch）· 待人过目 · 涉及 ⟨待拍板 2、4、5⟩
- 规约落点：`specs/rust/architecture.md` §1、§2

## 背景
设计稿 §6 初拟 7 个 crate（dsp · engine · speaker · io-macos · hal-plugin · control · eval）。p1 已起骨架：`vdn-dsp ← vdn-engine ← vdn-models ← vdn-cli`，`src/<短名>/`，虚拟 manifest，edition 2024。命门 1（`must-know.md`）要求离线评测与实时用同一份处理代码，且核心不依赖平台。

## 决定
1. 扁平 `src/<短名>/` + 虚拟 manifest + `[workspace.*]` 继承 + `xtask`；package 名 `vdn-<短名>`
2. 拆 crate 的唯一主判据是**依赖边界**（它不许看见谁），其次是实时档、可替换实现、产物形态、重依赖、unsafe 集中；代码量不是判据；不建 `utils`
3. **分阶段拆**，不一次建满：P1 保持 p1 的 4 个；P2 拆出 `vdn-types`（speaker 要发布给 engine 又不许依赖 engine）、`vdn-speaker`、`vdn-eval`；接 tract 时拆 `vdn-backend-ort/tract`；P3 加 `vdn-io-macos`、`vdn-rt-thread`；P4 加 `vdn-shm`、`vdn-control` 与两个进程 bin
4. 设计稿的 `hal-plugin` 移出 Rust workspace（C 写，003 已定），Rust 侧换成协议 crate `vdn-shm`（`no_std` + cbindgen）
5. 设计稿的 `engine` 拆成 `vdn-types`（词汇）+ `vdn-engine`（调度）+ `vdn-models`（每模型一个文件）

## 理由
- matklad「Fast Rust Builds」：「The most important property of a crate is which crates it doesn't (transitively) depend on」；「keep serialization at the boundary of the system, in the leaf crates」【官·作者一手】`notes/r2-crates.md` §1.3、`notes/r7-skeptic.md` C22
- rust-analyzer 架构文档：核心「never does any IO」，只有一个 crate「knows about LSP and JSON serialization」【官】`notes/r7-skeptic.md` C21
- Ruff、Bevy、Zed、Firewheel 都是扁平 `crates/` + 项目前缀【官】`notes/r2-crates.md` §1.6
- 分阶段：一次建 15 个 crate，前三个阶段大半是空壳，空壳的依赖规则没人验证；按「出现新的依赖边界才拆」能让每次拆分都有机器检查跟上【推】
- 互斥 feature 不被 Cargo 推荐（feature 会被统一），可替换后端拆 crate【官】Cargo Book「Features」

## 代价
- P2 要把 `engine::model` 搬家到 `vdn-types`，所有 `use` 改一遍（预计机械改动，约十几处）
- 两个进程两个 bin，多链接一个二进制；换来「送音进程不链推理库」可由 check-deps 机器证明（⟨待拍板 4⟩）
- crate 多了 Cargo.toml 多，靠 `[workspace.*]` 继承压住重复
