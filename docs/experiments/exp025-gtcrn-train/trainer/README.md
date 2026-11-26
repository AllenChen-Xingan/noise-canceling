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
| `watchdog.py` | 信号 · 心跳 · 看门狗（单写者心跳；先判断后写盘；不碰磁盘的本段时限兜底） |
| `smoke.py` | 开训前冒烟三件事 + 测每步耗时，一个命令 |
| `smoke.py --write-known-dev` | 只算起点模型的开发份参照读数（带数据版本 meta），给 `--known-dev` 用 |
| `cmp_ckpt.py` | 逐键比两个检查点的 `model`（续训核对） |
| `guards.py` | 训练时的自动保护：阈值正本（损失猛涨 · 最好的检查点 best.pt · 前 500 步太慢 · 卡住 900 s）；见 `plan-h100-smoke.md` 第十二节 |
| `gate_v4.py` | 单批过拟合判据第 4 版的正本（查「流程能学」）；也能按第 4 版重判旧报告 |
| `dev_gate.py` | 训练有没有用：300 步后开发份读数要低于第 0 步（CPU 参照） |
| `overfit_probe.py` · `data_probe.py` · `smoke.py --overfit-probe` | 过拟合慢的诊断：学习曲线 · 梯度 · 学习率 · 对齐 · 目标 · 关配方旋钮；只记录 |
| `oracle.py` · `smoke.py --overfit-diag` | 过拟合排查：掩码族能达到的损失（oracle）作对照，判 40 dB 下限是否可达；只记录，不判过不过 |
| `export.py` | 导出 ONNX（整段版）并与 PyTorch 对拍 |
| `parity_rs/` | 只读调用 `vdn-eval` 的小程序，给损失对拍用（不改 `src/eval`） |
| `official.py` | 找官方源码与起点检查点（核 sha256） |
| `smoke_reports/` | 冒烟报告（JSON） |
| `plan-h100-smoke.md` | 云上 H100 冒烟方案 |

## 用法

在仓库根装环境：`uv venv --python 3.13 .venv && uv pip install -p .venv -r docs/experiments/exp025-gtcrn-train/trainer/requirements.txt`。
官方源码默认按主仓 `resources/gtcrn/` 找；云上设 `VDN_GTCRN_DIR=<resources/gtcrn 的路径>`。下面的命令都在本目录跑：

