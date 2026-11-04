# 评审 023 · 开会场景真实录音数据集调研的结论层与卡片检查脚本（codex · gpt-6-astra · read-only）

> 对象：`docs/research/023_meeting-datasets/` 的 `decision.md` · `matrix.md` · `pick.md` · `sources.md` · `card-template.md` · `check_cards.py` · `test_check_cards.py`；依据：44 张新卡、`cards-014-addendum.md`、014 卡片索引、交代文件。**最新一轮：第 2 轮（2026-11-03）· 总评「修改」· 待第 3 轮**。
> 累计处理：P1×9、P2×7 逐条回卡片 / 决策正本核实，属实 16、驳回 0。新出的要人定的事：无（已有 4 题见 `decision.md` 第四节）。

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

## 第 2 轮（2026-11-03）· 总评「修改」

原文：主目录 `_tmp/20261103-b10/mtgdata/reviews/decision-codex-2.md`（不入库）。effort medium，耗时 117 秒。
上一轮处理的验证：codex 抽查 DR-VCTK、MOBIO、NISQA 整包、DNSMOS 数字、目录检查 5 处，均「核实属实」。
本轮 P0/P1：2 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜sources.md:18 许可口径仍按已作废的 D26 把 SA 判为只评测**（第 2 轮 #1）
  - 核实：属实（决策正本 D64：SA 可进训练份，NC 仍只评测；D26 已标作废）
  - 处理：5 张只带 SA 的新卡（DEMAND、CN-Celeb、ASCEND、QUT-NOISE、VocalSound）结论改「训练可（按 D64）」并在卡末记一行；3D-Speaker 括注改为「若同为 BY-SA 则按 D64 可训练」；matrix、pick 里 AliMeeting、AISHELL-4 / 5、CHiME-5/6、ASCEND 等改「训练可（D64）」；sources 写明调研时用了旧口径与改判；`cards-014-addendum.md` 页首加 ⚠️ 说明 014 卡 SA 结论按 D64 读
- **P1｜decision.md:24 等处「补不上 / 只能自录」仍过强**（第 2 轮 #2）
  - 核实：属实（MOBIO 已在第一节；LIVETALK 说话端是否笔记本有分歧；phone-booth 等格子本轮未专门检索）
  - 处理：decision 第二节、第 1 题选项 C，matrix 第二、三节，`notes/r2-place.md` 两处，统一改为「本轮未找到已确认的……；要补得自录 / 回放重录」；未专门检索的格子改「先合成，补查后再定」
- **P2｜decision.md:14 WenetSpeech 并入「有近讲参考」**（第 2 轮 #3）
  - 核实：属实（`cards/wenetspeech.md` 干净参考：没有）
  - 处理：单列为「有转写、没有干净参考」
- **P2｜matrix.md:23 只有背景的集放进麦阵终端列**（第 2 轮 #4）
  - 核实：属实（QUT 设备键 `non-laptop-studio`；TAU 手机 / 录音棚）
  - 处理：主表移除全部「只有背景」条目，另列「只有背景声（合成原料）」一节并按卡片设备键标注
- **P2｜decision.md:33 Ego4D「规模最大」依据不足**（第 2 轮 #5）
  - 核实：属实（卡上只有全集 3,025 h，地点子集规模未知）
  - 处理：改为「待核候选，规模未知」，第 4 题选项 B 后果同步
- **P2｜check_cards.py:164 相对路径漏检补充文件**（第 2 轮 #6）
  - 核实：属实（读代码：未 resolve 就与绝对路径比较）
  - 处理：入口 `resolve()`；补充文件改为在卡片目录上一级找；加测试 `test_addendum_checked_for_relative_and_absolute`；25 条测试过，从仓库根用相对路径跑也是 44 张卡 0 处问题【实测】

### 引用抽查

- 「不符」1 条（WenetSpeech）：已改（#3）
- 「核实属实」5 条
- 无法核实：021 / 022 正本格式（大总管已验收）；「未下载音频」只有自报

### 是否再审

本轮有 P1 → 改完开第 3 轮。
