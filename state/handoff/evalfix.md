# 现状页 · 手下 evalfix（#111 成绩单评分方法修正）

状态：进行中 · 第 2 组（修 bug）完成，下一步第 1 组（只汇总）
分支 `feat/scorecard-v2` · worktree `../noise_canceling-evalfix` · 管的实体：评测器 · 成绩单（规则与报告）· 定标等级

## 做到哪

#109 与 #110 已修，测试全绿；下一步做「从 clips.jsonl 重新汇总」入口，再按 D85 改汇总规则。

## 人定 D85（2026-11-04，替换计划里的「默认」）

G-1 B 均值 + 95% 区间三态（X1 X2 X4），最差 1–3 段送人耳 · G-2 B 人声起伏 / 音色 / 轻声字改在干净人声单独过一遍模型上算，照判，报告写明「只量到安静时的人声损伤」 · G-3 C 吃字单段多错 ≥ 5 字才否决，不做场景合并（E1 取消，E2 只整理正本）· G-4 A 门控项换成量输出（D1–D5）· G-5 4 dB · G-6 0.1 分 · G-7 咔嗒一直只报不判。代码与文档标「人定 D85」；`--rules v1|v2` 开关照留。

## 计划（依据：调研 025 `decision.md` 第 ①④ 节；G-1..G-7 按默认推进，待人确认）

所有改判定的地方做成一个开关 `--rules v1|v2`（默认 v2），旧规则原样保留，人改主意可切回；每处标「G-x 按默认，待人确认」。

### 第 1 组 · 只汇总（不重跑音频，不要名额，约 1 天）

| 项 | 改哪些文件 | 测试 |
|---|---|---|
| 新增「从 `clips.jsonl` 重新汇总」入口 `vdn-eval scorecard --from-clips <路径>` | `scorecard/mod.rs`（读回 `ClipRow`）· `main.rs` | 读回再写出逐字段一致 |
| X1 均值 + 按源片段成组的配对 bootstrap 95% 区间三态；物理项（V-NAN · V-LAT · V-MUTE · V-FRAME · V-SWITCH）仍逐段否决（G-1） | 新文件 `scorecard/aggregate.rs`（bootstrap · 三态）· `rules.rs` 调用 | 已知分布区间覆盖率 · 区间全在线内 / 外 / 跨线三例 · 物理项一段坏即否决 · 种子固定可复现 |
| X2 听感：每个评分模型 Δ 均值区间对 −δ 三态，δ = 0.1（G-6） | `rules.rs` 听感部分 | 两模型组合 4 种情形 |
| X4 Holm 校正，只在「不过」方向，每项一族（G-1） | `aggregate.rs` | 对照手算例 |
| X3 重估：哪些项在组合层多半落「待测」 | 只出报告，不改判定 | — |
| B4 真空感（V-VACUUM）进成绩单，只报不判，取 V-NR 现有数 | `standard.rs` · `card.rs` | 列存在且不进过不过 |
| E1 吃字合并字数 + 按句成组重采样；线 全集 1 pp · 场景×档 3 pp，标「暂定」（G-3） | `rules.rs` 吃字特判 · `thresholds/table.rs` | 合并公式对拍 jiwer 式累加 · 单段不再否决 |
| E2 吃字口径正本放 `thresholds/table.rs`，删 + 替 | `table.rs` · `qa/mod.rs` · `standard.rs` | 一致性测试 |
| E3 ESTOI 对拍容差 0.01 → 1e−4，补「整段零」样例 | `tests/estoi_parity.rs` · `verifiers.rs` 注册表说明 | 本身即测试 |
| G-2 人声起伏 / 音色 / 轻声字改「只报不判」；G-5 人声被压线 3 → 4 dB；G-7 咔嗒维持只报不判；D2a V-SIM-FA 撤下 | `standard.rs` · `table.rs` | 现有 `scorecard_standard` 扩充 |
| 产出：「新规则下的 exp014」对照表（旧 vs 新，每项 过 / 不过 / 待测 组合数） | `docs/experiments/exp014-scorecard-gtcrn/runs/<新 run-id>/` | — |

