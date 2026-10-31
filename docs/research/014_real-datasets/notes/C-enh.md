# C-enh · 调研与怀疑派记录

## 未找到 / 不符合命门
- Cadenza（CAD1/CAD2/CLIP）：数据是音乐/歌词可懂度，不是嘈杂环境语音录音，不符合本路需求，未出卡（zenodo.org/records/14333454；CLIP2 数据为 CPC-4 许可：zenodo.org/records/20413050）
- Clarity CEC1/CEC2：场景为模拟合成（CEC3 Task 3 也只是真实环境背景 + 模拟目标语音，Zenodo 17063983，CC BY 4.0），不符合命门 1，未出卡
- DNS 各届盲测集转写：ICASSP 2022 论文 §3.3 说盲测已转写，但仓库 v4/v5 README 均标 'Blind testset: <TBD>'，公开转写文件未查到
- DNS 各届测试集逐条设备/场景/距离元数据：论文只给总体比例（如 DNS4 盲测 638/859 为手机），逐条标签未查到；'笔记本内置麦 + 嘈杂人声'无法按标签筛出，需人工听选
- ICASSP 2021 pDNS（Track 2）盲测 500 条是否公开下载：V2_V3 合集脚本只列 Track 1 子目录，pDNS 盲测下载位置未查到
- DNS5（ICASSP 2023）盲测条数与设备清单：论文写 '[list of devices]' 占位，未给具体数字
- AEC Challenge：近端单讲在测试集中的条数、真实录音数据许可、朗读文本转写文件，均未查到
- URGENT 2024/2025/2026 真实录音的具体来源语料与各自许可：论文和官网均未点名（只说 unseen / public sources）
- REAL-M 数据许可：论文、SpeechBrain 模型卡均未写；压缩包内是否有 LICENSE 未核（按规定不下载本体）
- Clarity CEC3 Task 2 数据许可：官网与公告均未写
- 流程备注：核查 REAL-M 下载链接时，一次 WebFetch 误指向数据包 URL，因超过 10 MB 上限被工具中止，未落盘、未解压

## 预判核验
- 预判『DNS 真实盲测只有无参考 MOS、无转写，只适合人耳听测素材』→ 部分推翻：ICASSP 2022 盲测 859 条已由第三方转写并经专家校对（arXiv:2202.13288 §3.3），评分加了 WAcc，但公开仓库标 TBD、转写未放出；dev 集有公开转写（DNSChallenge4_devtest.tsv；DNS5 speakerphone dev 有真值转写，见 github.com/microsoft/DNS-Challenge v4 分支与 master README）。个性化赛道附干净 enrollment（ICASSP 2021 为 2 min；2022 为 2.5 min；2023 为 10–30 s）。因此 dev 集可做带 WAcc 的真实回归集，不止能当听测素材
- 预判『DNS 数据许可 CC BY 4.0 但各源混合』→ 对训练数据成立：README 写明 datasets provided under the original terms，逐源列出 LibriVox PD、VCTK ODC-By、DEMAND CC BY-SA 3.0、Freesound CC0、AudioSet CC BY 4.0 等。对真实测试集需修正：没有单列许可，只有仓库 LICENSE（Creative Commons Attribution 4.0 International，覆盖 documentation and content）；测试集是否在其覆盖范围内属推断，建议按只评测处理（github.com/microsoft/DNS-Challenge/blob/master/LICENSE）
- 预判『REAL-M 很小、无参考』→ 成立，补一点：1436 条、约 3 h、50 人，论文明说没有 ground truth；但每条有文本转写，并且有笔记本、手机、视频会议软件录制的真实双人重叠（arXiv:2110.10812 §2）。许可未写
- 预判『VOiCES 是回放重录、CC BY 4.0』→ 成立（registry.opendata.aws/lab41-sri-voices 写 Creative Commons BY 4.0）。补充：音箱回放 LibriSpeech 后由 12 支麦重录，距离 0.7–5.8 m，babble 为 9 路 MUSAN 会议人声回放；保留样本同步的源音频，可做有参考指标（arXiv:1804.05053 §2、表 1）
- 预判『DNS 盲测用多种设备录制』→ 成立：DNS1 盲测为 MTurk 耳机/免提（arXiv:2005.13981 §2.4.2）；DNS4 盲测 638/859 为手机、其余台式机/笔记本，dev 全部笔记本/台式机，近讲 + 1/2/3 m（arXiv:2202.13288 §3.2–3.3；v4 README）；DNS5 盲测为台式机/笔记本/手机，噪声明列 Café、Background babble、In-car neighboring talkers（arXiv:2303.11510 §3.3）。但没有逐条设备与场景标签，『笔记本 + 嘈杂人声』只能人工听选
- 附带发现：DNS 个性化赛道（ICASSP 2021 pDNS 盲测 500 条，噪声源多为第二说话人；ICASSP 2022 PDNS dev 有 166 条邻座 + 噪声、347 条只有邻座）是本路中与 D47『邻座重叠』最对口的真实数据

