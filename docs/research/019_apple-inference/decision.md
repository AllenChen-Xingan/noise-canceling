# 019 · 降噪模型在 Mac 上用什么推理后端（总方案）

> 2026-11-02 · applebe（苹果引擎）汇总 rustbe（纯 Rust）· 状态：**草稿 v4**——同模型 3 视角只读自审一轮、codex 三轮（全部属实已改，见 `docs/reviews/019_apple-inference-codex.md`）后修订。
> 判据：`standard.md`（17 个 verifier）。技术线【AI 定；人可改】；人定的：D75 判什么（对前台性能的影响 + 耗电主排序；`BE-IMPACT` 的 1% / 2% 两个数是 AI 对 D75 的理解，待人确认）、D78 最低 macOS 15（原误记为 D77）、D70 榨干硬件 · 低能耗 · 轻量。
> 数字一律本机实测（Apple M5 · macOS 27.0.1 · 插电），从 Rust 调（D68）；来源路径写在每条后面。**全部只在这一台机器上测过。**

## 1. 先看风险（大总管要求放在第一节）

1. **按实时节奏（每 16 ms 一帧）跑，每帧慢 5–7 倍；原因推断是帧间降频【假设】**（忙等也同时改变了核迁移、缓存热度和睡眠唤醒，还没分开测）。 连续跑：BNNS + GRU 展开 p50 0.155 ms，纯手写 0.088 ms。按节奏跑：BNNS 0.80 ms（rt）/ 0.82 ms（ui），纯手写 0.60–0.61 ms。帧间忙等能把 p50 拉回 0.16 ms，但 CPU 功率 4199 mW，是其他设置的 31–59 倍（`apple/results/20261102-183322-rt/summary.md`、`notes/rustbe-numbers.md`）。**所有连续跑的数都只能比相对快慢，不能当实际开销。**
2. **CPU% 已远超 D69 的两档，但 D75 已把它降为参考。** 按实时节奏，纯手写 · rt 推理本身就占 3.5% 个核；加上认人 1.2%、VAD 0.05%、场景 0.07%，已知部分合计 ≥ 4.8%（`standard.md` §6.1）。D75 改用「对前台性能的影响」（`BE-IMPACT`）和耗电判，**这两项还没有可用的数**（见第 5 条）。
3. **认人判断回路单项约 1.2% 个核**（exp005，连续跑口径），不归 019，已转交大总管。苹果引擎帮不上：ECAPA 转不出 Core ML（第 2 节）。
4. **FE-M 只有连续跑的数**（ORT p50 0.38 ms；按连续跑推算约 3.7% 个核，实时节奏已证明这种推算不可靠）。按 D75，要先补测它在 ORT 上按实时节奏跑的耗电和对前台的影响，才说得出要不要换后端。它转不了 Core ML（图里有 `view_as_complex`），手写本轮不做。是否留在候选里，待人定（rustbe 待拍板）。
5. **测量本身的强度不够**：
   - 只有一台机器（M5 · macOS 27）；D78 定的最低 macOS 15、老芯片（M1）都没测。
   - 耗电只跑了 1 轮 60 s，背景噪声约 ±30–40 mW，比后端之间的差（10–26 mW）还大；空闲基线（111 mW）甚至高于几段运行。所以**只能比相对高低，排名靠不住**。v2 测法（3 轮轮换、每轮各量空闲、只平均计时窗内样本）已写好，待名额（`apple-rs/tools/rt-matrix.sh`、`power_parse.py`）。
   - `BE-IMPACT` 第一批数据作废：applebe 的脚本留下 21 个孤儿降噪进程，「关」组其实也开着降噪。已修，v2 测法待名额（`apple-rs/tools/impact.sh`）。
   - 基准程序里的线程设置，不一定等于产品里实际线程的设置。
   - Core ML 那几行是挂着 malloc 计数钩子计时的（每次分配多一次回调），对它略不利；之后计数与计时已分开（`--count-malloc`）。

## 2. 结论（每个要上用户电脑的模型用什么后端）

> 手写版耗电暂不进排名，待 bench 补计时窗字段（`win_start_unix` / `win_end_unix` / `wall_s`，已转交大总管，派给「下界分析」手下）。

**GTCRN（主模型）——暂排第一：纯手写 Rust（`native`），线程 time-constraint 开（D74）。** 这是大总管定的技术默认（2026-11-02；人可改）。按 `standard.md` §6.2，挡线还没测全，所以是「暂排」不是「已定」：

