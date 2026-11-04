# exp014 · GTCRN 第一张场景成绩单

> 2026-11-03 起 · 手下 card1 · issue #92 · 分支 `exp/014-scorecard-gtcrn`
> 用途：把降噪模型 GTCRN 接进场景测试集，测每个开会地点的成绩，找出哪些场景、哪种声音要优化（人原话见 issue #92）。
> 本线只测、只分析，不改模型、不改评测器。结论在 `findings.md`；成绩单规则见 `specs/rubric/scorecard.md`。

## 一、用了什么

- **测试集**：exp013 场景测试集正式 build `20261103230756-17f5`（`docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/`），任务文件 `src/eval/suite/scenes.json`（60 条 = 15 个地点 × 常见 / 压力两档 × Air 内置麦 / 外接麦两种设备；共 480 段，每段 12 秒）
- **被测系统**（3 个）：
  - `passthrough` 原声直通：输出 = 输入，作底线
  - `ref` 参考降噪器：评测器自带的维纳滤波替身，不是产品模型
  - `gtcrn`：权重 `resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`（主目录，sha256 前 16 位 `b4718df6228e7bdf`），用环境变量 `VDN_GTCRN_ONNX` 指过去
- **评分**：带 `--qa`，即另跑吃字（本机 Qwen3-ASR 识别服务，每个文件转写 2 次）和两个听感评分模型（SCOREQ 主、Distill-MOS 交叉）

## 二、怎么跑

成绩单命令 `vdn-eval scorecard` 只输出按场景汇总的 `scorecard.json`，没有逐段分数；分设备、分「人声与底噪同房间」要逐段数。所以本线写了一个小驱动 `driver/`：

- 调用评测器同一组库函数（`scorecard::run` → `Card::build` → `write_new`），另把每段每项的分数写成 `clips.jsonl`
- 不改 `src/`；独立 cargo workspace，`Cargo.lock` 从仓库根复制，依赖版本相同
- 等价核对【实测】：cafe 常见档 Air 10 段 × 3 个系统（不带 `--qa`），驱动与 `vdn-eval scorecard` 的 `scorecard.md` 逐字相同，`scorecard.json` 除 `meta` 外相同
- 驱动测试：`cargo test --release`（在 `driver/` 下）

```sh
# 在 worktree 根目录
export VDN_GTCRN_ONNX=<主目录>/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx
(cd docs/experiments/exp014-scorecard-gtcrn/driver && cargo build --release)
caffeinate -i docs/experiments/exp014-scorecard-gtcrn/driver/target/release/exp014-driver \
  scorecard --suite src/eval/suite/scenes.json --system passthrough,ref,gtcrn --qa
# 输出：<主目录>/_tmp/scorecard/<run-id>/（scorecard.md · scorecard.json · clips.jsonl · qa/）
python3 docs/experiments/exp014-scorecard-gtcrn/analyze.py <run 目录> \
  docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/clips.jsonl \
  --out docs/experiments/exp014-scorecard-gtcrn/runs/<run-id>/analysis.md
python3 -m unittest discover -s docs/experiments/exp014-scorecard-gtcrn/tests
```

## 三、机器状态

- Apple M5 · 10 核 · 32 GB 内存 · macOS 27.0.1 · 插电、电量 100%
- 评测器提交：见各 run 的 `scorecard.json` → `meta.commit`
- 耗时【实测】：10 段 × 3 个系统带 `--qa` 3 分 13 秒；不带 `--qa` 时 GTCRN 10 段 7.5 秒

## 四、runs

| run id | 内容 | 备注 |
|---|---|---|
| `20261104-000444-e4511abc` | 正式跑：60 条任务 × 3 个系统，带 `--qa` | 见 `runs/20261104-000444-e4511abc/` |

作废：`20261104-000157-bafd17cf`（用命令行版起的正式跑，约 3 分钟后停掉，改用驱动重跑；目录留在主目录 `_tmp/scorecard/`，不进仓）。

## 五、文件

| 文件 | 管什么 |
|---|---|
| `driver/` | 跑成绩单 + 导出逐段分数 |
| `analyze.py` · `tests/` | 从逐段分数出对照表（分设备、分同房间、与原声直通配对比较、人耳候选） |
| `runs/<run-id>/` | `scorecard.md` · `scorecard.json` · `clips.jsonl`（逐段分数，不含音频）· `analysis.md` |
| `findings.md` | 结论与建议的优化方向 |
| `listen.md` | 人耳片段清单（音频在主目录 `_tmp/exp014/listen/<run-id>/`，不进仓） |
