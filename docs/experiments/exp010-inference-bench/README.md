# exp010 · 推理后端基准（降噪模型在 Mac 上用什么跑最快）

> 意图：人 2026-10-31 的决定（D53），原话「必须用 core ml 或者 mlx 就是所有的推理引擎如果现在的不好，我们要重新写，用rust 或者用苹果的引擎重新写，必须达到最好的效果，最快的效果」。之后又加了三条：D68「正式测速一律从 Rust 调」、D69「整机 CPU 目标 1%、上限 2%」、D70「榨干硬件、非常低能耗、轻量」。
> 本实验分两边：
> - **纯 Rust 一侧**：本文，手下 rustbe，issue #51
> - **苹果引擎一侧**：Core ML / BNNS Graph / MLX，手下 applebe，写进 `docs/research/019_apple-inference/`
>
> 两边用同一个基准程序（`bench/`，rustbe 维护）、同一份输入、同一套评测器。评测器的正本是 applebe 的 `docs/research/019_apple-inference/standard.md`。
> 本文的名字：
> - **手写版** = 纯 Rust 手写 GTCRN（后端名 `native`）
> - **手写 + Accelerate 版** = 同一份代码，矩阵乘交给苹果 Accelerate 库（后端名 `native-blas`）
> - **ORT** = ONNX Runtime 1.31（微软的推理库），CPU 单线程，产品现在用的就是它
> - **FE-M** = FastEnhancer-M（备选降噪模型）

## 一句话结论

- **主模型 GTCRN：手写版每帧平均 0.088 ms，是 ORT（0.381 ms）的 1/4.3**
  - 和 ORT 逐值一致：SNR 134 dB；10 分钟长跑 133 dB
  - 每帧零分配；冷启动 0.36 ms（ORT 是 11 ms）
  - 6 个评测器都过线
  - **还没到 D69 的推理目标「每帧平均 ≤ 0.08 ms」，差 10%**。手写 + Accelerate 版是最有希望的下一步，见「还能怎么更快」
- **tract（纯 Rust 的 ONNX 推理库）在两个模型上都比 ORT 慢**：GTCRN 慢 1.45–1.48 倍，FE-M 慢 1.78–1.80 倍。它每帧还要分配上千次，不能放在实时线程里
- **Rust 调 ORT 对 Python 调 ORT**：同一模型、逐位相同的输入，Python 的 p50 慢 5–6%，输出逐位相同
- **FE-M 目前最快的后端仍是 ORT**：p50 0.38 ms，约占 3.8% 个核，超过 D69 的 2% 上限。FE-M 要不要继续当候选，待人定（见文末）

以上都是本机实测：M5，macOS 27.0.1，插电，2026-11-02，每格 3 轮。

## 判据（评测器 · 本轮结果）

评测器的名字和过线值照 applebe 的 `standard.md` §1。下表中「过」「不过」都由机器按数字判；我没有给出主观判断。正式数据用 `results/20261102-r5b/`。

| 评测器 | 过线（摘自 standard.md） | ORT（基线） | 手写版 | tract |
|---|---|---|---|---|
| `BE-PARITY` 逐值一致 | 没有值差 > 1e-3，且 SNR ≥ 60 dB | —（参考） | **过**：134.0 dB；10 分钟长跑 133.1 dB；安静帧最差 94.2 dB | **过**：130.4 dB；6201 帧 122.2 dB |
| `BE-FRAME` 帧耗时尾部 | p99.9 ≤ 1.0 ms 且最大 ≤ 2.0 ms | 过：0.44–0.54 / 0.53–0.87 | **过**：0.11–0.13 / 0.11–0.15 | 过：0.60–0.66 / 0.71–0.82 |
| `BE-FASTER` 比 ORT 快 | 本后端最慢一轮的 p50、p99.9，都低于 ORT 最快一轮 | — | **过**：p50 0.088 < 0.379；p99.9 0.131 < 0.435 | **不过**：比 ORT 慢 |
| `BE-COLD` 冷启动 | 打开 + 200 帧预热 ≤ 500 ms | 过：约 87 ms | **过**：约 18 ms | 过：约 180 ms |
| `BE-MEM` 常驻内存增量 | 预热后 − 加载前 ≤ 10 MB | 过：0.6 MB | **过**：1.0–1.1 MB | **不过**：10.0–10.2 MB |
| `BE-CPU` 推理占几个核 | 挡线 ≤ 0.010 核；目标 ≤ 0.005 核 | **不过挡线**：约 0.024 | **过挡线，没到目标**：约 0.0055 | 不过：约 0.031 |
| `BE-RT-ALLOC` 每帧全进程分配 | = 0 | 〔见「全进程分配」一节〕 | Rust 侧 0；全进程见下 | 2247（Rust 侧）→ 不过 |
| `BE-DEADLINE` 10 分钟按实时节奏，超时帧 | 单帧 > 3.0 ms 的帧 = 0 | 尚无，待测 | 尚无，待测 | — |
| `BE-POWER` 每帧耗电 | 不高于 ORT | 尚无，待人用 sudo 跑 | 同左 | 同左 |

