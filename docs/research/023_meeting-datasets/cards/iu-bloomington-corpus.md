# IU Bloomington Corpus（COSINE + VOiCES 众包 MUSHRA 评分）

- **名称**：IU Bloomington Corpus（ConferencingSpeech 2022 附加语料）
- **链接**：https://drive.google.com/drive/folders/1wIgOqnKA1U-wZQrU8eb67yQyRVOK3SnZ
- **许可**：未查到（挑战仓库只要求引用；源 VOiCES 为 CC BY 4.0（014 卡），COSINE 许可本路未读）→ 结论：未查到【官方文档】
- **大小**：36,000 条（COSINE 18,000 + VOiCES 18,000），每条 3–6 s，约 45 h；GB 未查到【论文 §4.3】
- **场景键**：other（COSINE：多人在嘈杂环境中自然对话；VOiCES：房间回放重录）
- **设备键**：close-talk, unknown（COSINE 胸前 / 肩部麦；VOiCES 为研究用多麦）
- **笔记本段**：没有【论文】
- **干净参考**：有（COSINE 给近讲麦作参考，VOiCES 给前景语音作参考；另附低通「锚点」版本）【论文】
- **转写**：未查到
- **语种**：英语【推断：COSINE、VOiCES 均为英语】
- **语体**：COSINE 为多人自然对话（附和、笑声是否标注未查到）；VOiCES 为朗读回放【推断】
- **获取方式**：直接下载（Google Drive 文件夹，含音频、评分与读取示例）
- **与 D62 的关系**：COSINE 段是真实对话录音（胸 / 肩麦为带噪、近讲麦为参考，SNR 约 −10.1 ~ 11.4 dB）；VOiCES 段是回放重录。评分为 ITU-R BS.1534（MUSHRA）0–100 分，不是 1–5 MOS。若开发阶段拿来评测：COSINE 18,000 条可同时算全参考与无参考分，看两者在真实对话上是否一致；大小未查到
- **出处**：https://arxiv.org/abs/2203.16032 §4.3、§4.5【论文】; https://github.com/ConferencingSpeech/ConferencingSpeech2022/tree/main/Training/Dev%20datasets【官方文档】

## 备注
- 挑战方明说它不进盲测，因为量表不同（0–100）【论文】。
- 价值在于「真实对话 + 近讲参考 + 人评分」三样同时有，可以检验无参考 verifier 与全参考 verifier 在同一批真实片段上排序是否一致。
- 怀疑派待核：COSINE 许可原文、是否附转写。
