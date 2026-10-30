---
name: commit-issue-pr
description: 所有工作都要走提交 · issue · PR；管分支、开 issue、推送、开 PR、合并的时机与做法。不管提交信息格式（见 specs/README.md）
type: feedback
---

**为什么**：人 2026-10-30 原话「记住，所有东西需要提交以及commit以及 issue 和pr 别忘了」。之前（2026-10-29）只有本地自动提交、没有远程，推送和 PR 要人逐次同意；人改为全部由 AI 走完，并选了：私有仓库 `AllenChen-Xingan/noise-canceling`、每条线一个 issue、AI 开 PR 并自己合并。

**怎么用**（规则正本是 `CLAUDE.md` 铁律 15，这里只记操作顺序）：
1. 派一批活之前：`git switch -c batch/<YYYYMMDD>-<批次>`（auto-commit hook 会把手下的改动提交到这个分支）；每条线 `gh issue create`，编号记进 `state/coord.md` 手下表
2. 收尾：铁律 12 外发检查（密钥 / 个人信息 / 大文件）→ `git push -u origin <分支>` → `gh pr create`，正文逐条 `Closes #N` → 等 CI → `gh pr merge --merge --delete-branch` → `git switch main && git pull`
3. 手下仍然**不做任何 git 写操作**，这些都归大总管
4. 仓库改公开、删远程分支以外的不可逆操作，仍先问人
