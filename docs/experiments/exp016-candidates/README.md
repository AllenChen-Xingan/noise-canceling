# exp016 · 候选降噪模型上同一张场景成绩单

> issue #112 · 分支 `exp/016-candidates` · 手下 cands · 2026-11-04 起
> 意图（D80）：在开会场景里比 GTCRN 与其他候选，回答「GTCRN 在哪几个场景、哪种声音上落后于候选，落后多少」。只测、只分析，不训练、不改模型。
> 本批按 025 待拍板 G-1..G-7 的默认推进，**待人确认**。

## 怎么做

1. 外部离线输出入口：`vdn-eval scorecard --system external:<目录>`，形状见 [`external-entry.md`](external-entry.md)
2. 三个候选各写一个离线推理脚本（本目录），把 exp013 正式 build `20261103230756-17f5` 的 480 段跑成外部输出目录（主目录 `_tmp/exp016/<候选>/<run-id>/`，音频不进仓）
3. 每个候选出成绩单（带 `--qa`），与 GTCRN 做配对差值 → `findings.md`
4. 挑 ≤ 8 段人耳片段（盲听，揭盲表在 `builds/<build-id>/`）

## 三个候选怎么离线跑（查于 2026-11-04，不占名额，未跑推理）

| 候选 | 权重 · 来源 · sha256 | 运行方式 | 采样率 | 流式 / 延迟 | 480 段预计耗时 |
|---|---|---|---|---|---|
| **DFN3 低延迟版**（`dfn3-ll`） | 官方 `DeepFilterNet3_ll_onnx.tar.gz`（`resources/DeepFilterNet/models/`，上游提交 `d375b2d8`）`5998e58e…cf3a` | Hush 预编译 `libweya_nc.dylib`（内嵌 libDF，`model = deepfilternet3` 放行，按包里 `config.ini` 运行）经 ctypes 逐帧调用；dylib `4bcd3863…aa`，Hush 提交 `9f6414e9`；人定 2026-10-31 允许只作对照使用 | 模型 48 kHz：先在外面用 soxr 把 16 kHz 升到 48 kHz，处理后降回 16 kHz（dylib 自带的是线性重采样，不用）【读源码 · 015 notes/G-system-ladder】 | 逐帧流式（hop 480 @48 k = 10 ms）；`df_lookahead = conv_lookahead = 0`【读源码 · 包内 config.ini:51,54】；算法延迟 = 窗长 − hop，另加重采样滤波的前视，脚本实测后写进 `latency_samples`，截半自检把关 | 【推断】每帧 ≤ 2 ms（015 推算），≈ 0.2 倍实时；输入约 2.9 h 音频（480 × 12 s × mix + bg + trunc）→ 单进程约 35 min，4 进程约 10 min |
| **DPDFNet-2**（`dpdfnet2`） | `resources/weights/dpdfnet/dpdfnet2.onnx` `4f0ee289…c94c`（DPDFNet 提交 `9bd9844a`） | onnxruntime 逐帧（state 向量回灌），沿用 exp003 `enhance.py` 的 `Dpdfnet` 类与 `stream()`（导入，不复制） | 16 kHz 原生，不转 | 逐帧流式；输出晚 4 帧 = 640 样本（exp003 已用互相关核过）；2 段串联共 60 ms 算法延迟 | 【推断】本机 ort 每帧最坏 0.89 ms（exp001）+ Python FFT 开销，≈ 0.1 倍实时 → 单进程约 20 min |
| **NoNoise-Mac**（`nonoise-mac`） | 仓库 `ivalsaraj/NoNoise-Mac` v1.5.1（`b5844a83`，MIT）自带 Core ML `DeepFilterNet3_Streaming.mlmodelc`（半精度，权重 4.3 MB；元数据只写 TorchScript 转换，**普通版还是 `_ll` 版无法判定**） | 自带命令行 `NoNoiseMacCLI --denoise in.wav --output out.wav --preset meeting`（meeting 档 = 纯模型输出、无后处理链）；**不用驱动、不回录**。要下载 release 的 cli.zip（260 KB）+ app.zip（7.2 MB，命令行从 app 里找模型）；未公证的第三方二进制 → **待人定** | 输出固定 48 kHz → 降回 16 kHz；输入 16 kHz 由它内部处理（待实测） | 逐帧流式（10 帧窗口 + 循环状态，只出最新一帧，零前瞻）；开头输出缓冲未预填，约 512 样本偏移要实测 | 【推断】Core ML 走 Neural Engine，未量化；命令行每段一次进程启动 |

来源标注：【读源码】= 本机 `resources/` 或上游仓库源码；【推断】= 未实测。NoNoise-Mac 一行出自子代理只读查看其 GitHub 源码（未下载、未运行），细节文件行号见 `state/handoff/cands.md` 流水。

### 共同口径

- 输入：exp013 build 的 `mix.wav`；`bg` = `noise.wav + interf.wav` 逐样本相加；`trunc` = 每个场景清单顺序第一段的 mix，从第 floor(n/2) 样本起补零（与入口 `Kind::input` 同一算法，脚本以入口为正本、加一条测试比对）
- 输出：已对齐（去掉延迟）的 16 kHz 单声道 32 位浮点；`latency_samples` 写实测并经截半自检
- 不做响度统一：成绩单各项自己处理电平（与内置 GTCRN 同口径）
- 参数、权重哈希、清单进 `builds/<run-id>/`；音频只在主目录 `_tmp/exp016/`

### 延迟口径（codex 016 脚本第 1 轮 P1 #2 的回应）

- `latency_samples` 照模型接口正本的定义（`src/types/src/model.rs`「输出比输入晚的样本数（算法延迟，不含攒满一个 hop 的等待；那一项由 engine 记账）」）只算**算法前视**：对齐后第 t 个输出最晚用到第 t + L 个输入。内置 GTCRN 同口径（512 − 256 = 256），所以三者可比
- **不含**实时调度的等待：攒满一帧（DFN3 低延迟版 480 @48 k = 160 @16 k；DPDFNet-2 160；GTCRN 256）、soxr 内部攒块、音频设备块。这些在回放里不出现，成绩单的延迟项只证明「输出与声明一致」（`latency_declared_only`）
- 所以 DFN3 低延迟版的 284 样本是**下界**，不是「实时能跑到 17.75 ms」。实时线程能不能跑、端到端多少毫秒，引用 019 结论，不在本实验测

## 已知局限

- 评测音频是 16 kHz（8 kHz 以上没有内容），48 kHz 的两个 DFN3 吃的是升采样后的窄带输入，和它们训练时的全带不同【推断：对它们不利】
- 回放进 Engine 时延迟只是声明值，计时项判「不知道」；实时能不能跑引用 019 结论，不重测
