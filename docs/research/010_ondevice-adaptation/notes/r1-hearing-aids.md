# r1 · 助听器的环境分类、自动切换、偏好学习（公开原理）

> 2026-10-30 · 只读检索，未运行任何程序。等级：【官方】【论文】【源码】【二手】【推算】。厂商白皮书算【官方】，但都是自家说法、自家实验。

## 1. 分几类 · 用什么特征 · 分类器形态

| 厂商 | 类目 | 特征 / 分类器 | 出处 |
|---|---|---|---|
| Phonak AutoSense OS 3.0 | 7 类：Calm situation、Speech in noise、Speech in loud noise、Speech in car、Comfort in noise、Comfort in echo、Music；外加串流 2 类（Media speech / music） | 电平差、估计 SNR、跨频带起音同步（onset synchrony）、幅度、频谱；**每 0.4 s** 算一次各类概率；「机器学习训练、数千个声场景」 | 【官方】AB 白皮书 https://www.advancedbionics.com/content/dam/advancedbionics/documents/regional/es/seorl-2025/marvel-ci-resources/AutoSense%20OS%20White%20Paper.pdf.coredownload.pdf p1 |
| Phonak AutoSense OS（2015 版） | 同上 7 类 | 「35 维特征提取」 | 【官方】Rakita & Jones, Hearing Review 2015-12 https://www.phonak.com/content/dam/phonak/en/evidence-library/journal-article/autosense_os_hearing_review.pdf p1 |
| Phonak（学术前身） | — | Büchler/Allegro/Launer/Dillier, *EURASIP JASP* 2005:18, 2991–3002（doi:10.1155/ASP.2005.2991），听觉场景分析思路；全文未取到，分类器细节**没核实** | 【论文】 |
| Phonak OS 5.0/6.0/7.0 | 5.0 的类目表**没找到**；6.0 在「嘈杂」时自动切到 Spheric Speech in Loud Noise，由独立 DEEPSONIC DNN 芯片执行；7.0 称训练环境量 ×18、分类准确率 +24% | 【二手】AudiologyOnline 访谈 https://audiologyonline.com/interviews/next-level-sound-processing-with-29099 ；【官方】FieldStudyNews ASOS7 https://www.phonak.com/content/dam/phonak/en/evidence-library/field-studies/PH_FieldStudyNews_ASOS7-0_210x280_EN_028-2784-02.pdf （未细读） |
| Oticon More / Intent（MSI） | **不分离散类**：连续估计「环境复杂度」，再与验配时设定的个人需求比较；易 → Virtual Outer Ear，难 → Spatial Balancer + DNN 降噪 | DNN（离线训练，1200 万真实声场景）做**降噪**而非分类；Intent 加 4D 传感：电平、SNR、头动、身体运动、对话活动；调节跨度 5 dB 输出 SNR | 【官方】Oticon More 白皮书 2020 p4–5 https://forum.hearingtracker.com/uploads/short-url/9cZSSgyKTUtOY3paRzZqJMNWsK0.pdf ；Intent Product Guide 2024 p3–4 https://forum.hearingtracker.com/uploads/short-url/n9zZ7R6lldsf7MGhYMP7oe03xTQ.pdf |
| Widex EVOKE/MOMENT | 440 档 11 类：Quiet/Urban/Transport/Party（各分有无语音）+ Music Classical/Contemporary + Social；低档 7/4/3 类 | 特征、分类器**没找到** | 【二手】Hearing Review 2019 https://hearingreview.com/hearing-products/real-life-hearing-aid-benefit-widex-evoke ；PureSound 是低延迟通路，**与分类不对题** |
| Starkey Genesis / Edge AI | 分类类目**没找到**；Edge AI：DNN「常开、在主音频通路」，算力为 Genesis 的 100 倍，51 h 续航 | 「机器学习分类」识别安静/嘈杂/风等 | 【二手·厂方访谈】HHTM 2024-10-15 https://hearinghealthmatters.org/?p=38090 |
| Signia Xperience / IX | 自称「不归入预定义类」；声学 + 运动传感（走/跑/静止）+ 本人声音（OVP）+ SNR | 分类器形态**没找到** | 【官方】https://www.signia-pro.com/en/blog/global/signia-yoursound-technology-explained/ |
| ReSound Environmental Classifier | 7 类：quiet <54 dB、soft speech <60、loud speech >60、speech in moderate noise <75、speech in loud noise >75、moderate noise <75、loud noise >75 | 频谱内容、谱平衡、时间特性做语音/噪声检测；「概率模型」 | 【官方】https://www.gnhearing.com/en/research/features-explained/environmental-classifier |

