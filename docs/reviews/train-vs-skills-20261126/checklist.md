# 一次好的模型训练应该满足什么：从公开技能与指南整理的检查清单

用途：拿这份清单逐条给我们的语音增强训练代码打分。我们的训练是：在 RunPod 上用单张 H100，以 PyTorch 微调 GTCRN。
整理日期：2026-11-26。每条都来自实际抓取的页面（来源代号见第 0 节）。标 **【SE】** 的条目是语音增强专用的。
「出处」列的是原文说了什么。「对我们」是我的推断，用来把原文翻成我们这套环境里的做法，**不是原文**。

---

## 0. 来源代号与 URL

| 代号 | 来源 | URL |
|---|---|---|
| HF-LLM | HF 技能 `huggingface-llm-trainer` 的 SKILL.md 与 references（⚠️ 今天 2026-11-26 在提交 `aafb3ad` 被标为过时并删除；下面链接指向删除前的最后一版 `58cdf17`） | https://github.com/huggingface/skills/blob/58cdf1745d1d51748e38d1520cdc25898579c22f/skills/huggingface-llm-trainer/SKILL.md ；同目录 `references/troubleshooting.md`、`references/training_patterns.md` |
| HF-VIS | HF 技能 `huggingface-vision-trainer`（现存） | https://github.com/huggingface/skills/blob/main/skills/huggingface-vision-trainer/SKILL.md |
| HF-REL | 同上技能的 `references/reliability_principles.md` | https://github.com/huggingface/skills/blob/main/skills/huggingface-vision-trainer/references/reliability_principles.md |
| HF-HUB | 同上技能的 `references/hub_saving.md` 与 `scripts/estimate_cost.py` | https://github.com/huggingface/skills/blob/main/skills/huggingface-vision-trainer/references/hub_saving.md ；https://github.com/huggingface/skills/blob/main/skills/huggingface-vision-trainer/scripts/estimate_cost.py |
| HF-TIO | HF 技能 `huggingface-trackio`（SKILL.md 与 `references/alerts.md`） | https://github.com/huggingface/skills/blob/main/skills/huggingface-trackio/SKILL.md ；https://github.com/huggingface/skills/blob/main/skills/huggingface-trackio/references/alerts.md |
| HF-ST | HF 技能 `train-sentence-transformers`（SKILL.md、`references/training_args.md`、`references/hf_jobs_execution.md`、`references/troubleshooting.md`） | https://github.com/huggingface/skills/blob/main/skills/train-sentence-transformers/SKILL.md ；https://github.com/huggingface/skills/blob/main/skills/train-sentence-transformers/references/training_args.md |
| HF-TRL | HF 技能 `trl-training`（很薄，只有 API 速查，几乎没有检查项） | https://github.com/huggingface/skills/blob/main/skills/trl-training/SKILL.md |
| ORC | 社区技能库 Orchestra-Research/AI-Research-SKILLs 的 `ml-training-recipes` | https://github.com/Orchestra-Research/AI-Research-SKILLs/blob/main/10-optimization/ml-training-recipes/SKILL.md |
| GTP | Google《Deep Learning Tuning Playbook》 | https://github.com/google-research/tuning_playbook （下面引用写小节名） |
| KAR | Karpathy《A Recipe for Training Neural Networks》(2019) | https://karpathy.github.io/2019/04/25/recipe/ |
| GT | GTCRN 官方仓库（README、`loss.py`、`infer.py`） | https://github.com/Xiaobin-Rong/gtcrn |
| GT#n | GTCRN 仓库 issue 里作者本人的回答（#11 NaN、#57 数据配置、#67 复现、#72 VCTK 复现、#84 微调） | https://github.com/Xiaobin-Rong/gtcrn/issues/<n> |
| SET | GTCRN 作者的训练模板 SEtrain（`config.toml`、`trainer.py`、`train.py`） | https://github.com/Xiaobin-Rong/SEtrain |
| DNS | Microsoft DNS Challenge 数据合成配置与 DNSMOS | https://github.com/microsoft/DNS-Challenge/blob/master/noisyspeech_synthesizer.cfg ；https://github.com/microsoft/DNS-Challenge/blob/master/DNSMOS/README.md |
| AUR | auraloss 的多分辨率 STFT 损失（引 Yamamoto 2019） | https://github.com/csteinmetz1/auraloss/blob/main/auraloss/freq.py |

