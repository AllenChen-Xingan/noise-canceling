# 线 C · 「压缩过的众人耳朵」：众包 MOS 训练的无参考预测器，在增强语音上与人有多一致

> 调研日期 2026-10-31 · 只读调研，未下载任何权重或数据，未上传任何音频。
> 证据等级：【官方明载】仓库 README / 官方 benchmark · 【论文】arXiv / ISCA 原文 · 【源码可证】本地源码 · 【本机实测】本机跑过 · 【传言/未核】二手或没核到原文。
> 本地仓库前缀 `resources/` = `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/`。
> 不重复：许可与 SCOREQ 权重的细节见 `docs/research/004_eval-toolchain/sources.md:26-34`，选型初稿见 `docs/research/004_eval-toolchain/decision.md:21,113`，V-NRMOS 的登记见 `docs/research/012_eval-standards/verifiers.md:95`。本文只补「与人一致」的数字和「怎么接进来」。

---

## 0. 一句话结论

**在「增强后语音 + 众包 MOS」上，最稳的是 SCOREQ（natural, NR 模式）**：逐句 SRCC 0.79（URGENT 2024）和 0.75（CHiME-7 UDASE 真实录音），系统级 SRCC 0.93 和 0.90，同参考配对准确率 acc0 0.68 和 0.75。它是唯一在「模拟 + 真实」两边都站得住的现成模型，pip 加 ONNX 在 CPU 上就能跑，权重约 378 MB（低于 500 MB 线）。
**第二选择是 Distill-MOS**：轮子 15.6 MB、MIT、4.3 M 参数，逐句 0.75 和 0.71，配对 acc0 0.66 和 0.67，最便宜、马上能用。
DNSMOS OVRL 在增强语音上排序能力最弱：URGENT 2024 系统级 KRCC 0.42–0.46，CHiME-7 系统级 SRCC −0.10。我们现在只接了它，这本身就能解释一部分「机器分与作者一人只有 0.48 一致」。

**最关键的对照**：就算是「8 个众包人的平均 MOS」，最好的预测器做同参考配对（同一句话、不同系统）也只对 **0.68–0.75**，DNSMOS 只对 **0.51–0.63**（UrgentMOS 论文，见 §2.4）。我们 exp003 的配对一致率（DNSMOS OVRL 0.48，95% 区间 [0.26, 0.70]，`docs/experiments/exp007-eval-suite/calib/20261031-exp003-round1.md:9`）**落在「众人耳 vs DNSMOS」的区间内或略低**。光凭 48 对，分不清是作者偏好特殊，还是 DNSMOS 本来就这么差。

---

## 1. 读数之前必须分清的三件事

1. **语句级和系统级差一个数量级**：同一个模型，系统级 SRCC 可以到 0.9，逐句只有 0.6–0.8。原因是人耳本身逐句噪声很大。【论文】DNSMOS 原文 §4.1 写明：同一批 clip，两次 P.808 跑出来的逐句 PCC「only about 0.5」，按组平均后 >0.9（arXiv 2010.15258 §4.1）。DNSMOS P.835 原文写 P.835 两次跑的逐句 PCC 只有「0.7-0.8」（arXiv 2110.01763 §4.1）。**这就是「众人耳」逐句的天花板**，一个人的耳朵只会更低。
2. **我们的判据是「配对一致率」**（同屏谁更好），最接近的公开口径是 **UrgentMOS 的 acc0 / acc0.5**：同参考内容、不同系统配成对（arXiv 2601.18438 §3.2.2、§4.1.2）。它**不是**系统级相关，也不是逐句相关。拿系统级 0.9 和我们的 0.48 比，是错配。
3. **训练集泄漏**：NISQA 在 NISQA_TEST_* 上、Uni-VERSA-Ext 5metric 在 urgent2024-sqa 上、URGENT-PK 在 urgent24 上，都是「同源」或「同分布」，分数偏高。真正的「换分布」检验看 **CHiME-7 UDASE**（真实录音，5 个系统）。

---

## 2. 公开数字（只收增强语音或带噪通话语音上的）

### 2.1 URGENT 2024 盲测 MOS（P.808，每样本 8 人，300 句 × 23 个条件）

数据来源：【论文】arXiv 2506.01611 §3.1：「P.808 tests on a 300-sample subset」，「Each sample was evaluated by eight different listeners」；21 支队伍加基线加含噪输入。

