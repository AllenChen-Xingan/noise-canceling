# exp025 · trainer：训练器 · 训练损失 · 模型变体

> 2026-11-16 起 · 手下 trainer · 分支 `exp/025-trainer` · issue #157 · 对应 014 第二节第 3–5 步的代码
> 本目录只有代码、配置、测试与小报告；检查点与日志放仓外（默认 `~/vdn-data/runs/`）。**这里没有任何效果结论**：
> 本机只跑了单元测试和合成小样冒烟，正式冒烟与训练在云上做（D97）。

## 文件

| 文件 | 干什么 |
|---|---|
| `models.py` | 变体 A（官方 GTCRN 原样）· B（A + 时间注意力头回看 32 帧 + 条件输入口子，两者零初始化） |
| `losses.py` | 官方 HybridLoss（原样导入）+ 人声被压低 L_lvl + 字头 L_on；L2-SP · 老师蒸馏 · 电平归一做成开关，默认关 |
| `config.py` · `configs/` | 配置正本（dataclass）与实例：`default.yaml` 正式默认 · `smoke-cpu.yaml` 本机小冒烟 |
| `data.py` | 数据适配：traindata 的 loader（显卡上 GPU 混音）· 仓内合成小样（只给测试与冒烟） |
| `train.py` · `runrec.py` | 训练循环 · 运行记录 · 检查点 · 续训 · 时长与花费上限 · EMA（开关，默认关） |
| `smoke.py` | 开训前冒烟三件事 + 测每步耗时，一个命令 |
| `export.py` | 导出 ONNX（整段版）并与 PyTorch 对拍 |
| `parity_rs/` | 只读调用 `vdn-eval` 的小程序，给损失对拍用（不改 `src/eval`） |
| `official.py` | 找官方源码与起点检查点（核 sha256） |
| `smoke_reports/` | 冒烟报告（JSON） |

## 用法

在仓库根装环境：`uv venv --python 3.13 .venv && uv pip install -p .venv -r docs/experiments/exp025-gtcrn-train/trainer/requirements.txt`。
官方源码默认按主仓 `resources/gtcrn/` 找；云上设 `VDN_GTCRN_DIR=<resources/gtcrn 的路径>`。下面的命令都在本目录跑：

```sh
cargo build --release --manifest-path parity_rs/Cargo.toml   # 对拍测试要它，第一次约 25 秒
../../../../.venv/bin/python -m pytest -q tests/              # 111 条通过，3 条等 traindata 的 loader 合入（和小分片）后自动跑
python smoke.py --config configs/default.yaml --set data.shard_root=… --set data.recipe=… --out smoke.json
python train.py --config configs/default.yaml --set run.name=exp025-B-s0 --set data.shard_root=… --set data.recipe=…
python export.py --ckpt ~/vdn-data/runs/exp025-B-s0/ckpt/step-0005000.pt --out b.onnx
```

用同一个 `run.name` 再跑一次 `train.py`，就从最新检查点续训。续训时只许改步数、时长、花费上限、worker 数、日志间隔和设备；改了别的配置或数据 build，就拒绝续训。

## 核清的事

**参数量 48,245 和论文 23.67 K 的差**【实测，测试 `test_param_counts_explain_48245_vs_23_67k`】：
- 48,245 = 可训练 23,669 + 固定 24,576
- 固定的是两个频带滤波矩阵：`erb.erb_fc` 和 `erb.ierb_fc`，各 64 × 192 = 12,288 个，官方设了 `requires_grad=False`
- 论文写的 23.67 K 就是可训练的那部分
- B 比 A 多 1,344 个可训练参数：注意力 1,088 个（与 028 表一致），条件口子 8 × 32 = 256 个

**起点检查点**：用 `stream/onnx_models/model_trained_on_dns3.tar`，产品的 ONNX 由它导出，sha256 在 `official.py` 里核对。`checkpoints/` 下的同名文件哈希不同，不用它【实测】。

## 校验（全部有测试）

| 项 | 判据 | 读数 | 测试 |
|---|---|---|---|
| B 零初始化 = A | 输出差 ≤ 1e-6；cond 不给、全零、随机都一样 | 通过 | `test_b_zero_init_equals_a` |
| 条件口子全零 = 恒等（训后也成立，D98） | 差 ≤ 1e-6 | 通过 | `test_film_zero_cond_is_identity_after_training` |
| 不看未来帧（A · B） | 改 t0 之后的输入，t0 之前的输出不变；未来帧确实改了（反向对照） | 通过 | `test_no_lookahead` |
| 整段版注意力 = exp019 逐帧版 | 非零权重，K = 4 和 32，差 ≤ 1e-5 | 通过 | `test_offline_matches_exp019_stream` |
| 批归一化冻结：训练模式 = 评估模式 · 训练模式也不看未来帧 | 差 ≤ 1e-6；统计量不更新 | 通过 | `test_frozen_bn_train_mode_equals_eval_and_has_no_lookahead` |
| B 在 MPS（带 exp018 绕法）= CPU | 前向差 ≤ 1e-4 · 梯度相对差 ≤ 1e-3 | 本机通过 | `test_b_on_mps_matches_cpu` |
| L_lvl / L_on 和 Rust 正本对拍 | 有效 / 待测一致；读数差 f64 ≤ 1e-6 dB、f32 ≤ 2e-3 dB；字头数一致；10 ms 掩码和有声区间逐位一致 | 10 段夹具（含段长不整除、说话到段尾、字头窗口越界、全零），31 条通过 | `test_losses_parity.py` |
| 梯度检查 | 6 个随机方向上有限差分 vs 解析梯度，相对差 ≤ 1e-5 | 通过 | `test_*_gradient_finite_difference` |
| 样本不足 | 与 verifier 同条件跳过、不进分母；整批无效时该项 = 0 | 通过 | `test_short_samples_skipped_not_in_denominator` |
| 默认开关 = 官方损失 | 逐值相同 | 通过 | `test_trainloss_defaults_equal_official_hybrid` |
| 续训 = 不中断 | 先训 3 步再续到 6 步，和一口气训 6 步权重逐值相同 | **只在 CPU + 合成数据上**通过；云上 GPU + 真数据待测（现状页「转交」1） | `test_resume_equals_uninterrupted` |
| 第 k 步拿第 k 批 | 错位就报错停下 | 通过 | `test_train_stops_on_batch_misalignment` |
| 时长 / 花费上限 · 训崩 | 到线就存检查点停下；损失不是有限数时不存坏权重，之后拒绝续训 | 通过 | `test_time_and_cost_limits_stop` · `test_nonfinite_…` · `test_resume_refused_after_nonfinite` |
| 被抢占 · 取数卡住 · 检查点写坏 | SIGTERM 停下并存；取数超时 / 用完时干净地停；最新检查点读坏时退回上一份；续训按心跳补记花费 | 通过 | `test_review_fixes.py` |
| ONNX 对拍 | 两种长度；ONNX 离 float64 真值 ≤ max(1e-4, 4 × PyTorch float32 离真值) | A：ONNX 对 PyTorch 差 1.7e-7 / 4.8e-7；训后大权重的 B 也通过 | `test_export_*` |