查过但没有训练类技能的：`anthropics/skills`（19 个技能，没有训练类）。

---

## 1. 数据

1. **开训前先验证数据，不要假设它存在、格式对。** 先跑一个便宜的检查脚本，把「能用 / 需映射 / 不兼容」打印出来，再花 GPU 时间。原文说一半以上的训练失败是数据格式问题，CPU 上检查约 1 分钟、约 0.01 美元。— HF-LLM「Dataset Validation」、HF-VIS「Dataset Validation」、HF-REL 原则 1
   - 对我们：开训前对混合好的 (noisy, clean) 数据跑一次检查，看采样率、声道、长度、配对是否一一对应、有没有 NaN 或全零。
2. **用眼睛看数据，花几个小时。** 写代码按各种维度筛选、排序、画分布，特别是看离群值，离群值几乎总能暴露数据或预处理的 bug。— KAR §1
3. **在 `model(x)` 前一刻可视化输入。** 把真正喂进网络的张量解码回来看（听），这是唯一的真相源。— KAR §2「visualize just before the net」
   - 【SE】对我们：把 dataloader 吐出的 STFT 做 iSTFT 存 wav，人听几条。
4. **训练、验证、测试之间没有泄漏。** 验证指标出现周期性波动，可能是测试数据混进了训练集，或训练数据没打乱。— GTP「Setting up periodic evaluations」、ORC「data leakage」
   - 【SE】对我们：同一说话人、同一段噪声原始文件不要同时出现在训练集和验证集。
5. **【SE】信噪比与响度范围要写明并有依据。** DNS 官方合成：SNR −5 到 20 dB，目标电平 −35 到 −15 dBFS，采样率 16 kHz，每条 30 s。— DNS cfg
   - GTCRN 作者说复现差可能来自 SNR 范围、数据对数量、batch size 等多个因素（作者的配置以截图给出，本文未转录）。— GT#57
6. **【SE】混响：干净目标是否带早期混响要明确。** GTCRN 作者的做法是：干净目标加早期混响（取 RIR 前 100 ms）；他说现在很多文章是一半不加混响、一半加早期混响。— GT#57。DNS 合成脚本按 T60 0.3–1.3 s 选 RIR 加混响。— DNS cfg
7. **【SE】跳过削波和语音活动太低的源文件。** DNS 合成脚本检测到削波就换下一条，并设语音活动阈值 `clean_activity_threshold: 0.6`。配置里还有 SNR、归一化、采样率、削波四个单元测试开关。— DNS cfg 与 `noisyspeech_synthesizer_singleprocess.py`
8. **【SE】小模型优先用预先合成好的固定数据，而不是每轮动态混合。** 作者经验：对小模型，动态混合效果不如固定音频；他用了 2000 h、72 万条 10 s 的音频。— GT#67
   - 论文原文（issue 里引用）：DNS3 上 batch 16，切成 8 s，每轮随机取 4 万对。— GT#72
9. **【SE】微调要防灾难性遗忘：新数据里混入原训练数据。** 作者建议用 DNS3 的语音混新采集的噪声，同时保留一部分原来的噪声。— GT#84
10. **【SE】训练用 DNS3 的 wideband 部分（16 kHz）。** — GT#84
11. **不要拿 VCTK-DEMAND 当唯一基准。** 作者说它小、信噪比高，不适合当基准，建议在 DNS3 这类大集上评估。— GT#72
12. **能加真实数据就加真实数据，这是最可靠的正则化；其次才是数据增强。** — KAR §4

## 2. 基线与 sanity（开跑前的冒烟）

13. **先用极小规模冒烟：`max_steps=1` 加一小段数据，再跑长任务。** — HF-ST SKILL §4「Smoke-test before any long run」；HF-REL 原则 5「先用已知能跑的输入测」
14. **问人要不要先用约 10% 数据快速试一轮**（原文估价约 30–60 分钟、2–5 美元，用来验证配置）。— HF-VIS「Step 3」
15. **训练前先记一次基线分数 `baseline_eval`，训练后输出一行判定：`VERDICT: WIN|MARGINAL|REGRESSION | score | baseline | delta`。** — HF-ST SKILL §4
    - 【SE】对我们：基线 = 未微调的官方 GTCRN 权重（`model_trained_on_dns3.tar`），在同一验证集上的分数。
