# 两类发现 · 起步用哪个降噪模型与代码作为基线

> 2026-10-29 · 手下 model。依据全在 `sources.md`（带文件:行号）与 `docs/experiments/exp001-model-frame-time/`。
> 分级：P0 最高、P3 最低，**P0 / P1 / P2 清零，P3 酌情**。

## 先说一个口径问题（影响命门 1 的判定）

命门 1 草稿（`must-know.md:17`）写的是「窗长 + lookahead ≤ 40 ms」；但 40 ms 这个数出自 DNS Challenge，而 DNS 的算法多算一个步长。两种算法：

| 口径 | 怎么算 | 出处 |
|---|---|---|
| 窗 + lookahead | DFN 论文的算法 | arXiv 2305.08227 §3 |
| **窗 + hop + lookahead** | DNS Challenge 实时赛道的规则 | `resources/DNS-Challenge/docs/ICASSP_2021_DNS_challenge.pdf` 第 2 节 |

下表两种口径都列；端到端附加延迟（加管线）见 `decision.md`「延迟：三种算法分开列」。**40 ms 是 AI 起草的门槛，人还没定**——上限定几档是 decision.md 拍板 #1。

## 候选之间（同一批核查项，逐格对照）

| 核查项 | DPDFNet（16k：2 / 4；48k：2_hr） | DFN3 默认 | DFN3 _ll | GTCRN | UL-UNAS | nnnoiseless（垫底） |
|---|---|---|---|---|---|---|
| **命门 1a 因果** | 原生 ✅ | 原生 ✅ | 原生 ✅ | 原生 ✅ | 原生 ✅ | 原生 ✅ |
| **命门 1b 延迟（窗+LA / DNS 口径）** | **60 / 70 ms ❌**（前瞻 `df_lookahead=2`、`conv_lookahead=2`，`onnx_model/dpdfnet.py:547-548`；`mask_method='before_df'` 让两段串联成 4 帧，`:783-786`；官方脚本按 4 帧对齐，`infer_dpdfnet_onnx.py:18`） | 40 / 50 ms（压线 / ❌；两段前瞻并联取较大值，`libDF/src/tract.rs:309-316`） | **20 / 30 ms ✅** | 32 / 48 ms（✅ / ❌） | 32 / 48 ms（✅ / ❌） | 20 / 30 ms ✅ |
| **命门 1c 每 10 ms ≤ 2 ms（本机 M5，模型 + STFT 合计 max）** | dpdfnet2 0.89 · dpdfnet4 1.40 · 2_hr 1.25 ✅（dpdfnet8 2.52 ❌） | 官方 RTF 0.19（i5）≈ 1.9 ms，本机**未测** | **未测**，模型约为默认版 4.5 倍（按文件大小估） | 0.62 ms / 16 ms 帧 ✅ | 未测，算力与 GTCRN 同级（35 MMACs/s） | 未测，算力极低 |
| **命门 2 代码许可** | Apache-2.0 ✅ | MIT/Apache ✅ | 同左 ✅ | MIT ✅ | MIT ✅ | BSD-3 ✅ |
| **命门 2 权重许可** | Apache-2.0 ✅（模型卡） | **未写明** ❌（#697 #712 无人回） | 同左 ❌ | MIT ✅（作者 #104 书面确认） | 部分：仓库 MIT，无专门声明；训练数据含 DiDiSpeech 许可未核 | 部分：推断 BSD-3 |
| **命门 3 Rust 逐帧** | 部分：ONNX 状态显式 ✅，Rust 外壳要自写（图内已处理 wnorm，自写 STFT 直接喂未缩放 rfft） | 原生 ✅（libDF + tract），但非实时安全（每帧深拷贝 `tract.rs:541-542`） | 同左 | 原生 ✅：ONNX 状态显式 + biglinux Rust/ort 实现 | 部分：ONNX 状态显式，无 Rust 代码 | 原生 ✅ 纯 Rust |
| **排序依据：低 SNR 证据** | 最好：作者低 SNR 集 PESQ 3.14（2）/ 3.18（4）；无 babble 单列 | 同集 2.76；**lsnr < −10 dB 整帧静音**（`tract.rs:176-185`） | 无证据 | 同集 2.21 | 无低 SNR 证据；DNS3 合成集比 GTCRN 好 | 无 |
| 维护 | 活跃 | **停更**（2024-10） | 停更 | 低频活跃 | 低频 | 低频 |
| 采样率 | 16k / 48k 全频带 | 48k | 48k | 16k | 16k | 48k |

**一眼看下来**：按命门草稿原样（窗 + 前瞻 ≤ 40 ms），**三条全过的只有 GTCRN**（nnnoiseless 也过，但只作垫底基线）。按 DNS 口径则 GTCRN 也超（48 ms），只剩 nnnoiseless。