几点说明：
- **`BE-CPU` 是推断值**：每帧平均耗时 ÷ 帧长。例如手写版 0.088 ms ÷ 16 ms = 0.0055 核。按实时节奏实测占用，要等耗电那一轮一起测
- **FE-M**：ORT 那一行，`BE-CPU` 约 0.038 核，不过挡线；tract 那一行，`BE-FASTER`、`BE-MEM`（8.2–8.4 MB 过）、`BE-CPU` 不过
- **样本量**：每格 3 轮 × 6000 帧。p99.9 只由每轮最慢的约 6 帧决定，最大值只由 1 帧决定。所以尾部的「3 轮范围」比中位数更不可靠
- 评测器只管速度、内存、逐值一致；声音好坏不归它们管。手写版和 ORT 逐值一致，所以音质和 ORT 相同

## 结果（正式版 r5b，本机实测）

**测量条件**：
- 机器：M5（Mac17,3）· macOS 27.0.1 (26A434) · 插电 · rustc 1.99.1
- 时间：2026-11-02 17:11–17:13；跑前负载 2.59，跑后 2.42（1 分钟平均）
- 大总管确认没有别的重活。开跑前 CPU 最高的三个进程是两个 Claude 会话和 WindowServer，各占 10–20%
- 计时线程的调度等级设为 `USER_INTERACTIVE`
- 每轮把后端顺序轮换一位，共 3 轮；表里是 3 轮的范围，时间单位 ms

**数据**：`results/20261102-r5b/`
- `env.txt`：机器状态和所有文件的 sha256
- `parity.jsonl`：对拍
- `bench.jsonl`：计时
- `bench-extra.jsonl`：tract 复用状态的那次计时
- `amx.jsonl`：Accelerate 小矩阵实测
- `profile_native.txt`：手写版的热点分布

**GTCRN**（每帧 16 ms 一帧）

| 后端 | p10 | p50 | p90 | p99 | p99.9 | 最大 | 平均 | 冷启动（打开 + 第 1 帧） | 每帧 Rust 侧分配 | 内存增量 MB |
|---|---|---|---|---|---|---|---|---|---|---|
| ORT（基线） | 0.369–0.372 | 0.379–0.382 | 0.392–0.393 | 0.408–0.412 | 0.435–0.536 | 0.527–0.874 | 0.381–0.383 | 10.9 | 0（不含 C++ 侧） | 0.6 |
| **手写版** | 0.086 | **0.087–0.088** | 0.089–0.090 | 0.096–0.100 | 0.108–0.131 | 0.113–0.153 | **0.088** | **0.35–0.38** | **0** | 1.0–1.1 |
| tract 0.23.9 | 0.353–0.414 | 0.553–0.564 | 0.571–0.572 | 0.590–0.595 | 0.603–0.657 | 0.714–0.819 | 0.477–0.540 | 68.5–70.6 | 2247 | 10.0–10.2 |

**FE-M**（每帧 10 ms 一帧）

| 后端 | p10 | p50 | p90 | p99 | p99.9 | 最大 | 平均 | 冷启动 | 每帧 Rust 侧分配 | 内存增量 MB |
|---|---|---|---|---|---|---|---|---|---|---|
| ORT（基线） | 0.368–0.374 | 0.380–0.383 | 0.385–0.393 | 0.405–0.447 | 0.422–0.794 | 0.538–0.971 | 0.380–0.384 | 10.4–11.0 | 7（不含 C++ 侧） | 0.6–0.7 |
| tract（改写图） | 0.564–0.670 | 0.681–0.686 | 0.692–0.698 | 0.709–0.715 | 0.720–1.016 | 0.736–1.373 | 0.660–0.681 | 27.4–27.5 | 1041 | 8.2–8.4 |

