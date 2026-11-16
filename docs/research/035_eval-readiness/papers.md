# 035 · 论文清单

> 全文 PDF 在 `resources/papers/eval/`（被 git 忽略，不进仓）。哈希 = `shasum -a 256`。
> 「读完」= 逐页读完全文，由 3 个只读子代理分篇读，主会话抽查过的数字见 `README.md` 第六节。页码按 PDF 页。
> 一行一篇：文件名 · 哈希 · 是否读完 · 要点。

| 文件 | sha256 | 读完 | 要点 |
|---|---|---|---|
| 2025-zhang-urgent2024-lessons.pdf | cf52aa11d8b3154e9e81cde0f06c1fce48831ba98f7a004d04ce3d700031bc7a | 是（5 页） | 盲测一半合成一半真实，排名只看盲测（p1）；300 段 P.808、每段 8 人（p3）；两队 DNSMOS / NISQA 高而人评差，UTMOS、SCOREQ 与人评最相关，WADA-SNR 最差，总排名与 MOS 排名 Kendall 0.73（p4）；干净语料带噪标签、SNR 不代表难度（p2–3） |
| 2025-shi-urgent2025-p808-multilingual.pdf | 98ed854661e56ca2b44d14fc668b2de063b3f2d6a6509ed26c9fc996e07a1461 | 是（5 页） | P.808 多语种：每段 ≥ 8 评分、金标与陷阱题（p2）；生成式模型无参考分最高但 ESTOI / LPS 最差、有幻觉（p4）；判别式 PESQ / ESTOI 第一，MOS 却输（p4）；建议今后用 CCR（p4） |
| ../training/2025-saijo-urgent2025-challenge.pdf（已在 training 目录，不重复存） | 14f90c34339062882767ef4fa41d89a8f5765e425537e8eb22a0ace7c0b1c674 | 是（5 页） | 盲测一半真实、多来自没见过的数据集（p2–3）；生成式 T13 的 DNSMOS / NISQA / MOS 第一、LPS 最低，有幻觉（p4）；ACR 罚不到内容错误，CAcc 抓得到（p4） |
| 2024-leglaive-chime7-udase-eval.pdf | a8a36caa0984f109038024c952953880ce5c39c02fb5e6b02655809c73bf3933 | 是（21 页） | 真实录音 P.835 实验室测试，32 人，每条件每维 1024 票（p7–8）；贴近目标域的合成集 LibriCHiME-5（真实噪声段、实测 RIR、SNR 照目标域）上有参考指标排名与真实录音人评一致（p5–6、16）；优化 DNSMOS 的 CMGAN DNSMOS 最好、人评最差（p16）；无参考指标与人评 PCC ≤ 0.50，只有 DNSMOS BAK 0.73（p16）；白噪声输出 DNSMOS SIG 2.86（p12）；随机 / 理想两个对照条件（p11）；重复测量 ANOVA、α = 0.01、Holm（p14）；DNSMOS 对响度敏感（p7） |
| 2024-deoliveira-pesqetarian.pdf | 528135b8625645c201beb7749254d0a8a6939ffa52b89226a7313f347a46b7b4 | 是（5 页） | 只优化 PESQ 刷到 3.82，POLQA 1.46、听测最差（p3）；首样本改 666 即 PESQ 3.46（p3–4）；钻空子随初始化而变（p3）；训练指标不能单独拿来评（p4） |
| 2021-reddy-dnsmos.pdf | 2e5975d615e5348d35d82bd34f4cc4d277d571a429c160eaa27305e22442f2ef | 是（5 页） | DNSMOS 按模型 PCC 0.93（p3 表 2）；逐段两轮人评只相关约 0.5，按条件平均 > 0.9（p3）；分布一变相关就掉，唱歌 0.36（p4）；人给低音量打低分（p3） |
| 2022-reddy-dnsmos-p835.pdf | 11488d4c0d300f3118cd7bd1f04fe6931ca1e7434a6e5d0c6a1f0c3a477d7901 | 是（5 页） | 按模型 SIG / BAK / OVRL PCC 0.94 / 0.98 / 0.98，逐段 0.71 / 0.83 / 0.82（p4 表 3）；SIG 最难（p3）；只该按模型或条件汇总（p4） |
| 2022-dubey-dns-icassp2022.pdf | 60943c216ac05e2722607cc7151082e5924db4af39bdb4b56a123f7036e58bd3 | 是（5 页） | 开发集和盲测集都是真实录音，盲测 859 段，截止前 5 天发（p1–3）；P.835 + WAcc 排名，ANOVA 判平局（p3–4）；往届过度压制让 WAcc 变差（p1）；没有一队真提升 SIG，WAcc 比带噪差 2%（p4） |
| 2023-dubey-dns5-icassp2023.pdf | a8b695ffefe6ee0de0825bc3f35165d0777a0e469190f165083e8ce872883ad4 | 是（6 页） | 盲测是未公开的真实录音（p3）；WER > 0.5 常因旁人漏出、成插入错误（p4）；多数模型 SIG 下降（p5） |
| 2021-naderi-p835-crowdsourcing.pdf | 690b301bb85316f15a2c50f264f3e566680e0a8350e66cfe02eef428fad6ad66 | 是（5 页） | 众包 P.835 与实验室 PCC 0.925 / 0.984 / 0.974（p3 表 2）；OVRL = −0.844 + 0.644 SIG + 0.452 BAK，SIG 权重约 1.4 倍（p3）；BAK 拉满 OVRL 也只到 3.92（p4）；多数队 BAK +0.85–1.91、SIG −0.10–−0.78，一队 OVRL 比带噪还低（p4 表 4） |
| 2020-naderi-p808-open-source.pdf | 02805007b02a4741b48c7a7656cb8cecc1aade1ea364aa2dfc43bfe4ef272df8 | 是（5 页） | 众包与实验室 PCC 0.954（p3）；各轮绝对 MOS 漂移，改用减带噪原声的 DMOS 后 ICC 0.719 → 0.907（p4）；金标与环境测试显著提高一致性（p4 表 3） |
| itu-t-p808-2021.pdf | da0e7e96462da9c4434f428a6b5a58603d840a4c34d4b0ebb27254afd7370918 | 是（36 页） | ACR 每条件 96 票、每段 ≥ 8 人（p11–12）；筛人与作废规则（p13–16）；按条件报均值、标准差、95% 区间（p18）；DCR / CCR 对高质量更灵敏（p19、21）；附件 D 带噪语音分 SIG / BAK / OVRL 三项评（p23–25） |
| 2023-gonzalez-generalization-gap.pdf | 8ef3e78186ccca8f0508c368e98dc434fd6c40695c39fca3e7c9ca65dc13db04 | 是（13 页） | 同库不重叠切分算「匹配」（p2、4）；同库训测高估 ΔPESQ 最多 50%（p8）；说话人换库伤最重，单库 Conv-TasNet GΔESTOI −68%（p6）；匹配下最强的模型换库后被最简单的反超（p8）；只用一组训测库可能挑到最好的一折（p6、10）。表 V 列序错位，−38% 归谁未核 |
| 2022-iwamoto-se-artifacts-asr.pdf | b3cf315973b1bfcdd58d9155af259a62e9811702e209e8dc90132f7dccd6a3e1 | 是（5 页） | 增强误差拆成噪声残留与伪影，识别变差主要怪伪影（p1、4）；加回带噪原声 ω 0.3–0.8，CHiME-3 真实录音相对 WER 约改善 20%（p4） |
| 2021-bouthillier-variance-ml-benchmarks.pdf | 1f458f7e606e4db86b207315c049a38b52cacde15ad34def2cc6c29124fc5d4c | 是（23 页） | 数据抽样、初始化、数据顺序、超参的方差与已发表改进同量级（p4–5）；只跑一次比较漏判约 75%、误报约 10%，P(A>B) 法约 30% / 5%（p9 图 6）；γ = 0.75 需 N ≥ 29（p16）；配对种子降方差（p15）；硬件与库版本影响结果（p14） |
| 2018-dror-significance-testing-nlp.pdf | e3b6e07cf7e404443e4b516db9b2a72c5d24a38285a9a6691ed2e3b01b02235b | 是（10 页） | 选检验的决策树：小样本用随机化检验，bootstrap 在小测试集上差（p6）；多数据集比较极少校正（p7–8）；样本不独立时常用检验失效（p8） |

## 无法获取全文

| 文献 | 原因 |
|---|---|
| ITU-T P.835（现行 2026-07 版，T-REC-P.835-202607-I） | ITU 下载链接返回 500 服务器错误（2026-11-16 实测）；旧版 2003 已被取代，同样 500。用 P.808 附件 D 与 Naderi 2021 代替 |
