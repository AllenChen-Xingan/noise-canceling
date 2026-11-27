# 现状页 · trainer（批次 20261116-b19）

- **状态**：训练器加两样记录（#236）。分支 `feat/trainer-train-logs`：pi trainer-train-logs 写了 `trainlog.py`（梯度裁剪统计 · 开发评估存 wav），接进 train.py；trainer 复现红绿，本机 CPU 小训练核过文件数、采样率、位深和 4 个裁剪字段。codex 后开 PR 交大总管验收
- **手上**：训练器 · 训练损失 · 模型变体（issue #157）
- **交代**：`_tmp/20261116-b19/trainer.md`；新规则 `_tmp/20261116-b19/common-pi-rule-20261126.md`（代码派 pi 写）；大总管固定名 `noise-canceling-meta`（人定 2026-11-26，轮换后不变；旧名 noise-canceling-87 停用）
- **做到哪**（2026-11-26）：#228（正式训练改钉第五批）已合入。cloudrun 跑完正式训练（代码 7b52d395，两台 H100）；trainer 本机读取回包：6 个 best.pt sha256 与清单一致。B 的条件调制层练完仍全为 0（喂全零的直接后果），注意力输出层范数 0.22–0.28，拿掉注意力输出谱变 2.4–3.0%（一段示例录音）。待办 #198 · #210 · #217 · #231 · #233
- **下一步**：codex 评审 → PR（Closes #236）→ 大总管在独立副本跑 CPU 小训练验收。学习率 / 裁剪 / 衰减的短跑对比等 7 份成绩单、开训前清单和人点头后另派

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
- 11-25 大总管定过拟合判据第 3 版（事后改判据：AI 提、大总管定）：前 4 段 + oracle 下限 + 判据不可用（退出码 3）；32 段读数只作参考。`tests/test_overfit_gate.py` 7 条先红后绿，冒烟相关 49 条过。方案第五节加 H100 估时（推断），第九节加 CPU 核对命令
- 11-25 第 3 版判据云上 CPU 核对（cloudrun，a8168c98）：只有过拟合不过：前 4 段走完 oracle 距离 19.6%（要 50%），SI-SNR +2.5 dB（要 6）；其余三件过。报告 `trainer/smoke_reports/20261125-cpu-gate3-FAILED.json`；不降门槛，已报大总管
- 11-25 加 `overfit_probe.py` 与 `smoke.py --overfit-probe`（学习曲线到 2000 步 · 参数与梯度 · 学习率 3e-4 / 3e-3）；`tests/test_overfit_probe.py` 6 条先红后绿；方案第十节
- 11-25 加 `data_probe.py`（对齐 · 目标检查与 wav · 逐个关配方旋钮 6 组），并进 `--overfit-probe`；`tests/test_data_probe.py` 8 条先红后绿
- 11-25 `--overfit-probe` 云上 CPU 跑完（cloudrun，21e2e5b7，4429 s）：2000 步走完 oracle 距离 30.7%，学习率 3e-4/3e-3 差 3 个百分点内，梯度、对齐、目标、关旋钮都没找到故障。读数与建议 `trainer/smoke_reports/20261125-overfit-probe-reading.md`；建议先用官方 DNS3 权重做独立参照，待大总管定；H100 仍不开
- 11-25 判据第 4 版（大总管定）：`gate_v4.py` 正本、`dev_gate.py`、smoke 改用第 4 版、逐段读数；重判诊断为过（`smoke_reports/20261125-rejudge-v4.json`）。codex 3 轮（`reviews/003_gate-v4-codex.md`）满足停止线；待办 P2：诊断输出并发覆盖（不原子占用目录）
- 11-25 H100 乙段第 1 次（cloudrun，7eecbbe8）：冒烟全过（第 4 版过、第 3 版不过只记录；下降 24.8%，SI-SNR +2.53 dB；开发份读数 3.97339 对 CPU 3.97330；每步 0.126 s，取数 0.021 s），报告 `smoke_reports/20261125-h100-smoke-v4.json`。第 2 步因镜像没装 git 拒跑，第 3、4 步没跑；方案第四节乙加装 git
- 11-25 加人声忽大忽小损失 `loss.w_pump`（默认 0，D107），分支 feat/trainer-pump-loss 7f28e1a4：可微复刻 V-PUMPV，与正本对拍逐值一致；w_pump = 0 时与改动前 7eecbbe8 训练检查点逐值相同；238 条测试过
- 11-25 H100 乙段重跑（cloudrun，7eecbbe8，跳过第 1 步）全过：dev_gate A 3.90371、B 3.90497（第 0 步 3.97330）；续训逐值相同；3 个同时跑正常。报告进 `smoke_reports/20261125-h100-*.json`；方案第五节写正式训练估时估费（串着跑 1.8–1.9 h / 7.8–8.3 美元，同时跑 1.24 h / 5.4 美元），方案定稿
- 11-25 大总管定：正式训练每次 3 个同时跑；前 500 步慢于 0.39 s 的 1.2 倍由训练器自动判（guards ④）；D88 的 8 个 wav 留云上。#194 已合进 main（1c251d42），本地 exp/025-h100-smoke 已删；feat/trainer-pump-loss 合了一次 main（7bc8d573，278 条过）
- 11-25 开 feat/trainer-guards（从 origin/main 1c251d42）：训练时自动保护 ①–⑤，见方案第十二节
- 11-26 会话中断后接回（trainer-02）。feat/trainer-guards：codex 第 1、2 轮的 P1/P2 都已修，全套 221 过；train.py 中断前未提交的 2 行（续训时把最好成绩写回 run.json）是新规则前亲手写的，照样提交。开 issue #195。之后要改代码按新规则派 pi
- 11-26 pi 提交范围（在 trainer worktree 里，auto-commit hook 把 pi 半截代码以 Claude 署名提交了，大总管要求记下）：
  - pi trainer-guards-devnan（codex guards 第 3 轮 #1，花费 0.0099 美元）：1ba6c800..a1f00093（hook 提交 e4c58e28 · a1f00093）；验收提交 b37a8560 当时没加 `Code: pi` 行，在此补记：代码由 pi（isoquant/deepseek-v4.1-flash-1017-fast）写
  - pi trainer-guards-calibnan（codex guards 第 4 轮 #1，花费 0.0064 美元）：b37a8560..（hook 提交 13c19e5c · 1f1b7450，加本次验收提交）
  - 之后的 pi 改用 `pi-worker start --repo <主仓> --base feat/trainer-guards`，在自己的 worktree 与分支提交，验收后快进合入
