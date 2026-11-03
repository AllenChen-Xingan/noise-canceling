# 019 · 降噪模型在 Mac 上用什么推理后端（总方案）

> 2026-11-02 · applebe（苹果引擎）汇总 rustbe（纯 Rust）· 状态：**草稿**，第 4 节等「实时线程策略」实测补完；对抗自审未做。
> 判据：`standard.md`（16 个 verifier；技术线【AI 定；人可改】，整机 CPU 两档与最低系统【已定】）。数字一律本机实测（Apple M5 · macOS 27.0.1 · 插电），从 Rust 调（D68）；来源路径写在每条后面。
> 意图：D53「必须达到最好的效果，最快的效果」· D69 整机 CPU 目标 ≤ 1%、上限 ≤ 2% 个核 · D70「完全榨干硬件的性能，以及非常低能耗……非常清亮（轻量）」。

## 1. 先看风险（大总管要求放在第一节）

1. **按实时节奏跑，每帧慢约 5 倍。** 苹果最快的组合（BNNS Graph + GRU 展开）连续跑 p50 0.155 ms；按 16 ms 一帧跑 10 分钟，p50 0.82 ms、最大 11.8 ms、占 4.8% 个核（`exp010…/apple/results/20261102-173000-b2/summary.md`）。连续跑的数都低估了实时开销：rustbe 在手写版上复现了同一现象（连续 0.088 ms → 实时节奏 p50 0.61 ms、最大 6.95 ms、10 分钟里 6 帧超 3 ms，`notes/rustbe-numbers.md`），所以这不是苹果引擎特有的。节奏模式下手写 + Accelerate 版（p50 0.44、最大 4.17、1 帧超 3 ms）反比纯手写快约 21%。推断是帧间降频或被挪到能效核；实时线程策略的对照测法已写好（`apple-rs/tools/rt-matrix.sh`），**测完之前，所有「占百分之几个核」都只是下界**。
2. **CPU 总账已经很紧（D69）。** 用手写 Rust 方案，已经量过的部件合计 ≥ 1.95% 个核（降噪 0.63 · 认人 1.2 · VAD 0.05 · 场景 0.07），离 2% 上限只剩 0.05%；STFT、重采样、采集输出、修音四项还没量（`standard.md` §6.1）。再加上第 1 条的 5 倍效应，现在**任何方案都过不了 D69**，除非实时线程策略能把每帧拉回连续跑的水平。
3. **认人判断回路单项 ≈ 1.2% 个核**，超上限分摊 4 倍，不归 019（已转交大总管）。
4. **FE-M 在 ORT 上 ≈ 3.7% 个核**（连续跑），超上限；而且它转不了 Core ML / BNNS（见第 3 节）。它是否还留在候选里，待人定（rustbe 待拍板）。

5. **口径变了（D74 · D75）。** D74 让降噪线程接受低频（停在能效核），所以 CPU% 会变高、耗电却可能更低；D75 据此把挡线从 CPU% 改为「对前台性能的影响」（`BE-IMPACT`，≤ 1% 目标 / ≤ 2% 挡线）和耗电（`BE-POWER`，主排序）。第 1、2 条里的 CPU% 只作参考，过不过以 `BE-IMPACT` 和耗电为准，这两项还没测。

## 2. 结论（每个要上用户电脑的模型用什么后端）

| 模型 | 建议后端 | 为什么 | 状态 |
|---|---|---|---|
| **GTCRN（主模型）** | **手写 Rust + Accelerate**（rustbe 的 `native-blas`，线程 tc 开），苹果侧备选 **BNNS Graph + GRU 展开** | 手写 p50 0.088 ms，比 ORT 快约 4.4 倍，0 分配，134 dB；BNNS 展开版 0.155 ms、快约 2.5 倍，0 分配，134 dB，是苹果引擎里唯一过 `BE-RT-ALLOC` 的。两者都能在推理线程直接调 | 按 D75 主排序（耗电）暂排第一：71 mW，BNNS 97 mW（单次、噪声边缘，待重复）；`BE-IMPACT` 待重测 |
| FE-M（备选） | 暂无（继续 ORT） | 转不了 Core ML（图里有 `view_as_complex`）；手写 Rust 本轮不做；ORT 超 D69 上限 | 待人定是否留在候选里 |
| DFN3 `_ll`（015 候选） | **不进实时链**；下一轮盲听用**离线处理** | 算力下界（三图 S=1、单线程、连续跑）0.61 ms / 10 ms 帧 ≈ 6.1% 个核，是推理挡线 1.0% 的 6 倍；苹果最快后端提速 2.5 倍也约 2.4%（`dfn3ll_probe.json`）。流式重导出（把 GRU 状态做成输入输出）约 1 天，不做【AI 定，大总管同意】 | 已定 |
| 认人 ECAPA-TDNN512 | 待测（ORT vs BNNS，名额里一起跑，`apple-rs/tools/ecapa.sh`） | 每 0.5 s 一次、约 6 ms（含 fbank 与打分），单项 ≈ 1.2% 个核（exp005），D69 下要降约 4–8 倍 | 待测 |
| 场景 MLP · VAD | 不换 | 已是纯 Rust 零分配，占用在分摊内（`standard.md` §6.1） | — |
| 本机评分 / 调节模型（D52） | 推理另定；**本机微调用 MLX** | MLX 唯一有自动求导（`notes/mlx.md` §5），不在实时回路，每帧分配多不要紧 | 等 018 定结构 |

