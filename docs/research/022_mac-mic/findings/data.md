> workflow `wf_16dd3e65-ebc` · data 路调研员原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

# 第④路：Mac / 笔记本内置麦设备数据（调研代理交回，2026-11-03）

## 要点

1. **能拿来刻画 Mac 内置麦的公开数据，只有 DR-VCTK 一份。** 它有同一句话的 MacBook Air 录音和干净原声逐条对齐。许可已核清：CC BY 4.0，可以用于训练。原先说的「另附 End-user Licence」查实是存档方的 Depositor Agreement（存档协议），不约束使用者。【官方文档】
2. **DR-VCTK 里的 MacBook Air 是 Intel 老机型，不是 Apple 芯片。** 依据有两条：
   - 包内装置照片是 2010–2017 年款楔形银边的 13 寸 MacBook Air，日文键盘【推断：看图】。
   - 数据 2017 年录制，README 写「MacBookAir (ch.1)/(ch.2)」，即 2 声道。Apple 规格页写 MacBook Air (13-inch, 2017) 是「Dual microphones」【官方文档】。

   所以它**不能代表**现在的 Apple 芯片 Air（三麦阵列 + 波束成形，第三方只拿到 1 声道）。
3. **DR-VCTK 的 MacBook Air 段只在训练集里**（第 1、2 声道 × 开窗 / 关窗）。我用 HTTP 分段读取只读了 zip 的目录和配置小文件，没下音频。实测条数：第 1 声道关窗 951 条、开窗 1,111 条；第 2 声道关窗、开窗各 951 条；合计 3,964 条，16 kHz。完整 48 kHz 原录共 243 GB，要写信向作者索取。【官方文档】
4. **Apple 芯片 MacBook Air 的公开录音只有零星个人样本**，都没有干净参考：
   - DNS 2021 众包那 31 条（mtgdata 已列，机型未核）
   - Verge 2023 年的播客测试（只能听）
   - mictests.com 上的条目（只有格式字段，没有音频）

   频响、底噪拿不到公开数据，只能自己测。
5. **防伪类数据集里没有 Mac 当录音设备的**：ASVspoof 2017 v2、ReMASC、LRPD、EchoFake 都查过。ASVspoof 2017 和 EchoFake 里的 MacBook Pro 只当**回放扬声器**。【论文】
6. **Intel 老 Mac 的真人录音**有两处：
   - MOBIO 的 2008 款 MacBook（要签协议、限非商业研究，只能评测）
   - Freesound 零星 MacBook Pro 2010 等录音（多为 CC0，几乎都不是语音）
7. **其他笔记本内置麦集都没写品牌**：3D-Speaker（论文只写「PC laptops」）、MODALITY（有 B&K 测量麦作参考，但笔记本型号没写）。只能当 `laptop-unknown`。
8. **建议，待你定**：
   - 用 DR-VCTK 的 MacBook Air 段反推「Intel 双麦 Air」的设备频响，作为 `mac-intel-builtin` 的起点。
   - `mba-builtin`（Apple 芯片 Air）这一项没有公开参考数据，必须本机自测。这需要放声和录音，本次按限制没做。

## 逐条表

### A. 设备响应集 / 多设备同步重录