- **倍数**（按轮配对，3 轮）：
  - ORT ÷ 手写版：p50 4.33–4.37 倍，平均 4.33–4.35 倍
  - tract ÷ ORT：GTCRN 上 p50 1.45–1.48 倍、p99 1.44–1.46 倍；FE-M 上 p50 1.78–1.80 倍
- **「冷启动」口径**：打开 + 第 1 帧，不含 ORT 环境初始化。ORT 环境初始化另记在 `ort_env_ms`，每次约 17 ms，是一次性的；手写版不用 ORT，没有这一项
- **tract 的帧耗时分成两档**：GTCRN 的 p10 是 0.35–0.41，p50 是 0.55–0.56；FE-M 同样分两档
  - 设了 `USER_INTERACTIVE` 调度等级后，两档仍在，可见原因不是线程被调度到能效核
  - 真正原因没查；ORT 和手写版都没有这种现象
- **对拍**（`parity.jsonl`，`first_bad` 全部是 None）：
  - 手写版：610 帧，134.0 dB，安静帧最差 99.2 dB；10 分钟（37 500 帧，循环音频）133.1 dB，安静帧最差 94.2 dB，没有漂移
  - **reset 检查**：手写版跑 20 000 帧后调 `reset`，再和新开的实例同喂 17 500 帧，输出**逐位相同**（差 0）
  - tract：610 帧 130.4 dB；6201 帧 122.2 dB
  - FE-M：tract 跑改写图对 ORT 跑原图，110.8 dB；**ORT 跑改写图对 ORT 跑原图，逐位相同**（差 0），这证明了改写本身等价

## Rust 调用对 Python 调用（人定：「测试如果需要的话，也要知道rust 和python的差异」）

**做法**：
- Rust 侧：`gtcrn-bench ort` 先转储全部 6201 帧的输入和输出
- Python 侧：`tools/py_ort_bench.py`（onnxruntime 1.31.1，和 Rust 侧同版本）读同一份转储，逐位相同的输入
- 会话设置照上次 Python 帧耗时实验（exp001）：单线程，`ORT_SEQUENTIAL`；计时线程的调度等级同样设为 `USER_INTERACTIVE`
- Python 只计 `sess.run` 那一次调用；Rust 计时还多包含输入输出拷贝，对 Rust 略不利
- 两边交替跑 3 轮。数据：`results/20261102-pyrs1/`

| | p50 | p99 | p99.9 | 最大 | 平均 |
|---|---|---|---|---|---|
| Rust 调 ORT | 0.377–0.380 | 0.407–0.411 | 0.444–0.550 | 0.552–0.666 | 0.378–0.382 |
| Python 调 ORT | 0.399–0.401 | 0.427–0.434 | 0.445–0.558 | 0.576–0.672 | 0.400–0.401 |
| Python ÷ Rust（按轮） | 1.055–1.060 | 1.041–1.067 | 0.94–1.26 | 0.92–1.22 | 1.049–1.059 |

- Python 的输出和 Rust ORT **逐位相同**（最大差 0）
- **差多少**：中位数和平均都是 Python 慢 5–6%，约 0.02 ms。推断这部分是 Python 把 numpy 数组转成 ORT 张量的开销
- **尾部**（p99.9、最大）分不出谁快：3 轮里各有胜负，样本太少
- 结论：以前 Python 量出来的模型耗时，换成 Rust 调用约 0.94 倍。用 Python 估 Rust 的耗时，误差在 6% 以内（只对 ORT 这一个库成立）

## 还能怎么更快（D69：推理每帧平均 ≤ 0.08 ms）

**手写版的时间花在哪**（`results/20261102-r5b/profile_native.txt`；macOS `sample` 采样 4 秒，共约 1800 个样本，按栈顶函数计）：

| 部分 | 占比 |
|---|---|
| 编码器卷积 `ConvF` | 22% |
| GRU 单步 | 20% |
| `tanhf`（系统数学库） | 17% |
| 1×1 卷积 | 11% |
| 转置卷积 | 10% |
| DPGRNN 其余部分 | 9% |
| `expf`（sigmoid 用） | 6% |

