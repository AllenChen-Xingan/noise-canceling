# 019 · 推理后端评判标准（verifier 清单）

> 2026-11-02 · applebe 起草 · 依据 D68（人定：正式测速一律从 Rust 调；Python 的数要量出与 Rust 的差）与 CLAUDE.md 第五节第 3 条（说「更快 / 能用」必须给 verifier 名 · 分数 · 是否过线）。
> 用途：判「某个后端跑某个模型，能不能上用户电脑、比基线好还是差」。每条都是机器能判的 verifier，人只判第 10 节标出的那几处。
> 过线值的状态：**【已定】** 来自人定的决策或规约 · **【AI 定，依据 …；人可改】** 纯技术线，按 CLAUDE.md 第七节第 7 条由 AI 定，依据写在旁边，结论直接按这些线判。只有 `BE-OS` 留给人（待拍板 2-8）。
> 来源等级同 notes/：【官方】【实测-他人】【源码】【实测】（本机）【推断】。

## 0. 被评对象与基线

- **被评对象** = 一个（后端，模型，精度，配置）组合。例：（Core ML cpuOnly，GTCRN，fp32，缓存显式进出）。
- **必测模型**：GTCRN（主模型）· FastEnhancer-M（备选）。
- **基线**：ORT 1.31 CPU 单线程，配置同 `src/models/src/session.rs:30-45`。「更快 / 更好」都是对它说的。
- **同口径**（rustbe 与 applebe 已对齐）：输入 `resources/gtcrn/test_wavs/mix.wav`（16 kHz，610 帧，不够就循环）；1 帧冷启动 + 200 帧预热 + 6000 帧计时；单线程；3 轮交替跑；名额独占；记机型 · 系统 · 电源 · 跑前跑后负载。输出一行 JSON，字段以 rustbe 的 `bench/src/main.rs` 为准。
- **只认 Rust 调用测出的数**（D68）。Swift / Python 的数只作对照，见第 9 节。

## 1. 总表

下表列出 12 个 verifier，「挡」一列说它挡什么（挡合并 = CI 层，挡上用户电脑 = QA 层）。

| id | 维度 | 判什么 | 过线 | 状态 | 挡 |
|---|---|---|---|---|---|
| `BE-PARITY` | 正确性 | 与 ORT 输出逐值一致 | 前 610 帧没有任何值差 > 1e-3，且整段 SNR ≥ 60 dB | 1e-3 同 rustbe 现用口径【已定·线间约定】；60 dB【AI 定，依据 §2；人可改】 | 一票否决：不过的「更快」不算 |
| `BE-FRAME` | 每帧耗时与尾部 | p50 · p99 · p99.9 · max | p99.9 ≤ 1.0 ms 且 max ≤ 2.0 ms | 【AI 定，依据延迟账 3 ms 余量（§3）；人可改】 | 挡上用户电脑 |
| `BE-FASTER` | 比基线快 | 和 ORT 同场比 | 3 轮里本后端最慢一轮的 p50 和 p99.9，都低于 ORT 最快一轮 | 【AI 定，依据 3 轮范围不重叠（§3）；人可改】 | 写「更快」的前提 |
| `BE-RT-ALLOC` | 实时线程安全 · 分配 | 稳态每帧全进程 malloc 次数 | = 0 才算「可在推理线程直接调」；> 0 按函数名登记上限 | 0 来自规约 B 档对我们自己代码的要求【已定】；对第三方也要求 0【AI 定，依据：区分「能直接调」和「只能登记」（§4）；人可改】 | 决定接法（直接调 / 登记豁免） |
| `BE-RT-SAN` | 实时线程安全 · 锁与系统调用 | RTSan 报的唯一栈 | 0 个新栈（对豁免表） | 【已定】规约 §5.5 | 挡合并 |
| `BE-RT-XPROC` | 实时线程安全 · 跨进程 | 推理线程每帧发出的 mach 消息 | = 0 | 【AI 定，依据规约 A/B 档「不阻塞」（§4）；人可改】 | 决定接法 |
| `BE-COLD` | 冷启动 | 打开 + 预热耗时；第 1 帧 | 打开 + 200 帧预热 ≤ 500 ms；预热后第 1 帧 ≤ max 线 | 【AI 定，依据「换模式半秒内生效」（§5）；人可改】 | 挡上用户电脑 |
| `BE-CPU` | CPU | 按实时节奏跑时占几个核 | ≤ 0.05 核，且不高于 ORT | 【AI 定，依据 D31 整机约 0.2 核（§6）；人可改】 | 挡上用户电脑 |
| `BE-POWER` | 耗电 | 按实时节奏跑 60 s 的封装功率增量 | 不高于 ORT | 【AI 定，依据 D31（§6）；人可改】 | 挡上用户电脑（要人跑 sudo） |
| `BE-SIZE` | 体积 | 安装包里新增的字节数（库 + 模型文件） | 不高于 ORT 方案 | 【AI 定，依据：换后端不该让安装包变大（§7）；人可改】 | 挡上用户电脑 |
| `BE-OS` | 最低系统 | 后端要求的最低 macOS | ≤ 14.2 | 【已定】D07。⚠️ 与 BNNS Graph / `MLState`（要 15）冲突，见待拍板 2-8 | 挡上用户电脑 |
| `BE-SWAP` | 换模型难度 | 换一个新模型要做什么 | 一条命令重转 + `BE-PARITY` 过 | 【AI 定（§7）；人可改】 | 只记分，不挡 |
| `BE-TRAIN` | 本机微调能力 | 能否对 BN/LN 仿射参数求梯度 | 梯度与 PyTorch 相对差 ≤ 1e-3 | 【AI 定，依据 D52（§7）；人可改】 | 只记分，不挡（推理后端可以不会训练） |

