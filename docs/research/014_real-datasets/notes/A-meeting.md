# A-meeting · 调研与怀疑派记录

## 未找到 / 不符合命门
- 六个会议集里**没有一个用笔记本内置麦录音**，也没有咖啡馆、开放办公、餐厅这种一大片人声的场景。NOTSOFAR 用的是商用会议设备（论文表 2），D42 要在会议类以外另找。
- AISHELL-4 的头戴近讲在论文 §3 写明录了，但 OpenSLR SLR111 发布包没有列出近讲，未查到公开渠道。
- NOTSOFAR 单段的嘴到麦距离、各场次 SNR 数值：论文和 README 都未给（未查到）。
- AMI / ICSI 的具体重叠率、每场人数分布：本轮没有核到官方数字（未查到）。
- ICSI 在 LDC 的价格要登录才看得到，未查到。
- NOTSOFAR 的 HF 数据集是否需要人工审批（gated）：GitHub README 要求 HF_TOKEN，HF 页面没写审批条款，未查到确切说法。

## 预判核验
- NOTSOFAR-1「含单设备录音」：部分成立。有单通道（SC）流，但来自 Logitech MeetUp、Poly Studio、Jabra PanaCast 50、Yealink 等商用会议设备，**不是笔记本**。而且 SC 流已经过设备自带的 AEC、波束、降噪处理，不宜当降噪输入；能用的是 MC 原始 7 通道流里的单个麦。出处：arXiv 2401.08887 §4.1、表 2【论文】
- NOTSOFAR-1「许可允许商业评测」：推翻，实际比这更宽。DATA_LICENSE 是标准 CC BY 4.0，没有 NC/SA 条款，按项目规则可以训练，不只是评测；唯一限制是 Dev-set-2 限挑战赛相关发表。出处：https://raw.githubusercontent.com/microsoft/NOTSOFAR1-Challenge/main/DATA_LICENSE【官方】、GitHub README【官方】
- NOTSOFAR-1「最接近的英文集」：在会议类里成立（真实、重叠、多设备、许可干净），但接近度只有 3/5：没有笔记本麦，也没有一大片人声背景。近讲用 Shure WH20 头戴，只覆盖 train/dev，eval 没有。出处：arXiv 2401.08887 表 2【论文】
- AliMeeting「有头戴近讲、CC BY-SA」：成立。每人戴头戴麦，与远场同步到同一时间轴；许可是 CC BY-SA 4.0，按项目规则只评测。补充：平均重叠率 Train 42.27%，是这批里重叠最多的。出处：https://www.openslr.org/119/【官方】、arXiv 2110.07393 §2【论文】
- AISHELL-4「无近讲、CC BY-SA」：许可成立（CC BY-SA 4.0）。「无近讲」要修正：论文 §3 写明录了每人的头戴近讲，但 OpenSLR 发布包只有阵列数据，公开渠道看来没有近讲【推算】。出处：https://www.openslr.org/111/【官方】、arXiv 2104.03603 §3【论文】
- LibriCSS「回放重录（部分真实）」：成立。LibriSpeech 朗读句经多只扬声器在会议室回放，7 通道圆阵重录，距离 33–409 cm，共 10 h。许可：仓库 MIT，源语音 CC BY 4.0。出处：arXiv 2001.11482 §3.1【论文】、GitHub LICENSE【官方】
- AMI「CC BY 4.0」：成立。爱丁堡官方许可页写的是 Creative Commons Attribution 4.0 International Public License，信号、转写和部分标注都适用。出处：https://groups.inf.ed.ac.uk/ami/corpus/license.shtml【官方】
- ICSI「是否 LDC」：两种都有。LDC 发行 LDC2004S02（语音）和 LDC2004T04（转写），走 LDC User Agreement；爱丁堡 AMI 站点另托管一版，许可写明 CC BY 4.0，可直接下载。出处：https://catalog.ldc.upenn.edu/LDC2004S02【官方 LDC】、https://groups.inf.ed.ac.uk/ami/icsi/license.shtml【官方】

