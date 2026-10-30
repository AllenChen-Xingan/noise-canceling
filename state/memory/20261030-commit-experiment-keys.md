---
name: commit-experiment-keys
description: 实验的揭盲表、清单、结果、人的答卷都要当场提交进 git；听测页每次重建写新 build 目录、不覆盖。不管音频本身（音频不进仓）
type: feedback
---

**为什么**：2026-10-30 exp003 盲听，人答到第 6 题时 abtest 在同一路径重建了加 VoxLock 的新页面，旧 build 的揭盲表只放在 git 忽略的 `_tmp/` 里，被覆盖后人答的 6 题无法揭盲。人原话：「然后记住，这种时候每次的对照表等等等任何东西，都要git commit」。

**怎么用**（规则正本：`CLAUDE.md` 第五节第 1 条）：
1. 派实验线时，交代里写明：每个 build 一个新目录；揭盲表 / 清单 / 哈希等小文件复制进 `docs/experiments/<实验>/builds/<build-id>/`
2. 让人去听之前，大总管先确认当前 build 的揭盲表已提交；人开始听以后，禁止任何人重建同一路径
3. 人导出答卷后，第一件事是放进 `docs/experiments/<实验>/results/` 并提交，再揭盲
