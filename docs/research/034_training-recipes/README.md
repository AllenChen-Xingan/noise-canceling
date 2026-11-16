# 034 · 训练配方调研：现成 skill · 同类模型怎么训 · 小型因果降噪的训练方法

> 2026-11-16 · 只读调研子代理（大总管 b19 派）· 没装、没下、没跑任何外部代码
> 来源标注：【出处 Sn】编号见同目录 `sources.md` · 【读源码】读本仓 `resources/` 下的上游副本或 GitHub 原文件 · 【推断】本线推的
> 本文所有「建议」都待人定；没有任何实测训练结果。

## 给人的三句话

1. **有没有现成可装的训模型 skill**：有，但都不是给我们这种「自写 PyTorch 循环 + 4.8 万参数降噪 + RunPod」做的。最接近的是 Hugging Face 官方 `huggingface/skills` 的训练类 skill，绑在 HF Jobs（HF 的云 GPU）和 TRL（大语言模型训练库）上。建议不装，只抄做法。
2. **同类模型怎么训**：GTCRN 官方模板 SEtrain 用「预先混好的数据 + Adam + 预热余弦学习率 + 每轮 PESQ 验证 + 存最好检查点」；DeepFilterNet 用「现场混音 + 多种设备失真 + 多分辨率谱损失 + 批量逐步加大」。DeepFilterNet 那套现场混音最值得借。
3. **027/028 漏掉的**：设备链失真（削波 · 电平 · 均衡 · 编解码）、大模型当老师在真实录音上蒸馏、开训前的冒烟检查（先过拟合一批）、云端检查点必须外存。见第四节。

---

## 一、帮 AI 训模型的 skill / 插件

用途：判断能不能直接装一个来用，或者只借它的做法。

