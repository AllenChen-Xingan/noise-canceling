# exp010 · 推理后端基准（降噪模型在 Mac 上用什么跑最快）

> 意图：人原话（D53）「必须用 core ml 或者 mlx 就是所有的推理引擎如果现在的不好，我们要重新写，用rust 或者用苹果的引擎重新写，必须达到最好的效果，最快的效果」。
> 本实验分两边：**纯 Rust 一侧**（本文，手下 rustbe，issue #51）和**苹果引擎一侧**（Core ML / BNNS Graph / MLX，手下 applebe，写进 `docs/research/019_apple-inference/`）。两边用同一个基准程序和同一份输入，数字可以横比。
> 基准程序 `bench/` 归 rustbe 所有；字段改动先通知 applebe。

## 一句话结论（纯 Rust 一侧）

〔见下方「结果」一节，干净版数字出来后填〕

## 怎么跑

```sh
cd docs/experiments/exp010-inference-bench/bench && cargo build --release
B=target/release/exp010-bench
M=<主目录>                                    # 权重、ONNX、音频都在仓外
$B gtcrn-parity $M/_tmp/20261031-exp010/gtcrn_dns3 $M/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx $M/resources/gtcrn/test_wavs/mix.wav --vs native   # 或 --vs tract
$B gtcrn-bench native $M/_tmp/20261031-exp010/gtcrn_dns3 <wav>        # ort / tract 时第 2 个参数给 onnx
$B fem-parity ~/vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx <wav> --tract-onnx $M/_tmp/20261102-exp010/fastenhancer_m_noseq.onnx
$B fem-bench ort|tract <onnx> <wav>
bash ../tools/run_bench.sh <build-id> [轮数]    # 全套：3 个对拍 + 每轮 5 个基准，交替跑；结果写 results/<build-id>/，不覆盖
sudo bash ../tools/power_run.sh <build-id>       # 耗电（要 sudo，由人跑，见「耗电」）
```

所有子命令都输出一行 JSON。参数和字段的正本是 `bench/src/main.rs` 顶部注释。

## 口径（两边共用）

- **输入**：`resources/gtcrn/test_wavs/mix.wav`（16 kHz 单声道，610 帧）。基准跑 1 帧冷启动、200 帧预热，再计时 6000 帧；音频不够长就循环，STFT 中途不重置。对拍用整段音频，不循环
  - GTCRN 的一帧是同一个 STFT（`vdn-dsp`，512 点 / hop 256，sqrt-hann）出来的 257 个复数频点，16 ms 一帧
  - FE-M 的一帧是 160 个样本，10 ms 一帧（图里自带 STFT）
- **对拍**（逐值一致，exp004 口径）：同一份输入，ORT 和被测后端各从零状态顺序跑完所有帧，比较每帧的全部输出值。报三项：最大绝对差；整段 SNR，即 10·log10(Σref² / Σ差²)；第一个差值 > 1e-3 的位置（`first_bad`）。**`first_bad` 不为 None 的「更快」不算数**
- **单线程**：ORT 设 intra / inter = 1、不自旋、`PreferCPU`、`ORT_ENABLE_ALL`，和 `src/models/src/session.rs` 一样；tract 不设线程池，默认单线程；手写版只用一个线程
- **字段**：
  - `p50_ms` / `p99_ms` / `p99.9_ms` / `max_ms` / `mean_ms`：只算模型推理那一次调用的耗时
  - `open_ms`：加载模型和建会话的时间；`first_frame_ms`：第一帧耗时。两者合起来就是冷启动
  - `cpu_over_wall`：计时段内 CPU 时间除以墙钟时间。连续跑时约等于 1，表示只用了一个核
  - `rust_allocs_per_frame`：Rust 全局分配器的次数。ORT 在 C++ 里的分配不算；tract 和手写版是纯 Rust，这个数就是全进程的分配次数。applebe 用 malloc 钩子数全进程分配，字段名叫 `process_allocs_per_frame`，两者不混
  - `pace_ms`：大于 0 时按实时节奏每隔这么久喂一帧，测耗电用
- **转储**（给 applebe 的 Swift / Python 程序用）：加 `--dump-io <前缀>` 后，写出全部 6201 帧的输入和 ORT 输出（小端 f32），外加一份 json 记录 sha256

## 选哪个 Rust 推理库：tract

依据（子代理调研，2026-11-02；等级：源码 / 官方 / 公开实测）：

