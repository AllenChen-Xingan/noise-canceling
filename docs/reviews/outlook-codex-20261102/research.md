**5．总评与逐条核实**

**桌面文件判断可靠度：3／5。** 它对工程难度、评测必要性和相关职业方向的判断大体合理，但把“Mac 开源产品空白”和“评测方法空白”说得过满。已有近期发布的开源 Mac 降噪应用，其中 NoNoise-Mac 自带虚拟麦克风；无参考听感预测、人工评分校准和降噪后识别准确率也已有大量研究。公开证据不足以支持“个人项目能明显超过 RNNoise”，也不足以证明“在 babble 下必然追不上 Krisp”。更可信的项目定位是：**把可安装、可靠运行、少吃字和可复现评测做好，并通过实测证明具体优势。**

已按要求读完意向书、状态板前 60 行和桌面文件；全程只读，未修改文件。**以下所有外部链接的访问日期均为 2026-11-02**；发布时间另列。仓库星数来自本次 GitHub 公共 API 查询；网页和搜索索引可能滞后，未实装的软件不视为通过兼容性验证。

| 桌面文件的判断 | 核实结果 | 事实与推断的边界 |
|---|---|---|
| Mac 虚拟麦克风、音频管线有先例 | **证实** | [BlackHole](https://github.com/ExistentialAudio/BlackHole)、[NoNoise-Mac](https://github.com/ivalsaraj/NoNoise-Mac)提供实际代码；“中高难度”“慢一点能做完”仍是工程判断。 |
| 实时推理比训练模型更稀缺、更难 | **部分支持** | [Apple 岗位](https://jobs.apple.com/hu-hu/details/200680823-0836/)确实要求低延迟语音增强落地；但没有人才供给统计支持“会训练的人多、会实时部署的人少”。 |
| 嘈杂人声下兼顾压噪与保真很难 | **支持** | [URGENT 2024 的后续分析](https://arxiv.org/abs/2506.01611)仍把重叠语音、强噪声和混响列为难点；不能由此推导个人项目的成功概率。 |
| “在 Mac 上目前没有人做好”／“Mac 是空白” | **绝对表述被推翻** | 已有开源系统级路线及近期版本；但是否达到成熟商业产品的稳定性，仍需实测。[NoNoise-Mac](https://github.com/ivalsaraj/NoNoise-Mac)、[MetalVoice](https://github.com/Ghostkwebb/MetalVoice)。 |
| DNS Challenge 一直卡在“指标和人耳对不上” | **需要重大修正** | 指标存在泛化和局部失效，但 DNSMOS 已报告很高的系统级相关性，不能概括为普遍对不上。[DNSMOS](https://arxiv.org/pdf/2010.15258)、[DNSMOS P.835](https://arxiv.org/pdf/2110.01763)。 |
| 吃字率、人耳定标 verifier 正好填补评测空白 | **“空白”被推翻** | DNS 2022 已纳入识别准确率；SCOREQ、URGENT、UrgentMOS 已研究听感对齐。工程实现仍可有价值。[DNS 2022](https://arxiv.org/abs/2202.13288)、[UrgentMOS](https://arxiv.org/html/2601.18438v1)。 |
| 可以明显超过 RNNoise、接近 DeepFilterNet | **无法核实** | 属于尚未验证的项目目标；必须指定模型版本、场景、延迟和评价方法。RNNoise 也不是只有最初的旧权重。[RNNoise](https://github.com/xiph/rnnoise)。 |
| babble 下全面追平 Krisp 很难 | **合理推断，非已证实结论** | 有小规模公开比较，没有找到足够完整的同条件人工基准支持这条性能排序。[HushMic 对比](https://github.com/Fovty/HushMic/blob/main/docs/comparison.md)。 |
| NoiseTorch 曾有几千星，说明有人关注需求 | **数量方向证实** | 当前为 **10,354 星**；星数代表关注度，不能直接换算用户数或付费需求。[NoiseTorch](https://github.com/noisetorch/NoiseTorch)。 |
| 相关公司会关注这类能力 | **岗位方向证实，具体关注无法保证** | 本次找到 Apple、Google、Dolby、Cerence、Amazon 等相关岗位，见第 4 部分；不能据此保证他们会关注本项目。 |
| 可用于面试、技术演讲、小论文 | **合理机会判断** | 技能与岗位要求相符；论文是否成立取决于新增方法或证据，不能仅凭“ASR＋盲听”视为创新。[Apple](https://jobs.apple.com/hu-hu/details/200680823-0836/)、[SCOREQ](https://arxiv.org/abs/2410.06675)。 |
| GitHub 项目会带来主动联系；多 AI 协作故事有吸引力 | **仍无法核实** | 未找到能量化本项目获联系概率的证据；这是传播假设。作者履历不能证明招聘因果，见第 4 部分。 |
| 安装包、公开比较、演示、可复现评测是机会出现的必要条件 | **建议合理，“必要条件”过强** | 这些能降低评估成本，但没有证据表明缺一项就难获机会；研究库也可能凭论文、算法或贡献获得认可。[DeepFilterNet 论文及开源发布](https://arxiv.org/abs/2305.08227)。 |

**1．竞品现状**

这里将“系统级虚拟麦克风”严格定义为：**向 macOS 注册一个输入设备，其他允许选择麦克风的应用可以使用它。** 应用内部降噪、音频插件和模型库分别列出，不混算成完整产品。

**商业与系统内置方案——查到的事实**

| 方案 | 开源？ | Mac？ | 系统级虚拟麦克风？ | 维护／发布证据 | GitHub 星数 |
|---|---|---|---|---|---|
| **Krisp** | 否 | 是 | **是** | 官方更新页列出 **2026-10-19，3.18.3**；持续维护 | 不适用 |
| **Apple Voice Isolation** | 该功能未开源 | 是 | **否**，是支持 Mic Modes 的应用所用的系统处理模式 | 当前 Mac 用户指南仍支持；未找到独立算法版本或发布日期 | 不适用 |
| **Zoom 内置降噪／个性化隔离** | 否 | 是 | 否，仅 Zoom 内部 | 当前官方文档仍支持；未核实降噪模块最近一次独立更新日期 | 不适用 |
| **Microsoft Teams** | 否 | 是 | 否，仅 Teams 内部 | Mac 客户端记录至少更新至 **2026-10-22**；降噪模型不单独公开版本 | 不适用 |
| **Google Meet** | 否 | 是，浏览器使用 | 否，仅 Meet 内部 | 产品在 **2026-10** 持续更新；降噪模型不单独公开版本 | 不适用 |
| **NVIDIA Broadcast** | 否 | **不支持 Mac** | Windows 上有虚拟音频设备；Mac 无 | 官方当前下载版本 **2.3.0**；页面未明确展示发布日期 | 不适用 |

对应来源，均访问于 2026-11-02：

- **Krisp：**官方明确其虚拟麦克风／扬声器可接入其他应用。[产品架构说明](https://krisp.ai/security-for-call-centers-enterprises/)、[发布记录](https://whatsnew.krisp.ai/?categories=cat_fW6tCYWtLyE26%2Ccat_qgGhvf6uPKWLs%2Ccat_NXgxKYuCvTpF4)。
- **Apple：**官方说的是进入“支持 Mic Modes 的应用”，选项因应用而异，不能写成“任意软件必定可用”。[Mac 用户指南](https://support.apple.com/en-gb/guide/mac-help/mchle82b42f0/27/mac/27)。
- **Zoom：**普通降噪支持 macOS；个性化隔离可利用声纹区分目标说话人与背景。[音频设置](https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0059985)、[个性化隔离](https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0074698)。
- **Teams：**Mac 有降噪设置；另有使用语音档案的 Voice Isolation，不能把普通降噪和目标说话人隔离视为同一种功能。[Mac 设置说明](https://support.microsoft.com/en-us/teams/free/settings/manage-your-device-settings-in-microsoft-teams-free)、[语音隔离](https://support.microsoft.com/en-us/teams/meetings/voice-isolation-in-microsoft-teams-calls-and-meetings)、[版本记录](https://learn.microsoft.com/en-us/officeupdates/teams-app-versioning)。
- **Meet：**官方明确普通降噪针对非语音噪声，**电视声音、其他人讲话不会被消除**；使用范围受套餐与实现方式影响，可能涉及云端处理。[官方说明](https://support.google.com/meet/answer/9919960?hl=en-CM)、[2026-10 产品更新](https://workspaceupdates.googleblog.com/2026/10/)。
- **NVIDIA：**官方平台列表为 Windows，且要求支持的 NVIDIA RTX 硬件，没有 macOS 版本。[系统要求](https://www.nvidia.com/en-us/geforce/broadcasting/broadcast-app/)。

**开源方案——查到的事实**

维护判断根据本次查询到的提交和发布记录；“最近发布”不等于已经证明稳定可靠。

| 项目 | 开源／Mac 支持 | 系统级虚拟麦克风 | 最新维护证据 | 星数 |
|---|---|---|---|---:|
| [RNNoise](https://github.com/xiph/rnnoise) | BSD-3-Clause；可在 Mac 编译使用的 C 库 | **不自带** | 提交 **2025-02-22**；v0.2 发布 **2024-04-15**。近一年未见默认分支更新，不能称高频活跃 | **5,932** |
| [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) | 开源代码与权重；支持 macOS，包括 aarch64 分发 | **不自带 Mac 完整虚拟麦克风产品** | 提交 **2024-10-17**；v0.5.6 发布 **2023-08-31**。上游维护明显放缓 | **4,942** |
| [NoiseTorch](https://github.com/noisetorch/NoiseTorch) | 开源；面向 Linux，**不支持 Mac** | Linux 上有，Mac 无 | 提交 **2025-01-13**；v0.12.2 发布 **2022-06-17**。发布长期停滞 | **10,354** |
| [werman/noise-suppression-for-voice](https://github.com/werman/noise-suppression-for-voice) | GPL-3.0；有 macOS 音频插件 | **不自带 Mac 虚拟麦克风**，需宿主和路由 | 稳定版 v1.10 为 **2024-05-18**；另有 **2026-10-25 的 v1.22 预发布**，不能只看旧默认分支便认定停更 | **6,963** |
| [NoNoise-Mac](https://github.com/ivalsaraj/NoNoise-Mac) | MIT；Apple Silicon；DeepFilterNet3／CoreML | **有，自带 NoNoise Mic 驱动** | 提交与 v1.5.1 发布均为 **2026-09-11**。近期有维护 | **29** |
| [MetalVoice](https://github.com/Ghostkwebb/MetalVoice) | MIT；Apple Silicon；DeepFilterNet3／CoreML | **组合实现**：输出到 BlackHole，再供会议软件选择 | 提交与 v1.2.5 发布均为 **2026-08-31**。近期有维护 | **75** |
| [BlackHole](https://github.com/ExistentialAudio/BlackHole) | 开源 macOS 音频路由驱动 | **有虚拟设备，但本身不降噪** | 提交 **2026-10-28**；v0.7.1 发布 **2026-07-03**。活跃 | **20,001** |

发布记录：[DeepFilterNet](https://github.com/Rikorose/DeepFilterNet/releases)、[RNNoise](https://github.com/xiph/rnnoise/releases)、[NoiseTorch](https://github.com/noisetorch/NoiseTorch/releases)、[werman v1.22](https://github.com/werman/noise-suppression-for-voice/releases/tag/v1.22)、[NoNoise-Mac](https://github.com/ivalsaraj/NoNoise-Mac/releases)、[MetalVoice](https://github.com/Ghostkwebb/MetalVoice/releases)、[BlackHole](https://github.com/ExistentialAudio/BlackHole/releases)。访问：2026-11-02。

两个关键限制：

- **NoNoise-Mac 已覆盖核心产品形态，但成熟度未证实。** README 提供应用、CLI、自有虚拟麦克风和安装包，同时明确安装包未经常规签名认可，需在安全设置中允许；本次没有实装、测试睡眠恢复、设备切换或长时间运行。它还注明源自 MetalVoice，因此两者不是完全独立的技术路线。[NoNoise-Mac README](https://github.com/ivalsaraj/NoNoise-Mac)。
- **“Apple Silicon 支持”不等于支持所有 Mac。** MetalVoice 明确不支持 Intel Mac，并需要配置虚拟音频线。[MetalVoice README](https://github.com/Ghostkwebb/MetalVoice)。

**我的判断：**“Mac 上没有维护良好的开源系统级方案”作为绝对结论**不成立或至少证据不足**。“没有开源系统级方案”已被反例推翻；“维护良好”若指近期更新，也有反例；若指长期稳定、签名公证、广泛兼容和及时响应问题，则本次尚不足以确认这些小项目已做到，也不足以证明全部没做到。更准确的描述是：**已有近期维护的开源方案，但成熟度与真实会议效果仍缺少充分公开验证。**

**2．效果天花板：babble 下到底有没有公开比较？**

**有小规模比较；没有找到足以判定整体胜负的严格公开基准。**

最直接的证据来自 [HushMic 的对比文档](https://github.com/Fovty/HushMic/blob/main/docs/comparison.md)，访问：2026-11-02。

作者把**同一段人声**分别混入键盘、风扇和咖啡馆噪声，约 **+3 dB SNR**，用 DNSMOS P.835 评分。报告的**三段平均值**如下，注意这不是咖啡馆单独成绩：

| 模型 | DNSMOS OVRL | BAK | SIG |
|---|---:|---:|---:|
| DeepFilterNet 3 | 2.97 | 3.98 | 3.26 |
| Krisp，文档标为 v9.9.3 | 2.57 | 3.96 | 2.81 |
| RNNoise，EasyEffects 默认模型 | 2.01 | 3.93 | 2.60 |

**事实限制：**作者明确说，这不是人工听测；只有一位说话人、三段合成音频，完整评分运行的模型配置及逐段输出尚未在仓库归档。文档中的 Krisp 版本不能直接等同于当前 Mac 桌面版。因此它只能证明“存在包含咖啡馆人声的公开比较”，不能证明 DeepFilterNet 普遍胜过 Krisp，更不能代表 Mac 端到端使用效果。[方法与限制](https://github.com/Fovty/HushMic/blob/main/docs/comparison.md)。

另有学术基准比较 RNNoise、DeepFilterNet 等开源模型。例如 [GTCRN 作者的公开结果](https://sigport.org/sites/default/files/docs/GTCRN_poster.pdf)包含两者，但没有 Krisp，也不是专门的 Mac babble 人工对比，不能补上这个证据缺口。访问：2026-11-02。

**我的推断：**

- 桌面文件给出的“RNNoise → DeepFilterNet → Krisp”性能阶梯，**不应写成已经核实的普遍排序**。
- babble、一个清晰可辨的干扰说话人、电视对白，涉及的难度和目标可能不同；有声纹登记的目标说话人提取，应与不登记声纹的通用降噪分别比较。[DNS 2022 个性化赛道](https://arxiv.org/abs/2202.13288)。
- 本项目可以争取某些条件下的优势，但“从不吃字”属于必须由测试限定范围的承诺。一个直接的反例是 werman 在 **2026-10 的预发布说明**中，因新 RNNoise 权重会压掉部分词甚至整词而回退权重。[v1.22 发布说明](https://github.com/werman/noise-suppression-for-voice/releases/tag/v1.22)。

**3．评测难题与新颖性**

**DNS Challenge 的公开相关性结果**

P.808 主要用于整体听感众测；P.835 分开评价语音质量 **SIG**、背景质量 **BAK** 和整体质量 **OVRL**。DNSMOS 是预测这些人工评分的模型，不是人工评分本身。[DNSMOS P.835 论文](https://arxiv.org/pdf/2110.01763)，访问：2026-11-02。

| 赛季／研究 | 公开结果 | 能说明什么 |
|---|---|---|
| **第 1 届，INTERSPEECH 2020**；DNSMOS 论文复分析盲测结果 | 系统级 PCC／SRCC：PESQ **0.78／0.82**；SDR **0.23／0.25**；POLQA **0.79／0.84**；初始 DNSMOS **0.93／0.94** | SDR 很弱；PESQ 有一定相关性；DNSMOS 对系统排名明显更好 |
| **第 2 届，ICASSP 2021**；18 个系统 | 初始 DNSMOS 在新分布上的总体 SRCC 降至 **0.65**；自训练版本提高至 **0.90**。初始模型在声调语言和歌唱类别分别为 **0.51、0.36** | 跨语言、情绪和歌唱等分布变化会使相关性下降 |
| **第 3 届相关数据训练的 DNSMOS P.835**；另用 17 个微软内部系统、850 段未见音频验证 | 系统级 PCC：SIG **0.94**、BAK **0.98**、OVRL **0.98**；片段级 PCC：**0.71、0.83、0.82** | 系统平均成绩与单段判断必须分开；这些数值不是第 3 届所有参赛系统的直接相关性 |
| **第 4 届，ICASSP 2022** | 采用 P.835 人工评价，加入 **WAcc** 和 DNSMOS P.835 | 听感与识别准确率并行评价，早已是正式研究实践 |

前两行来源：[DNSMOS 论文，表 2、表 3](https://arxiv.org/pdf/2010.15258)；第三行：[DNSMOS P.835，实验设置与表 3](https://arxiv.org/pdf/2110.01763)；第四行：[ICASSP 2022 DNS Challenge](https://arxiv.org/abs/2202.13288)。均访问：2026-11-02。

**我的判断：**“客观指标不能完全替代人耳”站得住；“客观指标普遍和人耳对不上”站不住。更准确的风险是：**总体排名相关性高，不保证某段吃字能被发现；在旧数据上有效，不保证新语言、新噪声、新失真或被针对性优化后的模型上仍有效。**

**近年的相关工作**

| 工作 | 已经做了什么 | 对本项目的含义 |
|---|---|---|
| [SCOREQ，NeurIPS 2024](https://arxiv.org/abs/2410.06675) | 用对比回归改善无参考质量预测的跨域泛化 | “让自动评分贴近人耳”不是空白；选择 SCOREQ 有研究依据，但仍需本项目定标 |
| [JSQA，2025](https://arxiv.org/abs/2507.11636) | 使用基于刚可察觉差异的音频对进行感知预训练 | 人耳可辨差异和偏好监督已有方法研究 |
| [URGENT 2024 分析，Interspeech 2025](https://arxiv.org/abs/2506.01611) | 比较多种指标与人工 MOS；强调综合评价；公开相关性分析及 MOS 数据 | 可直接用于检验本项目评分器，而不必只依赖作者自己的听感 |
| [URGENT 2025 多语言 P.808 研究](https://arxiv.org/abs/2507.11306) | 指出生成式增强还需语音内容／音素保真指标，以发现幻觉 | “自然、干净”与“说对了什么”必须分别测 |
| [UrgentMOS，2026-01 预印本](https://arxiv.org/html/2601.18438v1) | 结合多指标监督与偏好学习进行稳健质量评估 | 与本项目的人耳校准、多指标路线直接相邻，不能声称无人做 |
| [URGENT 2024／2025](https://arxiv.org/abs/2406.04660) | 将 WAcc／WER 等下游指标纳入增强评测 | “降噪后再跑 ASR”本身不新颖 |

以上均访问：2026-11-02。URGENT 的[官方分析仓库](https://github.com/urgent-challenge/urgent2024_analysis)还提供人工 MOS 与多种自动指标的比较；作者发现综合排名和部分新无参考指标有较强相关性，而非“一切自动指标都无用”。

**对本项目评测方向的判断**

依据你提供的[意向书](/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/.42cog/intent.md)和[状态板](/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/state/board.md)，目前是 SCOREQ 为主、ASR 掉词探针、分场景回归、实时性能及盲听组合。

- **工程价值：明确。** 将这些方法接入同一实时处理代码、持续回归并公开失败样本，有实用价值。
- **方法新颖性：当前证据不足。** ASR、MOS 预测、盲听校准、综合指标均有先例；换用 Qwen3-ASR 或接入 CI 不自动构成研究创新。
- **可能形成贡献的地方——我的建议：**证明一个针对短促辅音、词尾、轻声、重叠语音的事件级指标，比现有 WER／SCOREQ 更能发现人耳确认的漏字；或者发布有多人盲听标签、跨设备和跨语言的 Mac 实时基准。

尤其需要修正“吃字率”的含义：**用 ASR 分别识别干净参考和处理结果，再比较掉词，测到的首先是识别输出差异，不是经过人工确认的真实吃字。** 建议保留人工核验文本，报告删除、替换、插入及处理前后的变化；把音频吃字事件另行标注。这是根据 WER 评测与上述内容保真研究提出的方法建议，不是对现有实现已完成的认定。

**4．职业与机会**

**近一年相关岗位：6 家公司的公开例子**

时间窗口为 **2025-11-02 至 2026-11-02**。明确发布日期与“本次仍可见岗位正文”分开；后者能证明近期公开招聘信息存在，但不能保证申请通道此刻仍开放。

| 公司 | 岗位与链接 | 日期／状态证据 | 与本项目的关系 |
|---|---|---|---|
| **Apple** | [Speech Processing ML Algorithm Engineer](https://jobs.apple.com/hu-hu/details/200680823-0836/) | 官方发布 **2026-08-29** | 直接要求低延迟、实时语音增强，包括降噪、去混响、回声抑制，原型到量产 |
| **Google** | [Audio Machine Learning Engineer, Google Beam](https://www.google.com/about/careers/applications/jobs/results/106471308924986054-audio-machine-learning-engineer-google-beam?hl=zh-MO&page=75) | 2026-11 本次可读官方岗位；页面未显示首次发布日期 | 语音增强、声音分离、音质评估、C++ 音频基础设施、CPU／GPU 低延迟推理 |
| **Dolby** | [AI Architect – Media](https://careers.dolby.com/job/Atlanta-AI-Architect-GA-30301/1372792800/) | 官方发布 **2026-10-30** | 端侧 ML／NPU、DSP 与模型整合、量化蒸馏、流式因果推理、质量与延迟权衡 |
| **Cerence AI** | [Senior Research Engineer AI Speech Enhancement](https://cerence.wd5.myworkdayjobs.com/Cerence/job/Ulm/Senior-Research-Engineer-AI-Speech-Enhancement_R0005975) | 官方记录截止 **2026-09-25**，应按近一年历史招聘处理 | 汽车等场景 DNN 增强、训练评测工具、生产落地；不是当前仍开放的保证 |
| **Amazon** | [Senior Inference Engineer, AGI](https://www.amazon.jobs/en/jobs/10503632/senior-inference-engineer-agi) | 2026-11 本次可读官方岗位；首次发布日期未核实 | 实时语音／音频模型推理、硬延迟预算、流式运行与评测；更偏服务端而非 Mac 降噪 |
| **NVIDIA** | [Speech LLM Engineer, Voice-First Agentic AI](https://nvidia.wd5.myworkdayjobs.com/nvidiaexternalcareersite/job/Speech-LLM-Engineer--Voice-First-Agentic-AI_JR2020909) | 2026-11 本次检索可见官方岗位；首次发布日期未核实 | 语音／多模态模型及语音质量、延迟、可靠性基准；不是专门的端侧降噪岗 |

全部访问：2026-11-02。

**查到的事实：**岗位确实覆盖本项目能够展示的技能，但多数要求多年经验，部分要求硕博或研究经历。Google 明列较长的软件与音频 ML 经验要求；项目作品不能自动替代这些要求。[Google 岗位](https://www.google.com/about/careers/applications/jobs/results/106471308924986054-audio-machine-learning-engineer-google-beam?hl=zh-MO&page=75)。

**我的推断：**对本项目最有说服力的职业成果，是可以解释的故障定位、真实实时性能、清楚的质量取舍和可复现比较。“用了 Rust”“接了模型”“用了多 agent”本身都弱于这些证据。Amazon 岗位甚至明确把开源推理／内核项目贡献列为加分项，但这仍不意味着任意开源音频项目都会带来录用。[Amazon 岗位](https://www.amazon.jobs/en/jobs/10503632/senior-inference-engineer-agi)。

另查了 [Krisp 当前招聘页](https://krisp.ai/careers/)：可见 Voice AI Solutions Engineer 等岗位，但本次没有确认到当前开放的核心降噪算法岗位。因此不能把桌面文件列出的所有公司都写成“现在正在招这类工程师”。访问：2026-11-02。

**开源作者的公开去向：能证实什么？**

| 案例 | 查到的事实 | 不能推出什么 |
|---|---|---|
| **RNNoise 作者 Jean-Marc Valin** | 本人履历：Mozilla **2011–2019**；AWS **2019–2024**；Google **2024 起**。RNNoise 论文署名单位已经是 Mozilla。[个人简介](https://jmvalin.ca/index.html)、[CV](https://www.jmvalin.ca/cv.html)、[RNNoise 论文](https://jmvalin.ca/papers/rnnoise_mmsp2018.pdf) | 不能说他“做 RNNoise 后才进入 Mozilla”；也没有查到 AWS／Google 是因 RNNoise 单独招募他的直接证据 |
| **DeepFilterNet 作者 Hendrik Schröter** | FAU 官方页面记录其语音研究及助听器低延迟降噪工作；开源项目与论文可核实。[FAU 项目页](https://www5.informatik.uni-erlangen.de/en/our-team/schroeter-hendrik/projects/index.html)、[DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) | 本次未核实其最新商业任职，更未找到“DeepFilterNet 直接带来某份工作”的公开因果叙述；同名搜索结果不能冒认 |

访问：2026-11-02。**严格回答：本次能证明开源音频贡献与相关职业经历共存，也能证明雇主看重部分开源贡献；没有找到足够直接的证据，把上述两位作者写成“靠这个开源降噪项目获得工作”的成功案例。**