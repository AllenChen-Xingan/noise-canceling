# 027 · 来源

> 编号供 `decision.md` 引用。M… = 方法（细节见 `notes/methods.md`）· D… = 数据（细节与 URL 见 `notes/data-licenses.md`，编号同那份文件的 URL 序号）· E… = 本仓实测与源码。
> 「核到哪级」：**复核** = 主会话亲手读了原文 / 源码 · 子代理 = 会话内子代理读到、主会话未复读 · 摘要 = 只读到摘要 · 未核。

## 一、方法（M）

| 编号 | 来源 | 用在 | 核到哪级 |
|---|---|---|---|
| M1 | Rong et al., GTCRN, ICASSP 2024, pp. 971–975, DOI 10.1109/ICASSP48485.2024.10448310 | 起点模型 | 正文未核（无 arXiv 版） |
| M2 | SEtrain（github.com/Xiaobin-Rong/SEtrain，MIT，提交 3ec437d）：`configs/cfg_train.yaml:10,14-17,30-33,36-38,43,61,63`；`train.py:70`（Adam）；`prepare_datasets/gen_DNS3_datasets.py:26,32,64,100-107` | 超参 · 混音写法 | **复核**（经 GitHub API 读原文件） |
| M3 | GTCRN 海报（SigPort fid=9363）：DNS3 约 2000 h + DiDiSpeech，SNR −5～15 dB，每轮 40,000 对 8 s | 背景 | 子代理 |
| M4 | SERIL，arXiv 2005.11760：式 5、式 9、表 1（SDR 11.75 / 6.99 / 9.31）；摘要「遗忘减少 52%」 | 防遗忘 | 摘要**复核**；式与表为子代理 |
| M5 | SAFE，Archives of Acoustics 51(2), 2026，§3.3、表 3：EMA 0.995 · L2-SP λ = 1e-4 · 老师一致性 λ = 0.1 · 回放 2 : 1 | 防遗忘的起点值 | 子代理（期刊较小） |
| M6 | arXiv 2603.07471（ICASSP 2026）§4.3、表 1–2：89k 参数骨干冻结，秩 1 适配器，SI-SDR +1.51 dB | 备选 · 小模型例子 | 子代理 |
| M7 | arXiv 2607.11630：全量微调伤降噪，LoRA 保住 | 背景 | 摘要（子代理） |
| M8 | VoiceFilter-Lite，arXiv 2009.04323，式 2–3（压过头的残差乘 α，原文 α = 10）、表 1 | 单边罚的思路 | 子代理；本线 013 已核过公式（`docs/research/013_rl-environment/notes/D-finetune-skeptic.md`） |
| M9 | arXiv 2001.10601，式 12、图 3、表 2：αL_speech + (1−α)L_noise | 权衡的旁证 | 子代理 |
| M10 | arXiv 2302.11768 §4.1：语音帧权重 μ | 权衡的旁证 | 子代理 |
| M11 | arXiv 2606.21635 表 1：字头 / 瞬态加权 | 旁证（多通道，离我们远） | 子代理 |
| M12 | arXiv 2009.12286 §3.4、表 1：压缩指数 0.3 | 保持官方 0.3 | 子代理 |
| M13 | arXiv 2306.12867 表 1、§4.1：风噪模拟（阵风 · 侧链压缩 · 削波），SNR U(−6, 14) dB；代码 `sp-uhh/storm`（MIT） | 风噪数据 | 子代理 |
| M14 | arXiv 2507.01821 §4.2：WindNetLite，SNR −20～20 dB | 风噪 SNR 下限 | 子代理 |
| M15 | `audiolabs/SC-Wind-Noise-Generator`（Python，MIT） | 风噪模拟器 | 子代理 |
| M16 | 调研 013 `notes/D-finetune.md` · `notes/D-finetune-skeptic.md`：监督锚点 + 不对称损失；FastEnhancer 约 12.8 万步；PyTorch #94691（GRU 静默错误） | 上游结论 | 本仓已核 |
| M17 | 调研 029 `docs/research/029_gtcrn-rl/decision.md`（rl 线，worktree `../noise_canceling-rl`，未合并）第 13、44、53、73 行：只奖励多压会多吞字头（54%）；GTCRN 人声压低 > 3 dB 12%；评分模型也偏爱多压 | 损失项 · 验收 | **复核**（读原文行） |

## 二、数据（D，编号 = `notes/data-licenses.md` 的 URL 序号）

| 编号 | 数据集 | 许可 | 核到哪级 |
|---|---|---|---|
| D1 | CochlScene（Zenodo 7080122；类别与条数：arXiv 2211.02289 表 1） | CC BY-SA 3.0 | **复核**（许可、大小、条数、13 类、可能含人声） |
| D2 | QUT-NOISE | CC-BY-SA（数据）· BSD（代码） | **复核**（许可、五个包大小） |
| D3 | SONYC-UST v2（Zenodo 3966543） | CC BY 4.0 | **复核**（许可、13.3 GB、18,515 条、类别含人声） |
| D4 | TUM 风噪（Zenodo 6687982） | CC BY 4.0 | **复核**（许可、137 MB） |
| D8 | DNS Challenge（github.com/microsoft/DNS-Challenge）：noise 含 DEMAND 整段 | 按来源 | 子代理 |
| D17 | OpenSLR 28 RIRS_NOISES | Apache 2.0（内部 RWCP · REVERB · AIR 未逐个核） | **复核**（许可、1.3 GB、组成） |
| D26 | LibriSpeech（OpenSLR 12） | CC BY 4.0 | **复核**（许可、6.3 / 23 GB） |
| 其余 | 见 `notes/data-licenses.md` | — | 子代理 |

## 三、本仓实测与源码（E）

| 编号 | 内容 | 位置 |
|---|---|---|
| E1 | 单步训练耗时（CPU · MPS，不带绕法） | `docs/experiments/exp018-gtcrn-finetune/runs/20261105-145149-step/` |
| E2 | 读入 + 混音耗时（干净重测；重叠的那次留作对照，标了 NOTE） | `runs/20261105-150423-mix-clean/` · `runs/20261105-145445-mix/` |
| E3 | MPS 正确性：不带绕法不过 | `runs/20261105-150438-parity/` |
| E4 | 逐层定位（前向） | `runs/20261105-150606-layers/` |
| E5 | MPS 正确性：带绕法过（含训后完整状态与导出口径） | `runs/20261105-152353-parity-fix2/`（之前一次 `runs/20261105-151236-parity-fix/`） |
| E6 | 最小复现与绕法；GRU 单独比 | `smoke/mps_fix.py` · `smoke/test_mps_fix.py`（`MpsTest` · `GruTest`） |
| E7 | GTCRN 弱项 83 组 / 守住项 71 组（cands 线，未合并） | `docs/experiments/exp016-candidates/runs/weak-spots-20261105/weak_spots.md`（在 `../noise_canceling-cands`） |
| E8 | 引擎原声混回 | `src/engine/src/engine.rs` `mix_out`；cands `runs/dfn3-ll-20261105-103000-4cf04ac1/notes.md` 第三节 |
| E9 | V-ONSET · V-SPLVL 定义 | `src/eval/src/verifiers/perceptual.rs:107-172` · `:263-291` |
| E10 | 评测集用了哪些素材、怎么防泄漏 | `docs/experiments/exp013-scene-suite/README.md` 第二、四节 |
