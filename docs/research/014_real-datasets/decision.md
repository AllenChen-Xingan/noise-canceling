# 取舍卡 · 真实回归集选哪几个公开数据集

> 2026-10-31 · 手下 datasets。依据：`cards/`（51 张，每张带出处与证据等级）· `notes/`（五路调研 + 五个怀疑派的更正与漏项）· `gap.md`。
> Workflow `wf_5c707fee-0a3`：5 路调研 × 每路 1 个怀疑派（10 代理，**未下载任何数据集本体、未运行任何程序**）；RealMAN、NISQA LIVETALK 由本会话补核。
> 已过三视角自审（事实 · 规矩符合度 · 实现）：事实层无 P0；改了「转写派生标签」的做法、两处替人定的默认、DR-VCTK 许可、获取步骤与大小。
> 引用位置：D42 / D47 / D39 见主目录 `docs/plan/001_decisions-20261029-b1.md:231,273`（本分支的 001 还没有这几条）；013 / 012（含 `verifiers.md` §3.4）以本分支所基于的 main `fc5418f` 为准。
> 规则：D19（含旁人声音的录音只存本机）· D22（数据放仓外，仓里只记路径 · 哈希 · 来源 · 许可）· D26（SA / NC 只评测）· 013「真实录音只做回归，永不进训练」〔已被 D63 修改（2026-11-01）：测试份永不进训练，训练份可做合成原料与训练〕。

## 一句话

**没有任何公开数据集是「笔记本内置麦 + 咖啡馆 + 有参考」**（五路全网搜，含 extended 模式，结论一致）。最贴近我们场景的是 **DNS Challenge（ICASSP 2022 / 2023）的真实测试集**：dev 全部用笔记本或台式机录（是否内置麦论文没说），有近讲与 1 / 2 / 3 m 三档，个性化 dev 里有 513 条「邻座同时说话」（盲测附使用者注册语音，dev 待核）；2023 盲测集还明列了咖啡馆和背景 babble。代价是没有干净参考、没有逐条场景标签，许可也没单独写。所以真实回归要**拼一个组合**：DNS 管「设备 + 邻座」，CHiME-6 和 AliMeeting 管「真实重叠 + 近讲参考 + 转写」（英 / 中），DR-VCTK 管「MacBook 内置麦的设备链」。**咖啡馆那一格，公开数据补不上**：建议 013 拍板 #4b 选 A（作者本人实录）；不选的话，真实一侧的咖啡馆格就是空的，报告里写明。

## 一、首选组合（5 个，按「接近度 × 参考 × 许可可得」排序）

