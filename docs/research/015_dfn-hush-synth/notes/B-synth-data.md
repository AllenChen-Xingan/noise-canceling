# B · 合成数据配方：给 D47 辅助小模型与 013 RL 环境

> 2026-10-31 · 手下 B · 只读调研，**没有运行任何训练或合成程序**；文中数字凡标【推算】的都没实测。`resources/` 路径指主检出 `../noise_canceling/resources/`（本 worktree 不含该目录）。

**结论（先看这 5 行）**
1. **公开配方没有一套是为「你 vs 邻座」设计的**：WHAM!/LibriMix 是全重叠分离，DNS/URGENT 是去噪 + 失真，Hush 只训「背景人声比主讲轻 12–24 dB」。唯一覆盖负 SIR 的是微软个性化 DNS（pDNS）合成器：第二说话人与主讲的比值和噪声 SNR 同取 −5~20 dB【源码 `DNS-Challenge/pdns_noisyspeech_synthesizer_singleprocess.py:422-436`】。**我们得自己写会话生成器**，按真实轮替统计排时间轴（Landini 2022 的 simulated conversations 思路），不能照搬 Hush。
2. **真实会话重叠率**：两人电话 10.5–13%，会议 5–20%，晚餐聚会 27–34%（出处见 §2.1）。全重叠 / 高重叠合成（SM 约 32%）训出来的模型在真实数据上重叠误报偏多。**建议训练集每段重叠率均值 15–25%，裁判集按真实分布另抽。**
3. **SIR 必须铺到负值**：建议你对邻座 −6~+20 dB，约 20% 的样本 ≤ 0 dB（exp003 邻桌 −5 dB，exp005 c02 邻座只轻 3 dB）。这样还能专门查模型是不是在偷懒、只靠「谁更响」来判，而不看声纹。
4. **两套生成器**：训练用自写 sim.py RIR + FRA-RIR（Apache-2.0），噪声用 MUSAN / FSD50K（只取 CC0 / CC BY）/ DNS5 中 Freesound CC0 部分；裁判用 pyroomacoustics（MIT）+ 实测 RIR（dEchorate、METU，均 CC BY 4.0），噪声用 WHAM!、DEMAND、TAU（只评测）。
5. **WHAM! 许可已核实：CC BY-NC 4.0**【官方 wham.whisper.ai 页尾 License 段】。014 卡上写的「未查到」要改；按命门 1 应定为「部分」（真实双耳实录噪声 + 人工混合）。它不能进训练，但它是最像咖啡馆的裁判噪声。另外它录音时前级增益是校准过的定值，绝对声级可以拿来校准 013 假设的 60–85 dBA（§3.4）。

---

## 1. 公开合成配方与工具清单

### 1.1 混合配方

