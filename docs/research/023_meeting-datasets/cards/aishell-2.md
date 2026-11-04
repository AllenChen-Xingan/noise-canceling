# AISHELL-2

- **名称**：AISHELL-2: Transforming Mandarin ASR Research Into Industrial Scale（AISHELL-2）
- **链接**：https://www.aishelltech.com/aishell_2
- **许可**：未查到许可原文（论文写「free for academic usage」；官网页面本轮只读到标题）→ 结论：未查到【论文】
- **大小**：1000 h（iOS 通道）· 1991 人；dev 2500 句（5 人）、test 5000 句（10 人）× 3 通道；GB 未查到【论文 §2】
- **场景键**：other（1347 人在录音棚，其余在有自然混响的客厅）
- **设备键**：phone, studio
- **笔记本段**：没有笔记本。训练集只有 iPhone 通道；dev / test 有 iOS、Android、Mic 三通道并行录音，按目录（dev_ios / dev_android / dev_mic 等）可筛设备【论文 §1–2、表 1】
- **干净参考**：部分（dev / test 的 Mic 通道与手机通道是同一句话并行录，可当参考；位置不同、未说明是否时间对齐）
- **转写**：有（句级，无时间戳）
- **语种**：普通话
- **语体**：朗读（含语音指令、地名、娱乐、金融、科技、体育、英文拼读、少量无主题自由说）；无笑声、附和、情绪
- **获取方式**：签协议（学术免费，申请流程本轮未读到原文，按常见做法为提交申请 / 协议【推断】）
- **与 D62 的关系**：真实录音（真人朗读）。若开发阶段拿来评测，可用 dev / test 的三通道平行段（7500 句 × 3）比较同一句在手机 / 麦上的吃字差；体积未查到
- **出处**：https://arxiv.org/abs/1808.10583 §1–2【论文】; https://www.aishelltech.com/aishell_2【官方文档，本轮只读到页面标题】

## 备注
- 2026-11-03 WebFetch 读 aishelltech.com/aishell_2 只拿到标题；英文页 404。许可与获取方式需怀疑派换浏览器复核。
- 中文里少见的「同一句话、手机与高保真麦并行」评测集，可用于设备差异校准；但是朗读、无会议噪声。
