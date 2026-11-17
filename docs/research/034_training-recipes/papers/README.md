# 034 · 论文全文精读索引

> 2026-11-16 · 调研子代理（大总管 b19 派）领 4 个会话内子代理分批读，主代理抽查了 5 处关键数字（SAFE 表 6、DFN2 批量理由、Braun 电平参数、Taherian 15% 全零样本、DPDFNet 长段微调），都和原文一致。
> 原文都在 `resources/papers/training/`。PDF 和整个 `resources/` 目录已被 `.gitignore` 忽略，不进仓库。每份笔记第一节写了 sha256。
> 来源：都在 `../sources.md`（S9、S17、S20–S32），以及 `docs/research/027_gtcrn-finetune/sources.md`（M1、M3–M14）和 `docs/research/028_gtcrn-arch/sources.md`（S2、S4），另补了 DeepFilterNet 1/2/3、多分辨率 STFT 的出处、DPDFNet 和 PVQE。同一篇只下了一次。
> 证据强度：除 Monir 2026 做过显著性检验外，所有消融都只训一次、测一个测试集、不报方差。下文的「证实」只说明「原文这样写、这样测过」，不说明在 GTCRN 上也成立。

## 一、索引

下表一行一篇。读者可以先按「一句话要点」挑出要细看的笔记。

| 笔记 | 读完了吗 | 一句话要点 |
|---|---|---|
| `2024-rong-gtcrn-poster.md` | 否：论文正文在 IEEE Xplore 要付费，**拿不到全文**，只读了作者在 SigPort 公开的 1 页海报 | 作者承认低 SNR 下性能下降；海报没写优化器、学习率、批量 |
| `2025-rong-ulunas.md` | 是（12 页） | GTCRN 作者的官方重训配方：Adam；预热 2.5 万步到 1e-3，余弦退到 25 万步；批量 32；SNR −5～15 dB |
| `2025-wang-adaptive-conv.md` | 是（14 页） | 混音后把整段幅度随机缩放到满幅的 0.01–0.99，约 40 dB 的电平范围 |
| `2021-schroeter-deepfilternet.md` | 是 | 每条语音叠最多 5 段噪声；SNR −5～40 dB；语音和噪声各加 ±6 dB 增益和均衡 |
| `2022-schroeter-deepfilternet2.md` | 是 | 三项调度合计 PESQ +0.11，多分辨率损失 +0.06，数据增强 +0.06 |
| `2023-schroeter-deepfilternet3.md` | 是（2 页演示稿） | 没有训练配方；局部 SNR 低于 −10 dB 时直接输出静音 |
| `2025-rika-dpdfnet.md` | 是（11 页） | 加过压损失和长段微调后，低 SNR 集上 SI-SNR 从 10.25 升到 13.27 dB |
| `2023-nathoo-two-step-kd-tiny-se.md` | 是 | 6.2 万参数的学生先只蒸馏、再纯监督，ΔSDR +0.44 dB；−5 dB 下收益最大 |
| `2022-thakker-e3net-kd.md` | 是 | 老师在真实录音上出伪标签训学生，误压时长从 3.57 降到 0.56 秒 |
| `2023-taherian-cross-task-kd-pse.md` | 是 | 加 15% 全零目标能治旁人泄漏，但会加重误压；按语音检测把难帧排出损失后，两头都能保住 |
| `2021-eskimez-pse-models-eval.md` | 是 | 提出逐帧误压判据（TSOS）；只罚欠估的不对称项让误压从 7.48% 降到 5.37% |
| `2024-serre-pdfn2-dual-stage.md` | 是 | DFN2 加声纹向量；损失是谱损失 + 四窗多分辨率 + 过压抑制，都没有消融 |
| `2024-parnamaa-pvqe.md` | 是 | 用模型自己的内部状态当声纹；Adam 6e-5，批量 64，23.4 万步，片段 40 秒 |
| `2023-wang-unified-pse.md` | 是 | 有声帧加权（µ=0.9）减少误压，BAK 略降；用随机低通、均衡、电平做增强，没消融 |
| `2020-wang-voicefilter-lite.md` | 是 | 不对称 L2 损失（α=10）减少误删；推理时自适应混回原声 |
| `2020-xia-weighted-distortion-loss.md` | 是 | 损失拆成语音失真项和残噪项；众包偏好的 α 和作者偏好的不同，客观分都预测不到这个差别 |
| `2020-braun-augmentation-loss-norm.md` | 是（8 页） | SNR 增强主要防过拟合（PESQ +0.04）；电平增强要配「按活跃语音电平归一的损失」，否则会小幅掉分 |
| `2020-braun-consolidated-loss.md` | 是 | 30% 复谱 + 70% 幅度、压缩 0.3 是测出的最优点（PESQ 3.31），也就是 GTCRN HybridLoss 的前两项 |
| `2019-yamamoto-parallel-wavegan-mrstft.md` | 是 | 多分辨率 STFT 三组 FFT 设置的出处（512/1024/2048） |
| `2020-defossez-demucs-realtime.md` | 是 | 去掉多分辨率 STFT 掉 0.18 PESQ，去掉随机平移再掉 0.30 |
| `2023-wan-multi-loss-tfa.md` | 是 | 23 万参数的模型加多分辨率 STFT 后 PESQ +0.10，字错率反而从 13.79 升到 14.35 |
| `2026-monir-tf-weighted-loss.md` | 是 | 按谱通量（逐帧能量突增）给损失加权，辅音识别率 +2.1 点，显著；−8 dB 下不如基线 |
| `2025-saijo-urgent2025-challenge.md` | 是 | 训练数据约 2.5 千小时扩到约 6 万小时（含带噪语音），同一队 PESQ 只变 ±0.02 |
| `2023-lemercier-wind-storm.md` | 是 | 风噪混音：SNR U(−6, 14) dB；以 0.75 概率削波，阈值 U(0.85, 1.0)×最大幅度 |
| `2025-eftekhari-windnetlite.md` | 是 | 24.9 万参数的因果模型能压风噪；通用模型的风噪泄漏更多（−1.76 对 −2.81） |
| `2020-lee-seril.md` | 是 | 按参数重要度加权的 L2 防遗忘，旧噪声上好于直接微调，但仍低于原模型（2.461 对 2.708） |
| `2026-cheng-lightweight-adaptation.md` | 是 | 冻结骨干 + 秩 1 适配器（512 参数）不漂移；全参连续更新会累积漂移 |
| `2026-bereuter-se-to-singing.md` | 是 | 全参微调在原领域 PESQ 降 0.28 / 0.40；LoRA 零遗忘，保留约 90% 收益 |
| `2026-sharma-safe-segan-few-shot.md` | 是（18 页） | **表 6：去掉 L2-SP 后四项分数与完整版完全相同；去掉 EMA 后四项全部更高** |
| `2021-fu-metricgan-plus.md` | 是 | 评分网络进损失后 PESQ 3.15，但背景分 CBAK 3.16，低于 MSE 版的 3.28 |
| `2025-li-perceptual-feedback-dpo-se.md` | 是 | 按 UTMOS 做 DPO 后 UTMOS 涨 56%，说话人相似度从 0.691 降到 0.651 |
| `2025-zhang-multi-metric-preference.md` | 是（16 页） | 只按一个指标对齐，其他指标会退步；用大模型出伪标签、再训小判别模型有效 |
| `2019-karpathy-training-recipe.md` | 是（博客，按小节读） | 冒烟步骤原文：核对初始化时的损失、过拟合一个批次、可视化送进网络前的数据 |
| `2023-godbole-tuning-playbook.md` | 是（约 12 万字符，分 4 段读完） | 批量取吞吐还在涨的最大值；训练步数在一次研究里不调；保留 N 个最好的检查点，事后挑 |

