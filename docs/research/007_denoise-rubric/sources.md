# 来源登记 · 007 降噪 rubric

> 2026-10-30 · rubric。编号被 `must-know.md` 以 [N] 引用。「本次复核」= Workflow `wf_158ffb40-3f9` 里怀疑派是否打开原文核对过。
> 标准原文（BS.1534-3、P.800）由调研代理从 ITU 官网取得、只读核对，**未进仓**；`resources/` 下仓库为浅克隆，git 忽略。

## 外部（他人积累）


| # | 来源 | 类型 | 支撑什么 | 本次复核 |
|---|---|---|---|---|
| 1 | ITU-R BS.1534-3（MUSHRA） | 标准 | 锚点、事后剔除、片段长度、淡入淡出、重复刺激 | 是 |
| 2 | ITU-T P.800 附录 A/B/D/E/F | 标准 | 会话时长、空对、CCR 正反序、重复呈现、困难度 | 是 |
| 3 | ITU-T P.835、P.808 原文 | 标准 | 三维度评分、众包协议 | 否（可免费获取，未读） |
| 4 | arXiv 2005.08138（P.808 开源实现） | 论文 | 12 个刺激、陷阱题、接收条件、5 轮只有 8 份被拒 | 是 |
| 5 | arXiv 2010.13200（P.835 众包） | 论文 | 强制重听、维度顺序、约 10 段 | 是 |
| 6 | microsoft/P.808 `docs/conf_master.md`、`gold_clips.md`、`prep_p835.md` | 仓库 | 10+1+1 结构、金标 ±1、训练段 | 是（方差阈值查不到） |
| 7 | Naderi 等 2015 Interspeech | 论文 | 陷阱题设计 | 否 |
| 8 | Bradley & Terry 1952（Biometrika 39） | 论文 | 多候选排名 | 否 |
| 9 | `resources/URGENT-PK/urgentpk/rank.py:52-91`、`README.md:3-6,39` | 仓库 | 双向呈现消位置偏差（模型比较，不是人耳） | 是 |
| 10 | arXiv 2606.28114 | 论文 | 陷阱 + 金标持续筛选能改善众包 | 仅摘要；文中数字未核，不引 |
| 11 | DNS 2021 / 2022 本地 PDF（`resources/DNS-Challenge/docs/`） | 论文 | 延迟门槛、平局规则、取消资格 | 是 |
| 12 | arXiv 2202.13288（DNS 4） | 论文 | 总分公式、ANOVA、取消资格 | 是 |
| 13 | arXiv 2303.11510（DNS 5）+ MS 官网 | 论文/网页 | 公式、RTF、20 ms、INS 漏音 | 是（第三路未核） |
| 14 | arXiv 2309.12553（AEC 2023） | 论文 | 6 项等权、RTF ≤ 0.5 | 是（「单遍处理」查不到） |
| 15 | arXiv 2204.05222（PLC 2022） | 论文 | CMOS + WER 公式、约 30% 回答被拒 | 是 |
| 16 | URGENT 2024/2025 规则页、urgent2026/track1 | 网页 | 名次平均、两段式、不设延迟约束 | 是 |
| 17 | arXiv 2505.23212（URGENT 2025） | 论文 | T13 案例、主观占 1/5 | 是 |
| 18 | arXiv 2507.11306 | 论文 | #13 的 ESTOI/LPS 数值 | 是 |
| 19 | `resources/urgent2024_analysis/README.md:198-207`、`mos/mos_correlations.png`；arXiv 2506.01611 | 仓库/论文 | 各指标与 MOS 的 KRCC/LCC | 是（T17/T18 未核） |
| 20 | `resources/urgent2024_analysis/tagging/README.md` | 仓库 | 难样本加权投票阈值 | 是 |
| 21 | arXiv 2311.14490、2006.11140；CEC2 DOI 10.1109/ICASSP49357.2023.10094918 | 论文 | Clarity 指标组合、5 ms 因果 | 部分（CEC2 为二手） |
| 22 | CHiME-7 task2 results 页 | 网页 | 两段选人、三张榜 | 是 |
| 23 | arXiv 2203.11389（VoiceMOS） | 论文 | 系统级 SRCC | 是 |
| 24 | arXiv 2509.12974（AATC） | 论文 | DNSMOS ρ=−0.8、高 SNR 被乱修 | 是（WAcc ρ 未核） |
| 25 | arXiv 2402.01413（CHiME-7 UDASE） | 论文 | CMGAN 翻车、逐样本 PCC | 是 |
| 26 | arXiv 2601.18438（UrgentMOS） | 论文 | SCOREQ/UTMOS 逐句相关 | 表 10 是；表 5 否 |
| 27 | arXiv 2406.03460 | 论文 | PESQ 被优化、单点点击 | 是 |
| 28 | arXiv 2403.11732 | 论文 | 无参考损失导致幻觉 | 是 |
| 29 | arXiv 2201.06685 | 论文 | 非线性伪影伤害 ASR | 否 |
| 30 | arXiv 2501.11378；arXiv 2402.08021 | 论文 | Whisper 幻觉 40.3%、约 1% | 是 |
| 31 | arXiv 2303.06566（SSI 挑战） | 论文 | 单段与模型层面相关、DIS 0.310 | 是 |
| 32 | arXiv 2110.09625、2211.02944 | 论文 | TSOS 过度抑制 | 是 |
| 33 | arXiv 2105.13079（Torcoli）；NAIST IWAENC 2008；Gerrits 2013 | 论文 | 音乐噪声度量 | Torcoli 是；NAIST 系数查不到；Gerrits 否 |
| 34 | arXiv 2212.11851（StoRM）、2601.12254 | 论文 | 生成式伪影与幻觉 | 是（摘要） |
| 35 | arXiv 2309.07385 | 论文 | 在 P.804 上扩展出混响维度 | 是（摘要） |
| 36 | `resources/NISQA/README.md:6,55`；arXiv 2104.09494 | 仓库/论文 | 四个分项、跨域使用 | README 是；论文否 |
| 37 | `resources/DNS-Challenge/README.md:9`、`WAcc/WAcc.py` | 仓库 | P.835 + WAcc、WAcc 走远端服务 | 是 |
| 38 | 维基 VAD 条目；专利 US7649988 | 网页 | 削波分类、noise pumping | 是（专利读了约 3/4） |
| 39 | ETSI TS 103 281；ITU FG-CARCOM 稿 | 标准 | S-MOS、抽吸感 | 否 |
| 40 | `resources/urgent2025_challenge/evaluation_metrics/` | 仓库 | LPS、SpkSim 计算脚本 | 仅确认文件存在 |
| — | arXiv 2001.08662（「每小时复检环境」） | 论文 | — | 查不到，已排除 |