| # | 数据集 | 管哪一格 | 参考 | 许可 → 结论 | 获取 | 下哪部分 · 大小 | 用途 |
|---|---|---|---|---|---|---|---|
| 1 | **DNS Challenge 4 / 5 真实测试集**（含个性化赛道）[卡](cards/dns4-icassp2022-real-testset-pdns.md) · [卡](cards/dns5-icassp2023-real-testset.md) | D42 设备链（dev：笔记本 / 台式机；手机只在盲测里）+ D47 邻座 | 转写：DNS4 dev 有公开转写稿 `WAcc/DNSChallenge4_devtest.tsv`（无时间戳，**只在 `v4dnschallenge_ICASSP2022` 分支**，master 已 404；覆盖哪些子集未写，用前先核）；盲测已转写但未公开 · 注册语音：盲测每人 2.5 min（DNS4）已证实，dev 是否附带待核；DNS5 写 10–30 s（正文写 30 s）· 无干净参考 · 无逐帧说话人标签 | 仓库原句：数据「provided under the original terms that Microsoft received such datasets」，测试集没单列 → **只评测、只存本机** | 直接下载（Azure Blob，仓库脚本） | DNS4 dev（930 + 1443 条，两档共享 193 条情感语音，去重约 2180 条）+ DNS5 盲测；DNS4 与 DNS5 的个性化 dev 疑为同一批，**只下一份**；大小只查到 DNS4 dev 包约 1.7 GB，DNS5 盲测下载前用 `curl -sI` 读 Content-Length | **只作吃字与人耳回归，不作辅助模型的标签集**（没有参考和逐帧标签，FA / GATE 算不了）：① 个性化 dev 的「主 + 邻座」347 条、「主 + 邻座 + 噪声」166 条，看整条链路下主说话人吃字（若 tsv 覆盖）和人耳判断邻座残留；② 人耳听测：从 DNS5 盲测**人工听选**咖啡馆 / babble 段。48 kHz 要先重采样到 `FeatureExtractor` 口径 |
| 2 | **CHiME-6**（CHiME-5 晚餐聚会重发）[卡](cards/chime5-chime6.md) | 真实多人重叠 + 嘈杂家庭聚餐（英） | 每人双耳近讲麦（Soundman OKM II）· 人工转写带时间戳与说话人 | CC BY-SA 4.0（2024-01-01 重发，官方写学术与商业均免费）→ **只评测**（D26） | 直接下载 openslr.org/150 | 只下 dev 11 GB + eval 12 GB（共约 9.7 h） | ① 真实回归：吃字率（转写）；② **辅助小模型的「转写派生标签」验收集**（按第三节的做法，按教师标签走 012 §2 定标，不是真值），量 012 §3.4 限定 1 要求的「模拟 → 真实」AP 落差；③ sim-to-real：重叠率、电平分布 |
| 3 | **AliMeeting**（M2MeT）[卡](cards/alimeeting.md) | 中文真实重叠（D47 中文） | 每人头戴近讲（Test 包也含）· TextGrid 时间戳转写 | CC BY-SA 4.0 → **只评测** | 直接下载 openslr.org/119（阿里云 OSS） | 只下 Eval 3.42 GB + Test 8.90 GB（14 h） | ① 中文吃字率回归；② 同 #2 ②，中文版的转写派生标签；③ 重叠率高（Eval 约 34%），专测「你和邻座同时说」。最小起步只下 Eval |
| 4 | **DR-VCTK**（Device Recorded VCTK）[卡](cards/dr-vctk.md) | MacBook 内置麦设备链 | 逐样本平行的干净源 | 包内 README 原句「You are free to use this database under Creative Commons Attribution License (CC-BY)」；datashare 页另附 End-user Licence **未读到** → **待核**，核完前按只评测 | 直接下载 datashare.ed.ac.uk/handle/10283/3038 | 全量 1.67 GB | **sim-to-real ① 的先验**，只给 **8 kHz 以下的相对频响**（且混着 Bose 音箱和房间响应，拆不开）。**估不出**：8 kHz 以上（发布版 16 kHz）、AGC（固定电平回放）、绝对灵敏度、麦自身底噪（静音段已裁、混窗外噪声）、系统语音处理、M 系列三麦波束成形。若 013 拍板 #4a 选做，本机扫频实测覆盖它 |
| 5 | **CHiME-3 / 4 真实部分（咖啡馆 CAF）** [卡](cards/chime3-chime4-real.md) —— 或替补 **ACE 的 Chromebook 档** [卡](cards/ace-challenge-corpus.md) | D42：设备麦 + 真实咖啡馆 | CHiME-4：近讲 ch0 + WSJ 句级转写 · ACE：无（消声室干净源需自己卷积相加） | CHiME-4：LDC User Agreement for Non-Members → 不可再分发、**付费**（LDC2017S24）· ACE：Zenodo 写 CC BY 4.0、Imperial 页写 CC BY-ND 4.0，冲突 → **只评测** | CHiME-4：LDC 付费（待人拍板）· ACE：直接下载 zenodo.org/records/6257551 | CHiME-4 CAF 真实段 train 0.76 h + dev 0.69 h · ACE Chromebook 档 1.1 GB | sim-to-real ③ 分布对齐：咖啡馆 / 大堂 babble 的长时谱、电平、4 Hz 调制深度；CHiME-4 还能做「平板麦 + 咖啡馆」的吃字回归。ACE 是**唯一查到的真笔记本内置麦 + 真人 babble**（4–7 人朗读，大楼大堂带咖啡厅） |

**首批下载量**：#1 约几 GB【推算】+ #2 23 GB + #3 12.3 GB + #4 1.67 GB + ACE 1.1 GB ≈ **40 GB 上下**，超出 D22 估的 15–20 GB；**解压时压缩包与文件并存，峰值约 2 倍**，还要和 013 的训练语料（6–70 GB）抢盘（见拍板 #3）。中间版：#1 + #2 只下 eval（12 GB）+ #3 只下 Eval（3.42 GB）+ #4 + ACE ≈ 20 GB。最小版（不越 D22）：#1 + #4 + ACE，几 GB。

## 二、次选与特定用途（不进首批）

