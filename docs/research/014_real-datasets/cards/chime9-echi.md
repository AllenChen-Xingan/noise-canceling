### CHiME-9 ECHI（2025，Sheffield 等 CHiME 组织方）
- **做什么**：嘈杂食堂中为听障者增强对话伙伴语音，要求低延迟
- **录音设备与距离**：一人戴 Meta Project Aria 眼镜（7 ch, 48 kHz），另一人戴助听器（每耳前后 2 麦共 4 ch, 48 kHz）；同桌 4 人，具体 m 数未查到（官网只给 Aria 麦位置与头部追踪数据）
- **环境**：单个房间，18 个扬声器：14 个播放 **LibriSpeech 朗读或 EARS 自发语音** 模拟最多 7 组同时进行的两人对话（train 只用 LibriSpeech），FSD50K 稀疏音效混入这 14 路；4 个角落扬声器播 WHAM! 背景。场间微调扬声器位置与吸声板。真人真对话 + 回放（且多为朗读语料）的干扰人声 → 部分
- **说话人数与重叠**：桌边 4 人真实对话 + 回放的多组干扰“对话”
- **语言**：英语
- **参考**：近讲 有（每人 1 路 48 kHz 近讲麦，**仅限训练时使用**，用于构造参考；麦型号未查到）· 转写 未查到（官网与数据卡均未提）· 说话人标签 有（按伙伴分轨）
- **大小**：train 18.0 h / dev 6.0 h / eval 5.4 h（eval 在截止前才发放）；共约 105 GB
- **许可**：自定义 DUA（2025-06）：非独占、不可转让；仅学术、算法开发与基准、署名发表；**禁再分发原始数据**、禁上传公共仓库/云平台；禁生物识别与再识别 → 只评测、不可入库
- **获取**：签协议 · huggingface.co/datasets/CHiME9-ECHI/CHiME9-ECHI（HF 账号申请 + 签 DUA + 组织方审批，机构邮箱优先）。后续 CHiME-10 ECHI-2：智能眼镜/助听器真实对话 + 视频；预览库 377 GB，仅限开发组，签 2026-09 版预览 DUA，计划 2027-02 公开；挑战页未写许可
- **接近度**：3.5/5 · 多路同时人声 babble 最接近 D42 的声学形态，伙伴重叠贴 D47；但干扰人声是扬声器回放的朗读语料，设备是眼镜/助听器而非笔记本
- **用途建议**：sim-to-real 校准（多说话人 babble 电平/谱，注意是回放）、本机回归评测、人耳听测；不训练、不入库
- **出处**：https://www.chimechallenge.org/challenges/chime9/task2/data 【官方】；https://huggingface.co/datasets/CHiME9-ECHI/CHiME9-ECHI 【官方】；https://www.chimechallenge.org/challenges/chime10/task1/index 【官方】；https://huggingface.co/datasets/jonbarker68/echi2-collab-preview 【官方】；arXiv:2609.26306（CHiME-9 ECHI 挑战总结）【论文】

<details><summary>另一路的卡片（chime9-echi-D-cafe.md，怀疑派已修正；与上文冲突处见 ../decision.md 附录「卡片冲突裁定」）</summary>

#### CHiME-9 ECHI（2025/2026，Sheffield 等，CHiME-9 Task 2）
- **做什么**：从助听器/眼镜麦中提取并增强对话伙伴的语音。
- **录音设备与距离**：Project Aria 眼镜（7 通道 48 kHz，有大小两种尺寸，时钟漂移约 100 ms/h）+ 助听器（4 通道 48 kHz，每耳前后各 1 麦）；围桌约 1 m 级【推算】。
- **环境**：真人在同一房间现场对话；背景由 14 个扬声器回放最多 7 组双人对话（LibriSpeech/EARS），另有 4 个扬声器朝墙角播放 WHAM! 环境声，FSD50K 稀疏事件**混入对话通道**——背景是回放的，所以算「部分」真实。
- **说话人数与重叠**：每桌 4 个座位，自然对话，有重叠。
- **语言**：英语。
- **参考**：近讲 有（每座 1 路，目录里 train/dev/eval 都有 ct，但**规定只能在训练时使用**；dev 下载说明里只提 HA/Aria，dev 是否附带近讲未确认）· 转写 数据卡未提及 · 座位标签 有。
- **大小**：29.4 h，约 105 GB。
- **许可**：ECHI DUA；禁止再分发原始数据，商用要另外联系 · 结论：只评测。
- **获取**：签 DUA，经 HF 门控由组织者审批：https://huggingface.co/datasets/CHiME9-ECHI/CHiME9-ECHI
- **接近度**：3/5（原 3.5，下调）· 结构贴近 D47，但背景人声是回放，设备是眼镜/助听器，近讲参考在评测时受限制。
- **用途建议**：真实回归集；听测。只存本机。
- **出处**：https://huggingface.co/datasets/CHiME9-ECHI/CHiME9-ECHI 【官方】· arXiv 2609.26306（挑战与结果论文，非数据描述论文；数据论文为 Sutherland et al., IEEE Data Descriptions 2026）【论文】

</details>
