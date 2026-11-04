# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-e3`（2026-11-02 换班；旧名 c4 · 69）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

## 当前大事（2026-11-03 起）

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
| card1 | exp014 第一张场景成绩单（GTCRN）· #92 · `exp/014-scorecard-gtcrn` · `../noise_canceling-card1` | 成绩单（报告）· 降噪模型 GTCRN · 评测器（只用） | 已报到（noise-canceling-card1-d8） |
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

- **记决策号前先 `grep -n '^### D' docs/plan/001_decisions-20261029-b1.md | tail -1` 看文件最大号，并向另一位大总管发「占用 Dxx」**（2026-11-02 两次撞号：47 方 D66–D68；主检出 D77 被另一会话先用，我方改 D78）

- **所有代码文件 ≤ 600 行**（人定 2026-11-02，经 8d 转，随 #62 写进 CLAUDE.md 第五节第 7 条；不设基线、不留豁免，含 docs/experiments）：fastpath 合并前把 gtcrn_native.rs 1926 · main.rs 1280 · gtcrn_native/v2.rs 605 拆到 600 以下；#58 合并后通知 8d 扩扫描到 docs/

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试 / 跑模型（1 个） | card1（正式成绩单 480 段 --qa，约 1–2.5 h） |

排队：无

**跨大总管约定**（与 noise-canceling-47，tmux vdn-rust，2026-11-02）：任一方跑计时基准前后互发「基准开始，约 N 分钟」/「基准结束」，期间对方不起编译、测试、推理；普通编译测试不打招呼。加锁文件：mkdir 主目录 `_tmp/heavy.lock`（内写 owner · 开始 · 预计结束，跑完 rmdir），由各自大总管放名额时建、收回时删，手下不碰（47 已确认）

## 进度板

进度（HH:MM）
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
