# GTCRN 会议海报（GTCRN: A Speech Enhancement Model Requiring Ultralow Computational Resources）

- 出处：Rong · Sun · Zhang · Hu · Zhu · Lu · 2024 · ICASSP 2024 海报（SigPort）· 代码 https://github.com/Xiaobin-Rong/gtcrn
- 文件：`resources/papers/training/2024-rong-gtcrn-poster.pdf` · sha256 `9de7a7474a295fc0e774da0c54b1afaa7c6cb2861571244f0139a145074a5840`
- 是否读完全文：否。论文正文在 IEEE Xplore 要付费，无法获取，只读了这份 1 页海报（已读完）。

## 模型与参数量
- 23.7 k 参数，39.6 M MACs/s（海报 p.1 引言）。
- 结构：频带合并 → 子带特征提取 → 2 层卷积 + 3 层分组时间卷积 → 分组双路径 RNN → 对称解码 → 频带拆分（p.1 方法图）。
- 时间循环注意力（TRA）：平方 → 频率平均池化 → GRU → 全连接 → Sigmoid（p.1）。

## 训练数据与混音做法
- DNS3：SNR −5 到 15 dB，约 2000 h，加 DiDiSpeech 中文语料（p.1）。
- 每轮 40 000 对 8 s 样本；验证 840，测试 800；另测 DNS3 盲测集（p.1）。
- VCTK-DEMAND：16 kHz，训练 10 000，验证 1572，测试 824（p.1）。
- 电平、均衡、混响细节：海报未写。

## 损失
- L = α·SI-SNR + (1−β)·压缩幅度 MSE + β·(实部 + 虚部 MSE)（p.1）。
- 幅度压缩 0.3；实 / 虚部除以 |S|^0.7（p.1）。
- α、β 取值：海报未写；UL-UNAS 论文给出 0.01 / 0.3（见 `2025-rong-ulunas.md`）。

## 优化器、学习率、批量、步数
- 海报未写。

## 训练硬件与时长
- 海报未写。

## 评测方法与主要数字
- VCTK-DEMAND：SI-SNR 18.83，PESQ 2.87，STOI 0.940（p.1 表 2）。
- DNS3 盲测集：DNSMOS P.808 3.44；P.835 BAK 3.90，SIG 3.00，OVRL 2.70（p.1 表 3）。
- 带噪：P.808 2.96；BAK 2.65，SIG 3.20，OVRL 2.33（p.1 表 3）。
- 处理后 SIG 3.00 低于带噪 3.20，即语音本身有损（p.1 表 3）。

## 消融里哪些做法真有效（带数字）
- DNS3 测试集（p.1 表 1；末列表头印作 PESQ，数值像 STOI）：
  - 三件都不加：SI-SNR 9.87，PESQ 1.87
  - 只加 TRA：10.25 / 1.91
  - 子带特征提取 + TRA（最终版）：10.39 / 1.94
  - 用 TFA 代替 TRA：10.29 / 1.92
- 单测试集单次运行，差值 0.03–0.07 PESQ。

## 对 GTCRN 加训：能借什么（【推断】）
- 作者自述局限：低 SNR 下性能下降；容量小导致泛化受限（p.1 讨论）。
- 这与本项目「低 SNR」补训方向一致，可作立项依据。
- 损失形式与本项目 HybridLoss 一致，可放心沿用。
- 每轮 4 万对 8 s 样本的「固定轮长」做法，便于按步数比较。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 海报没有优化器、学习率、批量，任何超参都不能以它为出处。
- SIG 降 0.20 提示原模型已有语音损伤；加训若只压背景，可能更伤语音。
- 正文不可得，海报数字无法与正文核对。
