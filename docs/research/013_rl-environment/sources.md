# 两类来源 · 013 RL 环境与分场景训练

> 2026-10-31。每条后面的「算数吗」= 今天还能不能拿来用。详细出处与行号在 `notes/`。

## 一、自有积累（可照搬，先确认时效）

| 是什么 | 在哪 | 算数吗 | 判据 |
|---|---|---|---|
| exp006 场景模拟器 | scene worktree `docs/experiments/exp006-scene-sim/`（`sim.py` · `recipes.json` · `sweep.py`） | **算，但只是分类器的数据工厂** | 7 细场景、40 个 LibriSpeech 说话人、单路、无内置麦 / AGC 模型；`make_clip(job)` 是纯函数，可在线采样（scene 回复，2026-10-31） |
| exp006 切换评测 | `builds/20261030214530-44df/eval_final.json` | 算 | 细场景提交准确率 0.867，babble ↔ neighbor 互混 181 / 113 次；序列 16 条，正确时间 0.66、错切 0.88 次/分钟 |
| verifier 清单与定标级 | evalstd worktree `docs/research/012_eval-standards/verifiers.md` | **算，是奖励的唯一来源（D44）** | 今天感知类全部 L0；物理量 L3；V-EAT-ASR 一票否决 |
| rubric 与校准 | `specs/rubric/denoise.md` · `docs/research/007_denoise-rubric/` | 算 | 硬规则（DNSMOS 只报警、增益为正记 0 等） |
| 设备端自适应 | `docs/research/010_ondevice-adaptation/decision.md` | 算 | 用户端只做零开销选择 + 空闲小更新；不在通话中探索；无参考 MOS 不当奖励 |
| 多人说话 | `docs/research/011_multitalker/decision.md` | 部分算 | 「内置麦 + 一大片人只能不吃字」与耳麦提示已被 D42 否决；babble 人数曲线实验（其实验 2）仍有效 |
| exp003 人耳答卷 | `docs/experiments/exp003-offline-ab/results/` | 算 | 「ASR 漏报吃字」的证据；可做第一次 verifier 定标 |
| 本机数据 | `~/vdn-data`（DEMAND 2.2G · LibriSpeech test-clean 687M · 模型 175M · zh 30M） | 只够评测 | 没有可训练规模的干净语音；硬盘剩约 176 GiB |

## 二、他人积累（只读借鉴，先查协议）

| 是什么 | 在哪 | 协议 | 角色 | 读多深 |
|---|---|---|---|---|
| GTCRN | `resources/gtcrn`（`loss.py:16-30` · `gtcrn.py` · `stream/`） | MIT | 起点 / 学生 | 损失与流式状态逐行；训练脚本在外部 SEtrain（MIT，未克隆） |
| LiSenNet | `resources/LiSenNet`（`model/model.py:130-166`） | MIT | 要学的：MetricGAN 式训练循环 | 逐行 |
| FastEnhancer | `resources/fastenhancer`（`train.py` · `losses.py:150-162` · `configs/`） | 代码 MIT，权重许可待核（D33） | 要学的：在线混合训练配置；非因果 Huge 可当老师结构 | 逐行 |
| UL-UNAS | `resources/ul-unas` | MIT | 学生候选 | 只读 README |
| VoxLock | `resources/voxlock` | Apache-2.0（上游训练数据许可未核） | 教师候选（TSE，需注册） | README 逐行 |
| WeSpeaker / 3D-Speaker | `resources/wespeaker` · `resources/3D-Speaker` | Apache-2.0（权重训练数据 VoxCeleb，待 005 #2） | 零件：声纹条件 | 一两行 |
| DNS-Challenge | `resources/DNS-Challenge/README.md:264-292` | 各子集不同 | 数据许可清单 | 逐条 |
| URGENT 2025 / 2026 | `resources/urgent2025_challenge` · `urgent2026_challenge_track1` | 各子集不同 | 数据许可清单；「真实盲测才算」 | 逐条 |
| 论文：Goodhart / RLHF | arXiv 2210.10760（Gao 2022）· DeepMind specification gaming 博客 2020-04-21 | — | 要学的：停训规则 | 节号见 `notes/B-reward*.md` |
| 论文：SE 讨好案例 | arXiv 2402.01413（CHiME-7 UDASE 听测）· 2406.03460（PESQetarian）· 2606.21458 · 2607.10191 · 2601.16483 | — | 反面教材 | 表号见 `notes/B-reward*.md` · `notes/D-finetune*.md` |
| 论文：不可微指标优化 | 1905.04874（MetricGAN）· 2104.03538（MetricGAN+）· 2002.05879 · 2507.09929（GSEPF） | — | 要比的 | 表号见 `notes/D-finetune*.md` |
| 论文：sim-to-real | 1912.06321（Kadian，Sim2Real 排名相关）· 1703.06907（Tobin）· 1910.07113（ADR）· 2005.13981（DNS 2020）· 2506.01611（URGENT 2024） | — | 要学的 | 见 `notes/A-env*.md` |
| 论文：自训练 / 蒸馏 | 2202.08862（RemixIT）· 2312.08979 · 2309.08144 · 2204.00771（E3Net） | — | 要比的 | 见 `notes/E-distill*.md` |
| 论文：助听器偏好学习 | Nielsen 2015 TASLP 23(1):162 · Widex SoundSense Learn（厂商自测）· Ni 2022 Sensors（PMC9414822） | — | 要学的 | 见 `notes/C-policy*.md` |
| 论文：Personal VAD | 见 `notes/F-pvad.md` | — | 第一个最小实验的蓝本（D47） | 见笔记 |
| 工具 | Optuna（MIT）· BoTorch（MIT）· cma（BSD-3）· Nevergrad（MIT）· Ray（Apache-2.0）· SB3（MIT） | 均可开发方离线用 | 零件 | 一两行 |
| 云 GPU 价格 | RunPod 官方页（2026-10-30 抓取）：L4 $0.44–0.59/h · A100 80G $1.19–1.79/h · H100 $2.69–3.99/h | — | 成本量级 | AWS / Lambda 价格未从官方页核到 |

## 三、数据许可速查（D26：能训练并发布权重？）

| 可以 | 风险（要人拍板） | 不行（只能评测） | 未核实 |
|---|---|---|---|
| LibriSpeech / LibriTTS（CC BY 4.0）· MUSAN（CC BY 4.0）· AISHELL-1（Apache-2.0）· MLS（CC BY 4.0）· CommonVoice（CC0）· DNS 的 LibriVox / Freesound-CC0 · AMI（CC BY 4.0，官方页） | DEMAND（CC BY-SA 3.0，SA 对权重是否生效）· DNS 里 AudioSet / VoxCeleb2（音频版权属原作者）· VoxConverse · VCTK（ODC-By / CC BY 两说） | WHAM!（CC BY-NC）· EARS（NC）· FSD50K-NC 部分 · WSJ（LDC）· NISQA 权重（CC BY-NC-SA） | CHiME-5/6（两说冲突）· DiPCo · NOTSOFAR · Lombard 语料 · VoxLock 上游训练数据 |
