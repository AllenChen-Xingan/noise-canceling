# 现状页 · 手下 rs-live（批次 20261102-b8-rust）

> 大总管：`noise-canceling-47`。交代：`../noise_canceling-skeleton/_tmp/20261102-b8-rust/rs-live.md`。
> 管的实体：`vdn-io-macos` · `vdn-rt-thread` · `vdn-denoised` · `vdn-cli`

## 已收尾（2026-11-02）
一句话：V5 这条路径用不开设备的测试证明没丢声音（两声道相同、电平差 < 0.2 dB）；嫌疑落在「声音太小 + 骨传导盖住」，
新加的 `--check --record` 让人回放判。实时优先级 code 4 已修（内核 50 µs 下限）。

**验证【测】**：`cargo test -p vdn-io-macos -p vdn-rt-thread -p vdn-denoised -p vdn-cli` 70 过 · 5 忽略（真设备 / 人工）；
clippy（--all-targets）0 警告 · fmt · `xtask tidy` · `xtask check-deps` 通过。`--gain-db 25`、录音路径不存在都在开设备前报错（实跑确认）

## 给人的 V5 复测（戴耳机）
```
cargo build -p vdn-cli && mkdir -p _tmp/v5
./target/debug/vdn run --passthrough --check --record _tmp/v5/v5-$(date +%H%M%S).wav
afplay _tmp/v5/v5-<上面那个时间>.wav        # 用耳机回放
```
- 跑的 5 秒里正常说话。文件名带时间，不会覆盖上一次的
- 算过：① 结束时最后一行是「过（机器部分）」；② 回放里清楚听到自己的声音、不断续、没有咔哒声
- 如果判定行提示「输入很小声」：再跑一次加 `--gain-db 12`，回放对比
- 跑的时候耳机里听不出自己是可能的（骨传导的原声先到，22 ms 后的小声副本被盖住）——以回放为准
- 麦克风单测（有环境声或说话时跑）：`cargo test -p vdn-io-macos --test real_device -- --ignored --nocapture`

## 本次新增的测试
- vdn-io-macos：`level::tests` 4 条（正弦电平对理论值 · 连续全零跨回调并在有声后清零 · −0 与非有限值 · 空计数）；`pump::tests::pumps_meter_mono_level`、`all_zero_input_flags_silent_after_one_second`；`real_device` 两条加断言（麦克风不全零 · 扬声器送出约 −20 dBFS）
- vdn-rt-thread：`tiny_measurement_gives_params_the_kernel_accepts`（回归：修前给 30 µs，内核返回 4）· `huge_measurement_is_clamped_to_kernel_max_and_period` · `for_block_with_tiny_load_is_valid` · `period_shorter_than_kernel_min_is_rejected_not_rewritten` · `kernel_raises_small_computation_to_half_constraint`；`invalid_params_are_rejected` 加上下限两例
- vdn-denoised：`denoise_rt_params_use_period_and_measured_window` 改为与正本比对 + 内核接受
- vdn-cli：`run_path_without_devices_passes_sine_through_at_same_level` · `run_path_gain_raises_output_level` · `run_path_gain_clips_and_counts` · `gain_range_is_checked` · `record_queue_full_drops_and_counts` · `judge_cases` · `silence_notice_fires_once_per_episode`；`v5_live_passthrough`（忽略，人工）改为带录音与判定

## 自审（三个视角）
- 实时回调：电平统计只用原子量、单写者、不分配；录音分流用 rtrb `push`，满了丢弃计数不等待；`no_alloc` 测试覆盖两个回调泵（含电平统计）
- 提示刷屏：一段连续全零只提示一次，有声后再全零满 1 s 才再提示，最多每秒一次（单测）
- 夹取会不会更易降级：下限把 < 50 µs 抬到 50 µs，原来是整个被内核拒收，现在能提升；内核本来就会抬到 constraint/2（8 ms），声明值不会比实际工作量小。上限只在周期 > 50 ms 时起作用，本项目周期 16 ms 碰不到

