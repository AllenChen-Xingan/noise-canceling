# 现状页 · 手下 prefs（批次 20261031-b4）

分支 `feat/eval-prefs` · worktree `noise_canceling-prefs` · 大总管 `noise-canceling-69`

| 时间 | 做了什么 |
|---|---|
| 10-31 | 报到；读完交代、common、exp003 / exp005 / exp006 README 与答卷、rubric §4、012 verifiers §2/§5、D38–D48 |
| 10-31 | 与 evalimpl（`noise-canceling-evalimpl-7f`）对齐任务格式：用 vdn-eval 的 JSON 清单（`id / set / polarity / origin / real / graders / xfail`），verifier 用名字 id（V-EAT-ASR 等），未知字段会被忽略，溯源字段可加 |
| 10-31 | 发现：exp003 处理后音频（`_tmp/20261030-exp003/`）本机已找不到，只有输入哈希 → 记缺口；exp005 README 写主人 3570，`clips.json` 写 `owner_speaker: 7127`（3570 是 c03 邻座）→ 记缺口，不改别人的文件 |
| 10-31 | `build_dataset.py` → `trials.jsonl`（29 屏）· `pairs.jsonl`（219 对）· `summary.json`；`test_build_dataset.py` 13 项全过（揭盲数与各实验 README 人工汇总逐项一致；exp006 假答卷能直接并入） |
| 10-31 | 发现：exp005 c06、c08 里「激进」与「只有 GTCRN」逐字节相同（c06 无邻座、c08 没锁）；c08 人开会选了其中一份 → 该对标 invalid。exp006 盲听页 12 屏里 10 屏「路由开」=「路由关」同一文件，路由效果只在 te0039 / te0011 / te0028 三屏可测 |
| 10-31 | `make_tasks.py` → `tasks/human-tasks.json`（回归 8 · 能力 12）；verifier id 对 main 注册表（PR #38 已合并，19 个）校验通过，与 ci.json / qa.json 无 id 冲突 |
| 10-31 | 会话因休眠中断后恢复；大总管改名 `noise-canceling-c4`；对应 issue #37 |
