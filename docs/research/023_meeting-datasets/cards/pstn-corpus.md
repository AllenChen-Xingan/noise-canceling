# PSTN Corpus（Microsoft / TU Berlin 公网电话语音质量语料）

- **名称**：PSTN Corpus（DNN No-Reference PSTN Speech Quality Prediction 数据集）
- **链接**：https://github.com/ConferencingSpeech/ConferencingSpeech2022/tree/main/Training/Dev%20datasets
- **许可**：未查到（论文说「open-source」「made publicly available」，但没有写许可名；挑战仓库目录页只要求引用论文）→ 结论：未查到【论文】【官方文档】
- **大小**：论文：训练 + 验证共 79,980 条 × 10 s（49,984 条基于带噪参考、29,996 条基于干净参考；训练 66,635 条，其余为验证；80 家运营商、50 余国）；真实测试集 1,040 条语音信箱录音【论文 §1–§2】。挑战仓库公开的「部分训练集」58,709 条 × 10 s（约 163 h【推断】），`train.zip` 7,939,820,986 字节（约 7.9 GB，HTTP 响应头 Content-Length）【官方文档】
- **场景键**：other（电话网络传输；训练段背景噪声是混进去的）
- **设备键**：phone, unknown（训练段为网关自动呼叫，无终端设备；测试段为众包者的固话或手机）
- **笔记本段**：没有。训练段是 PSTN 与 VoIP 端点之间自动通话，论文原话「do not contain the terminal device」；测试段是众包用户用「landline or mobile phone」打进 SkypeIn 语音信箱，逐条设备未标【论文 §2.1–§2.2】
- **干净参考**：部分（训练段由 LibriVox 片段经电话网传输，论文用参考算过 POLQA；挑战仓库原句：公开的 58,709 条中「40,739 based on noisy references and 17,970 based on clean references」；包内是否附参考文件未查到；测试段没有）【论文】【官方文档】
- **转写**：没有（未提）
- **语种**：英语（LibriVox 有声书）【论文】
- **语体**：朗读（训练段为有声书；测试段为众包者朗读 Graz 大学音高库里随机 10 句留语音信箱）；无笑声 / 附和标注【论文 §2.2】
- **获取方式**：直接下载（部分训练集：https://challenge.blob.core.windows.net/pstn/train.zip ；测试集下载位置未查到）
- **与 D62 的关系**：训练段是「干净有声书 + 混入 DNS 噪声（0–40 dB SNR）→ 经真实电话网传输」，即真实链路上的合成输入；测试段是真实来电。人评分：P.808 众包，训练每条 5 分、测试每条 30 分。若开发阶段拿来评测：只适合校准无参考音质分在「窄带电话链路」上的表现，与笔记本麦降噪离得远；用 5% 盲测部分约 2,900 条【推断】
- **出处**：https://arxiv.org/abs/2007.14598 §1、§2、§4.5、Table 4【论文】; https://arxiv.org/abs/2203.16032 §4.4【论文】; https://github.com/ConferencingSpeech/ConferencingSpeech2022【官方文档】

## 备注
- 论文 §4.5：作者模型在真实测试集上 PCC 只有 0.63（分数集中在 MOS 3.5–4）；均匀重采样后 0.84；ITU P.563 只有 0.25（均匀子集 0.49）。说明**真实录音上的无参考分比合成验证集低一截**【论文】。
- 挑战盲测用了 PSTN 的 5%，每条约 24 分（挑战论文 Table 3）【论文】。
- 怀疑派复核：改了 4 处——「大小」补 79,980 的带噪 / 干净参考拆分与 train.zip 约 7.9 GB；「干净参考」里 40,739 / 17,970 的出处从「检索摘要」改为挑战仓库原句【官方文档】；「笔记本段 / 设备键」补论文原话（测试段为固话或手机）；「语体」测试段内容从「未写」改为朗读 Graz 句子。许可「未查到」核对无误。
