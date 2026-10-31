# B-chime · 调研与怀疑派记录

## 未找到 / 不符合命门
- 本路所有数据集均未用笔记本内置麦录音（CHiME 系为 Kinect / 会议设备 / 眼镜 / 助听器阵列）——D42 的“笔记本麦”维度本路无真实集
- 本路无“真实咖啡馆/餐厅现场”录音：ECHI 与 EasyCom 的人群噪声都是扬声器回放；CHiME-6/DiPCo 是家庭/模拟晚餐
- MISP2021 的许可原文名与获取流程：官方页只写 available under license，未查到
- ECHI 近讲麦型号、说话人到设备距离、是否带转写：未查到
- Mixer 6 各路麦的类型与距离：在 LDC 的 mx6_collection_doc.pdf 内，未读到；LDC 用户协议原文名未逐字核实
- CHiME-5 Kinect 的代际（v1/v2）：论文与官网只写 Microsoft Kinect
- DiPCo 在 chimechallenge.org/datasets/dipco 的官方页 404；DiPCo 重叠率、近讲麦型号未查到
- CHiME-10 ECHI-2 的许可与时间表：仅预览库（限开发组，计划 2027-02 公开），挑战主页未写许可
- NOTSOFAR-1 HF 数据卡的 YAML license 字段为空，许可仅见正文与 GitHub README

## 预判核验
- “CHiME-5/6 禁商用、需签协议”——推翻：2024-01-01 已重新发布为 CC BY-SA 4.0，官方写“学术与商业均免费”；CHiME-6 在 openslr.org/150 直接下载，CHiME-5 经 myairbridge 下载（https://www.chimechallenge.org/datasets/chime5 【官方】，https://www.openslr.org/resources/150/LICENSE.txt 【官方】）。旧的 Sheffield 分非商业/商业两档许可已是历史（https://auditory.org/postings/2019/686.html 【二手】）。但按本项目规定 SA → 只评测
- “CHiME-5/6 最贴近嘈杂多人 + 近讲参考”——部分成立：对 D47（真实邻座重叠 + 双耳近讲 + 人工时间戳）最贴；对 D42 的“一大片人”babble 不如 ECHI（14 个扬声器同时放 7 组对话），但 ECHI 是回放（arXiv:1803.10609 §2；chime9/task2/data 【官方】）
- “DiPCo 是 CDLA-Permissive、可直接下载”——成立，版本为 1.0（论文 §4 指向 cdla.io/permissive-1-0，Zenodo 8122551 标 CDLA-Permissive 1.0）；Avignon 镜像写 2.0 属二手不一致。现行地址 Zenodo，论文原 S3 链接非正本（arXiv:1909.13447 §4 【论文】；https://zenodo.org/record/8122551 【官方】）
- “EasyCom 噪声是回放、CC BY-NC”——成立：10 个扬声器回放互不相关的餐厅噪声约 71 dB SPL，RT60 645 ms；许可 CC BY-NC 4.0（arXiv:2107.04174v2 §III 【论文】；GitHub README 【官方】）
- “ECHI 是 2025–26 新集、许可未知”——部分推翻：数据为 2025-06 版，许可已知 = 自定义 CHiME-9 ECHI Data Use Agreement（学术/算法开发，商用另洽，严禁再分发，需 HF 上签协议并经审批）；设备确为 Meta Aria 眼镜 + 助听器；噪声为 18 个扬声器模拟食堂（https://huggingface.co/datasets/CHiME9-ECHI/CHiME9-ECHI 【官方】）。后续 CHiME-10 ECHI-2 预览数据限开发组，计划 2027-02 公开
- 补充：CHiME-8 DASR 中的 NOTSOFAR1 场景与 NOTSOFAR 任务是同一批数据，只是改名（chime8/task1/data 【官方】）；NOTSOFAR-1 为 CC BY 4.0，是本路唯一同时满足真实录音 + 近讲 + 可训练的多人重叠集（另一个是 DiPCo）

