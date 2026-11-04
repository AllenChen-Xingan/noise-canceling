# 3D-Speaker（多设备 · 多距离 · 多方言说话人语料，阿里达摩院）

- **名称**：3D-Speaker: A Large-Scale Multi-Device, Multi-Distance, and Multi-Dialect Corpus（3D-Speaker）
- **链接**：https://3dspeaker.github.io/
- **许可**：官网原句「The provided 3D-Speaker metadata is available to download under a Creative Commons Attribution-ShareAlike 4.0 International License (CC BY-SA 4.0)」——字面只说 metadata，音频本身未另写 → 结论：未查到（音频本身的许可原文没找到；若音频也是元数据那样的 CC BY-SA 4.0，按 D64 可训练）【官方文档】
- **大小**：训练 10,000 人、579,013 条、有效语音 1,124 h；评测 240 人、18,782 条、15.42 h；train 包约 190 GB、test 包约 1.1 GB【论文 §3、表 5】【官方文档】
- **场景键**：other（录音房间类型未写）
- **设备键**：laptop-unknown, non-laptop-phone, non-laptop-tablet, non-laptop-mic-array, non-laptop-close-talk, non-laptop-unknown（iPad、安卓手机、iPhone、8 麦阵、PC 笔记本、录音笔、指向麦、未指明手机）
- **笔记本段**：有，且能筛。**训练集**里 PC 笔记本 96,262 条（16.63%，表 2 只统计训练集）；评测集里笔记本有多少条未查到；官网「Device labels of all utterances are provided」；笔记本都放在离说话人 1 m 以内。笔记本品牌型号未写（未查到是否含 Mac）【论文表 2、§3、§3.2】【官方文档】
- **干净参考**：部分（同一句由多台设备同时录，近讲指向麦 ≤ 0.3 m 可作近似参考；不是录音棚级干净）【论文 §3.1–§3.2】【推断】
- **转写**：部分（只有普通话语句给转写，方言语句没有；时间戳未查到）【官方文档】
- **语种**：普通话 + 13 种方言（训练集表 4 列 13 种、共 1,074 人 77.2 h；§3 正文另写 1,200 人，两处不一致；评测集 11 种）【论文 §3、§3.3、表 4、表 5】
- **语体**：未查到（论文未写朗读还是自由说）；无笑声、附和、情绪标注【论文】
- **获取方式**：直接下载（阿里云 OSS：`https://speech-lab-share-data.oss-cn-shanghai.aliyuncs.com/3D-Speaker/test.tar.gz` 等，含 MD5）
- **与 D62 的关系**：真实录音（真人，多设备同步录）。开发阶段若拿来评测：取 test 包（约 1.1 GB，15.42 h）里设备标签为 PC 的普通话语句（条数未查到；若 test 里 PC 太少，只能从 190 GB 的 train 包里挑），做「真实笔记本麦 + 普通话 + 有转写」的字错率回归；同一句的手机 / 近讲版本可作对照
- **出处**：https://arxiv.org/abs/2306.15354 §3、§3.1–§3.4、表 2–5【论文】; https://3dspeaker.github.io/（License、Download、FAQ）【官方文档】

## 备注
- 本路查到的公开集里，**笔记本麦条数最多、且逐条带设备标签**的一份；但噪声环境未写，可能偏安静【推断】，不能替代带噪会议录音。
- 中文路（r4）若另开同名卡，以先写者为准，后写者追加补充节。
- 怀疑派复核：重读官网许可句（字面只说 metadata）与论文 §3、表 2–5。改 4 处：①许可结论「只评测」→「未查到」（音频许可原文不存在，按规则不能填）；②「14 种方言」→ 表 4 实为 13 种；③96,262 条笔记本是训练集数字，评测集条数未查到，D62 一栏同步注明；④补评测集 15.42 h。