| 名称 | 生成什么 | 关键参数分布（已核） | 代码许可 | 底层数据许可 → 能否训练 | 对辅助模型的用处 |
|---|---|---|---|---|---|
| **WHAM!** | wsj0-2mix 两人全重叠 + 真实环境噪声 | 两说话人相对电平 0–5 dB；噪声 SNR（较响说话人对噪声，按 LUFS 算）**U(−6, +3) dB**；这个范围来自在同类场所实录对话后的估计；噪声里可懂人声用 iZotope 估出前景 SNR，**< −6 dB 的片段才收**（去掉约 5%）【论文 arXiv 1907.01160 §2】 | 脚本随数据发布 | 噪声 **CC BY-NC 4.0**【官方】；WSJ0 归 LDC 付费 → **不能训练** | 裁判噪声首选（咖啡馆 / 餐厅 / 酒吧实录）；全重叠混合本身不适用 |
| **WHAMR!** | WHAM! 加人工混响 | pyroomacoustics；房间长、宽 U(5,10) m，高 U(3,4) m；**T60 按噪声录音的主观混响程度分三档：高 U(0.4,1.0)、中 U(0.2,0.6)、低 U(0.1,0.3) s**；声源距麦 U(0.66, 2) m，高 U(0.9,1.8) m【论文 arXiv 1910.10279 表 1、§2】 | 同上 | 同上 → 不能训练 | 证实了「100–1000 ms」这个说法，但要补一句：是分三档、与噪声配对抽的，不是整段均匀抽 |
| **LibriMix** | LibriSpeech 2/3 人全重叠（+ WHAM! 噪声） | 每个说话人 U(−33, −25) LUFS（即 SIR 约 ±8 dB），噪声 U(−38, −30) LUFS【论文 arXiv 2005.11262 §2.2】 | MIT【官方 GitHub API】 | 语音 CC BY 4.0 可训练；带噪版含 WHAM!，不能训练 | 只取「干净混合」那部分思路；全重叠不像对话 |
| **SparseLibriMix** | 稀疏重叠的 2/3 人测试集 | 按 MFA 对齐出的子句拼接，每个子句单独抽电平（主导者会轮换），**重叠率 0/20/40/60/80/100% 各 500 条**，每条最长 15 s【论文 2005.11262 §2.2】 | MIT | 只有 test-clean，CC BY 4.0 | 「子句级拼接 + 每句重抽电平」可以借；它只出测试集 |
| **Landini 2022 simulated conversations（SC）** | 两人会话：按真实统计插入停顿与重叠 | 从真实会话估三种分布：同一人两段之间的停顿、换人时的间隙、换人时的重叠，按这些分布排时间轴；MUSAN 背景噪声 SNR 取 {5,10,15,20}；RIR 以 0.5 概率加。**SC 的重叠率 8.4%，SM 是 31.8%，真实 CALLHOME 11.8–13.1%**【论文 arXiv 2204.00890 §2、表 1】 | BUT 仓库，许可未核 | 依原语料 | **会话生成器最该借的思路**（读懂后自己写） |
| **DNS Challenge 合成器** | 单人语音 + 噪声（+ RIR） | SNR U[−5,20]、目标电平 −35~−15 dBFS、RIR T60 0.3–1.3 s【源码 `DNS-Challenge/noisyspeech_synthesizer.cfg:39-43,81-83`】 | 代码 MIT；仓库文档 CC BY 4.0【源码 `LICENSE`、`LICENSE-CODE`】 | 噪声：AudioSet（音频归 YouTube 版权）、Freesound **只选 CC0**、DEMAND（见 §3.3 冲突）；RIR：SLR26/28 Apache-2.0【源码 `README.md:265-292`】 | 噪声和 RIR 池能用；Freesound CC0 部分最干净 |
| **pDNS 合成器（DNS 4/5 个性化赛道）** | 主讲 + 第二说话人 + 噪声，三种组合 | 第二说话人对主讲、噪声对混合都从 [snr_lower, snr_upper] = **[−5, 20] dB** 取整数【源码 `pdns_noisyspeech_synthesizer_singleprocess.py:422-441`，`pdns_synthesizer_icassp2023.cfg`】 | MIT | 同 DNS | **唯一一个让干扰人声能比主讲响的公开配方**；主讲语音是一整段拼接，没有轮替结构 |
| **URGENT 2025 / 2026** | 通用语音修复：噪声 + 混响 + 失真 | SNR U(−5,20)；风噪 5%，SNR −10~15；混响概率 0.5；增广：限带、削波、mp3/ogg（vorbis、opus）编码 qscale 1–10、丢包 5–25%（20 ms 一包）；增广个数 0/1/2/3 的概率为 0.25/0.40/0.20/0.15【源码 `urgent2026_challenge_track1/conf/simulation_train.yaml:19-71`；2025 同值 `urgent2025_challenge/conf/simulation_train.yaml:58-103`】 | Apache-2.0 | 噪声 DNS5 / WHAM! / FSD50K / FMA，RIR 用 DNS5【源码 `urgent2025_challenge/README.md:79-85`】；**WHAM!、EARS 是 NC，FSD50K 部分 NC**（013 A-env）→ 剔除后可训练 | 失真增广的写法可以参考；**没有竞争说话人** |
| **MS-SNSD** | 单人语音 + 噪声，SNR 可配 | 用户自己配 SNR 档位【官方 README】 | MIT【GitHub API】 | 噪声来自 Freesound / DEMAND，逐条许可未核【二手】 | 被 DNS 取代，低优先 |
| **Hush** | DFN 改的「主讲 + 背景人声 + 噪声」 | **60% 的样本混入背景人声，SIR ∈ {24,20,18,16,14,12} dB（背景总是更轻）**【源码 `Hush/configs/default.ini:41-45`，`DATASETS.md:28`】；噪声 SNR {−100,−5,0,5,10,20,40}，−100 表示只有噪声【`default.ini:30`】；增益 {−6,0,6}【`:31`】；混响 p = 0.1（注释写 20%，与值不符）【`:35-36`，`DATASETS.md:45` 写 10%】；背景人声池与主讲**说话人不相交**【`DATASETS.md:20`】；训练前硬检查「主讲比干扰响多少」，不过线就报错【`training/train.py:638-651`】；背景头另有分离损失 sep_factor 0.1【`default.ini:112`】 | Apache-2.0 | 噪声表里有 **ESC-50（CC BY-NC 3.0）**【`DATASETS.md:38,73`】；它说 DNS 用「Microsoft Research License」【`:72`】，与 DNS 仓库自述（CC BY 4.0 / MIT）不一致 → 噪声表不能照搬 | 「说话人不相交池」和「训练前检查数据契约」值得学；**SIR 12–24 dB 恰好避开了我们最难的那一段**，C 线另议 |
| **SpatialScaper** | 多声源空间声景（SELD 任务） | 用实测 SRIR 摆放事件【论文 arXiv 2401.12238】 | **CC BY 4.0**（仓库已迁到 marl/SpatialScaper）【GitHub API】 | 默认的 TAU-SRIR 是 **other-nc**；METU SPARG 是 **CC BY 4.0**【官方 Zenodo API】 | 「近处 1–3 人摆位置」可以借思路；面向 FOA / 阵列，单麦用处有限 |

