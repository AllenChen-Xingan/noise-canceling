---
name: commit-issue-pr
description: 所有工作都要走提交 · issue · PR；管分支、开 issue、推送、开 PR、合并的时机与做法。不管提交信息格式（见 specs/README.md）
type: feedback
---

**为什么**：人 2026-10-30 原话「记住，所有东西需要提交以及commit以及 issue 和pr 别忘了」。之前（2026-10-29）只有本地自动提交、没有远程，推送和 PR 要人逐次同意；人改为全部由 AI 走完，并选了：私有仓库 `AllenChen-Xingan/noise-canceling`、每条线一个 issue、AI 开 PR 并自己合并。

**怎么用**（规则正本是 `CLAUDE.md` 铁律 15，这里只记操作顺序）：
1. 派活之前，**每条线一个分支 + 一个 worktree**（人 2026-10-30 追加：「现在必须开分之 调研分支」）：`git worktree add -b research/<NNN>-<slug> ../noise_canceling-<slug> main`（实验线用 `exp/`）；在那个目录里起手下（auto-commit hook 按会话所在目录提交，所以手下必须在自己的 worktree 里起会话）；每条线 `gh issue create`，编号记进 `state/coord.md` 手下表。`_tmp/` 不跨 worktree，交代文件给绝对路径。开之前先 `df -h` 看盘
2. 收尾（每条线各一个 PR）：铁律 12 外发检查（密钥 / 个人信息 / 大文件）→ `git push -u origin <分支>` → `gh pr create`，正文逐条 `Closes #N` → 等 CI → `gh pr merge --merge --delete-branch` → `git switch main && git pull` → `git worktree remove ../noise_canceling-<slug>`
3. 手下仍然**不做任何 git 写操作**，这些都归大总管
4. 仓库改公开、删远程分支以外的不可逆操作，仍先问人
