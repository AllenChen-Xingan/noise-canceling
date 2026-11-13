---
name: two-coordinators-hook-cwd
description: 两个大总管同机时，auto-commit hook 在会话工作目录运行——会话起在主检出的那位会顺带提交另一位在主检出的未提交改动；不管 hook 本身的规则、不管合并冲突
type: feedback
---

**为什么**：2026-11-04 大总管 a2 的会话工作目录是主检出（`noise_canceling/`），而主检出属另一位大总管 5-app-5e（分支 `chore/coord-20261103-b10`）。a2 每跑一条命令，`.claude/hooks/auto-commit.sh` 就在主检出里检查改动，超过 5 行即提交——三次把 5e 正在改的 `state/coord.md` · `state/pending-decisions.md` 提交到了 5e 的分支（7cf6133f · 51a50418 · 3397c1cf），署名是 a2 的会话。内容都是 5e 写的、没被改动，auto-commit 本是规矩，所以无害；但会让人以为 a2 动了别人的分支。前两次误以为是「往主检出 `_tmp/` 写文件」引起的，第三次（只在自己检出里写）仍发生，才确认根因是会话工作目录。

**怎么用**：
- 第二位大总管**从自己的检出目录启动会话**（如 `cd ../noise_canceling-coordfe && claude`），不要从主检出启动；已经起在主检出的，命令一律 `cd <自己检出> && …` 或 `git -C`，并接受 hook 仍会在主检出触发
- 发现 hook 在别人的分支提交了：查 `git log -1 --stat <hash>`，确认只含对方的文件、内容是对方写的，再一句话告诉对方；不要回滚
- 共享锁、批次交代这类必须放主检出 `_tmp/` 的文件照放（`_tmp/` 被 git 忽略，本身不会被提交）
