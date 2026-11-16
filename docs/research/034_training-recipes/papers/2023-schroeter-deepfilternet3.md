# 深度滤波降噪第三版演示（DeepFilterNet: Perceptually Motivated Real-Time Speech Enhancement）

- 出处：Schröter · Escalante-B. · Rosenkranz · Maier · 2023 · Interspeech 2023 演示稿 · arXiv:2305.08227
- 文件：`resources/papers/training/2023-schroeter-deepfilternet3.pdf` · sha256 `32bbbf4e14b7fae53b6dac3f5b57a56dc06623877dc52ac2daa3f39249e2938a`
- 是否读完全文：是（共 2 页，演示短文）

## 模型与参数量
- 48 kHz，20 ms 窗，10 ms 帧移，前瞻 2 帧，总延迟 40 ms（p.1）。
- 第一段：481 个频点压到 32 个 ERB 增益（p.2）。
- 第二段：5 阶复数滤波，只作用最低 96 个频点（约 4.8 kHz 以下）（p.2）。
- 编码器在帧级预测局部 SNR，范围 −15 到 35 dB（p.2）。
- 按局部 SNR 跳过解码器（p.2）：
  - <−10 dB：两个解码器都关，输出静音谱。
  - >20 dB：关深度滤波解码器。
  - 其余：全开。
- 参数量、MACS：文中未写。
- 单线程 i5-8250U 实时率 0.19；推理用 Rust + tract（p.1–2）。

## 训练数据与混音做法
- 全量多语种 DNS4，PTDB 与 VCTK 过采样 10 倍（p.2）。
- 其余混音细节：文中未写。

## 损失
- 只说损失利用「响度感知是对数的」（p.2），具体形式文中未写。

## 优化器、学习率、批量、步数
- 文中未写。

## 训练硬件与时长
- 文中未写。

## 评测方法与主要数字
- Voicebank+Demand（p.2 表 1）：
  - DFN1：PESQ 2.81，STOI 0.942
  - DFN2：PESQ 3.08，STOI 0.943
  - DFN3：PESQ 3.17，CSIG 4.34，CBAK 3.61，COVL 3.77，STOI 0.944
- 只有客观指标，没有听测。
- 演示：Linux 上经 PipeWire + LADSPA 插件做虚拟麦克风（p.2）。

## 消融里哪些做法真有效（带数字）
- 没有消融。DFN3 相对 DFN2 的改动只写「轻微修改的模型」（p.2）。

## 对 GTCRN 加训：能借什么（【推断】）
- 局部 SNR 门控思路：<−10 dB 时直接静音。可作为「停顿时背景稳定」的后处理参考。
- 风险是会吞弱起音；只能作对照实验，不应默认开。
- 虚拟麦克风接法与本项目同类，可作工程参照，不涉及训练。

## 对 GTCRN 加训：不能借什么 / 要小心（【推断】）
- 本稿没有训练配方，不能作为任何超参的出处。
- 「−10 dB 输出静音」与「字头被吞」直接冲突，引用时要写清风险。
- 训练细节须回看 DFN2 论文与开源代码。