### 1.2 RIR 生成器与实测 RIR

| 工具 / 库 | 类型 | 许可 | 训练 / 裁判建议 |
|---|---|---|---|
| exp006 自写 `sim.py` | 直达声 + 6 个早期反射 + 三频带指数尾，RT60 0.08–1.6 s，临界距离固定 0.8 m【源码 exp006 README:20；013 A-env】 | 自有 | **训练** |
| FRA-RIR（Luo & Yu 2023，arXiv 2304.08052） | 随机近似的快速 RIR，可在训练时在线生成 | Apache-2.0【GitHub API】 | **训练**（与自写的同属统计模型；主要用来增加多样性） |
| pyroomacoustics | 镜像源法 + 射线追踪混合，房间几何显式 | MIT【GitHub API】 | **裁判**（几何法，和训练那两套统计法机理不同） |
| gpuRIR | GPU 镜像源法 | **AGPL-3.0**【GitHub API】 | 只在离线生成时用，不改、不分发；能不用就不用 |
| RIR-Generator（Habets） | 经典镜像源法 | MIT【GitHub API】 | 备选裁判 |
| SLR26 模拟 RIR / SLR28 实测 RIR 与噪声 | 模拟 / 实测（RWCP、REVERB、AIR 汇编） | Apache-2.0【官方 openslr】；SLR28 里汇编的原库许可未逐条核【推算有风险】 | SLR26 训练；SLR28 先别用 |
| dEchorate / METU SPARG / ACE | 实测 RIR | 都是 CC BY 4.0【Zenodo API】；ACE 另有 Imperial 页写 BY-ND（014 记过冲突） | dEchorate + METU **裁判**；ACE 只评测 |
| TAU-SRIR DB | 实测空间 RIR | other-nc【Zenodo API】 | 只评测 |
| MIT IR Survey / OpenAIR / BUT ReverbDB（Hush 用的） | 实测 | MIT Survey 许可未查到；OpenAIR 逐条不同；BUT 的许可没在本轮核 | 核实之前不进训练 |

### 1.3 模拟「笔记本麦 / 设备频响 / 编解码」的做法

