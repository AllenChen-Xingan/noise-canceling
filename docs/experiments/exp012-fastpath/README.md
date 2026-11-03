# exp012 · 手写 GTCRN 还能快多少（研究 020 的实测）

> 意图（人原话）：「所以我们手写的引擎，有什么东西可以更好的加速吗 / 就是说，从数学上证明 / 来看，这个东西是否是数学上也最好的优化方法 / 如果不是，这个引擎就是可以更好的」
> 手下 fastpath · issue #58 · 分支 `exp/012-fastpath`。结论与取舍见 `docs/research/020_fastpath/decision.md`；改写的数学依据见 `docs/research/020_fastpath/rewrites.md`。
> 名字：**原写法** = exp010 的手写版（后端 `native`）；**v2** = 同一结构与权重、算子用 NEON 重写（`native-v2`）；**v2-fast** = v2 + 向量化多项式非线性（`native-v2-fast`）。

## 结果（待插电重跑，`results/20261102-v2ac/`）

（计时完成后填）

## 第二段第一次（`results/20261102-v2/`）：中途拔电、系统睡眠，只作电池对照

- ⚠️ 约 20:07 电源被拔（`pmset -g log` 自 20:07:02 起「Using Batt」），20:12:13 因空闲进入系统睡眠，20:54:17 被键盘唤醒。证据：round 2 · v2-fast · 节奏那一条单调时钟 `wall_s` = 95.98 秒（正好 6000 × 16 ms），墙钟窗口 `win_end_unix − win_start_unix` = 2605 秒；macOS 的单调时钟在睡眠时停走。那一条 `max_ms` = 296 ms 的帧开始于 20:12:05，压在进入睡眠的那一刻
- 不是程序错误。已加的防线：bench / gtcrn-layers / dsp-bench 发现墙钟比单调时钟多走 > 1 秒即判本轮作废（`SleepGuard`，测试 `sleep_guard_flags_wall_clock_jump`）；脚本自己 `caffeinate -dims`，开跑前与每轮前查插电
- 电池模式下性能核约降到一半频率：hw-peak 性能核 FMA 吞吐 round 1（插电）66.9 → round 2/3（电池）35.4 个/ns，FMA 延迟 0.81 → 1.53 ns；原写法连续跑 0.095 → 0.18–0.22 ms
- 所以只有 round 1 是插电数据（单轮，不下结论）；round 2/3 只能看同一轮内交替的相对比值

| 后端 | round 1 连续跑（插电） | round 1 节奏 16 ms 平均（插电） | round 2 连续跑（电池） | round 3 连续跑（电池） |
|---|---|---|---|---|
| native | 0.0946 ms | 0.511 ms | 0.2245 ms | 0.2019 ms |
| native-v2 | 0.0380 ms（原写法的 0.40） | 0.185 ms（0.36） | 0.0719 ms（0.32） | 0.0726 ms（0.36） |
| native-v2-fast | 0.0223 ms（0.24） | 0.130 ms（0.25） | 0.0424 ms（0.19） | 0.0423 ms（0.21） |

## 第一段：本机峰值与逐层计时（`results/20261102-peak1/`）

- 机器：Apple M5（4 性能核 + 6 能效核）· macOS 27.0.1 · 插电 · 计时名额独占（锁 owner = e3/fastpath）
- ⚠️ 开跑时系统负载 5.68（另一位大总管的手下还在收尾，大总管已叫停），跑完 1.05。这一段的数要在第二段复测对照后才用
- ⚠️ 这一段的两项微基准作废：
  - 非线性的「吞吐」：第一版通过函数指针调用，编译器不能内联，量到的是调用开销（第二段已改成泛型）
  - L1 读带宽约 30 GB/s：明显偏低，原因未查明；第二段加了另一种读核对照