**(a) 分析仓库图**（`resources/urgent2024_analysis/README.md:150-163`，图 `resources/urgent2024_analysis/mos/mos_correlations.png`）【官方明载】。论文正文**没写**是系统级还是逐句。数值与 Track2 BENCHMARK 的**系统级**列几乎一致：SCOREQ 图上 LCC 0.91，sys 0.92；UTMOS 0.94 对 0.95；DNSMOS 0.83 对 0.85；DNSMOS SRCC 0.57 对 0.56。所以按系统级读（这是推断，原文未载）。

| 指标 | KRCC | SRCC | LCC | 备注 |
|---|---|---|---|---|
| DNSMOS（OVRL） | 0.46 | 0.57 | 0.83 | 论文：「LSD and DNSMOS lag behind others on KRCC」（§3.2） |
| NISQA | 0.64 | 0.71 | 0.87 | |
| DNSMOS Pro | 0.57 | 0.71 | 0.88 | 挑战外指标 |
| UTMOS（= UTMOS22 strong，见 `resources/urgent2024_analysis/README.md:133` `--utmos_tag utmos22_strong`） | **0.75** | **0.91** | **0.94** | 论文：「UTMOS and SCOREQ demonstrate the highest correlations」（§3.2） |
| WV-MOS | 0.54 | 0.74 | 0.87 | |
| SCOREQ | **0.75** | **0.91** | 0.91 | |
| VQScore | 0.49 | 0.65 | 0.87 | |
| WADA-SNR | 0.24 | 0.35 | 0.11 | 最差 |
| 参照：挑战综合排名分 | 0.73 | 0.90 | 0.76 | 多指标综合，胜过所有单项官方指标 |

**(b) URGENT 2026 Track 2 官方 BENCHMARK**（`resources/urgent2026_challenge_track2/BENCHMARK.md:1-10`，测试集 = urgent2024-sqa blind_test_mos，6900 句、23 个系统，见同仓 `README.md:235-240`）【官方明载】

| 模型 | utt LCC | utt SRCC | utt KTAU | sys LCC | sys SRCC | sys KTAU |
|---|---|---|---|---|---|---|
| Uni-VERSA-Ext 5metric | 0.78 | 0.78 | 0.59 | 0.94 | 0.96 | 0.86 |
| Uni-VERSA-Ext MOS-only | 0.79 | 0.78 | 0.59 | 0.95 | 0.94 | 0.78 |
| **SCOREQ** | **0.79** | **0.79** | **0.60** | 0.92 | 0.93 | 0.77 |
| Distill-MOS | 0.76 | 0.75 | 0.57 | 0.91 | 0.75 | 0.63 |
| NISQA_MOS | 0.71 | 0.70 | 0.52 | 0.89 | 0.70 | 0.62 |
| DNSMOS_OVRL | 0.65 | 0.64 | 0.47 | 0.85 | 0.56 | 0.42 |
| UTMOS | 0.72 | 0.74 | 0.55 | 0.95 | 0.90 | 0.75 |

⚠️ 5metric 版用 urgent2024-sqa 训练（README:271-276 说 MOS-only 版要排除它），同分布，偏乐观。

### 2.2 CHiME-7 UDASE 评测集（真实录音，5 个系统，640 句）——换分布检验

`resources/urgent2026_challenge_track2/BENCHMARK.md:12-21`【官方明载】

| 模型 | utt LCC | utt SRCC | sys SRCC | sys KTAU |
|---|---|---|---|---|
| Uni-VERSA-Ext 5metric / MOS-only | 0.77 | 0.76 | 0.90 | 0.80 |
| **SCOREQ** | 0.75 | **0.75** | **0.90** | **0.80** |
| Distill-MOS | 0.72 | 0.71 | 0.70 | 0.60 |
| NISQA_MOS | 0.02 ⚠️ | 0.03 ⚠️ | 0.13 | 0.08 |
| DNSMOS_OVRL | 0.28 | 0.28 | **−0.10** | 0.00 |
| UTMOS | 0.26 | 0.33 | 0.20 | 0.20 |

