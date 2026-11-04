# WenetSpeech-Yue（粤语）

- **名称**：WenetSpeech-Yue: A Large-scale Cantonese Speech Corpus with Multi-dimensional Annotation（WenetSpeech-Yue）
- **链接**：https://huggingface.co/datasets/ASLP-lab/WenetSpeech-Yue
- **许可**：HF 数据卡 license 字段「cc-by-nc-4.0」（GitHub 仓库 Apache-2.0 只管代码）→ 结论：只评测【官方文档】
- **大小**：约 21,800 h；HF 仓库 11.3 GB（只含元数据，原音频经 `link` 指向原视频）；评测集 WSYue-ASR-eval Short（0–10 s）2861 / 9.46 h、Long（10–30 s）838 / 1.97 h（GitHub README 表头把 2861、838 这一列写作「Speakers」，按量级更像条数【推断】）【官方文档】
- **场景键**：other（10 个领域：讲故事、娱乐、剧、文化、vlog、解说、教育、播客、新闻、其他）
- **设备键**：unknown
- **笔记本段**：未查到（网络视频来源，无设备标签）
- **干净参考**：没有
- **转写**：有（自动 + 置信度；元数据含字级时间戳、SNR、DNSMOS；评测集多轮人工标注）
- **语种**：粤语（含粤英混说）
- **语体**：多领域网络语音；评测集有情绪、年龄、性别标签（GitHub README：「multi-round manual annotations including text transcripts, emotion, age, and gender labels」；HF 数据卡未提情绪）【官方文档】；笑声 / 附和未查到
- **获取方式**：直接下载元数据；预处理音频需联系作者（邮件 / 微信群）
- **与 D62 的关系**：真实录音（网络媒体二次采集）。开发阶段若用，可取 WSYue-ASR-eval 约 11.4 h 做粤语吃字检查；音频需另行申请
- **出处**：https://huggingface.co/datasets/ASLP-lab/WenetSpeech-Yue【官方文档】; https://github.com/ASLP-lab/WenetSpeech-Yue【官方文档】; https://arxiv.org/abs/2509.03959【论文，只读摘要】

## 备注
- 粤语集无会议、无设备信息；只为「粤语等」一项作记录。
- 怀疑派复核：HF 卡 cc-by-nc-4.0、约 21,800 h、11.3 GB、音频需联系作者核对一致。改动：评测集「列名未标」不对，README 表头是「Speakers」，已写明并标推断；情绪标签补原文出处（只在 GitHub README）。
