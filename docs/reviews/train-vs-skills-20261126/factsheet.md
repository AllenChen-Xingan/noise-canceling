# exp025 GTCRN 微调训练管线 · 事实清单（不是结论）

> 来源：`origin/main` @ `e78ad1fd`（2026-11-26 17:54 -0800，合并 #235）。只读 `git archive` 取出，未运行任何代码或测试。
> 路径前缀省略：`T/` = `docs/experiments/exp025-gtcrn-train/trainer/`，`D/` = `.../data/`，`C/` = `.../cloud/`，`E/` = `.../eval/`，`B/` = `.../baseline/`。
> 标注：【码】读源码 ·【报】仓内报告/README 里的实测读数 ·【推】我（子代理）的推断 · 「未找到」= 搜过没有。

---

## 1. 数据

**来源**（`D/recipe.json`、`D/split_rules.json`、`D/builds/df759bb8d606a209/build.json`）【码】
- 人声：LibriSpeech train-clean-100 一个来源（`recipe.json` 的 `speech.train`）。英文朗读，无中文（014 第六节写明「中文人声 D99 第一轮不加」）
- RIR：OpenSLR 28（600 间模拟房 + 3 个实测库；AIR 整库、RWCP 整库各算一个单位，见 `D/README.md:62-70`）。分片时按 exp013 `rirs.Measured.valid` 丢掉 3307 条（build.json `dropped_rir`）
- 噪声：QUT-NOISE · TUM 风噪 · DEMAND（train 时段）· CochlScene 3 类 · BUT ReverbDB 底噪（`but-reverbdb-silence-b5`）。13 个场景池映射见 `recipe.json` `noise_pools`
- 已知替代：rail-air-cabin 只有地铁车厢录音代替飞机舱；meeting-room 用的是三间**办公室**底噪，无会议突发声（`recipe.json` target_scene.src）
- 正式 build `df759bb8d606a209` 计数【报】：LibriSpeech train 22,952 句 / dev 2,775 句；RIR train 39,354 / dev 5,553；CochlScene train 16,193 / dev 1,436；QUT train 830 / dev 173；DEMAND train 480 / dev 96；TUM train 367 / dev 29

**切分**（`D/split_rules.json`；种子 20261116；四份 train · dev · final · d63）【码】
- 按「单位」切，`sha256(种子:来源:单位)` 排序 + 最大余数法（`D/README.md:49-51`）
- LibriSpeech train-clean-100：按说话人 0.8 / 0.1 / 0 / 0.1（train/dev/final/d63）；dev-clean + test-other 只进 final / d63（各 0.5）
- QUT · DEMAND：按时间段切（QUT 0–0.5 train · 0.5–0.6 dev · 0.6–0.8 final · 0.8–1.0 d63；DEMAND 0–0.5 train · 0.5–0.6 dev）
- OpenSLR 28 按房间；CochlScene 按录音人；BUT 按「房间 + 扬声器布置」
- 已知例外：hotel-room（BUT Room112）开发份与训练份是**同一间房**（`recipe.json` noise_pools._说明）
- 只有 train 与 dev 进分片；final / d63 不进训练 build（`D/README.md` 6.4 第 3 条）

**与评测集的泄漏检查**（`D/leakcheck.py`）【码 + 报】
- 对照 exp013 正式 build `20261103230756-17f5` 的 `exclude.json`（`leakcheck.py:15-17`）
- 查 train · dev · final 三份可取到的**全部**原料（`GUARDED`，`leakcheck.py:18`）。`exclude.json` 出现不认识的键就直接红
- 查的类：说话人 · DEMAND 时段 · ESC-50 · 合成函数 · RIR 文件 · 保留房间 · 房间噪声段 · 其他段
- 正式 build 结果：`ok: true`，`n_hits: 0`，查 102,445 行，10 个来源（build.json `leak_check`）
- `loader.py:221-225`：启动时 build 未登记或交集检查不过就 `SystemExit`

**信噪比**（`recipe.json` knobs.snr）【码】
- 均匀分布 −5 ~ 15 dB；风噪下限 −10 dB（027 推断）
- 写明官方 SEtrain 实际是名义值的 1.5 倍（−7.5 ~ 22.5 dB），本配方按真实 SNR 取
- `high_snr` 旋钮（默认关）：10% 段 20–40 dB，5% 段不加噪声