| 挡线 | 纯手写 · rt | 依据 |
|---|---|---|
| `BE-PARITY` | **过**：对 ORT 134.0 dB，无 first_bad | rustbe r4 |
| `BE-RT-ALLOC` | **过（整数 0）**：rustbe `results/20261102-alloc6k/malloc.jsonl`，连续跑 6000 帧，`process_allocs_total` = 0、`rust_allocs_total` = 0（同场 ORT 6000 帧共 9 342 000 次，合每帧 1557）。按实时节奏 · rt 的 300 帧另有一例也是 0（`verify1` 的 `bench_short_paced_rt`）。`blas1` 里的 0.000 只能说明 ≤ 1 次，不引 | rustbe |
| `BE-FRAME`（p99.9 ≤ 1.0 · max ≤ 2.0） | **过**（60 s）：p99.9 0.65 · max 0.66 ms | build 20261102-183322-rt |
| `BE-DEADLINE`（10 分钟 0 帧**迟到** > 3 ms） | **未测**：rt 下没跑过 10 分钟；ui 下 10 分钟有 6 帧调用耗时 > 3 ms（已不过）；「迟到」口径（含唤醒迟到）是 codex 第 1 轮后才加的 | `notes/rustbe-numbers.md` |
| `BE-IMPACT` | **未测**（第一批作废） | — |
| `BE-COLD` · `BE-SIZE` · `BE-MEM` · `BE-RT-SAN` | **未测**（`BE-OS`：纯手写无系统要求，过） | — |
| `BE-POWER`（主排序 + 「不高于 ORT」挡线） | CPU 功率 81 mW（单次、噪声里）；那批没跑 ORT，「不高于 ORT」**未判**；v2 测法已加同场 ORT | build 20261102-183322-rt |

为什么排它：
- 连续跑最快：p50 0.088 ms，同场 ORT 0.384 ms，约 4.4 倍（build 20261102-173000-b2；按 rustbe r4 的 0.10 / 0.38 算是 3.8 倍）。
- 每帧 0 次分配（整数，6000 帧，见上表）；不依赖任何推理库，体积最小（待 `BE-SIZE` 证实）；常驻内存预热后只涨 0.61–1.09 MB（rustbe `verify1` 与 `blas1`，各一次）。

**手写 + Accelerate：cblas / vDSP 路线已不采用；BNNS 路线待证实**（大总管定，AI 定，2026-11-02）。cblas 版（`native-blas`）实时节奏下 p50 0.43 ms、CPU 功率 71 mW（单次、噪声里），但每帧 11 次分配，rustbe 已查到出处：`cblas_sgemm → APL_sgemm_QR → posix_memalign`；换成 `vDSP_mmul` 也是每帧 11 次，原先的翻盘条件②做不到，所以这条路线不再采用。手写网络里改用 BNNS 算子的路线还没做，待证实。

**苹果侧备选（候选）：BNNS Graph + GRU 展开。** 连续跑 p50 0.155 ms（ORT 的 2.5 倍快），133.9 dB；分配显示 0.000 / 帧（旧 build 只有三位小数，`BE-RT-ALLOC` 待整数验证）。但在 D74 默认（rt）下按节奏跑 p99.9 1.34 ms，**过不了 `BE-FRAME`**（≤ 1.0）；加自建工作组区间后 p99.9 0.23–0.28 ms 能过；那一次 CPU 功率高 25–40 mW，但只是单次观察、差值与噪声同量级，省不省电要等交替重复测（codex 第 3 轮 #2）。

其他模型：

| 模型 | 建议 | 依据 | 状态 |
|---|---|---|---|
| FE-M（备选） | 暂无（继续 ORT） | 转不了 Core ML；手写本轮不做；实时节奏下的耗电没测，说不出必须换后端 | 先补测，再定；是否留在候选里待人定 |
| DFN3 `_ll`（015 候选） | **实时链暂缓（时间盒）**；下一轮盲听用离线处理 | 只有一个算力下界：三张子图 S=1、单线程、连续跑合计 0.61 ms / 10 ms 帧（Python 调 ORT，没有这个模型的 Rust 差值校准，`apple/results/20261102-173000-b2/dfn3ll_probe.json`）。按 D75，CPU% 不挡，这个数**不足以淘汰它**；要淘汰得补它的 Rust 实时测量（耗电 · `BE-IMPACT`），前提是先做流式重导出（约 1 天）。本轮按时间盒不做（codex 第 1 轮 #5） | 暂缓（AI，大总管同意）；离线盲听照做 |
| 认人 ECAPA-TDNN512 | 不换后端；先降频率（只在有人声时跑等） | 苹果路转不出来：onnx2torch 补两处后，coremltools 不支持 `prod` 算子（build 20261102-181700-ecapa） | 转交大总管 |
| 场景 MLP · VAD | 不换 | 已是纯 Rust 零分配 | — |
| 本机评分 / 调节模型（D52） | 推理另定；**本机微调用 MLX** | MLX 唯一有自动求导（`notes/mlx.md` §5），不在实时回路 | 等 018 定结构 |

