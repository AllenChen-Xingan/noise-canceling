> workflow `wf_16dd3e65-ebc` · data 路怀疑派原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

# 怀疑派复核：第④路「Mac / 笔记本内置麦设备数据」

**结论先行**：我复核了 24 条结论。DR-VCTK 的许可、条数和客观分都能复现。有 4 处要改：
- **机型分界线画错了。** 三麦阵列从 2018 年 Intel Retina Air 就有了，分界线不是 Intel 和 Apple 芯片。
- **「两个声道不是同一路复制」的论证不成立。** 第 1、2 声道是不同的句子，不能拿来比。
- **3D-Speaker 的 96,262 条是全库笔记本条数，不是训练集。**
- **要点 2 的「2010–2017 年款」和逐条表的「2013–2017」自相矛盾。**

下面这张表逐条列出判定和依据，用来决定报告哪里要改。

| # | 结论 | 判定 | 理由与核到的原文 | 修正建议 |
|---|---|---|---|---|
| 1 | DR-VCTK 许可 CC BY 4.0，可以训练 | 成立 | 我重新取回 README（`datashare.ed.ac.uk/bitstream/handle/10283/3038/README.txt`），原文：「You are free to use this database under Creative Commons Attribution License (CC-BY)… Attribution 4.0 International (CC BY 4.0)」。两份许可文件中「Depositor Agreement」那份没有复核。 | 保留。可注明 CC BY 要求署名，引用要写 VCTK 与 DR-VCTK 两条 |
| 2 | 能刻画 Mac 内置麦的公开数据「只有 DR-VCTK 一份」 | 存疑 | 这是全称否定，但报告只查了约 20 个集。漏了以下几处（见文末）：Microsoft 回声消除挑战赛（AEC Challenge）的真实录音、DAPS、Asahi Linux 的麦克风资料 | 改成「在已查的约 20 个集里，只有 DR-VCTK 带干净参考」 |
| 3 | 「同一句话的 MacBook Air 录音和干净原声逐条对齐」 | 成立，但缺限定 | 我统计了 `train_ch_log.txt`：11,572 条录音，每个文件名只对应 1 个声道，没有重复。也就是说，每句话只出现在 8 个声道中的 1 个。和干净原声是对齐的，但不同设备之间、两个 Air 声道之间都不是同一句话 | 加一句：「各声道句子互不重叠，不能做设备间逐句对比」 |
| 4 | 装置照片是楔形银边 13 寸 Air，日文键盘 | 成立（看图推断） | 我看了 `setup_s.jpg`：楔形 13 寸 Air，JIS 日文键盘，屏幕下沿印「MacBook Air」。iPad 上贴着「国立情報学研究所備品」 | 保留【推断】标记 |
| 5 | 机型年份范围：要点写「2010–2017 年款」 | 推翻（自相矛盾） | 逐条表写的是 2013–2017。2010–2012 款 Air 只有单麦。EveryMac 原文：「Mid-2012 models have a single side mounted internal microphone whereas the Mid-2013 models have dual」。Mid 2013 的规格页也写「Dual microphones」 | 统一成 2013–2017（Mid 2013 起才有双麦） |
| 6 | 「数据 2017 年录制」 | 存疑 | README 只写「RELEASE November 2017」，没写录制日期。用 2017 款规格页来佐证机型属于以偏概全，Mid 2013 到 2017 都是双麦 | 改成「2017 年 11 月发布；录制时间未写」，规格页改引 Mid 2013 到 2017 一组 |
| 7 | README 写「MacBookAir (ch.1)/(ch.2)」，所以是双麦 | 存疑 | README 里 iPhone 5S、iPad 同样标了 (ch.1)/(ch.2)。iPhone 5S 第 5、6 声道的中位分几乎一样（关窗 CSIG 1.63 和 1.62，PESQ 1.33 和 1.32），像是同一路信号拆成两份。所以「标了 2 声道」不能证明有 2 个物理麦 | 改成「README 标 2 声道，和双麦规格相容，但不能单独作证」 |
| 8 | 现在的 Apple 芯片 Air 是「三麦阵列 + 波束成形」，所以 DR-VCTK 不能代表它 | 存疑（分界线错） | 2018 年 Intel Retina Air 就是三麦：零售商原文「three microphones rather than the dual array of the previous model」，Inverse 也提到「three-microphone array」。Apple 的规格页 SP783 没取到麦克风那一行。真正的分界是 2013–2017 楔形双麦和 2018 年起的三麦 | 改成「DR-VCTK 代表 2013–2017 楔形双麦 Air，不代表 2018 年起的三麦 Air（含 Intel Retina 款）」，设备键也要相应拆开 |
| 9 | 「第三方只拿到 1 声道」 | 存疑 | 唯一依据是 mictests 页面，那是浏览器录音接口报的值，浏览器可能自己混成单声道。另有反例方向：Asahi Linux 在 Linux 下直接读阵列原始信号，再用自写波束成形（Triforce，MVDR 算法）处理，支持 M1/M2 Air。也就是说，在 Linux 下拿得到多路原始麦信号 | 改成「macOS 浏览器里报 1 声道；macOS 系统层未核；Linux（Asahi）能拿到阵列原始多路」 |
| 10 | 训练集 Air 条数：第 1 声道关窗 951、开窗 1,111；第 2 声道各 951；合计 3,964；只在训练集 | 成立（我复现了） | 我重新统计：第 1 声道关窗 951、开窗 1,111，第 2 声道关窗、开窗各 951，合计 3,964。测试集只有第 3、7 声道（205/209/205/205 条）。README 原文：「Channel 3 and 7 … were used as test set」 | 保留。补一句：第 1 声道开窗多出 160 条，原因未写 |
| 11 | 客观分中位数：Air 第 1 声道关窗 PESQ 1.29、CSIG 1.54、CBAK 1.56、SNR −1.11；第 2 声道关窗 PESQ 1.31、CSIG 1.30；开窗都是 1.27；Snowball 1.21 / 2.61；iPhone 5S 1.33 | 成立（我复现了） | 我按 `train.txt` 重算，数字逐一吻合。Blue Snowball 是第 4 声道，iPhone 5S 是第 5 声道（README 的声道表）。另外：Snowball 的 CSIG 2.61 最高，PESQ 1.21 却最低，各指标排序不一致 | 保留。注明这些分数都是「整条链对比原声」，不只是麦本身 |
| 12 | 「两个声道不是同一路信号复制」，依据是 CSIG 1.54 对 1.30 | 推翻（论证无效） | 第 1、2 声道是两组不同的句子（见第 3 条），中位分差可能来自句子不同。而且开窗时两声道的 CSIG 是 1.27 和 1.28，几乎一样，正好构成反例 | 删掉这条，或改成「未核，要取同句的两声道波形才能判断」。但同一句话本来就不在两个声道里，所以这个小子集里判断不了 |
| 13 | SNR 为负，多半是 2 m 混响加对齐残差，不代表底噪 | 成立（推断合理） | README 原文：「Estimated SNR and SSNR are computed after removing the DC shift and normalizing the amplitude」。这个算法把设备频响染色也算成「噪声」，所以负值主要反映失真，不反映底噪 | 把「频响染色」也加进原因 |
| 14 | 回放布置：Bose SoundLink III，距离 2 m，中型办公室，开窗 / 关窗，48 kHz 录、16 kHz 发布 | 成立 | README 原文逐字吻合：「set 2 meters from the microphones」「All data was recorded at 48 kHz」「down-sampled … to 16 kHz」 | 保留 |
| 15 | 可能用了「音频 MIDI 设置」的聚合设备，同步录内置麦和 USB 麦 | 存疑 | 照片里屏幕上是代码编辑器加一个设备列表窗口，看不清标题。iPhone 和 iPad 各自开着录音 App 独立录，不在聚合设备里；README 用自相关去延迟也说明各设备不同步 | 改成「至少 iPhone 和 iPad 是独立录的；Mac 这边是否用聚合设备看不清」 |
| 16 | 全量原录 243 GB，要写信索取；zip 1,794,891,244 B，24,806 个条目 | 成立（字节数未复核） | README 原文：「The entire DR-VCTK dataset has 243GB … contact Junichi Yamagishi」。我数了目录表 `drvctk.tsv`，24,805 行；如果末行没有换行符，就是 24,806 条，基本吻合。字节数我没能复核，HEAD 请求被重定向了 | 保留 |
| 17 | ASVspoof 2017 v2：MacBook Pro 只当回放，录音设备 R01–R25 里没有 Mac | 成立，但有一处盲点 | 我在 PDF 里查到：回放设备有「MacBook pro internal speakers」；录音设备表里没有 Mac，有 iPhone 5c / 7 plus / 4、Logitech C920 等。但表中有一项「Desktop PC microphone input」，没写品牌 | 保留，注明那一项品牌未写 |
| 18 | EchoFake 里的 Mac 只当回放 | 成立 | arXiv HTML 原文：「two playback devices (a MacBook Pro 14-inch 2021 and an iPad Mini, 7th generation) and two recording devices (an iPhone 13 mini and a Samsung Galaxy A54)」。另外，原文限定在「training, development, and closed-set evaluation subsets」 | 注明这只是闭集部分；开放集（open-set）的设备没核 |
| 19 | LRPD 的 19 部录音设备全是手机和平板 | 成立 | 表 3 共 19 项：华为 MatePad Pro / Mate 40 Pro 等、iPhone 7 到 12 Pro Max、三星、ZTE、Sony，没有 Mac。另外原文：「The recording was carried out using standard iOS/Android audio recording API」 | 保留 |
| 20 | ReMASC：录音端是研究用麦阵，源录音器是 iPod Touch 5 和 Tascam DR-05 | 成立 | 原文：「we use a low-cost recorder, i.e., an iPod Touch (Gen5), and a professional recorder, i.e., a Tascam DR-05」。回放设备是 Sony SRSX5 / X11、AT 耳机、iPod | 保留 |
| 21 | AVspoof：录音设备是 AT2020USB+（接在笔记本上）、三星手机和 iPhone 3GS | 成立 | Idiap 页面原文：「AT2020USB+, and two mobiles, (b) Samsung Galaxy S4 … (c) iPhone 3GS」。那台笔记本是什么机型，页面没写 | 保留 |
| 22 | 3D-Speaker：论文只写「PC laptops」，「训练集 96,262 条」，「exclusively situated within 1m」 | 部分推翻 | 表 2 里写的是「PC 96262, 16.63%」，这是全库 579,013 条里的笔记本条数，不是训练集。表 2 只写「PC」；「PC laptops」出现在 §3.1 正文。「within 1 meter」原文吻合 | 改成「全库笔记本录音 96,262 条（占 16.63%）」 |
| 23 | MODALITY：笔记本麦加 B&K 测量麦，44.1 kHz，超过 30 h | 成立（许可原句未复核） | about 页原文：「audio signals obtained from a microphone array and a laptop microphone」「44.1 kSa/s with 16-bit」「over 30 hours」。「limited to research and educational purposes」这句我在 about 页没抓到 | 许可那句补上出处位置 |
| 24 | DEBATE：「primarily … smartphone microphones and built-in laptop microphones」 | 成立 | 原文：「conducted using participants' own devices, primarily consisting of smartphone microphones and built-in laptop microphones」 | 保留 |
| 25 | mictests 条目：「MacBook Air Microphone (Built-in)」、1 声道、48000 Hz、16-bit、回声消除 / 降噪 Supported，没有音频 | 成立 | 页面字段逐一吻合。但「M1 MacBook Air (2020)」这个字样我在抓回的页面文本里没找到（页面主要靠 JS 渲染） | 机型标「页面声称，未核」；样本 1 条 |
| 26 | Verge 测试、Audacity 论坛、Freesound、DNS 2021 那 31 条 | 未复核（个人说法） | 报告已标【论坛 / 博客】或「未核」，标得恰当。我没有重抓 | 保留，不作为任何数字的依据 |

