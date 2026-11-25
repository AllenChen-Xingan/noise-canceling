# 方案：H100 上的开训前冒烟（014 第 3 步）

状态：方案稿，待大总管批；**不开机**。写于 2026-11-24，trainer。分支 `exp/025-h100-smoke`（从 origin/main 2362d715 开）。
开机、传文件、删机都由 cloudrun 照第四节做；trainer 只给命令、收报告、判过不过。

## 一、要回答的三件事

1. **三件事都过**（`smoke.py`，判据见 `README.md`「冒烟三件事」）：
   - 初始损失合理：B 与 A 相对差 ≤ 1e-5，且起点损失低于随机初始化
   - 单批过拟合：**两条都要过**。损失走完「初值 → 近乎完美输出（目标 + 40 dB 白噪声）的损失」距离的一半以上；这一批 SI-SNR 提高 ≥ 6 dB。B 的注意力输出层权重要动过
   - 原检查点复现：官方 `mix.wav` 的输出与 `enh.wav` 最大差 ≤ 2 个 16 位量化步；开发份读数与已知读数差 ≤ 1e-3（来源见第三节）
2. **每步耗时**：正式配置（B，批量 32 × 4 s，`num_workers` 8）在 H100 上每步多少秒；准备期（标定 + 建开发份）和每次开发评估各多少秒
3. **估时估费**：由第 2 条推算「A、B 两组 × 3 个种子 × 5,000 步」的总时长和总花费（公式见第五节）

另外顺带做两件便宜的事，各 1–3 分钟：
- GPU 上「续训 = 不中断」核对（现状页转交第 1 条，codex 第 5、6 轮留下的）
- 3 个训练同时跑在一张卡上时每步多少秒。这是省钱最大的一项：模型只有 23,669 个可训练参数，一个训练用不满 H100（推断，未测）

## 二、前提

- traindata 的现行 build `f032a75eedad80d6` 已登记进 main（PR #182 合并后）。分片还在网络盘 `/workspace/shards/first-batch`，PR #182 写明分片与旧 build 逐字节相同
- 代码：用 PR #182 合并后的 main 提交（下称 `<提交>`），放进网络盘新目录 `/workspace/repo-main-<提交前 8 位>`，不覆盖旧 checkout
- 官方 GTCRN 源码：上游 `github.com/Xiaobin-Rong/gtcrn` 提交 `502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73`（与 evalcmp 基线同一提交）。起点检查点由 `official.py` 核 sha256 `250fa282…f06442`，对不上就停
- 环境：用 data 线的 `bash cloud_steps.sh setup` 建的 `.venv`（Python 3.12.14，torch 2.15.0，numpy 2.5.4 …），再补 trainer 自己的 3 个包（`einops==0.7.0`、`pyyaml`、`pytest`）。不用镜像自带的 torch 2.5.1
  - 风险：trainer 在本机是 Python 3.13 测过的，3.12 没测过（推断：没用 3.13 专有语法，应该能跑）。第四节第 3 步先跑一遍单元测试来核
- 已花：`cloud/costs.tsv` 里机器估算合计约 1.43 美元，另有网络盘按月摊销；总上限 60 美元（D101）

## 三、开发份「已知读数」从哪来（需要你定）

`smoke.py` 遇到正式数据时，必须用 `--known-dev` 给一份「起点检查点在开发份上的已知读数」，否则记「未完成」。evalcmp 的基线成绩单是 480 段评测场景上的分数，不是这份开发份上的训练损失，不能直接拿来用。
**建议**：在 CPU 机器上算一次，作为参照。
- 怎么算：用同一份开发份（`DevSet` 在 CPU 上渲染，结果固定），起点模型在 CPU 上跑出 `hybrid` 的读数
- 怎么比：H100 冒烟时 GPU 上的读数要和它相差 ≤ 1e-3
- 能证明什么：GPU 上的取数、模型与损失和 CPU 参考一致
- 不能证明什么：「原检查点成绩没变」这件事，CPU 和 GPU 跑的是同一份代码，所以它证明不了。这一条由第一节「原检查点复现」（对上游 `enh.wav`）证明
如果你不回：默认按这个建议做，报告里写明「已知读数 = 同代码 CPU 参照」。