⚠️ **BENCHMARK 的 NISQA 行疑似抄错**：CHiME-7 那行（`BENCHMARK.md:19`）与 SOMOS 那行（`BENCHMARK.md:52`）逐格相同（1.38 / 0.02 / 0.03 / 0.02 / 1.10 / 0.12 / 0.13 / 0.08）。UrgentMOS 论文给的 NISQA-MOS 在 CHiME-7 上是逐句 LCC/SRCC **0.48 / 0.47**（arXiv 2601.18438 Table 5/10）【论文】。以论文为准，BENCHMARK 这一格标「不可信」。
系统只有 5 个，系统级 KRCC/SRCC 分辨率很粗（URGENT-PK 论文 Table II 脚注也这么说）。

### 2.3 URGENT 2025 SQA（逐句）与「同参考配对准确率」——最贴近我们判据的口径

【论文】UrgentMOS（arXiv 2601.18438，Table 5/10）。acc0 = 严格配对（MOS 不等即有胜负），acc0.5 = |ΔMOS| ≤ 0.5 当平局、要求预测出平局；配对是「同一参考内容、不同系统」（§3.2.2、§4.1.2、§5.1）。

| 模型 | 逐句 LCC/SRCC：U24 · CHiME-7 · U25 | **acc0**：U24 · CHiME-7 · U25 | acc0.5：U24 · CHiME-7 · U25 |
|---|---|---|---|
| DNSMOS | .65/.64 · .28/.28 · .65/.65 | **.63 · .51 · .63** | .54 · .37 · .52 |
| UTMOS | .72/.74 · .26/.33 · .65/.69 | .67 · .57 · .66 | .55 · .49 · .50 |
| NISQA-MOS | .71/.70 · .48/.47 · .68/.66 | .67 · .59 · .65 | .55 · .51 · .52 |
| Distill-MOS | .76/.75 · .72/.71 · .71/.70 | .66 · .67 · .67 | .54 · .55 · .55 |
| **SCOREQ** | .79/.79 · .75/.75 · .74/.74 | **.68 · .75 · .69** | .58 · .60 · .57 |
| UrgentMOS F4C1M5 | .80/.79 · .77/.76 · .75/.74 | .69 · .80 · .74 | .59 · .60 · .59 |

读法：**最好的现成模型对「8 人平均」的配对也只有约七成**。DNSMOS 在真实录音上 acc0 = 0.51，接近抛硬币。UrgentMOS 的权重与许可：论文正文**未查到**发布链接。

### 2.4 URGENT 2025 主赛道 / 2026 报告

- URGENT 2025 多语 P.808 论文（arXiv 2507.11306 §3.2）：每句 8 人评分、每语种 150 句，**没有算** DNSMOS/NISQA 与 MOS 的相关，只定性说无参考指标会漏掉「幻觉」【论文】。
- ICASSP 2026 URGENT 总报告（arXiv 2601.13531 §3、§6）：Track 2 测试集 = URGENT 2025 盲测里 16 个系统、8000 句带 MOS。Track 2 收到「six competitive submissions」，场外的 URGENT-PK 系统级相关最高。**具体数字在 HTML 版未查到**（链接到外部榜单）【论文】。
- URGENT-PK（arXiv 2506.23874 Table III，urgent25_en，22 个系统，**系统级**）【论文】：DNSMOS KRCC/SRCC/LCC 0.602/0.713/0.847；UTMOS 0.835/0.951/0.894；UG-PK_mel（NBS）0.853/0.960/0.935；UG-PK_UTMOS(ft) 0.879/0.972/0.959。换语种和换分布（Table II）：CHiME-7 UDASE 上 DNSMOS 0.000/−0.100/−0.140，UTMOS 0.200/0.200/0.267，UG-PK_mel 0.200/0.300/0.289，UG-PK_UTMOS(fix/ft) 0.400/0.500。**PK 在真实录音上也没赢 SCOREQ 的 0.80/0.90**（口径不同：PK 只给系统级）。论文的逐句 A/B 主观测试平均准确率：MOS_cp 0.89，UTMOS 类 0.86（Table IV）。

### 2.5 各模型自己论文里的数字（多为通话/带噪语料，非「增强后」）