## 查明的事
1. **输出两个声道都写了**【读源码】：`OutputPump` 经 `fan_out` 复制到每个声道；送音端（Forwarder → Out 环 → HalSource）没有会输出全零的地方
2. **麦克风有声**【大总管实测 18:20】：ffmpeg 与 `vdn-capture --source mic` 都录到声音（峰值约 −28 dBFS、均值约 −39 dBFS）；终端有麦克风权限
3. **推断（未核实，待人回放判）**：自己的声音靠骨传导先到，22 ms 后一个峰值 −28 dBFS 的小声副本听不出来。验证办法：`--record` 录下送进耳机的那一路，人事后回放
4. **code 4 = 内核拒收**【读 xnu 源码】：`osfmk/kern/thread_policy.c` 要求 `min_rt_quantum ≤ computation ≤ max_rt_quantum` 且 `computation ≤ constraint`；`osfmk/kern/sched_rt.c` 定 `min_rt_quantum = 50 µs`、`max_rt_quantum = 50 ms`。直通管线每周期实测几十微秒，× 1.5 仍低于 50 µs。链路测试里同样失败，但只计数不打印，所以以前没人看见
5. 内核还会把小于 `constraint / 2` 的 computation 自动抬到 `constraint / 2`（不报错）【读源码】：D68 声明的「computation ≈ 6 ms / 周期 16 ms」实际会被抬到 8 ms——**D68 实际按 8 ms / 16 ms 生效**（大总管认可写明）。新测试 `kernel_raises_small_computation_to_half_constraint` 钉住读回值

## 改了什么（清单：每条「做了什么 → 怎么证明」）
- `vdn-io-macos`：新模块 `level.rs`（`LevelMeter`：峰值 · 均方根 · 连续全零长度，原子量、单写者、不分配）；两个回调泵各量单声道电平；`StreamHealth::level()` / `silent_now()`（连续 1 s 全零）→ 单测 4 条（level）+ 2 条（pump）；`no_alloc` 测试照旧覆盖回调泵
- `real_device` 测试：麦克风 0.5 s 不许全零；扬声器测试顺带查送出电平约 −20 dBFS
- `vdn-rt-thread`：`MIN_COMPUTATION_NS = 50 µs`、`MAX_COMPUTATION_NS = 50 ms`；`for_block` / `from_measured` 都把 computation 夹进 [下限, min(上限, constraint)]；`validate` 改 pub、先在本 crate 拦下越界值 → 测试 5 条新增（极小 / 极大实测真提升、for_block 极小负载、周期过短报错不改写、读回抬到 constraint/2）
- `vdn-denoised`：`denoise_rt_params` 测试改成和正本 `from_measured` 比对 + 断言内核接受
- `vdn-cli`：
  - `live.rs` 拆成不开设备的 `prepare` · `input_callback` · `output_callback` · `denoise_loop`；`run` 只多了开设备
  - `--record <wav>`：输出回调经 rtrb（2 s）送样本给写文件线程，满了丢弃并计数
  - `--gain-db`（−60..=+20，默认 0）：加在送进耳机之前，超过 ±1 夹住并计数
  - `--check`：不给 `--seconds` 跑 5 s，结束判定（输入全零 / 没输入 / 直通时输出电平偏离「输入 + 增益」超过 ±3 dB → 不过，退出码 1）；输入峰值低于 −30 dBFS 时提示「很小声」
  - 运行中输入连续 1 s 全零立即提示权限位置；一段全零只提示一次
  - 结束时打印输入 / 输出的峰值、均方根（dBFS）
  - 链路测试三个线程 + `vdn run` 降噪线程：提升失败一律打印（`chain::promote_or_report`）
  - 测试：`run_path_without_devices_passes_sine_through_at_same_level`（同一套回调、2 声道输出，判两声道相同 · 相关 > 0.999 · 电平差 < 0.2 dB · 录音 = 送出的）· 增益 +12 dB · 增益夹取 · 增益范围 · 录音队列满 · 判定 6 种情形 · 提示节流；V5 人工测试改为带 `--record`

## 新增的第二处表示及其校验
| 第二处 | 正本 | 校验 |
|---|---|---|
| 「连续全零 1 s」的样本数 | `vdn_io_macos::SILENT_RUN_SAMPLES = IO_RATE` | 直接由 `IO_RATE` 导出，不手写 48000 |
| 降噪线程实时参数 | `vdn_rt_thread::RtParams::from_measured` | `vdn run`、`vdn-denoised`、chain-test 都经 `vdn_denoised::denoise_rt_params`；单测比对二者输出相同 |
| 内核 50 µs / 50 ms 限值 | xnu 源码（外部） | 真提升测试 `tiny_…` / `huge_…`：内核接受 = 过 |

## 待人拍板
（无）

## 依赖
- `vdn-cli` 新增直接依赖 `rtrb`（workspace 已有，0.4.1，MIT OR Apache-2.0，与 GPL-3.0 兼容）
- 不加 `objc2-avf-audio`（大总管定：权限已排除，不为它破 unsafe 禁令）

## 流水
- 18:1x 报到；读代码定位：输出声道与直通路径无嫌疑；code 4 定位到内核 50 µs 下限
- 大总管实测麦克风有声；定：不加 AVFoundation、链路测试失败要打印、加 `--record` / `--gain-db`
- 代码写完，等锁编译
- 大总管发「可以跑」：四个 crate 测试 / clippy / fmt / tidy / check-deps 全过；收尾