为此 trainer 要先加两处小代码（都带测试，每个文件仍 ≤ 600 行），你批了再写：
1. `smoke.py --write-known-dev <文件>`：只建开发份、跑起点模型、写出 `{"hybrid": …}` 后退出，不跑过拟合（CPU 上过拟合 150 步要十几分钟，推断）
2. `cmp_ckpt.py <检查点 1> <检查点 2>`：逐键比两个检查点的 `model`，打印最大差和不同的键名。续训核对用

## 四、跑法（给 cloudrun）

下面 `R=/workspace/repo-main-<提交前 8 位>`，`T=$R/docs/experiments/exp025-gtcrn-train/trainer`，`D=$R/docs/experiments/exp025-gtcrn-train/data`，`PY=$R/.venv/bin/python`，`G=/workspace/gtcrn-502ebfab`，`O=/workspace/runs/exp025-smoke-<UTC 时间>`。
每条后台命令都用 `rp.py job-start` 发、按运行编号 `job-wait`；任何一步退出码非 0 就停下，把日志尾发给大总管转 trainer。

### 甲、CPU 机（cpu3c 4 核，约 0.12 美元 / 小时）：备环境 + 算 CPU 参照

`python3 rp.py up --cpu --max-hours 1.5 --yes`，然后：

1. 拉代码与官方源码：`git clone <仓库> $R && git -C $R checkout <提交>`；`git clone https://github.com/Xiaobin-Rong/gtcrn $G && git -C $G checkout 502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73`
2. 装环境：`cd $D && bash cloud_steps.sh setup && uv pip install --python $PY einops==0.7.0 pyyaml pytest`
3. 单元测试（不要 Rust 的那几条会自动跳过）：`cd $T && VDN_GTCRN_DIR=$G VDN_LOADER_SHARDS=/workspace/shards/first-batch VDN_LOADER_RECIPE=$D/recipe.json $PY -m pytest -q tests/ 2>&1 | tail -20`
   - 预期：除对拍（要 `parity_rs`，云上不编）外全过；loader 的 3 条对接测试这次在真分片上跑
4. CPU 参照：`cd $T && VDN_GTCRN_DIR=$G $PY smoke.py --config configs/default.yaml --set device=cpu --set data.shard_root=/workspace/shards/first-batch --set data.recipe=$D/recipe.json --write-known-dev $O/known-dev-cpu.json`
5. 删机：`python3 rp.py down <机器> --yes`

估时（推断，未量化）：装 torch 2.15.0 要下约 2–3 GB；建开发份 640 段在 CPU 上渲染。整段估 30–60 分钟，约 0.1 美元。

### 乙、H100（约 4.39 美元 / 小时）：冒烟 + 测速

`python3 rp.py up --max-hours 1.5 --yes`。开出来先 `rp.py sh <机器> 'nvidia-smi; nproc'` 记显卡与核数。环境在网络盘上，不用重装。

1. **冒烟**（预计 5–10 分钟）：
   `cd $T && VDN_GTCRN_DIR=$G $PY smoke.py --config configs/default.yaml --set data.shard_root=/workspace/shards/first-batch --set data.recipe=$D/recipe.json --timing-steps 100 --known-dev $O/known-dev-cpu.json --out $O/smoke-h100.json`
   退出码 0 = 全过；1 = 有一件不过，报告照样写出，**也要取回**
2. **真训练测速**（300 步，预计 5–10 分钟）：量准备期、稳态每步、开发评估三段时间
   `cd $T && VDN_GTCRN_DIR=$G $PY train.py --config configs/default.yaml --set run.out_dir=$O --set run.name=speed-B --set schedule.max_steps=300 --set schedule.eval_every=100 --set schedule.ckpt_every=100 --set data.shard_root=/workspace/shards/first-batch --set data.recipe=$D/recipe.json`
   A 组再跑一次：把 `run.name=speed-B` 换成 `speed-A`，加 `--set model.variant=A`
