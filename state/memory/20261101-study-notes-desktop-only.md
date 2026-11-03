---
name: study-notes-desktop-only
description: 给人学习用的科普 / 讲解材料只放桌面（.md + .pdf），不进 docs/plan 或仓库其他目录；不管项目自己的方案、调研、决策
type: feedback
---

给人看的科普、讲解、学习材料，只放到桌面，不写进 `docs/plan/` 或仓库里其他目录。

**为什么**：人原话（2026-11-01）：「科普不要放在plan里面，放到桌面，给我学习用的」。`docs/plan/` 放的是项目方案，科普是给人自学用的，混进去会让方案目录变乱，编号也被占掉。当时 010–012 三份科普已从 `docs/plan/` 删除，历史留在提交 86e5257 · 9ecc8c6 · 1305410。

**怎么用**：
- 草稿写在 scratchpad 或 `_tmp/`（不入库），用 deliver-for-human 的 `to-desktop.sh` 复制到桌面，命名 `YYYYMMDD-vdn-<slug>-zh.{md,pdf}`，绝不覆盖
- 默认同时给 .md 和 .pdf（人用读屏，PDF 带标题层级）
- board 里只记一行「放了哪几份到桌面」，不在仓里留正文
- 桌面文件人可能会挪走或整理：再交付前先 `ls ~/Desktop` 看一眼，缺了就重放，不猜去向
