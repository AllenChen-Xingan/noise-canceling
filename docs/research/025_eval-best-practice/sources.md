# 025 · 来源

> 和 012 `sources.md` 重复的标准，只写 012 的编号（S…）和这次新核的条款，不抄 012 已写的内容。
> 本表编号 N… 供 `matrix.md` 引用。每组更细的条款和 URL 见 `notes/group-*.md` 末尾的来源清单。
> 「核到哪级」：原文 = 读到条款正文 · 代码 = 读到参考实现的文件与行 · 镜像 = 3GPP 条文读自第三方镜像站 itecspec，未逐字对官方文件 · 摘要 · 二手。
> 下载的原文放在主目录 `_tmp/025/`（共约 15 MB），不进仓。

## 一、和 012 重复的标准（只记增量）

| 编号 | 012 编号 | 这次新核的条款 | 核到哪级 | 用在 |
|---|---|---|---|---|
| N1 | S20 | 3GPP TS 26.132：§7.10.1、§8.10.1 发送延迟（P.501 CSS、互相关包络最大值、≥ 5 次取最大）；§7.10.4.2 用真实语音时每句一个 4 s 窗、窗心对准句子；§8.4.1 发送频响（P.501 语音、1/12 倍频程）；§8.12.1 噪声下发送质量（三路信号，16 句均值与标准差）。官方 V20.0.0 只读到 §8.12 开头 | 镜像；§8.12 开头读原文 | V-LAT · V-TIMBRE · F-9 |
| N2 | S15 | ITU-T G.160 附录 II，读的是 2008 版的 Amd.1（11/2009）：§II.2 参照 = 带噪输入；表 II.1 帧分档（−1 / −10 / −16 / −25 dB）；§II.4.3 TNLR 与 `cft_lvl` −48 dBov；§II.4.4 DSN；**§II.5 表 II.2 推荐值：SNRI ≥ 4 dB、TNLR 降低 ≥ 5 dB、−4 ≤ DSN ≤ 3 dB，都是全部测试条件的平均、只作参考**（012 没读到）。现行 (06/12) 正文 PDF 打不开，推断附录 II 数值未变 | 原文（Amd.1） | V-NR · V-SPLVL · V-WEAK · F-9 |
| N3 | S18 | ITU-T P.1120 (10/25)：§11.14.5.1 背景噪声电平波动「shall not vary more than 10 dB」（012 记成 should）、35 ms、噪声先放 ≥ 5 s；§11.14.6 舒适噪声 +2 / −5 dB(A) 与表 11-17 | 原文 | V-BGSTAB · V-VACUUM |
| N23 | S19 | 3GPP TS 26.131 V19.0.0：没有 TNLR 或背景电平波动条款；§5.7.1 / §6.7.1 舒适噪声「should be injected」 | 原文 | V-VACUUM |
| N17 | S38 | DNS ICASSP 2021（arXiv 2009.06122）§2 Track 1：处理一帧的时间 < 帧移；算法延迟 ≤ 40 ms | 原文 | V-FRAME · V-LAT |
| N18 | S40 | DNS 2022（arXiv 2202.13288）/ 2023（arXiv 2303.11510）个性化赛道：开发集分段（无「用户整段不在场」一类）；dSIG / dBAK / dOVRL；总分 0.5 × [WAcc + 0.25 × (OVRL − 1)]；ANOVA 判平局；dWAcc = 增强后 − 带噪 | 原文 | D 组 · V-EAT-ASR · 听感 |
| — | S11 | P.56：本仓自写版用 16 级门限、线性插值；STL `sv-p56.c` 是 15 级、二分插值（`:221-224`、`:140`）。差多少未对拍 | 代码 | V-SPLVL |
| — | S13 | P.1401 §7.3.1 真人置信区间约 0.15、分辨率 0.3 MOS：借来给模型分定容差上限的参考，不是标准原定用途 | 沿用 012 | 听感 δ |

## 二、新增来源