3. **续训核对**（预计 2–3 分钟）：同配置，`schedule.max_steps=6`、`ckpt_every=3`、`eval_every=1000`
   - 一次训满：`run.name=res-full`
   - 分两次：`run.name=res-split`，先 `max_steps=3` 跑完，再用同名、`max_steps=6` 续上
   - 比：`$PY cmp_ckpt.py $O/res-full/ckpt/step-0000006.pt $O/res-split/ckpt/step-0000006.pt`。CUDA 的非确定性算子可能让它只近似相等，记下最大差
4. **3 个同时跑**（预计 5 分钟）：第 2 步的 B 配置，`max_steps=200`，`num_workers=6`（3 × 6 = 18 个取数进程，容器配额约 22 核），`run.name` 分别为 `par-1`…`par-3`，用 3 个 `job-start` 同时发
5. 打包取回：`tar czf /workspace/out/trainer-smoke-<UTC 时间>.tar.gz -C $O --exclude='*.pt' .`，取回主仓 `_tmp/<日期>-trainer-h100-smoke/`，不覆盖；检查点不取回
6. 删机，报账上 0 台和本次花费

H100 这段总计估 30–45 分钟（推断），约 2.2–3.3 美元。上限 1.5 小时，最坏约 (1.5 + 5/60) × 4.39 ≈ 6.95 美元。

## 五、估时估费怎么算

从第四节乙的第 2 步的 `metrics.jsonl` 读数。每行带 `billed_s`（这次运行从开始计的秒数）：
- `t_setup`：第 1 步那行的 `billed_s`（含建模型、标定 200 批、建开发份 640 段）
- `s_step`：第 120 步到第 300 步之间，`billed_s` 差 ÷ 步数，扣掉中间那次开发评估
- `t_eval`：第 200 步前后两行 `billed_s` 的差，减去这几步的正常耗时

单个正式运行：`T_run = t_setup + 5000 × s_step + 10 × t_eval`（每 500 步评估 1 次，共 10 次；存检查点按实测，预计可忽略）。
两组 × 3 种子 = 6 个运行：
- 一个一个跑：`总时长 = 6 × T_run`
- 每次同时跑 k 个（按第 4 步实测的每步耗时 `s_step(k)`）：`总时长 = ⌈6 / k⌉ × T_run(k)`
- 另加每台机器的开机和收尾时间，按本次乙段实测的非训练时间算
- `总花费 = 总时长（小时）× 4.39 美元`

单价的正本是 `cloud/config.py` 的 `USD_PER_HR_EXPECTED`。每个运行另有训练器自带的 20 美元 / 6 小时保险，硬止损在 cloud。

举例（推断，数字只为示范公式，要用实测替换）：
- 设 `s_step` = 0.15 s，`t_setup` = 5 分钟，`t_eval` = 20 s：
  - 单个运行：5 + 12.5 + 3.3 ≈ 21 分钟
  - 6 个一个一个跑：约 2.1 小时，约 9.2 美元
- 设 `s_step` = 0.5 s：
  - 单个运行：约 50 分钟
  - 6 个一个一个跑：约 5 小时，约 22 美元
- 后一种情况下，同时跑 3 个是否值得，看第 4 步的实测

## 六、带回什么、谁判

- trainer 收到包以后做三件事：
  - 把 `smoke-h100.json`、`known-dev-cpu.json`、3 份 `metrics.jsonl` 的摘要、续训最大差、估时估费表放进 `trainer/smoke_reports/`
  - 在 `README.md` 写读数和过不过
  - 报大总管
- 过不过只看 `smoke-h100.json` 里的 `pass` 和每件的 `pass`，不靠我看着觉得行
- 三件都过、两条过拟合判据都过，才建议进数据消融（014 第 4 步）

## 七、如果出意外

- 单元测试在 Python 3.12 上红：停下，不开 H100。trainer 修到绿再来
- 起点检查点哈希对不上：上游提交拉错了，重新按提交号拉
- loader 拒绝 build（build id 对不上）：代码不是 PR #182 合并后的提交，换对的提交
- 过拟合只过一条：照样带回报告，判「不过」。trainer 分析是判据的问题还是训练的问题，先报大总管，不当场改判据
- 取数是瓶颈（`timing.data_s_median` 比 `step_s_median` 还大，或 GPU 利用率低）：照实记下，估费按实测算。加 worker、预渲染这些改法另立一条
- 机器开不出或没货：照 cloudrun 现行做法，不在本方案内处理
