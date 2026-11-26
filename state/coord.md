# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-e3`（2026-11-02 换班；旧名 c4 · 69）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

## 当前大事（2026-11-16 起，b19）

**开训：GTCRN 加训第一批**（人「我们应该可以开始训模型了」）。拆法与卡点 `docs/plan/014_training-kickoff.md`；训练机 RunPod + 网络存储盘（D96，人授权 AI 定）。批次 `_tmp/20261116-b19/`，tmux 工作区 `vdn`。b18（真实会议评测集 · 后处理）暂停，不在本窗口做。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| evalcmp | 配对比较子命令 + 早期回声参考轨 · #155 · `feat/eval-paired-compare` · `../noise_canceling-evalcmp` | 评测器 · 评测参考轨 · 成绩单 | 已报到（noise-canceling-evalcmp-aa） |
| traindata | 训练数据下载 · 切分 · 混音配方 · #156 · `exp/025-train-data` · `../noise_canceling-traindata` | 训练数据集 · 数据切分 · 混音配方 | 已报到（noise-canceling-traindata-cc） |
| trainer | 训练器 · 损失 · 注意力头与条件输入 · #157 · `exp/025-trainer` · `../noise_canceling-trainer` | 训练器 · 训练损失 · 模型变体 | 已报到（noise-canceling-trainer-95） |
| cloudrun | Chrome 开 RunPod · 网络盘 · 开关机脚本 · #158 · `feat/cloud-runpod` · `../noise_canceling-cloudrun` | RunPod 账号与密钥 · 网络存储盘 · 云训练机 | 已报到（noise-canceling-cloudrun-8b）；花钱动作先报 |

- 2026-11-16 定稿计划 014 + 调研 034 · 035 + D96–D99 经 PR #154 合并；四个手下从 main 6abdbcc2 开 worktree

## 上一件（b10–b18，2026-11-03 起）

### 降噪效果（2026-11-03 起）

**降噪效果**（D80，人「a，我觉得做这个吧」）：在难场景里比 GTCRN 与其他候选，定训练 / 改进 / 换模型。第一步：盘点评测集对难场景的覆盖（人追问「评测集是否覆盖了很多难的场景」）。批次 `_tmp/20261103-b10/`，大总管 `5-app-5e`，tmux 工作区 `vdn`。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| scenes | 021 开会场景目录 · #80 · `research/021-meeting-scenes` · `../noise_canceling-scenes` | 会议场景 · 噪声类别 · 场景声学条件 | 完成，验收过（25 测试 · 0 问题 · 路径清零），窗口已关；PR #83 已合并；4 题待拍板 |
| macmic | 022 Mac 内置麦克风 · #81 · `research/022-mac-mic` · `../noise_canceling-macmic` | 麦克风（设备链）· 设备链模拟 | 完成，验收过（12 测试 · 21 键 0 问题 · 路径清零），窗口已关；PR #84 已合并；1 题待拍板 |
| mtgdata | 023 真实录音数据集 · #82 · `research/023-meeting-datasets` · `../noise_canceling-mtgdata` | 真实数据集 · 真实锚 | 完成，验收过（25 测试 · 44 卡 0 问题 · codex 5 轮、4/5 轮无 P0/P1 · 无大文件无下载），分支已推送，窗口已关；PR #85 已合并；4 题待拍板 |

**b11（D81：按场景成绩单推进）**，批次 `_tmp/20261103-b11/`：

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| scenesuite | exp013 场景测试集 · #86 · `exp/013-scene-suite` · `../noise_canceling-scenesuite` | 场景测试集 · 合成数据生成器 · 设备链模拟 | 完成，PR #91 已合并（#86 关），窗口已关；1 题待拍板（会议室） |
| scorecard | 场景成绩单：评分模型 + 补评分项 + 标准 · #87 · `feat/scorecard` · `../noise_canceling-scorecard` | 评测器 · 成绩单 · 定标等级 | 完成，验收过（大总管复跑 131 测试 · tidy · schema 一致），窗口已关；PR #88 已合并（#87 关）；1 题待拍板（暂定过关线先用 vs 先盲听） |

两条已合并。

**b12**，批次 `_tmp/20261103-b12/`：

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| card1 | exp014 第一张场景成绩单（GTCRN）· #92 · `exp/014-scorecard-gtcrn` · `../noise_canceling-card1` | 成绩单（报告）· 降噪模型 GTCRN · 评测器（只用） | 完成，已收尾（窗口已关）· PR #97 待合并 |
| evalbp | 调研 025 成绩单评分方法对照业界做法 · #105 · `research/025-eval-best-practice` · `../noise_canceling-evalbp` | 评测器（只读）· 成绩单标准（只读）· 调研 025 | 完成，已收尾（窗口已关）· PR #108 待合并 · 开 issue #109 #110 |

**b15（2026-11-04，人「就是第一个大事情 / 就是评测集这个」）**：修好成绩单的评分方法 + 候选模型上同一张成绩单。批次 `_tmp/20261104-b15/`，大总管 `noise-canceling-84`，tmux 工作区 `vdn`。G-1..G-7 按默认推进，待人确认。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| evalfix | 成绩单评分方法修正（025）+ 重出 GTCRN 成绩单 · #111（含 #109 #110）· `feat/scorecard-v2` · `../noise_canceling-evalfix` | 评测器 · 成绩单 · 定标等级 | 已报到（noise-canceling-evalfix-07）；#109 #110 修完；D85 新规则写完，对照表 runs/20261104-231355-da5821a4-resum-v2/compare.md；中途评审①（2 轮，P1×4 已修）；第 3 组写完，下一步中途评审② → 1 场景冒烟 |
| cands | exp016 候选模型（DFN3 低延迟版 · DPDFNet-2 · NoNoise-Mac）上同一张成绩单 · #112 · `exp/016-candidates` · `../noise_canceling-cands` | 降噪模型候选 · 外部输出入口 · 成绩单（只用） | 已报到（noise-canceling-cands-7f）；入口 PR #115 开（大总管复跑 0 失败 · tidy 过），P2 → #113 #114；两候选冒烟过（每段约 0.8–0.9 s，480 段各约 15 min）；下一步中途评审② → 跑 480 段；NoNoise-Mac 待人 |

> 2026-11-13 清理：PR #93 合并（收拢 b10 · b9 · fe 三条账本分支，补 D82–D84）；删 worktree cands · mixretest · coord99 · coordfe；删已合并分支 本地 24 · 远程 10；scratch 存档 `_tmp/archive-worktrees-20261113/`。

**b18（2026-11-13，人：合成数据有问题、完全没有降噪；要真实会议录音与真实降噪输出做评测；不能混回、换办法保持真实感；没用的数据集可删 → D94）**：批次 `_tmp/20261113-b18/`，大总管 `noise-canceling-68`，tmux 工作区 `vdn`。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| realmtg | 调研 032 / exp024 公开真实会议录音评测集 + 本机数据集清点 · #151 · `research/032-real-meeting-eval` · `../noise_canceling-realmtg` | 真实数据集 · 真实会议评测集 · 本机数据存储 | 已报到（noise-canceling-realmtg-03）；选集 ICSI · AMI · AliMeeting，人批全下 6.0 GB（2026-11-13）；删除只出清单 |
| natural | 调研 033 / exp023 去掉混回、换办法保持真实感 · #152 · `exp/023-no-mixback` · `../noise_canceling-natural` | 引擎后处理 · 降噪模型输出 · 评测器（只用） | 已报到（noise-canceling-natural-b5）；options.md 初稿：A 舒适噪声 · B 字头放宽 · A+B，下一步冒烟；前 1–2 名要在 exp024 真实会议集上复跑；改引擎默认值前等放行 |

**b17（2026-11-13，人：按 hamel.dev《field guide》查评估器测一致性是否忽略假阳性 / 假阴性等问题，有问题就修）**：批次 `_tmp/20261113-b17/`，大总管 `noise-canceling-68`，tmux 工作区 `vdn`。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| evalaudit | 调研 031 评估器体检 + 修复 · #142 · `research/031-eval-judge-audit` · `../noise_canceling-evalaudit` | 评测器 · 人耳标注 · 成绩单标准 | 完成，验收过（大总管复跑 vdn-eval 全过 · clippy · fmt · tidy · Python 34 条；codex 4 轮第 3、4 轮无 P0/P1）；窗口已关；PR #149 已合并（#142 关），worktree 与分支已删；P2 → #143–#147，范围外 → #148；人待做：20 题校准听测 `_tmp/exp022-listen/20261113-122017-3126/listen/` |

⚠️ 人要真实数据集与 D62「开发阶段不用真实录音」可能冲突 → mtgdata 出选项，进待拍板。

## 上一件大事（2026-11-02，已收口）


**推理引擎：降噪模型在 Mac 上用什么跑最快、最稳、最省电**（「轻快」尺子 · D53）。人定「可以先做1」。批次 `_tmp/20261102-b8/`。**✅ 2026-11-02 四条线全部合并（#55 #59 #66），窗口全关；下一件大事待人定（建议「降噪效果」）**

| 手下 | 线 | 管的实体 |
|---|---|---|
| applebe | 019 苹果引擎（Core ML · BNNS Graph · MLX）+ 总方案 · #45 · ✅ PR #59 已合并 | 推理后端 · 模型 · 评测器 |
| rustbe | exp010 纯 Rust（手写 GTCRN · tract / candle / burn）+ 基准程序 · #51 · ✅ PR #55 已合并 | 推理后端 · 模型 · 评测器 |
| fastpath | 020 / exp012 数学上更好 · #58 · ✅ PR #66 已合并 | 推理后端 · 模型 · 评测器 |
| hygiene | **旁支（人 2026-11-02 点名「也让你来管理，然后修复吧」）** 代码卫生：600 行上限进 tidy · cli 同名类型 · 拆 cli 大文件；第二段等 #57 合并后做采样率只写一处 · 阈值比对 · #62 · `feat/code-hygiene` · `../noise_canceling-hygiene` | xtask · vdn-cli · vdn-types（采样率）· vdn-eval（阈值） |

上一件大事「把效果好不好变成能量的尺子」✅ 2026-11-02 收口（#47 #48 #49 #50）。

## 手下

| 窗口 | 任务 | 交代文件 | 现状页 | 状态 |
|---|---|---|---|---|
| model | 002 起步模型与代码 | `_tmp/20261029-b1/model.md` | `state/handoff/model.md` | 完成，已收尾（窗口已关）；剩 3 题 + 三命门待人拍板 |
| pipeline | 003 Mac 音频管线 | `_tmp/20261029-b1/pipeline.md` | `state/handoff/pipeline.md` | 完成，已收尾（窗口已关）；剩 10 条待人拍板 |
| pse | 005 个性化降噪（PSE）调研 · issue #1 · 分支 `research/005-pse`（主目录） | `_tmp/20261030-b2/pse.md` | `state/handoff/pse.md` | 完成，已收尾（窗口已关）；7 条待人拍板 |
| abtest | exp003 DPDFNet-2 vs GTCRN 离线 AB 听测 · issue #2 · 分支 `exp/003-offline-ab`（worktree `../noise_canceling-exp003`） | `_tmp/20261030-b2/abtest.md` | `state/handoff/abtest.md` | 完成，PR #7 已合并（issue #2 关），窗口已关 |
| sota | 006 降噪前沿全景 · issue #3 · 分支 `research/006-sota-landscape`（worktree `../noise_canceling-sota`） | `_tmp/20261030-b2/sota.md` | `state/handoff/sota.md`（在其 worktree） | 完成，PR #6 已合并（issue #3 关），窗口已关；4 题待人拍板 |
| rubric | 007 降噪评分标准 rubric · issue #4 · 分支 `research/007-denoise-rubric`（worktree `../noise_canceling-rubric`） | `_tmp/20261030-b2/rubric.md` | `state/handoff/rubric.md`（在其 worktree） | 完成，PR #8 已合并（issue #4 关），窗口已关；4 题待人拍板 |
| evaltool | 004 评测工具链 | `_tmp/20261029-b1/evaltool.md` | `state/handoff/evaltool.md` | 完成，已收尾（窗口已关）；剩 11 题待人拍板 |
| p1 | P1 离线原型（Rust · GTCRN · 三件保命）· issue #10 · 分支 `feat/p1-offline-gtcrn`（worktree `../noise_canceling-p1`） | `_tmp/20261030-b3/p1.md` | 其 worktree `state/handoff/p1.md` | 完成，PR #13 已合并（issue #10 关），窗口已关；3 题待人拍板 |
| rustarch | 008 Rust 架构（含 AST 模块化）· issue #11 · 分支 `research/008-rust-architecture` | `_tmp/20261030-b3/rustarch.md` | 其 worktree | 完成，PR #15 已合并（issue #11 关），窗口已关；7 题待人拍板 |
| rusteco | 009 GitHub 上可学的 Rust 项目 · issue #12 · 分支 `research/009-rust-ecosystem` | `_tmp/20261030-b3/rusteco.md` | 其 worktree | 完成，PR #14 已合并（issue #12 关），窗口已关；4 题待人拍板 |
| align | 架构对齐 + 智能体部件接口 + 延迟账 · issue #17 · `feat/arch-align` | `_tmp/20261031-b4/align.md` | 其 worktree | 完成，PR #26 已合并（issue #17 关），窗口已关；待人拍板：IO 96 帧（延迟账 47.00 → 43.67 ms） |
| p2 | P2 认出你的声音（离线）· issue #18 · `feat/p2-speaker` | 完成，PR #32 已合并（issue #18 关），窗口已关；4 题待人拍板；盲听页 v2 在人桌面 |
| scene | 场景识别与路由 + 修音链 + 模拟环境 · issue #19 · `feat/scene-polish` | 完成，PR #35 已合并（issue #19 关），窗口已关；6 题待人拍板；盲听页在人桌面；_tmp 音频已移到主目录 |
| ondevice | 010 设备端自适应 + 可靠性调研 · issue #20 · `research/010-ondevice-adaptation` | `_tmp/20261031-b4/ondevice.md` | 其 worktree | 完成，PR #23 已合并（issue #20 关），窗口已关；3 题待人拍板 |
| multitalker | 011 一大堆人同时说话（调研）· issue #21 · `research/011-multitalker` | `_tmp/20261031-b4/multitalker.md` | 其 worktree | 完成，PR 已合并（issue #21 关），窗口已关；6 题待人拍板 |
| evalstd | 012 工业级 rubric 与评测的行业标准 · issue #22 · `research/012-eval-standards` | `_tmp/20261031-b4/evalstd.md` | 其 worktree | 完成，PR #27 已合并（issue #22 关），窗口已关；9 题待人拍板 + rubric 修改清单 R1–R14 |
| rlenv | 013 强化学习环境与分场景训练方案 · issue #25 · `research/013-rl-environment` | `_tmp/20261031-b4/rlenv.md` | 其 worktree | 完成，PR #29 已合并（issue #25 关），窗口已关；9 题待人拍板 |
| datasets | 014 真实录音公开数据集 · issue #30 · `research/014-real-datasets` | `_tmp/20261031-b4/datasets.md` | 其 worktree | 完成，PR 已合并（issue #30 关），窗口已关；6 题待人拍板 |
| evalimpl | eval 实现：vdn-eval · CI / QA / 人工三层 · 回归集与能力集 · issue #31 · `feat/eval-verifiers` | 完成，PR #38 已合并（issue #31 关），窗口已关；CI 每个 PR 跑 eval ci（回归 13/17 · 能力 0/10 · 探针 5/5）；4 题已并入待拍板清单 |
| prefs | 人耳结果 → eval 任务 · 偏好数据集 · 分场景默认 · `specs/rubric/preferences.md` · issue #37 · `feat/eval-prefs` | `_tmp/20261031-b4/prefs.md` | 其 worktree | 完成，PR #43 已合并（issue #37 关），窗口已关；待拍板 4-7 / 4-8 已并入；worktree `../noise_canceling-prefs` 保留待清 |
| benchsynth | **b7 重派（2026-11-02，交代 `_tmp/20261102-b7/benchsynth.md`）· 已报到（noise-canceling-benchsynth-19），完成，PR #47 已合并（cabf3d0，issue #41 关），远端分支已删，窗口已关；worktree `../noise_canceling-benchsynth` 保留待清（手下自己提交并推送了现状页 780eea3，越过「手下不做 git 写」，内容无害，已记）** · 016 评测基准 + 合成数据标准（按评测集倒推合成数据，D50）· issue #41 · `research/016-bench-synth`（worktree `../noise_canceling-benchsynth`） | `_tmp/20261031-b6/benchsynth.md` | 其 worktree `state/handoff/benchsynth.md` | 已报到（noise-canceling-benchsynth-49）|
| ears | **b7 重派 · 完成，PR #48 已合并（issue #42 关），远端分支已删，窗口已关；worktree 保留待清；人答卷回来后由大总管放 exp009 results/** · 017 覆盖更多人耳反馈（公开人评数据集 · MOS 预测器 · 音频 LLM 评委 · 作者听测提效 · 少量现场真人）· issue #42 · `research/017-more-ears`（worktree `../noise_canceling-ears`） | `_tmp/20261031-b6/ears.md` | 其 worktree `state/handoff/ears.md` | 已报到（noise-canceling-ears-03）· 实验编号 exp009 |
| judgeloop | **b7 重派 · 完成，PR #49 已合并（issue #44 关），远端分支已删，窗口已关** · 018 本机评分模型 + 调节模型闭环（D52）· issue #44 · `research/018-ondevice-judge`（worktree `../noise_canceling-judge-loop`） | `_tmp/20261031-b6/judgeloop.md` | 其 worktree | 已报到 |
| engine | **停（2026-10-31 会话中断）→ 2026-11-02 由 b8 applebe / rustbe 接续** · 019 推理引擎 Core ML / MLX / 自写 Rust 对比（D53）· issue #45 · `research/019-apple-inference`（worktree `../noise_canceling-engine`） | `_tmp/20261031-b6/engine.md` | 其 worktree | 已报到 |
| p2fix | 让 vdn-speaker 过架构闸门（拆出离线实验工具 crate · 改 tier） | `_tmp/20261031-b4/p2fix.md` | p2 worktree | 完成（tidy / check-deps 全绿），随 PR #32 合并，窗口已关 |
| applebe | b8 · 019 苹果引擎 + 总方案 · #45 · `research/019-apple-inference` | `_tmp/20261102-b8/applebe.md` | engine worktree `state/handoff/engine.md` | 完成，PR #59 已合并（issue #45 关），远端分支已删，窗口已关；worktree 保留待清 |
| rustbe | b8 · exp010 纯 Rust + 基准程序 · #51 · `exp/010-rust-backend` | `_tmp/20261102-b8/rustbe.md` | rustbe worktree `state/handoff/rustbe.md` | 完成，PR #55 已合并（issue #51 关），远端分支已删，窗口已关；worktree 保留待清 |
| fastpath | b8 · 020 / exp012 数学上更好 · #58 · `exp/012-fastpath` | `_tmp/20261102-b8/fastpath.md` | fastpath worktree `state/handoff/fastpath.md` | 完成，PR #66 已合并（issue #58 关），远端分支已删，窗口已关；剩余 P2 → #67 |
| hygiene | b8 旁支 · 代码卫生 · #62 · `feat/code-hygiene` | `_tmp/20261102-b8/hygiene.md` | hygiene worktree `state/handoff/hygiene.md` | 已派（2026-11-02 22:0x）；第二段等 99 的 #57 合并后由大总管放行 |
| hygiene | **归另一位大总管 noise-canceling-8d**（人在 8d 窗口点名；交代误写 e3，已更正）· #62 代码卫生：600 行上限进 CI · cli 同名类型 · 采样率只写一处 · 阈值比对 · `feat/code-hygiene` | `_tmp/20261102-b8/hygiene.md` | hygiene worktree | 不归本账本，进度见 hygiene worktree `state/coord-8d.md` |