- **DPDFNet**：许可、算力、逐帧接口、效果证据都最好，**唯独延迟超线**。
- **DFN3 _ll**：延迟最好，但**权重许可不明、上游停更、算力没测**。
- **GTCRN / UL-UNAS**：许可清楚（GTCRN）、算力极低、逐帧接口现成，但**效果天花板低**（作者低 SNR 集 PESQ 2.21 vs DPDFNet-2 3.14）。

## 一、它们做到的 —— 以暂定首选 DPDFNet 为对象逐项核对

| # | 不可妥协的那条 | 状态 | 差距是什么 | 修起来要什么 | 分级 |
|---|---|---|---|---|---|
| 1a | 因果 | 已实现 | — | — | — |
| 1b | 延迟 ≤ 40 ms（草稿） | **偏离** | 内部 mask 延 2 帧 + DF 再延 2 帧，算法延迟 60 ms（窗+LA）/ 70 ms（DNS） | **能力本身没有**：要零 lookahead 权重，只能等 Ceva（#14 说「possible」）或自己重训（训练代码未开源）→ 要立项；或者**人放宽上限**（decision.md 拍板 #1） | **P0**（不解决就不能当首选；不放宽则首选换 GTCRN）|
| 1c | 每帧 ≤ 2 ms | 已实现（dpdfnet2/4，M5 实测，含 STFT） | 只测了 M5 与 Python 版 ORT，没测老机器与 Rust `ort` | 接一根线：Rust 版 bench，在 M1/M2 上复测 | P2 |
| 2 | 许可证 | 已实现 | 训练数据未披露 | 发布时附 NOTICE、在文档里写明「训练数据未披露」 | P3 |
| 3 | Rust 逐帧 | 部分（缺 Rust 外壳） | 官方仓无 Rust；要自写 STFT/OLA + state 初始化 + 4 帧对齐 | **能力已有，只是没接上**：ort Session + 自写 vorbis STFT/OLA（喂未缩放 rfft，irfft ÷N 后乘窗）+ 按元数据初始化 state + 逐帧循环；对拍基准用官方 `StreamEnhancer`（`package/src/dpdfnet/stream.py:117-156`） | P1 |
| — | 实时安全 | 部分 | ort 普通 `run` 每帧分配输出 | 推理放独立线程 + IoBinding 预分配，`assert_no_alloc` 实测 | P1 |

## 二、它们没做的

| 找过什么 | 结论 | 是哪一种 |
|---|---|---|
| babble（人声噪音）与分 SNR 档的效果数字 | **无人涉足**：DPDFNet、DFN3、GTCRN、UL-UNAS 论文和仓库都没有（DPDFNet 论文全文不出现 babble） | **没顾上做** → 机会，也是 004 评测必须补的格子。注意：通用模型本来就会保留邻桌人声（设计稿 §1），babble 的解法是说话人门控（M3），不是换主模型 |
| 零 lookahead、≤ 40 ms 的 DPDFNet 权重 | 没有现成的；维护者说可以做（#14），训练代码未开源（#5/#10） | 没顾上做（对我们是「做不到」：没训练代码） |
| DFN3 权重的商用许可 | 两个 issue 问了几个月，维护者没回 | 不确定（上游停更） |
| 端到端延迟总账（算法 + CoreAudio 缓冲 + 重采样 + 推理线程交接） | 各候选只给算法延迟 | 没顾上做 → 003 管线线 / 下一批原型来量 |
| DFN3 _ll 在 M 系列上的每帧耗时 | 没有官方数字，ONNX 状态不显式，exp001 没法直接测 | 没顾上做 → 要编译 libDF（重活） |
| 一个「因果 + 20/10 ms 帧 + 零 LA + 权重宽松 + 有流式 ONNX」的模型 | 查过 LiSenNet、Fast-ULCNet、FSPEN、aTENNuate 无 PreConv 版：都缺权重、缺官方仓或效果掉太多 | 做到的人少：这正是 DNS 实时赛道标准，但开源权重里只有 DFN3 _ll 和 nnnoiseless 达到 |

### 该不该自己跑一个

| 判据 | 结论 |
|---|---|
| DPDFNet 的 4 帧延迟 | **不用跑**：源码 + 官方对齐常数 + 第三方登记三方一致。可选：Rust 外壳写好后用脉冲实测一次端到端 |
| 低 SNR / babble 下谁更好 | **跑它**——输入（真实咖啡馆录音）不清楚，正是实验的场合；归 004 评测，同一批真实录音过 DPDFNet-2 / GTCRN / DFN3 / nnnoiseless |
| DPDFNet-2 比 GTCRN 多出的约 30 ms 延迟值不值 | **跑它，而且最便宜**：两边都有现成 Python 流式推理（`DPDFNet/package/src/dpdfnet/stream.py`、`gtcrn/stream/gtcrn_stream.py`），几段咖啡馆录音离线过一遍让人 AB 听，不用先写 engine（decision.md 拍板 #1 选项 D） |
| DFN3 _ll 能不能 2 ms 跑完 | **跑它，但只在人选「坚持 40 ms」时才跑**——不改变取舍的实验不值得跑 |
