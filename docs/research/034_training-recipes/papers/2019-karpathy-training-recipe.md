# 训练神经网络的配方（A Recipe for Training Neural Networks）
- 出处：Andrej Karpathy · 2019-04-25 · 个人博客（非论文，无同行评审）· karpathy.github.io/2019/04/25/recipe/
- 文件：`resources/papers/training/2019-karpathy-training-recipe.html` · sha256 `753210544b2068e6b1f05cc26f3e813cd2cfa22d845457c1aa549ca4ce576b2d`
- 是否读完全文：是（剥 HTML 标签后读完正文 6 节与结语；「页码」以小节名代替）

## 模型与参数量
- 不适用（通用方法文）。
- 选模型：「Don't be a hero」，先抄最相关论文里最简单的好结构（§3 picking the model）。

## 训练数据与混音做法
- 不适用于混音。通用做法：
- 先不碰模型代码，花数小时看上千条数据，找重复、损坏、不平衡（§1）。
- 写小脚本按各维度排序、筛选，看分布与离群点；离群点常暴露数据 bug（§1）。
- 冒烟阶段关掉数据增强（§2 simplify）。
- 正则化首选「加真实数据」，其次更激进的增强，再其次仿真 / 合成数据（§4）。

## 损失
- 「verify loss @ init」：初始损失要等于理论值，如 softmax 为 −log(1/类数）；L2、Huber 同理可推（§2）。
- 「init well」：最后一层偏置按目标均值初始化，避免前几步只在学偏置（§2）。

## 优化器、学习率、批量、步数
- 「adam is safe」：起步用 Adam，学习率 3e-4（§3）。
- 「do not trust learning rate decay defaults」：按 epoch 数衰减的默认值会随数据量变化而错（§3）。
- 作者自己先关掉学习率衰减，用常数学习率，最后再调（§3）。
- 减小批量在有 BatchNorm 时相当于加强正则（§4）。
- 多超参同时调，用随机搜索而非网格（§5）。
- 「leave it training」：验证损失看似平了也别急着停，网络会训很久（§6）。
- 早停：按验证损失停在快要过拟合处（§4 early stopping）。

## 训练硬件与时长
- 不适用。

## 评测方法与主要数字
- 评测要跑整个测试集，不靠按批平滑曲线（§2 add significant digits to your eval）。
- 冒烟步骤（§2 原文条目，依序）：
- 1）fix random seed：固定随机种子，两次运行结果一致。
- 2）simplify：关掉增强等多余花样。
- 3）add significant digits to your eval：全量测试集评测。
- 4）verify loss @ init：核对初始损失值。
- 5）init well：最后一层正确初始化。
- 6）human baseline：监控人能看懂的指标，与人的水平比。
- 7）input-independent baseline：输入置零训练，结果应更差。
- 8）overfit one batch：少到 2 条样本，过拟合到最低损失（如 0），并画标签与预测对齐。
- 9）verify decreasing training loss：加一点容量，训练损失应下降。
- 10）visualize just before the net：在 y_hat = model(x) 前一刻把张量解码可视化，这是「唯一真相源」。
- 11）visualize prediction dynamics：固定一批测试样本，训练中看预测怎么动。
- 12）use backprop to chart dependencies：损失设为第 i 条输出之和，只有第 i 条输入有梯度；自回归可同法查是否偷看未来。
- 13）generalize a special case：先写专用循环版，再逐步向量化并核对结果相同。

## 消融里哪些做法真有效（带数字）
- 无实验数字，全为经验之谈。
- 唯一带数的经验：集成「几乎稳涨 2%」，约 5 个模型后封顶（§4、§6）。

## 对 GTCRN 加训：能借什么（【推断】）
- 【推断】第 8 条（1 批过拟合）、第 4 条（初始损失核对）可直接做成开训前冒烟脚本。
- 【推断】第 12 条可改成因果性检查：对第 t 帧输出求梯度，确认 t 之后的输入梯度为 0。
- 【推断】第 10 条：在送入 GTCRN 前把混音后的 STFT 存成音频听，抓混音 / 增强 bug。
- 【推断】第 11 条：固定一小批「字头」「停顿」样本，每 N 步导出音频，看动态。
- 【推断】「一次只加一样复杂度」适合防遗忘手段：L2-SP、老师、回放、EMA 逐个加并量。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 【推断】文章面向从零训练；「初始损失等于理论值」在微调时应改为「等于原检查点在该数据上的已知损失」。
- 【推断】「1 批过拟合」要在副本上做，不能污染要加训的检查点。
- 【推断】「leave it training」与微调防遗忘冲突：微调训太久会漂离原模型，要按步数评估。
- 【推断】Adam 3e-4 是从零训的起点；微调常需更小（同批 SAFE 用 1e-5，Bereuter 用 5e-5）。
- 【推断】原文没有「原检查点在开发份上复测应等于已知成绩」这一条；那是建议 1 ③ 自加的。
