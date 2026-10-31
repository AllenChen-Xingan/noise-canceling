### VOiCES（2018，SRI / IQT Lab41）
- **做什么**：LibriSpeech 在家具真实房间用旋转平台上的扬声器重放，另有扬声器同时放干扰，12 麦远场重录。
- **录音设备与距离**：7 只 SHURE SM58（心形动圈）、4 只 AKG 417L（全向电容领夹）、1 只 SHURE SM11（全向动圈领夹）；**领夹麦是摆在房间里的，不是佩戴在说话人身上的近讲**；成对放在扬声器后、前方桌面、更远桌面、对角，其余部分遮挡；距离约 38–228 inch（约 1–5.8 m）。无笔记本麦。
- **环境**：论文 release 1 为 2 个房间（§2.2），项目页称 3 个房间；空调与窗外噪声真实。TV/音乐由 1 只扬声器、babble 由 3 只扬声器回放。
- **说话人数与重叠**：前景单说话人 + 回放 babble（重叠为回放）。
- **语言**：英语。
- **参考**：干净源 有 · 转写 有 · 说话人标签 有。
- **大小**：约 15 h 源×12 麦；总 GB 未查到。
- **许可**：CC BY 4.0（项目页：free for commercial, academic, and government use）· 结论：可训练。
- **获取**：直接下载（AWS 公共桶）· https://registry.opendata.aws/lab41-sri-voices/ ；https://iqtlabs.github.io/voices/
- **接近度**：2.5/5。
- **用途建议**：真实回归集；多距离 SNR/混响校准；训练。
- **出处**：https://registry.opendata.aws/lab41-sri-voices/ 【官方】；https://iqtlabs.github.io/voices/ 【官方】；arXiv:1804.05053 §2.1–2.2、表 1 【论文】

<details><summary>另一路的卡片（voices-lab41-sri.md，怀疑派已修正；与上文冲突处见 ../decision.md 附录「卡片冲突裁定」）</summary>

#### VOiCES（2018，SRI International + IQT Lab41）
- **做什么**：真实房间里音箱回放重录的远场语料（属「部分」真实）
- **录音设备与距离**：12 支麦——SHURE SM58 动圈、AKG 417L 领夹、SHURE SM11；距前景音箱约 0.7–5.8 m，含遮挡位；电动转台模拟转头
- **环境**：真实房间；干扰为电视、音乐、babble（MUSAN 会议人声多音箱回放）
- **说话人数与重叠**：前景 1 人（LibriSpeech clean 300 人）+ 回放的 babble
- **语言**：英语
- **参考**：近讲 有（与录音同步的源音频）· 转写 有 · 说话人标签 有
- **大小**：1440 h
- **许可**：CC BY 4.0（AWS Open Data Registry）→ 可训练（源语料 LibriSpeech 亦 CC BY 4.0）
- **获取**：直接下载，`aws s3 ls --no-sign-request s3://lab41openaudiocorpus/`（us-east-1，免账号）
- **接近度**：2.5/5 · 不是笔记本，babble 是回放的；但有干净参考和精确距离
- **用途建议**：按距离和 babble 档位校准模拟器；有参考回归；可进训练
- **出处**：arXiv:1804.05053 §2.1–2.2、表 1–2【论文，未独立复核】；iqtlabs.github.io/voices/downloads/【官方】；registry.opendata.aws/lab41-sri-voices/【官方】

</details>

<details><summary>另一路的卡片（voices.md，怀疑派已修正；与上文冲突处见 ../decision.md 附录「卡片冲突裁定」）</summary>

#### VOiCES（2018，SRI + IQT Lab41）
- 用扬声器回放 LibriSpeech，并同时回放电视、音乐或 babble 干扰，在真实房间里多麦重录。
- **许可**：CC BY 4.0 · 结论：可训练。
- **获取**：AWS Open Data https://registry.opendata.aws/lab41-sri-voices/
- **接近度**：1.5/5。
- **出处**：https://registry.opendata.aws/lab41-sri-voices/ 【官方】· arXiv 1804.05053 【论文】

</details>