| 名字 | 链接 | 做什么 | 适不适合我们 | 能借鉴的做法 |
|---|---|---|---|---|
| `huggingface-llm-trainer` | 【出处 S1 · S2】 | 用 TRL / Unsloth 在 HF Jobs 上微调语言与视觉模型；脚本内联提交，带 Trackio 监控、推到 Hub | **不适合直接用**：只支持 HF Jobs，不支持 RunPod；默认 TRL 的 SFT/DPO 数据格式，和音频降噪无关【推断】 | ① 「训练环境是临时的，不推到持久存储就全丢」写成硬规则 ② 超时至少 1–2 小时，再加 20–30% 余量 ③ 上 GPU 前先在 CPU 上校验数据格式（约 0.01 美元）④ 任务超过 1 小时或 5 美元时先跑估价脚本 ⑤ 提交后报告任务号、监控链接、预计时长与花费，**不自己轮询**【出处 S2】 |
| `trl-training` · `huggingface-vision-trainer` · `train-sentence-transformers` | 【出处 S1】 | 分别训语言模型、视觉检测分割、句向量模型 | 不适合：任务类型不同【推断】 | 无新增 |
| `huggingface-trackio` | 【出处 S1】 | 轻量实验跟踪（接口仿 W&B），面板可放在 HF Space | **可以考虑**：和训练框架无关，只要在循环里 `log`；但面板放 Space 涉及外发，过铁律 12 【推断】 | 每次运行有 `project` + 描述性 `run_name`，只记超参与数据信息【出处 S2】 |
| `huggingface-community-evals` | 【出处 S1】 | 用 inspect-ai / lighteval 评测 Hub 上的语言模型 | 不适合：语言模型评测【推断】 | 无 |
| `ml-research`（`krasserm/ml-plugins`） | 【出处 S3】 | HF `ml-intern` 的 Claude Code 版：先查文献找配方 → 校验数据 → 写脚本 → 提交**一个**任务 → 监控；另有「预算封顶的自动扫参」 | 部分适合：扫参循环的结构可借；但工具只对接 HF Jobs 与 GitHub【出处 S3】 | ① 扫参由 `program.md` 管：目标指标、可碰的东西、**预算包络**（最多几个任务 · 总墙钟 · 显卡型号 · 单任务超时）② 超预算的动作被拦下并报告，不静默放宽 ③ 进度落 `runs/summary.md`，中断可续 ④ 花钱或写远端前先停下确认【出处 S3】 |
| `autoresearch`（Orchestra Research `AI-Research-SKILLs`） | 【出处 S4】 | 两层循环：内层快速做实验，外层汇总、改方向；维护 `research-state.yaml` 等状态文件 | 不适合照搬：它明说「全自动，不问人」，和本项目「人拍板 + 验收靠 verifier」冲突【读源码 S4 SKILL.md】 | 每个假设一个目录：`protocol.md`（做什么、为什么、预测）+ `results/` + `analysis.md`；先写预测再跑【出处 S4】 |
| 同仓 `09-infrastructure` / `13-mlops` | 【出处 S4】 | 云 GPU（Lambda · Modal · SkyPilot）与跟踪（MLflow · SwanLab · TensorBoard · W&B）的使用说明 | 部分适合：SkyPilot 支持 RunPod（未核）；跟踪类可选一个【推断】 | 无新增 |
| Karpathy `autoresearch` | 【出处 S5】 | 三文件：`prepare.py`（固定数据与评测，不许改）· `train.py`（AI 唯一能改的）· `program.md`（人写的方向）；每轮固定训 5 分钟，比完留或丢 | 思路适合：我们的成绩单就相当于固定评测【推断】 | **评测脚本与训练脚本分开，AI 只许改训练脚本**；每轮固定时长预算，结果可比【出处 S5】 |
| Anthropic 官方 `anthropics/skills` | 【出处 S6】 | 19 个 skill：文档类、设计类、`mcp-builder`、`webapp-testing`、`skill-creator` 等 | **没有训练类 skill**【读 GitHub 目录 S6】 | 无 |
| 第三方目录里的 W&B / MLflow / 超参调优 skill | 【出处 S7】 | 讲怎么接 W&B、MLflow，怎么配贝叶斯 / 网格搜索 | 未审代码、维护情况未核；不建议装【推断】 | 无新增 |

**结论**：没有能直接装来训 GTCRN 的 skill【出处 S1–S7】。建议自己写一个项目内 skill（或 `scripts/` 下的脚本），把上面加粗的几条做法写进去：预算包络、检查点外存、评测脚本只读、先估价再提交（建议，待你定）。

---

## 二、同类降噪 / 增强模型的训练写法

用途：看别人的配置、混音、损失、步数、评估、导出怎么做，挑能借的。

### 2.1 GTCRN 官方 + SEtrain 模板

- **配置**：一份 `configs/cfg_train.yaml`，用 OmegaConf 读，支持 `${FFT.n_fft}` 这类引用；训练开始时把完整配置存进实验目录【读源码 S8 `train.py`】
- **数据**：**不现场混音**。先用 `prepare_datasets/gen_DNS3_datasets.py` 离线生成带噪 / 干净对，训练时直接读；每轮从 72,000 条（约 200 小时）里随机抽 10,000 条，每条 10 s【读源码 S8 `cfg_train.yaml` · `dataloader.py`】
- **损失**：`HybridLoss`：30 × 压缩复谱 MSE + 70 × 压缩幅度 MSE（压缩指数 0.3）+ SI-SNR【读源码 S8 · 本仓 `resources/gtcrn/loss.py`】
- **步数与批量**：批量 8 / 卡 × 2 卡；200 轮；学习率线性预热 25,000 步到 1e-3，余弦降到 250,000 步时 1e-6；梯度裁剪 3.0；固定种子 43【读源码 S8】
- **评估**：每轮在验证集上算宽带 PESQ（多进程），按 PESQ 存最好的检查点；TensorBoard 记损失与学习率【读源码 S8 `train.py`】
- **检查点与导出**：每轮存 `model_XXX.tar`（模型 + 优化器 + 调度器 + 轮数），可断点续训；流式与 ONNX 导出在另一个仓库 `TRT-SE`【读源码 S8 · 本仓 `resources/gtcrn/README.md:55`】
- **同作者的新模型 UL-UNAS**（更小、带因果时频注意力）也用这套模板，HF 上 Arm 发布的版本带 `SHA256SUMS`、`manifest.json`、基准文件【出处 S9 · S10】