| 数据集 | 为什么不进首批 | 什么时候用 |
|---|---|---|
| **CHiME-9 ECHI** [卡](cards/chime9-echi.md) | 真实 4 人围桌对话 + Aria 眼镜 / 助听器 + 近讲，很贴 D47；但食堂 babble 是 14 只音箱回放的朗读语料，且要签 DUA、组织方审批，原句含「禁再分发、禁上传公共仓库 / 云」，eval 未发放 | 拍板 #2 签了以后，作「一大片人 + 近距离对话」的听测与回归 |
| **DiPCo** [卡](cards/dipco.md) | 真实晚餐 + 近讲 + 转写，**CDLA-Permissive-1.0 可训练**；但只有 5.3 h，与 CHiME-6 同类 | 想要一份许可最干净的晚餐回归集时（许可上允许再分发，但按 D19 片段仍不外发）；按 013 不进训练 |
| **AMI** [卡](cards/ami.md) | 安静会议室，不贴 D42；但 CC BY 4.0 且官方带**词级时间戳**，是做「转写派生标签」最省事的英文集 | 第三节标签做法先在 AMI 上标定「标签误差对 AP 的影响」，再上 CHiME-6；只下 headset + 一路阵列的 eval 部分（大小未查到） |
| **LibriCSS** [卡](cards/libricss.md) | 回放重录（部分真实）；但源句已知，可强制对齐到采样点级，10 h、体积小 | 同上，作「标签几乎无误差」的对照 |
| **NOTSOFAR-1** [卡](cards/notsofar-1.md) | CC BY 4.0、真实会议、头戴近讲、商用会议设备；但不是笔记本麦，背景不是大片人声，SC 流设备内部已处理过（要用 MC 原始流） | 英文会议回归；许可最干净的大集 |
| **REAL-M** [卡](cards/real-m.md) | 笔记本 / 手机众包的真实双人同时说话，有句子转写；但无参考、许可未查到 | D47 人耳听测补充 |
| **EasyCom** [卡](cards/easycom.md) | 眼镜 + 回放餐厅噪声（约 71 dB SPL）；CC BY-NC 4.0；**佩戴者本人没有近讲** | 听测素材 |
| **MCoRec** [卡](cards/mcorec.md) · **MMCSG** [卡](cards/mmcsg.md) | MCoRec 一屋最多 8 人 4 组对话同时进行，是「一大片真人」最真实的；但 360° 相机麦、dev / eval 无近讲、自定义协议、229 GB · MMCSG 许可原文未读到 | 拍板 #2 一并决定 |
| **ASPIRE（Stirling）** [卡](cards/aspire-stirling.md) | 真实食堂午餐高峰 + iPad mini + 领夹；许可字段自相矛盾（Non-Commercial Government Licence 链向 Apache-2.0）→ 只评测；Grid 句式 | 咖啡馆 / 食堂 babble 校准的第二来源（13 GB） |
| **RealMAN** [卡](cards/realman.md) | 中文、CC BY 4.0、真实场景噪声 144.5 h；但语音是音箱回放、噪声分开录，场所名未查到 | 中文噪声分布校准（只下 test 39.3 GB）；若以后要真实噪声训练库，它是许可最干净的中文候选 |
| **NISQA LIVETALK** [卡](cards/nisqa-livetalk.md) | 德语、经通话链路；但**真实咖啡馆 / 商场 + 笔记本 VoIP，带每条 24 票人耳 MOS** | 012 的 V-NRMOS（无参考 MOS）在真实嘈杂录音上的定标旁证 |
| AMI · ICSI · AISHELL-4 · LibriCSS · VOiCES · DAPS · BUT ReverbDB 等 | 安静会议室 / 回放重录 / 无公开近讲 | 见各卡 |

**明确不用**：纯合成集（WHAM!、LibriMix、DNS 合成训练集）——命门 1 出局；URGENT 2024–2026 盲测真实部分——没有参考、没有标签、来源未点名；Common Voice——人工抽查只有 0.4% 有明显背景噪声。

## 三、怎么接 013 与 012

**接 013 §1.3 sim-to-real 四步**

| 013 步 | 公开数据能做什么 | 还缺什么 |
|---|---|---|
| ① 回放重录（设备链） | **DR-VCTK** 先给 MacBook Air 内置麦的频响 / 底噪先验；ACE Chromebook 档给笔记本麦 + 真实 RIR | 本机型（作者的 MacBook）实测仍要做（013 拍板 #4a），公开数据机型老、16 kHz |
| ② 咖啡馆实录 | **补不上**。最近的是 CHiME-4 CAF（平板、朗读、付费）和 DNS5 盲测里人工听选的咖啡馆段（无参考） | 作者实录（013 拍板 #4b）仍是唯一的「笔记本麦 + 咖啡馆 + 同步近讲」来源 |
| ③ 分布对齐 | 长时谱 / 电平 / 调制深度：CHiME-4 CAF、ACE 大堂 babble、ASPIRE 食堂、ARTE 咖啡馆 HOA（无语音，只作噪声底）；重叠率与说话人数：CHiME-6、AliMeeting | — |
| ④ 排名相关门槛 | 「真实一侧」的候选排名：在 #1–#3 上跑候选，用吃字率 + 人耳 | 人耳那一半仍要听测 |

