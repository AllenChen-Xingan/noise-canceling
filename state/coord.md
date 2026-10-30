# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-69`（换班后在这里改）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

## 手下

| 窗口 | 任务 | 交代文件 | 现状页 | 状态 |
|---|---|---|---|---|
| model | 002 起步模型与代码 | `_tmp/20261029-b1/model.md` | `state/handoff/model.md` | 已报到（noise-canceling-f0） |
| pipeline | 003 Mac 音频管线 | `_tmp/20261029-b1/pipeline.md` | `state/handoff/pipeline.md` | 已报到（noise-canceling-21） |
| evaltool | 004 评测工具链 | `_tmp/20261029-b1/evaltool.md` | `state/handoff/evaltool.md` | 已报到（noise-canceling-0b） |

## 规矩

- 见 `_tmp/20261029-b1/common.md`；本账本只记人后来改过的条目（标「人定 <日期>」）
- 调研必须用 workflow / deep research 做，要非常完整（人定 2026-10-29，原话：「需要用 workflow来做 researrch deep research 来弄，就是这个研究需要非常完整」）
- 提交由 hook 自动做（人定 2026-10-29，CLAUDE.md 铁律 14）；手下不做 git 写操作

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试（1 个） | model（exp001 每帧耗时，21:xx 起，预计 5 分钟） |

排队：（空）

等人许可的弹窗 / 长时间前台运行：（空）

## 进度板

进度（HH:MM）
- 21:xx model：must-know 已起草；workflow 10 代理（5 核查 + 5 怀疑派）已批准，exp001 之后申请名额
- 21:xx pipeline：must-know 已起草；5 子代理核 12 仓库 → workflow 约 8 代理（补检索 + 3 对抗视角）已批准
- 21:xx evaltool：起草 must-know 中，计划未报

## 发现

- 17:35 首派失败：spawn-worker.sh 不识别新版信任提示 → 超时退出，后两窗未开；已修脚本 BLOCK_RE（人定 2026-10-29）
- 交代里大总管名 5-app-89 失效，现为 noise-canceling-69，批次文件已改

## 待定清单（攒着等人一次看完）

- [ ] 个性化（只保留主人声音）：推后 / 核心 / 不做（默认推后）—— 来源 board
- [ ] 远程仓库地址 —— 来源 board
- [ ] 放权边界 —— 来源 board