## 报告漏查的重要方向

1. **Asahi Linux 的 Apple 芯片麦阵资料。** 包括 `asahi-audio` 和 Triforce（MVDR 波束成形，支持 M1/M2 Air 13"/15" 和 Pro）。这是目前唯一公开的「Apple 芯片 Mac 原始麦阵怎么处理」的一手代码。阵元数、间距、标定参数可能在仓库里，应读源码核实。出处：[Liliputing](https://liliputing.com/asahi-linux-adds-microphone-support-for-macbooks-with-apple-silicon/)、[Debian triforce-lv2](https://tracker.debian.org/media/packages/t/triforce-lv2/control-0.3.2-7)、[HN 讨论](https://svelte-brendan-jarvis.vercel.app/hackernews/43461701)
2. **2018–2020 年 Intel Retina Air（三麦）。** 这一段被「Intel 和 Apple 芯片」二分漏掉了，设备键要分成楔形双麦和三麦两段。出处：[inverse](https://inverse.com/article/50312-macbook-air-2018-price-specs-reviews-features-and-release-date)、[Apple SP783](https://support.apple.com/kb/SP783)
3. **DR-VCTK 的配套论文。** Sarfjoo、Yamagishi 等人用这份数据做「设备录音转高质量」，论文里可能写了录音软件、增益和机型。报告只读了 README。
4. **Microsoft 回声消除挑战赛（AEC Challenge）和 DNS 挑战赛的真实录音元数据。** 这些录音来自上千台真实设备，可能有设备 / 操作系统字段，可以从中筛 Mac。
5. **DAPS 等多设备重录集**（含 iPad / iPhone）。可以拿来做「同句跨设备」对照。DR-VCTK 的小子集做不到这一点（见第 3、12 条）。
6. **macOS 系统层对内置麦的处理。** 要分清两条路径：一条带系统语音处理（`AUVoiceIO`、Voice Isolation / Wide Spectrum 麦克风模式），一条是原始输入。另外，Apple 芯片机型在 Core Audio 层报几个声道也要核。这决定了「1 声道」是硬件事实还是接口选择。
7. **距离外推。** DR-VCTK 回放距离是 2 m，3D-Speaker 的笔记本在 1 m 以内，实际使用约 0.5 m。报告应明确：DR-VCTK 反推的是整条链的平均频响，混响和距离部分不能直接当成麦克风频响。

我复现用的文件：
- 本会话临时目录 `scratchpad/zipdir/train_ch_log.txt`（不入库）
- 本会话临时目录 `scratchpad/zipdir/train.txt`（不入库）

我没下载音频，也没改仓库文件。