## 2. 正确性：`BE-PARITY`

- **怎么测**：rustbe 的 `gtcrn-bench ort ... --dump-io <前缀>` 写出同一串输入和 ORT 输出（小端 f32，按帧拼接）。被评后端吃同一份输入，从零状态逐帧跑，缓存一路往下走。
- **算什么**：逐值最大绝对差 `parity_max_abs`、整段 SNR `parity_snr_db`、首个差 > 1e-3 的位置 `parity_first_bad`。算法同 rustbe 的 `parity`。
- **过线**：`parity_first_bad == None` 且 `parity_snr_db ≥ 60`。
- **60 dB 的依据**【推断】：rustbe 手写 Rust 对 ORT 是 134.0 dB，tract 130.4 dB，FE-M tract 110.8 dB（`notes/rustbe-numbers.md`）。fp32 不同实现之间差 100 dB 以上很常见。60 dB 相当于误差能量是信号的百万分之一，留了余量给「算子顺序不同的 fp32」，但会挡住 fp16（fp16 尾数只有 11 位，单次舍入误差约 −66 dB，累积后一般低于 60 dB【推断】）。
- **不拿 exp004 的 4.5e-7 当线**：那是 ORT 对 ORT（Rust 调与 Python 调同一个库），不是两个实现之间的差（怀疑派已指出，`notes/coreml.md` 文末）。
- **长时补充**：610 帧只有约 10 s。上用户电脑前还要在长音频（≥ 10 分钟，含长静音）上跑一遍，看缓存会不会漂、会不会出 NaN（DFN 有长静音出 NaN 的先例，`our-models.md` 第 3 行）。字段 `nonfinite_frames` 必须为 0。

## 3. 每帧耗时与尾部：`BE-FRAME` · `BE-FASTER`

- **怎么测**：只计「一次推理调用」（含输入写入和输出读出，不含 STFT）。计时器用 `mach_absolute_time` / `Instant`。
- **`BE-FRAME` 过线的依据**：延迟账里没有单列推理耗时。它被「跨进程环余量 3.00 ms」盖住，009 对这一项的定义是「盖住最坏推理耗时 + 唤醒抖动」（`docs/research/009_rust-ecosystem/decision.md:62`）。所以 max ≤ 2.0 ms 给唤醒抖动留 1 ms；p99.9 ≤ 1.0 ms 是今天 ORT 的水平（0.42–0.59 ms，r4）再放一倍【推断】。
- **`BE-FASTER` 的规则**：3 轮交替跑，各取 p50 和 p99.9。只有「本后端最慢一轮」比「ORT 最快一轮」还快，才写「更快」，并写倍数（ORT p50 中值 ÷ 本后端 p50 中值）。轮间范围有重叠就写「持平」。
- **FE-M 的预算**是每 10 ms 一帧，同一条线也适用（线是绝对毫秒，不是占预算比例）。

