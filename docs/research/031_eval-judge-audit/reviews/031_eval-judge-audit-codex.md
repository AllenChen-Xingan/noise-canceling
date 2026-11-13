# 031 评估器体检：代码修复与结论的 codex 评审

> 对象：`git diff b2e05478`（research/031-eval-judge-audit；`src/eval/` 定标去重 · 过关线混淆矩阵 · 随机抽查，`docs/experiments/exp022-judge-tpr-tnr/`，`docs/research/031_eval-judge-audit/`）· 类型 diff · effort medium · 模型 gpt-6-astra（日志头）· 只读沙箱
> 本记录放在调研目录里（手下 evalaudit 的可写范围）；`docs/reviews/README.md` 的索引行由大总管补。
> 停止线：连续两轮无 P0/P1。

## 第 1 轮（2026-11-13 12:11，59 s）· P0×0 P1×1 P2×2 · 总评：修改

| # | 级 | 发现 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P1 | 吃字题没答（字段缺、null、空列表）被当成「人判没吃」，污染真值 | 属实：exp003 明确「都没有」记为 `["none"]`，另一份旧答卷整份没有该字段 | `judge.rs` 的 `Item.eaten` 改 `Option<bool>`，没答 → 判不了并计数；Python 同改。测试：`judge_tests::unanswered_eaten_is_skipped_not_pass`、`test_stats.MissingEaten`。exp022 结果不变（exp003 每屏都答了） |
| 2 | P2 | 送人耳的段入样概率不等，等权累计的混淆矩阵不能当总体率 | 属实 | `judge.rs` 文件头与 `scorecard.md` 第六节写明「这批答卷上的率」，要估总体须按入样概率加权或分层；exp022 盲听页揭盲表记分层与抽样比例。加权实现留到有答卷后（随 issue 草稿 p2-7） |
| 3 | P2 | J = TPR + TNR − 1 < 0 时 Rogan-Gladen 并非无解 | 属实（例 TPR 0.2、TNR 0.3、观测 0.55 → 0.3） | `stats.rogan_gladen` 只在 |J| 近 0 时返回 None；加 `test_negative_j_still_recovers`；issue 草稿 p2-7 验收条件同步 |
| 抽查不符 | — | audit.md 称 L0–L3 只看方向与 τ，但 L3 另要求 rmse*、对抗样本、重测 | 属实 | 改为「L1、L2 只看这两个；L3 另有要求，但也没有过 / 不过口径」 |
