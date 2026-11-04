# AISHELL-DMASH（Distributed Microphone Arrays in Smart Home）/ FFSVC 2020 挑战数据

- **名称**：Distributed Microphone Arrays in Smart Home dataset（AISHELL-DMASH）；其子集 FFSVC 2020 Challenge Dataset 及 FFSVC2020-supplement（F / S / T 三次到访）
- **链接**：https://ffsvc.github.io/dataset
- **许可**：未见标准许可；Zenodo 申请条件原文含「I'll not modify, transfer, redistribute or sell the challenge data without the Beijing Shell Shell Technology」，且未经授权不得商用 → 结论：只评测【官方文档】
- **大小**：未查到总时长与 GB；FFSVC 2020 训练 120 人、开发 35 人、每个任务评测 80 人；每人每次到访 300+ 句【论文 §2】（第三方 hyper.ai 称 511 人、30,000 h，数字可疑，未采信【论坛 / 博客，个人说法】）
- **场景键**：home-family（两间真实智能家居房间；第一次到访有电扇 + 电视或办公室底噪，第二次安静，第三次只有电扇）
- **设备键**：phone, tablet, mic-array, close-talk, unknown（每组里「one microphone」型号未写，原卡记作 studio 无出处）
- **笔记本段**：没有笔记本。每个房间 7 个位置各放一组设备：iPhone、安卓手机、iPad、一支麦克风、5 cm 半径圆阵；另有 1 支近讲麦。挑战集只取近讲麦 + 25 cm 处 iPhone + 3 个圆阵；supplement 补了手机与平板通道。通道 / 位置可由文件命名区分【论文 §2.1–2.2；官方文档】
- **干净参考**：有（近讲麦与各设备同步并行录）
- **转写**：部分（每次到访前 30 句为固定文本「你好米雅」，接下来 60 句为「你好米雅」+ 随机文本，其余为文本无关句【论文 §2.1】；是否随包发转写未查到）
- **语种**：普通话
- **语体**：朗读（唤醒词、唤醒词 + 随机文本、文本无关句）；无笑声、附和、情绪
- **获取方式**：签协议（发邮件向组织方申请 FFSVC2020；supplement 需先在挑战提交平台注册、再在 Zenodo 用同一邮箱申请）——本路未申请
- **与 D62 的关系**：真实录音（真人在真实房间，手机 / 平板 / 阵列 / 近讲同步录）。若开发阶段拿来评测，可用「近讲 vs 手机 / 平板」的同步对做有参考的设备通道评测；体积未查到
- **出处**：https://arxiv.org/abs/2005.08046 §2.1–2.2【论文】; https://ffsvc.github.io/dataset【官方文档】; https://zenodo.org/records/6462609【官方文档】; https://hyper.ai/datasets/29380【论坛 / 博客，个人说法】

## 备注
- 中文公开集里唯一找到的「手机 + 平板 + 近讲同步、带真实家用噪声（电扇、电视）」组合；仍无笑声对话、无笔记本。
- 申请对象是杜克昆山大学组织方邮箱；数据权利属北京希尔贝壳。
- 怀疑派复核：FFSVC 官网（邮件申请）、Zenodo 6462609（无许可字段，申请须承诺不改不分发不卖）、论文 §2.1–2.2 设备组与三次到访噪声核对一致。改动：转写字段「前 90 句为固定唤醒词」与论文不符，更正为 30 句固定 + 60 句半固定；设备键 studio 无出处改 unknown。Zenodo 页「Data volume 27.6 TB」是平台统计口径，不是数据集大小，不采信。