## 怀疑派更正
- DNS1 · 接近度 2.5 → 2；补充：AudioSet 300 条真实片段来自 YouTube，第三方版权，不能当 MS 自有数据 · arXiv:2005.13981 §2.4.1；DNS README「datasets provided under the original terms」
- DNS 全系 · 许可写成「仓库 LICENSE CC BY 4.0」→ 更正为 README 原话：code MIT、documentation and other content CC BY 4.0，datasets「provided under the original terms that Microsoft received such datasets」、AS IS；测试集许可仍未查到 · github.com/microsoft/DNS-Challenge/blob/v4dnschallenge_ICASSP2022/README.md、master/README.md
- DNS2 · Track1 dev 400 条真实未说明来源 → 其中 300 英语条复用自 DNS1 blind 真实段，另 100 条非英语；dev 还有 200 条合成；blind 650 真实另 50 条是合成歌唱；pDNS dev 另有 500 条合成 · arXiv:2009.06122 §4.1.1–4.2.2
- DNS2 · 接近度 3.5 → 3（pDNS dev 只有 100 条真实，blind 是否公开未查到）· arXiv:2009.06122 §4.2
- DNS3 · 接近度 3 → 2.5；600 条这个数字论文没给，只出自下载脚本 · arXiv:2101.01902 §4
- DNS4 · 「dev 噪声列表里没有咖啡馆 babble」论文没这样写（只列风扇、打字、狗叫等，blind 噪声见图 1）→ 改为「未查到」；接近度 4 → 3.5 · arXiv:2202.13288 §3.2–3.3
- DNS4 · 补充个性化 dev 的完整分档：737 主说话人+噪声 / 166 主说话人+邻座+噪声 / 347 主说话人+邻座 / 193 情感；blind 约 70% 是手机 · arXiv:2202.13288 §3.2–3.3
- DNS5 · 「众包，台式机、笔记本、手机」→ dev 全部是笔记本/台式机，blind 设备清单在论文里是占位符 · arXiv:2303.11510 §3
- DNS5 · 「dev speakerphone 有真值转写」只是 README 的说法，论文写「Dev testset was not transcribed」，两者冲突，标为冲突；blind 已全部转写但没公开 · arXiv:2303.11510 §3.3 vs master/README.md
- DNS5 · 个性化 dev 1443 条的分档与 DNS4 PDNS dev 完全相同，疑为复用同一批，不能当两份独立数据【推算】；接近度 4 → 3.5 · arXiv:2303.11510 §3.2 与 arXiv:2202.13288 §3.2
- AEC · 「50,000+ 条、每年 test/blind 800 条」在 README 里没有核到，README 只说 10,000+ devices and speakers → 改为未核/论文二手 · github.com/microsoft/AEC-Challenge
- URGENT 2024 · reference「部分」→「没有」（HF 数据卡说 blind「no labels or meta information」）· huggingface.co/datasets/urgent-challenge/urgent2024_official
- URGENT 2025 · license_class「只评测」是从衍生的 sqa 集套来的 → 改为「未查到」（测试集本体官网没写许可）；接近度 2.5 → 2 · urgent-challenge.github.io/urgent2025/data/
- URGENT 2026 · HF urgent2026_leaderboard 没有数据卡（「No dataset card yet」），blind_noisy.zip 的路径在页面上没核到；接近度 1.5 → 1 · huggingface.co/datasets/urgent-challenge/urgent2026_leaderboard
- REAL-M · 接近度 3 → 2.5（没有背景人群、没有参考、许可不明）；「美英法意波斯印度非洲口音」论文没列出，改为「native/non-native」 · arXiv:2110.10812 §1–2
- CHiME-7 UDASE · 许可已回 LICENSE.txt 原文核实：CC BY-SA 4.0，没有 NC 或仅研究条款，但 SA 按项目规定只评测；UDASE 页面本身没写许可；接近度 3.5 → 3（家庭而非咖啡馆，非笔记本麦，没有干净参考）· openslr.org/resources/150/LICENSE.txt
- Clarity CEC3 T2 · 补距离：前方 9 个音箱距 2–4 m；麦间距约 1 cm；说话人标签「有」→ 未查到；接近度 2 → 1.5 · claritychallenge.org/docs/cec3/task_2/cec3_task2_data
- 通用 · 论文 arXiv:2506.01611、2505.23212、2601.13531、2307.03533、1804.05053、2309.12553 本轮没有回原文核，卡内统一标「论文，未独立复核」

## 怀疑派：漏掉的
- CHiME-4 real（BUS/CAF/PED/STR）· 平板 6 麦 + 同步近讲麦真实录音，含咖啡馆，最贴 D42，但文本基于 WSJ，需 LDC 许可 · https://www.chimechallenge.org/challenges/chime4/
- AliMeeting / M2MeT · 普通话真实会议，8 通道远场阵列 + 头戴近讲同步，有时间戳转写，重叠率高，对口 D47 中文 · https://www.openslr.org/119/
- AISHELL-4 · 普通话真实会议室 8 通道阵列，有说话人与时间戳转写 · https://www.openslr.org/111/
- RealMAN · 普通话真实录音，真实场景噪声（含餐厅、食堂等），有近讲参考 · arXiv:2406.19959
- AMI Meeting Corpus · 头戴/领夹 + 阵列同步，CC BY 4.0，可训练 · https://groups.inf.ed.ac.uk/ami/corpus/
- CHiME-8 NOTSOFAR-1 · Microsoft 真实办公会议，多种商用设备 + 近讲，带转写 · https://www.chimechallenge.org/challenges/chime8/task2/
- DiPCo（Amazon Dinner Party Corpus）· 真实晚餐多人对话，近讲 + 远场阵列，CDLA-Permissive · https://zenodo.org/records/8122551
- EasyCom（Meta）· 眼镜阵列在播放餐厅噪声的房间里录，有近讲麦与转写，许可为 CC BY-NC · https://github.com/facebookresearch/EasyComDataset
- MISP 2021/2022 · 普通话家庭真实对话，远/中/近场同步 · https://mispchallenge.github.io/
- DAPS · 同一段语音在多种真实设备和房间里重录，有干净参考 · https://zenodo.org/records/4660670
- ConferencingSpeech 2021 · 真实录音测试集，含普通话 · https://github.com/ConferencingSpeech/ConferencingSpeech2021
