# 评审 016 · exp016 候选模型上同一张成绩单（codex · gpt-6-astra · read-only）

> 本线（issue #112）按人定 2026-11-04 插三次 codex：① 外部入口开小 PR 前（中途，1–2 轮，P0/P1 修掉即停）· ② 第一个候选跑 480 段前审离线推理脚本（中途，同上）· ③ 收尾审 findings（连续两轮无 P0/P1）。
> **最新：① 外部入口第 2 轮（2026-11-04）**，见下。

## ① 外部离线输出入口

> 对象：`git diff 4493f0ce...HEAD`（`src/eval/src/system.rs` 分发 · `src/eval/src/system/external.rs` · `src/eval/tests/external_e2e.rs` · `src/eval/src/main.rs` 一行 · `docs/experiments/exp016-candidates/external-entry.md`）。重点：evalfix 提的 3 条公平性检查，尤其截半查偷看未来能否被无心绕过。
> 累计：P1×1、P2×4 逐条回源码核实，属实 5、驳回 0；改 3 条，2 条 P2 留作 issue（待大总管开）。

### 第 1 轮（2026-11-04）· 总评「修改」

原文：主目录 `_tmp/exp016/reviews/diff-4493f0ce-head-codex-1.md`（不入库）。effort medium，60 秒。行号指评审时的版本。
本轮 P0/P1：1 条 · 留作 issue 的 P2：#3 · #4

- **P1｜external.rs:132 不同目录同名系统被视为相等，入口也不拒绝**（第 1 轮 #1）
  - 核实：属实。`PartialEq` 按名字比，`load` 不查重；两个同名目录同时参评时成绩单 `system` 列分不清来源
  - 处理：`SystemKind::external` 改返回 `Result`，进程内名表查重，第二个同名目录报「已被另一个目录用过」；加测试 `duplicate_external_name_is_rejected`
- **P2｜external.rs:331 延迟 ≥ 半段时截半比较为空仍通过**（第 1 轮 #2）
  - 核实：属实。`saturating_sub` 得 0，最大差恒为 0
  - 处理：比较区间短于段长 1/4 报「截半自检无法验证」；加测试 `huge_declared_latency_cannot_be_verified`
- **P2｜external.rs:323 单段单切点不能可靠排除离线处理的未来依赖**（第 1 轮 #3）
  - 核实：属实（切点落静音时有限前视可能漏判；峰值在前半段的整段归一化可完全一致；`bg` 未查）
  - 处理：说明里写明「抽样证据，不是因果性证明」并列出漏判情形（`external-entry.md` 第 4 节）；多切点 / 有声区切点 / `bg` 截半**留作 issue**
- **P2｜external.rs:337 超差直接归因于未来依赖，未排除数值不可复现**（第 1 轮 #4）
  - 核实：属实（Core ML / Neural Engine 可能有运行间抖动）
  - 处理：说明里写明会误判；「同输入重复运行对照」**留作 issue**。实际跑候选时若被拒，先手工重复跑一次判是否抖动
- **P2｜external_e2e.rs:265 缺 bg 会让整段没跑成，不是只背景一路**（第 1 轮 #5）
  - 核实：属实（`runner.rs` 的 `prepare_clip` 一路失败即整段失败；`runner` 归 evalfix，不改）
  - 处理：改说明契约为「整段进没跑成」，并写明正式跑一律给 `bg/`

#### 引用抽查

- 「不符」1 条：`external-entry.md:47` 关于缺 bg 的表述 → 已按 #5 改
- 「核实属实」4 条
- 无法核实：Engine 故障复位后回放位置、计时项最终判定（实现文件不在 codex 可读范围）→ 由测试 `declared_latency_is_reinserted_and_timing_is_empty` 与逐位一致测试覆盖

#### 是否再审

有 P1 → 改完开第 2 轮。
