# r5 · 自监督信号从哪来 + 老虎机 / 偏好学习做参数选择

> 2026-10-30 · 只检索不运行 · 等级：【官方】【论文】【源码】【二手】【推算】

## 预判与证伪

- 预判 A「Widex 20 次比较就收敛」→ **部分成立**。原始论文写的是 **10–20 次成对评估（5–10 分钟），最多 4 个参数**，而且答题用的是「偏好程度」滑条，不是二选一；厂商稿写的是「20 步以内 / 约一打」。
- 预判 B「无参考质量分可以直接当奖励」→ **证伪**：域外泛化差，还会被优化方「讨好」（见 §2）。
- 预判 C「Krisp / Zoom 公开过在线自适应」→ **没找到**。

## 1 无标签在线信号

- **纯噪音段估计噪声谱**是经典做法：最小统计（minimum statistics），Martin 2001，IEEE TSAP 9(5):504–512，作者称适合实时实现 https://www.iks.rwth-aachen.de/publikationen/publications-detail//martin01c/ 【论文】。IMCRA（Cohen 2003，TSAP 11(5)）用语音存在概率做软判决，不做硬 VAD 判断 https://webee.technion.ac.il/Sites/People/IsraelCohen/Publications/sp_Nov2001.pdf 【论文】。**陷阱**：硬 VAD 把轻声、尾音判成噪音，会把语音吃进噪声谱，所以应该用软概率加长时最小值。
- **主人含噪语音当伪目标（个性化自监督）**：Sivaraman 等人按 SNR 给帧加权做「数据净化」，不用干净语音和声纹 arXiv:2104.02018 【论文】。期刊版：IEEE JSTSP 16(6) 2022 【论文】。
- **在现成模型里选（与我们的「只选不训」同构）**：同组用声纹门控在若干小专家模型里选一个，小专家组合胜过大通用模型 arXiv:2105.03542（WASPAA 2021）【论文】。
- **教师-学生自训练**：RemixIT 把教师分出的语音和噪音重新混合，生成伪标签 arXiv:2202.08862 【论文】。它要做反向传播，只能放学习回路。
- **用户动作**：见 §4。
- **厂商**：Krisp 只宣称本机处理、不传音频，提供「背景人声消除」（2023 年存档首页）https://webcf.waybackmachine.org/web/20230223194753/https://krisp.ai 【二手】。Krisp、Zoom 用在线自监督或用户信号做自适应：**没找到**。
- **把邻座当主人**：声纹在短语音上明显退化，评测语音从 3.59 s 缩到 2.05 s 时 EER 相对上升 46% arXiv:1810.10884 【论文】。所以声纹匹配要叠加「高 SNR、≥3 s、连续多段一致」才能当清晰语音段。微软 pDNSMOS 专门惩罚漏进来的旁人声（DNS-Challenge `DNSMOS/dnsmos_local.py` 第 108 行）【源码】。

## 2 无参考质量分能否当奖励

- **DNSMOS P.835**：模型级 PCC 为 SIG 0.94、BAK/OVRL 0.98；单条片段级更弱，因为每条只有 5 个评分 arXiv:2110.01763 【论文】。输入 16 kHz，9.01 s 窗、每 1 s 滑动一次（`dnsmos_local.py` 第 19–20、61–72 行）【源码】。权重 `sig_bak_ovr.onnx` 1.16 MB https://huggingface.co/spaces/alibabasglab/SpeechScore/tree/8d66ced92fad18c7c738e9a9197efc7cce17e48d/scores/dnsmos/DNSMOS 【二手】。按 fp32 推算 1.16 MB/4 B ≈ 29 万参数，约 GTCRN 的 6 倍【推算】；MACs 没找到。
- **泛化**：DNSMOS 在 DNS3 上好，换数据集就差；NISQA 更广 arXiv:2502.05356 【论文】。CHiME-7 UDASE 的结论是无参考指标与主观分「相关有限」，降噪「总以失真增加为代价」arXiv:2402.01413 【论文】。
- **被讨好**：只优化 DNSMOS 会得到高分但其他维度变差的输出，需要 DNSMOS + (1−WER) + UTMOS 组合奖励 arXiv:2606.21458（Interspeech 2026）【论文·只读到摘要】。MetricGAN-U 直接拿 DNSMOS 当训练目标 arXiv:2110.05866 【论文】。
- **SQUIM**：无参考估计 STOI / PESQ / SI-SDR，用 DNS2020 训练 arXiv:2304.01448 【论文】。权重约 28 MB【二手】。
- **结论**：只能做慢回路里的**否决和比较**，不能当在线奖励；要多指标组合，单条分数不可信。

