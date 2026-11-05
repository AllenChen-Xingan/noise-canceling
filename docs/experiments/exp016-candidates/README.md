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

### 延迟口径（codex 016 脚本第 1、2 轮 P1 的回应）

两种延迟分开记，**本实验只用第一种**：

| 量 | 定义 | DFN3 低延迟版（实测） | 用在哪 |
|---|---|---|---|
| 算法前视（`latency_samples`） | 对齐后第 t 个输出最晚用到第 t + L 个输入；截半实测 | 284 样本（17.75 ms）= libDF 一帧 160 + soxr HQ 前视 124 | 回放进 Engine 的延迟；决定输出内容 |
| 实时调度延迟 | 逐块喂入时，第 t 个输出最早在收齐多少输入后放出（`run_dfn3ll.scheduling_delay`） | **1393 样本（87 ms）**，稳定段与开头一样；soxr 流式内部攒大块所致（VHQ 2639 · MQ 1447 · LQ 939） | 只记录，进 `builds/…/params.json` |

- 模型接口正本（`src/types/src/model.rs`）的 `latency_samples` 是「算法延迟，不含攒满一个 hop 的等待」；内置 GTCRN 的 256 也是算法前视。所以三者在**算法前视**上同口径
- **撤回**之前「三者延迟可比」的说法：实时调度延迟取决于重采样器、攒块方式和设备块，本实验的 soxr 流式链路不代表产品实现。DFN3 低延迟版换成小块低延迟重采样器后能到多少，未量化
- 实时线程能不能跑、端到端多少毫秒，引用 019 结论，不在本实验测

## 已知局限

- 评测音频是 16 kHz（8 kHz 以上没有内容），48 kHz 的两个 DFN3 吃的是升采样后的窄带输入，和它们训练时的全带不同【推断：对它们不利】
- 回放进 Engine 时延迟只是声明值，计时项判「不知道」；实时能不能跑引用 019 结论，不重测

## runs 与配对

