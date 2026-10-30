# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-69`（换班后在这里改）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

## 手下

| 窗口 | 任务 | 交代文件 | 现状页 | 状态 |
|---|---|---|---|---|
| model | 002 起步模型与代码 | `_tmp/20261029-b1/model.md` | `state/handoff/model.md` | 四份初稿已写，自审中（noise-canceling-f0） |
| pipeline | 003 Mac 音频管线 | `_tmp/20261029-b1/pipeline.md` | `state/handoff/pipeline.md` | 六份已填，交付前自审中（noise-canceling-21） |
| evaltool | 004 评测工具链 | `_tmp/20261029-b1/evaltool.md` | `state/handoff/evaltool.md` | 已报到（noise-canceling-0b） |

## 规矩

- 见 `_tmp/20261029-b1/common.md`；本账本只记人后来改过的条目（标「人定 <日期>」）
- 调研必须用 workflow / deep research 做，要非常完整（人定 2026-10-29，原话：「需要用 workflow来做 researrch deep research 来弄，就是这个研究需要非常完整」）
- 提交由 hook 自动做（人定 2026-10-29，CLAUDE.md 铁律 14）；手下不做 git 写操作

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试（1 个） | 空 |

排队：（空）

等人许可的弹窗 / 长时间前台运行：（空）

## 进度板

进度（HH:MM）
- 21:xx model：must-know 已起草；workflow 10 代理（5 核查 + 5 怀疑派）已批准，exp001 之后申请名额
- 21:xx pipeline：must-know 已起草；5 子代理核 12 仓库 → workflow 约 8 代理（补检索 + 3 对抗视角）已批准
- 21:xx evaltool：must-know 已起草；workflow 10 代理（5 检索 + 5 怀疑派）已批准；P.808 克隆 1.1 GB 超线，暂留，收尾再定

## 发现

- 17:35 首派失败：spawn-worker.sh 不识别新版信任提示 → 超时退出，后两窗未开；已修脚本 BLOCK_RE（人定 2026-10-29）
- exp001（M5 · ORT CPU 单线程）每帧 p99：DPDFNet baseline 0.32 · dpdfnet2 0.78 · dpdfnet4 1.26 · dpdfnet8 2.21 · 2_48k 1.12 · 8_48k 3.24 · GTCRN 0.43 ms；dpdfnet8 两版超 2 ms 预算 → docs/experiments/exp001-model-frame-time/（README + raw-output，大总管已抽查数字一致，验收通过）
- model 发现：DPDFNet 流式输出落后输入 4 帧（mask 2 + DF 2），前瞻 40 ms；已要求区分算法延迟与端到端延迟两种口径
- pipeline 核实：第三方 HAL 插件由独立 helper 进程加载、不在 coreaudiod 里（开机日志实测）→ 设计稿「崩了整机没声音」前提过时；开源先例断供都送静音，违背「从不吃字」；异步推理线程应自建 work interval；本机缺 meson
- exp002（M5 CPU）：DNSMOS 每段约 10 s 音频打分约 0.05 s，PESQ/STOI 各约 0.05 s——评测几乎零成本；坑：DNS 仓 dnsmos_local.py:53 与 librosa≥0.10 不兼容，非 16 kHz 输入报错，需先自行重采样（smoke.py:80-86）→ docs/experiments/exp002-eval-smoke/（大总管已抽查：clean 自检 PESQ 4.64 / STOI 1.000，验收通过）
- 交代里大总管名 5-app-89 失效，现为 noise-canceling-69，批次文件已改

## 待定清单（攒着等人一次看完）

- [ ] 个性化（只保留主人声音）：推后 / 核心 / 不做（默认推后）—— 来源 board
- [ ] 远程仓库地址 —— 来源 board
- [ ] 放权边界 —— 来源 board
- [ ] 【最关键】附加延迟上限定多少（40 ms 是 AI 从 DNS Challenge 搬的草稿，人未定）：DPDFNet 前瞻 40 ms，端到端推算约 65–70 ms；零前瞻候选只有 DFN3 _ll（许可不明、停更）、GTCRN/UL-UNAS（约 38–42 ms、效果弱）、nnnoiseless —— 来源 model，见 docs/research/002_model-baseline/decision.md
- [ ] 【急】本机现在装 BlackHole 吗（M0 前提）：A 现在 brew 装并重启（建议）/ B 推后 M0 —— 来源 pipeline
- [ ] 【急】批准插件加载与崩溃实验（写 /Library/Audio/Plug-Ins/HAL + killall coreaudiod，越过本批红线）：A 批准、放 M3 开头（建议）/ B 等开发者账号 / C 不做 —— 来源 pipeline
- [ ] pipeline 其余 6 条（何时办开发者账号 · 出事送原声还是静音 · 回声消除用不用 process tap · BlackHole 阶段给不给外部用户 · 装完 killall 还是重启 · must-know 命门照核实改写）—— 见 docs/research/003_mac-audio-pipeline/decision.md 末节