| 名称 | 结论 | 原文或数字 | 出处 | 来源标记 | 样本量 |
|---|---|---|---|---|---|
| DR-VCTK（小子集） | 许可：训练可 | 条目元数据 `dc.rights`：「Creative Commons Attribution 4.0 International Public License」。README 原句：「You are free to use this database under Creative Commons Attribution License (CC-BY)」 | https://datashare.ed.ac.uk/server/api/pid/find?id=10283/3038 ；https://datashare.ed.ac.uk/bitstream/handle/10283/3038/README.txt | 【官方文档】 | — |
| DR-VCTK | 「End-user Licence」查实不是使用许可 | 条目里有两份许可文件：`license.txt`（4,540 B，原文开头「Depositor Agreement: Edinburgh DataShare」，讲存档方授权）和 `license_text`（CC-LICENSE 包，CC BY 4.0 全文） | DataShare 接口 `/server/api/core/bitstreams/c44d6902-…`、`c57a4752-…` | 【官方文档】 | — |
| DR-VCTK | 设备与布置 | 8 个声道：「Channel 1 MacBookAir (ch.1) / Channel 2 MacBookAir (ch.2) / Apogee MiC / Blue Snowball / iPhone 5S ×2 / iPad ×2」。原文还写：「Bose 404600 SoundLink speaker III … 2 meters from the microphones」「medium-sized office … windows either opened or closed」「All data was recorded at 48 kHz」，发布版降到 16 kHz | README.txt | 【官方文档】 | 1 台 MacBook Air，1 个房间 |
| DR-VCTK | 机型是 Intel 13 寸 MacBook Air（约 2013–2017 年款） | 包内 `configurations/IMG_0104.jpg` 是楔形银边的 13 寸 Air，日文键盘。2017 款规格页写「Dual microphones」，和数据里的 2 声道一致 | zip 内照片（我用分段读取取出，约 1.9 MB）；https://support.apple.com/kb/SP753 | 【推断】看图；规格为【官方文档】 | 1 台 |
| DR-VCTK | 录音时可能用了「音频 MIDI 设置」里的聚合设备，把内置麦和 USB 麦拼在一起录 | 照片里屏幕上开着一个设备列表窗口 | 同上 | 【推断】 | — |
| DR-VCTK | MacBook Air 段只在训练集 | `test_ch_log.txt` 里只有第 3 声道（Apogee）和第 7 声道（iPad）。`train_ch_log.txt` 的 MacBook Air 条数：第 1 声道关窗 951 / 开窗 1,111，第 2 声道关窗、开窗各 951 | zip 内 `configurations/*.txt`，我取出后统计 | 【官方文档】数据；统计为我实测 | 3,964 条 |
| DR-VCTK | 自带客观分（中位数，以原 VCTK 为参考） | MacBook Air 第 1 声道关窗：PESQ 1.29、CSIG 1.54、CBAK 1.56、SNR −1.11 dB；第 2 声道关窗：PESQ 1.31、CSIG 1.30；开窗两声道 PESQ 都是 1.27。对照：Blue Snowball 关窗 PESQ 1.21、CSIG 2.61；iPhone 5S 关窗 PESQ 1.33 | zip 内 `objective-measures/train.txt`，我按声道汇总 | 数字为【官方文档】；汇总为我实测 | 每格 951–1,111 条 |
| DR-VCTK | 两个声道不是同一路信号复制 | 第 1 声道 CSIG 1.54，第 2 声道 1.30（关窗） | 同上 | 【推断】 | — |
| DR-VCTK | SNR 为负，多半是 2 m 混响加对齐残差造成的，不代表真实底噪 | 原文「auto-correlation for removing the delay」 | README | 【推断】 | — |
| DR-VCTK | 大小 | `DR-VCTK.zip` 1,794,891,244 B，24,806 个条目。全量原录：「The entire DR-VCTK dataset has 243GB … contact Junichi Yamagishi」 | 接口与 README | 【官方文档】 | — |
| DR-VCTK | 能干什么 | 和 VCTK 原声逐条对齐，可以反推「Intel 双麦 Air + 2 m + 办公室」整条链的平均频响；能做侵入式评测锚。**不能**代表 Apple 芯片 Air | — | 【推断】 | — |
| Libri-Adapt | 没有 Mac 内置麦 | MacBook Pro 只是外接 Shure MV5 的主机（mtgdata 已核） | mtgdata `cards/libri-adapt.md` | 【官方文档】 | — |
| MobiPhone | 没有 Mac | 21 部手机、7 个品牌（有 iPhone 5），录 TIMIT，安静环境 | arXiv 2411.03668 表 III；arXiv 2410.06221 | 【论文】（完整设备表未读到） | — |
| DiPCo | 没有笔记本 | 5 台 7 麦圆阵加近讲 | 014 卡 `dipco.md` | 【论文】 | — |
| VOiCES | 没有笔记本 | SM58 / AKG 417L / SM11 | 014 卡 `voices-corpus.md` | 【论文】 | — |
| ReMASC | 没有 Mac | 录音端是 2、4、6、7 麦的研究阵列（含 Amlogic A113X1）；源录音器是 iPod Touch 5 和 Tascam DR-05 | arXiv 1904.03365 §3.3–3.4 | 【论文】 | — |
| ASVspoof 2017 v2 | Mac 只当回放，不当录音 | 回放设备「P18 MacBook pro internal speakers」；录音设备 R01–R25 里没有 Mac（有 iPhone 4、5c、7 plus 和 Logitech C920 等） | https://www.eurecom.fr/publication/5504/download/sec-publi-5504.pdf 表 2、表 4 | 【论文】 | — |
| LRPD | 没有 Mac | 19 部录音设备全是手机和平板（华为、iPhone 7–12、三星等） | arXiv 2309.17298 表 3 | 【论文】 | — |
| EchoFake | Mac 只当回放 | 原文「two playback devices (a MacBook Pro 14-inch 2021 and an iPad Mini…) and two recording devices (an iPhone 13 mini…)」 | arXiv 2510.19414 | 【论文】 | — |
| AVspoof / voicePA | 没有 Mac 内置麦 | 录音设备是「AT2020USB+」（接在笔记本上）、三星手机和 iPhone 3GS | https://www.idiap.ch/software/beat/platform/databases/avspoof/3/ | 【官方文档】 | — |
| DDS | 没有笔记本 | iPad Air 加两只电容麦 | mtgdata `cards/dds-device-degraded-speech.md` | 【官方文档】 | — |

