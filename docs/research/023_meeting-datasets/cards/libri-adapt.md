# Libri-Adapt（六种麦克风重录的 LibriSpeech）

- **名称**：Libri-Adapt: A New Speech Dataset for Unsupervised Domain Adaptation（Libri-Adapt，ICASSP 2020）
- **链接**：https://github.com/akhilmathurs/libriadapt
- **许可**：README 原句「The dataset is released under CC BY 4.0 license.」→ 结论：训练可【官方文档】；但 Rain / Wind / Laughter 三种噪声取自 ESC-50 再回放重录，ESC-50 仓库原句「available under the terms of the Creative Commons Attribution Non-Commercial license」（CC BY-NC 3.0），含噪声的三档是否受 NC 约束未查到，稳妥只把 Clean 档算训练可【官方文档】【推断】
- **大小**：72 个域（6 麦 × 3 口音 × 4 噪声档），每个域训练 28,540 条 / 100.6 h + 测试 2,600 条 / 5.4 h，合计 7,200 h【论文 §3、摘要】；下载包 en-us 54.4 GB（4×12 GB + 6.4 GB）、en-in 18.6 GB、en-gb 19.5 GB、noise 24 MB，合计约 92.5 GB【官方文档：README】【推断：加总】
- **场景键**：other（论文原句「in a quiet room」，监听音箱距麦 15 cm；噪声 Rain / Wind / Laughter 取自 ESC-50，在 6 支麦上回放重录后按 20 dB 信噪比与语音相加）
- **设备键**：non-laptop-phone（Google Nexus 6，3 通道）, non-laptop-mic-array（Matrix Voice、ReSpeaker，各 7 通道）, non-laptop-desktop（PlayStation Eye 4 通道、USB 麦）, non-laptop-studio（Shure MV5）
- **笔记本段**：没有。Shure MV5 以 MacBook Pro 作主机，但录的是外接 Shure，不是 Mac 内置麦；按设备分目录可筛【官方文档】【论文】
- **干净参考**：有（LibriSpeech 原音，回放重录，需自行对齐）【推断】
- **转写**：有（LibriSpeech 文本）
- **语种**：英语（en-us 为真人原音回放；en-in、en-gb 为 Google WaveNet 合成后回放）
- **语体**：朗读（有声书；en-in / en-gb 为 TTS 合成）；Laughter 是 ESC-50 笑声片段作噪声，不是说话人在笑
- **获取方式**：直接下载（README 给分区下载：en-us / en-in / en-gb / noise）
- **与 D62 的关系**：回放重录 + 噪声后加（语音与 ESC-50 噪声都是经 JBL LSR305 音箱回放、各麦重录，再数字相加，不是真实环境噪声）。开发阶段若拿来评测：取 en-us 测试集里 Nexus 6 的 Clean 档（每域 2,600 条 / 5.4 h），看手机麦频响下的字错率变化；单个子目录 GB 未查到
- **出处**：https://github.com/akhilmathurs/libriadapt（README）【官方文档】; https://arxiv.org/abs/2009.02814 摘要、§3、表 1–2【论文】; https://github.com/karolpiczak/ESC-50（许可）【官方文档】

## 备注
- 和笔记本会议离得远：设备偏 IoT / 智能音箱；保留它只因为它是少数「同一句、多设备」且 CC BY 4.0 可训练的集。
- 怀疑派复核：读论文 PDF 全文与 README。改 5 处：①7,200 h 论文摘要原文就有，去掉【推断】和博客出处；②GB 由「未查到」改为 README 分包大小（合计约 92.5 GB）；③录音在「quiet room」、距音箱 15 cm，补进场景键；④噪声来自 ESC-50 回放（原卡写「噪声是在各设备上真实录的」不准），ESC-50 为 CC BY-NC，许可加限定；⑤Shure MV5 以 MacBook Pro 为主机，论文 §3 原文核到，保留。