**混响**（knobs.reverb）【码】
- `prob 1.0`：每段都卷 RIR
- 目标 = 直达声 + 前 100 ms 早期混响。正本是 exp013 `rooms.EARLY_REVERB_MS` 与 `early_rir`
- 过拟合诊断记到 4 段里 3 段 T60 为 2.2 / 2.7 / 2.5 s【报，`T/smoke_reports/20261125-overfit-probe-reading.md`】

**其他增强**（全部在 `recipe.json`）【码】
- 电平：N(−28 dBFS, 10 dB)，截在 [−60, −10] dBFS
- 随机二阶均衡：每段都加，r_max 0.375；**人声的滤波同时作用在目标上**
- 削波：概率 0.75，阈值 U(0.85, 1.0)，只加在输入上
- 噪声段数：2 段或 3 段各 50%，第 2 段起相对第 1 段 U(−6, 0) dB
- 目标场景：4 个场景（rail-air-cabin · canteen · meeting-room · transit-hub）占每批 2/3；另 1/3 回放其他场景，逐样本标 `meta.replay`
- 近静音段：关（`near_silent.on: false`）
- `interf_ivs` 一律为空：没有旁人说话（`D/README.md:94`）

**混音执行**【码】
- 训练批在 GPU 上混音（`T/data.py:165-178`，`render="gpu"`）；CPU 版 `mix.py` 是正本
- `gpu_mix.py` 对拍 CPU 版，判据逐样本差 ≤ 1e-4（`D/README.md:26`）。README 七节写「云上 GPU 对拍还没跑」（`D/README.md:200`），之后有没有补跑未找到记录
- 开发份 `DevSet` 是 CPU 渲染、结果固定（`T/data.py:192-201`）

**数据版本与哈希**【码】
- `build_id` = sha256（分片索引 + 切分表哈希 + 各分片哈希 + 配方 sha256 + 数据代码 sha256）取前 16 位
- 进哈希的数据代码有 6 个：`mix.py`、`gpu_mix.py`、`loader.py`、`shard.py`、`leakcheck.py`、`split.py`，外加 exp013 的 `dsp.py`、`rooms.py`、`rirs.py`（`D/loader.py:29-33`、`:82-93`）
- 每个 build 都登记在 `D/builds/<id>/build.json`（仓里有 16 个）
- `T/buildpin.py` 与 `formal.py check` / `ablation.py check` 开训前核配方与代码 sha256
- 副作用：旧分片只能在当时的提交上读（`D/README.md:203`）
- 样本可复现：第 i 个样本只由 (seed, 份, i) 决定，第 k 批只由 (seed, k, B) 决定（`D/loader.py:231`）
- 开发份 = 20 批 × 32 段 = 640 段（`dev_batches=20`，`T/data.py:195`；`dev_n=2000` 只是上限）。标定另取 200 × 32 = 6400 段（`T/data.py:180-190`）

## 2. 基线与 sanity

**原版 GTCRN 基线成绩单**【报】
- 已跑：原版 GTCRN 不回混，480 段评测集（exp013 6323），有 `--qa`
- 产物：`B/runs/rerun-20261126-013313/scorecard.json` 与差距表 `B/gap/v2-20261126/README.md`
- 最弱前 4：rail-air-cabin 14 处不过 · canteen 13 · meeting-room 11 · transit-hub 11。这 4 个就是 D113 加训场景
- 「起点检查点读数」有两种：
  - 冒烟第 3 件：在官方 `mix.wav` 上和官方 `enh.wav` 比，最大差 3.05e-5，即 1 个 16 位量化步（判据 ≤ 2 步，`T/README.md:84`）
  - `--known-dev`：起点模型在开发份上的 hybrid CPU 参照。冒烟时必须给，跨设备容差 1e-3（`T/smoke.py:229-260`）。正式 build 上的参照是 3.81324（`T/README.md:132`）

**训练开始时的损失检查**（冒烟第 1 件）【报】
- 判据：B 与 A 相对差 ≤ 1e-5，且起点损失 < 随机初始化
- 读数（b4 build）：A = B = 3.7014，随机初始化 15.17（`20261126-h100-b4-smoke.json`）