〔每条的估计收益和对逐值一致的影响，见下方「各方法估计」；手写 + Accelerate 版的实测结果待名额，跑完补〕

**已做的**：卷积不再逐个元素判断边界，改成先算好有效区间（逐位不变：对拍的最大差、SNR、最差帧位置都和改前一样）。平均耗时从 0.097 降到 0.088 ms（−10%，2 轮冒烟），r5b 正式数字就是改后版本。

**Accelerate 在小矩阵上的实测**（`results/20261102-r5b/amx.jsonl`；用 GTCRN 实际用到的 5 种形状，各跑 20 万次取平均）：

| 形状（C = A·B） | 每帧次数 | 手写循环 µs / 帧 | Accelerate µs / 帧 |
|---|---|---|---|
| 1×1 卷积 24→16，33 频点 | 6 | 7.55 | 5.45 |
| 1×1 卷积 16→8，33 频点 | 6 | 2.98 | 1.99 |
| DPGRNN 全连接 16→16，33 频点 | 4 | 3.92 | 0.89 |
| 编码器第一层（im2col 45→16，65 频点） | 1 | 3.31 | 0.31 |
| GRU 输入投影 8→12，33 频点 | 8 | 3.41 | 2.69 |
| **合计** | | **21.2** | **11.3** |

- 测前推断是「矩阵太小，调 BLAS 不划算」，实测推翻了这个推断：5 种形状都是 Accelerate 更快
- 结果和手写循环的差 ≤ 2.4e-7（求和顺序不同）

**各方法估计**（〔推断〕都按上面的热点占比估；〔实测〕的已经量过）：

| 方法 | 估计收益（每帧） | 对逐值一致的影响 | 风险 · 状态 |
|---|---|---|---|
| 矩阵乘交给 Accelerate（走 AMX / SME 矩阵单元） | 小矩阵实测省约 10 µs（合计 21.2 → 11.3）；整网省多少待实测 | 求和顺序变了，不再逐位相同；预期 SNR 仍 > 100 dB | 实时线程里调 Accelerate 会不会分配内存或加锁，要用全进程 malloc 计数和 RTSan 查。**已写成 `native-blas`，待名额实测** |
| 快速 tanh / sigmoid（多项式或有理逼近，可向量化） | `tanhf` 加 `expf` 占 23%，约 20 µs；逼近版估计省 10–15 µs〔推断〕 | 相对误差约 1e-6–1e-4，取决于逼近阶数；要用 `BE-PARITY` 判 | 低风险，下一步就做 |
| GRU：输入投影先整批算，循环里只留隐藏状态那一半；4 条独立的 GRU 链交错执行 | GRU 占 20%；估计省 5–8 µs〔推断〕 | 只改执行顺序、不改每个值的算法，可以做到逐位不变 | 中等工作量 |
| 手写 NEON 内联（`std::arch::aarch64`） | 编译器已经把大部分内层循环向量化了；剩下的收益要看反汇编〔推断〕 | 可以逐位不变 | 代码难维护，排在上面几条之后 |
| 转置卷积改成「先插零、再当普通卷积」，也交给 Accelerate | 转置卷积占 10%，估计省 3–5 µs〔推断〕 | 不再逐位相同 | 和 Accelerate 那条一起做 |
| fp16 存权重和中间结果 | 权重总共只有 95 KB，已经整个放在 L1 / L2 缓存里，fp16 省不出内存带宽〔推断〕；M5 的 fp16 向量算力是 fp32 的 2 倍 | SNR 预计降到 60 dB 附近，可能不过 `BE-PARITY` 的 60 dB 线（standard.md §2 推断） | 不建议 |
| 多线程 | 一帧只有约 35 万次乘加，线程同步一次就要几 µs，得不偿失〔推断〕；还违反单线程口径 | — | 不建议 |

## 怎么跑