综述口径：高端分类器最多约 9 类，特征从总电平、分频带电平到调制频率/深度、MFCC，分类器有 Bayes、NN、HMM；**输出各类权重恒和为 100%**，后处理要么硬切、要么按类混合。【论文】Yellamsetty et al., *Trends in Hearing* 2021, doi:10.1177/2331216520980968 https://pmc.ncbi.nlm.nih.gov/articles/PMC7989119

## 2. 切换多快 · 怎么平滑

- **ReSound**：判定环境 ≤4 s；Environmental Optimizer 换增益时**以 1 dB/s 渐变**；双耳每 222 ms 交换一次环境数据。【官方】https://www.gnhearing.com/en/research/features-explained/environmental-optimizer
- **Phonak**：每 0.4 s 出一次概率（见上）；4 个可混合类最多 3 类同时混合，Speech in loud noise / car / Music 互斥不混；按比例混合得「200 多种可听出差异的设置」；有「大量平滑过渡，避免可闻的突变」，双耳同步。具体秒数**官方没找到**；论坛用户称验配软件里能调切换速度，「快」档约 10 s【二手】https://forum.hearingtracker.com/t/-/49957/35 。【官方】Rakita & Jones 2015 p2；AB 白皮书 p1
- **Oticon**：连续调节，无类间硬切；扫描「每秒 500 次」是风噪/操作噪声检测（Wind & Handling Stabilizer），不是场景分类。【官方】Intent Guide p4
- **滞回**：只有 Hearing Review 一句「需在一定时长、一定电平上被检测到才切」【官方·无数字】https://hearingreview.com/inside-hearing/research/automatic-feature-steering-adaptation-seamless-listening-experiences 。Yellamsetty 2021：切换节奏「各家专有，临床医生基本不知道」。

## 3. 偏好学习

- **GN ReSound Learning VC（ISAAR 2007）**——公开最完整：按 7 个环境类**各学一个音量偏移（dB）**；更新式 `G_lvc=|θ|·μ`，`μ[n]=α·μ[n-1]+(1-α)(G_vol[n]-G_init)`；θ 是用户一致性置信度——**操作不一致就降学习率直到不学**；只在「音量旋钮停留一段时间」视为显式同意才计入一次。41 耳·2 周：平均绝对学习量 2.4 dB，个体差异大；再戴 2 周增量降到 0.95 dB → **2–3 周饱和**。用户拧旋钮永远立即生效。【论文】Leenen et al. https://proceedings.isaar.eu/index.php/isaarproc/article/download/2007-50/278/580 p508–512
- **Siemens/Signia SoundLearning**：1.0（2008）按输入电平学增益/压缩/频响；2.0 按 speech/noise/music 3 类分别学压缩和频响，事件触发（每次调音量或 SoundBalance 时记录当前电平与环境），混合场景用连续插值；约 2 周收敛；验配师可重置单类/全部或不应用。n=14·2 周：言语可懂度约 +20%（p<0.001）。【官方】Hearing Review https://hearingreview.com/inside-hearing/research/finding-optimum-gain-compression-and-frequency-shape-in-different-listening-situations
- **Widex SoundSense Learn**：手机端 A/B 成对比较 + 高斯过程主动学习。Nielsen, Nielsen & Larsen, *IEEE/ACM TASLP* 2015, doi:10.1109/TASLP.2014.2377581：**10–20 次成对比较（5–10 分钟）就能个性化最多 4 个参数**（10 名受试）【论文·据摘要】。Hearing Review 称「约 20 次迭代」；n=19、9 个模拟环境，84% 偏好学到的设置；言语可懂度无提升【官方】https://www.hearingreview.com/hearing-products/hearing-aids/real-life-applications-machine-learning-hearing-aids-2 。「3 参数 × 13 档 >2000 种设置」【二手】unite.ai。第二代快 25%【二手】。Jensen et al. 2019 *Trends in Hearing*：偏好增益高度个体化、难预测【论文·据摘要】。
- **Widex My Sound**：**云端**汇总全球用户的「场景 + 偏好」数据，按活动/意图推荐 2 套设置；「每次改设置，偏好都由我们云端 AI 处理」。【官方】https://www.widexpro.com/en-us/evidence-technology/sound-sense-learn ；https://www.widexpro.com/en-us/local/en-us/press/my-sound-ai-enabled-feature-for-personalization/
- Phonak / Oticon 的现行学习音量：**没找到**官方说明。
- 结论：**学的都是旋钮值**（音量 dB、压缩、频响、EQ 档），按环境类分别存；不是网络权重。

