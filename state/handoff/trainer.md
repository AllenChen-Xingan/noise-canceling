# 现状页 · trainer（批次 20261116-b19）

- **状态**：进行中（自审 + codex 中）
- **手上**：训练器 · 训练损失 · 模型变体（issue #157，分支 `exp/025-trainer`）
- **交代**：`_tmp/20261116-b19/trainer.md`；大总管 2026-11-24 起是 `noise-canceling-33`（接替 e1）
- **做到哪**（2026-11-24）：交代 1–6 条代码与测试都写完。68 条测试通过，1 条等 traindata 的 loader 合入后自动跑。本机冒烟三件事全过（合成小样）
- **下一步**：多视角自审 → codex 评审（连续两轮无 P0/P1）→ 报「可以收尾」

## 改了哪些文件
全部在 `docs/experiments/exp025-gtcrn-train/trainer/`，说明见该目录 `README.md`：
`models.py` · `losses.py` · `config.py` · `configs/{default,smoke-cpu}.yaml` · `data.py` · `train.py` · `runrec.py` · `smoke.py` · `export.py` · `official.py` · `parity_rs/` · `tests/` · `smoke_reports/` · `requirements.txt` · `.gitignore` · `README.md`

## 发现
- 参数量（实测）：48,245 = 可训练 23,669 + 两个固定频带滤波矩阵 24,576
- 官方两份 `model_trained_on_dns3.tar` 哈希不同；起点用 `stream/onnx_models/` 那份（产品 ONNX 由它导出），`official.py` 核 sha256
- 原 torch 环境 `_tmp/exp008-torch-venv` 已不存在，改在 worktree 根建 `.venv/`
- `Tensor.unfold` 在时间长度可变时导不出 ONNX，注意力改成 K 个错位切片拼接（数学等价，有测试核）
- 本机用电池时长任务会空闲睡眠：2026-11-16 一次探测 CPU 136 秒，墙钟 56 分钟。已改规矩：超过 1 分钟先申请名额 + `caffeinate -dims` + 记电源
- macOS 没有 `timeout` 命令；限时用 `perl -e 'alarm N; exec @ARGV' …`
- 早期混响 100 ms 的正本归 exp013 `rooms.py`（evalcmp / traindata）；trainer 不切混响，没有第二处表示

## 转交（范围外、要云上或别的线做的）
1. **云上 GPU 续训等价**（cloudrun，codex 第 5 / 6 轮）：本机只验证了 CPU + 合成数据下「续训 = 不中断」逐值相同（`test_resume_equals_uninterrupted`）。云上第一次冒烟时请跑：同配置训 6 步，与「训 3 步停 → 续 3 步」比 `step-0000006.pt` 的 `model` 逐键相等（CUDA 非确定性算子可能让它只近似相等，记下最大差）；并在有小分片的机器上设 `VDN_LOADER_SHARDS` / `VDN_LOADER_RECIPE` 跑 `tests/test_data.py`
2. **正式冒烟的开发份基准**（evalcmp / cloudrun）：`smoke.py --known-dev` 要一份「起点检查点在开发份上的已知读数」，没有它正式冒烟记「未完成」
3. **traindata 的 loader 合入后**：`tests/test_data.py` 里 3 条跳过的对接测试会自动跑

## 待人拍板
（暂无。电平归一开不开是技术题，建议在数据消融里扫开 / 关两组，见 README「默认值与依据」）

## 流水
- 11-16 16:05 报到；大总管 e1 回「照交代做」
- 11-16 16:10 回 evalcmp：早期混响正本归 rooms.py
- 11-16 16:15 建 `.venv`（torch 2.15.0 · onnxruntime 1.32.0 · onnx 1.23.2）
- 11-16 16:40 模型变体 A / B + 12 条测试通过；参数量核清
- 11-16 与 traindata 商定 loader 接口（区间由它给，掩码由我做；第 k 批只由 seed、k 决定）
- 11-16 名额：编 `parity_rs`，25 秒，已还
- 11-16 损失 + 对拍 22 条 + 单元测试 10 条通过
- 11-16 训练循环 13 条通过（修了一个真 bug：开发评估没传 L2-SP 的起点权重，测试先红后绿）
- 11-16 未申请名额跑了一次过拟合探测，墙钟 56 分钟（电池睡眠），已自报
- 11-24 traindata 改成 GPU 混音（D102），data.py 已接；加了「第 k 步拿第 k 批」核对
- 11-24 e1 失联，`noise-canceling-33` 确认接任；名额：冒烟 209 秒，三件全过，已还
- 11-24 export 加 `.json` 报告；README 写完
