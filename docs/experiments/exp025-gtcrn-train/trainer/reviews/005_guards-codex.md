# 评审记录：训练时的自动保护 guards（codex，read-only，effort medium）

范围 `1c251d42...HEAD`（分支 feat/trainer-guards）。重点见 `_tmp/codex-h100/focus-guards.md`：误停 · 该停不停 · 现场与 best.pt · 续训 · 花钱。阈值大总管已定，不评阈值。

## 第 1 轮（2026-11-25）· 总评「修改」

P0×0 P1×3。逐条核实，3 条都属实，都修了；测试先红后绿（新 4 条），全套过。
- **P1｜开发份只查 hybrid**（第 1 轮 #1）：其他读数或 EMA 的读数是 NaN / Inf 时照训，还可能存 best.pt。改：`all_finite` 查原模型与 EMA 的全部读数
- **P1｜跑满后补评估的最好成绩续训时丢了**（第 1 轮 #2）：检查点存在补评估之前，guard 状态里没有这次最好。改：续训时最好的一次以 run.json 的 `best` 为准（每次更新 best 时都写 run.json）
- **P1｜存现场失败会绕过「训崩后拒绝续训」**（第 1 轮 #3）：改：先把 `stop.reason = nonfinite` 写进 run.json，再有时限（120 s）地尽力存现场；存不成记 `snapshot_error`，照样停、照样拒绝续训
