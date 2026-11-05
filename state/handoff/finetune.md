# 现状页 · 手下 finetune（调研 027 · #121 · exp018）

- **大事**：GTCRN 加训（D86）· 本线只看「在现有权重上微调 + 训练数据怎么配」（2026-11-05 范围调整；注意力头归 arch 028，强化学习归 rl 029，辅助小模型 / 混回比例归 aux 030）
- **管的实体**：GTCRN 权重 · 训练数据集 · 微调方法
- **分支**：`research/027-gtcrn-finetune` · worktree `../noise_canceling-finetune`

## 已做

1. `.gitignore` 加训练产物规则（`*.pt` `*.pth` `*.ckpt` `*.safetensors` `*.npz` `/runs/` `wandb/` `lightning_logs/` `tb_logs/`）；不写裸 `runs/`，以免忽略已入库的 `docs/experiments/*/runs/`
2. 单步耗时冒烟（实测，已发大总管）：脚本 `docs/experiments/exp018-gtcrn-finetune/smoke/step_timing.py`（unittest 3 条全过）；读数 `docs/experiments/exp018-gtcrn-finetune/runs/20261105-145149-step/`
   - 批量 8：CPU 0.368 s / 步，MPS 0.593 s / 步；批量 32：CPU 1.346 s，MPS 0.821 s（每段 4 s 音频）
   - MPS 回落算子 0 个（回落截获逻辑只用合成警告测过）

## 下一步

1. 读 013 · 014 · 018 · 019 · 023 · exp016 `runs/*/analysis.md` · 相关决定
2. 写 `docs/research/027_gtcrn-finetune/`（README · sources · decision，第一节「比较卡」）；验收节写清引擎原声混回的影响与 `safety: false` 口径
3. codex-review 1–2 轮 → 报「finetune 可以收尾」

## 待人拍板

（暂无）