### 2.2 DeepFilterNet（v1–v3）

- **配置**：一份 `config.ini`，分节 `[train]` `[distortion]` `[df]` `[optim]` 和每种损失一节；随检查点一起发布【读源码 本仓 `resources/DeepFilterNet/models/DeepFilterNet3.zip` 内 `config.ini`】
- **数据**：语音、噪声、房间冲激响应（RIR）分别打成 HDF5；`dataset.cfg` 列每个数据集和**采样倍率**（某类噪声少，就把倍率调大来过采样）【读源码 本仓 `README.md:179-230`】
- **现场混音**（Rust 写的数据加载器，每个样本现场生成）【读源码 `libDF/src/dataset.rs:640-735,1250-1300`】：
  - 每条语音配 **2–5 段噪声**叠加
  - SNR 从 `{-100, -5, 0, 5, 10, 20, 40}` dB 里抽（−100 dB 等于几乎纯噪声，教模型静音段怎么处理）；整体增益从 `{-6, 0, 6}` dB 抽
  - 语音与噪声各自做增强：去直流（概率 0.25）· 随机低阶滤波（0.25）· 随机双二阶均衡（高低架 · 峰值 · 陷波 · 高低通）· 随机重采样（0.1）
  - 失真只加在带噪输入上、目标保持干净：混响（DFN3 概率 0.1）· 削波 · 带宽限制 · 置零 · 空气吸收 · 干扰说话人（DFN3 发布配置里后几项概率为 0）
  - 混响时目标用「混响更少的版本」，不是完全干信号【读源码 `augmentations.rs:893-930`】
- **损失**（DFN3 发布配置）：多分辨率谱损失（FFT 256/512/1024/2048，幅度与复谱各权重 500，压缩 0.3）+ 局部 SNR 估计损失（1e-3）；还备有 SDR、分段 SDR、ASR 损失但权重为 0【读源码 `config.ini` · `df/loss.py`】
- **步数与批量**：最多 120 轮；片段 3 s；**批量逐轮加大**：第 0 轮 16 → 第 2 轮 24 → 第 5 轮 32 → 第 10 轮 64 → 第 20 轮 128；AdamW，学习率 1e-3，预热 3 轮，余弦到 1e-6，权重衰减从 1e-12 增到 0.01；验证损失 25 轮不降就早停【读源码 `config.ini`】
- **评估**：按验证损失选检查点；另有 `evaluation_utils.py` 算 PESQ / STOI / 复合指标【读源码 `df/` 目录】
- **导出**：发布 ONNX 版本与「低延迟」版（`_ll`），Rust 推理库 `libDF` 直接读【读源码 本仓 `models/` 文件名】

### 2.3 SpeechBrain 语音增强 recipe

- **配置**：HyperPyYAML，模型、增强、优化器都在 yaml 里用 `!new:` 直接实例化；发布到 HF 时 `hyperparams.yaml` 随权重一起上传【读源码 S11 · 出处 S12】
- **数据**：DNS recipe 先用微软 `noisyspeech_synthesizer` 离线合成，再打成 WebDataset 分片；训练时另加变速（95/100/105%）、频带丢弃、片段丢弃【读源码 S11 `sepformer-dns-16k.yaml`】
- **损失**：SepFormer 用 SI-SNR；`threshold_byloss: True` + `threshold: -30` 把太好的样本从损失里截掉【读源码 S11】
- **MetricGAN recipe**：判别器学着预测 PESQ，生成器朝判别器给的高分走；750 轮；20% 历史样本回放给判别器（`history_portion: 0.2`）；生成器与判别器学习率都是 5e-4【读源码 S13】
- **步数与批量**：SepFormer DNS 批量 4、片段 4 s、100 轮、学习率 1.5e-4、梯度裁剪 5、fp16；每 60 分钟存一次检查点【读源码 S11】
- **评估**：DNSMOS（SIG · BAK · OVRL）、PESQ、SI-SNR，写进模型卡【出处 S12】

