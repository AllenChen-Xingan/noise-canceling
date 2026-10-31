# 派活记录 · 008

> 2026-10-30。Workflow `wf_ec6b3516-162`，8 个代理，约 37 分钟，经大总管批准（计划「约 8 个」）。

| 代理 | 任务 | 产出 |
|---|---|---|
| research:ast | 「AST 做模块化」的四种含义逐种评估，给采用总表 | `notes/r1-ast.md` |
| research:crates | workspace / crate 划分、依赖图、拆分判据 | `notes/r2-crates.md` |
| research:traits | 核心 trait 草案、静态 vs 动态分发、跨线程原语、热切换 | `notes/r3-traits.md` |
| research:rt | 实时规则与「用工具证明」、FFI / unsafe 边界、panic | `notes/r4-realtime.md` |
| research:talks | 大会演讲与社区指南（只收有 URL 的） | `notes/r5-talks.md` |
| research:qa | 工具链与质量闸门、CI 分层、许可 | `notes/r6-quality.md` |
| skeptic | 挑 26 条关键结论逐条证伪 | `notes/r7-skeptic.md` |
| critic | 缺口、矛盾、「明天写代码卡在哪」，自补 8 项 | `notes/r8-critic.md` |

派活四件事都写进了提示：背景（项目、信号链、硬约束、人原话）· 范围（与 rusteco 分工：项目卡归它）· 证据规矩（每条带出处与等级，找不到就说没找到）· 交付（写文件、只回摘要）。

**教训**：提示里没写「不许构建」，子代理自己构建运行了探针，没走名额。下次提示写死：「不许 cargo build / run；要实测先返回申请」。