| 做法 | 谁在用 | 我们怎么用 |
|---|---|---|
| 限带重采样、削波、mp3/ogg/opus 编码、丢包 | URGENT【源码 `simulate_data_from_param.py:233-340`】 | 我们的输入是麦克风原声，**编解码与丢包只进评测链**（同意 013 §1.1） |
| 频响倾斜 ±1.5 dB/oct、蓝牙窄带、削波 | exp006【013 A-env】 | 保留 |
| 高通、慢 AGC、频率相关的扩散衰减、谱减残余 | 013 §1.2 建议【推算】 | 保留；先用推算值，等 1.3 ① 回放重录后换成实测 |
| 用实测设备脉冲响应卷积（DCASE 的 TAU Mobile 是这类数据） | DCASE【官方 Zenodo：other-nc】 | 公开的设备 IR 多是 NC。**我们自己回放重录得到的 MacBook 麦 IR 最干净**（013 #4a），裁判集用实测、训练集用推算加随机，让两边不一样 |
| 参数 EQ 随机化（如 audiomentations 的七段 EQ） | audiomentations（MIT） | 训练集随机 ±3 dB/段【推算】，用来防止模型记住某一种频响 |

### 1.4 WHAM! 核实与 014 更正建议

| 项 | 014 卡现写 | 核实结果 | 建议改为 |
|---|---|---|---|
| 许可 | 未查到 | 「This work is licensed under a Creative Commons Attribution-NonCommercial 4.0 International License」【官方 wham.whisper.ai，证书失效，用 `curl -k` 读到】；LibriMix 论文同说【论文 2005.11262 §2.1】 | **CC BY-NC 4.0 → 只评测（D26）** |
| 命门 1 | 归「纯合成」出局（decision.md:42，must-know.md:27） | 噪声是真实场所的双耳实录（Apogee Sennheiser，三脚架高 1.0–1.5 m，前级增益为校准定值，44 处地点约 80 h）；语音是 WSJ0 人工叠上去的【论文 1907.01160 §2】 | **「部分」**（真实噪声 + 人工混合，正对 must-know.md:16 的定义）。只拿噪声那一包当真实噪声素材，不用它的混合 |
| 用途 | 无 | 噪声里可懂人声已经滤掉（前景 SNR 估计 < −6 dB 才收） | ① 裁判集噪声 ② 分布对齐时当「真实咖啡馆背景」的长时谱、4 Hz 调制、**绝对声级**（增益是校准过的）参考。注意：它**不含近处邻座**，补不了 D47 的核心格子 |
| 获取 | 证书失效 | 页面能打开，但证书错配（指向 github.io）【本次读取】 | 写明下载前再核一次地址 |

---

## 2. 对 D47 辅助小模型最关键的合成要素

### 2.1 真实会话的重叠与轮替（有出处的数）

| 场景 | 重叠率 | 出处 |
|---|---|---|
| 两人电话（CALLHOME 1 / 2，DIHARD3 dev / eval） | 13.1 / 11.8 / 12.2 / 10.5%（同时静音约 9–11%） | 【论文 Landini 2022 arXiv 2204.00890 表 1】 |
| 会议：AMI / AliMeeting / AISHELL-4（测试集） | 14.6 / 20.4 / 5.0% | 【论文 REAL-T，Interspeech 2025，表 1，用 BUT diarization_utils 算】 |
| 晚餐聚会：DipCo / CHiME-6（测试集） | 27.5 / 33.9% | 同上 |
| 自然会议与晚餐聚会的概括 | 「一般 ≤ 20%」 | 【论文 LibriMix 2005.11262 §1 转引】 |
| 换人时的间隔 | **约 40% 的换人间隔是重叠**；55–59% 的换人间隔 < 200 ms | 【论文 Heldner & Edlund 2010，J. Phonetics 38:555】 |
| 笑声 | ICSI 会议里约占发声时间的 10%（各子集 6–15%） | 【论文 Laskowski & Burger 2007，来自摘要与转引；原文表格未核】 |
| 附和声（嗯 / 对） | Switchboard 对话行为里约占 19% 的话语 | 【二手，Stolcke 2000 SWBD-DAMSL，本轮未核原文】 |

**推论**：「你和邻座」不在同一个对话里，两人的重叠是**互相独立的两条时间轴撞上**，不是轮替。所以要分两种生成：①你那条会话（你 + 远端 / 同桌的人）按上表的轮替统计排；②邻座那条另起一条会话独立排，两条相加。重叠率随两人各自的说话占比变化【推算】，用不着单独设参数。

