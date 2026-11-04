# AISHELL-2

- **名称**：AISHELL-2: Transforming Mandarin ASR Research Into Industrial Scale（AISHELL-2）
- **链接**：https://www.aishelltech.com/aishell_2
- **许可**：AISHELL 基金会自定义申请条款（Kaldi 官方仓库 egs/aishell2/README.md 原文：iOS 数据「free for non-commercial research and education use」；申请表须确认填全大学院系 / 教育机构信息、只作非商业研究 / 教育用；院系内可共享，不得再分发给其他高校或机构）→ 结论：只评测【官方文档】
- **大小**：1000 h（iOS 通道）· 1991 人；dev 2500 句（5 人）、test 5000 句（10 人）× 3 通道；GB 未查到【论文 §2】
- **场景键**：home-quiet, other（1347 人在录音棚，其余在有自然混响的客厅【论文 §2】）
- **设备键**：phone, studio
- **笔记本段**：没有笔记本。训练集只有 iPhone 通道；dev / test 有 iOS、Android、Mic 三通道并行录音，按通道分别发布，可按设备筛（论文表 1 按 dev android / dev ios / dev mic 分列；包内目录名未查到）【论文 §1–2、表 1】
- **干净参考**：部分（dev / test 的 Mic 通道与手机通道是同一句话并行录，可当参考；位置不同、未说明是否时间对齐）
- **转写**：有（句级，无时间戳）
- **语种**：普通话
- **语体**：朗读（含语音指令、地名、娱乐、金融、科技、体育、英文拼读、少量无主题自由说）；无笑声、附和、情绪
- **获取方式**：签协议（向 aishell.foundation@gmail.com 索取并填写申请表，硬盘寄送或网盘下载，基金会承担费用；申请条件要求大学院系 / 教育机构信息【官方文档：Kaldi README】；个人项目是否符合条件未查到；dev / test 评测集 README 指向 aishelltech.com/aishell_eval 下载，该页未读到）——本路未申请
- **与 D62 的关系**：真实录音（真人朗读）。若开发阶段拿来评测，可用 dev / test 的三通道平行段（7500 句 × 3）比较同一句在手机 / 麦上的吃字差；体积未查到
- **出处**：https://arxiv.org/abs/1808.10583 §1–2【论文】; https://github.com/kaldi-asr/kaldi/blob/master/egs/aishell2/README.md【官方文档】; https://www.aishelltech.com/aishell_2【官方文档，本轮只读到页面标题】

## 备注
- 2026-11-03 WebFetch 读 aishelltech.com/aishell_2 只拿到标题；英文页 404。许可与获取方式需怀疑派换浏览器复核。
- 中文里少见的「同一句话、手机与高保真麦并行」评测集，可用于设备差异校准；但是朗读、无会议噪声。
- 怀疑派复核：许可原文在 Kaldi 官方仓库 egs/aishell2/README.md 读到（aishelltech.com 仍只返回标题），许可由「未查到」改为自定义条款 → 只评测；获取方式补申请表条件；场景键补 home（客厅）；dev / test 目录名原写法无出处，改为「按通道分列，目录名未查到」。
