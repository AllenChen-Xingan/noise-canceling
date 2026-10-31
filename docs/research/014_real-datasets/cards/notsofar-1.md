### NOTSOFAR-1（2024，Microsoft）
- **做什么**：CHiME-8 远场会议转写（DASR）基准，单通道 / 多通道两条赛道。
- **录音设备与距离**：桌中 4 台圆形 7 麦设备（Yealink SmartVision 60、Yealink Intelligent Speaker、EPOS Expand Capture 5、Sennheiser TeamConnect Intelligent Speaker），既出 SC 也出原始 MC 流；房间前方线阵设备只出 SC：Logitech MeetUp（全部集）、Poly Studio（仅 eval）、Jabra PanaCast 50（部分 eval）（论文表 2）。**SC 流是设备内部处理过的**（论文的说法是典型处理包括 AEC、去混响、波束、降噪，没说每台具体做了哪些）；MC 流是「未经处理直接从阵列录的原始格式」。**没有笔记本**。嘴到麦距离：未查到。
- **环境**：30 间微软会议室；包含走动、白板、进出、笑声咳嗽、瞬态噪声。不是咖啡馆那种一大片人声。
- **说话人数与重叠**：每场 4–8 人，共 35 人；元数据标注了重叠和声学事件。
- **语言**：英语。
- **参考**：近讲 有（Shure WH20 头戴 + TASCAM DR-40X，只有 train/dev，eval 没有；论文说近讲**可能有少量串音**）· 转写 有（专业转写员依据近讲人工转写，多数场次两人独立转写再由第三人裁定）· 说话人标签 有
- **大小**：train 110 场 55 h SC / 44 h MC；dev 35 场 17.5 / 14 h；eval 170 场 102 / 68 h（论文表 1，按设备累加，不是会议时长）。eval_small 的场次数：未查到。HF 仓总量 19.5 TB（含模拟集）。
- **许可**：CC BY 4.0，没有 NC/SA。Dev-set-2「只批准在 NOTSOFAR-1 挑战赛内使用」，因法律和质量原因已下架 · 结论：可训练（不要用 Dev-set-2）
- **获取**：HuggingFace `microsoft/NOTSOFAR`（脚本要 HF_TOKEN，HF 页面没写 gated 审批）；HF 数据卡另写了 Azure Blob + AzCopy 的旧路径。版本 240825.1_train / 240825.1_dev1 / 240825.1_eval_full_with_GT、240629.1_eval_small_with_GT；模拟集 v1.5
- **接近度**：2.5/5 · 真实，重叠多，英文，许可干净；但用的是会议设备不是笔记本麦，背景不是大片人声，SC 流已被处理过，得用 MC 原始流
- **用途建议**：英文真实回归集（取 MC 原始流的单个麦；eval 没有近讲，只能对照转写）；sim-to-real 校准混响和重叠统计；可训练
- **出处**：https://arxiv.org/abs/2401.08887 §4.1、表 1–2【论文】；https://github.com/microsoft/NOTSOFAR1-Challenge README（Dev-set-2 条款、版本号）【官方】；https://raw.githubusercontent.com/microsoft/NOTSOFAR1-Challenge/main/DATA_LICENSE【官方】；https://huggingface.co/datasets/microsoft/NOTSOFAR【官方】

<details><summary>另一路的卡片（notsofar-1-B-chime.md，怀疑派已修正；与上文冲突处见 ../decision.md 附录「卡片冲突裁定」）</summary>

#### NOTSOFAR-1（2024，Microsoft）
- **做什么**：远场会议转写（CHiME-8 Task 2）；CHiME-8 DASR 用同一批数据改名为第四场景
- **录音设备与距离**：桌面多台会议设备（Yealink SmartVision 60、Yealink Intelligent Speaker、EPOS Expand Capture 5、Sennheiser TeamConnect 等圆阵 + 前端线阵 Logitech MeetUp、Poly Studio、Jabra PanaCast 50）；单通道流是设备内部处理后的输出，不是单麦；距离未给（arXiv:2401.08887 §4.1）
- **环境**：约 30 间微软会议室，含瞬态与稳态噪声、走动、白板（§4.1）。非开放办公
- **说话人数与重叠**：每场 4–8 人，含打断、快速换人、重叠（未量化）
- **语言**：英语
- **参考**：近讲 有（头戴麦 + 录音机，型号见论文）· 转写 有（带时间戳）· 说话人标签 有
- **大小**：会议时长约 31.5 h【推算：315×6 min】；论文 Table 1 按设备累计 train 55 SC/44 MC h、dev 17.5/14、eval 102/68；DASR 版 train 38:43 / train_sc 53:43 / dev 13:25 / eval 20:00；HF 全库（含 1000 h 模拟）19.5 TB
- **许可**：CC BY 4.0（HF 卡正文与 DATA_LICENSE 文件；HF 元数据字段为空）→ 可训练
- **获取**：直接下载 · huggingface.co/datasets/microsoft/NOTSOFAR，HF 页未显示 gated，实际经 Azure Blob（官方 azure_storage.py 或 AzCopy）下载
- **接近度**：3/5 · 真实多人重叠 + 近讲；但会议室而非咖啡馆，设备为会议音箱，非笔记本麦
- **用途建议**：许可干净的真实多人重叠集 → 可训练；划出部分做办公场景回归集；校准重叠率与混响
- **出处**：https://github.com/microsoft/NOTSOFAR1-Challenge 【官方】；https://huggingface.co/datasets/microsoft/NOTSOFAR 【官方】；arXiv:2401.08887 §4.1 【论文】；https://www.chimechallenge.org/challenges/chime8/task1/data 【官方】

</details>

<details><summary>另一路的卡片（notsofar-1-D-cafe.md，怀疑派已修正；与上文冲突处见 ../decision.md 附录「卡片冲突裁定」）</summary>

#### NOTSOFAR-1（2024，Microsoft）
- **做什么**：远讲会议转写（CHiME-8 Task 2）。
- **录音设备与距离**：商用会议终端（桌面圆阵与墙前线阵，型号见论文 §4.1），**没有笔记本麦**。
- **环境**：Microsoft 办公楼会议室，有瞬态和稳态噪声；元数据带 #TransientNoise、#DebateOverlaps 等标签。不是咖啡馆。
- **说话人数与重叠**：每场 4–8 人，有重叠和打断。
- **语言**：英语。
- **参考**：近讲 有（头戴麦，eval 集没有）· 转写 有 · 说话人标签 有。
- **大小**：315 场；SC≈170 h / MC≈130 h；HF 仓库 19.5 TB（含模拟集）。
- **许可**：CC BY 4.0（数据）· 结论：可训练（含他人声音，仍按项目规定只存本机）。Dev-set-2 不在公开发布内。
- **获取**：HF microsoft/NOTSOFAR；仓库脚本 download_meeting_subset 需要 HF 账号 token（HF_TOKEN）。
- **接近度**：3/5 · 真实设备麦+重叠+近讲+许可宽松；但场景是会议室，设备是会议终端。
- **用途建议**：训练；真实回归集（#DebateOverlaps 测 D47）；混响/距离校准。
- **出处**：https://huggingface.co/datasets/microsoft/NOTSOFAR 【官方】· https://github.com/microsoft/NOTSOFAR1-Challenge 【官方】· arXiv 2401.08887 §4.1 【论文】

</details>