## 自家（自有积累）

| 来源 | 用来支撑什么 |
|---|---|
| `.42cog/intent.md`「真相源与权重」「三把尺子」 | 人耳与机器冲突时的裁决顺序；维度划分 |
| `docs/plan/001_decisions-20261029-b1.md` D01 · D04 · D10 · D28 | 延迟上限待定；GPL-3.0；门禁试跑值（删除率 +1 个百分点、N = 30）；Qwen3-ASR |
| `docs/research/002_model-baseline/decision.md` · `gap.md` | 候选的延迟三口径、许可、Rust 逐帧、论文分数 |
| `docs/research/004_eval-toolchain/decision.md` | 量具、统一前处理、回归门禁形状 |
| `docs/research/005_pse/decision.md` · `gap.md` | VoxLock 数据、IRL / TSOS 口径、门控失败线 |
| `docs/experiments/exp001-model-frame-time/README.md` | 每帧耗时实测（M5） |
| `docs/experiments/exp002-eval-smoke/README.md` | DNSMOS BAK 升而 SIG 降的小信号 |
| exp003（分支 `exp/003-offline-ab`）`make_clips.py` · `score.py` · `ab_page.html` · `build_page.py` | 片段编号、`scores.json` 字段、AB 页字段与陷阱题 |
| `vault/raw/20261029-voice-denoise-design.md` §3 · §9 · §13 | 痛点排序（吃字第一）、目标音色、评测飞轮、「AB 对比不打绝对分」 |
| 与 abtest 的会话间约定（2026-10-30，见 `state/handoff/rubric.md` 流水） | AB 页导出字段定稿 |
