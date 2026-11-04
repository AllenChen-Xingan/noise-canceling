# VocalSound（笑声、叹气、咳嗽等非言语人声）

- **名称**：VocalSound: A Dataset for Improving Human Vocal Sounds Recognition（VocalSound）
- **链接**：https://sls.csail.mit.edu/downloads/vocalsound/
- **许可**：官方页原句「distributed under the Creative Commons Attribution-ShareAlike (CC BY-SA) license」；GitHub README 写「Creative Commons Attribution-ShareAlike 4.0 International License」→ 结论：只评测【官方文档】
- **大小**：21,024 条，3,365 人；44.1 kHz 版 4.5 GB、16 kHz 版 1.7 GB；总时长未查到；国家数未查到；年龄从测试子集分组（18–25 / 26–48 / 49–80）看覆盖 18–80 岁【官方文档：官网与 GitHub README】
- **场景键**：home-family, other（众包，参与者自选地点；推断）
- **设备键**：non-laptop-unknown（Amazon Mechanical Turk 众包，参与者自有设备；官方页未写）
- **笔记本段**：可能有（众包常用笔记本或手机录），但无设备字段，不能筛【推断】
- **干净参考**：没有（单人近距离录制，本身较干净，但无独立参考）
- **转写**：没有（只有 6 类标签：笑声、叹气、咳嗽、清嗓、喷嚏、吸鼻子；附说话人年龄、性别、母语、国家、健康状况）
- **语种**：非言语（无词）
- **语体**：非言语人声；**笑声单类标注**；无附和、无情绪语音
- **获取方式**：直接下载（GitHub YuanGongND/vocalsound，未见注册要求）
- **与 D62 的关系**：真实录音（众包单人录制）。若开发阶段拿来评测：取「笑声」类做「降噪会不会把笑声当噪声压掉」的检查素材（混进会议语音或单独过模型看保留度），约 3,500 条笑声（按 6 类均衡推算）【推断】
- **出处**：https://sls.csail.mit.edu/downloads/vocalsound/【官方文档】; https://arxiv.org/abs/2205.03433【论文】

## 备注
- 咳嗽、清嗓、喷嚏在会议里也常见；它们该保留还是压掉，属于人耳偏好问题，不在本卡下结论。
- 怀疑派复核：删去无出处的「60 个国家」（摘要、官网、README 都没有）；补 GB（README：44.1 kHz 4.5 GB / 16 kHz 1.7 GB）；补 README 的许可全名 CC BY-SA 4.0；场景键里的来源标签挪进括号。