16. **固定随机种子。** — KAR §2；SET `train.py`（`torch.manual_seed(0)`）。但别指望换 GPU 或换 PyTorch 版本后逐位一致。— HF-ST `training_args.md`「Don't」
17. **初期关掉一切花活（包括数据增强）。** — KAR §2「simplify」；GTP「Choosing the initial configuration」（先用常数学习率，再加复杂调度）
18. **验证初始 loss 落在理论值附近。** — KAR §2「verify loss @ init」
19. **和输入无关的基线：把输入置零训练，效果应明显更差。** — KAR §2「input-independent baseline」
20. **先过拟合一个极小 batch（如 2 条），loss 应能降到接近最小。** 做不到就是有 bug。— KAR §2；ORC「Overfit one batch」
21. **固定一批样本，训练过程中看（听）它们的预测怎么变。** — KAR §2「visualize prediction dynamics」
    - 【SE】SET 每次验证把前 3 条增强结果存成 wav，文件名带轮数和 PESQ。— SET `trainer.py`
22. **小心 batch 维度被混在一起的 bug（view 与 transpose 用错）。** 办法：对第 i 个样本的 loss 反传，检查只有第 i 个输入有梯度。— KAR §2「use backprop to chart dependencies」

## 3. 优化与超参

23. **用成熟、常用的优化器；早期 Adam 3e-4 很稳。** — GTP「Choosing the optimizer」；KAR §3「adam is safe」
    - 【SE】GTCRN 的训练模板：Adam，lr 1e-3，梯度裁剪范数 3.0，`ReduceLROnPlateau`（按验证 loss，factor 0.5，patience 5），120 轮。— SET `config.toml`、`trainer.py`
    - 【SE】作者另说，数据大时用 StepLR 更好：200 轮，第 80/120/150/170/180/190 轮学习率 ×0.5。— GT#11
    - 【SE】微调时作者建议加大 warmup，并每轮结束看一次测试集结果。提问者用的是 lr 5e-5，结果比原模型差。— GT#84
24. **一定要做梯度裁剪；记录未裁剪的梯度范数（随步数曲线 + 直方图），阈值从 90 分位起步。** 超过 50% 的步被裁剪就太狠了，不如直接降学习率。— GTP「Gradient clipping」；GT#11（作者两次建议用 `clip_grad_norm_` 防 NaN）
25. **【SE】SI-SNR 损失的分母要加 eps。** issue 里有人指出 `y_pred == y_true` 时会出 NaN；现版 `loss.py` 分母已加 `+1e-8`。— GT#11、GT `loss.py`
26. **【SE】损失函数用官方的混合损失，或写明为什么换。** GTCRN `HybridLoss` = 30×（压缩复数谱实部 MSE + 虚部 MSE）+ 70×（幅度^0.3 的 MSE）+ SI-SNR。压缩指数 0.7/0.3，STFT 是 512 点、跳 256、sqrt-Hann 窗。— GT `loss.py`
    - 常见替代：多分辨率 STFT 损失（auraloss 默认 FFT 1024/2048/512，跳 120/240/50）。— AUR
27. **学习率是最重要的超参，最先调。** — ORC「Hyperparameter priority」；HF-ST（loss 出 NaN 先降学习率）
28. **不要盲信复用代码里的学习率衰减默认值**，特别是按 epoch 计的调度：数据量一变，含义就变了。每步打印学习率，确认调度真的生效了。— KAR §3；ORC「verify LR schedule is actually applied」
29. **一次只加一样复杂度，每加一样都确认有预期的提升。** — KAR §3「complexify only one at a time」
30. **不稳定时的修法按顺序：warmup → 梯度裁剪 → 换优化器 → 最后才降学习率。** 有的不稳只出现在最初约 500 步，评估太稀会看不到；可用 2 倍学习率跑约 500 步、每步评估来检查。— GTP「How can optimization failures be debugged」
31. **精度：权重和优化器状态用 fp32，前向用 bf16 autocast；不要把权重直接转成 bf16（Adam 的矩估计会退化）。H100 上用 bf16，不用 fp16。** — HF-ST `training_args.md`「Precision」；ORC「Mixed precision」
32. **先用稳的默认配置，性能优化（如 `torch.compile`）先注释掉并写明硬件，首轮跑通再开。** 原文例子：`torch.compile` 在 T4 上报了晦涩的错，被误诊成数据问题，耽误了几个小时。— HF-REL 原则 2
33. **改了 batch size，大多数超参都要重调。** — GTP「Changing the batch size requires re-tuning」
34. **多个超参一起搜时用随机或准随机搜索，不用网格。** — KAR §5；GTP FAQ（quasi-random search）。单次手调跑通之前别上自动调参。— HF-ST `training_args.md`「Hyperparameter search」
35. **调参时区分科学超参、干扰超参和固定超参。** — GTP「Identifying scientific, nuisance, and fixed hyperparameters」