## 怀疑派更正
- NOTSOFAR-1 · 接近度 3 → 2.5：没有笔记本麦，也没有大片人声背景，SC 流已被处理过；按命门打分偏高 · arXiv 2401.08887 表 2
- NOTSOFAR-1 · 「SC 流经设备 AEC、去混响、波束、降噪」→ 论文只说 SC 是「internally processed」，AEC/去混响/波束/降噪是对典型 SC 设备的笼统描述，没逐台确认；MC 流明确是 raw 未处理 · arXiv 2401.08887 §4.1
- NOTSOFAR-1 · Poly Studio、Jabra PanaCast 50 写得像全集都有 → Poly Studio 只在 eval，Jabra 只在部分 eval，Logitech MeetUp 才是全集都有 · arXiv 2401.08887 表 2
- NOTSOFAR-1 · 「Dev-set-2 限挑战赛相关发表」→ 原文是「approved for use exclusively as part of the NOTSOFAR-1 Challenge, but not beyond it」，且已因法律和质量原因下架 · https://github.com/microsoft/NOTSOFAR1-Challenge README
- NOTSOFAR-1 · 补漏：近讲「may still contain a small amount of speech leakage」；转写由专业转写员依据近讲完成，不是单纯「多人校核」 · arXiv 2401.08887
- NOTSOFAR-1 · 「eval_small 80 场」→ 官方页面没找到这个数字，改为未查到；size 里「SC 约 170 h + MC 约 130 h」应为 174.5 / 126 h，且是按设备累加，会议本身约 31.5 h【推算】 · arXiv 2401.08887 表 1、HF 数据卡
- NOTSOFAR-1 · 获取只写了 HF → HF 数据卡还写着 Azure Blob + AzCopy 的旧路径，GitHub README 指向 HF，两处并存 · https://huggingface.co/datasets/microsoft/NOTSOFAR
- AliMeeting · 「Test 的近讲不公开」→ 只是挑战期间不发；现行 OpenSLR SLR119 写明 Test 包（8.90 GB）含 8 通道阵列和 headset · https://www.openslr.org/119/
- AliMeeting · 「13 个会场」没拆分 → Train 12 间、Eval 5 间；Eval 平均重叠率论文表 1 写 34.20%、正文写 34.76%，两处自相矛盾，要注明 · arXiv 2110.07393 §2、表 1
- AliMeeting · 「阿里云 OSS 镜像」→ OSS 是唯一托管点，不是镜像，没有欧洲镜像 · https://www.openslr.org/119/
- AISHELL-4 · 补充：211 场（train 191 / eval 20），train、eval 各 5 间会场；欧洲镜像为 openslr.trmal.net / openslr.elda.org，中国镜像 openslr.magicdatatech.com；近讲录了但没公开，这一判断成立 · arXiv 2104.03603 §3、表 1；https://www.openslr.org/111/
- AMI · 「爱丁堡站每人戴 ME 3-N 头戴 + MKE 2-EW 领夹」→ meetingrooms 页对爱丁堡只写 ME 3-N 头戴；TNO 站明确「no lapel mic」；领夹麦的站点分布未核实；「发布为 16 kHz」未核实 · https://groups.inf.ed.ac.uk/ami/corpus/meetingrooms.shtml
- ICSI · 接近度 2.5 → 2：PDA 麦是 2000 年代初的器件，房间安静，与笔记本 + 大片人声差得远；size 里的「LDC 记 883 h、922 个文件」未核到，删掉；补上每场 3–10 人（平均 6 人）、53 位说话人、每场最多 16 通道 · https://www1.icsi.berkeley.edu/ftp/pub/speech/papers/icassp03-janin.pdf、https://catalog.ldc.upenn.edu/docs/LDC2004T04/overview.txt
- LibriCSS · 「会议室回放」→ GitHub README 写的是「played back from loudspeakers in an office room」；阵列几何和 33–409 cm 本轮没逐字核到论文，标为待核 · https://github.com/chenzhuo1011/libri_css

## 怀疑派：漏掉的
- CHiME-8 MMCSG（Meta Aria 眼镜，佩戴者与对面人真实对话重叠，有转写）——最贴 D47「使用者 + 1–3 m 内对方」的英文真实集；许可要单独查（疑为 Meta 自定义 / NC）。https://www.chimechallenge.org/challenges/chime8/task3/index
- EasyCom（Meta，AR 眼镜 + 头戴近讲，在回放的餐厅嘈杂背景里真实对话）——对 D42 餐厅场景有用，但背景噪声是扬声器回放，只算「部分」；许可 CC BY-NC 4.0，只评测。https://github.com/facebookresearch/EasyComDataset
- CHiME-5 / CHiME-6（家庭晚餐派对，真实厨房/客厅，Kinect 阵列 + 双耳近讲 + 转写）——比会议室嘈杂得多，有餐桌多人重叠；许可要查（CHiME-5 原协议仅研究、禁再分发）。https://www.chimechallenge.org/datasets/chime6
- DiPCo（Amazon 晚餐派对语料，真实录音，有近讲与转写）——同类场景，许可据称 CDLA-Permissive，可能可训练，待核。https://www.amazon.science/publications/dipco-dinner-party-corpus
- MISP 2021/2022（中文家庭客厅看电视时多人聊天，far / middle / near 三种距离 + 视频）——中文的嘈杂家居 + 重叠真实集；许可为签协议，仅研究，待核。https://mispchallenge.github.io/
- Mixer 6（LDC，CHiME-7 DASR 用到；房间内 14 支不同麦，含廉价麦，有近讲）——设备多样，可做设备频响校准；LDC 付费。https://catalog.ldc.upenn.edu/LDC2013S03
- RealMAN（真实录音的中文多通道数据集，含移动声源和真实环境噪声，有近讲直达声参考）——中文、真实声场、带参考，适合 sim-to-real；许可待核。https://github.com/Audio-WestlakeU/RealMAN