| 模型 | 数字 | 级别 · 系数 · 数据集 | 出处 |
|---|---|---|---|
| DNSMOS P.808 | M0：PCC 0.93 / SRCC 0.94 | **系统级**（每个降噪模型平均），Interspeech 2020 DNS 盲测 | arXiv 2010.15258 Table 2、§4.1【论文】 |
| 同上 | M1 总体 SRCC 0.90（M0 只有 0.65）；歌唱 0.61，情绪 0.58 | 系统级，ICASSP 2021 DNS 18 个降噪器 | 同上 Table 3【论文】 |
| DNSMOS P.835 | 模型级 PCC SIG/BAK/OVRL 0.94/0.98/0.98，SRCC 0.95/0.99/0.98；**逐句 PCC 0.71/0.83/0.82，SRCC 0.72/0.82/0.81** | 17 个微软内部降噪模型、850 句；逐句用 30 人/句 | arXiv 2110.01763 Table 3、§5.1–5.3【论文】 |
| NISQA v2（总分） | 按条件 PCC：TEST_P501 0.95 · FOR 0.95 · LIVETALK 0.90 · VAL_LIVE 0.82 | **按条件（≈系统级）**，一阶映射后，NISQA 语料（同源） | arXiv 2104.09494 Table 4【论文】 |
| NISQA 分项 | Noisiness / Coloration / Discontinuity / Loudness 在 LIVETALK：0.76 / 0.87 / 0.83 / 0.71；VAL_LIVE：0.73 / 0.57 / 0.55 / 0.73 | 按条件 PCC | 同上 Table 5【论文】 |
| SCOREQ（NR, Adapt） | PCC：NOIZEUS（增强语音）0.91 · TCD-VoIP 0.85 · Tencent 0.86 · P501 0.96；NOIZEUS SRCC 0.90 | **逐文件** | arXiv 2410.06675 Table 2/3/9【论文】；论文 Table 12 的 NOIZEUS 区间与正文差值对不上，抓取摘要也提到这一点（未逐字核） |
| Distill-MOS | PCC：DNS3 0.79 · DNS3 FB 0.80 · LIVETALK 0.86 · P501 0.90 · Tencent 0.95 · PSTN 0.58 · 加权均值 0.76（教师 0.81） | 逐句（论文未明说） | arXiv 2502.05356 Table I【论文】 |
| Uni-VERSA（WavLM） | MOS LCC/SRCC 0.77/0.78（无参考）；只预测 MOS 时 0.80/0.80 | **逐句**，URGENT24 提交作品 | arXiv 2505.20741 Table 3/4【论文】 |
| SQUIM 主观分（= NORESQA-MOS，NMR） | Tencent_woR LCC 0.50 · IUB_voices 0.70；SCOREQ 论文里 NORESQA-M 在 NOIZEUS 只有 0.15 | 逐句 | VQScore arXiv 2402.16321 Table 2；SCOREQ Table 2；SQUIM 论文 §2 说明 MOS 模型即 NORESQA-MOS【论文】 |
| SQUIM 客观分（STOI/PESQ/SI-SDR 估计） | 与**真实 PESQ** PCC 0.958（不是与人耳） | 逐句，DNS 2020 自造集 | arXiv 2304.01448 Table 2【论文】 |
| VQScore | LCC：Tencent_woR 0.72 · IUB_voices 0.86（同表 DNSMOS P.835 0.78 / 0.82） | 逐句 | arXiv 2402.16321 Table 2【论文】 |
| SpeechLMScore | LCC：Tencent_woR 0.71 · IUB_voices 0.69 | 逐句 | 同上（VQScore 论文转引）【论文】 |
| PAM | SRCC 0.88（PAMavg 0.93），同表 DNSMOS 0.975 | **系统级**，ICASSP 2021 DNS 盲测 20+ 模型 | arXiv 2402.00282 §5.4 Table 8【论文】 |
| DNSMOS Pro | 自己论文的数字**未查到**（ISCA PDF 抓取失败）；URGENT 2024 图上 KRCC 0.57 / SRCC 0.71 | 系统级（推断） | §2.1 |
| UTMOSv2 | 增强语音上的数字**未查到**。分析仓库脚本会算 UTMOSv2（`resources/urgent2024_analysis/mos/calculate_nonintrusive_mos.py:15,124-126`），但图里只放了 UTMOS22 | — | 【源码可证】 |
| WV-MOS | 只在 URGENT 2024 图上：KRCC 0.54 / SRCC 0.74 / LCC 0.87 | 系统级（推断） | §2.1 |

---

## 3. 本机能不能跑 · 大小 · 许可 · 本地有没有

