# exp010 · applebe build 20261102-173000-b2 小结

> 原始数据：同目录 `bench.jsonl`（每行一个组合一轮）· `plan.jsonl`（MLComputePlan）· `machine.jsonl` · `convert_*.json`。大文件在主目录 `_tmp/exp010-apple/20261102-173000-b2/`，哈希见 `artifacts.sha256`。
> 条件：Apple M5 · macOS 27.0.1 (26A434) · 插电 · 名额独占（大总管 17:14 起发给 applebe）· 计时线程 QoS user-interactive · 1 冷启动 + 200 预热 + 6000 计时 · 3 轮交替 · ORT 与手写 Rust（rustbe 的二进制）同场跑。
> 调用方：除标「Python」的两行外，全部从 Rust 调（`apple-rs`，D68）。单位 ms。分配：`p_alloc` = 全进程 malloc 次数 / 帧（malloc_logger 钩子，自检通过）；ORT 和手写版用 rustbe 程序，只有 Rust 分配器次数（`r_alloc`）。

## GTCRN

| 组合 | p50 | p99.9 | max | 每帧分配 | 与 ORT 的 SNR / 逐帧最小 SNR | BE-PARITY |
|---|---|---|---|---|---|---|
| ORT 1.31 CPU（基线） | 0.382–0.386 | 0.418–0.421 | 0.429–0.453 | r 0 | — | — |
| 手写 Rust（rustbe） | 0.088 | 0.109–0.110 | 0.113–0.125 | r 0 | 134.0 dB（rustbe 对拍） | 过 |
| **BNNS Graph · GRU 展开 · fp32** | **0.154–0.157** | **0.177–0.194** | **0.185–0.245** | **p 0.000（整数待验证）** | 133.9 / 106.3 dB | **过** |
| BNNS Graph · fp32 | 0.334–0.343 | 0.366–0.615 | 0.374–0.898 | p 0.000（整数待验证）| 133.9 / 106.3 dB | 过 |
| BNNS Graph · 多线程编译 · fp32 | 0.336–0.341 | 0.368–0.421 | 0.384–0.431 | p 0.000（整数待验证）| 133.9 / 106.3 dB | 过 |
| Core ML CPU · GRU 展开 · fp32 | 0.208–0.209 | 0.226–0.446 | 0.266–0.612 | p 302 | 133.9 / 106.3 dB | 过 |
| Core ML CPU · fp32 | 0.389–0.397 | 0.423–0.950 | 0.942–2.019 | p 302 | 133.9 / 106.3 dB | 过 |
| Core ML GPU · fp32 | 0.386–0.391 | 0.418–0.424 | 0.484–0.764 | p 302 | 133.9 / 106.3 dB | 过 |
| Core ML 神经引擎 · fp16 | 0.381–0.383 | 0.410–0.412 | 0.448–0.535 | p 340 | 54.1 / 26.5 dB | **不过** |
| Core ML 全部 · fp16 | 0.381–0.387 | 0.410–0.420 | 0.541–0.582 | p 340 | 54.1 / 26.5 dB | **不过** |
| Core ML CPU · fp16 | 0.375–0.377 | 0.404–0.719 | 0.439–1.073 | p 302 | 54.1 / 26.5 dB | **不过** |
| MLX CPU（Python 调，对照） | 5.70–6.20 | 7.52–8.52 | 10.3–11.2 | p ≈ 72 000 | 134.9 dB | 过 |
| MLX GPU（Python 调，对照） | 6.18–6.20 | 7.39–9.00 | 9.49–11.9 | p ≈ 100 000 | 134.6 dB | 过 |

Core ML GPU / 全部 · GRU 展开 · fp32 另跑了 3 轮，在同一 `bench.jsonl` 末尾。

## 读法（结论强度：本机 3 轮，自审未做）