**接 012 verifier 定标**

- **V-EAT-ASR（吃字）**：只有带转写的才能算——CHiME-6、AliMeeting（时间戳）、DNS4 dev（转写 tsv，先核覆盖）、CHiME-4。今天就能在真实录音上**算**，但按 012 定标前仍是 L0，只作约束 / 否决。
- **V-SIM-GATE / FA / SWLAT 的真实落差**（012 §3.4 限定 1）：CHiME-6 / AliMeeting 的转写只有**句级**起止（人工在佩戴者双耳麦上标，精度未给），近讲麦**串音**，不能当真值。做法（按教师标签处理，规则事先冻结进仓，同 013 §1.1）：
  1. **标签** = 转写段 ∩ 该人近讲上的词级强制对齐（对齐工具与阈值先冻结）；每个边界前后 ±100 ms 记为不计分区；`[redacted]` 段、缺失音频（如 CHiME-6 S05 的 U03 / U04、S12 的 U05 末 15 分钟）、未转写的笑声附和一律不计分。近讲能量不作真值。
  2. **输入**固定取阵列 ch0（Kinect / AliMeeting 8 通道的第 0 路）。这些集里所有人到麦的距离差不多，不像笔记本「你近、邻座远」，所以**只评辅助小模型本身（纯声纹条件），不评整条门控**（近讲线索在这里失效）。
  3. **注册**：用指定「使用者」在别的时间段的单人段（取自转写时间戳），与计分区不重叠；p2 的自动注册另报一行（它会锁在场说话最多的人，未必是指定者）。
  4. **同步核查**：每场每 30 s 做一次近讲与 ch0 的互相关求偏移曲线，残余偏移超过 1 帧（16 ms）的场次剔除或分段校正，偏移曲线随 build 进仓。
  5. **报告**：逐帧只报 AP 与误开（FA）；词首误关与换人延迟只报不计分区以外的部分，并标注「转写派生标签，非真值」。先在 AMI（词级时间戳）/ LibriCSS（源句已知）上量标签误差对 AP 的影响。
- **感知类 verifier**（V-NRMOS 等）：公开集只能给听测素材；NISQA LIVETALK 的人耳 MOS 可作旁证，定级仍按 012 §2 由作者听测决定。
- **听测页素材**：只从「只评测」集中切片，音频放 `_tmp/<实验>/<build-id>/`，揭盲表进 `docs/experiments/`（CLAUDE.md 第五节第 1 条）。

## 四、获取步骤（人拍板后由开发方执行；本线一律未下载）

1. 下载前先 `df -h`（自审时剩约 174 GiB）。数据放 `~/vdn-data/real/<数据集>/`（D22）。仓里清单**建议**放 `docs/datasets/real/<数据集>/manifest.tsv` + 同目录 `SHA256SUMS`（压缩包与逐文件各一份），每条记 **路径 · `shasum -a 256` · 来源 URL · 许可原句 · 下载日期**；目录由下一批定，首个数据集下载前建好。
2. 直接下载类（#1–#4、ACE）：**不要克隆 DNS 仓库**（体积大），用 `curl` 只拉下载脚本或 blob URL，只拉测试集；DNS4 转写 tsv 按 `v4dnschallenge_ICASSP2022` 分支取。CHiME-6 / AliMeeting 只拉 dev / eval（/ test）包。几乎每个包都超过 500 MB：按本批规矩（单次下载 > 500 MB 先问大总管）**首批合并成一次申请**。解压、校验哈希后删压缩包，只留哈希。
3. 签协议 / 付费类（CHiME-4、ECHI、MCoRec、MMCSG）：等拍板 #1 #2，**不代办**。
4. 所有片段只存本机、不入库、不外发（D19）；SA / NC / 许可未查到的，只进评测，不进任何训练集（D26）。

## 五、我放弃什么

放弃「找一个公开集直接当 D42 的验收集」：全网没有。公开数据只能把咖啡馆格子**两边夹住**——设备链（DNS、DR-VCTK、ACE）和真实嘈杂多人（CHiME-6、AliMeeting、ECHI），中间那块靠作者实录。也放弃用 CC BY 的真实集训练（NOTSOFAR、DiPCo、RealMAN 许可上可以），守 013「真实录音永不进训练」，否则回归集就不独立了。

