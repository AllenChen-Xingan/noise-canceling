# F-pvad · Personal VAD 调研（单代理，D47 后补）

> 2026-10-31 · 只读调研，未运行程序。未配怀疑派，关键数字在 decision 自审时复核。「主人音高」已按大总管更正改为「说话人音高」（每台电脑使用者自己的音高，自动注册学出）。

## 结论先行
- 起点就照搬 **PVAD 1.0 的 ET 结构**：2 层单向 LSTM，每层 64 维，约 13 万参数。条件输入改用 FiLM（2.0 的做法）。帧对齐 GTCRN 的 16 ms hop，直接复用它的 STFT 算 log-mel，不加任何延迟。
- 有几处要更正：PVAD 1.0 发在 **Odyssey 2020**，不是 Interspeech；编号是 **arXiv 1908.04284**，不是 2005.xxxxx。2005.07272 是 TS-VAD。PVAD 2.0 编号 2204.03793 没错，会议是 Interspeech 2022。
- 2.0 论文**没有给毫秒级延迟**，也**不支持多个注册人**。论文 §5 把多人列为未来工作。

## 1. 论文要点
**PVAD 1.0**【论文 arXiv 1908.04284】
- **四种结构**（§2.2，图 2）：
  - SC：普通 VAD × 声纹分数，是基线
  - ST：特征拼上逐帧声纹分数，41 维
  - ET：特征拼上 256 维 d-vector，296 维
  - SET：两样都拼，297 维
- **模型**（§3.3）：特征是 40 维 log-mel，帧长 25 ms、帧移 10 ms。网络是 2 层 LSTM（64 单元）接一个 FC。用 Adam，学习率 5e-5。8-bit 量化后，ET 约 13 万参数、130 KB。
- **数据**（§3.1–3.2）：LibriSpeech 960 h，每条随机拼 1–3 句（n~U(1,3)）。训练集 30 万条拼接句，测试集 5000 条。标签由 ASR 强制对齐得到。再加多风格增强（MTR）：用噪声和 300 万种房间配置做混响。
- **结果**（表 1，左右分别是无增强 / 有增强时的 mAP）：
  - SC：0.900 / 0.801
  - ET（交叉熵）：0.946 / 0.883，目标人那一类的 AP 是 0.932 / 0.878
  - ET + 加权成对损失（WPL）：0.959 / 0.912
  - SET：0.969 / 0.928，但运行时要额外跑 4.88M 参数的声纹模型
  - WPL 里 w⟨ns,ntss⟩=0.1 最好（图 4）。不能简单当成二分类来做（§3.5.2）
- **延迟**：论文没给毫秒数，只说"逐帧决策"（§1）。

**PVAD 2.0**【论文 arXiv 2204.03793】
- **结构**（§2.2、§3.2）：
  - 4 层因果 Conformer，维度 64，8 个注意力头，7×7 因果卷积，左上下文 31，无右上下文
  - 特征是 128 维 log-mel（窗 32 ms、帧移 10 ms），4 帧堆叠，3 倍降采样，所以约 30 ms 出一次结果
  - 条件注入用 FiLM，或用"说话人前置网络"算出余弦分数再走 FiLM
- **量化**：E4（8-bit）只有 1.0 MB、9.58 MFLOPs（表 1）。没有给毫秒延迟。
- **没注册时怎么退化**（Alg.1）：训练时以 p0=0.2 的概率把声纹置零，同时把"非目标人"标签改成"目标人"。这样没注册时就等于普通 VAD：表 2 里 VS 7.0、Non-Concat 10.1，和 Conformer 版普通 VAD 持平。
- **数据**：厂商采集，约 1600 h，6923 人（§3.1），不公开。

## 2. 开源实现与本机现状
- **wq2012/personal_vad**【官方 GitHub】
  - Apache-2.0，TF/Keras 写的
  - HF 上有 13 个权重（safetensors + tflite int8），1.0 和 2.0 都覆盖
  - 训练脚本：`scripts/train.py`、`run_benchmarks.py`
  - 数据是 MLS 8 语种 + LibriSpeech，只有 154 个说话人
  - 声纹向量是自制的 LDA/PCA 256 维，替代了闭源的 GE2E
  - 参数量：ET 129,795；2.0 E3 1,091,267
  - 说话人太少，**只适合当结构参照，不适合直接拿权重用**