## 规矩

- 见 `_tmp/20261029-b1/common.md`；本账本只记人后来改过的条目（标「人定 <日期>」）
- 调研必须用 workflow / deep research 做，要非常完整（人定 2026-10-29，原话：「需要用 workflow来做 researrch deep research 来弄，就是这个研究需要非常完整」）
- 一切走提交 · issue · PR（人定 2026-10-30，CLAUDE.md 铁律 15）：每条线一个分支 + worktree（人 2026-10-30 追加「现在必须开分之 调研分支」）：research/005-pse（Closes #1）· exp/003-offline-ab（Closes #2），各自 PR、CI 过后自己合并
- 提交由 hook 自动做（人定 2026-10-29，CLAUDE.md 铁律 14）；手下不做 git 写操作
- **大总管一次只做好一件大事，每个手下只管 2–4 个实体**（人定 2026-11-02，改 2026-10-31 版，见 CLAUDE.md 第五节第 4 条）：线 / 题 / 讨论都挂在当前实体下，其余按实体记进 `state/backlog.md`

- **自审改用 codex 换谱系评审**（人定 2026-11-02「审可以让codex来做」）：手下收尾前用 codex-review skill，连续两轮无 P0/P1 即停，结果落 `docs/reviews/`
- **codex 评审中途插入**（人定 2026-11-04「记住有的时候或者说合适的时候，需要插入 codex的交叉评审」）：耗时 ≥ 1 h 的重跑前 · 开 PR 前 · 结论给人前各插一次；派活交代里写明评审点。**中途只跑 1–2 轮**，修掉 P0/P1 就继续；多轮（连续两轮无 P0/P1）只在最后验收前（人定 2026-11-04「这种评审，基本上就是1-2轮就好……最后验收之前需要这种多伦评审」）

- **记决策号前先 `grep -n '^### D' docs/plan/001_decisions-20261029-b1.md | tail -1` 看文件最大号，并向另一位大总管发「占用 Dxx」**（2026-11-02 两次撞号：47 方 D66–D68；主检出 D77 被另一会话先用，我方改 D78）

- **所有代码文件 ≤ 600 行**（人定 2026-11-02，经 8d 转，随 #62 写进 CLAUDE.md 第五节第 7 条；不设基线、不留豁免，含 docs/experiments）：fastpath 合并前把 gtcrn_native.rs 1926 · main.rs 1280 · gtcrn_native/v2.rs 605 拆到 600 以下；#58 合并后通知 8d 扩扫描到 docs/

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试 / 跑模型（1 个） | 空（a2·fe3 借用跑评测器收口对拍后已归还；cafe 10 段成绩单改前改后逐字同；此前 card1 02:39 归还，run 20261104-000444-e4511abc） |

排队：evalfix 冒烟（咖啡馆 32 段，约 20 min）排在 cands 480 段 + DSP 草稿之后

**另一位大总管 noise-canceling-a2**（2026-11-04 起）：「前端只算一次」（D71），批次 b13 · exp015 · tmux vdn-fe · 检出 `../noise_canceling-coordfe` · 账本 `state/coord-fe.md`；我方下一个实验 exp016；会改 vdn-dsp · vdn-models · vdn-scene · vdn-speaker · vdn-engine，承诺 GTCRN 输出逐样本不变。a2 经人同意删了 16 个已合并 worktree（card1、coord99 保留）。

**跨大总管约定**（与 noise-canceling-47，tmux vdn-rust，2026-11-02）：任一方跑计时基准前后互发「基准开始，约 N 分钟」/「基准结束」，期间对方不起编译、测试、推理；普通编译测试不打招呼。加锁文件：mkdir 主目录 `_tmp/heavy.lock`（内写 owner · 开始 · 预计结束，跑完 rmdir），由各自大总管放名额时建、收回时删，手下不碰（47 已确认）

## 进度板

