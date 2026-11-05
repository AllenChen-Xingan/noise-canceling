# exp020 · GTCRN 强化学习的计算与离线推演（调研 029）

> 2026-11-05 · 手下 rl · issue #123。只读输入，不训练、不放声。音频不进仓。

## 1. 计时：每个检查项在一段 12 秒录音上要多久

- 程序：`timing/`（独立小工程，依赖 `src/eval`；构建产物放仓外）。
- 测法：release 版，插电，`caffeinate -i`，每项重复 5 次取中位，另记第一次（冷启动）。片段取 exp013 正式 build `20261103230756-17f5` 的 4 段：街边 · 共享办公 · 食堂 · 开放办公，均为 mba 设备、c00，输入哈希见 `runs/20261105-timing/inputs.sha256`。
- 复现：
  ```
  CARGO_TARGET_DIR=<仓外目录> cargo build --release --manifest-path docs/experiments/exp020-gtcrn-rl/timing/Cargo.toml
  # 在能找到 GTCRN 权重的目录下运行（见 src/models 的 default_path）
  <target>/release/exp020-timing <片段目录> gtcrn 5
  ```
- 测试：`cargo test --manifest-path docs/experiments/exp020-gtcrn-rl/timing/Cargo.toml`（覆盖登记表里每个 CI 层检查项）。

**读数**（`runs/20261105-timing/`，实测）：

| 项 | 中位耗时 | 说明 |
|---|---|---|
| GTCRN 过引擎一遍（12 s） | 306–311 ms，冷启动 327 ms | 约实时的 39 倍速；v2 规则每段 3 遍（含噪 · 只有噪音 · 只有干净人声），约 0.92 s |
| 原声直通过引擎一遍 | 0.8 ms | 引擎本身开销可忽略 |
| 全部 25 个 CI 层检查项合计 | 190–193 ms | 延迟检查 V-LAT 134 ms · 可懂度 V-ESTOI 36 ms · 咔嗒 V-CLICK 13 ms，其余都不到 3 ms |
| 语音识别（QA 层，未测） | 每次 2.6–4.4 s，每段 2 次 | 引自项目记忆「成绩单估时」的实测 |
| 评分模型（QA 层，未测） | — | 按 D81 不进奖励，不计入训练成本 |

局限：
- 4 段中有几项判了「不知道」，提前返回，所以它们的耗时偏低。例如个性化几项没有重叠段。合计按最坏估也不超过约 0.2 s。
- 用的是单核，8 核并行是推断。

## 2. 离线排序模拟：几种奖励和评分模型是否一致

- 脚本：`sim/reward_rank.py`。测试：`python3 -m unittest discover -s docs/experiments/exp020-gtcrn-rl/sim`。输入清单：`sim/runs.json`。
- 输入：GTCRN 成绩单 run `20261105-005916-4ccf0638`（含原声直通、参考降噪器）、DFN3 低延迟版与 DPDFNet-2 的 run（exp016，cands 分支）。哈希见 `runs/20261105-reward-rank/inputs.sha256`。
- 做法：
  - 480 段，每段 5 个系统。
  - 几种只由 DSP 检查项组成的奖励各给 5 个系统排序，再和 Distill-MOS / SCOREQ 的排序算 Kendall τ。
  - 95% 区间按场景整组重抽，共 15 个场景、2000 次。
  - 另记「奖励最高的系统比 GTCRN 多吞字头 > 1 dB」的段比例。
- 读数：`runs/20261105-reward-rank/result.json`；表格与读法见 `docs/research/029_gtcrn-rl/decision.md` 第二节。

要点（实测）：
1. 越偏向「多压」的奖励，和评分模型越一致，同时越吞字头：一致性 τ 为 0.77 时，54% 的段多吞字头。
2. 守住项改成硬约束后，不再吞字头，但和评分模型的一致性降到 τ ≈ 0.13。
3. 单段吃字否决（D85 口径：多错 ≥ 5 个字）在参考降噪器上也触发 2%，在三个降噪模型上触发 5%–6%。单段识别信号太吵。
4. 人声被压低超过 3 dB 的段：GTCRN 12%，DFN3 3%，DPDFNet-2 5%。

局限：
- 只有 5 个系统，系统级排序不下结论。
- 全是合成音频。
- 评分模型本身未被人耳定标（L0）。
- 守住项的容许量（1 / 2 / 1 dB）是建议值。
