# exp010 · 苹果引擎测量程序（applebe）

用途：在 Core ML（CPU / GPU / 神经引擎 / 全部）、BNNS Graph、MLX 上，按 rustbe 基准程序同口径测 GTCRN 与 FE-M，并与 ORT 输出逐值对拍。基准程序本体（ORT 基线、手写 Rust、tract）在 `../bench/`，归 rustbe。

## 同口径（与 `../bench/` 一致）
- 输入与参照：rustbe `gtcrn-bench ort <onnx> <wav> --dump-io <前缀>` 写出的 `<前缀>.in.f32` / `.ort.f32` / `.json`。小端 f32、按帧拼接；GTCRN 每帧 re[257] 再 im[257]，FE-M 每帧 160 个样本
- 帧：第 0 帧冷启动，200 帧预热，6000 帧计时；wav 是主目录 `resources/gtcrn/test_wavs/mix.wav`（610 帧，不够就循环）
- 逐值一致：前 610 帧（整段不循环），最大绝对差 · 整段 SNR · 首个差 > 1e-3 的位置
- 字段：`p50_ms p99_ms p999_ms max_ms mean_ms open_ms first_frame_ms cpu_over_wall nonfinite_frames` 同 rustbe；分配次数叫 `process_allocs_per_frame`（全进程 malloc 次数，钩子是 libmalloc 的 `malloc_logger`；**神经引擎守护进程里的分配看不到**），与 rustbe 的 `rust_allocs_per_frame`（只数 Rust 分配器）不是同一个量
- `cpu_over_wall` 只算本进程 CPU；神经引擎与 GPU 的工作不计入，耗电要看 `powermetrics`

## 文件
| 文件 | 用途 |
|---|---|
| `convert_gtcrn.py` | PyTorch 流式 GTCRN → Core ML：fp32 / fp16 显式缓存版 + 有状态 fp16 版 |
| `bench_coreml.swift` · `alloc_hook.{c,h}` · `build.sh` | Core ML 逐帧计时 + 逐值对拍（`bench`），算子落点（`plan`，用 `MLComputePlan`） |
| `power.sh` | 耗电测量命令，要 sudo，**由人跑** |

## 怎么跑（要名额）
```sh
V=../../../../../noise_canceling/_tmp/exp010-venv     # 主目录 venv
O=../../../../../noise_canceling/_tmp/exp010-apple    # 转出的模型与转储，不进库
$V/bin/python convert_gtcrn.py ../../../../../noise_canceling/resources/gtcrn/stream $O
./build.sh
for u in cpu gpu ane all; do build/bench_coreml bench $O/gtcrn_fp32.mlpackage $u $O/gtcrn.in.f32 $O/gtcrn.ort.f32; done
build/bench_coreml plan $O/gtcrn_fp16.mlpackage ane 2> $O/plan_fp16_ane.tsv
```
结果一行一条写进 `../results/apple_*.jsonl`（每次跑带机型、系统、电源状态）。