**单批过拟合**（冒烟第 2 件，正本 `T/gate_v4.py`）【码 + 报】
- 判据前后改过 4 版，前 3 版都判了「不过」
- 现行第 4 版：开发份第 0 批前 4 段，150 步，lr 1e-3。要求同时满足：
  - 训练损失相对下降 ≥ 15%（`DROP_FRAC_MIN`，`gate_v4.py:17`）
  - SI-SNR 提高 > 0 dB
  - 主干 4 块第 1 步梯度非零、有限
  - B 的注意力输出层动过
- 第 3 版的两条降为只记录：走完 oracle 距离 ≥ 50%、SI-SNR +6 dB
- 第 4 版是看到结果后改的（「事后改判据，大总管定，人未参与」，`gate_v4.py:1,20-23`）
- b4 读数：下降 20.5%，SI-SNR +2.33 dB，走完 oracle 距离 19.8%。`verdict=pass`，按第 3 版是 `verdict_v3=fail`

**训练有没有用**（`T/dev_gate.py`）【码 + 报】
- 判据：A、B 各训 300 步后，开发份 hybrid < 第 0 步 CPU 参照 − 1e-3（`dev_gate.py:58`）
- 正式 build 上：A 降 0.0352，B 降 0.0361，都过

**其他已有的检查**【报，`T/README.md:56-74`】
- B 零初始化后输出 = A（差 ≤ 1e-6）
- 不看未来帧
- 冻结的批归一化：训练模式 = 评估模式
- 有限差分梯度检查
- L_lvl / L_on 与 Rust verifier 对拍（31 条）
- 默认开关下的损失 = 官方 HybridLoss
- ONNX 对拍

## 3. 模型

**两个变体**（`T/models.py`）【码】
- A：官方 `GTCRN` 原样（`models.py:122-133`）
- B：A + 时间注意力 + 条件输入口子（FiLM），都插在第二个 DPGRNN 之后、解码器之前（`:87-116`）
  - 注意力：每个子带回看 K=32 帧，2 个头，残差接入；输出层零初始化（`:34-35`）
  - FiLM：不带偏置，零初始化（`:66-67`）

**参数量**【报】
- 总计 48,245 = 可训练 23,669 + 固定 24,576（两块 ERB 矩阵，官方 `requires_grad=False`）
- B 比 A 多 1,344 个可训练参数（`T/README.md:48-52`）

**初始化**【码】
- 从预训练起步：`model.init: official`，在 `train.py:66-67` 调 `load_base(load_start_state())`
- 起点检查点是 `stream/onnx_models/model_trained_on_dns3.tar`，开跑前核 sha256 `250fa282…6442`（`T/official.py:21-22,66-70`）
- B 新加的层：q / kv 按 `torch.manual_seed(run.seed)` 随机初始化（`train.py:64`）

**冻结的部分**【码】
- 不冻结任何可训练层。优化器拿全部 `requires_grad` 参数（`train.py:264`）
- 批归一化冻结统计量：BN 层留在评估模式，缩放与偏移照常训（`train.py:167-174`，`bn: frozen`）
- ERB 矩阵官方本来就不训

**FiLM 实际状态**【报】
- 条件输入一律喂全零（`train.py:74-77`，D98），所以 FiLM 权重的梯度恒为 0
- 正式训练后 FiLM 权重最大绝对值仍是 0。正式训练里的 B 实际是「A + 时间注意力」（`T/README.md:157`）

## 4. 损失

**总式**：`L = HybridLoss + w_lvl·L_lvl + w_on·L_on [+ w_sp·L_sp + w_kd·L_kd]`（`T/losses.py:3`）【码】

**HybridLoss**：官方 `resources/gtcrn/loss.py` 原样导入【码】
- 式子：`30·(实部 MSE + 虚部 MSE)`（压缩复谱 s/|s|^0.7）`+ 70·幅度 MSE`（|s|^0.3）`+ 1·(−log10 SI-SNR 比)`

