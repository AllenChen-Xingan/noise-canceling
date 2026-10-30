# 取舍卡 · 起步用哪个降噪模型与代码作为基线

> 2026-10-29 · 手下 model 起草。依据：`sources.md`（逐条带出处）· `gap.md`（候选 × 命门）· `docs/experiments/exp001-model-frame-time/`（本机实测）。
> ⚠️ 最后一栏「需要你拍板的地方」AI 只列选项与建议，**不代填**。

## 这次不可妥协的几条（AI 起草，待人改，见 `must-know.md`）

1. 能实时跑：因果、算法延迟 ≤ 40 ms、M 系列单核每 10 ms 帧 ≤ 2 ms
2. 代码和权重的许可证都允许开源发布与商用
3. 能在 Rust 里逐帧跑（Rust 原生，或 ONNX 且状态显式）

## 我选择什么

> **首选 DPDFNet-2（16 kHz）官方 ONNX，用 `ort` 在 CPU 上逐帧跑，Rust 外壳（STFT/OLA、state、4 帧对齐）自己写；engine 按「STFT 在图外 + 状态显式的单帧 ONNX」做成可换模型的接口，备选 GTCRN 用同一接口接入。**
> 前提：人接受 DPDFNet 约 60 ms 的算法延迟（拍板 #1）。不接受的话，首选换成 GTCRN。

## 我放弃什么

> 放弃「libDF 原样跑 DFN3 当 M1 基线」（设计稿 §16 的 M1）：DFN3 权重许可不明、上游 2024-10 起停更、默认阈值在 −10 dB 以下整帧静音，三样都直接撞本项目的命门和真难题。DFN3 只留作评测里的参照，libDF 只当「读懂后自己写」的 STFT 参考。

## 四选一判断

| 候选 | 判断 | 一句话理由（出处见 sources.md） |
|---|---|---|
| **DPDFNet** | **改造**（权重直接用，Rust 外壳自写） | Apache-2.0 权重；状态显式的单帧 ONNX；作者低 SNR 集上效果最好；本机 p99 0.78 ms（dpdfnet2）。缺：延迟 60 ms |
| **GTCRN**（+ biglinux 插件） | **改造**（备选） | 唯一一个许可证有作者书面确认、延迟 32 ms（窗口径）、Rust + ort 逐帧代码现成的；效果天花板低（低 SNR 集 PESQ 2.21 vs 3.14） |
| UL-UNAS | 改造（后续 A/B，非起点） | 和 GTCRN 接口几乎一样、DNS3 集上更好，但权重无专门声明、没实测 |
| DeepFilterNet3 / libDF | **只借鉴架构** | 代码最完整，但权重许可不明 + 停更 + 低 SNR 静音分支 + 每帧深拷贝 |
| nnnoiseless | 只借鉴架构 + 垫底基线 | Rust 逐帧 API 写法范本；效果最弱 |
| ort | 直接用（零件） | macOS arm64 预编译、IoBinding、单线程设置齐全；只有 rc 版，锁版本 |
| tract | 不适合作起点（备用） | 想去掉 C++ 依赖时再试；每个模型都要先冒烟 |
| aTENNuate / FRCRN / MP-SENet | 不适合 | 延迟 46.5 ms 且官方仓 404 / 12.3 GMACs/s 超预算 / 非因果 |

## 依据来自哪一类

- [x] 自有积累 —— 设计稿 §5、§11、§16：算力预算和方向仍算数；「DPDFNet-4 最平衡」被推翻一半（没算 4 帧内部延迟）
- [x] 他人积累 —— 8 个仓库读到源码行号、4 篇论文读到表格；每份核查都过了一轮怀疑派反驳（推翻并改正了 7 处，见 sources.md）
- [x] 自己跑的 —— exp001：把「DPDFNet-4 能实时」从估算变成本机实测，并把 dpdfnet8 排除

## 让哪把尺子变好（`.42cog/intent.md`）

- **清楚** ↑：DPDFNet 在低 SNR 集上 PESQ / SI-SNR 比 DFN3 高约 0.4 / 1.6 dB，比 GTCRN 高约 0.9 / 4.6 dB（arXiv 2512.16420v3 Table 1）；且没有 libDF 那条 −10 dB 静音分支
- **自然** ↑（待证）：训练加了防过度衰减的损失项（同论文 Sec 3），对应「背景变真空」痛点；没有我们自己的听感证据
- **轻快** ↓：算法延迟 60 ms，比 DFN3 _ll 的 20 ms、GTCRN 的 32 ms 都高——**这是选它付出的代价**，所以列为拍板 #1

---

## 拿两把尺子量一遍

### 一、复用分级

| 来源 | 权限 | 这次实际用到哪一步 |
|---|---|---|
| 自家的 | 可照搬 | 设计稿的预算与里程碑思路；exp001 的 bench 脚本可改成 Rust 版 |
| DPDFNet 权重（Apache-2.0） | 可直接分发，附 NOTICE | ONNX 权重原样加载 |
| DPDFNet / libDF / biglinux-gtcrn / nnnoiseless 代码 | 只读借鉴 | 读懂 STFT/OLA、state 初始化、逐帧循环的做法后**自己写**；不复制 |