## 4. 评测与停止

36. **按固定步数间隔做周期评估，不按时间间隔。** — GTP「Setting up periodic evaluations」
37. **周期评估用的抽样子集，要和完整离线评测集的结果对得上（没有偏差）。** — GTP「Choosing a sample for periodic evaluation」
38. **评估集最后一个不满的 batch 里，补齐的样本权重设为 0。** — GTP 同上
39. **每次评估存够做离线分析的信息，最好存一部分样本的预测。** — GTP 同上
    - 【SE】对我们：存增强后的 wav（同 SET 做法）。
40. **评估要用整个验证集算，别只看 batch 上的平滑曲线。** — KAR §2「add significant digits to your eval」
41. **除 loss 外看人能理解的指标，并和人的水平比。** — KAR §2「human baseline」
    - 【SE】SET 每轮在验证集上算宽带 PESQ，按 PESQ 选最好的 checkpoint。— SET `trainer.py`
    - 【SE】GTCRN 报告的指标：VCTK 上 SISNR / PESQ / STOI；DNS3 盲测集上 DNSMOS P.808 与 P.835（SIG / BAK / OVRL）。— GT README
    - 【SE】DNSMOS 是无参考指标，用于没有干净参考的真实录音；`-p` 参数算个性化 MOS（惩罚干扰说话人）。— DNS DNSMOS README
42. **【SE】PESQ 在不同训练之间波动很大，主观听感差得没那么多；不要用单次 PESQ 下结论。** — GT#72（作者原话）
43. **采纳一个改动之前，先把最好的那组配置用不同种子跑 N 次，弄清楚同配置重跑的波动有多大。** 光是重跑的波动就可能显著。— GTP「Determining whether to adopt a training pipeline change」
44. **至少看最好几组实验的训练曲线（训练与验证误差随步数变化）：查验证误差回升（有害过拟合）、后期步间方差大、还在涨就停了。** — GTP「Examining the training curves」
45. **固定总步数，事后挑最好的 checkpoint；保留历史最好的 N 个。最好的往往不是最后一个。** 有了这个，提前停止通常不必要。— GTP「Saving checkpoints and retrospectively selecting」
    - 另一派：用 `EarlyStoppingCallback(patience>=3)`，并要求 `load_best_model_at_end` 和 `metric_for_best_model` 已设好。— HF-ST `training_args.md`「Early stopping」；KAR §4「early stopping」
46. **`max_train_steps` 不在一次调参里调；看最优 checkpoint 落在哪：总在前 10% 说明太长，总在最后 25% 说明可以更长。拿不准就训更久。** — GTP「Deciding how long to train」
47. **选最优用的指标名必须和评估器实际写出的键一致，否则会悄悄用最后一个 checkpoint。** — HF-ST `troubleshooting.md`「Metrics don't improve」
48. **开了评估就必须给评估集，否则训练会卡住。** — HF-LLM `training_patterns.md`；HF-ST `training_args.md`
49. **最后才在测试集上看一次。** — GTP「After exploration concludes」

## 5. 可复现与记录

