# D-cafe · 调研与怀疑派记录

## 未找到 / 不符合命门
- 笔记本内置麦 + 咖啡馆/餐厅/开放办公 + 带参考的公开数据集：没有。全网搜索（含 extended 模式）未找到任何一个；最接近的是 CHiME-3/4 CAF（平板 6 麦）、ASPIRE（iPad+领夹，真实食堂）、DNS 真实盲测集（含笔记本/手机内置麦，但无参考、无场景标签）。
- Libri-Adapt（arXiv 2009.02814 §3）：6 种设备（Matrix Voice、ReSpeaker、PS Eye、USB、Nexus 6、Shure MV5，无笔记本内置麦）在安静房间回放重录，噪声（风、雨、笑声）为人工叠加 20 dB——不满足真实嘈杂声场，CC BY 4.0。
- SPEECHCOCKTAIL：未查到此名称的数据集；相近的是 MCoRec（已建卡）、Bern Cocktail Party（头戴麦测量 160 个场景，未深查）。
- REAL-T（HF REAL-TSE/REAL-T）：是从 AISHELL-4、AMI、AliMeeting、CHiME-6、DiPCo 切出的目标说话人提取评测集，不是新录音；许可受上游约束，未查到其自身许可。
- TAU Urban Acoustic Scenes / DCASE 场景录音：真实录音但无目标语音、无参考，仅能作噪声底；本轮未逐项核实其许可与是否含咖啡馆类别。
- AVA-Speech（arXiv 1808.00606）：YouTube 电影片段的语音活动标注，非设备现场录、无近讲参考，不满足命门 1【二手】。
- VoxConverse（arXiv 2007.01216）：YouTube 视频说话人日志，非受控设备录音【二手】。
- Earnings-21（arXiv 2104.11348）：财报电话会议音频，电话信道、无公共场所声场【二手】。
- MagicData-RAMC（arXiv 2203.16844）、KeSpeech：手机/电话对话或朗读，以安静环境为主，无嘈杂公共场所子集【二手，未逐项核实】。
- WenetSpeech（arXiv 2110.03370）：互联网音视频（播客、YouTube 等），非受控设备现场录，无近讲参考【二手】。
- MISP2021 / MISP-Meeting（中文家庭电视场景，远/中/近场麦）：官方页 challenge.xfyun.cn/misp_dataset 称需接受许可；第三方页面标题为「Non-commercial Data License Agreement」；场景与设备细节本轮未读到，未建卡。
- Google 的真实会议公开集：未查到 Google 发布的带设备麦与参考的真实会议数据集。
- ASpIRE（IARPA，Fisher 回放重录）：本轮未单独核实；与 Stirling ASPIRE 同名不同物，后者已建卡。
- MS-SNSD 等：合成混音，不满足命门 1。