| 编号 | 来源 | 现行版本 | 用到的条款 / 位置 | 能否本机跑 | 许可 | 核到哪级 | URL |
|---|---|---|---|---|---|---|---|
| N4 | Delcroix 等，Listen only to me! How well can target speech extraction handle false alarms? | Interspeech 2022；arXiv 2204.04811 | §4.1 式 (7) 对混合信号的衰减；§5.3 EER、SDRi、失败率（SDRi < 1 dB） | 公式可自写 | arXiv | 原文 | https://arxiv.org/abs/2204.04811 |
| N5 | Eskimez 等，Personalized Speech Enhancement: New Models and Comprehensive Evaluation | ICASSP 2022；arXiv 2110.09625 | §3 式 (2)、§4.3 式 (3) TSOS（p = 0.3，γ = 0.1）；§4.2 测试集；表 1 不处理时 TSOS = 0 | 公式可自写 | arXiv | 原文 | https://arxiv.org/abs/2110.09625 |
| N6 | Taherian 等，Breaking the trade-off in PSE with cross-task knowledge distillation | ICASSP 2023；arXiv 2211.02944 | §4.1 用户整段不在场测试集；§4.2 式 (7) TSOS、≥ 1 s 计次、式 (8) ΔN | 公式可自写 | arXiv | 原文 | https://arxiv.org/abs/2211.02944 |
| N7 | Thakker 等，Fast Real-time PSE: E3Net and Knowledge Distillation | Interspeech 2022；arXiv 2204.00771 | §4.1 TSOS 折成「每半小时秒数」 | — | arXiv | 原文 | https://arxiv.org/abs/2204.00771 |
| N8 | Zhang 等，Speaker Extraction with Detection of Presence and Absence of Target Speakers | Interspeech 2023 | §3.3 基于衰减量的 EER | — | ISCA | 原文（PDF 转写有缺损） | https://www.isca-archive.org/interspeech_2023/zhang23k_interspeech.pdf |
| N9 | Android OboeTester 文档（毛刺测试）与 Oboe PR #2326 | AOSP | Usage.md「Glitch Test」：锁相正弦、偏离预测即毛刺；PR 加扫频 / 多音 | 能（开源） | Apache-2.0 | 原文（文档）；PR 是否合并未核 | https://github.com/google/oboe |
| N10 | pystoi | 0.4.1 | `pystoi/stoi.py:6-115`；`pystoi/utils.py:9-191` | 能 | MIT | 代码 | https://github.com/mpariente/pystoi |
| N11 | NIST SCTK（sclite、sc_stats） | master | `doc/outputs.htm` Sum/Avg 行；`doc/st_opt.htm` 统计检验；`doc/mcn.htm` | 能（C，要编译） | 公有领域 | 原文 | https://github.com/usnistgov/SCTK |
| N12 | jiwer | master | `src/jiwer/process.py:179-246`（先累加再相除） | 能 | Apache-2.0 | 代码 | https://github.com/jitsi/jiwer |
| N13 | URGENT 2025 评测脚本与挑战论文 | main；arXiv 2505.23212 | `evaluation_metrics/calculate_wer.py:16、154-162、183-192`；论文 §3.5（CAcc，排名聚合） | 能（要下载识别模型） | Apache-2.0（未核） | 代码、原文 | https://github.com/urgent-challenge/urgent2025_challenge |
| N14 | microsoft/DNS-Challenge `DNSMOS/` | commit 591184a9（2024-03-29） | `dnsmos_local.py:19-20、33-47、49-99、107-110`；README 第 6–7 行（`-p` 个性化） | 能 | 以仓库 LICENSE 为准（未逐条核） | 代码 | https://github.com/microsoft/DNS-Challenge/tree/master/DNSMOS |
| N15 | SCOREQ（包与论文） | scoreq 1.0.1；arXiv 2410.06675 | `scoreq.py:12-21、93-98、205-216、240-245`；论文表 3 RMSE、附录 H | 能 | 代码 MIT / 权重 CC BY 4.0（017） | 代码、原文 | https://github.com/alessandroragano/scoreq |
| N16 | Distill-MOS（包与论文） | distillmos 0.9.1；arXiv 2502.05356 | `sqa.py:16-17、119-156`（含第 145 行峰值归一化）；论文表 I | 能 | MIT（017） | 代码、原文 | https://github.com/microsoft/Distill-MOS |
| N19 | Ding 等，Personal VAD | Odyssey 2020；arXiv 1908.04284 | §3.4 每类 AP 与 mAP | 能 | arXiv | 原文 | https://arxiv.org/abs/1908.04284 |
| N20 | Hagerman & Olofsson 2004 相位反转；Husstedt 等 2018（多信号）、2022（效度） | — | 两次处理拆人声 / 噪声分量；2022 年：误差集中在停顿与擦音附近 | 能（算法简单） | 期刊 | 摘要 / 二手 | https://pubs.asha.org/doi/10.1044/2022_AJA-22-00023 |
| N21 | Uemura 等 2008；Takahashi 等 APSIPA 2012（峰度比） | — | 处理后 ÷ 处理前峰度，与主观音乐噪声量相关 | 能 | 论文 | 摘要（原文 PDF 返回 503） | https://www.ieice.org/ken/paper/20080718SaF2/eng/ |
| N22 | Bisani & Ney 2004（ASR bootstrap 区间）；Liu 等 2019（按块重采样）；Gillick & Cox 1989（MAPSSWE） | ICASSP 2004；arXiv 1912.09508；ICASSP 1989 | 按句 vs 按块重采样；配对词错检验 | Bisani 工具 GPL；MAPSSWE 在 sc_stats | IEEE / arXiv | 摘要；Gillick & Cox 二手 | https://doi.org/10.1109/ICASSP.2004.1326009 |
| N24 | Godsill & Rayner；Vaseghi & Rayner 1990（AR 残差咔嗒检测）；CTU 2022 比较 | — | AR 残差阈值 | 可自写 | 论文 | 二手 / 摘要 | https://mmtg.fel.cvut.cz/wp-content/uploads/2022/07/NS_047_Ozdogru.pdf |
| N25 | X-TaSNet | arXiv 2010.12766 | 式 (4) 目标不在时的输出能量；NER | — | arXiv | 原文 | https://arxiv.org/abs/2010.12766 |
| N26 | VoiceFilter-Lite（arXiv 2009.04323）；Personalized PercepNet（arXiv 2106.04129） | Interspeech 2020 / 2021 | 只用 WER 或 P.808 与 PESQ，没有分段指标 | — | arXiv | 原文 | https://arxiv.org/abs/2009.04323 |
| N27 | Iwamoto 等 2022，增强误差对识别的影响 | arXiv 2201.06685 | 伪影是识别变差主因 | — | arXiv | 摘要 | https://arxiv.org/abs/2201.06685 |
| N28 | Hines 等 2013（VAD 字头削波对听感的影响）；VAD 评测四量 FEC / MSC / OVER / NDS | Interspeech 2013 | 字头削波 | — | ISCA | 摘要；四量定义二手（维基） | https://www.isca-archive.org/interspeech_2013/hines13_interspeech.pdf |
| N29 | Ma 等，Beyond the Stability–Plasticity Frontier in Streaming TSE | arXiv 2609.20463 v1 | 静默 30 s 后的恢复；1.6 s 混淆窗 | — | arXiv | 摘要 + 二手 | https://arxiv.org/abs/2609.20463 |

## 三、本仓内部依据

- exp014 第一张成绩单：运行 `20261104-000444-e4511abc` 的 `clips.jsonl`、`analysis.md`，以及 exp014 `findings.md` 第一节。写本文时它们在 exp014 分支（PR #97 未合并）。
- 评测器 issue 草稿：主目录 `_tmp/exp014/issue-ref-track.md`（不进仓）。
- fe3 识别重复性实测（a2 方，经大总管转来）：cafe 常见档 Air 10 段 × 3 个系统，用空缓存跑两次，80 条转写完全一致。
- 024 个性化降噪调研（`research/024-pse-landscape` 分支）：`notes/04-difficulties-data-eval.md`、`decision.md`。
- 017 评分模型调研：`docs/research/017_more-ears/decision.md`。

## 四、找过没找到

- DNS、URGENT 在干净输入上单独测「透传」的做法：没找到（不等于没有）。
- PSE / TSE 里「说话人切换后多久跟上」的现成指标：没找到。
- 「同一音频重复识别取多数」的标准做法：没找到。
- 「ΔESTOI 过线」的外部标准：没找到。