```sh
cd docs/experiments/exp010-inference-bench/bench && cargo build --release
B=target/release/exp010-bench
M=<主目录>                                    # 权重、ONNX、音频都在仓外
$B gtcrn-parity $M/_tmp/20261031-exp010/gtcrn_dns3 $M/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx <wav> --vs native   # 或 native-blas / tract
$B gtcrn-bench native|native-blas $M/_tmp/20261031-exp010/gtcrn_dns3 <wav>
$B gtcrn-bench ort|tract <onnx> <wav>
$B fem-parity <fastenhancer_m.onnx> <wav> --vs tract|ort --b-onnx <改写图>
$B fem-bench ort|tract <onnx> <wav>
$B amx-probe
bash ../tools/run_bench.sh <build-id>          # 全套对拍 + 基准，结果写 results/<build-id>/，不覆盖
bash ../tools/run_py_vs_rust.sh <build-id>     # Rust 对 Python
bash ../tools/run_blas_probe.sh <build-id>     # 手写版对手写 + Accelerate 版，加全进程 malloc 计数
sudo bash ../tools/power_run.sh <build-id>     # 耗电，要 sudo，由人跑
```

所有子命令都输出一行 JSON。参数和字段的正本是 `bench/src/main.rs` 顶部注释。

## 口径（两边共用，已和 applebe 对齐）

- **输入**：`resources/gtcrn/test_wavs/mix.wav`，16 kHz 单声道，610 帧
  - 基准：1 帧冷启动、200 帧预热，再计时 6000 帧；音频不够就循环，STFT 中途不重置
  - 对拍：默认用整段音频，不循环；`--frames N` 时循环
  - GTCRN 的一帧：同一个 STFT（`vdn-dsp`，512 点 / hop 256，sqrt-hann）出来的 257 个复数频点，16 ms
  - FE-M 的一帧：160 个样本，10 ms（图里自带 STFT）
- **对拍**（逐值一致，沿用上次 Rust 对拍实验 exp004 的做法）：
  - 同一份输入，ORT 和被测后端各从零状态顺序跑完，比较每帧的全部输出值
  - 报这几项：
    - 最大绝对差
    - 整段 SNR，即 10·log10(Σref² / Σ差²)
    - 逐帧 SNR 的最小值 `min_frame_snr_db`：只统计参考能量 ≥ 全段平均帧能量 1e-4 的帧，专看安静段
    - 非有限值个数
    - `first_bad`：第一个差 > 1e-3 或出现 NaN 的位置
  - `first_bad` 不为 None 的「更快」不算数
- **计时**：只计一次 `infer` 封装调用，包括输入输出拷贝和缓存回灌，不包括 STFT
- **单线程**：
  - ORT：intra / inter = 1、不自旋、`PreferCPU`、`ORT_ENABLE_ALL`，和 `src/models/src/session.rs` 一样
  - tract：不设线程池
  - 手写版、Accelerate：单线程调用
- **调度等级** `--qos`：默认 `ui`，即 `QOS_CLASS_USER_INTERACTIVE`
- **字段**：
  - `p10_ms` / `p50_ms` / `p90_ms` / `p99_ms` / `p999_ms` / `max_ms` / `mean_ms`：分位数按 round((n−1)·p) 取
  - `ort_env_ms`：ORT 环境初始化加读输入形状，计时前做，不算进 `open_ms`
  - `open_ms`：建模型；`first_frame_ms`：第 1 帧
  - `cpu_over_wall`：计时段 CPU 时间 ÷ 墙钟时间
  - `rust_allocs_per_frame`：Rust 分配器次数，不含 C / C++
  - `process_allocs_per_frame`：加 `--count-malloc` 时，用 `malloc_logger` 钩子数全进程分配；不加时为 null。钩子会拖慢分配，所以计数和计时分开跑
  - `footprint_before_mb` / `footprint_warm_mb` / `footprint_end_mb`：`ri_phys_footprint`，即活动监视器的「内存」列。分别在加载前（输入帧和 ORT 环境已在内存里）、预热完、跑完时读
  - `pace_ms`：大于 0 时按实时节奏喂帧
- **转储** `--dump-io <前缀>`：写出全部帧的输入和 ORT 输出（小端 f32），外加一份 json 记录 sha256。applebe 的程序和 Python 对照都吃这一份

## 选哪个 Rust 推理库：tract

依据来自子代理调研（2026-11-02）。来源等级：源码 / 官方 / 公开实测。

