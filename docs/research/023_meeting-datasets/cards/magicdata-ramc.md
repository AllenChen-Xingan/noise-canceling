# MAGICDATA Mandarin Chinese Conversational Speech Corpus（MagicData-RAMC）

- **名称**：MAGICDATA Mandarin Chinese Conversational Speech Corpus（MagicData-RAMC，OpenSLR SLR123）
- **链接**：https://www.openslr.org/123/
- **许可**：Attribution-NonCommercial-NoDerivatives 4.0 International Public License (CC BY-NC-ND 4.0) → 结论：只评测【官方文档】
- **大小**：180 h · 351 场对话 · 219,325 段 · 663 人；train 149.65 h / dev 9.89 h / test 20.64 h；MagicData-RAMC.tar.gz 15 GB【官方文档 · 论文 表 2】
- **场景键**：home
- **设备键**：phone
- **笔记本段**：没有。全部用手机录（安卓与 iPhone 约 1:1）；论文与官网都没说有逐条设备标签，能否按安卓 / iPhone 筛出来未查到【论文 §2.1】
- **干净参考**：没有（只有手机通道本身；录音环境安静，底噪 < 40 dBA，可近似当作「较干净的手机近讲」，但不是参考信号）
- **转写**：有（人工转写 + 专人校对；每段有说话人起止时间戳，精确到毫秒级）【论文 §2.1、§2.4】
- **语种**：普通话（南北口音约各半：北方 334 人、南方 329 人）
- **语体**：两人自由对话（15 个话题领域，每场平均 30.80 min）；笑声、音乐等非语言段用专门符号标注；口语、半截词、重复等不流利现象也标了；附和（嗯、对）是否单独标未查到；情绪语音未查到【论文 §2.1】
- **获取方式**：直接下载（OpenSLR 及欧洲、国内镜像，页面未要求注册；MagicHub 官方页另有入口，未打开）
- **与 D62 的关系**：真实录音（真人对话，手机录，非回放、非合成）。若开发阶段拿来评测，可用 test 20.64 h（43 场）或 dev 9.89 h（19 场）做「语音识别吃字检查」的中文对话素材；下载需整包 15 GB（单包，不能只下 test）【推断】
- **出处**：https://www.openslr.org/123/【官方文档】; https://arxiv.org/abs/2203.16844 §2.1–2.4、表 2【论文】

## 备注
- 录音环境：家里的小房间（< 20 m²），RT60 < 0.4 s，底噪 < 40 dBA【论文 §2.1】。所以它本身几乎无噪，不是「带噪真实会议」，更像「干净的手机对话」，可作为叠加噪声前的中文对话底料（但许可 ND，禁止改编分发，只能本机评测）【推断】。
- 两人是否各自一部手机、音频是单声道混合还是分通道：论文未写，未查到。
- 怀疑派复核点：许可只在 OpenSLR 页读到；MagicHub 页面条款未读。