## 4. 实时线程安全：`BE-RT-ALLOC` · `BE-RT-SAN` · `BE-RT-XPROC`

这三条决定后端的**接法**：三条都过 → 可以在推理线程每帧直接调；有不过的 → 像今天的 ORT 一样登记豁免，或者挪到别的线程，再看 `BE-FRAME` 的尾部。

- **`BE-RT-ALLOC`**：用 libmalloc 的 `malloc_logger` 钩子数全进程分配（`apple/alloc_hook.c`，Instruments 用的同一个钩子）。只数计时段，除以帧数，字段 `process_allocs_per_frame`。注意：
  - 这个数和 rustbe 的 `rust_allocs_per_frame`（只数 Rust 分配器）不是同一个量，两个名字不混用。
  - 别的进程（神经引擎守护进程 `aned` 等）里的分配看不到，所以要和 `BE-RT-XPROC` 一起看。
  - 规约 B 档只要求「我们的 Rust 代码零分配」，ORT 内部分配允许登记（`specs/rust/architecture.md` §4）。对第三方后端也要求 0，是为了区分「能直接调」和「只能登记」【AI 定；人可改】。
- **`BE-RT-SAN`**：Clang RealtimeSanitizer，B 档用法照规约 §5.5（`halt_on_error=false`、必须符号化、去重后的唯一栈对豁免表）。每个新后端要建自己的豁免表（命名空间不同：Core ML 是 Objective-C 方法名，BNNS 是 `BNNS*`，MLX 是 `mlx::`）【推断】。RTSan 拦不到不经 libc 的自旋和缺页（`notes/bnns-graph.md`），所以它过了也不等于没锁。
- **`BE-RT-XPROC`**：推理线程每帧发出的 mach 消息数必须为 0。要 sudo（`dtruss -t mach_msg` 或 `fs_usage`），命令写成文件交人跑。不能跑 sudo 时，用两条旁证：`MLComputePlan` 显示所有算子都在 CPU（Core ML / BNNS 才有）；`powermetrics` 里神经引擎和 GPU 功率在跑与不跑时没有差别。

## 5. 冷启动：`BE-COLD`

- **怎么测**：`open_ms`（含编译 / 加载）· 200 帧预热的总时长 · 预热后第 1 帧耗时。Core ML / BNNS 另报 `compile_ms`（mlpackage → mlmodelc → BNNS 图）。
- **为什么要看**：换模型时，新 Pipeline 在非实时线程建好、预热后再送进实时线程（规约 §3.5），所以冷启动不进实时线程，但决定「用户换模式后多久生效」和「开机多久能用」。500 ms 按「人按下开关后半秒内生效」定【AI 定；人可改】。
- 神经引擎有「空闲约 5 s 后第一帧约 260 ms」的他人实测（`notes/realtime-and-bindings.md`），所以对用神经引擎的组合，还要测「停 10 s 再来一帧」的耗时。

## 6. CPU 与耗电：`BE-CPU` · `BE-POWER`

- **`BE-CPU`**：按实时节奏跑（`pace_ms` = 16 或 10）60 s，`cpu_over_wall` = 进程 CPU 时间 ÷ 墙钟时间，单位是「核」。0.05 核的依据：D31 定整机约 0.2 核（`001_decisions…md:139-143`），降噪推理只是其中一块；今天 ORT GTCRN 约 0.37 ms / 16 ms ≈ 0.023 核【推断，按 r4 p50 算】。
- **注意**：神经引擎和 GPU 的工作不算进 `cpu_over_wall`，所以凡是用到它们的组合，必须看 `BE-POWER`。
- **`BE-POWER`**：`powermetrics -i 500 --samplers cpu_power,gpu_power,ane_power`，先空闲 30 s 取基线，再按实时节奏跑 60 s，报 CPU / GPU / 神经引擎三项功率的增量（mW）。要 sudo，由人跑：rustbe 的 `tools/power_run.sh`、applebe 的 `apple/power.sh`，两者口径要一致（合并前对一次）。