### 第 2 组 · 修 bug（不要名额，单测级，约半天）

- #109 延迟测量器：归一化互相关 + 窗放在人声最多的 4 s；先写红测试（合成 1.9 s 底噪 + 150 Hz 浊音、c07 两段回归、已知 300 样本）。文件 `analysis.rs` · `verifiers/physical.rs` · 注册表删「5 种子 = Teams」
- #110 V-WEAK 低采样率：频带上沿截到奈奎斯特，剩不到一半判「不知道」；V-TIMBRE 同理（剩 < 3 带判不知道）。8 / 12 / 16 kHz 测试，16 kHz 读数逐位不变。文件 `verifiers/speech.rs` · `verifiers/perceptual.rs`

### 第 3 组 · 重新打分 / 新运行（要名额）

- 只加旁报、判定不变：A2–A4 · B1 · B2 · B3（扣输入侧偏差，验收原声直通 ≈ 0 dB）· C1 · F1（`#[ignore]` 对拍 + 原声分按内容哈希缓存）· F2（注册表说明）
- 新算法：C2 干净人声单独过一遍模型（只报）· D1 TSOS「用户被误删」+ 原声直通噪声底 · D2b 漏过事件率 · D3 开口后被吞多久（只报）· D4 门控安静项降诊断 · D5 重叠段 SI-SDRi + 失败率
- 文件：`verifiers/sim.rs`（D 组，可能拆新文件 `verifiers/tsos.rs`）· `perceptual.rs` · `speech.rs` · `runner.rs`（C2 多一次处理）
- 名额：先在 1 个场景冒烟（约 10 分钟），报耗时；全量见第 4 组

### 第 4 组 · 重出成绩单 + 自检（要名额）

- passthrough · ref · GTCRN 全 480 段 `--qa`，新 run id：估 2.5 h（025 原估）+ C2 多一次处理约 1/3 → 约 3.3 h，冒烟后再报准数
- 自检：`xtask tidy` · `vdn-eval ci` · 全量 `cargo test`（约 10–20 分钟，申请名额）
- 收尾：codex-review 两轮无 P0/P1 即停；`findings.md` 末尾追加「新规则下的结论」

## 流水

- 2026-11-04 读完共同规矩、交代、025 decision；看了 `scorecard/` 代码与 exp014 run（`clips.jsonl` 1440 行已在仓）；出计划

## 待人拍板（G 题之外新冒出的）

（暂无）

## 转交

（暂无）
- 2026-11-04 大总管回「计划可以」：先做第 2 组 + 第 1 组不随 G 题部分；随 G 题的等消息
- 2026-11-04 写 #109 红测试（`analysis.rs` 迟到浊音样例）· #110 红测试（新文件 `verifiers/low_rate_tests.rs`）；申请首次编译名额，等回复
- 2026-11-04 回 cands：外部输出入口形状可以；要求 bg/ 保留、按输入哈希的索引做成通用（以后 clean/ 给 C2）、元数据记目录与 run.json 哈希
- 2026-11-04 名额：首次编译 vdn-eval，5 条红测试按预期红（迟到浊音量出 2014 样本；V-WEAK 8 k / 2 k 切片越界 panic；V-TIMBRE 报越界带），已归还
- 2026-11-04 #109 修：`analysis.rs` 归一化互相关 + `busiest_start` 选主讲人能量最多的 4 s 窗；`physical.rs` 调用改；注册表删「5 种子 = Teams」。测试：迟到浊音原声直通 = 0 · 已知 300 样本 · 全零 → None · `tests/lat_c07.rs`（ignore，要 _tmp 音频）c07 两段 = 0，已人工跑过
- 2026-11-04 #110 修：V-WEAK 不计上沿越过奈奎斯特的带、少于一半判不知道、截掉带时旁报 bands；V-TIMBRE 同理、少于 3 带判不知道。`spectrum_golden` 对拍通过（16 kHz 全部 verifier 读数逐位不变）· lib 67 条 · clippy 无警告
- 2026-11-04 cands 入口审完：同意形状；公平性 3 处（延迟只是声明值 · 回放无计时判不知道 · trunc/ 截断查偷看未来），大总管同意