## 预判核验
- 预判「CHiME-4 真实部分是平板麦在咖啡馆/公交录的、有转写」——成立。官方数据页：平板 6 麦（ch1–6）+ ch0 近讲，真实环境 BUS/CAF/PED/STR，real train 1600 句、dev 1640 句、test 1320 句，转写为 WSJ0 提示文本。出处 https://www.chimechallenge.org/challenges/chime4/data 【官方】。需要更正 4 点：(a) 真实录音其实是 CHiME-3 录的，CHiME-4 只是复用并限制测试通道数；(b) 音频只能从 LDC 付费取得（LDC2017S24，「LDC User Agreement for Non-Members」），不是免费下载，出处 https://www.chimechallenge.org/challenges/chime4/download 和 https://catalog.ldc.upenn.edu/LDC2017S24 【官方】；(c) 每段只有 1 人朗读，没有邻座重叠，D47 覆盖不到；(d) 近讲麦与阵列之间有同步误差（arXiv 2407.19485）【论文】。
- 预判「CHiME-4 可能是公开集里最接近『设备麦 + 咖啡馆』的」——部分成立。在「真实设备麦 + 真实咖啡馆 + 有近讲参考」这三条同时满足的集合里，它仍是首选。不过另有两个同档候选：ASPIRE（Stirling，午餐高峰真实食堂/餐厅，iPad mini 2 加领夹麦，arXiv 1910.00424 §2.3），以及 CHiME-9 ECHI（4 人真实围桌对话，但食堂 babble 是扬声器回放，有近讲参考，HF 数据卡）。如果看重「一大片真人同时说话」，MCoRec 最真实（一屋最多 8 人、4 组对话同时进行，arXiv 2510.23276 §2.1），但麦是 360° 相机。
- 预判「Common Voice 设备多样但多为安静环境」——成立。官方 tsv 字段（client_id、sentence、votes、age、gender、accent 等）里没有噪声/环境标签，出处 Mozilla Discourse 与 cv-dataset 仓库【二手】；有一项人工抽查显示明显背景噪声只占 0.417%（arXiv 2206.14053）【论文】。补充一点：现行托管方是 Mozilla Data Collective（Scripted Speech 23.0，CC0-1.0），但下载条款另外禁止再托管/再分享，也禁止识别说话人【官方】。
- 隐含问题「有没有笔记本内置麦 + 咖啡馆的公开集」——明确答案：没有。唯一明确把「笔记本/台式/手机内置麦」写进定义的真实录音，是 DNS Challenge Track 2 和它的盲测集（众包真实录音，约 70% 为手机，含邻近说话人，有转写，但没有干净参考，arXiv 2202.13288 §3.3）【论文】。
- 线索「Libri-Adhoc40 = 自组织阵列回放重录」——成立。但它是在安静的混响办公室录的（T60≈900 ms，几乎没有附加噪声），对 D42 帮助很小；仓库标的许可是 GPL-3.0，用在数据上含义不清（https://github.com/ISmallFish/Libri-adhoc40）【官方】。
- 线索「AISHELL-5 车载」——成立：OpenSLR SLR159，CC BY-SA 4.0，车内 4 通道，近讲只在 train 里有。场景与 D42/D47 不符。
- 线索「ARTE（Weisser 2019）真实餐厅场景」——成立：13 个环境，其中有咖啡馆×2、晚宴、美食广场×2，62 通道阵列录成 HOA，没有语音参考。许可有冲突：Zenodo 字段写 CC BY 4.0，说明里又写「仅非商业个人/教育/研究」，按严格的一方算只评测（https://zenodo.org/records/3386569）【官方】。
- CHiME-6 许可——OpenSLR SLR150 现行写的是 CC BY-SA 4.0，不再是早年 Sheffield 的非商用协议，但按项目规定（SA）仍只评测（https://openslr.org/150）【官方】。
- CC BY 4.0 / CDLA 这类可训练的真实多人重叠集只查到 NOTSOFAR-1（CC BY 4.0，会议设备麦 + Shure 头戴参考）、DiPCo（CDLA-Permissive-1.0，5.3 h）、AMI（官方现行 CC BY 4.0）；都不是咖啡馆，也都不是笔记本麦。

