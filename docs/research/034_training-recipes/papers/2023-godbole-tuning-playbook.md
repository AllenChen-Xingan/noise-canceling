# 深度学习调参手册（Deep Learning Tuning Playbook）
- 出处：Varun Godbole, George E. Dahl, Justin Gilmer, Christopher J. Shallue, Zachary Nado · 2023 · Google Research 的 GitHub 文档 v1.0（非论文，非 Google 官方产品）· github.com/google-research/tuning_playbook
- 文件：`resources/papers/training/2023-godbole-tuning-playbook.md` · sha256 `9ee275082cce3b7b1b49d1bfc7b1d6c0e2b4e6a415b6c67a73a72e2eedcd6187`
- 是否读完全文：是（2,155 行分 4 段读完，含全部折叠小节与 FAQ；「页码」以小节名代替）

## 模型与参数量
- 不适用。原则：先复用已能工作的结构，找最接近的论文复现（§Choosing the model architecture）。

## 训练数据与混音做法
- 不适用于混音。输入流水线：昂贵的在线预处理可改为离线做一次存下（§Optimizing the input pipeline）。
- 加数据增强、dropout 会加大梯度方差，可能要更多步（§Determining the number of steps）。

## 损失
- 不适用。

## 优化器、学习率、批量、步数
- 优化器：用该类问题最流行的；起步可固定 Adam 的 ε、β1、β2（§Choosing the optimizer）。
- Adam 调参按预算：<10 次只调学习率；10–25 次加 β1；25+ 次加 ε；远多于 25 次再加 β2（FAQ Adam）。
- 批量原文要点（§Choosing the batch size 及子节）：
- 「批量决定训练速度，不应拿来直接调验证集成绩」。
- 「理想批量常是硬件能放下的最大批量」。
- 按 2 的幂试批量跑少量步，测吞吐；批量翻倍吞吐应近翻倍，否则有 I/O 等瓶颈。
- 吞吐只涨到某个批量为止，就只考虑到该批量。
- 梯度累积不提供吞吐收益，应用中一般应避免。
- 换批量要重调大部分超参，尤其学习率、动量、正则（§Changing the batch size…）。
- 学习率：起步用常数学习率，之后再加衰减（§Choosing the initial configuration）。
- 默认衰减：线性或余弦（FAQ Which learning rate decay…）。
- 不要照抄论文里复杂的分段学习率；那常是人工看验证集临时改的（FAQ complicated schedules）。
- 步数原文要点（§Determining the number of steps / not compute-bound）：
- 「Never tune the max_train_steps number in a study」：一个值用于全部试验。
- 用回溯选检查点找到的最佳步数修正 max_train_steps。
- 最佳步数总在前 10%：步数太多；总在后 25%：可加长并重调衰减。
- 拿不准时宁可训长；配合回溯选检查点，训长不应变差。
- 受算力限制时：第 1 轮短跑找超参，第 2 轮少量长跑出最终模型（§compute-bound Round 1/2）。
- 短跑结论里，热身长度、初始化「很可能」可迁移；学习率衰减「不太可能」迁移（§Round 1）。

## 训练硬件与时长
- 不适用。只给原则：批量取吞吐最大处，训练时间 = 每步时间 × 步数（§Choosing the batch size to minimize training time）。

## 评测方法与主要数字
- 按固定步数间隔评测，不按时间间隔（§Setting up periodic evaluations）。
- 评测批量至少等于训练批量；补齐的样本权重设 0（同上）。
- 每次评测存部分样本的预测，便于事后排错（同上）。
- 抽样评测集要和全量评测集无偏差；类别少的记「对了几个」而非只记比率（§Choosing a sample…）。
- 检查点原文要点（§Saving checkpoints and retrospectively selecting the best checkpoint）：
- 「Run training for a fixed number of steps and retrospectively choose the best checkpoint」。
- 最佳检查点常不是最后一个；流水线记录目前最好的 N 个，训完再挑。
- 预设了试验预算并保留 N 个最好，一般不需要提前停。
- 实验记录表列：研究名、配置链接、简述、试验数、最佳检查点的验证成绩、复现命令（§Setting up experiment tracking）。
- 唯一数字：ResNet-50/ImageNet 换种子重训的波动约 ±0.1%（误差率约 23%）；6 次试验与 20 次试验的最优值四分位距差很大（FAQ How many trials，图 3）。

## 消融里哪些做法真有效（带数字）
- 无系统消融，以经验与图示为主。带数的条目：
- 某 WideResNet 改步长后不稳；1000 步热身后可在学习率 0.1 稳定训（FAQ 图 4）。
- 某例需要 40k 步热身才能训成（FAQ 图 6）。
- 热身步数按数量级扫：10、10³、10⁴、10⁵；最大可行值不超过 max_train_steps 的 10%（FAQ warmup）。
- 梯度裁剪阈值取梯度范数的第 90 百分位；超过 50% 的步被裁剪就算「过猛」，不如降学习率（FAQ Gradient clipping）。
- 排查早期不稳：用 2 倍最佳学习率跑约 500 步，每步都评测（FAQ Identifying unstable workloads）。

## 对 GTCRN 加训：能借什么（【推断】）
- 【推断】超参三分法（要研究的 / 要调公平的 / 固定的）可直接套到防遗忘实验：L2-SP 强度是「要研究的」，学习率是「要调公平的」。
- 【推断】固定 max_train_steps + 按固定步数评测 + 保留 N 个最好检查点，可写进加训脚本。
- 【推断】GTCRN 小、GRU 串行，先按 2 的幂测 4090 吞吐，再取吞吐饱和的批量。
- 【推断】晚期步间波动大时，手册列的解法含 Polyak 平均，与 EMA 同类，支持保留 EMA。
- 【推断】记录梯度范数直方图，按第 90 百分位设裁剪阈值。
- 【推断】换种子重训 N 次量出波动，再判断新做法是否真有效。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 【推断】手册面向从零训练与大规模调参；微调时「宁可训长」会与防遗忘冲突，需同时看旧测集。
- 【推断】「回溯挑最佳」若在测试集上挑会泄漏；须在独立开发份上挑，测试集只看一次。
- 【推断】准随机搜索、贝叶斯优化在我们的预算下（几次到十几次试验）意义有限；按手册只调学习率即可。
- 【推断】手册没有「先写预测再跑」这一条；建议 8 的这一半出处不是本手册。