## 3 老虎机 / 偏好学习的样本量

- **Nielsen, Nielsen & Larsen**（IEEE/ACM TASLP 23(1):162–173）：GP 加主动学习（双变量 EI），**10 名听损受试者，10–20 次成对评估（5–10 分钟）即可个性化最多 4 个参数**；每轮最多 30 次（摘要和第 III-B 节）http://www2.imm.dtu.dk/pubdb/edoc/imm6848.pdf 【论文】。
- **Widex SoundSense Learn**：3 个频段、每段 13 档，共 2197 种组合，「20 步或更少」；84% 受试者偏好它的设置 https://hearinghealthmatters.org/hearing-technologies/2018/soundsense-learn-machine-learning-widex-evoke-hearing-aid/ 【二手·厂商作者】。另一处说「约一打」https://www.eenewseurope.com/news/ai-optimizes-hearing-aid-performance-real-time 【二手】。注意：Widex 匿名程序可以存云端，我们不许。
- **Vyas 等 2022**（Smart Health）：在预设里成对比较，**模拟中位数 25 次**，比最好的基线少一半；真实用户的回答有噪声 https://pmc.ncbi.nlm.nih.gov/articles/PMC10312409 【论文】。
- **纯老虎机推算**：最优臂辨识的复杂度 ≈ Σ1/Δ²（Kaufmann 等 JMLR 2016，arXiv:1407.4443）【论文】。取 K=4、标准化噪声 σ=1、差距 Δ=0.3、δ=0.05：2·(4/0.09)·ln20 ≈ 2·44·3 ≈ **270 次拉臂**【推算】。拧旋钮约 1 次/会，就要几百场会，**不可行**。稠密的隐式信号每 10 s 算一次，约 45 分钟可行，但前提是奖励无偏。

## 4 隐式反馈怎么读

- **调低降噪有歧义**：最压噪音的算法偏好分最高、可懂度却最差；不同人偏好的降噪强度差异大，还随噪声类型变化 https://proceedings.isaar.eu/index.php/isaarproc/article/view/2011-54 、PMC5017568 【论文】。光看「调低」分不清是吃字还是不自然，要结合当时的 SIG/BAK 估计和场景。
- **可训练助听器**：Keidser & Alamudi 2013（Ear Hear 34(5):619–629）：想要改动的用户「大多训练有效」，可靠性数据有限 https://researchers.mq.edu.au/en/publications/real-life-efficacy-and-reliability-of-training-a-hearing-aid/ 【论文·摘要】。Zakis 等 2007 的具体次数**没找到**。
- **点击是有偏的**：点击「有信息但有偏」，宜转成**相对偏好**（Joachims 等 SIGIR 2005）https://www.sigir.org/wp-content/uploads/2017/06/p003.pdf 【论文】。
- **把「撤销 / 关掉」当负反馈的音频先例**：**没找到**。

## 5 冷启动与安全探索

- **保守老虎机**：任何时刻累积回报都不低于基线的 (1−α)，用默认臂兜底，代价是不可避免的额外遗憾 arXiv:1602.04282（ICML 2016）【论文】。
- **SafeOpt**：只在高概率安全的集合里扩张，已用于脊髓刺激调参 https://proceedings.mlr.press/v37/sui15.html 【论文】。
- **每个场景放几个候选**：文献里没有直接数字。按上面的推算，K≤4 才能在一两个小时的会议时长里辨识出来【推算】；助听器的成功案例都是 2–4 个参数。

## 对我们的含义

- **回路**：反射回路只用软 VAD 加最小统计更新噪声谱，零学习。判断回路（约 0.5 s）在 3–4 个预设里做**保守汤普森采样**，奖励来自多指标组合，每段 ≥10 s 才切一次，用户拧旋钮或撤销当**强制否决**而不是奖励。学习回路（空闲时）跑 DNSMOS / NISQA 影子比对，用主人高 SNR 段做 RemixIT 式小适配。
- **预算**：DNSMOS 9 s 窗只在空闲时离线跑；判断回路只用轻量特征（SNR、语音存在概率、声纹分）。
- **护栏**：默认臂 = 出厂配置，满足保守约束；切换时交叉淡入淡出；声纹门槛 ≥3 s；所有日志只在本机。
- **首选**：保守汤普森采样 × K≤4 预设 × 隐式稠密信号，加旋钮否决。
- **备选**：一次性主动的「调音向导」，GP 成对偏好学习，10–20 次 A/B，参照 Nielsen 2015。
- **三个风险**：①无参考指标被讨好或域外失效；②邻座人声被当成主人，造成伪目标污染；③「调低降噪」语义含糊，奖励方向可能反了。