**两项保护**【码】
- L_lvl：人声被压低。可微复刻 V-SPLVL，罚 relu(−d − 2 dB)（`losses.py:111-122, 262-266`）
- L_on：字头。可微复刻 V-ONSET，罚 relu(超出量)（`:152-176, 267-271`）
- 权重默认 `auto`：开跑前在 6400 段开发数据上标定，让每项约占 |HybridLoss| 的 10%（`share=0.1`）；某项初始为 0 时按「超出 1 dB 时等于 10%」标定（`losses.py:285-297`、`train.py:105-135`）
- **实际标定出的权重数值：仓内未找到**（只在仓外 `run.json` 的 `calib` 里）

**默认关的项**【码】
- L2-SP，`w_sp` 1e-4
- 老师蒸馏，`w_kd` 0.1，只在回放样本上算
- 电平归一，`level_norm`
- `on_skip_leading`
- 这四项在正式与消融配置里都没开（`T/configs/default.yaml:37-43`；`formal.yaml` 不覆盖）

**评估与挑检查点用哪个读数**【码】
- 用开发份上的 `hybrid`，不是含保护项的 `total`（`train.py:376`、`guards.py:163`）

## 5. 优化（`T/configs/default.yaml` + `T/train.py`）

| 项 | 值 | 出处 |
|---|---|---|
| 优化器 | Adam，betas (0.9, 0.999) | `train.py:264-265`；yaml:29-32 |
| 学习率 | 1e-4 | yaml:29 |
| 学习率计划 | 前 500 步线性升温，之后**恒定**，无衰减 | `train.py:177-178,266` |
| 权重衰减 | 无（Adam 未传 weight_decay；未找到 AdamW） | `train.py:264` |
| 梯度裁剪 | 全局 L2 范数 3.0 | `train.py:418` |
| 批量 | 32 段 × 4 s（16 kHz） | yaml:20-21 |
| 步数 | 5,000（消融 1,000） | yaml:48；`ablation.yaml:17` |
| 混合精度 / bf16 / 自动混合精度 | 未找到，全程 fp32 | grep `autocast\|bf16\|GradScaler` 无结果 |
| EMA | 有实现，默认关（decay 0.999） | yaml:44-46；`train.py:267,365-371` |
| `torch.compile` / `pin_memory` | 未找到 | — |
| 取数 worker | 正式 6 个 / 运行（3 个运行同卡共 18 个） | `formal.yaml:16` |

- 换算【推】：5,000 × 32 × 4 s = 640,000 s ≈ 178 小时混音；训练人声 22,952 句，约 80 小时（按 train-clean-100 的 80% 估）。约 2.2 轮人声
- 裁剪前梯度范数在已记录步上最大 A 66–105、B 66–161，比阈值 3.0 大 1–2 个数量级【报，`T/README.md:134`】。裁剪是不是几乎每步都生效：未量化【推：很可能】
- 过拟合诊断里扫过学习率，只用于诊断、不是正式扫参：300 步时 3e-4 / 1e-3 / 3e-3 分别走到 20.2% / 22.2% / 23.2%【报】

## 6. 评测与停止

**训练中的验证**【码】
- 每 500 步一次（`eval_every`）。训完时如果最后一步还没评过，补评一次（`train.py:438-447`）
- 训练中的读数只有开发份损失分项（hybrid / lvl / on / total）
- 训练中没有 PESQ、STOI、DNSMOS 或语音识别读数（未找到）
- `losses.si_snr_db` 只用在冒烟报告里（`losses.py:179-184`）

**挑检查点**【码】
- 开发份 hybrid 最低的一次另存 `best.pt`，不参与轮换（`train.py:379-382`）
- 评测执行单定用 `best.pt`，不评 `step-0005000.pt`（`E/cloud-run.md` 第一节）

**早停**【码】
- 没有按开发份早停
- 连续 3 次比最好值高 1e-3 以上（`PATIENCE`、`BEST_TOL`）只记警告、不停（`guards.py:7,23-24`）

**自动停止条件**（`guards.py:20-33`）【码】
- 损失猛涨：最近 50 步中位数 − 前 20 步中位数 > 2·|b|
- 太慢：第 50–500 步每步中位数 > 1.2 × `expect_s_step`
- 卡住：900 s 没有进展
- 读数不是有限数
- 上限：时长 / 花费 / 信号 / 取数超时
- 退出码：跑满或到上限为 0，其他为 1（`train.py:490-492`）

