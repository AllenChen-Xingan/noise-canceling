# 008 · Rust 代码怎么组织（crate 划分 · 核心 trait · 实时安全 · AST 工具）

> 2026-10-30，手下 rustarch（批次 20261030-b3），分支 `research/008-rust-architecture`。
> **结论的正本是规约 `specs/rust/architecture.md`**；本目录放取舍过程与证据。

| 文件 | 回答 |
|---|---|
| [`must-know.md`](must-know.md) | 三条命门：同一份处理代码 · 实时安全要机器证明 · 模型可替换 |
| [`sources.md`](sources.md) | 两类来源登记（自有 · 他人），还算不算数 |
| [`assign.md`](assign.md) | 这次怎么派活（Workflow 8 代理） |
| [`gap.md`](gap.md) | 它们做到的 / 没做的 |
| [`decision.md`](decision.md) | **取舍卡**，末尾「待人拍板」 |
| [`adr/`](adr/) | 5 条架构决策记录 |
| [`p1-review.md`](p1-review.md) | 对 p1 骨架的初审（含一处更正） |
| [`notes/`](notes/) | 8 份调研笔记原件（r7 / r8 的更正优先） |
| [`appendix-probes/`](appendix-probes/) | 本机探针：源码、版本、RTSan 原始日志 |

同批：p1 骨架在分支 `feat/p1-offline-gtcrn`；生态逐项目核查见 rusteco 的 `docs/research/009_rust-ecosystem/`。
