# exp021 · 不重训主模型：原声混回比例的离线模拟（调研 030 · #124）

**问的是**：引擎在模型后面把原声混回（说话时 −6 dB、停顿时 −12 dB，正本 `src/types/src/desc/mod.rs` 的 `OPEN_DB` · `FLOOR_DB`）。
改这两个数，降噪量、可懂度能涨多少，字头、背景稳、人声电平会退多少？按场景各挑最好的一档，上限是多少？

**只做离线模拟，不改引擎代码。** 结论写在 `docs/research/030_gtcrn-aux/decision.md`。

## 做法

1. 原始输出：候选模型的外部离线目录本来就是「混回前」的模型输出（主目录 `_tmp/exp016/…`）；GTCRN 用 `driver raw` 重出（引擎三件保命全关）。
2. 重混：`driver remix` 把原始输出交给**真引擎**（`vdn_engine::Engine`），只换 `open_db` / `floor_db`，写成新的外部目录。包络与混回只有引擎这一个正本，本实验不另写。
3. 打分：`vdn-eval scorecard`（不带评分模型），配 `driver suite` 由 `src/eval/suite/scenes.json` 程序派生、全部任务 `safety=false` 的测试集，免得混两次。
4. 读数：`analyze.py check`（逐段核对）· `plot.py`（取舍曲线）。

## 核对：这条路和成绩单原路径等价吗

`runs/20261105-parity-dfn3/check.md`（实测，DFN3 低延迟版，480 段）：默认参数预混 + 关保命，与原始目录 + 默认保命逐段比对，
**21 项值差 0、判定差 0**。只有读引擎门控状态的两项会变（V-SIM-GATE 门控模拟 · V-SIM-FA 门控误开），本实验不用它们。
坏值 / 超幅保护在预混那一遍照开（`driver/src/tests.rs` 的 `faults_handled_in_premix_same_as_engine_path`）。

## 怎么跑（需要重活名额）

```sh
# worktree 根目录
CARGO_TARGET_DIR=$PWD/target cargo test --release --manifest-path docs/experiments/exp021-gtcrn-aux/driver/Cargo.toml
CARGO_TARGET_DIR=$PWD/target cargo build --release --manifest-path docs/experiments/exp021-gtcrn-aux/driver/Cargo.toml
cargo build --release -p vdn-eval
target/release/exp021-remix suite --out _tmp/030/scenes-safety-off.json
docs/experiments/exp021-gtcrn-aux/sweep.sh dfn3 _tmp/exp016/dfn3-ll/20261105-063623-e4eb13d9
python3 -m unittest docs/experiments/exp021-gtcrn-aux/test_analyze.py docs/experiments/exp021-gtcrn-aux/test_plot.py
```

音频只在主目录 `_tmp/030/`，不进仓；仓里留 `runs/<run>/` 的成绩单、`clips.jsonl`、`run.json` 与哈希。

## 文件

| 文件 | 用途 |
|---|---|
| `driver/` | 重混 · 生成 GTCRN 原始输出 · 派生测试集（Rust，独立 crate，测试 7 条） |
| `sweep.sh` | 一个模型扫 8 档 + 不混回，一次打分 |
| `analyze.py` · `test_analyze.py` | 逐段核对 · 按场景平均 |
| `plot.py` · `test_plot.py` | 取舍曲线 SVG |
| `runs/` | 每次运行的读数 |