**正式效果评测的设计**（`E/../eval-plan.md`、014 第三节）【码】
- 评分项：可懂度 V-ESTOI、两个听感评分模型（SCOREQ、Distill-MOS）、吃字（语音识别）、字头 V-ONSET、背景跳变 V-BGSTAB、音色 V-TIMBRE 等
- 统计：配对比较，按源片段成组 bootstrap 95% 区间，10,000 次重抽（`SC-AGG`）
- 阈值正本在 `src/eval/src/thresholds/accept.rs`
- 现状：**只写了执行单，没跑**（`E/cloud-run.md` 抬头「只写，没跑；开机等大总管批」）

## 7. 超参与消融

**数据消融**（`T/ablation.py`、`T/configs/ablation-runs.yaml`）【码 + 报】
- 7 个运行：2 个基线（种子 0、1）+ 4 个关旋钮（eq · clip · n_noises · reverb）+ 1 个开 high_snr
- 每个运行：模型 A，1000 步，各 1 个种子
- 判法 RULE 跑之前写定（`ablation.py:45-60`）：
  - 噪声底 floor = |基线 s0 − 基线 s1|
  - d = 旋钮运行的 hybrid − 两个基线的均值
  - |d| > floor 才判「有用」或「有害」
- 读数（b4 build `bc0e9e7b`，不是正式 build）：floor 0.00779；reverb +0.0910（11.7 倍）· eq +0.0216（2.8 倍）· n_noises +0.0051 · clip −0.0049 · high_snr +0.0010（`T/README.md:111-120`）
- 结论：配方不改
- README 自述的限制：噪声底只来自 2 个种子；每个旋钮只跑 1 次；判据只用开发份 hybrid，不用成绩单

**正式训练**【码】
- 6 个运行 = 模型 A、B × 种子 0、1、2
- 没有扫学习率、批量、步数、损失权重

**没扫的开关**【码】
- 电平归一：README 建议在消融里扫开 / 关（`T/README.md:177`），但 7 个消融运行里没有这一组
- `bn: train`、L2-SP、蒸馏、EMA 都只是「扫参选项」，没有跑过的记录（未找到）

**统计比较**
- 训练读数只报 3 个种子的均值和极差，没有检验（`T/README.md:136-154`）
- 成绩单的 bootstrap 见第 6 节，还没跑

## 8. 可复现与记录

**种子与确定性**【码】
- `torch.manual_seed(run.seed)` 只在 `build_model` 里调（`train.py:64`）
- 数据随机数 = `np.random.default_rng([seed, 份代码, i])`（`D/loader.py:231`）
- 开发份种子 12345
- 未找到 `cudnn.deterministic`、`use_deterministic_algorithms`、`np.random.seed`
- 续训一致性：GPU + 真数据上实测，278 个键最大差 0（`20261126-h100-b4-cmp-resume.json`）【报】

**运行记录**【码】
- `run.json` 记：配置快照、git 提交号、是否有未提交改动、分支、torch / CUDA / cuDNN / 显卡、训练与开发份的 data build id、标定读数、参数量、每次续训、停止原因（`T/runrec.py:1-58`）
- 拿不到 git 提交号就拒跑（`train.py:210`，`runrec.py:33-47`）
- 续训时只许改 8 项，其余改了就拒绝（`runrec.py:25-27`）
- 指标写 `metrics.jsonl`：每 20 步一行训练读数，每 500 步一行开发读数
- 另有 `heartbeat.json`
- 未找到 tensorboard / wandb / trackio / mlflow

**仓内能看到的**
- 只有汇总 JSON（`T/smoke_reports/`），每条运行的 `metrics.jsonl` 不在仓里
- 正式读数由脚本 `20261126-formal-readings-script.py.txt` 从运行记录生成
- 正式代码提交 `7b52d395`；6 个 `best.pt` 的 sha256 与检查点清单逐个相同【报】

## 9. 检查点与恢复

**怎么存**【码】
- 每 500 步存一次（`ckpt_every`）。只留最近 3 份（`keep_last`）加第 0 步
- 另存 `best.pt`；训崩时存 `crash-<步>.pt`
- 写法是「临时文件 → fsync → 改名」；最新一份读坏时退回上一份（`runrec.py:64-78`）
- 检查点内容：模型、优化器、学习率计划、EMA、步数、计费秒数、标定、配置、自动保护状态（`train.py:329-333`）

