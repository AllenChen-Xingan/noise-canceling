# Speech Commands v2（Google 众包关键词语料）

- **名称**：Speech Commands: A Dataset for Limited-Vocabulary Speech Recognition（Speech Commands v0.02）
- **链接**：https://arxiv.org/abs/1804.03209 ; http://download.tensorflow.org/data/speech_commands_v0.02.tar.gz
- **许可**：论文原句「released under the Creative Commons BY 4.0 license」→ 结论：训练可【论文 §2】
- **大小**：105,829 条、2,618 人，每条 1 s（约 29 h【推断：105,829 × 1 s】）；解压约 3.8 GB、gzip 包 2.7 GB（论文）；v0.02 包实测 2,428,923,189 字节【论文 §6】【实测：HTTP 头 Content-Length】
- **场景键**：home, other（论文原句「wherever users happened to be」，地点未标）
- **设备键**：laptop-other, desktop, phone, unknown（经网页 WebAudio 录：桌面浏览器 Firefox / Chrome 与安卓手机；iOS 当时不支持）
- **笔记本段**：有，但不能筛。论文原句「utterances were captured through phone or laptop microphones」；文件名经哈希，无设备字段；是否含 Mac 内置麦未查到（桌面浏览器可在 Mac 上跑，但论文没写比例）【论文 §3、§5.2】【推断：Mac 段存在与否】
- **干净参考**：没有（真实录音，无平行干净版）
- **转写**：有（每条只有一个词，标签即转写；无时间戳，已切成 1 s）【论文 §6】
- **语种**：英语
- **语体**：孤立词朗读（35 个词，如 Yes / No / Stop / Go）；无对话、无笑声 / 附和 / 情绪
- **获取方式**：直接下载（TensorFlow 公开桶，tar.gz）
- **与 D62 的关系**：真实录音（众包真人在自己电脑或安卓手机上录，带各自房间的底噪与设备频响）。开发阶段若拿来评测：只能当「真实消费设备麦的底噪 / 频响」旁证或关键词识别回归；1 s 孤立词不能测会议语句的吃字；全包 2.4 GB
- **出处**：https://arxiv.org/abs/1804.03209 §2、§3、§5、§6【论文】; http://download.tensorflow.org/data/speech_commands_v0.02.tar.gz（HTTP 头）【实测】

## 备注
- 找漏补卡：这是少数「笔记本内置麦真人录音 + CC BY 4.0 可训练」的公开集，但没有设备标签、每条只有 1 s，和会议语句差得远【推断】。
- 另有 `_background_noise_` 目录（论文 §5.6 后），含几段真实录的环境噪声与生成的白 / 粉噪声，量很小【论文】。
- 怀疑派复核：本卡由怀疑派新写（原调研员未列此集）；数字与许可均出自论文原文。
