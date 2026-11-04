# QUT-NOISE（及 QUT-NOISE-TIMIT）

- **名称**：QUT-NOISE database and protocols（QUT-NOISE）
- **链接**：https://github.com/qutsaivt/QUT-NOISE
- **许可**：README 原句「The QUT-NOISE data is licensed CC-BY-SA」；生成 QUT-NOISE-TIMIT 的代码为 BSD → 结论：训练可（按 D64：SA 可训练，训出的权重按相同条款共享并署名）【官方文档】
- **大小**：20 段噪声会话，每段 ≥30 min，合计 >10 h；zip 合计约 7.7 GB（cafe 1.6 / car 1.7 / home 1.4 / reverb 1.4 / street 1.6 GB）【论文】【官方文档】
- **场景键**：cafe, home-family, street-outdoor, car-passenger, other（室内泳池、半封闭停车场，用于混响）
- **设备键**：non-laptop-studio
- **笔记本段**：没有。全部用 Zoom H2 手持立体声录音笔（后置麦对），48 kHz / 16 bit【论文 §2.2】
- **干净参考**：没有（噪声本身无参考；QUT-NOISE-TIMIT 是把 TIMIT 干净语音混进去的合成集，那部分有参考）
- **转写**：没有（噪声段无转写；TIMIT 合成部分有 TIMIT 原转写）
- **语种**：背景人声为澳大利亚布里斯班当地（英语）【推断】
- **语体**：CAFE 段「中到高强度背景 babble + 厨房声」；HOME-LIVING 段「孩子唱歌、说话、玩耍 + 电视 / 音乐」；CAR 段明确「无收音机、无人声」【论文 §2.1】；笑声、附和未标注
- **获取方式**：直接下载（GitHub README 列出 6 个 zip）
- **与 D62 的关系**：真实录音（噪声）；QUT-NOISE-TIMIT 是合成。若开发阶段拿来评测：用 CAFE（户外咖啡馆 + 商场美食广场）、HOME-LIVINGB（孩子 + 电视）、CAR（开窗 / 关窗，高速 / 市区）、STREET-CITY 当混合噪声；每类 2 段 × ≥30 min，各约 1.4–1.7 GB【论文】【官方文档】
- **出处**：https://github.com/qutsaivt/QUT-NOISE【官方文档】; https://www.isca-archive.org/interspeech_2010/dean10_interspeech.pdf §2【论文】

## 备注
- 「家里有孩子和电视」这种声音，本路只在 QUT-NOISE HOME-LIVINGB 和 CHiME-Home 里找到成段真实录音。
- 每段会话标了坏段（麦故障）位置，可以避开。
- 怀疑派复核：无改动。已读 README（CC-BY-SA；代码 BSD 式）与 Interspeech 2010 论文 §2.1–2.2：20 段 ≥30 min、合计 >10 h、Zoom H2 后置麦对 48 kHz、CAFE / HOME-LIVINGB / CAR「no radio or speech noise」均属实；zip 大小与 README 一致。
- 主控复核（2026-11-03，codex 第 2 轮 #1）：许可结论按 D64 由「只评测」改为「训练可」。