- **pirxus/personalVAD**【官方 GitHub】：GPL-3.0（会传染，只能参考思路）。2021 年的本科论文，带权重，大约 800 MB。数据是 LibriSpeech 拼接，加可选的 MUSAN/RIR 增强（要用 Kaldi）。
- **nomo-pvad**：18.16M 参数，每 160 ms 出一次，Apache-2.0，训练数据没公开。项目里已有记录：
  - `docs/research/011_multitalker/findings/evidence.md:58、71、72、76、83`
  - `gap.md:16、36`
  - `decision.md:15`
- **Personal VAD 的已有记录**：
  - `docs/research/005_pse/sources.md:39、71`
  - `gap.md:8、19、43`
  - `decision.md:60`
  - `must-know.md:43`
  - `assign.md:125`
- **本机 resources/ 下都在**：wespeaker、3D-Speaker、voxlock、diart、pyannote-rs、speakrs、voice_activity_detector、silero-findit、earshot，另有 speakerbeam、wesep、sherpa-onnx、mlx-examples、DNS-Challenge。许可：
  - wespeaker、3D-Speaker、voxlock：Apache-2.0
  - diart、pyannote-rs：MIT
  - earshot、silero-findit：MIT 与 Apache 双许可

## 3. 相关工作，哪种最合适
- **TS-VAD**【论文 arXiv 2005.07272，Interspeech 2020，二手摘要】：CNN + BLSTM，输入 i-vector，面向 CHiME-6 的说话人分离。双向结构、不因果、模型大，**不合适**。
- **SpeakerBeam-SS**【论文 arXiv 2407.01857】：做的是提取，输出音频；比因果 Conv-TasNet 少 78% 实时率（RTF）。**不是 VAD**，只能参考它的 SSM 做法。
- **Hey Siri 个性化**【官方 Apple ML Journal 2018】：注册时存 5 个声纹向量，之后把通过的语句**隐式追加**，最多 40 个，用余弦打分。这是"零设置自动注册"的工业先例。
- **HyWA**【论文 arXiv 2510.12947，预印本】：用超网络给现成 VAD（FSMN / MarbleNet）的少数几层生成个性化权重。用在插话检测上，误打断从 88.9% 降到 9.9%。是"不改底座"的另一条路。
- **Silero VAD**【官方 README】：大约 2 MB，每块 30 ms 以上，单线程不到 1 ms，MIT。**参数量和结构官方都没公开（未能核实）**。
- **earshot**【本机 README:3-5】：16 ms 帧，RTF 0.0003。参数量未能核实。
- **推荐**：ET + FiLM 的 2×64 LSTM 或 GRU，帧移 16 ms。【推算】输入 40 维 mel，FiLM 由 192 维声纹映射到 64×2，总参数约 8.5–10 万。Conformer 版有 70 万以上参数，超出预算。

## 4. 声纹从哪来
- **候选模型**【本机 3D-Speaker/README.md:28、30】：ECAPA-TDNN 20.8M 参数，CAM++ 7.2M。都输出 192 维向量，每 0.5 s 更新一次，只在判断回路里跑。
- **许可风险**：WeSpeaker 在 VoxCeleb 上训的权重"跟随数据集许可"，即 CC BY 4.0【本机 wespeaker/docs/pretrained.md:21-23】。CAM++ 权重的训练数据许可**未能核实**。
- **注册向量（零设置）**：
  - 冷启动时用零向量，模型等同普通 VAD（PVAD 2.0 Alg.1）
  - 之后把高置信、单人的片段的 ECAPA 向量累积进档案，参照 Hey Siri 的 5→40 个
  - 训练时用 Makishima 的"同一句话既当注册又当输入，加上增强"【论文 arXiv 2106.12132，Interspeech 2021】，以适配短而嘈杂的注册
  - 训练时注册向量要从"自动注册得到的嘈杂向量"里采样，不能只用干净的

## 5. 训练可行性
- **MPS 上的 LSTM**：历史上有正确性 bug，都已修复：
  - batch_first 输出布局（#80306，由 PR #80597 修复）
  - 多层前向与反向（PR #95137）
  - grad_y 缺失（#96601）
  - macOS 12 上会退化为逐个 LSTMCell 跑（#83144）
  - 社区说法是 2.0 版之后基本修好（#90440）【源码/GitHub】
  - 近期"MPS 比 CPU 慢"的 issue 没找到，**未能核实**
  - 建议第一天就让 CPU 和 MPS 对拍一个 batch 的 loss