| 用途 | run-id | 内容 | 位置 |
|---|---|---|---|
| 候选输出（草稿用，被 `20261105-061524-e4eb13d9` 取代） | `20261104-235743-73c68379` | DPDFNet-2 · 971 个文件 · 14 分 07 秒 · 声明延迟 960 | 音频：主目录 `_tmp/exp016/dpdfnet2/<run-id>/`；清单：`builds/dpdfnet2-<run-id>/` |
| 候选输出（草稿用，被 `20261105-063623-e4eb13d9` 取代） | `20261105-001245-201407de` | DFN3 低延迟版 · 971 个文件 · 15 分 32 秒 · 声明延迟 284 | 音频：主目录 `_tmp/exp016/dfn3-ll/<run-id>/`；清单：`builds/dfn3-ll-<run-id>/` |
| 草稿成绩单（草稿，被 `20261105-103000-4cf04ac1` · `20261105-115707-fcf7cfb0` 取代） | `20261105-002946-e21f3a93`（元数据带 `-dirty`，见 `provenance.json`） | gtcrn · dpdfnet2 · dfn3-ll，**不带 `--qa`**，旧规则，0 段没跑成，7 分 33 秒 | `runs/draft-20261105-002946-e21f3a93/` |
| 最终版 · GTCRN 一侧 | `20261105-005916-4ccf0638`（evalfix 跑，规则 v2，进行中） | passthrough · ref · gtcrn，带 `--qa` | evalfix 跑完放 `docs/experiments/exp014-scorecard-gtcrn/runs/20261105-005916-4ccf0638/` |
| 最终版 · 候选一侧（待跑） | 待定 | dpdfnet2 · dfn3-ll，带 `--qa`；须与上一行同 build · 同规则版本 · 同评分模型版本（跑前逐项核对上一行 README 记的版本） | 跑完填在这里 |
| 最终版 · 候选输出 | `20261105-061524-e4eb13d9` | DPDFNet-2 · 1451 个文件（mix · bg · trunc · clean）· 20 分 50 秒 · 声明延迟 960 | 音频：主目录 `_tmp/exp016/dpdfnet2/<run-id>/`；清单：`builds/dpdfnet2-<run-id>/` |
| 最终版 · 候选输出 | `20261105-063623-e4eb13d9` | DFN3 低延迟版 · 1451 个文件（同上）· 22 分 42 秒 · 声明延迟 284 · `native_rate` 48000 | 音频：主目录 `_tmp/exp016/dfn3-ll/<run-id>/`；清单：`builds/dfn3-ll-<run-id>/` |
| ~~锚点成绩单~~ **作废** | `20261105-065944-a0a04ae6` | passthrough · `--qa --rules v2`；06:59 机器关机重启（系统有 shutdown_stall 报告，起因未查明），只留 4 个识别 wav、没有 `clips.jsonl`。不续用，新跑另起 run-id | 主目录 `_tmp/scorecard/20261105-065944-a0a04ae6/`（原样保留） |
| DPDFNet-2 成绩单输出目录（缓存已预置，11:57 起跑） | 开跑时生成；目录名 `dpdfnet2-seeded-20261105`（用 `--out` 指定） | 同下一行 DFN3 的做法：识别缓存从锚点 run 预置，1920 条，sha256 `61305aba6104a6b470d098431e6fe1cf1407322c331d5208c3f2047a2c0a549f`；外部系统 `_tmp/exp016/dpdfnet2/20261105-061524-e4eb13d9` | 主目录 `_tmp/scorecard/dpdfnet2-seeded-20261105/` |
| DPDFNet-2 成绩单（已完成） | `20261105-115707-fcf7cfb0`，11:57–13:17；配对分析在 `runs/dpdfnet2-20261105-115707-fcf7cfb0/`；识别缓存核对：原 1920 条全在且逐条不变，新增 960 条，共 2880 | — | — |
| DFN3 成绩单（已完成） | `20261105-103000-4cf04ac1`，10:30–11:47；配对分析在 `runs/dfn3-ll-20261105-103000-4cf04ac1/` | — | — |
| DFN3 成绩单输出目录（缓存已预置） | 开跑时生成；目录名 `dfn3-ll-seeded-20261105`（用 `--out` 指定） | 大总管定 2026-11-05：识别漂移实测为 0，所以把原声和干净人声两路的识别结果从锚点 run 灌进来。来源是锚点 run 的 `qa/asr_cache.json`，1920 条（960 条音频 × 2 次），sha256 `61305aba6104a6b470d098431e6fe1cf1407322c331d5208c3f2047a2c0a549f`。读源码：`ensure_fresh` 只查 `scorecard.json`/`.md`；`Asr::new` 读出已有缓存，解析失败就当空缓存，所以跑完要核对缓存条数 ≥ 1920 且这 1920 条原样还在 | 主目录 `_tmp/scorecard/dfn3-ll-seeded-20261105/` |
| 锚点成绩单（已完成，比对过线） | `20261105-073042-a0a04ae6-dirty` | passthrough · `--qa --rules v2`，07:30–10:26，480 段。与 GTCRN run 的 passthrough 逐段比：23 项全部差 0（`runs/anchor-…/anchor.json`）；识别缓存 1920/1920 逐字相同（`asr_drift.json`）；评分模型 sha256 与 GTCRN run 相同（SCOREQ `d493ba8a…` · Distill-MOS `b8e2dc18…`）；识别服务 `qwen3-asr-plus:1.7b` · 4412418975 字节。`-dirty` 只因开跑时本 README 有 3 行未提交（上面「作废」那一行等），`src/` 与 `Cargo.lock` 与 HEAD `a0a04ae6` 相同（实测 `git status --short` 只列本文件）；不重跑（大总管定 2026-11-05） | 主目录 `_tmp/scorecard/20261105-073042-a0a04ae6-dirty/` |

最终版两个候选都**另开新 run**，mix 与 clean（干净人声单独过一遍，成绩单新规则 v2 用）一次跑全；DFN3 新 run 的 `run.json` 直接写 `"native_rate": 48000`。旧的两个 run 不补、不改，跑完后在上表标「草稿，被 <新 run-id> 取代」（大总管定 2026-11-05，撤回先前的「补 native_rate + amend.json」）。

分析：`analyze.py <run 目录> <exp013 clips.jsonl> --out analysis.md --json analysis.json`（测试 `tests/test_analyze.py`）。草稿只出单 run 里的配对；最终版需要把两个 run 的 `clips.jsonl` 合到一起再分析（到时给 analyze.py 加多 run 输入，带测试）。