```sh
cargo build --release --manifest-path parity_rs/Cargo.toml   # 对拍测试要它，第一次约 25 秒
../../../../.venv/bin/python -m pytest -q tests/              # 119 条通过，4 条等 traindata 的 loader / cloud 合入（和小分片）后自动跑
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
| B 在 MPS（带 exp018 绕法）= CPU | 前向最大差 ≤ 1e-4 · 梯度范数相对差 ≤ 1e-3；差 1% 必红 | 本机通过 | `test_b_on_mps_matches_cpu` |
| L_lvl / L_on 和 Rust 正本对拍 | 有效 / 待测一致；读数差 f64 ≤ 1e-6 dB、f32 ≤ 2e-3 dB；字头数一致；10 ms 掩码和有声区间逐位一致 | 10 段夹具（含段长不整除、说话到段尾、字头窗口越界、全零），31 条通过 | `test_losses_parity.py` |
| 梯度检查 | 6 个随机方向上有限差分 vs 解析梯度：\|差\| ≤ 1e-5·\|方向导数\| + 1e-12（方向导数非近零）；梯度放大 1% 必红（反向对照） | 通过 | `test_*_gradient_finite_difference` |
| 样本不足 | 与 verifier 同条件跳过、不进分母；整批无效时该项 = 0 | 通过 | `test_short_samples_skipped_not_in_denominator` |
| 默认开关 = 官方损失 | 逐值相同 | 通过 | `test_trainloss_defaults_equal_official_hybrid` |
| 续训 = 不中断 | 先训 3 步再续到 6 步，和一口气训 6 步权重逐值相同 | **只在 CPU + 合成数据上**通过；云上 GPU + 真数据待测（现状页「转交」1） | `test_resume_equals_uninterrupted` |
| 第 k 步拿第 k 批 | 错位就报错停下 | 通过 | `test_train_stops_on_batch_misalignment` |
| 时长 / 花费上限 · 训崩 | 到线就存检查点停下；损失不是有限数时不存坏权重，之后拒绝续训 | 通过 | `test_time_and_cost_limits_stop` · `test_nonfinite_…` · `test_resume_refused_after_nonfinite` |
| 被抢占 · 取数卡住 · 检查点写坏 | SIGTERM 停下并存；取数超时 / 用完时干净地停；最新检查点读坏时退回上一份；续训按心跳补记花费 | 通过 | `test_review_fixes.py` |
| ONNX 对拍 | 两种长度；ONNX 离 float64 真值 ≤ max(1e-4, 4 × PyTorch float32 离真值) | A：ONNX 对 PyTorch 差 1.7e-7 / 4.8e-7；训后大权重的 B 也通过 | `test_export_*` |

## 冒烟三件事（014 第 3 步）

判据（下面的读数是**本机 CPU 合成小样读数，云上正式冒烟再核**；2026-11-24 第二次跑，墙钟 52 s，电池）：

| 件 | 判据 | 本机读数（2026-11-24，电池，合成小样，批量 4 × 2 s） |
|---|---|---|
| 1 初始损失合理 | B 与 A 相对差 ≤ 1e-5，且起点损失 < 随机初始化 | A 4.388 = B 4.388；随机初始化 37.43（批归一化冻结后，训练模式与评估模式同值） |
| 2 单批过拟合（第 4 版，事后改判据，2026-11-25，大总管定，人未参与；正本 `gate_v4.py`） | 开发份第 0 批前 4 段，150 步：训练损失相对下降 ≥ 15%；SI-SNR 提高 > 0；主干各层第 1 步梯度非零有限；B 的注意力输出层动过；初值不是正数 → 判据不可用（退出码 3）。第 3 版（oracle 距离 ≥ 50%、+6 dB）降为只记录；另由 `dev_gate.py` 判 300 步开发份读数低于第 0 步。见 `plan-h100-smoke.md` 第八至十一节 | 云上 CPU 真数据（2026-11-25）：下降 24.7%，SI-SNR +2.52 dB，按第 4 版重判为过（`smoke_reports/20261125-rejudge-v4.json`）；第 3 版判不过（`20261125-cpu-gate3-FAILED.json`） |
| 3 原检查点读数复现 | 在官方 `mix.wav` 上和官方 `enh.wav` 的最大差 ≤ 2 个 16 位量化步 | 3.05e-5，即 1 个量化步 |

- 正式数据（`data.kind: loader`）冒烟**必须**用 `--known-dev` 给开发份已知读数，否则记「未完成」、不通过（014 第 3 步）；基准等 evalcmp / cloudrun 出数
- 同名运行同时只能有一个进程（`run.lock`，flock）
- 看门狗：已该停（到上限 / 收到信号）而主线程 `limits.hang_s`（默认 120 s）没有进展时，写 `hung.json` 后强退；正式数据要求 `num_workers ≥ 1`，取数超时才生效
- 标定单独取满 `calib_batches` 批（默认 200 × 32 段），不被开发份大小截短
- **第 2 件的判据改过几次，现行是第 4 版（见上表与 `plan-h100-smoke.md` 第十一节）；下面是第 2 版那次的历史记录（事后改判据，AI 定，大总管认可）**：
  - 原判据：(初值 − 终值) / (初值 − 「输出恰好等于目标」的损失) ≥ 0.5
  - 为什么定错：输出与目标完全相同时，官方 HybridLoss 的 SI-SNR 项被分母里的 1e-8 下限顶成约 −8，下限因此是 −10.04，任何训练都到不了。本机 150 步走到 −0.92，只占 37%，判「不过」。报告是 `smoke_reports/20261124-cpu-synthetic-v2-floor-exact-FAILED.json`
  - 新判据：下限改成「目标 + 40 dB 白噪声」这个近乎完美输出的损失（本机 −0.41），走完 111%，判过。另加一个不依赖下限的读数：这一批的 SI-SNR 过拟合后要提高 ≥ 6 dB（阈值 AI 定）。**本机还没用这条测过**，名额已收回；云上 GPU 冒烟时两个读数都要过
- 测速只作记录：本机每步中位 0.25 s（批量 4 × 2 s），取数 0.007 s。电池下测的，**不能和云上比**
- 本机跑任何超过 1 分钟的命令，都要先申请名额，套 `caffeinate -dims`，并记电源状态。2026-11-16 有一次探测因为用电池、空闲睡眠，CPU 只用了 136 秒，墙钟却拖到 56 分钟

云上 H100 冒烟的方案（大总管已批，开机前先过 codex）：`plan-h100-smoke.md`。开发份参照用 `smoke.py --write-known-dev` 在 CPU 机上算；续训核对用 `cmp_ckpt.py`。

**云上 H100 冒烟结果（2026-11-25，代码 7eecbbe8，实测，全过）**：
- 冒烟三件事全过：初始损失 B = A（相对差 0）；单批过拟合按第 4 版过（下降 24.8%，SI-SNR +2.53 dB）；复现差 0；开发份读数 3.97339，CPU 参照 3.97330，差 8e-5。报告 `smoke_reports/20261125-h100-smoke-v4.json`
- 训练有没有用：A、B 各训 300 步，开发份 hybrid 从 3.97330 降到 3.90371（A）和 3.90497（B），都过（`dev_gate.py`）
- 续训 = 不中断：分两次训 6 步与一次训满，278 个键最大差 0
- 每步：单跑 B 0.1665 s、A 0.148 s；3 个 B 同时跑每个 0.39 s。正式训练 6 个运行：串着跑约 1.8–1.9 小时、7.8–8.3 美元；3 个同时跑约 1.24 小时、5.4 美元（按 4.39 美元 / 小时，由实测推算，见方案第五节）

**新基线 build 上的冒烟与数据消融（2026-11-26，H100，代码 98f1cf32，build `bc0e9e7b566421f0`，实测）**：执行单 `plan-h100-b4.md`，cloudrun 跑，trainer 在本机读取回的文件判。
- 闸门全过：
  - 冒烟 `pass` 为 true（`smoke_reports/20261126-h100-b4-smoke.json`）
  - 训练有没有用：开发份 hybrid 从第 0 步 3.94881（CPU 参照 `20261126-b4-known-dev-cpu.json`）降到 3.91744（A）、3.91445（B），判线 0.001，都过（`20261126-h100-b4-dev-gate-{A,B}.json`）
  - 续训核对：278 个键最大差 0（`20261126-h100-b4-cmp-resume.json`）
- 数据消融 7 组：都正常结束，`ablation.py judge` 退出 0，7 个运行都完整（`smoke_reports/20261126-ablation-judge.json`）。判法是 `ablation.py` 的 `RULE`（大总管定，跑之前写定）
  - 两个基线（种子 0、1）：3.83752、3.84532。均值 3.84142，噪声底（两者之差）0.00779
  - 差值 d = 旋钮运行的开发份 hybrid − 基线均值，越大说明关掉以后越差

| 旋钮 | 这次怎么动 | d | 约为噪声底的几倍 | 判定 |
|---|---|---|---|---|
| 混响 | 关掉 | +0.0910 | 11.7 | 有用，保持开 |
| 均衡 | 关掉 | +0.0216 | 2.8 | 有用，保持开（证据不强，见下） |
| 多噪声叠加 | 关掉 | +0.0051 | 0.7 | 分不出，保持开 |
| 削波 | 关掉 | −0.0049 | 0.6 | 分不出，保持开 |
| 高信噪比 | 打开 | +0.0010 | 0.1 | 分不出，保持关 |

- 两点限制（大总管要求写明）：
  - 噪声底只来自 2 个种子，每个旋钮也只跑了 1 次（1000 步，模型 A）。噪声底本身可能偏小或偏大，未量化
  - 均衡按规则判为「有用」，但只有 2.8 倍噪声底，证据不强。混响约 12 倍，比较稳
- 结论：配方不用改。改配方归 traindata
- 耗时（`20261126-h100-b4-timing.json`）：3 个 A 同时跑每步 0.342–0.349 s（之前推断 0.347 s），准备期 152–159 s；单跑 A 每步 0.1415 s。冒烟 197 s，A、B 各 300 步 216 s、299 s。H100 花 4.32 美元
- 和执行单不同的一处：每批 3 个运行放在同一个 job 里用后台进程同时跑，没有分 3 个 job-start（cloudrun 报）。不影响读数

报告：`smoke_reports/20261124-cpu-synthetic-v2.json`（第 2 版判据，历史）；`20261124-cpu-synthetic.json` 是自审前旧判据那次。

## 默认值与依据

- 超参照 027 第 3.1 节：Adam，学习率 1e-4，前 500 步线性预热，梯度裁剪 3.0，批量 32 × 4 s，5,000 步
- L_lvl / L_on 的权重默认 `auto`：开跑前在开发份上量，定成约占 HybridLoss 的 10%（027 第 3.2 节第 1 步）。如果某项初始为 0，按「超出 1 dB 时等于 10%」标定
- L2-SP、老师蒸馏、EMA 默认关（034 papers 第二节建议 7）
- 电平归一默认关：它的作用是「加了电平随机以后不掉分」，收益 0.01–0.05 PESQ（034 修正 3）。traindata 的配方开了电平随机，所以建议在数据消融时扫一组开 / 关
- 每次运行的花费保险默认 $20（014 第五节），单价默认 $4.39/小时（H100，D102）。单价的正本是 `../cloud/config.py` 的 `USD_PER_HR_EXPECTED`，`test_usd_per_hour_matches_cloud_config` 比对，cloud 合入后自动跑。$20 是每次运行的保险，和 cloud 的总上限 $60 不是同一个量
- 字头项默认与 V-ONSET 正本同口径；`loss.on_skip_leading` 可跳过 a = 0 的区间（随机裁切的假字头），扫参选项，默认关
- 批归一化默认冻结统计量（`model.bn: frozen`，AI 定）：训练模式的批归一化用整段（含未来帧）与整批的统计量，和推理不一致，还会冲掉 DNS3 的统计量。`train` 作扫参选项
- 条件口子本次喂全零，W·0 对 W 的梯度恒为 0，所以训练中它不会动——这是 D98「这次喂全零」的直接后果，不是 bug；以后喂向量时它才开始学
- 条件输入本次一律喂全零（D98），`train.cond_for` 是以后改喂场景向量或声纹的唯一入口。以后要加宽维度，可以给权重补零列，不改结构（推断，未测）

## 止损分工：训练器尽力而为，硬止损在 cloud

（大总管 2026-11-24 定，依 CLAUDE.md 二节「同一类问题第二次出现就改结构」）

- **训练器内（本目录，尽力而为）**：
  - 到步数、时长上限或花费上限时，存检查点停下
  - 收到 SIGTERM / SIGINT 时存盘停下；取数超时时停下
  - 后台心跳线程每 30 s 记一次账，兼作看门狗：已该停、而主线程 `hang_s` 内没有进展，就强退
  - 一道不碰磁盘的本段时限兜底
  - 网络盘卡住、写盘失败这类故障下，这些保险不保证生效
- **硬止损（cloudrun）**：机器到时限自删（实测有效），加上本机看门狗按登记的删机时刻删机。说明见 `../cloud/README.md`「防忘删机」一节。**只有删机才停止计费**
- 待办 issue：#159（启动期网络盘阻塞，T-1）· #160（心跳改名阻塞的跨运行覆盖，T-2）
- 评审：codex 第 9 轮起，「故障注入下训练器的看门狗能不能强退」这一类条目记 P2 待办，不进停止线；其他 P1 照修

## 不在本目录

- 流式导出（每帧一调、带缓存）：A 可以走官方 `convert_to_stream`；B 的注意力要在 Rust 引擎里另写（014 第六节，backlog）
- 效果验收（配对比较、区间）：evalcmp 的 `vdn-eval` 子命令
