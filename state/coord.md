# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-69`（换班后在这里改）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

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
| p2 | P2 认出你的声音（离线）· issue #18 · `feat/p2-speaker` | `_tmp/20261031-b4/p2.md` | 其 worktree | 已报到（noise-canceling-p2-63） |
| scene | 场景识别与路由 + 修音链 + 模拟环境 · issue #19 · `feat/scene-polish` | `_tmp/20261031-b4/scene.md` | 其 worktree | 已报到（noise-canceling-scene-f7） |
| ondevice | 010 设备端自适应 + 可靠性调研 · issue #20 · `research/010-ondevice-adaptation` | `_tmp/20261031-b4/ondevice.md` | 其 worktree | 完成，PR #23 已合并（issue #20 关），窗口已关；3 题待人拍板 |
| multitalker | 011 一大堆人同时说话（调研）· issue #21 · `research/011-multitalker` | `_tmp/20261031-b4/multitalker.md` | 其 worktree | 完成，PR 已合并（issue #21 关），窗口已关；6 题待人拍板 |
| evalstd | 012 工业级 rubric 与评测的行业标准 · issue #22 · `research/012-eval-standards` | `_tmp/20261031-b4/evalstd.md` | 其 worktree | 完成，PR #27 已合并（issue #22 关），窗口已关；9 题待人拍板 + rubric 修改清单 R1–R14 |
| rlenv | 013 强化学习环境与分场景训练方案 · issue #25 · `research/013-rl-environment` | `_tmp/20261031-b4/rlenv.md` | 其 worktree | 已派 |

## 规矩

- 见 `_tmp/20261029-b1/common.md`；本账本只记人后来改过的条目（标「人定 <日期>」）
- 调研必须用 workflow / deep research 做，要非常完整（人定 2026-10-29，原话：「需要用 workflow来做 researrch deep research 来弄，就是这个研究需要非常完整」）
- 一切走提交 · issue · PR（人定 2026-10-30，CLAUDE.md 铁律 15）：每条线一个分支 + worktree（人 2026-10-30 追加「现在必须开分之 调研分支」）：research/005-pse（Closes #1）· exp/003-offline-ab（Closes #2），各自 PR、CI 过后自己合并
- 提交由 hook 自动做（人定 2026-10-29，CLAUDE.md 铁律 14）；手下不做 git 写操作

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试（1 个） | scene（exp006 开 / 关对比推理 + DNSMOS + Qwen 打分） |

排队：（空）

等人许可的弹窗 / 长时间前台运行：（空）

## 进度板

进度（HH:MM）
- align 全量：fmt / clippy 过；test debug 73 · release 71 全过；真权重冒烟（GTCRN · FE-M）debug / release 过；tidy / check-deps 过
- align 延迟账改逐项硬加：按 009 目标 IO 256 帧 = 47.00 ms（超 D32 2 ms，测试里钉成已知缺口）；提议 IO 96 帧（两模型 hop 公约数）= 43.67 ms → 待人拍板（代价：回调约 2 ms 一次的 CPU 开销、掉帧风险、蓝牙能否支持）
- 010 ondevice：最值得照搬 ReSound Learning VC（每场景一个偏移量 + 滑动平均 + 停手才算同意 + 不一致降学习率）；接口需求 13 条已给 align；要另开实验：powermetrics 耗电、一周空闲窗口计数；⚠️ 一路检索子代理违规跑了一行只读 python3 -c（已自报）
- p2 段②：fbank 对拍最大差 1.4e-4、声纹余弦 1.000000；8 段锁定 8/8 都锁对主人（含「开头别人先说 25 s / 45 s」），锁定后主人独白 0 次判成别人；⚠️ 个性化只比不个性化多压邻座约 0.5–3 dB——门控要等 1.5 s 声纹窗确认，邻座每段 3–8 s，大半时间门开着
- p2 段①：vdn-speaker 13 测试过（含实时不分配断言，hop 256 / 160），clippy / fmt 过；段②进行中
- align：vdn-types（零依赖，模型接口 + 感知 / 策略 / 场景参数表 / 记忆 / 决策日志 / 延迟账，40 单测）；008 P1 五项完成；xtask 放 src/xtask/（避开铁律 10）；已要求删 `.cargo/config.toml`（新顶层目录，铁律 10），改写全命令
- multitalker ②（待自审核实）：MacBook 内置三麦对第三方只给 1 路、已被系统波束成形（本机 system_profiler 1ch/48k），AirPods 单声道 → 内置麦上自己做波束成形走不通，降为「外接多麦」实验线；语音突显只对开了 VPIO 的 app 生效、需用户手动选——可考虑用 VPIO 采集让系统先突显，VPIO 附加延迟与能否关其 AGC / 降噪待实验
- scene 滞回已修：已生效准确率（held-out，不含开机头 1 s）噪音大类 87% · 用户层 93%（5 s 后 95%）；换场景中位 3.7 s 生效、错切 0.88 次 / 分钟；参数 EMA 0.5 s / 确认 1 s / 置信 0.4 / 停留 3 s（已发 align）；算力约 0.07% 个核；「多人会议」只留手动占位，自动识别列待人拍板
- scene：exp006 模拟 856 段 / 2.6 h；特征 + 判断合计约 0.06% 个核；分类器（held-out 逐窗）细场景 73% · 用户层 85% · 蓝牙检出 93% 误报 1.3%；滞回「第一判即生效」有 bug（已生效准确率 41%）修复中。名额规则补充：单 crate 增量构建 + 秒级特征提取不占名额，推理 / 全量 / 超 1 分钟满载仍要申请
- 第四批 20261031-b4（人原话见 `_tmp/20261031-b4/common.md`；硬约束：延迟账 ≤ 45 ms · 察觉不到占用 · 无云端 · 不同线不改同一批文件）：align · p2 · scene · ondevice 已起；人追问「一大堆人」后追加 multitalker（011）；人要求「所有效果都要有工业级的 rubrics 和 eval」后追加 evalstd（012）；人提出「别人做不好的才做 · RL 环境分场景训模型」（D42 · D43）后追加 rlenv（013）；大总管分支 `chore/coord-20261031-b4`
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

