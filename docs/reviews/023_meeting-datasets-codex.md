# 评审 023 · 开会场景真实录音数据集调研的结论层与卡片检查脚本（codex · gpt-6-astra · read-only）

> 对象：`docs/research/023_meeting-datasets/` 的 `decision.md` · `matrix.md` · `pick.md` · `sources.md` · `card-template.md` · `check_cards.py` · `test_check_cards.py`；依据：44 张新卡、`cards-014-addendum.md`、014 卡片索引、交代文件。**最新一轮：第 1 轮（2026-11-03）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×7、P2×3 逐条回卡片核实，属实 10、驳回 0。新出的要人定的事：无（已有 4 题见 `decision.md` 第四节）。

## 第 1 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/20261103-b10/mtgdata/reviews/decision-codex-1.md`（不入库）。effort medium，耗时 111 秒。行号指评审时的版本。
本轮 P0/P1：7 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜matrix.md:18 DR-VCTK 标「训练可」**（第 1 轮 #1）
  - 核实：属实（D65 第 4 条写 DR-VCTK End-user Licence 核清前只评测；补充卡也写未读正文）
  - 处理：matrix、pick 两处改为「按 D65 只评测」
- **P1｜matrix.md:42「只有背景、只能自录」过强**（第 1 轮 #2）
  - 核实：属实（同表已列 LIVETALK 手机真人通话、CHiME-3 街边平板朗读；Ego4D 卡场景键含街边 / 车站却未入表）
  - 处理：改为「没有『笔记本 + 该地点 + 主说话人』的公开集」，列出手机 / 平板 / Ego4D 的真人来源；matrix 第零、二、三节，decision 第一节第 3 条，pick 第三节，decision 第三节 Ego4D 行与第 4 题选项 B 同步
- **P1｜decision.md:11「Mac 只有两处」漏 MOBIO**（第 1 轮 #3）
  - 核实：属实（`cards/mobio.md` 设备键 `mac-intel-builtin`，2008 款 MacBook 真人说话）
  - 处理：decision、matrix 改为三处；pick 1 号理由改为「不用签协议就能拿到的只在这里」；「Apple 没有公开」改为「检索未命中，不等于没有」
- **P1｜decision.md:14 中文「没有能按设备筛的笔记本段」与 3D-Speaker 卡冲突**（第 1 轮 #4）
  - 核实：属实（`cards/3d-speaker.md` 训练集 96,262 条笔记本，可按设备筛）
  - 处理：改为「只有 3D-Speaker，音频许可未查到」；matrix、pick 补「按设备可筛」
- **P1｜matrix.md:17 ACE 当成回放录音锚**（第 1 轮 #5）
  - 核实：属实（补充卡 ACE 节：实测 RIR + 真实噪声 + 需自己卷积的语音；014 卡：消声室语音）
  - 处理：matrix 的 meeting-room / open-office / library-classroom 三格移除 ACE，注明「合成原料」；pick 第一节改「不补真实锚格子」、第三节移除
- **P1｜pick.md:26 SIG 测试集可复现相关数字，依据不足**（第 1 轮 #6）
  - 核实：属实（SIG 卡：逐条 P.804 人评分未公开；论文数字测在 9 队盲测输出上）
  - 处理：pick 3 号改为「只能当无标签真实样本」；decision 第 2 题选项 B 后果同步
- **P1｜pick.md:31 合计 < 10 GB 忽略 LIVETALK 只随 15.9 GB 整包发布**（第 1 轮 #7）
  - 核实：属实（`cards/nisqa-corpus.md` 大小与获取方式；014 LIVETALK 卡获取写「含其它子集」）
  - 处理：pick 2 号补整包说明；合计改为「不含整包 < 10 GB，含整包约 20–25 GB」；顺带补 AISHELL-5 Eval1 约 1.7 GB；decision 第 2 题选项 C 同步
- **P2｜decision.md:16 把 NISQA 断续维度 0.478 算进 DNSMOS**（第 1 轮 #8）
  - 核实：属实（`notes/r3-noref.md` 总表：DNSMOS 单条 0.656 / 0.695，0.478 是 NISQA 断续维）
  - 处理：改为「DNSMOS 各维单条 0.55–0.70；NISQA 整体 0.68；评分协议不同，只能粗比」
- **P2｜matrix.md:19 场景与设备映射偏离卡片**（第 1 轮 #9）
  - 核实：属实（Speech Commands 卡场景键 `home-quiet`；VOiCES 是录音棚麦 + 领夹，不是麦阵）
  - 处理：Speech Commands 移到 home-quiet 的「其他笔记本」列；MagicData-RAMC 只留在 home-quiet 并注「是否安静未写」；VOiCES 移出麦阵列
- **P2｜check_cards.py:149 目录输错或为空仍报 0 处问题、退出 0**（第 1 轮 #10）
  - 核实：属实（读代码）
  - 处理：目录不存在 / 不是目录 / 无卡片 → 退出码 2；加测试 `test_bad_card_dir_exit_2`；24 条测试过【实测】

### 引用抽查

- codex 列的「不符」2 条（decision 3D-Speaker、pick SIG）：都已改（即 #4、#6）
- codex 列的「核实属实」4 条
- 无法核实：021 / 022 键表不在白名单（已由大总管验收：指向两边 worktree 时 0 处问题）；「未下载音频」只有自报

### 是否再审

本轮有 P1 → 改完开第 2 轮，带本记录。另外，评审期间 `check_cards.py` 按大总管要求改了取正本的方式（只认本仓库路径 + `VDN_KEYS_DIR` + `--research`），第 2 轮一并审。