本地查询：`ls resources/`、`ls ~/vdn-data/models`（只有 fastenhancer / voxlock / wespeaker，**没有任何 MOS 预测器权重**）、`~/.cache/huggingface/hub`（没有相关模型）。大小用 HTTP HEAD 查，没有下载。

| 模型 | 本地已有？ | 权重大小 | 许可 | Mac M 系列 CPU | 能否立刻用 |
|---|---|---|---|---|---|
| DNSMOS P.808 / P.835 | **有**：`resources/DNS-Challenge/DNSMOS/DNSMOS/{model_v8,sig_bak_ovr,sig,bak_ovr}.onnx`、`pDNSMOS/sig_bak_ovr.onnx`（0.2–1.2 MB） | ~1 MB | 代码 MIT / 内容 CC BY 4.0（见 004 sources:26） | 能，exp007 已接【本机实测】 | 已在用（V-DNSMOS） |
| NISQA v2（含 4 个分项） | **有**：`resources/NISQA/weights/nisqa.tar` 1.05 MB | 1 MB | 代码 MIT，权重 **CC BY-NC-SA 4.0**（004 sources:27） | 能（PyTorch CPU，小 CNN） | 能，**尚未接** |
| SCOREQ natural NR | 否（代码未克隆，pip 包 `scoreq` 1.0.1，MIT，wheel 8.6 KB，首次运行下载 ONNX） | ONNX ~378 MB（004 sources:31） | 代码 MIT / 权重 CC BY 4.0（004 sources:31） | 能，默认就是 ONNX Runtime CPU（官方 README） | **能**，下载 < 500 MB |
| Distill-MOS | 否 | **wheel 15.6 MB**（PyPI distillmos 0.9.1，权重随包）；4.3 M 参数 | MIT（PyPI 元数据） | 应可以（4.3 M 参数，16 kHz）。官方没写 CPU，**待测** | **能** |
| UTMOS22 strong（SpeechMOS） | 代码有：`resources/SpeechMOS/hubconf.py:11`、`resources/UTMOS22/`；**权重没有** | 411 MB（GitHub release，HEAD 实查） | MIT（`resources/SpeechMOS/hubconf.py:14`） | 能（torch.hub，`map_location="cpu"`，`hubconf.py:31`） | 能，但真实录音上差（CHiME-7 SRCC 0.33） |
| UTMOSv2 | 代码有：`resources/UTMOSv2/`（MIT）；权重没有 | **单折 819 MB**（HF Space `fold0_s42_best_model.pth`，HEAD 实查），另需 wav2vec2-base | MIT | 理论上能 | **超 500 MB，不下**，只记大小 |
| Uni-VERSA-Ext（URGENT 2026 T2 基线） | 代码有：`resources/urgent2026_challenge_track2/`；权重没有 | `universa-ext_wavlm-base_5metric/model.pt` **512 MB**（HEAD 实查），另加 WavLM-base | 代码 Apache（仓库无 LICENSE 文件，**未核**）；权重许可在 HF 页**未查到** | 能（PyTorch） | **超 500 MB，不下**。另外 README:33 提醒 HyperPyYAML 配置等同于代码 |
| URGENT-PK | 代码有：`resources/URGENT-PK/`（Apache，`LICENSE.txt`） | mel_resnet34 **21.6 MB**；utmos_ft 427 MB（HEAD 实查） | 代码 Apache；权重许可**未查到** | 能（ResNet34，PyTorch） | mel 版**能**，做系统级排名；只给成对 / 系统级，不给单句分 |
| DNSMOS Pro | 否（`resources/urgent2024_analysis/mos/README.md:46` 指向 github.com/fcumlin/DNSMOSPro，权重随仓库 `runs/NISQA/model_best.pt`） | 大小未查到（论文称「reduced-size」） | MIT（GitHub 页） | 能 | 能，但排序不如 SCOREQ |
| WV-MOS | 否 | 未查到 | 未查到 | 能 | 意义不大 |
| VQScore | 否 | 未查到（代码与权重在 github.com/JasonSWFu/VQscore，`mos/README.md:240-266`） | 未查到 | 能 | 排序弱（KRCC 0.49） |
| SQUIM（torchaudio） | 否 | 未查 | OBJECTIVE CC BY 4.0、**SUBJECTIVE CC BY-NC 4.0**；torchaudio 2.9 进入维护期（004 sources:34） | 能 | 主观分 = NORESQA-MOS，弱（NOIZEUS 0.15）；客观分是在估 PESQ，不是人耳 |
| SpeechLMScore / PAM | 否 | 未查 | 未查 | 能 | 没有增强语音上的逐句或配对数字，不优先 |
| VERSA 工具箱 | **有**：`resources/versa/`（Apache）。`docs/supported_metrics.md:10-62` 已封装 nisqa、squim、utmosv2、scoreq、universa、dnsmos_pro、wvmos、vqscore | — | Apache | 能 | 可当统一调用壳，但依赖重，按需 |