**苹果引擎各条路的判定**（`BE-EXHAUST` 矩阵见 `standard.md` §7.1）：
- **Core ML：不用。** cpu / gpu / ane 三种设置下，被指派了计算设备的算子全部落在 CPU（`MLComputePlan`；fp32 版 322 个在 CPU、0 个在 GPU / 神经引擎，另有 1785 个没有设备归属——`deviceUsage` 返回空，多为常量这类不在设备上执行的节点，采集代码见 `apple/bench_coreml.swift:112–113`），GPU 和神经引擎用不上；每帧约 300 次 malloc；fp16 只有 54 dB，过不了 `BE-PARITY`。
- **BNNS Graph：苹果侧的候选**（见上；`BE-FRAME` 在 rt 下不过、`BE-RT-ALLOC` 待整数验证）。要 macOS 15，在 D78 之内。
- **MLX：实时推理暂缓（时间盒），本机微调用它。** 暂缓的依据是执行机制：每帧建图 + `eval` 要分配、加锁、等后台线程（`notes/mlx.md` §3，读源码），和推理线程的规约冲突。Python 调 5.7–6.2 ms 没有同模型 Rust 对照，按 D68 不能当淘汰理由（codex 第 2 轮 #3）。
- **神经引擎 / GPU（经 Core ML）：没用上。** 被指派了设备的算子全部落在 CPU（`MLComputePlan`，实测；另有一批没有设备归属的常量类节点，见上）。**Metal 手写核：时间盒暂缓，无收益未验证**——原先「派发开销大于手写整帧」的比较不成立（异机数据，又和连续跑比），要判得在本机测一条逐帧融合的 Metal 路径（codex 第 2 轮 #4）。

## 3. 绕过了哪些坑（复现时必看）

- **coremltools 9.0 会把 GTCRN 转错**，第 0 帧对 ORT 差 0.77。原因有两层：先把缓存的原地切片写回改成拼接，差到 0.49；再逐块对拍，定位到解码器第 4 层的分组 ConvTranspose2d（groups=2，单块差 3.46），拆成每组一个再拼接后，对 ORT 差 2.4e-7（`apple/convert_gtcrn.py` 的 `SplitConvT` 与原地写回替换）。**以后任何经 coremltools 的模型都要逐块对拍。**
- coremltools 9.0 在 Python 3.13 / numpy 2.5 上会崩或报错：转换用单独的 3.12 venv（主目录 `_tmp/exp010-venv-ct`）加 `apple/ct_compat.py`（只把单元素数组压成标量，数值不变）。
- GRU 要在 PyTorch 里先手动展开（`UnrolledGRU`）：coremltools 会把 GRU 翻成逐步循环。展开后 BNNS 快 2.2 倍、Core ML 快 1.9 倍，对 torch 原版差 3.6e-7。
- 有状态模型（`MLState`）coremltools 转不出来（`No matching select or slice.`）；缓存走显式输入输出。

## 4. 实时线程 · 线程设置 · 耗电

**能不能直接调（候选接法，未确认）**：按 `standard.md` §4，要分配（`BE-RT-ALLOC`）、RTSan（`BE-RT-SAN`）、跨进程（`BE-RT-XPROC`）三项都过才能确认「可以在推理线程直接调」。现在只有分配这一项有数：BNNS Graph 显示 0.000 / 帧（applebe 全进程钩子，自检通过；旧 build 只有三位小数，整数待重测），纯手写整数 0（rustbe `alloc6k`，6000 帧）。RTSan、跨进程都还没测，零分配不能证明不加锁、不阻塞（codex 第 4 轮 #1）。Core ML 每帧约 300 次 malloc（ORT 1557 次，rustbe 实测），只能登记豁免。自建工作组区间的那几段每帧有约 0.012 次 malloc（区间 start / finish 偶发分配，未查是哪一个），严格说也过不了 `BE-RT-ALLOC`。

