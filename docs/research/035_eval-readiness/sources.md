# 035 · 来源链接

> 全部为公开合法全文。下载时请求头只用通用浏览器标识，不带个人信息。本卡没有使用 025 `sources.md` 以外的第三方转述。

## 论文与标准（已下载全文）

- URGENT 2024 复盘：Zhang 等，Lessons Learned from the URGENT 2024 Speech Enhancement Challenge，Interspeech 2025 — https://arxiv.org/abs/2506.01611
- URGENT 2025 多语种 P.808：Shi 等，P.808 Multilingual Speech Enhancement Testing — https://arxiv.org/abs/2507.11306
- URGENT 2025 挑战赛：Saijo 等 — https://arxiv.org/abs/2505.23212（PDF 已在 `resources/papers/training/`）
- CHiME-7 UDASE 客观与主观评测：Leglaive 等 — https://arxiv.org/abs/2402.01413
- The PESQetarian：de Oliveira 等，Interspeech 2024 — https://arxiv.org/abs/2406.03460
- DNSMOS：Reddy 等，ICASSP 2021 — https://arxiv.org/abs/2010.15258
- DNSMOS P.835：Reddy 等，ICASSP 2022 — https://arxiv.org/abs/2110.01763
- DNS 挑战赛 ICASSP 2022（DNS4）：Dubey 等 — https://arxiv.org/abs/2202.13288
- DNS 挑战赛 ICASSP 2023（DNS5）：Dubey 等 — https://arxiv.org/abs/2303.11510
- 众包 P.835：Naderi & Cutler，Subjective evaluation of noise suppression algorithms in crowdsourcing — https://arxiv.org/abs/2010.13200
- P.808 开源实现：Naderi & Cutler — https://arxiv.org/abs/2005.08138
- ITU-T P.808（06/2021）— https://www.itu.int/rec/T-REC-P.808-202106-I
- 泛化差距：Gonzalez 等，Assessing the Generalization Gap of Learning-Based Speech Enhancement Systems in Noisy and Reverberant Environments — https://arxiv.org/abs/2309.06183
- 增强伪影与识别：Iwamoto 等，How bad are artifacts? — https://arxiv.org/abs/2201.06685
- 基准方差：Bouthillier 等，Accounting for Variance in Machine Learning Benchmarks，MLSys 2021 — https://arxiv.org/abs/2103.03098
- 显著性检验：Dror 等，The Hitchhiker's Guide to Testing Statistical Significance in NLP，ACL 2018 — https://aclanthology.org/P18-1128/

## 无法获取全文

- ITU-T P.835（07/2026）— https://www.itu.int/rec/T-REC-P.835 （下载返回 500）

## 检索时看到、未下载、未引用

- URGENT 2024 复盘幻灯片 — https://urgent-challenge.github.io/urgent2024/assets/files/Slides_URGENT2024_Lessons.pdf
- SonicSim（合成数据对真实录音的迁移）— https://arxiv.org/abs/2410.01481
- MOS-Bench（质量预测模型的泛化）— https://arxiv.org/abs/2411.03715
- CHiME-3 单通道增强对真实录音识别 — https://arxiv.org/abs/2205.01751

## 本仓内部依据

- `specs/rubric/scorecard.md` · `docs/research/025_eval-best-practice/{decision,matrix}.md` · `notes/group-E.md`
- `docs/research/031_eval-judge-audit/audit.md`
- `docs/research/027_gtcrn-finetune/decision.md` 第三、四、五、六节
- `docs/plan/014_training-kickoff.md`
- `docs/experiments/exp013-scene-suite/README.md`
- `../noise_canceling-realmtg/docs/experiments/exp024-real-meeting-suite/README.md`（只读）
- `src/eval/src/main.rs:484-495` · `src/eval/src/scorecard/aggregate.rs:38-40`
- `state/pending-decisions.md:66-68`（F-1 至 F-3，真实录音题）
