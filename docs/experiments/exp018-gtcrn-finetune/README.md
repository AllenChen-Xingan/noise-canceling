# exp018 · GTCRN 微调前的冒烟：本机能不能训、多快、算得对不对

> issue #121 · 分支 `research/027-gtcrn-finetune` · 手下 finetune · 2026-11-05 · 服务调研 027
> **没有训练**：输入是随机噪声或本机已有的音频，权重是官方 DNS3 检查点，只量速度和正确性。正式训练实验另开编号。

## 假设

1. GTCRN 能在本机 MPS 上训练，并且比 CPU 快
2. 训练卡在算力，不在数据读入与混音
3. MPS 上算出来的前向、梯度与 CPU 一致（调研 013 担心 GRU 有静默错误，PyTorch #94691）

## 结论（实测；环境 torch 2.15.0 · macOS 27.0.1 · Apple M5 · 插电）

| 假设 | 结论 | 读数 |
|---|---|---|
| 1 速度 | 批量 32 × 4 s：MPS 0.821 s / 步，CPU 1.346 s / 步。批量 8 时反而 CPU 快（0.368 vs 0.593 s）。**这是不带绕法的 MPS，算错了**。带绕法重测：批量 32 时 0.856 s（不带绕法同场 0.827 s，慢约 3.5%），CPU 1.372 s；批量 8 时 0.600 s | `runs/20261105-145149-step/` · `runs/20261105-153553-step-fix/` |
| 2 瓶颈 | 成立：单进程 476 段 / 秒，4 进程 775 段 / 秒（每墙钟小时约 3,100 小时音频），是训练（CPU 约 95 小时 / 小时）的 30 倍以上 | `runs/20261105-150423-mix-clean/`（`20261105-145445-mix/` 与别的满载任务重叠，留作对照） |
| 3 正确性 | **不成立**：MPS 前向差 61%、梯度差约 84%。根因不是 GRU（GTCRN 三种尺寸的 GRU 单独比，前向与梯度差 < 1e-5，`test_mps_fix.py` 的 `GruTest`），是逐通道、沿时间轴带空洞的 `Conv2d`；转置卷积对输入的梯度用的是同一个算子，也错。GTCRN 中招 4 处 | `runs/20261105-150438-parity/` · `runs/20261105-150606-layers/` |
| 3′ 绕法 | `smoke/mps_fix.py` 把这 4 个卷积换成数学等价写法后，正式尺寸（8 × 4 s，20 步）前向差 1.2e-6、整模型梯度差 6.1e-5、损失差 4.4e-6、训后完整状态（含 BatchNorm 统计）差 1.0e-4；MPS 训出的状态装进没打绕法的原模块，eval 前向与 CPU 训出的差 1.0e-3，全过 | `runs/20261105-152353-parity-fix2/`（`20261105-151236-parity-fix/` 是补这两项之前的一次） |

## 文件

| 文件 | 干什么 | 测试 |
|---|---|---|
| `smoke/step_timing.py` | 单步训练耗时（前向 + 反向 + Adam），可选 `--mps-fix` | `test_step_timing.py` |
| `smoke/mix_timing.py` | 读入 + 在线混音耗时 | `test_mix_timing.py` |
| `smoke/mps_parity.py` | CPU 与 MPS 比前向 · 整模型梯度 · N 步训练 · 训后完整状态 · 导出口径（原模块 eval） | `test_mps_parity.py` |
| `smoke/layer_diff.py` | 逐层比 CPU 与 MPS，找第一个差的层 | `test_layer_diff.py` |
| `smoke/mps_fix.py` | MPS 绕法（只训练时用，导出用原模块） | `test_mps_fix.py`（含「原写法在 MPS 上仍错」两条：上游修好时会红） |

跑全部测试（在 `smoke/` 下）：`GTCRN_DIR=<主目录>/resources/gtcrn python -m unittest test_step_timing test_mps_parity test_mps_fix test_layer_diff test_mix_timing`——26 条全过。用的 venv 是主目录 `_tmp/exp008-torch-venv`（torch 2.15.0，有 einops，无 pytest）。

## 边界

- 时间边界：只对 torch 2.15.0 成立；换版本先跑 `test_mps_fix.py`
- 规模边界：只量了批量 8 / 32、4 s 片段、20 步；长时间训练的数值稳定性未测
- 混音测速用的是本机 LibriSpeech test-clean 与 DEMAND，只为贴近真实读盘，不代表训练集选型（选型见调研 027，评测集素材不进训练）
- 系统缓存大概率已热，冷盘第一遍没单独测；训练与读数据同时跑时抢 CPU，没测
- MPS 回落开关的截获逻辑只用合成警告文本测过