### B. 含 MacBook 内置麦的公开集或样本（按机型分）

| 机型 | 名称 | 结论 · 原文 | 许可 | 有无干净参考 | 用途 | 出处 · 标记 · 样本量 |
|---|---|---|---|---|---|---|
| **MacBook Air · Intel**（约 2013–2017） | DR-VCTK | 见 A 表 | 训练可（CC BY 4.0） | 有，逐条对齐 | 反推频响；评测锚 | 【官方文档】；1 台；3,964 条 |
| **MacBook Air · Apple 芯片** | DNS 2021 众包 | 文件名带 mac / macbook 的 31 条，未核机型，可能含 Pro 和 Intel | 未查到（DNS README：按原条款） | 无 | 人耳旁证 | mtgdata 转交【实测：读 zip 目录】；4 位众包者 |
| MacBook Air · Apple 芯片 | mictests.com 条目 | 「MacBook Air Microphone (Built-in)」「M1 MacBook Air (2020)」，1 声道，48000 Hz，16-bit；「Echo Cancellation: Supported / Noise Suppression: Supported」是浏览器录音接口报的能力，不是硬件处理 | 页面无许可 | 无，也没有音频 | 只能旁证「1 声道 48 kHz」 | https://mictests.com/reviews/39467 【论坛 / 博客，个人说法】；1 台；能力解读为【推断】 |
| MacBook Air M2 / MacBook Pro 16（2021） | Verge 2023 年时代广场笔记本麦测试 | 6 台笔记本走过时代广场录音。转载摘要说 MacBook Pro 2021「clear … above the ambience」；M2 Air 的结果没读到原文 | 版权内容，只能听 | 无 | 主观对比旁证 | 播客 2023-04-05（https://metacast.app/podcast/the-vergecast/pcHTvkju/how-to-use-microsoft-s-ai-copilot-laptop-microphones-and-amazon-sidewalk/jolavMXI ）；Verge 原文没读到，转载站返回 403【论坛 / 博客，个人说法】；1 人 |
| MacBook Pro · Apple 芯片（M1） | Audacity 论坛对比 | 内置麦对比 Boya BY MM1，附 .m4a 两段，用 QuickTime 自动电平录。原话「I assumed the MacBook mic would sound horrible but it sounded surprisingly good」 | 未写 | 无（对比麦也不是参考级） | 只够人耳旁证 | https://forum.audacityteam.org/t/m1-macbook-pro-mic-vs-boya-by-mm1-mic/63696 【论坛 / 博客，个人说法】；1 台、2 人 |
| **Intel MacBook**（2008 款） | MOBIO | 原文「a standard 2008 MacBook」「only used to capture part of the first session」 | 只评测（签协议，限非商业研究） | 无 | 历史 Intel 机真人录音 | mtgdata `cards/mobio.md` 【官方文档】；量未查到 |
| Intel MacBook Pro（2010 款等） | Freesound | 搜「macbook built-in microphone」共 18 条（16 条 CC0、2 条 CC BY）。例如 kim.headlee「built-in macbook pro 2010 microphone」（小孩笑声 44 s）；多为非语音 | 多为 CC0 | 无 | 基本没用；最多测底噪 | https://freesound.org/search/?q=macbook+built-in+microphone 【论坛 / 博客，个人说法】；各 1 台 |
| 品牌未写 | 3D-Speaker | 原文只写「PC laptops」，训练集 96,262 条，「exclusively situated within 1m」 | 未查到（mtgdata 已核） | 近讲麦可作近似参考 | 笔记本麦评测 | arXiv 2306.15354 §3 【论文】；型号未查到 |
| 品牌未写 | MODALITY（格但斯克理工大学） | 原文「a microphone located in a laptop placed in front of the speaker, at the lap level」；同时录 8 只 B&K 测量麦（50 / 100 / 150 cm），44.1 kHz，有交通、babble、工厂噪声回放 | 原文「limited to research and educational purposes only」，即只评测 | 有（B&K 50 cm，非逐样本干净） | 「笔记本麦 vs 测量麦」频响旁证，但不是 Mac | https://rd.springer.com/content/pdf/10.1007/s10844-016-0438-z.pdf §3.2；https://knot927.eti.pg.gda.pl/MODALITY/about.php 【论文】【官方文档】；超过 30 h |