### 2.4 ESPnet-SE（`egs2/*/enh1`）

- **配置**：每个模型一份 `conf/tuning/train_enh_*.yaml`；编码器 / 分离器 / 解码器 / 损失各一节，损失可以列多个、各带权重【读源码 S14】
- **数据**：默认读 Kaldi 风格列表；也能现场加噪声、混响、干扰说话人：`preprocessor: enh`，参数 `noise_db_range`（默认 `3_10` dB）、`rir_apply_prob`、`noise_apply_prob`、`speech_volume_normalize`【读源码 S15 · S16】
- **损失**：DNS 上 DCCRN 用 SI-SNR【读源码 S14】
- **步数与批量**：DCCRN 批量 32、片段 64,000 点（16 kHz 下 4 s）、100 轮、Adam 1e-3；验证损失不降时学习率 × 0.7；耐心 10 轮【读源码 S14】
- **检查点**：按验证 SI-SNR 最大与验证损失最小各选最好的，`keep_nbest_models` 控制留几个（ESPnet 通常把 n 个最好的平均成一个）【读源码 S14；「平均」是常见用法，本次未核这份配置的平均数】
- **相关挑战赛 URGENT 2025**：基线在 ESPnet 上；仿真同时最多叠 5 种失真：加性噪声 · 混响 · 削波 · 带宽限制 · 编解码（MP3/OGG）· 丢包 · 风噪【出处 S17】。教训：Track 2 数据约 60,000 小时，但没有一个系统超过 Track 1（约 2,500 小时）的最好系统；作者推测是因为多出的数据里含带噪语音【出处 S17】

### 2.5 ClearerVoice-Studio（阿里通义）

- **配置**：每个模型一份 yaml（`config/train/FRCRN_SE_16K.yaml` 等），区分 `init_learning_rate`（新训）和 `finetune_learning_rate`（微调）【读源码 本仓 `resources/ClearerVoice-Studio/train/speech_enhancement/config/train/`】
- **微调学习率 = 新训的 1/10 或 1/5**：FRCRN 1e-3 → 1e-4；MossFormerGAN 5e-4 → 1e-4【读源码 同上】。和 027 取的 1/10 一致
- **数据**：离线生成：`train/data_generation/speech_enhancement/` 下 `generate_noisy_speech` 与 `generate_reverb_noisy_speech` 两套脚本；训练读 `.scp` 列表【读源码】
- **损失**：FRCRN 用复数掩码 + SI-SNR；MossFormer2_48K 用相位敏感掩码（PSM）损失；MossFormerGAN 用 CMGAN 式对抗训练（判别器预测指标）【读源码 `losses/loss.py:13-143` · README】
- **步数与批量**：FRCRN 批量 4、片段 1 s、100 轮、梯度裁剪 10、权重衰减 1e-5【读源码】

### 2.6 HF Hub 上的模型卡

