# 现状页 · trainer（批次 20261116-b19）

- **状态**：进行中
- **手上**：训练器 · 训练损失 · 模型变体（issue #157，分支 `exp/025-trainer`）
- **交代**：`_tmp/20261116-b19/trainer.md`
- **做到哪**（2026-11-16）：读完 027 / 028 / 034 / 014 与官方源码；建好 `.venv`（torch 2.15.0）。参数量差已算清（见「发现」），待测试核实
- **下一步**：写模型变体 A / B 与测试，再写损失与 Rust 对拍

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `docs/experiments/exp025-gtcrn-train/trainer/requirements.txt` | 依赖钉版本 |
| `docs/experiments/exp025-gtcrn-train/trainer/.gitignore` | 忽略本目录的构建产物与 ONNX |

## 发现
- 原 torch 环境（主目录 `_tmp/exp008-torch-venv`）已不存在，改在 worktree 根建 `.venv/`
- 参数量（读源码算，待测试核）：ERB 两个固定矩阵 `erb_fc` / `ierb_fc` 各 64×192 = 12,288，`requires_grad=False`；48,245 − 24,576 = 23,669 ≈ 论文 23.67 K
- 早期混响 100 ms 的正本：evalcmp 提议放 exp013 `rooms.py` 的 `EARLY_REVERB_MS`；trainer 不切混响，只读数据 build 里的目标，已回复 evalcmp 去找 traindata 对齐

## 待人拍板
（暂无）

## 流水
- 16:05 报到；大总管回「照交代做」
- 16:10 回 evalcmp：早期混响正本归 rooms.py，trainer 不留第二处
- 16:15 读完材料；建 `.venv`，装 requirements.txt（torch 2.15.0 · onnxruntime 1.32.0 · onnx 1.23.2）