合计：下载 34 份（32 份 PDF、1 份 html、1 份 md）。33 份读完全文。1 份拿不到全文：GTCRN 论文正文（只有海报）。

## 二、全文读完后，对 `../README.md` 第四节 10 条建议的修正

用途：标出每条建议是被原文证实、要改，还是站不住。页码指 PDF 页；非论文的长文写小节名。

### 建议 1 · 开训前的冒烟三件事 —— 证实前两件，第三件出处要改

- 证实：「初始化时核对损失」「过拟合一个批次」原文都有（`2019-karpathy`「2. Set up the end-to-end training/evaluation skeleton」小节：verify loss @ init · overfit one batch）。
- 要改：第 ③ 件「原检查点在开发份上复测，读数等于已知成绩」原文没有，是我们自己加的。它仍然合理，但出处要标【推断】。
- 要改：我们是微调，第 ① 件应改成「损失等于原检查点在这批数据上的已知损失」，不是随机初始化的理论值【推断】。

### 建议 2 · 云端检查点外存与预算包络 —— 只证实一半

- 证实：Playbook 说存检查点是为了扛实例中断（`2023-godbole`「Saving checkpoints」小节）。
- 预算上限（几小时、几美元）在 34 份里都没有出处，仍只靠 S2、S3 两个 skill 的做法。

### 建议 3 · 设备链仿真 + 损失电平归一 —— 方向证实，参数和预期收益要改