**线程设置 × 耗电**（人用 sudo 跑，只有 powermetrics 是 root；`apple/results/20261102-183322-rt/summary.md`）。⚠️ 单次 60 s；空闲基线 CPU 111 mW，比好几行都高，所以下表给的是**绝对 CPU 功率**，只能比相对高低；差 < 30 mW 的不算有差别：

| 设置 | p50 | p99.9 | max | 核占用 | CPU 功率 |
|---|---|---|---|---|---|
| BNNS · rt（D74 默认） | 0.80 ms | 1.34 | 1.41 | 4.8% | 97 mW |
| BNNS · rt + 自建工作组区间（截止 1–3 ms） | 0.19–0.22 ms | 0.23–0.28 | 0.27–0.33 | 1.3–1.5% | 123–138 mW |
| BNNS · rt，只把截止报紧到 1 ms | 0.81 ms | 1.25 | 1.30 | 4.8% | 124 mW |
| BNNS · 帧间忙等（只排查） | 0.16 ms | 0.42 | 0.61 | 99.9% | 4199 mW |
| 纯手写 · rt | 0.60 ms | 0.65 | 0.66 | 3.5% | 81 mW |
| 纯手写 · ui | 0.60 ms | 0.72 | 2.69 | 3.5% | 104 mW |
| 手写 + Accelerate · rt | 0.43 ms | 0.52 | 0.56 | 2.6% | 71 mW |

读法（强度：单次、噪声大、自审已核数）：
1. **慢 5–7 倍，推断是帧间降频**【假设】：忙等就快回来，但耗电 31–59 倍。忙等同时也改变了核迁移、缓存热度和睡眠唤醒，现有计时分不开这几个因素；要证实还得记核驻留和频率（`powermetrics` 的 cluster 频率、每核驻留），交替重复（codex 第 1 轮 #7）。产品里不能忙等，这一点不受影响。
2. **自建工作组区间**（`AudioWorkIntervalCreate`，每帧报自己的截止期）能把 BNNS 拉回约 4 倍快，但 CPU 功率高 25–40 mW（单次，差值和噪声同量级）；只把 time-constraint 的截止报紧没用。「提速要花电」是【假设】，这一批证据**推翻不了 D74**（接受低频、工作组默认不加）。以后要压尾部或 `BE-IMPACT`，它是现成的旋钮。
3. **纯手写 ui → rt**：p50 不变，最大一帧从 2.69 降到 0.66 ms，支持 D74「time-constraint 默认开」。
4. **超 3 ms 的帧**：这一批 9 种设置、各 60 s，调用耗时 > 3 ms 的都是 0（当时还没有「迟到」口径，唤醒迟到没算进去）。但 `BE-DEADLINE` 要 10 分钟：rt 下还没跑过 10 分钟；ui 下 10 分钟，BNNS 最大 11.8 ms，纯手写 6 帧、Accelerate 版 1 帧超 3 ms（`apple/results/20261102-173000-b2/summary.md`、`notes/rustbe-numbers.md`）。

## 5. 怎么接进代码（接口建议；结构细节归 Rust 骨架线）

依据 D71（人定：频谱前端每帧只算一次、各模型共用，代码复杂度低）。下面只写推理后端这一侧要满足什么，crate 怎么拆、谁持有前端，由另一位大总管的 Rust 骨架线定。