### 2.2 近讲 vs 邻座：电平与 DRR（全部【推算】，等回放重录校准）

| 量 | 你（嘴到麦 0.3–0.6 m） | 邻座（0.7–2.0 m，侧向 / 背向） | 算法 |
|---|---|---|---|
| 直达声电平差 | 0 dB（参照） | 比你低 3–16 dB（只算距离） | 20·log10(d邻/d你) |
| 嘴部指向性 | 正对 | 侧向时高频再低约 3–8 dB | 【二手】语音指向性随频率升高 |
| DRR | 约 +3~+13 dB | 约 −6~+4 dB | DRR ≈ 20·log10(rc/d)，rc 取 0.64–1.97 m（013 §1.2） |
| 说话人自身响度 | ±6 dB，按 Lombard 上抬 | ±6 dB，独立抽 | 013 §1.2 |

所以「你 vs 邻座」的 SIR 自然落在约 −6~+20 dB：邻座嗓门大或者坐得近，就会比你响。**DRR 不单独抽，用 RIR 和距离推出来**，免得电平和 DRR 互相矛盾。

### 2.3 合成参数表建议（与 013 对比）

| 量 | 013 现设计（decision §1.2、§4.2） | Hush | 本建议 | 差异 / 理由 |
|---|---|---|---|---|
| 会话结构 | LibriSpeech 1–3 人拼接，**加重叠段** | 整段叠加 | **两条独立会话时间轴**；子句按 MFA 边界切开，换人间隔从真实分布抽（约 40% 为重叠，中位数约 200 ms 量级） | 新增。拼接 + 随手加重叠会把重叠率做到 SM 那样的约 32%，真实只有 10–34%（§2.1） |
| 每段重叠率 | 未定 | 不控制 | 训练：均值 15–25%，范围 0–50%；裁判：按 §2.1 的场景混合另抽 | 新增 |
| 你 vs 邻座 SIR | 只有干扰者距离 1–4 m、增益 0.8/d·U(0.6,1.0) | 12–24 dB | **−6~+20 dB，约 20% ≤ 0 dB**，由距离 + 响度推出，不单独抽 | 新增。负 SIR 是专门查「只靠响度判」这条捷径的 |
| 邻座距离 | 近层 1–3 m | — | 0.7–2.0 m，方向 0–180°，侧向 / 背向时加指向性衰减 | 比 013 更近，对应 D47 说的「邻座」 |
| DRR | 由 RIR 隐含决定 | — | 由距离 + rc 推出，报告里给分布 | 只是把它显式写出来 |
| 远处 babble | 10–40 人扩散场；N ∈ {1,2,3,6,12,24} | 无 | 不变 | 一致 |
| 背景声级 / SNR | 60–85 dBA，SNR 由声级联动，加 Lombard | SNR 离散 7 档 | 不变；用 WHAM! 的校准声级核这个区间 | 补一个校准源 |
| 词首 | V-SIM-GATE 把词首 200 ms 单列 | 无 | **≥ 30% 的段落里，你在邻座说话途中开口**；词首轻读不另做衰减，靠素材本身自然的轻读，并按词首的能量分桶报告 | 新增采样约束 |
| 附和 / 笑声 | 无 | 无 | 你和邻座都插入 0.2–0.8 s 的附和、0.5–2 s 的笑声；素材用 AMI 近讲（CC BY 4.0，有标注）；**笑声算不算「你」待拍板（#3）** | 新增 |
| 语料 | LibriSpeech + AISHELL-1 + MLS | Libri / VCTK / CV | 再加 AMI 近讲轨补对话体（要先剔除串音段）；所有角色说话人不相交，背景人声池、裁判池再单独划开 | 学 Hush 的不相交池 |
| 混响 | exp006 自写 RIR，RT60 0.08–1.6 s | p = 0.1 | 训练：自写 RIR + FRA-RIR，用 p = 1（你和邻座分别过 RIR）；裁判：换 pyroomacoustics + 实测 | 013 已定「换生成器」，这里点名 |
| 设备链 | 高通 / AGC / 扩散衰减【推算】 | 无 | 不变，另加 EQ 随机化；裁判集用回放重录实测的 IR | 让训练和裁判的设备链不同 |
| 冷启动 | 20% 置零声纹，「别人」标签改成「你」 | — | 不变 | 一致 |
| 含真人说话的实录噪声 | 不进训练 | — | 不变；WHAM! / DEMAND 只进裁判，帧标签记「背景」，不计入 V-SIM-FA 的误开 | 补了裁判集里的标签规则 |