## 7. 体积、系统、换模型、微调：`BE-SIZE` · `BE-OS` · `BE-SWAP` · `BE-TRAIN`

- **`BE-SIZE`**：数安装包里因这个后端多出的字节（动态库 / 静态链接增量 + 模型文件）。系统自带框架（Core ML、Accelerate）记 0。ORT 静态链接的增量要实测（`cargo bloat` 或对比带 / 不带 ort 的二进制大小）。
- **`BE-OS`**：读头文件的 `API_AVAILABLE` 标注。已知：BNNS Graph、`MLState` 要 macOS 15；`MLComputePlan` 要 14.4（`notes/realtime-and-bindings.md` §0）。D07 定的是 14.2+。冲突已登记为待拍板 2-8，**等实测再问**：如果 BNNS 不比手写 Rust 快，这题作废。
- **`BE-SWAP`**：记三项——要不要重写代码（手写 Rust 要）· 有没有一条命令的转换脚本（Core ML / BNNS / MLX 要从 PyTorch 重新转，ORT / tract 直接吃 ONNX）· 转换后 `BE-PARITY` 是否过。只记分，不挡。
- **`BE-TRAIN`**：对 BN/LN 仿射参数（GTCRN 约 4.8 K 个）求一次损失梯度，和 PyTorch autograd 比，相对差 ≤ 1e-3。目前只有 MLX 有自动求导（`notes/mlx.md` §5）。推理后端不会训练不扣分：D52 的本机微调可以单独用一个后端，推理再用别的。

## 8. 参照：成熟产品与开源实时实现怎么守这些线

> 详细出处见 `notes/references-products.md`（子代理在写，写完补这一节的摘要）。

苹果官方立场（已在 notes 里核过）：
- WWDC25 276：「We recommend BNNSGraph for audio and other latency-sensitive use cases because it gives you control over allocating memory and multithreading.」【官方】（`notes/bnns-graph.md` §2）
- 苹果在 WWDC24 10211 的演示里，从 Audio Unit 渲染内核里直接调 `BNNSGraphContextExecute`【官方】。

## 9. Rust 与 Python 的差（D68 第 ③ 条）

**规则**：Python 测出的数不进结论。要引用 Python 的数（exp001 的 Python ORT、MLX Python 版），先在同一模型、同一输入上量出 Rust 调与 Python 调的差，写进下表。

| 模型 | 后端 | Rust p50 / p99.9 / max | Python p50 / p99.9 / max | 差 | Python 数能不能用 |
|---|---|---|---|---|---|
| GTCRN | ORT 1.31 CPU | r4：0.37–0.38 / 0.42–0.59 / 0.43–1.00 | 待测（exp001 记的是 p50 0.40，口径不同） | 待测 | 待测 |
| GTCRN | MLX CPU | 待测（`apple-rs` 经 mlx-c 载 `.mlxfn`） | 待测（`apple/mlx_gtcrn.py`） | 待测 | 待测 |
| GTCRN | MLX GPU | 待测 | 待测 | 待测 | 待测 |

**判法**【AI 定，依据：GIL 与垃圾回收主要伤尾部；人可改】：Python 与 Rust 的 p50 差 ≤ 10%，且 p99.9 差 ≤ 20%，才允许在 Python 里做「同一后端、不同配置」的相对比较；绝对数一律用 Rust 的。Python 的尾部还受 GIL 和垃圾回收影响，所以 max 永远不用 Python 的【推断】。

## 10. 留给人判的（验证红线）

- 第 1 节里【AI 定】的过线值，人可以随时改；改了就按新线重判。
- `BE-POWER` 和 `BE-RT-XPROC` 要 sudo，由人跑命令。
- `BE-OS` 冲突（待拍板 2-8）。
- `BE-PARITY` 只证明「和 ORT 算得一样」。如果某个后端只能 fp16、`BE-PARITY` 不过但更快，它听起来是否有差别，只能人盲听判，并且要先改线。

## 11. 结果写在哪

- 每次跑：一行 JSON，追加到 `docs/experiments/exp010-inference-bench/results/apple_<日期>-<build-id>.jsonl`，每行带机器状态。
- 判定表：`docs/research/019_apple-inference/decision.md` 里每个组合一行：每个 verifier 的分数 · 是否过线 · 线的状态（已定 / 建议）。