1. **后端不做 STFT。** 前端每 hop 算一次频谱（sqrt-Hann 512 / 256，`vdn_dsp::StreamingStft`），GTCRN、修音链频谱节点、场景特征都吃这一份。后端的输入是「已算好的一帧频谱」（GTCRN：re[257] + im[257]），输出是增强后的频谱。**这个口径只对 GTCRN 成立**：GTCRN 的基准只计推理调用，不含 STFT（`standard.md` §3）。FE-M 现在测的是 wav2wav 图，STFT / iSTFT / 重叠相加都在图里，计时自然包含它们（端到端口径）；换成 spec2spec 后，要用频谱口径重测，不能直接拿现有的数比后端（codex 第 5 轮 #3）。
2. **自带 STFT 的模型要换成频谱进、频谱出的版本。** FE-M 今天用的 wav2wav 图把 STFT / iSTFT 包在图里（`notes/our-models.md` 第 2 行），违背 D71，也正是它转不了 Core ML 的原因。FE-M 若留在候选里，应换官方 spec2spec 版（`fastenhancer_m.spec.onnx`），由共用前端喂频谱。
3. **认人不吃这份频谱**：ECAPA 吃 80 维 fbank（25 ms 窗、10 ms 移），窗长、移步都和降噪的 STFT 不同；它在判断线程里自己算 fbank（`src/speaker/src/fbank.rs`）。要不要从共用前端派生，交 Rust 骨架线判断。
4. **后端 trait 最小面（建议）**：非实时线程 `open(model) -> Self` 并预热；实时线程 `run(&mut self, input: &[f32], output: &mut [f32])`，零分配、不加锁，缓存由后端自己持有（A/B 两组换指针，见 `apple-rs/src/bnns.rs`）；`reset()` 把缓存清零。和规约里的 `Model` 接口的差别只在「进出的是频谱，不是波形」。
5. **实现落点（建议）**：纯手写 GTCRN 直接作为一个模型实现，不经推理库；ORT 留给 FE-M / 认人（`vdn-backend-ort`）；BNNS Graph 作苹果侧备选（`vdn-backend-bnns`，C FFI，链接名带 `_v2`，吃 `.mlmodelc`，由 `apple/convert_gtcrn.py` 加 `compile` 子命令生成）。线程设置按 D74：time-constraint 开；工作组区间留作可选旋钮。
6. RTSan 豁免表、`no_alloc` 测试要按新后端重建。

## 6. 对延迟账的影响

- 延迟账里没有单列推理耗时。009 当时把它算进「跨进程环余量 3.00 ms」（`docs/research/009_rust-ecosystem/decision.md:62`）；**D74 已把产品的缓冲余量改为 4 ms**（lead 192 样本），现在以 4 ms 为准，3 ms 只作 `BE-DEADLINE` 里更严的技术线。模型带来的 32 ms 是 STFT 窗长决定的算法延迟，换后端改不了。所以换后端**不减延迟账**，收益在耗电（D70 · D75）和尾部。
- 尾部：产品的缓冲余量按 D74 是 4 ms（`BE-DEADLINE` 取 3 ms 作更严的技术线）。在 D74 默认（rt）下，60 s 内所有设置的最大调用耗时都 ≤ 1.41 ms；10 分钟长跑只在 ui 下跑过，那时纯手写有 6 帧、BNNS 至少 1 帧调用耗时超过 3 ms（是否超过 4 ms、会不会送原声没分开统计）。rt 下、按「迟到」口径的 10 分钟长跑待测。
- 最低系统：D78 定为 macOS 15，BNNS Graph / `MLState` 都不再有系统版本冲突（原待拍板 2-8 随之解决）。

## 7. 还没做的（需要名额或人）

- **要名额**：`BE-POWER` 重复 3 轮、含同场 ORT（v2 测法，人跑 sudo，约 35 分钟）· `BE-IMPACT` 重测（v2：性能核个数个线程、15 对、成对差、要超过噪声底）· `BE-DEADLINE` 在 rt 下 10 分钟（纯手写 · 手写 + Accelerate · BNNS）· `BE-SIZE`（ORT 静态库增量）· `BE-MEM`（纯手写版的常驻内存）· `BE-RT-SAN`。
- **自审留下的 P2（不挡这一版）**：逐值一致的「反向对照」（每帧清零缓存时必须判为不一致，用来证明对拍真能抓到缓存出错）· 分配计数器的自检补 calloc / realloc / posix_memalign / 大块分配 / C++ new · Core ML 那约 300 次 malloc 里有没有我们自己包装的份（加一帧「空 autoreleasepool」基线）· `dfn3ll_probe.py` 解包会覆盖已有目录 · FE-M 在 pace 为 0 时 time-constraint 周期取了 16 ms（应 10 ms）。
- **转交**：rustbe 的 bench 要输出 `win_start_unix` / `win_end_unix` / `wall_s`，它那几行耗电才能进排名（rustbe 会话已结束，交大总管）· apple-rs 与 rustbe bench 合成一个共用 crate（现在靠 `apple-rs/tests/fields.rs` 交叉校验字段，大总管）· 认人降频（大总管）· native-blas 的 11 次分配（rustbe）· FE-M 是否留在候选里（人）· engine 送原声分支不检查原声里的 NaN（大总管，审查顺带发现，不属 019）。
