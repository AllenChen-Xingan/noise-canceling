# WenetSpeech（重点：TEST_MEETING 真实会议测试集）

- **名称**：WenetSpeech: A 10000+ Hours Multi-domain Mandarin Corpus（WenetSpeech，OpenSLR SLR121）
- **链接**：https://wenet-e2e.github.io/WenetSpeech
- **许可**：「available to download for non-commercial purposes under a Creative Commons Attribution 4.0 International License」（CC BY 4.0 + 仅非商业的附加限制；音频版权归原视频 / 音频所有者）→ 结论：只评测【官方文档】
- **大小**：总 22,435 h（强标注 10,005 h、弱标注 2,478 h、无标注 9,952 h）；DEV 20 h、TEST_NET 23 h、TEST_MEETING 15 h；HF 仓库总 1.02 TB【官方文档 · 论文 表 4】
- **场景键**：meeting-room（只 TEST_MEETING）, other（训练集与 TEST_NET 是 YouTube 与播客：有声书、解说、纪录片、剧、访谈、新闻、综艺等）
- **设备键**：non-laptop-unknown（TEST_MEETING 录音设备论文未写；训练集为网络视频 / 播客，设备不明）
- **笔记本段**：未查到。TEST_MEETING 只写「far-field」「sampled from 197 real meetings in a variety of rooms」，没有设备说明，无设备标签【论文 §3.4】
- **干净参考**：没有
- **转写**：有（TEST_MEETING 由专业标注员人工校对；训练集由字幕 OCR / 识别生成并带置信度）；段级时间戳有（论文 §3.1：元数据 JSON 为每段给出 timestamp、label、confidence）【论文】
- **语种**：普通话
- **语体**：TEST_MEETING 为远场、对话、自发的会议语音，话题含教育、房产、金融、家居、科技、访谈【论文 §3.4】；笑声 / 附和 / 情绪是否标注未查到
- **获取方式**：注册（填 Google 表单申请，邮件收密码后用脚本下载；HF 镜像要求同意共享联系方式）——本路未申请
- **与 D62 的关系**：TEST_MEETING 是真实录音（真实会议，远场）。若开发阶段拿来评测，只用 TEST_MEETING 15 h 做中文会议「语音识别吃字检查」底料；整套音频经 Opus 32 kbps 压缩【论文 §3.1】，有编码失真；TEST_MEETING 单独体积未查到（按 32 kbps 推算约 0.2 GB【推断】）
- **出处**：https://wenet-e2e.github.io/WenetSpeech【官方文档】; https://github.com/wenet-e2e/WenetSpeech【官方文档】; https://huggingface.co/datasets/wenet-e2e/wenetspeech【官方文档】; https://arxiv.org/abs/2110.03370 §3.1、§3.4、表 4【论文】

## 备注
- 和本项目最相关的是 TEST_MEETING：中文、真实会议、远场、自发——但设备未知、音频是 Opus 32 kbps 有损压缩，降噪评测时压缩失真会和降噪失真混在一起【推断】。
- 官网 wenet.org.cn 证书过期（2026-11-03 实测打不开），用 wenet-e2e.github.io 读到许可。
- 第三方汇总（huggingface.co/datasets/xuyaya/ASR-Testset）把 ws-meeting 当作中文会议识别测试集之一【论坛 / 博客，个人说法】。
- 怀疑派复核：官网许可原句、Google 表单申请流程、HF 门控与 1.02 TB 逐条读到；论文表 1（10005 / 2478 / 9952，合计 22435）、表 4（20 / 23 / 15 h）、§3.1 Opus 32 kbps、§3.4「197 real meetings in a variety of rooms」核对一致。改动：转写字段的时间戳出处改写为论文原文含义。
