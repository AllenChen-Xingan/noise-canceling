# KeSpeech

- **名称**：KeSpeech: An Open Source Speech Dataset of Mandarin and Its Eight Subdialects（KeSpeech）
- **链接**：https://github.com/KeSpeech/KeSpeech
- **许可**：KeSpeech 自定义协议（dataset_license.md：不得商用、不得向第三方分发、不得改编）→ 结论：只评测【官方文档】
- **大小**：1,542 h · 27,237 人 · 34 个城市；GB 未查到【论文 摘要】
- **场景键**：other（众包，各人所在环境不一；第二期每条带「clean / noisy」人工标签）
- **设备键**：non-laptop-phone
- **笔记本段**：没有。「recorded with mobile phones from many different brands」；有无逐条手机型号标签未查到；第二期有 `phase2.utt2env`（clean / noisy）环境标签【论文 §1、§3.3】
- **干净参考**：没有
- **转写**：有（句级文本；无时间戳【推断：朗读句子，按句切分】）
- **语种**：普通话 + 8 种次方言（官话分支）
- **语体**：朗读（按给定文本录，部分文本普通话与方言平行录）【论文】；无笑声、附和；情绪语音未查到
- **获取方式**：直接下载（百度网盘链接 + 提取码；下载即视为同意协议，无审批）【官方文档】
- **与 D62 的关系**：真实录音（真人对手机朗读，非回放、非合成）。若开发阶段拿来评测，可取第二期 noisy 标签的普通话段做「手机 + 真实底噪」的吃字检查素材；体积未查到
- **出处**：https://github.com/KeSpeech/KeSpeech【官方文档】; https://github.com/KeSpeech/KeSpeech/blob/master/dataset_license.md【官方文档】; https://datasets-benchmarks-proceedings.neurips.cc/paper/2021/file/0336dcbab05b9d5ad24f4333c7658a0e-Paper-round2.pdf §1、§3.3【论文】

## 备注
- 贝壳找房经纪人众包录制；手机品牌多、环境多，是中文「真实手机 + 真实底噪」的少见来源，但都是朗读，不是会议。
- 论文摘要写「free for all academic usage」，协议正文写禁止商用 / 分发 / 改编，二者不冲突。
- 怀疑派复核：dataset_license.md 原文（不得商用、不得分发、不得改编）、百度网盘 + 提取码「下载即同意」、论文 1,542 h / 27,237 人 / 34 城、手机多品牌、phase2.utt2env（clean / noisy 人工标注）、众包「reading」逐条核对一致。无改动。
