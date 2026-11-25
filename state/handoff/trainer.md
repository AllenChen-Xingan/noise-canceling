# 现状页 · trainer（批次 20261116-b19）

- **状态**：H100 冒烟方案大总管已批；两处新代码写完、codex 3 轮达停止线；等大总管推送分支、cloudrun 开机（分支 `exp/025-h100-smoke`）
- **手上**：训练器 · 训练损失 · 模型变体（issue #157，分支 `exp/025-trainer`）
- **交代**：`_tmp/20261116-b19/trainer.md`；大总管 2026-11-24 起是 `noise-canceling-33`（接替 e1）
- **做到哪**（2026-11-24）：交代 1–7 条全部完成。测试 119 条通过，4 条等 traindata / cloud 合入后自动跑。本机冒烟三件事过（合成小样，新判据）。自审 3 个视角 + codex 10 轮，按大总管口径已达停止线；待办 #159、#160
- **下一步**：等验收；云上正式冒烟见「转交」

## 改了哪些文件
全部在 `docs/experiments/exp025-gtcrn-train/trainer/`，说明见该目录 `README.md`：
`models.py` · `losses.py` · `config.py` · `configs/{default,smoke-cpu}.yaml` · `data.py` · `train.py` · `watchdog.py` · `runrec.py` · `smoke.py` · `export.py` · `official.py` · `parity_rs/` · `tests/` · `smoke_reports/` · `reviews/001_trainer-codex.md` · `requirements.txt` · `.gitignore` · `README.md`
另：`state/handoff/trainer.md`（本页）

## 发现
- 参数量（实测）：48,245 = 可训练 23,669 + 两个固定频带滤波矩阵 24,576
- 官方两份 `model_trained_on_dns3.tar` 哈希不同；起点用 `stream/onnx_models/` 那份（产品 ONNX 由它导出），`official.py` 核 sha256
- 原 torch 环境 `_tmp/exp008-torch-venv` 已不存在，改在 worktree 根建 `.venv/`
- `Tensor.unfold` 在时间长度可变时导不出 ONNX，注意力改成 K 个错位切片拼接（数学等价，有测试核）
- 本机用电池时长任务会空闲睡眠：2026-11-16 一次探测 CPU 136 秒，墙钟 56 分钟。已改规矩：超过 1 分钟先申请名额 + `caffeinate -dims` + 记电源
- macOS 没有 `timeout` 命令；限时用 `perl -e 'alarm N; exec @ARGV' …`
- 早期混响 100 ms 的正本归 exp013 `rooms.py`（evalcmp / traindata）；trainer 不切混响，没有第二处表示
- 单价的正本是 `cloud/config.py` 的 `USD_PER_HR_EXPECTED`（H100 $4.39/小时）；trainer 默认值与它一致，比对测试等 cloud 合入后跑
- 冒烟「单批过拟合」判据改过一次：原下限（输出恰好等于目标）被 SI-SNR 的 1e-8 顶成 −10.04，到不了；改为 40 dB 近乎完美参照 + SI-SNR 提高 ≥ 6 dB。事后改判据，AI 定，大总管认可
- `tests/test_review_fixes.py` 579 行，接近 600 行上限；再加测试时按「损失 / 模型 / 训练循环」拆文件