| 库 | 版本 | GRU | ConvTranspose | LayerNorm | 结论 |
|---|---|---|---|---|---|
| tract | 0.23.9（2026-10-15） | 有〔源码 `onnx/src/ops/rec.rs`〕 | 有 | 有 | **选它**：tract 自己的每晚基准里就有 GTCRN 流式单帧模型（`ci/benches.toml` 第 546–552 行），三个状态张量和我们的形状一样；M1 Max 单线程每帧 0.294 ms〔公开实测，不同机器〕；2026-08 至 10 月专门做了 GRU 融合〔官方 CHANGELOG〕 |
| rten | 0.27.0 | 有 | 有 | 有〔源码〕 | 第二候选；没有小模型的公开实测，本轮没测 |
| burn-onnx | 0.22.0 | 有 | 有 | 有〔官方算子表〕 | 把 ONNX 生成 Rust 代码；CPU 上小算子的性能没有数据，本轮没测 |
| candle-onnx | 0.11.1 | **无** | **无** | **无**〔源码 `candle-onnx/src/eval.rs`〕 | 跑不了 GTCRN，排除 |

- tract 的官方文档写明「每次推理都会分配内存」（`doc/allocator.md`），这一条违反实时规则（`specs/rust/architecture.md` 实时线程不许分配）。本轮已实测这个分配次数
- Accelerate / BNNS：
  - `accelerate-src` 只负责链接 Accelerate 框架
  - `apple-accelerate` 0.4.2 刚出 5 个月，每次调用都会分配内存
  - BNNSGraph 执行时不分配内存，但要先转成 Core ML，归 applebe 那一侧
  - GTCRN 的矩阵最大只有 48×16，调 BLAS 的开销很可能比计算本身还大〔推断〕
  - 所以手写版先只用编译器自动向量化，不接 Accelerate

## 手写 GTCRN（`bench/src/gtcrn_native.rs`）

- 照 `resources/gtcrn/stream/gtcrn_stream.py` 的 `StreamGTCRN.forward` 一层层手写，不依赖任何库
- 权重由 `tools/dump_gtcrn.py` 从 DNS3 checkpoint 导出：主目录 `_tmp/20261031-exp010/gtcrn_dns3.{bin,txt}`，bin 的 sha256 是 `e88502c4…5174`。它和 `gtcrn_simple.onnx`（sha256 `b4718df6…bb87`）同源
- 加载时把 BN 折进卷积；ERB 矩阵只存非零段
- 所有缓冲在构造时一次分配好，`infer` 每帧零分配、不加锁、不做系统调用（实测分配 0 次，见结果）
- 2026-10-31 由 engine 写完，本轮第一次编译和对拍，**没改一行就通过了**：
  - 37 段（mix.wav 加 exp004 的 36 段，含 clean / noisy / enroll）对 ORT：最差 SNR 119.2 dB，最大差 1.6e-4，没有任何值的差 > 1e-3
  - 数据：`results/parity_native_vs_ort.jsonl`

## 踩到的坑

1. **tract 复用同一个运行状态会算错**：同一个 `SimpleState` 连续跑多帧时，第 0 帧对，第 1 帧起就错（SNR 19 dB）。每帧新开一个状态就对了（130 dB），代价是 p50 多约 0.02 ms（2000 帧冒烟实测）
   - 原因没查清。推断是某个算子的会话状态跨帧保留了下来
   - 现在默认每帧新开；设 `EXP010_TRACT_REUSE=1` 能复现错误
   - 这个问题值得报给 tract 上游，待定
2. **tract 不支持 FE-M 图里的 `SplitToSequence`**：`tools/fem_strip_seq.py` 把 8 处「`SplitToSequence(axis 0, keepdims 0)` 加 `SequenceAt(常数 0)`」换成等价的 `Gather(axis 0, 下标 0)`
   - 遇到别的用法，脚本直接报错退出，不硬改
   - 改后的图在主目录 `_tmp/20261102-exp010/fastenhancer_m_noseq.onnx`
   - 等价性已实测：ORT 跑原图对 tract 跑改后的图，976 帧，SNR 110.8 dB，最大差 4.2e-6
3. **机器上有别的重活**：第一版（`results/20261102-r1-busy/`）测的时候，另一个大总管手下在 `../noise_canceling-rs-shm` 跑 `cargo miri test`，占约 1 个核，负载约 4–5。那一版只作参考；干净版见下

## 结果（正式版 r4，本机实测）

