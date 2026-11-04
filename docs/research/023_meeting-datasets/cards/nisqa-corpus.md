# NISQA Corpus（TU Berlin 语音质量语料，全集）

- **名称**：NISQA Corpus（含 TRAIN_SIM / VAL_SIM / TRAIN_LIVE / VAL_LIVE / TEST_LIVETALK / TEST_FOR / TEST_NSC / TEST_P501 八个子集）
- **链接**：https://github.com/gabrielmittag/NISQA/wiki/NISQA-Corpus
- **许可**：按子集各异。全集原句「all of the files in this corpus can be used for non-commercial research purposes」，并要求看包内各子集 readme / license；SIM 与 LIVE 段原句「provided under the original terms of the used source speech and noise samples」，维基称 LIVE 段可用于商业与非商业研究；LIVETALK「made available for research purposes under」CC BY 4.0；TEST_FOR 只许非商业研究与司法；TEST_NSC 只许非商业研究；TEST_P501 不得进商业产品；Zenodo 记录的许可字段为 `other-at` → 结论：只评测【官方文档】
- **大小**：> 14,000 条：TRAIN_SIM 10,000 · VAL_SIM 2,500 · TRAIN_LIVE 1,020 · VAL_LIVE 200 · LIVETALK 232 · FOR 240 · NSC 240 · P501 240；`NISQA_Corpus.zip` 15,945,499,507 字节（约 15.9 GB，Zenodo API 文件元数据）【官方文档】
- **场景键**：home, cafe, car, transit, street, other（LIVE 段在室内放音并制造敲键盘、开窗街噪；LIVETALK 含咖啡馆、高速车内、电梯、商场、地铁站、街边）
- **设备键**：phone, laptop-other, unknown
- **笔记本段**：有（回放进笔记本麦）但不能筛。TRAIN_LIVE / VAL_LIVE：有声书经录音棚监听音箱放进终端（论文写「phone/laptop」，维基写 Skype 为两台笔记本互打）；TEST_FOR / NSC / P501 的 VoIP 段是「played back directly from the laptop」——这是从笔记本直接送入信号，**不经笔记本麦**，不算笔记本麦录音；LIVETALK 说话端是手机还是笔记本，论文与维基说法不一（见 r3 补充文件）。逐条设备标签未查到；笔记本型号未写【论文 §3】【官方文档】
- **干净参考**：部分（SIM 与 LIVE 段源自 LibriVox 等干净语料，可做全参考；LIVETALK 没有）【论文】
- **转写**：没有（未提）
- **语种**：英语（SIM、LIVE、FOR、P501）；德语（NSC、LIVETALK）
- **语体**：朗读为主（有声书、Harvard 句子）；AusTalk 含访谈对话；FOR、NSC 为会话片段；笑声 / 附和未标注【论文】
- **获取方式**：直接下载（https://depositonce.tu-berlin.de/bitstream/11303/13012.5/9/NISQA_Corpus.zip ；镜像 https://zenodo.org/record/4728081）
- **与 D62 的关系**：SIM 段是合成损伤；LIVE 段是回放重录 + 真实网络传输；LIVETALK 是真人对着手机 / 笔记本说的真实通话。人评分：TRAIN / VAL 为 P.808 众包每条 5 分，FOR / NSC / P501 论文写每条 30 分（维基：滤除前 50 分，滤后约 30 / 27 / 28 分），LIVETALK 为实验室 P.800 每条 24 分；全部有整体质量 + 噪声感、音色、断续、响度四个维度。若开发阶段拿来评测：用 TEST 四个子集（952 条，论文 §3 原数）校准无参考音质分；只发整包 15.9 GB，单拿 TEST 子集的大小未查到
- **出处**：https://github.com/gabrielmittag/NISQA/wiki/NISQA-Corpus【官方文档】; https://arxiv.org/abs/2104.09494 §3、Table 4【论文】; https://github.com/gabrielmittag/NISQA【官方文档】

## 备注
- 014 只开了 LIVETALK 子集卡（`014_real-datasets/cards/nisqa-livetalk.md`）；本卡是全集，二者实体不同。
- NISQA 模型与 SCOREQ（natural 域）都在 TRAIN_SIM 上训练，**用本集测试它们会偏高**（见过同源分布）。
- 「四个维度」是本集独有价值：可以分开看无参考模型对「噪声感」与「音色」各自准不准。
- 怀疑派复核：改了 4 处——「许可」原稿把 SIM / LIVE 子集的句子当成全集原句，已换成维基的全集原句（非商业研究）并补 Zenodo 许可字段；「大小」补 zip 约 15.9 GB；「笔记本段」指出 FOR / NSC / P501 是笔记本直接送信号、不经笔记本麦；「与 D62 的关系」补维基滤后票数。条数、P.808 每条 5 分、LIVETALK P.800 每条 24 分回论文 §3 核对一致。