---

## 3. 防 sim-to-real 落差与防讨好奖励

### 3.1 两套生成器：具体选型

| 部件 | 训练集（A） | 裁判集（B，只评测，从不进训练和参数筛选） |
|---|---|---|
| RIR | 自写 sim.py + FRA-RIR（都是统计模型） | pyroomacoustics 几何法 + dEchorate / METU 实测 |
| 噪声 | MUSAN、FSD50K 只取 CC0 / CC BY 的片段、DNS5 中 Freesound CC0 部分、exp006 合成噪声 | WHAM!、DEMAND、TAU Urban（NC，只评测） |
| 会话时间轴 | 自写轮替生成器（参数取 CALLHOME / AMI 统计） | 另一种：SparseLibriMix 式子句拼接（固定重叠率档位） |
| 语音 | LibriSpeech / LibriTTS / AISHELL-1 / AMI 近讲 | VCTK、Common Voice（CC0），说话人与训练不相交 |
| 设备链 | 推算参数 + EQ 随机 | 回放重录实测 IR（013 #4a） |

### 3.2 可以干净训练的噪声 / RIR 源（排除 NC 与 SA）

| 源 | 许可 | 备注 |
|---|---|---|
| MUSAN | CC BY 4.0【官方 openslr 17】 | 013 已列 |
| FSD50K | 逐片段混合许可（Zenodo 标 other-at）【官方】 | 按片段元数据只取 CC0 / CC BY |
| DNS5 噪声里的 Freesound 部分 | CC0（只选了 CC0）【源码 DNS README】 | AudioSet 部分的音频归 YouTube 版权，不进 |
| SLR26 模拟 RIR | Apache-2.0 | 可以 |
| dEchorate、METU SPARG | CC BY 4.0 | 建议留给裁判 |
| **DEMAND** | ⚠️ 冲突：DNS README 写 CC BY-SA 3.0；Zenodo 记录 1227121 现标 **cc-by-4.0**【官方 Zenodo API】 | 许可要交 014 复核；但它含真人说话，013 本来就不让它进辅助模型训练 |

### 3.3 合成训、真实测：查到的落差数字

| 研究 | 合成上 | 真实上 | 说明 |
|---|---|---|---|
| **PVAD 类** | PVAD 1.0：目标人 AP 0.932（无增强）/ 0.878（加噪声与混响增强）【013 F-pvad】 | **没找到公开的真实会话数字**；PVAD 2.0 自己说没有合适的公开数据集【二手搜索】 | 这是空白。014 的 CHiME-6 / AliMeeting 转写派生标签集正好能填 |
| EEND 说话人日志（逐帧判谁在说，和 PVAD 最近） | — | CALLHOME DER：SM 训 15.09%；SC 训 8.64%；SM 再用真实数据微调 8.44%；SC 再微调 7.28%【论文 2204.00890 表 2】 | **只把合成数据的时间结构改像真实会话，就补回了几乎全部「要真实数据微调才有」的收益** |
| 目标说话人提取（TSE） | Libri2Mix SI-SDR 12.95–18.05 dB | REAL-T 真实会话上 TER 57.6–91.2%（中 / 英）【论文 REAL-T 表 5】 | 合成分数高，真实识别几乎全错 |
| DNS 2020 | 合成测试集好 | 真实录音明显下滑，客观指标与主观评分相关性差【013 A-env，arXiv 2005.13981】 | — |

### 3.4 防「讨好模拟器」的具体招