## 六、待人拍板

| # | 题面 | 选项 | 代价 | 建议 | 默认（不拍时） |
|---|---|---|---|---|---|
| 1 | 买不买 CHiME-3 / 4（LDC2017S24）？唯一「设备麦 + 真实咖啡馆 + 近讲」 | A 买 · B 先不买，用 ACE Chromebook + DNS5 盲测听选代替 · C 不买 | A：LDC 非会员费用要登录才看得到（未查到），签 LDC 协议、不可再分发 | **B**：第一轮回归若发现咖啡馆段不够再买 | B |
| 2 | 签不签 CHiME-9 ECHI 的 DUA（及 MCoRec、MMCSG 的自定义协议） | A 签 ECHI · B 三个都签 · C 都不签 | 需 HF 账号 + 机构邮箱 + 审批；条款限学术 / 算法开发与评测，商用另谈，禁再分发与上传公共云 | **A**：免费，最贴 D47 + 大片人声；只本机评测，与 D39 / D19 不冲突。若项目以后商用要重看 | C |
| 3 | 首批下多少（D22 原估 15–20 GB）| A 全量约 40 GB（解压峰值约 80 GB）· B 中间版约 20 GB · C 最小版：#1 + #4 + ACE，几 GB | **A、B 等于修改 D22**；盘空间与 013 训练语料抢 | **B**（顺带把 D22 的估算改为约 20–40 GB） | C |
| 4 | 许可不清的怎么办：DNS 测试集（未单列）、ACE（BY / BY-ND 冲突）、REAL-M（未查到）、DR-VCTK（End-user Licence 未读到，下载页可能要点击同意） | A 只评测、只存本机；DR-VCTK 下载时由人读 End-user Licence 再点同意 · B 发邮件问作者澄清 | B 要人代发邮件，属对外联络 | **A**；只有 #5 选 B 时才需要为 ACE 走 B | A |
| 5 | CC BY 的真实集（NOTSOFAR、DiPCo、RealMAN 噪声）以后要不要进训练 | A 维持 013：真实录音永不进训练 · B 允许 RealMAN 的「噪声」部分进训练噪声库（语音不进） | B 会让真实噪声分布同时出现在训练和回归里 | **A**（至少到第 3 档训练触发前） | A |
| 6 | 公开集里的旁人声音（DNS 众包、CHiME-6 家庭聚会、AliMeeting）切成听测片段，是否照 D19 处理即可 | A 照 D19：只存本机、听测页音频放 `_tmp/`、不入库不外发 · B 只用 AMI / DiPCo 这类许可最宽的做听测 | B 会丢掉最贴近场景的素材 | **A** | A |

## 附录 · 卡片冲突裁定

跨路重复的卡合并进同一文件，`<details>` 里保留另一路的版本。冲突以下表为准：

| 数据集 | 冲突 | 裁定 |
|---|---|---|
| NOTSOFAR-1 | 获取：「注册（需 HF_TOKEN）」vs「HF 未 gated，实际走 Azure Blob」；接近度 2.5 vs 3 | 获取写「注册（HF 账号 token）+ Azure Blob」；接近度取 2.5（无笔记本麦、无大片人声）；时长按会议本身约 31.5 h【推算】，170 h 是按设备累加 |
| AliMeeting | 「Test 近讲不公开」vs「Test 包含 headset」 | 以现行 OpenSLR 为准：Test 包含近讲 |
| AMI | 获取「直接下载」vs「注册」；历史版本写 CC BY-NC-SA | 许可以现行官方 license 页 CC BY 4.0 为准；获取写直接下载（注册说法未复核） |
| CHiME-5 / 6 | 两路两张卡 | 合并；许可 CC BY-SA 4.0（2024-01-01 重发），旧 Sheffield 非商用协议作历史 |
| EasyCom | 「近讲 有」vs「佩戴者本人无近讲」 | 佩戴者和服务员都没有近讲，只有其他参与者有 |
| CHiME-9 ECHI | 接近度 3.5 vs 3；babble 是否真实对话 | 接近度 3；babble 是 14 路音箱回放的朗读 / EARS 语音 + FSD50K 音效 |
| DiPCo | 「每人一支近讲」 | 只确认「single-channel close-talk microphone」，每人一支为论文说法未复核 |
| DNS5 dev 转写 | README 说 speakerphone dev 有真值，论文说「Dev testset was not transcribed」 | 冲突保留；用前先看包里有没有转写文件 |
| DNS5 个性化 dev | 与 DNS4 PDNS dev 分档完全同数 | 疑为同一批【推算】，不当两份独立数据 |