50. **启动时把完整配置打出来；外部调用失败时打出 stdout/stderr；环境变量早校验，缺了就带提示地立刻失败。** — HF-REL 原则 4
51. **脚本自包含：依赖写全并锁版本，不假设环境里已有东西；别为了「简化」删依赖。** — HF-REL 原则 3
52. **每次训练都接实时监控（原文用 Trackio），run 名字要让人认得出；配置里只放超参和模型/数据信息。** — HF-LLM「Monitoring and Tracking」；HF-TIO
53. **在训练代码里埋告警：loss 发散、NaN、loss 停滞、loss 近零，分 INFO/WARN/ERROR 三级。** 原文说这是 AI 代理自主迭代实验的主要机制：代理轮询告警、读指标、停掉重来。— HF-TIO `alerts.md`
54. **快速失败：loss 大于阈值（原文例子 >100）就直接退出，别空烧算力。** — ORC「Training loop」
55. **日志同时写进文件 `logs/{RUN_NAME}.log`；把 HTTP 下载类库的日志压到 WARNING，避免刷屏。** — HF-ST SKILL §4
56. **每次跑完往 `logs/experiments.md` 追加一条。** — HF-ST SKILL §5
57. **实验台账至少记：实验名、配置所在位置、简述、跑了几组、最好 checkpoint 的验证分数、复现命令和未提交的改动。没记录的实验等于没做。** — GTP「Setting up experiment tracking」；ORC 用 TSV 记 commit / 指标 / 显存 / keep 或 discard
58. **保存训练配置本身到实验目录。** — SET `trainer.py`（训练开始时把 config 存进实验路径）
59. **监控面板默认可能是公开的，涉及未公开数据要设私有。** — HF-TIO SKILL（auto-created Spaces 默认公开，`private=True`）

## 6. 检查点与恢复

60. **训练环境是临时的：结果必须推到实例外面，否则「全部训练成果丢失」。** — HF-LLM「Critical: Saving Results to Hub」
    - 对我们：RunPod 的 pod 停掉后容器盘会清空；checkpoint 要写到网络卷或推到远端存储。
61. **每次存 checkpoint 时就推出去（`hub_strategy="every_save"`），不要只在结束时推一次。** 原文说只在结束时推，一旦超时就全丢。— HF-ST `hf_jobs_execution.md`；HF-LLM
62. **按步数存 checkpoint（原文 `save_steps=0.1`，即每 10% 存一次），存盘间隔是评估间隔的整数倍，保证最优那次评估的权重确实在盘上。** — HF-ST `training_args.md`「Evaluation and checkpointing」
63. **恢复训练要恢复优化器、调度器、随机种子、步数计数器；流式数据的读取顺序框架不管，要自己处理。** — HF-ST `training_args.md`「Resuming training」
    - 【SE】对照：SET 的 checkpoint 只存了 epoch、optimizer、model，**没有存调度器状态**（这一点是我读 `trainer.py` 的推断）。— SET `trainer.py`
64. **结束时显式推一次最终模型，并包在 try/except 里。** — HF-ST SKILL §4；HF-LLM `troubleshooting.md`（`trainer.push_to_hub()`）
65. **推出去之后核实对方确实收到了（仓库里能看到 checkpoint 文件夹）。** — HF-LLM `troubleshooting.md`「Checkpoint/Resume Issues」

## 7. 成本与算力

66. **开跑前估算时间和花费；任务超过 1 小时或超过 5 美元时主动给估算。** — HF-LLM「Cost Estimation」
67. **超时上限 = 估计训练时间 × 1.3**（留出加载、缓存、存盘、上传的时间）。原文默认 30 分钟「太短」；超时会被立即杀掉，没存的全丢。— HF-ST `hf_jobs_execution.md`；HF-LLM「Timeout Management」；HF-HUB `estimate_cost.py`（另加约 10% 评估开销和 5 分钟固定开销）
    - 对我们：RunPod 没有内置超时，要自己设上限和自动停机，防止空跑计费。
68. **提交后报告：任务 ID、监控链接、预计时长、预计花费；然后等人来问，不要反复轮询。** — HF-LLM「Asynchronous Job Guidelines」；HF-VIS「Step 5」
69. **硬件按模型规模选，先用小硬件试。OOM 时先减 batch、加梯度累积，再考虑升级硬件。** — HF-LLM「Hardware Selection」「Common Failure Modes」；HF-VIS「Hardware recommendation」
70. **查输入管线是否拖慢训练：数据不在本地、在线预处理太贵（改成离线做一次存盘）、意外的同步点。** — GTP「Optimizing the input pipeline」
    - 【SE】对我们：GTCRN 很小（48.2K 参数），H100 上大概率瓶颈在读盘和 STFT，不在 GPU（推断，未量化）。
71. **测速时排除最初几步（JIT 编译慢）；可以算 MFU 看 GPU 利用率。** — ORC「Mixed precision」「MFU」
72. **用时间预算做实验，可让不同硬件之间的结果可比。** — ORC「Training loop」