- 证实：语音和噪声各加二阶均衡和随机增益（`2021-schroeter-deepfilternet` p.3；`2023-wang-unified-pse` p.3）。削波只加在输入上（`2022-schroeter-deepfilternet2` p.3）。
- 证实：电平增强配标准损失会掉分，测试 PESQ 3.32 → 3.30；按活跃语音电平归一后回到 3.31（`2020-braun-augmentation-loss-norm` p.4–6）。原文的损失式就是 c=0.3、α=0.3，和 HybridLoss 前两项一样（`2020-braun-consolidated-loss` p.4 表 2）。
- 要改 ①：参数改用原文值。电平 N(−28 dBFS, 10 dB)，SNR N(5, 10) dB，二阶滤波系数 r∈[−3/8, 3/8]（`2020-braun-augmentation-loss-norm` p.5）。「±6 dB 起」太窄：同一组作者用满幅 0.01–0.99，约 40 dB（`2025-wang-adaptive-conv` p.7）。原文写「variance 10 dB」，没说清是方差还是标准差，实现时要写明。
- 要改 ②：别期待归一会涨分。它的作用是「加了电平增强以后不掉分」，收益只有 0.01–0.05 PESQ，只训了一次（同上 p.6）。
- 要改 ③：归一除以活跃语音的标准差。近静音样本会除以接近 0 的数，要加下限【推断】。
- 补充：削波有现成参数，概率 0.75，阈值 U(0.85, 1.0)×最大幅度（`2023-lemercier-wind-storm` p.3 表 1）。

### 建议 4 · 多段噪声 + 近静音样本 —— 前一半要改，后一半站不住（会伤字头）

- 要改：DFN1 是「最多 5 段」（`2021-schroeter-deepfilternet` p.3）。另一份公开配方是「80% 叠 1 段、20% 叠 2 段」（`2025-li-perceptual-feedback-dpo-se` p.3 §3.1）。所以不是「每条都叠 2–3 段」，应改成按比例抽段数。
- **站不住（照原样做有害）**：全零目标样本占 15% 时，旁人泄漏指标从 46.5 升到 148.3 dB，删字率从 2.55% 升到 6.17%，误压时长从 0.45 秒升到 2.54 秒（`2023-taherian-cross-task-kd-pse` p.4 表 1）。这是个性化降噪的结果，但机理相同：模型学会「拿不准就关」。要留近静音样本，必须配「难帧不计损失」或防误压项（同文 p.4 表 2）。
- 另一条路：DPDFNet 治停顿背景和静音后首字被截，靠的是长段连续微调，不是加样本。配方是批量 1，连续 30–40 秒片段，5000 步，学习率 1e-5（`2025-rika-dpdfnet` p.7）。
- URGENT 官方用 webrtcvad 把静音为主的样本**删掉**（`2025-saijo-urgent2025-challenge` p.2）。

### 建议 5 · 大模型老师 + 真实录音 —— 证实，风险描述要改

- 证实：老师在真实录音上出伪标签训学生，误压时长 3.57 → 0.56 秒，DNSMOS 3.38 → 3.48（`2022-thakker-e3net-kd` p.4 表 1）。但学生有 450 万参数，比 GTCRN 大约 90 倍。
- 证实：两步法（先只蒸馏、再纯监督）ΔSDR 6.34 → 6.77，−5 dB 下 10.82 → 11.73（`2023-nathoo-two-step-kd-tiny-se` p.3 表 2、p.4 表 3–4）。但那是从零训学生，不是微调。
- 要改：「老师的吞字头会传给学生」不一定成立。有一例学生的误压比老师还少，老师 14.61 秒，学生 4.83 秒（`2022-thakker-e3net-kd` p.4 表 1）。老师输出仍要先过检查，但别默认学生会更差。
- 要改：要检查的不只字头和背景，还有字准确率。生成式模型主观 MOS 最高（3.69），字准确率却低于带噪输入（67.87 对 73.41）（`2025-saijo-urgent2025-challenge` p.4 表 2–3）。DPDFNet 拿模型清洗过的语音当目标，但没评估清洗瑕疵会不会传下去（`2025-rika-dpdfnet` p.4）。

### 建议 6 · 多分辨率 STFT 当可选第 5 项损失 —— 证实，加一条守门要求

- 证实：去掉它 PESQ 掉 0.18（`2020-defossez-demucs-realtime` p.4 表 3）；加上它 DFN2 PESQ +0.06（`2022-schroeter-deepfilternet2` p.4）。FFT 设置可用 {512/240/50, 1024/600/120, 2048/1200/240}（`2019-yamamoto-parallel-wavegan-mrstft` p.3 表 1）。
- 证实「不替换 HybridLoss」：HybridLoss 前两项就是 Braun 测出的最优配比（`2020-braun-consolidated-loss` p.4 表 2）。
- 要改：加多分辨率 STFT 后 PESQ 升，字错率从 13.79 升到 14.35（`2023-wan-multi-loss-tfa` p.4 表 3）。扫参时必须同时看吞字 / 字准确率，不能只看 PESQ。

