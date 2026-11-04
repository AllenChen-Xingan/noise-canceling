# Libri-Adapt（六种麦克风重录的 LibriSpeech）

- **名称**：Libri-Adapt: A New Speech Dataset for Unsupervised Domain Adaptation（Libri-Adapt，ICASSP 2020）
- **链接**：https://github.com/akhilmathurs/libriadapt
- **许可**：README 原句「The dataset is released under CC BY 4.0 license.」→ 结论：训练可【官方文档】
- **大小**：LibriSpeech clean-100（100 h）× 6 设备 × 多口音 / 噪声条件；72 个域 × 100 h ≈ 7,200 h【推断：据 VentureBeat 报道与论文域数相乘】；GB 未查到
- **场景键**：other（录音房间未写；噪声 Rain / Wind / Laughter 为「在 6 支麦上分别录噪声再与语音相加」）
- **设备键**：phone（Google Nexus 6，3 通道）, array（Matrix Voice、ReSpeaker，各 7 通道）, desktop（PlayStation Eye 4 通道、USB 麦）, studio（Shure MV5）
- **笔记本段**：没有。Shure MV5 以 MacBook Pro 作主机，但录的是外接 Shure，不是 Mac 内置麦；按设备分目录可筛【官方文档】【论文】
- **干净参考**：有（LibriSpeech 原音，回放重录，需自行对齐）【推断】
- **转写**：有（LibriSpeech 文本）
- **语种**：英语（en-us 为真人原音回放；en-in、en-gb 为 Google WaveNet 合成后回放）
- **语体**：朗读（有声书）；Laughter 是噪声条件，不是说话人在笑
- **获取方式**：直接下载（README 给分区下载：en-us / en-in / en-gb / noise）
- **与 D62 的关系**：回放重录 + 噪声后加（噪声是在各设备上真实录的，再与语音数字相加）。开发阶段若拿来评测：取 en-us 下 nexus6 子目录，看手机麦频响下的字错率变化；大小未查到
- **出处**：https://github.com/akhilmathurs/libriadapt（README）【官方文档】; https://arxiv.org/abs/2009.02814【论文】; https://venturebeat.com/technology/researchers-data-set-pinpoints-challenges-adapting-speech-recognition-models-to-new-hardware【论坛 / 博客，个人说法】

## 备注
- 和笔记本会议离得远：设备偏 IoT / 智能音箱；保留它只因为它是少数「同一句、多设备」且 CC BY 4.0 可训练的集。