- 搜 `gtcrn` 有 13 个仓库：多是 ONNX / NPU 移植（sherpa-onnx、AXERA、CoreAI），**没有一个附训练配方**【出处 S18，2026-11-16 查】
- `Vasubuha/gtcrn-speech-enhancement`：用 GTCRN 结构在军用噪声上重训，给了 `best.pt` 与流式 ONNX，按噪声类别报 SNR 提升 / STOI / PESQ；训练细节未写【出处 S19】
- `speechbrain/sepformer-dns4-16k-enhancement`：卡里有 `model-index` 结构化指标（SI-SNR · PESQ · DNSMOS 三项），仓里带 `hyperparams.yaml`【出处 S12】
- `Arm/ulunas-speech-enhancement-fp32-litert`：带 `SHA256SUMS`、`manifest.json`、目标设备基准 yaml、样例输入输出【出处 S10】——**发布物带哈希 + 清单 + 基准**，和我们铁律 7 / 第五节第 5 条同向

**横向小结**【推断】：

- 配置：全是「一份文件描述一次训练」，并随检查点一起存；没有人用 Hydra 的多层组合
- 混音：GTCRN / ClearerVoice / SpeechBrain-DNS 离线混；DeepFilterNet / ESPnet 现场混。现场混的好处是每轮样本都不同、改 SNR 分布不用重新生成
- 微调学习率普遍取新训的 1/10～1/5

---

## 三、小型因果降噪模型的训练方法

用途：每一条是一种可借的做法，给出处与和我们的关系。

### 3.1 微调防遗忘

- 027 已有 L2-SP、原模型当老师、回放旧数据、EMA【读 `docs/research/027_gtcrn-finetune/decision.md:41,53-54`】，不重查
- ClearerVoice 的微调只降学习率，没有其他防遗忘措施【读源码 2.5】

### 3.2 知识蒸馏（大模型当老师）

- **两步蒸馏**（Bose，Nathoo 等 2023）：老师与学生都是 CRUSE，学生 6.2 万参数（老师的 3.3%，压缩 63 倍），算法延迟 32 ms。**先只用蒸馏损失预训学生，再切到纯监督**，不是两种损失加权混；另加「层内相似度」损失（匹配激活的 Gram 矩阵）。−5 dB 输入下 SDR 比基线高 0.9～1.1 dB，越难的条件涨得越多【出处 S20】。规模和 GTCRN（4.8 万参数）同级
- **E3Net + 蒸馏**（微软，Thakker 等 2022）：用蒸馏把学生做小，学生快 2～4 倍；**学生能在没有干净参考的带噪真实数据上训**，并加 ASR 多任务损失【出处 S21】
- **跨任务蒸馏**（微软，Taherian 等 2023）：用于个性化降噪，缓解「压低目标说话人」和「漏旁人」的取舍，不加推理代价【出处 S22】
- 和我们的关系【推断】：027 的老师是**原版 GTCRN 自己**（只防遗忘）。若用更大的模型（例如本仓已有的 DeepFilterNet3 · DPDFNet）当老师，可以在**没有干净参考的真实录音**上产生训练目标，补「弱场景训练数据要先下载」（D87）的缺口。注意：老师自己在弱场景也会吞字头，老师目标要过 027 的字头 / 背景检查再用

### 3.3 数据仿真

- **SNR 要宽、要连续**：把 SNR 从少数几档换成更宽的连续分布，有正则作用；**电平增强**（整段音量随机）和**频谱整形**（随机均衡滤波，语音与噪声都加）都有帮助【出处 S23，Braun & Tashev 2020】
- **电平增强会伤依赖电平的损失**：要对损失做归一化（按目标能量归一）才不退化【出处 S23】。GTCRN 的 `HybridLoss` 里压缩幅度 MSE 依赖电平【推断】
- **多段噪声叠加、近静音段**：DeepFilterNet 每样本 2–5 段噪声、SNR 抽到 −100 dB【读源码 2.2】
- **设备链**：URGENT 2025 把削波、带宽限制、MP3/OGG 编解码、丢包、风噪当一等失真【出处 S17】；DeepFilterNet 有削波（语音 0.05～0.9、噪声 0.01～0.5 的削波比）、重采样、双二阶均衡【读源码 2.2】。我们是 Mac 麦克风 + 会议软件链路，**编解码与丢包发生在我们之后**，未必要仿；麦克风均衡、自动增益、削波发生在我们之前，要仿【推断】
- **混响目标**：GTCRN 官方取早期混响（前 100 ms）当目标（027 已用）；DeepFilterNet 取「混响更少的版本」【读源码】
- **数据多不一定好**：URGENT 2025 的 60,000 小时没赢 2,500 小时【出处 S17】