进度（HH:MM）
- 2026-11-06 00:xx #134（030，Closes #124）合并，aux 关窗清理；开 #135（sweep 指纹 P2）· #133（hook）。四条路总览 PDF 上桌面；人选 D90：H-1 先重测再盲听（派 cands，分支 exp/021-mix-retest）· H-2 微调与注意力头一起做 · T-1 音色线 3 暂定 · L-1 盲听页已复制到 ~/Desktop/20261106-blind-listen-gtcrn-dfn3/
- 2026-11-05 16:0x 已合并 #130（016 勘误）· #131（027 微调，Closes #121）· #132（028 改结构，Closes #122）；main 全量 746 过 0 败。人定 D88 云 GPU 训练（数据等开训练线再下到云上）· D89 旁人说话交给另外的小模型（backlog）。finetune · arch · rl · timbre 关窗清理；剩 aux（扫 GTCRN 中）与 cands（等盲听）。桌面 PDF：026 · exp016（含勘误版）· 029 · 027 · 028
- 2026-11-05 15:4x 人定 D87：训练数据先不下；CI 账单停摆期按本机验收合并 → #120（81730e0f）· #127 · #128 已合并（PR 留言写明），#119 #112 #123 关闭；索引冲突由我两行都留。待：main 上全量测试（排在 aux → finetune 测速 → arch 测速之后）。三份结论 PDF 放桌面（026 · exp016 · 029）。timbre 关窗；cands 留着等盲听揭盲
- 2026-11-05 15:2x PR #120（timbre）· #127（cands）· #128（rl）都已开、本机亲手验收全过；⚠️ GitHub Actions 因账单付款失败不启动（「recent account payments have failed or your spending limit needs to be increased」），已问人：修账单或授权按本机验收合并。rl 收尾关窗（结论：RL 现在不做，排在微调之后）。名额 aux 扫混回（砍 DPDFNet-2，改扫 GTCRN）
- 2026-11-05 15:4x 人要多开手下比较 GTCRN 改进的几条路 → 四线：finetune 027 #121（微调+数据）· arch 028 #122（改结构/注意力头）· rl 029 #123（强化学习）· aux 030 #124（辅助判场景 + 混回比例，不重训）。共同交代 _tmp/20261104-b15/gtcrn-options-common.md，统一比较卡。exp 号预分：018 finetune · 019 arch · 020 rl · 021 aux
- 2026-11-05 15:2x 人定 D86：加训 GTCRN（微调 · RL · 加注意力头都在范围），不换模型；DPDFNet-2 前视 4 帧结构定死出局，DFN3 留作对照。开调研 027 #121，手下 finetune（vdn:finetune，worktree ../noise_canceling-finetune，实体：GTCRN 模型 · 训练数据集 · 训练方法）。timbre PR #120 已开（我在 1265fe37 亲手验收 745 过 0 败），等 CI
- 2026-11-05 12:4x skillfix 收尾：codex-review 隐私扫描已修（iconv → perl 严格校验；23 测试我亲手重跑全绿；timbre 原文件过、假密钥仍拦；codex 2 轮无 P0/P1）。技能真身在 2-aartifacts/skills/（不在 git，备份为按记录还原版）
- 2026-11-05 12:2x 人要「专门让一个手下来修」codex-review 隐私扫描误拦（P1/P2 开头被 file 认成图片，第二次）→ 起手下 skillfix（tmux vdn:skillfix，交代 _tmp/20261104-b15/skillfix.md，实体只一个：该技能的隐私扫描；先备份、先写红测试、不放松规则）
- 2026-11-05 11:57 timbre 名额 11:48–11:56 用完：旧代码三系统 243 项逐项同基线；新 ERB 量法提交 88de9880，对拍 0.000 dB；golden 只 V-TIMBRE 变；基线只 36 项 V-TIMBRE 变、通过数不变（GTCRN 2.27 · FE-M 2.02 < 3）。名额 → cands 跑 DPDFNet-2（约 13:15），之后 timbre 全量测试
- 2026-11-05 11:48 cands DFN3 成绩单跑完（run 20261105-103000-4cf04ac1，77 min，0 段失败，缓存 1920 条原样 + 新增 960）；DFN3 降噪 +2.16 dB、听感 +0.05、可懂度 +0.01，咔哒 +6/min、字头 −0.93 dB、背景稳 −0.74 dB；组合 过1/不过42/待测17 vs GTCRN 0/40/20。名额 → timbre（第 1 段半 + 第 2 段），之后回 cands 跑 DPDFNet-2（约 80 min）
- 2026-11-05 00:59 evalfix 全量重出 run 20261105-005916-4ccf0638 开跑（权重哈希 · 工作区干净 · 接电源 · build 17f5 已核），预计 03:30；延迟预算 planned_platform_for 进 vdn-types（只加法，16 kHz 逐位不变）；cands 等合并后跑两候选最终版
- 2026-11-05 00:07 evalfix 中途评审② 2 轮 P1×3 已修（SI-SDR 全零 · 干净人声掩码 · 重叠段桥接），申请冒烟名额 → 排在 cands 后；cands 草稿只跑 DSP 项，最终版只跑两候选、与 evalfix 的 GTCRN run 配对
- 2026-11-04 #115 外部入口 CI 过、已合并（分支保留）；名额给 cands 跑 DPDFNet-2 + DFN3 低延迟版 480 段（约 30 min）；NoNoise-Mac 人未选 → 默认 C 推迟；待告诉 a2：spectrum_golden 改冻结法（a2 会话现不在线）
- 2026-11-04 23:3x 会话重启，大总管改名 noise-canceling-26，已通知两手下；cands 入口 PR #115；evalfix spectrum_golden 改冻结法（技术题我定，要告诉 a2）
- 2026-11-04 人逐题定评分方法 → D85（G-1 B · G-2 B 干净人声单独跑 · G-3 C 单段 ≥ 5 字否决 · G-4 A · 4 dB · 0.1 · 不听咔嗒）；已转 evalfix。evalfix 首次编译名额已还；对 cands 外部入口提 3 处公平性（延迟标声明值 · 最坏一帧判不知道 · 截断查偷看未来），我同意
- 2026-11-05 人问音色项「代码和文档到底谁对，要研究和证据，开手下做，有错就修」→ 派 timbre（调研 026 · #119 · worktree ../noise_canceling-timbre · 分支 research/026-timbre-measure；实体：音色项 · N4 条文 · 判定线）；交代 _tmp/20261104-b15/timbre.md；名额排在 cands 后。更正 #119「没抓到」→ 已在 PENDING 登记
- 2026-11-05 机器 06:59 关机 07:00 重启（shutdown_stall 报告，起因未查）：tmux 与手下会话全断，大总管改名 noise-canceling-74；两个候选离线新 run 完整（各 mix/clean 480），锚点成绩单 065944 中断作废；名额重发给 cands，`claude --continue` 接回（交代 `_tmp/20261104-b15/msg-cands-resume.txt`），从锚点成绩单接着跑。evalfix 窗口随重启消失，worktree/分支仍待人删
- 2026-11-05 #116 CI 过（macos 4m18s）→ 合并 90ed08f2；关 #111 #109 #110；开 #117（e2e 夹具并发）#118（候选复位）；evalfix 验收通过、worktree 干净，笔记另存主目录 _tmp/evalfix/worktree-scratch-20261105/；关窗 + 删 worktree / 分支被权限拦下，等人处理；通知 cands merge main 后申请名额（估 3–3.3 h，含直通锚点 + 识别漂移核对）
- 2026-11-05 合并提交 bdf0ad57 + 987a3af8（cands 的 clean/ 入口补丁 + runner 没给 clean 时那 3 项只报不判、开头写明）；我亲跑 fmt · tidy · clippy 过，全量只一次 external_e2e 假红（与 evalfix 并发互删夹具目录，开 #117 P2），单跑 2 次 11/11 过；推送开 PR #116，等 CI。cands 最终跑定为另开新 run（mix + clean 一次跑全，不补旧 run、不写 amend），估 2.5–2.8 h，开跑前 codex 审 1 轮
- 2026-11-05 evalfix 收尾（codex 收尾 2 轮无 P0/P1，P2×5 已修，e3be788d）；待人 2 题记为 G-8 G-9。我在其 worktree 合 origin/main：解 main.rs · 评审索引冲突，system.rs:471 测试补 build 第二参数；全量只红 external_e2e 主测试（外部入口 × v2 干净人声路查不到输出，推断）→ 交 evalfix + cands 对齐修，合并未提交
- 2026-11-05 我验收 evalfix：tidy 过；全量测试首跑 vdn-schema 红（eval-suite.schema.json 少导出 4 个新评分项名）→ evalfix 重导出；--no-fail-fast 全量 0 红；名额释放；evalfix 开收尾多轮 codex 评审。教训：亲跑一律 --no-fail-fast（已记 memory tests-by-default）
- 2026-11-05 evalfix 全量 run 20261105-005916-4ccf0638 跑完（9986 s，1440 段，0 没跑成，规则 v2，供电 ac），名额归还；我核 clips.jsonl 1440 段齐；GTCRN 场景 0 过 / 40 不过 / 20 待测（只汇总版 1 / 42 / 17，请它说明变化），PSE 重叠项 passthrough 均值 0 与失败率 = 1 的 242 段待它说明；我占名额亲跑 tidy + 全量测试
- 2026-11-04 人选继续「降噪效果」（评测集）→ 合并 #97 #108；开 #111 #112、两个 worktree；派 b15 evalfix · cands；G-1..G-7 按默认推进，待人确认；合并顺序 cands 入口小 PR 先、evalfix 后
- 2026-11-04 evalbp 收尾：025 结论「算法大多对得上，偏在汇总（任一段不过 vs 均值 + 区间）」；codex 9 轮清零；待定 G-1..G-7 合进清单（替代 F-9 F-10）；开 #109 延迟测量器、#110 V-WEAK 低采样率；PR #108
- 2026-11-04 a2：#107（评测器 4 处改调统一频谱模块）CI 真跑通（run 41866359378，main+#107，跳过 0），已合并 8fea2ef2，fe3 worktree 与分支已删，a2「前端只算一次」收口、暂不碰 src/eval；a2 会话 auto-commit 在主检出替我提交了 207a847f（coord + 待定，原样，不丢东西），提交信息照留
- 2026-11-04 人（经 a2 转）「具体的评测方法，要看是否和好的 best practice 一致」→ 评测器归我方，派 evalbp 做调研 025（#105，b14，以 012 为底，给 F-9 / F-10 业界依据）；a2 派 fe3 做评测器频谱收口 #104（只换计算）
- 2026-11-04 允许 a2（D82 第二段）把评测器 estoi.rs · speech.rs · system.rs 的频谱计算收进 vdn-dsp：单独 PR、不改评分逻辑，cafe 常见档 10 段成绩单 JSON 逐位对拍；开工、合并前通知我；我方 F-9 或评测器 issue 动这些文件前先找 a2，先开工者先做
- 2026-11-04 #97 与 main 冲突（评审索引）→ 两边都留，08c59771 已推
- 2026-11-04 a2 fecore 对拍：基准在 990e7843 生成，改后 GTCRN 输出与引擎最终输出都逐位相同（src/models/tests/golden.rs）；exp014 成绩单在 a2 改动后仍可比
- 2026-11-04 a2 占用 D82（认人收进统一频谱模块）、D83（放弃认人门控，改做个性化降噪 PSE：同时说话也只留用户）→ a2 再占 D84（E-4 按需启停）→ 我方下一条从 D85 起；a2 占用调研 024（个性化降噪 / 目标说话人提取）→ 我方调研从 025 起；V-SIM-GATE / V-SIM-FA 这类门控评分项的意义要随 D83 重看（F-9 一起问）
- 2026-11-04 card1 收尾：成绩单 run 20261104-000444-e4511abc；评分项自检（V-SIM-GATE 对原声应「不适用」；V-PUMPV/V-TIMBRE 把残留噪声算成人声变化 → 看配对差值；V-INTF 口径对）；codex 3 轮清零；PR #97；人耳 7 题放桌面 20261104-vdn-gtcrn-listen；待定 F-9 F-10
- 2026-11-04 card1 带 --qa 10 段实测 193 s（3 系统，CPU 约 56%，吃字与听感都出分）→ 480 段约 2 h 35 min，开跑
- 2026-11-03 card1 冒烟过（cafe 常见档 ref 2.6 s · GTCRN 7.5 s）；名额给它正式跑（约 1–2.5 h，先量 10 段实测耗时）。观察：原声直通在 cafe 上 V-TIMBRE · V-PUMPV · V-SIM-GATE 也不过，待分析
- 2026-11-03 人「你直接帮我merge」→ #88 #91 合并（#91 前在其分支 merge main、解索引冲突、DATA_FILES 登记 scenes.json，131 测试过）；人「就留下第五版吧」→ 删 _tmp/exp013 前四版音频（约 13 GB，清单仍在仓）；人要手下跑成绩单 → 派 card1（b12）
- 2026-11-03 scenesuite 收尾验收过，窗口已关；b11 两线全收尾，tmux 无窗口。卡在人：#88 合并 · skill 第 269 行修复
- 2026-11-03 scenesuite 第五版（正式）build 20261103230756-17f5 通过（480 段 · 混响 0 不过 · SNR 99.4% / SIR 98.6% 段差 ≤ 1 dB · 68 测试 · 本机路径 0）；scenes.json 指向它；五版音频共约 16 GB 都在 _tmp，去留待定
- 2026-11-03 scenesuite codex 4 轮达停止线（15 条全属实，改 13，剩 2 条 P2）；第五版（最后一版）build 名额给它；开 issue #89（设备链后有效 T60 / DRR）· #90（codex-review 位图误判，skill 改动等人执行）
- 2026-11-03 codex-review 误判位图根因【实测】：macOS iconv 输出直接写 /dev/null 且超过约 100 字节就报「Inappropriate ioctl for device」，privacy-scan.sh:269 的 UTF-8 复核因此失败 → 长的 codex 输出被拦。修法：`iconv … -- "$r" 2>/dev/null | cmp -s - "$r"`。改 skill 被权限检查拦（自我修改），等人。016 synth-standard 第 45 行改为按时段切（S10①）
- 2026-11-03 scenesuite 第四版 build 20261103223725-72df 通过（同房间 37.5% · 混响 0 不过 · SNR 99.2% / SIR 98.2% 段差 ≤ 1 dB · 63 测试）；codex-review 隐私扫描把「P1」开头的输出误认成位图，第二次出现（macmic 一次）→ 改结构
- 2026-11-03 scenesuite codex 第 1 轮 P1×3 P2×2 + 实测房间自审已修（DEMAND 4 个未用环境按时段切 · ESC-50 截尾 · 4 条 MIT IR 直达起点 · build 自验电平 · 混响检查按声音类别）→ 第四版 build 名额给它；63 测试过。大总管定技术题：第 5 题按时段切（S10① 允许：前一半测试份、后一半裁判原料）· 第 2 题合成不像的声音在成绩单标「合成，待真实录音校」· 第 3、4 题已定（人选 A；阈值暂定）
- 2026-11-03 scorecard 收尾：大总管复跑 vdn-eval + vdn-schema 131 过 · tidy 过 · schema 一致 · levels 原 16 项未变；开 PR #88，CI 过；合并被权限检查拦（上次「可以 merge」只覆盖 #83–#85），等人；窗口已关。转交：scenesuite 合并时在 src/schema/src/eval.rs DATA_FILES 加 scenes.json 一行；ci / qa / calib 报告会覆盖旧文件（旧代码）→ backlog
- 2026-11-03 scenesuite 第三版 build 20261103222950-de56 通过（480 段 · 同房间 37.5% · 混响检查 0 不过 · 54 测试 · SNR/SIR 99.2% 段差 ≤ 1 dB）；在查「三版削波都是 0」
- 2026-11-03 scorecard 合并前复跑：nrmos_parity 3/3（SCOREQ 差 0 · Distill-MOS 0.00225）· 全量 release 84 组过；名额转 scenesuite
- 2026-11-03 scenesuite 混响修法实现（54 测试过；三包大小与方案一致；预演 240 段约 37% 人声与底噪同房间；reverb_check 480 段 0 不过、6 个负对照全红；修 MIT 列车 IR 直达峰误判；SCAFE 户外露台拆出咖啡馆类）→ 申请第三版 build，排队等 scorecard
- 2026-11-03 scenesuite 混响方案批（一段 = 一个房间；6 个地点同房间实测 RIR + 噪声：ACE · McVAMPIRE；其余 MIT 271 条实测 IR 按 T60 区间配 DEMAND；公交 / 地铁 / 机舱 / 候车厅标「非同房间」；检查改生成端约束 + 清单比对）。人在问答里选下载 A 约 685 MB；T60 阈值 ×[0.8, 1.25] 大总管定为暂定
- 2026-11-03 scorecard V-CLICK 排查：自造平滑增益负例原误报 1746 次 / 分钟 → 修算法（离群跳变 · 干净人声解释不了 ≥ 一半 · 背景分轨排除）+ 两条负例测试；参考降噪器 cafe 约 100 → 28 次 / 分钟，剩余机器分不开，5 处样本在 `_tmp/scorecard/click-check-20261103/`。大总管定：V-CLICK 先只报不判，等 GTCRN 成绩单再决定要不要人听；P06 拆为 30% xfail + P10 40% 必须判不过；scorecard 起 codex
- 2026-11-03 scenesuite 第二版 build 20261103215813-c025 通过（480 段 · 40 测试 · 0 削波 · scenes.json 60 条）；两版音频共 6.5 GB 都留；codex 推迟到混响修完
- 2026-11-03 人试听 b22d 反馈：说话声与场景底噪的混响不一致（原话「很多说话声和场景实际上能给的环境混响不一致啊」）。原因：底噪是真实地点录音、人声套随机 T60 的模拟长方体房间。已转 scenesuite 出修法方案（同房间实测 RIR + 噪声如 ACE · 实测 RIR 库 · 加「人声 vs 底噪混响一致」机器检查）
- 2026-11-03 scenesuite 自审修 7 项（干轨提前 1–2 ms · 会议室电平 · 补 7 种声音 · 风噪 · 有规律声音电平 · 方位角 · 防泄漏加对照 exp006 test/seq 只报告）+ 场景任务改 60 条（加外接麦对照）→ 名额给它出第二版 build；第一版保留
- 2026-11-03 scorecard 名额归还（通过）：eval ci 门禁过、探针 9/9；全量 release 83 组过；gtcrn 因 worktree 无权重被跳过 → 告知 VDN_GTCRN_ONNX；P06 由 30% 改 40% 打回 → 30% 留 xfail 记局限 + 40% 必须判不过
- 2026-11-03 scorecard：成绩单命令 · scorecard.md（与代码比对）· 片段清单 schema 写完；cafe 常见档试跑能出表；V-CLICK 在参考降噪器上约 100 次 / 分钟（直通为 0），疑误报待查；external-mic 240 段未进任务 → 转 scenesuite；名额给它跑 eval ci + 全量测试
- 2026-11-03 人要试听 → 大总管从 build b22d 挑 11 段（8 个场景常见段 + 家里补一段 + 咖啡馆压力段 + 不加麦模拟对照）调到约 −23 dBFS 复制到 `~/Desktop/20261103-vdn-scene-samples/`（原件不动，非盲听），等人反馈「像不像真的」
- 2026-11-03 scenesuite 第一版 build 20261103214702-b22d 通过（480 段，18 s，音频 3.2 GB 在 _tmp；清单 / 哈希进仓；39 测试 · 防泄漏三项交集 0 · SNR / SIR 98.2% 段差 ≤ 1 dB）；scenes.json 30 条 graders 待 scorecard 填；锁已删
- 2026-11-03 scenesuite 下载完成实测约 447 MB（单包最大 130 MB），登记 materials.json；自报冒烟跑满规模 17 s 未申请名额（已记，以后整批先申请）；名额给它跑第一版 build（480 段 = 15 地点 × 16 段 × 2 设备，约 2.5 GB）
- 2026-11-03 scorecard 名额归还（通过）：Distill-MOS 导出 ONNX（复数 STFT 换实数卷积，对原模型差 1.4e-6，17.6 MB 在 ~/vdn-data，哈希进仓）；对拍 6 段 SCOREQ 差 0、Distill-MOS 0.00225；onnx 包另装在主目录 _tmp 29 MB；锁已删
- 2026-11-03 scorecard：4 个新评分项 + 探针 36 测试过（ESTOI 对 pystoi 差 0）；批改 src/schema/（片段清单 schema 导出与正负例）；levels.json 手加 6 条打回，改为 calib --write-levels 自动补新项为 L0 + 测试
- 2026-11-03 人「可以 merge」→ #83 #84 #85 按序合并（#84 #85 与评审索引冲突，保留两行解决），#80–#82 关；两个 b11 分支已 merge main。scorecard 方案批（SCOREQ ONNX · Distill-MOS 导 ONNX 后对拍）；scenesuite 方案批（15 × 16 段 × 12 s = 240 段；防泄漏按 LibriSpeech 奇偶位 + DEMAND 后 40%；下载 ESC-50 约 75 MB + DEMAND 三包约 350 MB，累计约 425 MB，大总管批）
- 2026-11-03 人定 D81（按场景测能力 → 成绩单 → 针对优化；听感用 SCOREQ 主 + Distill-MOS 交叉）；派 b11 两线 scenesuite · scorecard
- 2026-11-03 macmic 路径清零复查过，窗口已关；开 PR #84（022）· #85（023）。三线全收尾；合并被本机权限检查拦，等人。人新方向：按场景测能力 → 成绩单 → 针对优化（待确认后记决策）
- 2026-11-03 macmic 报收尾（codex 5 轮，4/5 轮无 P0/P1，22 条全改；12 测试 · 21 键 0 问题，大总管复跑过）；打回 1 处：findings/ 10 处本机路径。同类第二次 → 新记忆 `state/memory/20261103-no-local-paths-in-research.md`。另记：codex-review 隐私扫描把「P1」开头的输出误判为 PBM 位图（macmic 现状页「发现」），属 skill 问题，待告诉人
- 2026-11-03 macmic 发现 devices.md 第二节模式值被当设备键读（25 → 21 键），已修并加检查；大总管复跑 023 检查 0 问题，改 addendum 一处 unknown → non-laptop-unknown（3c9b779）；macmic codex 第 4 轮无 P0/P1，第 5 轮在跑
- 2026-11-03 PR #83 CI 过；大总管合并被本机权限检查拦下（「未经评审合并」），等人放行或人手动合并
- 2026-11-03 scenes 路径清零复查过，推送、开 PR #83（先于 022 / 023 合并），窗口已关
- 2026-11-03 scenes 报收尾（codex 5 轮，4/5 轮无 P0/P1，23 条全改）：空格 43 → 40；电视人声 SIR 改为「未知，0 ± 6 dB 只是测试假设」；三处不再写「只能自录」。验收打回 1 处：raw/ 三处本机绝对路径（铁律 12）。DNS 2020 实录 · Krisp 测试集两条候选记 backlog
- 2026-11-03 macmic 5 份交付齐，codex 第 1 轮在跑。关键发现【读系统文件】：本机系统调音文件显示，第三方普通采集拿到的 1 路内置麦已经过「每麦降噪 + 固定波束成形 + 均衡」（022 `findings/tunings.md`）。转交 mtgdata 的 DR-VCTK 改判由大总管在 023 分支改（d3743ac）并回写 D65；1 题待拍板（要不要在人的 Mac 上实测）
- 2026-11-03 scenes 五样产出齐，codex 第 1 轮在跑：无公开「按开会地点」分类，15 地点为本项目自定；15 地点 × 9 因素有 43 个空格（距离 · 突发 · 声级与 SNR 联动整列空；家里客厅、会议室 0 条）；4 题待拍板。mtgdata 已关，其转交（catalog v2 只加列 · DCASE 2016 声景线索 · AliMeeting RT60）由大总管接：catalog v2 下 check_cards 复跑 44 卡 0 问题；线索记 backlog。转交大总管：噪音 7 类缺 4 种声音 → backlog 场景分类器
- 2026-11-03 mtgdata 收尾验收过，窗口已关
- 2026-11-03 mtgdata codex 第 3 轮 P1×1 P2×5 全改；「没有…公开集」说法过强第三次出现 → 按规矩改结构（三份汇总页首加措辞口径）；第 4 轮在跑
- 2026-11-03 mtgdata codex 第 2 轮 P1×2 P2×4 全改；发现调研代理沿用已作废 D26（SA 只评测）→ 按 D64 改 5 张卡为可训练；放开它改 014 cards/README.md「结论」列（铁律 5）；第 3 轮在跑
- 2026-11-03 mtgdata codex 第 1 轮 P1×7 P2×3 全改（补 MOBIO Mac 录音 · DR-VCTK 只评测 · ACE 移出真实锚 · 下载估计改为 < 10 GB / 含 LIVETALK 整包约 20–25 GB）；第 2 轮在跑；检查脚本修复已验收
- 2026-11-03 mtgdata 6 份交付写完、codex 第 1 轮在跑；4 道待拍板（开发阶段用不用真实录音评测 · 下多少 · 三格要不要自录 · 申不申请 Ego4D）等三线齐了一起问。打回 1 处：check_cards.py 去旁边 worktree 找键表 → 改只认本仓库 + 环境变量显式指；合并顺序 021、022 先于 023
- 2026-11-03 macmic 调研已起（wf_16dd3e65-ebc）；设备键表 v1：加 mac-unknown 与 non-laptop 8 个子键，原 12 键不改名
- 2026-11-03 scenes 调研已起；键表 v1：15 个地点 + 兜底 other（加 bus-metro-cabin），三方已对齐
- 2026-11-03 macmic 设备键表 v0（12 个键）已发；调研计划批准（4 路 + 4 怀疑派 = 8 代理；补：Air 单列 · 嘈杂环境录音什么样 · 可执行的 Air 麦模拟步骤）
- 2026-11-03 scenes 草稿键表 v0（14 个地点）已发；调研计划批准（5 路 + 2 怀疑派 = 7 代理）
- 2026-11-03 mtgdata 四路调研完（44 张新卡，检查 0 问题）：无「Mac 内置麦 + 会议 + 设备标签」公开集；开放办公 · 机场车站 · 街边只能自录；家里 MISP2021；笑声附和 ICSI；约 20 项要注册。Mac 材料已转 macmic。scenes 因大总管漏回报到停了约半小时，已催
- 2026-11-03 mtgdata 调研计划批准（4 路 + 4 怀疑派 = 8 代理；补：按设备筛笔记本段 · 笑声 / 附和 / 情绪语音 · 与 D62 关系只列事实）
- 2026-11-03 b10 派三条调研线（scenes · macmic · mtgdata）补评测维度；名额不需要（只调研）
- 2026-11-02 约 22:40 e3 窗口收工：人「这些东西下一阶段要做的东西，都记下来，后面的窗口可以再定」→ 待拍板第〇·五组 E-1–E-3 · backlog · board 接续点已写；名额空、锁无
- 2026-11-02 fastpath 收尾：PR #66 合并（默认 native-v4i · codex 3 轮 · 600 行拆分）；剩余 P2 → issue #67；转交「接起来」：产品接入 v4i、src/dsp 重采样多累加器；测速提醒：rt 连续跑会被降级，连续峰值用 --qos ui，长计时 caffeinate + 查电源
- 2026-11-02 fastpath 600 行拆分完成：gtcrn_native.rs 1926 → 445（+ weights / cost / kern / ops / block / dpgrnn / simd / tests）· main.rs 1280 → 485（+ alloc / nets / util / parity），最大 532；35 测试过、11 个后端对 ORT 指标拆前后一字不差；已 merge main 并删 fields.rs 三条 APPLE_ONLY。下一步结论 + codex
- 2026-11-02 22:13 fastpath v4short（电池，3 轮，第 2 轮受干扰）：16 ms 节奏每帧平均 v2-fast 0.091–0.135 · **v4i 0.073–0.092 ms**（同轮比 0.62 / 0.93 / 0.80），CPU 0.5–0.7%，0 帧 > 3 ms；ui 整帧 v2-fast 43.7–48.7 → v4i 35.8–36.2 µs（快 17–26%）。逐层：两输出一组卷积 en0 −30% · 深度卷积 4.35 → 2.77 · LN 2.59 → 0.48 / 3.18 → 0.99 · GRU 交错仅 −7%。**AI 定默认候选 v2 → v4i**。600 行拆分进行中
- 2026-11-02 22:03 fastpath v3short（电池，3 轮有效，负载 2.90）：16 ms 节奏 rt 每帧平均 **ORT 1.77–1.92 ms（CPU 11–12%，2 轮有 > 3 ms 帧 4 / 7 帧）** · v2-fast 0.125–0.130 ms（0.9%，约快 14 倍，0 帧 > 3 ms）· v3 0.130–0.134（没快）。分拆：深度卷积 3.8 → 2.7 µs、LN 2.3 → 0.45 / 2.8 → 0.95 µs 有效；倒数代除法让 GRU 7.7 → 8.8 µs 变慢（假设被推翻）。下一步 v3 改回除法 + GRU 显式交错（v3i）
- 2026-11-02 22:00 手下 hygiene（#62 代码卫生）归大总管 noise-canceling-8d（人在 8d 窗口点名），交代误写 e3 已更正；已转 99 对齐要点；8d 计时前照旧打招呼
- 2026-11-02 fastpath v3（按能效核写）：sigmoid/tanh 除法换倒数估计 + 2 次牛顿 · 3×3 深度卷积向量化（原离下界 5.1 倍）· LN / ERB 4 路 NEON 累加（原离下界 10–12 倍）；133.12 dB、最大差 7.6e-6、0 分配、32 条测试过；短计时同场加 ORT
- 2026-11-02 人定停测：「我觉得不用测了，基本上效果非常好」→ fastpath 停 paced-batt2；随后人改「你可以继续优化，然后schema他们那个可以开始做」→ fastpath 不收尾、继续 A 类优化，机器先给 99；耗电 · 插电节奏组 · 预取整帧标「待产品接入时测」
- 2026-11-02 fastpath 电池组 v2batt（3 轮全有效）：16 ms 节奏 rt 每帧平均 原 0.50–0.59 · v2 0.18–0.21 · v2-fast 0.14–0.17 ms，均无 > 3 ms 帧；CPU 原 3.2–3.8% · v2 1.2–1.4% · v2-fast 1.0–1.2%；重采样产品 60 µs/帧 → NEON 14 µs（差 2.4e-7）；STFT 7.3 + 4.9 µs；电量 35 min 内显示不变（满电附近不更新）
- 2026-11-02 人定 D79：优化与测速以能效核为准；先做输出不变的改写，改输出的留到降噪效果大事
- 2026-11-02 人要测不插电性能（「我其实想测试一下不插电的性能」）→ 电池组与插电组分开跑、分目录
- 2026-11-02 更正：run_v2.sh「卡住」不是程序错误——20:07 拔电、20:12 空闲睡眠、20:54 唤醒；单调时钟睡眠时停走。round 2/3 是电池数据（性能核乘加吞吐 66.9 → 35.4 /ns），旧目录留作电池对照。fastpath 已加：睡眠检测作废 · wait_due 测试 · 脚本 caffeinate + 每轮查插电 · v2-fast sigmoid 夹 ±87 防次正规数；27 条测试过
- 2026-11-02 20:55 fastpath run_v2.sh 卡住：round 2 native-v2-fast pace 16 进程跑 43 min（应约 96 s），停在 main.rs:674 sleep → 让其自停、修等待计算、加测试、只补跑缺的；锁延到约 21:30
- 2026-11-02 fastpath peak1（3 轮 × 6000 帧，插电，负载 5.7 → 1.1）：实时节奏 + rt 手写每帧 0.38–0.55 ms ≈ 能效核 0.59 ms，是连续跑 0.091 ms 的 4–6 倍 → 推断实时节奏下在能效核约 1 GHz（乘加延迟 E 4.4 ns / P 0.77 ns），下界与耗电按能效核算；两项微基准量错待重跑
- 2026-11-02 19:49 applebe 收尾：PR #59 合并（codex 8 轮 41 条全改，7、8 轮无 P0/P1）；fastpath 编译 + 17 条单元测试过、端到端仍 134.00 dB，开始计时
- 2026-11-02 applebe：Accelerate 措辞已写；`apple-rs/tests/fields.rs` 以 bench 为唯一真相源比对 JSON 键（抓到 late_* ≠ release_* 同名不同义并改正）；main.rs 4 条 + power_parse 4 条测试通过；codex 第 6 轮在跑。转交：bench 补计时窗字段（win_start_unix / win_end_unix / wall_s），否则手写版耗电不进排名 → 派给「下界分析」手下
- 2026-11-02 rustbe 收尾：PR #55 合并（0 分配整数证据 6000 帧 · 10 条单元测试 · 形状交叉校验）。AI 定：Accelerate「cblas / vDSP 路线已不采用；BNNS 路线待证实」；JSON 字段交叉校验由 applebe 在 apple-rs 加测试
- 2026-11-02 19:25 rustbe 报可以收尾（codex 5 轮，P1×7 P2×20 全改，第 4、5 轮无 P0/P1）；缺口：评审后改动只 cargo check → 给 5 min 名额重编 + 小对拍再开 PR。其待拍板：①手写替换 ORT → AI 定「随『接起来』大事做，挡线补齐前不进产品」②FE-M 去留（已在总清单）③报 tract bug → 4-12
- 2026-11-02 19:01 rustbe rt1（单次 60 s，只看方向）：按节奏手写 p50 ui 0.39 · **rt 0.21** · ui+工作组 0.62 · rt+工作组 0.61 ms（工作组变差，但设备未跑音频 IO）；同为 ui，10 min 那次 0.61 vs 这次 0.39 → 单次波动大于设置间差；native-fast 过对拍（133.4 dB），连续 0.0838 vs 手写 0.0898 ms。名额超时约 8 min（重编 9 min）。issue #53（engine NaN）已转 47
- 2026-11-02 18:43 人跑 powermetrics 矩阵完成（`apple/results/20261102-183322-rt/`，9 设置 × 60 s，插电）：CPU 功率 native-blas rt 71 mW · native rt 81 · BNNS rt 97 · BNNS + 自建工作组区间 123–138 mW（但每帧 0.80 → 0.2 ms、核 4.8% → 1.4%）· 忙等 4.2 W（证实降频）。空闲基线 111 mW 高于多段 → 背景噪声约 ±30–40 mW，只作相对比较。结论：支持 D74（提速花电），功率最低 = 手写 + Accelerate + time-constraint
- 2026-11-02 18:31 ⚠️ applebe `impact.sh` 留下 21 个孤儿基准进程（`eval "$CMD" &` 的 `$!` 是子 shell，kill 没杀到程序），18:10–18:31 一直按节奏跑约半个核 → BE-IMPACT 数据作废；人授权由大总管 pkill 清理；脚本已改（exec · kill+wait+kill -0 · trap）。教训：后台起进程的脚本必须 trap 清理并核实退出
- 2026-11-02 applebe：ECAPA 转不出 Core ML（补了 onnx2torch Clip 后又卡在 coremltools 不支持 prod），记「转不出」；BE-IMPACT 不能下结论：7 次「关」之间离散 10–22%，BNNS rt 慢 5.9%（噪声 10.6%）、手写 rt −0.6%（噪声 22%），都在噪声里 → 测法要改（前台负载不占满核 · 多跑几对 · 取最小值），留下次。暂停让 47 陪人跑真设备小测试（约 2 min），之后人跑 powermetrics
- 2026-11-02 18:02 rustbe 实时节奏 10 min（37 500 帧，ui，插电）：手写 p50 0.61 ms（连续跑 0.088）· p99.9 0.87 · max 6.95 ms · 6 帧 > 3 ms（BE-DEADLINE 不过）· 3.2% 核 · 0 分配；Accelerate 版 p50 0.44 · max 4.17 · 1 帧 > 3 ms · 2.5% 核（实时节奏下比手写快约 21%，原因未查）。结论：连续跑的帧耗时在产品节奏下不成立
- 2026-11-02 人答 D69/D74 冲突 → D75：看实际耗电（mW）+ 对其他程序性能的影响（固定负载变慢 ≤ 1% 目标 / 2% 上限），活动监视器 CPU% 只作参考
- 2026-11-02 47 方决策（其分支，编号已顺延）：D73 最低系统 14.4；D74 降噪线程接受低频（能效核约 1.1 GHz，0.7 ms 计算变约 3 ms；功耗 72 / 77 / 101 mW；time-constraint 默认开、Workgroup 默认不加）。⚠️ D74 与 D69「CPU 1%」口径打架：低频下 CPU% 高、耗电低 → 待问人。编号约定：记决策前看对方最大号、发「占用 Dxx」；我方下一个从 D75 起
- 2026-11-02 rustbe ①：Accelerate 版逐值一致（134.2 dB）但只快 4.7%（0.0844 vs 0.0886 ms），每帧 11 次分配（Accelerate 内部），过不了零分配；全进程每帧分配：手写 0 · ORT 1557（FE-M 1023）· tract 2247；bench 加了 `--qos rt` 与 `frames_over_3ms`。② 实时节奏长跑到 18:07
- 2026-11-02 17:42 applebe 归还：苹果引擎最快 = BNNS Graph + GRU 展开，GTCRN p50 0.155 ms（ORT 0.384，快约 2.5 倍）、每帧 0 分配、134 dB；手写 Rust 0.088 仍最快。Core ML CPU fp32 0.39 / 展开 0.21 ms、每帧约 300 次 malloc、所有算子落 CPU、fp16 54 dB 不过；MLX Python 5.7–6.2 ms；FE-M 转不了 Core ML（view_as_complex）；修了 coremltools 分组 ConvTranspose2d 错转。⚠️ **按实时节奏 16 ms/帧跑 10 min：p50 0.82 ms（连续跑 0.155）、max 11.8 ms、4.8% 核，超 D69 上限，BE-DEADLINE 不过**——推断降频或被挪到能效核。DFN3 `_ll` 下界：连续跑三图 p50 0.61 ms / 10 ms ≈ 6.1% 核 → 不做流式重导出（AI 定），盲听用离线处理即可
- 2026-11-02 人同意：GTCRN 主模型（D33）当假设继续验证 → 下一轮盲听放 DFN3 `_ll`（backlog 人耳）；019 把 DFN3 加进必测，看 1% 预算内跑不跑得动
- 2026-11-02 17:30 applebe 主基准跑完，在跑 10 min 实时节奏超时帧检查（到约 17:40）。venv 核清：实际下载约 270 MB（exp010-venv 约 200 MB + 新建转换用 exp010-venv-ct 约 73 MB，因 coremltools 9.0 在 py3.13 崩），无单次 > 500 MB；du 982 MB 是 torch 解压后体积
- 2026-11-02 17:13 rustbe r5b：手写 GTCRN 逐位不变优化后 p50 0.0875 ms（D69 预算 ≤ 0.08 ms，差约 10%）；Python ORT p50 0.40 vs Rust ORT 0.38 ms（Python 慢约 5%，输出逐位相同）；10 min 长跑 133 dB · reset 逐位相同 · FE-M 改写图逐位相同；Accelerate 小矩阵 11 µs vs 手写 21 µs，下一步接进去
- 2026-11-02 人补「或者说最高2%」→ D69 改：目标 1%、硬上限 2%（AI 理解，待确认）；已转 applebe
- 2026-11-02 applebe 写 standard.md §6.1 CPU 分摊（合计 1.00%）：降噪推理 0.50 · 认人 0.15 · VAD 0.06 · 场景 0.07 · STFT 与重采样 0.07 · 采集输出 0.05 · 修音 0.05 · 余量 0.05。风险：认人 1.2% 超总线（→ backlog 认人）；手写 GTCRN 0.63% 超 0.50%（019 内看）；FE-M 在 ORT 上约 3.7%，D33 备选要重估（收尾时出题）；四项未量（→ backlog）
- 2026-11-02 人定 D69「我的底线是1%」→ 整个降噪软件常开 ≤ 0.01 核（AI 理解口径，待人确认）；已转 applebe 改 BE-CPU
- 2026-11-02 applebe 调研收回（8 代理，各过怀疑派，本机未测）：只有 BNNS Graph 能在推理线程每帧直接调（单线程、预分配后不分配）；Core ML 每次预测新建 ObjC 对象，只能登记豁免，神经引擎 fp16 且 GRU 上不去；MLX 每帧加锁等后台线程，长处是本机微调。补了 GRU 手动展开变体。题 2-8（最低系统版本）已登记，等实测再问
- 2026-11-02 人问「苹果引擎怎么写的 · 参照标准是啥」→ D68：先写评判标准（applebe，standard.md）· 一切测速从 Rust 调 · Python 数字量出与 Rust 的差（rustbe 测 Python ORT vs Rust ORT）
- 2026-11-02 16:33 rustbe 基准结束（正式版 r4，16:32:36 开跑，在干扰时段之后，负载 2.25 → 2.04；r1–r3 留作参考）：GTCRN 手写 Rust p50 0.10 ms、最大 0.13–0.28 ms、每帧零分配，比 ORT（p50 0.37–0.38 ms、最大 0.43–1.0 ms）快约 3.7 倍，对拍 SNR 134 dB；tract 跑 GTCRN p50 0.37–0.56 ms、每帧分配 2247 次，不如 ORT；FE-M tract p50 0.58–0.68 ms vs ORT 0.36–0.38 ms，ORT 更快。名额收回转 47
- 2026-11-02 rustbe：tract 对拍过（GTCRN SNR 130 dB；FE-M 缺 SplitToSequence，等价改写为 Gather 后对原图 110.8 dB）；冒烟 GTCRN p50 tract 0.47 ms / ORT 0.51 ms（非正式）。另一位大总管 noise-canceling-47（tmux vdn-rust，Rust 骨架线 rs-shm / rs-entry）的 miri 占满 1 核 → 已结束；已向 47 提议两边「计时基准前互相打招呼」
- 2026-11-02 rustbe：手写 GTCRN 编译过、与 ORT 逐值对拍过（37 段，最差 SNR 119.2 dB，最大差 1.6e-4，无一帧差 > 1e-3）；下一步加 tract 后端（吃 ONNX · 支持 GRU · 面向流式），再申请基准名额。applebe 已报到（M5 · macOS 27.0.1 · 插电），起调研 workflow：Core ML · BNNS Graph · MLX · 实时线程可调性与 Rust 绑定，各配怀疑派
- 2026-11-02 judgeloop 收尾：exp011 三参数自动调无收益（SCOREQ −0.027 [−0.037, −0.016]，41 段 / 10 说话人；上界 +0.008）→ 第一版不做；PR #49 合并；转交两条已登记（3-6 · backlog 下一轮盲听）
- 2026-11-02 15:55 judgeloop：14:15 电脑睡眠中断回复，空闲约 1.5 h，已催续跑。停在 exp011「留出评分 + 吃字」：奖励打分 · 拼接 · 红线已提交；留出评分因识别读不了 32 位浮点 WAV 超时没落盘 → 改 judge（转 16 位 · 60 s 超时重试 2 次 · 先落盘）续跑，推算约 25 min
- 2026-11-02 ears 收尾：PR #48；E2 按人「可以」记 D66；E6 登记 4-11
- 2026-11-02 ears ①② 完成：① `notes/I-why-scorers-pick-dpdfnet.md`（实测 12 段：停顿处背景 DPDFNet-2 −7.8 dB vs GTCRN −5.1 dB；GTCRN 人声音色近原声 0.6 dB；高频、吃字无差）② 2 道众人锚点题 build 20261102132015-ccd3，揭盲表已提交；已查无答案后复制到 `~/Desktop/20261102-vdn-crowd-anchor-listen/`。揭晓时要告诉人：第 1 题两边响度差约 9.5 dB；两题众人都选 Y 是随机分配。答卷 `answers-20261102132015-ccd3.json` 交回给 ears 放 results/
- 2026-11-02 ears 自审更正：「作者偏离大众」参照线抽样结构与作者答卷不同；换同构参照线后作者 0.40 vs 线 0.389 → 结论由「有证据」改为「边缘，不能说作者偏离」，decision 已改。①② 在做：① 12 段纯信号量 GTCRN vs DPDFNet-2（不跑模型）② 盲听用 URGENT 2026 配对打分集（CC BY 4.0），按行取 2 对，几 MB
- 2026-11-02 人新提（转 ears，优先于自审）：「我们到底用什么模型 / 为啥评分器选择的都是dfn 的模型，但我选择的是另外的 / 评分器用的这个模型的逻辑是啥 / 帮我找两个大众都弄过的，让我听一下」→ ① 评分器逻辑短文 + GTCRN vs DPDFNet-2 实测差别 ② 2 道大众已评分的盲听题
- 2026-11-02 benchsynth 收尾：PR #47（Closes #41）；第②段续跑与 LibriSpeech 6.6 GB 下载转 backlog；7 题登记为 4-9
- 2026-11-02 ears 第二段：作者 vs SCOREQ 一致率 0.375（48 对 · 9 屏，区间 0.14–0.64，事先定的线 0.516）；~~Distill-MOS 0.562；两台评分机器不一致~~（ears 更正：三台评分机器彼此一致 0.76–0.81，作者对三台都偏低 0.375 / 0.562 / 0.478；分歧集中在 GTCRN vs DPDFNet-2，作者 16 次里 13 次选 GTCRN，三台都排 DPDFNet-2 第一；英文 6 段两者各吃 7 词。作者与大众不同 vs 机器在这类片段不准，分不开，要靠 E2），结论强度低。名额转 judgeloop
- 2026-11-02 judgeloop：r1 / r2 / r5 补完（各经怀疑派核对）；decision.md 与 exp011 设计已写（`docs/research/018_ondevice-judge/decision.md` · `docs/experiments/exp011-ondevice-loop/README.md`）；申请名额跑 exp011 第 1–3 步
- 2026-11-02 benchsynth 第①段完成（4 系统 × 348 段，838 s，0 错误）：阶梯 12 → 15 系统；交互一致性结论不变——SNR / 噪声 / 邻座干扰三轴两套生成器一致（多数 r 0.5–0.9，p ≤ 0.025），混响轴不一致（r −0.07–0.61，p ≥ 0.13）；新发现 DNSMOS SIG 在邻座干扰轴 r 0.04；已答 judgeloop 三问。名额转 ears
- 2026-11-02 ears 第 3 步完成：017 待定题 7 → 2（5 题被 D58–D62 答掉，已划并写依据）；剩 E2「每轮盲听多听 4–6 道大众已评题，多约 2–3 分钟」· E6「用人的 GitHub 账号向上游报两处数据错误」，默认都不做，收尾时并入待拍板清单；与 D62 冲突 5 处已改；等名额跑第二段
- engine workflow 已起（7 代理）：Core ML · BNNS Graph（macOS 15+ 实时音频 CPU 推理，号称零分配）· MLX · 纯 Rust（candle / burn / tract / Accelerate / Metal）· 公开实测 · 五类模型盘点 + 1 核实员；自写 GTCRN Rust 先行
- judgeloop workflow 已起（7 代理）：R1 评分模型小型化 · R2 评分当奖励 / 在线调参先例 · R3 声音→效果参数与助听器在线偏好学习 · R4 本机微调风险与护栏 · R5 本机盘点 + 2 怀疑派
- benchsynth 自审改结论：S7「自身稳」被打掉（错配格 τ 也 0.82，无区分力）→ 改交互一致性 S7b：SNR / 噪声 / SIR 轴 A/B 一致（r 0.5–0.9，p ≤ 0.04），混响轴不一致；S8「反向」降为「判不了」；红队 P0：exp008 素材与 exp006 训练段重叠，项目模型上阶梯前要换素材池；D51 合成好坏配对 P1–P7 已写；第二轮 RNNoise / WebRTC 出音频，torch 组在跑（负载 34）；engine 给 exp010、judgeloop 给 exp011
- 人定 D52（评分模型放到用户电脑上，调节模型取代场景查表，本机微调）· D53（推理必须 Core ML / MLX 或自写 Rust，推翻 009）→ 派 judgeloop（018 #44）· engine（019 #45）
- ears exp009 第一段：CHiME-7 32 人单人 vs 其余 7 人一致率中位 0.79；DNSMOS vs 单人 0.45；作者的 0.48 说明不了偏好特殊；SCOREQ 可真检验「作者 vs 大众」
- prefs 收尾验收过（自测 18/18、无绝对路径、无音频）→ PR #43；偏好 K1–K9 均低 / 很低置信；⚠️ `exp006-scene-sim/unblind.py` 全同检验题撞名 bug（选任一槽都判通过）——exp006 答卷到了以 `human-prefs/build_dataset.py` 为准或先修；待拍板 4-7 / 4-8 已并入
- 人定 D51：评委不按作者微调，只校准作者 vs 大众听感；再用更多合成数据告诉模型什么好 → 已转 ears / benchsynth
- ears 补检索已派（人追问更多人耳数据）：CPC1/2/3 听写 · DNS5 WAcc 逐条 · SSI/AEC 逐条 P.835/P.804 · speecheval/speechjudge → notes/H-extra-sources.md
- prefs 恢复：偏好数据集 29 屏 / 219 对、eval 任务 20 个已完成；发现 ① exp005 c06 / c08「激进」与「只有 GTCRN」逐字节相同，c08 开会那 1 票随机、不算数 ② exp006 盲听页 12 屏中 10 屏「路由开」=「路由关」同文件，路由效果只 3 屏可测
- 巡检：benchsynth、prefs 因电脑休眠中断（API Error），已发消息让其从现状页续做；ears 调研 7/8 完成，D 笔记：免调开源音频 LLM 在噪声维与人相关≈0，闭源成对 70–74% 但为云端
- ears workflow 已起（8 代理）：A 通话类人评集 · B 增强语音人评集（逐听众评分？）· C 无参考预测器与人相关性 · D 音频 LLM 评委 · E 单人耳朵增效 · F 个体 vs 大众偏差 + 现场最少人数 · G 其他 + 1 完整性批评；之后做 exp009
- 人（2026-10-31）：「如果只有我一个人的话，我觉得肯定不够……我们必须做到这样」→ 派 ears（017）
- 人定（2026-10-31）：Hush arm64 预编译 dylib 允许（只作对照）· torch 现在就装（人选，非 AI 建议的「等第一轮」）
- benchsynth exp008 第一轮（12 系统 + oracle，348 段，6287 s，0 错）：生成器 A/B 排名一致 τ 中位 0.79–0.85（SI-SDR 0.55）→ 合成格自身稳；合成格预测 exp003 人耳配对一致率 0.25–0.38（48 对、3 系统，功效低），直接在 exp003 片段上 0.48–0.56 → 预测人耳尚无证据、可能反向
- benchsynth 系统阶梯：不装新环境可凑 16 个（5 个族）；名额给第一轮 12 个 + 理想掩码；批准 cargo 小 crate（nnnoiseless / WebRTC NS）与 OpenNC v1、FE VCTK ONNX 小下载；Hush 预编译 dylib、装 torch（约 0.5–1 GB，换架构独立的 FRCRN / MossFormer 等）待人定
- benchsynth exp008 数据侧（build 20261031-102406-0c67，348 段，A/B 两生成器）：可复现 60/60 · 可控 12/12 · 三轨残差 −149 dB 全过；统计对齐：exp006 合成杯碟不过（LTAS 距离 10.4 dB，线 5.3）· 合成音乐不过（9.0，线 2.5）· babble 临界；稳态噪声无真实参照只到 D0；两坑进标准（float wav 头带时间戳→用样本哈希；电平须在设备链后定，否则 SNR 偏 6.3 dB）
- 人定（2026-10-31，原话「可以多弄几个来对比」）：exp008 系统阶梯扩到 ≥ 12 个，独立模型为主；非因果 / 大模型可当对照，不进产品；已转 benchsynth
- benchsynth：MVP 实验编号给 exp008（`docs/experiments/exp008-bench-validity/`）；约束：vdn-eval 任务集编译进去、只有 3 个系统 → 排名效度要 ≥ 8 个系统（8 个时 τ ≥ 0.57 才显著）；MVP 先 Python 数据侧，`--suite` + 系统阶梯写成给实现线的建议
- benchsynth：四条诊断核实成立；新发现：所有带人耳答卷的片段都是合成（exp003 c01–c03 是 URGENT 2026 现成加法混合，其余与 exp005 全是本地合成），exp007 QA「真实片段」也是这 11 段 → 真实锚为零，效度只能先到「合成 vs 人耳」；workflow wf_4b3f7b2d-afd 已起：6 路调研（URGENT · DNS/P.835 · VB-DEMAND 批评 · 效度方法 · 倒推先例 · 因素自然分布）× 每路 1 怀疑派 = 12 代理
- align 全量：fmt / clippy 过；test debug 73 · release 71 全过；真权重冒烟（GTCRN · FE-M）debug / release 过；tidy / check-deps 过
- align 延迟账改逐项硬加：按 009 目标 IO 256 帧 = 47.00 ms（超 D32 2 ms，测试里钉成已知缺口）；提议 IO 96 帧（两模型 hop 公约数）= 43.67 ms → 待人拍板（代价：回调约 2 ms 一次的 CPU 开销、掉帧风险、蓝牙能否支持）
- 010 ondevice：最值得照搬 ReSound Learning VC（每场景一个偏移量 + 滑动平均 + 停手才算同意 + 不一致降学习率）；接口需求 13 条已给 align；要另开实验：powermetrics 耗电、一周空闲窗口计数；⚠️ 一路检索子代理违规跑了一行只读 python3 -c（已自报）
- ✅ evalimpl 真模型确认：GTCRN 上纯噪声停顿门控误开 noise_open 0.99；根因 `src/dsp/src/envelope.rs:91-97` + `src/engine/src/engine.rs:355`；已写成回归任务 R17（xfail，留给修复线）；任务 schema `docs/experiments/exp007-eval-suite/task-schema.md`
- evalimpl 首轮：CI 层 16 条，ref 回归集 pass@3 13/16、pass^3 12/16，能力集 0/10（设计如此）；5 探针全拿低分（P02 / P05 证明 V-NR 单独可被刷）；定标 0 条 ≥ L1（exp003 48 对，最好 Δ(del+sub) 一致率 0.64 [0.41, 0.84]，DNSMOS OVRL 0.48≈抛硬币）；整套 CI 层 35 s；⚠️ 发现：纯噪声停顿里 engine 门控约 50% 时间开着、混回 −6 dB 原声 → 停顿只压约 5 dB（推断是 exp005「都没降噪」的引擎层原因），真模型验证中
- scene 重跑（AGC τ 4 s）修音链开 vs 关（GTCRN，244 段胜负）：音量起伏 143:97 · 音乐噪声 186:57 · 背景忽有忽无 180:62 · 发闷 163:34；代价 DNSMOS BAK 40:204、OVRL 2.42 → 2.32、ESTOI 0.732 → 0.726；路由开 DNSMOS 61:18 但删除率 2.4% → 4.5%、FE-M 音乐段背景忽有忽无 +10 dB；盲听页生成中
- scene 第一轮打分（244 段，ASR 92 段）：修音链对数峰度比 1.74 → 1.02（188:55）、背景忽有忽无 9.2 → 6.8 dB（198:43）、发闷（142:56）；AGC 让电平更起伏（96:146）→ 时间常数 0.6 s 改 4 s（120 段对比 4 s 最平），整批重跑中；路由开 DNSMOS 更高（61:18）但删除率 2.4% → 4.5%，交人耳
- scene 4 版推理完（244 段，修音链约 0.05% 个核）；自审修 P0 1 + P1 3（底噪跟踪开头静音卡死 · 平滑方向反 · 时域 AGC 不工作 · 手动「会议」不生效），+5 回归测试共 15 过
- p2 build 7448 作废：人耳「所有的都没降噪」（与数据一致：GTCRN 只压邻座 0.6–5.6 dB、混回 −6 dB 原声、下限 −12 dB），反馈已提交 results/human-feedback.md；下一版四候选
- p2 段②：fbank 对拍最大差 1.4e-4、声纹余弦 1.000000；8 段锁定 8/8 都锁对主人（含「开头别人先说 25 s / 45 s」），锁定后主人独白 0 次判成别人；⚠️ 个性化只比不个性化多压邻座约 0.5–3 dB——门控要等 1.5 s 声纹窗确认，邻座每段 3–8 s，大半时间门开着
- p2 段①：vdn-speaker 13 测试过（含实时不分配断言，hop 256 / 160），clippy / fmt 过；段②进行中
- align：vdn-types（零依赖，模型接口 + 感知 / 策略 / 场景参数表 / 记忆 / 决策日志 / 延迟账，40 单测）；008 P1 五项完成；xtask 放 src/xtask/（避开铁律 10）；已要求删 `.cargo/config.toml`（新顶层目录，铁律 10），改写全命令
- multitalker ②（待自审核实）：MacBook 内置三麦对第三方只给 1 路、已被系统波束成形（本机 system_profiler 1ch/48k），AirPods 单声道 → 内置麦上自己做波束成形走不通，降为「外接多麦」实验线；语音突显只对开了 VPIO 的 app 生效、需用户手动选——可考虑用 VPIO 采集让系统先突显，VPIO 附加延迟与能否关其 AGC / 降噪待实验
- scene 滞回已修：已生效准确率（held-out，不含开机头 1 s）噪音大类 87% · 用户层 93%（5 s 后 95%）；换场景中位 3.7 s 生效、错切 0.88 次 / 分钟；参数 EMA 0.5 s / 确认 1 s / 置信 0.4 / 停留 3 s（已发 align）；算力约 0.07% 个核；「多人会议」只留手动占位，自动识别列待人拍板
- scene：exp006 模拟 856 段 / 2.6 h；特征 + 判断合计约 0.06% 个核；分类器（held-out 逐窗）细场景 73% · 用户层 85% · 蓝牙检出 93% 误报 1.3%；滞回「第一判即生效」有 bug（已生效准确率 41%）修复中。名额规则补充：单 crate 增量构建 + 秒级特征提取不占名额，推理 / 全量 / 超 1 分钟满载仍要申请
- 第四批 20261031-b4（人原话见 `_tmp/20261031-b4/common.md`；硬约束：延迟账 ≤ 45 ms · 察觉不到占用 · 无云端 · 不同线不改同一批文件）：align · p2 · scene · ondevice 已起；人追问「一大堆人」后追加 multitalker（011）；人要求「所有效果都要有工业级的 rubrics 和 eval」后追加 evalstd（012）；人提出「别人做不好的才做 · RL 环境分场景训模型」（D42 · D43）后追加 rlenv（013）；人要真实数据集后追加 datasets（014）；人问「verifier 开始写了吗」并指定参考 Anthropic《Demystifying evals for AI agents》后追加 evalimpl；大总管分支 `chore/coord-20261031-b4`
- 009 rusteco：进程内依赖 ort · realfft · rubato · rtrb · triple_buffer · biquad · earshot · hound；最值得学 voice-strip（Rust HAL 虚拟麦 + 共享内存环）；⚠️ 默认参数延迟账约 56 ms 超 D32，重采样与 IO 缓冲必须调参
- 008 rustarch 实测（RTSan）：ort 跑 GTCRN 稳态每帧约 1550 次 malloc、约 380 对 mutex——推理不能进 IO 回调线程，架构分档（IO 硬实时 / 推理软实时 + 截止时间送原声）；p1 的「0 次分配」只计 Rust 侧，exp004 README 需补注
- ⚠️ rustarch 的 Workflow 子代理未申请名额就构建运行了小探针（scratchpad，未动仓库），已主动交代；规则：起 Workflow 时提示里写死「不许构建，实测先申请」
- P1（p1 报，待抽查）：Rust 对 Python 对拍 GTCRN 最大误差 4.5e-7（SNR ≥124 dB）、FastEnhancer-M 逐样本相同；单线程每帧 p99.9/max GTCRN 1.20/1.32 ms · FE-M 1.02/1.20 ms（与 Python 持平）；GTCRN 推理每帧 0 次分配，⚠️ FE-M 7 次（ort 内部）；故障注入 4 类全接住；~~快开慢关 96–99% 帧判有声~~ 自审 P0（底噪钉 −120 dB）已修，但有声帧仍 96.0% / 94.6%——主因是只靠能量 + 逐帧最小值分不开停顿与说话（模型残噪逐帧起伏 10–25 dB）；建议 P1 默认关掉快开慢关，等 P2 用真 VAD / 声纹；交替 3 轮计时 GTCRN max 0.89–0.93 ms · FE-M 0.78–0.93 ms；接缝 48 点无咔声
- 第三批 20261030-b3 **全部收尾**：PR #13（P1）#14（009）#15（008）已合并
- 第三批 20261030-b3（人原话见 `_tmp/20261030-b3/common.md`）：p1 · rustarch · rusteco 已起；大总管分支 `chore/coord-20261030-b3`（含技术方案 003）
- 第二批 20261030-b2 **全部收尾**：PR #5 #6 #7 #8 已合并；人定 D32 延迟 ≤ 45 ms · D33 主模型 GTCRN、FastEnhancer-M 备选 · D34 个性化前移
- 第二批追加 sota（006）、rubric（007）：人原话见 `_tmp/20261030-b2/sota.md`、`rubric.md`「人的原话」
- 第二批 20261030-b2（人原话「好可以的」）：pse must-know 已起草，workflow 批准约 12 代理（原计划 18，怀疑派合并）· abtest 素材定 12 段（casino 3 + LibriSpeech 合成 7 + 中文 2）；批准下载 whisper large-v3-turbo 1.6 GB 到 ~/vdn-data/models/（超 500 MB 线，大总管批：吃字率是 D01 关键旁证）
- 21:20 model：完成，大总管验收通过，窗口已关
- 21:05 pipeline：完成，大总管验收通过，窗口已关
- 21:12 evaltool：完成，大总管验收通过，窗口已关

