# exp019 · GTCRN 结构候选：数乘加、测每帧耗时、验热启动

> 2026-11-05 · 手下 arch（大总管 noise-canceling-74）· 调研 028 · issue #122
> 只数和测，不训练。结论写在 `docs/research/028_gtcrn-arch/decision.md`。

## 做了什么

把 7 个结构候选挂到 GTCRN 的中间瓶颈处（第二个双路 RNN 之后），每个都满足：
- 残差 + 输出零初始化：刚接上时输出与原模型相同（误差 ≤ 1e-6），能从现有权重热启动（测试 `test_zero_init_equals_base`）
- 只看当前帧和过去帧，不加算法延迟（测试 `test_causal_no_lookahead`）

| 候选键 | 是什么 |
|---|---|
| `base` | 原模型（对照） |
| `tattn-k16/32/64` | 时间方向注意力：每个子带看过去 K 帧（K=32 约 0.5 s），2 头 |
| `fattn` | 频率方向注意力：同一帧 33 个子带互相看，2 头 |
| `film-d8` | 条件输入：外部给 8 维场景向量，算每通道缩放和偏移 |
| `dpgrnn3` | 多加一层同形的双路 RNN |
| `dpgrnn3-w32` | 多加一层宽 32 的双路 RNN（1×1 投影进出） |

## 文件

| 文件 | 干什么 |
|---|---|
| `gtcrn_src.py` | 找到主目录 `resources/gtcrn/`（官方代码，只读导入，不抄），加载产品用的 DNS3 权重 |
| `variants.py` | 7 个候选模块 |
| `model.py` | 把候选挂进官方流式模型，一次推一帧 |
| `macs.py` | 在真模型上挂钩子数每帧乘加、参数、缓存字节 |
| `bench.py` | 单线程测每帧耗时：连续推帧（burst）/ 按 16 ms 节奏（paced）；各候选轮流跑；写 `runs/<run-id>/` |
| `report.py` | 合成汇总表 `runs/<run-id>/summary.md`，并推算手写 Rust 版（020 的 v4i）的每帧耗时 |
| `tests/test_exp019.py` | 12 条测试（见下） |

## 怎么跑

用主目录 `_tmp/exp008-torch-venv`（torch 2.15.0，已有 einops，不装新包）。在本目录下：

```
python tests/test_exp019.py -v                         # 约 9 s
python bench.py --mode burst --frames 500 --rounds 3   # 约 15 s
python bench.py --mode paced --frames 300 --rounds 1   # 约 45 s
python report.py runs/<burst-run> runs/<paced-run>
```

worktree 里没有 `resources/`，脚本按 `git rev-parse --git-common-dir` 找主仓；也可设 `VDN_MAIN_ROOT`。

## 测试（加测试是默认行为）

| 测试 | 证明什么 |
|---|---|
| `test_base_equals_official_stream` | 包装后的 base 与官方 `StreamGTCRN` 逐值相同 |
| `test_zero_init_equals_base` | 每个候选零初始化后输出与原模型相同（误差 ≤ 1e-6） |
| `test_perturbed_changes_output` | 反向对照：候选权重不为零时输出确实变了 |
| `test_causal_no_lookahead` | 改未来帧不影响已出的帧 |
| `test_output_aligned_with_input` | 候选模块输出与输入同帧对齐：只改第 t 帧输入，第 t 帧输出就变 |
| `test_alignment_check_catches_one_frame_delay` | 反向对照：纯延后一帧的模块被判为不对齐 |
| `test_tattn_window_is_k_frames` | K = 16 / 32 / 64：最老窗内帧起作用，紧邻窗外帧不起作用 |
| `test_film_zero_cond_is_bypass_after_training` | 场景条件权重非零时，全零向量仍是恒等旁路（无偏置） |
| `test_params_match_official` | 可训练参数总数 23,669，与官方 23.67 K 一致（只比总数） |
| `test_macs_near_official` | 每帧乘加落在官方 33.0 MMACs/s ÷ 63 帧的 75–100%（我们不计归一化、激活） |
| `test_extra_macs_formulas` | 候选乘加 = 比较卡里的手算公式 |
| `test_dpgrnn3_costs_one_block` | 多一层双路 RNN 的乘加 = 原第一层的乘加 |

## 口径与限制

- 一帧 = 16 ms（512 点窗、256 点移），不是 10 ms
- torch 单帧耗时含大量 Python 调度开销，只比相对快慢、不进结论（019 标准第 9 节）；用户端耗时按 020 的 v4i 实测 × 乘加比例做情景推算，标「推断」，不是上下界
- 乘加口径与 020 的 `cost.rs`（366,460/帧）不同：我们数 420,944/帧，主要差在 ERB 两个固定矩阵：我们按稠密矩阵数（192×64×3 + 64×192×2 = 61,440），020 按稀疏滤波数（约 2,900）；扣掉后 359,504 对 366,460，差 1.9%。推算用户端耗时时，新增乘加一律除以 020 的 366,460（`report.py` 的 `V4I_BASE_MACS`）
- 动态卷积、仿射 PReLU 两个候选没在这里实现，代价引用论文（见 028 sources S2、S4）

## 读数（2026-11-05，Apple M5 · macOS 27.0.1 · 插电 · torch 2.15.0 · 单线程）

| run | 跑法 | 用途 |
|---|---|---|
| `runs/20261105-153711-burst-41c362/` | 连续推帧，8 候选 × 3 轮 × 500 帧，轮流跑 | 相对快慢；`summary.md` 是汇总表 |
| `runs/20261105-153736-paced-c3a867/` | 16 ms 节奏，8 候选 × 1 轮 × 300 帧，依次跑 | **只记录**：1 轮、读数随先后漂移，不用于比较 |

要点（连续推帧中位）：原模型 1.540 ms；时间注意力 +2–4%；多一层双路 RNN +31%（乘加只 +16.7%）。用户端（Rust v4i）是「与乘加成正比」的情景推算，见 `summary.md` 与 028 decision；torch 数按 019 标准第 9 节不进结论。
