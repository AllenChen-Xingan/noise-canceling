# 031 盘点：评估器 · 人耳数据 · 怎么校准过

> 手下 evalaudit，2026-11-13，只读盘点。来源标注：〔源码〕读代码 ·〔文档〕读仓内文档 ·〔实测〕本线跑出的数。

## 一、评估器清单（判什么 · 输出 · 有没有拿人耳校准）

下表列出每一类评估器，供第二节、第三节对照。成绩单里每项的线与判法，正本在 `src/eval/src/thresholds/table.rs` 和 `src/eval/src/scorecard/rules_v2.rs`，人读版在 `specs/rubric/scorecard.md:46-71`。

| 评估器 | 判什么 | 单段输出 | 成绩单里怎么用 | 人耳校准 |
|---|---|---|---|---|
| 物理硬事实 V-NAN · V-LAT · V-MUTE · V-FRAME · V-SWITCH | 坏值、延迟、静音、切换 | 数值 → 过 / 不过 | 逐段，任一段不过即否决 | 不需要。按 012 记 L3（5 次重复区间够窄，`012 verifiers.md:35`）〔文档〕 |
| DSP 感知项 V-ONSET · V-WEAK · V-PUMPV · V-TIMBRE · V-MUSIC · V-NR · V-INTF · V-BGSTAB · V-ESTOI · V-SPLVL | 字头被削、轻声字、忽大忽小、音色、乐音噪声、压噪、压旁人、背景跳变、可懂度、人声被压低 | 数值 → 过 / 不过（线在 `table.rs`） | 组合均值 + 95% 区间三态，「不过」过 Holm 校正（`scorecard.md:84-87`） | **从没拿人耳定过线**。只有人造劣化的探针（`src/eval/src/probes.rs:1-5`）证明「明显坏的会被抓到」。exp003 定标只覆盖了 V-NR、ESTOI〔源码 `calib.rs:35-83`〕 |
| 听感评分模型 SCOREQ（主）· Distill-MOS（交叉） | 听感分 Δ = 输出 − 带噪原声 | 连续分 → 过 / 不过（v1 单段 Δ ≥ 0；v2 组合均值区间对 Δ ≥ −0.1） | 两个都过才过；分歧送人耳（`scorecard.md:88`） | **从没进过 `vdn-eval calib`**。exp009 在同一批 48 对上算过方向一致率：SCOREQ 0.375，Distill-MOS 0.562（`docs/experiments/exp009-more-ears/results/author_vs_crowd.json`）〔文档〕 |
| 吃字 V-EAT-ASR | 语音识别（qwen3-asr）删 + 替比原声多几个 | 数值 → 过 / 不过（v1：多 > 1 个百分点；v2：多 ≥ 5 个字） | 逐段否决（`scorecard.md:83`） | 方向一致率 0.64 [0.41, 0.84]（exp007 定标第 1 轮）。012 要求「与人耳吃字比假报率与漏报率」（`012 verifiers.md:80`），**没做过** |
| 只报不判 V-CLICK · V-VACUUM · V-TSOS · V-SWALLOW · V-LEAK · V-PSE-OVL · V-SIM-GATE · V-SIM-FA | 见 `scorecard.md` 第三节 | 数值 | 不进判定 | 无 |
| 定标器 `vdn-eval calib`（`src/eval/src/calib.rs`） | 评估器和人耳排序同不同向 | 每个指标：成对方向一致率 + 按屏 bootstrap 区间 + 片段内 Kendall τ + 级别 L0–L3 | 决定感知项能不能升级 | —— |

## 二、人耳数据（只有一名听者）

下面列出仓里全部人耳答卷，看哪些能和评估器分数配成对。盘点由本线子代理做，主表核对过。

| 数据 | 量 | 能否和评估器配对 |
|---|---|---|
| exp003 build f8c3（`docs/experiments/exp003-offline-ab/results/20261030-human-build-20261030110755-f8c3.json`） | 12 屏正式题 × 3 个模型 = 36 条；每条有「开会能用吗」ok / marginal / no、「吃字」勾选、缺陷标签；每屏「最清楚」「最自然」。会话打折：全同陷阱没过 | **能，唯一一份**。SCOREQ / Distill-MOS 在 `exp009-more-ears/results/exp003_predictors.json`（对作者实际听到的那份音频打的分）；识别删 / 替在 `exp003-offline-ab/scores.json` |
| exp003 build 68ab | 能揭盲的只有 2 屏 | 太少 |
| exp005 build ab48 | 8 屏，三选一 + 吃字 | 没有听感分；人听 25 秒节选、机器跑整段，对不齐 |
| exp009 锚点题 | 2 题 | 太少 |
| exp016 GTCRN 对 DFN3 | 4 题，3 题答「分不出」 | 太少；DFN3 那份输出是不是同一份没核过 |

- 听者：全部是作者一人（`specs/rubric/preferences.md:9`）
- 没有重测（同一题隔天再听）（`preferences.md` U8）
- 整理好的数据集 `docs/experiments/human-prefs/pairs.jsonl` 不带评估器分数

## 三、过去怎么量「和人耳一致」

量法只有一种：成对方向一致率（`calib.rs:2-9`）。下面列出它的几个关键选择，供 `audit.md` 逐条对照。

| 问题 | 现在的做法 | 出处 |
|---|---|---|
| 指标 | 成对方向一致率 + 片段内 Kendall τ；**没有**真阳性率 / 真阴性率、混淆矩阵、κ | `calib.rs:202-279`；全仓 grep 无命中〔实测〕 |
| 判的是什么 | 「A 比 B 好」的方向。**不判**成绩单里真正用的过关线判得准不准 | `012 verifiers.md:31-34` |
| 样本 | 48 对，去重后 20 对，12 屏，1 名听者 | `exp007-eval-suite/calib/20261031-exp003-round1.md`；去重数〔实测〕 |
| 区间 | 按屏聚类 bootstrap 10000 次，种子固定 | `calib.rs:229-240` |
| 数据切分 | 奇数轮进定标集、偶数轮进验证集；判级只看验证集。过关线不是拿人耳数据调出来的 | `calib.rs:7`、`calib.rs:191` |
| 结果 | 只跑过第 1 轮（定标轮），全部 L0；验证集为空 | 同上 |

## 四、本线新算的（exp022）

真阳性率 / 真阴性率是第一次算。脚本、测试、结果在 `docs/experiments/exp022-judge-tpr-tnr/`，结论写在 `audit.md` 第 1、2 条。