## 待定清单（攒着等人一次看完）

- [x] 第一批 24 题 + 个性化 + P.808 —— 2026-10-30 已拍板，见 `docs/plan/001_decisions-20261029-b1.md`
- [x] 远程仓库 —— 2026-10-30 建私有仓库 AllenChen-Xingan/noise-canceling
- [ ] 放权边界 —— 来源 board
- [ ] P1 待人拍板 3 题（快开慢关默认关掉 · GTCRN 权重复制到 ~/vdn-data · CI 建议）—— `docs/experiments/exp004-p1-rust-parity/README.md`
- [ ] 009 待人拍板 4 题（最低系统 14.2→14.4 · GPL-3.0-or-later vs -only · HAL 拿共享内存方式 · RustFFT 破例）—— `docs/research/009_rust-ecosystem/decision.md` §五
- [ ] 008 待人拍板 7 题（意向书「不分配不加锁」改三档表述 · 目录名 · panic=abort · 两个进程 · HAL 目录放哪 · 延迟超 45 ms 先砍哪 · tract 对照实验）—— `docs/research/008_rust-architecture/decision.md`
- [ ] 008 给 p1 后续批次 5 项改动（含 exp004 README 补注「0 次分配只计 Rust 侧」）—— 同上「给 p1 线的后续改动」
- [ ] 012 待人拍板 9 题（「工业级」在本项目指什么 · 众包与 D39 · rlenv 过渡期能否用 L1 当弱奖励等）+ rubric 修改清单 R1–R14 —— `docs/research/012_eval-standards/decision.md`；升级清单 P0：人耳加 P.835 三量表 · RL 奖励同策略审计 · ETSI 噪声格子 · verifier 定标统计 L0–L3
- [ ] 011 待人拍板 5 题（「只留我」一个开关 · 声纹认好后默认开 · 会议室自动放开 · 试借语音突显 · 菜单栏小图标提前）；~~#6 提示规则~~ 人已否决（D42：不做耳麦提示）—— `docs/research/011_multitalker/decision.md`
- [ ] 待开实验（011）：① VPIO / 语音突显 + 4 声道 + 蓝牙麦延迟（随 dev-4，P0）② babble 人数×SNR 曲线（dev-3）③ 近 / 远特征 AUC ④ 会议室识别回归 ⑤ 商业对照；提醒 dev-4：别的进程在内置麦开 VPIO 时 HAL 读到的电平骤降（cubeb 源码）
- [ ] 010 待人拍板 3 题（空闲小更新素材：只累计归一化统计 · 通话中不探索 · 软件内「检查更新」算云端、不做）—— `docs/research/010_ondevice-adaptation/decision.md` 第五节
- [ ] 006 待人拍板 4 题（延迟察觉实验 · RX 当天花板参照 · 开 issue 问许可等）—— `docs/research/006_sota-landscape/decision.md` 末节
- [ ] 007 待人拍板 4 题（权重 · 耳朵与机器怎么合 · V8 口径 · 下一轮怎么听）—— `docs/research/007_denoise-rubric/decision.md`
- [ ] 要不要去 FastEnhancer 仓库开 issue 问权重许可（对外发言）：仓库 MIT，权重无单独声明（训练数据 DNS3 宽带 + VCTK 0.92）；建议等听测后它排前面再问 —— 来源 rubric / abtest
- [ ] 005 PSE 待人拍板 7 条（训练与否 · VoxCeleb 声纹模型 · 耗电上限 · 零设置口径 · 隐私存什么 · 状态提示 · 对外开 issue）—— 见 `docs/research/005_pse/decision.md` 末节；#3 耗电与 VoxLock 等 exp003 实测后再问
- [x] 开下一批 005 PSE + exp003 AB 听测 —— 人 2026-10-30 同意，已派