| 风险 | 招 |
|---|---|
| 模型学会「更响的就是你」 | 裁判集固定留一块 SIR ≤ 0 dB；**辅助模型在这一块的 AP 不许比全集低超过 X**（X 待定，§要人拍板 #2）；再加一道探针：把声纹换成邻座的，输出也该跟着反过来 |
| 学会 RIR / 噪声库自己的特征 | 两套生成器（3.1）；每版报训练集 → 裁判集的 AP 落差，落差变大就先修模拟器（013 §1.3 ④ 同理） |
| 学会真值规则本身 | 真值规则（帧长、门限、hangover）写死进仓，只在干净轨上算（012 §3.4 限定 3） |
| 反复按裁判选模型，把裁判也拟合了 | 裁判集轮换、限制查询次数（013 §2.3） |
| 绝对声级不对 | 用 WHAM! 的校准声级分布 + 014 的 CHiME-4 CAF / ACE 做 013 §1.3 ③ 的分布对齐 |

---

## 4. 合成数据怎么接强化学习

| 环节 | 内容 |
|---|---|
| 每一步（一个 episode） | 生成器按当前课程档位抽场景参数 θ（SIR、重叠率、N、RT60、设备链），输出：麦克风混合、分轨（你 / 邻座 / babble / 噪声）、按 256 hop 对齐的逐帧三类标签、换人时刻表、词首帧掩码。段长：GATE / FA 用 4–8 s，SWLAT 用 ≥ 30 s 的序列（要有多次换人） |
| 策略输出 | 辅助模型逐帧给三类概率 → 门控（滞回 / 保持时长）→ 开关序列。**序列这一步不可微**，是用 RL 的唯一理由；逐帧概率那一层直接用监督学习 |
| 奖励（全部来自模拟真值，012 登记为 L3 仅限模拟） | V-SIM-GATE：你说话的帧被误关的比例，**词首 200 ms 被误关即一票否决**（奖励直接压到最低）；V-SIM-FA：只有别人在说的帧被误开的比例；V-SIM-SWLAT：换人后门控跟上的延迟，P90 ≤ 300 ms。合起来是 −(w₁·GATE + w₂·FA + w₃·max(0, SWLAT−300 ms))，权重在训练前锁定 |
| 锚点 | 保留交叉熵 + WPL 监督损失防漂；按 013 §2.3 停训（裁判分一转跌就停） |
| 先黑盒再 RL | 013 §4.4 已定：先用 Optuna / CMA-ES 调门控的 3–4 个参数，监督训练进入平台期后才上策略梯度。本线不改这个顺序 |

**课程学习（SIR 由易到难）有没有证据**

| 证据 | 结果 | 能不能用到我们这 |
|---|---|---|
| Liu 2024 TSE 课程学习（Interspeech，arXiv 2406.07845） | 按性别、声纹相似度、SDR 由易到难排，比不用课程高**约 1 dB**（Libri2talker，全在模拟上） | 方向一致；只在模拟上测过 |
| Liu 2026 多因素课程（arXiv 2603.04943） | SNR 门限、说话人数、重叠率、合成 / 真实比例一起排，摘要说优于随机采样，多人场景提升最大；摘要没给数 | 正好对上我们的四个旋钮；预印本 |
| Braun 2017 噪声 ASR（综述 arXiv 2101.10382 转引） | **反课程**（从低 SNR 开始）更好 | 方向相反 |
| ADR（arXiv 1910.07113，013 已引） | 成功率过阈值就自动放宽范围 | 013 只准用在策略层 |

**判断**：证据薄，方向也不一致，提升约 1 dB 量级。课程学习当作**可选的超参数**：和「全范围均匀混合」做一次 A/B，用 V-SIM-* 定，不预设它一定有用。负 SIR 段不能留到课程最后才出现，从第一档起就保留 ≥ 10% 的样本【推算】，否则模型会先学会「靠响度判」这条捷径。

---

## 要人拍板的