### 3.4 损失

- **多分辨率 STFT 损失**：DeepFilterNet3 主损失（4 种 FFT 长度）【读源码】；实时 Demucs 用 L1 波形 + 多分辨率 STFT【出处 S24】；一篇 23 万参数模型的消融里去掉它 PESQ 掉约 0.10【出处 S25】。GTCRN 的 HybridLoss 只用一种分辨率（512 点）【读源码】
- **SI-SNR**：SEtrain / ESPnet / SpeechBrain 都用；SEtrain 的版本乘了「输出与目标标准差之比」防止音量漂移【读源码 S8 `loss_factory.py`】
- **非对称损失**（压过头罚得更重）：个性化降噪里显著减少「误删目标说话人」，代价是旁人漏得更多【出处 S26】。027 的字头单边惩罚同一思路
- **感知类 / 指标类**：MetricGAN / MetricGAN+ 用判别器学 PESQ 当损失【出处 S13 · S27】；DeepFilterNet 备有 ASR 损失【读源码】；E3Net 用 ASR 多任务【出处 S21】。027 已定评分模型不进损失（D81），本节不改这一点
- **STOI 可微损失**：SEtrain 的损失工厂里有 `NegSTOILoss`（`torch_stoi`）【读源码 S8】

### 3.5 强化学习 / 偏好优化

- 029 已调研，结论是先监督微调【读 029】，不重查
- 新出处：DPO 用于**生成式**语音增强，用 UTMOS（MOS 预测器）当偏好来源，DNS 2020 测试集上相对提升最高 56%（ICASSP 2026）【出处 S28】；多指标偏好对（8 万对）比单指标更不容易「骗分」【出处 S29】
- 和我们的关系【推断】：两篇都是生成式模型，不是 GTCRN 这种掩码回归；「多指标同时都好才算好的偏好对」和 D81「两个评分模型都说好才算好」同向，可留作 029 的补充

### 3.6 通用配方

- **Karpathy《A Recipe for Training Neural Networks》**【出处 S30】：先看数据 → 搭端到端骨架和最笨的基线 → **初始化时核对损失值** → **先过拟合一个批次** → 再加正则 → 再调参 → 最后榨性能（平均、多训）。核心：一次只改一处，每步都可验证
- **Google《Deep Learning Tuning Playbook》**【出处 S31】：
  - 批量取硬件吞吐还能涨的最大值，不当成调效果的旋钮；改批量要重调学习率
  - 一次只加一个改动；收益超出「运气好」才采纳
  - 每次实验把超参分三类：要研究的 · 要调到公平的 · 固定的
  - 探索期用准随机搜索，收尾再用贝叶斯优化
  - **不要在一次研究里调训练步数**；按固定步数训，存 N 个最好的检查点，事后挑
  - 按步数（不是按时间）定期评估；最好点落在搜索边界就扩边界
  - 每次研究记：名字 · 配置链接 · 一句描述 · 试了几组 · 最好检查点的验证成绩 · 复现命令

### 3.7 辅助小模型（个性化提取、场景判别）

- **pDeepFilterNet2**：在 DeepFilterNet2 上加 ECAPA-TDNN 声纹向量，比较注入位置，**统一编码器注入最好**；有旁人说话时提升最明显【出处 S32】
- 同一线有 15 万参数的小声纹编码器，用对比式蒸馏训练，并在线细化声纹【出处 S32 检索结果，年份未核】
- ClearerVoice 有 `target_speaker_extraction_online` 训练目录（以唇形 / 手势条件为主）【读源码 本仓 `resources/ClearerVoice-Studio/train/`】
- 场景 / 噪声类型判别的小模型训练配方：本次**未找到**专门出处

