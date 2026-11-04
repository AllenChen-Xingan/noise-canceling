# r2 按地点找真实录音

> 2026-11-03 · 手下 r2-place。管的实体：真实数据集 · 真实锚。新卡在 `../cards/`，014 已有集的补充字段在 `r2-place-addendum.md`。
> 下表用途：每个地点有哪些公开真实录音、能拿来评测什么、还缺什么。「人声」指录音里有真人说话；「目标语音」指有一个被评测的主说话人。
> 许可结论口径：CC BY / CC0 → 训练可；SA / NC / 自定义 → 只评测；LDC 付费 → 不可；读不到 → 未查到。

## 总结（先看这里）
- 没有一个公开集是「笔记本内置麦 + 真实公共地点 + 主说话人 + 干净参考」。最接近的是 DNS5 dev（笔记本 / 台式机真录，但无设备标签、无参考）【沿用 014】。
- 每个地点都能找到**真实背景噪声**；带**真人目标语音**的只有会议室、家里、车内、咖啡馆 / 食堂四处。机场 / 车站、开放办公、街边只有背景，没有目标说话人的公开真实集（CHiME-3 例外，但 LDC 付费）。
- 语体缺口：笑声有 CHiME-5/6 `[laughs]`、VocalSound；附和有 AMI 对话行为层、CANDOR backbiter；情绪有 DNS5 193 条、MSP-Podcast、IEMOCAP。

## meeting-room 会议室
- **候选**：NOTSOFAR-1 · AMI · ICSI · AliMeeting · AISHELL-4 · MISP-Meeting（均 014）；DEMAND OMEETING（新卡 `demand.md`，只有背景）
- **小时**：NOTSOFAR 会议时长约 31.5 h；AMI 约 100 h；AliMeeting 118.75 h；AISHELL-4 120 h【沿用 014】
- **参考 / 转写**：NOTSOFAR、AMI、AliMeeting 有近讲 + 带时间戳转写；AISHELL-4 只有转写
- **许可**：AMI、ICSI、NOTSOFAR 训练可（CC BY 4.0）；AliMeeting、AISHELL-4 只评测（CC BY-SA 4.0）
- **缺口**：全部是会议终端或研究麦阵，没有笔记本麦；会议室安静，无外部 babble

## home 家里（家人说话、电视）
- **候选**：CHiME-5/6 · DiPCo（014，晚餐聚会真人对话）；QUT-NOISE HOME-LIVINGB（新卡，孩子唱歌说话 + 电视）；CHiME-Home（新卡，标了儿童语音、电视段）；DEMAND DLIVING / DKITCHEN；VoiceHome-2（014，法语）
- **小时**：CHiME-5/6 约 50 h；DiPCo 5.3 h；QUT HOME 2 段 × ≥30 min；CHiME-Home 约 6.8 h【官方文档】【论文】
- **参考 / 转写**：CHiME-5/6、DiPCo 有近讲参考 + 转写；QUT、CHiME-Home、DEMAND 无
- **许可**：DiPCo 训练可（CDLA-Permissive-1.0）；CHiME-5/6 只评测（SA）；QUT-NOISE 只评测（CC BY-SA）；CHiME-Home 只评测（CC BY-NC-SA 3.0）；DEMAND 只评测（PDF 写 CC BY-SA 3.0，与 Zenodo 字段冲突）
- **缺口**：CHiME-5/6 明确禁止背景电视【沿用 014】；「主说话人开会 + 家人在旁说话 / 电视」的真实录音没有，只能把 QUT / CHiME-Home 的电视段混进去（合成）或自录

## cafe 咖啡馆 / 餐厅 / 食堂
- **候选**：ASPIRE（真人在食堂朗读）· EasyCom（真人对话 + 回放餐厅噪声）· CHiME-9 ECHI（真人对话 + 回放干扰）· CHiME-3 CAF（真人朗读）· ARTE · WHAM! · DNS5 blind café（以上 014）；QUT-NOISE CAFE、DEMAND PCAFETER / PRESTO / SCAFE（新卡，只背景）
- **小时**：ASPIRE 真实嘈杂 5,000 句；EasyCom 5 h 18 min；ECHI 29.4 h；CHiME-3 CAF train 0.76 h + dev 0.69 h；QUT CAFE 2 段 × ≥30 min；DEMAND 3 × 5 min
- **参考 / 转写**：ASPIRE 领夹（在食堂里，不干净）+ 脚本转写；EasyCom、ECHI 有近讲；CHiME-3 有 ch0 近讲
- **许可**：全部只评测或不可（ASPIRE 非商用、EasyCom NC、ECHI 自定义、CHiME-3 LDC、QUT / DEMAND SA）；WHAM! 未查到
- **缺口**：没有「真人坐在真实咖啡馆、对笔记本开会」的公开录音；最近的是 ASPIRE（iPad + 朗读）与 CHiME-3（平板 + 朗读）

