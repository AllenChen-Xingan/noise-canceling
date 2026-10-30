# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-69`（换班后在这里改）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

## 手下

| 窗口 | 任务 | 交代文件 | 现状页 | 状态 |
|---|---|---|---|---|
| model | 002 起步模型与代码 | `_tmp/20261029-b1/model.md` | `state/handoff/model.md` | 完成，已收尾（窗口已关）；剩 3 题 + 三命门待人拍板 |
| pipeline | 003 Mac 音频管线 | `_tmp/20261029-b1/pipeline.md` | `state/handoff/pipeline.md` | 完成，已收尾（窗口已关）；剩 10 条待人拍板 |
| pse | 005 个性化降噪（PSE）调研 · issue #1 · 分支 `research/005-pse`（主目录） | `_tmp/20261030-b2/pse.md` | `state/handoff/pse.md` | 完成，已收尾（窗口已关）；7 条待人拍板 |
| abtest | exp003 DPDFNet-2 vs GTCRN 离线 AB 听测 · issue #2 · 分支 `exp/003-offline-ab`（worktree `../noise_canceling-exp003`） | `_tmp/20261030-b2/abtest.md` | `state/handoff/abtest.md` | 已搬进 worktree、新会话已报到（noise-canceling-exp003-2d）；范围加 VoxLock |
| sota | 006 降噪前沿全景 · issue #3 · 分支 `research/006-sota-landscape`（worktree `../noise_canceling-sota`） | `_tmp/20261030-b2/sota.md` | `state/handoff/sota.md`（在其 worktree） | 已报到（noise-canceling-sota-4f） |
| rubric | 007 降噪评分标准 rubric · issue #4 · 分支 `research/007-denoise-rubric`（worktree `../noise_canceling-rubric`） | `_tmp/20261030-b2/rubric.md` | `state/handoff/rubric.md`（在其 worktree） | 已报到（noise-canceling-rubric-e1） |
| evaltool | 004 评测工具链 | `_tmp/20261029-b1/evaltool.md` | `state/handoff/evaltool.md` | 完成，已收尾（窗口已关）；剩 11 题待人拍板 |

## 规矩

- 见 `_tmp/20261029-b1/common.md`；本账本只记人后来改过的条目（标「人定 <日期>」）
- 调研必须用 workflow / deep research 做，要非常完整（人定 2026-10-29，原话：「需要用 workflow来做 researrch deep research 来弄，就是这个研究需要非常完整」）
- 一切走提交 · issue · PR（人定 2026-10-30，CLAUDE.md 铁律 15）：每条线一个分支 + worktree（人 2026-10-30 追加「现在必须开分之 调研分支」）：research/005-pse（Closes #1）· exp/003-offline-ab（Closes #2），各自 PR、CI 过后自己合并
- 提交由 hook 自动做（人定 2026-10-29，CLAUDE.md 铁律 14）；手下不做 git 写操作

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试（1 个） | abtest（exp003 VoxLock 一轮：计时 + 处理 + 建页 + 打分） |

排队：（空）

等人许可的弹窗 / 长时间前台运行：（空）

## 进度板

进度（HH:MM）
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
- [ ] 005 PSE 待人拍板 7 条（训练与否 · VoxCeleb 声纹模型 · 耗电上限 · 零设置口径 · 隐私存什么 · 状态提示 · 对外开 issue）—— 见 `docs/research/005_pse/decision.md` 末节；#3 耗电与 VoxLock 等 exp003 实测后再问
- [x] 开下一批 005 PSE + exp003 AB 听测 —— 人 2026-10-30 同意，已派
