# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-c4`（2026-10-31 换班；旧名 noise-canceling-69）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

## 当前实体（2–4 个）

> ⚠️ 2026-11-02 规则改为「大总管一件大事 · 每个手下 2–4 个实体」；本表是旧版写法，定下当前大事后改写

> 总入口：`docs/plan/009_model-map.md`（模型大问题的四个实体 · 矛盾 · 待拍板 M-1–M-6）。人定 2026-10-31：「就关于模型这个大问题的4个实体，我们吧ta 里清楚」。

| 实体 | 挂在下面的线 |
|---|---|
| **模型**（降噪 · 认人 · 场景 · 评分 · 调节） | 018 judgeloop |
| **推理后端**（ORT → Core ML / MLX / 自写 Rust，D53） | 019 engine |
| **评测器**（verifier · 评委 · 人耳） | 017 ears |
| **数据**（合成格 · 真实锚 · 众人评分） | 016 benchsynth |

训练是作用在「数据 → 模型、由评测器判」上的事，不单列。

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
| benchsynth | 016 评测基准 + 合成数据标准（按评测集倒推合成数据，D50）· issue #41 · `research/016-bench-synth`（worktree `../noise_canceling-benchsynth`） | `_tmp/20261031-b6/benchsynth.md` | 其 worktree `state/handoff/benchsynth.md` | 已报到（noise-canceling-benchsynth-49）|
| ears | 017 覆盖更多人耳反馈（公开人评数据集 · MOS 预测器 · 音频 LLM 评委 · 作者听测提效 · 少量现场真人）· issue #42 · `research/017-more-ears`（worktree `../noise_canceling-ears`） | `_tmp/20261031-b6/ears.md` | 其 worktree `state/handoff/ears.md` | 已报到（noise-canceling-ears-03）· 实验编号 exp009 |
| judgeloop | 018 本机评分模型 + 调节模型闭环（D52）· issue #44 · `research/018-ondevice-judge`（worktree `../noise_canceling-judge-loop`） | `_tmp/20261031-b6/judgeloop.md` | 其 worktree | 已报到 |
| engine | 019 推理引擎 Core ML / MLX / 自写 Rust 对比（D53）· issue #45 · `research/019-apple-inference`（worktree `../noise_canceling-engine`） | `_tmp/20261031-b6/engine.md` | 其 worktree | 已报到 |
| p2fix | 让 vdn-speaker 过架构闸门（拆出离线实验工具 crate · 改 tier） | `_tmp/20261031-b4/p2fix.md` | p2 worktree | 完成（tidy / check-deps 全绿），随 PR #32 合并，窗口已关 |

## 规矩

- 见 `_tmp/20261029-b1/common.md`；本账本只记人后来改过的条目（标「人定 <日期>」）
- 调研必须用 workflow / deep research 做，要非常完整（人定 2026-10-29，原话：「需要用 workflow来做 researrch deep research 来弄，就是这个研究需要非常完整」）
- 一切走提交 · issue · PR（人定 2026-10-30，CLAUDE.md 铁律 15）：每条线一个分支 + worktree（人 2026-10-30 追加「现在必须开分之 调研分支」）：research/005-pse（Closes #1）· exp/003-offline-ab（Closes #2），各自 PR、CI 过后自己合并
- 提交由 hook 自动做（人定 2026-10-29，CLAUDE.md 铁律 14）；手下不做 git 写操作
- **大总管一次只做好一件大事，每个手下只管 2–4 个实体**（人定 2026-11-02，改 2026-10-31 版，见 CLAUDE.md 第五节第 4 条）：线 / 题 / 讨论都挂在当前实体下，其余按实体记进 `state/backlog.md`

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试（1 个） | benchsynth（exp008 第二轮：装 torch + 补 Hush · DFN3 · RNNoise · WebRTC NS · FRCRN · MossFormer 系 · BSRNN · G-MaP-SE 重跑排名） |

排队：ears（exp003 音频重生成 + SCOREQ / Distill-MOS / DNSMOS 打分，已批）→ engine（venv：torch + coremltools + mlx + onnxruntime，< 300 MB；exp010 基准）→ judgeloop

等人许可的弹窗 / 长时间前台运行：benchsynth 跑 Hush 预编译 dylib（Hush ×2 + DFN3）被其会话自动权限检查拦下（Code from External）；人的批准经大总管转达不算该会话授权 → 需人在 tmux vdn 的 benchsynth 窗口直接批准，否则记缺口

## 进度板

进度（HH:MM）
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
