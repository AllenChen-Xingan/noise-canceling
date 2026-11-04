# CN-Celeb（CN-Celeb1 / CN-Celeb2）

- **名称**：CN-Celeb: multi-genre speaker recognition（CN-Celeb，OpenSLR SLR82）
- **链接**：https://www.openslr.org/82/
- **许可**：Attribution-ShareAlike 4.0 International（CC BY-SA 4.0）→ 结论：只评测【官方文档】
- **大小**：3,000 人（CN-Celeb1 1,000 人、约 270 h、13 万+ 句；CN-Celeb2 2,000 人、52 万+ 句）；cn-celeb_v2.tar.gz 22 GB + CN-Celeb2 三包 26 / 26 / 23 GB【官方文档 · 论文 §1、§3】
- **场景键**：other（网络媒体：11 种体裁，含访谈、vlog、演讲、朗诵、唱歌、直播、影视等）
- **设备键**：non-laptop-unknown
- **笔记本段**：未查到（音频取自 B 站等媒体，录音设备未知，无设备标签；按体裁有标签）
- **干净参考**：没有
- **转写**：没有（说话人识别集，只有说话人与体裁标签）
- **语种**：普通话为主（名人语音）
- **语体**：多体裁「野外」语音：对话 / 访谈 / vlog / 直播等，含笑声与情绪语音【推断：体裁决定，无标注】；附和未标
- **获取方式**：直接下载（OpenSLR 欧洲、国内镜像）
- **与 D62 的关系**：真实录音（网络媒体二次采集，非回放重录、非合成，但经平台编码）。开发阶段若用，只能作说话人相似度类检查的中文素材；无转写，不能做吃字检查；最小一包 22 GB
- **出处**：https://www.openslr.org/82/【官方文档】; https://arxiv.org/abs/2012.12468 §1、§3【论文】

## 备注
- 体裁列表中「vlog / 直播 / 访谈」里可能有笔记本或手机录的段，但无标签无法筛，属【推断】。
- 怀疑派复核：OpenSLR 页许可（CC BY-SA 4.0）、四个包大小、13 万 / 52 万句核对一致；论文确认 CN-Celeb1 1,000 人 270 h、数据取自 Bilibili 等媒体（CN-Celeb2 另有网易云、唱吧、喜马拉雅等）。无改动。
