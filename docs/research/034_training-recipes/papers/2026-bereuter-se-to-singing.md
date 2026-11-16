# 把降噪模型迁移到人声分离（Teaching Speech Enhancement Models to Sing: Domain Adaptation from Speech Enhancement to Singing Voice Separation）
- 出处：Paul A. Bereuter, Mark D. Plumbley, Alois Sontacchi · 2026 · 会议名文中未写（版式为 ICASSP 类）· arXiv:2607.11630v1
- 文件：`resources/papers/training/2026-bereuter-se-to-singing.pdf` · sha256 `7b8746fc719f80fd0ea02a9dfbfbc5f9d647b4081a130966bbdffe7d4091f999`
- 是否读完全文：是（共 5 页，表 1–3 已对图核对）
- 代码：github.com/pablebe/se2svs（p.4 脚注）

## 模型与参数量
- 判别式：BSRNN，48 kHz，Hann 窗，FFT 960、跳 480（p.2 §3.3）。
- BSRNN 切 34 个子带，每带 196 通道；6 层，每层两个 BiLSTM（p.2–3 §3.3）。
- BSRNN 底座 37.80 M 参数，84.31 GMACs/s（p.4 表 3）。
- 生成式：SGMSE（扩散，NCSN++ 骨干），FFT 1534、跳 384（p.3 §3.3）。
- SGM 底座 64.74 M 参数，399.64 GMACs/s（p.4 表 3）。
- 两者都非因果（BiLSTM / 扩散），与 GTCRN 量级相差约 800 倍【推断，按参数数算】。

## 训练数据与混音做法
- 预训练：BSRNN 用 URGENT 数据约 700 h；SGM 用 EARS-WHAM 约 87 h（p.2 §3.1）。
- 迁移：MUSDB18-HQ + MoisesDB 合计约 35 h（p.2 §3.2）。
- 增强：随机增益、随机重组混音（p.2 §3.2）。
- MUSDB 从 44.1 kHz 升到 48 kHz（p.2 §3.2）。
- MSRBench 测前按 EBU R128 对齐响度，避免增益偏差（p.2 §3.2）。

## 损失
- BSRNN：20% 波形 L1 + 80% SI-SDR（p.3 §3.3）。
- SGM：条件去噪分数匹配（p.3 §3.3）。

## 优化器、学习率、批量、步数
- 优化器名：文中未写。
- BSRNN 微调（全参与 LoRA）：批量 14，学习率 5e-5（p.3 §3.4）。
- SGM 全参：批量 4，学习率 1e-4；SGM LoRA：批量 3，学习率 2e-4（p.3 §3.4）。
- 从零训：学习率 1e-4；批量 SGM 4、BSRNN 14（p.3 §3.4）。
- 全部训练 550 个 epoch（p.3 §3.4）。
- LoRA：BSRNN 秩取 16/32/128，α = 2r；秩 8 分离效果不够（p.3 §3.4）。
- LoRA：B 初始化为 0，A 用 Kaiming 均匀；不调偏置；dropout 0.005（p.3 §3.4）。
- 检查点选法：在 GenSVS 测试集上 SDR 最高的那个（p.3 §3.4）。

## 训练硬件与时长
- SGM 一次训练约 6 天，3 张 RTX 6000 Blackwell Max-Q（p.3 §3.4）。
- BSRNN 时长：文中未写。

## 评测方法与主要数字
- 原领域（降噪）：EARS-WHAM 前 5 s，SI-SDR、PESQ、DistillMOS（p.3 §3.5）。
- 新领域：GenSVS（SDR、MR-Loss、MERT-MSE）；域外：MSRBench（p.3 §3.5）。
- SGM 每条生成 10 次取平均（p.3 §3.3）。
- BSRNN 全参：GenSVS SDR 9.87；但降噪 PESQ 2.80→2.52（p.3 表 1）。
- BSRNN LoRA 32：GenSVS SDR 8.92；降噪 PESQ 保持 2.80（p.3 表 1）。
- SGM 全参：GenSVS SDR 7.62；降噪 PESQ 2.49→2.09（p.3 表 1）。
- 正文称 PESQ 降 0.28（BSRNN）和 0.40（SGM）（p.4）。
- 迁移 vs 从零：BSRNN SDR 9.87 vs 8.05；SGM 7.62 vs 6.94（p.3 表 1）。

## 消融里哪些做法真有效（带数字）
- 从降噪预训练起步好于从零：SDR 高 0.29–1.8 dB（p.1 摘要；p.3 表 1）。
- 全参微调新领域最好，但旧领域遗忘：PESQ −0.28 / −0.40（p.4）。
- LoRA 保住旧领域：关掉适配支路即还原底座（p.2 §2.2；p.3 表 1）。
- LoRA 32 保留全参约 90% 的新领域收益，只多 12.12% 参数（p.4）。
- 秩 16→128：GenSVS 只多 0.47 dB，参数多到 +48.49%（p.4）。
- 未测：全参 + 正则（L2-SP、EWC）或回放；无这类对照。

## 对 GTCRN 加训：能借什么（【推断】）
- 【推断】「全参微调会遗忘、遗忘量要在原领域测集上量出来」可直接借为评测规矩。
- 【推断】必须同时报「新场景收益」与「旧测集 PESQ 降幅」，画成取舍图（同 p.4 图 1）。
- 【推断】LoRA 初始化 B=0，等于从原模型出发，可作为最保守的对照组。
- 【推断】学习率比从零训低一个量级附近（5e-5 vs 1e-4），支持微调用小学习率。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 【推断】任务跨度大（语音→唱歌伴奏分离），比我们补弱场景的迁移大得多。
- 【推断】模型大、非因果、48 kHz；超参不能照抄到 4.8 万参数的 GTCRN。
- 【推断】检查点按测试集 SDR 挑，有测试集泄漏；我们应按独立开发份挑。
- 【推断】LoRA「关掉即还原」的好处只在部署可切换时有用；我们只发一个模型。
- 【推断】每组只跑一次，无多种子方差。
