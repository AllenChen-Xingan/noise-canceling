# PSTN Corpus（Microsoft / TU Berlin 公网电话语音质量语料）

- **名称**：PSTN Corpus（DNN No-Reference PSTN Speech Quality Prediction 数据集）
- **链接**：https://github.com/ConferencingSpeech/ConferencingSpeech2022/tree/main/Training/Dev%20datasets
- **许可**：未查到（论文说「open-source」「made publicly available」，但没有写许可名；挑战仓库目录页只要求引用论文）→ 结论：未查到【论文】【官方文档】
- **大小**：论文：训练 / 验证 79,980 条 × 10 s（自动拨打 50 余国、80 家运营商）；真实测试集 > 1,000 通众包真实来电。挑战仓库公开的「部分训练集」58,709 条 × 10 s（约 163 h【推断】）；GB 未查到【论文 §1–§2】【官方文档】
- **场景键**：other（电话网络传输；训练段背景噪声是混进去的）
- **设备键**：phone, unknown
- **笔记本段**：没有。训练段是 PSTN 与 VoIP 端点之间自动通话；测试段是众包用户打进 Skype 语音信箱，用户设备未标【论文】
- **干净参考**：部分（训练段由 LibriVox 干净片段经电话网传输，论文用参考算过 POLQA；后续文献称 40,739 条基于带噪参考、17,970 条基于干净参考；包内是否附参考文件未查到）【论文】【论坛 / 博客，个人说法：检索摘要】
- **转写**：没有（未提）
- **语种**：英语（LibriVox 有声书）【论文】
- **语体**：朗读（有声书）；测试段为众包真人来电，内容未写；无笑声 / 附和标注【论文】
- **获取方式**：直接下载（部分训练集：https://challenge.blob.core.windows.net/pstn/train.zip ；测试集下载位置未查到）
- **与 D62 的关系**：训练段是「干净有声书 + 混入 DNS 噪声（0–40 dB SNR）→ 经真实电话网传输」，即真实链路上的合成输入；测试段是真实来电。人评分：P.808 众包，训练每条 5 分、测试每条 30 分。若开发阶段拿来评测：只适合校准无参考音质分在「窄带电话链路」上的表现，与笔记本麦降噪离得远；用 5% 盲测部分约 2,900 条【推断】
- **出处**：https://arxiv.org/abs/2007.14598 §1、§2、§4.5、Table 4【论文】; https://arxiv.org/abs/2203.16032 §4.4【论文】; https://github.com/ConferencingSpeech/ConferencingSpeech2022【官方文档】

## 备注
- 论文 §4.5：作者模型在真实测试集上 PCC 只有 0.63（分数集中在 MOS 3.5–4）；均匀重采样后 0.84；ITU P.563 只有 0.25（均匀子集 0.49）。说明**真实录音上的无参考分比合成验证集低一截**【论文】。
- 挑战盲测用了 PSTN 的 5%，每条约 24 分（挑战论文 Table 3）【论文】。