| 库 | 版本 | GRU | ConvTranspose（转置卷积） | LayerNorm | 结论 |
|---|---|---|---|---|---|
| tract | 0.23.9（2026-10-15） | 有〔源码〕 | 有 | 有 | 选它，理由见表下 |
| rten | 0.27.0 | 有 | 有 | 有〔源码〕 | 第二候选；没有小模型的公开实测，本轮没测 |
| burn-onnx | 0.22.0 | 有 | 有 | 有〔官方算子表〕 | 把 ONNX 生成 Rust 代码；小算子在 CPU 上的性能没有数据，本轮没测 |
| candle-onnx | 0.11.1 | **无** | **无** | **无**〔源码〕 | 跑不了 GTCRN，排除 |

选 tract 的理由：
- tract 自己的每晚基准里就有 GTCRN 流式单帧模型（`ci/benches.toml` 第 546–552 行），状态张量的形状和我们的一样
- 公开实测 M1 Max 单线程每帧 0.294 ms
- 2026-08 至 10 月专门做了 GRU 融合〔官方 CHANGELOG〕

实测下来 tract 没赢 ORT。结论只能说「tract 没赢 ORT」，不能推到「Rust 推理库都赢不了」：rten 和 burn 本轮没测。

tract 公开的 0.294 ms 和本机的 0.55 ms（p50）对不上。「每帧新开状态」的代价实测只有约 0.06 ms（复用状态时 p50 0.552、平均 0.493，见 `bench-extra.jsonl`），解释不了这个差距。原因没查。

## 手写 GTCRN（`bench/src/gtcrn_native.rs`）

- **来源**：照 `resources/gtcrn/stream/gtcrn_stream.py` 的 `StreamGTCRN.forward` 一层层手写，不依赖任何库。自审时已对着 Python 逐层核对过语义〔读源码〕
- **权重**：
  - 由 `tools/dump_gtcrn.py` 从 DNS3 checkpoint 导出，放在主目录 `_tmp/20261031-exp010/gtcrn_dns3.{bin,txt}`
  - bin 的 sha256 是 `e88502c4a3f8271b1803202d4ecb712fe687f789746ab596cb0f7b978c9a5174`
  - 和 `gtcrn_simple.onnx`（sha256 `b4718df6…bb87`）同源
- **加载**：
  - BN（批归一化）折进卷积；ERB（频带合并）矩阵只存非零段
  - 校验 txt 与 bin 的总长、每个权重的偏移，GRU 宽度不超过栈缓冲；对不上就返回错误，不 panic
- **运行**：
  - 所有缓冲在构造时一次分配好；每帧零次 Rust 分配〔实测〕
  - 每帧不加锁、不做系统调用〔读源码，没有机器检查〕
- **对拍**：2026-10-31 由 engine 写完，本轮第一次编译和对拍，没改一行就通过了
  - 37 段（mix.wav 加 exp004 的 36 段）对 ORT：最差 SNR 119.2 dB，最大差 1.6e-4
  - 数据：`results/parity_native_vs_ort.jsonl`。这份文件是旧版 bench 写的，没有 `min_frame_snr_db` 字段

## 踩到的坑

1. **tract 复用同一个运行状态会算错**：
   - 现象：第 0 帧对，第 1 帧起错，SNR 19.4 dB（`results/20261102-r5b/parity.jsonl` 里的 `tract-reuse-known-wrong`）
   - 原因〔读源码，自审查到〕：tract 把 ONNX 的 GRU 转成 `GruSeq` 时设了 `reset_every_turn: false`（`tract-onnx-0.23.9/src/ops/rec/gru.rs:72-74`）。这样一来，除了第一次调用，GRU 都忽略输入的初始状态，接着用上一次的末状态（`tract-core-0.23.9/src/ops/gru_seq.rs:180-185`）。GTCRN 频率方向的 GRU 每帧都要从零开始，所以出错
   - 办法：每帧新开一个状态，就和 ORT 一致（130.4 dB）。代价见上一节；设 `EXP010_TRACT_REUSE=1` 可以复现错误
   - 要不要报给 tract 上游，待人定
2. **tract 不支持 FE-M 图里的 `SplitToSequence`**：
   - 办法：`tools/fem_strip_seq.py` 把 8 处「`SplitToSequence(axis 0, keepdims 0)` 接 `SequenceAt(常数 0)`」换成等价的 `Gather`
   - 改后的图在主目录 `_tmp/20261102-exp010/fastenhancer_m_noseq.onnx`
   - 已证明改写等价：ORT 跑原图对 ORT 跑改写图，逐位相同