- **MLX**：有 `nn.LSTM(input_size, hidden_size)`，单层，没有多层或双向参数【官方文档】，性能未能核实。不建议为一个 10 万参数的模型换框架。
- **时长推算**【推算】，假设如下：
  - 300 h 合成数据，16 ms 帧，约 6.75e7 帧/epoch
  - 特征预先算好，存 fp16 约 5.4 GB
  - batch 256 × 4 s 片段，约 1050 步/epoch
  - 每步 50–150 ms
  - 结果：每个 epoch 1–3 分钟，30 个 epoch **约 1–2 小时**
  - 真正的瓶颈是合成数据和教师打标签：VoxLock 27M 参数，用 2 线程跑 20 ms 块时 RTF 0.86【本机 voxlock/README.md:145】，几十小时真实录音要过夜跑
- **数据许可**：
  - LibriSpeech：CC BY 4.0
  - DNS：CC BY 4.0，但内含的 VoxCeleb2 等子集各有许可【本机 DNS-Challenge/README.md:264-290】
  - VoxCeleb：CC BY 4.0，可是"视频版权仍归原作者"【官方 KAIST 许可】，商用有争议，**建议不进训练集**
  - VoxLock：Apache-2.0，NOTICE 写明微调自 REAL-TSE，数据来自 LibriSpeech + AMI【本机 voxlock/NOTICE】
  - AMI：CC BY 4.0

## 6. RL / 不可微奖励
- **直接先例**："VAD 用强化学习、以 ASR 删词率为奖励"的工作**没有找到，未能核实**。
- **最接近的两个**：
  - Amazon 的自适应端点检测【论文 Amazon Science PDF §2.4】：用上下文多臂老虎机，奖励是延迟（ms）和截断指示变量的线性组合，不需要真值标签
  - 二手资料提到 Google 把最小词错率（MWER）训练用于 RNN-T 端点检测：WER 降 8%，P90 延迟降 130 ms【二手，arXiv 2004.11544 未核实原文】
- **判断**：先用监督学习（WPL 损失）训到收敛。RL 只用来调**少数几个门控参数**：阈值、滞回、保持时长。这本质上是老虎机或 CMA-ES 问题，不必对网络权重做策略梯度。
- **建议奖励**：主人说话时，只要门控误关落在词首 200 ms 内，或造成整词删除，就**一票否决**（奖励 −∞）。否则奖励为负的"干扰者残留帧比例"，减去"切换延迟"乘一个系数 λ。

## 7. 两周最小实验草案
- **数据**：
  - LibriSpeech 960 h 拼接，每条 1–3 人，加混叠段（1.0 版没有重叠，这是已知空白，见 005 gap.md:19）
  - 加 DNS 噪声和 RIR，合成约 300 h，标签由模拟器给真值
  - 测试集是不重叠的说话人
  - 另录 1–2 h 真实录音，用 VoxLock 能量差加 ECAPA 离线打教师标签，再抽检
- **模型**：ET-FiLM，2×64 LSTM 或 GRU，三分类（主人 / 他人 / 静音），WPL 损失（w=0.1）。训练时声纹以 20% 概率置零，注册向量用自动注册的嘈杂版本。
- **步骤**：
  - 第 1–3 天：数据与特征
  - 第 4–6 天：训练和 CPU/MPS 对拍
  - 第 7–9 天：评测
  - 第 10–12 天：门控参数做老虎机调参
  - 第 13–14 天：导出 ONNX，测单帧耗时
- **评测指标**：
  - 主人类 AP 和 mAP
  - 主人说话误关率，单列词首 200 ms
  - 只有干扰者说话时的误开率
  - 主人↔他人切换延迟的 P50 和 P90
  - 单帧 CPU 耗时
- **成功标准**【推算，需人定标】，与算法版基线（多线索 + 说话人音高）相比：
  - 词首 200 ms 误关率**不高于基线**（一票否决项）
  - 干扰者误开率相对降低 ≥30%
  - 主人类 AP ≥ 0.90（参照 1.0 有增强时 0.878–0.916）
  - 切换延迟 P90 ≤ 300 ms
  - 每帧耗时 < 0.1 ms