**苹果引擎各条路的判定**（`BE-EXHAUST` 矩阵见 `standard.md` §7.1）：
- Core ML：所有算子都落在 CPU（`MLComputePlan`，三种计算单元设置都一样），GPU / 神经引擎对 GTCRN 用不上；每帧约 300 次 malloc，过不了 `BE-RT-ALLOC`；fp16 只有 54 dB，过不了 `BE-PARITY`。**不用**。
- BNNS Graph：见上，**苹果侧唯一可用**。代价：要 macOS 15，而最低系统已定为 14.4（D73）——`BE-OS` 冲突，待拍板 2-8，见第 6 节。
- MLX：Python 调 5.7–6.2 ms（ORT 的 15 倍慢），每帧 7–10 万次分配。**不做实时推理**，只做本机微调。
- 神经引擎 / GPU：对这么小、带 GRU 的逐帧模型没有收益；Metal 手写核不写（每帧派发的固定开销已大于手写 Rust 整帧，`notes/mlx.md` §2）。

## 3. 绕过了哪些坑（复现时必看）

- coremltools 9.0 把**分组 ConvTranspose2d（groups=2）转错**：第 0 帧就差 0.77。逐块对拍定位到解码器第 4 层（差 3.46），拆成每组一个再拼接后对 ORT 差 2.4e-7（`apple/convert_gtcrn.py` 的 `SplitConvT`）。**以后任何经 coremltools 的模型都要逐块对拍。**
- coremltools 在 Python 3.13 / numpy 2.5 上会崩或报错：转换用单独的 3.12 venv（`_tmp/exp010-venv-ct`）加 `apple/ct_compat.py`。
- GRU 要在 PyTorch 里先手动展开（`UnrolledGRU`）：coremltools 会把 GRU 翻成逐步循环，展开后 BNNS 快 2.2 倍、Core ML 快 1.9 倍，输出不变（对 torch 差 3.6e-7）。
- 有状态模型（`MLState`）coremltools 转不出来（`No matching select or slice.`）；缓存走显式输入输出。

## 4. 实时线程能不能直接调 · 线程设置 · 耗电

**直接调**：手写 Rust、BNNS Graph 每帧 0 次分配（全进程 malloc 钩子，自检通过），可以在推理线程直接调；RTSan 豁免表还没建（`BE-RT-SAN` 待测）。Core ML 每帧约 300 次 malloc（ORT 1557 次），只能登记豁免，不建议用。

**线程设置 × 耗电**（人用 sudo 跑，`exp010…/apple/results/20261102-183322-rt/summary.md`；单次 60 s，噪声约 ±30–40 mW）：

| 设置 | p50 | max | 核占用 | CPU 功率 |
|---|---|---|---|---|
| BNNS · rt（D74 默认） | 0.80 ms | 1.41 | 4.8% | 97 mW |
| BNNS · rt + 自建工作组区间（截止 1–3 ms） | 0.19–0.22 ms | 0.27–0.33 | 1.3–1.5% | 123–138 mW |
| BNNS · 帧间忙等（只排查） | 0.16 ms | 0.61 | 99.9% | 4199 mW |
| 手写 Rust · rt | 0.60 ms | 0.66 | 3.5% | 81 mW |
| 手写 Rust · ui | 0.60 ms | 2.69 | 3.5% | 104 mW |
| **手写 + Accelerate · rt** | 0.43 ms | 0.56 | 2.6% | **71 mW** |