3. **别的会话的重活干扰计时**：r1 有 miri 并行；r2、r3 和别处一次编译重叠（16:31–16:32），r2 有一轮 ORT 最大值到了 62.9 ms。这几版只作参考，不进结论；原因写在各自的 `env.txt` 里
4. **r4 → r5b 口径变了**，所以 r4 的数字不再用：
   - 加了调度等级，帧耗时多报 p10 / p90
   - 「打开」不再包含 ORT 环境初始化：r4 里 ORT GTCRN 打开 18 ms，r5b 是 10 ms 加上单列的 17 ms 环境
   - tract FE-M 的「打开」不再含 ORT 建会话
   - 手写版换成了去分支后的版本
   - r4 的结论方向和 r5b 一致
5. **exp004 量的 ORT p50 是 0.70 ms，本轮是 0.38 ms**：
   - 主要原因推断是机器状态不同。依据：exp004 的 FE-M 计时不含外部 STFT，也从 0.67 ms 降到 0.38 ms，降幅和 GTCRN 一样
   - exp004 计时包含 GTCRN 的 STFT（`src/models/src/gtcrn.rs:88-104`；计时取 `src/engine/src/engine.rs:270-272` 的 `last_model_time`），但这部分占多少没量
6. **r5 刚开头就因脚本 bug 退出了**：`ps | head` 在 `pipefail` 下触发 SIGPIPE，没跑任何对拍或基准，目录里只有一份注明原因的 `env.txt`。正式版是 r5b

## 耗电

`tools/power_run.sh` 要 sudo，由人来跑：

```sh
sudo bash docs/experiments/exp010-inference-bench/tools/power_run.sh 20261102-p1
```

- 跑法：按实时节奏，每个后端各跑 60 秒，同时用 `powermetrics` 采 CPU 功率；另加 60 秒空闲作对照。约 8 分钟
- 结果写进 `results/20261102-p1/power/summary.txt`
- 每帧耗电 ≈（后端平均功率 − 空闲平均功率）× 帧间隔
- 〔尚未跑，待人〕

## 转交（超出 rustbe 范围，报大总管）

手写版要进产品（`src/models`），自审列出还缺这些（`review-native` 的 P1、P2）：
1. **过不了 `vdn-models` 的禁 panic 规则**：clippy 报 142 处越界检查；构造代码和实时代码混在同一个 `impl` 块里
2. **缺接入件**：没有 `Model` 适配器（STFT 分析 → 推理 → 合成），没有 `AnyModel` 变体，没有进 CI 的测试（对拍、零分配、reset、故障注入）
3. **输入没有防护**：入口不检查非有限值和长度。NaN 进来后会一直留在循环状态里，直到 `reset` 才清掉
4. **权重在仓外**：加载时不校验 sha256。权重只有约 95 KB，GTCRN 是 MIT 许可，可以考虑直接编进程序
5. **对拍的参考是 ORT，不是 PyTorch**：要宣称「和原模型等价」，还要补一次手写版对 PyTorch 流式版的逐帧对拍

## 待人拍板

每题写人能感知到的后果；不回就按默认。

1. **手写版要不要替换产品里的 ORT 来跑 GTCRN？**
   - 好处：每帧推理少 0.29 ms，约 1.8% 个核降到 0.55%；冷启动少约 28 ms
   - 代价：要多维护一份约 800 行的实现，权重格式也要从 ONNX 换成自导出的格式
   - 建议：替换，但先做完「转交」里的 5 件事
   - 不回默认：先不进产品，等苹果引擎那一侧的数字出来再一起定。后果：产品暂时还跑 ORT，CPU 不过 D69 的挡线
2. **FE-M 还要不要当候选？**
   - 现状：FE-M 在 ORT 上约占 3.8% 个核，超过 D69 的 2% 上限。要留，就得给它也手写一份，工作量和 GTCRN 相当
   - 不回默认：先不做，只做主模型 GTCRN。后果：备选模型暂时不达标
3. **要不要把 tract 的 GRU 状态 bug 报给上游？**
   - 这是对外发言，要你同意；报之前会先做一个最小复现
   - 不回默认：不报。后果：我们这边每帧新开状态照样能用，只是上游不知道这个问题
