# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-69`（换班后在这里改）。tmux 工作区：`vdn`（`tmux attach -t vdn` 可以看）。

## 手下

| 窗口 | 任务 | 交代文件 | 现状页 | 状态 |
|---|---|---|---|---|
| model | 002 起步模型与代码 | `_tmp/20261029-b1/model.md` | `state/handoff/model.md` | 完成，已收尾（窗口已关）；剩 3 题 + 三命门待人拍板 |
| pipeline | 003 Mac 音频管线 | `_tmp/20261029-b1/pipeline.md` | `state/handoff/pipeline.md` | 完成，已收尾（窗口已关）；剩 10 条待人拍板 |
| evaltool | 004 评测工具链 | `_tmp/20261029-b1/evaltool.md` | `state/handoff/evaltool.md` | 完成，已收尾（窗口已关）；剩 11 题待人拍板 |

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
- 21:20 model：完成，大总管验收通过，窗口已关
- 21:05 pipeline：完成，大总管验收通过，窗口已关
- 21:12 evaltool：完成，大总管验收通过，窗口已关

## 发现

- 17:35 首派失败：spawn-worker.sh 不识别新版信任提示 → 超时退出，后两窗未开；已修脚本 BLOCK_RE（人定 2026-10-29）
- exp001 重跑（M5 · ORT 1.31.1 单线程 · 真实带噪语音 · 真 state · 0 NaN · 含 STFT/iSTFT）p99.9/max ms：baseline 0.37/0.56 · dpdfnet2 0.84/0.89 · dpdfnet4 1.32/1.40 · dpdfnet8 2.26/2.52 · 2_48k 1.18/1.25 · 8_48k 3.47/3.99 · GTCRN 0.48/0.62；全部在 5 ms 线内，2 ms 线只 dpdfnet8 两版超；与首跑基本一致 → docs/experiments/exp001-model-frame-time/（大总管抽查：state 按元数据初始化 bench.py:51-53、逐帧 isfinite 检查 :109、合计 max 为两项 max 相加属保守上限、旧版见 89bf562，验收通过）
- evaltool 收尾补充：回归门禁两基线（对原声直通用绝对判据；对上一版用相对容差 + 配对 bootstrap）；吃字率任一家 ASR 报警即拦；SCOREQ 在 URGENT 2024 逐句相关 .79、真实录音 .75（已核原表），走 ONNX 不需 PyTorch
- evaltool 取舍：每把尺子「主指标 + 交叉指标」、分 CI 层 / 本机层；吃字 = whisper.cpp 删除率（中文加 SenseVoice 第二意见）；自然整体 = SCOREQ + UTMOS（DNSMOS 降为冒烟）；NISQA 权重 NC-SA 只作用户自装；PESQ 不进仓；附加延迟现成工具全不覆盖，要自写
- model 发现：DPDFNet 流式输出落后输入 4 帧（mask 2 + DF 2），前瞻 40 ms；分三列：窗长+前瞻 60 ms · DNS 口径（窗长+步长+前瞻）70 ms · 端到端推算约 65–70+ ms（DNS 定义已对 PDF 原文核实）
- pipeline 结论：开发期 cpal 双流写 BlackHole + rubato 按水位补漂移；最终自写极简 C 回环插件，推理永不进插件；不用共享内存 / DriverKit
- pipeline 转交：结论进 vault/truth/（等人拍板后由大总管做）；推理预算 → 已转 model；旁路交叉淡化时长怎么评 → 已转 evaltool
- pipeline 核实：第三方 HAL 插件由独立 helper 进程加载、不在 coreaudiod 里（开机日志实测）→ 设计稿「崩了整机没声音」前提过时；开源先例断供都送静音，违背「从不吃字」；异步推理线程应自建 work interval；本机缺 meson
- exp002（M5 CPU）：DNSMOS 每段约 10 s 音频打分约 0.05 s，PESQ/STOI 各约 0.05 s——评测几乎零成本；坑：DNS 仓 dnsmos_local.py:53 与 librosa≥0.10 不兼容，非 16 kHz 输入报错，需先自行重采样（smoke.py:80-86）→ docs/experiments/exp002-eval-smoke/（大总管已抽查：clean 自检 PESQ 4.64 / STOI 1.000，验收通过）
- 交代里大总管名 5-app-89 失效，现为 noise-canceling-69，批次文件已改

## 待定清单（攒着等人一次看完）

- [x] 第一批 24 题 + 个性化 + P.808 —— 2026-10-30 已拍板，见 `docs/plan/001_decisions-20261029-b1.md`
- [ ] 远程仓库地址 —— 来源 board
- [ ] 放权边界 —— 来源 board
- [ ] 开下一批 005「个性化降噪（PSE）」调研？（D08 定为核心）—— 等人点头