- 11-26 pi trainer-guards-finite（改结构：唯一查有限值入口 · 停止原因优先级 · 防绕过扫描）：在自己的 worktree 与分支 w/trainer-guards-finite，提交 6be34442..370af014（Agent: pi），花费 0.0428 美元；trainer 验收后快进合入 feat/trainer-guards。全套 369 过
- 11-26 pi trainer-guards-watchdog（codex guards 第 5 轮 #3）：自己的 worktree，提交 7da07932..f7fef94d（Agent: pi），花费 0.0039 美元；快进合入 feat/trainer-guards
- 11-26 codex guards 第 6 轮：只 1 条 P2；按本项目口径第 5、6 轮连续两轮无 P0/P1，停。剩余 P2 开 issue #198。推送 feat/trainer-guards、开 PR（Closes #195），交大总管验收。pi 花费合计：devnan 0.0099 · calibnan 0.0064 · finite 0.0428 · watchdog 0.0039 = 0.063 美元
- 11-26 早先用 --cwd 起的 pi trainer-guards-devnan · trainer-guards-calibnan 跑在 trainer 自己的 worktree 里，没有单独的 worktree 可清；trainer-guards-finite · trainer-guards-watchdog 已 pi-worker clean
- 11-26 正式训练配置草稿（issue #200，分支 feat/trainer-formal-config 从 main ab0f4f0a）：pi trainer-formal-config 提交 4c687043..2228fc9b（Agent: pi），花费 0.0113 美元；trainer 读 diff、亲手跑 test_formal + test_train 33 条过、`formal.py check` 现退出 1（2 个 TODO + 场景空），快进合入。不开机，等第二批数据登记、第二版差距表、D110

- 11-26 加训场景（issue #209，分支 feat/trainer-target-scenes 从 main da9008e1）：pi trainer-target-scenes 提交 d9c5f527..3afe9e9c（Agent: pi），花费 0.0027 + 0.0050 = 0.0077 美元。pi-worker 中途 runner / tail 进程死了，pi 收不到「准许」卡约 50 分钟，stop 杀不掉；trainer kill 后用 `--cwd --session` 接回原会话。trainer 复现第 1 个提交的红（8 败 17 过），读 diff、亲手跑 test_formal + test_train 41 条过、`formal.py check` 只报两处路径占位、退出 1，文件 ≤ 600 行，快进合入。pi 是 `--cwd` 接回的，`pi-worker clean` 不认，worktree `noise_canceling.worktrees/trainer-target-scenes` 与分支 `w/trainer-target-scenes` 请大总管删