| 量 | 性能核（`--qos rt`，3 轮） | 能效核（`--qos bg`，3 轮） | 测法 |
|---|---|---|---|
| NEON FMA 吞吐（f32 乘加 / ns） | 67.3 – 69.0 | 11.6 – 21.9 | 4–32 条独立累加链取最大；每轮 9 次取中位 |
| 向量 FMA 依赖延迟（ns） | 0.762 – 0.796 | 4.26 – 4.56 | 一条 vfmaq 依赖链 800 万次 |
| 向量加依赖延迟（ns） | 0.562 – 0.583 | 2.14 – 2.31 | 同上 |
| sig_fast 依赖延迟（ns，含调用） | 16.9 – 17.7 | 71.3 – 81.4 | 标量依赖链 200 万次 |

推断（未单独证实）：能效核 FMA 延迟约为性能核的 5.7 倍，按 FMA 延迟 4 拍折算，能效核约 0.9 GHz。

手写版（原写法）整帧，每轮 6000 帧、预热 200 帧，含 44 个打点（打点开销未单独量）：

| 跑法 | 第 1 轮平均 | 第 2 轮 | 第 3 轮 | p50（3 轮） |
|---|---|---|---|---|
| 连续跑，rt | 93.1 µs | 91.4 µs | 91.1 µs | 90.0 – 92.4 µs |
| 16 ms 节奏，rt | 380.9 µs | 504.0 µs | 549.6 µs | 303 – 618 µs |
| 16 ms 节奏，bg | 588.4 µs | 587.5 µs | 558.7 µs | 611 – 621 µs |

- 连续跑与 exp010 的 88 µs 同量级（多出约 3–5 µs，含打点）
- **按实时节奏，rt 与 bg 接近**：rt 的 p50 3 轮里 2 轮 ≈ 615 µs，与 bg 的 611–621 µs 几乎相同。推断：每帧只算约 0.1 ms 就睡 16 ms，调度器把线程留在能效核、低频上（与 D74「降噪线程用低频就行」一致）。所以**产品里真正要压的是「能效核低频下的每帧工作量」**，下界也按能效核算

## 怎么跑

```
cd docs/experiments/exp010-inference-bench/bench && cargo build --release
docs/experiments/exp012-fastpath/tools/run_peak_layers.sh <目录名> 3 native   # 第一段
docs/experiments/exp012-fastpath/tools/run_v2.sh <目录名> 3                    # 第二段
python3 docs/research/020_fastpath/tools/bound_table.py docs/experiments/exp012-fastpath/results/<目录名> --qos-peak rt --layers-qos rt --pace 0
```

## 测试

- 单元测试（`cargo test --release`，24 个，不需要权重）：计算量统计与模型实际参数 / 参照循环交叉校验（4 条）· v2 每个算子对原写法（最大差 < 1e-5，GRU 连跑 33 步）· 向量 exp 对 libm（相对误差 < 1e-6）· 重采样对照版对 vdn-dsp（差 < 1e-5）· 打点不改输出（逐位相同）· bench 输出计时窗字段
- 端到端（`cargo test --release -- --ignored --test-threads 1`，要仓外权重）：5 个手写后端各自对 ORT 过 BE-PARITY + 复位检查

| 后端 | 最大差 | SNR | 最小帧 SNR | BE-PARITY | 复位 |
|---|---|---|---|---|---|
| native | 1.10e-5 | 134.00 dB | 99.21 dB | 过 | 逐位一致 |
| native-fast | 1.10e-5 | 133.40 dB | 96.81 dB | 过 | 逐位一致 |
| native-v2 | 8.58e-6 | 134.99 dB | 98.78 dB | 过 | 逐位一致 |
| native-v2-fast | 1.43e-5 | 132.72 dB | 96.90 dB | 过 | 逐位一致 |

（mix.wav 610 帧；native 改了沿频率 GRU 的循环顺序后，仍是 134.00 dB、与改前相同）

- 每帧分配（500 帧，`--count-malloc`）：native-v2、native-v2-fast 的 Rust 分配与全进程 malloc 都是 0