⚠️ 交付前回参考区比对：本批只写了调研文档和一个 50 行的 Python bench（只调 onnxruntime 公共 API），没有从参考区搬代码。

### 二、停止线

> **能列出拍板点、且首选与备选各自的失败条件清楚，就停。** 不再继续找第六个候选：已查过 LiSenNet、Fast-ULCNet、FSPEN、aTENNuate 变体，都缺权重或官方仓。

## 支持它的最强依据

- 在作者自建的低 SNR 多语种测试集上，DPDFNet-2 / 4 的 PESQ 3.14 / 3.18 明显高于 DFN3 2.76、GTCRN 2.21（arXiv 2512.16420v3 Table 1）
- 状态显式的单帧 ONNX：exp001 直接回灌 state 跑通 6000 帧，dpdfnet2 p99 0.78 ms、max 1.15 ms（Apple M5）
- 代码与权重都是 Apache-2.0（`resources/DPDFNet/LICENSE`；HF 模型卡）；仓库活跃、维护者回 issue

## 反对它的最强依据

- **延迟**：输出比输入落后 4 帧（`onnx_model/layers.py:377-383`、`multiframe.py:221`、`infer_dpdfnet_onnx.py:18`），算法延迟 60 ms；按 DNS 实时赛道规则是 70 ms，远超 40 ms。零 lookahead 版本没有、训练代码也没开源，**我们自己压不下来**
- **效果证据全是作者自己的**：自建测试集、自定义 PRISM 指标，没有 babble 单列，也没有第三方复现
- **将来微调受限**：intent 写「只在评测矩阵指出弱项时针对性微调」，但 DPDFNet 训练代码未开源（issue #5、#10）；模型结构（`model/*.py`）和 PyTorch checkpoint 是公开的，要微调得自己写训练循环

## 三个风险

1. **延迟叠加后超出会议可接受范围**：60 ms 算法延迟 + CoreAudio 缓冲 + 推理线程交接，端到端可能到 80 ms 上下，对方会感到「慢半拍」。缓解：003 管线线先量端到端；GTCRN 用同一接口随时可切
2. **babble 下表现未知**：所有候选都没有 babble 数字。缓解：004 评测第一批真实咖啡馆录音里同时跑 DPDFNet-2 / GTCRN / nnnoiseless，人耳 AB 定
3. **Rust 外壳细节出错导致音质差但看不出来**：wnorm 要在图的输入和输出两头各补一次、state 要按元数据初始化、干湿混合要把原声延后 4 帧（sources.md DPDFNet「E」「I」）。缓解：先用官方 Python 推理（`infer_dpdfnet_onnx.py`）产出参考 WAV，Rust 输出与之逐样本对拍

## 仍未补齐的缺口

> **真实会议录音（尤其 babble）上，DPDFNet-2 与 GTCRN 的听感差多少**——这决定 60 ms 的延迟代价值不值。归 004 评测，不再在本线上搜。

## ⚠️ 需要你拍板的地方

| # | 需要拍板的 | 选项 | 代价 | 我的建议 | 默认（不拍时） | 拍了吗 |
|---|---|---|---|---|---|---|
| 1 | **命门 1 的延迟线**（决定首选是谁） | A 放宽到算法延迟 ≤ 70 ms（DNS 口径）→ 首选 DPDFNet · B 坚持 40 ms、按「窗 + lookahead」算 → 首选 GTCRN · C 坚持 40 ms、按 DNS 口径 → 三条全过的只有 nnnoiseless（DFN3 _ll 卡在权重许可） | A：「轻快」变差，换「清楚」· B：效果天花板低 · C：效果最弱 | **A**，但仅作 M1–M2 的起点；GTCRN 同接口留在评测里对照，004 拿到真实录音听感后再定 | A | ☐ |
| 2 | 采样率 | A 16 kHz（dpdfnet2）· B 48 kHz 全频带（dpdfnet2_48khz_hr，本机 p99 1.12 ms） | A：要重采样、高频没了，但效果证据覆盖它 · B：免重采样、更「自然」，但论文低 SNR 表没覆盖它 | A 起步，B 直接放进 004 评测比 | A | ☐ |
| 3 | 型号大小 | A dpdfnet2（p99 0.78 ms）· B dpdfnet4（p99 1.26 ms，设计稿原推荐） | B 只比 A 高 PESQ 0.04，在 M1/M2 老机器上余量更小 | A | A | ☐ |
| 4 | 设计稿 §16 的「M1 接入 DFN3（libDF 原样跑）」还做不做 | A 跳过，M1 直接上 DPDFNet（ort）· B 照做 | B 要编译 libDF、引入权重许可不明的模型，而且它会被替换 | A；DFN3 只在评测里当参照 | A | ☐ |
| 5 | `must-know.md` 的三条命门本身 | 改 / 不改 | 改了会改变上面 1–4 的结论 | 至少把延迟用哪种口径写进命门 1 | 不改 | ☐ |

---

## 收尾三件

1. **判断落回真相源**：等拍板后，把确认的结论整理进 `vault/truth/`（本线不写，由大总管安排）
2. **交付前回一趟参考区**：已查，见「复用分级」
3. **放行自检**：选了什么——DPDFNet-2 16k + ort + 自写 Rust 外壳；舍了什么——libDF 原样跑 DFN3；哪里要拍板——上表 5 条，#1 最要紧