**续训测过没有**【报】
- CPU + 合成数据：`test_resume_equals_uninterrupted` 逐值相同
- H100 + 真数据：两次（11-25、11-26），278 个键最大差 0
- 被抢占（SIGTERM）、取数超时、检查点写坏都有单元测试（`test_review_fixes.py`）

**存在哪**
- 云上网络盘 `/workspace/runs/exp025-formal`
- 取回本机 `_tmp/20261126-trainer-formal/`（不含 `step-*.pt`）
- 现状页写「评测完以前不删」（`state/handoff/trainer.md`）

## 10. 成本与算力

**机器**【码】
- H100 80GB HBM3，单价 4.39 美元 / 小时，US-MO-1（`C/config.py:11-15`）
- 数据处理与评测用 4 核 CPU 机，0.12 美元 / 小时
- 网络盘 150 → 200 GB

**开机前估算**【码】
- 有：`plan-h100-formal.md` 第四节估 6.6–7.4 美元（按高 20% 单价为 7.9–8.9 美元）
- 开训前清单要人点头（`plan-h100-formal.md:20`）

**花费上限与止损**【码】
- 训练器内每个运行 20 美元、1.5 小时（`formal.yaml:19`）
- cloud 总上限 `SPEND_CAP_USD` 100 美元（`C/config.py:39`）
- 硬止损：机器到时限自删（实测有效）+ 本机看门狗按「时限 + 5 分钟」删机（`C/README.md:51-58`、`C/watchdog.py`）
- 训练器内另有心跳看门狗（`T/watchdog.py`，`hang_s` 120 s）

**实际花费**【报】
- `C/costs.tsv` 记到 2026-11-26 07:44 UTC：机器估算合计 7.96 美元，其中 H100 5.17 美元
- b4 那一单 H100（4.32 美元，`T/README.md:126`）和正式训练两台 H100 的账，**不在 main 的 costs.tsv 里**
- 训练器给每个运行记的账：A 2.56 美元、B 2.74–2.76 美元。这是按每个运行独占一台机器算的，3 个运行同卡时会重复计

**吞吐**【报】
- 单跑：A 0.142 s / 步，B 0.1665 s / 步
- 3 个同卡：A 0.346 s / 步，B 0.3985 s / 步
- 准备期：A 批 360–364 s，比旧机 152–159 s 慢 2.3 倍；B 批 257 s

**取数瓶颈**
- 没量过 GPU 利用率和等数据时间
- 开了 #233「训练慢在哪：GPU 利用率 / 等数据」，测不测待人定（`T/README.md:167`）

## 11. 导出与部署一致性

**ONNX 导出**（`T/export.py`）【码 + 报】
- 导出的是**整段版**，不是流式版
- 对拍：两种长度（T = 63、250），ONNX 离 float64 真值 ≤ max(1e-4, 4 × PyTorch float32 离真值)（`export.py:60-92`）
- A 的 ONNX 对 PyTorch 差 1.7e-7 / 4.8e-7；训后的 B 也通过

**两条评测路对拍**（`E/parity.py`，`E/runs/parity-20261126/parity.txt`）【报】
- 比的是原版权重：整段离线 ONNX（甲路）对产品内置流式 GTCRN
- 5 段、22 项、76 对，最大差 3.8e-5，容差 1e-3，PASS
- 9 项未验证：没带 `--qa`，吃字与两个听感项为空
- 延迟、最坏一帧两项跳过

**流式 / Rust 一致性**
- 未做。A 的流式导出待办 #231
- B 的注意力要在 Rust 引擎里另写，在 backlog（`T/README.md:200`）
- 甲路只有声明延迟，所以评测总判定最多到「判不了」（`E/cloud-run.md`）

**训练损失与 Rust verifier 对拍**
- `T/parity_rs/` 只读调用 `vdn-eval`，对拍 L_lvl / L_on；`test_losses_parity.py` 31 条【报】

## 12. 结果

**正式训练**（build `df759bb8d606a209`，代码 `7b52d395`，`T/README.md:129-161`、`smoke_reports/20261126-h100-formal-readings.json`）【报】
- 6 个运行都跑满 5000 步，停止原因 `max_steps`，自动保护都没触发
- 开发份 hybrid（越低越好）：