## 转交（范围外、要云上或别的线做的）
1. **云上 GPU 续训等价**（cloudrun，codex 第 5 / 6 轮）：本机只验证了 CPU + 合成数据下「续训 = 不中断」逐值相同（`test_resume_equals_uninterrupted`）。云上第一次冒烟时请跑：同配置训 6 步，与「训 3 步停 → 续 3 步」比 `step-0000006.pt` 的 `model` 逐键相等（CUDA 非确定性算子可能让它只近似相等，记下最大差）；并在有小分片的机器上设 `VDN_LOADER_SHARDS` / `VDN_LOADER_RECIPE` 跑 `tests/test_data.py`
2. **正式冒烟的开发份基准**（evalcmp / cloudrun）：`smoke.py --known-dev` 要一份「起点检查点在开发份上的已知读数」，没有它正式冒烟记「未完成」
3. **traindata 的 loader 合入后**：`tests/test_data.py` 里 3 条跳过的对接测试会自动跑；cloud 合入后单价比对测试会自动跑
4. **云上正式冒烟**（cloudrun）：`smoke.py --config configs/default.yaml --set data.shard_root=… --set data.recipe=… --known-dev <基准.json>`；过拟合要同时过「40 dB 下限」和「SI-SNR 提高 ≥ 6 dB」两条（后者本机还没测过）
5. **待办 issue**：#159（启动期网络盘阻塞，T-1）· #160（心跳改名阻塞的跨运行覆盖，T-2）

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
- 11-24 自审 3 个对抗视角：P1 7 条全修
- 11-24 codex 10 轮（gpt-6-astra，只读）：第 9 轮起按大总管口径，故障注入类记 P2；第 9、10 轮达停止线。记录 `trainer/reviews/001_trainer-codex.md`
- 11-24 名额：冒烟重跑两次（45 s 旧下限不过 → 改判据 → 52 s 过），已还
- 11-24 开 issue #159 · #160（仓库公开经大总管确认是人 2026-11-11 要求的）
- 11-24 已收尾，报大总管
- 11-24 大总管验收通过（本机重跑 119 passed · 4 skipped · 73 s；最大文件 579 行）；PR 排在 evalcmp → cloudrun → traindata 之后，由大总管开；trainer 待命
- 11-24 #157 合进 main（PR #179，2362d715）；大总管派新活：写 H100 冒烟方案。从 origin/main 开 `exp/025-h100-smoke`，方案 `trainer/plan-h100-smoke.md`，待批
- 11-24 大总管批方案（参照读数用 CPU 机同代码算，大总管定，人未参与；乙段 1.5 小时 / 7 美元）。写了 `smoke.py --write-known-dev` 与 `cmp_ckpt.py`，测试先红后绿。名额：全套测试 66 s，139 过 2 跳过，已还。合进 main 73631939。codex 3 轮（P1×4 全修；第 2、3 轮无 P0/P1，达停止线），记录 `trainer/reviews/002_h100-smoke-codex.md`
- 11-25 甲段第一次跑（cloudrun）：113 过 · 1 跳过 · 5 红，都是环境问题（漏装 onnx；git archive 没有 .git，运行记录 commit 为空）。改：方案第 2 步两份 requirements 一起装、代码必须 git clone / bundle 带 .git；`runrec.git_info` 拿不到提交号就在开跑前失败（`tests/test_git_record.py` 先红后绿）
- 11-25 甲段第二次跑（cloudrun，d2477f63）全过：单元测试 122 过 · 1 跳过（MPS），321 s；CPU 参照 hybrid 3.97330（build f032a75eedad80d6），存进 `trainer/smoke_reports/20261125-known-dev-cpu.json`；等乙段
- 11-25 乙段第 1 步（cloudrun）：冒烟只有过拟合不过（损失走完 14%、SI-SNR +3.5 dB，要 50% 与 +6 dB）；其余三件过。测速：每步 0.126 s、取数 0.033 s（中位，100 步）。报告 `trainer/smoke_reports/20261125-h100-overfit-FAILED.json`；已删机，等大总管定怎么重跑
- 11-25 过拟合本机排查（名额）：合成小样 4×2 s 与 32×4 s 都过（走完 111% / 110%，SI-SNR +12.9 / +16.0 dB；占掩码族最好损失距离 73%）。第 2 组期间本机合盖睡眠，墙钟作废；caffeinate 挡不住合盖。报告 `trainer/smoke_reports/20261125-overfit-diag-local-cpu.json`
- 11-25 大总管批云上 CPU 真数据排查：加 `oracle.py` 与 `smoke.py --overfit-diag`（`tests/test_overfit_diag.py` 先红后绿，冒烟相关 42 条过，电池下 125 s）；方案第八节
- 11-25 云上 CPU 真数据排查（cloudrun，1831886c，1237 s）：走完 40 dB 下限距离 14.4%，与 H100 几乎逐值相同（终值 2.3814 对 2.3808）→ H100 没问题。oracle 损失 −1.312（SI-SNR 23.5 dB），40 dB 下限 −3.307 低于它 → 不可达；走完 oracle 距离 20.6%。报告 `trainer/smoke_reports/20261125-overfit-diag-cloud-cpu-real.json`；判据改法待大总管定