### C. 他人公开的「Mac 内置麦 vs 参考麦」频响测量

| 结论 | 原文或数字 | 出处 | 来源标记 | 样本量 |
|---|---|---|---|---|
| 没有查到任何 MacBook 内置麦的公开频响曲线 | Faber Acoustical 测过 iPhone 16/17 Pro 和 Vision Pro（用 PCB 378B02 参考麦），**没测过 Mac** | https://www.faberacoustical.com/blog/wp-json/wp/v2/posts/2286 等 | 【论坛 / 博客】 | 0 |
| 测量圈普遍认为内置麦不能用来做测量，没有给出曲线 | AVNirvana 帖 | https://www.avnirvana.com/threads/can-macbook-pro-integrated-mic-be-used-to-measure.6551 | 【论坛 / 博客，个人说法】 | 少数几人 |
| GitHub / Hugging Face 上没有 Mac 麦脉冲响应 | 只找到 MacBook 机身敲击数据（AGPL-3.0，不是脉冲响应）和 Surrey 的 25 只录音棚麦脉冲响应 | https://huggingface.co/datasets/CodeWithWinton/macbook-unibody-acoustic-tap-dataset ；https://zenodo.org/records/4633508 | 【官方文档】 | — |

## 转交 mtgdata

- **MODALITY**：有笔记本内置麦加 B&K 参考麦，有 babble、交通、工厂噪声回放，英语词表朗读，只评测，笔记本品牌未写。设备键建议 `laptop-unknown`。
- **DEBATE**（arXiv 2506.07502，普通话）：录音用参与者自己的设备，原文「primarily … smartphone microphones and built-in laptop microphones」。品牌和许可未查。
- **NISQA_TEST_LIVETALK**：原文「recorded on a Laptop for the VoIP calls」，CC BY 4.0，笔记本型号未写（mtgdata 已有卡，这里只补许可原句）。
- **Mixer 6**：14 只麦里有没有笔记本麦，我没读到采集文档（链接返回 HTML），未查到。

## 没查到 / 仍不确定

1. **Apple 芯片 MacBook Air（M1–M4）**：没有任何带干净参考的公开录音，也没有频响或底噪数字。【未查到】
2. DR-VCTK 里 MacBook Air 的确切年份：只能从外观和 2 声道推断是 2013–2017 年款 13 寸 Intel Air，录音软件和采集路径没写。**DR-VCTK 2 m 回放的结果**能不能外推到 Air 通常 0.5 m 左右的使用距离，也未核。【推断】
3. DNS 2021 那 31 条：是不是内置麦、什么机型，未核。
4. 3D-Speaker 测试集里笔记本条数和品牌：未查到（标签只读论文，没读元数据；测试包 1.1 GB，超下载上限）。
5. MOBIO 笔记本段的时长：未查到。
6. ReMASC、LRPD、ASVspoof 2017 的许可：没核，因为它们不含 Mac，用不上。
7. Verge 测试里 M2 Air 的结论原文：没读到。
8. Freesound 那 18 条的完整描述：只看了搜索页摘要，没逐条打开。

临时文件（DR-VCTK 目录、配置、客观分、照片和两个只读脚本）都在 本会话临时目录 `scratchpad/zipdir/`（不入库）。没下载任何音频，没改仓库里任何文件。