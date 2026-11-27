# 大总管交接页（2026-11-27 收工，会话 noise-canceling-meta，窗口 vdn:coord2）

- **固定名**：`noise-canceling-meta`（轮换不变）
- **当前大事（b19）**：GTCRN 加训第一批。D119「开，练完接着评」。训练已完成（6 个运行 × 5000 步）；评测**今晚按人要求停在半路**
- **收工状态**：人要求全部收工（2026-11-27）。4 个手下都回了「可以关了」，窗口由人关。云上 0 台（大总管亲手跑 `rp.py spend`：open_pods 空，合计 23.35 美元 / 上限 100）

## 各线（全部已推送、工作区干净、pi 全退）

| 线 | 分支 · 最后提交 | 做到哪 · 下一步 | 交接页 |
|---|---|---|---|
| cloudrun | exp/025-cloud-baseline · 855ccda9 | 评测机 1 回归检查过；原版离线推理 43 min 未完，部分结果取回 `_tmp/20261127-cloudrun-eval-formal/eval-formal-m1-partial.tar.gz`（sha 14cdf91b…）；机 2、机 3 没开。**下一步**：按实测重算机数与花费（可能 4 台），报大总管→人批（待定 12-1） | `../noise_canceling-cloudrun/state/handoff/cloudrun.md` 末节 |
| evalcmp | exp/025-eval-cloud · de988bf7 | 判读副本 `../noise_canceling-evalrun` 停在 2a07e8ed、已编译；6 个 ONNX 已在网络盘。成绩单回来后跑 12 份对比 + 2 份验收 | `state/handoff/evalcmp.md`（在其 worktree） |
| trainer | feat/trainer-train-logs · fd14a9e8（#236，未开 PR） | 加裁剪记录 + 开发评估存 wav；codex 第 1 轮 2 严重 2 次要，修到一半，WIP a3b22d05；test_trainlog 18 过，test_train 与全套未跑 | `state/handoff/trainer.md`（在其 worktree）①–⑤ |
| traindata | feat/data-batch5 · a78a3cdc | 待命；只剩 #222 · #226 · batch2 删前清单（交 cloudrun，按 D118） | `state/handoff/traindata.md`（在其 worktree） |

- evalcmp、traindata 分支各有只改交接页的提交（1 个、2 个）未进 main，下次一起合

## 待人定
- `state/pending-decisions.md` 第十二组：12-1 评测续跑机数与花费 · 12-2 #233 测速

## 之后（不在本窗口做，等成绩单）
- 训练对照评审 `docs/reviews/train-vs-skills-20261126/README.md` 第 2–4 条：学习率 1e-3 · 放宽裁剪 · 衰减短跑（约 6 运行 × 1000 步，花 H100）→ 先过开训前清单、人点头
- #231 A 组流式导出（评测后）

## 检查点与数据
- 检查点 `/workspace/runs/exp025-formal/exp025-formal-<A|B>-s<0..2>/ckpt/`，清单 `_tmp/20261126-trainer-formal/batch2/trainer-formal-ckpt-manifest-20261127T002828Z.txt`；评测完前不删
- 训练数据第五批 build `df759bb8d606a209`，分片 `/workspace/shards/batch5`

## 重开时
1. 加载 coordinator-dispatch skill，`--rename-self` 成 noise-canceling-meta
2. 读本页 · `state/coord.md` 末 40 行 · `docs/plan/001_decisions-20261029-b1.md` 末尾 D110–D123 · `state/pending-decisions.md` 第十二组
3. 窗口要重开：先 `spawn-worker.sh … --cmd "claude --continue"`，首条消息让手下读自己的交接页