1. **苹果引擎里最快的是 BNNS Graph + GRU 展开**：p50 约 0.155 ms，是 ORT 的约 2.5 倍快（0.384 ÷ 0.155），每帧分配显示 0.000（三位小数，整数待验证），与 ORT 逐值一致。仍比手写 Rust（0.088 ms）慢约 1.8 倍。
2. **GRU 展开是关键**：同一个 BNNS，不展开 0.34 ms，展开 0.155 ms；Core ML CPU 不展开 0.39 ms，展开 0.21 ms。印证调研说的「coremltools 把 GRU 翻成逐步循环」（`notes/coreml.md`）。
3. **Core ML 的 GPU / 神经引擎对 GTCRN 没用**：三种设置下，被指派了计算设备的算子全部落在 CPU（`MLComputePlan`；fp32 版 322 个在 CPU、0 个在 GPU / 神经引擎，另有 1785 个没有设备归属——`deviceUsage` 返回空，设备归属未知；采集代码只记「返回空」，没记它们是什么算子，见 `apple/bench_coreml.swift:112–124`）（`plan.jsonl`）；所以 GPU / 神经引擎几行的耗时和 CPU 一样。fp16 版逐值一致不过（54 dB < 60 dB），且第 0 帧第 0 个值就差 > 1e-3。
4. **Core ML 每帧全进程 malloc 约 300 次**，过不了 `BE-RT-ALLOC`（0）；BNNS Graph 显示 0.000 / 帧（三位小数，整数待验证），与苹果说法和调研一致。
5. **多线程编译 BNNS 没有提速**（0.34 vs 0.34），小模型不值得开线程。
6. **MLX（Python 调）比 ORT 慢约 15 倍，每帧 7–10 万次分配**。Rust 经 mlx-c 的版本还没做（见下），按 D68 这两行不进结论；但 Python 只能解释其中一部分，MLX 每帧建图和 `eval` 的分配大概率仍在【推断】。

## ⚠️ 按实时节奏跑：每帧慢约 5 倍（`deadline_bnns_unrolled.json`）

BNNS Graph · GRU 展开 · fp32，`--pace-ms 16`，37 500 帧（10 分钟），QoS user-interactive：

| 量 | 连续跑（上表） | 实时节奏 |
|---|---|---|
| p50 | 0.155 ms | **0.82 ms** |
| p99.9 | 0.18–0.19 ms | 1.57 ms |
| max | 0.19–0.25 ms | **11.8 ms**（> 3.0 ms，`BE-DEADLINE` 不过；这一版程序还没数超时帧个数，已补字段 `frames_over_3ms`） |
| cpu_over_wall | — | **0.048 核**（超 D69 上限 0.02 核） |
| 每帧分配 · 常驻内存增量 | 显示 0.000（整数待验证）· 9.0 MB | 显示 0.000（整数待验证）· 9.0 MB（跑完不涨；含基准自己的留存缓冲，见 standard `BE-MEM`） |

推断原因：帧间空闲时 CPU 降频，或线程被调度到能效核；QoS user-interactive 不够。**连续跑的数会低估实时开销**，所以 `BE-CPU` 只认实时节奏的数（标准里本来就这么写）。下一步：试实时线程策略（`THREAD_TIME_CONSTRAINT_POLICY` / Audio Workgroup），手写 Rust 与 ORT 也要按实时节奏重测（已告诉 rustbe）。

## DFN3 `_ll` 算力下界（`dfn3ll_probe.json`，Python 调 ORT，只作估算）

官方三张子图取 S = 1、单线程、连续跑：enc 0.120 + erb_dec 0.199 + df_dec 0.292 = **0.61 ms / 10 ms 帧 ≈ 6.1% 核**，是 D69 推理挡线（1.0%）的 6 倍、目标（0.5%）的 12 倍。这还是下界：不含 STFT / ERB / DF，不含实时节奏的 5 倍效应，GRU 没带状态。即使苹果最快后端提速 2.5 倍也约 2.4%。~~结论【推断】：DFN3 `_ll` 在 D69 预算里跑不动。~~ **这条推论已撤回**（codex 第 1–3 轮）：0.61 ms 是 Python 调 ORT、没有这个模型的 Rust 差值校准；别的模型的提速倍数不能套过来；D75 后 CPU% 不挡。数值照留，结论改为「实时链暂缓（时间盒），离线盲听照做」，见 `docs/research/019_apple-inference/decision.md` 第 2 节。

## 没做成 / 绕过的

- **FE-M 转不了 Core ML**：wav2wav 图里的 `torch.view_as_complex`（STFT）coremltools 9.0 不支持。要走 spec2spec 版并在 Rust 做 STFT，还要另做 ORT 转储。FE-M 是否还在候选里待人定（rustbe 待拍板），本轮不做。
- **有状态版（MLState）转不了**：coremltools 报 `No matching select or slice.`。
- **coremltools 的三处坑**（都已在转换脚本里绕过，注释写明）：① numpy 2.5 上单元素数组转标量报错（`apple/ct_compat.py`）；② Python 3.13 上 MIL 管线崩（GIL 致命错误），改用 3.12 的转换专用 venv；③ **分组 ConvTranspose2d（groups=2）转出来是错的**，逐块对拍定位到解码器第 4 层差 3.46，拆成每组一个再拼接后对 ORT 差 2.4e-7。另外缓存的原地切片写回也改成了拼接（单独改它不够，③ 才是主因）。
- **MLX 的 Rust 路**：要先编 mlx-c（本机没有 Metal 编译器，只能对着 pip 的 libmlx 编），本次名额内没做。