- 11-26 数据消融（issue #212，分支 feat/trainer-ablation-scenes 从 main f1efacbc）：pi 共起 7 次（trainer-ablation 0.0419 · fix1 0.0135 · fix2 0.0216 · fix3 0.0282 · fix4 0.0084 · fix5 0.0075 · fix6 0.0066，合计 0.1277 美元），提交 ad5980f6..95afdd65（Agent: pi），fix1 起用 `--cwd` 接同一个 worktree，trainer 用 merge 合入。trainer 自查发现 2 处（基线不完整仍算 base · 只比数据字段），codex 7 轮：P1 共 6 条全修（整份配置比对 · 设计固定项 · 运行身份 · 续训核对开发份 build · 旧记录续训兼容 · dev_gate 与冒烟参照用开发份 build），每条都由 trainer 在修复前的提交上复现红。P2：metrics 畸形行、缺键与 null 已修；run.json 结构变化留 #217。合入后 10 个测试文件 239 过 2 跳过。pi 两次 `pi-worker stop` 停不掉，trainer 用 kill；pi 一次用了 git stash（共享栈，已当场归还，已叮嘱不再用）
- 11-26 新基线开跑准备 + #213（分支 feat/trainer-b4-run）：pi 起 3 次（trainer-b4-pin 约 0.039 · trainer-scene-src 约 0.018 · trainer-b4-fix1 约 0.012，合计约 0.069 美元，按各自最后一次花费通知），提交 31037607..0584092e（rebase 后）、a9cd0d80..0cd4e275、bddb7ed9..43eb949f（Agent: pi），trainer 用 merge 合入。#213 与 fix1 的红由 trainer 在修复前的复制件上复现（6 败 · 8 败）。codex 4 轮（记录 reviews/008_b4-run-codex.md）：P1 共 3 条全修（旧变体记录被放行 · 乙段时限超批准 · 甲段时限超 0.1 美元），P2 2 条已修；第 3、4 轮无问题，达停止线。合入后 11 个测试文件 294 过 2 跳过
- 11-26 新基线一单的读数（分支 feat/trainer-b4-results）：cloudrun 取回主仓 _tmp/20261126-trainer-b4/，两个包 sha256 本机重算一致（O e6925628…、ablation 8aa73703…）。trainer 本机跑 `ablation.py judge` 退出 0：base 3.84142、噪声底 0.00779；混响 +0.091 有用、均衡 +0.022 有用（证据不强）、削波 −0.005 / 多噪声 +0.005 / 高信噪比 +0.001 分不出。3 个 A 同时跑每步实测 0.342–0.349 s（之前推断 0.347 s）。没用 pi，没花模型费。
- 11-26 正式训练改用第五批（分支 feat/trainer-formal-b5）：pi trainer-b5-pin 提交 dd5ed3e4..2b0cd25a（Agent: pi，0.0106 美元），trainer 用 merge 合入。红灯由 trainer 在 dd5ed3e4 的复制件上复现（9 败 95 过）；合入后 trainer 全部测试 531 过 2 跳过；formal / ablation 两个 check 退出 0。codex 2 轮（P2 各 1 条，都当场改了），达停止线，记录 trainer/reviews/009_formal-b5-codex.md。
- 11-26 正式训练读数（分支 feat/trainer-formal-results）：cloudrun 取回主仓 _tmp/20261126-trainer-formal/（运行包 a0b7b3c7… / 6c7e2c09…）。trainer 本机只读：6 个运行训满 5000 步、退出码 0、自动保护都没触发；开发份 hybrid 3.8132 → A 3.6532 / B 3.6518（3 种子均值），种子极差 0.008–0.025，B − A 在 ±0.002 以内。另量了 B 多出部件离初值多远（smoke_reports/20261126-formal-ab-drift.json）。两台新 H100 墙钟慢 1.4–2.1 倍，但每步耗时不变，慢在准备期，开 #233。没用 pi，没花模型费。
- 11-26 训练记录两样（issue #236，分支 feat/trainer-train-logs）：pi trainer-train-logs 提交 b6e51eea..4394afd0（Agent: pi，0.0158 美元），trainer 用 merge 合入（bd8e7b7c）。红灯由 trainer 在 b6e51eea（测试导入即报缺模块）和 13966a72（参照缺一份不补，1 败）的复制件上复现；HEAD 上 test_trainlog + test_train 28 过。本机 CPU 合成数据 20 步：dev_audio 2 个评估目录各 4 个 wav、ref 8 个，都是 16 kHz PCM_16；5 条训练行都有 4 个裁剪字段，clip_n 之和 20。