## open-office 开放办公
- **候选**：ARTE 办公场景（014，「open-plan office, people typing, chatting and talking on the phone」）；DEMAND OOFFICE（新卡，小办公室三人用电脑）/ OHALLWAY；DNS5 dev 办公段（014，无逐条场景标签）
- **小时**：ARTE 单场景时长未查到；DEMAND 每段 5 min
- **参考 / 转写**：均无
- **许可**：ARTE 只评测（说明写仅非商业）；DEMAND 只评测；DNS5 未查到
- **缺口**：**公开集没有开放办公里带主说话人的真实录音**，只能自录，或把 ARTE / DEMAND 背景混进干净语音（合成）

## car 车内
- **候选**：AISHELL-5（014，中文 2–4 人车内自然对话，即 ICMC-ASR 修正版）；AVICAR（新卡，英语朗读，行驶车内 8 麦线阵）；QUT-NOISE CAR（新卡，开窗 / 关窗，无人声）；DEMAND TCAR；DNS5 dev/blind 车内段（014）
- **小时**：AISHELL-5 100+ h 语音 + 约 40 h 车内噪声；AVICAR 约 59,000 条（小时未查到）；QUT CAR 2 段 × ≥30 min
- **参考 / 转写**：AISHELL-5 有转写，近讲只 train 有；AVICAR 有脚本转写；QUT 无
- **许可**：AISHELL-5 只评测（CC BY-SA 4.0）；AVICAR 未查到（需申请）；QUT 只评测
- **缺口**：没有「车内用笔记本开会」的录音；AISHELL-5 是门把麦，不是笔记本位置

## transit 车站 / 机场 / 地铁
- **候选**：TAU Urban Acoustic Scenes 2020 Mobile（新卡，机场、地铁站、公交、电车、地铁车厢）；DEMAND PSTATION / TMETRO / TBUS；ARTE 火车站；CHiME-3 BUS（真人在公交上朗读）；DNS5 blind 机场广播段
- **小时**：TAU 64 h（10 个场景约各 6.4 h【推断：按均衡划分】）；DEMAND 每段 5 min；CHiME-3 BUS 小时见 014 卡
- **参考 / 转写**：TAU、DEMAND、ARTE 无；CHiME-3 有近讲 + 转写
- **许可**：TAU 只评测（Other Non-Commercial）；DEMAND 只评测；CHiME-3 不可（LDC）
- **缺口**：**机场 / 车站里有主说话人的公开真实录音没有找到**（CHiME-3 BUS 是车上，非车站）；只能自录或回放重录

## street 街边
- **候选**：TAU street_pedestrian / street_traffic / public_square；QUT-NOISE STREET；DEMAND STRAFFIC / SPSQUARE；CHiME-3 PED / STR；VoxCeleb 户外采访（新卡，无地点标签）；Ego4D 户外对话（新卡，需签协议）
- **小时**：TAU 每场景约 6.4 h【推断】；QUT STREET 2 段 × ≥30 min
- **参考 / 转写**：CHiME-3 有；其余无（Ego4D 部分有转写，未核实）
- **许可**：TAU、QUT、DEMAND 只评测；CHiME-3 不可；VoxCeleb、Ego4D 未查到
- **缺口**：带主说话人、按地点标注、许可清楚的街边真实录音没有；VoxCeleb 需人工挑段

## 顺带：语体（笑声、附和、情绪）
- **笑声**：CHiME-5/6 转写 `[laughs]`【官方文档】；VocalSound 6 类之一（新卡，CC BY-SA → 只评测）；Buckeye `<LAUGH-…>`（新卡，非商用，需注册）；Switchboard NXT 音素词层含 laughter（新卡，音频 LDC 付费）；NOTSOFAR 元数据声学事件含笑声
- **附和**：AMI 对话行为层 backchannel 类；CANDOR backbiter 口径带附和时间戳（新卡，需申请，许可未查到）；Switchboard SWDA
- **情绪**：DNS5 / DNS4 193 条情绪语音（014）；MSP-Podcast 409 h（新卡，学术许可）；IEMOCAP 约 12 h（新卡，表演情绪，需签表）
- **缺口**：中文笑声 / 附和 / 情绪的真实会议标注本路没找到（AliMeeting、AISHELL-4 是否标笑声未查到）

## 需要注册 / 签协议（本轮均未注册）
- CHiME-9 ECHI：HF 门控 + 签 DUA（014）
- Ego4D：签 Ego4D 协议，约 48 h 发 AWS 凭据
- CANDOR：向 BetterUp 申请
- MSP-Podcast：机构签学术许可
- IEMOCAP：在线 release form，学术邮箱
- Buckeye：站点注册
- VoxCeleb：申请表拿密码
- AVICAR：向伊利诺伊大学申请
- Freesound：单条下载要登录；批量走 Data Packs 申请
- Switchboard：LDC 付费

## 本轮未查（留给下一轮）
- DCASE TUT 2016–2019 场景集（含 cafe/restaurant、office、home、car、train）的许可
- CAVEMOVE（2025，车内 RIR + 噪声，Zenodo 10.5281/zenodo.13594242）：数据许可未读
- MSP-Conversation、TalkBank CANDOR 分段版的获取条件
