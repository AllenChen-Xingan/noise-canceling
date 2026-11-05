# 方案：DFN3 低延迟版换前视更小的重采样器

> **状态：这批不做**（大总管定 2026-11-05，技术题），方案留着，记 backlog。第六节第 2 条已定为记账错，转 evalfix 修。

> 起因：草稿里 DFN3 低延迟版的延迟项 V-LAT = 45.4 ms，超否决线 45 ms 0.4 ms；声明延迟 284 样本里有 124 样本（7.75 ms）是本实验选的 soxr HQ 流式重采样的前视（`findings.md` 第三节）。大总管 2026-11-05 要求写成具体方案。

## 一、关键事实（先读）

- **产品链路的输入输出是 48 kHz**（`vdn_types::IO_RATE`）。GTCRN 是 16 kHz 模型，产品里要 48→16→48 两跳重采样；成绩单延迟项已经替它记了这两跳的预算，各 96 @48 k = 2 ms（`src/types/src/latency.rs` 的 `planned_platform`：`resample-down` · `resample-up`）【读源码】
- **DFN3 是 48 kHz 模型，放进产品不需要任何重采样**。本实验的 16→48→16 只是因为评测音频是 16 kHz；它的前视是评测带进来的，不是 DFN3 自己的【推断，由上一条直接推出】
- 所以 V-LAT 对 DFN3 **重复记了**重采样：一份在 `latency_samples` 里（soxr 7.75 ms），一份在平台预算里（4 ms）

## 二、换哪个

**换成产品自己的重采样器 `vdn_dsp::{Interpolator3, Decimator3}`**（`src/dsp/src/resample.rs`）：

| | soxr HQ 流式（现在） | `vdn-dsp` 3 倍重采样（方案） |
|---|---|---|
| 来源 | 第三方 Python 包 | 产品正本（唯一真相源，不另抄系数） |
| 结构 | 线性相位，内部攒大块 | 线性相位 FIR，97 抽头，按块因果处理 |
| 每跳延迟 | 前视合计 124 @16 k（实测） | 48 @48 k = 1 ms【读源码 `resample.rs:7`】 |
| 往返前视（16 kHz 样本） | 124 | **32（2 ms）**【推算：96 @48 k ÷ 3】 |
| 实时调度延迟 | 1393（87 ms，攒大块） | 预计 ≈ 32 + 攒满一帧 160 − 1 ≈ 191【推断，要实测】 |
| 阻带 | soxr HQ | 截止 7.2 kHz，Blackman 窗（骨架版，与 GTCRN 在产品里过的是同一个） |

预计 DFN3 低延迟版声明延迟 = libDF 一帧 160 + 往返 32 = **192 样本（12 ms）**，V-LAT ≈ 45.4 − (284 − 192) ÷ 16 = **39.7 ms，过线**【推算】。

不选的：soxr 低档（LQ 前视 31，但调度延迟 939、阻带差，且仍是第三方）· scipy 自设最小相位滤波器（要另写系数，产生第二处表示）。

## 三、怎么接（不改 `src/`）

`vdn-dsp` 是 Rust，DFN3 走 Python ctypes。分三步、全部按块因果：

1. 新增一个小 Rust 程序 `docs/experiments/exp016-candidates/resample3/`（独立 cargo workspace，按路径依赖 `vdn-dsp`，同 exp014 driver 的做法；`target/` 先确认被 `.gitignore` 覆盖——现有规则只盖 `docs/experiments/*/driver/target/`，要么放 `driver/` 名下，要么请大总管加一行）：`up <in16.wav> <out48.wav>` / `down <in48.wav> <out16.wav>`，每次 768 样本一块（与 `Pipeline` 同块长，3 的倍数），每段新建、结尾补零冲出延迟
2. `run_dfn3ll.py` 加一个 `--resampler vdsp` 选项：16 kHz 输入 → `resample3 up` → 48 kHz 逐帧过 dylib → `resample3 down` → 按 160 + 32 平移对齐；默认仍是 soxr，两种结果 run-id 分开、`fingerprint` 里记重采样器
3. 系统名另起 `dfn3-ll-vdsp`，与现有 `dfn3-ll` 并列进成绩单，不覆盖旧 run

## 四、怎么验（verifier · 阈值 · 是否过线由数据定）

| # | 验什么 | 怎么验 | 过线 |
|---|---|---|---|
| 1 | 重采样器延迟与源码一致 | Rust 测试：脉冲过 `up` / `down`，峰位 = 48 @48 k / 16 @16 k | 逐样本相等 |
| 2 | 往返链路对齐、前视 | `tests/test_ext.py` 加一条：恒等模型过 `vdsp` 链路，`xcorr_lag` = 0，`measure_lookahead`（容差 1e-7）≤ 32 | 是 |
| 3 | 实时调度延迟 | 同 `scheduling_delay()`，逐块记出数时刻 | 只记录；预计 ≈ 191 |
| 4 | 真模型冒烟 | `--probe`：滞后 0 · 前视 ≤ 声明 192 · A→B→A 差 ≤ 1e-6 | 是 |
| 5 | 入口自检 | 480 段跑完，外部入口载入（截半自检 15 段） | 0 段没跑成 |
| 6 | 延迟否决项 | 成绩单 V-LAT（线 45 ms） | 预计 ≈ 39.7 ms，480/480 过 |
| 7 | 换重采样器没改坏效果 | `analyze.py` 配对：`dfn3-ll-vdsp` − `dfn3-ll`，逐项看区间 | 噪声压低量 · 可懂度 · 吞字头三项区间跨 0（分不出）；若某项「差」，在 findings 里如实写 |

## 五、代价

- 开发：Rust 小程序约 80 行 + 测试，Python 改约 40 行 + 测试；中途 codex 审 1–2 轮
- 名额：冒烟 ≤ 1 分钟；480 段约 16 分钟（同现在）；DSP 项打分约 3 分钟；最终版带 `--qa` 时多一个系统约 50 分钟
- 盘：约 750 MB（主目录 `_tmp/exp016/dfn3-ll-vdsp/`，不进仓）

## 六、要人定 / 要 evalfix 定的

1. **做不做**：大总管定**这批不做**，记 backlog
2. **48 kHz 原生模型的平台预算**（大总管定：记账错，evalfix 修；外部系统要能声明原生采样率，字段由 evalfix 与 cands 定，`external.rs` 由 cands 改）：V-LAT 对 48 kHz 模型是否应去掉 `resample-down` / `resample-up` 两项预算（各 2 ms）。去掉后 DFN3 低延迟版即使不换重采样器也约 41.4 ms 过线【推算】。这一条改的是成绩单规则，不在 cands 范围