| # | 题 | 选项 | 代价 | 建议 | 默认 |
|---|---|---|---|---|---|
| 1 | 训练用 SIR 下限 | A −6 dB（约 20% ≤ 0）· B 0 dB · C 照 Hush 12 dB | A 难训，但能盖住 exp003 的 −5 dB；C 正好避开攻坚格子 | **A** | A |
| 2 | 「只靠响度判」探针的过线门槛 X（SIR ≤ 0 那一块的 AP 跌幅） | A 0.05 · B 0.10 · C 只报告不设门槛 | A 可能卡住第一版 | **B**，第一版跑完再收紧 | B |
| 3 | 你自己笑、附和时，标签算不算「你」 | A 算（门要开）· B 算「无人声」 · C 单独一类，不计分 | A 会把附和送出去；B 会吃掉「嗯，对」 | **A**（会议里附和本身就是要传过去的内容）【假设】 | A |
| 4 | 会话体素材用 AMI 近讲轨进训练 | A 用（CC BY 4.0，剔除串音）· B 只用 LibriSpeech 朗读 | A 要写串音剔除规则 | **A** | A |
| 5 | 014 的 WHAM! 卡改为「CC BY-NC 4.0 · 命门 1 = 部分 · 只评测」，并把它列为 015 裁判噪声 | A 改 · B 维持「纯合成出局」 | A 要下载 76 GB 里的噪声那部分（16 kHz 版更小，大小未查） | **A** | A |
| 6 | DEMAND 许可冲突（BY-SA 3.0 vs Zenodo CC BY 4.0） | A 按更严的 BY-SA 处理 · B 交 014 复核后再定 | — | **A**，直到复核完 | A |

## 来源

- 【官方】WHAM! 页面 License 段与录音说明：https://wham.whisper.ai/ （证书错配，`curl -k` 读取，2026-10-31）
- 【论文】Wichern 等，WHAM!，Interspeech 2019，arXiv 1907.01160 §2
- 【论文】Maciejewski 等，WHAMR!，ICASSP 2020，arXiv 1910.10279 表 1、§2
- 【论文】Cosentino 等，LibriMix，arXiv 2005.11262 §1、§2.1–2.2
- 【论文】Landini 等，From Simulated Mixtures to Simulated Conversations，Interspeech 2022，arXiv 2204.00890 表 1–2
- 【论文】Li 等，REAL-T，Interspeech 2025，doi 10.21437/Interspeech.2025-2662，表 1、表 5
- 【论文】Heldner & Edlund 2010，Pauses, gaps and overlaps in conversations，J. Phonetics 38(4):555–568
- 【论文 / 二手】Laskowski & Burger 2007（笑声约占 10%，来自摘要与转引）；Stolcke 2000（附和约 19%，未核原文）
- 【论文】Liu 等 2024 arXiv 2406.07845；Liu 等 2026 arXiv 2603.04943；Soviany 等课程学习综述 arXiv 2101.10382
- 【论文】Luo & Yu，FRA-RIR，arXiv 2304.08052；Roman 等，SpatialScaper，arXiv 2401.12238
- 【源码】主检出 `resources/`：`Hush/DATASETS.md:20,28,38,45,72-73`、`Hush/configs/default.ini:30-45,112`、`Hush/training/train.py:578-651`；`DNS-Challenge/noisyspeech_synthesizer.cfg`、`pdns_synthesizer_icassp2023.cfg`、`pdns_noisyspeech_synthesizer_singleprocess.py:422-441`、`README.md:265-292`；`urgent2026_challenge_track1/conf/simulation_train.yaml:19-71`、`simulation/simulate_data_from_param.py:233-340`；`urgent2025_challenge/conf/simulation_train.yaml:58-103`、`README.md:79-85`
- 【官方】许可查询：GitHub API（pyroomacoustics MIT · gpuRIR AGPL-3.0 · FRA-RIR Apache-2.0 · LibriMix / SparseLibriMix / MS-SNSD MIT · SpatialScaper CC BY 4.0 · RIR-Generator MIT · audiomentations MIT）；Zenodo API（TAU-SRIR other-nc · METU CC BY 4.0 · DEMAND cc-by-4.0 · FSD50K other-at · TAU Urban 2019/2020 other-nc · dEchorate CC BY 4.0 · ACE CC BY 4.0）；openslr 17/26/28
- 本仓：`docs/research/013_rl-environment/decision.md`（§1.1–1.3、§2.3、§4）、`notes/A-env.md`、`notes/A-env-skeptic.md`、`notes/F-pvad.md`；`docs/research/014_real-datasets/decision.md:42`、`must-know.md:16,27`、`cards/wham-noise.md`；`docs/research/012_eval-standards/verifiers.md:122-124`；`docs/experiments/exp006-scene-sim/README.md:19-20`