## 怀疑派更正
- CHiME-9 ECHI · 原说法「14 个扬声器播放最多 7 组同时进行的两人对话」（暗示是真实对话 babble）→ 更正：14 路播放的是 LibriSpeech 朗读或 EARS 自发语音拼成的“对话”（train 只用 LibriSpeech），FSD50K 音效也混在这 14 路里；干扰人声是回放的朗读语料，不是真实人群。接近度 4→3.5 · https://www.chimechallenge.org/challenges/chime9/task2/data 【官方】；https://huggingface.co/datasets/CHiME9-ECHI/CHiME9-ECHI 【官方】
- CHiME-9 ECHI · 原说法「eval 5.4 h」未注明状态 → 更正：eval 尚未发放，截止前才下发；近讲麦每人 1 路 48 kHz，只许训练时用；转写官网与数据卡都没提 · 同上两页 【官方】
- CHiME-9/10 · 原说法「ECHI-2 挑战主页未写许可」→ 补充：预览库自带 CHiME-10 ECHI-2 Preview Data Use Agreement（2026-09 版），禁再分发、需审批，377 GB，计划 2027-02 公开 · https://huggingface.co/datasets/jonbarker68/echi2-collab-preview 【官方】
- MISP-Meeting · 原说法 access「签协议」→ 更正：「直接下载」，官方页直接给 zip 链接，写明「By using the corpus, you agree to the terms of this License」，没有签字审批步骤；许可仍是非商业 → 只评测 · https://challenge.xfyun.cn/misp_dataset 【官方】
- MISP2021 · 原说法「challenge.xfyun.cn/misp_dataset 获取（流程未查到）」→ 补充：现页面对 MISP2021/2022/2023 各语料都没有列下载链接，只写 available under license；远场阵列麦间距 35 mm 未核实，另有二手来源写 92 mm；253 人与 98 男 +165 女自相矛盾 · https://challenge.xfyun.cn/misp_dataset 【官方】；Interspeech 2022 zhou22g 摘要页没有设备细节 https://www.isca-archive.org/interspeech_2022/zhou22g_interspeech.html 【论文】
- NOTSOFAR-1 · 原说法「HF 下载需 HF token」→ 更正：HF 页没显示 gated，数据实际用官方 azure_storage.py 或 AzCopy 从 Azure Blob 拉；HF 全库（含 1000 h 模拟）19.5 TB；「约 170 h」是按设备累计的时长，真实会议时长约 315×6 min≈31.5 h【推算】 · https://huggingface.co/datasets/microsoft/NOTSOFAR 【官方】
- CHiME-7/8 DASR · 原说法 access「签协议」（整体）→ 澄清：只有 Mixer 6 要把 LDC_CHiME8_Evaluation_Agreement 发给 LDC 审批，CHiME-6/DiPCo/NOTSOFAR1 由 chime-utils 直接下载；CHiME-8 数据页本身没写许可，各子集许可要回原出处查；NOTSOFAR dev 真值不公开，经排行榜评分 · https://www.chimechallenge.org/challenges/chime8/task1/data 【官方】
- Mixer 6 · 原说法「仅会员享 reduced-license fee」→ 更正：目录页写的是“members-only corpus”，许可栏列 Subscription & Standard Members 与 Non-Members，费用需登录查看；reduced fee 的说法在页上查不到；目录页也没说附带转写 · https://catalog.ldc.upenn.edu/LDC2013S03 【官方】
- CHiME-5/6 · 接近度 4→3.5（4 人家庭聚会，没有 babble，用 Kinect 而不是笔记本麦）；补 CHiME-5 包大小 91G/10G/13G + 转写 10 MB；LICENSE.txt 核实为 CC BY-SA 4.0 International，没有 NC 或仅研究条款，要求引用 Barker 2018 · https://www.chimechallenge.org/datasets/chime5 【官方】；https://www.openslr.org/resources/150/LICENSE.txt 【官方】
- DiPCo · 原说法「唯一许可干净的晚餐重叠真实集 → 可进训练」→ 压低：只有 dev+eval 共 5.3 h，没有 train 划分，参与者是 Amazon 员工；拿来训练就失去回归集的价值，更适合整份做回归/校准；接近度 3.5→3。许可 CDLA-Permissive 1.0、13.4 GB、1.12–3.95 m 都已核实 · https://zenodo.org/record/8122551 【官方】；arXiv:1909.13447 §2.2–2.3、Table 1–3、§4 【论文】
- EasyCom · 核实无误（CC BY-NC 4.0、10 路回放、约 71 dB SPL、RT60 645 ms、5 h 18 min 11 s / 6.6 h）。补充：眼镜佩戴者和服务员都没有头戴近讲 · https://github.com/facebookresearch/EasyComDataset 【官方】

## 怀疑派：漏掉的
- CHiME-3 / CHiME-4 真实子集：参与者对着平板读 WSJ 句子，平板边框嵌 6 麦（形态最接近笔记本内置麦），在真实咖啡馆、公交、街道、步行区录制，并同步录了近讲麦；真实部分约 3 h train + dev/test；许可与获取方式本轮未核实（chimechallenge.org/datasets/chime4 返回 404）· https://shadow.merl.com/publications/docs/TR2015-136.pdf
- MMCSG（CHiME-8 Task 3）：两人都戴 Aria 眼镜做真实对话，含佩戴者与对方重叠，贴 D47；需注册，禁止分发给未注册者，许可名未查到 · https://ai.meta.com/datasets/mmcsg-dataset/
- AMI Meeting Corpus：真实会议，有头戴、领夹和桌面阵列，人工时间戳转写，常见说法是 CC BY 4.0（本轮未核实）；可作许可干净的英文重叠基线 · https://groups.inf.ed.ac.uk/ami/corpus/
- AliMeeting / M2MeT：中文真实会议，2–4 人，有头戴近讲加 8 麦阵列，重叠率高；openslr 119，常见说法是 CC BY-SA 4.0（未核实）；补中文 D47 · https://www.openslr.org/119/
- AISHELL-4：中文真实会议室 8 麦圆阵，4–8 人重叠，有时间戳转写；openslr 111（许可本轮未核实）· https://www.openslr.org/111/
- MCAS（与 MMCSG 配套）：Aria 眼镜实录 RIR 与消声室传递函数，可直接用于 sim-to-real 的设备响应校准 · https://ai.meta.com/datasets/mcas-dataset/
