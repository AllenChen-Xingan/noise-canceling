# 来源登记 · 014 真实录音公开数据集

> 2026-10-31。**两类来源，同一个问题：它今天还算数吗？**

## 一、自有积累

| 来源 | 用到什么 | 还算数吗 |
|---|---|---|
| `docs/research/013_rl-environment/decision.md` §1.1、§1.3、§4.2 | 真实录音只做回归、永不进训练；sim-to-real 四步；辅助小模型要真实验收集 | 算数（2026-10-31 合并） |
| `docs/research/012_eval-standards/decision.md`、`verifiers.md` §3.4 | V-EAT-ASR 要转写；模拟真值指标到真实要量落差；感知类全 L0 | 算数；§3.4 当时尚未进 main（见 013 现状页） |
| `docs/plan/001_decisions-20261029-b1.md` D19 / D22 / D26 / D42 / D47 | 隐私、存放、许可、攻坚场景 | 算数 |
| 013 `sources.md` 第三节 | 干净语料（LibriSpeech、AISHELL-1、MLS）与噪声（DEMAND、MUSAN）的许可 | 算数；本线不重复 |

## 二、他人积累

全部是网页 / 论文 / LICENSE / 数据卡，**没有克隆仓库、没有下载数据本体**。逐条出处在各卡末尾，证据等级【官方】【论文】【二手】【推算】。

| 族 | 卡片 | 主要一手来源 | 还算数吗 |
|---|---|---|---|
| 会议 | ami · icsi · alimeeting · aishell-4 · notsofar-1 · libricss | groups.inf.ed.ac.uk/ami、openslr.org/119 /111、github.com/microsoft/NOTSOFAR1-Challenge、arXiv 2401.08887 / 2110.07393 / 2104.03603 | 算数；NOTSOFAR Dev-set-2 已下架 |
| CHiME 与可穿戴 | chime5-chime6 · chime7-chime8-dasr · chime9-echi · dipco · mixer6 · misp2021 · misp-meeting · easycom · mmcsg · mcorec | chimechallenge.org、openslr.org/150 LICENSE.txt、HF CHiME9-ECHI 数据卡、zenodo 8122551、github.com/facebookresearch/EasyComDataset | 算数；CHiME-5/6 许可 2024-01-01 改为 CC BY-SA 4.0；ECHI eval 未发放；ECHI-2 计划 2027-02 公开 |
| 降噪真实测试集 | dns1–dns5 · dns-challenge-blind-real · ms-aec-challenge-real · urgent2024/2025/2026 · real-m · chime7-udase · clarity-cec3-task2 | github.com/microsoft/DNS-Challenge（各届分支 README）、arXiv 2005.13981 / 2009.06122 / 2101.01902 / 2202.13288 / 2303.11510 / 2110.10812 | 算数；DNS 盲测转写未公开 |
| 咖啡馆 / 设备麦搜罗 | chime3-chime4-real · aspire-stirling · arte · common-voice · aishell-5 · libri-adhoc40 · wham-noise | chimechallenge.org/challenges/chime4、catalog.ldc.upenn.edu/LDC2017S24、zenodo 4585619 / 3386569 | 算数；Common Voice 托管已迁 mozilladatacollective.com，条款页本轮没能复核 |
| 设备链与 RIR | dr-vctk · ace-challenge-corpus · daps · voices-corpus · but-reverbdb · aachen-air · mirage · mit-ir-survey · voicehome-2 · dirha-english-wsj · dechorate · ears-reverb · openair | datashare.ed.ac.uk/handle/10283/3038 README.txt、zenodo 6257551、Imperial ACE 项目页 | 算数；ACE 两处官方许可冲突；OpenAIR 证书过期、AIR 官方页拒连 |
| 主线补核 | realman · nisqa-livetalk | github.com/Audio-WestlakeU/RealMAN、arXiv 2406.19959、NISQA wiki | 算数 |

**怀疑派提到、本轮没有建卡的**：MC-WSJ-AV / REVERB 真实部分（LDC2014S03，付费）· ConferencingSpeech 2021 · Mixer 6 CHiME-8 转写版（LDC2025S07）· Ego4D 对话子集 · MCAS（Aria 设备响应）· TAU / DCASE 场景录音 · Bern Cocktail Party · Libri-Adapt（噪声人工叠加，不满足命门 1）· OpenSLR SLR28。判断：都不改变首选组合（要么付费 / 签协议、要么不是嘈杂多人或设备麦），留作下次补卡。
