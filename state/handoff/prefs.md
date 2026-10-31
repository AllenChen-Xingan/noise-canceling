# 现状页 · 手下 prefs（批次 20261031-b4）

分支 `feat/eval-prefs` · worktree `noise_canceling-prefs` · 大总管 `noise-canceling-69`

| 时间 | 做了什么 |
|---|---|
| 10-31 | 报到；读完交代、common、exp003 / exp005 / exp006 README 与答卷、rubric §4、012 verifiers §2/§5、D38–D48 |
| 10-31 | 与 evalimpl（`noise-canceling-evalimpl-7f`）对齐任务格式：用 vdn-eval 的 JSON 清单（`id / set / polarity / origin / real / graders / xfail`），verifier 用名字 id（V-EAT-ASR 等），未知字段会被忽略，溯源字段可加 |
| 10-31 | 发现：exp003 处理后音频（`_tmp/20261030-exp003/`）本机已找不到，只有输入哈希 → 记缺口；exp005 README 写主人 3570，`clips.json` 写 `owner_speaker: 7127`（3570 是 c03 邻座）→ 记缺口，不改别人的文件 |