## 冒烟三件事（014 第 3 步）

判据（下面的读数是**本机 CPU 合成小样读数，云上正式冒烟再核**）：

| 件 | 判据 | 本机读数（2026-11-24，电池，合成小样，批量 4 × 2 s） |
|---|---|---|
| 1 初始损失合理 | B 与 A 相对差 ≤ 1e-5，且起点损失 < 随机初始化 | A 4.388 = B 4.388；随机初始化 37.43 |
| 2 单批过拟合 | 150 步走完「初值到完美输出损失」距离的 ≥ 一半；B 的注意力输出层权重要动过 | 7.64 → −0.08（旧判据下降 101%；新判据自审后改，待下次冒烟重读） |
| 3 原检查点读数复现 | 在官方 `mix.wav` 上和官方 `enh.wav` 的最大差 ≤ 2 个 16 位量化步 | 3.05e-5，即 1 个量化步 |

- 正式数据（`data.kind: loader`）冒烟**必须**用 `--known-dev` 给开发份已知读数，否则记「未完成」、不通过（014 第 3 步）；基准等 evalcmp / cloudrun 出数
- 同名运行同时只能有一个进程（`run.lock`，flock）
- 看门狗：已该停（到上限 / 收到信号）而主线程 `limits.hang_s`（默认 120 s）没有进展时，写 `hung.json` 后强退；正式数据要求 `num_workers ≥ 1`，取数超时才生效
- 标定单独取满 `calib_batches` 批（默认 200 × 32 段），不被开发份大小截短
- 测速只作记录：本机每步中位 1.17 s，取数 0.015 s，主进程 CPU 99%。电池加高负载下测的，**不能和云上比**
- 本机跑任何超过 1 分钟的命令，都要先申请名额，套 `caffeinate -dims`，并记电源状态。2026-11-16 有一次探测因为用电池、空闲睡眠，CPU 只用了 136 秒，墙钟却拖到 56 分钟

报告：`smoke_reports/20261124-cpu-synthetic.json`。

## 默认值与依据

- 超参照 027 第 3.1 节：Adam，学习率 1e-4，前 500 步线性预热，梯度裁剪 3.0，批量 32 × 4 s，5,000 步
- L_lvl / L_on 的权重默认 `auto`：开跑前在开发份上量，定成约占 HybridLoss 的 10%（027 第 3.2 节第 1 步）。如果某项初始为 0，按「超出 1 dB 时等于 10%」标定
- L2-SP、老师蒸馏、EMA 默认关（034 papers 第二节建议 7）
- 电平归一默认关：它的作用是「加了电平随机以后不掉分」，收益 0.01–0.05 PESQ（034 修正 3）。traindata 的配方开了电平随机，所以建议在数据消融时扫一组开 / 关
- 花费上限默认 $20、单价 $0.50/小时（014 第五节 · D96）。这是训练器自己的保险：从开跑第一行起计费，后台线程每 30 s 写一次心跳，被抢占时最多漏记 30 s；美元按段累计，续训改单价不追溯。机器的真实账单归 cloudrun
- 字头项默认与 V-ONSET 正本同口径；`loss.on_skip_leading` 可跳过 a = 0 的区间（随机裁切的假字头），扫参选项，默认关
- 批归一化默认冻结统计量（`model.bn: frozen`，AI 定）：训练模式的批归一化用整段（含未来帧）与整批的统计量，和推理不一致，还会冲掉 DNS3 的统计量。`train` 作扫参选项
- 条件口子本次喂全零，W·0 对 W 的梯度恒为 0，所以训练中它不会动——这是 D98「这次喂全零」的直接后果，不是 bug；以后喂向量时它才开始学
- 条件输入本次一律喂全零（D98），`train.cond_for` 是以后改喂场景向量或声纹的唯一入口。以后要加宽维度，可以给权重补零列，不改结构（推断，未测）

## 不在本目录

- 流式导出（每帧一调、带缓存）：A 可以走官方 `convert_to_stream`；B 的注意力要在 Rust 引擎里另写（014 第六节，backlog）
- 效果验收（配对比较、区间）：evalcmp 的 `vdn-eval` 子命令