## 8. 导出与部署一致性

73. **【SE】训练和推理用同一套 STFT 参数：512 点 FFT、跳 256、窗长 512、sqrt-Hann 窗、16 kHz。** `infer.py` 里断言采样率是 16000。— GT `infer.py`、`loss.py`；SET `config.toml`
74. **推理时的预处理要和训练一致；不一致是「效果差」的常见原因。** — ORC「Model not learning / wrong preprocessing at inference」
75. **模型之外的预处理配置要一起保存发布。** 原文：图像处理器要单独存，标签映射要写进配置。— HF-HUB「Save Image Processor」「Saving Label Mappings」
    - 【SE】对我们的类比：STFT 参数、采样率、压缩指数和权重放在一起存。
76. **【SE】流式模型与离线模型的结果要对齐。** 作者提供流式版本，用 ONNX 部署，并给出流式与离线两份增强结果（`enh_stream.wav` 与 `enh.wav`）供比对。— GT `stream/`
77. **【SE】可流式的结构约束：显式的特征重排层（feature shuffle）会让模型无法流式，作者把它删了。** 微调时不要引入破坏流式的层。— GT README「Note」
78. **记录参数量与计算量（GTCRN：48.2K 参数、33.0 MMACs/s），改结构后重算。** — GT README

## 9. 其他

79. **优先复用官方模板，不要凭描述自己合成训练脚本**；模板里有前人反复漏掉的关键代码。— HF-ST SKILL（「This SKILL.md is a router, not a manual」）
80. **架构「别逞英雄」：先找与问题最接近的论文，照搬其最简单的可用架构。** — KAR §3；GTP「Choosing the model architecture」
81. **能用预训练就用预训练。** — KAR §4「pretrain」
    - 【SE】对我们：从 `model_trained_on_dns3.tar` 起步。
82. **出问题先读症状索引式排错表，即使这次跑得正常也扫一遍标题。** — HF-ST SKILL §2
83. **一个长期规律：同样算力下，训练配方比架构更重要。** — ORC「Architecture Selection」（引 ResNet Strikes Back）

---

## 10. HF 技能里特有、别处没有的

下面这些是 HF 技能里有、而 Google 调参手册、Karpathy 和 GTCRN 资料里都没有的。它们大多关于**怎样让 AI 代理在临时云机器上可靠地把任务跑完**：

- **把临时环境当默认前提**：不推到外面就全丢；每次存盘就推；结束再推一次并包 try/except；推完要核实（HF-LLM、HF-ST、HF-HUB）
- **超时 = 估计 × 1.3**，并用脚本估算时间和花费、主动报价（HF-LLM、HF-ST、HF-HUB `estimate_cost.py`）
- **数据检查脚本给机器可读的三档结论**（READY / NEEDS MAPPING / INCOMPATIBLE），并直接给出映射代码（HF-LLM、HF-VIS `dataset_inspector.py`）
- **训练末尾一行固定格式的判定** `VERDICT: WIN|MARGINAL|REGRESSION | score | baseline | delta`，供监控程序抓取；训练前必须先记基线（HF-ST）
- **在训练代码里埋分级告警，代理轮询告警自主迭代**（HF-TIO）
- **「可靠优先于性能」写成规则**：首跑不开 `torch.compile` 等优化；先用 10% 数据试跑，再全量（HF-REL、HF-VIS）
- **把提问做成选择题**：开跑前用选项问人「先快速试还是全量」「分不分验证集」「用哪档 GPU（附每小时价格）」（HF-VIS Step 3）
- **代理行为规范**：提交后报告任务 ID、链接、时长、花费，然后不主动轮询；把提交的脚本存到 `submitted_jobs/` 留档（HF-LLM、HF-VIS）
- **压掉刷屏日志**（httpx、huggingface_hub 等压到 WARNING），避免占满代理的上下文（HF-ST）
- **各种「静默失败」的具体坑**：开了评估却没给评估集会卡住；选最优指标名写错会悄悄用最后一个；监控面板默认公开（HF-ST、HF-LLM、HF-TIO）

---

说明：HF 主力训练技能 `huggingface-llm-trainer` 今天（2026-11-26）刚从仓库删掉（标为过时）。本文用的是删除前最后一版，链接已固定到那个提交。现存的同类内容主要在 `huggingface-vision-trainer` 和 `train-sentence-transformers`。