## 发现

- 17:35 首派失败：spawn-worker.sh 不识别新版信任提示 → 超时退出，后两窗未开；已修脚本 BLOCK_RE（人定 2026-10-29）
- exp001 重跑（M5 · ORT 1.31.1 单线程 · 真实带噪语音 · 真 state · 0 NaN · 含 STFT/iSTFT）p99.9/max ms：baseline 0.37/0.56 · dpdfnet2 0.84/0.89 · dpdfnet4 1.32/1.40 · dpdfnet8 2.26/2.52 · 2_48k 1.18/1.25 · 8_48k 3.47/3.99 · GTCRN 0.48/0.62；全部在 5 ms 线内，2 ms 线只 dpdfnet8 两版超；与首跑基本一致 → docs/experiments/exp001-model-frame-time/（大总管抽查：state 按元数据初始化 bench.py:51-53、逐帧 isfinite 检查 :109、合计 max 为两项 max 相加属保守上限、旧版见 89bf562，验收通过）
- evaltool 收尾补充：回归门禁两基线（对原声直通用绝对判据；对上一版用相对容差 + 配对 bootstrap）；吃字率任一家 ASR 报警即拦；SCOREQ 在 URGENT 2024 逐句相关 .79、真实录音 .75（已核原表），走 ONNX 不需 PyTorch
- evaltool 取舍：每把尺子「主指标 + 交叉指标」、分 CI 层 / 本机层；吃字 = whisper.cpp 删除率（中文加 SenseVoice 第二意见）；自然整体 = SCOREQ + UTMOS（DNSMOS 降为冒烟）；NISQA 权重 NC-SA 只作用户自装；PESQ 不进仓；附加延迟现成工具全不覆盖，要自写
- model 发现：DPDFNet 流式输出落后输入 4 帧（mask 2 + DF 2），前瞻 40 ms；分三列：窗长+前瞻 60 ms · DNS 口径（窗长+步长+前瞻）70 ms · 端到端推算约 65–70+ ms（DNS 定义已对 PDF 原文核实）
- exp003 新 build 20261030110755-f8c3（DPDFNet-2 · GTCRN · FastEnhancer-M；FastEnhancer-M 最坏一帧 1.15 ms、端到端约 38–42 ms）：对照表已提交（8d71693、f2cff86），已复制到人桌面 `~/Desktop/20261030-vdn-ab-listen/`
- exp003 build f8c3 客观分（人听完前**不转述**）：OVRL FastEnhancer-M 2.81 ≈ DPDFNet-2 2.75 > GTCRN 2.45；ESTOI DPDFNet-2 0.63 > 两者 0.59；删除率 GTCRN 4.3 · DPDFNet-2 4.6 · FastEnhancer 6.1（原声 3.2），去掉 c01–c03 后 0.9 / 3.1 / 4.8；替换率 FastEnhancer 30.1 最高；Qwen 84/84 两次一致
- 006 sota 待转 pse 线（已收尾，留给 005 后续）：arXiv 2610.35272 开源因果主讲人隔离器（Krisp 基准 WER 5.74% 对 Krisp 5.63%，权重许可 other）；FVAD / EEND-SAA 无注册前景说话人检测，可补门控「开头别人先说」
- 007 rubric：`specs/rubric/denoise.md`（406 行）+ 筛选表；现只能比「轻快」（GTCRN 22/25 > DPDFNet-2 16/25），清楚、自然等人耳；⚠️ dryrun 里对具体模型的判断**人听完 AB 前不转述**
- exp003 两模型轮（abtest 报，待抽查）：DNSMOS OVRL DPDFNet-2 2.75 vs GTCRN 2.44（9/12 段赢）· ESTOI 0.63/0.59 · SI-SDR 6.6/3.7 dB；⚠️ 删除率 4.6% / 4.3% 都高于原声 3.2%（踩「从不吃字」），已要求逐段列被删词；Qwen3-ASR 两次转写 48/48 逐字一致；页面 ../noise_canceling-exp003/_tmp/20261030-exp003/index.html
- 005 pse：VoxLock（Apache-2.0、因果、ONNX、自动注册）是唯一现成个性化开源模型，Xeon 单线程算力是预算 3–5 倍 → 并进 exp003 实测 + 进 AB 页（大总管定，issue #2 已注）
- pipeline 结论：开发期 cpal 双流写 BlackHole + rubato 按水位补漂移；最终自写极简 C 回环插件，推理永不进插件；不用共享内存 / DriverKit
- pipeline 转交：结论进 vault/truth/（等人拍板后由大总管做）；推理预算 → 已转 model；旁路交叉淡化时长怎么评 → 已转 evaltool
- pipeline 核实：第三方 HAL 插件由独立 helper 进程加载、不在 coreaudiod 里（开机日志实测）→ 设计稿「崩了整机没声音」前提过时；开源先例断供都送静音，违背「从不吃字」；异步推理线程应自建 work interval；本机缺 meson
- exp002（M5 CPU）：DNSMOS 每段约 10 s 音频打分约 0.05 s，PESQ/STOI 各约 0.05 s——评测几乎零成本；坑：DNS 仓 dnsmos_local.py:53 与 librosa≥0.10 不兼容，非 16 kHz 输入报错，需先自行重采样（smoke.py:80-86）→ docs/experiments/exp002-eval-smoke/（大总管已抽查：clean 自检 PESQ 4.64 / STOI 1.000，验收通过）
- 交代里大总管名 5-app-89 失效，现为 noise-canceling-69，批次文件已改