---

## 四、对照 027/028：我们漏了什么、该补什么（建议，待你定）

用途：列出 027/028 没写、但同类项目普遍在做的事，按「代价小、风险大」排序。依据是读 027/028 正文关键词，没逐句核。

1. **开训前的冒烟三件事（漏了）**：① 初始化时核对损失值合理 ② 在 1 个批次上过拟合到接近 0 ③ 原检查点在开发份上跑一遍，读数应等于已知成绩【出处 S30】。在 4090 上约几分钟【推断】。027 第五节没写
2. **云端检查点外存与预算包络（漏了）**：RunPod 实例是临时的；每 N 步把检查点 + 配置 + 哈希同步到持久盘或网盘，按本仓铁律 8 只在仓里留路径与哈希；每个任务写死最多几小时、几美元，超了自动停【出处 S2 · S3；套到 RunPod 是推断】。028 写了「云机器怎么开待定」，可以并入这条
3. **设备链仿真（漏了）**：027 混音只有 SNR + 混响。建议补三项发生在模型之前的：**电平随机**（±6 dB 起）· **随机均衡**（麦克风频响）· **轻度削波**；同时按 Braun 2020 做**损失电平归一**，否则电平增强会让压缩幅度 MSE 退化【出处 S23 · 读源码 DeepFilterNet】。编解码 / 丢包发生在我们之后，先不加【推断】
4. **多段噪声 + 近静音样本（漏了）**：每样本叠 2–3 段噪声，留一小份「几乎只有噪声」的样本，专练停顿时的背景稳定（对应 V-BGSTAB）【读源码 DeepFilterNet；和 V-BGSTAB 的关系是推断】
5. **大模型老师 + 真实录音（027 只有自蒸馏）**：用 DeepFilterNet3 / DPDFNet 在本机真实录音上出目标，可绕开 D87 的下载限制；可试「先只蒸馏、再纯监督」两步法【出处 S20 · S21】。风险：老师的吞字头会传给学生，老师输出要先过字头 / 背景检查【推断】
6. **多分辨率 STFT 当可选第 5 项损失**：只作扫参里的一组，不替换 HybridLoss【出处 S24 · S25；在 GTCRN 上是否有效未知】
7. **检查点选择按固定步数 + 留 N 个最好、事后挑**：027 用 EMA（保留）；补「不调步数」「按步数评估」两条【出处 S31】。ESPnet 式「n 个最好平均」可当 EMA 之外的对照【读源码 S14】
8. **实验登记格式**：每个实验先写预测（`protocol.md` 式），再跑；扫参记「要研究的 / 要调公平的 / 固定的」三类超参【出处 S4 · S31】。可并进现有 `docs/experiments/expNNN/` 结构，不新开目录
9. **评测脚本对训练方只读**：AI 改训练脚本时不许碰成绩单与 verifier【出处 S5】。和第五节第 3 条「验收靠 verifier」一致
10. **028 注意力头**：DeepFilterNet 的「批量逐步加大」与 GPU 上 GRU 串行瓶颈有关时，可在 4090 上先测单步吞吐再定批量【出处 S31「批量取吞吐最大」；对 GRU 的判断是推断】

**不建议做的**：装第三方 W&B / 超参调优 skill（未审代码）；照搬 `autoresearch` 的「全自动不问人」；把 MetricGAN / DPO 类评分模型目标放进第一轮（违反 D81，029 已定先监督微调）。

---

收尾：一个子代理完成；读了 027/028 正文与本仓 `resources/` 下 GTCRN · DeepFilterNet · ClearerVoice 副本；GitHub 与 HF 只读 API 取原文件；未做自审第二轮，未过 codex。