结论（强度：单次、噪声大）：
1. **「实时节奏慢 5 倍」是帧间降频**：忙等把 p50 拉回连续跑的水平，但耗电是 30–40 倍。产品里不能忙等。
2. **自建工作组区间（`AudioWorkIntervalCreate` + 每帧 start / finish）能把速度拉回 4 倍、核占用降到约 1.4%，但功率高 25–40 mW。** 提速要花电，所以**没有推翻 D74**（接受低频、工作组默认不加）。如果以后要压 `BE-IMPACT` 或尾部，它是现成的旋钮。
3. **目前功率最低的是手写 + Accelerate · rt（71 mW）**，其次手写 · rt（81 mW）、BNNS · rt（97 mW）。差值在噪声边缘，`BE-POWER` 要交替重复 3 轮以上才能定排名（`standard.md` 已注明）。
4. 所有设置 10 分钟 / 60 s 内超 3 ms 的帧都是 0（`BE-DEADLINE` 这一段都过；第 1 条风险里 BNNS 那次 11.8 ms 是在 qos ui 下）。

## 5. 怎么接进代码（接口建议；结构细节归 Rust 骨架线）

依据 D71（人定：频谱前端每帧只算一次、各模型共用，代码复杂度低）。下面只写推理后端这一侧要满足什么，crate 怎么拆、谁持有前端，由另一位大总管的 Rust 骨架线定。

1. **后端不做 STFT。** 前端每 hop 算一次频谱（sqrt-Hann 512 / 256，`vdn_dsp::StreamingStft`），GTCRN、修音链频谱节点、场景特征都吃这一份。后端的输入是「已算好的一帧频谱」（GTCRN：re[257] + im[257]），输出是增强后的频谱或掩码。本次所有基准正是这个口径：只计推理调用，不含 STFT（`standard.md` §3）。
2. **会自带 STFT 的模型要换成频谱进、频谱出的版本。** FE-M 今天用的 wav2wav 图把 STFT / iSTFT 包在图里（`notes/our-models.md` 第 2 行），违背 D71；它恰好也是转不了 Core ML 的原因（`view_as_complex`）。FE-M 若留在候选里，应换官方 spec2spec 版（`fastenhancer_m.spec.onnx`，009 decision:73），由共用前端喂频谱。
3. **认人不吃这份频谱**：ECAPA 吃的是 80 维 fbank（25 ms 窗、10 ms 移，Kaldi 口径），窗长、移步都和降噪的 STFT 不同，不能直接共用；它在判断线程里自己算 fbank（`src/speaker/src/fbank.rs`）。要不要从共用前端派生 fbank，交 Rust 骨架线判断。
4. **后端 trait 最小面（建议）**：
   - 非实时线程：`open(model) -> Self`，并预热；
   - 实时线程：`run(&mut self, input: &[f32], output: &mut [f32])`，零分配、不加锁；缓存由后端自己持有（A/B 两组换指针，`apple-rs/src/bnns.rs`）；
   - `reset()`：缓存清零（换模型、静音过长时用）。
   这和规约里的 `Model` 接口（一个 hop 进、一个 hop 出）的差别只在「进出的是频谱，不是波形」。
5. **实现落点（建议）**：手写 GTCRN 直接作为一个模型实现，不经推理库；ORT 留给 FE-M / 认人（`vdn-backend-ort`）；BNNS Graph 作苹果侧备选（`vdn-backend-bnns`，C FFI，链接名带 `_v2`）。BNNS 吃 `.mlmodelc`，由 `apple/convert_gtcrn.py` 加 `compile` 子命令生成。
6. RTSan 豁免表、`no_alloc` 测试要按新后端重建（`notes/our-models.md` 对 vdn 的含义第 5 条）。

## 6. 对延迟账的影响

- 延迟账里没有单列推理耗时，它被「跨进程环余量 3.00 ms」盖住（`docs/research/009_rust-ecosystem/decision.md:62`）。模型带来的 32 ms 是 STFT 窗长决定的算法延迟，换后端改不了。
- 所以换后端**不减延迟账**；收益在 CPU / 耗电（D69 · D70）和尾部。注意第 1 条风险：实时节奏下 BNNS 最大一帧 11.8 ms，已经超过 3 ms 余量，**按现状会出现送原声的帧**。
- 最低系统（待拍板 2-8）：只有选 BNNS 才需要 macOS 15。如果手写 Rust 是主路径，这题可以作废（手写版没有系统版本要求）。

## 7. 还没做的

- 实时线程策略对照（要名额，约 18 分钟）· `BE-POWER` 耗电（要人跑 sudo）· `BE-RT-SAN` · `BE-SIZE` 实测（ORT 静态库增量）· 手写 Rust 与 ORT 按实时节奏重测（rustbe 在跑）· 对抗自审一轮。