**众包 MOS 数据（用来直接检验「作者 ≠ 大众？」）**
- `urgent-challenge/urgent2024_mos`（HF，CC BY-NC-SA 4.0，未设门槛）：3 个 parquet 分片，分别 442 / 471 / 430 MB，合计 **1.34 GB**（HF API 实查）→ **超 500 MB，不下**。单个分片 < 500 MB，只取一片要不要算作违反「> 500 MB 不做」，**列为待拍板**。
- CHiME-7 UDASE eval（Zenodo 10418311，CC BY-SA 4.0）：**3.49 GB** → 不下（卡片见 `docs/research/014_real-datasets/cards/chime7-udase.md`）。

---

## 4. 怎么把「压缩过的众人耳」做成 verifier（全部本机，不上传）

**V-CROWD-PRED（建议名）**：用 SCOREQ-NR 和 Distill-MOS 打 exp003 的同一批音频，按 exp007 的口径（48 对、按屏聚类 bootstrap）算与作者的配对一致率，对照「众人耳基线」：

| 预测器 | 与众人耳（公开，acc0） | 与作者（待测） | 解读 |
|---|---|---|---|
| DNSMOS OVRL | 0.51–0.63（§2.3） | 0.48 [0.26, 0.70]【本机实测，calib round1:9】 | 与众人基线一致 → **不能**说作者特殊 |
| SCOREQ | 0.68–0.75 | 待测 | 若与作者明显 < 0.6 且区间上界 < 0.68 → 作者偏离大众，或我们的样本不在模型分布内 |
| Distill-MOS | 0.66–0.67 | 待测 | 同上 |

判据：一个预测器「接近众人耳」的证据，是它在公开同参考配对上 acc0 ≥ 0.68（SCOREQ 水平）。把它当作者偏好的代理之前，要先过 exp007 的 L1 线（一致率下界 > 0.5）。**目前没有任何预测器在我们的数据上过线**（尚无判据，待测）。

**为什么不是 URGENT-PK**：它只能比「同一句话、不同系统」，正好对应我们的盲听格式，mel 版也只有 21.6 MB。但论文只报系统级，我们只有 3 个系统（KRCC 分辨率极粗），在真实录音（CHiME-7）上也不比 SCOREQ 好。列为第二批。

**为什么不训自己的**：Uni-VERSA-Ext 和 UrgentMOS 是目前最强的，但前者权重 > 500 MB，后者权重未查到；要用 urgent2024-sqa 自训，数据 1.34 GB 起步。不在这一轮。

---

## 5. 待拍板

1. 是否允许只下载 `urgent2024_mos` 的**单个分片**（442 MB < 500 MB，但数据集合计 1.34 GB），让作者在本机盲听其中的同参考配对，直接量「作者 vs 8 人众包」的一致率？这是唯一能把「作者偏好 ≠ 大众」与「机器不准」分开的现成材料。许可 CC BY-NC-SA 4.0（非商用，不需签署）。
2. NISQA 权重是 CC BY-NC-SA：只作内部评测、不随产品分发，是否可以？

## 6. 局限（未覆盖）

- 所有公开数字都来自 URGENT、DNS 这类「大模型 + 多种失真」的系统池，**没有一份专门针对「因果、低延迟小模型 + 会议场景」**。我们 3 个系统之间的差异可能比 URGENT 系统之间小得多，相关会更低（假设）。
- 全部预测器都是 16 kHz 输入（`resources/urgent2024_analysis/README.md:62-105`），48 kHz 的高频差异看不到（NISQA 除外，支持 48 kHz）。
- URGENT 2026 Track 2 参赛队的数字**未查到**。