- 测量条件：M5（Mac17,3）· macOS 27.0.1 (26A434) · 插电 · rustc 1.99.1 · 2026-11-02 16:32
  - 跑前负载 2.25，跑后 2.04
  - 跑之前确认过，别的会话都没有在编译或跑 miri
- 每个后端跑 3 轮，交替进行；表里写的是 3 轮的范围，时间单位 ms
- 数据：`results/20261102-r4-clean/`（`env.txt` 记机器状态和所有文件的 sha256，`parity.jsonl` 是对拍，`bench.jsonl` 是计时）

**GTCRN**（每帧 16 ms 预算）

| 后端 | p50 | p99 | p99.9 | 最大 | 打开 | 第 1 帧 | 每帧分配 | 对 ORT 的 SNR |
|---|---|---|---|---|---|---|---|---|
| ORT 1.31 CPU（基线） | 0.37–0.38 | 0.41–0.45 | 0.42–0.59 | 0.43–1.00 | 17.7–18.1 | 0.54–0.56 | 0 | — |
| **手写 Rust** | **0.10** | 0.11–0.14 | 0.12–0.22 | 0.13–0.28 | 0.2–0.3 | 0.11 | **0** | 134.0 dB |
| tract 0.23.9 | 0.37–0.56 | 0.59–0.79 | 0.62–1.05 | 0.71–1.43 | 69–80 | 0.52–0.75 | 2247 | 130.4 dB |

**FastEnhancer-M**（每帧 10 ms 预算）

| 后端 | p50 | p99 | p99.9 | 最大 | 打开 | 第 1 帧 | 每帧分配 | 对 ORT 的 SNR |
|---|---|---|---|---|---|---|---|---|
| ORT 1.31 CPU（基线） | 0.36–0.38 | 0.40–0.43 | 0.44–0.67 | 0.56–1.03 | 9.9–10.0 | 0.50–0.53 | 7 | — |
| tract 0.23.9（改写图） | 0.58–0.68 | 0.71 | 0.73–1.08 | 0.86–1.35 | 37–42 | 0.76–0.83 | 1041 | 110.8 dB |

所有对拍的 `first_bad` 都是 None，即没有任何输出值和 ORT 差超过 1e-3。连续跑时 `cpu_over_wall` 在 0.995–1.0 之间，说明三个后端都只用一个核。

**读法**（判据都是上面的数字；每格只有 3 轮，**样本量小**；评测器只覆盖速度和逐值一致，声音好坏不归它管）：
- GTCRN：手写版的 p50 是 ORT 的 1/3.7（0.10 对 0.38 ms），最大值 ≤ 0.28 ms，占 16 ms 预算的 1.8%。冷启动从 18 ms 降到 0.3 ms，每帧零分配，符合实时规则
  - 推断：ORT 的时间主要花在逐个算子的调度上，GTCRN 本身的计算量很小
- tract 赢不了 ORT：
  - GTCRN 上 p50 打平到慢 1.5 倍，尾部更差
  - FE-M 上 p50 慢 1.6–1.8 倍
  - 每帧分配上千次，实时线程不能用
  - 公开实测里 tract 在 M1 Max 上是 0.294 ms，和本机 0.37–0.56 ms 对不上。可能是测法不同，比如它不算我们这种每帧新开状态的开销。没查
- FE-M 目前只有 ORT 和 tract 两个后端，ORT 最快。要像 GTCRN 那样快一截，得给 FE-M 也手写一份（本轮没做）

**旧数字作参考**（只作参考，不进结论）：
- `results/20261102-r1-busy/`：有 miri 在并行
- `results/20261102-r2-clean/`、`results/20261102-r3-clean/`：和别处一次 release 编译重叠（16:31–16:32），r2 第 2 轮 ORT 的最大值到了 62.9 ms
- 各版的 `env.txt` 里都写了原因。手写版在每一版里都是最快的
- exp004 量的 ORT p50 是 0.70 ms，本轮是 0.38 ms。原因有两个，没拆开量：
  - exp004 计的是整个 `process` 调用（含 STFT 和 iSTFT，读源码 `src/cli/src/main.rs:305`），本轮只计推理那一次
  - 两次的机器状态不同

## 耗电

`tools/power_run.sh` 要 sudo，由人来跑。它按实时节奏让每个后端各跑 60 秒，同时用 `powermetrics` 采 CPU 功率，另加 60 秒空闲作对照。〔尚未跑，待人〕

## 待人拍板

〔见文末，收尾时填〕