## 4. 有没有厂商在设备上更新神经网络权重？——预判**成立**，有一处要修正

- Starkey CTO：网络「预训练」，运行时只有参数随输入变化；被问到能否学习用户的重复选择时答「想做，但我们没做」。【二手·厂方原话】HHTM 2024
- Oticon 专利 US12143775（优先权 2019-10-08）：候选网络**离线训练**，设备/外部设备只**在预训练候选里选**；提到「可只更新最后几层」但没说产品已实现。【源码级·专利】https://patents.google.com/patent/US12143775
- **修正**：Widex 的偏好学习有一部分在**云端跨用户**做（My Sound），所以「全在本机」不成立——这条正好落在我们「无云端」的禁区里，只能学原理，不能照搬。
- 没找到任何厂商公开说在设备上更新 DNN 权重。

## 5. 可控 vs 自动

- Phonak 约 15 万次验配里 92% 把自动程序设为开机程序；Lübeck 研究：受试者**挑不准**哪个手动程序听得更好；n=14 实景（客厅、车、咖啡馆）全都更偏好 AutoSense，车内和混响室里言语得分更高（未给百分比）。【官方】Rakita & Jones 2015 p2–3
- 手动程序覆盖自动、按住按钮或重新充电后回到自动——只有论坛说法【二手】https://forum.hearingtracker.com/t/phonak-users-who-dont-use-autosense/77546
- 错判率：Cui & Groth 2017（ReSound 赞助）：安静/语音/白噪声 ≥96%；speech-in-noise ReSound 98%，其余 ≤60%；某机把超市语音 100% 判成音乐。【官方·利益相关】https://www.audiologyonline.com/articles/accurate-environmental-classifiers-in-hearing-19796 。Yellamsetty 2021：同一 5 dB SNR 美食广场场景，五家判成 speech-in-noise 的比例在 38–100%；引 Groth & Cui 平均约 67%。【论文】
- Starkey Edge Mode+：自动判断环境大变，开了以后一直开着，直到用户手动关。【二手】

## 对我们的含义

**回路归属**：分类 + 选参数 → 判断回路（0.4 s 一拍，与 Phonak 同级）；渐变 → 反射回路只做插值（参照 1 dB/s · 判定 ≤4 s）；音量/强度偏好学习 → 学习回路（空闲时更新，按场景类各存一个 dB 偏移）。
**预算量级**【推算】：每 0.4 s 算约 35 维特征 + 小 GMM/逻辑回归，占 GTCRN 33 MMACs/s 的 <1%；偏好表 = 场景类数 × 若干标量，<1 KB。
**护栏**：一致性门控（θ 低就不学）；只在用户停手一段时间后才算「同意」；手动永远立即生效并覆盖自动；可重置单类/全部；互斥类不混合；双耳/双通道同步。

- **首选**：ReSound LVC 式——规则 + 概率分类（4–7 类，混合权重和为 1）+ 每类一个 EMA 偏移 + 一致性门控，2–3 周饱和。全在本机，可解释，能回滚。
- **备选**：Widex 式主动 A/B（GP 偏好学习，10–20 次比较），只在用户主动点「帮我调」时触发，结果存为该场景的偏好。
- **风险**：①分类错判（各家 speech-in-noise 38–100%），参数随之跳 → 必须渐变 + 滞回 + 有手动兜底；②学到用户的「反射动作」（LVC 论文里真遇到过：有用户换环境就条件反射地拧固定量）→ 一致性门控 + 上限；③业界没有任何「本机更新 DNN 权重」的先例，我们的空闲小更新缺对标 → 只动调节向量/归一化统计，影子比对通过才换，且可关。