## 待定清单

**已移到唯一清单 `state/pending-decisions.md`**（2026-10-31，人要求「要问我的问题单独放在一个地方」）。本账本不再列题。

- [ ] 待开实验（011）：① VPIO / 语音突显 + 4 声道 + 蓝牙麦延迟（随 dev-4，P0）② babble 人数×SNR 曲线（dev-3）③ 近 / 远特征 AUC ④ 会议室识别回归 ⑤ 商业对照；提醒 dev-4：别的进程在内置麦开 VPIO 时 HAL 读到的电平骤降
- [ ] 规约跟进（下一批接入线）：`specs/rust/architecture.md` §2.1 / §4 补 vdn-speaker（B 档）、vdn-speaker-lab、vdn-scene、vdn-polish；speaker-lab tier；CI 补 `--features onnx`；ORT 一个进程只许一个 Environment
- [ ] 008 给 P1 后续改动第 5 项（exp004 README 补注「0 次分配只计 Rust 侧」）是否已由 align 完成——下一批核对

> 2026-11-13 14:xx 删除（人批，选择题两题都选「删」）：`~/vdn-data` 已解压的压缩包（demand 10 个 zip · rir 3 个 · librispeech tar，删前逐个核对成员都在解压目录）· `_tmp` 三个 venv（exp008-torch · exp010 · exp010-ct）· `_tmp/030` 混回旧产物（dfn3 · gtcrn · gtcrn-h · gtcrn-raw · check-* · smoke-* · mixretest 两轮 · sweep-*），保留 natural 在用的 `gtcrn-h-raw` 与 `scenes-safety-off*.json`。系统盘剩余 69 → 103 GB。清单 `docs/research/032_real-meeting-eval/storage-inventory.md`（realmtg 分支）。
> 2026-11-13 删除（人批）：`~/vdn-data/real-meeting/alimeeting/Eval_Ali.tar.gz` 3.4 GB（realmtg 核过 66 文件与解压目录一致）。人批 `uv cache prune`（只清没在用的），首次被 realmtg 运行中的 uv 锁住 300 s 超时，未用 --force，待 realmtg 归还名额后再跑。
> 2026-11-13 约 14:0x 意外：人另开的全机深度清理会话 noise-canceling-19 删了全盘 target/ 与 .venv（含本项目主目录 + realmtg + natural 三处 target），realmtg 的 ICSI 打分在 50/72 段崩（50 段结果已落盘）。已请 19 不再碰 noise_canceling* 与 ~/vdn-data。名额给 realmtg 重编 + 正式 199 段（约 2 h）；natural 排队。uv prune 暂不做（19 在做 uv cache clean）。
> 2026-11-13 14:3x 续：全机清理会话 14:13 还删了主目录 `_tmp/` 的 exp016 · exp013 · 20261031-exp006 · exp008 · 20261031-exp005 · exp009-venv（清点里多数标「留」）；全是合成数据，可按仓内脚本 + 种子 + hashes 重建。另：我删 `_tmp/030/gtcrn-h` 与 mixretest 时漏查 natural 拿它们作对照组。exp013 已派子代理按原种子重建并逐文件核哈希；对照组由 natural 重生成；exp005（QA 层）· exp006 · exp008 · exp016 等用到时再重建（记 backlog）。
> 2026-11-13 uv cache prune 搁置：锁被常驻 uvx mcp-server-linkedin 占着，不用 --force；人退出 LinkedIn 工具后再清。清理会话 noise-canceling-19 确认删除清单并承诺不再碰 noise_canceling* 与 ~/vdn-data。
> 2026-11-13 exp013 重建完成：原 build id 20261103230756-17f5，子代理 53 s 生成，大总管复核 shasum 4540/4540 一致，测试 68 过；名额并行特许给 natural（-j ≤ 4）。
- 2026-11-13 删 _tmp/exp023/20261113-full1（4.2 GB，被 full2 替代，人同意）
- 2026-11-13 D95：后处理人耳分不出，exp023 盲听作废；已通知 natural 改用 exp024 成绩单选
- 2026-11-13 natural 申请名额：批冒烟 3 段并行（限 15 min，不拿 heavy.lock）；正式跑先用 80 段子集，等 realmtg 交回名额；结果相当就选不混回，不补全量；去舒适噪声两组不做。natural 自报未申请就跑了一次 cargo test（约 1 min），已记
- 2026-11-13 派子代理核实网络会议 / 通话真实录音数据集（DNS 盲测真实录音、AEC 近端单讲），只读网页，结果在 scratchpad
- 2026-11-13 21:40 natural 冒烟完、名额已还：每条约 24 s（realmtg 约 62 s/条，差异待查）；80 段正式跑报约 1.5 h，排在 realmtg 之后；评分指纹一致 c91c80bdb0ae4696
- 2026-11-16 名额 → trainer：cargo build --release parity_rs（限 15 min）；trainer 报模型变体 A/B 12 测试过，参数量 23,669 可训练 + 24,576 ERB 固定 = 48,245
- 2026-11-16 trainer 归还名额（25 s，Cargo.lock 已提交）；名额 → evalcmp：scorecard 单测 + vdn-schema 导出 + 3 个集成测试（冷编译，限 25 min）
- 2026-11-16 evalcmp 归还名额：scorecard 单测 65/65 · schema +7 −0 与正本一致 · 集成测试 12+11+5+2 全过（自报，验收时大总管复跑）
- 2026-11-16 trainer 自报未申请名额跑单批过拟合探测（CPU 136 s，墙钟 56 min：电池 + 降频），已结束无残留；已记、以后先申请 + caffeinate + 记电源。读数：起点单批损失 10.19→0.95（150 步）、随机初始化 38.07→4.84；原检查点复现官方 enh.wav 最大差 1 个量化步；45 条测试过（自报）
- 2026-11-24 大总管（重启后接 e1）名额 → trainer：CPU 小样冒烟 smoke.py --config configs/smoke-cpu.yaml（限 10 min，电池、负载约 21，只冒烟不计时）
- 2026-11-24 trainer 归还名额：CPU 小样冒烟三项过（初始损失 4.388 · 单批过拟合 7.64→−0.08 · 原检查点最大差 3.05e-5 = 1 量化步），墙钟 209 s，无残留；报告 trainer/smoke_reports/（自报，验收时复跑）
- 2026-11-24 名额 → trainer：新判据 CPU 小样冒烟重跑（限 10 min）；trainer codex 10 轮停（第 9 轮起故障注入类看门狗记 P2，人未参与，大总管定），待开 issue T-1 T-2
- 2026-11-24 trainer 还名额：新判据冒烟第一次没过（过拟合下限 −10.04 被官方分母下限顶住、到不了），改下限为「目标 + 40 dB 白噪声」（−0.41）后三件过；事后改判据，大总管认可，要求补报过拟合后 SI-SNR
- 2026-11-24 大总管验收 trainer：本机重跑 pytest 119 passed / 4 skipped（73 s，电池 24%）；文件最大 579 行；issue #159 #160 已开；PR 排在 evalcmp → cloudrun → traindata 之后
- 2026-11-25 00:30Z cloudrun：第一批 21 包 18.15 GB + E3 3 文件下完、哈希核对过；CPU 机已删，0 台在跑；累计估 1.15 美元；网络盘 150 GB 保留（约 10.5 美元/月，现用 33 GB）。E3 第 8 次重试才成，traindata 待加长间隔。下一步 cloudrun 自审 + codex → PR #158。
- 2026-11-25 00:45Z 大总管放行：cloudrun 开一台 CPU 机（上限 3 h / 1 美元），先跑 traindata split→shard→register，再在 evalcmp 合并后跑 exp013 新 build（480 段带早期回声参考，音频留网络盘）；builds 小文件由 evalcmp 提交。上云出 build 是大总管定的技术题。
- 2026-11-25 00:50Z evalcmp：功能与测试已完成（scorecard 68/68、exp013 Python 77 条、clippy、tidy 全过，自报），codex 第 1 轮在跑；发 1 个重活名额给 codex 修复后的 e2e 重跑；此前自行跑的 e2e 补记现状页、已认可。
- 2026-11-25 00:25Z cloudrun 开 CPU 机 8mnlr0uvh5pu4a（cpu3c 4 核，US-MO-1，挂 bzpbedqdsi，0.12 美元/h），最晚 03:30Z 自删；用 traindata 7b6ac21e 跑 inventory→split→shard→register→pack。
- 2026-11-25 大总管定（人未参与）：exp013 新 build 本机出一份（约 30 s CPU，音频进 _tmp，小文件进 builds/<id>/）+ 云上从同一批素材再出一份交叉核对；cloudrun 从本机上传 exp013 素材约 3.3 GB（含 exp013 版 DEMAND，云上 16 kHz 版不同）到网络盘新目录；验收 material_index.sha256 一致、files=1994。改掉此前「只在云上出」。
- 2026-11-25 00:42Z 人在 history 会话批准腾硬盘：主仓 _tmp/ 下 11-24 前的目录全删（含 exp002-venv、exp013 旧 build 音频、scorecard/）。处理：evalcmp 重建 venv 并冻结依赖、按仓内清单重出旧 build 核哈希；旧成绩单以仓内为准，clips.jsonl 等云上基线重生成。
- 2026-11-25 01:15Z 本机 tmux 会话 vdn 消失，四个手下全部退出（原因未查明）。云上：traindata split 失败 rc=1（OpenSLR28 RWCP_type4_rir_000l.wav 不在任何 rir_list，查不到房间号，切分拒绝按文件切），素材上传中断在 650 MB；机器空转，大总管取回 split.log/prod.log 到 _tmp/20261125-cloudrun-shard/ 后删机 8mnlr0uvh5pu4a，账上 0 台，累计估 1.26 美元。大总管亲手重跑 evalcmp：exp013 Python 81/81、tidy、vdn-eval 单元 171 与集成测试全过（spectrum_golden 之后被中断未跑完）。
- 2026-11-25 01:29Z cloudrun 开 CPU 机 e49nwj60n7hfpg（最晚 04:34Z 删），跑 traindata 8b4194d3 六步 + 续传 exp013 素材。cloudrun codex 第 5 轮 3 条新类 P1 已修（跨进程账本锁、put/get 先核后替换、job-start 同步准备），83 条测试过；issue #163–#167；大总管要求再跑到连续两轮无 P0/P1。PR #162（evalcmp）已开，CI 等待中。
- 2026-11-25 02:00Z 合并 PR #168（url 2.5.9，RUSTSEC-2026-0592，大总管自改锁文件）与 PR #162（evalcmp，Closes #155，合并提交 83280aa0）；远端分支已删。evalcmp 留窗待核云上 build。
- 2026-11-25 02:40Z traindata 六步全过（split 1346 s · shard 1212 s 15 GB · register build 49349c158a4defdc），产物已核、进仓。云上 exp013 build aa9f 与本机 6323 非逐位一致（1086 处，最大差约 4e-13，跨平台浮点）。大总管定（人未参与）：云上评测也用 6323 音频（上传约 3.7 GB 到 /workspace/exp013-builds/），aa9f 只作跨平台可复现证据；取 5 段 wav 比，逐样本最大差 ≤1e-9 判等价。
- 2026-11-25 PR #170 合并（exp013 云上 build aa9f 跨平台交叉核对，容差第 3 版：4 × 峰值最小单位 + 差信号 ≥120 dB；10 段 97 轨过，大总管亲手重跑）。evalcmp 线可关。
- 2026-11-25 03:40Z evalcmp 基线 README 写完（分支 exp/025-baseline-scorecard，codex 第 1 轮中）。大总管定（人未参与）：基线成绩单全在本机跑（A；识别服务 42model 只有 macOS 版，云上省不了本机时间），首个约 2.6 h、之后每个约 1.6 h，6 个合计约 10.6 h，费用 0。发现：按「GTCRN 自己多弱」排的前 4 场景与临时选的四个只重合 1 个（车站）；gap.py 出弱项表 + 提升空间表两种，等不回混基线出数后交人定加训场景。
- 2026-11-25 03:55Z PR #172 合并（cloudrun，Closes #158）。put-secret 等新提交另开 PR。
- 2026-11-25 04:10Z PR #177 合并（traindata，Closes #156）。残余 P2 #174–#176。
- 2026-11-25 04:17Z cloudrun 开 CPU 机 ciuc12isly6w84（自删 1 h，最晚 05:22Z），用 main 86698924 跑 setup → register → pack。
- 2026-11-25 04:30Z cloudrun register+pack 完成：新 build f032a75eedad80d6，交集检查过；机器 ciuc12isly6w84 已删，0 台；累计 1.55 美元。
- 2026-11-25 04:45Z PR #178 合并（put-secret）。PR #179（trainer）、#181（基线+识别上云）、#182（register2）CI 中。
- 2026-11-25 PR #179 合并（trainer，Closes #157）。开训前四条线全部合进 main。
- 2026-11-25 04:45Z PR #181（基线 + 识别上云）与 #182（register2，现行 build f032a75e）合并。
- 2026-11-25 04:42Z cloudrun 开 CPU 机 5f62454se0t6hs（自删 6 h，最晚 10:47Z），main 73631939 跑基线成绩单：put-secret 假值冒烟 → 真密钥 → 5 段冒烟 → 等放行跑 480 段。识别账本复制上云接着记，本机冻结直至取回合并。
- 2026-11-25 trainer H100 冒烟方案 codex 3 轮过（第 2、3 轮无 P0/P1）；大总管重跑 148 过 2 跳过；推送 exp/025-h100-smoke 901a9f46。乙段 H100 上限 1.5 h / 7 美元。
- 2026-11-25 05:10Z cloudrun 新 CPU 镜像 rust:1.99.1-trixie 冒烟：python3 3.13.5 / glibc 2.41 在、机器内自删过（试验机 f77s3d58riy6p1，0.005 美元）。主机器 e5sjgks999cqbf 开机（最晚 11:15Z 删），接 put-secret 假值 → 编译 vdn-eval → 甲段 d2477f63。
- 2026-11-25 PR #183 合并（识别账本加闸：缺失拒发 · 显式新建 · 封存 · VDN_ASR_LEDGER）。
- 2026-11-25 05:40Z 新镜像四样冒烟过；甲段（d2477f63）122 过 1 跳过；CPU 参照 hybrid 3.9733（build f032a75e，2000 段）。放行：基线 5 段冒烟（同机）+ H100 乙段（H≥1 h、最坏 ≤7 美元即开）。累计约 1.64 美元。
- 2026-11-25 05:24Z H100 乙段开机 l4ch9o7sv13psu（4.39 美元/h，H=1.5 h，最坏 6.95 美元，最晚 06:59Z 删）；CPU 机基线 5 段冒烟同时跑。累计约 1.65 美元。
- 2026-11-25 05:35Z 基线 5 段冒烟过（每段约 30 s，识别 30 次 0.0027 美元，评分模型有值；识别不一致率 4/15=27%，交 evalcmp 查）。放行 480 段整套，推算约 4 h，约 09:35Z 完。PR #184（setup_uv）已开。
- 2026-11-25 05:30Z H100 乙段第 1 步冒烟不过：overfit 只降到离下限 14.4%（3.341→2.381，SI-SNR 8.66→12.18 dB），init/reproduce/known_dev 过。已删机 l4ch9o7sv13psu（约 0.41 美元），累计 2.07 美元。交 trainer 先本机复现、查判据，改判据须大总管定。
- 2026-11-25 人定「改 hook」：auto-commit.sh 只在本项目仓库（含 worktree）里提交，按公共 git 目录判；自测 4 条归属用例过，gtcrn 克隆实测不提交。
- 2026-11-25 18:45Z 本机睡眠约 10 h。基线 480 段 08:13Z 跑完（云上约 2.7 h），机器到时自删；识别中途停发：账本 calls 1170、review「费用未知，curl 超时」。OpenRouter 实测本密钥今日用量 1.104 美元 vs 账本约 0.105 美元（约 10 倍），交 evalcmp 只读查。cloudrun 开 1 h 机取回结果。RunPod 累计约 3.07–3.22 美元。
- 2026-11-25 19:30Z 过拟合排查（CPU 真数据）：走完下限距离 14.4%，与 H100 一致 → H100 无问题；40 dB 下限 −3.31 低于 oracle −1.31，不可达；走完 oracle 距离 20.6%。机器 zq05j06wg96pxa 已删，约 0.08 美元，累计约 3.31 美元。判据改法待 trainer 报。
- 2026-11-25 20:00Z 判据第 3 版 CPU 核对不过：4 段走完 oracle 19.6%（门槛 50%），SI-SNR +2.52 dB（门槛 6）；其余三项过。机器已删，约 0.03 美元，累计约 3.34 美元。交 trainer 查学习曲线（到 2000 步）、梯度、学习率，不降门槛。
- 2026-11-25 PR #188（same_text）· #189（逐次记账 · clear_review · 跨平台对拍）合并，main 048ad638。trainer --overfit-probe 21e2e5b7 交 cloudrun（1.5 h / 0.25 美元）；8 个 wav 留网络盘待人定。
- 2026-11-25 21:20Z overfit-probe 完成（4429 s，机器已删，约 0.15 美元，累计约 3.51 美元）：2000 步走完 oracle 30.7%（+3.87 dB），曲线变平；lr 3e-4/1e-3/3e-3 在 300 步为 20.2/22.2/23.2%；关旋钮帮助小（全关 23.0%）；数据侧正常（时延 0、无静音、无削波）；BN 22 层全评估模式（配置 bn: frozen，待 trainer 确认）。
- 2026-11-25 21:36Z PR #192 合并（成绩单分片 + asr-probe），main 4fb3d8cc。
- 2026-11-25 人定 D105：先出场景表、人定场景再正式训练。evalcmp 出第一版表（不含吃字）。
- 2026-11-25 人定 D106：先补训练噪声（缺池场景），再按差距表加训。派 traindata 调研 + 下载清单；loader 支持多个目标场景。
- 2026-11-25 21:51Z H100 乙段开机 ijfkm5fvi6xfp7（4.39 美元/h，H 1.5 h，最坏 6.95 美元，最晚 23:19Z 删）；第 0 步在 CPU 机 t7wneq6dc0l3lv 做完已删（0.006 美元）；第 1–3 步在跑。
- 2026-11-25 人定 D107：V-PUMPV 两个开关（traindata 干净输入旋钮 · trainer w_pump），默认关，消融。
- 2026-11-25 22:25Z H100 乙段第 1 步过（第 4 版判据，每步 0.126 s），第 2 步因镜像无 git 失败，机器已删（约 1.33 美元，累计约 4.86 美元）；放行重开从第 2 步起。人定 D108：只下 CochlScene 3 类，放不下扩到 200 GB。
- 2026-11-25 22:05Z H100 重开 htpp90buaagezi（4.39 美元/h，H 1.5 h，最晚 23:33Z 删）；新 rp.py preflight（装 git + 核提交号，失败删机，带测试）过；从第 2 步跑。
- 2026-11-25 PR #193 合并（#191 未知选项拒绝）。
- 2026-11-25 人批：云盘删可重建项（6 份旧仓库+venv 约 47 GB、两个上传分块 7 GB、.tgz），再扩到 200 GB，下 CochlScene。待办：共享 venv。
- 2026-11-25 22:40Z 云盘清理 128→80 GB（13/13 小文件核过才删）；rp.py volume-resize 扩到 200 GB（每月约 14 美元）；CochlScene 22:36Z 开下（gwgj2a97e1a92q）。H100 第 2、3 步过：speed-B 460 s、speed-A 282 s，dev-gate A/B 都过，续训逐值相同；第 4 步在跑。
- 2026-11-25 22:45Z H100 乙段全过：dev-gate B 3.9733→3.9050、A →3.9037；续训逐值相同；3 并发各 200 步约 234 s。机器 htpp90buaagezi 已删（约 2.62 美元），累计约 7.56 美元。正式训练按 D105/D106 待数据与场景。
- 2026-11-25 23:00Z trainer 自动保护核对：NaN 停（部分）、卡住超时（部分）有；猛涨停、最好检查点、太慢停没有 → 大总管批 feat/trainer-guards（阈值见 trainer 方案），正式训练前合入。
- 2026-11-25 PR #194 合并（trainer H100 冒烟 · 判据第 4 版 · 开训前检查）。
- 2026-11-25 钉温度探针：温度 0 钉不住（1/15 = 6.7% > 2%，45 次 0.0043 美元）→ 走 A（有分歧调第 3 次取多数）。CochlScene z01–z03 md5 对，盘 110 GB。
- 2026-11-26 00:10Z 新大总管接班（本窗口）：tmux 会话 vdn 已不在，四个手下全部退出；无未合并 PR。各 worktree 留有少量未提交改动：cloudrun（costs.tsv +2、交接页 +1）· evalcmp（fix/asr-probe-dedup，asr_vote.rs）· trainer（feat/trainer-guards，train.py）。云上仍有 1 台：CochlScene 下载机 gwgj2a97e1a92q（cpu3c，0.12 美元/h，机器内自删 04:11Z；本机看门狗进程已不在）。人定：之后写代码一律派 pi 手下，其他活照常。
- 2026-11-26 人更正：pi 由 Claude Code 手下派遣，大总管不直接派 pi；三层是 大总管 → Claude Code 手下 → pi（只写代码）。
- 2026-11-26 00:20Z 大总管 noise-canceling-c4 核对 CochlScene 下载机：3 类解压完 17629/17629（与 want.txt 一致，7z Everything is Ok），用 rp.py down 删机（约 0.25 美元）；云上 0 台，累计约 7.77 美元。
- 2026-11-26 00:25Z 重开 tmux vdn，四个手下用 claude --continue 接回：trainer-02 · evalcmp-77 · traindata-56 · cloudrun-39；交代 `_tmp/20261116-b19/msg-resume-*-20261126.txt` + 新规则 `common-pi-rule-20261126.md`（代码由手下派 pi 写，pi 单任务 ≤2 美元）。后台子代理在改 coordinator-dispatch 技能（pi 由手下派、只写代码）。
- 2026-11-26 00:40Z 回三个手下：evalcmp 场景表第一版收下不重出、第二版含吃字；trainer 开 guards issue；traindata 压缩包不删待人定（10-2）、这次不筛人声、目标场景待人定（10-1）。
- 2026-11-26 00:50Z cloudrun-39 报到：0 台、7.77 美元核对一致，未提交改动已提交 4a3ed933。大总管删 gwgj2a97e1a92q 前没取回容器盘日志 → z04 · z05 · .zip 的 md5 只剩间接证据；下次登记开机时顺手对 6 卷补跑 md5sum。规矩：删机前一律先取回日志。
- 2026-11-26 01:00Z CochlScene 6 个分卷已被下载脚本解压核对后自动删除（删机前目录实测无分卷）→ 待拍板 10-2 作废；不重下，接受间接证据（7z 逐文件 CRC + 17629/17629）。cloudrun 改脚本：删分卷前写 md5 小文件、删机前取回（派 pi）。
- 2026-11-26 01:10Z trainer 还名额：全套测试 221 过 · 2 跳过 · 0 失败（86 s，自报）；名额 → evalcmp（clippy · 全量测试 · fmt，约 4 min）。
- 2026-11-26 01:15Z traindata 派第一个 pi：traindata-cochlscene（预算 2 美元，任务 _tmp/20261116-b19/pi-traindata-cochlscene.md）：sources · split_rules（按录音人 9:1）· split.py · recipe 加 4 个噪声池 + 测试。采样率由 shard.py 按 wav 头重采样、loader 断言 16 kHz（读源码）。
- 2026-11-26 01:20Z cloudrun 派 pi：cloudrun-venvcache（预算 2 美元，分支 w/cloudrun-venvcache 从 exp/025-cloud-baseline 开，只本地提交；任务 _tmp/20261116-b19/pi-cloudrun-venvcache.md）：共享 venv（按 Python+uv+requirements sha256 定名，tar 放网络盘、解到容器盘，分步计时）+ 下载记账（每包 md5 追加 checksums.txt，脚本不删东西）；测试用假命令、不连网。
- 2026-11-26 01:30Z trainer（issue #195，头 1ba6c800）codex 第 3 轮 1 条 P1 属实：开发份 NaN 碰上花费/时长上限时被「中断」吞掉 → 退出码 0、不存现场、允许续训（EMA 同）。派 pi trainer-guards-devnan（预算 2 美元），只改 train.py + test_guards.py，先红后绿。
- 2026-11-26 大总管定：单个测试文件 ≤1 分钟不算重活、不用名额；全套测试（分钟级）、构建、计时才要名额。
- 2026-11-26 01:45Z cloudrun 验收 pi cloudrun-venvcache 并合入自己分支 exp/025-cloud-baseline（0f91ec80；pi 花 0.0162 美元）：venv_cache.sh · fetch_archives.sh；打回 P1×1（GNU mktemp 严格模板，加假 mktemp 先红后绿）P2×3；cloudrun 重跑 132 条全过（自报），≤600 行。未上云实跑，等登记那次一起。
- 2026-11-26 技能 coordinator-dispatch 改完（技能仓 771ce0c，未推送）：三层分工正本在 references/pi-worker.md 第 1 节；pi 只写代码、不提交、不跑长任务；e2e-mock · git-policy · hook.test 全过（子代理自报）。大总管定（技术题）：pi 不提交暂靠文字约束，机器强制（git-policy 拦 pi 提交）记 backlog；pi-worker review/clean 保留不删；pi 只跑交代里点名的短测试。
- 2026-11-26 02:00Z trainer codex 第 4 轮 P1：标定阶段不查 NaN（同类第二次：NaN 被按上限停吞掉）→ 大总管要求改结构：读数统一过一个查有限值函数 · 停止原因优先级正本（nonfinite > 猛涨 > 卡住 > 上限）+ 表驱动测试 · 查绕过处。evalcmp 名额中：fmt · clippy · tidy 已过，测试仍在跑（约 6 min）。
- 2026-11-26 02:10Z 批 trainer 结构改动方案（require_finite 抛异常 · STOP_PRIORITY + pick_reason 表驱动测试 · 扫描防绕过）。发现 auto-commit hook 会提交 pi 未验收代码且署名错 → 待拍板 10-3；临时：手下记 pi 提交号范围，验收提交加 Code: pi。
- 2026-11-26 02:20Z evalcmp 还名额：fix/asr-probe-dedup fmt · clippy -D warnings · tidy · cargo test --workspace（99 组）全过（自报）；下一步 codex。名额空闲，trainer 两个 pi 交完后申请。
- 2026-11-26 02:30Z cloudrun 实测：pi 在自己分支 w/cloudrun-venvcache 提交（ec06f94e..0f91ec80，署名 Agent: pi），快进合入，hook 没碰；交接页 42078e34。→ 10-3 建议改为「pi 一律用自己的 worktree + 分支，不改 hook」；本机盘剩 48 GB，worktree 约 150 MB。
- 2026-11-26 人定 D109：pi 可以提交、提交要细（回滚点），在自己的 worktree + 分支；PR · issue 只归 Claude Code 手下；10-3 关闭，hook 不改。
- 2026-11-26 02:45Z evalcmp codex 第 1 轮 P0×0 P1×0 P2×2（第 3 次失败不记账 · 并发预取缺多路与失败测试），派 pi evalcmp-votecount（--repo + --base fix/asr-probe-dedup，自己分支 w/evalcmp-votecount，预算 2 美元）。
- 2026-11-26 00:25Z 名额 → evalcmp（替 pi evalcmp-votecount：qa::asr_vote 测试 + clippy，连用最长 20 min）。
- 2026-11-26 traindata 验收 pi traindata-cochlscene 前 7 项（195 过 · 1 跳过网络，0.05 美元，自报）；发现老漏洞：噪声池引用的来源不在分片里时 loader 静默跳过 → 打回 pi 第 8 项：改成报错停 + build.json 记实际场景。
- 2026-11-26 trainer 验收 pi calibnan（test_guards 31 绿，0.0064 美元）；pi 提交范围 devnan 1ba6c800..a1f00093 · calibnan b37a8560..7c793840；结构改动 pi trainer-guards-finite 已起（自己分支 w/trainer-guards-finite，2 美元）；防绕过扫描只收 train · guards · watchdog（大总管同意）。
- 2026-11-26 名额规则细化：针对性测试合计约 1 分钟以内且不计时，不用名额（不限文件数）；全套测试 · 构建 · 计时才要。
- 2026-11-26 00:26Z evalcmp 还名额（pi evalcmp-votecount 5 次小测试 + clippy 全过，自报）；名额空闲，trainer 结构改动交完后申请。
- 2026-11-26 00:27Z 名额 → evalcmp：验收 pi evalcmp-votecount（98bdfa6b..acc34d33 已快进合入 fix/asr-probe-dedup，0.0153 美元）：旧提交上看新测试红 + 全工作区 clippy · 全量测试，约 6 min。
- 2026-11-26 00:28Z traindata 登记代码验收（200 过 · 1 跳过，pi 0.072 美元；pi 代码 c5394187..8420eef2 被 hook 署名 Claude，验收提交 e3fdac7d 带 Code: pi）；命令 data/cloud-batch2.md 交 cloudrun，新分片 /workspace/shards/batch2 约 21 GB（推算）。大总管定：traindata 自己推送（D109）；cowork/open-office 共用拥挤室内录音致权重翻倍 → 按来源去重抽样（派 pi）；pi-worker 报语法错 = 子代理改技能中途，宣布改完前停新起 pi。
- 2026-11-26 00:29Z 批 cloudrun 第二批登记开机：cpu3c 4 核 · 自删 3 h · ≤0.36 美元；等 traindata 推送 feat/data-batch2 后开；新 clone /workspace/repo-<前 8 位> + venv_cache.sh 共享 venv（首次实跑，取分步计时）；跑完盘约 139/200 GB（推算）；删机前取回 /root/jobs 日志。
- 2026-11-26 00:30Z traindata 推送 feat/data-batch2（c0c9bda3，外发检查无命中）；拥挤室内权重偏差这批记进 recipe.json noise_pools._说明（进 build 身份）与交接页，按来源去重抽样下一批派 pi 修。
- 2026-11-26 00:31Z（cloudrun 报）开 CPU 机 g1wyqz5tsvqvia（cpu3c 4 核，0.12 美元/h，自删 3 h），代码 c0c9bda3 → /workspace/repo-c0c9bda3；共享 venv 只按 data/requirements.txt；开机前累计 7.78 美元。
- 2026-11-26 技能 coordinator-dispatch 第二次改完（技能仓 c3546a9，未推送）：pi 档 git 规则机器强制（提交须带 Agent: pi + Model:，push · gh pr/issue create 拦下）；大总管亲手复跑 bash -n · e2e-mock PASS · git-policy PASS。通知四个手下可以新起 pi，验收提交加 Code: pi 取消。合并方式仍为 merge commit（铁律 15），pi 标注能留在 main。
- 2026-11-26 00:36Z traindata 派 pi traindata-dedup（w/traindata-dedup，基于 feat/data-batch2，2 美元）：按成员完全相同的噪声池去重（只有 cowork = open-office），build.json 记 scene_groups。大总管定：云上 c0c9bda3 那次不打断；新提交若只影响 register/pack，只重跑这两步，分片不重打。
- 2026-11-26 00:36Z evalcmp 还名额：pi 提交 98bdfa6b 上新测试 5 条红（在 pi 的 worktree 里看），acc34d33 上 fmt · clippy · tidy · 全量 99 组全过（自报）；下一步 codex 第 2 轮。名额空闲。
- 2026-11-26 00:38Z trainer 验收 pi trainer-guards-finite（6be34442..370af014，4 个提交带 Agent: pi，0.043 美元，已快进合入）；train.py 515→494 行。名额 → trainer 全套测试约 2 min。
- 2026-11-26 00:39Z traindata 去重抽样验收（209 过 · 1 跳过，pi 0137540a..2eaf21b9 带 Agent: pi，0.0125 美元），推送 5d12fc2e；c0c9bda3 build 作废。cloudrun 在同目录 ff 到 5d12fc2e 只重跑 test → register → pack；预期红 1 条用 --deselect 排除并记日志；取回切分表回仓。
- 2026-11-26 人定 D110：场景按第二版差距表定；缺数据先补；开训前全部就绪再开正式训练。派 traindata 调研 3 个缺口场景的补法（不写码、不下载）。
- 2026-11-26 00:40Z trainer 还名额：全套 369 过 · 2 跳过 · 0 失败（78 s，自报；新增表驱动 121 条）；下一步 codex 第 5 轮。
- 2026-11-26 00:42Z trainer codex 第 5 轮：P1×1（写盘满 run.json 写不进）按 33 口径（故障注入类）改记 P2，大总管同意；补防线：开训练机前网络盘余量 ≥30 GB（交 cloudrun 进 preflight）。P2 #3 看门狗静默退出派 pi trainer-guards-watchdog。第 5 轮算停止线连续第 1 轮。
- 2026-11-26 人定 D111（缺口场景补数据四题，见决策记录）。
- 2026-11-26 cloudrun（c0c9bda3 那轮）：只读 · 共享 venv · test · split 过，shard 00:41Z 起在跑。实测：17629 wav 44.1 kHz 单声道 10 s；385 录音人，训练 347/16193 · 开发 38/1436；共享 venv 装包 52.8 s + 发布 109 s，对比上次写网络盘 889 s（单次实测）；派 pi cloudrun-diskcheck（≥30 GB 门槛进 preflight）。
- 2026-11-26 人选：BUT 2019 版 9.3 GB 下（Apache 2.0 署名）；飞机舱先不补（不给 Freesound 密钥）。
- 2026-11-26 01:08Z 大总管验收并合并：PR #197（识别取多数，独立副本 fmt · clippy · tidy · 全量测试退出 0，合并 a4ec1bf4）· PR #199（训练自动保护，独立副本 371 过 · 2 跳过，合并见下）。
- 2026-11-26 PR #199 合并 ab0f4f0a（#195 关）。开训前卡点①完成。放行 cloudrun 补跑识别（约 2003 次 · 约 0.18 美元）。
- 2026-11-26 大总管删已合并分支 feat/trainer-guards · fix/asr-probe-dedup（本地 + 远端）。trainer 开 #200 feat/trainer-formal-config（pi trainer-formal-config 写 formal.yaml · formal-runs.yaml · formal.py check/commands）。识别补跑交 cloudrun（_tmp/20261126-asr-rerun/brief-cloudrun.md，闸 0.25 美元 / 2800 次）。
- 2026-11-26 trainer 更正：pi worktree clean 第一次被 git 钩子拦（命令里夹删分支），补跑后 trainer-guards-finite · -watchdog 的 worktree 与 w/ 分支已删。
- 2026-11-26 traindata 清掉 pi traindata-dedup · traindata-screen 的 worktree 与 w/ 分支（先核已在 feat/data-batch2、无未提交改动）。
- 2026-11-26 trainer git 钩子仍登记 feat/trainer-guards（分支在启动时由 CD_WORKER_BRANCH 定）→ 大总管关窗、用 --continue 重开 trainer，钩子改登记 feat/trainer-formal-config（#200）。pi trainer-formal-config 交 2 个提交 4c687043 · 2228fc9b（0.011 美元）。
- 2026-11-26 trainer 重开后会话名 noise-canceling-trainer-f0（旧 trainer-02）。教训：我关窗时它正在跑测试，被杀（退出码 137）——关手下窗口前先问它有没有进程在跑。
- 2026-11-26 01:16Z 验收 PR #201（独立副本全套 389 过 · 2 跳过；formal.py check 3 条退出 1 符合预期），等 CI。w_pump 第一批正式训练不开（D107 默认关）。traindata：第一轮 build 889a389708c8a1e9（分片 1819 s，valid）作废；第二轮多红 1 条（test_products 手写 SHARDED 漏 cochlscene，读到 builds/ 里未进仓的旧 build）→ 挪出旧 build 重跑，在 fac82500 上 register + pack；回来后派 pi 把 SHARDED 改成从 recipe 导出（唯一真相源）。
- 2026-11-26 01:19Z 第二批登记完成（实测，cloudrun 报）：build 6173bf9ca6a6e143（fac82500，valid，11 场景，scene_groups cowork↔open-office，missing []），盘 121/200 GB。开训前卡点②完成。BUT 在 g1wyqz5tsvqvia 下载中（md5 未知模式，按字节数判 + 记 md5/sha256）。批识别补跑另开 cpu3c 4 核 · 自删 6 h · 机器 ≤0.72 美元 · 识别 ≤0.25 美元。
- 2026-11-26 01:21Z cloudrun 开识别补跑机 glx9kkda13gckv（cpu3c 4 核，自删 6 h），preflight 过（剩 79 GB），main ab0f4f0a；手敲提交号出错被 checkout 挡下（退出 11，无调用），改为从 git 取完整号。云上 2 台在跑。
- 2026-11-26 01:29Z PR #201 合并 d9ab0807（#200 关）。删 traindata 本地 rebased-backup-20261126（与 feat/data-batch2 数据目录无差异，未推送）。验收 PR #203（第二批数据）：独立副本 unittest 236 条 OK · 1 跳过，等 CI。
- 2026-11-26 traindata 核「unit 写 file」：数据正确（逐行核单位 = userNNNN，385 人无跨份），只是摘要标签易误读 → issue #204（P2，下一批派 pi 加字段，unit 不动）。
- 2026-11-26 01:51Z cloudrun 删 g1wyqz5tsvqvia（约 0.16 美元，累计 8.02 美元），日志已取回。BUT 实测：字节数 · gzip · tar 过，librispeech 0；解出 wav 12307（RIR 2325 + silence 9982）占 23 GB；盘 153/200 GB。大总管定：BUT 压缩包 9.3 GB 删（依据：人批过删可重建项 + 下完删包规则；md5/sha256 已记），在识别补跑机上顺手删。识别补跑 01:33Z 起，01:50Z +101 次 +0.0097 美元。
- 2026-11-26 01:52Z PR #203 合并 37c75609（#202 关，第二批数据进 main）；删远端 feat/data-batch2（本地分支在 traindata worktree 里，换分支后再删）。traindata 开第三批里不依赖定场景的部分（BUT 钉校验 · 按房间切 · 底噪筛查 · 会议室/酒店池 · #204）。
- 2026-11-26 01:53Z 替 traindata 建 feat/data-batch3（37c75609 + cherry-pick 4862e227 · d1d2e920），删本地 feat/data-batch2；确认无后台进程后关窗并 --continue 重开，钩子改登记 feat/data-batch3。同意按「房间 + 扬声器布置」切 BUT 底噪，开发份对会议室 / 酒店只测同房不同场次（要写明）。
- 2026-11-26 traindata 重开后会话名 noise-canceling-traindata-f0（旧 traindata-56），在 feat/data-batch3 HEAD d1d2e920，开 issue 派 pi。
- 2026-11-26 名额规则：pi 的轻活误报（改文本 · 读文件 · 单个短测试）手下可直接放、事后告诉大总管；起 pi 时用 --heavy-re 收窄重活规则。pi-worker 默认重活规则把含 python3 的命令都判成重活 → 记 backlog。
- 2026-11-26 cloudrun 删 BUT 压缩包（删前核 checksums.txt 与文件 md5），盘 145/200 GB。识别补跑 02:00Z +431 次 +0.041 美元，4 片在跑；按此速度约 2003 次需约 2 h（推算，约 03:35Z 完）。
- 2026-11-26 02:09Z traindata 第三批（#205，feat/data-batch3 354f39fe）代码验收：276 过 · 1 跳过，pi 22 个提交 0.086 美元。批上云：优先在识别补跑机 glx9 上顺手跑；预计盘约 168/200 GB，超 170 先停报。旧分片删不删待 traindata 查清在用引用。
- 2026-11-26 大总管定旧分片：第三批 valid 并取回核对后删 first-batch（现行代码读不了，可用旧提交约 30 min 重建，先取回 build 小文件）；batch2 留到第一个正式运行过开发份检查。cloud_steps.sh 默认 VDN_SHARDS 改成必须显式给（traindata 派 pi）；增量分片模式开 issue 下一件。
- 2026-11-26 更正：给 cloudrun 的「删 first-batch」指令被自动模式权限拦下（判为破坏性），未发出 → 删不删 first-batch 改为待人定（10-4）。
- 2026-11-26 traindata：增量分片 #206（下一件）；README 加「旧分片只能用当时提交复现 · 删不删由人定」（cb817ab1）；派 pi traindata-shards-env（1 美元，--heavy-re 收窄）：shard/register/pack 必须显式 VDN_SHARDS 否则退出 2；cloud_steps.sh 不进 build id，云上第三批不受影响（读源码）。
- 2026-11-26 人定 D112：第三批 valid 后删 first-batch。
- 2026-11-26 02:13Z 批第三批另开 cpu3c 4 核 · 自删 3 h · ≤0.36 美元（实测识别补跑机 load 19–36 / 4 核，识别不是在等网络，大总管原判断错）。识别补跑 02:12Z +857 次 +0.083 美元。
- 2026-11-26 traindata：cloud_steps.sh 改完推 8c6e05c3（282 过，pi 0.0068 美元）；告知 D112，README 改引用。
- 2026-11-26 02:28Z 第三批机 jm4m7pg5zmif3n（02:14Z 开）：准备 · 测试 276 过 · 候选 868 条后，筛查停在 screen.py:285（假模型接口与真 PANNs 不符），疑收音机排除未生效；无分片写出，盘 145 GB。cloudrun 等 30 min 否则删机。识别补跑 02:27Z +1380 次 +0.132 美元。
- 2026-11-26 02:39Z traindata 实测收音机：54 份 spk_meta 只有 L207 两份带 EnvSpk2，C236 · R112 没有，868 条正确，非 bug。真 spk_meta 两份进测试夹具（9291ec39，Apache 2.0），pi traindata-spkmeta（1 美元）改夹具。cloudrun：两房间 keep 各 ≥ 一半就继续 split → shard → register → pack。
- 2026-11-26 02:47Z 识别补跑完成（cloudrun 实测）：+1953 次 · +0.1875 美元（上限 0.25），吃字有值 82→452/480，结果 _tmp/20261126-asr-rerun/rerun-20261126-013313/；机器 glx9 已删（约 0.17 美元），累计 8.22 美元。交 evalcmp 出第二版差距表。第三批 traindata 修 7badd9a1（契约 2 条 · 全套 288 过），C236 筛查 02:46Z 起。
- 2026-11-26 02:48Z evalcmp 确认无后台进程后，大总管建 exp/025-gap-v2 并 --continue 重开，钩子改登记。第二版弱项表前 4：rail-air-cabin · canteen · meeting-room · transit-hub（与第一版同组，3、4 名对调）；28 段无值分在 9 场景。
- 2026-11-26 人定 D113：加训场景 = 最弱前 4（rail-air-cabin · canteen · meeting-room · transit-hub）。第三批筛查：R112 keep 209/248 正常；C236 keep 32/620（不平稳 545 · 说话帧略超 355；标签多为 Music / Tuning fork / Sine wave），停在 split 前，机器已删，累计 8.31 美元，云上 0 台。
- 2026-11-26 人定（D113 续）：C236 先查清（诊断 + 取 3 段约 1 MB 给人听），R112 先登记，阈值不放宽。
- 2026-11-26 03:31Z PR #208 合并（第二版差距表，#207 关），删远端 exp/025-gap-v2（本地分支在 evalcmp worktree，换线时再删）。
- 2026-11-26 大总管定（技术题）：第三批 target_scene 先写 3 个（meeting-room 待池），这批 build 只验证管线、不正式训练；正式训练用会议室池登记后那批 build（D110）。限制：rail-air-cabin 只有地铁车厢录音，canteen 只有 DEMAND PCAFETER 一个来源。
- 2026-11-26 traindata 派 pi traindata-b3b（2 美元）：BUT 只留 R112 · target_scene 3 个 · 配方说明写限制 + diagnose.py（复用 screen.py 平稳度算法）+ 测试；筛查结果 jsonl 868 行进仓（89ab84e8）；3 段 C236 试听已选：1 条 keep · 1 条典型 drop（Tuning fork）· 1 条说话帧最高。
- 2026-11-26 trainer 开 #209 feat/trainer-target-scenes（pi 填 4 个场景 + check 按 MeetingPlace 正本查名）；删远端 feat/trainer-formal-config。钩子登记旧分支问题第 4 次（trainer×2 · traindata · evalcmp）→ 按「同类错误第三次沉淀规则」：手下换线 = 大总管建分支 + 确认无进程 + --continue 重开；根治记 backlog（钩子从登记文件读分支）。
- 2026-11-26 03:38Z traindata 第三批续跑备好（b81a9645，pi traindata-b3b 4 提交 0.0276 美元，316 过 · 3 跳过）；批 cloudrun 开 cpu3c 4 核 · 自删 3 h · ≤0.36 美元跑 data/cloud-batch3b.md（诊断 C236 · 取 3 段试听 · 只切 R112 · shard · register · pack；期望 12 个噪声场景含 hotel-room 不含 meeting-room）。
- 2026-11-26 03:51Z cloudrun 取回 3 段 C236 到 _tmp/20261126-c236-listen/（各 10 s 16 kHz，共 0.96 MB，sha 核过，git 忽略）。诊断 10 条：逐秒电平起伏中位 7.46 dB；频谱峰 250 Hz 占 6/10。第三批续跑机 byvkixm3sfzju6（03:40Z 开），测试 316 过，R112 切 209 条，shard 03:47Z 起（盘 148 GB）。
- 2026-11-26 07:30Z **大总管换为 noise-canceling-87（会话 439efe9c；首次误记为 43），tmux vdn:coord**。由 history 项目大总管 history-a0 应人要求代开。不是正常交接：约 04:44Z（本地 11-25 20:44）tmux 整个消失，旧大总管 c4 与四个手下窗口都已停。接班体检（实测，只查不动）：① RunPod 接口查得 0 台机器；续跑机 byvkixm3sfzju6 已于 04:26Z 由 cloudrun 删（约 0.09 美元，R112 登记 build 10aa3a9fb8cef48f）；网络盘 bzpbedqdsi 200 GB 仍在、按月计费。② 本项目 pi 0 个在跑（pi-worker 状态目录 20 个全部退出码 143）；在跑的 pi 都属 history 项目（kb-*），未动。③ worktree：主仓 chore/coord-20261116-b19 领先远端 23 个未推送；cloudrun 交接页有未提交改动（first-batch 删前依赖已核完，等下一台机开机先删）；traindata feat/data-batch3 干净已推送（0425e820）；trainer 的 pi trainer-target-scenes 在 w/trainer-target-scenes 留了 3 个提交（3afe9e9c，20:41 本地）未合入 feat/trainer-target-scenes、手下未验收；evalcmp 已收口待命。C236 诊断结果（traindata 交接页末）没送到旧大总管：每段第 1 秒高 20–30 dB、250 Hz 强单音，待人听 _tmp/20261126-c236-listen/ 3 段。重开手下等人点头。
- 2026-11-26 07:50Z 人定：开训前清单逐项弄清、人点头才开正式训练（014 第七节）。D114（C236 机器判）· D115（上限 100 美元）。重开 trainer（--continue，#209 验收）· traindata（新会话，旧 516k）· cloudrun（--continue：改上限 · 开 CPU 机删 first-batch · 推分支开 PR）。