| 时点 | A（3 个种子均值） | B（3 个种子均值） | 种子极差 |
|---|---|---|---|
| 第 0 步（CPU 参照） | 3.8132 | 3.8132 | — |
| 第 5000 步 | 3.6532 | 3.6518 | A 0.0101 · B 0.0125 |

- B − A 在每个评估点都在 ±0.0018 以内，小于种子极差：开发份上分不出 A、B
- 曲线：第 2500 步到第 5000 步只降 0.017–0.018，接近走平
- `best.pt` 所在步：A 为 4000 / 5000 / 4500，B 为 4000 / 5000 / 4000。3 个种子里有 2 个的最好点在最后两次评估（4500、5000 步）

**部件漂移**（`20261126-formal-ab-drift.json`）【报】
- 注意力输出层范数 0.22–0.28
- 拿掉注意力，输出谱变 2.4–3.0%（只测了一段示例录音）
- 主干离起点 3.2–3.8%

**微调有没有赢过原版**
- **还不知道**：微调后模型的成绩单仓里未找到（执行单 `E/cloud-run.md` 未跑）
- 唯一的对比是开发份 hybrid 降了 4.2%（3.813 → 3.653）。这和原版是同一个损失、同一份开发数据，不是评测集上的成绩
- README 明写「降噪效果要等 evalcmp 的成绩单」

## 13. 测试数

- `T/tests/` 里 `def test_` 共 **327 个**，分在 21 个文件里（本次按函数计数，未运行；`parametrize` 展开后会更多）
  - 最多的几个：`test_buildpin` 37 · `test_review_fixes` 37 · `test_formal` 31 · `test_guards` 30 · `test_finite` 23
- `T/README.md:38` 写「119 条通过」，这个数已过时
- 同口径的其他目录：`D/tests` 494 · `C/tests` 147 · `E/test_offline.py` 28 · `B/test_*.py` 44
- 有几个文件带 skip / ignore 标记（需要真分片、显卡等）

## 14. 已知问题与失败报告

**失败报告**（`T/smoke_reports/*FAILED*`）【报】

| 文件 | 失败在哪 | 读数 | 后续 |
|---|---|---|---|
| `20261124-cpu-synthetic-v2-floor-exact-FAILED.json` | 过拟合判据第 2 版：下限取「输出 = 目标」 | 下限被 SI-SNR 的 1e-8 顶成 −10.04；150 步只走到 36.8% | 判据改成 40 dB 参照 |
| `20261125-h100-overfit-FAILED.json` | 40 dB 下限 + SI-SNR ≥ 6 dB | 走到 14.4%，SI-SNR +3.52 dB | H100 停机 |
| `20261125-cpu-gate3-FAILED.json` | 第 3 版：oracle 距离 ≥ 50% + SI-SNR ≥ 6 dB | 走到 19.6%，SI-SNR +2.52 dB | 诊断：2000 步 30.7%，曲线走平；换学习率、关旋钮都没用 |

- 之后改成第 4 版并重判为过（`20261125-rejudge-v4.json`）。4 版判据都是 AI / 大总管定的，人未参与
- 诊断报告建议的独立参照（用官方权重在同 4 段上算能走多远），在仓里没找到有没有跑过

**其他已知问题**
- #159：启动期网络盘阻塞
- #160：心跳改名阻塞的跨运行覆盖
- #198 · #210 · #217：现状页列的待办，本次未逐个读
- #231：流式导出
- #233：训练慢在哪（`T/README.md:195`；`state/handoff/trainer.md`）
- 云上准备期慢 2.3 倍：原因未测，推断慢在开进程、读网络盘或标定
- FiLM 不学：喂全零的直接后果。B 的条件输入口子这次没有被验证
- 数据：
  - 飞机舱用地铁代替；meeting-room 是办公室底噪；hotel-room 开发份与训练份同一间房
  - MUSAN 点声源因许可没核而不用
  - 「从官方源可复现」未验证（`D/README.md:182`）
  - `split_rules.json` 里有几条说明已过时（`:207`）
  - 云上 GPU 混音对拍的状态未找到记录
- 消融只有 2 个种子的噪声底；均衡的证据只有 2.8 倍噪声底
- 评测：甲路判不了延迟，总判定最多到「判不了」；B 没有流式路