## 怀疑派更正
- DNS Challenge 真实盲测集 · 原说法「仓库 LICENSE 为 CC BY 4.0」 → README 写明 CC BY 4.0 只覆盖文档，代码是 MIT，数据集「provided under the original terms that Microsoft received such datasets」，盲测集一节没有写许可，因此许可改为未查到；接近度 3→2.5 · https://github.com/microsoft/DNS-Challenge
- ASPIRE · 原说法「许可未查到、获取渠道不明」 → 在 Zenodo 找到了，可直接下载 ASPIRE.zip（13.0 GB）。许可字段写「Non-Commercial Government Licence」，但链接指向 Apache-2.0，两者矛盾，按非商用算只评测 · https://zenodo.org/records/4585619
- MCoRec · 原说法「领夹音频是否发布未查到；语言为推算；智能手机视频另计」 → HF 数据卡：手机领夹近讲麦只在 train 中发布，dev/eval 只有中心 360° 相机音频（16 kHz 单通道）；语言为英语；总量 229 GB；arXiv 2510.23276 的作者是 Nguyen, Zmolikova, Ma, Pham, Fuegen, Waibel · https://huggingface.co/datasets/MCoRecChallenge/MCoRec
- MCoRec · 原说法「仓库另写 CC-BY-NC」 → 本轮未核实，已从许可名中删除；以 HF 上的 Data Agreement 为准 · https://huggingface.co/datasets/MCoRecChallenge/MCoRec
- CHiME-9 ECHI · 原说法「近讲仅 train 提供」「助听器通道数与论文有出入」 → 数据卡的目录结构里 train/dev/eval 都有 ct，但规定近讲只能在训练时使用；助听器是每耳 2 麦（前/后）共 4 通道，数据卡本身前后一致；数据卡没有提到转写；arXiv 2609.26306 是挑战结果论文，数据描述论文是 Sutherland et al., IEEE Data Descriptions 2026；背景回放的 FSD50K 事件是混入对话通道的。接近度 3.5→3 · https://huggingface.co/datasets/CHiME9-ECHI/CHiME9-ECHI
- CHiME-3/4 · 原接近度 4 → 3.5（平板、朗读、没有重叠、只有英语）；LDC 页明确说包内含人工混合的模拟数据，342 h 不全是真实录音；test 时长官方页没有给 · https://catalog.ldc.upenn.edu/LDC2017S24 · https://www.chimechallenge.org/challenges/chime4/data
- NOTSOFAR-1 · 原说法「直接下载」 → 仓库下载脚本需要 HF 账号 token（HF_TOKEN），改为「注册」；Dev-set-2 不公开发布；HF 仓库总量 19.5 TB；会议数按 HF 为 315 场 · https://github.com/microsoft/NOTSOFAR1-Challenge · https://huggingface.co/datasets/microsoft/NOTSOFAR
- EasyCom · 原说法「近讲 有」 → 只有非佩戴者有头戴近讲，AR 眼镜佩戴者本人（相当于本项目的「使用者」）和服务员都没有近讲参考 · https://github.com/facebookresearch/EasyComDataset
- DiPCo · 原说法「每位说话人一支近讲麦」 → Zenodo 只写了「a single-channel close-talk microphone」，没写清每人一支，标为论文说法、未复核 · https://zenodo.org/records/8122551
- AliMeeting · 补充：train 的近讲音频单独打包（Train_Ali_near 22.85 GB），eval/test 包里远场和近讲都有 · https://www.openslr.org/119/
- AISHELL-5 · 补充：近讲只用于标注，只有 train 带近讲；Eval1 3.3 h、Eval2 3.58 h；train 包 54 GB · https://www.openslr.org/159/
- MMCSG · 原说法「大小未查到」 → 约 26 h（train 8.5/dev 8.4/eval 9.4 h，CHiME-2024 论文表 1，经检索摘要转述，标为二手）· https://www.isca-archive.org/chime_2024/zmolikova24_chime.html
- DAPS · 原说法「体积未查到」 → daps.tar.gz 16.1 GB，CC BY-NC 4.0 已确认 · https://zenodo.org/record/4660670
- Common Voice · 原 URL datacollective.mozillafoundation.org 已 301 跳转到 mozilladatacollective.com；新页面是 JS 渲染，许可与条款本轮无法复核 · https://mozilladatacollective.com/datasets/cmflnuzw52mzok78yz6woemc1
- AMI · 许可 CC BY 4.0 已由官方 license 页确认；「注册后下载」是否属实未复核 · https://groups.inf.ed.ac.uk/ami/corpus/license.shtml
- WHAM! · 二次检索仍找不到 CC BY-NC 4.0 的原文，维持未查到；wham.whisper.ai 的证书指向 GitHub，托管地址可能已变（lhotse 配方指向 wham.csail.mit.edu，TIB 记录写 wham.isir.org）· https://lhotse.readthedocs.io/en/stable/_modules/lhotse/recipes/wham.html
- 已核实无误：CHiME-6 SLR150 CC BY-SA 4.0 及 97/11/12 GB；ARTE 的 Zenodo 字段 CC BY 4.0 与「仅非商业」说明冲突，9.2 GB；AISHELL-4 CC BY-SA 4.0 且 OpenSLR 只描述远场；VOiCES CC BY 4.0；Libri-Adhoc40 GPL-3.0 且 train 未发布；DiPCo CDLA-Permissive-1.0、13.4 GB

## 怀疑派：漏掉的
- RealMAN（西工大等，2024）：中文真实录音，32 通道阵列在几十个真实场景录制，含食堂、商场等嘈杂公共场所，并提供直达声目标——中文 D42 最相关，需核实许可。arXiv 2406.19959 · https://github.com/Audio-WestlakeU/RealMAN
- Mixer 6 – CHiME-8 Transcribed（LDC2025S07）：真实房间访谈与通话，多支室内麦同步录制并附 CHiME-8 转写；需核实是否含笔记本位麦。https://catalog.ldc.upenn.edu/LDC2025S07
- MISP 2021/2022/2025（中文家庭电视场景与会议，远/中/近场同步，有转写）：中文 D47 的补充，需核实非商用协议。https://challenge.xfyun.cn/misp_dataset
- LibriCSS（Microsoft，2020）：在会议室回放重录的重叠语音，7 通道阵列，带干净源可对齐；属部分真实，可作重叠回归的对照。https://github.com/chenzhuo1011/libri_css
- ICSI Meeting Corpus（LDC2004S02）：真实会议，有头戴近讲，桌面麦里有廉价 PDA 麦，是老牌的「消费设备麦+重叠」数据。https://catalog.ldc.upenn.edu/LDC2004S02
- MC-WSJ-AV / REVERB 真实录音部分：真实房间远讲，有近讲（头戴/领夹）同步。https://catalog.ldc.upenn.edu/LDC2014S03
- URGENT Challenge（2024/2025）真实录音测试集：通用语音增强挑战，含多来源真实嘈杂录音，可作无参考回归。https://urgent-challenge.github.io/urgent2025/
- DCASE/TAU Urban Acoustic Scenes（含 metro station、shopping mall 等真实场景）：可作噪声底与谱校准，需核实是否含餐厅/咖啡馆类别及许可。https://dcase.community/challenge2020/task-acoustic-scene-classification
- Ego4D 社交/对话子集（含 Aria 与头戴相机音频、转写与说话人标签）：大量真实公共场所多人对话，签协议获取。https://ego4d-data.org/
