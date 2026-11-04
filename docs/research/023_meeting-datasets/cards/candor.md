# CANDOR Corpus（视频聊天自然对话）

- **名称**：CANDOR Corpus（Conversation: A Naturalistic Dataset of Online Recordings）
- **链接**：https://www.betterup.com/research/candor-research
- **许可**：BetterUp 官方页未写许可；Science Advances 论文本身为 CC BY-NC 4.0（那是论文的许可，不是数据的）；ConvoKit 写「原发布方许可适用」→ 结论：未查到【官方文档】【论文】
- **大小**：1,656 场对话（ConvoKit 写 1,650），850 h，>1 TB 音频，700 万词以上【论文摘要】【官方文档】
- **场景键**：home, other（2020 年 1–11 月，参与者各自在家 / 所在地上视频聊天，具体地点未标）【推断】
- **设备键**：unknown（视频聊天，参与者自有设备，推测多为笔记本或台式机摄像头麦）【推断】
- **笔记本段**：可能大量是笔记本内置麦，但官方页、ConvoKit 均未写设备；无逐条设备标签【推断】
- **干净参考**：没有
- **转写**：有（自动转写，三种切分口径；其中 backbiter 口径带附和语的文字、起止时间）【官方文档：ConvoKit】
- **语种**：美式英语
- **语体**：两名陌生人自由聊天；有附和（backchannel）标注；笑声是否标注未查到
- **获取方式**：签协议（向 BetterUp 申请；TalkBank CABank 另有分段版，访问条件未核实）
- **与 D62 的关系**：真实录音（远程视频通话，非回放），离「在笔记本前开会」的声学条件可能最近，但设备与噪声未标。若开发阶段拿来评测：抽少量对话做「附和、笑声、抢话被吃掉」的语音识别吃字检查与人耳试听；全量 >1 TB，只需几十场（各约 30 min）【推断】
- **出处**：https://www.betterup.com/research/candor-research【官方文档】; https://convokit.cornell.edu/documentation/candor.html【官方文档】; https://ui.adsabs.harvard.edu/abs/2023SciA....9F3197R/abstract【论文】; https://talkbank.org/ca/access/CANDOR.html【官方文档】

## 备注
- 「设备是笔记本」是推断，需读论文方法部分（PMC10065445，本轮被浏览器校验页挡住）核实。
- 附和语有时间戳，是评测「降噪会不会吃掉嗯 / 对」的少见真实素材。