### 建议 7 · 固定步数 + 留 N 个最好、事后挑 —— 证实，但「保留 EMA」要改

- 证实：原文有「Never tune the max_train_steps number in a study」，也有「保留 N 个最好、事后挑」（`2023-godbole`「Saving checkpoints」等小节）。
- **要改**：EMA 不能默认保留。SAFE 去掉 EMA 后目标域 PESQ 1.141、源域 1.887，都高于完整版的 1.113 和 1.849（`2026-sharma-safe-segan-few-shot` p.11 表 6，主代理已核原表）。同一张表里，去掉 L2-SP 后四项分数与完整版完全相同。027 取的「EMA 0.995 · L2-SP 1e-4 · 老师 0.1」出自这篇，**这些数只能当扫参起点，不能当已验证的值**。EMA 要和不用 EMA 的权重在开发份上对比。
- 补充：DFN2 按验证损失挑检查点（`2022-schroeter-deepfilternet2` p.3）；不能在测试集上挑（`2026-bereuter-se-to-singing` p.3 §3.4 这样做了，属泄漏）。

### 建议 8 · 实验登记格式 —— 证实一半

- 证实：超参分「要研究的 / 要调公平的 / 固定的」三类；实验记录表 6 列（`2023-godbole`「Identifying scientific, nuisance, and fixed hyperparameters」小节）。
- 「先写预测再跑」在 34 份里都没有出处，仍只来自 S4 的 skill。
- 补充：判「更好」可用逐句配对的 Wilcoxon 符号秩检验，α=5%（`2026-monir-tf-weighted-loss` p.2），可以写进检验脚本（verifier）。

### 建议 9 · 评测脚本对训练方只读 —— 无新证据

- 34 份里没有直接相关的出处，仍只来自 S5。

### 建议 10 · 先测吞吐再定批量 —— 做法证实，引的理由站不住

- 证实做法：批量按 2 的幂试；批量翻倍时吞吐也该接近翻倍，吞吐不再涨就停；梯度累积一般应避免；换批量要重调学习率（`2023-godbole`「Determining the feasible batch sizes」小节）。
- **站不住**：「DFN 批量逐步加大和 GRU 串行瓶颈有关」没有出处。DFN2 原文只说是为了「开头收敛更快」，引的是 Smith 2017「别降学习率，加批量」（`2022-schroeter-deepfilternet2` p.2，主代理已核原文）。而且那是从零训练的配方，微调要不要逐步加批量，没有证据。

### 10 条之外，全文里冒出来、值得进第一轮扫参的三件事（建议，待你定）

1. **防吞字的损失项**，下面三种写法任选：
   - 只在输出能量低于目标的频点上，再算一遍压缩幅度项（`2025-rika-dpdfnet` p.5 式 8–10）
   - 不对称欠估惩罚，误压从 7.48% 降到 5.37%（`2021-eskimez-pse-models-eval` p.4 表 1）
   - 按谱通量给字头加权，辅音识别率 +2.1 点（`2026-monir-tf-weighted-loss` p.3 表 2）

   三种都有代价，是停顿残噪变多。DPDFNet 的过压权重从 0 → 500 → 1000，语音保留比约 0.83 → 0.89 → 0.94，非语音残留约 −57 → −52.7 → −52 dB（p.8 图 4，读图得来）。所以必须和停顿背景的检验脚本一起看。
2. **长段连续微调当收尾**：批量 1，连续 30–40 秒片段，5000 步，学习率 1e-5（`2025-rika-dpdfnet` p.7）。在 4090 上代价未量化。论文只有一张频谱图示例，所以要配我们自己的「静音后首字」检验。
3. **最保守的对照组**：冻结原模型，只加秩 1 适配器。512 参数时 SI-SDR 6.75 → 8.17，好于秩 64（`2026-cheng-lightweight-adaptation` p.4 表 2）。GTCRN 大的全连接层很少，适配器能加在哪要另查【推断】。

另外，「评分模型不进损失」（D81）有两篇新证据支持：MetricGAN+ 的 CBAK 3.16 低于 MSE 版的 3.28（`2021-fu-metricgan-plus` p.4 表 3）；按 UTMOS 做 DPO 后，说话人相似度从 0.691 降到 0.651（`2025-li-perceptual-feedback-dpo-se` p.4 表 1）。
